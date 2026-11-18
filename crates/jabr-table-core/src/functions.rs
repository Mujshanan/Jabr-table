//! Standalone spreadsheet-style data operations.
//!
//! These functions operate directly on values. They do not parse formulas,
//! maintain dependencies, or evaluate expressions.

use crate::CellValue;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FunctionError {
    #[error("lookup value was not found")]
    NotFound,
    #[error("lookup range is empty")]
    EmptyRange,
    #[error("column index must be at least 1")]
    InvalidColumnIndex,
    #[error("row index must be at least 1")]
    InvalidRowIndex,
    #[error("table must contain at least one row and one column")]
    EmptyTable,
    #[error("table rows must all have the same number of columns")]
    RaggedTable,
}

/// Exact-match MATCH. Returns a 1-based position, matching spreadsheet conventions.
/// If duplicate values exist, the first matching position is returned.
pub fn match_index(lookup_value: &CellValue, lookup_range: &[CellValue]) -> Result<usize, FunctionError> {
    if lookup_range.is_empty() {
        return Err(FunctionError::EmptyRange);
    }
    lookup_range
        .iter()
        .position(|value| value == lookup_value)
        .map(|index| index + 1)
        .ok_or(FunctionError::NotFound)
}

/// Exact-match VLOOKUP. Searches the first column and returns the requested
/// 1-based column from the first matching row.
pub fn vlookup<'a>(
    lookup_value: &CellValue,
    table: &'a [Vec<CellValue>],
    return_column: usize,
) -> Result<&'a CellValue, FunctionError> {
    if return_column == 0 {
        return Err(FunctionError::InvalidColumnIndex);
    }
    validate_table(table)?;
    if return_column > table[0].len() {
        return Err(FunctionError::InvalidColumnIndex);
    }

    table
        .iter()
        .find(|row| &row[0] == lookup_value)
        .map(|row| &row[return_column - 1])
        .ok_or(FunctionError::NotFound)
}

/// Exact-match HLOOKUP. Searches the first row and returns the requested
/// 1-based row from the first matching column.
pub fn hlookup<'a>(
    lookup_value: &CellValue,
    table: &'a [Vec<CellValue>],
    return_row: usize,
) -> Result<&'a CellValue, FunctionError> {
    if return_row == 0 {
        return Err(FunctionError::InvalidRowIndex);
    }
    validate_table(table)?;
    if return_row > table.len() {
        return Err(FunctionError::InvalidRowIndex);
    }

    table[0]
        .iter()
        .position(|value| value == lookup_value)
        .map(|column| &table[return_row - 1][column])
        .ok_or(FunctionError::NotFound)
}

fn validate_table(table: &[Vec<CellValue>]) -> Result<(), FunctionError> {
    let Some(first_row) = table.first() else {
        return Err(FunctionError::EmptyTable);
    };
    if first_row.is_empty() {
        return Err(FunctionError::EmptyTable);
    }
    if table.iter().any(|row| row.len() != first_row.len()) {
        return Err(FunctionError::RaggedTable);
    }
    Ok(())
}

/// MAX over a range of cell references. Empty, text, and boolean values are
/// ignored, consistent with spreadsheet range aggregation. Returns None if
/// the range contains no numeric values.
pub fn max(values: &[CellValue]) -> Option<f64> {
    values.iter().filter_map(numeric_value).reduce(f64::max)
}

/// MIN over a range of cell references. Empty, text, and boolean values are
/// ignored, consistent with spreadsheet range aggregation. Returns None if
/// the range contains no numeric values.
pub fn min(values: &[CellValue]) -> Option<f64> {
    values.iter().filter_map(numeric_value).reduce(f64::min)
}

fn numeric_value(value: &CellValue) -> Option<f64> {
    match value {
        CellValue::Number(number) if number.is_finite() => Some(*number),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(value: &str) -> CellValue {
        CellValue::Text(value.to_owned())
    }

    fn number(value: f64) -> CellValue {
        CellValue::Number(value)
    }

    #[test]
    fn match_returns_first_one_based_exact_match() {
        let range = vec![text("A"), text("B"), text("B")];
        assert_eq!(match_index(&text("B"), &range), Ok(2));
    }

    #[test]
    fn match_reports_missing_and_empty_ranges() {
        assert_eq!(match_index(&text("X"), &[text("A")]), Err(FunctionError::NotFound));
        assert_eq!(match_index(&text("X"), &[]), Err(FunctionError::EmptyRange));
    }

    #[test]
    fn vlookup_returns_value_from_first_matching_row() {
        let table = vec![
            vec![text("A"), number(10.0)],
            vec![text("B"), number(20.0)],
            vec![text("B"), number(30.0)],
        ];
        assert_eq!(vlookup(&text("B"), &table, 2), Ok(&number(20.0)));
    }

    #[test]
    fn vlookup_checks_column_index_and_rectangular_shape() {
        let table = vec![vec![text("A"), number(10.0)]];
        assert_eq!(vlookup(&text("A"), &table, 0), Err(FunctionError::InvalidColumnIndex));
        assert_eq!(vlookup(&text("A"), &table, 3), Err(FunctionError::InvalidColumnIndex));
        let ragged = vec![vec![text("A")], vec![text("B"), number(2.0)]];
        assert_eq!(vlookup(&text("A"), &ragged, 1), Err(FunctionError::RaggedTable));
    }

    #[test]
    fn hlookup_returns_value_from_requested_row() {
        let table = vec![
            vec![text("A"), text("B"), text("C")],
            vec![number(1.0), number(2.0), number(3.0)],
        ];
        assert_eq!(hlookup(&text("B"), &table, 2), Ok(&number(2.0)));
    }

    #[test]
    fn hlookup_checks_row_index() {
        let table = vec![vec![text("A"), text("B")]];
        assert_eq!(hlookup(&text("A"), &table, 0), Err(FunctionError::InvalidRowIndex));
        assert_eq!(hlookup(&text("A"), &table, 2), Err(FunctionError::InvalidRowIndex));
    }

    #[test]
    fn max_and_min_ignore_non_numeric_values() {
        let values = vec![CellValue::Empty, text("ignored"), CellValue::Boolean(true), number(-4.0), number(8.5)];
        assert_eq!(max(&values), Some(8.5));
        assert_eq!(min(&values), Some(-4.0));
    }

    #[test]
    fn max_and_min_return_none_without_numbers() {
        let values = vec![CellValue::Empty, text("not numeric")];
        assert_eq!(max(&values), None);
        assert_eq!(min(&values), None);
    }
}
