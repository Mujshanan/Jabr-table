# Jabr Table

A focused data-grid and matrix workbench for developing Jabr's table engine independently from the main worksheet UI.

## Run the UI prototype

The current prototype is dependency-free. Open index.html directly in a browser or serve this directory with any static file server.

It provides:

- An editable engineering-style sample table
- Cell selection, address and value inspection
- Add-row and add-column controls
- Basic CSV import and export
- A matrix input surface with editable dimensions
- An XML preview generated from the sample table
- A small function roadmap for lookup, matching, extrema and matrix operations

The prototype is a UI harness only. It does not evaluate formulas, perform VLOOKUP/HLOOKUP/MATCH, execute matrix algebra, validate XML schemas or persist data.

## Responsibility boundary

- **Jabr Table** owns tabular data structures, typed columns, formula/lookup execution, matrix operations, import/export and XML parsing/querying.
- **Jabr Table UI prototype** exists only to provide representative engine inputs and inspect results as those engine APIs are developed.
- **Jabr-UI** owns the production table/database blocks, worksheet-level presentation, shared document state and cross-component integration.

Keep the engine API independent of this prototype. Jabr-UI should integrate the stable table engine through an adapter and provide the unified production experience.

## Engineering use cases

The UI intentionally exposes representative data shapes for structural calculations. The engine roadmap includes lookups (VLOOKUP, HLOOKUP, MATCH), extrema (MAX, MIN), matrix/vector operations used by FEM workflows, and XML data ingestion. These are targets for the engine, not capabilities claimed by this prototype.
