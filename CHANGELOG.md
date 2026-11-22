# Changelog

Notable changes to Oxide Studio are recorded here. Dates use ISO 8601 calendar dates.

## [0.2.0] - 2026-11-09

### Added
- Typed SQLite value retention for data browsing and editing, including distinct `NULL`, integer, real, text, and BLOB handling.
- Guided row insertion, update, and deletion with bound values.
- Primary-key-based row identity for single and composite keys, plus hidden-rowid fallback for eligible rowid tables without a declared primary key.
- Table creation with one or more primary-key columns.
- Table rename, column add/rename/drop, and table drop workflows.
- Per-operation SQLite savepoints for row and schema mutations.
- Explicit `DEFERRED`, `IMMEDIATE`, and `EXCLUSIVE` transaction controls with commit and confirmation-gated rollback.
- Destructive-operation confirmation dialogs that require typing `DELETE`, the target object name, or `ROLLBACK`.
- Editing/transaction documentation, migration guide, release notes, and expanded database tests.

### Changed
- Version and in-app product identity updated to `0.2.0`.
- Read-only sessions disable guided editing and transaction start; an SQL-created read transaction can still be ended.
- The data browser now exposes row selection and guided editing actions.
- Schema and security documentation now describe the 0.2 write paths and SQL Workbench boundary.

### Limitations
- Database operations remain synchronous on the UI thread.
- Raw SQL in the SQL Workbench is not sandboxed and bypasses guided destructive-action confirmations.
- Existing column type/constraint changes requiring a table rebuild, dedicated index/trigger editors, SQL parameter UI, import, backup/restore, and script execution are deferred.
- Source validation in the generation environment did not include Rust compilation or native GUI execution; see `docs/BUILD_VALIDATION.md`.

## [0.1.0] - 2026-11-07

### Added
- Native Rust desktop application shell with a dark egui interface.
- Open/create SQLite databases and read-only sessions.
- Schema explorer for tables, views, indexes, and triggers.
- Paginated table/view browsing and column metadata inspection.
- Single-statement SQL workbench, bounded results, and session query history.
- CSV export and SQLite integrity check.
- Demo SQL fixture, CI workflow, user/developer documentation, and repository templates.
