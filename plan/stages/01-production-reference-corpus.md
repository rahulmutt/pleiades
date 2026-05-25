# Phase 1 — Production Reference Backend and Corpus

## Goal

Move from checked-in regression fixtures to production-grade public reference
inputs that can validate release claims and generate the 1500-2500 CE compressed
artifact.

## Current baseline

- JPL/Horizons snapshot and hold-out CSV fixtures are checked in and validated.
- Source, frame, time-scale, schema, checksum, redistribution posture, and
  exact J2000 fixture-exactness evidence are reported through CLI, validation,
  backend-matrix, and bundle surfaces; the release-facing body/date/channel
  posture now derives from validated corpus evidence rather than narrative
  prose.
- The current corpus covers useful boundary, bridge, selected-asteroid, lunar,
  and comparison slices, but remains sparse and fixture-oriented.

## Remaining implementation work

- Choose and implement the production source strategy:
  - a pure-Rust reader/parser for public JPL-style data products, or
  - a documented reproducible generation pipeline that produces a broad checked
    reference corpus from public inputs.
- Broaden source coverage for all release-claimed bodies, channels, frames, and
  epoch classes across 1500-2500 CE.
- Keep fitting/reference, independent hold-out, boundary-overlay,
  fixture-exactness, and provenance-only evidence separate in data and reports.
- Define minimum source density/cadence requirements per body class, including
  luminaries, major planets, Pluto policy, lunar points, baseline asteroids, and
  any custom/numbered body examples.

## Exit criteria

- A clean checkout can reproduce or verify the reference inputs from documented
  public sources.
- Corpus validation fails on missing bodies, epochs, channels, frames,
  apparentness policy, schema drift, or checksum/source-revision drift.
- Release-claimed body/channel/frame/date coverage is broad enough to feed the
  production artifact generator and hold-out comparisons.
