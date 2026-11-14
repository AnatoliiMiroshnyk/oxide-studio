# Changelog

All notable changes to Oxide Studio are documented here.

The project follows a lightweight Keep a Changelog structure and Semantic Versioning once stable release contracts are established.

## [Unreleased]

### Changed

- Renamed the product branding, Rust package, application type, executable, branding asset, and project documentation to Oxide Studio.

### Fixed

- Updated SQLite column-name collection to consume the iterator returned by `rusqlite` directly.
- Updated egui styling to use the per-theme `style_mut_of` API and explicitly select the dark theme, compatible with egui 0.37.

## [0.1.0] - 2026-11-09

### Added
- Initial native Rust desktop application.
- SQLite database connection management.
- Database creation and opening.
- Read-only database mode.
- Database schema explorer.
- Tables, views, indexes and triggers inspection.
- Table and view data browser.
- SQL workbench.
- SQL query execution.
- Query result grid.
- Session-based SQL history.
- CSV export.
- SQLite integrity checking.
- Dark desktop interface.

### Documentation
- Added architecture documentation.
- Added product requirements.
- Added security model.
- Added development guide.
- Added user guide.
- Added feature roadmap.
- Added contribution guidelines.

### Notes
- Initial prototype release.
- Some advanced SQLiteStudio features are planned
  for future versions.
