# Jabr Table

Jabr Table is the independent Rust data subsystem for engineering worksheet tables, lookup operations, interchange and small dense matrix utilities. It is a library, not a spreadsheet application or a formula evaluator.

## Jabr ecosystem ownership

| Component | Responsibility |
|---|---|
| **Jabr Table** | Table/row/column identity, typed data structures, deterministic data operations, CSV/TSV/JSON/XML interchange, and small dense matrix/vector primitives |
| **Jabr Math** (`Mujshanan/Jabr-math`) | Mathematical syntax, parsing, AST, mathematical values, quantities, units, dimensional rules and expression evaluation |
| **Jabr-engine** (planned; repository not yet created) | Worksheet node/port mapping, cross-block bindings, graph construction, scheduling/recalculation orchestration and calculation-result lifecycle |
| **Jabr-kernel** | Domain-neutral NodeId/Port, Revision, Snapshot<T>, ResultStore<V>, generic graph, diagnostics, spans and envelopes |
| **Jabr-UI** | BlockSuite custom blocks, editing interactions, authored Yjs document and presentation |

Jabr Table remains independently buildable and must not depend on Math, Engine, Kernel, BlockSuite, Yjs or UI frameworks. Engine composes Table operations with Math evaluation through explicit adapters. Table never builds the worksheet dependency graph or owns calculation scheduling.

## Current implementation

- Stable table, row and column identifiers; cells are addressed by stable column IDs.
- Typed cell values and table validation/CRUD.
- Exact-match `MATCH`, `VLOOKUP`, and `HLOOKUP`; numeric `MAX` and `MIN`.
- CSV/TSV, JSON and XML import/export.
- Small row-major dense matrix operations for element-level engineering/FEM calculations.

The current `CellValue` model is a typed data representation (`Empty`, `Text`, `Number(f64)`, `Boolean`). It is not a formula language or a complete authored-cell-source contract. Do not parse expressions or treat a text literal as an evaluated Math value. A dedicated authored formula/source representation and exact numeric interchange policy require a separate reviewed model change.

## FEM boundary

The dense matrix module targets small element-level calculations and fixtures. It is not a sparse global assembly or FEM solver. Meshes, boundary conditions, global sparse assembly, solving and structural design semantics remain outside this crate's current scope.

## Development and validation

See [development.md](./development.md) for architecture, milestones and validation commands.

The workspace declares Rust 1.85 as its minimum supported version. Run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```
