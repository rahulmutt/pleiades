# Phase 1 — Production Reference/Source Corpus

## Goal

Provide public, documented, deterministic source inputs broad enough to support release body claims, backend validation, hold-out testing, and compressed-artifact generation.

## Starting point

The repository has checked-in JPL Horizons snapshots, comparison fixtures, selected-asteroid slices, and hold-out rows. They are valuable regression evidence, but are still described as repository-checked fixtures rather than a broad production corpus or general source reader.

## Implementation goals

- Choose the production source strategy: broader checked-in public fixtures, a pure-Rust reader/parser for public ephemeris files, or a documented hybrid.
- Record provenance, source revisions, license/redistribution posture, frame, time scale, schema, generation command, checksums, and evidence class for every corpus member.
- Cover every release-claimed body/channel/frame over the advertised date range with separate reference and hold-out sets.
- Keep boundary overlays, fixture-exactness samples, provenance-only rows, and validation rows separate in data and reports.
- Make corpus expansion reproducible from public inputs without network access during normal tests.

## Completion criteria

- Validation commands can identify the corpus version and reproduce checksums from a clean checkout.
- Hold-out samples exercise the same release bodies and channels as the fitting corpus without being consumed by fitting.
- Release profiles and backend matrices cannot claim broader body/date/channel support than the corpus validates.
