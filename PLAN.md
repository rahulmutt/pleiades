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

`spec/roadmap.md` remains the concise normative roadmap. This plan is the execution-oriented companion that explains sequencing, dependencies, stage outcomes, cross-cutting workstreams, and how contributors should move through the repository without introducing architecture drift.

## Planning principles

These principles govern the entire `plan/**` tree:

- **Every stage ends in a usable repository state.** No stage is allowed to defer basic buildability, documentation, or testability to a later cleanup phase.
- **Each stage should unlock a real maintainer or user workflow.** If a stage only adds scaffolding with no practical next action, it is too large or ordered incorrectly.
- **Complexity should be introduced only after the simpler prerequisite is stable.** Types and contracts come before algorithms, algorithms before reference validation, and validation before compression/release hardening.
- **Cross-cutting standards live outside the stage docs.** Sequencing belongs in `plan/stages/`, expectations in `plan/tracks/`, gates in `plan/checklists/`, and traceability material in `plan/appendices/`.
- **The plan must track the spec.** If sequencing changes because the spec changes, update both together instead of letting planning documents drift.

## Current execution status

The staged plan is no longer hypothetical. Based on the current repository state reflected in the stage documents:

| Stage | Status | Meaning for contributors |
| --- | --- | --- |
| 1. Workspace bootstrap | Complete | The workspace/tooling foundation exists and should now be treated as the baseline to preserve. |
| 2. Domain types and backend contract | Complete | Shared types and backend contracts are established; changes here should be deliberate and spec-driven. |
| 3. Chart MVP and algorithmic baseline | Complete | A usable chart workflow exists and should remain working while later stages expand breadth and confidence. |
| 4. Reference backend and validation | In progress / substantially landed | Validation and reference-backed comparison exist, but this area still evolves as coverage and reports improve. |
| 5. Compression and packaged data | Complete | Packaged-data support exists and should now be refined through validation rather than redesigned casually. |
| 6. Compatibility expansion and release hardening | Active | This is the main planning frontier: breadth completion, release discipline, and interoperability hardening. |

If you are deciding what to do next, start with the Stage 6 document and then consult the relevant track and checklist docs before making changes.

## Plan Index

### Start here

- [plan/overview.md](plan/overview.md) — orientation, reading order, directory usage, and how to navigate this plan set day to day

### Orientation

- [plan/overview.md](plan/overview.md) — how to read and maintain this plan set

### Checklists

- [plan/checklists/01-stage-gates.md](plan/checklists/01-stage-gates.md) — completion gates for each stage
- [plan/checklists/02-release-artifacts.md](plan/checklists/02-release-artifacts.md) — non-code outputs and release bundle expectations

### Appendices

- [plan/appendices/01-stage-to-spec-map.md](plan/appendices/01-stage-to-spec-map.md) — traceability from execution stages back to normative spec documents
- [plan/appendices/02-stage-workable-state-matrix.md](plan/appendices/02-stage-workable-state-matrix.md) — stage-by-stage summary of the minimum usable repository state and the workflow each stage unlocks

### Sequential stages

- [plan/stages/01-workspace-bootstrap.md](plan/stages/01-workspace-bootstrap.md) — create the reproducible workspace and enforce crate boundaries first
- [plan/stages/02-domain-types-and-backend-contract.md](plan/stages/02-domain-types-and-backend-contract.md) — lock down shared semantics before implementation breadth grows
- [plan/stages/03-chart-mvp-algorithmic-baseline.md](plan/stages/03-chart-mvp-algorithmic-baseline.md) — deliver the first useful chart workflow with pure-Rust algorithmic backends
- [plan/stages/04-reference-backend-and-validation.md](plan/stages/04-reference-backend-and-validation.md) — add source-backed validation and evidence-driven comparison
- [plan/stages/05-compression-and-packaged-data.md](plan/stages/05-compression-and-packaged-data.md) — ship the common-range packaged-data path for 1500-2500 CE
- [plan/stages/06-compatibility-expansion-and-release-hardening.md](plan/stages/06-compatibility-expansion-and-release-hardening.md) — finish breadth and make releases dependable

### Cross-cutting tracks

- [plan/tracks/01-workspace-and-tooling.md](plan/tracks/01-workspace-and-tooling.md)
- [plan/tracks/02-domain-and-public-api.md](plan/tracks/02-domain-and-public-api.md)
- [plan/tracks/03-backends-and-distribution.md](plan/tracks/03-backends-and-distribution.md)
- [plan/tracks/04-validation-and-release.md](plan/tracks/04-validation-and-release.md)

## Directory structure

This plan uses a deliberately small top-level structure so contributors can find the right planning document quickly without scattering ad hoc notes across the repository.

The current `plan/**` layout is intentionally simple and now separates **sequence**, **responsibility**, and **gates**:

- `plan/overview.md` — entry point and usage guidance
- `plan/stages/*.md` — the ordered delivery path; read these top to bottom
- `plan/tracks/*.md` — cross-cutting concerns that span multiple stages
- `plan/checklists/*.md` — shared completion gates and release-output expectations
- `plan/appendices/*.md` — traceability aids and other supporting reference material

This structure keeps the plan readable while still separating **sequence** from **responsibility**, **quality control**, and **traceability**:

- use a **stage document** to answer “what should happen next?”
- use a **track document** to answer “what standards apply to this area?”
- use a **checklist document** to answer “what must be true before we call this done?”

## Recommended reading paths

### For a new contributor

1. [SPEC.md](SPEC.md)
2. [plan/overview.md](plan/overview.md)
3. stage documents in order
4. [plan/checklists/01-stage-gates.md](plan/checklists/01-stage-gates.md)
5. the relevant track document for the area being worked on

### For maintainers planning the next milestone

1. review the current stage's exit criteria
2. verify the workable-state rule is satisfied
3. identify the next smallest reviewable increment inside the next stage
4. check track documents for cross-cutting requirements before implementation begins

### For contributors working on the current repository state

1. read [plan/stages/06-compatibility-expansion-and-release-hardening.md](plan/stages/06-compatibility-expansion-and-release-hardening.md) first
2. read the matching track doc for the subsystem being changed
3. use [plan/checklists/01-stage-gates.md](plan/checklists/01-stage-gates.md) for stage-level quality gates
4. use [plan/checklists/02-release-artifacts.md](plan/checklists/02-release-artifacts.md) if the work changes release-facing outputs
5. consult the appendices when a change might drift from the spec or from the workable-state rule

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
- known gaps versus the full specification are documented explicitly,
- contributors can tell which parts are production-ready, experimental, or not yet started.

## Stage outcomes at a glance

| Stage | Primary result | First unlocked workflow | Why it matters |
| --- | --- | --- | --- |
| 1 | Reproducible Rust workspace with correct crate boundaries | Contributor can clone, enter the managed tool environment, and run workspace checks | Prevents architecture drift from the start |
| 2 | Stable core types and backend contracts | Backend author can implement a toy backend and compile against shared APIs | Lets multiple backends evolve without API churn |
| 3 | First useful end-to-end chart workflow | User can generate a practical chart through `pleiades-core`/CLI with documented limits | Creates immediate product value |
| 4 | Validation-grade reference backend and reports | Maintainer can compare algorithmic results against reference data reproducibly | Grounds correctness claims in evidence |
| 5 | Fast packaged 1500-2500 backend | Application can ship a compact offline backend for the common date range | Serves the common deployment target |
| 6 | Broad compatibility plus release discipline | Release maintainer can publish a version with explicit coverage, evidence, and artifacts | Makes the project dependable for consumers |

## Stage dependency map

| Stage | Depends on | Enables next |
| --- | --- | --- |
| 1. Workspace bootstrap | none | all later work lands in the right crates and toolchain |
| 2. Domain types and backend contract | stage 1 | backend and domain implementations can proceed without repeated shared-type redesign |
| 3. Chart MVP with algorithmic baseline | stages 1-2 | practical end-user workflows and early consumer feedback |
| 4. Reference backend and validation | stages 1-3 | trustworthy comparisons, regression detection, and artifact-source generation |
| 5. Compression and packaged data | stages 1-4 | fast offline deployment for the common 1500-2500 window |
| 6. Compatibility expansion and release hardening | stages 1-5 | dependable releases with broad interoperability coverage |

## Recommended execution loop inside each stage

Use the same lightweight loop throughout the plan:

1. reread the stage doc and the relevant spec drivers,
2. pick the next smallest slice that preserves buildability,
3. check the relevant track doc for cross-cutting expectations,
4. implement code, tests, and docs together,
5. validate against the stage gate before calling the slice or stage complete.

This keeps the plan practical instead of treating stages as one large branch or milestone.

## Readiness checklist for moving between stages

Do not advance a stage just because code exists. Advance when the current stage also has:

- tests for the newly introduced behavior,
- updated docs or compatibility notes,
- explicit limits and failure modes,
- no layering violations against `spec/architecture.md`,
- a clear statement of what the next stage is allowed to assume.

Use [plan/checklists/01-stage-gates.md](plan/checklists/01-stage-gates.md) as the shared gate before moving forward.

## Slice-sizing rule inside each stage

Each stage should be implemented as a series of **small, independently shippable slices** rather than one large merge. A good slice:

- changes one architectural concern at a time,
- leaves the workspace buildable and testable,
- adds docs/tests together with behavior,
- makes the next slice simpler instead of compensating for hidden debt.

The stage documents describe recommended slice order so maintainers can keep progress incremental without losing the larger roadmap.

## Cross-cutting priorities

These priorities apply in every stage:

- preserve the crate layering from `spec/architecture.md`,
- keep the project pure Rust with no mandatory C/C++ dependencies,
- add tests and docs together with behavior,
- publish capability and compatibility information as features expand,
- prefer minimal, reviewable increments over speculative rewrites.

## Plan maintenance rules

When this repository evolves, update this plan set with the code/spec changes instead of letting it drift.

At minimum:

- update the relevant stage document when scope or sequencing changes,
- update the relevant track document when expectations or standards change,
- update the relevant checklist when completion or release expectations change,
- keep `PLAN.md` as the stable top-level index into `plan/**`,
- refresh the status snapshot near the top of `PLAN.md` when the active stage changes materially,
- avoid adding one-off planning files at the repository root when they belong under `plan/stages/`, `plan/tracks/`, or `plan/checklists/`,
