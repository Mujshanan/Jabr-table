# Jabr Table Structural Operations and Reference-Rewriting Contract

**Status:** Proposed architecture contract; integration sequence step 5.  
**Depends on:** CELL-ENGINE-INTEGRATION-CONTRACT.md (steps 1–4).  
**Scope:** Insert, delete, move, and sort semantics; stable identity; index maps; range behavior; reference rewriting; atomicity; acceptance tests.  
**Non-goal:** This document does not implement structural operations, a formula parser, formula evaluation, or a spreadsheet-compatible UI.

## 1. Ownership and invariants

| Concern | Owner |
|---|---|
| Table/row/column/cell data identity, ordered row/column structure, deterministic structural operations, and structural change maps | Jabr Table |
| Formula reference syntax, reference AST, source formatting/rewriting, and reference diagnostics | Jabr Math |
| Coordinating edits across Table, Math reference bindings, Kernel revisions, dependency graph invalidation, and commit/rollback | Jabr-engine |
| Generic revision/snapshot/diagnostic primitives | jabr-kernel-api / Kernel |
| Authored document transaction, collaboration, undo/redo and durable Yjs persistence | Jabr-UI |

Dependency direction remains unchanged: Table does not depend on Math, Engine, Kernel, BlockSuite, or Yjs. Math does not depend on Table. Engine orchestrates adapters and transactions.

Mandatory invariants:

1. Stable table, row and column IDs survive position changes.
2. A cell's stable identity is the composite (table_id, row_id, column_id). The current Rust model has no independent cell UUID; do not add one unless a concrete requirement justifies it.
3. Position is not identity. A row or column index is a current coordinate only.
4. Table never parses or rewrites formula text.
5. Every successful structural operation returns a deterministic structural change set sufficient for Engine to update bindings and for Math to rewrite affected references.
6. A failed structural operation changes nothing.
7. Engine owns the cross-engine transaction and revision lifecycle. Table does not invent a second revision/snapshot system.
8. No reference may silently retarget to an unrelated cell after a destructive edit.

## 2. Reference classes and binding

### 2.1 A1 references

A1 references are positional references in authored formula syntax. Math parses them into reference AST nodes, including row/column absolute-vs-relative flags where the language supports those semantics. Engine binds each reference to the stable table/row/column identity that currently occupies the referenced coordinate.

After a structural edit, Engine supplies the structural change set and previous binding to Math. Math determines the corresponding new reference AST/address and formats the updated source. Engine commits the rewritten source and new binding together with the table edit.

- Table returns identity/index changes only; it does not edit source strings.
- Math owns AST transformation and source formatting; it does not inspect Table internals.
- Engine decides which formulas must be offered for rewrite and commits all affected changes atomically.
- Unaffected source text must remain byte-for-byte unchanged.
- If a target was deleted with no valid successor under the operation's rules, Math emits an explicit broken-reference node/diagnostic (for example, a #REF!-equivalent); it must never substitute zero, an empty cell, or the cell now occupying the old coordinate.

Absolute/relative markers control copy/fill behavior, not whether a structural edit is allowed to update a reference to preserve its target. The exact Math AST representation is owned by Math.

### 2.2 Structured identity references

A structured reference that resolves to a stable row, column, or cell identity follows that identity across moves and sorts. Deletion of its target produces an explicit unresolved/deleted-reference diagnostic. Renaming a display label does not change the stable identity; Engine/Math may update presentation text only through a deliberate source-rewrite operation.

### 2.3 Ranges

A range is a bounded ordered selection, not an unqualified list of values. Its resolved form retains stable table identity, endpoint identities and the range's membership policy.

For a rectangular A1 range, the default structural policy is:

- Insertion strictly between the current first and last included row/column expands the range to include inserted rows/columns.
- Insertion immediately before the first boundary or immediately after the last boundary does not expand the range.
- Deletion of members shrinks the range.
- If a boundary row/column is deleted, the boundary re-anchors to the nearest surviving member in the same direction, where one exists.
- If no members survive, the range becomes an explicit invalid/empty-reference outcome according to Math's range type; it must not silently become a different non-empty range.
- A whole-row or whole-column range includes future inserted members along that whole dimension.

For a move operation, range endpoints follow their stable boundary identities. The resulting range is the contiguous rectangle between their new positions; membership may therefore change when an endpoint moves. For a sort operation, the range remains at the same positional extent and contains the values now occupying those coordinates.

These rules apply independently to row and column dimensions. A range crossing multiple tables is invalid unless a later Math language contract explicitly introduces such a construct.

## 3. Structural change-set contract

Every structural operation returns both the resulting Table state and a deterministic StructuralChangeSet. The following is a logical schema, not a frozen Rust API:

    struct StructuralChangeSet {
        table_id: TableId,
        operation: StructuralOperationKind,
        old_row_order: Vec<RowId>,
        new_row_order: Vec<RowId>,
        old_column_order: Vec<ColumnId>,
        new_column_order: Vec<ColumnId>,
        row_index_map: IndexMap<RowId>,
        column_index_map: IndexMap<ColumnId>,
        affected_cells: Vec<CellRef>,
    }

    struct IndexMap<Id> {
        old_to_new: BTreeMap<Id, Option<usize>>,
        new_to_old: Vec<Option<usize>>,
        inserted: Vec<Id>,
        deleted: Vec<Id>,
    }

Contract requirements:

- Indices are zero-based in the Rust API. A1 rendering converts to one-based row/column labels in Math.
- old_to_new maps every pre-operation stable ID to its new zero-based index, or None if deleted.
- new_to_old maps each post-operation index to the corresponding pre-operation index, or None if newly inserted.
- Moved entities appear in both maps at their new positions; they are not classified as deleted and reinserted.
- inserted and deleted are stable IDs, deterministically ordered by resulting/pre-operation index respectively.
- Old and new order vectors make the operation auditable and allow independent validation of the maps.
- affected_cells is deterministic and contains the composite stable identities whose existence, coordinate, or cell membership changed. It may be represented compactly in the concrete API if the full list would be too large; any compact form must be lossless.
- A no-op operation returns an explicit no-op change set or a documented no-change result; it must not fabricate changed identities.
- Maps are scoped to exactly one table and one operation. They are not durable IDs and must not be reused against a later table state.

The Table crate must not include Kernel Revision or NodeId types in this structure. Engine associates the change set with the relevant Kernel revision and transaction identity.

## 4. Operation semantics

### 4.1 Insert rows or columns

- Insertion creates new stable row/column IDs; an ID already present in the table is rejected.
- Existing IDs and cell values are preserved.
- New row cells are initialized to Empty for every current column.
- A new column is initialized to Empty for every current row.
- The caller supplies an insertion position in the pre-operation coordinate space, in the inclusive range 0..=len.
- The change set reports inserted IDs and shifts all affected old indices.
- Insertion is atomic: invalid positions, duplicate IDs, invalid column definitions, or validation failures leave the table unchanged.

### 4.2 Delete rows or columns

- Deletion removes the selected stable IDs and their associated cell data.
- Remaining IDs and cell values are preserved; only their positions change.
- The change set maps deleted IDs to None and reports all shifted indices.
- Deleting an unknown ID fails without mutation.
- Deleting multiple IDs is treated as one operation against the original pre-operation state; duplicates are rejected.

### 4.3 Move rows or columns

- Move is a stable-identity-preserving reorder, not delete-plus-insert.
- The moved row/column and its cell values retain their IDs.
- The destination index is interpreted after removing the moved item from the original order, and is validated against the resulting order length.
- The initial API should support a single item move unless a multi-item requirement is demonstrated.
- The change set reports old and new positions for every affected ID, not only the moved ID.
- A move that produces the same order is a no-op.

For A1 formulas, bound references to moved entities follow the entity and Math rewrites the A1 address to its new coordinate. Structured identity references remain bound to the same IDs. Range endpoints follow their boundary IDs as specified in section 2.3.

### 4.4 Sort rows

Sort is a data permutation, distinct from moving a row as an explicit identity operation:

- Sorting reorders row IDs and their complete row records; it does not recreate rows or columns.
- The sort specification identifies one or more stable column IDs, direction, and deterministic tie-break behavior.
- Initial sort behavior is stable: rows equal under all sort keys retain their previous relative order.
- Empty, text, boolean, error and numeric ordering must be explicit in the sort specification. No implicit conversion between unlike types is allowed.
- Sorting must be deterministic across platforms and repeated runs.
- The change set reports the full old-to-new and new-to-old row permutation.
- Sorting does not change column order.

For A1 references, sorting is a positional data operation: external references remain bound to the same A1 coordinates and therefore observe the values now occupying those coordinates. Formula cells within sorted rows travel with their row records; their internal relative references are preserved according to Math's formula-copy/move semantics. Structured identity references continue to follow stable row/cell IDs.

The initial implementation may support sorting by one column only, but the contract and change-set format must not preclude multiple stable sort keys.

## 5. Atomic cross-engine rewrite protocol

A structural edit can affect table data, formula source, resolved references and the Engine dependency graph. It must be treated as one logical transaction:

1. UI submits a structural command against the current document/table revision.
2. Engine checks the expected revision and obtains the current Table snapshot/state.
3. Table validates and computes the candidate new state plus StructuralChangeSet without publishing a partial mutation.
4. Engine identifies affected formula bindings and asks Math to transform the relevant reference ASTs using the operation kind, previous bindings and change set.
5. Math returns rewritten source/AST and explicit broken-reference diagnostics. It does not commit Table or Kernel state.
6. Engine validates all candidate table and formula changes, updates dependency edges/ports and invalidates affected results.
7. Engine commits the table state, rewritten formula source, bindings, graph changes and revision as one transaction.
8. UI receives the committed result and publishes the corresponding collaborative document transaction through its established integration boundary.

If any validation or rewrite step fails, the operation is not partially committed. The UI/Engine must either commit the entire edit or leave the pre-edit state intact. A deliberate user-approved deletion that creates broken references may commit with explicit diagnostics; a parser/adapter failure may not.

Undo/redo must reverse/reapply the same logical transaction, including the table permutation and every rewritten formula source/binding. Replaying a transaction must be deterministic and must not generate new stable IDs for entities that are being restored.

## 6. Acceptance and regression tests

Required tests before structural operations are considered Engine-ready:

### Identity and maps

1. Insert one row at the beginning, middle and end; verify old IDs survive and maps are correct.
2. Insert one column at the beginning, middle and end; verify all existing cell values remain attached to stable column IDs.
3. Delete first, middle and last rows/columns; verify deleted IDs map to None and remaining cells retain identity.
4. Move rows/columns forward and backward; verify the complete permutation and stable IDs.
5. Sort rows with duplicate keys; verify stable tie ordering and identity preservation.
6. Verify old_to_new and new_to_old are inverse partial mappings for every surviving ID.
7. Verify invalid operations are atomic and do not mutate the original table.
8. Verify a no-op move/sort produces no false invalidation.

### References and ranges

9. Insert strictly inside a range; verify it expands and includes inserted rows/columns.
10. Insert immediately outside each range boundary; verify the range does not expand.
11. Delete a range member and a range boundary; verify shrink/re-anchor behavior.
12. Delete all range members; verify explicit invalid/empty outcome.
13. Move a referenced row/column; verify A1 source rewrites to preserve the moved target and structured references retain stable identity.
14. Move a range endpoint; verify the range follows its stable endpoint and recalculates its contiguous extent.
15. Sort rows; verify external A1 references remain coordinate-bound while structured references follow stable identities.
16. Sort rows containing formula cells; verify formula cells travel with their rows and internal references follow Math's move/copy rules.
17. Delete a referenced cell; verify an explicit broken-reference diagnostic, never zero/empty/retargeting.
18. Verify unaffected formula source is byte-for-byte unchanged.

### Transactions and graph integration

19. Force Math rewrite failure; verify no table/formula/graph partial commit.
20. Force graph update failure; verify rollback of the whole logical transaction.
21. Verify successful commit advances the Engine/Kernel revision exactly once.
22. Verify affected dependencies invalidate and unrelated dependencies remain current.
23. Verify undo/redo restores exact row/column order, identities, source text, bindings and results according to snapshot policy.
24. Replay the same transaction fixture and verify deterministic change sets and rewritten source.

### Property-based checks

- Random insert/delete/move sequences preserve uniqueness of all surviving IDs.
- Index maps agree with old/new order vectors.
- Every surviving cell composite reference resolves to exactly one cell.
- No deleted identity is silently rebound to a different entity.
- Sort is stable, deterministic and idempotent for a fixed comparator.
- Range resolution always returns an ordered, in-bounds selection or an explicit invalid outcome.

## 7. Implementation sequencing and exit criteria

Implement in this order:

1. Pure Table change-set types and map validation, without formula integration.
2. Atomic row insert/delete/move operations and tests.
3. Atomic column insert/delete/move operations and tests.
4. Deterministic row sort and ordering tests.
5. Math reference AST transformation API and golden rewrite fixtures.
6. Engine adapter coordinating Table change sets and Math rewrites.
7. Engine graph invalidation/transaction tests.
8. UI undo/redo and collaborative transaction integration tests.

Step 5 exit criteria: all pure Table structural operations and change-set tests pass; Math rewrite semantics are specified and covered by golden fixtures; Engine transaction tests prove atomic commit/rollback; reference behavior is explicit for insertion, deletion, move and sort. Do not mark step 5 complete based on Table-only CRUD tests.

## 8. Relationship to prior contracts

This document refines the range/reference placeholder in CELL-ENGINE-INTEGRATION-CONTRACT.md and the ownership boundaries in ARCHITECTURE.md. It is specification-only. It does not change the existing Rust Table API, which currently exposes row/column append and removal primitives but not a complete change-set, move, sort, or reference-rewrite API.

Step 6 (matrix ownership and grid-range conversion), step 7 (Kernel diagnostics/spans/envelopes and interchange hardening), and step 8 (CI/repository hygiene and release policy) remain separate follow-up contracts.
