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
  prose, and the checked-in JPL-style snapshots now have reusable pure-Rust CSV
  parsing entry points for their manifest and row data.
- The current corpus covers useful boundary, bridge, selected-asteroid, lunar,
  and comparison slices, but remains sparse and fixture-oriented.

## Remaining implementation work

- Broaden the production source strategy on top of the exposed pure-Rust CSV
  parsing path:
  - ingest broader public JPL-style data products, or
  - extend the documented reproducible generation pipeline to produce a broader
    checked reference corpus from public inputs.
- Broaden source coverage for all release-claimed bodies, channels, frames, and
  epoch classes across 1500-2500 CE.
- Keep fitting/reference, independent hold-out, boundary-overlay,
  fixture-exactness, and provenance-only evidence separate in data and reports.

## Exit criteria

- A clean checkout can reproduce or verify the reference inputs from documented
  public sources.
- Corpus validation fails on missing bodies, epochs, channels, frames,
  apparentness policy, schema drift, or checksum/source-revision drift.
- Release-claimed body/channel/frame/date coverage is broad enough to feed the
  production artifact generator and hold-out comparisons.
