# Phase 2 — Production Compressed Ephemeris

## Goal

Ship a deterministic packaged-data backend for 1500-2500 CE whose measured errors, output classifications, random-access performance, and artifact provenance satisfy `spec/data-compression.md`.

## Starting point

`pleiades-compression` and `pleiades-data` implement an artifact format, decoder, draft fixture, profile metadata, checksum verification, reports, and benchmarks. Current comparison envelopes still exceed production expectations, so the artifact remains draft-grade.

## Implementation goals

- Generate from the Phase 1 production source corpus, not from ad hoc or under-covered fixtures.
- Finalize body/channel-specific accuracy thresholds and enforce them in validation gates.
- Improve fitting/reconstruction where needed: Chebyshev or other polynomial segments, residual tables, body-specific segmentation, speed derivation policy, and channel-specific handling.
- Keep stored, derived, approximated, and unsupported output classifications explicit and fail-closed.
- Measure artifact size, decode latency, random lookup latency, batch lookup throughput, and chart-style workloads.
- Preserve deterministic regeneration, manifest, checksum, and source-provenance sidecars.

## Completion criteria

- `pleiades-data` passes published thresholds for every advertised body/channel in 1500-2500 CE.
- Release reports include measured error envelopes against reference and hold-out corpora.
- Artifact regeneration is deterministic from documented public inputs.
