# Status 2 — Next Slice Candidates

This file lists focused, reviewable implementation slices that map to the current phase ladder. It intentionally omits completed scaffold work and old progress-note history.

Recently completed: Pluto explicit downgrade and corpus cleanup, plus the production-generation boundary overlay corpus and report entry that now seed the full independent hold-out validation snapshot for validation and future artifact-generation work. Release-grade comparison and tolerance reports now use a Pluto-excluded corpus, while the full snapshot corpus remains available for provenance and archaeology. The interpolation-quality corpus now tracks the current 41-sample, 10-body, 6-epoch evidence set, its report surface now includes an explicit provenance line derived from the checked-in reference snapshot, and the interpolation posture itself is now surfaced as a typed release-facing summary so the runtime-validation/transparency-only decision stays explicit. The selected-asteroid reference slice now spans J2000, 2001-01-01, 2132-08-31, and 2500-01-01 for Ceres, Pallas, Juno, Vesta, and asteroid:433-Eros, and the JPL snapshot report now also surfaces a broader 20-sample source-backed asteroid window set for provenance alongside the exact J2000 slice. The selected-asteroid reporting now also includes a dedicated body-window summary so the per-asteroid epoch spans are visible without decoding the fixture, and both the selected-asteroid evidence and window summaries now validate and fail closed on drift. Those selected-asteroid evidence/window slices are now also surfaced in the backend-matrix-summary, release-notes-summary, and release-summary views so report consumers can inspect them without opening the detailed backend report. The reference snapshot source summary now also includes the body/epoch coverage windows that define the checked-in snapshot, so the release-facing evidence surfaces both provenance and window coverage directly. The checked-in reference snapshot source-window summary now also has a typed, validated body-window breakdown for archaeology and report parity, the comparison snapshot source-window summary now does the same for the 70-row comparison corpus, and the new comparison body-class coverage summary now mirrors that validation slice in release-facing reports and now also surfaces in the validation-report and release-summary renderers; the independent hold-out snapshot now also has typed, validated source-window and body-class coverage breakdowns for the validation slice; the release summary and validation report now surface that overlap line alongside the hold-out source-window evidence so the shared validation slice stays visible in the main release-facing reports. The checked-in reference snapshot body-class coverage summary now also surfaces typed major-body and selected-asteroid window breakdowns in release-facing reports, the packaged-artifact fixture has been regenerated from the updated reference snapshot, and packaged-data regression coverage now checks the earliest and latest source-snapshot epoch for every bundled artifact body so the artifact boundary evidence spans the full packaged body set. The production-generation boundary overlay now also has a standalone request corpus, inventory summary, per-body window summary, provenance summary, and report surface for reuse in validation and future artifact-generation work, and the merged production-generation corpus now also has a release-facing source-window summary so combined reference+boundary spans are visible per body without decoding the fixture. The combined JPL evidence summary now surfaces the boundary-overlay provenance line, and the production-generation boundary window summary now validates its derived slice and report drift now fails closed as well; the packaged artifact regeneration provenance now carries an explicit profile identifier, and residual-body coverage now cross-checks against the bundled body list before release-facing validation proceeds. The packaged-data crate now also exposes deterministic generator-parameter and manifest outputs alongside the production-profile skeleton, and the validation artifact summary now surfaces the generation manifest directly next to the skeleton so the current regeneration posture is visible in the release-facing artifact report. The artifact inspection summary and artifact report now also surface a compact body-class mix for the bundled prototype so the current packaged composition is visible without decoding the body list. The lunar source-window summary now also includes the reference-only apparent Moon comparison windows alongside the published 1992-04-12 Moon example and the checked-in 2451911.5/2451912.5 high-curvature samples, and the JPL source-window summary now exposes per-body coverage windows for the checked-in reference snapshot; the new major-body high-curvature reference slice now extends that coverage through the 2451913.5 boundary day across all comparison bodies so the broader corpus evidence is visible without decoding the fixture, and the new major-body high-curvature body-window summary now exposes the per-body spans directly in report surfaces. The house-validation corpus summary now also surfaces the baseline formula-family coverage and the documented latitude-sensitive constraint notes for Placidus, Koch, and Topocentric directly in its release-facing summary. The ayanamsa catalog validation summary now also names custom-definition-only labels directly in its sidereal-metadata coverage line so release-facing catalog evidence shows which labels are intentionally metadata-light, and the compatibility profile now cross-checks those custom-definition ayanamsa labels against the metadata-coverage summary while also keeping the profile-only custom-definition labels (`True Balarama`, `Aphoric`, `Takra`) distinct from the catalog's intentionally unresolved names; the compatibility-profile report now also lists the documented custom-definition ayanamsa labels explicitly in the coverage section. Representative ayanamsa reference-offset examples now also render in the compatibility-profile and release-summary reports, and the baseline/release golden tests now pin Lahiri and Galactic Equator (Fiorenza) against their published epoch/offset metadata. The built-in house catalog now fails closed if a descriptor ever falls back to an unknown formula family, keeping new house-system variants from silently entering the release profile without audit. Request-semantic summaries, CLI help, and policy docs now also spell out the deferred apparent-place, topocentric, and native-sidereal posture explicitly, and the release-facing request-semantics prose is now centralized behind a shared formatter in `pleiades-validate`. That formatter now validates the shared request-policy summary before rendering the observer, apparentness, and request-policy lines so drifted policy prose fails closed in the report layer. The observer and apparentness policy summaries themselves are now typed and validated before the report layer renders them, so those direct surfaces fail closed too. Chart-snapshot diagnostics now also reuse the shared backend time-scale, frame, and apparentness summaries so rendered policy text stays aligned with that same contract. The shared request-policy summary validation now also rejects blank, whitespace-padded, and multiline drift so the compact report wording stays fail-closed, and the JPL provenance summary wrappers now reject embedded line breaks as well. The direct request validator now also checks both mean and apparent value-mode capabilities, so future apparent-only or mean-only backends will fail closed instead of silently accepting unsupported coordinates. The shared frame-policy summary now also fails closed when it drifts from the current canonical frame-policy posture, and the comparison/regression tolerance audits now use body-class-specific release thresholds for the active planetary corpus. The chart CLI now also accepts explicit UTC/UT1 TT-offset aliases so the common convenience conversion step is spelled out without changing the direct backend policy surface, and the policy docs now mention those aliases alongside the existing help text. Compatibility-profile summaries now also surface house formula families alongside the existing house-code and ayanamsa breadth evidence.

## 2. Production reference corpus expansion

**Phase:** 1 — Accuracy Closure and Request Semantics

**Goal:** provide enough trusted source/reference material for validation and artifact generation.

**Work items:**

- Expand JPL/reference rows for all release-claimed major bodies and selected asteroids. The production-generation boundary overlay now includes the full independent hold-out snapshot, the merged reference-plus-boundary corpus now totals 81 rows across 15 bodies and 9 epochs, and the interpolation-quality corpus now includes the current 41-sample, 10-body evidence set; the overlay also now has a standalone request corpus, summary, and provenance line for generation tooling, and the combined JPL evidence summary now includes the overlay provenance as well. The selected-asteroid source window summary now also surfaces the broader 20-sample asteroid provenance slice, the reference-snapshot reports now expose the broader Moon source-window summary with the apparent comparison windows, and the source-window reporting now covers both the checked-in reference snapshot and the merged production-generation corpus; the checked-in reference source-window summary is now typed and validated, it now derives its body order directly from the checked-in snapshot entries, the new reference/hold-out overlap summary shows that the current validation slice still shares 32 body-epoch pairs with the reference corpus, so any future expansion must draw from fresh public-source rows rather than borrowing the hold-out fixture, the checked-in reference corpus now also carries an added 2001-01-04 boundary slice that brings the snapshot to 100 rows across 15 bodies and 11 epochs, the comparison snapshot/report surfaces now remain aligned with the current 70-row / 9-epoch comparison corpus, and the independent hold-out body-class summary is now available to mirror the comparison/reference archaeology.
- Preserve deterministic manifests, provenance summaries, checksums, and pure-Rust parsing/loading.
- Decide whether interpolation is a runtime feature, generation-only aid, or transparency-only evidence.

## 3. Advanced request semantics decision

**Phase:** 1 — Accuracy Closure and Request Semantics

**Goal:** make time/observer/apparentness behavior release-ready.

**Work items:**

- Decide first-release posture for built-in Delta T and UTC/UT1 convenience conversion. The current backend/report surface now exposes the deferred no-built-in-Delta-T posture as a separate typed summary, but the decision itself is still to keep or implement a real model.
- Decide first-release posture for apparent-place corrections.
- Decide first-release posture for topocentric body positions.
- Update metadata, errors, docs, CLI flags, and tests so unsupported modes continue to fail closed. The explicit UTC/UT1 alias spelling is already wired through the CLI and now documented in the policy notes; the remaining task is broader reference-corpus expansion.

## 4. Production artifact profile and generator skeleton

**Phase:** 2 — Production Compressed Artifacts

**Goal:** prepare the artifact-generation path without overstating prototype accuracy.

**Work items:**

- Define production artifact profile identifiers, body sets, stored/derived/unsupported outputs, and target thresholds. The packaged-data crate now exposes a dedicated production-profile skeleton summary, a structured target-threshold scaffold that captures the current measured fit envelope and body-class scope envelopes, and deterministic generator-parameter/manifest outputs that capture the current prototype posture; the target-threshold scaffold now also carries the release-profile identifier, the compact validation and release summaries now surface the target-threshold posture and generation manifest, and the remaining work is to finalize the release thresholds and align the manifest with production fit validation.
- Keep the prototype artifact labeled as prototype until Phase 2 fit validation passes.
- Add tests for profile/header validation and claim drift.

## 5. Artifact fit improvement loop

**Phase:** 2 — Production Compressed Artifacts

**Goal:** reduce packaged-data error to release-grade thresholds.

**Work items:**

- Tune segment lengths, polynomial order, quantization, and residual corrections by body class.
- Add interior samples, segment-boundary checks, and any additional high-curvature body windows needed beyond the current lunar and major-body evidence slices.
- Compare decoded outputs against Phase 1 generation sources.
- Publish measured errors and benchmarks in artifact summaries.

## 6. House-system evidence audit

**Phase:** 3 — Compatibility Evidence and Catalog Completion

**Goal:** ensure every release-advertised house system has formula and failure-mode evidence.

**Work items:**

- Add or verify golden scenarios across hemispheres, latitudes, and polar/high-latitude constraints. The release-facing house-validation corpus now also surfaces the baseline formula-family coverage and the documented latitude-sensitive constraint notes for Placidus, Koch, and Topocentric directly in its summary, and it now includes a mirrored southern polar stress chart so both hemispheres are represented in the high-latitude evidence slice.
- Check cusp ordering, angle derivation, normalization, and house placement. The compatibility-profile display and shared validation summary now also render the latitude-sensitive house-system set from the same formatter, keeping the release-facing constraint list in sync with the house evidence audit.
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
