# Plan Overview

This directory contains the forward-looking implementation plan for Pleiades. It omits completed bootstrap, crate skeleton, MVP API, report-surface, and release-rehearsal work so maintainers can focus on remaining specification gaps.

## How to use this plan

1. Read `SPEC.md` and the relevant files under `spec/`.
2. Read `PLAN.md` for the current phase ladder.
3. Read `plan/status/01-current-execution-frontier.md` for the active frontier.
4. Choose a focused slice from `plan/status/02-next-slice-candidates.md`.
5. Check the relevant track document for cross-cutting constraints.
6. Use `plan/checklists/01-phase-gates.md` before considering a phase milestone done.

## Directory guide

- `stages/`: remaining implementation phases only.
- `status/`: current active frontier and suggested next slices.
- `tracks/`: durable standards that apply across phases.
- `checklists/`: reusable completion and release gates.
- `appendices/`: traceability from phases to specification requirements and workable-state promises.

## Foundation already in place

The workspace already has the required crate family, backend abstraction, typed domain model, chart façade, broad catalogs, validation/reporting commands, release-bundle rehearsal tooling, VSOP87B generated tables for Sun-through-Neptune, a compact lunar baseline, JPL snapshot fixtures, compression codecs, and a deterministic prototype packaged-data backend.

## Current remaining work at a glance

- Broaden reference-grade ephemeris evidence and decide the Pluto/lunar release posture.
- Finalize built-in versus deferred behavior for Delta T, UTC convenience, apparent-place, topocentric body-position, native sidereal, and frame precision semantics.
- Produce a production 1500-2500 CE compressed artifact with reproducible generation and acceptable measured fit error.
- Finish house/ayanamsa formula, alias, provenance, sidereal metadata, custom-definition, and failure-mode evidence.
- Promote release rehearsal outputs into blocking release gates and final documentation.

## Planning maintenance

When implementation closes a remaining gap, remove that task from the active phase and status files. Do not keep completed task lists as historical records; git history and validation reports provide that context.

When a new spec requirement is added, map it into an active or queued phase and update `plan/appendices/01-phase-to-spec-map.md`.
