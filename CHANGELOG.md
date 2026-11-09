# Changelog

All notable changes to Oxide Studio are documented here.

The project follows a lightweight Keep a Changelog structure and Semantic Versioning once stable release contracts are established.

## [Unreleased]

### Changed

- Renamed the product branding, Rust package, application type, executable, branding asset, and project documentation to Oxide Studio.

### Fixed

- Updated SQLite column-name collection to consume the iterator returned by `rusqlite` directly.
- Updated egui styling to use the per-theme `style_mut_of` API and explicitly select the dark theme, compatible with egui 0.37.

## [0.1.0] - 2026-11-07

### Added

- Native Rust desktop shell using eframe/egui.
- SQLite database opening, creation, read-only mode, and in-memory demo workspace.
- Schema explorer for tables, views, indexes, and triggers.
- Paginated table data browser and schema inspector.
- Single-statement SQL workbench with bounded results and execution timing.
- Session query history, CSV export, and SQLite integrity check.
- Unit tests for schema reads, query execution and row limits, display-value truncation, read-only behavior, and CSV output.
- Product requirements, architecture, UX, roadmap, user, development, security, test, release, build-validation, and contribution documentation.
- GitHub Actions workflow for formatting, Clippy, tests, and cross-platform builds.
