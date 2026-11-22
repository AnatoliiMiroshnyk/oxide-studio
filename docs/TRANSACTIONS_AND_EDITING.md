# Data editing and transaction workflows

This document defines how Oxide Studio 0.2.0 writes to a database, how a record is identified, and which safeguards apply.

## 1. Write-path rules

The guided editor does not construct SQL values from display text. Each row is loaded into owned SQLite values (`NULL`, integer, real, text, or BLOB). A mutation then binds those values through rusqlite parameters. Table and column names are escaped as SQLite identifiers separately from bound values.

A row edit changes only fields whose input or NULL state differs from the original value. This avoids rewriting untouched BLOBs or converting every cell to text. The editor distinguishes an empty string from SQL `NULL`. For BLOB input, use the `hex:` prefix followed by an even number of hexadecimal characters, for example `hex:00A1FF`.

Declared type affinity controls parsing:

| Declared type affinity | Guided editor behavior |
|---|---|
| Type containing `INT` | Parse a signed 64-bit integer; reject invalid input |
| `REAL`, `FLOAT`, `DOUBLE` | Parse a finite floating-point value |
| `NUMERIC`, `DECIMAL`, `BOOLEAN` | Prefer integer, then finite real; otherwise preserve text |
| `BLOB` | Require `hex:` binary input |
| Other / no declared type | Preserve entered text |
| NULL checkbox | Bind SQL NULL regardless of text-field content |

SQLite remains the final authority for NOT NULL, UNIQUE, CHECK, foreign-key, generated-column, and trigger constraints. An operation rejected by SQLite is reported without claiming a write succeeded.

## 2. Row identity policy

The browser prefers declared primary keys. Composite keys are ordered by SQLite's `primary_key_order` metadata and all key values are bound in the mutation predicate. The predicate uses `IS` comparisons so SQLite NULL semantics are deterministic for unusual schemas.

For an ordinary rowid table with no declared primary key (virtual tables are excluded from this fallback), the browser chooses the first unshadowed hidden alias among `rowid`, `_rowid_`, and `oid`. The internal rowid is not shown as a normal data column. It can be used for update/delete while browsing that loaded page.

Views are browse-only. Tables where neither a declared primary key nor an available hidden rowid can be safely identified are also browse-only for edit/delete. A hidden rowid is not a durable business identifier and can be affected by table rebuilds or `VACUUM`; declare a primary key for long-lived identity. If an external process changes a database after a row was loaded, refresh the page before editing.

## 3. Insert, update, and delete

### Insert

1. Select a base table in the Explorer.
2. Choose **+ New row**.
3. For each column, choose whether to include it, enter a value, or explicitly select NULL.
4. Leave columns unchecked when you want SQLite defaults or generated values to apply.
5. Select **Insert row**. The UI reports the number of rows inserted and refreshes the page.

An insert with no included values uses `INSERT ... DEFAULT VALUES`. Primary-key columns and columns with a stored default are initially omitted from the insert form. Required columns without defaults must be supplied.

### Update

1. Select a row number in the first grid column.
2. Choose **Edit selected**.
3. Change one or more values. Unmodified fields are not included in the UPDATE statement.
4. Choose **Save changes**. SQLite validates the resulting row, and the grid refreshes.

### Delete

1. Select a row and choose **Delete selected**.
2. Review the preview and permanent-action warning.
3. Type `DELETE` exactly and choose **Confirm destructive action**.

The delete statement targets the row's declared key or internal rowid identity, not a broad unfiltered table delete.

## 4. Schema operations

The Schema Inspector provides the following guided operations for writable base tables:

- **Create table:** define one or more columns and mark one or more primary-key columns. SQLite validates duplicate names and constraints.
- **Rename table:** rename a table while allowing SQLite to update supported schema references.
- **Add column:** add a declared type, optional NOT NULL constraint, and optional literal default (including an empty string when the Default checkbox is enabled). Defaults are treated as literal values, not SQL expressions.
- **Rename column:** use SQLite's `ALTER TABLE ... RENAME COLUMN` support.
- **Drop column:** type the exact column name in the confirmation dialog. SQLite may reject columns referenced by primary keys, indexes, constraints, generated expressions, triggers, or views.
- **Drop table:** type the exact table name in the confirmation dialog. Foreign-key enforcement and dependencies can cause SQLite to reject the operation.

Every guided schema mutation runs inside a named SQLite savepoint. A failed statement attempts `ROLLBACK TO` and `RELEASE`, protecting an outer user transaction from a partially applied guided operation.

This version does not rebuild tables to modify an existing column's affinity, nullability, collation, generated expression, or arbitrary constraints. It does not have dedicated forms for index or trigger authoring. These remain explicit SQL Workbench tasks until a migration-preview workflow exists.

## 5. Explicit transactions

The top bar provides a transaction mode selector and buttons:

| Mode | SQLite behavior | Typical use |
|---|---|---|
| DEFERRED | Acquires locks as reads/writes require them | Read-first work or default transactional workflows |
| IMMEDIATE | Starts a write transaction immediately | Guided data editing; default selection |
| EXCLUSIVE | Requests the strongest database lock available under the journal mode | Controlled maintenance when blocking other connections is acceptable |

Select **Begin transaction** to issue `BEGIN <mode>`. Guided row and schema changes and SQL Workbench statements execute on this same connection and remain pending until **Commit**. **Rollback…** opens a confirmation dialog; type `ROLLBACK` to discard all uncommitted work.

Additional behavior:

- A second explicit transaction cannot be started while one is active.
- The application blocks switching to another database while a transaction is active.
- The connection is rolled back when the application closes or the connection is dropped, but users should explicitly commit or roll back before exiting.
- A statement-level SQL `COMMIT` or `ROLLBACK` issued from the Workbench is reflected by SQLite's actual autocommit state; the transaction badge reads from the connection instead of maintaining an independent flag.
- If commit fails (for example because a deferred constraint fails), SQLite keeps the transaction active where applicable; inspect the error and decide whether to correct the data or roll back.

## 6. Savepoints and error handling

Guided mutations run inside `SAVEPOINT oxide_studio_mutation`. Savepoints compose with an explicit outer transaction: releasing a mutation savepoint commits only that operation into the outer transaction, not to the database file. If an operation fails, Oxide requests rollback to the savepoint and releases it. A rollback failure is logged as a diagnostic. If releasing a savepoint fails (for example, a deferred constraint is checked at the outermost release), the helper also attempts rollback before returning the release error.

Single SQL statements are also atomic under SQLite's own statement semantics. Arbitrary multi-statement SQL scripts are not supported in the Workbench 0.2.0.

## 7. Read-only sessions and advanced SQL

Open a database in read-only mode to disable guided row/schema writes and transaction start. The connection also uses SQLite read-only flags and `PRAGMA query_only=ON`. A read-only transaction started manually in the Workbench can still be ended using the toolbar controls or SQL.

The SQL Workbench is intentionally an advanced direct-execution surface. Destructive statements typed there do not pass through the guided confirmation dialogs. There is no SQL parser or policy layer that guarantees a statement is non-destructive. Use read-only mode when inspecting an untrusted database, and keep verified backups before executing write SQL.
