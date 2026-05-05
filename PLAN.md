# Pleiades Development Plan

This plan is the forward execution map for `pleiades` after the workspace, core type model, backend trait, chart façade, baseline catalogs, validation/reporting tools, release-bundle rehearsal, source-backed VSOP87 major-planet path, compact lunar baseline, JPL snapshot fixture, and prototype packaged-data backend have already landed.

Completed bootstrap, MVP, and scaffolding work is intentionally not listed here. Use git history and validation reports for implementation archaeology. This document tracks only the remaining work needed to satisfy `SPEC.md` and `spec/*.md` as a production-quality, Swiss-Ephemeris-class, pure-Rust ephemeris workspace.

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

The repository currently provides all required first-party crates, pure-Rust development tooling, shared domain types, backend metadata and validation helpers, composite/routing helpers, a chart façade, baseline-plus-expanded house and ayanamsa catalogs, CLI/validation commands, release-profile summaries, workspace audits, release-bundle generation/verification, and broad tests around those surfaces.

Important landed implementation state:

- `pleiades-vsop87` uses generated binary tables from public VSOP87B sources for the Sun through Neptune, with Pluto still on an approximate mean-elements fallback that is excluded from release-grade comparison evidence.
- `pleiades-elp` provides a documented compact Meeus-style lunar baseline for the Moon, mean/true node, and mean apogee/perigee; it is not yet a full ELP coefficient implementation.
- `pleiades-jpl` provides a checked-in JPL Horizons fixture/snapshot backend, exact fixture epochs, interpolation transparency evidence, equatorial reconstruction, expanded selected-asteroid rows (including the added 2001-01-07 boundary slice, the 2378498.5 early asteroid boundary slice, the 2003-12-27 asteroid slice for Ceres, Pallas, Juno, and Vesta, and the Apophis selected-asteroid rows now added at J2000, 2001-01-05, 2003-12-27, 2132-08-31, and 2500-01-01), a selected-asteroid boundary-day summary spanning JD 2451914.5..JD 2451915.5, a new reference-snapshot boundary-epoch coverage summary that now includes a fully populated 2451915.5 boundary day inside the 2451913.5..2451917.5 window, expanded reference-corpus coverage across 272 rows, 16 bodies, and 20 epochs with 87 asteroid rows, a full 2451917.5 major-body boundary sample, a fully populated 2451915.5 boundary day, a 2451918.5 Mars/Jupiter boundary sample, and the 2451910.5 high-curvature boundary sample, and expanded interpolation-quality reference coverage across 122 samples, 10 bodies, and 15 epochs around JD 2451910.5; its reference corpus now also exposes the 2453000.5 and 2451914.5 boundary-day extensions for the major-body high-curvature slice, that high-curvature slice now also extends through JD 2451916.5 for an additional major-body boundary sample, and the reference snapshot summary now also surfaces the 2453000.5 late major-body boundary evidence inline, with direct CLI paths in both front ends; the selected-asteroid coverage line now explicitly names the 2451914.5, 2451915.5, and 2451918.5 boundary coverage inside the 2451910.5..2451918.5 window, the selected-asteroid source-evidence and batch-parity slices now have direct CLI paths in both CLIs, and the comparison/reference snapshot manifest summaries now also have direct CLI paths for provenance inspection; the Apophis selected-asteroid outer-boundary row at JD 2451918.5 now keeps that boundary window fully populated, but it is not yet a broad production JPL reader/corpus.
- `pleiades-data` ships a small deterministic prototype compressed artifact with codec validation, regeneration helpers, checksums, summaries, benchmark evidence, a structured production-profile / target-threshold skeleton, and deterministic generator-parameter/manifest scaffolding; the target-threshold summary is now explicitly tied to the release-profile identifier and now also surfaces body-class scope envelopes, and the packaged-artifact storage/reconstruction summary now also has a direct CLI path in both CLIs; the scope envelopes now also list the bundled bodies contributing to each class, making the phase-2 body-class breakdown more explicit in release-facing reports, and the artifact summary / validate-artifact surfaces now also include the packaged-artifact output-support posture alongside the profile and storage/reconstruction posture. The packaged-artifact regeneration summary now also rejects source and checksum drift directly, tightening the regeneration provenance posture, and the production-profile drift matrix now also pins stored-channel drift through the artifact-profile field. The checked-in fixture is regenerated from the updated reference snapshot, which now includes the full Sun-through-Pluto 1800-01-03 major-body boundary sample, but it is not yet a 1500-2500 CE production artifact with acceptable measured fit error.
- Public request policy is explicit: first-party backends currently accept mean geometric, tropical, geocentric TT/TDB requests; apparent-place, topocentric body positions, native sidereal backend output, and built-in Delta T modeling remain future work or explicit unsupported modes. The shared backend reports now also surface the deferred Delta T posture as its own typed policy summary so the no-built-in-Delta-T decision remains explicit without being conflated with request-scale wording.
- Compatibility profiles are generated and verified against current catalogs, which are broad but still need final formula/reference audits before full interoperability claims; built-in house and ayanamsa descriptor-local normalization now also fails closed during profile validation before coverage checks run. The compatibility-profile verification summary now also classifies the custom-definition ayanamsa label bucket explicitly, and it now also surfaces house-system and ayanamsa alias counts alongside the uniqueness checks, keeping the profile-only custom-definition set and alias posture visible beside the broader custom-definition label list.

## Remaining specification gaps

The remaining gaps are implementation and evidence gaps, not workspace-structure gaps:

1. Close ephemeris accuracy gaps, especially lunar-theory scope, broader JPL/reference data, and production error envelopes.
2. Replace the prototype compressed artifact with a reproducible, validated 1500-2500 CE data product generated from trusted public inputs.
3. Finish compatibility-catalog evidence: formulas, aliases, latitude/numerical constraints, sidereal metadata, and truthful release-profile claims.
4. Decide and implement or explicitly defer advanced request semantics: Delta T policy, UTC convenience, apparent-place corrections, topocentric body positions, and optional native sidereal/backend behavior.
5. Turn release rehearsal outputs into release gates backed by current artifacts, reports, checksums, rustdoc, and user-facing documentation.

## Planning principles

1. **Plan only remaining work.** Do not reintroduce completed bootstrap or scaffolding tasks.
2. **Evidence before claims.** Accuracy, compatibility, and release readiness require tests, validation reports, and documented tolerances.
3. **Reference first, package second.** Production compressed artifacts must be generated from validated source outputs.
4. **Catalog breadth must stay API-compatible.** New houses, ayanamsas, aliases, and bodies must not require public redesign.
5. **Unsupported modes must fail closed.** Apparent, topocentric, sidereal-backend, out-of-range, and missing-data requests must remain structured errors until actually implemented.
6. **Keep pure Rust mandatory.** Readers, generators, benchmarks, and release tools must preserve the no-required-C/C++ policy.

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
| 1 | Accuracy closure and request semantics | Production artifacts and release claims depend on trustworthy source outputs and explicit time/observer/apparentness behavior | Major-body, lunar-point, selected-asteroid, frame, time-scale, and unsupported-mode behavior has documented validation evidence and no known tolerance outliers in claimed scopes | [plan/stages/01-accuracy-closure-and-request-semantics.md](plan/stages/01-accuracy-closure-and-request-semantics.md) |
| 2 | Production compressed artifacts | The current packaged artifact is a prototype; the spec requires reproducible 1500-2500 CE distributable data with measured fit error | Maintainers can regenerate, inspect, validate, benchmark, and ship a deterministic production artifact from public inputs | [plan/stages/02-production-compressed-artifacts.md](plan/stages/02-production-compressed-artifacts.md) |
| 3 | Compatibility evidence and catalog completion | Catalog breadth exists, but interoperability claims require formula/reference evidence, aliases, and failure-mode audits | Release profiles accurately describe implemented house/ayanamsa coverage, constraints, aliases, custom definitions, and known gaps | [plan/stages/03-compatibility-evidence-and-catalog-completion.md](plan/stages/03-compatibility-evidence-and-catalog-completion.md) |
| 4 | Release hardening and publication | Public release requires current reports, checksums, docs, CI gates, and reproducible bundle verification over real artifacts | Maintainers can publish an audited release bundle with archived reports, manifests, docs, and compatibility claims | [plan/stages/04-release-hardening-and-publication.md](plan/stages/04-release-hardening-and-publication.md) |

## Current planning posture

| Phase | Status | Summary |
| --- | --- | --- |
| 1. Accuracy closure and request semantics | Active | Prioritize broader reference evidence and explicit advanced-request decisions; Pluto release-grade cleanup is complete |
| 2. Production compressed artifacts | Queued, with prototype groundwork landed | Begins after Phase 1 produces trusted generation inputs and tolerances |
| 3. Compatibility evidence and catalog completion | Parallelizable | Continue formula, alias, sidereal metadata, and profile-truthfulness audits without blocking backend accuracy work |
| 4. Release hardening and publication | Queued, with rehearsal tooling landed | Finalizes release gates after accuracy, artifacts, and compatibility evidence are current |

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

1. [plan/stages/04-release-hardening-and-publication.md](plan/stages/04-release-hardening-and-publication.md)
2. [plan/tracks/04-validation-and-release.md](plan/tracks/04-validation-and-release.md)
3. [plan/checklists/02-release-artifacts.md](plan/checklists/02-release-artifacts.md)

## Detailed plan index

### Orientation

- [plan/overview.md](plan/overview.md)

### Remaining phases

- [plan/stages/01-accuracy-closure-and-request-semantics.md](plan/stages/01-accuracy-closure-and-request-semantics.md)
- [plan/stages/02-production-compressed-artifacts.md](plan/stages/02-production-compressed-artifacts.md)
- [plan/stages/03-compatibility-evidence-and-catalog-completion.md](plan/stages/03-compatibility-evidence-and-catalog-completion.md)
- [plan/stages/04-release-hardening-and-publication.md](plan/stages/04-release-hardening-and-publication.md)

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
- remove or rewrite completed tasks instead of accumulating progress-note history.

Status: Updated 2026-05-05 after release-bundle snapshot-summary staging and verification coverage, plus the reference-corpus expansion review against `SPEC.md`, `spec/*.md`, validation summaries, the current implementation state, and the refreshed maintainer-facing CLI/release documentation for benchmark, comparison-audit, native-dependency-audit, and compatibility-caveats smoke paths, with benchmark-corpus guard-epoch validation now pinned in the validation tests.
