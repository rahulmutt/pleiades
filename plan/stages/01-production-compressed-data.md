# Phase 1 — Artifact Accuracy and Packaged-Data Production

## Goal

Ship a production-quality compressed ephemeris artifact for 1500-2500 CE that satisfies `spec/data-compression.md`, `requirements.md` FR-9, and the packaged-backend responsibilities in `spec/backends.md`.

## Starting point

The codec, artifact structures, deterministic regeneration path, checksums, generation manifest, output-support profile, body-class cadence summary, bundled-body cadence summary, channel fit-outlier reports, boundary checks, lookup benchmarks, batch-lookup benchmarks, and decode benchmarks already exist. The packaged artifact profile validation now fails closed if any built-in output would remain unlisted, so profile drift cannot silently omit a requested output class. The generator now applies a six-point Chebyshev-Lobatto fit plus measured-fit comparison against fallback reconstruction on short spans, can attach residual-correction channels across the bundled body set when they improve the measured fit, now explores residual-channel combinations rather than only greedy single-channel additions, and now uses a denser residual sample lattice for luminaries and custom bodies so the Moon-adjacent and long-window custom-body correction search is body-specific instead of flat; the checked-in fixture is regenerated from that path. Fit-outlier diagnostics now use a denser report lattice for body/channel summaries without changing the calibrated threshold lattice, and the generator now applies body-specific validation/outlier sampling for luminaries, lunar points, selected asteroids, Pluto, and custom bodies. The current generator also gives luminaries, lunar points, Pluto, selected asteroids, and custom bodies a denser fit-candidate lattice before the legacy fallback ladder on those sensitive spans, now including 10-point and 12-point Chebyshev-Lobatto options for the dense bodies, and the best dense candidate now wins before fallback. The normalized-intermediate summary now carries a deterministic checksum, and the target-threshold posture is now modeled as a typed production-ready state so the finalized production-threshold policy stays explicit instead of being hidden in raw text. The storage/reconstruction and production-profile summary surfaces now route through explicit validated wrappers before rendering, so release-facing artifact posture lines fail closed when summary validation drifts. The validation and artifact smoke-report paths now cache expensive report objects and use reduced timing subsets for tractable bundle verification under the test harness, and the regeneration helper now caches the deterministic rebuilt artifact in-process so repeated validation and report invocations avoid rebuilding the full artifact.

## Implementation goals

- Rework fitting and reconstruction until body/channel errors meet published thresholds for the advertised scope.
- Use validated public inputs from Phase 2 rather than ad hoc sparse samples for production fitting.
- Keep the finalized body-class/channel thresholds synchronized with source-fit and independent hold-out validation.
- Decide whether polynomial order, Chebyshev fitting, residual tables, per-body cadence, or channel-specific storage changes are required.
- Keep the artifact profile explicit about stored, derived, unsupported, and approximated outputs.
- Preserve deterministic generation parameters, normalized-intermediate checksums, artifact checksums, and encoded-size accounting.
- Make artifact validation fail on threshold violations, capability drift, checksum drift, malformed manifests, or unsupported request shapes.
- Keep performance benchmarks visible, but treat accuracy, reproducibility, and report tractability as release blockers.

## Completion criteria

Phase 1 is complete when a clean checkout can regenerate the packaged artifact and pass published reference/hold-out thresholds for the advertised bodies, channels, request modes, and 1500-2500 CE range.

## Out of scope

- Broadening body claims without Phase 2/3 evidence.
- Implementing apparent, topocentric, UTC/Delta-T, or native-sidereal behavior.
- Promoting catalog compatibility claims.
