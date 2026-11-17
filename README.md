# Jabr-table

**Jabr-table is the Rust data and calculation subsystem for Jabr's engineering worksheet.** It owns typed tables, vectors, matrices, lists, conversions, spreadsheet-style data functions, and CSV/XLSX I/O. Its first priority is a reliable Rust calculation core and a stable bridge to the Calcpad-based worksheet engine—not a large spreadsheet UI.

## Repository boundaries

| Repository / layer | Responsibility |
|---|---|
| **Jabr-table (this repository)** | Rust data model, validation, conversions, range and matrix operations, spreadsheet-compatible functions, CSV/XLSX I/O, versioned bridge contract |
| **Jabr-cli** | Calcpad/Rust worksheet engine integration, expression/AST dispatch, variables, scopes, dependency tracking, units, calculation orchestration |
| **Jabr-UI** | Separate BlockSuite custom-block implementation, editing interactions, menus, selection, rendering, Yjs persistence |
| **CivilKit / engineering consumers** | Consume the stable calculation/data bridge; no UI or BlockSuite dependency |

Jabr-table must remain usable as a standalone Rust library. Jabr-UI must not implement calculations or file-format semantics. Jabr-cli must not fork data operations already implemented here. Keep the bridge narrow and versioned.

## Rust-first scope

1. Typed values and stable error semantics.
2. Tables, vectors, matrices, and heterogeneous lists.
3. Loss-aware conversions between those structures.
4. Range operations and spreadsheet-compatible functions needed by engineering worksheets.
5. CSV and XLSX import/export with explicit safety and compatibility policies.
6. A versioned Rust-facing API and serialized bridge for Jabr-cli and downstream engineering consumers.

This is **not** a promise of full Microsoft Excel compatibility. Implement and test individual functions, documenting supported argument forms and differences.

## BlockSuite integration

The custom BlockSuite UI belongs in [Jabr-UI](https://github.com/Mujshanan/Jabr-UI), not here. Jabr-table publishes the data contract and operations that those blocks call.

The first UI block candidates are:
- jabr:table — adapt the data model and interaction pattern of BlockSuite's existing affine:table as a reference, but keep Jabr's own schema and cell-value model.
- jabr:matrix — compact rectangular numeric/data grid with shape-aware operations and matrix ↔ table conversion.
- jabr:vector — one-dimensional sequence with explicit row/column orientation and optional engineering units.
- jabr:list — ordered heterogeneous values; render as a minimal list and convert explicitly to vector/table.

The existing jabr:math block in Jabr-UI remains the worksheet expression entry point. It should request evaluation through the worksheet controller and bridge; it must not contain the Rust engine or duplicate formula evaluation.

Keep the UI deliberately small: grid editing, row/column insertion, paste, import/export, conversion, and basic error display. No pivot tables, charts, formula bar, full workbook shell, or elaborate toolbar in the first milestones.

## Current implementation

- Versioned TypeScript wire contract.
- Rust workspace and jabr-table-core crate.
- Tagged scalar values plus Table, Vector, Matrix, and List models.
- Structural validation for table and matrix shape.
- Initial Matrix ↔ Table conversions with explicit header options and label preservation.
- Rust CI for formatting, Clippy, and tests.

The implementation is an early foundation, not yet a complete Calcpad bridge, spreadsheet function library, or Excel-compatible workbook engine.

## Development

Start with [development.md](./development.md) for the Rust-first milestones, bridge contract, BlockSuite reuse findings, testing strategy, and repository ownership rules. See [docs/architecture.md](docs/architecture.md) for the detailed model and conversion contract and [docs/roadmap.md](docs/roadmap.md) for the staged backlog.
