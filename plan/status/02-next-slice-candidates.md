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

Progress note (2026-04-24): the first source-data increment has landed for the Sun path. `pleiades-vsop87` now evaluates a checked-in truncated leading-term slice of public IMCCE VSOP87B Earth coefficients, transforms it into geometric geocentric solar coordinates, and tests the J2000 result against a full-file VSOP87B golden value.

Progress note (2026-04-24): the same truncated VSOP87B spherical-coefficient representation now covers Mercury's heliocentric channel. Mercury geocentric output is reduced against the VSOP87B Earth slice and has a J2000 regression test against full-file IMCCE VSOP87B Mercury/Earth golden values, with backend provenance updated to distinguish the Mercury source-backed path from the remaining orbital-element fallback planets.

Progress note (2026-04-24): the truncated VSOP87B source-backed path has been extended to Venus. `pleiades-vsop87` now evaluates a checked-in leading-term IMCCE VSOP87B Venus slice, reduces it against the Earth slice, and tests the J2000 geocentric Venus result against full-file IMCCE VSOP87B Venus/Earth golden values.

Progress note (2026-04-24): the same truncated VSOP87B source-backed path has been extended to Mars. `pleiades-vsop87` now evaluates a checked-in leading-term IMCCE VSOP87B Mars slice, reduces it against the Earth slice, reports Mars provenance in metadata, and tests the J2000 geocentric Mars result against full-file IMCCE VSOP87B Mars/Earth golden values.

Progress note (2026-04-24): the mixed VSOP87 implementation now exposes per-body source profiles from `pleiades-vsop87` and renders them in the validation backend matrix. The profiles distinguish source-backed truncated VSOP87B paths from fallback mean-element paths, reducing ambiguity until complete generated tables land.

Progress note (2026-04-24): the truncated VSOP87B source-backed path has been extended to Jupiter. `pleiades-vsop87` now evaluates a checked-in leading-term IMCCE VSOP87B Jupiter slice, reduces it against the Earth slice, reports Jupiter provenance in metadata/source profiles, and tests the J2000 geocentric Jupiter result against full-file IMCCE VSOP87B Jupiter/Earth golden values.

Progress note (2026-04-24): the same truncated VSOP87B source-backed path has been extended to Saturn. `pleiades-vsop87` now evaluates a checked-in leading-term IMCCE VSOP87B Saturn slice, reduces it against the Earth slice, reports Saturn provenance in metadata/source profiles, and tests the J2000 geocentric Saturn result against full-file IMCCE VSOP87B Saturn/Earth golden values.

Progress note (2026-04-24): the checked-in truncated VSOP87B source-backed path now covers the remaining VSOP87 major planets, Uranus and Neptune. Both bodies evaluate leading-term IMCCE VSOP87B slices, reduce against the Earth slice, report source-backed provenance in metadata/source profiles, and have J2000 geocentric golden tests against full-file IMCCE VSOP87B Uranus/Neptune/Earth values. Pluto remains a separate non-VSOP87 special case, and complete generated coefficient-table ingestion remains outstanding.

Progress note (2026-04-24): validation comparison reports now render per-body expected tolerance status in addition to aggregate and per-body measured deltas. The current tolerance table is explicitly labeled as Phase 1 interim evidence for truncated VSOP87B planetary paths, compact ELP lunar paths, and the Pluto mean-elements fallback, making measured exceedances visible while complete source-backed tables remain pending.

Progress note (2026-04-24): the validation backend matrix now prints canonical J2000 source-backed VSOP87B evidence for the Sun and major planets, including measured deltas against the same public IMCCE reference values used by the regression tests. The compact validation and backend-matrix summaries now also surface the same source-backed evidence snapshot, giving the current truncated coefficient path a visible, reproducible error summary while the generated complete-table ingestion work remains queued. The VSOP87 source documentation is now also structured per body, with machine-readable variant, frame, units, reduction, truncation, and date-range fields so future generated-table ingestion can consume the current provenance model directly.

## 2. Lunar theory source selection

**Goal:** turn `pleiades-elp` into a planned production implementation instead of an approximate placeholder.

Suggested scope:

- document the chosen ELP/lunar-theory source and license/provenance;
- define supported channels and date range;
- implement Moon longitude/latitude/distance for a small validated epoch set;
- explicitly mark node/apogee/perigee support as implemented or unsupported with structured errors.

Progress note (2026-04-24): the current compact `pleiades-elp` path now reports deterministic central-difference mean-motion estimates for the Moon, mean node, and true node. This improves chart-facing lunar motion semantics, but does not replace the planned source-backed lunar theory selection, coefficient implementation, or reference validation.

Progress note (2026-04-24): `pleiades-elp` now exposes mean lunar apogee and mean lunar perigee as explicitly supported mean-only lunar points using Meeus-style mean perigee/apogee formulae, including equatorial transforms and finite-difference longitude speeds. True apogee and true perigee remain unsupported until a source-backed true-point model is selected and validated.

## 3. JPL reader/interpolator expansion

**Goal:** build on the completed small fixture interpolator and turn `pleiades-jpl` into a stronger reference backend.

Completed first slice:

- defined the checked-in derivative CSV fixture format in crate metadata and docs;
- parse multiple epochs in pure Rust;
- linearly interpolate Cartesian vectors between adjacent same-body samples;
- preserve exact fixture epochs as golden tests;
- distinguish unsupported bodies from out-of-range fixture requests.

Progress note (2026-04-24): `pleiades-jpl` now derives coarse leave-one-out interpolation quality samples from the checked-in sparse fixture and the validation backend matrix renders those measured linear-interpolation errors. These checks are intentionally labeled as transparency evidence rather than production tolerances.

Remaining suggested scope:

- add a larger documented public-input-derived fixture with more bodies and denser samples;
- validate interpolation error against independent held-out JPL Horizons epochs beyond the sparse checked-in fixture;
- report interpolation quality and tolerances in validation summaries using the existing aggregate and per-body comparison sections;
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

Progress note (2026-04-24): `pleiades-core::ChartRequest` now includes explicit time-scale conversion conveniences, including a generic caller-supplied offset builder and a UT1-to-TT helper. That keeps chart assembly aligned with the typed offset policy while still requiring the caller to choose the conversion model.

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

Remaining suggested scope:

- add tests around additional known release-profile entries and release-profile status lines.

## Selection guidance

Prioritize slices 1-4 for Phase 1. Slices 5 and 6 are safe parallel preparatory work if they do not distract from production ephemeris accuracy.
