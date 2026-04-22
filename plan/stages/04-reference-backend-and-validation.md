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

## Suggested tasks

1. Implement a narrow but solid JPL-backed slice before broadening coverage.
2. Define validation report formats and storage conventions.
3. Compare VSOP87/ELP outputs against reference values over representative date ranges.
4. Expand asteroid support only when the validation story is clear.
5. Integrate validation commands into CI or release checks where feasible.

## Exit criteria

- at least one source-backed backend works in pure Rust
- validation reports can be reproduced from documented inputs
- capability matrices exist for all implemented backends
- regression cases are archived for previously found issues

## Risks to avoid

- depending on opaque or legally unclear reference material
- treating one validation snapshot as permanent truth without reproducibility
- broadening coverage faster than validation can support
