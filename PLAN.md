# Pleiades Development Plan

This is the active forward plan for `pleiades`. It is intentionally limited to work that remains against `SPEC.md` and `spec/*.md`; completed bootstrap, scaffolding, diagnostic-report, summary-wrapper, bundle-rehearsal, and cache-hardening work has been removed from the active task list.

## Current implementation baseline

The workspace has completed the original foundation roadmap:

- all mandatory `pleiades-*` crates exist and follow the specified layering;
- shared types cover angles, instants/time scales, coordinate frames, observers, bodies, houses, ayanamsas, zodiac modes, compatibility profiles, and request policy;
- backend traits, metadata, batch helpers, and composite/routing helpers exist;
- `pleiades-core` exposes chart façade APIs, sign/aspect/house summaries, release compatibility metadata, and API-stability reporting;
- house and ayanamsa crates contain broad catalog descriptors plus baseline calculations/conversions;
- VSOP87-style planetary, compact lunar/lunar-point, JPL snapshot, and packaged-data backend crates exist;
- validation, CLI, release-bundle, audit, benchmark, and report-generation surfaces exist.

The repository is therefore no longer in a bootstrap phase. The remaining work is productionization: source coverage, production compressed data, body-claim accuracy, optional request modes, catalog evidence, and release gates.

## Important current limits

- `pleiades-data` ships a stage-5 draft artifact and decoder, but current comparison envelopes still exceed production tolerance for many bodies/channels. It is a reproducibility fixture, not a release-grade 1500-2500 CE ephemeris product.
- `pleiades-jpl` uses checked-in Horizons snapshots and hold-out fixtures. These are useful regression evidence, but not yet a broad production source corpus or general JPL reader.
- Pluto is still approximate/fallback-backed in first-party algorithmic paths.
- `pleiades-elp` is a compact Meeus-style lunar baseline, not a full ELP coefficient implementation.
- Selected asteroid evidence exists for bounded fixtures, but broad asteroid release claims are not yet supported.
- First-party body-position requests are mean geometric and geocentric. Apparent-place corrections, topocentric body positions, native sidereal backend output, and built-in civil-time/Delta-T modeling remain unsupported unless explicitly supplied by the caller or a future backend.
- Broad house and ayanamsa catalogs are present, but entries still need formula/provenance audits before compatibility claims can be widened.

## Active phases

| Phase | Focus | Workable-state promise | Details |
| --- | --- | --- | --- |
| 1 | Production reference/source corpus | Public, reproducible inputs are broad enough to validate release body claims and generate artifacts. | [plan/stages/01-production-reference-corpus.md](plan/stages/01-production-reference-corpus.md) |
| 2 | Production compressed ephemeris | The 1500-2500 CE packaged backend is generated from Phase 1 inputs and meets published accuracy/size/speed thresholds. | [plan/stages/02-production-compressed-ephemeris.md](plan/stages/02-production-compressed-ephemeris.md) |
| 3 | Body/backend claim completion | Sun, Moon, planets, lunar points, Pluto, and selected asteroids are either source-backed or explicitly constrained in every public surface. | [plan/stages/03-body-and-backend-claims.md](plan/stages/03-body-and-backend-claims.md) |
| 4 | Advanced request modes | UTC/Delta-T convenience, apparent place, topocentric body positions, and native sidereal output are implemented with validation or consistently rejected. | [plan/stages/04-advanced-request-modes.md](plan/stages/04-advanced-request-modes.md) |
| 5 | Compatibility and release readiness | House/ayanamsa compatibility claims, release profiles, validation reports, audits, and bundles fail closed on drift or overclaiming. | [plan/stages/05-compatibility-and-release-readiness.md](plan/stages/05-compatibility-and-release-readiness.md) |

## Current priority

Start with **Phase 1**. A production compressed artifact cannot be promoted until its generation source and hold-out corpus are production-grade. Phase 2 work may continue in parallel when it improves the generator without broadening release claims.

## Plan maintenance rules

- Do not add completed summary surfaces, aliases, helper wrappers, cache changes, or historical implementation notes to this plan.
- When an implementation gap closes, remove it from phase/status files rather than moving it to a completed section.
- Keep `README.md`, release profiles, generated reports, and this plan aligned when public behavior or release claims change.

Status: refreshed 2026-05-20 after reviewing `SPEC.md`, `spec/*.md`, current workspace crates, README status, and CLI/report posture.
