# Phase 1 — Production Compressed Data

## Goal

Turn the current stage-5 draft packaged-data fixture into a production-quality 1500-2500 CE compressed artifact that satisfies `spec/data-compression.md` and `requirements.md` FR-9/NFR-4.

## Starting point

The workspace already has artifact structures, codec roundtrips, checksums, residual support, manifest summaries, regeneration helpers, benchmark/report surfaces, and a draft artifact. The draft artifact is not production-grade: validation reports show large fit errors and threshold violations.

## Implementation goals

- Replace the draft fitting approach with a production strategy suitable for Sun, Moon, planets, and selected asteroid coverage.
- Define body-class and channel-specific target thresholds before claiming success.
- Generate normalized intermediates and compressed artifacts from validated public inputs with deterministic parameters.
- Keep stored, derived, and unsupported outputs explicit in the artifact profile.
- Improve segment selection, polynomial/Chebyshev order, quantization, and residual correction until measured errors fit the published profile.
- Benchmark lookup latency, batch throughput, decode cost, artifact size, and chart-style packaged-data use.
- Make artifact validation fail when fit errors, checksums, manifests, or advertised capabilities drift.

## Completion criteria

Phase 1 is complete when a clean checkout can regenerate the packaged artifact, verify byte/checksum/profile metadata, and pass published error thresholds over reference and hold-out corpora for the advertised body set and 1500-2500 CE range.

## Out of scope

- Broadening compatibility catalogs.
- Implementing apparent/topocentric/native-sidereal modes.
- Claiming bodies not covered by the artifact profile and validation corpus.
