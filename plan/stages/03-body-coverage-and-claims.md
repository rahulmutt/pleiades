# Phase 3 — Body-Model Completion and Claim Boundaries

## Goal

Make release body coverage truthful for `requirements.md` FR-1/FR-2/FR-7/FR-8: every advertised body or point must be backed by source evidence, bounded validation, or an explicit constrained/approximate/unsupported status.

## Starting point

The workspace can compute Sun through Neptune with VSOP87B source-backed tables, has an approximate Pluto fallback, provides a compact Meeus-style lunar baseline for the Moon and selected lunar points, and includes JPL snapshot evidence for major bodies and selected asteroids. Baseline asteroid support and packaged-data body coverage are still narrower than the end-state requirements.

## Implementation goals

- Resolve Pluto's production posture: source-backed implementation, artifact-backed implementation, or constrained/excluded release claim.
- Decide whether the first production release needs fuller ELP-style lunar coefficient support; if so, implement pure-Rust ingestion/evaluation with provenance and tests.
- Keep current compact lunar support documented as a baseline unless stronger lunar theory evidence lands.
- Align mean/true node and apogee/perigee claims with implemented formulas and validation evidence.
- Promote Ceres, Pallas, Juno, Vesta, and other selected asteroids only when source coverage and tolerances support them.
- Keep custom/numbered body identifiers extensible without implying unsupported bodies are available from every backend.
- Update backend metadata, release compatibility profiles, docs, and validation reports together whenever a body claim changes.

## Completion criteria

Phase 3 is complete when release summaries, backend matrices, validation reports, and compatibility profiles agree on which bodies are exact/source-backed, approximate, interpolated, constrained, or unsupported.

## Out of scope

- General artifact fitting work, except where needed to support a body claim.
- House and ayanamsa catalog audits.
- Advanced request semantics such as apparent/topocentric behavior.
