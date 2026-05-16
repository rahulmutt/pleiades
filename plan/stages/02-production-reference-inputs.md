# Phase 2 — Reference/Source Corpus Productionization

## Goal

Provide documented public source inputs broad enough for backend validation, release-grade body claims, and production artifact fitting.

## Starting point

`pleiades-jpl` includes checked-in JPL Horizons snapshots, hold-out fixtures, provenance summaries, batch/single parity checks, source-window summaries, selected asteroid samples, selected-asteroid request corpora, and production-generation manifest reports. This is regression-quality evidence, but not yet a broad production reader or generated corpus covering all advertised body/date/channel claims.

## Implementation goals

- The production source strategy is documented as a hybrid fixture corpus: checked-in reference and hold-out fixtures plus a separate generation-input path.
- Record source provenance, frame, time scale, columns/channels, source revision, generation command, checksums, and redistribution posture.
- Completed: the production-generation source summary now records the generation command alongside the checked-in CSV provenance, frame, time scale, schema, and checksum markers.
- Expand coverage only where it supports release claims or artifact fitting.
- Keep reference, hold-out, boundary-overlay, fixture-exactness, and provenance-only evidence classes separate.
- Publish body-class tolerance reports and empirical error envelopes for release-claimed backend paths.
- Preserve small golden fixtures for fast regression tests even if broader ingestion lands.
- Ensure all ingestion/build steps remain pure Rust and deterministic.

## Completion criteria

Phase 2 is complete when validation and artifact-generation inputs cover the advertised release body set and range with documented provenance, reproducible generation, checksums, and current tolerance reports.

## Out of scope

- Shipping the compressed artifact; that belongs to Phase 1.
- Deciding final body release status; that belongs to Phase 3.
- Promoting house or ayanamsa catalog claims; that belongs to Phase 5.
