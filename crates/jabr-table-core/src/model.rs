use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum CellValue {
    Empty,
    Boolean(bool),
    Integer(i64),
    /// Canonical base-10 text; avoids silently converting authored values to f64.
    Decimal(String),
    Text(String),
    DateTime(String),
    Error(ErrorValue),
    List(Vec<CellValue>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorValue {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColumnType {
    Any,
    Boolean,
    Integer,
    Decimal,
    Text,
    DateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Column {
    pub id: String,
    pub title: String,
    pub value_type: ColumnType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Row {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Values are aligned to the table's ordered column list.
    pub cells: Vec<CellValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JabrTable {
    pub schema_version: u32,
    pub id: String,
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorOrientation {
    Row,
    Column,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JabrVector {
    pub schema_version: u32,
    pub id: String,
    pub orientation: VectorOrientation,
    pub values: Vec<CellValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JabrMatrix {
    pub schema_version: u32,
    pub id: String,
    pub rows: usize,
    pub columns: usize,
    /// Row-major storage. Length must equal rows * columns.
    pub values: Vec<CellValue>,
    #[serde(default)]
    pub row_labels: Vec<String>,
    #[serde(default)]
    pub column_labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JabrList {
    pub schema_version: u32,
    pub id: String,
    pub values: Vec<CellValue>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DataError {
    #[error("matrix shape {rows}x{columns} does not match {actual} stored values")]
    MatrixShape { rows: usize, columns: usize, actual: usize },
    #[error("matrix row-label count {labels} does not match row count {rows}")]
    MatrixRowLabels { labels: usize, rows: usize },
    #[error("matrix column-label count {labels} does not match column count {columns}")]
    MatrixColumnLabels { labels: usize, columns: usize },
    #[error("duplicate or empty {kind} identifier: {id}")]
    InvalidId { kind: &'static str, id: String },
    #[error("column index {index} is outside table width {columns}")]
    ColumnIndex { index: usize, columns: usize },
    #[error("row {row_id} has {actual} cells; expected {expected}") ]
    RowWidth { row_id: String, actual: usize, expected: usize },
    #[error("nested list depth exceeds the configured maximum of {max}")]
    ListDepth { max: usize },
}
