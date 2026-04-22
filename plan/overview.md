# Plan Overview

This directory turns the product specification into an execution plan that is easier to follow during day-to-day development.

The plan is organized in three complementary views:

- **Stages**: the main delivery sequence from foundations to release hardening.
- **Tracks**: cross-cutting workstreams that run through multiple stages.
- **Checklists**: shared completion gates and release-output expectations.

Use the stages when deciding **what to build next**. Use the tracks when deciding **where a task belongs** and what other work it depends on. Use the checklists when deciding **whether a stage or release is actually done**.

## Reading Order

If you are starting from scratch, read in this order:

1. [PLAN.md](../PLAN.md)
2. [plan/overview.md](overview.md)
3. [plan/stages/01-workspace-bootstrap.md](stages/01-workspace-bootstrap.md)
4. the remaining stage documents in order
5. the relevant track document for the area you are changing

## Directory Layout

- [plan/overview.md](overview.md) — how to use the plan
- [plan/stages/](stages/) — sequential implementation stages
- [plan/tracks/](tracks/) — cross-cutting workstreams and quality boundaries
- [plan/checklists/](checklists/) — reusable stage gates and release-output expectations

The structure is intentionally shallow so contributors can find the current plan quickly:

- if the question is about **ordering**, start in `plan/stages/`
- if the question is about **standards or scope for a subsystem**, start in `plan/tracks/`
- if the question is about **completion criteria or release outputs**, start in `plan/checklists/`

## Stage-first Rule

The repository should remain in a workable state after every stage. That means each stage should end with:

- a buildable workspace,
- tests that cover the new behavior,
- enough documentation for contributors to continue,
- explicit notes about what is not done yet,
- no ambiguity about whether the next stage is extending a stable base or cleaning up missing foundations.

## Track List

- [plan/tracks/01-workspace-and-tooling.md](tracks/01-workspace-and-tooling.md)
- [plan/tracks/02-domain-and-public-api.md](tracks/02-domain-and-public-api.md)
- [plan/tracks/03-backends-and-distribution.md](tracks/03-backends-and-distribution.md)
- [plan/tracks/04-validation-and-release.md](tracks/04-validation-and-release.md)

## Checklist List

- [plan/checklists/01-stage-gates.md](checklists/01-stage-gates.md)
- [plan/checklists/02-release-artifacts.md](checklists/02-release-artifacts.md)

## Maintenance guidance

When adding or revising planning material:

- prefer editing the existing stage, track, or checklist document instead of creating overlapping notes,
- keep `PLAN.md` as the authoritative top-level index,
- make sequencing changes in stage documents, policy/quality changes in track documents, and completion-output changes in checklist documents,
- ensure plan changes still reflect `SPEC.md` and the normative documents in `spec/*.md`.
