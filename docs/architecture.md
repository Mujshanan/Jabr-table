# Jabr-table architecture and integration contract

## 1. Ownership boundaries

| Concern | Owner |
|---|---|
| BlockSuite blocks, selection, keyboard interaction, toolbar, visual rendering | Jabr-UI |
| Yjs persistence, collaboration, undo/redo | Jabr-UI / BlockSuite host |
| Authored table/list/vector/matrix data contract and conversions | Jabr-table |
| CSV/XLSX parsing and export policy | Jabr-table |
| Deterministic operations over typed arrays, matrices, tables and ranges | Jabr-table Rust core |
| Worksheet syntax, AST, variables, scopes, dependency graph, units and evaluation lifecycle | Jabr-CLI |
| Calling Jabr-table operations from worksheet expressions | Jabr-CLI adapter |

Jabr-table must remain usable without a browser or BlockSuite. Jabr-UI must not reimplement matrix arithmetic, range functions, spreadsheet formula parsing, or file-format semantics. Jabr-CLI should call the shared Rust operations rather than fork them.

## 2. Canonical value model

Use an explicit tagged value model, not arbitrary JavaScript cell values. Initial scalar variants: Empty, Boolean, Integer, Decimal (canonical decimal string), Text, DateTime (normalized ISO-8601), and Error (stable code plus message).

Formula cells contain authored expression source and stable cell/range references. Computed values and diagnostics are transient evaluation output, not authoritative authored content. Units and dimensional quantities use the Jabr-CLI quantity contract; Jabr-table preserves quantity payloads and does not invent a second unit system.

## 3. Data structures

### Table
Stable table ID; ordered columns with stable IDs, title, declared/inferred type, optional unit hint and formatting metadata; ordered rows with stable row IDs and cells keyed by column ID; optional import metadata and formula source. No UI coordinates, selection, viewport, or rendered dimensions.

### Vector
Ordered one-dimensional typed sequence, optional semantic name and unit metadata. Preserve row/column orientation when imported from a matrix or worksheet range. A vector is not a one-column table.

### Matrix
Rectangular row-major values with explicit row and column counts. Invariant: values.length == rows * columns. Optional row/column labels and semantic metadata. Ragged input is rejected or normalized only through an explicit policy; never silently padded.

### List
Ordered sequence that may be heterogeneous, with bounded nesting and cycle-free serialization. Lists are distinct from numeric vectors; numeric/homogeneous operations validate before execution.

## 4. Conversion rules

Conversions are explicit and return typed results or structured diagnostics:
- Table → Matrix: caller chooses columns and policy for headers, blanks, errors, and nonnumeric cells. Retain column IDs/labels in metadata.
- Matrix → Table: each matrix column becomes a table column; labels become headers only when requested. Preserve shape and source metadata.
- Table → Vector: select exactly one column or row and declare orientation.
- Vector → Table: choose row/column orientation and optional header.
- Table → List: select rows, columns, or records; preserve order.
- List → Table: supply or derive a schema; reject inconsistent records unless an explicit fill/error policy is selected.

No silent truncation, coercion, or loss of headers.

## 5. Spreadsheet functions and formula boundary

Implement spreadsheet functions as typed Rust operations over values/ranges, not as a competing formula parser. Jabr-CLI parses worksheet expressions and dispatches recognized functions to this library.

Initial function families:
- Aggregation: SUM, AVERAGE, MIN, MAX, COUNT, COUNTA.
- Conditional aggregation: SUMIF, COUNTIF, AVERAGEIF.
- Lookup/reference: INDEX, MATCH, XLOOKUP (exact-match baseline first).
- Logical: IF, AND, OR, NOT.
- Text: CONCAT, TEXTJOIN, LEFT, RIGHT, MID, LEN, TRIM.
- Numeric: ABS, ROUND, ROUNDUP, ROUNDDOWN, MOD, POWER, SQRT.
- Array/range: TRANSPOSE, FILTER, UNIQUE, SORT.
- Matrix: TRANSPOSE, MMULT, MINVERSE, MDETERM, identity/shape helpers.

Compatibility is function-by-function; never claim full Excel compatibility. Formula syntax, references, recalculation graph, named ranges, and workbook formula persistence remain under Jabr-CLI/workbook integration. Errors use stable codes (#VALUE!, #REF!, #DIV/0!, #N/A!, #NUM!) with structured details.

## 6. Data I/O

CSV import/export has explicit delimiter, encoding, header, quote, decimal and date policies. XLSX import/export preserves sheet names, ordering, cell types, formula source where supported, and basic number formats. Import parses into a staging model with diagnostics before caller commit. Export distinguishes literal values from formula source; cached results are not authoritative unless produced by a compatible calculation pass.

Enforce file-size, row, column, nesting, and formula-length limits. Never execute macros, external links, embedded scripts, or arbitrary workbook code.

## 7. Rust workspace and TypeScript boundary

Proposed workspace:
- `crates/jabr-table-core`: model, validation, conversions, range operations, spreadsheet functions.
- `crates/jabr-table-io`: CSV/XLSX import/export adapters.
- `bindings/`: versioned JSON/Serde DTOs and a narrow Jabr-CLI adapter; select browser bindings only after runtime requirements are validated.
- `packages/contracts/`: TypeScript types and schema/version guards consumed by Jabr-UI.

Keep core deterministic and independent of filesystem/UI. IO owns file bytes and format-specific parsing. Use versioned serialized DTOs at process boundaries. Preserve exact decimal text and stable IDs.

## 8. BlockSuite composition (implemented in Jabr-UI)

Focused block flavours, not one monolithic block:
- `jabr:table`: editable grid, row/column operations, import/export, range operations.
- `jabr:vector`: one-dimensional typed sequence, orientation and unit display.
- `jabr:matrix`: rectangular grid, shape controls, matrix operations, conversion to/from table.
- `jabr:list`: ordered heterogeneous/nested list with explicit conversion commands.

Persist authored data and stable IDs only. Selection, active cell, clipboard, editing mode, formula diagnostics, computed values, and transient progress remain outside persisted block props unless intentionally authored. BlockSuite schema migrations belong to Jabr-UI; payload schema versions belong to Jabr-table. Conversion is one coherent authored command for undo/redo.

## 9. Acceptance principles

Pure Rust tests cover shape, typing, conversions, error propagation and numerical edge cases. Golden fixtures cover CSV/XLSX round trips and spreadsheet functions. TypeScript tests prove JSON compatibility and schema-version rejection. Jabr-UI browser tests cover editing, selection, keyboard ownership, paste/import, conversion, undo/redo and persistence. Jabr-CLI integration tests prove AST dispatch to the same Rust operations without duplicate implementations.
