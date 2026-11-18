# Jabr-table — Development Guide

> **Architecture baseline (2026-11-18):** Table owns table structure, stable table/row/column/cell identity, authored cell text and table metadata. It does not parse cell expressions, evaluate formulas, or store Math evaluation values. Jabr Math parses and evaluates cell text. Jabr-engine (planned; repository not yet created) chooses graph nodes (including table columns where appropriate), builds dependencies, orchestrates calculation, and owns ResultStore<MathValue>. Kernel provides only generic infrastructure; Table must not access Kernel result storage. Jabr-UI owns the custom blocks and authored BlockSuite/Yjs document.

> **Existing-model audit:** the current jabr-table-core uses CellValue::{Empty, Text, Number, Boolean}. This is a real existing implementation that must be reconciled with the authored-text contract. Keep typed import/export and pure table-operation representations separate from the canonical authored worksheet source. Do not remove working interchange or matrix operations as part of a documentation-only change, and do not convert Math values into Table-owned values.

## 1. Product direction

Build the **Rust table/data-operation layer first**. Jabr-table is not the place to build a spreadsheet application or a rich BlockSuite editor. Its job is to provide reliable, deterministic table structures and pure data operations for Jabr-engine and downstream engineering consumers such as CivilKit.

The visual editor is a separate project: [Mujshanan/Jabr-UI](https://github.com/Mujshanan/Jabr-UI). Keep this repository free of BlockSuite, Lit, DOM, Yjs, MathLive, and editor dependencies.

## 2. Repository ownership and call direction

    Jabr-UI (BlockSuite UI; authored document state)
        | versioned requests / source revisions
        v
    Jabr-engine (worksheet graph and calculation orchestration)
        |------> Jabr-math (parse, AST, evaluate, units)
        `------> Jabr-table (table text, structure, data operations)

**Ownership rules**

- Jabr-table owns table structure, stable identities, authored cell text, metadata, import/export and table-specific operations.
- Jabr-math owns worksheet syntax, AST, mathematical values, units, dimensional analysis and expression evaluation.
- Jabr-engine owns worksheet graph construction, symbol binding, dependency resolution, recalculation ordering, scheduling and the lifecycle of Math results stored in Kernel's generic ResultStore<V>.
- Jabr-kernel owns generic NodeId/Port, Revision, Snapshot<T>, ResultStore<V>, graph primitives, diagnostics and spans; it does not own variables or Math values.
- Jabr-UI owns BlockSuite schemas/views, keyboard and selection behavior, menus, paste interaction, authored document persistence and collaboration.
- CivilKit and other engineering consumers may consume stable Rust APIs; they must not need BlockSuite or a browser.
- Never implement the same numerical/data operation independently in UI and CLI.

The normal product direction is UI → Engine → Math/Table/Kernel. Engine selects the appropriate Math and Table APIs; Table never calls back into Math, Engine, or the editor.

## 3. Rust workspace design

Current core crate: crates/jabr-table-core.

Planned crate split:
- jabr-table-core: canonical models, validation, conversions, range primitives, deterministic operations.
- jabr-table-functions: spreadsheet-compatible and matrix functions, depending on core.
- jabr-table-io: CSV/XLSX parsing and export; owns file bytes and format-specific dependencies.
- jabr-table-bridge: stable DTO/ABI adapter only if required by Jabr-engine's selected integration mode.

Do not split crates prematurely. Keep the first vertical slice in jabr-table-core; extract functions, IO, or bridge crates when the public APIs and dependency sets justify it.

### Data model rules

- Use tagged scalar values; do not store arbitrary JS values.
- Preserve authored decimal values as canonical decimal strings or exact decimal/rational representation. Do not silently round-trip through f64.
- Matrices are rectangular, row-major, and validate values.len() == rows * columns.
- Vectors are one-dimensional and carry explicit row/column orientation.
- Lists may be heterogeneous and nested within a documented depth limit.
- Tables have stable row and column IDs; cell order is defined by the ordered column list.
- Errors are values with stable codes and structured diagnostics, not empty cells.
- Units and quantities remain owned by Jabr Math. Table operations may accept plain numeric/data inputs but must not create a competing quantity or unit system.

## 4. Calcpad engine bridge

The table data-operation contract is the highest-priority integration boundary. Engine is the worksheet orchestrator; Math owns syntax and evaluation; Table owns the reusable data operations. Keep the contract small, deterministic, versioned, and independent of the UI.

### What the bridge carries

- Tagged scalar values and structured data payloads (table/vector/matrix/list).
- Operation/function identifier and typed arguments.
- Stable error code plus diagnostic context.
- Schema/API version and capability information.
- Optional source provenance (worksheet block ID, range or import origin) as metadata—not as a calculation dependency.

### What stays in Jabr Math and Jabr-engine

**Jabr Math** owns parsing worksheet expressions and function-call syntax, AST construction, mathematical values, quantities, units, dimensions, conversions and expression evaluation. It does not own worksheet-wide graph construction or recalculation scheduling.

**Jabr-engine** owns worksheet node/port mapping, symbol binding across blocks and table columns, dependency graph construction, recalculation ordering, stale-result protection, scheduling, and the lifecycle of results held in Kernel's generic ResultStore<V>. Engine dispatches supported AST-level table operations to Jabr-table; Table never sees Math result values or parses authored cell text.

### Integration strategy

1. First expose idiomatic Rust APIs from jabr-table-core; keep them callable in-process by Rust consumers.
2. Inspect Jabr-engine's chosen integration with the Jabr Math facade before selecting transport.
3. Prefer a direct Rust dependency when the independent repositories can be consumed in one compatible Rust process without creating a shared workspace.
4. If Engine is a separate process or language boundary, use a versioned JSON request/response protocol over the selected worker/RPC boundary. Avoid a premature C ABI or WASM layer.
5. Add a thin adapter in Jabr-engine. It maps Math AST function calls and Engine-owned data to Jabr-table APIs; it does not reimplement table operations.
6. Prove parity: direct Rust call and Engine-dispatched call must produce the same value/error for the same inputs.

Do not choose FFI, WASM, JSON-RPC, or a subprocess until the Jabr-engine and Jabr Math runtime/build layouts have been inspected. The bridge contract should not depend on the transport.

### First bridge vertical slice

- Matrix and Table cross the bridge with schema version and exact scalar values.
- Matrix → Table and Table → Matrix are callable operations.
- Invalid dimensions, unknown schema versions, invalid column indices, and unsupported coercions return structured errors.
- A Jabr-engine integration test dispatches a worksheet function to the same Rust conversion implementation.
- A standalone Rust test calls the function directly and compares output.

## 5. Spreadsheet and engineering function priorities

Implement functions as pure Rust operations over explicit data/range inputs. Math parses syntax and exposes the AST call; Engine dispatches the operation and manages its result.

Suggested order:
1. Range primitives: shape, indexing, slicing, flattening, transpose.
2. Aggregation: SUM, AVERAGE, MIN, MAX, COUNT, COUNTA.
3. Numeric and logical: ABS, ROUND, MOD, POWER, SQRT, IF, AND, OR, NOT.
4. Conditional aggregation: SUMIF, COUNTIF, AVERAGEIF.
5. Lookup: INDEX, MATCH; then exact-match XLOOKUP.
6. Text helpers required for imported engineering schedules.
7. Matrix functions: MMULT, MINVERSE, MDETERM.
8. Dynamic-array helpers: FILTER, UNIQUE, SORT.

Engineering priorities:
- Exact decimal handling where practical.
- Clear distinction between empty, zero, text, and error.
- Unit-aware values follow the Jabr Math quantity contract; Table does not own unit semantics.
- Deterministic results and documented numerical tolerances.
- Explicit shape/dimension diagnostics.
- No silent coercion of text to numbers unless the function's policy says so.

Excel compatibility is function-by-function. Macros, VBA, external workbook links, scripts, and arbitrary workbook execution are out of scope.

## 6. Data I/O

CSV first, XLSX second.

CSV:
- Explicit delimiter, encoding, quote/escape, header, decimal and date policies.
- Import to a staging payload and return diagnostics before the caller commits it.
- Preserve text values and avoid guessing dates/numbers without an explicit import policy.

XLSX:
- Preserve sheet names, row/column order, cell types and formula source where supported.
- Never execute macros or external links.
- Document unsupported formula features, formatting, named ranges and workbook constructs.
- Cached formula results are not authoritative unless generated by a compatible calculation pass.

Both paths must enforce configurable limits for bytes, rows, columns, formula length, and nesting depth.

## 7. BlockSuite research and custom-block targets

The custom UI implementation belongs in **Jabr-UI**. Jabr-table should provide contracts, fixtures and engine operations only.

### Existing BlockSuite implementation worth adapting

The strongest reference is BlockSuite/AFFiNE's existing **Table block**:
- Model/schema: packages/affine/model/src/blocks/table/table-model.ts
- View and grid rendering: packages/affine/blocks/table/src/table-block.ts
- Data mutation/ordering: packages/affine/blocks/table/src/table-data-manager.ts
- Registration: packages/affine/blocks/table/src/store.ts

Useful patterns to study/adapt:
- Stable row and column IDs.
- Explicit row/column ordering rather than relying on object insertion order.
- Cell lookup by row ID + column ID.
- A separate data manager between view and persisted model.
- Transactional row/column/cell mutations.
- Selection and grid interaction separated from the data model.

Do not copy AFFiNE's complete table implementation or its dependencies wholesale. It includes selection layers, virtual padding, resize affordances, comments, captioning and editor-specific services that Jabr does not need initially. Do not store the Jabr Rust engine's canonical data as one opaque JSON string inside a BlockSuite block.

### Jabr block mapping (implement in Jabr-UI)

| Jabr block | BlockSuite reference | Recommendation |
|---|---|---|
| jabr:table | affine:table | Adapt its row/column/cell model and minimal data-manager pattern. Keep Jabr schema, stable IDs and typed cell values. |
| jabr:matrix | affine:table grid rendering | Reuse the simple grid interaction, but add fixed rectangular shape and numeric/data validation. No separate matrix UI framework. |
| jabr:vector | affine:table single-axis grid | Render as one row or one column, preserving orientation. Keep vector as a distinct data type in the engine. |
| jabr:list | BlockSuite list/paragraph blocks | Prefer native list/paragraph rendering for simple ordered text; create a Jabr custom list block only when heterogeneous/nested typed values or conversion actions are required. |

The current Jabr-UI prototype has jabr:root, jabr:worksheet, jabr:math, and jabr:text. The math block is the correct place for the worksheet expression trigger, but its evaluation request must go through the controller/bridge. Add data blocks to the worksheet's allowed children only in Jabr-UI.

### Minimum viable block UI

Keep only:
- Editable cells and keyboard navigation.
- Add/remove row and column.
- Paste rectangular tabular data.
- Convert matrix ↔ table and vector ↔ table.
- Import/export action.
- Minimal error/validation indication.

Defer formula bars, pivot tables, charts, filters UI, elaborate formatting, column type menus, custom selection overlays, and workbook tabs. The Rust engine and bridge should be validated before polishing these views.

## 8. Development sequence

### Milestone 1 — Rust model and conversion correctness
- [x] Establish workspace and core typed models.
- [x] Add structural validation for matrix and table.
- [x] Add matrix ↔ table conversion baseline.
- [ ] Test zero-width/zero-height shapes, empty labels, mixed values, selected columns and round-trip identity.
- [ ] Add serialization round-trip and unsupported schema tests.
- [ ] Run CI and fix all format/lint/test failures.

### Milestone 2 — Math/Engine table-operation contract
- [ ] Inspect Jabr-math's public facade and Jabr-engine integration design.
- [ ] Keep table-cell authored text separate from Math evaluation values.
- [ ] Define how Engine dispatches supported AST-level table operations to Jabr-table.
- [ ] Freeze versioned data-operation DTOs and stable error codes.
- [ ] Add direct Rust and Engine-dispatch parity tests.

### Milestone 3 — Core range and spreadsheet functions
- [ ] Add typed range abstraction.
- [ ] Implement first aggregation/logical/numeric functions.
- [ ] Implement matrix transpose and matrix arithmetic functions.
- [ ] Publish supported-function compatibility table.

### Milestone 4 — CSV/XLSX I/O
- [ ] CSV staging import/export and diagnostics.
- [ ] XLSX workbook/sheet/cell parsing and safe export.
- [ ] Round-trip and malformed/oversized input tests.

### Milestone 5 — Jabr-UI integration
- [ ] Publish stable TypeScript contracts and Rust fixtures.
- [ ] Build minimal table/matrix blocks in Jabr-UI.
- [ ] Add vector and list blocks only after data model and bridge behavior is stable.
- [ ] Test Yjs persistence, undo/redo, clipboard and conversion.
- [ ] Keep UI work in Jabr-UI; do not add BlockSuite dependencies here.

## 9. Local development

Requirements: stable Rust toolchain with Cargo.

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace --all-targets

When additional crates are added, run the same workspace checks and include fixture-based tests for all bridge-visible operations.
