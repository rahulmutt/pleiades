# Pleiades Development Plan

This plan is the forward execution map for `pleiades` after the workspace bootstrap, required crate family, typed public model, backend trait, chart façade, baseline catalogs, CLI/reporting surfaces, validation scaffolding, source-backed VSOP87B tables for Sun-through-Neptune, compact lunar baseline, JPL snapshot fixture, and prototype packaged-data backend have already landed.

The plan intentionally tracks only remaining specification gaps. Completed scaffolding and release-rehearsal work should stay in git history, tests, and reports rather than being carried forward as active tasks.

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

- all mandatory first-party crates with the `pleiades-*` prefix;
- pure-Rust workspace tooling and native-dependency audit/report surfaces;
- shared types for bodies, custom identifiers, time scales, coordinates, observers, house systems, ayanamsas, errors, and compatibility-profile metadata;
- backend traits, metadata validation, batch APIs, routing/composite helpers, and structured unsupported-mode errors;
- a high-level chart façade with sidereal conversion, house calculation, sign/house placement, aspects, summaries, and request-shape diagnostics;
- broad built-in house and ayanamsa catalogs, aliases, descriptor validation, compatibility-profile generation, release-facing summaries, shared compatibility-summary helpers for house formula families, latitude-sensitive house systems, custom-definition ayanamsa labels, and catalog inventory, and representative ayanamsa provenance audit excerpts;
- `pleiades-vsop87` with generated public VSOP87B tables for Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, and Neptune; Pluto remains an explicitly approximate fallback excluded from release-grade major-body claims;
- `pleiades-elp` with a compact Meeus-style lunar baseline for Moon, mean/true node, and mean apogee/perigee, with true apogee/perigee unsupported;
- `pleiades-jpl` with checked-in JPL Horizons snapshot/hold-out fixtures, new 1500-01-01 early-boundary coverage for Sun, Moon, Mercury, and Venus, dedicated 1500-01-01, 1600-01-11, 1750-01-01, 1800-01-03, 1900-01-01, and 2200-01-01 selected-body boundary report surfaces, and a top-level reference snapshot summary plus release notes summary that now also surface the 1500-01-01, 1600-01-11, 1900-01-01, and 2200-01-01 selected-body boundary slices alongside the earlier boundary summaries, a lunar boundary evidence summary, a 2500-01-01 selected-body boundary slice for Mars, Mercury, Moon, Sun, and Venus, an added 1750-01-01 interior boundary slice for Sun through Neptune with a first-class 1750 major-body interior report surface, an added 2360234.5 interior comparison slice now exposed through a dedicated report surface, a new 2451545.0 major-body boundary report surface for the J2000 reference slice with direct CLI/report aliases, a generic major-body boundary summary now also surfaced through the top-level reference snapshot summary, an exact-J2000 evidence summary now promoted into the top-level reference snapshot summary, an added 2451915.25/2451915.75 high-curvature hold-out window for Sun, Moon, Mercury, and Venus, a 2451912.5 major-body boundary report surface plus 2451913.5/2451914.5/2451915.5/2451917.5/2451918.5/2451919.5 major-body boundary report surfaces and a 2451916.0 interior reference surface with a direct CLI/report alias and a 2451920.5 interior reference slice, an added 2451916.5 dense boundary summary now promoted into the top-level reference snapshot summary alongside the generic dense boundary day, sparse boundary summary, and pre-bridge boundary summary, a 2453000.5 major-body boundary summary, a 2600000.0 major-body boundary summary for the Mars outer-boundary anchor with CLI/report/help aliases, plus a Mars outer-boundary summary now surfaced in the top-level reference snapshot summary, with the validation CLI now explicitly regression-testing the 2451920.5 interior slice in the combined reference snapshot summary, an epoch-specific 2451914 major-body pre-bridge alias, an epoch-specific 2451915 major-body bridge alias, and a newly surfaced 2451917 major-body bridge summary, plus a public 2451918 major-body boundary compatibility alias over the Mars/Jupiter slice with explicit 2451918 wording in release-facing reports, the JPL backend API now also exposes epoch-specific aliases for the 2451914 pre-bridge, 2451914 bridge, and 2451915 bridge report surfaces, and the CLI parity layer now mirrors the 2451912, 2451913, 2451914, 2451918 boundary aliases plus the 2451914 pre-bridge and bridge-day aliases, the 2451915 bridge alias, and the 2451916 dense-boundary alias, while the validation-report and release-summary layers now also surface the 2451918 boundary slice explicitly, selected asteroid rows, interpolation transparency evidence, provenance summaries, and validation helpers, and the combined JPL evidence report now also explicitly surfaces the 2451910 through 2451915 major-body boundary summaries, plus the 2451917 boundary and bridge summaries and the 2451918/2451919 boundary slices with the 2451920 interior slice, alongside selected-asteroid boundary, bridge, dense, terminal, and source evidence/window summaries; the release notes summary and validation report now also surface the terminal slice explicitly; the top-level reference snapshot summary now composes the explicit 2451914 helper aliases for the pre-bridge surface, now also surfaces the generic bridge-day summary, uses the direct bridge-day helper in that slot, and now also surfaces the distinct 2451914 major-body bridge-day alias, plus the 2451917 major-body boundary and bridge summaries, and now also surfaces the 2360233.5 and 2378499.0 alias views of the 1749 and 1800 boundary slices; it is not yet a broad production JPL reader/corpus;
- `pleiades-compression` and `pleiades-data` with codec validation, profile metadata, checksums, residual support, a deterministic prototype artifact, regeneration helpers, packaged lookup behavior, explicit lookup-epoch and speed-policy metadata in the production-profile draft, explicit storage/reconstruction summaries that distinguish derived ecliptic/equatorial coordinates from unsupported apparent, topocentric, sidereal, and motion outputs, and release-bundle emission of the packaged lookup-epoch policy, packaged-artifact profile coverage, generation policy, generation-manifest summary, output-support, speed-policy, and frame-treatment summaries alongside the production-profile and target-threshold summaries; the checked-in fixture now tracks the expanded reference-snapshot slice, the packaged-artifact-prefixed lookup-epoch policy aliases now mirror the packaged lookup policy in the CLI/validation front ends, but the artifact is not yet a production 1500-2500 CE data product;
- CLI and validation commands for compatibility profiles, backend matrices, request policies, comparison/corpus summaries, artifact inspection/regeneration, benchmarks, audits, release summaries, and release-bundle generation/verification, with the release bundle now also staging the comparison-body-class-tolerance summary alongside the comparison-envelope and comparison-corpus release-guard outputs plus release-grade body-claims, Pluto-fallback, and reference snapshot source summaries; explicit evidence-classification blocks cover release-tolerance, hold-out, fixture exactness, and provenance-only report surfaces; request-surface summaries now call out Delta T as its own report entrypoint alongside UTC-convenience policy, the release-grade body-claims posture now has a standalone summary command, and frame policy now ties equatorial precision to the shared mean-obliquity frame round-trip envelope.

## Remaining specification gaps

The open work is concentrated in five areas:

1. **Reference-grade ephemeris evidence** — broaden source/reference coverage, keep Pluto explicitly approximate unless a source-backed path is later validated, keep the compact lunar baseline as the first-release posture, and publish body-class tolerances.
2. **Request and time semantics** — either implement or explicitly defer built-in Delta T/UTC convenience, apparent-place corrections, topocentric body positions, and native sidereal backend output; keep frame precision explicit via the shared mean-obliquity frame round-trip envelope.
3. **Production compressed artifacts** — replace the prototype packaged artifact with a reproducible 1500-2500 CE artifact generated from validated public inputs and measured against published error thresholds.
4. **Compatibility catalog evidence** — complete house and ayanamsa formula/provenance audits, alias checks, latitude/numerical failure-mode coverage, custom-definition posture, and truthful release-profile claims.
5. **Release hardening** — turn existing rehearsal outputs into blocking release gates with current reports, checksums, docs, audits, and reproducible bundle verification.

## Planning principles

1. **Plan only remaining work.** Remove completed tasks instead of keeping progress-note history.
2. **Evidence before claims.** Accuracy, compatibility, and release readiness require tests, validation reports, tolerances, and documented provenance.
3. **Reference first, package second.** Production compressed artifacts must be fitted from trusted source outputs.
4. **Fail closed.** Unsupported apparent, topocentric, sidereal-backend, out-of-range, missing-data, and unsupported time-scale requests must remain structured errors until implemented and validated.
5. **Preserve pure Rust and layering.** New readers, generators, data products, and tooling must respect the crate boundaries in `spec/architecture.md`.

## Remaining development phases

| Phase | Focus | Workable-state promise | Detailed doc |
| --- | --- | --- | --- |
| 1 | Reference accuracy and request semantics | Release-claimed ephemeris behavior has source provenance, tolerances, explicit request policy, and no unstated approximate paths | [plan/stages/01-accuracy-closure-and-request-semantics.md](plan/stages/01-accuracy-closure-and-request-semantics.md) |
| 2 | Production compressed artifacts | Maintainers can regenerate, validate, benchmark, and ship a deterministic 1500-2500 CE packaged-data artifact | [plan/stages/02-production-compressed-artifacts.md](plan/stages/02-production-compressed-artifacts.md) |
| 3 | Compatibility evidence and catalog truthfulness | Release profiles accurately describe implemented house/ayanamsa formulas, aliases, constraints, custom definitions, and known gaps | [plan/stages/03-compatibility-evidence-and-catalog-completion.md](plan/stages/03-compatibility-evidence-and-catalog-completion.md) |
| 4 | Release hardening and publication | A clean checkout can produce and verify a release bundle with current reports, checksums, docs, audits, and compatibility claims | [plan/stages/04-release-hardening-and-publication.md](plan/stages/04-release-hardening-and-publication.md) |

## Current planning posture

| Phase | Status | Summary |
| --- | --- | --- |
| 1. Reference accuracy and request semantics | Active | Prioritize production-suitable reference coverage, explicit Pluto posture, compact lunar posture, and final request/time semantics decisions. |
| 2. Production compressed artifacts | Queued, prototype groundwork landed | Begins once Phase 1 provides trusted generation inputs and tolerance thresholds. |
| 3. Compatibility evidence and catalog truthfulness | Parallelizable | Formula/provenance audits and release-profile truthfulness stay current as new release-advertised catalog entries are added. |
| 4. Release hardening and publication | Queued, rehearsal tooling landed | Finalizes gates and bundles after accuracy, artifact, and catalog evidence are current. |

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
- [plan/stages/01-accuracy-closure-and-request-semantics.md](plan/stages/01-accuracy-closure-and-request-semantics.md)
- [plan/stages/02-production-compressed-artifacts.md](plan/stages/02-production-compressed-artifacts.md)
- [plan/stages/03-compatibility-evidence-and-catalog-completion.md](plan/stages/03-compatibility-evidence-and-catalog-completion.md)
- [plan/stages/04-release-hardening-and-publication.md](plan/stages/04-release-hardening-and-publication.md)
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

When implementation closes a gap, remove it from the active plan/status docs and update the phase map if needed. When a spec requirement changes, map it into an active or queued phase in the same change.

Status: Updated 2026-05-05 after reviewing `SPEC.md`, `spec/*.md`, the current crate implementations, and the existing plan/status documents. This revision removes accumulated progress-note history and restates only the remaining implementation goals.
