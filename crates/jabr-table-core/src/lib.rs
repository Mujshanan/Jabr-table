mod model;
mod validation;

pub use model::{
    CellValue, Column, ColumnType, DataError, ErrorValue, JabrList, JabrMatrix,
    JabrTable, JabrVector, Row, VectorOrientation,
};
pub use validation::{validate_matrix, validate_table, validate_vector};
