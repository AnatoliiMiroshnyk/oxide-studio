# User guide

## 1. First launch

Oxide Studio opens to a welcome screen. No database is created or modified until you open/create one or load the in-memory demo.

Choose one of:

- **Open database** — select an existing `.db`, `.sqlite`, or `.sqlite3` file for read/write access.
- **Open read-only** — open a file with SQLite's read-only connection flag and enable `PRAGMA query_only`. Ordinary write statements are rejected. This is a safety mode for normal use, not a sandbox for untrusted SQL: do not execute scripts from unknown sources, and do not attempt to override connection pragmas.
- **Create database** — choose a file path; SQLite creates an empty database file if it does not already exist.
- **Load demo** — open a sample database in memory. It contains `customers`, `orders`, indexes, and a `customer_order_totals` view. Changes to the demo disappear when the app closes or another database is opened.

## 2. Explore the schema

The left explorer groups schema objects into Tables, Views, Indexes, and Triggers. Use the filter box to narrow names. Select a table or view to open its data; select an index or trigger to inspect its stored SQL definition.

Use **Structure** from the data browser to inspect a table/view's column ordinal, name, declared type, `NOT NULL` setting, default value, and primary-key order.

## 3. Browse table data

The data browser reads up to 200 records per page. Use **Next** and **Previous** to move through the result. This is a read-only grid in v0.1.0. To change data, use SQL statements in the SQL Workbench.

A value displayed as `NULL` is SQLite NULL. BLOBs are represented by a size marker and cannot be previewed in this prototype.

## 4. Run SQL

Open **SQL Workbench** from the left workspace navigation.

1. Type one SQL statement in the editor.
2. Select **Run query** or press `Ctrl + Enter`.
3. Read the output grid, row count, affected-row count, and elapsed time.
4. Use **Export CSV** to save the most recent tabular query result.

The editor prepares a single statement per run. It does not expose bind-parameter entry yet. If you need multiple statements, execute them one at a time. A statement returning rows is limited to 1,000 rows in the displayed result; each text cell is capped at 16,384 characters for display. The current prototype executes on the UI thread; avoid long-running or unbounded write operations.

### Example queries

```sql
-- List user-defined tables and views
SELECT type, name
FROM sqlite_schema
WHERE name NOT LIKE 'sqlite_%'
ORDER BY type, name;

-- Inspect customer orders
SELECT c.name, o.order_total, o.status, o.ordered_at
FROM customers AS c
JOIN orders AS o ON o.customer_id = c.id
ORDER BY o.ordered_at DESC;

-- Preview a query plan
EXPLAIN QUERY PLAN
SELECT * FROM orders WHERE customer_id = 1;
```

## 5. Query history

The history view lists statements executed during the current application session. Select **Load into editor** to reuse a statement. History is not persisted across application restarts in v0.1.0.

## 6. Integrity check

Choose **Integrity check** in the explorer. Oxide runs `PRAGMA integrity_check` and displays the returned status. A successful `ok` result is a useful SQLite consistency signal, but it is not a replacement for backups or application-level validation.

## 7. Export query results

Run a query that returns columns, then choose **Export CSV**. The CSV contains the column headers and currently materialized result rows. If the query reached the 1,000-row display cap, the CSV contains only those displayed rows. A full streaming export is planned for a later release.

## 8. Safe operating practices

- Make a backup before schema changes or bulk updates.
- Open valuable or untrusted databases read-only when possible.
- Review `UPDATE` and `DELETE` statements before running them; add a `WHERE` clause when appropriate.
- Use an explicit transaction for groups of writes when you need all-or-nothing behavior:

```sql
BEGIN IMMEDIATE;
UPDATE customers SET country = 'Ukraine' WHERE id = 2;
COMMIT;
```

If a statement fails within a manually started transaction, inspect the connection state and issue `ROLLBACK` before continuing. The prototype does not offer a transaction-state indicator or rollback toolbar yet.

## 9. Known limitations

See [Roadmap](ROADMAP.md) for the full list. Most notably: no inline CRUD grid, no import wizard, no SQL autocomplete/highlighting, no multi-tab editor, no database attachment, no backup wizard, no query cancellation, and no persistent workspace settings.
