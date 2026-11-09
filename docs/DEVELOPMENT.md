# Development guide

## Toolchain

The repository pins Rust 1.95.0 in `rust-toolchain.toml`. Install it with Rustup:

```bash
rustup toolchain install 1.95.0
rustup component add rustfmt clippy --toolchain 1.95.0
```

The `rust-toolchain.toml` file makes Cargo select that toolchain when invoked from the repository root.

## Platform prerequisites

### Windows

Install Visual Studio 2022 Build Tools (or Visual Studio) with the **Desktop development with C++** workload and the Windows SDK. Install Rustup using the MSVC target. No Node.js, WebView2, or SQLite SDK is required for this native eframe app.

```powershell
rustup default 1.95.0-x86_64-pc-windows-msvc
```

### macOS

Install Xcode Command Line Tools:

```bash
xcode-select --install
```

### Debian/Ubuntu Linux

Install the development packages listed in the README. Depending on the selected session backend, `rfd` may also need an XDG desktop portal implementation and `zenity` at runtime.

## Run in development

```bash
cargo run
```

Run with verbose tracing:

```bash
RUST_LOG=debug cargo run
```

PowerShell equivalent:

```powershell
$env:RUST_LOG = "debug"
cargo run
```

## Test and lint

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

The database unit tests use in-memory SQLite databases or temporary files. They should not require a live user database.

## Release build

```bash
cargo build --release
```

The native executable is written to `target/release/`. Installer generation is not configured in the prototype; it is a release-engineering task for the 0.5/1.0 milestones.

## Suggested local workflow

1. Create a feature branch from `main`.
2. Keep one concern per commit and write tests before adding complex database mutation logic.
3. Run `cargo fmt`, `cargo clippy`, and the relevant tests.
4. Update architecture/roadmap/user docs when public behavior changes.
5. Open a pull request using the repository template.

## Debugging SQLite behavior

Use the built-in SQL workbench to inspect:

```sql
SELECT sqlite_version();
PRAGMA compile_options;
PRAGMA journal_mode;
PRAGMA foreign_keys;
PRAGMA integrity_check;
```

The app enables foreign-key enforcement on each connection and requests WAL mode on writable file databases. If a filesystem does not support WAL correctly, opening the database still proceeds; inspect the effective journal mode before relying on concurrent readers/writers.

## Dependency updates

- Update one dependency group at a time.
- Review crate release notes and minimum supported Rust versions.
- Run the full CI matrix after updating eframe/egui, rusqlite/SQLite, or rfd.
- Consider `cargo audit` and `cargo deny check` as part of the release pipeline once policy files are adopted.
- Commit `Cargo.lock` for the application repository after the first verified Cargo resolution. It is intentionally not fabricated in this source bundle because Cargo is not installed in the generation environment.
