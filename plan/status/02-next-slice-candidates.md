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

## 2. Lunar theory source selection

**Goal:** turn `pleiades-elp` into a planned production implementation instead of an approximate placeholder.

Suggested scope:

- document the chosen ELP/lunar-theory source and license/provenance;
- define supported channels and date range;
- implement Moon longitude/latitude/distance for a small validated epoch set;
- explicitly mark node/apogee/perigee support as implemented or unsupported with structured errors.

## 3. JPL reader/interpolator expansion

**Goal:** build on the completed small fixture interpolator and turn `pleiades-jpl` into a stronger reference backend.

Completed first slice:

- defined the checked-in derivative CSV fixture format in crate metadata and docs;
- parse multiple epochs in pure Rust;
- linearly interpolate Cartesian vectors between adjacent same-body samples;
- preserve exact fixture epochs as golden tests;
- distinguish unsupported bodies from out-of-range fixture requests.

Remaining suggested scope:

- add a larger documented public-input-derived fixture with more bodies and denser samples;
- validate interpolation error against held-out JPL Horizons epochs;
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

## 5. Artifact profile schema draft

**Goal:** prepare Phase 2 without blocking Phase 1 accuracy work.

Suggested scope:

- define the serialized artifact header/profile fields in `pleiades-compression` docs;
- record stored/derived/unsupported channel semantics;
- add round-trip tests for profile metadata only;
- avoid claiming generated production artifacts until source-backed generation exists.

## 6. Compatibility-profile verification tightening

**Goal:** prevent catalog metadata drift while backend work proceeds.

Suggested scope:

- make verification fail when a release-profile catalog entry lacks descriptor metadata;
- check alias uniqueness within each catalog;
- report implementation status separately from catalog presence;
- add tests around known release-profile entries.

## Selection guidance

Prioritize slices 1-4 for Phase 1. Slices 5 and 6 are safe parallel preparatory work if they do not distract from production ephemeris accuracy.
