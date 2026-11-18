//! UI-independent CSV/TSV, JSON, and XML table interchange adapters.
use crate::{CellValue, Column, ColumnType, Id, Table, TableError};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InterchangeError {
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("XML error: {0}")]
    Xml(String),
    #[error("table validation failed: {0}")]
    InvalidTable(#[from] TableError),
    #[error("XML record element '{0}' was not found")]
    RecordNotFound(String),
    #[error("record {record} has {actual} fields; expected {expected}")]
    FieldCount {
        record: usize,
        expected: usize,
        actual: usize,
    },
}

#[derive(Debug, Clone)]
pub struct DelimitedOptions {
    pub table_id: Id,
    pub delimiter: u8,
    pub has_headers: bool,
}
impl DelimitedOptions {
    pub fn csv(table_id: impl Into<Id>) -> Self {
        Self {
            table_id: table_id.into(),
            delimiter: b',',
            has_headers: true,
        }
    }
    pub fn tsv(table_id: impl Into<Id>) -> Self {
        Self {
            table_id: table_id.into(),
            delimiter: b'\t',
            has_headers: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct XmlImportOptions {
    pub table_id: Id,
    pub record_element: String,
    /// XML child-element name -> stable table column ID.
    pub field_to_column: BTreeMap<String, Id>,
}
impl XmlImportOptions {
    pub fn new(table_id: impl Into<Id>, record_element: impl Into<String>) -> Self {
        Self {
            table_id: table_id.into(),
            record_element: record_element.into(),
            field_to_column: BTreeMap::new(),
        }
    }
}

pub fn import_delimited(
    input: &str,
    options: &DelimitedOptions,
) -> Result<Table, InterchangeError> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(options.delimiter)
        .has_headers(options.has_headers)
        .flexible(false)
        .from_reader(input.as_bytes());
    let headers: Vec<String> = if options.has_headers {
        reader.headers()?.iter().map(str::to_owned).collect()
    } else {
        let count = reader.headers()?.len();
        (0..count).map(|i| format!("Column {}", i + 1)).collect()
    };
    let records: Vec<Vec<String>> = reader
        .records()
        .map(|record| record.map(|r| r.iter().map(str::to_owned).collect()))
        .collect::<Result<_, _>>()?;
    let columns = headers
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let kind = infer_type(
                records
                    .iter()
                    .filter_map(|row| row.get(i))
                    .map(String::as_str),
            );
            Column {
                id: format!("column_{}", i + 1),
                name: name.clone(),
                value_type: kind,
            }
        })
        .collect();
    let mut table = Table::new(options.table_id.clone(), columns)?;
    for (index, record) in records.iter().enumerate() {
        if record.len() != headers.len() {
            return Err(InterchangeError::FieldCount {
                record: index + 1,
                expected: headers.len(),
                actual: record.len(),
            });
        }
        let row_id = format!("row_{}", index + 1);
        table.add_row(row_id.clone())?;
        for (i, raw) in record.iter().enumerate() {
            table.set_cell(&row_id, &format!("column_{}", i + 1), parse_cell(raw))?;
        }
    }
    Ok(table)
}

pub fn export_delimited(
    table: &Table,
    delimiter: u8,
    include_headers: bool,
) -> Result<String, InterchangeError> {
    validate_table(table)?;
    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(Vec::new());
    if include_headers {
        writer.write_record(table.columns.iter().map(|c| c.name.as_str()))?;
    }
    for row in &table.rows {
        writer.write_record(
            table
                .columns
                .iter()
                .map(|c| cell_string(row.cells.get(&c.id).unwrap_or(&CellValue::Empty))),
        )?;
    }
    let bytes = writer
        .into_inner()
        .map_err(|e| InterchangeError::Xml(e.to_string()))?;
    String::from_utf8(bytes).map_err(|e| InterchangeError::Xml(e.to_string()))
}

pub fn import_json(input: &str) -> Result<Table, InterchangeError> {
    let table: Table = serde_json::from_str(input)?;
    validate_table(&table)?;
    Ok(table)
}
pub fn export_json(table: &Table) -> Result<String, InterchangeError> {
    validate_table(table)?;
    Ok(serde_json::to_string_pretty(table)?)
}

/// Import repeated XML records. Namespaced fields are matched by local name.
/// DTD processing and external entity resolution are not enabled.
pub fn import_xml(input: &str, options: &XmlImportOptions) -> Result<Table, InterchangeError> {
    let document =
        roxmltree::Document::parse(input).map_err(|e| InterchangeError::Xml(e.to_string()))?;
    let records: Vec<_> = document
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == options.record_element)
        .collect();
    if records.is_empty() {
        return Err(InterchangeError::RecordNotFound(
            options.record_element.clone(),
        ));
    }
    let mut names = Vec::<String>::new();
    for record in &records {
        for field in record.children().filter(|n| n.is_element()) {
            let name = field.tag_name().name().to_owned();
            if !names.contains(&name) {
                names.push(name);
            }
        }
    }
    // Canonical exports carry enough column metadata to restore IDs, names,
    // and declared types. Generic XML falls back to inference and mapping.
    let metadata: BTreeMap<String, (String, String, ColumnType)> = document
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "column")
        .filter_map(|n| {
            let id = n.attribute("id")?.to_owned();
            let name = n.attribute("name").unwrap_or(&id).to_owned();
            let kind = match n.attribute("type").unwrap_or("any") {
                "text" => ColumnType::Text,
                "number" => ColumnType::Number,
                "boolean" => ColumnType::Boolean,
                _ => ColumnType::Any,
            };
            Some((xml_tag(&id), (id, name, kind)))
        })
        .collect();
    let field_values: Vec<Vec<String>> = names
        .iter()
        .map(|name| {
            records
                .iter()
                .flat_map(|record| {
                    record
                        .children()
                        .filter(|n| n.is_element() && n.tag_name().name() == name)
                        .map(|n| n.text().unwrap_or("").trim().to_owned())
                })
                .collect()
        })
        .collect();
    let columns = names
        .iter()
        .enumerate()
        .map(|(i, field_name)| {
            if let Some((id, name, kind)) = metadata.get(field_name) {
                let mapped_id = options
                    .field_to_column
                    .get(field_name)
                    .cloned()
                    .unwrap_or_else(|| id.clone());
                Column {
                    id: mapped_id,
                    name: name.clone(),
                    value_type: *kind,
                }
            } else {
                let id = options
                    .field_to_column
                    .get(field_name)
                    .cloned()
                    .unwrap_or_else(|| {
                        field_name
                            .strip_prefix("c_")
                            .unwrap_or(field_name)
                            .to_owned()
                    });
                let kind = infer_type(field_values[i].iter().map(String::as_str));
                Column {
                    id,
                    name: field_name.clone(),
                    value_type: kind,
                }
            }
        })
        .collect();
    let mut table = Table::new(options.table_id.clone(), columns)?;
    for (index, record) in records.iter().enumerate() {
        let row_id = record
            .attribute("id")
            .filter(|id| !id.trim().is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("row_{}", index + 1));
        table.add_row(row_id.clone())?;
        for field in record.children().filter(|n| n.is_element()) {
            let field_name = field.tag_name().name();
            let id = options
                .field_to_column
                .get(field_name)
                .cloned()
                .or_else(|| metadata.get(field_name).map(|entry| entry.0.clone()))
                .unwrap_or_else(|| {
                    field_name
                        .strip_prefix("c_")
                        .unwrap_or(field_name)
                        .to_owned()
                });
            let target_id = options
                .field_to_column
                .get(field_name)
                .cloned()
                .unwrap_or(id);
            table.set_cell(
                &row_id,
                &target_id,
                parse_cell(field.text().unwrap_or("").trim()),
            )?;
        }
    }
    Ok(table)
}

/// Export canonical XML with table/column metadata and row records.
pub fn export_xml(table: &Table) -> Result<String, InterchangeError> {
    validate_table(table)?;
    let mut out = format!("<table id=\"{}\"><columns>", xml_escape(&table.id));
    for c in &table.columns {
        out.push_str(&format!(
            "<column id=\"{}\" name=\"{}\" type=\"{}\"/>",
            xml_escape(&c.id),
            xml_escape(&c.name),
            type_name(c.value_type)
        ));
    }
    out.push_str("</columns><rows>");
    for row in &table.rows {
        out.push_str(&format!("<row id=\"{}\">", xml_escape(&row.id)));
        for c in &table.columns {
            let tag = xml_tag(&c.id);
            out.push_str(&format!(
                "<{tag}>{}</{tag}>",
                xml_escape(&cell_string(
                    row.cells.get(&c.id).unwrap_or(&CellValue::Empty)
                ))
            ));
        }
        out.push_str("</row>");
    }
    out.push_str("</rows></table>");
    Ok(out)
}

fn infer_type<'a>(values: impl Iterator<Item = &'a str>) -> ColumnType {
    let mut kinds = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            continue;
        }
        let kind = if value.trim().parse::<f64>().is_ok_and(f64::is_finite) {
            1
        } else if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
            2
        } else {
            3
        };
        kinds.insert(kind);
    }
    match kinds.len() {
        0 => ColumnType::Any,
        1 => match *kinds.iter().next().unwrap() {
            1 => ColumnType::Number,
            2 => ColumnType::Boolean,
            _ => ColumnType::Text,
        },
        _ => ColumnType::Any,
    }
}
fn parse_cell(raw: &str) -> CellValue {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        CellValue::Empty
    } else if trimmed.eq_ignore_ascii_case("true") {
        CellValue::Boolean(true)
    } else if trimmed.eq_ignore_ascii_case("false") {
        CellValue::Boolean(false)
    } else if let Ok(n) = trimmed.parse::<f64>() {
        if n.is_finite() {
            CellValue::Number(n)
        } else {
            CellValue::Text(raw.to_owned())
        }
    } else {
        CellValue::Text(raw.to_owned())
    }
}
fn cell_string(value: &CellValue) -> String {
    match value {
        CellValue::Empty => String::new(),
        CellValue::Text(s) => s.clone(),
        CellValue::Number(n) => n.to_string(),
        CellValue::Boolean(b) => b.to_string(),
    }
}
fn validate_table(table: &Table) -> Result<(), InterchangeError> {
    Table::new(table.id.clone(), table.columns.clone())?;
    let ids: BTreeSet<_> = table.columns.iter().map(|c| c.id.as_str()).collect();
    let mut row_ids = BTreeSet::new();
    for row in &table.rows {
        if row.id.trim().is_empty() {
            return Err(TableError::EmptyRowId.into());
        }
        if !row_ids.insert(row.id.as_str()) {
            return Err(TableError::DuplicateRowId(row.id.clone()).into());
        }
        for (id, value) in &row.cells {
            let col = table
                .columns
                .iter()
                .find(|c| c.id == *id)
                .ok_or_else(|| TableError::UnknownColumn(id.clone()))?;
            let valid = match (col.value_type, value) {
                (_, CellValue::Empty) => true,
                (ColumnType::Any, CellValue::Number(n)) => n.is_finite(),
                (ColumnType::Any, _) => true,
                (ColumnType::Text, CellValue::Text(_)) => true,
                (ColumnType::Number, CellValue::Number(n)) => n.is_finite(),
                (ColumnType::Boolean, CellValue::Boolean(_)) => true,
                _ => false,
            };
            if !valid {
                return Err(TableError::TypeMismatch(id.clone()).into());
            }
        }
        if row.cells.keys().any(|id| !ids.contains(id.as_str())) {
            return Err(InterchangeError::Xml(
                "row contains an unknown column ID".into(),
            ));
        }
    }
    Ok(())
}
fn type_name(t: ColumnType) -> &'static str {
    match t {
        ColumnType::Any => "any",
        ColumnType::Text => "text",
        ColumnType::Number => "number",
        ColumnType::Boolean => "boolean",
    }
}
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
fn xml_tag(id: &str) -> String {
    let mut out = String::from("c_");
    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> Table {
        let mut t = Table::new(
            "members",
            vec![
                Column {
                    id: "name".into(),
                    name: "Name".into(),
                    value_type: ColumnType::Text,
                },
                Column {
                    id: "area".into(),
                    name: "Area".into(),
                    value_type: ColumnType::Number,
                },
                Column {
                    id: "active".into(),
                    name: "Active".into(),
                    value_type: ColumnType::Boolean,
                },
            ],
        )
        .unwrap();
        t.add_row("r1").unwrap();
        t.set_cell("r1", "name", CellValue::Text("Beam, B".into()))
            .unwrap();
        t.set_cell("r1", "area", CellValue::Number(12.5)).unwrap();
        t.set_cell("r1", "active", CellValue::Boolean(true))
            .unwrap();
        t
    }
    #[test]
    fn csv_round_trip_handles_quoted_fields_and_types() {
        let csv = export_delimited(&sample(), b',', true).unwrap();
        let t = import_delimited(&csv, &DelimitedOptions::csv("copy")).unwrap();
        assert_eq!(
            t.cell("row_1", "column_1").unwrap(),
            &CellValue::Text("Beam, B".into())
        );
        assert_eq!(
            t.cell("row_1", "column_2").unwrap(),
            &CellValue::Number(12.5)
        );
        assert_eq!(
            t.cell("row_1", "column_3").unwrap(),
            &CellValue::Boolean(true)
        );
    }
    #[test]
    fn tsv_import_uses_tab_delimiter() {
        let t = import_delimited("Name\tArea\nBeam\t25\n", &DelimitedOptions::tsv("t")).unwrap();
        assert_eq!(
            t.cell("row_1", "column_2").unwrap(),
            &CellValue::Number(25.0)
        );
    }
    #[test]
    fn json_round_trip_preserves_ids_and_types() {
        let t = sample();
        assert_eq!(import_json(&export_json(&t).unwrap()).unwrap(), t);
    }
    #[test]
    fn xml_import_supports_mapping_and_namespaces() {
        let xml = r#"<root xmlns:x="urn:x"><x:item><x:label>Beam A</x:label><x:area>18.2</x:area></x:item></root>"#;
        let mut opts = XmlImportOptions::new("t", "item");
        opts.field_to_column
            .insert("label".into(), "member_name".into());
        let t = import_xml(xml, &opts).unwrap();
        assert_eq!(
            t.cell("row_1", "member_name").unwrap(),
            &CellValue::Text("Beam A".into())
        );
        assert_eq!(t.cell("row_1", "area").unwrap(), &CellValue::Number(18.2));
    }
    #[test]
    fn xml_export_import_round_trip_preserves_column_and_row_metadata() {
        let original = sample();
        let xml = export_xml(&original).unwrap();
        let imported = import_xml(&xml, &XmlImportOptions::new("members", "row")).unwrap();
        assert_eq!(imported, original);
    }
    #[test]
    fn xml_export_escapes_text() {
        let xml = export_xml(&sample()).unwrap();
        assert!(xml.contains("<c_name>Beam, B</c_name>"));
    }
    #[test]
    fn malformed_xml_is_rejected() {
        assert!(import_xml("<root><item>", &XmlImportOptions::new("t", "item")).is_err());
    }
}
