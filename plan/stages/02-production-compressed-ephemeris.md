# Phase 2 — Release-Grade Compressed Ephemeris

## Goal

Promote `pleiades-data` from a draft reproducibility fixture to a release-grade
1900–2100 CE packaged backend (first release; 1600–2600 CE is documented as a
future expansion, available opt-in via `generate-artifact`, not yet gated).

## Current baseline (after SP1 + SP2)

- `pleiades-compression` defines artifact structures and codec helpers.
- `pleiades-data` decodes a checked-in ARTIFACT_VERSION 7 artifact.
- Generation is now rebased on a dense, de440-backed within-span fit: each of
  the 10 major bodies (Sun, Moon, Mercury-Pluto) is fit by least-squares
  polynomials sampled densely from de440 within each per-body segment span,
  kernel-gated behind `PLEIADES_DE_KERNEL` (same gate as corpus_regen). The
  Phase 1 corpus is validation-only; de440 is the dense generation source.
- ARTIFACT_VERSION bumped 4→5 (per-body segment count widened u16→u32 to hold
  the dense Moon's ~91k segments), then 5→6 (per-body `StoredFrame` byte added
  to the codec). Generation is byte-deterministic, verified by a kernel-gated
  reproduce test (`crates/pleiades-data/tests/artifact_regen.rs`).
- The constrained asteroid (433-Eros) is re-derived from the committed reference
  snapshot (absent from de440 and sb441-n16), constrained to 1900-2100.
- **SP2 (done): heliocentric-planet reframe.** The eight planets (Mercury–Pluto)
  are now stored heliocentrically; the Sun remains geocentric. At lookup the
  runtime reconstructs geocentric ecliptic via `P_geo = P_helio + S_geo`
  (Cartesian addition in ecliptic-of-date; no obliquity rotation required). Moon
  and Eros remain geocentric. The Sun-presence fail-closed invariant is enforced
  at artifact-construction time. See `spec/data-compression.md` §Per-Body
  Storage Frame for the full invariant specification.
- A per-body accuracy baseline vs the de440-derived hold-out is committed in
  `crates/pleiades-data/src/accuracy_baseline.rs` and exposed via
  `packaged-artifact-accuracy-baseline-summary`. After SP2: all bodies sub-arcsec
  (Uranus ~0.0036″, Neptune ~0.0020″, Pluto ~0.0018″, Saturn ~0.0009″,
  Jupiter ~0.0004″; inner bodies + Sun + Moon remain sub-arcsec). SP1 measured
  accuracy; SP2 delivered the reframe.
- Artifact profile, output-support, checksum, boundary, benchmark, regeneration,
  and request-policy summaries exist.
- Size/perf baseline (budgeted — size hard-gated ≤ 12 MB, latency tracked): ~10.0 MB
  (1900–2100), decode ~260 ms, single lookup ~3.3 ms.
- **SP3 (done): thresholds + budgets + motion-derived.** Published per-body-class accuracy
  ceilings enforced as CI gates (see `crates/pleiades-data/src/thresholds.rs`). Hard size gate
  (≤ 12,000,000 bytes). Latency targets tracked in `PACKAGED_BUDGETS`; opt-in enforcement via
  `PLEIADES_ENFORCE_LATENCY`. Motion output (`SpeedPolicy::FittedDerivative`, `Motion = Derived`)
  implemented and measured. ARTIFACT_VERSION is now 7.

## Remaining implementation work

### SP2 — Accuracy tuning (done)

- The heliocentric-planet reframe reduced outer-planet longitude errors from
  the SP1 baseline (~192″ Uranus, ~109″ Neptune, ~62″ Pluto, ~9.5″ Saturn, ~1.5″ Jupiter)
  to sub-arcsec across all bodies. See accuracy numbers above.

### SP3 — Thresholds, size, and latency budgets (done)

- Published accuracy thresholds by body class and channel (longitude, latitude,
  distance, lon/lat speed, radial speed) defined and enforced as CI gate.
- Size budget enforced (≤ 12,000,000 bytes, hard-gated). Latency budgets tracked
  in `PACKAGED_BUDGETS` (not hard CI gate by default; opt-in via
  `PLEIADES_ENFORCE_LATENCY`).
- Motion output (`SpeedPolicy::FittedDerivative`, `Motion = Derived`) implemented,
  measured, and gated against published speed ceilings.

### Ongoing

- Keep stored, derived, approximated, and unsupported output channels explicit in
  the artifact profile.
- Keep apparent, topocentric, native sidereal, civil-time, and unsupported motion
  outputs rejected unless implemented and validated.
- Ensure deterministic regeneration and byte/checksum verification from a clean
  checkout.

## Exit criteria

- The packaged artifact covers the advertised 1900–2100 CE body/channel profile
  (first release; 1600–2600 CE expansion is documented future work, opt-in via
  `generate-artifact`, not gated for this phase).
- Reference and hold-out comparisons pass the published per-body-class accuracy
  ceilings from `thresholds.rs`.
- Encoded artifact size is within the hard-gated budget (≤ 12,000,000 bytes).
- Latency targets are tracked; hard enforcement is opt-in.
- Motion output is `Motion = Derived` (FittedDerivative); speed ceilings are gated.
- Artifact manifests, checksums, generation provenance, output-support profile,
  and benchmarks are current and release-bundle verified.

**Phase 2 is complete** (SP1 + SP2 + SP3 all done).
