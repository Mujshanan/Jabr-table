# Jabr-table Development Plan

## Product direction

Jabr-table is the engineering data-table and database presentation layer in the Jabr ecosystem. It is developed independently and will later integrate with Jabr-UI / BlockSuite and Jabr-CLI.

The first release is **UI-first**. Establish a clear, usable table and database experience before introducing formula evaluation, numerical matrix computation, or FEM workflows.

### Product surfaces

1. **Engineering Table** — a spreadsheet-like grid for direct cell editing, rectangular selection, keyboard navigation, copy/paste, and engineering data entry.
2. **Database Table** — a structured record view with named columns, data types, sorting, filtering/search, and record management.

These are two presentations over a shared table model, not separate data systems.

## Principles and boundaries

- Keep the table model independent of the UI framework and BlockSuite.
- Keep Jabr-table independently developable; integrate through explicit adapters later.
- BlockSuite/Yjs will own persisted collaborative document state after integration.
- Do not let a formula library own the Jabr document or table model.
- Do not store large FEM global stiffness matrices as millions of collaborative table cells.
- Use stable row and column IDs. Do not use mutable row/column positions as durable identity.
- Keep imported source/provenance metadata separate from cell values.
- Make bulk paste/import a single logical undoable operation after document integration.
- Keep the initial UI focused; avoid building an Excel clone or a full database application.

## Technology direction

The initial UI prototype follows the Jabr-UI toolchain for compatibility:

- TypeScript
- Vite
- Lit
- pnpm 10.34.5
- Node.js `^20.19.0 || >=22.12.0`

This prototype deliberately uses local in-memory sample data. It does not yet depend on BlockSuite/Yjs, Formualizer, faer, or Jabr-CLI.

## Development stages

### Stage 0 — UI foundation (current)

**Goal:** establish the visual language, shared table presentation, and basic interactions.

- [x] Scaffold a standalone TypeScript/Lit/Vite application.
- [x] Define Engineering Table and Database Table view modes.
- [x] Add sample engineering section records.
- [x] Add cell editing, row insertion, column insertion, search, sorting, CSV export, and CSV import.
- [ ] Validate the UI in Chromium and refine keyboard/pointer interactions.
- [ ] Add automated UI tests for core table interactions.
- [ ] Review the visual design before introducing BlockSuite.

**Exit criteria:** both views are usable against the same sample dataset; core interactions work without a backend; the app builds and typechecks.

### Stage 1 — Table interaction PoC

- Define a framework-independent table schema: table, columns, rows, cells, cell values, and data types.
- Support stable row/column identifiers.
- Add rectangular range selection and keyboard navigation.
- Add TSV clipboard copy/paste.
- Add robust CSV parsing, quoted fields, headers, empty values, and type inference preview.
- Add JSON import/export.
- Add XML import/export with configurable record element and field mapping.
- Preserve unmapped XML attributes/nested content where round-trip fidelity is required.
- Add unit tests for data model, import/export, and selection behavior.

**Exit criteria:** table interactions and data exchange work independently of BlockSuite.

### Stage 2 — Formula PoC

Evaluate Formualizer behind a Jabr-owned formula adapter.

Initial functions:
- `VLOOKUP`
- `HLOOKUP`
- `MATCH`
- `MAX`
- `MIN`

Validate exact/approximate matching, missing values, duplicate matches, text/numeric identifiers, empty cells, invalid ranges, dependency updates, and recalculation.

Formualizer is an evaluation candidate only. Jabr-table retains ownership of the table model and authored formulas.

### Stage 3 — Matrix PoC

Evaluate `faer` as the primary numerical backend candidate, with `nalgebra` as a possible complement for small fixed-size geometry and element matrices.

Initial operations:
- Matrix addition and subtraction
- Matrix multiplication
- Scalar multiplication
- Transpose
- Matrix-vector multiplication

Start with small dense matrices. Validate dimensions, errors, precision, and results against independent fixtures.

### Stage 4 — Engineering/FEM demonstration

Use a small, independently verifiable 2D truss example:

- Enter nodes, elements, material properties, and section properties.
- Display element stiffness matrices.
- Assemble a small global stiffness matrix.
- Display load and displacement vectors.
- Compare results against independently verified references.

This is a data and matrix workflow demonstration, not a complete FEM solver product.

### Stage 5 — MVP document integration

- Add a BlockSuite table block through a dedicated adapter.
- Persist table structure and authored cell values in BlockSuite/Yjs.
- Add undo/redo, collaboration, and persistence tests.
- Keep formula results and numerical result caches outside authored collaborative state unless explicitly designed otherwise.
- Preserve row/column identity through supported reorder and insert/delete operations.
- Integrate with Jabr-UI through a documented component contract.

### Stage 6 — MVP data exchange and hardening

- CSV import/export
- TSV and clipboard import/export
- JSON import/export
- XML import/export and mapping preview
- Validation for malformed and oversized inputs
- Accessibility and keyboard interaction checks
- Performance tests with realistic engineering datasets
- Integration tests across Jabr-table, Jabr-UI, and Jabr-CLI boundaries

## MVP scope

### Included

- Engineering Table and Database Table presentations
- Typed columns and editable records
- Search, sorting, and basic filtering
- Stable row and column identity
- Formula support for the five initial functions
- Small dense matrix editing and operations
- CSV, TSV, JSON, XML, and clipboard exchange
- BlockSuite/Yjs persistence and collaboration through an adapter
- Matrix inspection for selected engineering/FEM data

### Explicitly deferred

- Full Excel compatibility and XLSX workbook fidelity
- VBA/macros, pivot tables, and advanced charting
- Relational database engine and SQL
- Full FEM solver and mesh generation
- Large sparse matrix editing as ordinary table cells
- Advanced formula library coverage
- External data synchronization and live database connectors

## Initial repository layout (target)

```text
src/
  app/
  table-core/
  table-ui/
  database-ui/
  data-io/
  adapters/
tests/
  unit/
  e2e/
```

Do not create every package before it is needed. Extract modules when the interfaces are stable and the separation provides real value.

## Current UI milestone checklist

- [x] Standalone UI scaffold
- [x] Shared sample dataset for both views
- [x] Engineering Table grid
- [x] Database Table presentation
- [x] Inline cell editing
- [x] Add row and column
- [x] Search and column sorting
- [x] CSV import/export
- [ ] Chromium E2E validation
- [ ] TSV clipboard handling
- [ ] Column type editing and filter controls
- [ ] Accessibility review

## Development commands

```sh
pnpm install
pnpm dev
pnpm typecheck
pnpm build
```

## Current limitations

The first UI prototype uses in-memory data and local CSV import/export. It is not a persistent database, does not yet support collaboration, does not evaluate formulas, and does not perform matrix or FEM calculations. Those capabilities are staged above and must not be implied by the UI.
