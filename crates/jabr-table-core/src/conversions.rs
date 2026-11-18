use crate::{
    CellValue, Column, ColumnType, DataError, JabrMatrix, JabrTable, Row, validate_matrix,
    validate_table,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatrixToTableOptions {
    /// When true, matrix column labels become column titles.
    pub use_column_labels: bool,
}

impl Default for MatrixToTableOptions {
    fn default() -> Self {
        Self {
            use_column_labels: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableToMatrixOptions {
    /// Optional ordered subset of table column indices. None selects every column.
    pub columns: Option<Vec<usize>>,
    /// When true, a text header row is prepended to matrix values.
    pub include_headers: bool,
}

impl Default for TableToMatrixOptions {
    fn default() -> Self {
        Self {
            columns: None,
            include_headers: false,
        }
    }
}

pub fn matrix_to_table(
    matrix: &JabrMatrix,
    options: MatrixToTableOptions,
) -> Result<JabrTable, DataError> {
    validate_matrix(matrix)?;

    let columns = (0..matrix.columns)
        .map(|column_index| {
            let title = if options.use_column_labels && !matrix.column_labels.is_empty() {
                matrix.column_labels[column_index].clone()
            } else {
                format!("Column {}", column_index + 1)
            };
            Column {
                id: format!("{}:column:{}", matrix.id, column_index),
                title,
                value_type: infer_column_type(matrix, column_index),
                unit_hint: None,
            }
        })
        .collect();

    let rows = (0..matrix.rows)
        .map(|row_index| Row {
            id: format!("{}:row:{}", matrix.id, row_index),
            label: if matrix.row_labels.is_empty() {
                None
            } else {
                Some(matrix.row_labels[row_index].clone())
            },
            
            cells: (0..matrix.columns)
                .map(|column_index| {
                    matrix.values[row_index * matrix.columns + column_index].clone()
                })
                .collect(),
        })
        .collect();

    let table = JabrTable {
        schema_version: 1,
        id: format!("{}:table", matrix.id),
        columns,
        rows,
    };
    validate_table(&table)?;
    Ok(table)
}

pub fn table_to_matrix(
    table: &JabrTable,
    options: &TableToMatrixOptions,
) -> Result<JabrMatrix, DataError> {
    validate_table(table)?;
    let selected: Vec<usize> = options
        .columns
        .clone()
        .unwrap_or_else(|| (0..table.columns.len()).collect());

    for &index in &selected {
        if index >= table.columns.len() {
            return Err(DataError::ColumnIndex {
                index,
                columns: table.columns.len(),
            });
        }
    }

    let mut values = Vec::with_capacity(
        (table.rows.len() + (if options.include_headers { 1 } else { 0 })) * selected.len(),
    );
    if options.include_headers {
        values.extend(
            selected
                .iter()
                .map(|&index| CellValue::Text(table.columns[index].title.clone())),
        );
    }
    for row in &table.rows {
        values.extend(selected.iter().map(|&index| row.cells[index].clone()));
    }

    let matrix_rows = table.rows.len() + (if options.include_headers { 1 } else { 0 });
    let column_labels = selected
        .iter()
        .map(|&index| table.columns[index].title.clone())
        .collect();
    let row_labels = if table.rows.iter().any(|row| row.label.is_some()) {
        table
            .rows
            .iter()
            .map(|row| row.label.clone().unwrap_or_default())
            .collect()
    } else {
        vec![]
    };
    let matrix = JabrMatrix {
        schema_version: 1,
        id: format!("{}:matrix", table.id),
        rows: matrix_rows,
        columns: selected.len(),
        values,
        row_labels,
        column_labels,
    };
    validate_matrix(&matrix)?;
    Ok(matrix)
}

fn infer_column_type(matrix: &JabrMatrix, column: usize) -> ColumnType {
    let mut found = None;
    for row in 0..matrix.rows {
        let value = &matrix.values[row * matrix.columns + column];
        let candidate = match value {
            CellValue::Empty => continue,
            CellValue::Boolean(_) => ColumnType::Boolean,
            CellValue::Integer(_) => ColumnType::Integer,
            CellValue::Decimal(_) => ColumnType::Decimal,
            CellValue::Text(_)
            | CellValue::DateTime(_)
            | CellValue::Error(_)
            | CellValue::List(_) => ColumnType::Any,
        };
        if let Some(previous) = found {
            if previous != candidate {
                return ColumnType::Any;
            }
        } else {
            found = Some(candidate);
        }
    }
    found.unwrap_or(ColumnType::Any)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_matrix() -> JabrMatrix {
        JabrMatrix {
            schema_version: 1,
            id: "m".into(),
            rows: 2,
            columns: 2,
            values: vec![
                CellValue::Integer(10),
                CellValue::Decimal("2.50".into()),
                CellValue::Integer(20),
                CellValue::Decimal("3.75".into()),
            ],
            row_labels: vec!["Dead".into(), "Live".into()],
            column_labels: vec!["Load".into(), "Factor".into()],
        }
    }

    #[test]
    fn matrix_to_table_preserves_shape_values_and_labels() {
        let table = matrix_to_table(&sample_matrix(), MatrixToTableOptions::default()).unwrap();
        assert_eq!(table.columns.len(), 2);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.columns[0].title, "Load");
        assert_eq!(table.rows[0].label.as_deref(), Some("Dead"));
        assert_eq!(table.rows[1].cells[1], CellValue::Decimal("3.75".into()));
    }

    #[test]
    fn table_to_matrix_round_trips_without_header_by_default() {
        let table = matrix_to_table(&sample_matrix(), MatrixToTableOptions::default()).unwrap();
        let matrix = table_to_matrix(&table, &TableToMatrixOptions::default()).unwrap();
        assert_eq!(matrix.rows, 2);
        assert_eq!(matrix.columns, 2);
        assert_eq!(matrix.values, sample_matrix().values);
        assert_eq!(matrix.row_labels, sample_matrix().row_labels);
    }

    #[test]
    fn table_to_matrix_can_include_header_row() {
        let table = matrix_to_table(&sample_matrix(), MatrixToTableOptions::default()).unwrap();
        let matrix = table_to_matrix(
            &table,
            &TableToMatrixOptions {
                columns: None,
                include_headers: true,
            },
        )
        .unwrap();
        assert_eq!(matrix.rows, 3);
        assert_eq!(matrix.values[0], CellValue::Text("Load".into()));
    }
}
