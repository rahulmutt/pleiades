# Status 1 — Current Execution Frontier

## Frontier

The active frontier is **Phase 1 — Production Ephemeris Accuracy**.

The repository has a complete architectural foundation and broad planning/reporting scaffolding. The next material risk is correctness: the backends that drive chart outputs, validation comparisons, and future compressed artifacts must move from preliminary/sample behavior to source-backed astronomical implementations with measured accuracy.

## Why this is first

Several downstream requirements depend on trusted ephemeris outputs:

- compressed artifacts must be generated from validated source data;
- release compatibility profiles must not imply unsupported accuracy;
- chart APIs need deterministic body positions across supported bodies and time ranges;
- validation reports need real error envelopes rather than placeholder comparisons.

## Current repo state summary

Completed foundations:

- per-body comparison summaries in validation reports, so measured backend deltas are visible by body as well as in aggregate;
- managed Rust workspace and mandatory crate layout;
- typed domain vocabulary in `pleiades-types`;
- backend contract, metadata, errors, batch fallback, and composite routing in `pleiades-backend`;
- house and ayanamsa catalogs with descriptors and aliases;
- chart façade, sign/house/aspect summaries, sidereal conversion, and profile exports in `pleiades-core`;
- preliminary `pleiades-vsop87`, `pleiades-elp`, and `pleiades-data` crates, plus a small JPL Horizons derivative-fixture backend with exact lookup and linear interpolation proof of concept;
- compression model and sample artifact lookup;
- CLI, validation reports, backend matrix, release notes/checklists, bundle generation, and bundle verification;
- tests and doctests for current behavior.

Active gaps:

- production VSOP87 coefficient handling and transformations;
- production lunar theory implementation and lunar point semantics;
- larger JPL-style reference corpus, interpolation validation, and documented tolerance envelopes beyond the current small fixture proof of concept;
- explicit Delta T, time-scale, apparent/mean, and frame conversion policies;
- reference-backed tolerance tables and validation reports.

## Recommended first slice

Start with a narrow `pleiades-vsop87` increment:

1. document the selected VSOP87 source and coefficient ingestion strategy;
2. implement one body/channel path with tests against a canonical reference epoch;
3. expose accurate metadata and unsupported-mode errors;
4. extend validation output to report that body's measured error.

This proves the source-data pattern before repeating it across all major planets.

## Constraints

- Keep all implementation pure Rust.
- Do not move astrology-domain logic into source-specific backends.
- Do not expand release claims until validation evidence exists.
- Keep changes small enough to review and test independently.
