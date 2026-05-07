# Track 1 — Workspace and Tooling

## Role

Maintain a reproducible pure-Rust development environment while active phases add source data, artifact generation, validation thresholds, benchmarks, and release gates.

## Standards

- Keep standard developer tools declared in `mise.toml` when available through mise.
- Use `devenv.nix` only for tools or system libraries that cannot reasonably be managed by mise, and document why.
- Do not add curl-based bootstrap scripts or undocumented global tool assumptions.
- Preserve the mandatory `pleiades-*` crate prefix for first-party crates.
- Keep generated data paths deterministic, checksumed, and reviewable.

## Checks to preserve

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- workspace pure-Rust/native-dependency audits
- artifact validation and release-bundle verification when release-facing files change
