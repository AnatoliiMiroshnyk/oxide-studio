# Oxide Studio

> A local-first, native SQLite desktop workspace built with Rust.

Oxide Studio is a desktop client for inspecting, editing, and querying SQLite databases. It combines a native Rust application, a bundled SQLite engine, typed data editing, and explicit transaction controls without an embedded browser or local web server.

The name is a nod to iron oxide and Rust; “Studio” reflects its role as a focused database workspace.

<pre>
┌───────────────────────────────────────────────────────────────────────────────┐
│  ◈ OXIDE STUDIO   ● inventory.sqlite     New  Open  Read-only  Run  Txn       │
├──────────────────────┬────────────────────────────────────────────────────────┤
│ EXPLORER             │ products                               DATA BROWSER   │
│                      │ + New row  Edit selected  Delete selected             │
│ TABLES               │ ┌─────┬─────────┬───────────────┬──────────────┐       │
│  ▦ products          │ │ ROW │ id      │ name          │ price        │       │
│  ▦ customers         │ ├─────┼─────────┼───────────────┼──────────────┤       │
│                      │ │ 1   │ 101     │ Keyboard      │ 79.90        │       │
│ VIEWS / INDEXES      │ │ 2   │ 102     │ Mouse         │ 24.50        │       │
│                      │ └─────┴─────────┴───────────────┴──────────────┘       │
│ WORKSPACE            │                                                        │
│  SQL Workbench       │  ◉ Transaction active       Commit  Rollback…          │
│  Query history       │                                                        │
└──────────────────────┴────────────────────────────────────────────────────────┘
</pre>

*The frame above is a layout sketch, not a screenshot of a compiled application.*

## Release status

**Version 0.2.0 — Data editing and transaction management.**

The release adds guided row CRUD, typed/NULL-aware editing, table and column schema workflows, explicit transaction control, and confirmation gates for destructive actions. The feature set is deliberately bounded to SQLite's well-supported `ALTER TABLE` operations; it does not claim full SQLiteStudio parity.

## 0.2.0 capabilities

### Database and schema

- Open, create, and open SQLite databases in read-only mode.
- Browse tables, views, indexes, and triggers.
- Inspect declared column type, nullability, default value, primary-key order, and stored SQL definition.
- Create tables with typed columns and single or composite primary keys.
- Rename tables and columns; add and drop columns; drop tables.
- Let SQLite validate constraints and object dependencies. Schema mutations run in savepoints so a failed operation is rolled back atomically.

### Data editing

- Browse tables in 200-row pages and select individual records.
- Insert, update, and delete records through dedicated forms.
- Preserve SQLite value types instead of writing display strings back to the database.
- Distinguish `NULL` from an empty string; parse integer and real affinities; edit BLOB values with a `hex:` representation.
- Identify rows by declared primary key, including composite keys. For ordinary rowid tables without a declared primary key, use an unshadowed SQLite rowid alias when available. Views and tables without a safe identity remain browse-only.
- Parameterize all value writes. Identifiers are escaped separately and are never interpolated from cell values.

### Transaction management

- Start `DEFERRED`, `IMMEDIATE`, or `EXCLUSIVE` transactions.
- Commit explicitly or roll back after typing the confirmation token `ROLLBACK`.
- Keep CRUD/schema operations inside the current transaction when one is active; per-operation savepoints provide an additional failure boundary.
- Prevent switching database connections while a transaction is active.

### Destructive-operation safety

- Row deletion requires a confirmation dialog and the exact token `DELETE`.
- Dropping a column or table requires typing the exact object name.
- Rollback requires the exact token `ROLLBACK`.
- Guided workflow confirmations do not wrap arbitrary SQL typed into the advanced SQL Workbench. Treat raw SQL as a deliberate power-user interface.

### Existing 0.1 features

- Native Rust/egui desktop shell, native file dialogs, and bundled SQLite.
- SQL Workbench for one statement per run, query history, bounded result display, and CSV export.
- SQLite integrity check and in-memory demo workspace.

## Known limitations and scope boundaries

This is an actively developed desktop client, not a claim of complete SQLiteStudio feature parity or a security-certified database administration product.

- Database operations still run synchronously on the UI thread. Large scans, locks, and long-running SQL can make the window temporarily unresponsive. A dedicated database actor, cancellation, and progress reporting remain future work.
- SQL Workbench executes statements directly. Its guided confirmation dialogs do not parse or intercept destructive SQL, and SQL is not sandboxed.
- The row editor supports SQLite storage classes and type-affinity parsing, but it is not a full custom-function, locale-aware decimal, date/time, or binary hex editor.
- Tables without a declared primary key use hidden rowid only when an unshadowed alias is available. Hidden rowids are implementation identities, not durable business keys; prefer declared primary keys. Views are read-only in the data browser.
- Schema editing supports create table, rename table, add column, rename column, drop column, and drop table. Rebuilding a table to change an existing column's type, nullability, generated expression, or constraints is not included. Creating/editing indexes and triggers has no dedicated form yet; use carefully reviewed SQL.
- Multi-statement script execution, SQL bind-parameter UI, syntax highlighting, autocomplete, import wizards, backup/restore, persistent workspaces, ER diagrams, database compare, and query cancellation are not part of 0.2.0.
- The application source bundle intentionally does not include compiled binaries. Build validation must be completed on supported operating systems before publishing release binaries.

See [the roadmap](docs/ROADMAP.md) for planned work and [the build-validation record](docs/BUILD_VALIDATION.md) for the checks actually performed on this source snapshot.

## Technology

| Layer | Technology | Responsibility |
|---|---|---|
| Desktop shell/UI | Rust + `eframe` / `egui` | Native window, layout, controls, application state |
| SQLite access | `rusqlite` + `bundled` | Embedded SQLite, typed values, transactions, metadata |
| File dialogs | `rfd` | Native open/save dialogs |
| CSV | `csv` | CSV export with proper quoting |
| Errors | `anyhow` | Context-rich application errors |
| Diagnostics | `tracing` + `tracing-subscriber` | Structured logs without recording cell contents |

## Requirements

- Rust **1.95.0** and Cargo (pinned by `rust-toolchain.toml`).
- Windows 10/11 with Visual Studio C++ Build Tools for MSVC, current macOS/Xcode Command Line Tools, or Linux build dependencies below.
- No system SQLite installation is required.

### Linux build dependencies (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install -y \
  build-essential pkg-config cmake \
  libx11-dev libxrandr-dev libxi-dev libxcursor-dev libxinerama-dev \
  libxkbcommon-dev libwayland-dev libgl1-mesa-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libdbus-1-dev libssl-dev
```

`rfd` uses desktop file-dialog backends on Linux. Install a portal backend and/or `zenity` if native dialogs do not appear in your desktop environment.

## Quick start

### Windows (PowerShell)

```powershell
rustup toolchain install 1.95.0
rustup component add rustfmt clippy --toolchain 1.95.0
cd oxide-studio
cargo run
```

### macOS / Linux

```bash
rustup toolchain install 1.95.0
rustup component add rustfmt clippy --toolchain 1.95.0
cd oxide-studio
cargo run
```

On the welcome screen choose **Load demo** or open a local database. Select a table, choose **+ New row**, **Edit selected**, or **Delete selected**. The transaction controls are in the top bar. See the [user guide](docs/USER_GUIDE.md) and the detailed [editing/transaction workflow](docs/TRANSACTIONS_AND_EDITING.md).

## Quality gates

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
```

Commit the generated `Cargo.lock` after the first dependency resolution and keep it under version control for a desktop application. Do not publish a release until the quality gates and native GUI smoke tests have passed on the release matrix.

## Security posture

Oxide Studio is a local desktop application and does not upload database contents or start an application network service. Read-only mode combines SQLite's read-only open flags and `PRAGMA query_only=ON`; guided writes and transaction start are disabled. A read-only transaction started manually in the Workbench can still be ended. Guided writes use bound values and quoted identifiers. SQL remains powerful by design; raw SQL can change or delete data without the guided confirmation dialogs. Use a database copy or backup before modifying important files.

See [Security model](docs/SECURITY_MODEL.md) for the threat model and operational boundaries. This release has not been independently security-audited.

## Documentation

- [User guide](docs/USER_GUIDE.md)
- [Data editing and transaction workflows](docs/TRANSACTIONS_AND_EDITING.md)
- [0.1 to 0.2 migration guide](docs/MIGRATION_0.1_TO_0.2.md)
- [0.2.0 release notes](RELEASE_NOTES.md)
- [0.2.0 implementation summary](docs/IMPLEMENTATION_SUMMARY_0.2.0.md)
- [Product requirements](docs/PRODUCT_REQUIREMENTS.md)
- [Architecture](docs/ARCHITECTURE.md)
- [UI/UX guidelines](docs/UI_UX_GUIDELINES.md)
- [Roadmap](docs/ROADMAP.md)
- [Development guide](docs/DEVELOPMENT.md)
- [Security model](docs/SECURITY_MODEL.md)
- [Test strategy](docs/TEST_STRATEGY.md)
- [Release process](docs/RELEASE_PROCESS.md)
- [Build validation](docs/BUILD_VALIDATION.md)
- [0.2.0 commit plan](docs/COMMIT_PLAN.md)
- [ADR-0001: native Rust desktop](docs/decisions/0001-native-rust-desktop.md)

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md), follow the quality gates, include tests for database behavior, and never include customer databases, secrets, or production data in issue reports.
