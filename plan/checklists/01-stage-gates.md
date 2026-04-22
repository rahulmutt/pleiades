# Checklist 1 — Stage Gates

Use this checklist before declaring a stage complete and moving on.

The goal is to preserve the plan's core rule: **the repository must remain workable at every stage**.

## Universal gate

Every stage should satisfy all of the following:

- the workspace builds successfully with the repository-managed toolchain
- relevant tests exist for the newly introduced behavior
- documentation is updated for public-facing or contributor-facing changes
- implemented scope and known gaps are called out explicitly
- crate boundaries still match `spec/architecture.md`
- no mandatory C/C++ dependency or unmanaged tool has been introduced
- the next stage can rely on the current stage without first cleaning up avoidable debt

## Stage-specific gates

### Stage 1 — Workspace bootstrap

- all required first-party crates exist with `pleiades-*` names
- `mise.toml` covers standard tooling needed for local development
- `devenv.nix` is used only when mise is insufficient and the reason is documented
- formatting, linting, and test commands are documented and runnable
- basic CI or equivalent automation proves the pure-Rust workflow

### Stage 2 — Domain types and backend contract

- shared types define units, normalization rules, and time semantics clearly
- backend request/result and metadata models exist
- structured errors distinguish invalid input, unsupported features, and range issues
- a mock or toy backend proves the contract can be implemented cleanly
- rustdoc or examples show the minimal query flow

### Stage 3 — Chart MVP and algorithmic baseline

- at least one end-to-end chart workflow works through `pleiades-core`
- baseline built-ins for houses and ayanamsas are materially implemented
- `pleiades-cli` or equivalent user-facing entry point demonstrates the workflow
- current compatibility coverage and known gaps are published
- chart-level tests or snapshots cover realistic usage

### Stage 4 — Reference backend and validation

- at least one source-backed backend is usable in pure Rust
- comparison and benchmark tooling is reproducible
- capability matrices exist for implemented backends
- validation reports can be regenerated from documented inputs
- regression cases discovered so far are preserved in the test corpus

### Stage 5 — Compression and packaged data

- artifact format and versioning are documented
- generated artifacts are reproducible from public inputs
- measured artifact error envelopes are published
- packaged lookups are validated at segment boundaries and representative epochs
- out-of-range or unsupported packaged-data behavior is explicit

### Stage 6 — Compatibility expansion and release hardening

- release compatibility profile is current and versioned
- remaining catalog coverage is either implemented or explicitly scheduled
- release gates from `spec/validation-and-testing.md` are automated where practical
- docs explain how maintainers reproduce builds, validation, and artifacts
- API stability posture is stated clearly for consumers

## When not to advance

Do **not** mark a stage complete if any of these are true:

- the main workflow only works on one maintainer machine
- tests exist only for happy paths while known edge cases remain undocumented
- the repository structure contradicts the architecture spec
- a later stage is being used to justify skipping foundational cleanup that should be done now
- compatibility breadth is being claimed without a published profile of actual coverage
