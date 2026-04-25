# Phase 1 — Production Ephemeris Accuracy

## Purpose

Replace preliminary, sample-driven, or simplified backend behavior with source-backed astronomical calculations and explicit accuracy evidence. This phase is active because the workspace structure and user-facing APIs exist, but production release claims require validated ephemeris output.

## Spec drivers

- `spec/requirements.md`: FR-1, FR-2, FR-3, FR-7, FR-8, NFR-2, NFR-3
- `spec/backend-trait.md`: request/result semantics, metadata, errors, composite backends
- `spec/backends.md`: `pleiades-jpl`, `pleiades-vsop87`, `pleiades-elp` responsibilities
- `spec/api-and-ergonomics.md`: deterministic typed APIs, batch queries, failure modes
- `spec/validation-and-testing.md`: reference comparison and golden/regression tests

## Current baseline

Implemented foundations include backend traits, metadata, composite routing, major body identifiers, lunar points, baseline asteroids, a chart façade, CLI commands, a snapshot-style `pleiades-jpl` with exact lookup plus cubic/quadratic/linear interpolation, selected asteroid coverage, and expanded public-input leave-one-out interpolation quality reporting, preliminary `pleiades-vsop87` and `pleiades-elp` crates, deterministic central-difference motion estimates for the current VSOP87 path and compact ELP Moon/lunar-point path, explicit mean-only support for mean lunar apogee and mean lunar perigee, explicit rejection of unsupported topocentric and apparent requests in geocentric/mean-only backends, an initial time-scale/Delta T/observer policy document, caller-supplied time-scale offset helpers for explicit external UT1-to-TT/related policies, validation corpus summaries that now label their unique epochs with explicit time-scale tags, vendored IMCCE VSOP87B Earth, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, and Neptune source files for geocentric Sun-through-Neptune output, plus generated binary coefficient-table slices for the Sun/Earth, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, and Neptune paths, VSOP87 per-body source profiles rendered in validation backend matrices, explicit VSOP87 frame-treatment notes for the J2000 ecliptic/equinox source frame and mean-obliquity equatorial transform, and validation reports that expose aggregate plus per-body comparison error summaries, explicit interim tolerance status by body, compact source-backed VSOP87 evidence snapshots, and a concise VSOP87 body-evidence outlier note when a measured body falls outside the interim envelope.

## Remaining implementation goals

1. Maintain and document the production `pleiades-vsop87` planetary calculations.
   - Keep the generated VSOP87 coefficient data path reproducible in pure Rust.
   - Document variant, truncation policy, frames, units, and date range.
   - Preserve geocentric astrology-facing ecliptic positions for Sun and planets.
   - Keep the source-backed major-planet coefficient path reproducible from public inputs and validate error envelopes.
   - Add batch-path tests covering all supported planets at canonical epochs.
   - Progress note: the crate now ships a maintainer-facing regeneration helper plus binary that rewrites the checked-in generated blobs from the vendored source text.
   - Progress note: the canonical J2000 VSOP87 evidence summary now names the body that drives each maximum delta axis, and now also records the source kind and source file behind each axis peak, which makes the release-facing envelope easier to audit without widening the public request model.
   - Progress note: the VSOP87 source-audit summary now also prints the deterministic fingerprint count directly in release-facing output, so the reproducibility line stays explicit alongside the raw source-size and term-count evidence.
   - Progress note: the VSOP87 regression suite now also covers the full supported planetary batch at J2000, including the Pluto fallback path, so batch-path verification now spans the complete supported set rather than only the source-backed subset. The validation summary now also names Pluto explicitly as the mean-element fallback body in the source-documentation line, keeping the remaining non-VSOP87 special case visible in release-facing output.
   - Progress note: the canonical VSOP87 body-evidence envelope is now owned by `pleiades-vsop87` as a structured backend summary, and validation reuses it directly instead of recomputing the source-kind and interim-limit breakdown in the tooling layer. The VSOP87 canonical evidence, source documentation, source audit, and body-evidence report strings are now backend-owned too, so the release-facing reporting surface stays co-located with the source metadata instead of being rebuilt privately in validation. The canonical J2000 evidence summary now also reports mean absolute deltas and an out-of-limit count, so the compact release-facing envelope now mirrors the richer lunar and JPL summaries a little more closely.

2. Implement production `pleiades-elp` lunar calculations.
   - Select and document a pure-Rust lunar theory source.
   - Support Moon longitude, latitude, distance, and useful speed outputs.
   - Implement or explicitly defer mean/true nodes and apogee/perigee where mathematically justified.
   - Add regression tests around high-curvature lunar intervals.
   - Progress note: the backend now has an explicit lunar-theory policy document plus a short high-curvature Moon-window regression test, so the current baseline provenance and nearby motion continuity are exercised while source-backed ELP selection remains queued.
   - Progress note: the same high-curvature Moon-window regression slice now also has a backend-owned continuity summary rendered in validation/backend-matrix reports, so the nearby-motion check is visible in release-facing output rather than only in the unit test.
   - Progress note: the current lunar-theory selection now exposes a stable source identifier, canonical bibliographic citation, explicit source-family label, and explicit license/provenance posture alongside the supported and unsupported lunar bodies, and the validation/reporting layer renders those provenance details in release-facing summaries.
   - Progress note: validation backend-matrix output now also renders the ELP unsupported lunar bodies explicitly, so the release-facing lunar-theory section shows both the supported channels and the deferred true apogee/perigee slots.
   - Progress note: the Moon path now uses a Meeus-style truncated lunar position series instead of the earlier simplified orbital surrogate, and the backend now validates against the published 1992-04-12 geocentric Moon example in addition to the J2000 lunar-point checks.
   - Progress note: the compact lunar backend now also exports a canonical reference evidence slice for the published Moon example plus the J2000 lunar-point samples, and validation reports render that slice in the backend matrix and summary output so the current lunar baseline is easier to audit without changing the API contract.
   - Progress note: the lunar reference evidence summary now also carries the JD 2419914.5..2459278.5 range explicitly, making the current 1913-node-to-2021 coverage window visible in release-facing summaries.
   - Progress note: the lunar backend now also has a batch-path regression over the Moon, nodes, and mean apogee/perigee evidence slice, so the current supported lunar points are exercised through `positions()` in addition to single-request coverage. The same backend now also rejects true apogee and true perigee through batch requests with the same structured `UnsupportedBody` error used for single-request calls, keeping the deferred lunar channels explicit even when they arrive through `positions()`.
   - Progress note: the compact lunar-theory specification now also exposes a structured request-policy field — TT/TDB only, tropical only, mean only, ecliptic/equatorial frames, and no topocentric observer mode — so the current baseline remains visibly scoped while source-backed ELP selection stays queued.
   - Progress note: the compact lunar-theory specification now also carries explicit truncation and output-unit notes, and the validation/reporting layer renders those notes alongside the provenance fields so the current truncated baseline is easier to audit without changing the public API. The same baseline now also exposes its source-family label directly, which keeps the source-selection posture structured without widening the API surface.
   - Progress note: the lunar-theory selection data is now centralized into a single static specification, and the canonical evidence slice is checked against that supported-body list so the current Meeus-style baseline stays internally consistent as the source-selection work continues.
   - Progress note: the backend now also exposes dedicated supported/unsupported lunar-body accessors from that centralized selection, and the backend metadata derives its body coverage from the same source-of-truth slice so future source-selection swaps have one less place to drift.
   - Progress note: the compact lunar-theory metadata now also exposes its current validation window as a structured `TimeRange` in addition to the prose date-range note, so release-facing summaries can render the sampled epoch span without inferring it from freeform text.
   - Progress note: the release-facing lunar summary formatter now lives in `pleiades-elp`, which keeps the lunar provenance text owned by the backend crate rather than duplicated in validation/reporting code.
   - Progress note: that lunar summary is now parameterized through `format_lunar_theory_specification(&LunarTheorySpecification)`, so future source-selection variants can reuse the same backend-owned one-liner without rebuilding the template in validation.
   - Progress note: the canonical lunar reference evidence summary now also lives in `pleiades-elp`, and validation/reporting reuses that backend-owned summary and formatter instead of keeping a duplicate copy in the tooling layer.
   - Progress note: the new compact lunar capability summary helper now gives validation a structured body/frame/time-scale snapshot without parsing prose, keeping the source-selection posture easier to reuse in later release reports.
   - Progress note: the backend-owned lunar reference evidence summary now also has a measured error-envelope companion, and both the detailed report and backend-matrix summary render that envelope line so the current lunar evidence slice exposes a compact residual snapshot alongside the canonical samples.
   - Progress note: the lunar reference slice now also includes the 1992 Moon example, J2000 lunar-point anchors, published 1913-05-27 true-node and mean-node examples, a published 1959-12-07 mean-node example, and a published 2021-03-05 mean-perigee example, which broadens the current validation evidence without changing the public request model.
   - Progress note: the `pleiades-elp::LunarTheorySpecification` record now carries the selected source family directly, which keeps the current lunar baseline selection structured for future source-backed swaps and simpler release reporting.
   - Progress note: the lunar reference and equatorial error envelopes now also report mean residuals and explicit out-of-limit sample counts alongside the existing maxima, which makes the release-facing lunar evidence a little easier to scan while the current Meeus-style baseline remains the active lunar source-selection placeholder. The lunar error-envelope report strings now also spell out the applied regression limits directly, so the phase-1 lunar evidence stays explicit about the current threshold policy in release-facing summaries.

3. Upgrade `pleiades-jpl` from snapshot fixture to reference backend.
   - Parse documented public JPL-style files or a reproducible derivative format in pure Rust.
   - Support multiple epochs through interpolation rather than exact fixture lookup only.
   - Include selected asteroid coverage for Ceres, Pallas, Juno, and Vesta when source data is available.
   - Preserve snapshot fixtures as small regression/golden tests.
   - Progress note: the reference snapshot now has exact J2000 regression coverage for Ceres, Pallas, Juno, Vesta, and the custom 433-Eros asteroid entry, so the baseline asteroid subset is exercised as a golden path while broader corpus work remains queued.
   - Progress note: validation reports, backend matrices, and release notes now render the exact J2000 asteroid evidence rows for that subset instead of listing the bodies alone, which makes the source-backed asteroid fixture path visible in release-facing output.
   - Progress note: the checked-in JPL reference snapshot now prefers cubic interpolation on four-sample windows when the fixture has enough same-body epochs, with quadratic and linear fallbacks for smaller windows, which tightens the current pure-Rust interpolator without changing the public request model.
   - Progress note: the JPL reference backend now also has a batch-path regression over the exact asteroid evidence slice, so the same source-backed rows are verified through `positions()` as well as one-body-at-a-time queries.
   - Progress note: the JPL interpolation-quality summary now also carries the worst-case epoch for each measured envelope, and now reports mean bracket-span and mean error metrics alongside the maxima, so the report can name both the body and the held-out instant that produced the current interpolation peak while also exposing the average leave-one-out envelope.
   - Progress note: the checked-in JPL comparison snapshot now includes an added 2600000.0 Mars hold-out epoch in addition to the 2400000.0 epoch across the Sun-through-Pluto bodies, expanding the fixture to 46 rows across 15 bodies and 6 epochs and broadening the leave-one-out evidence to 21 samples across 10 bodies.
   - Progress note: the JPL interpolation-quality summary now lives in `pleiades-jpl`, and validation reuses the backend-owned summary/formatter instead of recomputing the same envelope privately, so the interpolation evidence now follows the same backend-owned reporting pattern as the lunar reference summary.
   - Progress note: the compact release summary now also surfaces the JPL interpolation-quality envelope, so the one-screen overview keeps the reference backend's leave-one-out evidence visible alongside the comparison totals. The same release overview now also carries the body-class tolerance posture and expected tolerance status, so release-facing error envelopes stay visible without opening the full validation report.

4. Strengthen time, apparentness, observer, and coordinate semantics.
   - Expand the initial Delta T policy into implemented conversion support or a release-grade caller-provided conversion contract.
   - Clarify TT/UT/TDB handling in requests and validation data.
   - Keep chart house observers separate from backend topocentric position requests unless an explicit topocentric chart mode is added.
   - Implement equatorial/ecliptic transforms where the release profile claims them.
   - Ensure topocentric observer requests and apparent/mean flags either produce distinct documented behavior or return structured unsupported-feature errors.
   - Progress note: the shared `pleiades-types::EclipticCoordinates::to_equatorial` helper now centralizes the mean-obliquity ecliptic-to-equatorial rotation used by the VSOP87 and ELP paths, so the frame transform is implemented once and covered by shared unit tests instead of being duplicated per backend.
   - Progress note: `pleiades-backend::EphemerisRequest::new` now defaults to mean geometric output, which keeps bare geocentric requests aligned with the current mean-only backend policy while still allowing callers to opt into apparent requests explicitly.
   - Progress note: the VSOP87 and compact lunar backend paths now accept both TT and TDB dynamical-time requests while still rejecting UT-based requests explicitly, which makes the current phase-1 time-scale policy easier to exercise in validation without adding a built-in Delta T or relativistic conversion model.
   - Progress note: the backend crates now also share centralized request-policy helpers for time-scale, frame, apparentness, and observer validation, which keeps the existing structured errors consistent while reducing duplicated guardrail code. The shared batch adapter now also has a fail-fast regression that preserves structured error kinds when a request sequence contains an unsupported body, so chart-style batch queries keep the same explicit failure semantics as single-request calls. The same batch adapter now also preserves structured apparentness rejections in mean-only backends, so unsupported `Apparentness::Apparent` requests still fail with the backend's explicit `InvalidRequest` kind when they arrive through `positions()`.
   - Progress note: the JPL, ELP, and VSOP87 batch-path regressions now also exercise explicit equatorial-frame requests, so the shared mean-obliquity transform and frame-preserving batch contract are covered in addition to the existing ecliptic batch tests.
   - Progress note: the `pleiades-cli` chart command now accepts an explicit `--tdb` instant tag in addition to its default TT-tagged request, so the user-facing chart report can surface a TDB-tagged instant directly instead of implying a hidden conversion policy.
   - Progress note: `pleiades-types` and `pleiades-core::ChartRequest` now also expose caller-supplied UTC-to-TDB helpers in addition to the UT1/UTC-to-TT and TT-to-TDB conveniences, so civil-time chart inputs can be lifted to TDB with explicit offset policy steps instead of hidden conversion logic.
   - Progress note: chart assembly now also has a regression proving house observers are used for house calculations only and are not forwarded into geocentric body-position backend requests, which keeps the observer/topocentric separation explicit in the façade layer.
   - Progress note: UT1-tagged callers now also have a direct caller-supplied UT1-to-TDB helper, so explicit dynamical-time staging can stay typed even when a caller already has TT-UT1 and TDB-TT offsets available.
   - Progress note: TDB-tagged callers now also have a caller-supplied TT-from-TDB helper in `pleiades-types` and `pleiades-core::ChartRequest`, which keeps the explicit conversion policy symmetric for callers that already begin from a TDB-tagged instant.
   - Progress note: the VSOP87 and compact ELP batch-path regressions now also cover explicit TDB-tagged requests for their supported body slices, so the current time-scale policy is exercised through `positions()` as well as single-request lookups without adding a built-in Delta T or relativistic conversion model.
   - Progress note: the `pleiades-cli` chart command now exposes explicit `--utc` / `--ut1` instant tags plus caller-supplied `--tt-offset-seconds` and `--tdb-offset-seconds` flags, so command-line chart runs can exercise the same conversion policy without implying a built-in Delta T or relativistic model.

5. Expand validation evidence.
   - Add golden positions for major bodies, lunar points, and baseline asteroids.
   - Generate cross-backend comparison reports with body/date/error summaries. Aggregate and per-body summary sections are now implemented; future source-backed backend increments should populate them with tighter measured errors.
   - Progress note: the validation and artifact reports now name the body driving each max longitude, latitude, and distance delta in the body-comparison and body-class envelope sections, which makes the release-facing error envelopes easier to audit without changing the measured corpus.
   - Progress note: the comparison-audit failure path now mirrors the body-class error envelopes, body-class tolerance posture, and tolerance policy sections, so release-failure output carries the same coarse envelope detail as the full validation report.
   - Progress note: the expected-tolerance status lines now also report signed margins to each class limit alongside the measured maxima, and the compact validation report summary now mirrors those margins in its expected-tolerance rows, which makes the remaining headroom or overshoot explicit in release-facing validation output without changing the comparison corpus.
   - Progress note: the compact release summary now also carries a short validation evidence line with comparison sample and tolerance counts, and now names the comparison-audit status explicitly, so the release-facing overview exposes a little more of the current error envelope without expanding into the full validation report.
   - Progress note: the same compact release summary now also surfaces the body-named comparison envelope, so the one-screen overview now mirrors the detailed comparison report a little more closely while the broader release-grade threshold work remains queued. The compact packaged-artifact summary now also includes the expected-tolerance rows and comparison tolerance audit counts, so the artifact release view now carries the current regression posture without forcing maintainers to open the full validation report.
   - Store expected tolerances by backend and body class.
   - Keep validation reproducible and pure Rust.

## Done criteria

- `pleiades-vsop87`, `pleiades-elp`, and `pleiades-jpl` metadata describe real source material and accuracy class.
- Major planets, Sun, Moon, and baseline asteroid support are tested against reference data.
- Unsupported bodies, frames, time scales, topocentric modes, and apparentness modes fail explicitly.
- Validation reports include measured errors and are consumable by release tooling.
- `cargo test --workspace` passes after the backend upgrades.

## Work that belongs in later phases

- Generating compressed 1500-2500 artifacts from these outputs belongs to Phase 2.
- Broad house/ayanamsa compatibility audits belong to Phase 3.
- Public release bundle publication and archival policies belong to Phase 4.
