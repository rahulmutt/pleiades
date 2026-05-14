# Phase 1 — Artifact Accuracy and Packaged-Data Production

## Goal

Ship a production-quality compressed ephemeris artifact for 1500-2500 CE that satisfies `spec/data-compression.md`, `requirements.md` FR-9, and the packaged-backend responsibilities in `spec/backends.md`.

## Starting point

The codec, artifact structures, deterministic regeneration path, checksums, generation manifest, output-support profile, body-class cadence summary, channel fit-outlier reports, boundary checks, lookup benchmarks, batch-lookup benchmarks, and decode benchmarks already exist. The checked-in artifact is still a draft fixture: current validation reports show very large longitude, latitude, and distance errors versus the comparison corpus, so it must not be treated as production ephemeris data.

## Implementation goals

- Rework fitting and reconstruction until body/channel errors meet published thresholds for the advertised scope.
- Use validated public inputs from Phase 2 rather than ad hoc sparse samples for production fitting.
- Define body-class/channel thresholds before promoting the artifact beyond draft status.
- Decide whether polynomial order, Chebyshev fitting, residual tables, per-body cadence, or channel-specific storage changes are required.
- Keep the artifact profile explicit about stored, derived, unsupported, and approximated outputs.
- Preserve deterministic generation parameters, normalized-intermediate checksums, artifact checksums, and encoded-size accounting.
- Make artifact validation fail on threshold violations, capability drift, checksum drift, malformed manifests, or unsupported request shapes.
- Keep performance benchmarks visible, but treat accuracy and reproducibility as release blockers.

## Completion criteria

Phase 1 is complete when a clean checkout can regenerate the packaged artifact and pass published reference/hold-out thresholds for the advertised bodies, channels, request modes, and 1500-2500 CE range.

## Out of scope

- Broadening body claims without Phase 2/3 evidence.
- Implementing apparent, topocentric, UTC/Delta-T, or native-sidereal behavior.
- Promoting catalog compatibility claims.
