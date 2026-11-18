# Jabr-table

Jabr-table is the Rust data and computation engine for engineering tables in the Jabr ecosystem.

- **Jabr-table:** typed table model, standalone data functions, interchange adapters, and small dense matrix utilities.
- **Jabr-UI:** all UI, BlockSuite presentation, and final product integration.
- **Jabr-CLI:** calculation language, parsing, evaluation, and engineering calculation semantics.

This repository does not contain a standalone table editor, UI, or formula parser.

## Current engine capabilities

- Typed cells, stable row/column IDs, and table CRUD.
- Exact-match MATCH, VLOOKUP, and HLOOKUP; numeric MAX and MIN.
- CSV/TSV import and export with quoted-field support and basic type inference.
- JSON import/export preserving the native table representation.
- XML import with configurable record elements and field mapping; canonical XML export.
- Dense row-major matrices: addition, subtraction, multiplication, transpose, and matrix-vector multiplication.

The interchange and matrix APIs are in the `jabr-table-core` crate under the `interchange` and `matrix` modules. These are engine APIs only; presentation and editing remain in Jabr-UI.

## FEM boundary

The dense matrix module targets small element-level calculations and fixtures. It is not a sparse global assembly or FEM solver. Meshes, boundary conditions, global sparse assembly, solving, and structural design semantics remain outside this crate's current scope.

## Development

See [development.md](./development.md) for the staged PoC-to-MVP plan, remaining hardening work, and integration boundaries.
