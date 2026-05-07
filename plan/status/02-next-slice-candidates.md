# Status 2 — Next Slice Candidates

This file lists focused implementation slices for the current phase ladder. Completed report aliases, fixture row promotion, and release-rehearsal cleanup are intentionally omitted.

## Phase 1 — Production compressed data

### 1. Artifact fitting strategy

- The Moon slice has already moved from residual-correction segments to quadratic base fits.
- The bundled-body quadratic-window slice has landed, and the latest follow-on slice now uses adjacent quadratic windows with linear tails where needed; current thresholds are calibrated to the latest draft artifact.
- Evaluate whether the remaining high-error bodies should move to denser windows, Chebyshev, or higher-order polynomial segments.
- Split body classes by cadence and segment length: inner planets, outer planets, Pluto, and selected asteroids may still need different strategies.
- Body/channel-specific fit reports now identify the worst segments and source intervals; use them to prioritize the next fit changes.
- Keep failures explicit until measured deltas are inside the production target profile.

### 2. Artifact generation manifest

- Regeneration provenance now records encoded artifact size alongside the checksum, keeping size accounting explicit in the manifest trail.
- Ensure generator parameters fully describe source inputs, segment strategy, quantization scales, residual policy, checksums, and output profile identifiers.
- Keep normalized intermediate summaries deterministic and reproducible.
- Make regenerated artifact bytes/checksums comparable from a clean checkout.
- Use the improved mixed-order linear-span fixture as the new baseline for any follow-on fit experiments.

### 3. Artifact benchmark coverage

- Benchmark single lookup, batch lookup, decode cost, encoded size, and full-chart packaged-data use.
- Track benchmark rows in release summaries without treating speed as a substitute for accuracy.

## Phase 2 — Production reference inputs

### 1. Source ingestion decision

- Decide whether to implement a broader JPL reader/parser, a generated public-data fixture corpus, or both.
- Document provenance, license/redistribution posture, frame, time scale, columns, source revision, and checksum expectations.

### 2. Body and epoch coverage

- Expand coverage only where it supports advertised release claims or artifact fitting.
- Preserve evidence classes: reference, hold-out, fixture exactness, and provenance-only.
- Keep selected asteroid support bounded to validated bodies and epochs.

### 3. Release-grade body posture

- Keep Pluto approximate/excluded unless a source-backed path passes thresholds.
- Keep the compact lunar baseline unless fuller ELP-style coefficient support lands with provenance and validation.
- Keep lunar point/apogee claims aligned with supported algorithms.

## Phase 3 — Advanced request support

- Decide first-release UTC/Delta-T convenience policy.
- Implement apparent-place or topocentric body support only with capability metadata, validation, and docs.
- Keep native sidereal backend output unsupported unless a backend explicitly implements it.
- Add precedence tests for invalid/unsupported request combinations when behavior changes.

## Phase 4 — Compatibility catalog evidence

- Add formula/reference evidence for any house system promoted to fully implemented status.
- Add provenance/reference evidence for any ayanamsa promoted beyond descriptor/custom-only status.
- Verify release profiles fail on overstated catalog claims.

## Phase 5 — Release gate hardening

- Make release-gate commands fail on stale generated files, artifact threshold violations, profile drift, or native-dependency regressions.
- Stage and verify all release-bundle artifacts from a clean checkout.
- Keep README/docs aligned with the published release compatibility profile and known gaps.
