# Pleiades Development Plan

This is the active forward plan for `pleiades`. It is intentionally limited to work that remains against `SPEC.md` and `spec/*.md`; completed bootstrap, scaffolding, diagnostic-report, summary-wrapper, bundle-rehearsal, and cache-hardening work has been removed from the active task list.

## Current implementation baseline

The workspace has completed the original foundation roadmap:

- all mandatory `pleiades-*` crates exist and follow the specified layering;
- shared types cover angles, instants/time scales, coordinate frames, observers, bodies, houses, ayanamsas, zodiac modes, compatibility profiles, and request policy;
- backend traits, metadata, batch helpers, and composite/routing helpers exist;
- `pleiades-core` exposes chart façade APIs, sign/aspect/house summaries, release compatibility metadata, explicit latitude-sensitive house-constraint reporting, and API-stability reporting;
- release-facing backend matrix summaries now also surface corpus-derived claim posture by folding in the comparison corpus release-grade guard, reference/hold-out overlap, independent hold-out, release-grade body claims, and Pluto fallback lines;
- release bundle verification now also re-checks the comparison-corpus summary against the current renderer, so the release corpus posture fails closed on semantic drift;
- release bundle generation and verification now also carry the body/date/channel claims summary and checksum, keeping the compact body-claim boundary visible in staged bundles;
- release bundle verification now also re-checks the backend matrix report and summary against the current renderers, so staged backend-matrix artifacts fail closed on semantic drift;
- JPL source-corpus evidence now includes a dedicated provenance-only posture line so provenance-only rows remain separate from tolerance, hold-out, and fixture-exactness evidence in release-facing reports, the reference and hold-out provenance summaries now also record explicit frame-treatment and time-scale posture for the checked-in corpus slices, and the JPL source corpus contract now has a standalone CLI/report entrypoint for direct release-audit checks;
- production-generation source-revision checksums now validate against the current fixture contents before they are surfaced or bundled, so the release-facing checksum line fails closed on drift;
- the production-generation corpus-shape summary now validates both ecliptic and equatorial boundary request corpora, so frame posture is explicit in the release-facing contract surface;
- the backend matrix and release summary now also surface the production-generation source line and source-revision line, keeping the documented hybrid/pure-Rust provenance block visible alongside the validated corpus-shape and manifest summaries;
- the release summary, validation report, and compact corpus views now also surface a consolidated source-corpus line combining the comparison corpus release-grade guard, the JPL source corpus contract, and the phase-2 corpus alignment, so provenance and coverage stay visible in the top-level release surfaces; the release summary and backend matrix now also avoid duplicating the JPL source corpus contract label when embedding that contract line; the consolidated source-corpus surface now also carries the shared schema label explicitly, the JPL evidence-classification and provenance-only split is now surfaced there too, the source corpus summary now also surfaces the generation-command fragment and production-generation coverage posture, and the backend matrix summary now mirrors that consolidated source-corpus posture line as well; the release summary and backend matrix now also surface the dedicated catalog posture line derived from the compatibility profile; the catalog posture line now also surfaces ayanamsa alias-bearing entry counts, keeping alias audits explicit alongside the metadata-bearing/descriptor-only split; the consolidated source-corpus surface now also binds the release-grade body-claims posture directly; the release summary and backend matrix now also surface a corpus-derived body/date/channel claims line that binds release-grade body claims to the current corpus-shape posture; release bundle generation now verifies the freshly rendered validation-report summary against the in-memory report used to write it, while standalone bundle verification still compares on-disk output against the current renderer;
- the staged release bundle now also carries `source-corpus-summary.txt`, and bundle verification checks its checksum and current posture alongside the existing comparison-corpus artifacts;
- the reference and hold-out source summaries now also carry explicit frame/time-scale posture fields, the reference snapshot equatorial parity summary is now bundled alongside the source-window artifact, and the release-facing body-class coverage summaries now reuse the shared celestial-body class taxonomy;
- release bundles now carry independent-holdout body-class coverage alongside the existing independent-holdout source-window evidence so hold-out coverage stays explicit in staged artifacts;
- the checked-in independent-hold-out fixture now includes selected-asteroid and major-body bridge rows at JD 2451915.5 in addition to the existing JD 2378498.5, JD 2451545, JD 2451917.5, JD 2453000.5, and JD 2500000 anchors, and the production-generation boundary summaries now reflect the expanded 77-row, 16-body, 14-epoch hold-out slice;
- release bundles now also carry the production-generation corpus-shape summary alongside the source-window and manifest summaries, and bundle verification fails closed if the staged directory or manifest omits it;
- the release summary now surfaces validated production-generation body-class coverage and corpus-shape lines alongside the release-grade body-claims line, keeping the top-level release overview tied to corpus evidence; the same corpus-derived body/date/channel claims posture now also has a standalone `body-date-channel-claims-summary` / `body-date-channel-claims` surface for direct inspection, and that surface is now field-validated so the release/backend matrix views derive from a shared structured record; the release-grade body-claims posture now also spells out the supported lunar points and the unsupported true apogee/perigee boundary explicitly;
- the release notes summary now also surfaces the house-validation corpus plus the ayanamsa catalog validation and sidereal-metadata coverage lines, so the compact release notes view carries explicit compatibility-evidence checks alongside the catalog inventory;
- house and ayanamsa crates contain broad catalog descriptors plus baseline calculations/conversions, the compatibility inventory now also surfaces ayanamsa alias-bearing entry counts to keep alias audits explicit, and the release-house-validation summary now also surfaces house-code aliases alongside the baseline house corpus;
- VSOP87-style planetary, compact lunar/lunar-point, JPL snapshot, and packaged-data backend crates exist;
- validation, CLI, release-bundle, audit, benchmark, and report-generation surfaces exist.

The repository is therefore no longer in a bootstrap phase. The remaining work is productionization: source coverage, production compressed data, body-claim accuracy, optional request modes, catalog evidence, and release gates.

## Important current limits

- `pleiades-data` ships a stage-5 draft artifact and decoder, but current comparison envelopes still exceed production tolerance for many bodies/channels. It is a reproducibility fixture, not a release-grade 1500-2500 CE ephemeris product.
- `pleiades-jpl` uses checked-in Horizons snapshots and hold-out fixtures. These are useful regression evidence, but not yet a broad production source corpus or general JPL reader.
- Pluto is still approximate/fallback-backed in first-party algorithmic paths.
- `pleiades-elp` is a compact Meeus-style lunar baseline, not a full ELP coefficient implementation.
- Selected asteroid evidence exists for bounded fixtures, but broad asteroid release claims are not yet supported.
- selected-asteroid source evidence and source-window summaries now also surface explicit evidence-class, frame, and time-scale posture so the selected-asteroid corpus slice stays aligned with the broader provenance contract.
- release bundles now also carry the selected-asteroid source request corpus equatorial summary, so the staged selected-asteroid slice surfaces both request-frame variants alongside the ecliptic request corpus.
- release bundles now also carry the production-generation boundary request corpus equatorial summary, so the staged boundary corpus exposes both request-frame variants and the equatorial checksum/semantic posture is verified alongside the ecliptic boundary request corpus.
- release bundles now also carry the catalog posture summary and checksum, so catalog-posture release surfaces stay aligned with the compatibility profile.
- the production-generation boundary request corpus now also exposes a dedicated equatorial summary surface, matching the selected-asteroid frame split and making the boundary corpus directly inspectable in both frames; boundary-request validation now also reports field-specific drift for request count, body count, bodies, epoch count, epochs, time scale, zodiac mode, and apparentness instead of a generic derived-summary mismatch.
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

Status: refreshed 2026-05-21 after reviewing `SPEC.md`, `spec/*.md`, current workspace crates, README status, and CLI/report posture; updated 2026-05-21 after stabilizing release-bundle summary verification.
