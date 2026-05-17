# Status 2 — Next Slice Candidates

This file lists focused implementation slices for the current phase ladder. Completed report surfaces and historical cleanup tasks are intentionally omitted.

## Phase 1 — Artifact accuracy and packaged-data production

### Completed diagnostic slice

- The channel-major fit-outlier report now keeps segment-span and family sample-count context, tie-breaks prefer the shorter failing family when two candidates have the same delta, and distance-channel entries are rendered first in the body/channel summaries; the report lattice is denser than the calibration lattice.
- The validation report now renders the body-class span-cap summary without duplicating the summary prefix.
- The packaged-artifact generator now tries six-point Chebyshev-Lobatto fits before falling back to the previous cubic/quadratic ladder.
- The generator now applies measured-fit subdivision on short spans, chooses the better candidate-versus-fallback reconstruction when the span is tiny, compares span-limited candidates against their fallback reconstruction before accepting them, regenerates the checked-in fixture, can attempt residual-correction channels across the bundled body set when they improve the measured fit, and now explores residual-channel combinations rather than only greedy single-channel additions while still caching artifact-derived fit samples so CLI/report rendering stays tractable; the luminary and custom-body residual lattices now use denser body-specific cadences instead of the prior flat sample grid, and the validation lattice now gets body-specific density for luminaries, selected asteroids, Pluto, and custom bodies.
- The validation, benchmark, and packaged-artifact smoke-report paths now cache expensive report objects and use reduced timing subsets so bundle verification stays tractable under the test harness.
- The packaged-artifact fit envelope now uses the same body-specific dense validation cadence as the outlier summaries for luminaries, selected asteroids, Pluto, and custom bodies.
- The packaged-artifact regeneration helper now caches the deterministic rebuilt artifact in-process, so repeated CLI regression invocations only pay the full rebuild cost once per process.
- The normalized-intermediate summary now carries a deterministic checksum so the regeneration provenance can fail closed if the rendered payload drifts.
- The packaged artifact now exposes a bundled-body cadence summary surface so future body-specific window tuning stays explicit without perturbing the current validation lattice.
- The packaged-artifact target-threshold posture is now represented by a typed release-state enum, the current posture is recorded as production-ready, the target-threshold validation now fails closed when any advertised scope exceeds the calibrated fit thresholds, and the parameter-validation regression now checks that a Draft posture is rejected against that baseline.
- The regenerated packaged-artifact fixture is now resynced to the current code path, and the release bundle directory/manifest listings now include the release-house-validation summary with checksum verification.

### 1. Improve fitting/reconstruction strategy

- Evaluate denser source windows, body-specific cadence, Chebyshev segments, higher-order fits, residual tables, or channel-specific reconstruction.
- The current generator now compares span-limited polynomial candidates against their fallback reconstruction before accepting them and can already attach residual-correction channels across the bundled body set when that improves the measured fit; further improvement can still come from denser source windows, residual tables, or channel-specific reconstruction.
- Keep artifact size and decode benchmarks current, but do not trade correctness away for size.

### 2. Keep the finalized threshold policy aligned with Phase 2 corpus evidence

- Completed: the packaged-artifact target-threshold summary now carries explicit phase-2 reference, comparison, and independent hold-out corpus alignment evidence and fails closed on phase2 alignment drift.
- Completed: direct and batch request-policy validation now pins invalid-observer precedence ahead of unsupported-body coverage, so mixed request-shape failures keep surfacing the malformed observer before body-coverage rejection.

## Phase 2 — Reference/source corpus productionization

### 1. Source ingestion decision

- Completed: the production source posture is now documented as a hybrid fixture corpus, with checked-in reference and hold-out fixtures plus a separate generation-input path; the reference snapshot and independent hold-out source summaries now expose deterministic source checksums alongside the CSV column schema metadata, the checked-in snapshot manifests now also record a redistribution posture comment, the production-generation source summary now includes explicit per-fixture checksum revision markers plus CSV column schema metadata and the generation command, the boundary overlay source summary now carries an explicit hold-out checksum marker, the selected-asteroid source request corpus now exposes frame-specific and mixed-frame validation shapes, and the production-generation boundary source summary now stages and verifies in the release bundle.
- Completed: the packaged-artifact phase-2 corpus alignment posture now carries source/provenance lines for the reference, comparison, and hold-out corpora alongside the body-class coverage evidence, the standalone validated CLI summary surface stays in sync with that richer corpus posture, it now also carries the selected-asteroid source evidence and window summaries in the same phase-2 posture, and the regenerate-packaged-artifact regression now batches its sidecar writes so the slow fixture rewrite stays tractable under the test harness.
- Continue documenting provenance, license/redistribution posture, frame, time scale, columns, source revision, and checksum expectations for any broader source-corpus expansion.

### 2. Coverage expansion

- Completed: the JPL interpolation-quality report now also includes body-class error envelopes for the luminary and major-planet slices, keeping empirical envelope evidence explicit in the validation path.
- Expand body/epoch/channel coverage only where it supports artifact fitting or release claims.
- Preserve evidence classes: reference, hold-out, boundary overlay, fixture exactness, and provenance-only.
- Keep selected asteroid support bounded to validated bodies and epochs.

## Phase 3 — Body-model completion and claim boundaries

- Resolve Pluto as source-backed, artifact-backed, approximate, constrained, or excluded.
- Decide whether fuller ELP-style lunar coefficients are required for first production release.
- Keep lunar node/apogee/perigee claims aligned with implemented formulas and evidence.
- Completed: the JPL reference snapshot already contains source evidence for Ceres/Pallas/Juno/Vesta, and the release body claims summary now explicitly lists the selected asteroid validation bodies (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) while keeping them validation-only rather than release-grade claims; the backend-matrix and reference-asteroid selected-asteroid coverage line now routes through a validated summary helper.

## Phase 4 — Advanced request modes and policy

- Decide whether built-in UTC/Delta-T convenience belongs in the first production release.
- Implement apparent-place or topocentric body support only with capability metadata, validation, and docs.
- Keep native sidereal backend output unsupported unless a backend explicitly implements it.
- Add precedence tests for invalid/unsupported request combinations when behavior changes.
- Completed: the frame-policy summary surface now validates before rendering in the CLI and validation report paths.
- Completed: request-policy validation now preserves invalid-observer precedence ahead of unsupported-body coverage in both direct and batch metadata checks, and the request-policy/request-semantics report titles now share a guarded validation helper so the paired release-facing blocks fail closed if the title wording drifts.

## Phase 5 — Compatibility catalog evidence

- Add formula/reference evidence for any house system promoted to fully implemented status.
- Add provenance/reference evidence for any ayanamsa promoted beyond descriptor/custom-only status.
- Completed: representative golden/reference tests now pin the latitude-sensitive topocentric house snapshot and baseline ayanamsa epoch offsets.
- The compatibility profile summary now fails closed if it stops describing the baseline/release split explicitly, and the house-formula-family / latitude-sensitive / house-code-alias / custom-definition report surfaces now use validated wrappers; the individual house and ayanamsa descriptor summary surfaces now also validate before rendering; the ayanamsa catalog validation summary itself now has a validated wrapper in release-facing report surfaces; the catalog inventory summary now also validates the release-profile identifiers before rendering; the compatibility-caveats and release-notes summary surfaces now also route through validated latitude-sensitive house-system, house-code alias, and custom-definition ayanamsa wrappers; the body-class comparison-envelope formatter now validates before rendering; extend similar claim checks only to any newly identified release-facing summary surfaces that still bypass profile validation.

## Phase 6 — Release gate hardening

- Make release-gate commands fail on stale generated files, artifact threshold violations, profile drift, unsupported-mode claim drift, or native-dependency regressions.
- Stage and verify all release-bundle artifacts from a clean checkout.
- Completed: release bundles now stage and verify the release-house-validation summary alongside the other release catalog evidence files, and now also stage and verify the packaged-artifact phase-2 corpus alignment summary in the bundle manifest and directory checks; the bundle generator now validates the release-grade body-claims and Pluto-fallback summaries before serializing them, the comparison-corpus release-grade guard summary now validates before rendering in comparison/report surfaces, and the backend matrix summary now validates the compatibility profile before rendering.
- Completed: the backend matrix and validation-report summary surfaces now route house-code aliases through a validated profile-backed helper, keeping the release-facing summary surfaces aligned with the compatibility-profile validation gate.
- Keep README/docs aligned with the published release compatibility profile and known gaps.
