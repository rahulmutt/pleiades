# Pleiades Development Plan

This document is the top-level execution guide for `pleiades`.

It translates `SPEC.md` and the normative documents in `spec/*.md` into an implementation order that:

- starts with the simplest foundational work,
- adds real user and maintainer value early,
- keeps the repository buildable and understandable at every stage,
- defers heavier optimization and compatibility breadth until the core seams are proven.

`spec/roadmap.md` is the concise roadmap. This `PLAN.md` is the practical index into the more detailed planning documents under `plan/**`.

## Planning rules

These rules apply to the whole plan set:

1. **Workable state after every stage.** Each stage must leave the repo buildable, testable, documented enough for the next contributor, and usable for at least one realistic workflow.
2. **Simple before complex.** Tooling, types, contracts, and baseline domain logic come before reference data, compression, and release hardening.
3. **Spec-first sequencing.** Planning documents must stay aligned with `SPEC.md` and the normative `spec/*.md` files.
4. **Small shippable slices inside each stage.** A stage is not one giant merge; it is a sequence of reviewable increments that preserve repository health.
5. **Clear separation of sequence, standards, and gates.** Stage ordering lives in `plan/stages/`, cross-cutting expectations live in `plan/tracks/`, completion checks live in `plan/checklists/`, and traceability material lives in `plan/appendices/`.
6. **One place per planning concern.** Do not add ad hoc root-level planning notes when the same material belongs in an existing `plan/` subdirectory.

## Plan structure at a glance

The planning tree is intentionally organized by purpose instead of by date:

```text
PLAN.md                     # top-level index and execution summary
plan/
  overview.md               # orientation and reading order
  stages/                   # sequential delivery path
  status/                   # current frontier and next-slice guidance
  tracks/                   # cross-cutting expectations by subsystem
  checklists/               # reusable done/release gates
  appendices/               # traceability and supporting reference material
```

This structure is a good fit for the current repository because it keeps three different planning needs separate:

- **sequence**: what comes next overall,
- **execution**: what the best next slice is right now,
- **governance**: what must stay true across all slices.

## Recommended implementation order

The project should be advanced in this order:

| Stage | Focus | Why this comes now | Detailed doc |
| --- | --- | --- | --- |
| 1 | Workspace bootstrap | Establish reproducible tooling, crate boundaries, and contributor workflows first | [plan/stages/01-workspace-bootstrap.md](plan/stages/01-workspace-bootstrap.md) |
| 2 | Domain types and backend contract | Stable shared semantics are needed before backend/domain breadth grows | [plan/stages/02-domain-types-and-backend-contract.md](plan/stages/02-domain-types-and-backend-contract.md) |
| 3 | Chart MVP and algorithmic baseline | Deliver the first useful astrology workflow with pure-Rust baseline backends | [plan/stages/03-chart-mvp-algorithmic-baseline.md](plan/stages/03-chart-mvp-algorithmic-baseline.md) |
| 4 | Reference backend and validation | Add higher-confidence comparison data only after an end-to-end workflow exists | [plan/stages/04-reference-backend-and-validation.md](plan/stages/04-reference-backend-and-validation.md) |
| 5 | Compression and packaged data | Turn validated results into a practical 1500-2500 offline distribution path | [plan/stages/05-compression-and-packaged-data.md](plan/stages/05-compression-and-packaged-data.md) |
| 6 | Compatibility expansion and release hardening | Complete breadth, document shipped coverage, and make releases dependable | [plan/stages/06-compatibility-expansion-and-release-hardening.md](plan/stages/06-compatibility-expansion-and-release-hardening.md) |

## Current stage snapshot

The detailed status lives under `plan/status/`, but the top-level plan should also make the current posture obvious.

| Stage | Status | Primary outcome today |
| --- | --- | --- |
| 1. Workspace bootstrap | Complete | Reproducible workspace, tooling, CI, and crate skeletons are in place |
| 2. Domain types and backend contract | Complete | Shared semantic foundation and backend contract are established |
| 3. Chart MVP and algorithmic baseline | Complete | End-to-end chart workflow exists with baseline houses and ayanamsas |
| 4. Reference backend and validation | In use | Reference snapshots and validation tooling exist and support regression checking |
| 5. Compression and packaged data | Complete | Packaged artifact format and bundled backend exist for the common range |
| 6. Compatibility expansion and release hardening | Active | Current work should focus on compatibility clarity, release integrity, and remaining breadth |

For the live execution frontier, see [plan/status/01-current-execution-frontier.md](plan/status/01-current-execution-frontier.md).

## Stage outcomes and workable-state expectations

Each stage should unlock a concrete workflow rather than only adding scaffolding.

| Stage | Minimum outcome | Workable-state expectation |
| --- | --- | --- |
| 1 | Workspace, toolchain, crate skeletons, contributor commands | A maintainer can clone the repo, enter the managed environment, and run the standard checks |
| 2 | Stable shared types, backend trait, capability metadata, basic façade seams | A backend author can implement against the shared contract without redesigning fundamentals |
| 3 | First practical chart path with baseline bodies, houses, and ayanamsas | A user or maintainer can compute a basic chart through `pleiades-core` and the CLI with documented limits |
| 4 | Reference-backed validation and comparison tooling | A maintainer can measure baseline accuracy and detect regressions reproducibly |
| 5 | Packaged compressed-data backend for 1500-2500 CE | An application can use a compact offline backend for the common date range |
| 6 | Broader compatibility catalog, release profiles, hardened tooling | A release maintainer can publish a version with explicit coverage, evidence, and reproducible artifacts |

## How to use the `plan/**` tree

The current directory structure is intentionally shallow and role-based:

- [plan/overview.md](plan/overview.md) — orientation, reading order, and maintenance guidance
- [plan/stages/](plan/stages/) — the sequential delivery path; use these to answer **what should happen next in the overall program**
- [plan/status/](plan/status/) — current execution-frontier notes; use these to answer **what is the most sensible next slice right now**
- [plan/tracks/](plan/tracks/) — cross-cutting standards by area; use these to answer **what else does this work affect**
- [plan/checklists/](plan/checklists/) — completion and release gates; use these to answer **what must be true before this is done**
- [plan/appendices/](plan/appendices/) — traceability and supporting reference material

A good default workflow is:

1. read the relevant spec documents,
2. read the active stage document,
3. read the current status note,
4. read the relevant track doc,
5. confirm the applicable checklist before calling work done.

## Plan index

### Orientation

- [plan/overview.md](plan/overview.md)

### Sequential stages

- [plan/stages/01-workspace-bootstrap.md](plan/stages/01-workspace-bootstrap.md)
- [plan/stages/02-domain-types-and-backend-contract.md](plan/stages/02-domain-types-and-backend-contract.md)
- [plan/stages/03-chart-mvp-algorithmic-baseline.md](plan/stages/03-chart-mvp-algorithmic-baseline.md)
- [plan/stages/04-reference-backend-and-validation.md](plan/stages/04-reference-backend-and-validation.md)
- [plan/stages/05-compression-and-packaged-data.md](plan/stages/05-compression-and-packaged-data.md)
- [plan/stages/06-compatibility-expansion-and-release-hardening.md](plan/stages/06-compatibility-expansion-and-release-hardening.md)

### Current status and next-slice guidance

- [plan/status/01-current-execution-frontier.md](plan/status/01-current-execution-frontier.md)
- [plan/status/02-next-slice-candidates.md](plan/status/02-next-slice-candidates.md)

### Cross-cutting tracks

- [plan/tracks/01-workspace-and-tooling.md](plan/tracks/01-workspace-and-tooling.md)
- [plan/tracks/02-domain-and-public-api.md](plan/tracks/02-domain-and-public-api.md)
- [plan/tracks/03-backends-and-distribution.md](plan/tracks/03-backends-and-distribution.md)
- [plan/tracks/04-validation-and-release.md](plan/tracks/04-validation-and-release.md)

### Checklists

- [plan/checklists/01-stage-gates.md](plan/checklists/01-stage-gates.md)
- [plan/checklists/02-release-artifacts.md](plan/checklists/02-release-artifacts.md)

### Appendices

- [plan/appendices/01-stage-to-spec-map.md](plan/appendices/01-stage-to-spec-map.md)
- [plan/appendices/02-stage-workable-state-matrix.md](plan/appendices/02-stage-workable-state-matrix.md)

## Reading paths

### If you are new to the repository

1. [SPEC.md](SPEC.md)
2. [plan/overview.md](plan/overview.md)
3. the stage documents in order
4. [plan/checklists/01-stage-gates.md](plan/checklists/01-stage-gates.md)
5. the relevant track document for your subsystem

### If you are planning the next implementation slice

1. reread the active stage document
2. read [plan/status/01-current-execution-frontier.md](plan/status/01-current-execution-frontier.md)
3. use [plan/status/02-next-slice-candidates.md](plan/status/02-next-slice-candidates.md) to choose a slice shape
4. check the relevant track document for cross-cutting expectations
5. choose the smallest slice that preserves a workable repository state
6. implement code, tests, and docs together
7. verify the stage and release checklists before calling the slice done

### If you are validating a release-oriented change

1. [plan/stages/06-compatibility-expansion-and-release-hardening.md](plan/stages/06-compatibility-expansion-and-release-hardening.md)
2. [plan/tracks/04-validation-and-release.md](plan/tracks/04-validation-and-release.md)
3. [plan/checklists/01-stage-gates.md](plan/checklists/01-stage-gates.md)
4. [plan/checklists/02-release-artifacts.md](plan/checklists/02-release-artifacts.md)

## Traceability back to the spec

The plan is derived from these key normative sources:

- [SPEC.md](SPEC.md)
- [spec/requirements.md](spec/requirements.md)
- [spec/architecture.md](spec/architecture.md)
- [spec/backend-trait.md](spec/backend-trait.md)
- [spec/astrology-domain.md](spec/astrology-domain.md)
- [spec/data-compression.md](spec/data-compression.md)
- [spec/api-and-ergonomics.md](spec/api-and-ergonomics.md)
- [spec/validation-and-testing.md](spec/validation-and-testing.md)

For a stage-to-spec mapping, see [plan/appendices/01-stage-to-spec-map.md](plan/appendices/01-stage-to-spec-map.md).

## Maintenance rules

When changing scope, sequencing, or release expectations:

- update the relevant stage doc for sequencing changes,
- update `plan/status/` when the current execution frontier or default next slices change,
- update the relevant track doc for cross-cutting standards,
- update the relevant checklist for completion or release-output changes,
- update `plan/appendices/` when traceability or workable-state expectations change,
- keep this file as the stable top-level index into `plan/**`,
- avoid ad hoc root-level planning notes when the material belongs in the structured `plan/` tree.

Status: Updated 2026-04-23 after review against `SPEC.md` and the current `spec/*.md` set.
