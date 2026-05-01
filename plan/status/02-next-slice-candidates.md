# Status 2 — Next Slice Candidates

This file lists focused, reviewable implementation slices that map to the current phase ladder. It intentionally omits completed scaffold work and old progress-note history.

Recently completed: Pluto explicit downgrade and corpus cleanup, plus the production-generation boundary overlay corpus and report entry that now seed the full independent hold-out validation snapshot for validation and future artifact-generation work. Release-grade comparison and tolerance reports now use a Pluto-excluded corpus, while the full snapshot corpus remains available for provenance and archaeology. The interpolation-quality corpus also picked up a 2451910.5 boundary sample with quadratic coverage. The selected-asteroid reference slice now spans J2000, 2001-01-01, 2132-08-31, and 2500-01-01 for Ceres, Pallas, Juno, Vesta, and asteroid:433-Eros, and the packaged-artifact fixture has been regenerated from the updated reference snapshot. The production-generation boundary overlay now also has a standalone request corpus, inventory summary, and provenance summary for reuse in validation and future artifact-generation work, the combined JPL evidence summary now surfaces the boundary-overlay provenance line, and the packaged artifact regeneration provenance now carries an explicit profile identifier. Request-semantic summaries, CLI help, and policy docs now also spell out the deferred apparent-place, topocentric, and native-sidereal posture explicitly.

## 2. Production reference corpus expansion

**Phase:** 1 — Accuracy Closure and Request Semantics

**Goal:** provide enough trusted source/reference material for validation and artifact generation.

**Work items:**

- Expand JPL/reference rows for all release-claimed major bodies and selected asteroids. The production-generation boundary overlay now includes the full independent hold-out snapshot, and the interpolation-quality corpus now includes a 2451910.5 boundary sample; the overlay also now has a standalone request corpus, summary, and provenance line for generation tooling, and the combined JPL evidence summary now includes the overlay provenance as well. Remaining work is to add any further trusted source windows needed for production artifact generation and lunar-fit closure.
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

- Define production artifact profile identifiers, body sets, stored/derived/unsupported outputs, and target thresholds. The packaged-data crate now exposes a dedicated production-profile skeleton summary plus a structured target-threshold scaffold that captures the current prototype posture; the remaining work is to finalize the release thresholds and generator manifest shape.
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
