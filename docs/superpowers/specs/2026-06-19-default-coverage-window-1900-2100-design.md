# Default coverage window 1900–2100 (with opt-in wider generation)

**Date:** 2026-06-19
**Status:** Design — approved, pending implementation plan
**Supersedes (in part):** `2026-06-14-packaged-range-1600-2600-design.md` (major-body window)

## Problem

The packaged artifact ships a ~47.5 MB binary covering a **1600–2600 CE** major-body
window (~1000 years). Artifact size and test time are both driven by that span:

- Size is dominated by fast-cadence bodies. The Moon needs a backbone epoch roughly
  every 30 days (~3,000+ epochs over the current span); inner planets are similar.
  Epoch count scales ~linearly with span, so the major-body window is the lever.
- The slow test (`packaged_artifact_baseline`, ~1.7 s) is dominated by *decoding* the
  47.5 MB artifact. A smaller artifact decodes proportionally faster.

The asteroid window (`AST_RANGE_*`) is **already 1900–2100**. Narrowing the major-body
window to match collapses span by 5× for the bodies that dominate size.

## Goal

1. Ship a **1900–2100 CE** artifact by default — smaller binary, faster tests.
2. Let power users generate **wider** ranges themselves via a public API + thin CLI,
   without forking source.

## Non-goals (YAGNI)

- Changing the asteroid window (already 1900–2100).
- Changing the SP1 artifact format.
- Wide **kernel-free** generation — major-body generation fits densely from the de440
  kernel, so any window (default or wider) requires the kernel. The only kernel-free
  path is runtime-decode of the committed bytes (window-agnostic, see Reproduction
  paths below).
- SP3 size/perf **budget** enforcement — still deferred. We measure, we don't gate.

## Reproduction paths (corrected from investigation)

The shipped artifact's **major bodies are fit densely from the de440 kernel**
(`build_packaged_artifact_from_reference_over` → `fitting_segment_boundaries` +
`fit_segment_within_span`, Moon every 4 days, degree-8 LSQ). The curated
`reference_snapshot.csv` is used **only for the constrained asteroid (Eros)**, whose
data is absent from de440.

There are therefore three relevant paths:

1. **Kernel path** (`regenerate_packaged_artifact_from_kernel`) — source of the shipped
   bytes. Gated test `regenerated_artifact_matches_committed` pins it.
2. **Runtime-decode path** (`regenerate_packaged_artifact`) — the real kernel-free
   path; just decodes committed bytes. Works for any window for free.
3. **Legacy snapshot reconstruction** (`regenerate_packaged_artifact_from_snapshot` →
   `packaged_body_artifacts_from_snapshot`) — reconstructs **all** bodies from the
   ~32-epoch `reference_snapshot.csv`. Its segmentation (between sparse snapshot
   entries) is structurally different from the dense de440 fit, so it **cannot** be
   byte-identical to the shipped artifact. Its `#[ignore]`d byte-identity test
   (`tests/codec.rs`) predates the dense-generation switch (commit `aab351a5`) and is
   dead. **Decision: retire the legacy major-body branch** — restrict snapshot
   reconstruction to the asteroid (its live use) and delete the stale ignored test.

The "fitted to JPL Horizons reference epochs (1800, 2000, 2500 CE)" text in
`lib.rs` is **provenance prose only** — those epochs do not drive fitting or
validation. The fit reads the window constants; nothing is anchored to 1800/2500.

## Approach

Parameterize the major-body coverage window so one code path serves both the shipped
default and custom user artifacts. The two existing constants become the *default*
window, flipped to 1900–2100. Wider-than-default generation requires the de440 kernel.

## Design

### 1. Coverage window model

- Introduce `CoverageWindow { start_jd, end_jd }` in `pleiades-jpl`, next to the
  existing range constants.
- Flip `RANGE_START_JD` / `RANGE_END_JD` (`crates/pleiades-jpl/src/spk/corpus_spec.rs:9-10`)
  to **1900–2100**: `2_415_020.5 … 2_488_069.5`. These become the values backing
  `CoverageWindow::default()`. Major-body and asteroid windows now coincide.
- The five derivation functions that read the constants directly take a
  `CoverageWindow` argument instead of reaching for the global:
  `interior_backbone_epochs`, `boundary_epochs`, `fast_cluster_epochs`,
  `holdout_epochs`, and the major-body span uses in `regenerate.rs`. Default callers
  pass `CoverageWindow::default()`. Any window a user passes therefore produces an
  internally consistent backbone/boundary/holdout set.

### 2. Generation API + CLI

- **Core API:** add a public `regenerate_packaged_artifact_from_kernel_over(kernel,
  window)` in `crates/pleiades-data/src/regenerate.rs` taking an explicit
  `CoverageWindow`; the existing `regenerate_packaged_artifact_from_kernel` becomes a
  thin wrapper passing `CoverageWindow::default()`. The window flows into the already
  window-parameterized `build_packaged_artifact_from_reference_over(reference,
  base_window)`. Single path for default and custom artifacts.
- **Thin CLI:** a new `pleiades-cli` subcommand `generate-artifact` (matching the
  existing `generate-spk-corpus` / `generate-fixture-golden` dispatch in
  `crates/pleiades-cli/src/cli.rs`), parsing `<kernel.bsp> --start <S> --end <E>
  --out <path>`. `--start`/`--end` accept **either** calendar **years** *or* JD —
  years primary, JD for precision. Builds a `CoverageWindow`, calls the core API,
  writes the encoded artifact. No logic beyond arg-parsing.
- **Constraint:** all major-body generation requires the de440 kernel (dense fit), so
  the CLI always needs a kernel — there is no kernel-free wide generation. Kernel-free
  callers use runtime-decode of the committed default bytes.

### 3. Retire legacy snapshot path + regenerate committed data

- **Retire the legacy snapshot major-body branch:** restrict
  `packaged_body_artifacts_from_snapshot` (and `try_regenerate_packaged_artifact_from_snapshot`)
  to the constrained asteroid, and delete the stale `#[ignore]`d byte-identity test
  in `crates/pleiades-data/src/tests/codec.rs`
  (`packaged_artifact_generation_from_supplied_snapshot_matches_the_default_fixture`).
  See Reproduction paths above for why it can't hold.
- **Regenerate all committed corpus slices** at the new window. These are produced by
  `generate_slice` (driven by `corpus_spec` epoch functions, which read the window
  constants) via the CLI:
  `cargo run -p pleiades-cli -- generate-spk-corpus <kernel> --emit-slices
  crates/pleiades-jpl/data/corpus` — rewrites `boundary.csv`, `interior.csv`,
  `fast_clusters.csv`, `holdout.csv`, `manifest.txt`. All currently span 1600–2600 and
  become orphaned when narrowed. The gated `regenerated_corpus_matches_checked_in` test
  verifies the result.
- **`fixture_golden.csv`** is hand-populated from trusted Horizons fixtures (not
  backend-generated). If its epochs fall outside 1900–2100, repopulate it via
  `generate-fixture-golden` before re-emitting the manifest.
- **`reference_snapshot.csv` + summary:** the major-body rows at out-of-window epochs
  (1500/1600/1749/1800/2500/2600/2634) are now used only for provenance summaries
  (`reference_summary`), not artifact generation. Prune them to the new window and
  update the `REFERENCE_SNAPSHOT_*_EPOCH_JD` summary constants in `backend.rs` +
  `reference_snapshot_summary` validation so the summary stays consistent. The Eros /
  asteroid rows (1900–2100) are unchanged.
- **Regenerate the packaged artifact** from the kernel and update committed bytes.
- **Recompute the golden accuracy baseline** in
  `crates/pleiades-data/src/accuracy_baseline.rs` against the new artifact + holdout
  (golden is inline assertions; update the per-body buckets from the new run).

### 4. Documentation + terminology

- Update the ~50 prose references to "1600–2600" across `PLAN.md`, `SPEC.md`,
  `README.md`, CLI help text, and report templates to the 1900–2100 default plus the
  new wider-generation capability.
- Update the `crates/pleiades-data/src/lib.rs` provenance comment (the
  "1800, 2000, 2500 CE" reference-epoch text) to the new anchors.

### 5. Testing + verification

- **Byte-identity** (`regenerated_artifact_matches_committed`, kernel-gated): verifies
  the regenerated default matches committed bytes — the end-to-end correctness anchor.
- **Drift-gate / accuracy baseline**: re-greens against the new golden.
- **Size/perf baseline** (`sp1_draft_size_perf_baseline`): records the new (smaller)
  size + faster decode. Before/after numbers recorded in this spec (below) and in
  `PLAN.md`.
- **New tests:**
  - `CoverageWindow` parameterization — `default()` is 1900–2100; a custom window
    yields a corpus whose backbone/boundary/holdout epochs all fall inside the
    requested window.
  - CLI smoke test — arg parsing (years and JD) and the "wide window without kernel"
    error path.
- The holdout in-window assertion (`corpus_spec.rs:246`) stays, now asserting against
  the passed window.

## Size / perf baseline

Recorded during implementation from `sp1_draft_size_perf_baseline` (and mirrored into
`PLAN.md`):

| Metric            | Before (1600–2600) | After (1900–2100) |
| ----------------- | ------------------ | ----------------- |
| Artifact size     | ~47.5 MB           | _(measured)_      |
| Decode latency    | _(measured)_       | _(measured)_      |
| Lookup latency    | _(measured)_       | _(measured)_      |
| Baseline test     | ~1.7 s             | _(measured)_      |

## Blast radius (reference)

- Window constants are a clean single source of truth: `corpus_spec.rs:9-10`. No
  duplicated JD values in code.
- Direct consumers: `interior_backbone_epochs`, `boundary_epochs`,
  `fast_cluster_epochs`, `holdout_epochs` (all `corpus_spec.rs`), and the major-body
  span uses in `regenerate.rs:2531-2533`.
- Fit-anchor constants: `backend.rs:1557,1559,1560`.
- Committed data to regenerate: `reference_snapshot.csv`, `holdout.csv`,
  `interior.csv`, `boundary.csv`, the packaged artifact bytes, and the golden baseline.
- Prose-only "1600/2600" references (~50): `PLAN.md`, `SPEC.md`, `README.md`, CLI
  help, report templates, `lib.rs` comment.
