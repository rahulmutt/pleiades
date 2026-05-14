# Status 1 — Current Execution Frontier

## Frontier

The active frontier is **Phase 1: Artifact accuracy and packaged-data production**, with **Phase 2: Reference/source corpus productionization** and **Phase 3: Body-model completion and claim boundaries** as dependencies.

Completed work such as workspace bootstrap, broad catalog scaffolding, report aliases, release-bundle rehearsal, artifact manifest completeness, body-class cadence summaries, span-cap summaries, and packaged lookup/decode benchmark surfaces is no longer listed as active implementation work.

## Why this frontier comes first

The specification requires a compressed 1500-2500 CE artifact with measured accuracy, deterministic generation, efficient random access, and explicit stored/derived/unsupported output semantics. The current artifact has the structure and reports needed to diagnose it, and the draft path now passes the current calibrated thresholds, but release-grade thresholds and corpus coverage are still not finalized.

Accuracy work should therefore precede any release claim broadening. Phase 2 source/corpus work still needs to lock down the production threshold policy before broader artifact claims move forward.

Recent progress: the packaged-artifact fit-outlier diagnostics now preserve segment-span and family-sample-count context, the validation report no longer double-prefixes the body-class span-cap summary, the packaged-artifact generator now applies measured-fit subdivision with an error-aware span-limited candidate-versus-fallback choice, the checked-in fixture has been regenerated, artifact-derived fit samples are cached so summary reports stay tractable, the target-threshold posture is now carried by a typed draft/production-ready state while still rendering calibrated fit envelope recorded with production thresholds pending, and the JPL source posture is now documented as a hybrid fixture corpus.

## Immediate blockers

1. **Production thresholds** — the draft artifact now passes the current calibrated fit thresholds, but release-grade body/channel thresholds still need to be defined and validated before the artifact can be promoted.
2. **Source coverage** — checked-in JPL snapshots and boundary overlays are not yet a broad production corpus for all release bodies and epochs.
3. **Body claim boundaries** — Pluto, fuller lunar theory, lunar points, Ceres/Pallas/Juno/Vesta, and selected custom asteroids need source-backed validation or constrained/excluded release status.
4. **Release fail-closed behavior** — release gates must block artifact threshold failures, stale generated summaries, and overbroad compatibility/backend claims.

## Recommended next slice

Promote the draft threshold posture to a production threshold policy once the Phase 2 corpus is ready:

- define the body-class/channel thresholds that will gate release claims;
- require both source-fit and hold-out validation before expanding the advertised scope;
- keep the artifact labeled draft until the production thresholds and source corpus are aligned;
- continue using the cached fit summaries and measured-fit subdivision path to keep report generation tractable.

## Parallel safe work

- Expand the documented hybrid source corpus with broader coverage and provenance.
- Promote the measured-fit draft posture into release-grade thresholds once the Phase 2 source corpus is ready.
- Audit body release status for Pluto, lunar channels, and selected asteroids.
- Audit house/ayanamsa entries whose release claims are stronger than their evidence.
- Keep request-policy docs and structured unsupported errors synchronized.
- Harden release-bundle verification without changing feature claims.
