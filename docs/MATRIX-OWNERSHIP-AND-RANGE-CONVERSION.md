# Jabr Matrix Ownership and Table-Range Conversion Contract

**Status:** Proposed architecture contract; integration sequence step 6.  
**Depends on:** CELL-ENGINE-INTEGRATION-CONTRACT.md (steps 1–4) and STRUCTURAL-OPERATIONS-REFERENCE-REWRITING.md (step 5).  
**Scope:** Matrix ownership, mathematical matrix values, conversion between Table ranges and Math matrices, validation, units, provenance, result projection and tests.  
**Non-goal:** This document does not implement matrix values in Math, a table formula parser, a worksheet scheduler, or a sparse global FEM solver.

## 1. Decision: Math owns matrices; Table owns grids

**Jabr Math owns matrix value semantics and matrix algebra. Jabr Table owns tabular storage and deterministic extraction/projection of rectangular grid data. Jabr-engine owns the conversion adapter, bindings, evaluation order and result lifecycle.**

| Concern | Owner |
|---|---|
| Matrix value type, shape semantics, scalar element semantics, units/dimensions, algebra, indexing/slicing, matrix functions and errors | Jabr Math |
| Tables, rows, columns, cells, stable identity, rectangular range selection, row-major data extraction, neutral range snapshots and display/export projection | Jabr Table |
| Resolving references, evaluating formula cells before conversion, converting neutral range snapshots into Math values, invoking Math operations, mapping results back to output projections and graph invalidation | Jabr-engine |
| Generic graph nodes/ports, revisions, snapshots, diagnostics, spans and opaque result storage | jabr-kernel-api / Kernel |
| Authored matrix/table blocks, range selection UI, rendering, collaboration and authored persistence | Jabr-UI |

Dependency direction is mandatory:

- Jabr Math MUST NOT depend on Jabr Table.
- Jabr Table MUST NOT depend on Jabr Math or Kernel.
- Engine is the only composition layer between them.
- A neutral DTO/wire contract may be shared, but it must not import either engine's implementation types.

A grid range is not inherently a matrix. Conversion is an explicit, validated operation requested by Engine or an authored Math operation.

## 2. Mathematical matrix model owned by Math

Math defines a first-class matrix value as a rectangular, row-major collection of Math scalar values. The conceptual shape is:

    MatrixValue {
        rows: usize,
        columns: usize,
        elements: Vec<MathValue>,
    }

The concrete Rust type, serialization and public facade are owned by Jabr Math and must use its value/envelope contracts. This conceptual definition does not prescribe a public Rust signature.

Requirements:

1. Shape is explicit and independent of storage implementation.
2. Element count must equal rows multiplied by columns, using checked arithmetic.
3. Elements are stored in row-major order: index = row * columns + column.
4. Ragged matrices are invalid.
5. Zero-row or zero-column matrices are invalid for the initial worksheet matrix value, matching the current dense-matrix contract. Empty ranges remain valid Table selections but cannot be converted to a Math MatrixValue.
6. Matrix algebra is implemented by Math over Math-owned scalar values, not by Table over f64.
7. Math owns dimension compatibility, exactness, precision, overflow, unit propagation, and operation diagnostics.
8. Matrix operations must not silently discard units or convert exact values to f64.
9. Matrix identity and equality semantics are Math decisions; Table row/column/cell IDs are not matrix element identities.
10. Math may use a numerical backend internally for supported approximate numeric operations, but backend types must not leak through its public value contract.

### 2.1 Element and unit semantics

A matrix may contain Math values that include exact scalars, approximate scalars, quantities and units. Table does not infer or normalize these.

- Addition/subtraction require element-wise compatible shapes and Math-defined compatible element dimensions/units.
- Scalar multiplication/division and matrix multiplication follow Math's dimensional rules. For matrix multiplication, every product term contributing to a sum must be dimensionally compatible; Math must reject incompatible sums rather than discard units.
- Transpose preserves each element value and unit unchanged.
- Indexing/slicing returns Math values or matrices according to Math's shape rules.
- Determinant, inverse, decomposition and solver operations are added only when Math defines their numerical and dimensional constraints. This contract does not require them for initial integration.
- A matrix is not required to have one common unit for every element. Compatibility is checked by each operation, not by a blanket Table column type.

Matrix operations that cannot be represented safely by the current Math value system must return explicit unsupported/invalid outcomes. They must not silently coerce to f64.

## 3. Existing Jabr-table matrix module: migration decision

The current Jabr-table repository contains matrix::DenseMatrix, a row-major f64-only matrix with addition, subtraction, multiplication, transpose and matrix-vector multiplication. It is currently used for small element-level FEM stiffness fixtures and optional backend benchmarks.

This module is **not** the long-term owner of worksheet matrix semantics. The decision is:

1. Keep the current module operational during the transition; do not break existing fixtures or remove the benchmark harness in this documentation step.
2. Treat it as a legacy/provisional numeric utility, not the canonical Jabr MatrixValue API.
3. Move or re-home its generic dense numeric kernels and independently verified element-level fixtures under a Math-owned numerical/matrix subsystem when Math has an appropriate public facade and tests.
4. Keep numerical backend selection behind Math's implementation boundary. Do not make Table depend on faer, nalgebra, or Math.
5. Do not duplicate a second permanent matrix algebra implementation in Table after the Math matrix API is available.
6. Any move must preserve the independently derived truss and beam stiffness fixtures and the non-finite/overflow rejection tests.
7. Sparse global FEM assembly/solving and structural design workflows remain outside Jabr Table. Their eventual owner is the engineering calculation layer; this decision does not make Table a FEM solver.

The current DenseMatrix uses f64 and is therefore not equivalent to the Math-owned exact/unit-aware worksheet matrix value. Migration is an explicit follow-up implementation, not an implicit reinterpretation of its serialized or runtime values.

## 4. Table range snapshot contract

Table provides a neutral, deterministic snapshot of a selected rectangular range. It does not construct a Math MatrixValue.

Conceptual DTO:

    GridRangeSnapshot {
        table_id: TableId,
        range: StableRangeRef,
        row_ids: Vec<RowId>,
        column_ids: Vec<ColumnId>,
        rows: usize,
        columns: usize,
        cells: Vec<GridCellSnapshot>, // row-major
        source_revision: ExternalRevisionToken,
    }

    GridCellSnapshot {
        cell_ref: CellRef,
        authored_source: Option<String>,
        primitive_value: NeutralCellValue,
        opaque_math_value: Option<VersionedOpaqueEnvelope>,
        cell_revision: Option<ExternalRevisionToken>,
    }

This is a logical schema only. The concrete DTO must follow the agreed cross-repository versioned envelope and identity conventions. Table does not import Kernel Revision; source_revision/cell_revision are opaque caller-supplied tokens or are carried in the Engine adapter envelope.

Requirements:

- Snapshot dimensions and row/column identities must agree with the requested stable range and its resolved current order.
- Cells are emitted in row-major visual order: top to bottom; within each row, left to right.
- Each cell retains its stable composite identity (table_id, row_id, column_id).
- The snapshot includes sufficient metadata to detect stale conversion results: table/range identity and source revision or an equivalent Engine-owned revision token.
- A snapshot is immutable and represents one consistent Table state. It must not mix cells from different revisions.
- Table does not evaluate authored source. A snapshot may carry authored source, primitive data, and opaque result envelope separately.
- Engine decides whether a cell's current evaluated Math result is available and whether that result is valid for this evaluation.
- The Table API exposes range extraction by stable references. A1 and structured reference parsing/resolution remains Math/Engine-owned as defined in steps 1–4.

## 5. Table range to Math matrix conversion

Conversion is performed by the Engine adapter using the neutral Table snapshot and Math's public value facade.

### 5.1 Evaluation and binding order

1. Math parses the range/matrix expression and produces reference AST nodes.
2. Engine resolves the references to a stable Table range.
3. Engine ensures every source/formula cell required by the range has a current, valid evaluation result under the worksheet execution model.
4. Engine requests one consistent GridRangeSnapshot from Table.
5. Engine converts each snapshot cell into a Math scalar value using the Math value-envelope/primitive import contract.
6. Engine asks Math to construct a MatrixValue with the snapshot's exact row/column shape and row-major elements.
7. Engine records the range identity and source revision as provenance for dependency tracking and invalidation.

A source-bearing cell is not parsed by Table. A formula cell must be evaluated by Math through Engine before its value can be used as a matrix element. A stale, failed, incomplete or skipped formula result is not a current scalar value.

### 5.2 Default cell conversion policy

The default conversion is strict and lossless:

| Table cell state | Matrix conversion |
|---|---|
| Current valid Math scalar result/envelope | Import as the corresponding Math scalar/quantity, preserving exactness and units |
| Primitive numeric lexeme/value | Parse/import through Math's numeric-value API; preserve lexical exactness where available |
| Empty | Reject conversion with a cell-addressed diagnostic |
| Text | Reject conversion; no implicit numeric-string parsing |
| Boolean | Reject conversion unless a Math operation explicitly defines boolean-to-scalar semantics |
| Table import/data error | Reject with the original cell identity and structured error context |
| Formula with current valid scalar result | Use the Engine-owned evaluated result, not the formula source string |
| Formula with no result, stale result, or non-scalar result | Reject with explicit incomplete/stale/type diagnostic |

No default policy fills empty cells with zero, drops rows/columns, skips text, coerces booleans, or parses numeric-looking text. If a future operation needs such behavior, it must request an explicit conversion policy and record that policy in the evaluation provenance. A default fill value must be a Math value, not a Table-owned number.

A matrix conversion must fail as a whole if any selected cell cannot be converted under the chosen policy. It must not return a silently shortened or ragged matrix.

### 5.3 Shape and resource validation

- Reject ragged or inconsistent shape before constructing MatrixValue.
- Use checked multiplication for rows * columns.
- Enforce configurable Engine resource limits for maximum rows, columns and total elements before allocating.
- Reject overflow and oversized ranges with structured diagnostics.
- Preserve the stable row/column/cell references so a Math diagnostic can identify the originating cell.
- A range including a header row is treated exactly as selected; Table must not infer or strip headers. Header exclusion is an explicit range selection or Engine conversion option.
- A range may be transposed only by an explicit Math operation; Table extraction always uses the documented row-major order.

## 6. Math matrix to Table/output projection

The reverse path is a projection, not a direct mutation of authored source data:

1. Math returns a MatrixValue with explicit shape and Math scalar elements.
2. Engine validates the result and associates it with the calculation execution/revision.
3. Engine serializes each Math scalar through the versioned opaque value-envelope contract and may request a separate display projection for ordinary text/number rendering.
4. Table may accept a neutral rectangular output projection for display, interchange, or an explicit export operation. It preserves row-major order and shape.
5. UI renders the derived result separately from authored table contents unless the user explicitly performs a paste-values or equivalent authored edit.

Rules:

- A computed matrix result MUST NOT silently overwrite authored Table cells, source text, or Yjs document state.
- A display/export projection is derived and non-authoritative. Kernel/Engine owns result freshness, revision and snapshots.
- The Math value envelope is authoritative for the mathematical value; any primitive display value is only a presentation projection.
- Units and exact values must remain available through the opaque Math envelope even if the display string is rounded or formatted.
- Projection shape must match the Math matrix shape exactly. No implicit truncation, expansion or broadcasting.
- If a user explicitly pastes matrix values into a Table, UI submits a separate authored edit command; Engine validates target shape and revision, and Table applies the resulting primitive/envelope payloads atomically. Formula source is never overwritten without explicit user action.
- A computed matrix output is not automatically a new worksheet variable or a Table declaration. Engine owns naming, declaration identity and dependency semantics.

## 7. Diagnostics and provenance

Conversion diagnostics are owned by Math/Engine and should conform to Kernel's shared diagnostic/span contracts. Table returns structural/data errors with stable cell references and must not create a competing Math diagnostic model.

At minimum, diagnostics distinguish:

- Range not found or no longer valid.
- Stale source revision.
- Empty cell under strict conversion.
- Text/boolean/error cell not convertible.
- Formula result missing, stale, skipped, incomplete or failed.
- Non-scalar Math value in a matrix element position.
- Ragged/invalid shape or dimension overflow.
- Resource limit exceeded.
- Math value-envelope schema/version unsupported.
- Unit/dimensional incompatibility during matrix operations.

Provenance for a converted MatrixValue records the stable Table ID, resolved stable range/cell references, source revision, conversion policy, row-major shape and relevant Engine execution identity. This allows Engine to invalidate the matrix result when any source cell or relevant range membership changes.

## 8. Acceptance and golden tests

### Table snapshot tests

1. A rectangular range snapshot preserves exact row/column/cell identities.
2. Snapshot cell order is row-major (top-to-bottom, left-to-right).
3. Snapshot shape agrees with the cell count using checked multiplication.
4. A range containing a header is not silently modified.
5. Snapshot is revision-consistent and cannot combine cells from different source revisions.
6. Structural changes from step 5 produce the expected new range membership and identity list.

### Conversion tests

7. Numeric lexemes convert through Math without an intermediate f64 round-trip.
8. Quantities and units survive Table opaque-envelope -> Engine adapter -> Math value conversion.
9. Empty, text, boolean and error cells produce deterministic cell-addressed diagnostics under strict policy.
10. Numeric-looking text is not implicitly parsed.
11. Formula source is never used as the matrix element; only its current valid Engine/Math result is accepted.
12. Stale, failed, skipped, incomplete and non-scalar formula results are rejected.
13. Rectangular 2x3 and 3x2 ranges preserve shape and row-major element identity.
14. Ragged shapes, multiplication overflow and configured resource-limit violations are rejected before allocation.
15. One invalid element rejects the complete conversion; no silent dropping or zero fill occurs.
16. An explicit conversion policy, if added, is recorded in provenance and has independent tests.

### Matrix semantics tests in Math

17. Matrix addition/subtraction require equal shapes and compatible Math element values.
18. Matrix multiplication validates inner dimensions and Math dimensional compatibility of product sums.
19. Transpose preserves exact element values, units and shape semantics.
20. Matrix operations do not silently coerce exact or unit-aware values to f64.
21. Backend numerical approximations are explicit Math value kinds with defined tolerances and diagnostics.

### Result projection and integration tests

22. Math matrix -> neutral Table output projection preserves exact shape and element order.
23. Opaque envelopes preserve quantities/units even when display projections are rounded.
24. Computed results do not mutate authored Table cells or Yjs state.
25. Explicit paste-values is a separate atomic authored edit and cannot overwrite formula source without explicit user action.
26. Stale matrix results are rejected/invalidated when a source cell or range membership changes.
27. Cross-repository golden fixtures round-trip matrix shape, scalar envelopes, units and stable cell provenance.

## 9. Implementation sequence and exit criteria

Implement in this order:

1. Agree the Math-owned MatrixValue contract and scalar import/export facade.
2. Define the neutral GridRangeSnapshot and versioned wire/golden fixtures without adding a Math dependency to Table.
3. Implement Table range snapshot extraction with stable identities and row-major order.
4. Implement Engine conversion adapter and strict conversion diagnostics.
5. Add Math matrix construction, algebra and unit-aware tests behind the Math public facade.
6. Implement derived result projection from Math to UI/Table interchange without mutating authored source.
7. Add explicit paste-values only as a separate authored edit workflow, if required.
8. Migrate the legacy Table DenseMatrix numerical kernels/fixtures to a Math-owned numerical subsystem after parity tests pass; keep Table's legacy module until that migration is independently validated.

**Step 6 exit criteria:** Math owns a tested matrix value/algebra API; Table exposes a stable neutral range snapshot and no Math dependency; Engine converts ranges to matrices and back using versioned envelopes; strict error/shape/unit/provenance tests pass; computed results remain derived and do not mutate authored document state.

## 10. Relationship to prior contracts

This contract refines the matrix ownership placeholder in CELL-ENGINE-INTEGRATION-CONTRACT.md and the integration sequence following STRUCTURAL-OPERATIONS-REFERENCE-REWRITING.md. It does not change the current Table Rust API or claim that Math MatrixValue, Table range snapshots or Engine conversion adapters are implemented.

Step 7 (Kernel diagnostics/spans/envelopes and interchange hardening) and step 8 (CI/repository hygiene, MSRV, Clippy, Serde compatibility and release policy) remain separate follow-up contracts.
