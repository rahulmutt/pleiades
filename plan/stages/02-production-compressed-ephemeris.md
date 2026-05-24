# Phase 2 — Release-Grade Compressed Ephemeris

## Goal

Promote `pleiades-data` from a draft reproducibility fixture to a release-grade
1500-2500 CE packaged backend.

## Current baseline

- `pleiades-compression` defines artifact structures and codec helpers.
- `pleiades-data` decodes a checked-in stage-5 draft artifact.
- Artifact profile, output-support, checksum, boundary, benchmark, regeneration,
  and request-policy summaries exist.
- Current model-error envelopes still exceed production tolerance for many
  body/channel combinations.

## Remaining implementation work

- Rebase artifact generation on the Phase 1 production reference and hold-out
  inputs.
- Define published accuracy thresholds by body class and channel, including
  longitude, latitude, distance, and supported speed/motion outputs.
- Improve fitting, interpolation, residual, quantization, and reconstruction
  logic until reference and hold-out errors pass thresholds.
- Keep stored, derived, approximated, and unsupported output channels explicit in
  the artifact profile.
- Keep apparent, topocentric, native sidereal, civil-time, and unsupported motion
  outputs rejected unless implemented and validated.
- Track encoded size, decode latency, single lookup latency, batch throughput,
  and chart-style workload performance.
- Ensure deterministic regeneration and byte/checksum verification from a clean
  checkout.

## Exit criteria

- The packaged artifact covers the advertised 1500-2500 CE body/channel profile.
- Reference and hold-out comparisons pass the published thresholds.
- Artifact manifests, checksums, generation provenance, output-support profile,
  and benchmarks are current and release-bundle verified.
