# Track 1 — Workspace and Tooling

## Role

Maintain a reproducible pure-Rust development environment while remaining phases add source data, artifact generators, validation thresholds, benchmarks, and release gates.

## Standards

- Keep standard developer tools declared in `mise.toml` when available through mise.
- Use `devenv.nix` only for tools or system libraries that cannot reasonably be managed by mise, and document why.
- Do not add curl-based bootstrap scripts or undocumented global tool assumptions.
- Preserve the mandatory `pleiades-*` crate prefix for first-party crates.
- Keep the workspace buildable on the declared stable Rust toolchain unless the spec is updated.
- Keep generated data paths deterministic, checksumed, and reviewable.

## Checks to preserve

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- doc tests for public examples when affected
- workspace-native pure-Rust/native-dependency audits
- release-bundle verification when release-facing files change

## Phase-specific notes

- Phase 1 source readers and reference corpora must remain pure Rust and deterministic.
- Phase 2 artifact generation must be reproducible from documented public inputs and commands.
- Phase 3 catalog tests must not depend on concrete backend crates.
- Phase 4 release bundles should capture source revision, workspace status, tool versions, checksums, validation parameters, and artifact-generation parameters.
