# ADR-0001: Use a native Rust desktop UI

- **Status:** Accepted for the prototype
- **Date:** 2026-11-07

## Context

The product is intended to be a fast, resource-efficient SQLite desktop client. A desktop webview stack (for example, Rust + Tauri + React) would provide a familiar web UI toolchain, but it introduces a JavaScript frontend/build process and a native webview runtime. The user requested Rust as the primary development language and emphasized resource efficiency.

## Decision

Use `eframe`/`egui` for a native desktop interface and `rusqlite` with its `bundled` feature for SQLite access. Keep the entire application behavior in Rust.

## Consequences

### Benefits

- One primary language and build system.
- No Node.js/npm toolchain or browser runtime dependency.
- Direct native database access and a compact application state model.
- Immediate-mode UI can be efficient for a focused database tool.
- Easier to reuse core database modules in tests.

### Costs

- `egui` is immediate-mode and may require more deliberate state modeling for a complex desktop workspace.
- Rich text editor features (SQL grammar, completion, diagnostics, multi-cursor editing) need additional implementation or carefully selected crates.
- Platform-native look and accessibility need focused polish.
- Large data grids need virtualization and custom layout work.

## Revisit when

Revisit the UI toolkit if accessibility, native editor capabilities, docking, or theming become major blockers. A webview UI is an acceptable alternative only if a measured user-experience benefit outweighs its runtime/build footprint and does not move database logic out of Rust.
