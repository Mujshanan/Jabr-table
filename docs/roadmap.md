# Jabr-table implementation roadmap

The order is deliberately Rust-first. Do not begin substantial BlockSuite UI work until the data model and Calcpad bridge have a tested vertical slice.

## Phase 0 — Repository and ownership
- [x] Define Jabr-table, Jabr-cli, Jabr-UI and CivilKit responsibilities.
- [x] Define typed scalar, Table, Vector, Matrix and List models.
- [x] Define explicit conversion and diagnostic rules.
- [x] Identify AFFiNE/BlockSuite table implementation as the grid reference.
- [x] Separate custom block implementation into Jabr-UI.

## Phase 1 — Rust core correctness
- [x] Create Cargo workspace and jabr-table-core.
- [x] Add tagged scalar values and stable error structures.
- [x] Add Table, Vector, Matrix and List models.
- [x] Add structural validation for matrix shape and table rows.
- [x] Add initial Matrix ↔ Table conversions.
- [ ] Add full serde round-trip tests and schema-version guards.
- [ ] Cover zero-sized shapes, empty labels, mixed values, selected columns, duplicate IDs and conversion identity.
- [ ] Make fmt, Clippy and test CI green.

## Phase 2 — Calcpad / Jabr-cli bridge (highest priority)
- [ ] Inspect the current Jabr-cli Calcpad engine runtime and Rust build boundary.
- [ ] Decide direct Rust dependency or versioned worker/RPC transport from the actual runtime; do not guess or introduce FFI/WASM prematurely.
- [ ] Define versioned request/response DTOs, capabilities and stable error mapping.
- [ ] Bridge Table, Matrix, Vector and List values without losing exact decimal data or unit metadata.
- [ ] Dispatch matrix/table conversion operations through the Calcpad worksheet engine.
- [ ] Add direct-Rust versus worksheet-dispatch parity tests.
- [ ] Confirm CivilKit can consume the stable Rust API without UI dependencies.

## Phase 3 — Engineering data operations and spreadsheet functions
- [ ] Add typed range abstraction and indexing/slicing.
- [ ] Implement aggregation and numeric/logical functions.
- [ ] Implement conditional aggregation and lookup functions.
- [ ] Implement matrix transpose, multiplication, determinant and inverse with documented numerical behavior.
- [ ] Publish a function compatibility table and error semantics.
- [ ] Preserve quantity/unit metadata through the Calcpad engine contract.

## Phase 4 — Data I/O
- [ ] CSV staging import/export with explicit parsing policies.
- [ ] XLSX workbook/sheet/cell import and safe export.
- [ ] Preserve formula source where supported without duplicating formula evaluation.
- [ ] Add round-trip, malformed and oversized-input tests.

## Phase 5 — Minimal BlockSuite custom blocks (Jabr-UI only)
- [ ] Publish stable TypeScript contracts and fixtures from Jabr-table.
- [ ] Adapt the BlockSuite affine:table row/column/cell data pattern into a Jabr-owned table block.
- [ ] Reuse the same minimal grid interaction for matrix and one-axis vector blocks.
- [ ] Add a custom typed list block only where native list/paragraph blocks cannot represent the data.
- [ ] Implement paste, conversion and import/export commands.
- [ ] Verify Yjs persistence, undo/redo and collaboration-safe authored operations.
- [ ] Keep toolbar and visual styling minimal; no spreadsheet application shell.

## First complete vertical slice

1. Rust Table and Matrix models with stable serialization.
2. Matrix ↔ Table conversion with explicit header/label policy.
3. Calcpad/Jabr-cli bridge dispatching those operations.
4. Direct Rust and worksheet result parity tests.
5. A small Jabr-UI block prototype calling the bridge and undoing the conversion as one authored operation.

## Deferred

- Full Excel formula compatibility and VBA/macros.
- Pivot tables, charts, workbook tabs and formula bar.
- Elaborate cell styling, merged cells and advanced selection overlays.
- Dedicated WASM/C ABI unless the runtime analysis demonstrates that it is required.
- Broad custom block UI before the engine bridge is stable.
