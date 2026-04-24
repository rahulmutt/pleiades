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

- per-body comparison summaries and expected tolerance status sections in validation reports, so measured backend deltas are visible by body and checked against explicit interim evidence thresholds;
- managed Rust workspace and mandatory crate layout;
- typed domain vocabulary in `pleiades-types`;
- backend contract, metadata, errors, batch fallback, and composite routing in `pleiades-backend`;
- house and ayanamsa catalogs with descriptors and aliases;
- chart façade, sign/house/aspect summaries, sidereal conversion, and profile exports in `pleiades-core`;
- preliminary `pleiades-vsop87`, `pleiades-elp`, and `pleiades-data` crates, including finite-difference mean-motion estimates in the VSOP87 path and compact ELP Moon/lunar-point path, truncated IMCCE VSOP87B Earth, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, and Neptune coefficient slices for J2000-tested geocentric Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, and Neptune output, per-body VSOP87 source profiles that distinguish truncated source-backed paths from fallback mean-element paths in validation reports, explicit geocentric-only rejection of direct topocentric requests in the VSOP87/ELP placeholder backends, plus a small JPL Horizons derivative-fixture backend with exact lookup and linear interpolation proof of concept;
- compression model and sample artifact lookup;
- CLI, validation reports, backend matrix, release notes/checklists, bundle generation, and bundle verification;
- tests and doctests for current behavior.

Latest progress (2026-04-24): the JPL snapshot backend now exposes coarse leave-one-out interpolation quality samples derived from the checked-in fixture, and backend matrix release artifacts render those sparse-fixture error checks. The VSOP87 backend matrix entry now also prints canonical J2000 source-backed evidence for Sun-through-Neptune at the same public IMCCE reference points used by the regression tests, so the truncated coefficient paths carry visible measured deltas while complete generated tables remain pending.

Previous progress (2026-04-24): backend matrix release artifacts now report implementation status separately from body/catalog presence. Each implemented backend has an explicit status label and note (fixture reference, partial source-backed, preliminary algorithm, prototype artifact, or routing façade), and the compact matrix summary counts those statuses so release artifacts do not imply production accuracy merely because a backend advertises body coverage.

Active gaps:

- production VSOP87 coefficient handling and transformations;
- production lunar theory implementation and lunar point semantics;
- larger JPL-style reference corpus, interpolation validation, and documented tolerance envelopes beyond the current small fixture proof of concept;
- production Delta T conversion, TDB handling, apparent-place corrections, and validated frame-conversion error envelopes beyond the initial documented policy;
- source-backed evidence tables, reference-backed tolerance tables, and broader validation reports.

## Recommended next slice

Continue expanding the `pleiades-vsop87` source-data increment:

1. replace the checked-in truncated Earth/Mercury/Venus/Mars/Jupiter/Saturn/Uranus/Neptune coefficient slices with a reproducible generated complete-table path;
2. preserve Pluto as an explicitly documented non-VSOP87 special case until a Pluto-specific source path is selected;
3. extend validation output to report measured error for the source-backed body paths and body-source profiles.

Small preparatory backend-semantics increments have landed: supported planetary results now include deterministic central-difference mean-motion estimates, the compact ELP Moon/node path now exposes matching finite-difference mean-motion estimates, mean lunar apogee and mean lunar perigee are now explicitly supported as mean-only lunar points with finite-difference longitude speeds, direct observer-bearing requests to the geocentric VSOP87/ELP placeholders now fail explicitly instead of implying topocentric support, unsupported apparent requests are rejected rather than silently returning mean values, the initial time/observer policy is documented, caller-supplied UT1-to-TT/scale-offset helpers now exist in the shared type layer without introducing a built-in Delta T model, the Sun path now uses a truncated IMCCE VSOP87B Earth coefficient slice tested against a full-file J2000 golden value, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, and Neptune now use the same truncated VSOP87B coefficient path with full-file J2000 golden tests, and the validation backend matrix now surfaces per-body VSOP87 source profiles. A parallel release-safety increment tightened compatibility-profile verification so descriptor metadata and repeated exact labels are checked before bundle publication. Complete generated VSOP87 coefficient ingestion remains outstanding; Pluto remains a special case outside VSOP87's major-planet files; true lunar apogee/perigee remain deferred until a source-backed true-point model is selected.

## Constraints

- Keep all implementation pure Rust.
- Do not move astrology-domain logic into source-specific backends.
- Do not expand release claims until validation evidence exists.
- Keep changes small enough to review and test independently.
