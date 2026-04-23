# Pleiades Development Plan

This document is the top-level execution map for `pleiades`.

It translates `SPEC.md` and the normative documents in `spec/*.md` into a staged delivery plan that:

- starts with the simplest foundations,
- preserves a workable repository state after every stage,
- delivers useful user and maintainer workflows as early as possible,
- defers breadth, optimization, and release polish until the lower layers are stable,
- points to the detailed planning material under `plan/**`.

`spec/roadmap.md` remains the concise roadmap. This file is the practical index for day-to-day execution.

## Source material

This plan is derived from:

- [SPEC.md](SPEC.md)
- [spec/vision-and-scope.md](spec/vision-and-scope.md)
- [spec/requirements.md](spec/requirements.md)
- [spec/architecture.md](spec/architecture.md)
- [spec/backend-trait.md](spec/backend-trait.md)
- [spec/astrology-domain.md](spec/astrology-domain.md)
- [spec/data-compression.md](spec/data-compression.md)
- [spec/backends.md](spec/backends.md)
- [spec/api-and-ergonomics.md](spec/api-and-ergonomics.md)
- [spec/validation-and-testing.md](spec/validation-and-testing.md)
- [spec/roadmap.md](spec/roadmap.md)

## Planning principles

These rules shape the whole `plan/**` tree:

1. **Workable after every stage.** Each stage must leave the workspace buildable, testable, and understandable enough for the next contributor.
2. **Simple before complex.** Shared types, contracts, and baseline domain behavior come before reference backends, compression, and release hardening.
3. **Useful outcomes, not scaffolding only.** Every stage must unlock at least one concrete maintainer or user workflow.
4. **Spec-first evolution.** If implementation direction changes, update plan and spec together rather than letting them drift.
5. **Small reviewable slices inside each stage.** A stage is a sequence of shippable increments, not one large merge.
6. **Cross-cutting concerns stay separate from sequencing.** Stages describe order; tracks describe standards; checklists describe completion gates; appendices describe traceability.

## Plan directory structure

The planning documents are organized by purpose:

```text
PLAN.md                     # top-level index and staged execution summary
plan/
  overview.md               # orientation, reading order, maintenance guidance
  stages/                   # sequential delivery path
  status/                   # current frontier and next-slice guidance
  tracks/                   # cross-cutting expectations by subsystem
  checklists/               # reusable done/release gates
  appendices/               # traceability and supporting reference material
```

### Directory index

- [plan/overview.md](plan/overview.md) — orientation and how to use the plan set
- [plan/stages/](plan/stages/) — the main development sequence
- [plan/status/](plan/status/) — current priority and next-slice guidance
- [plan/tracks/](plan/tracks/) — subsystem and cross-cutting expectations
- [plan/checklists/](plan/checklists/) — stage-completion and release gates
- [plan/appendices/](plan/appendices/) — stage-to-spec mapping and workable-state reference material

This structure is intentionally role-based rather than date-based so planning stays navigable as the project grows.

## Development ladder

The project should move through the following stages in order.

| Stage | Focus | Why it comes here | Workable-state promise | Detailed doc |
| --- | --- | --- | --- | --- |
| 1 | Workspace bootstrap | Reproducible tooling and crate boundaries must exist before deeper implementation work | A maintainer can clone the repo, enter the managed environment, and run standard checks | [plan/stages/01-workspace-bootstrap.md](plan/stages/01-workspace-bootstrap.md) |
| 2 | Domain types and backend contract | Shared semantics and interfaces must stabilize before backend and domain breadth expands | A backend author can implement against the common contract without redesigning core types | [plan/stages/02-domain-types-and-backend-contract.md](plan/stages/02-domain-types-and-backend-contract.md) |
| 3 | Chart MVP and algorithmic baseline | The first useful astrology workflow should arrive before heavier reference and packaging work | A caller can compute a baseline chart through `pleiades-core` and the CLI with explicit limits | [plan/stages/03-chart-mvp-algorithmic-baseline.md](plan/stages/03-chart-mvp-algorithmic-baseline.md) |
| 4 | Reference backend and validation | Accuracy evidence should be added only after an end-to-end workflow exists | A maintainer can compare results, validate assumptions, and capture regressions reproducibly | [plan/stages/04-reference-backend-and-validation.md](plan/stages/04-reference-backend-and-validation.md) |
| 5 | Compression and packaged data | Packaged offline distribution should be built on validated source outputs and stable formats | An application can use a compact 1500-2500 backend with documented capability limits | [plan/stages/05-compression-and-packaged-data.md](plan/stages/05-compression-and-packaged-data.md) |
| 6 | Compatibility expansion and release hardening | Breadth and release rigor should come after the architecture and common workflows are proven | A release maintainer can publish explicit coverage, validation evidence, and reproducible artifacts | [plan/stages/06-compatibility-expansion-and-release-hardening.md](plan/stages/06-compatibility-expansion-and-release-hardening.md) |

## What each stage must leave behind

To keep development incremental and safe, every completed stage should leave behind:

- a buildable workspace,
- focused tests for the introduced behavior,
- documentation for newly exposed user or maintainer workflows,
- explicit limits and known gaps,
- no architectural shortcuts that make later compatibility expansion harder.

## Current planning posture

The detailed execution frontier lives under `plan/status/`, but the top-level plan should make the current posture obvious.

| Stage | Status | Summary |
| --- | --- | --- |
| 1. Workspace bootstrap | Complete | Managed workspace, crate layout, and reproducible tooling are established |
| 2. Domain types and backend contract | Complete | Shared semantics and backend-facing contracts are in place |
| 3. Chart MVP and algorithmic baseline | Complete | Baseline chart workflows exist with domain-layer house and ayanamsa support |
| 4. Reference backend and validation | Complete | Source-backed validation and regression tooling are available |
| 5. Compression and packaged data | Complete | Packaged artifact and bundled backend support the common date range |
| 6. Compatibility expansion and release hardening | Active | Current work should focus on coverage clarity, release integrity, and remaining catalog breadth |

For the live execution frontier, see:

- [plan/status/01-current-execution-frontier.md](plan/status/01-current-execution-frontier.md)
- [plan/status/02-next-slice-candidates.md](plan/status/02-next-slice-candidates.md)

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
4. read the relevant track document for cross-cutting expectations
5. confirm the applicable checklist before calling the work done

### If you are making a release-facing or compatibility-facing change

1. [plan/stages/06-compatibility-expansion-and-release-hardening.md](plan/stages/06-compatibility-expansion-and-release-hardening.md)
2. [plan/tracks/04-validation-and-release.md](plan/tracks/04-validation-and-release.md)
3. [plan/checklists/01-stage-gates.md](plan/checklists/01-stage-gates.md)
4. [plan/checklists/02-release-artifacts.md](plan/checklists/02-release-artifacts.md)

## Detailed plan index

### Orientation

- [plan/overview.md](plan/overview.md)

### Sequential stages

- [plan/stages/01-workspace-bootstrap.md](plan/stages/01-workspace-bootstrap.md)
- [plan/stages/02-domain-types-and-backend-contract.md](plan/stages/02-domain-types-and-backend-contract.md)
- [plan/stages/03-chart-mvp-algorithmic-baseline.md](plan/stages/03-chart-mvp-algorithmic-baseline.md)
- [plan/stages/04-reference-backend-and-validation.md](plan/stages/04-reference-backend-and-validation.md)
- [plan/stages/05-compression-and-packaged-data.md](plan/stages/05-compression-and-packaged-data.md)
- [plan/stages/06-compatibility-expansion-and-release-hardening.md](plan/stages/06-compatibility-expansion-and-release-hardening.md)

### Status and next-slice guidance

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

## Plan maintenance rules

When the plan changes:

- update `PLAN.md` when the top-level structure, stage ordering, or planning model changes,
- update `plan/stages/` when sequencing or stage outcomes change,
- update `plan/status/` when the current frontier or best next slices change,
- update `plan/tracks/` when cross-cutting standards change,
- update `plan/checklists/` when completion or release gates change,
- update `plan/appendices/` when traceability or workable-state references change,
- keep links current and avoid ad hoc root-level planning notes outside the structured `plan/` tree.

## Traceability

For explicit stage-to-spec mapping, see [plan/appendices/01-stage-to-spec-map.md](plan/appendices/01-stage-to-spec-map.md).

For the per-stage workable-state reference, see [plan/appendices/02-stage-workable-state-matrix.md](plan/appendices/02-stage-workable-state-matrix.md).

Status: Updated 2026-04-23 after review against `SPEC.md` and the current `spec/*.md` set.
