# Kernel Contracts, Golden Fixtures, and Table Interchange Hardening

**Status:** Proposed Step 7 integration contract.  
**Depends on:** CELL-ENGINE-INTEGRATION-CONTRACT.md, STRUCTURAL-OPERATIONS-REFERENCE-REWRITING.md, and MATRIX-OWNERSHIP-AND-RANGE-CONVERSION.md.  
**Scope:** Align Table-facing diagnostics, source spans and JSON transport with Kernel 0.1; define cross-repository golden fixtures; specify CSV/XML safety, limits and round-trip behavior.  
**Status of implementation:** Specification only. No Rust code, Kernel API, or interchange behavior is changed by this document.

## 1. Decisions and ownership

The existing Kernel 0.1 public contract is the normative generic contract for this step:

- `Diagnostic`: `code`, `severity`, `message`, optional `span`.
- `Severity`: `info`, `warning`, `error`.
- `Span`: UTF-8 byte offsets, half-open range `[start, end)`.
- `Location`: one-based line and one-based Unicode-scalar column.
- `Envelope<T>`: `schema_version`, optional `result`, and `diagnostics`.
- `ENVELOPE_SCHEMA_VERSION`: currently `1`; deserialization rejects every other version.

The canonical consumer dependency is the accepted `jabr-kernel-api` public-contract crate once that crate boundary is available. Until then, the current `jabr-kernel` 0.1 source is the reference implementation of the contract. Consumers must depend on the public API surface, not Kernel implementation internals. The eventual crate split must preserve the documented public types and wire representation unless an explicitly reviewed breaking version says otherwise.

| Concern | Owner |
|---|---|
| Generic Diagnostic, Severity, Span, Location, Envelope and schema-version rules | Kernel public API |
| Mathematical error codes, parser/evaluator source spans, mathematical context | Jabr Math |
| Table data and interchange errors, record/row/column/cell context | Jabr Table |
| Mapping/aggregation of engine errors, source identity, request correlation, execution context and stale-result rejection | Jabr-engine / Orchestrator |
| Authored document identity, source text ownership, UI presentation and collaboration | Jabr-UI |

Dependency rules:

1. Kernel remains domain-neutral and does not depend on Math, Table, Engine or UI.
2. Table remains independent of Kernel and Math. It exposes Table-owned errors and structured context; the Engine adapter maps them to Kernel diagnostics.
3. Math may depend on the Kernel public API for generic diagnostics/spans/envelopes through the public contract, but Kernel does not depend on Math.
4. Engine coordinates diagnostics and transport. It must not parse human-readable messages to infer error meaning.
5. UI may localize/render diagnostics, but machine behavior uses stable codes and structured context, never message text.

## 2. Diagnostic alignment

### 2.1 Kernel diagnostic shape remains generic

Do not add Table-specific fields, cell IDs, source IDs, row numbers, import format, evaluation state, or domain error enums to Kernel's `Diagnostic` in this step.

Use the existing fields as follows:

- `code`: stable, machine-readable, lower-case namespaced identifier owned by the emitting domain.
- `severity`: Kernel's generic `Info`, `Warning`, or `Error`.
- `message`: human-readable explanatory text; may change and must never be parsed.
- `span`: optional primary source span, expressed as UTF-8 byte offsets into the exact source string identified by the surrounding domain/Engine context.

Example code namespaces:

- `kernel.*`: generic Kernel input/contract failures only.
- `math.parse.*`, `math.type.*`, `math.unit.*`, `math.eval.*`: Math-owned failures.
- `table.import.csv.*`, `table.import.xml.*`, `table.export.*`, `table.data.*`: Table-owned failures.
- `engine.binding.*`, `engine.execution.*`, `engine.transport.*`: Engine-owned failures.

Codes are stable API. Renaming a code is a compatibility change. Messages are not stable API. A code must describe the failure category, not contain interpolated values, row numbers or localized prose.

### 2.2 Domain context stays outside Kernel Diagnostic

Kernel Diagnostic has only one optional Span and no source/document identity. Do not pretend the span is globally meaningful by itself.

The emitting domain or Engine adapter must retain the context needed to resolve it:

- source/document identifier and source revision;
- source kind (worksheet expression, formula cell, CSV input, XML input, etc.);
- optional stable block/cell/row/column identity;
- optional one-based line/column for display, derived from the exact source string;
- related locations or multiple cell references when the error is not representable by one primary span.

This context may be carried in an Engine-owned diagnostic context DTO or in a domain-specific result payload adjacent to Kernel Diagnostic. It is not a new Kernel field and must not be serialized by pretending it belongs to Kernel's generic contract.

For Table interchange errors, prefer stable structural context (record index, field/column identity, row/cell identity) over fabricated source spans. Only attach a Span when the parser/adapter can establish correct UTF-8 byte offsets into the original input string. A CSV record number is not a source span.

### 2.3 Mapping Table errors to Kernel diagnostics

Table retains its own typed Rust error enums. The Engine adapter maps each typed variant to a stable `table.*` diagnostic code, Kernel severity and a concise message. Do not map every error to one generic `table.error` code.

Mapping requirements:

- Invalid user/import data: usually `Error`.
- Recoverable data normalization or explicitly lossy import: `Warning` only when the operation is allowed to complete and the loss is reported.
- Informational notices: `Info`.
- Preserve the original typed Table error and structural context in the adapter/domain payload for programmatic handling.
- Never use a message substring as the discriminator.
- A failed import must not return a partially populated Table unless a separately named partial-import API explicitly defines that behavior.

### 2.4 Diagnostic versus evaluation/compliance state

Kernel Severity is not an evaluation state and not a CheckBlock compliance result.

Keep these independent:

- Evaluation: `VALID`, `ERROR`, `INCOMPLETE`, `SKIPPED`, `STALE`.
- Check compliance: `PASS`, `FAIL`, `WARN`, `INCOMPLETE`.
- Diagnostic severity: `INFO`, `WARNING`, `ERROR`.

For example, a CheckBlock can have a valid evaluation and produce compliance `FAIL`; that is not automatically a Kernel Severity::Error. Conversely, an expression parse error is an evaluation error with an Error diagnostic. Engine owns the association and lifecycle.

## 3. Source span and location alignment

### 3.1 Normative coordinate rules

Use Kernel's existing definitions without reinterpretation:

- Span offsets are UTF-8 byte offsets.
- Span is half-open: start inclusive, end exclusive.
- Empty spans are valid.
- Reversed spans are invalid.
- Location line and column are one-based.
- Location column counts Unicode scalar values, not bytes, UTF-16 code units, grapheme clusters or visual tab stops.
- Kernel validates span ordering and nonzero Location coordinates; it does not map a span to a location.

Every span must refer to the exact immutable source revision from which it was produced. Never apply a span to a later edited string without remapping/reparsing.

### 3.2 Source identity and revision

Because Kernel Span contains offsets only, Engine/domain context must bind each span to:

- a stable source identity (for example block ID + field/cell identity);
- a source revision or content digest;
- the exact source string or a source provider capable of retrieving that revision.

A span without matching source identity/revision is not safe to navigate. If the source revision is unavailable, retain the diagnostic but omit navigation rather than guessing.

### 3.3 Unicode and parser adapter requirements

- Parsers must report UTF-8 byte offsets or provide a verified mapping to UTF-8 byte offsets.
- Do not copy UTF-16 offsets from browser editors directly into Kernel Span.
- Do not convert Unicode-scalar Location columns to UTF-16 indices for storage.
- UI adapters may derive editor-native offsets from the exact source text and editor encoding.
- CRLF, LF and Unicode text must have golden tests.
- Invalid UTF-8 cannot enter a Rust `&str` API; byte-oriented import adapters must report decoding errors separately and must not fabricate character spans.

### 3.4 Multiple and related spans

Kernel Diagnostic supports one primary span. For diagnostics involving multiple operands/cells:

- Attach the most actionable primary span when one exists.
- Keep related source locations and stable cell references in the domain/Engine diagnostic context.
- Do not overload the message string with serialized locations.
- Do not extend Kernel Diagnostic with a vector of related spans in this step. A future generic related-location API requires a separate Kernel compatibility proposal.

## 4. Versioned JSON envelope alignment

### 4.1 Kernel Envelope v1 is normative

The existing generic shape is:

    {
      "schema_version": 1,
      "result": null,
      "diagnostics": []
    }

Rules:

1. `schema_version` is required and must equal `ENVELOPE_SCHEMA_VERSION`.
2. Unsupported historical or future versions are rejected during deserialization.
3. `result` is optional/nullable and its payload schema is owned by the domain.
4. `diagnostics` is an array of Kernel Diagnostic objects.
5. Diagnostic-code stability is independent of envelope schema version.
6. Consumers construct envelopes through Kernel constructors; callers must not mutate the public schema version field.
7. Do not silently deserialize unknown fields into guessed domain behavior or reinterpret unknown payload versions.

### 4.2 Success, failure and partial results

The generic Kernel envelope permits an optional result and diagnostics; it does not itself define the semantic status of every operation.

Engine/domain operation contracts must define the interpretation:

- Successful operation: result present; diagnostics may be empty or contain non-error notices.
- Failed atomic operation: result absent; at least one Error diagnostic.
- Partial result: only permitted when the specific operation explicitly documents partial semantics and carries an explicit completeness/status field in its result payload. Do not infer partial success merely because both result and diagnostics are present.
- A failed CSV/XML import is atomic by default: no result Table is published.
- A calculation with a valid result and Warning diagnostic may be valid; Engine decides whether the warning affects downstream evaluation according to domain rules.

Do not add a generic `ok`, `status`, `request_id`, `source_revision` or `execution_id` field to Kernel Envelope in this step. Transport correlation and revision tokens belong to the Engine/UI request-response wrapper. Domain result payloads may carry their own typed status and provenance.

### 4.3 Payload ownership and serialization

- Table JSON payloads serialize Table-owned structures only.
- Math values/quantities use Math-owned versioned value payloads; Table does not serialize Math structs directly.
- Opaque Math value envelopes are carried as opaque versioned payloads where Table integration requires them; Table validates only the outer transport contract and does not inspect mathematical semantics.
- Engine owns cross-engine DTO composition and adapter migrations.
- JSON numbers must not be used for integers outside the interoperable JavaScript-safe range unless represented as decimal strings or an explicit tagged representation. This is particularly relevant to identifiers and exact numeric lexemes.
- The current Table `Number(f64)` representation is approximate and does not promise exact decimal preservation. Do not describe JSON round-trip of that variant as exact arithmetic.

### 4.4 Compatibility and migration

- Envelope v1 is strict. Unknown versions fail closed.
- A v2 consumer must use an explicit v1-to-v2 adapter/migration; do not relax the v1 deserializer.
- Additive changes to a domain payload still require that payload's own compatibility policy. The generic Envelope version does not automatically version every nested domain schema.
- Golden fixtures pin the exact wire representation and are reviewed as compatibility changes.
- When the accepted `jabr-kernel-api` crate boundary is published, consumer imports move to that crate without changing the contract or creating a second, competing envelope type.

## 5. Cross-repository golden fixture strategy

Golden fixtures are compatibility tests, not illustrative snippets. They must be deterministic, UTF-8, formatted JSON/XML/text with stable ordering and no timestamps, random IDs or machine-specific paths.

### 5.1 Ownership and layout

- Kernel repository owns canonical generic fixtures for Kernel Diagnostic, Span, Location and Envelope v1.
- Each domain repository owns its domain payload fixtures and its adapter tests.
- Table repository owns CSV/XML import/export fixtures and Table-specific envelope payload fixtures.
- Engine owns end-to-end cross-engine fixtures when the Engine repository exists.
- Consumer repositories may vendor byte-identical copies of generic Kernel fixtures for offline CI, with a manifest recording the upstream Kernel commit and fixture digest. A vendored copy is not a second authority.

Suggested Table fixture layout:

    tests/
      fixtures/
        kernel-v1/
          diagnostic-error.json
          diagnostic-with-span.json
          envelope-success.json
          envelope-failure.json
          envelope-warning-with-result.json
          manifest.json
        interchange/
          csv/
            quoted-fields.csv
            embedded-newlines.csv
            empty-vs-empty-string.csv
            unicode-crlf.csv
            duplicate-headers.csv
            malformed-quoting.csv
            over-limit.csv
          xml/
            canonical-v1.xml
            namespaced-fields.xml
            whitespace-preservation.xml
            escaped-text.xml
            duplicate-fields.xml
            malformed.xml
            doctype-rejected.xml
            over-limit.xml
        expected/
          interchange-cases.json

### 5.2 Required generic Kernel fixture cases

The canonical Kernel fixture corpus must cover:

1. Diagnostic without span.
2. Diagnostic with a UTF-8 byte span.
3. Span at start, empty span, and non-ASCII source offsets.
4. Location one-based coordinates.
5. Envelope v1 success with result.
6. Envelope v1 failure without result and with Error diagnostic.
7. Envelope v1 result plus Warning diagnostic.
8. Rejection of schema version 0.
9. Rejection of future schema version 2.
10. Rejection of reversed spans and zero-valued locations.

Kernel owns these fixtures and tests deserialization/serialization against its actual public API. Table consumers test that their adapter can transport them unchanged.

### 5.3 Table/domain golden cases

For every interchange fixture, record:
- input bytes/text and format/options;
- expected success/failure;
- expected typed table values and stable IDs when successful;
- expected diagnostic code/severity and structural context when unsuccessful;
- whether the operation is lossless, lossy, or intentionally rejected;
- any canonical output bytes/text where round-trip equality is expected.

Do not assert unstable human-readable diagnostic messages as compatibility API. Assert stable code, severity, span/context and typed result.

### 5.4 Determinism and review

- Sort object keys only where the serializer contract requires it; preserve semantically ordered arrays (columns, rows, diagnostics).
- No floating-point fixture should rely on platform-specific formatting; specify canonical serialization and finite-value policy.
- A fixture update requires an explanation of the intended compatibility change.
- CI must run fixtures without network access.
- Cross-repository fixture versions/digests must be pinned, not fetched from `main` during tests.

## 6. CSV interchange hardening

The current CSV/TSV adapter uses the `csv` crate, supports quoted fields and configurable headers/delimiters, infers homogeneous number/boolean/text columns, and stores numbers as `f64`. It currently reads all records into memory before constructing the Table. The current model does not preserve a numeric source lexeme.

The following behavior is normative for the hardened adapter.

### 6.1 Explicit options and defaults

Define an explicit options object (concrete Rust signatures deferred) with:

- delimiter (comma for CSV, tab for TSV);
- header mode: present, absent, or explicit supplied header mapping;
- type inference mode: enabled or all-text;
- configurable import limits;
- blank/empty handling;
- duplicate/empty header policy;
- numeric policy acknowledging f64 approximation;
- export quoting and optional spreadsheet-formula-injection mitigation.

Default behavior must be deterministic and documented. Never silently change an imported value's type based on locale.

### 6.2 Parsing and data semantics

- Use a standards-aware CSV parser; do not hand-split lines.
- Support quoted delimiters, escaped quotes, CRLF/LF, and embedded newlines in quoted fields.
- Preserve text cell whitespace exactly. Trimming may be used only for type recognition and numeric/boolean parsing; a value classified as Text retains its original field contents.
- Empty unquoted fields map to Empty under the default policy. Distinguishing an empty quoted string from an empty field requires an explicit representation/policy; do not claim the current CellValue model preserves that distinction.
- Header names are preserved as text. Empty or duplicate headers must be handled by an explicit policy (reject or deterministic disambiguation); do not silently create ambiguous columns.
- Column IDs generated during import must be deterministic and unique within the imported Table. They are new IDs, not source identity from CSV.
- Rows must have the exact expected field count. Ragged records fail with record context; no silent padding/truncation.
- Type inference must be deterministic and locale-independent. Mixed columns become Any or follow an explicit all-text policy.
- Numeric import into the current `Number(f64)` is approximate. Reject non-finite values. If a decimal cannot be represented within the declared precision policy, either preserve it as text or report a lossy-conversion warning according to explicit options; do not claim exactness.
- Booleans are recognized only by the documented true/false grammar, case-insensitive if that is the selected policy.
- CSV cannot preserve Table IDs, row/column IDs, declared types or units without an external schema/sidecar. Do not promise identity-preserving CSV round trips.

### 6.3 Export semantics

- Quote fields as required by the CSV standard, including delimiters, quotes and embedded newlines.
- Export rows and columns in Table order.
- Define stable formatting for numbers and booleans; reject non-finite numbers.
- Export Empty as an empty field. The current data model cannot distinguish it from an authored empty text string in plain CSV.
- Preserve text contents; do not trim on export.
- Spreadsheet formula injection is a separate security concern: offer an explicit safe-for-spreadsheet export mode that prefixes/escapes dangerous leading characters according to documented policy. Do not silently mutate canonical CSV data export.
- Return UTF-8 output and propagate writer/encoding errors as CSV/export errors, not XML errors.
- CSV export/import round-trip tests compare the values representable by the format and selected type policy, not table identity or metadata that CSV cannot encode.

### 6.4 Resource limits and streaming

Provide configurable hard limits at least for:

- total input bytes;
- number of records/rows;
- number of columns;
- maximum field/cell byte length;
- total cells;
- optionally total decoded text bytes.

Check limits while reading, before allocating the full table. Prefer streaming records into the Table builder. Abort atomically on limit violation and return a structured error with the limit name and observed/allowed count. Defaults must be finite and conservative; applications may override within documented maximums.

## 7. XML interchange hardening

The current XML adapter uses `roxmltree`, selects repeated record elements by local name, supports a field-to-column mapping, imports canonical column metadata, and exports table/column/row metadata. The current implementation trims XML field text, matches namespaced fields by local name only, and sanitizes arbitrary column IDs into XML element names; these behaviors need explicit hardening.

### 7.1 Security and parser limits

- Keep external entity resolution and network access disabled. Never fetch external DTDs, schemas or entities.
- Reject DOCTYPE declarations explicitly unless a future separately reviewed safe subset is implemented.
- Enforce maximum input bytes before parsing, then maximum records, columns, fields per record, nesting depth and text bytes.
- XML parsing must not permit unbounded memory/CPU work from untrusted input.
- Malformed XML fails atomically with structured context.
- Do not resolve arbitrary filesystem paths, URLs or external references from XML content.
- Tests must include DOCTYPE/XXE attempts, entity expansion attempts, deeply nested input and over-limit documents.

### 7.2 Namespace and field mapping

- Resolve XML names using expanded names (namespace URI + local name), not local name alone, when namespaces are present.
- A local-name-only mapping may remain as a documented convenience only when it is unambiguous across the selected records.
- If two namespace-qualified fields share the same local name and no explicit mapping disambiguates them, reject rather than merge them.
- Record element selection must use the same expanded-name policy.
- Define deterministic behavior for missing fields, repeated fields, unknown fields and unexpected attributes.
- Repeated child elements must not silently overwrite earlier values. Reject duplicates by default or use an explicit aggregation policy.
- Preserve row/column IDs only when present in validated canonical metadata; generic XML imports generate deterministic new IDs.

### 7.3 Text and type semantics

- Preserve text node contents exactly, including leading/trailing whitespace. Trim only a temporary view used for number/boolean recognition.
- Empty/missing element and empty text element behavior must be explicitly defined. The current CellValue model may not distinguish them; document the loss or add a future explicit presence model.
- Never silently coerce malformed numeric/boolean text.
- Reject non-finite numbers.
- Mixed or unknown values become Any/Text according to explicit import options.
- Canonical metadata type declarations must be validated against imported values; mismatches fail with row/field context.
- Unknown metadata versions or unsupported type names fail closed or use an explicitly documented generic-XML fallback; never guess canonical semantics.

### 7.4 Canonical XML format

Define a versioned canonical XML format for Table interchange:

- Root identifies the Table format and includes a format version.
- Column metadata includes stable column ID, display name and declared type.
- Rows include stable row IDs.
- Cell values have an explicit representation sufficient to distinguish empty, text, finite number and boolean under the current CellValue model.
- Arbitrary IDs must be encoded into collision-free XML element/attribute representations; lossy character replacement such as mapping every unsupported character to underscore is not sufficient.
- XML text and attribute values must be escaped correctly.
- Export order is deterministic: columns in Table order, rows in Table order, cells in column order.
- Import rejects unsupported canonical format versions. Generic third-party XML import remains a separate explicit mode.
- A canonical XML export/import round trip preserves Table ID, row/column IDs, names, types and values for the supported current model.

### 7.5 Resource and streaming policy

Provide finite configurable limits for input bytes, records, columns, nesting depth, fields per record and text bytes. The byte limit must be checked before building the in-memory XML tree. If the chosen parser cannot enforce a required structural limit during parsing, document the parser's bounds and reject over-limit byte size before parse; do not claim a streaming guarantee that the parser does not provide.

## 8. Error mapping and atomicity matrix

| Failure | Table-level outcome | Engine/Kernel transport |
|---|---|---|
| Malformed CSV/XML | Typed import error; no Table result | Error diagnostic, `table.import.*` |
| Input/record/cell limit exceeded | Typed limit error; no Table result | Error diagnostic with limit context |
| Duplicate/ambiguous headers or XML fields | Typed schema/mapping error | Error diagnostic |
| Numeric precision loss under strict policy | Reject or preserve-as-text per explicit option | Error or Warning only if conversion completes and loss is disclosed |
| Unsupported canonical XML version | Reject | Error diagnostic |
| Invalid table on export | No output string | Error diagnostic |
| Safe spreadsheet export changed output | Export succeeds with explicit safe mode | Warning/info metadata may disclose transformation; never mutate Table |
| Parser can identify exact input span | Preserve byte span in adapter context | Kernel Diagnostic with primary Span |
| Parser cannot identify reliable byte span | Keep record/field context; no fabricated span | Kernel Diagnostic without span |

Every failed import/export operation is atomic. A failed import must not publish a partial Table; a failed export must not return a partial output string as if successful.

## 9. Acceptance tests and golden corpus

### 9.1 Kernel contract conformance

1. Table/Engine adapter round-trips Kernel Envelope v1 without changing fields.
2. Unsupported envelope schema versions are rejected.
3. Diagnostic codes are namespaced and stable; messages are not parsed.
4. Span offsets are UTF-8 byte-based and half-open.
5. Unicode Location columns count Unicode scalar values.
6. A diagnostic span is only navigable when source identity and revision match.
7. Domain context does not leak into Kernel's generic Diagnostic type.

### 9.2 CSV

8. Quoted delimiters, escaped quotes, embedded newlines and CRLF/LF.
9. Unicode headers and values.
10. Empty field versus whitespace-only text.
11. Duplicate and empty headers under every supported policy.
12. Ragged rows and malformed quoting fail atomically.
13. Numeric, boolean, text and mixed-type inference is deterministic.
14. Non-finite and over-precision numeric inputs follow the explicit numeric policy.
15. Maximum input bytes, rows, columns, field length and total cells are enforced before unbounded allocation.
16. Export quoting, stable order, Empty handling and spreadsheet-safe mode.
17. CSV round-trip asserts only properties representable by CSV.

### 9.3 XML

18. Canonical XML round-trip preserves supported Table metadata and values.
19. Namespace URI disambiguation and ambiguous local-name rejection.
20. Leading/trailing text whitespace preservation.
21. Missing, empty and repeated fields follow explicit rules.
22. Duplicate IDs and lossy XML-name collisions are rejected or safely encoded.
23. Escaping of ampersand, angle brackets, quotes and apostrophes.
24. DOCTYPE, external entity, entity-expansion and external-resource attempts are rejected.
25. Malformed, deeply nested and over-limit XML fails atomically.
26. Unsupported canonical XML versions and type metadata fail closed.
27. XML record/field errors preserve deterministic row/field context.

### 9.4 Cross-repository fixtures

28. Kernel canonical JSON fixtures pass Kernel's own tests.
29. Table's vendored Kernel fixture copies match the pinned manifest digests.
30. Table's CSV/XML expected outputs are deterministic across CI runs.
31. A future Engine integration test composes Table import -> Engine mapping -> Kernel Envelope/Diagnostic and verifies provenance without Table depending on Kernel.

## 10. Implementation order and exit criteria

1. Publish/confirm the Kernel public API fixture corpus and pin its revision/digests.
2. Add Table-owned interchange golden fixtures and an expected-case manifest.
3. Define typed Table import/export error variants and the Engine mapping table; do not make Table depend on Kernel.
4. Implement CSV options, streaming/limits, deterministic inference and explicit numeric-loss policy.
5. Implement XML expanded-name mapping, text preservation, canonical format versioning, collision-free IDs and security/resource limits.
6. Add deterministic serialization and atomicity tests.
7. Add Engine adapter conformance tests when the Orchestrator repository/API is available.

**Step 7 exit criteria:** all consumers use the same Kernel Diagnostic/Span/Location/Envelope v1 semantics; domain context remains outside generic Kernel types; generic and Table-specific golden fixtures are pinned and tested; CSV/XML have finite resource limits, explicit lossy behavior, deterministic canonical formats and atomic failure semantics; no dependency-cycle or ownership boundary is violated.

## 11. Explicit non-goals

- No Kernel API expansion or schema v2.
- No new generic related-span/source-identity fields in Kernel Diagnostic.
- No Table -> Kernel or Table -> Math dependency.
- No formula parsing or evaluation in Table.
- No promise of exact arithmetic from current `Number(f64)`.
- No full XLSX workbook support.
- No XML schema/DTD network resolution.
- No streaming claim for an in-memory XML parser.
- No runtime implementation or repository merge is authorized by this specification.
