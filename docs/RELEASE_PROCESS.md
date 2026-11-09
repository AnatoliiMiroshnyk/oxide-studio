# Release process

## 1. Versioning

Use Semantic Versioning:

- `MAJOR` for breaking user-visible behavior, workspace formats, or public extension APIs.
- `MINOR` for backward-compatible features.
- `PATCH` for backward-compatible bug and security fixes.

Pre-1.0 releases may change behavior more freely, but every release must still have a changelog entry and clear migration notes if database behavior changes.

## 2. Branch and tag policy

- `main` must remain buildable and pass required CI gates.
- Feature branches should be short-lived and focused.
- Release candidates use tags such as `v0.5.0-rc.1`.
- Stable releases use signed annotated tags such as `v1.0.0` after all release gates pass.
- Never rewrite a published release tag; issue a new patch version instead.

## 3. Pre-release checklist

- [ ] Review `CHANGELOG.md` and update version metadata.
- [ ] Update and commit `Cargo.lock` with the pinned Rust toolchain.
- [ ] Run formatting, Clippy, unit, integration, UI smoke, and platform build tests.
- [ ] Run dependency vulnerability and license checks; generate an SBOM.
- [ ] Run large-database, read-only, corrupt-file, and export regression fixtures.
- [ ] Verify that no telemetry or network behavior was added without explicit product approval and privacy documentation.
- [ ] Review data-loss warnings, defaults, release notes, known issues, and upgrade notes.
- [ ] Produce signed platform artifacts and verify checksums.
- [ ] Test install, launch, database open, query, CSV export, and uninstall on clean VMs.

## 4. Platform packaging target

The prototype produces a native executable with `cargo build --release`; it does not yet package OS installers. Planned distribution formats:

| Platform | Target package |
|---|---|
| Windows | Signed `.msi` or `.exe` installer; portable ZIP as a secondary option |
| macOS | Signed and notarized `.dmg` / `.app` bundle |
| Linux | `.AppImage` plus distro-native `.deb` / `.rpm` packages where feasible |

Keep SQLite and required runtime libraries bundled or explicitly documented. Installer generation, code signing, notarization, update feeds, and auto-update are not part of v0.1.0.

## 5. Release validation and rollback

- Test every artifact from a clean user account and a clean OS image.
- Verify checksums and signatures before publishing.
- Keep a previous stable release available for rollback.
- Never downgrade or rewrite a user's database automatically. Schema migrations, if ever introduced for Oxide metadata, must be versioned and reversible independently of user SQLite files.
- Publish known issues and data-safety caveats with the release.

## 6. Security releases

Treat confirmed vulnerabilities in SQLite, Rust dependencies, or Oxide's file-handling/query paths as release blockers according to severity. Coordinate private disclosure, assess affected versions, publish a patched version and advisory, and avoid publishing proof-of-concept material that exposes user database contents.
