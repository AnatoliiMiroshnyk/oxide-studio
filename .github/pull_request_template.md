## Summary

What does this change do, and why?

## Type of change

- [ ] Bug fix
- [ ] New feature
- [ ] Performance improvement
- [ ] Documentation
- [ ] Refactor / maintenance

## Validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all-targets`
- [ ] Manual smoke test (describe below)

## Data safety review

- [ ] No destructive database operation was added, or destructive behavior is explicitly confirmed and tested.
- [ ] SQL values are bound; dynamic identifiers are quoted.
- [ ] Large result and BLOB memory behavior is bounded.
- [ ] Errors do not reveal database contents or secrets.

## Screenshots / test notes

Add UI evidence or explain why a screenshot is not applicable. Never include real database records or secrets.
