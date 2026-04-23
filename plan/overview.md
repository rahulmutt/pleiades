# Plan Overview

This directory turns the product specification into an execution plan that is easier to follow during day-to-day development.

The plan is organized in four complementary views:

- **Stages**: the main delivery sequence from foundations to release hardening.
- **Tracks**: cross-cutting workstreams that run through multiple stages.
- **Checklists**: shared completion gates and release-output expectations.
- **Appendices**: reference material that helps maintainers keep the plan aligned with the spec.

Use the stages when deciding **what to build next**. Use the tracks when deciding **where a task belongs** and what other work it depends on. Use the checklists when deciding **whether a stage or release is actually done**.

## Current status snapshot

The repository has already progressed beyond pure bootstrap planning.

- Stages 1, 2, 3, and 5 are treated as completed foundations in the stage documents.
- Stage 4 validation work is substantially in place and should be extended carefully, not restarted.
- Stage 6 compatibility expansion and release hardening is the active planning frontier.

That means most contributors should read the earlier stages as **constraints and preserved foundations**, then treat Stage 6 plus the track/checklist docs as the main guide for ongoing work.

## Reading Order

If you are starting from scratch, read in this order:

1. [PLAN.md](../PLAN.md)
2. [plan/overview.md](overview.md)
3. [plan/stages/01-workspace-bootstrap.md](stages/01-workspace-bootstrap.md)
4. the remaining stage documents in order
5. the relevant track document for the area you are changing

If you are working on the current active roadmap, read in this order:

1. [PLAN.md](../PLAN.md)
2. [plan/stages/06-compatibility-expansion-and-release-hardening.md](stages/06-compatibility-expansion-and-release-hardening.md)
3. the relevant track document
4. [plan/checklists/01-stage-gates.md](checklists/01-stage-gates.md)
5. [plan/checklists/02-release-artifacts.md](checklists/02-release-artifacts.md) when release-facing output is affected

## Directory Layout

- [plan/overview.md](overview.md) — how to use the plan
- [plan/stages/](stages/) — sequential implementation stages
- [plan/tracks/](tracks/) — cross-cutting workstreams and quality boundaries
- [plan/checklists/](checklists/) — reusable stage gates and release-output expectations
- [plan/appendices/](appendices/) — supporting reference material and traceability aids

The structure is intentionally shallow so contributors can find the current plan quickly:

- if the question is about **ordering**, start in `plan/stages/`
- if the question is about **standards or scope for a subsystem**, start in `plan/tracks/`
- if the question is about **completion criteria or release outputs**, start in `plan/checklists/`
- if the question is about **which spec documents govern a stage**, start in `plan/appendices/`

## Stage-first Rule

The repository should remain in a workable state after every stage. That means each stage should end with:

- a buildable workspace,
- tests that cover the new behavior,
- enough documentation for contributors to continue,
- explicit notes about what is not done yet,
- no ambiguity about whether the next stage is extending a stable base or cleaning up missing foundations.

Inside a stage, prefer the smallest slice that still leaves one coherent maintainer workflow intact. The stage documents therefore include suggested slice sequencing, not just destination-state goals.

## Track List

- [plan/tracks/01-workspace-and-tooling.md](tracks/01-workspace-and-tooling.md)
- [plan/tracks/02-domain-and-public-api.md](tracks/02-domain-and-public-api.md)
- [plan/tracks/03-backends-and-distribution.md](tracks/03-backends-and-distribution.md)
- [plan/tracks/04-validation-and-release.md](tracks/04-validation-and-release.md)

## Checklist List

- [plan/checklists/01-stage-gates.md](checklists/01-stage-gates.md)
- [plan/checklists/02-release-artifacts.md](checklists/02-release-artifacts.md)

## Appendix List

- [plan/appendices/01-stage-to-spec-map.md](appendices/01-stage-to-spec-map.md)
- [plan/appendices/02-stage-workable-state-matrix.md](appendices/02-stage-workable-state-matrix.md)

## Maintenance guidance

When adding or revising planning material:

- prefer editing the existing stage, track, or checklist document instead of creating overlapping notes,
- keep `PLAN.md` as the authoritative top-level index,
- make sequencing changes in stage documents, policy/quality changes in track documents, and completion-output changes in checklist documents,
- update the status snapshot here and in `PLAN.md` when the active stage meaningfully changes,
- ensure plan changes still reflect `SPEC.md` and the normative documents in `spec/*.md`.
