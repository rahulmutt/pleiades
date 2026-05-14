# Status 1 — Current Execution Frontier

## Frontier

The active frontier is **Phase 1: Artifact accuracy and packaged-data production**, with **Phase 2: Reference/source corpus productionization** and **Phase 3: Body-model completion and claim boundaries** as dependencies.

Completed work such as workspace bootstrap, broad catalog scaffolding, report aliases, release-bundle rehearsal, artifact manifest completeness, body-class cadence summaries, span-cap summaries, and packaged lookup/decode benchmark surfaces is no longer listed as active implementation work.

## Why this frontier comes first

The specification requires a compressed 1500-2500 CE artifact with measured accuracy, deterministic generation, efficient random access, and explicit stored/derived/unsupported output semantics. The current artifact has the structure and reports needed to diagnose it, but validation still shows draft-grade errors far outside production thresholds.

Accuracy work should therefore precede any release claim broadening. If the current sparse snapshots cannot support production fitting, Phase 2 source/corpus work should happen before more artifact tuning.

Recent progress: the packaged-artifact fit-outlier diagnostics now preserve segment-span and family-sample-count context, and the validation report no longer double-prefixes the body-class span-cap summary.

## Immediate blockers

1. **Artifact model error** — current packaged-data comparison reports show very large longitude, latitude, and distance deviations; the draft artifact cannot be shipped as production ephemeris data.
2. **Source coverage** — checked-in JPL snapshots and boundary overlays are not yet a broad production corpus for all release bodies and epochs.
3. **Body claim boundaries** — Pluto, fuller lunar theory, lunar points, Ceres/Pallas/Juno/Vesta, and selected custom asteroids need source-backed validation or constrained/excluded release status.
4. **Release fail-closed behavior** — release gates must block artifact threshold failures, stale generated summaries, and overbroad compatibility/backend claims.

## Recommended next slice

Implement one artifact-accuracy slice that starts from a concrete outlier family:

- select a high-error body/channel/segment class from the current fit-outlier report;
- confirm whether the error is caused by sparse source cadence, coordinate conversion, distance-unit reconstruction, longitude wrapping, segment order, or quantization;
- add or expand source samples only when needed and document their provenance;
- update fitting/reconstruction logic and regression tests together;
- keep the artifact labeled draft until all advertised scopes pass published thresholds.

## Parallel safe work

- Decide and document the production source-ingestion strategy.
- Audit body release status for Pluto, lunar channels, and selected asteroids.
- Audit house/ayanamsa entries whose release claims are stronger than their evidence.
- Keep request-policy docs and structured unsupported errors synchronized.
- Harden release-bundle verification without changing feature claims.
