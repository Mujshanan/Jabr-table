# Jabr-table

Jabr-table is the engineering data-table and database presentation layer in the Jabr ecosystem. It is developed independently and will later integrate with Jabr-UI / BlockSuite and Jabr-CLI.

The first milestone is UI-first:

- **Engineering Table:** spreadsheet-like cell editing and engineering data entry.
- **Database Table:** typed columns, structured records, search, and sorting.

The current prototype uses in-memory sample data. It does not yet provide persistence, collaboration, formula evaluation, matrix calculations, or FEM analysis.

## Requirements

- Node.js `^20.19.0 || >=22.12.0`
- pnpm `10.34.5`

## Run

```sh
pnpm install
pnpm dev
```

## Validate

```sh
pnpm typecheck
pnpm build
```

See [development.md](./development.md) for the PoC-to-MVP roadmap and scope.
