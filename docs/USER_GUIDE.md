# User guide

This guide covers Oxide Studio 0.2.0. The application is a local desktop client for SQLite databases.

## 1. Open or create a database

Use **Open database** for an existing `.db`, `.sqlite`, or `.sqlite3` file. Use **Open read-only** when you only need to inspect the file. Use **New database** to create a new local database file. **Load demo** creates a temporary in-memory database populated from `assets/sample/demo.sql`; it is not written to disk.

A read-only session is marked in the header. Guided data/schema editing and transaction start are disabled. If you manually start a read-only transaction in the SQL Workbench, use the visible Commit/Rollback controls or SQL to end it before switching databases. The connection uses SQLite read-only open flags plus `PRAGMA query_only=ON`.

## 2. Explore the schema

The Explorer lists tables, views, indexes, and triggers. Use the filter field to narrow objects by name. Select an object to open its data or structure view. The structure view reports column position, declared type, NOT NULL state, default value, primary-key order, and stored SQL definition.

Use the **+** button beside the Explorer refresh action to create a new table. A new table can have one or more primary-key columns. SQLite validates the final definition.

## 3. Browse and edit rows

Select a table. The data browser fetches up to 200 records per page; use **Next** and **Previous** to page through the table. Select a row using its row number in the first grid column.

### Insert

Choose **+ New row**. The editor shows every column and its declared type. Uncheck **Include** to omit a column and let SQLite use a default or generated value. Check **NULL** to bind SQL NULL; an empty text field with NULL unchecked is an empty string. BLOB values use `hex:` followed by hexadecimal bytes.

### Update

Select a row, choose **Edit selected**, and change only the intended fields. The editor binds only changed fields; untouched values, including BLOB payloads, are retained as their original SQLite values. Press **Save changes** to apply the update.

### Delete

Select a row and choose **Delete selected**. Review the row preview, type `DELETE`, and confirm. The delete uses the row's identity and affects at most one row.

### Which rows can be edited?

Rows are identified by a declared primary key, including composite keys. Ordinary rowid tables without a declared primary key may use an available hidden `rowid`, `_rowid_`, or `oid` alias. The alias is hidden from the normal columns. Views and tables without a safe identity remain browse-only for update/delete. A declared primary key is recommended; hidden rowids are not durable business identifiers.

## 4. Edit table structure

Open **Structure** for a base table. The guided Schema Editor supports:

- Rename table.
- Add a column with a declared type, optional NOT NULL, and an optional literal default (enable the Default checkbox to use even an empty-string default).
- Rename a column.
- Drop a column after typing its exact name.
- Drop a table after typing its exact name.

SQLite may reject an operation when columns participate in constraints, indexes, generated expressions, triggers, or dependent views. The application reports the database error and rolls back the failed guided schema mutation using a savepoint.

Changing an existing column's type, collation, nullability, or arbitrary constraints requires a table rebuild and is not included in this version. Dedicated index/trigger forms are also deferred. Carefully reviewed SQL can be used in the Workbench for advanced tasks, but it bypasses the guided confirmation dialogs.

## 5. Transactions

The top bar includes a transaction mode selector:

- **Deferred:** starts a transaction without immediately reserving the write lock.
- **Immediate:** requests the write transaction at start; this is the default selection.
- **Exclusive:** requests the strongest lock available for the active journal mode.

Choose **Begin transaction** before a group of writes. All guided row/schema changes and SQL Workbench statements run on the same connection until committed. Choose **Commit** to persist the transaction. Choose **Rollback…**, type `ROLLBACK`, and confirm to discard every uncommitted change.

The app prevents switching databases while a transaction is active. Closing the application or dropping the connection causes SQLite to roll back an uncommitted transaction, but users should explicitly finish the transaction before exiting. See [Data editing and transaction workflows](TRANSACTIONS_AND_EDITING.md) for implementation details and edge cases.

## 6. SQL Workbench

The Workbench executes one SQL statement per run. `Ctrl + Enter` also runs the editor contents. Results are capped at 1,000 rows; cell text display is capped at 16,384 characters. Statements that do not return columns display their affected-row count. Query history is retained for the current session only.

The Workbench is a direct SQL interface, not a sandbox. It does not parse destructive statements to show confirmation dialogs and does not provide a bind-parameter editor or multi-statement script runner. Review write statements, especially `DELETE`, `DROP`, `UPDATE`, and schema-changing statements, before running them.

Example:

```sql
SELECT c.name, o.status, o.ordered_at
FROM customers AS c
JOIN orders AS o ON o.customer_id = c.id
ORDER BY o.ordered_at DESC;
```

## 7. Integrity check and CSV export

Choose **Integrity check** in the Explorer to run `PRAGMA integrity_check`. A result of `ok` is a SQLite consistency signal, not a substitute for application-level validation or backup.

Run a tabular SQL query and choose **Export CSV** to save the currently materialized rows. If the result is capped, the CSV contains only the displayed result set. Streaming full-result export is planned for a later release.

## 8. Safe operating practices

- Back up important files before editing records or changing schema.
- Test workflows on a copy of a production database first.
- Use read-only mode when inspecting an untrusted or valuable database.
- Use explicit transactions for related writes and inspect the connection's transaction state before moving on.
- Prefer declared primary keys over hidden rowids.
- Remember that raw SQL bypasses guided destructive-action confirmations.
- Never attach production database files, exported customer data, or secrets to a public issue.

## 9. Current limitations

The UI and database operations remain synchronous; long-running queries can temporarily block the window. There are no import wizards, backup/restore UI, multiple SQL tabs, syntax highlighting, autocomplete, saved queries, query cancellation, database actor thread, persistent workspace settings, or ER diagram. See [Roadmap](ROADMAP.md).
