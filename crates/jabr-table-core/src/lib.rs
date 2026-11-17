mod conversions;
mod model;
mod validation;

pub use conversions::{
    matrix_to_table, table_to_matrix, MatrixToTableOptions, TableToMatrixOptions,
};
pub use model::{
    CellValue, Column, ColumnType, DataError, ErrorValue, JabrList, JabrMatrix,
    JabrTable, JabrVector, Row, VectorOrientation,
};
pub use validation::{validate_matrix, validate_table, validate_vector};
