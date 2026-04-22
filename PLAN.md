# Pleiades Development Plan

This plan translates `SPEC.md` and `spec/*.md` into an execution sequence that keeps the repository in a **workable state at every stage**.

The ordering follows a simple rule: **foundations first, useful product early, validation before optimization, breadth and hardening last**.

That gives this development arc:

1. establish reproducible workspace foundations,
2. lock down shared types and backend contracts,
3. ship a minimal but usable chart-calculation path,
4. add stronger reference backends and formal validation,
5. add packaged data and performance-oriented distribution,
6. complete compatibility breadth and release hardening.

`spec/roadmap.md` remains the concise normative roadmap. This plan is the execution-oriented companion that explains sequencing, dependencies, stage outcomes, and cross-cutting workstreams.

## Plan Index

### Orientation

- [plan/overview.md](plan/overview.md) — how to read and use this plan

### Sequential stages

- [plan/stages/01-workspace-bootstrap.md](plan/stages/01-workspace-bootstrap.md)
- [plan/stages/02-domain-types-and-backend-contract.md](plan/stages/02-domain-types-and-backend-contract.md)
- [plan/stages/03-chart-mvp-algorithmic-baseline.md](plan/stages/03-chart-mvp-algorithmic-baseline.md)
- [plan/stages/04-reference-backend-and-validation.md](plan/stages/04-reference-backend-and-validation.md)
- [plan/stages/05-compression-and-packaged-data.md](plan/stages/05-compression-and-packaged-data.md)
- [plan/stages/06-compatibility-expansion-and-release-hardening.md](plan/stages/06-compatibility-expansion-and-release-hardening.md)

### Cross-cutting tracks

- [plan/tracks/01-workspace-and-tooling.md](plan/tracks/01-workspace-and-tooling.md)
- [plan/tracks/02-domain-and-public-api.md](plan/tracks/02-domain-and-public-api.md)
- [plan/tracks/03-backends-and-distribution.md](plan/tracks/03-backends-and-distribution.md)
- [plan/tracks/04-validation-and-release.md](plan/tracks/04-validation-and-release.md)

## Stage sequencing rationale

### Stage 1 → Workspace bootstrap
Create the workspace, crate boundaries, and reproducible tooling first so later implementation work lands in the right structure.

### Stage 2 → Domain types and backend contract
Define stable semantics next so backend and domain work can proceed without redesigning shared types repeatedly.

### Stage 3 → Chart MVP with algorithmic baseline
Deliver the first practical user value early: chart generation from pure-Rust algorithmic components plus baseline houses and ayanamsas.

### Stage 4 → Reference backend and validation
Only after an MVP exists should the project invest in heavier reference-data work to validate and calibrate the baseline.

### Stage 5 → Compression and packaged data
Compression comes after validation so packaged artifacts are shaped by measured behavior instead of guesswork.

### Stage 6 → Compatibility expansion and release hardening
Broader catalog coverage and release polish come last, once the architectural seams and validation workflow have been proven.

## Working-state rule

Each stage should end with all of the following true:

- the workspace builds and tests successfully,
- the public API is internally coherent for what is implemented,
- at least one realistic user or maintainer workflow is supported,
- known gaps versus the full specification are documented explicitly.

## Stage outcomes at a glance

| Stage | Primary result | Why it matters |
| --- | --- | --- |
| 1 | Reproducible Rust workspace with correct crate boundaries | Prevents architecture drift from the start |
| 2 | Stable core types and backend contracts | Lets multiple backends evolve without API churn |
| 3 | First useful end-to-end chart workflow | Creates immediate product value |
| 4 | Validation-grade reference backend and reports | Grounds correctness claims in evidence |
| 5 | Fast packaged 1500-2500 backend | Serves the common deployment target |
| 6 | Broad compatibility plus release discipline | Makes the project dependable for consumers |

## Readiness checklist for moving between stages

Do not advance a stage just because code exists. Advance when the current stage also has:

- tests for the newly introduced behavior,
- updated docs or compatibility notes,
- explicit limits and failure modes,
- no layering violations against `spec/architecture.md`.

## Cross-cutting priorities

These priorities apply in every stage:

- preserve the crate layering from `spec/architecture.md`,
- keep the project pure Rust with no mandatory C/C++ dependencies,
- add tests and docs together with behavior,
- publish capability and compatibility information as features expand,
- prefer minimal, reviewable increments over speculative rewrites.
