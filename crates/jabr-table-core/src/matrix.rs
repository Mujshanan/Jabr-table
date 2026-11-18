//! Small dense matrix and vector primitives for element-level FEM calculations.
//! Global sparse assembly/solving is intentionally outside this crate stage.

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MatrixError {
    #[error("matrix dimensions must be non-zero")]
    EmptyDimensions,
    #[error("matrix data length {actual} does not match {rows}x{columns}")]
    DataLength { rows: usize, columns: usize, actual: usize },
    #[error("matrix contains a non-finite value at flat index {0}")]
    NonFiniteValue(usize),
    #[error("dimension mismatch for {operation}: left is {left_rows}x{left_columns}, right is {right_rows}x{right_columns}")]
    DimensionMismatch {
        operation: &'static str,
        left_rows: usize,
        left_columns: usize,
        right_rows: usize,
        right_columns: usize,
    },
    #[error("vector length {actual} does not match matrix column count {expected}")]
    VectorLengthMismatch { expected: usize, actual: usize },
}

/// Row-major dense matrix with validated finite f64 entries.
#[derive(Debug, Clone, PartialEq)]
pub struct DenseMatrix {
    rows: usize,
    columns: usize,
    data: Vec<f64>,
}

impl DenseMatrix {
    pub fn new(rows: usize, columns: usize, data: Vec<f64>) -> Result<Self, MatrixError> {
        if rows == 0 || columns == 0 { return Err(MatrixError::EmptyDimensions); }
        let expected = rows.checked_mul(columns).ok_or(MatrixError::DataLength {
            rows, columns, actual: data.len(),
        })?;
        if data.len() != expected {
            return Err(MatrixError::DataLength { rows, columns, actual: data.len() });
        }
        if let Some(index) = data.iter().position(|value| !value.is_finite()) {
            return Err(MatrixError::NonFiniteValue(index));
        }
        Ok(Self { rows, columns, data })
    }

    pub fn from_rows(rows: Vec<Vec<f64>>) -> Result<Self, MatrixError> {
        let Some(first) = rows.first() else { return Err(MatrixError::EmptyDimensions); };
        let columns = first.len();
        if columns == 0 { return Err(MatrixError::EmptyDimensions); }
        if rows.iter().any(|row| row.len() != columns) {
            return Err(MatrixError::DataLength {
                rows: rows.len(), columns,
                actual: rows.iter().map(Vec::len).sum(),
            });
        }
        Self::new(rows.len(), columns, rows.into_iter().flatten().collect())
    }

    pub fn rows(&self) -> usize { self.rows }
    pub fn columns(&self) -> usize { self.columns }
    pub fn as_slice(&self) -> &[f64] { &self.data }

    pub fn get(&self, row: usize, column: usize) -> Option<f64> {
        if row >= self.rows || column >= self.columns { None }
        else { Some(self.data[row * self.columns + column]) }
    }

    pub fn add(&self, other: &Self) -> Result<Self, MatrixError> {
        self.zip_map(other, "addition", |a, b| a + b)
    }

    pub fn subtract(&self, other: &Self) -> Result<Self, MatrixError> {
        self.zip_map(other, "subtraction", |a, b| a - b)
    }

    pub fn multiply(&self, other: &Self) -> Result<Self, MatrixError> {
        if self.columns != other.rows {
            return Err(MatrixError::DimensionMismatch {
                operation: "multiplication",
                left_rows: self.rows, left_columns: self.columns,
                right_rows: other.rows, right_columns: other.columns,
            });
        }
        let mut result = vec![0.0; self.rows * other.columns];
        for row in 0..self.rows {
            let output_row = &mut result[row * other.columns..(row + 1) * other.columns];
            for inner in 0..self.columns {
                let left = self.data[row * self.columns + inner];
                let right_row = &other.data[inner * other.columns..(inner + 1) * other.columns];
                for (output, right) in output_row.iter_mut().zip(right_row) {
                    *output += left * right;
                }
            }
        }
        Self::new(self.rows, other.columns, result)
    }

    pub fn transpose(&self) -> Self {
        let mut result = vec![0.0; self.data.len()];
        for row in 0..self.rows {
            for column in 0..self.columns {
                result[column * self.rows + row] = self.data[row * self.columns + column];
            }
        }
        Self { rows: self.columns, columns: self.rows, data: result }
    }

    pub fn multiply_vector(&self, vector: &[f64]) -> Result<Vec<f64>, MatrixError> {
        if vector.len() != self.columns {
            return Err(MatrixError::VectorLengthMismatch {
                expected: self.columns, actual: vector.len(),
            });
        }
        if let Some(index) = vector.iter().position(|value| !value.is_finite()) {
            return Err(MatrixError::NonFiniteValue(index));
        }
        let mut result = vec![0.0; self.rows];
        for (row, output) in result.iter_mut().enumerate() {
            *output = (0..self.columns)
                .map(|column| self.data[row * self.columns + column] * vector[column])
                .sum();
        }
        if let Some(index) = result.iter().position(|value| !value.is_finite()) {
            return Err(MatrixError::NonFiniteValue(index));
        }
        Ok(result)
    }

    fn zip_map(&self, other: &Self, operation: &'static str, f: impl Fn(f64, f64) -> f64)
        -> Result<Self, MatrixError>
    {
        if self.rows != other.rows || self.columns != other.columns {
            return Err(MatrixError::DimensionMismatch {
                operation, left_rows: self.rows, left_columns: self.columns,
                right_rows: other.rows, right_columns: other.columns,
            });
        }
        Self::new(self.rows, self.columns,
            self.data.iter().zip(&other.data).map(|(a, b)| f(*a, *b)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition_and_subtraction_are_elementwise() {
        let a = DenseMatrix::from_rows(vec![vec![1.0, 2.0], vec![3.0, 4.0]]).unwrap();
        let b = DenseMatrix::from_rows(vec![vec![4.0, 3.0], vec![2.0, 1.0]]).unwrap();
        assert_eq!(a.add(&b).unwrap().as_slice(), &[5.0, 5.0, 5.0, 5.0]);
        assert_eq!(a.subtract(&b).unwrap().as_slice(), &[-3.0, -1.0, 1.0, 3.0]);
    }

    #[test]
    fn multiplication_uses_row_major_dimensions() {
        let a = DenseMatrix::from_rows(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]).unwrap();
        let b = DenseMatrix::from_rows(vec![vec![7.0, 8.0], vec![9.0, 10.0], vec![11.0, 12.0]]).unwrap();
        assert_eq!(a.multiply(&b).unwrap().as_slice(), &[58.0, 64.0, 139.0, 154.0]);
    }

    #[test]
    fn transpose_swaps_dimensions_and_entries() {
        let a = DenseMatrix::from_rows(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]).unwrap();
        let t = a.transpose();
        assert_eq!((t.rows(), t.columns()), (3, 2));
        assert_eq!(t.as_slice(), &[1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
    }

    #[test]
    fn matrix_vector_multiplication() {
        let a = DenseMatrix::from_rows(vec![vec![2.0, 1.0], vec![1.0, 3.0]]).unwrap();
        assert_eq!(a.multiply_vector(&[4.0, 5.0]).unwrap(), vec![13.0, 19.0]);
    }

    #[test]
    fn rejects_invalid_shapes_and_non_finite_values() {
        assert_eq!(DenseMatrix::new(2, 2, vec![1.0]), Err(MatrixError::DataLength {
            rows: 2, columns: 2, actual: 1,
        }));
        assert_eq!(DenseMatrix::new(1, 1, vec![f64::NAN]), Err(MatrixError::NonFiniteValue(0)));
    }

    #[test]
    fn rejects_dimension_mismatches() {
        let a = DenseMatrix::new(2, 2, vec![1.0; 4]).unwrap();
        let b = DenseMatrix::new(3, 1, vec![1.0; 3]).unwrap();
        assert!(matches!(a.multiply(&b), Err(MatrixError::DimensionMismatch { .. })));
        assert!(matches!(a.multiply_vector(&[1.0]), Err(MatrixError::VectorLengthMismatch { .. })));
    }
}
