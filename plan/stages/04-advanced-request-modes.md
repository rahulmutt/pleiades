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
- Policy summaries now reflect topocentric as a supported chart-layer opt-in
  correction; native sidereal backend output remains the only unsupported Phase 4
  mode.
- Motion/speed output, built-in civil-time UTC/UT1 → TT/TDB conversion,
  apparent-place-of-date, and chart-layer topocentric positions are implemented
  and gated (see git history and the `pleiades-time`/`pleiades-apparent` crates).

## Remaining implementation work

- Keep native sidereal backend output unsupported unless a backend provides
  validated native behavior distinct from chart-layer sidereal conversion
  (deliberate non-goal; this is the only remaining Phase 4 item).

## Exit criteria

- Unsupported request modes produce structured, documented errors everywhere.
- Implemented request modes have validation fixtures, rustdoc/API examples, CLI
  coverage, backend metadata, and release-profile entries.
