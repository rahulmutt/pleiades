# Status 2 — Next Slice Candidates

This file lists focused implementation slices for the current phase ladder. Completed report surfaces and historical cleanup tasks are intentionally omitted.

## Phase 1 — Artifact accuracy and packaged-data production

### Completed diagnostic slice

- The channel-major fit-outlier report now keeps segment-span and family sample-count context, and tie-breaks prefer the shorter failing family when two candidates have the same delta.
- The validation report now renders the body-class span-cap summary without duplicating the summary prefix.
- The packaged-artifact generator now tries six-point Chebyshev-Lobatto fits before falling back to the previous cubic/quadratic ladder.
- The generator now applies measured-fit subdivision on short spans, chooses the better candidate-versus-fallback reconstruction when the span is tiny, regenerates the checked-in fixture, and caches artifact-derived fit samples so CLI/report rendering stays tractable.
- The packaged-artifact target-threshold posture is now represented by a typed release-state enum, which keeps the draft-versus-production-ready hook explicit for the eventual production-threshold policy.

### 1. Improve fitting/reconstruction strategy

- Evaluate denser source windows, body-specific cadence, Chebyshev segments, higher-order fits, residual tables, or channel-specific reconstruction.
- Treat distance-channel outliers as a first-class blocker; do not hide them behind longitude-only thresholds.
- If higher-order interpolation remains draft-grade, add error-aware subdivision so segment splitting depends on measured fit error instead of quantization alone.
- Keep artifact size and decode benchmarks current, but do not trade correctness away for size.

### 2. Promote draft thresholds to production thresholds

- Define body-class/channel thresholds before claiming production readiness.
- Require both source-fit and independent hold-out checks for advertised scopes.
- Keep unsupported outputs explicit in the artifact profile.

## Phase 2 — Reference/source corpus productionization

### 1. Source ingestion decision

- Completed: the production source posture is now documented as a hybrid fixture corpus, with checked-in reference and hold-out fixtures plus a separate generation-input path.
- Document provenance, license/redistribution posture, frame, time scale, columns, source revision, and checksum expectations.

### 2. Coverage expansion

- Expand body/epoch/channel coverage only where it supports artifact fitting or release claims.
- Preserve evidence classes: reference, hold-out, boundary overlay, fixture exactness, and provenance-only.
- Keep selected asteroid support bounded to validated bodies and epochs.

## Phase 3 — Body-model completion and claim boundaries

- Resolve Pluto as source-backed, artifact-backed, approximate, constrained, or excluded.
- Decide whether fuller ELP-style lunar coefficients are required for first production release.
- Keep lunar node/apogee/perigee claims aligned with implemented formulas and evidence.
- Promote Ceres/Pallas/Juno/Vesta only when source coverage and backend support are ready.

## Phase 4 — Advanced request modes and policy

- Decide whether built-in UTC/Delta-T convenience belongs in the first production release.
- Implement apparent-place or topocentric body support only with capability metadata, validation, and docs.
- Keep native sidereal backend output unsupported unless a backend explicitly implements it.
- Add precedence tests for invalid/unsupported request combinations when behavior changes.

## Phase 5 — Compatibility catalog evidence

- Add formula/reference evidence for any house system promoted to fully implemented status.
- Add provenance/reference evidence for any ayanamsa promoted beyond descriptor/custom-only status.
- Verify release profiles fail on overstated catalog claims.

## Phase 6 — Release gate hardening

- Make release-gate commands fail on stale generated files, artifact threshold violations, profile drift, unsupported-mode claim drift, or native-dependency regressions.
- Stage and verify all release-bundle artifacts from a clean checkout.
- Keep README/docs aligned with the published release compatibility profile and known gaps.
