# Product roadmap and SQLiteStudio capability map

This roadmap distinguishes implemented prototype capabilities from product targets. A checked item means it exists in the v0.1.0 source, not that it has been independently audited or validated on every platform.

## 0.1.0 — foundation prototype

- [x] Native desktop application (Rust + egui).
- [x] Open/create SQLite file; read-only open mode.
- [x] Bundled SQLite engine.
- [x] Explorer for tables, views, indexes, triggers.
- [x] Table/view browser with paging.
- [x] Column metadata and stored SQL definition inspection.
- [x] Single-statement SQL execution.
- [x] Bounded result grid and affected-row count.
- [x] CSV export of the last query result.
- [x] Session-only query history.
- [x] Integrity check.
- [x] Demo database and automated unit tests for database primitives.

## 0.2 — database editing foundation

- [ ] Typed cell values and NULL-aware data grid.
- [ ] Insert, edit, duplicate, and delete row dialogs.
- [ ] Safe write operations with preview, confirmation, and transaction rollback on error.
- [ ] Primary-key and rowid strategy, including composite keys and `WITHOUT ROWID` tables.
- [ ] Table/column create, alter, rename, and drop workflows.
- [ ] Index and trigger creation/editing UI.
- [ ] Multi-statement SQL scripts with statement-level execution results.
- [ ] Bind-parameter editor with typed values.
- [ ] SQL selection execution and execution plan (`EXPLAIN QUERY PLAN`).

## 0.3 — import/export and query workflow

- [ ] CSV import wizard with delimiter, encoding, header, null, and type mapping options.
- [ ] JSON import/export.
- [ ] SQL dump export and restore workflow.
- [ ] Export selected rows and selected columns.
- [ ] Multiple SQL tabs, saved queries, snippets, and persistent history.
- [ ] SQL lexer, syntax highlighting, autocomplete, formatter, and bracket matching.
- [ ] Search/filter/sort in table data; column pinning and width persistence.

## 0.4 — reliability and advanced SQLite

- [ ] Database actor thread, cancellation, progress reporting, and UI responsiveness guarantees.
- [ ] Attach/detach databases and cross-database browsing.
- [ ] Backup/restore using SQLite backup API.
- [ ] `VACUUM`, `ANALYZE`, `REINDEX`, WAL checkpoint, and pragma inspector.
- [ ] Foreign-key browser and validation.
- [ ] BLOB inspector with image/text/hex previews and configurable byte limits.
- [ ] Database diff and schema migration preview.
- [ ] ER diagram and relation navigation.
- [ ] Virtual table / FTS support diagnostics.

## 0.5 — professional workflows

- [ ] Session persistence and recent databases.
- [ ] Theme system (dark/light/high-contrast) and configurable shortcuts.
- [ ] Accessibility audit and keyboard-only workflows.
- [ ] Plugin API with sandboxed capabilities and explicit permissions.
- [ ] Localization (English first, then Ukrainian and other languages).
- [ ] Crash reporting opt-in; local diagnostic bundle with redaction.
- [ ] Signed installers and platform-specific update channel.

## 1.0 — release gates

- [ ] Feature coverage matrix approved against target SQLiteStudio workflows.
- [ ] Cross-platform release matrix green (Windows, macOS, Linux).
- [ ] Fuzzing for SQL-to-display conversion, file parsing, and import pipeline.
- [ ] Security review and dependency/license audit.
- [ ] Performance budgets met on reference low-end hardware and large fixtures.
- [ ] Backup, rollback, and destructive-operation UX tested.
- [ ] User documentation, release notes, support policy, and migration guide published.

## Feature parity checklist

| Capability area | SQLite desktop-client expectation | Oxide v0.1.0 | Target |
|---|---|---:|---|
| Connection/database | Open/create, multiple files, read-only | Partial | 0.2/0.5 |
| Schema browser | Tables, views, indexes, triggers | Basic | 0.2 |
| Data grid | View, sort, filter, edit, insert, delete | View + paging | 0.2/0.3 |
| SQL editor | Syntax, completion, tabs, params, history | Basic editor + history | 0.3 |
| SQL execution | SELECT/DML/DDL/PRAGMA, scripts, cancel | One statement/run | 0.3/0.4 |
| Import/export | CSV, JSON, SQL, selected rows | Query CSV export | 0.3 |
| Schema changes | Create/alter/drop objects | Inspect only | 0.2 |
| Diagnostics | Integrity, FK check, query plan, pragma | Integrity check | 0.4 |
| Backup/restore | Consistent backup and recovery | Not implemented | 0.4 |
| Visualization | ER diagram and relations | Not implemented | 0.4 |
| Extensibility | Plugins, custom functions | Not implemented | 0.5 |
| Security | Read-only mode, safe defaults, least privilege | Read-only open; local-only | 0.4/1.0 |

## Prioritization criteria

Prioritize features by user workflow value, data-loss risk, architectural leverage, and testability. Editing and import should not be shipped until transaction handling, operation preview, rollback behavior, and row identity rules are explicit. The database actor should land before long-running import/export or query cancellation work.
