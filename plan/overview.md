# Plan Overview

This directory turns the product specification into an execution plan that is easier to follow during day-to-day development.

The plan is organized in two complementary views:

- **Stages**: the main delivery sequence from foundations to release hardening.
- **Tracks**: cross-cutting workstreams that run through multiple stages.

Use the stages when deciding **what to build next**. Use the tracks when deciding **where a task belongs** and what other work it depends on.

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
- [plan/tracks/](tracks/) — cross-cutting workstreams and ownership boundaries

## Stage-first Rule

The repository should remain in a workable state after every stage. That means each stage should end with:

- a buildable workspace,
- tests that cover the new behavior,
- enough documentation for contributors to continue,
- explicit notes about what is not done yet.

## Track List

- [plan/tracks/01-workspace-and-tooling.md](tracks/01-workspace-and-tooling.md)
- [plan/tracks/02-domain-and-public-api.md](tracks/02-domain-and-public-api.md)
- [plan/tracks/03-backends-and-distribution.md](tracks/03-backends-and-distribution.md)
- [plan/tracks/04-validation-and-release.md](tracks/04-validation-and-release.md)
