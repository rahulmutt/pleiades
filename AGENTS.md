# AGENTS.md

Guidance for AI coding agents working in this repository.

## Purpose

This repository is a Rust-first project for the `pleiades` workspace. Agents must preserve a clean, reproducible, maintainable development environment and a modular architecture aligned with the project specification in `SPEC.md` and `spec/*.md`.

When making changes, prefer small, reviewable diffs, keep decisions explicit, and optimize for long-term maintainability over short-term convenience.

---

## Source of Truth

Before making architectural or API changes, read the relevant specification documents:

- `SPEC.md`
- `spec/architecture.md`
- `spec/requirements.md`
- `spec/api-and-ergonomics.md`
- `spec/validation-and-testing.md`
- any other `spec/*.md` file relevant to the task

If code and spec disagree, do not silently invent a new direction. Either:

1. implement the spec,
2. or update the spec and code together in a clearly justified change.

---

## Environment and Tooling Policy

### 1) Use `mise.toml` for tool management

All standard developer tools must be managed through `mise.toml` whenever possible.

Use `mise.toml` for things such as:

- Rust toolchains
- `cargo`-adjacent tools
- `rustfmt`
- `clippy`
- `cargo-nextest`
- `cargo-deny`
- `cargo-audit`
- `just`
- `taplo`
- `mdbook`
- common CLI tooling available through mise backends

Agent rules:

- If a tool can be installed via `mise`, declare it in `mise.toml`.
- Do not introduce ad hoc bootstrap scripts that curl-install tooling.
- Do not rely on globally installed developer tools.
- Prefer pinned or constrained tool versions for reproducibility.
- Keep `mise.toml` human-readable and grouped by purpose.

### 2) Use `devenv.nix` only for tools not installable with `mise.toml`

If a required tool cannot be installed cleanly via `mise.toml`, add it to `devenv.nix`.

Use `devenv.nix` for:

- system packages not supported by mise
- native libraries needed for optional development workflows
- uncommon CLIs unavailable through mise
- environment setup that must be reproducible across Linux/macOS developer machines when mise alone is insufficient

Agent rules:

- Prefer `mise.toml` first.
- Only add to `devenv.nix` after verifying the tool is not reasonably available through mise.
- Document why the tool lives in `devenv.nix` instead of `mise.toml`.
- Keep `devenv.nix` minimal; do not move standard language tooling there unless necessary.

### 3) No unmanaged tooling

Do not introduce dependencies on:

- manually installed system packages without documentation
- one-off shell scripts that mutate a developer machine
- unpinned CI-only tools that developers cannot reproduce locally

If a new tool is required, update the environment files as part of the same change.

---

## Software Development Best Practices

Agents must follow standard professional engineering practices.

### Change management

- Make the smallest change that fully solves the problem.
- Avoid opportunistic refactors unless they are necessary for correctness or maintainability.
- If a refactor is necessary, separate it logically from behavioral changes when possible.
- Preserve backwards compatibility unless the task explicitly allows breaking changes.

### Readability and maintainability

- Write code for the next maintainer, not just for the compiler.
- Prefer clear names over clever code.
- Keep functions focused and modules cohesive.
- Remove dead code, commented-out code, and unused abstractions.
- Add comments only where intent is non-obvious; do not narrate trivial code.

### Testing

Add or update tests for behavior changes.

Prefer:

- unit tests for local logic
- integration tests for crate boundaries and user-visible behavior
- regression tests for bugs
- property tests for invariants and conversions where appropriate

Do not claim something is fixed without adding or updating validation when the repository supports it.

### Documentation

Update documentation when changing:

- public APIs
- developer workflows
- environment setup
- architecture decisions
- user-visible CLI behavior

For public Rust APIs, prefer rustdoc examples where practical.

### Error handling

- Fail explicitly and informatively.
- Avoid `unwrap`, `expect`, and panics in library paths except where a true invariant is being enforced.
- Propagate structured errors with useful context.
- Distinguish invalid input, unsupported operations, configuration issues, and internal failures.

### Security and reliability

- Minimize dependency additions.
- Prefer mature, well-maintained crates.
- Do not add network access, file writes, or shell execution to library paths unless clearly required.
- Treat parsing, serialization, and external data ingestion as untrusted input boundaries.

### Performance

- Measure before making non-obvious performance claims.
- Prefer simple and correct implementations first.
- Optimize hot paths that affect chart computation, batch queries, data decode, or validation workloads.
- Avoid premature micro-optimizations that reduce clarity.

---

## Software Architecture Best Practices

Agents must preserve the intended workspace architecture.

### Follow the specified layering

Respect the architecture in `spec/architecture.md`:

- `pleiades-types`: primitive/shared types only
- `pleiades-backend`: backend traits and capability metadata
- source-specific backend crates: implementation details for a source family
- domain crates: houses, ayanamsa, compression, etc.
- `pleiades-core`: high-level façade
- tooling crates: CLI, validation, benchmarks, report generation

Do not create dependency cycles.
Do not make low-level crates depend on high-level crates.
Do not put source-specific logic into generic domain crates.

### Preserve modularity

- Keep backend implementations isolated in their own crates.
- Keep astrology-domain calculations backend-agnostic.
- Favor composition over hardwiring one backend into the public API.
- Model capabilities explicitly rather than assuming every backend supports every feature.

### Design for extensibility

When adding new functionality:

- avoid closed designs that block future house systems, ayanamsas, bodies, or backends
- prefer enums plus extension points, trait-based composition, and explicit configuration objects
- ensure additional backends can be added without redesigning core APIs

### Maintain clear boundaries

Separate:

- domain types
- backend contracts
- algorithm implementations
- imported raw data
- normalized intermediate data
- compressed artifacts
- user-facing tools

Do not mix data generation concerns into runtime query APIs unless explicitly intended.

### API ergonomics

Per `spec/api-and-ergonomics.md`, APIs should be:

- strongly typed
- deterministic
- explicit about units, frames, and assumptions
- batch-friendly for chart workloads
- documented with failure modes and examples

Avoid stringly typed APIs where typed models are practical.

### Architectural changes require justification

If changing crate boundaries, public traits, or core data flow:

- explain the reason in code comments, docs, or commit context
- update the relevant spec docs
- preserve consistency across the workspace

---

## Rust Best Practices

This is a pure-Rust project. Rust quality is not optional.

### General Rust guidance

- Prefer stable Rust unless the task explicitly requires otherwise.
- Keep the project compatible with the toolchain declared in `mise.toml`.
- Use edition-appropriate idioms.
- Prefer ownership and borrowing models that make invariants obvious.
- Avoid unnecessary cloning and allocation.
- Favor iterators and expressive standard library APIs when they improve clarity.

### Library design

- Prefer `Result`-based error handling for recoverable failures.
- Define domain-specific error types; use `thiserror` or equivalent when appropriate.
- Use `#[non_exhaustive]` carefully for public extensibility when warranted.
- Prefer newtypes for domain-sensitive values when they improve correctness.
- Avoid leaking implementation details into public APIs.

### Unsafe code

- Avoid `unsafe` unless there is a compelling, documented reason.
- If `unsafe` is required, keep it minimal, isolate it, document invariants, and test it thoroughly.
- Never introduce `unsafe` for convenience alone.

### Clippy, formatting, and linting

- Code must be `rustfmt`-clean.
- Code should pass strict `clippy` checks for the workspace.
- Prefer fixing lint root causes over silencing lints.
- If allowing a lint is necessary, scope it narrowly and document why.

### Testing and validation in Rust

At minimum, agents should run relevant subsets of:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`

When available and relevant, also use:

- `cargo nextest run`
- `cargo bench`
- validation commands from `pleiades-validate`

### Dependency hygiene

- Prefer small, well-maintained crates.
- Avoid duplicate dependencies that solve the same problem.
- Avoid adding heavy dependencies for trivial tasks.
- Review feature flags and disable unnecessary default features when appropriate.
- Preserve the project's pure-Rust constraint: no required C/C++ dependencies.

### Serialization and features

- Make serde support optional unless it is truly core.
- Use Cargo features to isolate heavy functionality, validation-only paths, or optional datasets.
- Keep default features minimal and predictable.

### Numerical and domain correctness

This project deals with astronomy/astrology computations, so correctness is critical.

- Be explicit about units, epochs, reference frames, and normalization rules.
- Avoid hidden assumptions in angle math, time scales, and coordinate conversions.
- Add regression tests for edge cases such as boundary dates, polar latitudes, and normalization wraparound.
- Document expected precision and known limitations.

---

## Project-Specific Rules for `pleiades`

### Crate naming

All first-party workspace crates must use the `pleiades-*` prefix.

### Purity requirement

The project must remain pure Rust with no mandatory C/C++ dependencies, wrappers, or native-toolchain requirements for normal build and test workflows.

### Backend abstraction

Do not couple the public API to a single ephemeris source.
All backend-specific functionality must remain behind clear backend boundaries.

### Compatibility scope

The architecture and API must support:

- a complete, extensible house-system catalog
- a complete, extensible ayanamsa catalog
- multiple backend families
- compressed data products optimized for 1500-2500 CE

Do not design short-term code that makes the long-term compatibility target harder to achieve.

---

## Preferred Agent Workflow

When implementing a task:

1. Read the relevant spec/docs first.
2. Inspect existing code and crate boundaries.
3. Make a minimal design that respects layering.
4. Update `mise.toml` first for new standard tools.
5. Use `devenv.nix` only for tooling that mise cannot provide.
6. Implement code and tests together.
7. Run formatting, linting, and tests.
8. Update docs/specs if behavior or architecture changed.
9. Summarize changes, validation performed, and any follow-up work.

---

## What Agents Should Avoid

Do not:

- bypass `mise.toml` with undocumented tool installation
- add tools to `devenv.nix` when mise can manage them
- introduce architecture drift from the spec
- add hidden global state when explicit configuration is better
- add unnecessary dependencies or feature bloat
- hardcode assumptions that only one backend, one house system subset, or one ayanamsa subset will ever matter
- trade correctness for convenience in numerical code
- leave the repo in a partially documented or partially validated state

---

## If Required Files Are Missing

If `mise.toml`, `devenv.nix`, workspace `Cargo.toml`, or crate manifests are missing and the task requires them:

- create them in a minimal, well-structured form
- keep them aligned with the rules in this document
- do not over-engineer the bootstrap

---

## Deliverable Expectations

Any substantial agent change should leave the repository in a state where a human maintainer can:

- understand the reasoning
- reproduce the environment
- build the project
- run tests and validation
- extend the architecture without rework

That is the standard.
