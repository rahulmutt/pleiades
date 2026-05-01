# Status 2 — Next Slice Candidates

This file lists focused, reviewable implementation slices that map to the current phase ladder. It intentionally omits completed scaffold work and old progress-note history.

Recently completed: Pluto explicit downgrade and corpus cleanup, plus the production-generation boundary overlay corpus and report entry that now seed the full independent hold-out validation snapshot for validation and future artifact-generation work. Release-grade comparison and tolerance reports now use a Pluto-excluded corpus, while the full snapshot corpus remains available for provenance and archaeology. The interpolation-quality corpus now tracks the current 41-sample, 10-body, 6-epoch evidence set, and its report surface now includes an explicit provenance line derived from the checked-in reference snapshot. The selected-asteroid reference slice now spans J2000, 2001-01-01, 2132-08-31, and 2500-01-01 for Ceres, Pallas, Juno, Vesta, and asteroid:433-Eros, and the JPL snapshot report now also surfaces a broader 20-sample source-backed asteroid window set for provenance alongside the exact J2000 slice. The reference snapshot source summary now also includes the body/epoch coverage windows that define the checked-in snapshot, so the release-facing evidence surfaces both provenance and window coverage directly. The packaged-artifact fixture has been regenerated from the updated reference snapshot. The production-generation boundary overlay now also has a standalone request corpus, inventory summary, provenance summary, and report surface for reuse in validation and future artifact-generation work, the combined JPL evidence summary now surfaces the boundary-overlay provenance line, and the packaged artifact regeneration provenance now carries an explicit profile identifier. The packaged-data crate now also exposes deterministic generator-parameter and manifest outputs alongside the production-profile skeleton, and the validation artifact summary now surfaces the generation manifest directly next to the skeleton so the current regeneration posture is visible in the release-facing artifact report. The lunar source-window summary now also includes the reference-only apparent Moon comparison windows alongside the published 1992-04-12 Moon example and the checked-in 2451911.5/2451912.5 high-curvature samples. Request-semantic summaries, CLI help, and policy docs now also spell out the deferred apparent-place, topocentric, and native-sidereal posture explicitly. The shared request-policy summary validation now also rejects blank, whitespace-padded, and multiline drift so the compact report wording stays fail-closed. The direct request validator now also checks both mean and apparent value-mode capabilities, so future apparent-only or mean-only backends will fail closed instead of silently accepting unsupported coordinates. The shared frame-policy summary now also fails closed when it drifts from the current canonical frame-policy posture.

## 2. Production reference corpus expansion

**Phase:** 1 — Accuracy Closure and Request Semantics

**Goal:** provide enough trusted source/reference material for validation and artifact generation.

**Work items:**

- Expand JPL/reference rows for all release-claimed major bodies and selected asteroids. The production-generation boundary overlay now includes the full independent hold-out snapshot, the merged reference-plus-boundary corpus now totals 81 rows across 15 bodies and 9 epochs, and the interpolation-quality corpus now includes the current 41-sample, 10-body evidence set; the overlay also now has a standalone request corpus, summary, and provenance line for generation tooling, and the combined JPL evidence summary now includes the overlay provenance as well. The selected-asteroid source window summary now also surfaces the broader 20-sample asteroid provenance slice, and the reference-snapshot reports now expose the broader Moon source-window summary with the apparent comparison windows; remaining work is to widen source windows beyond lunar-only evidence when production artifact generation needs it.
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

- Define production artifact profile identifiers, body sets, stored/derived/unsupported outputs, and target thresholds. The packaged-data crate now exposes a dedicated production-profile skeleton summary, a structured target-threshold scaffold that captures the current measured fit envelope, and deterministic generator-parameter/manifest outputs that capture the current prototype posture; the remaining work is to finalize the release thresholds and align the manifest with production fit validation.
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
