# Jabr-table implementation roadmap

## Phase 0 — Repository and contract bootstrap
- [x] Define ownership boundaries across Jabr-table, Jabr-UI, and Jabr-CLI.
- [x] Define canonical table, list, vector, matrix, and scalar value semantics.
- [x] Define conversion rules and loss/diagnostic policy.
- [x] Define spreadsheet function families and limit compatibility claims.
- [x] Define Rust workspace split and TypeScript integration contract.
- [ ] Validate the Rust adapter against the current Jabr-CLI repository layout and runtime.

## Phase 1 — Rust core data model
- [ ] Create Cargo workspace and `jabr-table-core`.
- [ ] Implement tagged scalar values and stable error types.
- [ ] Implement Table, Vector, Matrix, and List models with validation.
- [ ] Implement JSON/Serde versioned DTOs.
- [ ] Add shape, ordering, identity, and decimal-preservation tests.

## Phase 2 — Conversions and range primitives
- [ ] Implement table ↔ matrix conversions with explicit policies.
- [ ] Implement table ↔ vector conversions.
- [ ] Implement table ↔ list conversions.
- [ ] Implement matrix transpose and basic shape operations.
- [ ] Add round-trip and data-loss diagnostic tests.

## Phase 3 — Spreadsheet-compatible operations
- [ ] Implement typed range abstraction.
- [ ] Implement aggregation and logical functions.
- [ ] Implement conditional aggregation and lookup functions.
- [ ] Implement text and numeric functions.
- [ ] Implement matrix functions (MMULT, MINVERSE, MDETERM).
- [ ] Define compatibility matrix and error semantics.
- [ ] Add deterministic golden tests.

## Phase 4 — Data I/O
- [ ] Implement CSV import/export with explicit options and diagnostics.
- [ ] Implement XLSX workbook/sheet/cell import.
- [ ] Implement safe XLSX export and formula-source preservation policy.
- [ ] Add malformed, oversized, and adversarial input tests.
- [ ] Validate round trips and document unsupported workbook features.

## Phase 5 — Jabr-UI BlockSuite integration
- [ ] Add shared TypeScript contracts to Jabr-UI through a stable package/API boundary.
- [ ] Implement `jabr:table` block and grid interaction.
- [ ] Implement `jabr:vector`, `jabr:matrix`, and `jabr:list` blocks.
- [ ] Add matrix ↔ table and table ↔ vector/list conversion commands.
- [ ] Implement paste/import/export flows and keyboard ownership.
- [ ] Test undo/redo, Yjs persistence, collaboration-safe authored operations, and browser interactions.

## Phase 6 — Jabr-CLI Rust engine integration
- [ ] Add a thin CLI adapter dispatching AST function calls to Jabr-table Rust operations.
- [ ] Preserve CLI ownership of parsing, names, scopes, dependency graph, units, and recalculation.
- [ ] Add table/vector/matrix/list values to the CLI evaluation bridge.
- [ ] Add integration tests proving CLI and direct Rust calls produce identical results.
- [ ] Verify no duplicate function implementations are introduced.

## First vertical slice

1. Rust core models for Table and Matrix.
2. Matrix ↔ Table conversion with explicit header/blank/type policies.
3. JSON round-trip and shape validation.
4. Minimal TypeScript contract.
5. Jabr-UI matrix/table block prototype converting a small authored matrix into a table and undoing the conversion.

Do not begin with a full Excel clone or a large spreadsheet grid. Prove the data model, conversion semantics, persistence boundary, and Rust/UI contract first.
