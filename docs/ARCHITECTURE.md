# Jabr Table Architecture

**Status:** Current mainline architecture baseline; 2026-11-18.

## Responsibility

Jabr Table is a UI-independent Rust library for structured tabular data, pure data operations, interchange and small dense matrix/vector operations.

It does not own worksheet parsing, mathematical evaluation, cross-block dependency scheduling, UI, or document persistence.

## Ownership boundaries

| Concern | Owner |
|---|---|
| Table/row/column identity, typed tabular data and table operations | Jabr Table |
| Authored BlockSuite blocks, selection/editing, Yjs persistence and collaboration | Jabr-UI |
| Mathematical syntax, parsing, AST, values, quantities, units and evaluation | Jabr Math |
| Worksheet graph construction, symbol binding across blocks/tables, scheduling, recalculation and result lifecycle | Jabr-engine (planned) |
| Generic IDs/ports, revisions, snapshots, opaque result storage, graph primitives, diagnostics and spans | Jabr-kernel |

Dependency direction is through public APIs and adapters. Jabr Table must not depend on Math, Engine, Kernel, BlockSuite, Yjs or browser APIs. Jabr-engine composes the engines; it does not reimplement Table operations or Math semantics.

## Current data model

The current `jabr-table-core` model includes:

- `Table` with a stable table ID, ordered columns and rows.
- Stable row and column IDs.
- Row cells keyed by column ID rather than display position.
- `CellValue::{Empty, Text, Number(f64), Boolean}`.
- A column type contract and validation errors.

The current `CellValue` is a typed data value. It is not a formula AST, Math value, unit-aware quantity, or complete authored formula-source representation. Table must not parse the text variant as a formula. A dedicated authored source/formula-cell model should be designed with Jabr-UI and Jabr-engine before any such behavior is added.

The current numeric representation uses `f64`. Exact decimal/rational representation, rounding rules and JSON precision guarantees are open model/API decisions; do not silently promise exact arithmetic.

## Current modules

- `functions`: standalone lookup and aggregation functions. No parser or dependency graph.
- `interchange`: CSV/TSV, JSON and XML adapters with explicit options and validation.
- `matrix`: validated dense row-major matrix operations for small element-level calculations.

The dense matrix API is not a sparse global FEM solver and does not own mesh, boundary-condition, solver or structural-design semantics.

## Integration boundary

Jabr-engine may call Table operations through a typed Rust API or a future transport adapter. The transport choice (direct Rust dependency, WASM, FFI or RPC) must be selected only after the Engine and Math build/runtime boundaries are inspected.

Requests and responses crossing the UI boundary should carry request IDs and source/document revisions. Engine owns stale-response rejection and calculation-result lifecycle. Table operations should remain deterministic and independent of worksheet scheduling.

## Migration sequence

1. Preserve and test the current standalone data model and interchange operations.
2. Design a separate authored cell-source/formula representation without conflating it with typed imported data.
3. Define stable DTOs and serialization/versioning rules, including numeric precision.
4. Add cross-repository adapter tests from Jabr-engine.
5. Benchmark before selecting matrix backends or adding sparse FEM dependencies.
6. Keep UI integration in Jabr-UI; do not add editor frameworks to this repository.
