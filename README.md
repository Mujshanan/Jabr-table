# Jabr-table

Jabr-table is the data-structures and tabular-computation subsystem for Jabr Studio. It provides a reusable, editor-independent core for tables, lists, vectors, matrices, spreadsheet-style functions, and structured data import/export.

It is developed independently from Jabr-UI and Jabr-CLI. Jabr-UI provides the BlockSuite editing experience; Jabr-CLI remains the worksheet language/evaluation boundary and integrates the Rust data operations exposed here.

## Scope
- Structured tables with column schemas, headers, row identity, and typed cell values.
- Lists and typed vectors, including conversion and interoperability.
- Matrices with shape, element type, and explicit row/column semantics.
- Table ↔ matrix/vector/list conversions with documented loss/retention rules.
- Spreadsheet-style functions over ranges and arrays.
- CSV and Excel workbook import/export, with explicit type and formula policies.
- Rust implementation for deterministic data operations, validation, conversions, and spreadsheet-compatible functions.
- TypeScript contracts/adapters for BlockSuite blocks and worksheet integration.

## Architecture

```text
Jabr-UI (BlockSuite blocks and interactions)
          | versioned TS contract / commands
          v
Jabr-table (data model, conversions, IO, Rust data functions)
          | Rust API / adapter
          v
Jabr-CLI (worksheet language, AST, symbols, dependency graph, evaluation)
```

The Rust data operations are a reusable library, not a second worksheet language. Jabr-CLI remains authoritative for parsing worksheet expressions, variable scope, dependency tracking, units, and calculation orchestration. Jabr-table owns typed data structures and operations on those values.

No BlockSuite, Yjs, AFFiNE, DOM, or MathLive dependency belongs in the core Rust crate.

## Development status
The repository is being bootstrapped. The first milestone defines the data model and cross-repository contracts before implementing editor widgets or broad Excel compatibility.

See [Architecture](docs/architecture.md) and [Roadmap](docs/roadmap.md).
