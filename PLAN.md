# Pleiades Development Plan

This document is the top-level execution map for `pleiades` after the initial workspace, baseline domain API, validation shell, and packaged-data scaffolding have landed.

It translates the remaining requirements in `SPEC.md` and `spec/*.md` into a forward-looking plan. Completed bootstrap and MVP tasks are intentionally not listed here; use git history for those details. This plan now focuses only on work still needed to make Pleiades a production-quality, Swiss-Ephemeris-class, pure-Rust ephemeris workspace.

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

## Current implementation baseline

The repository currently has all required first-party crates, shared domain types, a backend trait, composite routing helpers, a chart façade, baseline and expanded house/ayanamsa catalogs, preliminary algorithmic/snapshot/packaged backends, CLI and validation commands, release-profile reporting, bundle verification, and tests for those surfaces.

Those capabilities are a strong foundation, but several spec requirements are not yet production-complete:

- the `pleiades-vsop87`, `pleiades-elp`, and `pleiades-jpl` crates need production-grade astronomical algorithms/readers and documented error envelopes, not only deterministic sample or simplified behavior;
- the compressed-data pipeline needs a reproducible generator from public source inputs, measured fit error, and versioned binary artifacts for 1500-2500 CE;
- compatibility catalogs need remaining formula validation, alias audits, latitude/numerical failure evidence, and release-profile truthfulness checks;
- topocentric handling, Delta T policy, apparent/mean semantics, and equatorial/ecliptic transforms need stronger end-to-end implementation and documentation;
- release gates need to be exercised on real artifacts and validation reports.

## Planning principles

1. **Do not re-plan completed bootstrap work.** Keep the plan focused on remaining implementation goals.
2. **Evidence before claims.** Accuracy, compatibility, and release readiness require measurements or generated reports.
3. **Reference first, package second.** Compressed artifacts must be generated from validated source outputs.
4. **Catalog breadth must stay API-compatible.** New houses, ayanamsas, aliases, and bodies must not force public redesign.
5. **Keep pure Rust mandatory.** New readers, generators, benchmarks, and release tools must preserve the no-required-C/C++ policy.
6. **Ship in reviewable slices.** Each phase is decomposed into independently testable increments.

## Plan directory structure

```text
PLAN.md                     # top-level index and forward execution summary
plan/
  overview.md               # orientation and maintenance guidance
  stages/                   # remaining implementation phases only
  status/                   # current frontier and next-slice guidance
  tracks/                   # cross-cutting expectations by subsystem
  checklists/               # reusable phase/release gates
  appendices/               # traceability and workable-state references
```

## Remaining development phases

| Phase | Focus | Why it comes next | Workable-state promise | Detailed doc |
| --- | --- | --- | --- | --- |
| 1 | Production ephemeris accuracy | Real source-backed astronomy must replace simplified backend behavior before release claims expand | Major bodies and baseline lunar points have documented, tested accuracy against authoritative references | [plan/stages/01-production-ephemeris-accuracy.md](plan/stages/01-production-ephemeris-accuracy.md) |
| 2 | Reproducible compressed artifacts | Packaged data must be generated and validated from public inputs, not hand-maintained samples | A maintainer can regenerate, inspect, validate, and use a 1500-2500 artifact deterministically | [plan/stages/02-reproducible-compressed-artifacts.md](plan/stages/02-reproducible-compressed-artifacts.md) |
| 3 | Compatibility catalog completion | Swiss-Ephemeris-class interoperability requires formula, alias, and failure-mode evidence across houses and ayanamsas | Release profiles accurately describe shipped catalog coverage and known gaps | [plan/stages/03-compatibility-catalog-completion.md](plan/stages/03-compatibility-catalog-completion.md) |
| 4 | Release stabilization and hardening | Public release requires validation reports, API documentation, audit gates, and reproducible bundles | Maintainers can publish a release with archived artifacts, reports, checksums, and clear compatibility claims | [plan/stages/04-release-stabilization-and-hardening.md](plan/stages/04-release-stabilization-and-hardening.md) |

## Current planning posture

| Phase | Status | Summary |
| --- | --- | --- |
| 1. Production ephemeris accuracy | Active | Start by replacing preliminary/snapshot calculations with source-backed algorithms and validation evidence |
| 2. Reproducible compressed artifacts | Queued | Depends on trusted source outputs and measured error targets |
| 3. Compatibility catalog completion | Queued | Can proceed in parallel where formulas and references are independent of backend accuracy |
| 4. Release stabilization and hardening | Queued | Finalizes documentation, reports, and release bundles after accuracy and artifacts mature |

For the live execution frontier, see:

- [plan/status/01-current-execution-frontier.md](plan/status/01-current-execution-frontier.md)
- [plan/status/02-next-slice-candidates.md](plan/status/02-next-slice-candidates.md)

## Reading paths

### If you are planning the next implementation slice

1. [SPEC.md](SPEC.md) and the relevant `spec/*.md` files
2. [plan/status/01-current-execution-frontier.md](plan/status/01-current-execution-frontier.md)
3. the active phase document under [plan/stages/](plan/stages/)
4. the relevant track document under [plan/tracks/](plan/tracks/)
5. [plan/checklists/01-phase-gates.md](plan/checklists/01-phase-gates.md)

### If you are making release-facing changes

1. [plan/stages/04-release-stabilization-and-hardening.md](plan/stages/04-release-stabilization-and-hardening.md)
2. [plan/tracks/04-validation-and-release.md](plan/tracks/04-validation-and-release.md)
3. [plan/checklists/02-release-artifacts.md](plan/checklists/02-release-artifacts.md)

## Detailed plan index

### Orientation

- [plan/overview.md](plan/overview.md)

### Remaining phases

- [plan/stages/01-production-ephemeris-accuracy.md](plan/stages/01-production-ephemeris-accuracy.md)
- [plan/stages/02-reproducible-compressed-artifacts.md](plan/stages/02-reproducible-compressed-artifacts.md)
- [plan/stages/03-compatibility-catalog-completion.md](plan/stages/03-compatibility-catalog-completion.md)
- [plan/stages/04-release-stabilization-and-hardening.md](plan/stages/04-release-stabilization-and-hardening.md)

### Status and next-slice guidance

- [plan/status/01-current-execution-frontier.md](plan/status/01-current-execution-frontier.md)
- [plan/status/02-next-slice-candidates.md](plan/status/02-next-slice-candidates.md)

### Cross-cutting tracks

- [plan/tracks/01-workspace-and-tooling.md](plan/tracks/01-workspace-and-tooling.md)
- [plan/tracks/02-domain-and-public-api.md](plan/tracks/02-domain-and-public-api.md)
- [plan/tracks/03-backends-and-distribution.md](plan/tracks/03-backends-and-distribution.md)
- [plan/tracks/04-validation-and-release.md](plan/tracks/04-validation-and-release.md)

### Checklists

- [plan/checklists/01-phase-gates.md](plan/checklists/01-phase-gates.md)
- [plan/checklists/02-release-artifacts.md](plan/checklists/02-release-artifacts.md)

### Appendices

- [plan/appendices/01-phase-to-spec-map.md](plan/appendices/01-phase-to-spec-map.md)
- [plan/appendices/02-phase-workable-state-matrix.md](plan/appendices/02-phase-workable-state-matrix.md)

## Plan maintenance rules

When the plan changes:

- update `PLAN.md` when phase ordering or the planning model changes,
- update `plan/stages/` when remaining implementation goals change,
- update `plan/status/` when the current frontier or best next slices change,
- update `plan/tracks/` when cross-cutting standards change,
- update `plan/checklists/` when phase or release gates change,
- update `plan/appendices/` when traceability or workable-state references change,
- do not reintroduce completed bootstrap/MVP task lists unless they become active remediation work.

Status: Updated 2026-04-24 after review against `SPEC.md`, `spec/*.md`, and the current implementation state.
