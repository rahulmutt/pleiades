# Stage 1 — Workspace Bootstrap

## Goal
Create a reproducible Rust workspace skeleton that matches the specified architecture and is pleasant to build on.

## Why this stage comes first
The spec puts strong emphasis on crate boundaries, reproducibility, and pure-Rust tooling. Those decisions are easiest to enforce before implementation details accumulate.

## Primary deliverables

- workspace `Cargo.toml` with first-party `pleiades-*` crates
- `mise.toml` for standard Rust/tooling setup
- `devenv.nix` only if a required tool cannot be managed by mise
- crate skeletons for:
  - `pleiades-types`
  - `pleiades-backend`
  - `pleiades-core`
  - `pleiades-houses`
  - `pleiades-ayanamsa`
  - `pleiades-compression`
  - `pleiades-jpl`
  - `pleiades-vsop87`
  - `pleiades-elp`
  - `pleiades-data`
  - `pleiades-cli`
  - `pleiades-validate`
- workspace linting, formatting, and test commands
- initial CI pipeline that proves pure-Rust build/test workflow
- crate-level README or rustdoc stubs describing responsibilities

## Workable state at end of stage
A contributor can clone the repo, enter the managed tool environment, run formatting/lint/tests, and understand where new functionality belongs.

## Suggested tasks

1. Create the workspace manifest and member crates.
2. Configure shared lint settings and edition/toolchain policy.
3. Add baseline dependencies only where needed.
4. Set up `cargo fmt`, `clippy`, and `cargo test` in CI.
5. Add placeholder modules and docs that codify layering rules.
6. Add one smoke test per crate or one workspace integration smoke test.

## Exit criteria

- all first-party crates follow the `pleiades-*` naming rule
- workspace builds on supported targets in pure Rust mode
- formatting and linting are reproducible locally
- architecture docs and crate skeletons agree
- no crate dependency cycle exists

## Risks to avoid

- adding implementation-heavy dependencies too early
- collapsing multiple responsibilities into `pleiades-core`
- introducing unmanaged tooling outside `mise.toml` / `devenv.nix`
