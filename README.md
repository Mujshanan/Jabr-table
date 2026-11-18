# Jabr-table

Jabr-table is the Rust data and computation engine for engineering tables in the Jabr ecosystem.

- **Jabr-table:** typed table model, data operations, interchange, and matrix utilities.
- **Jabr-UI:** all UI, BlockSuite presentation, and final product integration.
- **Jabr-CLI:** calculation language, parsing, evaluation, and engineering calculation semantics.

This repository does not contain a standalone table editor or UI.

## Development

See [development.md](./development.md) for the staged PoC-to-MVP plan.

The first milestone is a minimal, tested Rust table core. Formula parsing, UI, collaboration, and FEM solving are outside the initial PoC.

## Matrix and XML engine workbench

A companion input harness is available at [engine-demo/index.html](./engine-demo/index.html). It exposes sample table data, matrix inputs and an XML preview for engine development. It does not implement formulas, matrix operations or XML parsing; the production table/database UI belongs in Jabr-UI.
