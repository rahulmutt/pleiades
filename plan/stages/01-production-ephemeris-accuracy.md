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

Implemented foundations include backend traits, metadata, composite routing, major body identifiers, lunar points, baseline asteroids, a chart façade, CLI commands, a snapshot-style `pleiades-jpl` with exact lookup, linear interpolation, selected asteroid coverage, and expanded public-input leave-one-out interpolation quality reporting, preliminary `pleiades-vsop87` and `pleiades-elp` crates, deterministic central-difference motion estimates for the current VSOP87 path and compact ELP Moon/lunar-point path, explicit mean-only support for mean lunar apogee and mean lunar perigee, explicit rejection of unsupported topocentric and apparent requests in geocentric/mean-only backends, an initial time-scale/Delta T/observer policy document, caller-supplied time-scale offset helpers for explicit external UT1-to-TT/related policies, a vendored IMCCE VSOP87B Earth source file for geocentric Sun output plus truncated IMCCE VSOP87B Mercury, Venus, Mars, Jupiter, Saturn, Uranus, and Neptune coefficient slices for geocentric planet output with J2000 golden tests, VSOP87 per-body source profiles rendered in validation backend matrices, and validation reports that expose aggregate plus per-body comparison error summaries, explicit interim tolerance status by body, and compact source-backed VSOP87 evidence snapshots.

## Remaining implementation goals

1. Implement production `pleiades-vsop87` planetary calculations.
   - Load or encode VSOP87 coefficient data in pure Rust.
   - Document variant, truncation policy, frames, units, and date range.
   - Produce geocentric astrology-facing ecliptic positions for Sun and planets.
   - Add batch-path tests covering all supported planets at canonical epochs.

2. Implement production `pleiades-elp` lunar calculations.
   - Select and document a pure-Rust lunar theory source.
   - Support Moon longitude, latitude, distance, and useful speed outputs.
   - Implement or explicitly defer mean/true nodes and apogee/perigee where mathematically justified.
   - Add regression tests around high-curvature lunar intervals.

3. Upgrade `pleiades-jpl` from snapshot fixture to reference backend.
   - Parse documented public JPL-style files or a reproducible derivative format in pure Rust.
   - Support multiple epochs through interpolation rather than exact fixture lookup only.
   - Include selected asteroid coverage for Ceres, Pallas, Juno, and Vesta when source data is available.
   - Preserve snapshot fixtures as small regression/golden tests.

4. Strengthen time, apparentness, observer, and coordinate semantics.
   - Expand the initial Delta T policy into implemented conversion support or a release-grade caller-provided conversion contract.
   - Clarify TT/UT/TDB handling in requests and validation data.
   - Keep chart house observers separate from backend topocentric position requests unless an explicit topocentric chart mode is added.
   - Implement equatorial/ecliptic transforms where the release profile claims them.
   - Ensure topocentric observer requests and apparent/mean flags either produce distinct documented behavior or return structured unsupported-feature errors.

5. Expand validation evidence.
   - Add golden positions for major bodies, lunar points, and baseline asteroids.
   - Generate cross-backend comparison reports with body/date/error summaries. Aggregate and per-body summary sections are now implemented; future source-backed backend increments should populate them with tighter measured errors.
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
