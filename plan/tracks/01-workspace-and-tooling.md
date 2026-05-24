# Track 1 — Workspace and Tooling

## Role

Preserve the reproducible Rust workspace while active phases add source data,
artifact generation, validation, and release gates.

## Standards

- Manage standard tools through `mise.toml`.
- Keep normal build/test workflows pure Rust with no mandatory C/C++ dependency.
- Preserve first-party `pleiades-*` crate naming and workspace membership.
- Keep CI/local commands reproducible from a clean checkout.

## Checks to preserve

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- native dependency/build-hook audit
- release-smoke and release-gate commands once production gates are enabled
