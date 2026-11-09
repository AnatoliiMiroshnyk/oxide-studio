# Contributing

Thanks for considering a contribution to Oxide Studio.

## Before starting

- Check open issues and the [roadmap](docs/ROADMAP.md).
- For a new capability, describe the user workflow and acceptance criteria before building a large implementation.
- For database mutation features, discuss transaction boundaries, row identity, rollback, and data-loss prevention first.

## Development

1. Install Rust 1.95.0 and platform dependencies from [Development guide](docs/DEVELOPMENT.md).
2. Create a focused branch, for example `feat/data-grid-editing`.
3. Keep changes cohesive and write unit tests for database behavior.
4. Run formatting, Clippy, and tests before opening a PR.
5. Update the user guide and roadmap when behavior or supported workflows change.

## Code standards

- Prefer clear ownership and small modules over shared mutable global state.
- Keep database connection ownership inside the database layer.
- Bind values with rusqlite parameters. Quote identifiers when dynamic identifiers are unavoidable.
- Keep UI presentation concerns out of database modules.
- Do not silently change or normalize user SQL.
- Avoid logging SQL text, data rows, BLOB contents, or personal information.
- Document any `unsafe` code and justify why safe APIs are insufficient.

## Commit messages

Use a conventional, imperative format:

- `feat(db): add backup API`
- `fix(ui): preserve selected table after refresh`
- `test(db): cover read-only connections`
- `docs: clarify CSV import behavior`

## Pull requests

Use the pull request template. Include reproduction steps for fixes and before/after screenshots for visible UI changes. A PR that changes database write behavior must include tests for success, failure, rollback, and read-only mode.

## Security issues

Do not report exploitable vulnerabilities in public issues. Follow `SECURITY.md`.
