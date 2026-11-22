# Product roadmap and SQLite desktop-client capability map

A checked item means the guided workflow is present in the 0.2.0 source snapshot; it does not mean the feature has passed cross-platform release validation or a security audit.

## 0.1.0 — foundation prototype

- [x] Native Rust/egui desktop application.
- [x] Open/create SQLite files and read-only sessions.
- [x] Bundled SQLite engine.
- [x] Explorer for tables, views, indexes, and triggers.
- [x] Paginated table/view browser and schema inspector.
- [x] One-statement SQL Workbench, bounded results, and session query history.
- [x] CSV export and SQLite integrity check.

## 0.2.0 — data editing and transaction management

- [x] Preserve owned SQLite values in table pages: NULL, integer, real, text, and BLOB.
- [x] Distinguish NULL from empty text in edit forms; parse numeric affinities and hex BLOB input.
- [x] Insert, update, and delete rows with parameterized values.
- [x] Target rows by single/composite primary key and hidden rowid fallback where safe.
- [x] Create tables with typed columns and primary-key definitions.
- [x] Rename tables and columns; add and drop columns; drop tables.
- [x] Run each guided mutation in a savepoint and preserve outer transaction state on errors.
- [x] Start DEFERRED, IMMEDIATE, or EXCLUSIVE transactions; commit explicitly; confirm rollback.
- [x] Require typed confirmation for guided row/table/column deletion and rollback.
- [x] Disable guided write features in read-only sessions.
- [ ] Dedicated index and trigger creation/editing forms.
- [ ] Table-rebuild migration UI for existing column type/nullability/constraint changes.
- [ ] Transaction change preview and undo journal beyond SQLite rollback.

## 0.3 — import/export and query workflow

- [ ] CSV import wizard with delimiter, encoding, header, NULL, and type mapping options.
- [ ] JSON import/export and SQL dump export/restore.
- [ ] Export selected rows and selected columns; streaming exports with byte budgets.
- [ ] Multiple SQL tabs, saved queries, snippets, and persistent history.
- [ ] SQL syntax lexer, highlighting, autocomplete, formatter, bracket matching, and typed bind parameters.
- [ ] Search/filter/sort in table data; keyset pagination and selected-column navigation.
- [ ] Multi-statement script execution with statement-level results and explicit transaction policy.
- [ ] Query-plan visualizer and selected-statement execution.

## 0.4 — reliability and advanced SQLite

- [ ] Database actor thread, cancellation, progress reporting, and UI responsiveness guarantees.
- [ ] Attach/detach databases and cross-database browsing.
- [ ] Backup/restore using SQLite backup API, including verification and recovery UX.
- [ ] VACUUM, ANALYZE, REINDEX, WAL checkpoint, and pragma inspector.
- [ ] Foreign-key browser and validation workflow.
- [ ] BLOB inspector with image/text/hex previews and configurable byte limits.
- [ ] Database diff and migration preview.
- [ ] ER diagram and relation navigation.
- [ ] Virtual table / FTS support diagnostics.

## 0.5 — professional workflows

- [ ] Session persistence, recent databases, and workspace restoration.
- [ ] Dark/light/high-contrast themes and configurable keyboard shortcuts.
- [ ] Accessibility audit and keyboard-only workflows.
- [ ] Plugin API with sandboxed capabilities and explicit permissions.
- [ ] Localization (English first, then Ukrainian and other languages).
- [ ] Opt-in crash diagnostics with local redaction controls.
- [ ] Signed installers and platform-specific update channel.

## 1.0 — release gates

- [ ] Feature coverage matrix approved against target workflows.
- [ ] Cross-platform release matrix green on supported Windows, macOS, and Linux targets.
- [ ] Fuzzing for SQL-to-display conversion, file paths, import pipeline, and data editor value conversion.
- [ ] Security review and dependency/license audit.
- [ ] Performance budgets met on reference hardware and large fixtures.
- [ ] Backup, rollback, destructive-operation, and migration-preview UX tested.
- [ ] User documentation, release notes, support policy, and recovery guidance published.

## Feature parity snapshot

| Capability area | SQLite desktop-client expectation | Oxide 0.2.0 | Next target |
|---|---|---|---|
| Connection/database | Open/create/read-only/multiple workspaces | Single connection, open/create/read-only | 0.5 |
| Schema browser | Tables, views, indexes, triggers | Basic inspection | 0.3/0.4 |
| Data grid | View, CRUD, sort, filter, search | Paginated view + guided CRUD | 0.3 |
| Row identity | PK, composite PK, rowid handling | PK/composite PK + hidden rowid fallback | 0.3 hardening |
| SQL editor | Syntax, completion, tabs, params, history | Single statement + session history | 0.3 |
| Transactions | Explicit begin/commit/rollback and safe failure handling | Modes + savepoints for guided mutations | 0.3 |
| Schema changes | Create/alter/drop with preview | Create, rename, add/drop column, drop table | 0.3 migration editor |
| Import/export | CSV, JSON, SQL, selected rows | Query CSV export only | 0.3 |
| Diagnostics | Integrity, FK check, plan, pragmas | Integrity check | 0.4 |
| Backup/restore | Consistent backup and recovery | Not implemented | 0.4 |
| Visualization | ER diagram and relations | Not implemented | 0.4 |
| Extensibility | Plugins and custom functions | Not implemented | 0.5 |
| Security | Least privilege, safe defaults, recoverability | Read-only mode, bound guided writes, confirmations | 0.4/1.0 |

## Prioritization criteria

Prioritize work by user workflow value, data-loss risk, architectural leverage, and testability. Do not ship import, table rebuilds, or bulk editing until preview, backup, rollback, row identity, and failure recovery are explicit. The database actor should land before long-running import/export, multi-statement jobs, and cancellation.
