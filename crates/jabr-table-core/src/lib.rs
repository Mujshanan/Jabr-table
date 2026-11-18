//! Minimal, UI-independent table data model for Jabr.

pub mod functions;
pub mod interchange;
pub mod matrix;

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Stable identifier supplied by the owning document/integration layer.
pub type Id = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum CellValue {
    Empty,
    Text(String),
    Number(f64),
    Boolean(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColumnType {
    Any,
    Text,
    Number,
    Boolean,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Column {
    pub id: Id,
    pub name: String,
    pub value_type: ColumnType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Row {
    pub id: Id,
    /// Values are keyed by stable column ID, never by display position.
    pub cells: BTreeMap<Id, CellValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub id: Id,
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
}

#[derive(Debug, Error, PartialEq)]
pub enum TableError {
    #[error("table ID must not be empty")]
    EmptyTableId,
    #[error("column ID must not be empty")]
    EmptyColumnId,
    #[error("row ID must not be empty")]
    EmptyRowId,
    #[error("duplicate column ID: {0}")]
    DuplicateColumnId(Id),
    #[error("duplicate row ID: {0}")]
    DuplicateRowId(Id),
    #[error("unknown column ID: {0}")]
    UnknownColumn(Id),
    #[error("unknown row ID: {0}")]
    UnknownRow(Id),
    #[error("value type does not match column {0}")]
    TypeMismatch(Id),
}

impl Table {
    pub fn new(id: impl Into<Id>, columns: Vec<Column>) -> Result<Self, TableError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(TableError::EmptyTableId);
        }

        let mut seen = BTreeSet::new();
        for column in &columns {
            if column.id.trim().is_empty() {
                return Err(TableError::EmptyColumnId);
            }
            if !seen.insert(column.id.clone()) {
                return Err(TableError::DuplicateColumnId(column.id.clone()));
            }
        }

        Ok(Self {
            id,
            columns,
            rows: Vec::new(),
        })
    }

    pub fn add_column(&mut self, column: Column) -> Result<(), TableError> {
        if column.id.trim().is_empty() {
            return Err(TableError::EmptyColumnId);
        }
        if self.columns.iter().any(|existing| existing.id == column.id) {
            return Err(TableError::DuplicateColumnId(column.id));
        }
        self.columns.push(column);
        Ok(())
    }

    pub fn add_row(&mut self, id: impl Into<Id>) -> Result<(), TableError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(TableError::EmptyRowId);
        }
        if self.rows.iter().any(|row| row.id == id) {
            return Err(TableError::DuplicateRowId(id));
        }

        let cells = self
            .columns
            .iter()
            .map(|column| (column.id.clone(), CellValue::Empty))
            .collect();
        self.rows.push(Row { id, cells });
        Ok(())
    }

    pub fn set_cell(
        &mut self,
        row_id: &str,
        column_id: &str,
        value: CellValue,
    ) -> Result<(), TableError> {
        let column = self
            .columns
            .iter()
            .find(|column| column.id == column_id)
            .ok_or_else(|| TableError::UnknownColumn(column_id.to_owned()))?;
        let row = self
            .rows
            .iter_mut()
            .find(|row| row.id == row_id)
            .ok_or_else(|| TableError::UnknownRow(row_id.to_owned()))?;

        let matches_type = matches!(
            (&column.value_type, &value),
            (_, CellValue::Empty)
                | (ColumnType::Any, _)
                | (ColumnType::Text, CellValue::Text(_))
                | (ColumnType::Number, CellValue::Number(_))
                | (ColumnType::Boolean, CellValue::Boolean(_))
        );

        if !matches_type {
            return Err(TableError::TypeMismatch(column_id.to_owned()));
        }

        row.cells.insert(column_id.to_owned(), value);
        Ok(())
    }

    pub fn cell(&self, row_id: &str, column_id: &str) -> Result<&CellValue, TableError> {
        self.columns
            .iter()
            .find(|column| column.id == column_id)
            .ok_or_else(|| TableError::UnknownColumn(column_id.to_owned()))?;
        let row = self
            .rows
            .iter()
            .find(|row| row.id == row_id)
            .ok_or_else(|| TableError::UnknownRow(row_id.to_owned()))?;
        Ok(row.cells.get(column_id).unwrap_or(&EMPTY_CELL))
    }

    pub fn remove_row(&mut self, row_id: &str) -> Result<Row, TableError> {
        let index = self
            .rows
            .iter()
            .position(|row| row.id == row_id)
            .ok_or_else(|| TableError::UnknownRow(row_id.to_owned()))?;
        Ok(self.rows.remove(index))
    }

    pub fn remove_column(&mut self, column_id: &str) -> Result<Column, TableError> {
        let index = self
            .columns
            .iter()
            .position(|column| column.id == column_id)
            .ok_or_else(|| TableError::UnknownColumn(column_id.to_owned()))?;
        for row in &mut self.rows {
            row.cells.remove(column_id);
        }
        Ok(self.columns.remove(index))
    }
}

static EMPTY_CELL: CellValue = CellValue::Empty;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Table {
        Table::new(
            "sections",
            vec![
                Column {
                    id: "mark".into(),
                    name: "Mark".into(),
                    value_type: ColumnType::Text,
                },
                Column {
                    id: "area".into(),
                    name: "Area".into(),
                    value_type: ColumnType::Number,
                },
            ],
        )
        .unwrap()
    }

    #[test]
    fn creates_rows_and_updates_typed_cells() {
        let mut table = sample();
        table.add_row("r1").unwrap();
        table
            .set_cell("r1", "mark", CellValue::Text("UB 305".into()))
            .unwrap();
        table
            .set_cell("r1", "area", CellValue::Number(51.3))
            .unwrap();

        assert_eq!(
            table.cell("r1", "mark").unwrap(),
            &CellValue::Text("UB 305".into())
        );
        assert_eq!(table.cell("r1", "area").unwrap(), &CellValue::Number(51.3));
    }

    #[test]
    fn rejects_wrong_cell_type() {
        let mut table = sample();
        table.add_row("r1").unwrap();

        assert_eq!(
            table.set_cell("r1", "area", CellValue::Text("51.3".into())),
            Err(TableError::TypeMismatch("area".into()))
        );
    }

    #[test]
    fn stable_ids_survive_column_removal() {
        let mut table = sample();
        table.add_row("row-stable").unwrap();
        table
            .set_cell("row-stable", "area", CellValue::Number(10.0))
            .unwrap();
        table.remove_column("mark").unwrap();

        assert_eq!(
            table.cell("row-stable", "area").unwrap(),
            &CellValue::Number(10.0)
        );
    }

    #[test]
    fn rejects_duplicate_ids() {
        let mut table = sample();
        assert_eq!(table.add_row("r1"), Ok(()));
        assert_eq!(
            table.add_row("r1"),
            Err(TableError::DuplicateRowId("r1".into()))
        );
    }
}
