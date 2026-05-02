# Status 1 — Current Execution Frontier

## Frontier

The active frontier is **Phase 1 — Accuracy Closure and Request Semantics**.

The repository is past bootstrap and release-rehearsal scaffolding. The next production blocker is closing the remaining accuracy and request-policy gaps that prevent truthful release-grade ephemeris and packaged-data claims.

## Evidence reviewed

Current summaries show:

- all mandatory crates and release/reporting commands exist;
- VSOP87B source-backed generated binary paths cover the Sun through Neptune;
- Pluto remains an approximate mean-elements fallback in the backend catalog, while release-grade comparison and tolerance reports now exclude Pluto from evidence;
- the compact lunar baseline has documented reference evidence for supported lunar channels but is not a full ELP coefficient implementation;
- the JPL backend is a checked-in fixture/snapshot backend with expanded selected asteroid rows (J2000 plus 2001-01-01, 2001-01-05, 2132-08-31, and 2500-01-01), a dedicated selected-asteroid body-window summary, a selected-asteroid boundary-day summary, and expanded interpolation-quality coverage across 80 samples, 10 bodies, and 10 epochs, not a broad production reader/corpus; the reference snapshot source summary now also surfaces the broader body/epoch coverage windows that underpin the fixture, and the reference/comparison/hold-out manifest wrappers now fail closed on expected column-layout drift, and the checked-in reference corpus now includes an added 2001-01-05 boundary slice that brings the snapshot to 125 rows across 15 bodies and 13 epochs; the comparison snapshot now carries its own typed body-window summary and a matching body-class coverage summary for the 90-row comparison slice, and the validation/release summary surfaces now stay aligned with the current 90-row / 11-epoch comparison corpus; the independent hold-out corpus now also exposes typed source-window and body-class coverage summaries for its validation slice, and the new reference/hold-out overlap summary makes the current 32 shared body-epoch pairs explicit so future corpus expansion avoids reusing hold-out rows; the release summary and validation report now surface that overlap line alongside the hold-out source-window evidence so the shared validation slice stays visible in the main release-facing reports;
- selected asteroid reporting now also surfaces a broader source-backed window set (25 samples across 5 bodies and 5 epochs) alongside the exact J2000 slice for provenance and validation, plus a mixed-frame batch-parity summary for the exact J2000 request slice; those source evidence/window summaries now validate and fail closed on drift, and the selected-asteroid evidence/window/batch-parity slices are now also surfaced in the backend-matrix-summary, release-notes-summary, and release-summary views so report consumers can inspect them without opening the detailed backend report;
- the JPL backend now exposes a production-generation boundary overlay corpus that appends the full independent hold-out validation snapshot to the checked-in reference snapshot for validation and artifact-generation work, and the evidence report now also surfaces the boundary-overlay provenance alongside that broader coverage slice; the overlay now also has a standalone request corpus, inventory summary, per-body window summary, body-class coverage summary, and provenance summary for generation tooling, and the validation report now surfaces the boundary request corpus directly; the production-generation boundary window summary now validates its derived slice and the release-facing report path fails closed on drift; the lunar and major-body high-curvature reference summaries now also validate their summary lines and report wrappers fail closed on drift; the interpolation-quality corpus now tracks the current 80-sample, 10-body, 10-epoch evidence set;
- the packaged artifact is deterministic and validated as a prototype, and the checked-in fixture has been regenerated from the updated reference snapshot, but current fit errors are not release-grade; the artifact summary now also surfaces the deterministic generation manifest alongside the production-profile skeleton and a compact body-class mix so release-facing inspection includes the manifest currently driving regeneration and the current bundled body composition; the compact validation and release summaries now also surface the current packaged-artifact target-threshold posture, body-class scope envelopes, and generation manifest so the prototype production posture is visible from the main release-facing report surfaces, and the target-threshold scaffold now carries the release-profile identifier through that same surface; the merged production-generation corpus now also exposes a release-facing source-window summary so the combined reference+boundary span is visible per body in reports before Phase 2 fit claims harden, and the merged and boundary production-generation body-class coverage summaries now surface the major-body versus selected-asteroid split directly in those same reports;
- interpolation-quality reporting now includes an explicit provenance line derived from the checked-in reference snapshot, alongside the existing interpolation-quality and hold-out summaries, and the interpolation posture itself is now surfaced as a typed release-facing summary so the runtime-validation/transparency-only decision stays explicit;
- reference-snapshot reporting now also surfaces the broader lunar source-window summary combining the published 1992-04-12 Moon example with the J2000 high-curvature 2451911.5/2451912.5 window used for lunar-fit closure evidence plus the reference-only apparent Moon comparison windows used to broaden lunar source provenance, and the snapshot source-window summary now breaks the checked-in reference coverage out by body for release-facing archaeology with typed window validation; the new major-body high-curvature reference slice now extends that coverage through the 2451914.5 boundary day across all comparison bodies so the broader corpus evidence is visible without decoding the fixture, and the new major-body high-curvature body-window summary now exposes the per-body spans directly in report surfaces;
- request policy is explicit: mean, tropical, geocentric TT/TDB requests are supported; shared request validation now fails closed on both mean-only and apparent-only value-mode mismatches; apparent-place corrections, topocentric body-position requests, native sidereal backend output, and built-in Delta T/UTC convenience conversion are not implemented today; the chart CLI now also accepts explicit UTC/UT1 TT-offset aliases plus explicit UTC/UT1 TDB-offset aliases for the common conversion step without changing backend policy, and the policy docs plus README now mention those aliases directly; the shared request-policy and frame-policy summaries now fail closed on blank, whitespace-padded, or multiline drift, and the JPL provenance summary wrappers now reject embedded line breaks; release-facing request-semantics wording is now centralized behind a shared formatter in `pleiades-validate`, and that formatter now validates the shared request-policy summary before rendering the observer, apparentness, and request-policy lines so drifted policy prose fails closed in the report layer; the observer and apparentness policy summaries themselves are now typed and validated before the report layer renders them, and the Delta T posture is now surfaced separately as its own typed policy summary so the deferred no-built-in-Delta-T decision stays explicit too; chart-snapshot diagnostics now reuse the shared backend time-scale, frame, and apparentness summaries so the render path stays aligned with the same contract, and the core frame-transform regression coverage now includes a near-polar mean-obliquity round-trip case so the precision expectations stay visible at the edge of the ecliptic/equatorial conversion; compatibility-profile summaries now also surface house formula families alongside the broader catalog breadth, and the compatibility-profile validation now fails closed if documented custom-definition ayanamsa labels drift away from metadata coverage; routing metadata now canonicalizes mixed-scale nominal ranges so valid composite providers keep validating cleanly; comparison and regression tolerance audits now use body-class-specific release thresholds for the active planetary corpus.

## Why this frontier comes first

Phase 2 production artifacts require trusted generation inputs and tolerances. Phase 4 release claims require the same evidence. Therefore the next work should close source-backed accuracy gaps before expanding packaged-data or compatibility claims that depend on those outputs.

## Immediate blockers

1. **Reference corpus breadth** — Expand source/reference data enough to support production validation and artifact generation, not only fixture exactness.
2. **Advanced request semantics** — Decide whether Delta T/UTC convenience, apparent corrections, and topocentric body positions are implemented for the first release or intentionally deferred with metadata and structured errors.

## Recommended next slice

Pluto downgrade and corpus cleanup is complete:

- keep Pluto explicitly labeled as approximate in backend metadata and fallback provenance;
- use the Pluto-excluded release-grade corpus for comparison/tolerance evidence;
- preserve the full snapshot corpus for provenance and validation archaeology;
- keep release tolerance audits fail-closed if Pluto is advertised beyond its evidence.

The next slice should focus on broader reference coverage and the remaining request-semantics decisions.

## Parallel safe work

The following can proceed without distracting from accuracy closure:

- house/ayanamsa formula and alias audits;
- documentation cleanup for already-explicit request policy;
- release-bundle smoke-test maintenance;
- artifact-profile identifier groundwork and production-profile skeleton plumbing that do not claim production fit accuracy yet, including the release-profile identifier now threaded through packaged-artifact target thresholds.
- reference snapshot source-window formatting parity now matches the other coverage summaries so report surfaces can rely on `to_string()` consistently, the reference source-window summary now derives its body order directly from the checked-in snapshot entries, the new reference body-class coverage summary now surfaces typed major-body and selected-asteroid window breakdowns in release-facing reports, the compatibility profile now also renders the latitude-sensitive house-system set through the shared formatter used by validation reports, and the new reference/hold-out overlap summary guards against silently collapsing the validation slice into the reference corpus.

## Constraints

- Preserve pure Rust and first-party crate layering.
- Do not couple domain crates to concrete backends.
- Do not loosen unsupported-mode errors to silently satisfy apparent/topocentric/native-sidereal requests.
- Do not publish broader compatibility or accuracy claims until validation evidence supports them.
