# Test strategy

## Principles

- Test database logic independently of egui.
- Use in-memory databases for fast behavior tests and temporary files for open/read-only/file lifecycle tests.
- Treat value fidelity, key targeting, atomicity, and transaction state as data-safety requirements.
- Do not use example SQL strings as a substitute for negative and failure-path tests.

## Test layers

| Layer | Scope | Examples | 0.2.0 coverage target |
|---|---|---|---|
| Unit | Small pure functions | Identifier quoting, type parsing, BLOB hex decoding, display limits | Implemented tests in schema/mutations/app value parsing |
| Database integration | Real SQLite behavior | CRUD, composite keys, rowid fallback, read-only protection, DDL, transaction commit/rollback | Implemented in-memory/temp-file unit tests |
| Failure/atomicity | Write failure behavior | Constraint failures inside savepoints, rollback within outer transaction, unsupported DROP COLUMN | Database savepoint/transaction tests included; broaden fixtures before 1.0 |
| UI interaction | Modal state and user workflows | Insert, edit, delete confirmation, schema confirmation, transaction badge | Manual GUI smoke tests required |
| Cross-platform | Native build and dialogs | Windows, macOS, Linux launch/open/save | CI build matrix; manual smoke test required |
| Performance | Large tables/wide values | 100k row offset, large BLOB/TEXT columns, mutation latency | Not yet benchmarked |
| Security | Adversarial inputs | Quoted identifiers, type injection, malicious text values, read-only SQL | Basic unit tests; security audit deferred |

## Required 0.2 database tests

1. Preserve NULL, integer, real, text, and BLOB storage classes when loading typed table pages.
2. Insert a value containing SQL metacharacters and prove it is stored as data rather than executed as SQL.
3. Update and delete a row by a single primary key.
4. Update and delete by a composite primary key in primary-key ordinal order.
5. Update/delete using an unshadowed hidden rowid when no primary key exists.
6. Confirm views and tables with no available safe identity are browse-only.
7. Apply table creation, table rename, column add/rename/drop, and table drop.
8. Reject a malicious type declaration without changing the database schema.
9. Start and commit a transaction; start and roll back a transaction; inspect `is_autocommit` state.
10. Load a generated column and verify it is visible but not writable; verify hidden virtual-table columns are not exposed as editable fields.
11. Cause a constraint failure inside a mutation savepoint and verify the outer transaction remains usable and earlier pending writes remain pending.
12. Verify read-only mode blocks row and schema writes as well as ordinary SQL writes.
13. Verify a connection cannot be replaced while an explicit transaction is active.

## CI gates

Every pull request should pass:

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

Release branches additionally run `cargo build --release` on Windows, macOS, and Linux, dependency/license audit, and manual GUI smoke tests using temporary database copies. The CI workflow currently builds the native app matrix and runs formatting, Clippy, and tests on Linux; ensure the lockfile is committed and matrix test coverage is expanded before a 1.0 release.

## Manual 0.2 GUI smoke test

1. Launch the app and load the demo database.
2. Insert a customer with NULL in one nullable field; verify NULL differs from empty text.
3. Edit one field and verify untouched fields remain unchanged.
4. Insert a text value containing quotes and SQL comment characters; verify it remains literal data.
5. Delete a row, cancel the first dialog, then repeat and confirm with `DELETE`.
6. Begin an IMMEDIATE transaction, insert a row, and roll back with typed confirmation; verify the row is absent.
7. Begin a DEFERRED transaction, insert, commit, and verify persistence.
8. Create a table with a composite primary key, insert and edit a row, and verify targeting.
9. Add/rename/drop a column and rename/drop a disposable table, testing cancellation of the destructive dialogs.
10. Open a copy read-only; verify guided writes and transaction start are disabled, SQL writes are rejected, and a manually started read transaction can be ended.
11. Open the SQL Workbench and verify one valid statement, one syntax error, and an explicit SQL transaction are reflected in the status/header.
12. Run `PRAGMA integrity_check` and expect `ok` on the test database.
