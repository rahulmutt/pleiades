# Phase 4 — Request-Mode Semantics

## Goal

Make all advanced request modes either implemented with documented assumptions
and validation or consistently rejected with structured errors.

## Current baseline

- Backend request metadata records frame, time scale, observer, zodiac,
  apparentness, and related policy.
- First-party backend body positions are mean geometric and geocentric.
- Chart-level sidereal conversion is handled above backends by the domain/catalog
  layer.
- Policy summaries document unsupported UTC convenience, Delta T, apparent-place,
  topocentric body positions, and native sidereal backend output.

## Completed in SP3

Motion/speed output (`SpeedPolicy::FittedDerivative`, `Motion = Derived`) was
implemented and gated in SP3. The packaged artifact profile classifies longitude,
latitude, and distance speed channels as `Motion = Derived`. Published per-body-class
speed ceilings are enforced by the CI gate (lon/lat speed: 0.5 ″/day for
luminaries/inner planets, 0.05 ″/day for outer planets, 120 ″/day for asteroids;
radial speed: 1×10⁻⁴ AU/day for luminaries/inner/outer, 1×10⁻² AU/day for
asteroids). See `crates/pleiades-data/src/thresholds.rs`.

## Remaining implementation work

- Decide whether built-in UTC/UT1 convenience and Delta-T modeling are in scope
  for the first production release.
- If civil-time conversion is implemented, define inputs, time-scale outputs,
  error handling, data/provenance requirements, and tests.
- Implement apparent-place corrections only with documented astronomy formulas,
  source references, and validation fixtures.
- Implement topocentric body positions only with clear observer semantics,
  coordinate-frame handling, and tests.
- Keep native sidereal backend output unsupported unless a backend provides
  validated native behavior distinct from chart-layer sidereal conversion.
- The remaining motion scope for Phase 4 is **apparent, topocentric, native
  sidereal, and civil-time motion output** only — derived geometric speed is
  already implemented (SP3).

## Exit criteria

- Unsupported request modes produce structured, documented errors everywhere.
- Implemented request modes have validation fixtures, rustdoc/API examples, CLI
  coverage, backend metadata, and release-profile entries.
