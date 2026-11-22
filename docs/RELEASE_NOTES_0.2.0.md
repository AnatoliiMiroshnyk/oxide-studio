# Oxide Studio 0.2.0 — Data editing and transaction management

Oxide Studio 0.2.0 expands the native SQLite workspace from inspection and querying into guided row editing, a bounded schema editor, and explicit transaction management. The release keeps the Rust + egui desktop architecture and builds its guided write paths around SQLite typed values, parameter binding, identifier quoting, and per-operation savepoints.

## Highlights

- **Row CRUD:** insert, update, and delete table rows from guided forms. Cell values are bound as SQLite parameters; unchanged values are not rewritten.
- **Typed values:** NULL remains distinct from empty text. Integer and real affinities are parsed, and BLOB fields can be edited with hexadecimal input.
- **Safe row targeting:** declared primary keys and composite primary keys are preferred. An unshadowed hidden rowid alias is used only when a normal rowid table has no declared key and SQLite exposes a safe alias.
- **Generated columns:** generated values are visible but read-only in row forms. Virtual-table hidden implementation columns are omitted from the normal grid.
- **Schema editor:** create tables with primary-key definitions; rename tables and columns; add/drop columns; drop tables.
- **Transaction controls:** start DEFERRED, IMMEDIATE, or EXCLUSIVE transactions, commit explicitly, and roll back only after a typed confirmation.
- **Mutation savepoints:** each guided write is isolated in a savepoint, including inside an explicit user transaction. A failed form submission should not discard earlier writes in that outer transaction.
- **Destructive-operation UX:** row deletion requires `DELETE`; table/column drops require the exact object name; transaction rollback requires `ROLLBACK`.
- **Read-only mode:** guided write workflows and transaction start are disabled for a read-only connection; the UI can still end a read transaction started manually in the Workbench.

## Upgrade and compatibility

No Oxide-specific migration is applied to user database files. Oxide Studio 0.2.0 opens the same SQLite database format as 0.1.0 and does not rewrite schemas at startup. Review `docs/MIGRATION_0.1_TO_0.2.md` before upgrading the source tree.

## Important limitations

- Database operations still run synchronously on the UI thread; a long query or lock wait can temporarily block the window.
- The SQL Workbench is a direct SQL interface. It executes one statement per run and does **not** intercept destructive SQL for confirmation. Treat raw SQL as a deliberate advanced operation.
- Schema editing is intentionally limited to table creation, table/column rename, add/drop column, and drop table. Changing an existing column's type, collation, nullability, or arbitrary constraints requires a table-rebuild workflow and is not included.
- Dedicated index/trigger editors, import wizards, backup/restore, query cancellation, multiple SQL tabs, syntax highlighting, autocomplete, saved workspaces, and ER diagrams are not included.
- Hidden rowid values are implementation identities, not durable business keys. Prefer declared primary keys.
- This source snapshot was not compiled or tested with Cargo in the generation environment. Run the full validation commands in `docs/BUILD_VALIDATION.md` before publishing binaries.

## Installation

Download the source archive, extract it, install Rust `1.95.0`, and follow the build steps in `README.md`. This source bundle does not contain prebuilt platform binaries.

## Data-safety recommendation

Make a verified backup before editing important databases. Test destructive operations and transaction workflows on disposable copies first. See `docs/TRANSACTIONS_AND_EDITING.md` and `docs/SECURITY_MODEL.md` for the full behavior and threat boundaries.
