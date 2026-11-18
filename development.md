# Jabr-table Development Plan

## Purpose

Jabr-table is the **data and computation engine** for tabular engineering data in the Jabr ecosystem. It is a Rust library, not a standalone application.

- **Jabr-table:** table data model, data operations, interchange, and matrix utilities.
- **Jabr-UI:** all visual presentation, editing interactions, BlockSuite integration, and final product integration.
- **Jabr-CLI:** calculation language, expression parsing/evaluation, and engineering calculation semantics.

Jabr-table must expose small, stable APIs and must not implement UI components, a standalone editor, or a second formula language.

## Scope and architecture

### Core table model

- Tables, columns, rows, and typed cell values.
- Stable row and column IDs; never use row/column positions as durable identity.
- Basic create/read/update/delete operations.
- Validation for column types and row shape.
- Framework-independent, serializable data structures.
- No BlockSuite or Yjs dependency in the core.

### Data operations

Initial functions:
- Exact and approximate lookup primitives.
- `VLOOKUP`, `HLOOKUP`, and `MATCH` semantics.
- Numeric `MAX` and `MIN` aggregation.

These are library operations, not a formula parser. Formula syntax, dependency tracking, and worksheet evaluation belong behind the Jabr-CLI/formula-engine boundary. Evaluate Formualizer as an implementation candidate only after defining and testing the adapter contract.

### Data interchange

- CSV and TSV
- JSON
- XML

Use explicit import/export adapters. Preserve typed values where possible. XML must support configurable record elements and field mapping; do not assume every XML document is a flat table. Protect XML parsing against DTD/external-entity expansion and test namespaces and malformed input.

### Matrix utilities

- Small dense matrices and vectors for element-level operations.
- Evaluate `faer` for numerical operations and sparse solver capabilities.
- Consider `nalgebra` for small fixed-size geometry/element matrices.
- Do not store large FEM global sparse matrices as ordinary table cells.
- FEM assembly, mesh, boundary conditions, and structural design logic are outside Jabr-table; they belong in the engineering calculation layer.

## Development stages

### Stage 0 — Core table PoC (start here)

- [ ] Rust library crate and CI.
- [ ] Typed cell values and column definitions.
- [ ] Stable row/column IDs.
- [ ] Table CRUD operations.
- [ ] Validation and error types.
- [ ] Unit tests for valid and invalid operations.

**Exit criteria:** a UI-independent Rust library can create, inspect, update, and remove tabular data with deterministic behavior and tests.

### Stage 1 — Lookup and aggregation PoC

- [ ] Exact-match `MATCH`.
- [ ] Exact-match `VLOOKUP` and `HLOOKUP`.
- [ ] Numeric `MAX` and `MIN`.
- [ ] Explicit behavior for duplicates, missing keys, empty values, and type mismatches.
- [ ] Add approximate matching only with documented ordering requirements and tests.

**Exit criteria:** the requested functions work as independently testable APIs without a formula parser.

### Stage 2 — Data interchange MVP

- [ ] CSV import/export.
- [ ] TSV import/export.
- [ ] JSON import/export.
- [ ] XML import/export with mapping and preview data structures.
- [ ] Round-trip, malformed-input, type inference, and size-limit tests.

**Exit criteria:** data can be exchanged without coupling to UI or document storage.

### Stage 3 — Matrix PoC

- [ ] Dense matrix/vector value representation or adapter contract.
- [ ] Addition, subtraction, multiplication, transpose, and matrix-vector multiplication.
- [ ] Dimension and numerical error handling.
- [ ] Benchmark `faer` and `nalgebra` against small FEM element-matrix fixtures.

**Exit criteria:** correct small dense matrix operations with verified test fixtures. Sparse global FEM solving remains out of scope for this stage.

### Stage 4 — Integration contract

- [ ] Publish Rust API and serialization contracts for Jabr-UI.
- [ ] Define adapter boundary for Jabr-CLI formula evaluation.
- [ ] Keep persistence/collaboration ownership with Jabr-UI/BlockSuite.
- [ ] Add cross-repository contract fixtures and compatibility tests.

**Exit criteria:** Jabr-UI can consume table data operations without importing UI concerns into this repository.

## Non-goals

- No standalone UI, web app, or table/database editor.
- No BlockSuite blocks, Yjs document ownership, or visual components.
- No full Excel clone, XLSX workbook fidelity, macros, or pivot tables.
- No SQL database engine.
- No formula language or expression parser duplicating Jabr-CLI.
- No full FEM solver, meshing, or structural design checks.

## Initial repository layout

```text
crates/
  jabr-table-core/
    src/
    tests/
Cargo.toml
development.md
```

Add packages only when a stable boundary and real implementation need exist.

## Validation

Every stage requires unit tests and CI. Numerical stages require independently verified fixtures. Import/export stages require round-trip and malformed-input tests. Do not claim a feature is implemented until its tests pass.
