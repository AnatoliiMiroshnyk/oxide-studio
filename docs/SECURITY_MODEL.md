# Security model

## Scope and assumptions

Oxide Studio is a local desktop application intended to open SQLite database files selected by the user. The application is not a database server and does not intentionally transmit database contents over a network. The current prototype is not security-audited and must not be treated as a hardened forensic or enterprise product.

## Assets

- Database file contents and schema.
- User-entered SQL and its results.
- Local exports created by the user.
- File paths and current database session state.

## Threats

| Threat | Prototype control | Remaining exposure |
|---|---|---|
| Accidental writes to a valuable DB | Explicit read-only open action | Default open is read/write; no write confirmation or transaction UI |
| SQL injection through app-generated schema queries | Bound values for query filters; quoted identifiers for table names | SQL workbench intentionally accepts arbitrary SQL |
| Malformed/corrupt database | SQLite parser; errors surfaced through `anyhow` | No process isolation or fuzz-tested file handling |
| Huge query output | 1,000-row cap and 16,384-character text cap per cell | A total-result memory budget and query cancellation are not yet implemented |
| Huge table scan | 200-row pages | `OFFSET` can become expensive on large tables; no keyset navigation |
| BLOB memory pressure | BLOBs are summarized by byte length | BLOB-specific preview/export pipeline not implemented |
| UI freeze from long query | None beyond row cap | Query executes synchronously on UI thread; no cancel/progress |
| Sensitive diagnostic logging | Errors only; no query text is intentionally logged | `tracing` configuration should be reviewed before production |
| Data loss from application crash | SQLite journaling behavior | No integrated backup/restore workflow or recovery UX |

## Implemented safeguards

1. **Read-only mode:** opens the selected database with `SQLITE_OPEN_READ_ONLY` and initially enables `PRAGMA query_only=ON`; the UI displays a read-only marker. This protects the selected file in normal usage, but the SQL console is not a security sandbox: an expert user can change connection pragmas or attach other files.
2. **Identifier quoting:** application-generated table reads quote identifiers by doubling embedded double quotes.
3. **Bound pagination values:** `LIMIT` and `OFFSET` are passed as SQLite parameters.
4. **Bundled SQLite:** avoids silently binding to an arbitrary system SQLite build/version.
5. **Busy timeout:** SQLite lock waits are bounded to five seconds.
6. **Foreign keys:** `PRAGMA foreign_keys = ON` is enabled per connection.
7. **Bounded output:** query output is capped at 1,000 rows, table browsing at 200 rows per fetch, and text display at 16,384 characters per cell. A total byte budget is still required for production.
8. **BLOB summarization:** raw BLOB payload is not copied into the textual grid.

## SQL execution warning

The SQL editor is intentionally powerful. A statement such as `DROP TABLE`, `DELETE FROM`, or `UPDATE` without a restrictive `WHERE` clause can permanently modify data. Oxide does not block destructive SQL in the workbench. Users should open a read-only session or make a verified copy before experimenting with important databases.

## Production security requirements

Before 1.0:

- Add explicit query confirmation for destructive statements, with a setting for expert users.
- Add transactions and rollback UI for grid edits and import operations.
- Add a database backup/snapshot action before schema migrations and destructive import.
- Add bounded channels, result byte budgets, and cancellation in the DB actor.
- Avoid logging SQL text, bound values, file contents, or returned rows unless explicit diagnostic redaction is enabled.
- Add dependency auditing, license compliance checks, and SBOM generation.
- Fuzz SQL value conversion, CSV export, import parsers, and schema introspection.
- Add tests for symlinks, unusual filenames, read-only files, locked databases, corrupt DB files, WAL sidecars, and `WITHOUT ROWID`/virtual tables.
- Define plugin capabilities and sandboxing before allowing third-party code execution.
- Complete a threat-model review and independent security assessment.

## Disclosure

Report vulnerabilities privately to the maintainers using the contact channel configured in the GitHub repository. Do not include production database contents or credentials in public issue reports.
