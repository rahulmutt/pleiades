# Status 2 — Next Slice Candidates

This file lists focused, reviewable implementation slices that map to the current phase ladder. It intentionally omits completed scaffold work and old progress-note history.

## 1. Pluto source-backed accuracy path

**Phase:** 1 — Accuracy Closure and Request Semantics

**Goal:** remove the current Pluto tolerance outlier from release-grade major-body claims.

**Work items:**

- Select a pure-Rust public source strategy for Pluto: broader JPL-derived data, another documented public ephemeris source, or a clearly bounded downgrade in release profiles.
- Add representative 1500-2500 validation rows and body-class tolerance thresholds.
- Update metadata, source documentation, backend matrix, validation report, and release profile wording.
- Add regression tests that prevent silent reintroduction of approximate release-grade Pluto claims.

## 2. Production reference corpus expansion

**Phase:** 1 — Accuracy Closure and Request Semantics

**Goal:** provide enough trusted source/reference material for validation and artifact generation.

**Work items:**

- Expand JPL/reference rows for all release-claimed major bodies and selected asteroids.
- Include boundary dates, high-curvature lunar windows, and artifact-generation sampling points.
- Preserve deterministic manifests, provenance summaries, checksums, and pure-Rust parsing/loading.
- Decide whether interpolation is a runtime feature, generation-only aid, or transparency-only evidence.

## 3. Advanced request semantics decision

**Phase:** 1 — Accuracy Closure and Request Semantics

**Goal:** make time/observer/apparentness behavior release-ready.

**Work items:**

- Decide first-release posture for built-in Delta T and UTC/UT1 convenience conversion.
- Decide first-release posture for apparent-place corrections.
- Decide first-release posture for topocentric body positions.
- Update metadata, errors, docs, CLI flags, and tests so unsupported modes continue to fail closed.

## 4. Production artifact profile and generator skeleton

**Phase:** 2 — Production Compressed Artifacts

**Goal:** prepare the artifact-generation path without overstating prototype accuracy.

**Work items:**

- Define production artifact profile identifiers, body sets, stored/derived/unsupported outputs, and target thresholds.
- Add generator parameter structs and deterministic manifest outputs.
- Keep the prototype artifact labeled as prototype until Phase 2 fit validation passes.
- Add tests for profile/header validation and claim drift.

## 5. Artifact fit improvement loop

**Phase:** 2 — Production Compressed Artifacts

**Goal:** reduce packaged-data error to release-grade thresholds.

**Work items:**

- Tune segment lengths, polynomial order, quantization, and residual corrections by body class.
- Add interior samples, segment-boundary checks, and high-curvature lunar windows.
- Compare decoded outputs against Phase 1 generation sources.
- Publish measured errors and benchmarks in artifact summaries.

## 6. House-system evidence audit

**Phase:** 3 — Compatibility Evidence and Catalog Completion

**Goal:** ensure every release-advertised house system has formula and failure-mode evidence.

**Work items:**

- Add or verify golden scenarios across hemispheres, latitudes, and polar/high-latitude constraints.
- Check cusp ordering, angle derivation, normalization, and house placement.
- Update compatibility caveats and profile verification for descriptor-only or constrained entries.

## 7. Ayanamsa evidence audit

**Phase:** 3 — Compatibility Evidence and Catalog Completion

**Goal:** ensure every release-advertised ayanamsa has provenance, sidereal metadata, aliases, and tests appropriate to its claim.

**Work items:**

- Add golden/reference offsets for baseline and representative release-specific entries.
- Classify custom-definition-only entries explicitly.
- Verify alias uniqueness and compatibility-profile labels.
- Keep sidereal conversion deterministic and domain-layer owned.

## 8. Release gate hardening

**Phase:** 4 — Release Hardening and Publication

**Goal:** turn release rehearsal commands into final publication gates.

**Work items:**

- Wire final validation thresholds, artifact checksums, compatibility verification, bundle verification, and pure-Rust audit into CI/release workflow.
- Regenerate release summaries from current code and production artifacts.
- Review README/docs/rustdoc for units, frames, time scales, failure modes, and known limitations.

## Selection guidance

Prioritize slices 1-3 for the active frontier. Slices 4-5 should wait for trusted source inputs before making production accuracy claims. Slices 6-8 are safe parallel work when they do not depend on unresolved backend accuracy decisions.
