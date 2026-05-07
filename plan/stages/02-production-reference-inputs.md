# Phase 2 — Production Reference Inputs

## Goal

Provide a pure-Rust source/reference path broad enough to support backend validation, release-grade body claims, and Phase 1 artifact generation.

## Starting point

`pleiades-jpl` currently uses checked-in JPL Horizons snapshot and hold-out fixtures with provenance and report surfaces. This is useful validation evidence, but it is not yet a broad production reader/corpus.

## Implementation goals

- Decide whether `pleiades-jpl` should parse public JPL files directly, ingest a documented derived public dataset, or maintain a generated fixture corpus with reproducible provenance.
- Expand source coverage to the bodies, epochs, frames, and channels required by release claims and artifact fitting.
- Keep reference, hold-out, fixture-exactness, and provenance-only evidence separated in reports.
- Publish body-class tolerances and empirical error summaries for each release-claimed backend path.
- Resolve release posture for Pluto: either validate a source-backed path or keep it explicitly approximate/excluded from release-grade claims.
- Decide whether fuller ELP-style lunar coefficient support is required for the first production release; if so, implement pure-Rust ingestion/evaluation with provenance and tests.
- Preserve small golden fixtures for regression tests even if broader source readers land.

## Completion criteria

Phase 2 is complete when validation and artifact-generation inputs cover the advertised release body set and range with documented public provenance, deterministic ingestion, and current tolerance reports.

## Out of scope

- Shipping compressed artifacts; that belongs to Phase 1.
- Changing high-level astrology-domain catalog claims; that belongs to Phase 4.
