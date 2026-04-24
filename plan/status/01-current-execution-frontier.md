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
- preliminary `pleiades-vsop87`, `pleiades-elp`, and `pleiades-data` crates, including finite-difference mean-motion estimates in the VSOP87 path, truncated IMCCE VSOP87B Earth, Mercury, and Venus coefficient slices for J2000-tested geocentric Sun, Mercury, and Venus output, explicit geocentric-only rejection of direct topocentric requests in the VSOP87/ELP placeholder backends, plus a small JPL Horizons derivative-fixture backend with exact lookup and linear interpolation proof of concept;
- compression model and sample artifact lookup;
- CLI, validation reports, backend matrix, release notes/checklists, bundle generation, and bundle verification;
- tests and doctests for current behavior.

Active gaps:

- production VSOP87 coefficient handling and transformations;
- production lunar theory implementation and lunar point semantics;
- larger JPL-style reference corpus, interpolation validation, and documented tolerance envelopes beyond the current small fixture proof of concept;
- production Delta T conversion, TDB handling, apparent-place corrections, and validated frame-conversion error envelopes beyond the initial documented policy;
- reference-backed tolerance tables and validation reports.

## Recommended next slice

Continue expanding the narrow `pleiades-vsop87` source-data increment:

1. replace the checked-in truncated Earth/Mercury/Venus coefficient slices with a reproducible generated complete-table path;
2. extend the coefficient representation to Mars and the outer major planets;
3. expose more granular metadata for mixed source-backed versus fallback element paths beyond the current provenance notes;
4. extend validation output to report measured error for the source-backed body paths.

Small preparatory backend-semantics increments have landed: supported planetary results now include deterministic central-difference mean-motion estimates, direct observer-bearing requests to the geocentric VSOP87/ELP placeholders now fail explicitly instead of implying topocentric support, unsupported apparent requests are rejected rather than silently returning mean values, the initial time/observer policy is documented, the Sun path now uses a truncated IMCCE VSOP87B Earth coefficient slice tested against a full-file J2000 golden value, and Mercury and Venus now use the same truncated VSOP87B coefficient path with full-file J2000 golden tests. Complete VSOP87 coefficient ingestion remains outstanding.

## Constraints

- Keep all implementation pure Rust.
- Do not move astrology-domain logic into source-specific backends.
- Do not expand release claims until validation evidence exists.
- Keep changes small enough to review and test independently.
