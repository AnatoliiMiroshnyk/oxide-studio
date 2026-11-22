# Oxide Studio 0.2.0 implementation summary

## Release objective

Deliver the next vertical slice after 0.1.0: guided data editing, a carefully bounded SQLite schema editor, explicit transaction control, and user-visible protection around destructive guided operations.

## Code changes

| Area | Files | Implementation |
|---|---|---|
| Typed table reads | `src/db/schema.rs`, `src/db/mod.rs` | `TableData` now carries display rows, owned `rusqlite::types::Value` rows, and an identity per row. Metadata uses `PRAGMA table_xinfo`; generated columns remain visible, virtual-table hidden implementation columns are excluded. |
| CRUD and DDL | `src/db/mutations.rs` (new), `src/db/connection.rs`, `src/db/mod.rs` | Adds parameter-bound insert/update/delete, identifier quoting, guarded declared-type grammar, literal defaults, create/rename/add/drop schema operations, affected-row checks, and savepoint rollback on failure. |
| Row targeting | `src/db/schema.rs`, `src/db/mutations.rs` | Prefers declared primary keys (including composite keys). Uses an unshadowed hidden rowid alias only for eligible rowid tables without a primary key. |
| Transaction lifecycle | `src/db/connection.rs`, `src/app/mod.rs` | Uses SQLite autocommit state as the source of truth; supports DEFERRED, IMMEDIATE, EXCLUSIVE, COMMIT, ROLLBACK, and prevents replacing a connection while a transaction is active. |
| Guided UX | `src/app/mod.rs` | Adds insert/edit forms, NULL toggles, numeric and BLOB parsing, generated-column read-only state, row selection, typed destructive confirmations, create-table dialog, schema editor, and transaction controls. |
| Tests | `src/db/mutations.rs`, `src/db/schema.rs`, `src/db/connection.rs`, `src/app/mod.rs` | Adds database and parser tests for CRUD, composite keys, rowid fallback, generated columns, DDL validation, transaction/savepoint behavior, read-only protection, and typed value conversion. |

## Documentation and release files

- `README.md` — updated product capabilities, supported stack, usage, validation, and limitations.
- `CHANGELOG.md` and `RELEASE_NOTES.md` — 0.2.0 user-facing release summary.
- `docs/USER_GUIDE.md` — practical UI workflows for row, schema, and transaction operations.
- `docs/TRANSACTIONS_AND_EDITING.md` — detailed mutation and transaction contract.
- `docs/MIGRATION_0.1_TO_0.2.md` — source and operational upgrade guidance.
- `docs/ARCHITECTURE.md` — typed data path, identity derivation, savepoint, and transaction design.
- `docs/PRODUCT_REQUIREMENTS.md`, `docs/ROADMAP.md`, `docs/UI_UX_GUIDELINES.md` — updated scope and future work.
- `docs/SECURITY_MODEL.md` — write-path safeguards and remaining exposures.
- `docs/TEST_STRATEGY.md`, `docs/BUILD_VALIDATION.md`, `docs/DEVELOPMENT.md`, `docs/RELEASE_PROCESS.md` — quality and release gates.
- `docs/COMMIT_PLAN.md` — file-by-file proposed Git history.

## Intentional exclusions

This release does not implement generic table rebuilds, index/trigger GUI editors, transaction diff/undo, background DB worker, cancellation, streaming large exports, or multi-statement scripts. Raw Workbench SQL bypasses guided confirmations. The source-generation environment did not have the Rust toolchain; compilation and runtime validation remain required before publication.
