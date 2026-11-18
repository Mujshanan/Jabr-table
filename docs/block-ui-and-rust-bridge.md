# Jabr Block Registry and Engine Bridge Contract

## 1. Canonical custom-block and repository map

The product uses AFFiNE/BlockSuite as its document host. The following is the ownership map; engine repositories remain independently testable and do not own production block UI.

| Document element / block | Production owner | Engine owner | Rust bridge |
|---|---|---|---|
| Math / calculation | Jabr-UI | Jabr-math (locally maintained Calcpad Rust engine) | Native Rust for tests/desktop; versioned WASM adapter for browser |
| Table | Jabr-UI | Jabr-table (`jabr-table-core`) | Native Rust plus planned wasm-bindgen adapter |
| Matrix | Jabr-UI | Jabr-table (`jabr-table-core`) for dense matrix primitives; Jabr-math for worksheet-level expressions | Table operations through typed engine adapter; expressions through math adapter |
| Vector | Jabr-UI | Jabr-table (`jabr-table-core`) for vector/matrix primitives; Jabr-math for mathematical evaluation | Same split as Matrix |
| Plot | Jabr-UI | No separate plot-engine repository currently established; data comes from Jabr-math/Jabr-table | Plot rendering stays in UI; typed series data crosses existing math/table adapters |
| Diagram | Jabr-UI | Jabr-diagram (diagram model and geometry engine) | Rust bridge only if/when the engine core is Rust; current repository contents do not establish a Rust/WASM API yet |
| Text / prose | AFFiNE native paragraph where possible; Jabr-UI only for Jabr-specific note semantics | None | No Rust bridge |
| Root / worksheet container | AFFiNE owns production document root; Jabr-UI owns worksheet orchestration | None | No Rust bridge |

**Important:** Matrix and Vector are distinct document block presentations, but they do not require separate engine repositories. Plot is a Jabr-UI block, not an excuse to create another engine repository before the data/plot contract requires it. Text should use native AFFiNE paragraph behavior whenever possible.

## 2. Ownership boundaries

- **AFFiNE/BlockSuite:** production editor shell, root/page lifecycle, block identity/order, Yjs persistence/collaboration, document undo/redo, host selection and drag/drop.
- **Jabr-UI:** custom block schemas/views, document presentation, contextual editing controls, transient result state, orchestration, host adapters, and integration tests.
- **Jabr-math:** parsing/evaluation/calculation semantics and Rust engine API. Its test UI remains a minimal engine workbench.
- **Jabr-table:** typed tabular data, lookup/aggregation, interchange, and dense matrix/vector primitives. No UI, BlockSuite, Yjs, or formula parser.
- **Jabr-diagram:** diagram model, geometry, rendering/export core and its independently testable workbench. No final product shell.
- **No engine repository owns final production block styling.**

## 3. Shared engine bridge contract

### 3.1 Boundary

Use a TypeScript adapter in Jabr-UI that isolates BlockSuite views from Rust/WASM implementation details:

`Block view → block controller → typed TS engine adapter → WASM binding → Rust API`

The adapter must be replaceable by a native Rust host binding in a future desktop build without changing authored block schemas or UI-facing domain DTOs.

Do not call generated wasm-bindgen functions directly from Lit/BlockSuite view components. Do not expose Rust structs, pointers, wasm-bindgen classes, or engine-internal error enums as persisted document data.

### 3.2 API shape

Each engine adapter exposes:

- `initialize(): Promise<EngineInfo>`
- `getCapabilities(): EngineCapabilities`
- `execute(request): Promise<Response>`
- `dispose(): void` where the runtime owns resources.

Each request/response includes:
- `protocolVersion`
- `engineName`
- `engineVersion`
- `requestId`
- `documentId` and `blockId` when invoked from a document
- `sourceRevision` or `inputRevision`
- A discriminated success/error result
- Structured diagnostics with stable code, severity, message, and optional source span/path.

Use JSON-compatible, versioned DTOs at the WASM boundary. Prefer integer IDs/strings and explicit tagged unions; do not depend on JS floating-point serialization for exact engineering quantities. Exact decimals/rationals should be represented as canonical strings or tagged values where supported by the engine. Large binary/typed-array payloads may use explicit transfer buffers only after profiling demonstrates need.

### 3.3 Execution rules

- All expensive operations are asynchronous from the UI perspective.
- Use request IDs and source/input revisions to discard stale responses.
- Never block the browser main thread for long calculations; use a Worker when measured execution time warrants it.
- Do not silently fall back to mock or approximate results. An unavailable engine is an explicit state.
- Do not evaluate arbitrary code or HTML from engine responses.
- HTML reports, if returned by the math engine, are output artifacts and must be sandboxed; they are not the canonical data contract.
- Cancellation is best-effort and must not be confused with successful completion.
- Enforce payload/resource limits at the adapter boundary.

### 3.4 Data and persistence

- BlockSuite/Yjs persists authored source, table values, diagram geometry, plot configuration, and user-authored display preferences.
- Rust engines receive immutable request snapshots and return results; they do not mutate Yjs or access BlockSuite.
- Computed values, diagnostics, progress, caches, request IDs, and engine runtime state remain transient unless the user explicitly inserts a result as authored content.
- Never persist a WASM object or engine-internal memory handle.
- Stable block/row/column/diagram IDs are durable identities; array positions are not.

### 3.5 Errors and diagnostics

Use a shared normalized diagnostic shape:
- `code`: stable machine-readable identifier
- `severity`: info | warning | error
- `message`: user-readable message
- `location`: optional source range, cell coordinate, matrix index, or diagram object ID
- `details`: optional safe structured metadata

The UI may translate diagnostics into block-local messages, cell markers, or source annotations. Do not display raw Rust panic text as the user-facing message. A panic must be contained at the FFI boundary and surfaced as an internal engine failure with a correlation/request ID.

## 4. Per-block UI and Rust bridge specifications

### 4.1 Math / Calculation — Jabr-UI + Jabr-math

**UI**
- MathLive is the sole structured mathematical input editor.
- Keep authored source and calculated output visually distinct; output is read-only unless explicitly inserted as authored text.
- Provide run/recalculate, pending, unavailable, success, warning, stale, and error states.
- Keep display-unit preference as authored configuration only when user-selected.
- Diagnostics should map to source spans where the Rust API provides them.
- Never present the current mock evaluator as a real calculation engine.

**Rust bridge**
- Jabr-math owns the Calcpad Rust engine and a Jabr-owned stable adapter.
- Browser calls the WASM adapter asynchronously; native CLI is for engine tests and future native integration, not a browser subprocess.
- Initial operation: evaluate a worksheet snapshot and return normalized diagnostics plus report output.
- Structured values, units, precision, source spans, AST, and standalone LaTeX must be exposed only when the Rust engine API actually supports them; do not infer them by scraping report HTML.
- Native and WASM builds must share golden fixtures and produce semantically equivalent outcomes.
- Version/capability handshake must identify the protocol and supported output fields.

### 4.2 Table — Jabr-UI + Jabr-table

**UI**
- Render as document content, not a spreadsheet app window.
- Support stable row/column identity, cell selection/editing, headers, paste, resize, add/remove rows and columns, empty/error states, and internal overflow handling.
- Contextual actions appear on focus/selection; do not persist selection/focus.
- Preserve typed cell values and schema/column metadata.
- Large tables need virtualization only when profiling shows it is required.

**Rust bridge**
- Rust core remains UI-agnostic and serializable; no BlockSuite/Yjs dependency.
- Expose typed operations for table CRUD, lookup/aggregation, and interchange.
- Use stable row/column IDs in DTOs. Use positions only as operation arguments after resolving them against a revisioned snapshot.
- Include table revision and request ID; reject responses for stale revisions.
- WASM binding is a planned adapter, not a current capability unless the repository has implemented and tested it. Keep native Rust API authoritative and add cross-target contract tests before UI integration.
- CSV/TSV/JSON/XML import/export must enforce size/row/column limits and return structured validation diagnostics.

### 4.3 Matrix — Jabr-UI + Jabr-table

**UI**
- Use mathematical delimiters and compact numeric/text entry cells; do not make the matrix look like an ordinary spreadsheet.
- Clearly show dimensions and preserve row/column alignment.
- Keyboard navigation between entries must be predictable; dimension changes must preserve values where unambiguous and warn before destructive truncation.
- Display calculated matrices as read-only output unless explicitly converted to editable authored data.

**Rust bridge**
- Use the dense matrix/vector API in `jabr-table-core` for supported operations.
- Validate dimensions and numeric constraints in Rust; return typed dimension/numerical errors.
- Preserve row-major layout and explicit row/column counts in transport DTOs.
- Do not serialize large global FEM sparse systems through this small dense matrix block; that is a separate future solver boundary.
- Worksheet expressions and unit/dimensional semantics remain Jabr-math responsibilities.

### 4.4 Vector — Jabr-UI + Jabr-table/Jabr-math

**UI**
- Use vector notation with compact component editing and clear dimension/orientation.
- Distinguish authored components from calculated vector results.
- Support keyboard component navigation and explicit dimension changes.
- Units and dimensional meaning are displayed from engine metadata, not guessed by the UI.

**Rust bridge**
- Use Jabr-table for dense numeric vector/matrix primitives.
- Use Jabr-math for worksheet expressions, units, and dimensional validation.
- DTO includes component count, ordering, numeric representation, unit metadata when available, and revision.
- Reject mismatched dimensions explicitly; do not silently pad or truncate.

### 4.5 Plot — Jabr-UI

**UI**
- Plot block owns visual rendering, axes, labels, legend, theme, and accessible data summary.
- Configuration (series references, axis labels, units, ranges, style choices) is authored block data.
- Plot data/results are derived from referenced table/math outputs and remain transient or revision-keyed cache.
- Editing controls are contextual; the default block is a clean document plot.
- Provide empty, loading, invalid-data, and populated states.

**Rust bridge**
- No separate plot Rust engine is currently assigned.
- Request numeric series from Jabr-math/Jabr-table through their typed adapters; do not add plotting semantics to those engines merely to render charts.
- Keep chart rendering in Jabr-UI unless a measured performance or export requirement justifies a dedicated rendering core.
- Define stable series DTOs (x/y arrays, labels, units, missing-value policy, source revision) and validate finite values/range sizes at the UI boundary.

### 4.6 Diagram — Jabr-UI + Jabr-diagram

**UI**
- Diagram appears as a compact document object by default; entering edit mode exposes Jabr Draw tools.
- Define explicit enter/exit editing, zoom/pan, resize, keyboard ownership, Escape-to-document, selection, and export behavior.
- Preserve engineering geometry and aspect ratio; do not distort diagrams to fit document width.
- Diagram model data is authored and persisted; viewport/selection/tool state is transient unless explicitly saved as a view preference.

**Rust bridge**
- The Jabr-diagram repository is the designated owner of diagram model/geometry behavior.
- A Rust bridge contract is specified as the target, but the current repository inspection did not establish a Cargo workspace or an existing Rust/WASM API. Do not claim that such a bridge already exists.
- Before selecting Rust/WASM, confirm the engine's implementation language and ownership. If the core is Rust, expose versioned wasm-bindgen DTO operations for validate, transform, bounds/layout, and export; keep pointer/selection UI state in Jabr-UI.
- If the current engine core is TypeScript, do not introduce Rust solely to satisfy architectural symmetry. Define the same typed adapter contract and preserve the option to add Rust if a concrete need appears.
- SVG export must be deterministic and independent of editor chrome; sanitize untrusted SVG before insertion/rendering.

### 4.7 Text / prose

Use AFFiNE native paragraph blocks whenever possible. No Rust engine bridge is required. A Jabr-specific note block must follow host typography, text selection, and document keyboard behavior.

## 5. Cross-repository compatibility and acceptance

- Keep shared JSON schema/fixture definitions in Jabr-UI integration tests or a separately agreed contract package; do not duplicate divergent DTO definitions in every UI component.
- Each engine repository tests its native API independently.
- Each Rust engine with a browser integration publishes a WASM build and tests the same contract fixtures as its native build.
- Jabr-UI tests adapter behavior with deterministic fixtures, including success, validation error, engine unavailable, panic/transport failure, cancellation, and stale response.
- End-to-end document tests cover mixed paragraphs and all supported blocks, reload, undo/redo, theme changes, narrow widths, and engine disconnection.
- No block is production-ready until its document data contract, UI state contract, and engine adapter contract are all tested.
