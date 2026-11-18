use std::collections::HashSet;

use crate::{CellValue, DataError, JabrMatrix, JabrTable, JabrVector};

pub fn validate_matrix(matrix: &JabrMatrix) -> Result<(), DataError> {
    let expected =
        matrix
            .rows
            .checked_mul(matrix.columns)
            .ok_or(DataError::MatrixSizeOverflow {
                rows: matrix.rows,
                columns: matrix.columns,
            })?;
    if matrix.values.len() != expected {
        return Err(DataError::MatrixShape {
            rows: matrix.rows,
            columns: matrix.columns,
            actual: matrix.values.len(),
        });
    }
    if !matrix.row_labels.is_empty() && matrix.row_labels.len() != matrix.rows {
        return Err(DataError::MatrixRowLabels {
            labels: matrix.row_labels.len(),
            rows: matrix.rows,
        });
    }
    if !matrix.column_labels.is_empty() && matrix.column_labels.len() != matrix.columns {
        return Err(DataError::MatrixColumnLabels {
            labels: matrix.column_labels.len(),
            columns: matrix.columns,
        });
    }
    Ok(())
}

pub fn validate_vector(_vector: &JabrVector) -> Result<(), DataError> {
    Ok(())
}

pub fn validate_table(table: &JabrTable) -> Result<(), DataError> {
    let mut column_ids = HashSet::new();
    for column in &table.columns {
        if column.id.trim().is_empty() || !column_ids.insert(&column.id) {
            return Err(DataError::InvalidId {
                kind: "column",
                id: column.id.clone(),
            });
        }
    }
    let mut row_ids = HashSet::new();
    for row in &table.rows {
        if row.id.trim().is_empty() || !row_ids.insert(&row.id) {
            return Err(DataError::InvalidId {
                kind: "row",
                id: row.id.clone(),
            });
        }
        if row.cells.len() != table.columns.len() {
            return Err(DataError::RowWidth {
                row_id: row.id.clone(),
                actual: row.cells.len(),
                expected: table.columns.len(),
            });
        }
        for cell in &row.cells {
            validate_cell_depth(cell, 0, 32)?;
        }
    }
    Ok(())
}

fn validate_cell_depth(cell: &CellValue, depth: usize, max: usize) -> Result<(), DataError> {
    if depth > max {
        return Err(DataError::ListDepth { max });
    }
    if let CellValue::List(values) = cell {
        for value in values {
            validate_cell_depth(value, depth + 1, max)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Column, ColumnType, JabrTable, Row};

    #[test]
    fn rejects_matrix_dimensions_that_overflow() {
        let matrix = JabrMatrix {
            schema_version: 1,
            id: "overflow".into(),
            rows: usize::MAX,
            columns: 2,
            values: vec![],
            row_labels: vec![],
            column_labels: vec![],
        };
        assert!(matches!(
            validate_matrix(&matrix),
            Err(DataError::MatrixSizeOverflow { .. })
        ));
    }

    #[test]
    fn rejects_matrix_with_inconsistent_shape() {
        let matrix = JabrMatrix {
            schema_version: 1,
            id: "m1".into(),
            rows: 2,
            columns: 2,
            values: vec![CellValue::Integer(1), CellValue::Integer(2)],
            row_labels: vec![],
            column_labels: vec![],
        };
        assert!(matches!(
            validate_matrix(&matrix),
            Err(DataError::MatrixShape { .. })
        ));
    }

    #[test]
    fn accepts_empty_zero_by_zero_matrix() {
        let matrix = JabrMatrix {
            schema_version: 1,
            id: "m0".into(),
            rows: 0,
            columns: 0,
            values: vec![],
            row_labels: vec![],
            column_labels: vec![],
        };
        assert_eq!(validate_matrix(&matrix), Ok(()));
    }

    #[test]
    fn rejects_table_rows_with_wrong_width() {
        let table = JabrTable {
            schema_version: 1,
            id: "t1".into(),
            columns: vec![Column {
                id: "c1".into(),
                title: "A".into(),
                value_type: ColumnType::Integer,
                unit_hint: None,
            }],
            rows: vec![Row {
                id: "r1".into(),
                label: None,
                cells: vec![],
            }],
        };
        assert!(matches!(
            validate_table(&table),
            Err(DataError::RowWidth { .. })
        ));
    }
}
