# Pleiades Development Plan

This plan is the forward execution map for `pleiades` after the architectural bootstrap and release-rehearsal foundation have landed. It tracks only remaining specification gaps from `SPEC.md` and `spec/*.md`; completed scaffolding, report-alias work, and historical phase notes belong in git history, not in the active plan.

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

The repository currently provides:

- all mandatory `pleiades-*` workspace crates and pure-Rust tooling checks;
- shared domain types, backend traits, capability metadata, batch APIs, structured errors, and composite-routing helpers;
- a high-level chart façade with sidereal conversion, house/sign placement, aspects, diagnostics, and compatibility-profile access;
- broad house and ayanamsa catalogs with aliases, descriptor validation, compatibility summaries, and release-profile verification;
- a VSOP87B-backed planetary implementation for Sun through Neptune, with Pluto still treated as an approximate fallback rather than a release-grade source-backed body;
- a compact Meeus-style lunar baseline for the Moon and supported lunar points, with fuller ELP coefficient support deferred;
- checked-in JPL Horizons snapshot/hold-out fixtures used for reference comparison, validation summaries, and artifact-generation rehearsal;
- `pleiades-compression` and `pleiades-data` codec/profile/checksum support plus a stage-5 draft packaged-data artifact for the 1500-2500 CE range; the packaged artifact has recently been retuned to recursively subdivided cubic windows with longitude unwrapping and tighter cadence-aware body-class span caps, materially improving the fit envelope and body-class cadence reporting while remaining draft-grade; the artifact reports now also surface channel-major fit-outlier and body-class cadence views to help prioritize the next fit slices;
- CLI and validation tooling for chart inspection, compatibility profiles, request policies, backend matrices, artifact reports, benchmarks, audits, and release-bundle rehearsal.

The implementation is therefore past the original bootstrap/foundation phases. Remaining work is productionization: source breadth, artifact accuracy, advanced request behavior, catalog evidence, and release gates. The artifact reporting surface now also includes separate single-lookup, batch-lookup, and decode benchmark sections so throughput evidence stays visible alongside the fit report.

## Remaining specification gaps

1. **Production compressed data** — the packaged artifact is still a draft fixture. It must become a reproducible 1500-2500 CE data product with measured errors inside published thresholds.
2. **Reference/data-source breadth** — the JPL path is a checked-in reference fixture, not a broad production reader or corpus suitable for all validation and artifact-generation needs.
3. **Advanced request semantics** — first-party backends still explicitly reject or defer built-in UTC/Delta-T convenience, apparent-place corrections, topocentric body positions, and native sidereal backend output.
4. **Release-grade body coverage** — Pluto, fuller lunar theory, lunar points, and selected asteroid claims need either source-backed validation or explicit exclusion/constrained status in release claims.
5. **Compatibility evidence** — the broad house and ayanamsa catalogs need continuing formula/provenance/reference audits whenever entries are claimed as implemented rather than descriptor-only or constrained.
6. **Release hardening** — release rehearsal tooling exists, but final gates must fail closed on stale profiles, native-dependency drift, artifact threshold failures, inaccurate backend claims, or unreproducible bundles.

## Planning principles

1. **Plan only remaining work.** Remove completed report surfaces, aliases, fixture rows, and scaffolding tasks from active plans.
2. **Evidence before claims.** Accuracy, compatibility, and release readiness require current tests, validation reports, tolerances, and provenance.
3. **Package from trusted inputs.** Production compressed artifacts must be generated from validated public source outputs, not from undocumented or ad hoc samples.
4. **Fail closed.** Unsupported apparent, topocentric, native-sidereal, out-of-range, missing-data, and unsupported time-scale requests must remain structured errors until implemented and validated.
5. **Preserve pure Rust and layering.** New readers, generators, datasets, and tooling must respect `spec/architecture.md`.

## Active development phases

| Phase | Focus | Workable-state promise | Detailed doc |
| --- | --- | --- | --- |
| 1 | Production compressed data | Maintainers can regenerate, validate, benchmark, and ship a deterministic 1500-2500 CE artifact within published thresholds | [plan/stages/01-production-compressed-data.md](plan/stages/01-production-compressed-data.md) |
| 2 | Production reference inputs | Maintainers have a pure-Rust source/reference path broad enough for backend validation, body claims, and artifact fitting | [plan/stages/02-production-reference-inputs.md](plan/stages/02-production-reference-inputs.md) |
| 3 | Advanced request support decisions | The public API either implements or explicitly defers UTC/Delta-T convenience, apparent corrections, topocentric positions, and native sidereal backend output | [plan/stages/03-advanced-request-support.md](plan/stages/03-advanced-request-support.md) |
| 4 | Compatibility catalog evidence | Release profiles accurately distinguish implemented, constrained, descriptor-only, custom, and unsupported house/ayanamsa entries | [plan/stages/04-compatibility-catalog-evidence.md](plan/stages/04-compatibility-catalog-evidence.md) |
| 5 | Release gate hardening | A clean checkout can produce verified release artifacts whose claims match current evidence | [plan/stages/05-release-gate-hardening.md](plan/stages/05-release-gate-hardening.md) |

## Current planning posture

| Phase | Status | Summary |
| --- | --- | --- |
| 1. Production compressed data | Active | Highest leverage next work: continue reducing the remaining high-error bodies after the cadence-aware cubic-window slice; the new channel-major fit-outlier and body-class cadence views should help split cadence-driven follow-on work, then tighten thresholds and benchmarks. |
| 2. Production reference inputs | Active dependency | Expand or replace checked-in fixtures with production-suitable public inputs where Phase 1 and body claims require stronger evidence. |
| 3. Advanced request support decisions | Deferred by policy | Keep current structured rejections truthful unless implementation and validation land. |
| 4. Compatibility catalog evidence | Parallelizable | Continue formula/provenance audits as catalog claims change; do not relabel descriptor-only entries as fully implemented without evidence. |
| 5. Release gate hardening | Queued/continuous | Maintain rehearsal tooling now; make it blocking once artifact, reference, and compatibility evidence are current. |

For live execution guidance, see:

- [plan/status/01-current-execution-frontier.md](plan/status/01-current-execution-frontier.md)
- [plan/status/02-next-slice-candidates.md](plan/status/02-next-slice-candidates.md)

## Plan directory structure

```text
PLAN.md
plan/
  overview.md
  stages/
  status/
  tracks/
  checklists/
  appendices/
```

## Detailed plan index

- [plan/overview.md](plan/overview.md)
- [plan/stages/01-production-compressed-data.md](plan/stages/01-production-compressed-data.md)
- [plan/stages/02-production-reference-inputs.md](plan/stages/02-production-reference-inputs.md)
- [plan/stages/03-advanced-request-support.md](plan/stages/03-advanced-request-support.md)
- [plan/stages/04-compatibility-catalog-evidence.md](plan/stages/04-compatibility-catalog-evidence.md)
- [plan/stages/05-release-gate-hardening.md](plan/stages/05-release-gate-hardening.md)
- [plan/status/01-current-execution-frontier.md](plan/status/01-current-execution-frontier.md)
- [plan/status/02-next-slice-candidates.md](plan/status/02-next-slice-candidates.md)
- [plan/tracks/01-workspace-and-tooling.md](plan/tracks/01-workspace-and-tooling.md)
- [plan/tracks/02-domain-and-public-api.md](plan/tracks/02-domain-and-public-api.md)
- [plan/tracks/03-backends-and-distribution.md](plan/tracks/03-backends-and-distribution.md)
- [plan/tracks/04-validation-and-release.md](plan/tracks/04-validation-and-release.md)
- [plan/checklists/01-phase-gates.md](plan/checklists/01-phase-gates.md)
- [plan/checklists/02-release-artifacts.md](plan/checklists/02-release-artifacts.md)
- [plan/appendices/01-phase-to-spec-map.md](plan/appendices/01-phase-to-spec-map.md)
- [plan/appendices/02-phase-workable-state-matrix.md](plan/appendices/02-phase-workable-state-matrix.md)

## Plan maintenance rules

When implementation closes a gap, remove it from the active plan/status docs and update the phase map. When a spec requirement changes, map it into an active or queued phase in the same change.

Status: Updated 2026-05-07 after reviewing `SPEC.md`, `spec/*.md`, current crate implementations, README status, and existing plan/status documents. This revision removes completed historical phase tasks and replaces them with production-focused implementation phases.
