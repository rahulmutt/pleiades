# Plan Overview

This directory contains the forward-looking implementation plan for Pleiades. It intentionally omits completed bootstrap, crate-skeleton, baseline API, MVP workflow, and release-rehearsal scaffolding tasks so maintainers can focus on remaining specification gaps.

## How to use this plan

1. Read `SPEC.md` and the relevant files under `spec/`.
2. Read `PLAN.md` for the current phase ladder.
3. Read `plan/status/01-current-execution-frontier.md` to understand the active frontier.
4. Choose a focused slice from `plan/status/02-next-slice-candidates.md`.
5. Check the relevant track document for cross-cutting constraints.
6. Use `plan/checklists/01-phase-gates.md` before considering a phase milestone done.

## Directory guide

- `stages/`: remaining implementation phases only.
- `status/`: current active frontier and suggested next slices.
- `tracks/`: durable standards that apply across phases.
- `checklists/`: reusable completion and release gates.
- `appendices/`: traceability from phases to specification requirements and workable-state promises.

## Current foundation already in place

The workspace already contains:

- all mandatory first-party crates named with the `pleiades-*` prefix;
- shared typed models for bodies, time, coordinates, houses, ayanamsas, observers, custom definitions, and compatibility metadata;
- backend traits, metadata validation, batch behavior, errors, and composite/routing helpers;
- domain crates for house and ayanamsa catalogs plus chart-facing sidereal/house workflows;
- source-backed VSOP87B generated tables for the Sun through Neptune, a compact lunar baseline, a JPL snapshot fixture, and a prototype packaged-data backend;
- compression data structures, artifact validation, checksums, regeneration helpers, and packaged lookup behavior;
- CLI and validation/reporting commands for profiles, matrices, artifacts, reports, audits, benchmarks, release summaries, and bundle verification.

## Current remaining work at a glance

- Close ephemeris accuracy gaps, especially Pluto and release-grade source/reference evidence.
- Produce a real 1500-2500 CE compressed artifact with acceptable measured fit error.
- Finish house/ayanamsa formula, alias, sidereal metadata, and failure-mode evidence.
- Decide whether advanced request modes are implemented now or explicitly deferred.
- Promote rehearsal release tooling into final release gates and documentation.

## Planning maintenance

When implementation closes a remaining gap, remove that task from the active phase document and update the status files. Do not keep completed task lists as historical records; git history provides that context.

When a new spec requirement is added, map it into an active or queued phase and update `plan/appendices/01-phase-to-spec-map.md`.
