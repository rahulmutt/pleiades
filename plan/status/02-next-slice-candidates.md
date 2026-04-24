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

Progress note (2026-04-24): validation comparison reports now render per-body expected tolerance status in addition to aggregate and per-body measured deltas. The current tolerance table is explicitly labeled as Phase 1 interim evidence for full-file VSOP87B planetary paths, compact ELP lunar paths, and the Pluto mean-elements fallback, making measured exceedances visible while complete generated tables remain pending. The validation report now also includes body-class error envelopes, and the comparison report now adds a body-class tolerance posture that counts within/outside-tolerance bodies by class and lists the outliers, so release-facing summaries can show coarse luminary/planet grouping alongside the per-body tolerance table.

Progress note (2026-04-24): the validation backend matrix now prints canonical J2000 source-backed VSOP87B evidence for the Sun and major planets, including measured deltas against the same public IMCCE reference values used by the regression tests. The compact validation and backend-matrix summaries now also surface the same source-backed evidence snapshot, and now include a concise source-documentation count for the structured VSOP87 body profiles, giving the current full-file source path a visible, reproducible error summary while the generated complete-table ingestion work remains queued. The VSOP87 source documentation is now also structured per body, with machine-readable variant, frame, units, reduction, and date-range fields so future generated-table ingestion can consume the current provenance model directly.

Progress note (2026-04-24): the VSOP87 validation output now also pairs each source-backed body profile with its source file, provenance, and measured canonical deltas, and the compact summary now reports a body-profile evidence count. The release-facing validation summaries now also distinguish the generated binary Sun-through-Neptune paths from the Pluto mean-element fallback, which makes the current source state clearer while the documented VSOP87 regeneration tooling work remains queued.

Progress note (2026-04-24): the VSOP87 crate now exposes the canonical J2000 body-evidence envelope as public structured data, and the validation layer reuses that shared summary instead of duplicating the same envelope privately. This keeps the release-facing evidence shape consistent between the backend crate and validation reports while broader error-envelope auditing remains queued.

Progress note (2026-04-24): the batch API for the source-backed Sun-through-Neptune sample set is now covered by a canonical J2000 regression test, so the current VSOP87 evidence is verified both through single-position queries and through the default `positions` batch path.

Progress note (2026-04-24): the VSOP87 body-specific source metadata, canonical samples, and per-body profiles now all derive from a single internal catalog table, which reduces drift in the release-facing documentation and gives the eventual generated-table path one structured place to attach new source-backed bodies. The VSOP87 crate now also exposes a deterministic source-audit manifest for the vendored source files, and a maintainer-facing regeneration helper plus binary now rewrite the checked-in generated coefficient blobs from the vendored source text before the runtime path is rewritten.

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

## 3. JPL reader/interpolator expansion

**Goal:** build on the completed small fixture interpolator and turn `pleiades-jpl` into a stronger reference backend.

Completed first slice:

- defined the checked-in derivative CSV fixture format in crate metadata and docs;
- parse multiple epochs in pure Rust;
- linearly interpolate Cartesian vectors between adjacent same-body samples;
- preserve exact fixture epochs as golden tests;
- distinguish unsupported bodies from out-of-range fixture requests.

Progress note (2026-04-24): `pleiades-jpl` now derives coarse leave-one-out interpolation quality samples from the checked-in public-input fixture, which now includes an additional 2500000.0 TDB epoch across the comparison-body set, and the validation backend matrix renders those measured linear-interpolation errors. These checks are intentionally labeled as transparency evidence rather than production tolerances.

Progress note (2026-04-24): the compact validation report summary now also carries a JPL interpolation-quality envelope, so the current leave-one-out evidence is visible alongside the comparison summaries without changing the backend contract.

Progress note (2026-04-24): the JPL snapshot backend now uses quadratic interpolation on three-sample windows when possible, with a linear fallback for sparse bodies. This keeps the checked-in public-input fixture pure Rust and geocentric while making the reference backend slightly stronger before any larger corpus work lands.

Remaining suggested scope:

- add additional public-input epochs or bodies if broader interpolation coverage is needed beyond the current expanded fixture;
- validate interpolation error against independent held-out JPL Horizons epochs beyond the checked-in fixture;
- consider higher-order interpolation once measured linear error is insufficient.

## 4. Delta T, time-scale, and observer policy

**Goal:** make time and observer semantics explicit before more accuracy claims are added.

Suggested scope:

- add a project-level policy document or rustdoc section;
- identify which APIs accept UTC/UT/TT/TDB and where conversion is caller-provided versus library-provided;
- keep chart-level house observers separate from topocentric backend position requests unless a chart API explicitly adds a topocentric position mode;
- add tests for unsupported or ambiguous time-scale and observer-bearing topocentric requests;
- update backend metadata and validation reports.

Progress note (2026-04-24): chart assembly now uses the observer location for house calculations without passing it into geocentric body-position backend requests, and the VSOP87/ELP placeholder backends now reject direct observer-bearing requests with `InvalidObserver`.

Progress note (2026-04-24): the initial time-scale, Delta T, apparentness, and observer policy is documented in `docs/time-observer-policy.md`. The current VSOP87 and ELP paths now reject `Apparentness::Apparent` requests with structured `InvalidRequest` errors instead of silently returning mean geometric coordinates, matching the existing JPL and packaged-data behavior.

Progress note (2026-04-24): `pleiades-types` now provides caller-supplied time-scale offset helpers (`JulianDay::add_seconds`, `Instant::with_time_scale_offset`, and `Instant::tt_from_ut1`) plus a structured `TimeScaleConversionError`. This does not add a built-in Delta T model, but it gives applications and validation fixtures a typed way to apply an explicit external `TT - UT1` policy before querying TT-only backends.

Progress note (2026-04-24): `pleiades-core::ChartRequest` now includes explicit time-scale conversion conveniences, including a generic caller-supplied offset builder and a UT1-to-TT helper. UTC-tagged chart requests now have the same explicit TT conversion convenience, which keeps chart assembly aligned with the typed offset policy while still requiring the caller to choose the conversion model.

Progress note (2026-04-24): validation report corpus summaries now print explicit epoch labels with time-scale tags, so the report layer now exposes the reference time-scale choice alongside the corpus windows while the project still relies on caller-provided conversion policy.

## 5. Artifact profile schema draft

**Goal:** prepare Phase 2 without blocking Phase 1 accuracy work.

Completed first slice (2026-04-24):

- added a versioned `ArtifactProfile` to `pleiades-compression` headers;
- serialized stored channel, derived output, unsupported output, and speed-policy metadata in the deterministic binary payload;
- added round-trip tests for default and explicit profile metadata;
- updated the packaged-data prototype to expose the conservative ecliptic-only/no-motion profile without expanding production artifact claims.

Remaining suggested scope:

- refine profile fields when generated artifacts introduce body-specific stored/derived semantics;
- connect profile summaries to validation and release reports;
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

Progress note (2026-04-24): `verify-compatibility-profile` now prints an explicit release-posture line plus custom-definition and known-gap counts, so the release-facing verification output calls out baseline preservation, explicit release additions, and documented caveats in addition to the descriptor/alias counts.

Remaining suggested scope:

- add broader coverage for any still-unpinned release-profile spellings or status lines that future catalog batches introduce.

## Selection guidance

Prioritize slices 1-4 for Phase 1. Slices 5 and 6 are safe parallel preparatory work if they do not distract from production ephemeris accuracy.
