# Pleiades Development Plan

This plan translates `SPEC.md` and `spec/*.md` into an implementation sequence that keeps the repository in a **workable state at every stage**.

The sequencing principle is:

1. establish reproducible workspace foundations,
2. ship a minimal but usable chart-calculation path early,
3. add stronger reference backends and validation,
4. add packaged data and performance improvements,
5. complete compatibility breadth and release hardening.

`spec/roadmap.md` remains the concise normative roadmap. The documents below provide a more execution-oriented plan with stage goals, deliverables, exit criteria, and why each stage is ordered the way it is.

## Plan Structure

- [plan/stages/01-workspace-bootstrap.md](plan/stages/01-workspace-bootstrap.md)
- [plan/stages/02-domain-types-and-backend-contract.md](plan/stages/02-domain-types-and-backend-contract.md)
- [plan/stages/03-chart-mvp-algorithmic-baseline.md](plan/stages/03-chart-mvp-algorithmic-baseline.md)
- [plan/stages/04-reference-backend-and-validation.md](plan/stages/04-reference-backend-and-validation.md)
- [plan/stages/05-compression-and-packaged-data.md](plan/stages/05-compression-and-packaged-data.md)
- [plan/stages/06-compatibility-expansion-and-release-hardening.md](plan/stages/06-compatibility-expansion-and-release-hardening.md)

## Working-State Rule

Each stage should end with all of the following true:

- the workspace builds and tests successfully,
- the public API remains internally coherent and documented for what is implemented,
- at least one realistic user workflow is demonstrably supported,
- known gaps versus the full spec are documented rather than implied.

## Recommended Milestones

### Stage 1 outcome
A reproducible Rust workspace exists with crate boundaries, tooling, CI/lint/test scaffolding, and placeholder docs.

### Stage 2 outcome
Consumers can depend on stable core types and backend contracts without committing to any specific ephemeris source.

### Stage 3 outcome
A first useful product exists: a pure-Rust algorithmic chart pipeline for major bodies, baseline houses, baseline ayanamsas, and a CLI demonstration path.

### Stage 4 outcome
Reference-grade comparison infrastructure exists, plus a stronger source-backed backend for validation and artifact generation.

### Stage 5 outcome
The common 1500-2500 workflow is fast and offline-friendly through packaged compressed data.

### Stage 6 outcome
The project approaches full compatibility breadth with documented release profiles, stronger QA, and a stable public release posture.

## Cross-Cutting Priorities

These priorities apply in every stage:

- preserve the crate layering from `spec/architecture.md`,
- keep the project pure Rust with no mandatory C/C++ dependencies,
- add tests and docs together with behavior,
- publish capability/compatibility information as features expand,
- prefer minimal, reviewable increments over large speculative rewrites.
