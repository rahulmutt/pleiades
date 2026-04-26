# Status 2 — Next Slice Candidates

Use these as candidate implementation slices. Each slice should be independently reviewable and leave the workspace buildable and tested.

## 1. VSOP87 source-data path

**Goal:** establish the production pattern for formula-based planetary calculations.

Suggested scope:

- choose and document the VSOP87 variant/source;
- add pure-Rust coefficient representation or generation step for one body;
- compute a canonical ecliptic position at J2000;
- add reference/golden tests and metadata updates;
- extend validation summaries with measured error for the implemented body. Aggregate and per-body comparison summaries are now present, so the remaining work is to attach the new source-backed VSOP87 evidence to those reports.

Progress note (2026-04-24): the placeholder `pleiades-vsop87` path now reports deterministic central-difference longitude/latitude/distance speeds for supported planets. This improves chart-facing motion semantics but does not replace the planned source-backed VSOP87 coefficient work above.

Progress note (2026-04-24): the VSOP87 source-data path now covers the Sun through Neptune with public IMCCE/CELMECH VSOP87B inputs, exact J2000 tests, and validation evidence for all source-backed major planets. The Earth, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, and Neptune paths now use generated binary coefficient-table slices derived from the vendored source files, and the remaining Phase 1 work for this track is release-grade error envelopes.

Progress note (2026-04-24): the first source-data increment has landed for the Sun path. `pleiades-vsop87` now evaluates a generated binary coefficient table derived from the vendored public IMCCE VSOP87B Earth source file, transforms it into geometric geocentric solar coordinates, and tests the J2000 result against a full-file VSOP87B golden value.

Progress note (2026-04-24): the Mercury source-data increment now uses the vendored public IMCCE VSOP87B Mercury file. Mercury geocentric output now evaluates the full source file, carries exact J2000 golden values against the Earth slice, and reports vendored provenance in the backend/source-profile evidence.

Progress note (2026-04-24): the source-data increment now also includes Venus via the full public IMCCE VSOP87B file. `pleiades-vsop87` now parses the vendored `VSOP87B.ven` source directly, reduces it against the Earth slice for geocentric Venus output, and tests the J2000 geocentric Venus result against full-file IMCCE VSOP87B Venus/Earth golden values.

Progress note (2026-04-24): the Mars source-data increment now also uses the vendored public IMCCE VSOP87B Mars file. `pleiades-vsop87` now evaluates the full Mars source file, reduces it against the Earth slice, reports vendored Mars provenance in metadata, and tests the J2000 geocentric Mars result against full-file IMCCE VSOP87B Mars/Earth golden values.

Progress note (2026-04-24): the VSOP87 implementation now exposes per-body source profiles from `pleiades-vsop87` and renders them in the validation backend matrix. The profiles distinguish vendored full-file VSOP87B paths from the fallback mean-element path, reducing ambiguity while the generated-table work remains queued.

Progress note (2026-04-24): the VSOP87 source-backed path now continues with Jupiter. `pleiades-vsop87` now evaluates the vendored public IMCCE VSOP87B Jupiter source file, reduces it against the Earth slice, reports Jupiter provenance in metadata/source profiles, and tests the J2000 geocentric Jupiter result against full-file IMCCE VSOP87B Jupiter/Earth golden values.

Progress note (2026-04-24): the VSOP87 source-backed path now continues with Saturn. `pleiades-vsop87` now evaluates the vendored public IMCCE VSOP87B Saturn source file, reduces it against the Earth slice, reports Saturn provenance in metadata/source profiles, and tests the J2000 geocentric Saturn result against full-file IMCCE VSOP87B Saturn/Earth golden values.

Progress note (2026-04-24): the VSOP87 source-backed path now covers the remaining VSOP87 major planets, Jupiter, Saturn, Uranus, and Neptune. All four bodies evaluate vendored public IMCCE VSOP87B source files, reduce against the Earth slice, report source-backed provenance in metadata/source profiles, and have J2000 geocentric golden tests against full-file IMCCE VSOP87B Jupiter/Saturn/Uranus/Neptune/Earth values. Pluto remains a separate non-VSOP87 special case, and complete generated coefficient-table ingestion remains outstanding.

Progress note (2026-04-24): validation comparison reports now render per-body expected tolerance status in addition to aggregate and per-body measured deltas. The current tolerance table is explicitly labeled as Phase 1 interim evidence for full-file VSOP87B planetary paths, compact ELP lunar paths, and the Pluto mean-elements fallback, making measured exceedances visible while complete generated tables remain pending. The validation report now also includes body-class error envelopes, and the comparison report now adds a body-class tolerance posture that counts within/outside-tolerance bodies by class and lists the outliers, so release-facing summaries can show coarse luminary/planet grouping alongside the per-body tolerance table. The expected tolerance policy is now also surfaced as a structured backend-family/body-class catalog, and the compact release summary now also carries per-scope body-and-sample coverage counts on that policy line, keeping the report text aligned with the underlying threshold table. That tolerance policy line now also names the bodies covered by each scope and prints `none` for empty scopes, so the scope-to-body mapping remains visible in compact release output.

Progress note (2026-04-24): the validation backend matrix now prints canonical J2000 source-backed VSOP87B evidence for the Sun and major planets, including measured deltas against the same public IMCCE reference values used by the regression tests. The compact validation and backend-matrix summaries now also surface the same source-backed evidence snapshot, and now include a concise source-documentation count for the structured VSOP87 body profiles, giving the current full-file source path a visible, reproducible error summary while the generated complete-table ingestion work remains queued. The VSOP87 source documentation is now also structured per body, with machine-readable variant, frame, units, reduction, and date-range fields so future generated-table ingestion can consume the current provenance model directly.

Progress note (2026-04-25): the body-class tolerance posture now also names the bodies that drive each class-level max longitude, latitude, and distance delta, making the release-facing coarse tolerance audit easier to inspect without changing the comparison corpus or the interim thresholds.

Progress note (2026-04-24): the VSOP87 validation output now also pairs each source-backed body profile with its source file, provenance, and measured canonical deltas, and the compact summary now reports a body-profile evidence count. The release-facing validation summaries now also distinguish the generated binary Sun-through-Neptune paths from the Pluto mean-element fallback, which makes the current source state clearer while the documented VSOP87 regeneration tooling work remains queued.

Progress note (2026-04-24): the VSOP87 crate now exposes the canonical J2000 body-evidence envelope as public structured data, and the validation layer reuses that shared summary instead of duplicating the same envelope privately. This keeps the release-facing evidence shape consistent between the backend crate and validation reports while the new comparison-audit gate and broader source-backed envelope refinement proceed in validation.

Progress note (2026-04-24): the batch API for the source-backed Sun-through-Neptune sample set is now covered by a canonical J2000 regression test, so the current VSOP87 evidence is verified both through single-position queries and through the default `positions` batch path.
Progress note (2026-04-25): the canonical VSOP87 evidence helper now derives its release-facing envelope from one batch query over the full source-backed sample set, and the regression suite also checks that reversed batch requests preserve canonical sample order. The same envelope summary now also carries the source kind and source file for each axis peak, which keeps the release-facing audit trail tied to the exact generated-binary or vendored input path while the broader release-grade envelope work remains queued. The VSOP87 body-evidence summary now also surfaces the number of bodies outside the interim envelope explicitly alongside the list, which makes the compact validation audit slightly easier to scan.

Progress note (2026-04-25): the VSOP87 batch-path regression now also covers the full supported major-planet batch at J2000, including the Pluto fallback path, so the batch contract is now exercised across the entire supported planetary set rather than only the source-backed subset.

Progress note (2026-04-24): the VSOP87 body-specific source metadata, canonical samples, and per-body profiles now all derive from a single internal catalog table, which reduces drift in the release-facing documentation and gives the eventual generated-table path one structured place to attach new source-backed bodies. The VSOP87 crate now also exposes a deterministic source-audit manifest for the vendored source files, and a maintainer-facing regeneration helper plus binary now rewrite the checked-in generated coefficient blobs from the vendored source text before the runtime path is rewritten.

Progress note (2026-04-25): the VSOP87 source documentation summary now also reports an explicit source-path breakdown for generated-binary, vendored full-file, truncated, and fallback body profiles, so the release-facing validation output can distinguish the current source mix without changing the canonical body-evidence line. The same summary now also carries the fallback-body list as structured data, which keeps the Pluto fallback name available without recomputing it from the body-profile table in the reporting layer. The compact VSOP87 body-evidence summary now also names any bodies outside interim limits explicitly (or `none` when the current sample set stays within the interim envelope), which makes the source-backed audit easier to scan without opening the full backend matrix. The same evidence summary is now owned by `pleiades-vsop87` as a structured backend helper, and validation reuses it directly instead of recomputing the source-kind and interim-limit breakdown in the tooling layer. The canonical evidence, source documentation, and source audit report strings are now backend-owned as well, keeping the VSOP87 reporting surface consistent with the JPL and ELP helpers. The canonical J2000 evidence summary now also carries mean absolute deltas and an out-of-limit count, so the compact release-facing envelope shows a little more shape without changing the underlying comparison corpus. The source-data regeneration hook now also reports structured parse failures for malformed vendored rows instead of silently collapsing them, which makes the checked-in coefficient pipeline safer to regenerate and audit.

Progress note (2026-04-25): the canonical VSOP87 body evidence and release-facing backend-matrix lines now also spell out the per-axis interim limits and signed margins alongside the measured deltas, which makes the source-backed error envelope easier to audit without changing the underlying J2000 reference rows.
Progress note (2026-04-26): the canonical VSOP87 evidence summary now also includes median and RMS longitude, latitude, and distance deltas, so the release-facing source-backed envelope now carries both central-tendency and spread statistics in addition to the existing mean/max lines.

## 2. Lunar theory source selection

**Goal:** turn `pleiades-elp` into a planned production implementation instead of an approximate placeholder.

Suggested scope:

- document the chosen ELP/lunar-theory source and license/provenance;
- define supported channels and date range;
- implement Moon longitude/latitude/distance for a small validated epoch set;
- explicitly mark node/apogee/perigee support as implemented or unsupported with structured errors.

Progress note (2026-04-24): the current compact `pleiades-elp` path now reports deterministic central-difference mean-motion estimates for the Moon, mean node, and true node. This improves chart-facing lunar motion semantics, but does not replace the planned source-backed lunar theory selection, coefficient implementation, or reference validation.

Progress note (2026-04-24): `pleiades-elp` now exposes mean lunar apogee and mean lunar perigee as explicitly supported mean-only lunar points using Meeus-style mean perigee/apogee formulae, including equatorial transforms and finite-difference longitude speeds. True apogee and true perigee remain unsupported until a source-backed true-point model is selected and validated.

Progress note (2026-04-24): the compact ELP backend now also publishes a structured lunar-theory specification naming the current Meeus-style analytical baseline, the supported lunar channels, the explicitly unsupported true apogee/perigee bodies, the provenance note, and the current J2000-centered validation window. The validation backend matrix and compact summaries now render that specification explicitly, so the current Moon/node/apogee/perigee baseline is visible in release-facing metadata while a full ELP coefficient implementation remains pending.

Progress note (2026-04-24): the ELP baseline now also carries an explicit lunar-theory policy doc plus a regression test over a short high-curvature Moon window, so the current source/provenance posture and nearby motion continuity are both exercised before the eventual source-backed ELP selection lands.
Progress note (2026-04-25): that high-curvature Moon window now also has a backend-owned continuity summary rendered in validation and backend-matrix output, so the regression slice is visible in the release-facing lunar evidence alongside the canonical samples.

Progress note (2026-04-24): `pleiades-elp` now exposes a stable lunar-theory source identifier plus an explicit redistribution posture for the current Meeus-style analytical baseline, and the validation/reporting layer renders those provenance details alongside the supported/unsupported lunar bodies.
Progress note (2026-04-25): the backend-matrix ELP section now also prints the unsupported lunar bodies explicitly, so the release-facing summary now shows the deferred true apogee/perigee channels alongside the supported lunar baseline.

Progress note (2026-04-25): the `pleiades-elp` Moon path now uses a Meeus-style truncated lunar position series and validates against the published 1992-04-12 geocentric Moon example, which gives the current lunar baseline a concrete external reference point while a full ELP coefficient selection remains queued. The lunar-theory metadata now also includes an explicit Meeus citation plus a conservative license/provenance note, so validation and release summaries can distinguish source citation from redistribution posture more clearly.

Progress note (2026-04-25): the compact lunar baseline now also publishes a dedicated equatorial cross-check against the published 1992-04-12 Moon RA/Dec example, and the validation/backend-matrix summaries surface that cross-check explicitly so the shared mean-obliquity transform stays visible in release-facing output while the broader source-backed lunar selection remains queued.

Progress note (2026-04-25): the supported lunar-point slice now also has a batch-path regression over the Moon, nodes, and mean apogee/perigee samples, so the current compact lunar baseline is verified through both batch and single-request lookups. The same lunar backend now also rejects true apogee and true perigee through batch requests with the same structured `UnsupportedBody` error used for single-request calls, which keeps the deferred lunar channels explicit even when they appear in `positions()`.

Progress note (2026-04-25): the compact lunar-theory specification now also renders its request policy explicitly — TT/TDB only, tropical only, mean only, equatorial/ecliptic frames, and no topocentric observer mode — so the current source/provenance posture is easier to audit while the eventual source-backed ELP selection remains pending. The compact lunar-theory specification now also carries explicit truncation and output-unit notes, and the validation/release summaries render those notes alongside the provenance fields, so the current truncated baseline is easier to inspect without changing the public API. The baseline also exposes an explicit source-family label in metadata and summaries, which keeps the source-selection posture structured without widening the API surface. The lunar-theory selection data is now centralized into a single static specification, and the canonical evidence slice is checked against that supported-body list so the current Meeus-style baseline stays internally consistent while the source-backed ELP selection remains pending. The backend now also exposes dedicated supported/unsupported lunar-body accessors from that centralized selection, and the backend metadata derives its body coverage from the same source-of-truth slice so future source-selection swaps have one less place to drift. The lunar metadata now also exposes its validation window as a structured `TimeRange`, which lets the release summaries render the sampled epoch span directly instead of inferring it from prose. The canonical lunar reference evidence summary and formatter now also live in `pleiades-elp`, so the validation layer can reuse the backend-owned counts and epoch range without duplicating the same summary logic. The lunar reference slice now also includes the 1992 Moon example, J2000 lunar-point anchors, published 1913-05-27 true-node and mean-node examples, a published 1959-12-07 mean-node example, and a published 2021-03-05 mean-perigee example, broadening the current validation evidence without changing the public request model. The backend-owned lunar reference summary now also has a measured error-envelope companion, so the validation layer can surface the current residual snapshot alongside the canonical samples in release-facing summaries. The backend-owned lunar summary is now parameterized through `format_lunar_theory_specification(&LunarTheorySpecification)`, which keeps the release-facing one-liner reusable for future source-selection variants without re-implementing the template in validation.

Progress note (2026-04-26): the lunar validation/reporting path now also includes a second reference-only apparent Moon anchor from the 1968-12-24 low-accuracy Meeus-style example, broadening the apparent evidence slice while apparent requests continue to fail explicitly. The same lunar baseline is now also exposed as a one-entry catalog plus catalog summary helper, which gives the release-facing source-selection posture a stable selection surface for future source-backed variants without changing the public API. The catalog now also has explicit typed lookup helpers for source identifier, model name, family label, and documented aliases, so the selection surface is easier to extend without relying on one generic resolver. The selected lunar source can now also round-trip through a typed source-selection key and resolver helper, and now has a structured family-key lookup as well, which tightens the future source-backed swap path without widening the public catalog shape. The catalog surface now also has a backend-owned consistency check that enforces the selected-entry round trip plus case-insensitive uniqueness for identifiers, model names, family labels, and documented aliases, so the one-entry baseline is already guarded against the most obvious future source-selection collisions. The validation, backend-matrix, and release-summary views now also surface that catalog-validation status line, and the catalog validation state is now also exposed as structured backend-owned data before formatting the release-facing line, so the round-trip and alias-uniqueness health is visible alongside the catalog summary in release-facing output. The compact lunar capability summary now also reports whether the catalog validates cleanly, so the same catalog-health signal is visible in the broader release-facing capability output too. The compact lunar source-selection, catalog, and capability summaries now also carry the typed source-family enum directly, so future source-backed lunar entries can consume a structured family key without reparsing the human-readable label. The typed resolver surface is now also pinned with case-insensitive regression coverage across source identifier, model name, family label, and alias keys, and the release-facing lunar catalog validation line now spells out case-insensitive key matching too, so the current one-entry catalog has a small explicit guardrail before any future source-backed lunar entries land.
Progress note (2026-04-26): the compact lunar source-selection summary and validation output now also print the typed selected catalog key alongside the selected source, which keeps the one-entry lunar catalog provenance aligned with the typed lookup surface and makes the short release-facing provenance line more explicit.
Progress note (2026-04-26): the compact compatibility profile, release notes summary, and release summary now explicitly surface the latitude-sensitive house-system subset, so release-facing catalog summaries now call out the polar-failure-constraint systems alongside the baseline and release-specific counts.

## 3. JPL reader/interpolator expansion

**Goal:** build on the completed small fixture interpolator and turn `pleiades-jpl` into a stronger reference backend.

Completed first slice:

- defined the checked-in derivative CSV fixture format in crate metadata and docs;
- parse multiple epochs in pure Rust;
- linearly interpolate Cartesian vectors between adjacent same-body samples;
- preserve exact fixture epochs as golden tests;
- distinguish unsupported bodies from out-of-range fixture requests.

Progress note (2026-04-24): `pleiades-jpl` now derives coarse leave-one-out interpolation quality samples from the checked-in public-input fixture, which now includes an additional 2500000.0 TDB epoch across the comparison-body set, and the validation backend matrix renders those measured linear-interpolation errors. These checks are intentionally labeled as transparency evidence rather than production tolerances.

Progress note (2026-04-25): the checked-in JPL reference snapshot now includes an added 2600000.0 Mars hold-out epoch in addition to the 2400000.0 comparison epoch across the Sun-through-Pluto bodies, expanding the fixture to 46 rows across 15 bodies and 6 epochs and broadening the leave-one-out evidence to 21 samples across 10 bodies.

Progress note (2026-04-24): the compact validation report summary now also carries a JPL interpolation-quality envelope, so the current leave-one-out evidence is visible alongside the comparison summaries without changing the backend contract.

Progress note (2026-04-25): the JPL snapshot backend now prefers cubic interpolation on four-sample windows when the fixture has enough same-body epochs, with quadratic and linear fallbacks for smaller windows. This keeps the checked-in public-input fixture pure Rust and geocentric while making the reference backend slightly stronger before any larger corpus work lands.

Progress note (2026-04-25): the JPL reference asteroid slice now also has a batch-path regression over the exact J2000 evidence rows, so the current public-input fixture is exercised through `positions()` as well as single-request lookups.

Progress note (2026-04-25): the compact JPL interpolation-quality summary now also names the worst-case body for each error metric, so the current hold-out envelope is easier to audit while the broader public-input corpus work remains queued. The backend now also publishes a distinct-body interpolation-kind coverage helper for the cubic/quadratic/linear tiers, giving the hold-out transparency slice a small extra audit hook without expanding the fixture. The compact release summary now also surfaces that JPL interpolation-quality envelope, so the one-screen release overview keeps the reference backend's leave-one-out evidence visible alongside the comparison totals. The validation comparison envelope now also reports RMS longitude, latitude, and distance deltas in the full report and compact summary, broadening the release-grade comparison statistics without changing the underlying corpus.
Progress note (2026-04-25): the same summary now also carries the held-out epoch for each peak, and now reports mean bracket-span and mean error metrics alongside the maxima, so the interpolation audit can name the exact instant that produced the current maximum when a maintainer is comparing runs while also exposing the average leave-one-out envelope. The same summary now also reports RMS longitude, latitude, and distance error, which gives the leave-one-out evidence a clearer spread metric without expanding the fixture or the public request model. The summary now also carries median bracket-span and median error magnitudes, which rounds out the compact interpolation envelope with a robust central-tendency view. Latest progress (2026-04-26): the same interpolation-quality summary now also reports distinct epoch coverage plus the earliest/latest hold-out epoch window, so the release-facing transparency evidence shows the sampled span alongside the body/kind breakdown. Latest progress (2026-04-26): the independent JPL hold-out summary now also reports 95th-percentile longitude, latitude, and distance errors, keeping the smaller public-input validation corpus aligned with the main interpolation-quality envelope without changing the checked-in rows.

Progress note (2026-04-25): the JPL interpolation-quality samples now derive from leave-one-out runtime interpolation against held-out exact rows, so the transparency evidence now measures the backend's actual interpolation path instead of only a linear counterfactual. The same summary now also distinguishes cubic, quadratic, and linear fallback tiers, so the checked-in fixture can show which interpolation path each held-out sample exercised.

Progress note (2026-04-25): the checked-in JPL reference snapshot now also has exact J2000 golden coverage for the named asteroid subset and the custom 433-Eros body, so the source-backed asteroid fixture path is now exercised with explicit coordinates in addition to the interpolation transparency checks.

Progress note (2026-04-25): the checked-in JPL reference snapshot now also publishes a compact coverage summary with row, body, epoch, asteroid-row, and epoch-range counts, and the validation/release-facing summaries now surface that snapshot coverage alongside the exact asteroid evidence rows.
Progress note (2026-04-25): the JPL snapshot coverage and exact asteroid evidence summaries now also label their reference epochs with TDB in release-facing output, which keeps the checked-in JPL snapshot aligned with the current time-scale policy while broader corpus expansion remains queued.

Progress note (2026-04-25): the JPL asteroid subset now also renders exact J2000 evidence rows in backend matrices, validation reports, and release notes, making the source-backed asteroid fixture visible in release-facing output rather than only in tests. The JPL snapshot coverage and exact asteroid evidence summaries now also live in `pleiades-jpl`, and validation reuses those backend-owned report helpers directly so the release-facing coverage text stays co-located with the snapshot backend.

Progress note (2026-04-25): the JPL interpolation-quality summary plus distinct-body coverage line now also live in `pleiades-jpl` as one backend-owned report helper, and validation reuses that combined formatter directly so the release-facing interpolation evidence stays co-located with the snapshot backend instead of rebuilding the two-line report privately. The same backend now also publishes a compact request-policy line for the reference snapshot posture, so the current TT/TDB, equatorial/ecliptic, mean-only, and geocentric-only semantics stay visible in release-facing reports alongside the interpolation evidence.

Progress note (2026-04-26): the JPL snapshot backend now also publishes a small independent Mars/Jupiter hold-out corpus from the same public Horizons source material, and the validation/release evidence now renders that separate hold-out interpolation envelope alongside the existing leave-one-out transparency summary, so the first suggested JPL hold-out validation slice has landed while broader corpus expansion remains conditional on the remaining envelope.
Progress note (2026-04-26): the JPL snapshot backend now also has a batch-path regression over the full checked-in reference snapshot in equatorial mode, preserving exact order and exact-coordinate parity across every reference row while exercising the mean-obliquity transform on the complete public-input fixture. The checked-in JPL reference and hold-out CSV comments are now parsed into structured manifest metadata too, and the release-facing evidence helper now surfaces the hold-out source line from that metadata so the public-input provenance is less hardcoded.
Progress note (2026-04-26): the parsed JPL snapshot manifests are now also exposed through public summary-line helpers, so downstream code can reuse the checked-in reference and hold-out provenance metadata directly instead of treating the fixture headers as report-only strings.

Remaining suggested scope:

- broaden the public-input derivative corpus further only if the remaining interpolation error envelope still needs tightening.

## 4. Delta T, time-scale, and observer policy

**Goal:** make time and observer semantics explicit before more accuracy claims are added.

Suggested scope:

- add a project-level policy document or rustdoc section;
- identify which APIs accept UTC/UT/TT/TDB and where conversion is caller-provided versus library-provided;
- keep chart-level house observers separate from topocentric backend position requests unless a chart API explicitly adds a topocentric position mode;
- add tests for unsupported or ambiguous time-scale and observer-bearing topocentric requests;
- update backend metadata and validation reports.

Progress note (2026-04-24): chart assembly now uses the observer location for house calculations without passing it into geocentric body-position backend requests, and the VSOP87/ELP placeholder backends now reject direct observer-bearing requests with `InvalidObserver`.

Progress note (2026-04-25): the JPL snapshot and packaged-data backends now also have explicit observer-bearing and apparent-place rejection regressions, so the geocentric-only request policy is covered beyond the shared backend helpers and the VSOP87/ELP placeholder path.

Progress note (2026-04-25): the shared time-scale helpers now also include a direct caller-supplied UT1-to-TDB convenience in both `pleiades-types` and `pleiades-core::ChartRequest`, so dynamical-time chart staging can compose TT-UT1 and TDB-TT offsets explicitly even when the input begins at UT1.
Progress note (2026-04-25): the validation-report-summary and release-summary views now also render an explicit time-scale policy line summarizing the TT/TDB direct-request posture and caller-supplied UTC/UT1 conversion model, so the report layer mirrors the documented Delta T policy more directly.

Progress note (2026-04-24): the initial time-scale, Delta T, apparentness, and observer policy is documented in `docs/time-observer-policy.md`. The current VSOP87 and ELP paths now reject `Apparentness::Apparent` requests with structured `InvalidRequest` errors instead of silently returning mean geometric coordinates, matching the existing JPL and packaged-data behavior. The shared backend request-policy helpers now centralize those time-scale, frame, apparentness, and observer checks so the concrete backends can keep their source-specific body logic while sharing the common guardrails.

Progress note (2026-04-24): `pleiades-types` now provides caller-supplied time-scale offset helpers (`JulianDay::add_seconds`, `Instant::with_time_scale_offset`, and `Instant::tt_from_ut1`) plus a structured `TimeScaleConversionError`. This does not add a built-in Delta T model, but it gives applications and validation fixtures a typed way to apply an explicit external `TT - UT1` policy before querying TT/TDB-only backends. Signed UT1-to-TT and UTC-to-TT helpers now round out the same surface, so historical or externally modeled negative Delta T policies can stay explicit too.

Progress note (2026-04-25): `pleiades-types` also now centralizes the mean-obliquity ecliptic-to-equatorial rotation in a reusable `EclipticCoordinates::to_equatorial` helper, and the VSOP87/ELP paths now call that shared transform instead of duplicating the same conversion math locally. The shared type layer now also exposes `EquatorialCoordinates::to_ecliptic`, so the same obliquity policy can be round-tripped in tests and documentation when a caller needs the inverse geometric rotation. The JPL snapshot backend now also accepts equatorial-frame requests and derives equatorial coordinates from the shared ecliptic fixture, so the frame-transform policy now reaches the reference backend as well.

Progress note (2026-04-25): `pleiades-types` and `pleiades-core::ChartRequest` now also expose caller-supplied TT-to-TDB conversion conveniences alongside the existing UTC/UT1 helpers, keeping TDB-tagged backend requests explicit without adding a built-in relativistic model.

Progress note (2026-04-25): the same time-scale surface now also has a symmetric caller-supplied TT-from-TDB helper, which keeps TDB-tagged requests round-trippable without implying a built-in relativistic conversion model.

Progress note (2026-04-25): the TT-, UT1-, and UTC-to-TDB helper surface now also carries signed TDB-TT variants in `pleiades-types` and `pleiades-core::ChartRequest`, and the TDB-to-TT path now also has an explicit `*_signed` alias for naming symmetry with the other conversion helpers, so the current policy docs and builder API can represent negative TDB corrections explicitly instead of forcing every caller through a non-negative duration.

Progress note (2026-04-24): `pleiades-core::ChartRequest` now includes explicit time-scale conversion conveniences, including a generic caller-supplied offset builder and a UT1-to-TT helper. UTC-tagged chart requests now have the same explicit TT conversion convenience, which keeps chart assembly aligned with the typed offset policy while still requiring the caller to choose the conversion model.

Progress note (2026-04-24): validation report corpus summaries now print explicit epoch labels with time-scale tags, so the report layer now exposes the reference time-scale choice alongside the corpus windows while the project still relies on caller-provided conversion policy.

## 5. Artifact profile schema draft

**Goal:** prepare Phase 2 without blocking Phase 1 accuracy work.

Completed first slice (2026-04-24):

- added a versioned `ArtifactProfile` to `pleiades-compression` headers;
- serialized stored channel, derived output, unsupported output, and speed-policy metadata in the deterministic binary payload;
- added round-trip tests for default and explicit profile metadata;
- updated the packaged-data prototype to expose the conservative ecliptic-only/no-motion profile without expanding production artifact claims.

Progress note (2026-04-25): artifact profile summaries now appear in `artifact-summary`, `validation-report-summary`, and `release-summary`, which makes the Phase 2 artifact capability posture visible in release-facing reports before generated production artifacts exist. The packaged-artifact profile summary now also reports how many bundled bodies share the same conservative capability profile, which makes the current single-profile packaged scope a little more explicit while body-specific generated artifacts remain pending. The packaged-data provenance line now also names the full 11-body packaged set through a shared helper, so the artifact metadata no longer suggests the smaller Sun/Moon-only wording from the earlier placeholder summary. The packaged-data request-policy note is now also structured instead of string-only, and the validation/artifact reports reuse that structured record so the current geocentric-only, TT/TDB, ecliptic, tropical, mean-only posture stays synchronized with metadata. The compression crate now also exposes explicit body and segment random-access helpers, giving the artifact lookup path a reusable body/interval boundary API for future validation work. The checksum-verification path now also has explicit corruption regressions in both `pleiades-compression` and the packaged-data fixture path, so artifact decode failures on tampering stay covered as the reproducible-data work continues. The packaged-artifact regeneration command now also includes the JPL reference-snapshot coverage line in its emitted provenance text, which keeps the checked-in regeneration path tied to its public-input corpus instead of only to the output bytes. The artifact profile summary now also uses stable `Display` labels for stored channels, derived outputs, unsupported outputs, and speed policy, which keeps release-facing capability text independent of enum debug names. The packaged-data backend now also publishes a backend-owned frame-treatment summary and the artifact, validation, release, and CLI summary paths render it alongside the existing packaged request policy, so the packaged artifact's ecliptic-only posture stays explicit in release-facing provenance.

Remaining suggested scope:

- refine profile fields when generated artifacts introduce body-specific stored/derived semantics;
- avoid claiming generated production artifacts until source-backed generation exists.

## 6. Compatibility-profile verification tightening

**Goal:** prevent catalog metadata drift while backend work proceeds.

Completed first slice (2026-04-24):

- `verify-compatibility-profile` now rejects catalog descriptors with blank canonical names, blank notes metadata, blank labels, or repeated exact labels within each house-system/ayanamsa catalog;
- regression tests cover duplicate house labels and missing ayanamsa descriptor notes;
- redundant resolver-equivalent house aliases were pruned where canonical or retained alias matching already covers the label.

Progress note (2026-04-24): backend matrix reports now include explicit implementation-status labels and notes for each backend, and the compact matrix summary counts statuses separately from body/catalog coverage. This keeps fixture, partial source-backed, preliminary algorithm, prototype artifact, and routing façade states visible in release artifacts without implying production accuracy from catalog presence alone.

Progress note (2026-04-24): validation-side compatibility-profile coverage now also asserts several recently added release-profile entries — including the Equal (MC) and Equal (1=Aries) house-table forms, Pullen SR, True Citra Paksha, P.V.R. Narasimha Rao, and B. V. Raman — so the rendered release profile stays anchored to the current catalog breadth while the broader release-profile status audit continues.

Progress note (2026-04-24): the compatibility-profile command now also pins additional release-profile spellings such as Equal table of houses, Whole Sign system, Morinus house system, Galactic Equator (Fiorenza), Valens Moon, and Babylonian (House Obs), extending the release-facing breadth checks without changing the verification model.

Progress note (2026-04-25): the compatibility-profile regressions now also pin the exact Polich-Page "topocentric" table of houses, T Polich/Page ("topocentric"), Poli-equatorial, horizon/azimuth, horizon/azimut, Babylonian Huber, Galactic Equator (True), and Valens Moon ayanamsa spellings surfaced by the current release profile, keeping the source-label appendix text anchored as more catalog breadth lands. The latest regression pass also anchors the Galactic Center (Rgilbrand), Galactic Center (Mardyks), Galactic Center (Mula/Wilhelm), Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, PVR Pushya-paksha, and Galactic Center (Cochrane) entries so the release-profile audit stays tied to the current historical/reference-frame coverage.

Progress note (2026-04-24): `verify-compatibility-profile` now prints an explicit release-posture line plus custom-definition and known-gap counts, so the release-facing verification output calls out baseline preservation, explicit release additions, and documented caveats in addition to the descriptor/alias counts.

Progress note (2026-04-25): the compatibility-profile verification slice now also pins the exact Swiss Ephemeris house-table code spellings and the J2000/J1900/B1950 and Suryasiddhanta source-label forms, keeping the release-facing appendix text anchored to the same interoperability spellings that the generated profile renders. The custom-definition verifier now also rejects accidental built-in ayanamsa collisions while explicitly allowing the current intentional Babylonian house-family homographs, so the release-profile custom-definition bucket stays disjoint from both built-in catalogs without breaking the currently published profile.

Progress note (2026-04-25): the same verification slice now also pins the Placidus, Koch, Whole Sign, and Sunshine table-of-houses spellings, so the exact release-profile source-label wording for a few more canonical house-system aliases stays covered as the catalog expands. The built-in house source-label appendix now also carries the Vehlow Equal table of houses and Pullen SD (Sinusoidal Delta) table of houses forms explicitly, which keeps those table-form variants visible in the compatibility profile alongside the broader alias coverage.

Progress note (2026-04-26): the compatibility-profile regression coverage now also pins the True Chitra Paksha and True Chitrapaksha spellings alongside the already-covered True Citra Paksha alias batch, so the release-facing ayanamsa appendix keeps that interoperability trio explicitly covered in both the CLI and validation smoke tests.
Progress note (2026-04-26): the compatibility-profile smoke tests now also pin True Sheoran, closing the remaining release-profile spelling for that ayanamsa variant in the CLI and validation audit paths.
Progress note (2026-04-26): the compatibility-profile regressions now also pin the WvA alias from the APC house-family appendix, keeping that short-form house label explicitly covered in the release-facing catalog checks alongside the broader source-label batches.
Progress note (2026-04-26): the compatibility-profile smoke tests now also pin the Horizon/Azimuth family labels (`Horizon house system`, `Horizontal house system`, `Azimuth house system`, `Azimuthal house system`) plus the Carter and Krusinski interoperability spellings, extending the release-facing house-system drift checks a little further without changing the verification model.
Progress note (2026-04-26): the compatibility-profile command tests now also pin the `Equal (cusp 1 = Asc)` and `Whole Sign (house 1 = Aries)` forms, so the ascendant-anchored equal-house and whole-sign source-label wording stays anchored in the release-facing verification path. The compact compatibility-profile summary and verification output now also label the release-specific house-system and ayanamsa canonical-name lists explicitly, so the short release-facing profile audit makes those canonical slices visible instead of implying them only through the rendered entries.
Progress note (2026-04-26): the compatibility-profile and release-summary smoke tests now also pin `Albategnius` and `Gauquelin sectors`, keeping the remaining release-specific house-system additions visible in the compact release-facing catalog views as the broader alias audit continues.

Progress note (2026-04-26): the compatibility-profile smoke tests now also pin `True Sheoran`, which keeps one more release-profile ayanamsa spelling explicitly covered in both the core release-note rendering and the CLI compatibility-profile output as the breadth audit continues.
Progress note (2026-04-26): the compatibility-profile smoke tests now also pin `Skydram/Galactic Alignment` and `Skydram (Mardyks)`, keeping the Galactic Center (Mardyks) alias spellings explicitly covered in the CLI compatibility-profile output as the breadth audit continues.

Progress note (2026-04-26): the same compatibility-profile slice now also pins the remaining Equal/MC and Equal/1=Aries table-and-system spellings, including the `Equal/MC table of houses`, `Equal/MC house system`, `Equal/1=Aries table of houses`, `Equal/1=Aries house system`, `Equal/1=0 Aries`, `Equal (cusp 1 = 0° Aries)`, and `Whole Sign (house 1 = Aries) table of houses` forms, so the release-facing equal-house appendix stays anchored as the catalog grows.
Progress note (2026-04-26): the release-notes regression now also anchors `Krusinski/Pisa/Goelzer` and `Equal/MC = 10th`, keeping a couple of still-visible release-profile house spellings exercised in the compact maintainer-facing summary while the broader catalog audit continues.

Progress note (2026-04-25): the compatibility-profile verification slice now also anchors the Babylonian (Eta Piscium) and Suryasiddhanta / Surya Siddhanta 499 CE source-label spellings, so the remaining historical-source labels in the rendered profile stay pinned as the catalog grows.

Progress note (2026-04-25): validation coverage now also pins the additional ayanamsa spellings for True Revati, True Pushya, Lahiri (ICRC), Lahiri (1940), and Yukteshwar, keeping a few more release-profile source-label entries from drifting as the catalog grows.

Progress note (2026-04-25): the compatibility-profile command now also pins the remaining Dhruva Galactic Center (Middle Mula) aliases plus the explicit Suryasiddhanta (Revati), Suryasiddhanta (Citra), and True Pushya (PVRN Rao) spellings, so the current release-profile text stays anchored to the latest ayanamsa and reference-frame wording.

Progress note (2026-04-25): the compatibility-profile and release-notes regressions now also pin Babylonian (Britton), Babylonian (Aldebaran), Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Galactic Equator (IAU 1958), and Galactic Equator (Mula), tightening the remaining release-profile reference and mean-sun alias coverage without changing the published catalog shape.

Progress note (2026-04-26): the compatibility-profile smoke tests now also pin Lahiri (VP285) and Krishnamurti (VP291), keeping the remaining release-profile ayanamsa variants visible in both the CLI and validation coverage paths as the catalog-breadth audit continues.

Remaining suggested scope:

- add broader coverage for any still-unpinned release-profile spellings or status lines that future catalog batches introduce.

## Selection guidance

Prioritize slices 1-4 for Phase 1. Slices 5 and 6 are safe parallel preparatory work if they do not distract from production ephemeris accuracy.
