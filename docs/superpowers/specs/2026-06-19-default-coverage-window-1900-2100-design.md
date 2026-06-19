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
- Wide **kernel-free** generation — the committed reference snapshot is bounded and
  cannot synthesize coverage beyond what ships.
- SP3 size/perf **budget** enforcement — still deferred. We measure, we don't gate.

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

- **Core API:** promote the regeneration entry point in
  `crates/pleiades-data/src/regenerate.rs` to a documented public function taking an
  explicit `CoverageWindow` + kernel path, defaulting to the shipped window. Single
  path for default and custom artifacts.
- **Thin CLI** (`pleiades-gen`): parses `--start` / `--end` (accepting **either**
  calendar **years** *or* JD — years primary, friendlier; JD for precision),
  `--kernel`, and an output path. Builds a `CoverageWindow`, calls the core API,
  writes the artifact. No logic beyond arg-parsing.
- **Constraint:** wider-than-default generation requires the de440 kernel. The
  kernel-free snapshot path stays pinned to reproducing the shipped 1900–2100 default.
  The CLI errors clearly when asked for a window wider than the default without a
  kernel.

### 3. Fit anchors + committed data regeneration

- **Fit anchors** move inside the window: 1800/2000/2500 → **1900 / 2000 / 2100**.
  Update `REFERENCE_SNAPSHOT_*_EPOCH_JD` in `crates/pleiades-jpl/src/backend.rs`
  (1800 → 1900, 2500 → 2100; J2000 unchanged) and regenerate the affected rows of
  `crates/pleiades-jpl/data/reference_snapshot.csv` from the kernel.
- **Regenerate all committed corpus slices** at the new window — `holdout.csv`,
  `interior.csv`, `boundary.csv`, and any other backbone data. All are currently
  spread across 1600–2600 and become orphaned (outside coverage) when narrowed.
- **Regenerate the packaged artifact** and update committed bytes.
- **Recompute the golden accuracy baseline** in
  `crates/pleiades-data/src/accuracy_baseline.rs` against the new artifact + holdout.

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
