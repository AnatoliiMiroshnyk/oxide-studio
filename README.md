# Oxide Studio

> A local-first, native desktop workspace for SQLite — built with Rust.

Oxide Studio is an open-source SQLite desktop client prototype focused on a clean workspace, predictable database access, and a small dependency surface. Its long-term goal is to become a serious alternative to SQLiteStudio, with a modern information architecture and a Rust-first core.

The name is a nod to iron oxide and rust; “Studio” reflects its role as a focused SQL workspace for SQLite databases.

<pre>
┌─────────────────────────────────────────────────────────────────────────────┐
│  ◈ OXIDE STUDIO     ● sample.sqlite                 New  Open  Read-only  Run │
├───────────────────────┬─────────────────────────────────────────────────────┤
│ EXPLORER              │  customers                           DATA BROWSER   │
│                       │                                                     │
│  TABLES               │  5 columns    4 rows on this page    Offset 0      │
│   ▦ customers         │  ┌──────────┬────────────────┬───────────────────┐ │
│   ▦ orders            │  │ id       │ name           │ email             │ │
│                       │  ├──────────┼────────────────┼───────────────────┤ │
│  VIEWS                │  │ 1        │ Avery Stone    │ avery@example...  │ │
│   ◫ customer_order…   │  │ 2        │ Mila Novak     │ mila@example...   │ │
│                       │  └──────────┴────────────────┴───────────────────┘ │
│  INDEXES              │                                                     │
│   ⌁ idx_orders_…      │                                                     │
│                       │                                                     │
│  WORKSPACE            │                                                     │
│   ⌘ SQL Workbench     │                                                     │
│   ◷ Query history     │                                                     │
├───────────────────────┴─────────────────────────────────────────────────────┤
│ ● Connected · local session                                  SQLite version │
└─────────────────────────────────────────────────────────────────────────────┘
</pre>

*The frame above is a layout sketch, not a screenshot of a compiled application.*

## Current prototype status

<em>Version 0.1.0 — foundation prototype</em>

The current application includes:

- Native desktop window with a custom dark, teal-accented interface.
- Open an existing SQLite database, create a new database file, or open a database in read-only mode.
- Built-in in-memory sample workspace with related tables, indexes, and a view.
- Schema explorer for tables, views, indexes, and triggers, with object filtering.
- Table/view data browser with 200-row pages.
- Schema inspector for column name, declared type, nullability, default value, primary-key order, and stored SQL definition.
- SQL workbench for one statement per execution, including `SELECT`, DDL, DML, and `PRAGMA` statements.
- Query output with elapsed time, affected-row count, a 1,000-row result cap, and a per-cell text display cap.
- CSV export of the most recent tabular query result.
- Session-only query history and a SQLite `integrity_check` action.
- Bundled SQLite through `rusqlite`, so a system SQLite installation is not required.

### Explicit prototype boundaries

This is a runnable foundation, not feature parity with SQLiteStudio yet. The following are not implemented in v0.1.0: inline cell editing, row insert/update/delete forms, import wizards, SQL syntax completion/highlighting, multiple query tabs, multi-statement script result handling, parameter binding UI, attached databases, ER diagrams, database compare, extensions management, plugin API, schema migration UI, backup/restore wizard, and query cancellation. Long-running statements execute synchronously and can temporarily block the UI. See [the roadmap](docs/ROADMAP.md).

## Technology choices

| Layer | Technology | Responsibility |
|---|---|---|
| Desktop shell and UI | Rust + `eframe` / `egui` | Native window, layout, controls, interaction |
| SQLite access | `rusqlite` with `bundled` | Embedded SQLite engine and typed database access |
| File dialogs | `rfd` | Native open/save dialogs |
| CSV | `csv` | Correct CSV quoting and export |
| Errors | `anyhow` | Context-rich application errors |
| Diagnostics | `tracing` + `tracing-subscriber` | Structured logs |

The stack deliberately avoids Electron, a browser runtime, a local HTTP server, and a JavaScript build chain. `egui` is an immediate-mode GUI, which keeps the UI state model compact and all primary application behavior in Rust. See [ADR-0001](docs/decisions/0001-native-rust-desktop.md).

## Requirements

- Rust **1.95.0** (pinned in `rust-toolchain.toml`) and Cargo.
- Windows 10/11 with the Visual Studio C++ Build Tools (MSVC) for the MSVC target; or a current macOS/Xcode Command Line Tools installation; or Linux development libraries listed below.
- No separate SQLite installation is required: SQLite is built from the `rusqlite` bundled feature. Read-only sessions set both SQLite read-only open flags and `PRAGMA query_only=ON`.

### Linux build dependencies

On Debian/Ubuntu, install the native windowing, graphics, and file-dialog development packages before building:

```bash
sudo apt update
sudo apt install -y \
  build-essential pkg-config cmake \
  libx11-dev libxrandr-dev libxi-dev libxcursor-dev libxinerama-dev \
  libxkbcommon-dev libwayland-dev libgl1-mesa-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libdbus-1-dev libssl-dev
```

`rfd` uses the desktop portal backend by default on Linux and may fall back to `zenity`; install a portal backend and `zenity` if native dialogs do not appear in your desktop environment.

## Quick start

`config/default.toml` documents the intended defaults for the next configuration milestone; v0.1.0 currently uses matching compile-time constants and does not parse that file at runtime. The Cargo lockfile is generated on the first Cargo resolution on your machine and should then be committed to the application repository. The source snapshot was not compiled in its generation environment because the Rust toolchain was unavailable; see [Build validation](docs/BUILD_VALIDATION.md).

### Windows (PowerShell)

```powershell
# Install the stable Rust toolchain with rustup if it is not installed.
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

On first run Cargo downloads crates and builds the native application. On the welcome screen select **Load demo** to explore the sample database, or **Open database** to work with your own `.db`, `.sqlite`, or `.sqlite3` file.

For the full operating-system-specific setup, see [Development setup](docs/DEVELOPMENT.md) and [the user guide](docs/USER_GUIDE.md).

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl + Enter` | Execute the SQL editor contents |

## Project layout

```text
oxide-studio/
├── .github/                  # CI, issue templates, PR template
├── assets/
│   ├── branding/             # Brand source assets
│   └── sample/               # Demo SQL schema and data
├── config/                   # Reference defaults (not yet loaded at runtime)
├── docs/                     # Product, architecture, UX, testing, security, release, ADRs
├── scripts/                  # Local development helpers
├── src/
│   ├── app/                  # Application state and workspace UI
│   ├── db/                   # SQLite connection, schema, query, export
│   ├── ui/                   # Theme and UI styling
│   ├── lib.rs
│   └── main.rs
├── Cargo.toml
├── rust-toolchain.toml
├── rustfmt.toml
└── README.md
```

## Quality commands

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
```

## Security posture

Oxide Studio is a local desktop database client. It does not upload database contents or start a network service. SQL is powerful by design: executing a statement can modify or delete data in the selected database. Use read-only mode when you only need to inspect an untrusted or important database, and make backups before running write statements.

Implementation controls and gaps are recorded in [Security model](docs/SECURITY_MODEL.md). This prototype does not claim to be security-audited.

## Architecture and product plan

- [Product requirements](docs/PRODUCT_REQUIREMENTS.md) — personas, workflows, requirements, and measurable 1.0 acceptance targets.
- [Architecture](docs/ARCHITECTURE.md) — modules, data flow, boundaries, and scaling strategy.
- [UI/UX guidelines](docs/UI_UX_GUIDELINES.md) — layout, visual system, and interaction rules.
- [Roadmap](docs/ROADMAP.md) — SQLiteStudio capability map and staged delivery plan.
- [User guide](docs/USER_GUIDE.md) — application walkthrough and current limitations.
- [Development guide](docs/DEVELOPMENT.md) — setup, quality gates, packaging, debugging.
- [Security model](docs/SECURITY_MODEL.md) — threat model and mitigations.
- [Test strategy](docs/TEST_STRATEGY.md) — test layers, fixtures, performance and CI gates.
- [Release process](docs/RELEASE_PROCESS.md) — versioning, packaging targets, and release checks.
- [Build validation](docs/BUILD_VALIDATION.md) — what was checked in this environment and the required local smoke tests.
- [Commit plan](docs/COMMIT_PLAN.md) — ordered commit history with exact file lists.
- [ADR-0001](docs/decisions/0001-native-rust-desktop.md) — native Rust UI decision.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. Feature work should include tests, user-facing documentation, and an update to the roadmap when scope changes.

## License

MIT. See [LICENSE](LICENSE).
