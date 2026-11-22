# Build and validation record

**Snapshot:** Oxide Studio `0.2.0` — Data editing and transaction management.

## Validation status

This source snapshot was prepared in an environment without `rustc`, `cargo`, `rustfmt`, or `rust-analyzer`. Rust formatting, dependency resolution, compilation, Clippy, Rust unit tests, release linking, and native GUI smoke tests could not be executed here. **This archive is not represented as build-verified or production-certified.**

The project pins Rust `1.95.0` in `rust-toolchain.toml` and uses `eframe 0.37.0`. The source archive does not include a fabricated `Cargo.lock`; run Cargo dependency resolution in the target development environment and commit the resulting lockfile before treating the build as reproducible.

## Static checks performed for this snapshot

The static checks completed for this snapshot are: `Cargo.toml` and `config/default.toml` parse as TOML; all internal Markdown links resolve; the demo SQL fixture executes in Python's SQLite driver with `PRAGMA integrity_check = ok`, four customer rows, and four rows in the `customer_order_totals` view; a separate SQLite behavior simulation passes for parameterized CRUD, composite-key metadata, supported DDL, generated-column metadata, savepoint error isolation, outer-transaction preservation, commit, and rollback; and all 24 changed/new files are assigned in `docs/COMMIT_PLAN.md`. ZIP/TAR integrity and SHA-256 checks are recorded after the archive is created. These checks do **not** compile or execute the Rust implementation and must not be confused with the Rust tests below.

## Required checks on a machine with Rust 1.95.0

Run from the repository root after dependency resolution:

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
```

Keep the generated `Cargo.lock` in version control. Resolve every compiler warning and lint rather than suppressing diagnostics wholesale.

## Required 0.2 database regression tests

The repository includes Rust tests for these cases; they must be executed locally before publishing a binary:

- Typed table reads preserve NULL, integer, real, text, and BLOB values.
- Generated columns remain readable but are non-editable in guided forms; virtual-table hidden columns are excluded from the visible projection.
- Primary-key and composite-primary-key identities target the intended row; an unshadowed hidden rowid is used only as a fallback for eligible rowid tables.
- Bound INSERT/UPDATE/DELETE values containing SQL metacharacters remain data.
- A failed mutation rolls back its savepoint without aborting an enclosing transaction or discarding earlier pending writes.
- Table creation, rename, column add/rename/drop, and table drop use the supported DDL subset.
- Unsafe type declarations are rejected before SQLite executes generated DDL.
- DEFERRED, IMMEDIATE, and EXCLUSIVE transaction modes, explicit COMMIT/ROLLBACK, and connection-replacement guards behave as documented.
- Read-only sessions reject guided mutations and ordinary Workbench write statements.

## Manual GUI smoke test

Use disposable copies of databases, never the only copy of important data.

1. Launch Oxide Studio and load the in-memory demo database.
2. Insert a row with a NULL field and a separate empty-text field; verify the values remain distinct.
3. Insert text containing quotes, semicolons, and SQL comment markers; verify it is stored literally.
4. Edit one field and verify untouched fields, including BLOB values, remain unchanged.
5. Confirm that generated columns are displayed but cannot be edited.
6. Delete a row, cancel once, then repeat and confirm only after typing `DELETE`.
7. Begin an IMMEDIATE transaction, edit data, and roll back after typing `ROLLBACK`; verify the change is absent.
8. Begin a DEFERRED transaction, write data, commit, and verify it remains.
9. Create a table with a composite primary key; insert, update, and delete a row.
10. Exercise table rename, column add/rename/drop, and table drop on disposable objects. Verify the destructive dialogs require the exact object name.
11. Open another copy read-only and verify guided editing and transaction start are disabled, SQL writes are rejected, and a read transaction started in the Workbench can be ended.
12. Run a destructive statement in the SQL Workbench only on a disposable database, confirming the visible warning that the Workbench bypasses guided confirmations.
13. Run `PRAGMA integrity_check;` and expect `ok` for a healthy fixture.

## Release evidence to record

For each supported OS, attach CI logs containing OS version, target triple, Rust/Cargo versions, SQLite version, commit SHA, formatting/check/Clippy/test/build results, and GUI smoke-test outcome. Publish checksums for each release asset. Installer creation, code signing/notarization, dependency audit, SBOM generation, and an independent security review remain separate release gates.
