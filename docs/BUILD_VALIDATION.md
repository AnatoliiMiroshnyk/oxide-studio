# Build and validation record

**Snapshot:** Oxide Studio `0.1.0` prototype source bundle.

## Environment limitation

The source bundle was prepared in an environment without `rustc`, `cargo`, `rustfmt`, or `rust-analyzer`. Consequently, Rust formatting, dependency resolution, compilation, Clippy, Rust unit tests, and native GUI smoke tests were not run during generation. No successful Rust build is claimed.

The project pins Rust `1.95.0` because the selected `eframe 0.37.0` release declares that minimum supported Rust version. First local Cargo resolution will create `Cargo.lock`; commit that generated lockfile before using the project as a reproducible application build.

## Follow-up compiler fixes

The first user-reported Windows compile exposed four API mismatches. The source has been patched as follows:

- `src/db/query.rs`: removed `.iter()` from `Statement::column_names()` and collect directly from its iterator.
- `src/db/schema.rs`: applied the same iterator correction to table data column names.
- `src/ui/theme.rs`: replaced the removed `Context::style` / `Context::set_style` usage with `Context::style_mut_of(egui::Theme::Dark, ...)`, and explicitly selected the dark theme.

These source changes directly address the four reported E0599 errors. This environment still has no Rust toolchain, so a fresh `cargo check` must be run locally to discover whether any further compiler diagnostics remain.

## Checks that were completed

- Parsed `Cargo.toml` and `config/default.toml` as valid TOML.
- Executed `assets/sample/demo.sql` with SQLite through Python's standard `sqlite3` driver.
- Verified expected sample tables and view, four customer records, four view rows, and `PRAGMA integrity_check = ok`.
- Checked all relative Markdown links in the documentation tree.
- Verified the 16 proposed commits are sequentially numbered and the source bundle has no empty files or known placeholder repository URL.

These checks validate the SQL fixture and repository metadata, not the Rust application.

## Required checks on a machine with Rust 1.95.0

Run from the repository root:

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo run
```

Then perform a GUI smoke test on each supported OS:

1. Launch the app and confirm the welcome screen renders.
2. Load the in-memory demo and browse `customers` and `orders`.
3. Inspect the `customer_order_totals` view and the table structure.
4. Run `SELECT sqlite_version();` and an intentional SQL syntax error.
5. Export a query result to CSV and open the file in a spreadsheet editor.
6. Open a temporary database in read-only mode and verify a normal `INSERT` is rejected.
7. Close/reopen the application and inspect a real test database copy.

Record OS, target triple, GPU/renderer, Rust version, SQLite version, command output, and commit SHA in the release CI artifacts. Resolve any compile warnings or platform-specific failures before publishing a binary.
