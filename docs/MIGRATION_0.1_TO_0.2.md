# Migration guide: Oxide Studio 0.1.0 to 0.2.0

## Database compatibility

Oxide Studio 0.2.0 uses the same SQLite file format and does not require a database schema migration when opening databases created with 0.1.0. The application does not automatically change user database schemas at startup.

**Always make a backup before using the new editing or schema features on important files.** A successful 0.1.0 open does not imply that an arbitrary database is safe to modify.

## Source and build migration

1. Commit or back up your 0.1.0 repository state.
2. Replace the source tree with the 0.2.0 source bundle, or merge the new changes through the supplied commit plan.
3. Confirm `Cargo.toml` reports `version = "0.2.0"`.
4. Run `cargo fmt --all`, `cargo check --all-targets`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-targets` on Rust 1.95.0.
5. Commit the Cargo-generated `Cargo.lock` if your repository does not yet track one. Keep it committed for reproducible application builds.
6. Run the GUI smoke tests described in `docs/BUILD_VALIDATION.md`.

## Behavior changes

- Data tables now support guided insert/update/delete where a safe row identity is available.
- `NULL` is distinct from an empty string in the row editor.
- Single and composite primary keys are preferred for row targeting. Ordinary rowid tables without a primary key may use an unshadowed internal rowid; views and unidentified tables remain read-only in the data browser.
- Schema editing is available for create table, rename table, add/rename/drop column, and drop table. Unsupported table-rebuild operations remain out of scope.
- Explicit transactions can be started as DEFERRED, IMMEDIATE, or EXCLUSIVE and committed or rolled back.
- Guided destructive operations require exact typed confirmation. SQL Workbench statements remain direct SQL and bypass guided confirmations.

## Rollback plan

If 0.2.0 causes a workflow regression, stop modifying the affected database and preserve a copy of the file. Revert the application source to the last known 0.1.0 commit or tag. The 0.2.0 editor does not perform an automatic data migration, so rolling back application code does not require reversing an app-managed database version change. Manually undo any data or DDL changes already committed by the user; restoring a pre-change backup is the safest recovery method.
