# Stage 4 — Reference Backend and Validation

## Goal
Add a stronger source-backed backend and formal validation tooling so accuracy claims and future artifact generation are grounded in reproducible evidence.

## Why this stage comes fourth
Once the project already has a useful algorithmic chart path, reference data can be introduced to validate, calibrate, and extend it instead of becoming the only route to a usable product.

## Primary deliverables

### `pleiades-jpl`
- pure-Rust parsing/reading of selected public JPL ephemeris inputs or derivative public products
- metadata on source provenance and supported range
- support for selected asteroids where source data and scope justify it

### `pleiades-validate`
- compare-backends command
- benchmark command
- report generation command
- artifact-validation scaffolding for later packaged data

### Validation assets
- canonical test epochs and sample charts
- cross-backend error summaries
- documented backend capability matrices
- empirical accuracy notes for implemented bodies and features

## Workable state at end of stage
The project remains usable for chart generation, but now also has an evidence-backed path for comparing implementations, detecting regressions, and generating trustworthy downstream artifacts.

## Suggested implementation slices

1. Implement a narrow but solid JPL-backed slice for a small body set before broadening coverage.
2. Define validation report formats, fixture layout, and storage conventions early so later reports stay comparable.
3. Add `pleiades-validate` commands for compare-backends and benchmark on the narrow slice first.
4. Compare VSOP87/ELP outputs against reference values over representative date ranges and preserve discovered regressions.
5. Expand body coverage or asteroid support only when the validation workflow is already proving useful.
6. Integrate validation commands into CI or release checks where feasible.

This stage should improve trustworthiness in layers: first provenance, then comparisons, then breadth.

## Exit criteria

- at least one source-backed backend works in pure Rust
- validation reports can be reproduced from documented inputs
- capability matrices exist for all implemented backends
- regression cases are archived for previously found issues

## Progress update

Stage 4 validation has started as of 2026-04-22.

- [x] `pleiades-jpl` now ships a narrow JPL Horizons reference snapshot backend at J2000.0 with explicit provenance metadata and a checked-in source data file.
- [x] `pleiades-validate` now exposes compare-backends, benchmark, and report commands that operate on the snapshot corpus.
- [x] Validation reports render backend capability matrices, corpus metadata, per-body deltas, and a dedicated notable-regressions section against the JPL snapshot.
- [x] Validation reports now distinguish the multi-epoch JPL comparison corpus from a representative 1500-2500 benchmark corpus, so the time-window coverage for stage-4 comparison and benchmarking is explicit.
- [x] Archived regression cases are now preserved in the validation report so previously observed deltas remain visible in the test corpus.
- [x] Selected asteroid support is now present in the JPL snapshot backend, and the validation corpus intentionally remains on the planetary comparison subset so existing report baselines stay stable.
- [x] The JPL snapshot loader now reports malformed rows as structured load errors instead of panicking, so reference data issues fail explicitly and can be surfaced by validation tooling.

## Risks to avoid

- depending on opaque or legally unclear reference material
- treating one validation snapshot as permanent truth without reproducibility
- broadening coverage faster than validation can support
