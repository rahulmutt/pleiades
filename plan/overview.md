# Plan Overview

This directory turns the specification into an execution plan that contributors can follow during day-to-day work.

Use it to answer four different questions:

- **Stages** — what should be built next?
- **Tracks** — what cross-cutting standards apply to this area?
- **Checklists** — what must be true before we call this done?
- **Appendices** — how does the plan trace back to the spec?

## Directory layout

- [plan/overview.md](overview.md) — orientation and maintenance guidance
- [plan/stages/](stages/) — sequential implementation stages from foundations through release hardening
- [plan/status/](status/) — current execution-frontier guidance and next-slice selection help
- [plan/tracks/](tracks/) — cross-cutting workstreams and subsystem-specific expectations
- [plan/checklists/](checklists/) — reusable completion gates and release-output expectations
- [plan/appendices/](appendices/) — traceability aids and supporting reference material

The structure is intentionally shallow so planning notes do not sprawl into unrelated root-level documents, and so sequence, execution guidance, and governance material stay easy to find.

## Stage-first rule

The project should remain in a workable state after every stage.

That means each stage should end with:

- a buildable workspace,
- tests for the newly introduced behavior,
- enough documentation for the next contributor to continue safely,
- explicit notes about what is still out of scope,
- no ambiguity about whether the next stage extends a stable base or compensates for missing foundations.

Inside each stage, prefer the smallest reviewable slice that still preserves one coherent user or maintainer workflow.

## Recommended reading order

### Starting from scratch

1. [PLAN.md](../PLAN.md)
2. [SPEC.md](../SPEC.md)
3. [plan/stages/01-workspace-bootstrap.md](stages/01-workspace-bootstrap.md)
4. the remaining stage documents in order
5. the relevant track and checklist documents for the area you are changing

### Planning the next slice

1. [PLAN.md](../PLAN.md)
2. the active stage document
3. [plan/status/01-current-execution-frontier.md](status/01-current-execution-frontier.md)
4. [plan/status/02-next-slice-candidates.md](status/02-next-slice-candidates.md)
5. the relevant track document
6. [plan/checklists/01-stage-gates.md](checklists/01-stage-gates.md)
7. [plan/checklists/02-release-artifacts.md](checklists/02-release-artifacts.md) when release-facing output is affected

## Stage list

- [plan/stages/01-workspace-bootstrap.md](stages/01-workspace-bootstrap.md)
- [plan/stages/02-domain-types-and-backend-contract.md](stages/02-domain-types-and-backend-contract.md)
- [plan/stages/03-chart-mvp-algorithmic-baseline.md](stages/03-chart-mvp-algorithmic-baseline.md)
- [plan/stages/04-reference-backend-and-validation.md](stages/04-reference-backend-and-validation.md)
- [plan/stages/05-compression-and-packaged-data.md](stages/05-compression-and-packaged-data.md)
- [plan/stages/06-compatibility-expansion-and-release-hardening.md](stages/06-compatibility-expansion-and-release-hardening.md)

## Status list

- [plan/status/01-current-execution-frontier.md](status/01-current-execution-frontier.md)
- [plan/status/02-next-slice-candidates.md](status/02-next-slice-candidates.md)

## Track list

- [plan/tracks/01-workspace-and-tooling.md](tracks/01-workspace-and-tooling.md)
- [plan/tracks/02-domain-and-public-api.md](tracks/02-domain-and-public-api.md)
- [plan/tracks/03-backends-and-distribution.md](tracks/03-backends-and-distribution.md)
- [plan/tracks/04-validation-and-release.md](tracks/04-validation-and-release.md)

## Checklist list

- [plan/checklists/01-stage-gates.md](checklists/01-stage-gates.md)
- [plan/checklists/02-release-artifacts.md](checklists/02-release-artifacts.md)

## Appendix list

- [plan/appendices/01-stage-to-spec-map.md](appendices/01-stage-to-spec-map.md)
- [plan/appendices/02-stage-workable-state-matrix.md](appendices/02-stage-workable-state-matrix.md)

## Maintenance guidance

When revising the plan:

- keep `PLAN.md` as the top-level index and execution summary,
- make sequencing changes in `plan/stages/`,
- make current-frontier or next-slice guidance changes in `plan/status/`,
- make cross-cutting policy or subsystem expectation changes in `plan/tracks/`,
- make completion or release-output changes in `plan/checklists/`,
- update appendices when traceability or stage-to-spec mapping changes,
- keep the plan aligned with `SPEC.md` and the normative `spec/*.md` documents,
- avoid adding new planning files outside the structured `plan/` tree unless the planning model itself changes.
