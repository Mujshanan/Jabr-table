# Jabr Table Development Plan

> **Architecture baseline (2026-11-18):** Jabr Table owns table structures, stable table/row/column/cell identity, typed data operations and interchange. Jabr Math owns parsing, AST, values, units and expression evaluation. Jabr-engine (planned; repository not yet created) owns worksheet graph construction, cross-block bindings, scheduling and result lifecycle. Jabr-kernel provides generic NodeId/Port, Revision, Snapshot<T>, ResultStore<V>, graph, diagnostic, span and envelope primitives only. Jabr-UI owns BlockSuite/Yjs authored document state and presentation.

## Purpose

Jabr-table is the **data and computation engine** for tabular engineering data in the Jabr ecosystem. It is a Rust library, not a standalone application.

- **Jabr-table:** table data model, data operations, interchange, and matrix utilities.
- **Jabr-UI:** all visual presentation, editing interactions, BlockSuite integration, and final product integration.
- **Jabr-math:** calculation language, expression parsing/evaluation, and engineering calculation semantics.

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

These are library operations, not a formula parser. Formula syntax, AST and expression evaluation belong to Jabr Math; cross-block dependency construction, worksheet scheduling and result lifecycle belong to Jabr-engine. Table exposes pure data operations and does not parse formulas or evaluate Math expressions.

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

- [x] Exact-match `MATCH` (`match_index`, 1-based result).
- [x] Exact-match `VLOOKUP` and `HLOOKUP` (1-based return row/column).
- [x] Numeric `MAX` and `MIN` (ignore empty, text, and boolean range values).
- [x] Explicit behavior for duplicates, missing keys, empty ranges, invalid indexes, and ragged lookup tables.
- [ ] Decide whether approximate lookup is needed; do not add it without explicit ordering semantics and tests.

**Exit criteria:** the requested functions work as independently testable APIs without a formula parser.

**Current implementation note:** lookup operations use exact equality and return the first match. `MATCH` returns a 1-based position. `VLOOKUP` searches the first column; `HLOOKUP` searches the first row. `MAX`/`MIN` return `Option<f64>`, with `None` when no finite numeric values exist.

### Stage 2 — Data interchange MVP

- [x] CSV import/export with quoted fields, configurable headers, and type inference.
- [x] TSV import/export through the same delimiter-aware adapter.
- [x] JSON import/export preserving the native table structure and stable IDs.
- [x] XML import/export with configurable record elements, field-to-column mapping, namespace-local field names, and safe parser defaults.
- [x] Round-trip and malformed-input tests for supported formats.
- [ ] Add configurable import limits and explicit schema/column mapping for CSV/JSON/XML.
- [ ] Add larger-file streaming benchmarks and resource-limit tests.

**Current implementation:** `interchange` exposes delimited-text, JSON, and XML adapters. CSV/TSV use a standards-aware parser/writer; delimited imports infer homogeneous number/boolean/text columns and use `Any` for mixed or empty columns. JSON preserves the serde table representation. XML accepts a configurable repeated record element and field-to-column mapping, and recognizes namespace-local names. XML parsing does not enable external entity resolution. The canonical XML exporter includes table and column metadata.

**Remaining hardening:** explicit input-size/row/column limits, richer XML metadata round-tripping, and streaming performance tests.

**Exit criteria:** data can be exchanged without coupling to UI or document storage.

### Stage 3 — Matrix PoC

- [x] Dense row-major matrix representation with validated dimensions and finite values.
- [x] Addition, subtraction, multiplication, transpose, and matrix-vector multiplication.
- [x] Dimension and numerical error handling.
- [x] Deterministic matrix identity checks across rectangular shapes.
- [x] Closed-form 2D truss local stiffness fixture.
- [x] Closed-form 2D Euler-Bernoulli beam local stiffness fixture.
- [x] 2D truss coordinate transformation and global stiffness fixture.
- [x] Arithmetic overflow rejection tests.
- [x] Add an optional, single-threaded benchmark harness comparing DenseMatrix, faer, and nalgebra on representative matrix sizes.
- [ ] Run benchmarks on target Windows and Linux hardware and record environment plus raw results.
- [ ] Decide whether to retain the small dense implementation or delegate operations to a numerical backend.

**Current implementation:** matrix::DenseMatrix is a dependency-light, row-major dense matrix for small element-level calculations. Its integration fixtures verify the 2D truss axial stiffness matrix, the 2D Euler-Bernoulli beam local stiffness matrix, and the transformed global stiffness matrix for a 30-degree truss. Deterministic identity checks cover rectangular shapes, and overflow tests ensure arithmetic does not return non-finite matrix/vector results. Fixture values are derived from the closed-form element stiffness equations and checked independently of the implementation's matrix multiplication path. An optional backend benchmark harness is available at `crates/jabr-table-core/benches/matrix_backend_compare.rs`; methodology and workloads are documented in `docs/matrix-backend-benchmark.md`. The candidate libraries are feature-gated and excluded from default builds. Actual performance measurements and backend selection remain outstanding. Sparse global FEM solving remains out of scope for this stage.

### Stage 4 — Integration contract

- [ ] Publish Rust API and serialization contracts for Jabr-UI.
- [ ] Define adapter boundary for Jabr-math formula evaluation.
- [ ] Keep persistence/collaboration ownership with Jabr-UI/BlockSuite.
- [ ] Add cross-repository contract fixtures and compatibility tests.

**Exit criteria:** Jabr-UI can consume table data operations without importing UI concerns into this repository.

## Non-goals

- No standalone UI, web app, or table/database editor.
- No BlockSuite blocks, Yjs document ownership, or visual components.
- No full Excel clone, XLSX workbook fidelity, macros, or pivot tables.
- No SQL database engine.
- No formula language or expression parser duplicating Jabr-math.
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
