# Phase 2 — Release-Grade Compressed Ephemeris

## Goal

Promote `pleiades-data` from a draft reproducibility fixture to a release-grade
1600-2600 CE packaged backend.

## Current baseline (after SP1)

- `pleiades-compression` defines artifact structures and codec helpers.
- `pleiades-data` decodes a checked-in ARTIFACT_VERSION 5 draft artifact.
- Generation is now rebased on a dense, de440-backed within-span fit: each of
  the 10 major bodies (Sun, Moon, Mercury-Pluto) is fit by least-squares
  polynomials sampled densely from de440 within each per-body segment span,
  kernel-gated behind `PLEIADES_DE_KERNEL` (same gate as corpus_regen). The
  Phase 1 corpus is validation-only; de440 is the dense generation source.
- ARTIFACT_VERSION bumped 4→5 (per-body segment count widened u16→u32 to hold
  the dense Moon's ~91k segments); regenerated artifact is ~201,873 segments /
  ~49.78 MB. Generation is byte-deterministic, verified by a kernel-gated
  reproduce test (`crates/pleiades-data/tests/artifact_regen.rs`).
- The constrained asteroid (433-Eros) is re-derived from the committed reference
  snapshot (absent from de440 and sb441-n16), constrained to 1900-2100.
- A per-body accuracy baseline vs the de440-derived hold-out is committed in
  `crates/pleiades-data/src/accuracy_baseline.rs` and exposed via
  `packaged-artifact-accuracy-baseline-summary`. Measured: inner bodies + Sun +
  Moon are sub-arcsec; outer planets are draft-level (Uranus ~156″, Neptune
  ~90″, Pluto ~62″, Saturn ~11″, Jupiter ~1.7″). SP1 measured accuracy; it did
  not enforce thresholds or tune spans/degrees.
- Artifact profile, output-support, checksum, boundary, benchmark, regeneration,
  and request-policy summaries exist.
- Draft size/perf baseline (measured, not budgeted): ~49.78 MB, decode ~197 ms,
  single lookup ~1.7 ms.
- The artifact remains explicitly draft-grade.

## Remaining implementation work

### SP2 — Accuracy tuning

- Tune per-body segment spans and polynomial degrees against the measured SP1
  accuracy baseline, prioritising outer planets (Uranus ~156″, Neptune ~90″,
  Pluto ~62″, Saturn ~11″, Jupiter ~1.7″).
- Improve fitting, interpolation, residual, quantization, and reconstruction
  logic until reference and hold-out errors approach target thresholds.

### SP3 — Thresholds, size, and latency budgets

- Define and enforce published accuracy thresholds by body class and channel,
  including longitude, latitude, distance, and supported speed/motion outputs.
- Define and track size and latency budgets (encoded size, decode latency, single
  lookup latency, batch throughput, chart-style workload performance).

### Ongoing

- Keep stored, derived, approximated, and unsupported output channels explicit in
  the artifact profile.
- Keep apparent, topocentric, native sidereal, civil-time, and unsupported motion
  outputs rejected unless implemented and validated.
- Ensure deterministic regeneration and byte/checksum verification from a clean
  checkout.

## Exit criteria

- The packaged artifact covers the advertised 1600-2600 CE body/channel profile.
- Reference and hold-out comparisons pass the published thresholds.
- Artifact manifests, checksums, generation provenance, output-support profile,
  and benchmarks are current and release-bundle verified.
