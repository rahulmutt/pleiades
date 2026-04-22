# Track 1 — Workspace and Tooling

## Purpose
Keep the repository reproducible, pure Rust, and easy to work on locally and in CI.

## Scope

- workspace layout and crate manifests
- `mise.toml` and tool pinning
- `devenv.nix` only for tools that mise cannot manage cleanly
- formatting, linting, tests, and CI workflows
- contributor-facing setup docs

## Primary stages

- Stage 1 is the main delivery point
- Stages 2-6 extend this track as new crates, commands, and release checks appear

## Key milestones

1. Workspace skeleton builds with all required `pleiades-*` crates.
2. Shared lint/test commands are documented and reproducible.
3. CI proves pure-Rust build and test workflows.
4. Release automation validates artifacts, reports, and compatibility profiles.

## Done criteria for work in this track

- tooling is declared in repository-managed config
- local and CI commands match
- no undocumented machine-specific setup is required
- new developer workflows are documented when introduced

## Common failure modes

- adding a tool outside `mise.toml` without justification
- hiding required setup in ad hoc scripts
- letting CI drift away from the local developer experience
