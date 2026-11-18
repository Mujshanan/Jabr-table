# Jabr Table–Math–Engine Cell Integration Contract

**Status:** Proposed architecture contract; steps 1–4 of the Table integration sequence.  
**Scope:** Formula ownership, function-provider boundary, cell value representation, addressing, ranges and Kernel ports.  
**Non-goal:** This document does not implement formula evaluation, spreadsheet functions, worksheet scheduling, or a new formula language.

## 1. Ownership and dependency direction

The subsystem boundary is:

| Concern | Owner |
|---|---|
| Authored cell source text, table/row/column/cell identity, grid structure, deterministic table operations and interchange | Jabr Table |
| Formula syntax, parsing, AST, mathematical values, quantities, units, dimensional semantics and expression evaluation | Jabr Math |
| Binding cell references, materializing table inputs, registering providers, mapping table outputs to Kernel ports, dependency graph construction, scheduling, invalidation and result lifecycle | Jabr-engine |
| Generic NodeId, Port, Revision, Snapshot, ResultStore, graph, diagnostic, span and versioned envelope contracts | Jabr-kernel-api / Kernel |

Jabr Table MUST NOT depend on Jabr Math, Jabr-engine, Jabr-kernel, BlockSuite, Yjs, or UI frameworks. Jabr Math MUST NOT depend on Jabr Table. Engine is the composition layer.

### 1.1 Authored source and evaluated values are distinct

A cell may contain authored source text, a typed data value, or both:

- **Authored source** is the exact text entered or imported as source. Table stores and returns it unchanged. Table does not tokenize, classify, parse, normalize, or evaluate it.
- **Primitive cell value** is a small table/interchange representation used for empty, literal number, text, boolean, and error states.
- **Math value envelope** is an opaque, versioned payload produced/consumed by Math. It can represent quantities, units, dimensions, exact numeric forms, and other Math-owned values without Table reproducing those types.
- **Evaluation result and lifecycle** are owned by Engine/Kernel. A table-side cached projection, if retained for rendering or interchange, is a non-authoritative cache and MUST NOT determine freshness, validity, dependency state, or whether a result is current.

A source-bearing cell is not automatically a formula. Engine decides whether and when source is submitted to Math. Math parses it according to the Math language. A text literal must not be reinterpreted as an expression merely because its storage type is text.

Table MUST preserve source text exactly, including whitespace and lexical numeric spelling, unless an explicit edit operation changes it. It MUST NOT silently rewrite references during row/column operations.

### 1.2 Formula evaluation flow

1. UI authors source text and persists it in the collaborative document.
2. The Table adapter stores/retrieves that source without interpreting it.
3. Engine resolves the cell's table references and supplies explicit bindings.
4. Math parses and evaluates the source against those bindings.
5. Engine commits the result and its revision/dependency metadata through Kernel.
6. Any table-side display cache is updated only as a projection of the Engine-owned result.

No second formula parser or evaluator is permitted in Table.

## 2. Range functions and the provider interface

Math owns a small, table-agnostic function-provider interface. It defines how a function is named, how typed arguments are passed, how the result is returned, and how provider errors/diagnostics are represented. It MUST NOT import Table types or know about table IDs, row IDs, column IDs, or Table's internal data structures.

Engine registers adapters implementing that Math interface. A Table-function adapter may delegate to pure Jabr Table operations. The adapter performs all conversion at the boundary:

1. Math parses a function call into its AST.
2. Math dispatches the call through the registered provider interface.
3. Engine's Table adapter resolves/materializes arguments and ranges into the provider's table-agnostic argument representation.
4. The adapter invokes the corresponding pure Table operation.
5. The adapter converts the result or structured error back to a Math value/result.

Table does not implement the Math provider trait and does not depend on Math to do so. The adapter belongs in Engine (or a separately versioned integration crate only if a later dependency review justifies one).

Initial provider-backed operations may include exact-match `VLOOKUP`, `HLOOKUP`, and `MATCH`, plus range aggregation `MAX` and `MIN`. Function names and argument semantics remain Math language/API decisions; Table supplies deterministic data operations. Matrix algebra is not part of this provider decision.

The provider contract must support explicit argument shape/type errors, missing values, deterministic results, and diagnostics. It must not silently coerce incompatible values or hide Table errors.

## 3. Cell value and opaque Math-value slot

The current `CellValue::{Empty, Text, Number(f64), Boolean}` is insufficient as the long-term cell contract. In particular, `f64` cannot preserve exact decimal/rational lexical input, and Table must not create a competing unit/quantity system.

The target logical model is:

```rust
struct Cell {
    id: CellId,
    source: Option<AuthoredSource>,
    value: CellValue,
    math_value: Option<OpaqueMathValueEnvelope>,
}

enum CellValue {
    Empty,
    Number(NumberLexeme),
    Text(String),
    Boolean(bool),
    Error(CellError),
}
```

This is a conceptual schema, not an approved Rust signature. The final concrete ID/envelope types must follow the repository's versioned DTO conventions and Kernel's public contract.

Rules:

- `NumberLexeme` preserves the authored/imported decimal spelling as text; Table does not perform engineering arithmetic on it. Math converts it into its own exact numeric representation.
- `OpaqueMathValueEnvelope` carries a schema identifier and version plus an opaque payload conforming to the Math value-envelope contract. Table may validate envelope framing/version compatibility, but MUST NOT inspect or interpret Math payload semantics.
- The envelope must preserve Math-owned quantities, units, dimensions, exact values and future value kinds without adding these types to Table's enum.
- `CellError` is a table-level data/import error representation, not a substitute for Math diagnostics or Engine evaluation states.
- Source text, primitive/interchange value, and Math envelope are separate fields with explicitly documented precedence and lifecycle. They must not be conflated into a single tagged enum.
- Engine/Kernel remain authoritative for evaluation status, diagnostics, revisions, staleness, snapshots and result history. Table must not introduce a parallel evaluation-state machine.

Migration from the existing `Number(f64)` must be explicit and tested. Existing serialized data must not be silently reinterpreted as exact values; the migration policy must state whether a legacy float is converted to a shortest round-trippable decimal lexeme or preserved as a legacy approximate payload.

## 4. Addressing, ranges and Kernel ports

### 4.1 Two addressing layers

Table exposes stable-identity references as its canonical internal addressing form:

- `TableRef`: stable table identity.
- `RowRef`: stable table and row identities.
- `ColumnRef`: stable table and column identities.
- `CellRef`: stable table, row and column identities.
- `RangeRef`: stable table identity and stable boundary/selection identities, with a documented ordering and membership policy.

A1 references (for example `B4` or `B4:D12`) are positional syntax, not durable identity. Math owns parsing the reference syntax. Engine resolves the parsed address against the current Table layout and binds it to stable IDs. Structured references (for example a table name and column name/ID) are also parsed by Math and resolved by Engine against Table's stable identities and metadata.

Table provides deterministic lookup/resolution primitives for row/column positions and stable IDs. It does not parse A1 syntax or structured-reference syntax.

### 4.2 Range semantics

A range must retain its table identity and explicit boundaries/selection semantics. A range is not merely an unqualified vector of values. Engine materializes a range into the Math argument/value representation when evaluating an expression or invoking a range provider.

The precise behavior of ranges under insert, delete, move, and sort operations—including whether inserted rows become members of a bounded range—is completed in the structural-operation/reference-rewriting contract (step 5). Until that policy and its tests exist, structural edits MUST NOT claim spreadsheet-compatible reference rewriting.

### 4.3 Kernel graph mapping

Engine owns all Kernel graph identities and mapping. The intended default is:

- One Kernel `NodeId` per Table entity, created/owned by Engine and associated with the stable Table ID.
- Stable output `Port` identities for column outputs, keyed by stable column identity.
- Range outputs use a deterministic range/selection port identity or an Engine-managed query port; the choice must not encode mutable row/column indices as durable identity.
- Cell-level formula evaluation nodes, where required, are Engine/Math execution concerns and are not implicitly created by Table.
- Table exposes data and stable references; it never creates Kernel nodes, ports, graph edges, or schedules.

This one-node-per-table model is the default to avoid one graph node per cell. Engine may introduce finer-grained nodes/ports only when measured invalidation or scheduling requirements justify them. Such a change must preserve the public Table API and stable reference semantics.

The exact `NodeId`/`Port` DTO fields and serialization must use the versioned `jabr-kernel-api` contract. Table must not copy or fork Kernel implementation types; cross-repository adapters consume the public API or its agreed wire representation.

## 5. Required acceptance tests before Engine integration

1. Source text round-trips byte-for-byte and is never parsed or normalized by Table.
2. A text literal remains text unless Engine explicitly submits source for Math evaluation.
3. A source-bearing cell can carry an opaque Math envelope without Table interpreting its payload.
4. Math envelope schema/version framing is validated; unknown versions fail explicitly without payload guessing.
5. Decimal lexical input such as `0.1`, `1.2300`, and large integer tokens is preserved without an intermediate `f64` conversion.
6. Error cells remain distinct from Math diagnostics and Engine evaluation states.
7. Math's provider interface compiles without a dependency on Jabr Table.
8. An Engine-side adapter can register Table lookup/aggregation operations through the provider interface without creating a dependency cycle.
9. Provider errors and type/shape mismatches are returned deterministically and are not silently coerced.
10. A1 and structured references are parsed by Math and resolved by Engine to stable Table IDs.
11. A1 position changes do not alter stable Table identity; unresolved/deleted references produce explicit outcomes.
12. A single table maps to one Engine-owned Kernel node with stable column/range ports; Table itself has no Kernel dependency.
13. Golden fixtures verify stable reference and envelope round-trips across repository boundaries.

## 6. Deliberately deferred work

The following remain separate follow-up contracts and must not be smuggled into this change:

- Step 5: structural operations, old-to-new index maps, and reference rewriting under insertion/deletion/move/sort.
- Step 6: matrix ownership and conversion between grid ranges and Math matrix values.
- Step 7: Kernel diagnostic/span/envelope golden fixtures, robust CSV/XML resource limits and parser hardening.
- Step 8: CI/repository hygiene, MSRV, Clippy, Serde compatibility tests and release/tag policy.

The existing dense matrix module remains in the repository during this contract stage. Its long-term ownership and migration are decided explicitly in step 6; this document does not silently move or remove it.

## 7. Relationship to existing architecture

This contract refines the ownership boundaries in `docs/ARCHITECTURE.md` and the Stage 4 integration work in `development.md`. It does not replace the existing Table data model or claim that formula cells, Engine integration, exact Math values, or spreadsheet reference rewriting are implemented.
