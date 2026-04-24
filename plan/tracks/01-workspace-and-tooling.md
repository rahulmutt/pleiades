# Track 1 — Workspace and Tooling

## Role

Maintain a reproducible pure-Rust development environment while the remaining phases add source data, generators, validation, benchmarks, and release tooling.

## Standards

- Keep all standard tools declared in `mise.toml` when available through mise.
- Use `devenv.nix` only for tools or system libraries that cannot reasonably be managed by mise, and document why.
- Do not add curl-based bootstrap scripts or undocumented global tool assumptions.
- Preserve the mandatory `pleiades-*` crate prefix for first-party crates.
- Keep the workspace buildable on stable Rust unless a spec-approved change says otherwise.

## Checks to preserve

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- doc tests for public examples
- pure-Rust/native-dependency audits in validation or release tooling

## Phase-specific notes

- Phase 1 source readers and coefficient data should not require C/C++ code generation or native libraries.
- Phase 2 artifact generation must be reproducible from documented commands.
- Phase 4 release bundles should capture tool versions and source revisions.
