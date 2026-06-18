# SP1 — Dense de440-Backed Generation Source + Accuracy Baseline

Date: 2026-06-18
Status: Approved design (pre-implementation)
Context: First sub-project of the "fitting-grade artifact" effort that the halted
2026-06-17 corpus-rebase slice exposed as missing.

## Summary

Give the packaged-artifact generator a genuinely fitting-grade reference source
by fitting each segment against the de440 ephemeris (`SpkBackend`) sampled
densely *within* each segment's time span, instead of against the sparse
committed reference points. Regenerate the (still-draft) artifact from this
source and produce a per-body **accuracy baseline** measured against the
committed hold-out corpus. The baseline quantifies how good the current fit
model is and sizes the follow-on work.

## Why this exists (root cause)

The 2026-06-17 slice tried to rebase generation onto the committed Phase 1
corpus and failed: that corpus samples roughly **two points per orbital period
for every body** (Moon 30 d vs 27 d period; Sun 180 d; Mercury 60 d), a
*validation* sampling strategy unusable for *fitting*. Investigation also showed
the prior narrow `reference_snapshot` was no denser (≈30 points per fast body
across ~1500–2500 CE). **No fitting-grade generation source has ever existed in
this repo** — the compressed artifact has always been a sparse-sourced draft.
SP1 creates the first real one.

## Scope

This is **SP1** of a three-part sequence agreed during brainstorming:
- **SP1 (this spec):** dense de440-backed generation source + accuracy baseline.
- **SP2 (later):** tune per-body spans/degrees (and basis if needed) until error
  envelopes meet target levels.
- **SP3 (later):** define and enforce published per-body-class/channel
  thresholds; budget artifact size/perf; promote the artifact out of draft.

In scope for SP1:
- A kernel-gated generation path that fits each segment from de440 sampled
  densely within the segment span.
- A per-body segment-span / within-span-sampling / degree model with documented
  initial defaults.
- Regeneration of the committed draft artifact bytes + checksum from this model.
- A committed, drift-gated per-body accuracy-baseline summary measured kernel-free
  against the committed hold-out corpus.
- Baseline size/perf metrics via existing benchmark surfaces.

Out of scope for SP1 (later sub-projects):
- Tuning spans/degrees or changing the polynomial basis for accuracy (SP2).
- Defining or enforcing accuracy thresholds; budgeting/optimizing artifact size
  or latency; promoting the artifact out of draft (SP3).

## Sourcing model (Approach B)

Generation fits against de440 in-memory; only the artifact (bytes + checksum) is
committed — no dense intermediate CSV.

- **Regeneration (kernel-gated):** behind `PLEIADES_DE_KERNEL` (mirroring the
  existing `corpus_regen` contract), build `SpkBackend` from de440 and fit the
  artifact. Deterministic → byte-identical output across runs.
- **Runtime (kernel-free):** unchanged — `pleiades-data` decodes the committed
  `packaged-artifact.bin` via `include_bytes!`.
- **Verification (kernel-free):** the committed artifact bytes + checksum prove
  byte identity; the committed sparse hold-out corpus spot-checks decoded
  accuracy; `--check` validates decode + checksum (it cannot re-fit without the
  kernel).
- **Reproduce (kernel-gated):** gated regeneration reproduces the committed
  bytes from de440.

Consequence accepted by the user: artifact regeneration now **requires the
kernel** (it can no longer run from a clean checkout). This matches the corpus's
existing reproduce-from-de440 contract. In-process kernel-free consumers that
previously called the regenerator switch to decoding the committed bytes.

## Fit model

The current generator creates **one segment per consecutive control-point pair**,
so denser samples would explode segment count. SP1 decouples sample density from
segment count:

- Each body is partitioned into segments of a per-body **span** `[t0, t1]`.
- Each segment is fit by sampling de440 at **many points within** `[t0, t1]`
  (`(degree + 1) × oversample` points) and fitting a polynomial channel via the
  existing primitives (`PolynomialChannel`,
  `channel_from_fit_samples_with_control_points`).
- Segment **count** is driven by span (slow bodies → long spans → few segments);
  **accuracy** is driven by within-span density + degree.
- The per-segment fit path in `crates/pleiades-data/src/regenerate.rs` is reworked
  from consecutive-point windowing to this sample-within-span fit. Because the fit
  now sources positions from `SpkBackend` (de440), the reference-backend parameter
  becomes `&dyn EphemerisBackend` (subsuming the reverted Task-3 change). The kept
  Task-2 `SnapshotCorpusBackend` and Task-1 hold-out accessor are reused on the
  measurement side.

### Initial per-body defaults (documented, tunable in SP2)

Named, documented constants (in `corpus_spec.rs` or a new `generation_spec`):

| Body | segment span |
|---|---|
| Moon | ~4 d |
| Mercury | ~8 d |
| Venus / Sun | ~16 d |
| Mars | ~32 d |
| Jupiter | ~128 d |
| Saturn | ~256 d |
| Uranus / Neptune / Pluto | ~512 d |
| Eros (constrained) | ~16 d, only within its 1900–2100 corpus window |

- Within-span sampling: `(degree + 1) × oversample`; initial degree ≈ 8–10,
  oversample ≈ 3 (~30 samples/segment).
- Basis: reuse existing channel primitives. If those are power-series and
  conditioning is poor, revisiting Chebyshev is an SP2 decision, not SP1.
- These spans are **accuracy-safe but not size-optimized**; the baseline artifact
  will be substantially larger than today's ~4 MB draft (e.g. ~4-day Moon spans
  over 1000 yr ≈ 90k Moon segments). That is expected and measured, not budgeted,
  in SP1.

## Accuracy baseline measurement & reporting

- After regeneration, measure the **decoded** artifact against reference
  positions: per body, per channel (ecliptic **longitude**, **latitude**,
  **distance**), reporting **max** and **RMS** error — longitude/latitude in
  arcseconds, distance in km.
- **Reference set:** the committed sparse **hold-out** corpus (`holdout.csv`,
  500 de440 rows, independent of the fitting samples), keeping the baseline
  **kernel-free and reproducible**. A kernel-gated denser comparison against
  de440 may supplement the picture, but the committed baseline number derives
  from the kernel-free hold-out.
- **Surface:** a generated, validated `packaged-artifact-accuracy-baseline`
  summary (+ CLI command), consistent with the repo's validated-summary pattern —
  committed and drift-gated, giving SP2 a fixed reference. The report states
  explicitly that these are **draft error envelopes, not thresholds**.

## Determinism, size, performance

- The dense sampling grid and fit are deterministic; gated regeneration is
  byte-identical across runs.
- Record artifact **byte size**, **decode latency**, and **single-lookup
  latency** as baseline metrics via existing benchmark surfaces. Size will grow
  markedly; not budgeted in SP1 (SP3).

## Testing

- **Kernel-free:**
  - Decoded committed artifact validates.
  - The accuracy-baseline summary matches the committed value (drift gate).
  - Hold-out error measurement reproduces the committed baseline numbers within a
    fixed tolerance.
  - `--check` validates decode + checksum.
- **Kernel-gated (`PLEIADES_DE_KERNEL`):**
  - de440 regeneration reproduces the committed artifact bytes.
  - Within-span fit unit test: a known body/span yields a channel of the expected
    degree fitting de440 within the expected residual.

## Error handling

- Gated paths fail closed when the kernel is missing/mismatched (reuse the
  existing `PLEIADES_DE_KERNEL` gating and kernel-SHA pinning).
- Fit failures (insufficient samples, non-finite coefficients, ill-conditioned
  fit) fail closed with a body/segment-scoped error rather than emitting a
  degenerate channel.
- The artifact remains explicitly **draft-grade** in all release-facing
  summaries; asteroid (Eros) coverage stays constrained to 1900–2100.

## Risks and mitigations

- **Artifact size / regen time blow up.** Dense within-span fitting over 1000 yr
  is heavy. Mitigation: SP1 *measures* both as baseline outputs and keeps spans
  accuracy-safe; SP3 budgets/optimizes. If regen is intractably slow, the
  per-body span defaults are the first lever (longer spans + higher degree).
- **Existing channel primitives may be power-series and ill-conditioned at high
  degree.** Mitigation: keep degree modest (≈8–10) with short spans in SP1;
  Chebyshev migration is an explicit SP2 option.
- **Kernel-gated regeneration reduces clean-checkout reproducibility.** Accepted
  by the user; matches the corpus contract; clean-checkout integrity is preserved
  via committed bytes + checksum + kernel-free hold-out spot-checks.

## Follow-ups (not SP1)

- SP2: tune spans/degrees/basis against the SP1 baseline until error envelopes
  reach target levels.
- SP3: define/enforce per-body-class/channel thresholds; budget size/latency;
  promote artifact out of draft.
- Prune the stale Phase-1 status entries (`plan/status/01`, `plan/status/02`) and
  align `PLAN.md`/stage docs once SP1 lands.
