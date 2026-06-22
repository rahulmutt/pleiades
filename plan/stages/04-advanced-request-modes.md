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
- Policy summaries document topocentric body positions and native sidereal
  backend output as remaining unsupported modes (UTC convenience, Delta T, and
  apparent-place are now implemented; see "Completed" sections below).

## Completed in SP3

Motion/speed output (`SpeedPolicy::FittedDerivative`, `Motion = Derived`) was
implemented and gated in SP3. The packaged artifact profile classifies longitude,
latitude, and distance speed channels as `Motion = Derived`. Published per-body-class
speed ceilings are enforced by the CI gate (lon/lat speed: 0.5 ″/day for
luminaries/inner planets, 0.05 ″/day for outer planets, 120 ″/day for asteroids;
radial speed: 1×10⁻⁴ AU/day for luminaries/inner/outer, 1×10⁻² AU/day for
asteroids). See `crates/pleiades-data/src/thresholds.rs`.

## Completed: civil-time UTC/UT1 → TT/TDB conversion

Built-in civil-time conversion is implemented in the `pleiades-time` crate:

- Civil UTC or UT1 calendar datetimes are converted to TT or TDB `Instant`s
  over the 1900–2100 window.
- UTC conversion uses the IERS Bulletin C leap-second table (exact, from 1972).
- UT1 conversion uses the observed/extrapolated Delta-T table (IERS/USNO +
  Espenak–Meeus); future epochs beyond the observed table fall back to
  Delta-T extrapolation.
- A TT↔TDB periodic relativistic correction is applied when TDB output is
  requested.
- A typed `ConversionProvenance` record carries the path, quality
  (`exact` / `observed` / `predicted`), Delta-T seconds, TAI−UTC count,
  and data sources on every result.
- High-level entry point: `pleiades_core::ChartRequest::from_civil(...)`,
  returning `CivilChartRequest { request, provenance }`.
- CLI: `chart --civil <YYYY-MM-DDTHH:MM:SS> [--civil-scale utc|ut1]
  [--civil-target tt|tdb]`; provenance/quality line appended to chart output.
- The caller-supplied retagging path (`--tt-*`/`--tdb-*` offset flags, direct
  `Instant` helpers) remains unchanged as the lower-level alternative.

## Completed: apparent place of date (chart layer)

Apparent-place corrections are implemented as the default chart-layer output in
the `pleiades-apparent` crate:

- Light-time iteration brings the source body back to its retarded position.
- Precession-to-date applies the J2000→equinox-of-date rotation (IAU 1976/1980
  luni-solar precession).
- Annual aberration applies the velocity-of-Earth correction in the ecliptic plane.
- Nutation-in-longitude (Δψ) shifts the ecliptic longitude to the true equinox
  of date.
- Gravitational light-deflection is omitted (documented).
- Release-grade bodies are supported; the existing J2000 corpus remains the
  geometric-core gate.
- The `pleiades-backend` apparentness policy summary reflects the chart-layer
  default; backends themselves remain mean-only and J2000 at the backend
  boundary.

## Remaining implementation work

- Implement topocentric body positions only with clear observer semantics,
  coordinate-frame handling, and tests.
- Keep native sidereal backend output unsupported unless a backend provides
  validated native behavior distinct from chart-layer sidereal conversion.
- The remaining motion scope for Phase 4 is **topocentric and native sidereal
  motion output** only — derived geometric speed (SP3), civil-time conversion,
  and apparent-place corrections are all implemented.

## Exit criteria

- Unsupported request modes produce structured, documented errors everywhere.
- Implemented request modes have validation fixtures, rustdoc/API examples, CLI
  coverage, backend metadata, and release-profile entries.
