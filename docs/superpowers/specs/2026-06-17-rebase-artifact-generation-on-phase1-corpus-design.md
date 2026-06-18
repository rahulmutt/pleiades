# Rebase Artifact Generation on the Phase 1 Corpus

Date: 2026-06-17
Status: **HALTED 2026-06-18 — premise invalidated.** See "Halt note" below.
Phase: 2 — Release-grade compressed ephemeris (first slice only)

## Halt note (2026-06-18)

This slice was halted during implementation. Its central premise — that the
committed Phase 1 corpus under `crates/pleiades-jpl/data/corpus/` is a usable
*generation* source — is false. The corpus samples roughly **two points per
orbital period for every body** (e.g. Moon at 30-day cadence vs a 27-day period;
Sun at 180-day; Mercury at 60-day), which is a *validation* sampling strategy
(sparse spot-checks against an independently-generated artifact), far below the
density required to *fit* a continuous polynomial artifact. Rebasing generation
onto it produced an aliased, qualitatively-wrong artifact (e.g. the Moon),
which the regeneration step could only "pass" by widening lookup tolerances.

Outcome:
- **Kept** (useful infrastructure, landed on branch `phase2-rebase-artifact-generation`):
  Task 1 typed corpus accessors (`pleiades-jpl::corpus`) and Task 2
  `SnapshotCorpusBackend`.
- **Reverted:** Task 3 (generator rebase) and Task 4 (artifact regeneration),
  plus their downstream tasks.
- **Real prerequisite:** a dense, fitting-grade generation corpus
  (body-appropriate cadence) must exist before generation can be rebased. That
  is a separate design effort and supersedes this spec.

## Summary

Make the `pleiades-data` packaged-artifact generator fit from the broad Phase 1
reference corpus committed under `crates/pleiades-jpl/data/corpus/` instead of
the narrow `reference_snapshot()` fixture. Regenerate the committed artifact
deterministically from the corpus and keep it draft-labeled. This is the first
Phase 2 slice ("rebase artifact generation on the Phase 1 production reference
and hold-out inputs") and the prerequisite for the rest of the phase; it does
**not** define or enforce production accuracy thresholds.

## Scope

In scope:

- Expose the committed corpus slices as typed accessors from `pleiades-jpl`.
- Add a corpus-backed reference backend so fitting can evaluate intermediate
  positions against arbitrary corpus rows.
- Re-point the generator and its input guard at the corpus.
- Regenerate the committed (draft) artifact bytes deterministically from the
  corpus so `generate-packaged-artifact --check` passes.
- Source `asteroid:433-Eros` from the `asteroid_constrained` slice.
- Deduplicate the corpus embedding currently held in `pleiades-validate`.
- Update posture/summary strings and tests that assert narrow-fixture sourcing.

Out of scope (later Phase 2 slices):

- Defining published accuracy thresholds per body class and channel.
- Enforcing reference/hold-out error thresholds or improving fitting to pass
  them.
- Promoting the artifact out of draft grade.
- Broadening asteroid coverage beyond the bodies already in the corpus.

## Current state

- The generator (`crates/pleiades-data/src/regenerate.rs`) builds the artifact
  per body by fitting polynomial segments. `regenerate_packaged_artifact()`
  sources entries from `reference_snapshot()` (the narrow fixture) and uses
  `JplSnapshotBackend` — a unit struct that reads the *global narrow snapshot* —
  to evaluate intermediate positions during fitting.
- The broad Phase 1 corpus (~25,659 data rows: `interior`, `boundary`,
  `fast_clusters`, `holdout`, `fixture_golden`, `asteroid_reference`,
  `asteroid_constrained`) lives under `crates/pleiades-jpl/data/corpus/` but is
  only embedded in `pleiades-validate` via `../../../pleiades-jpl/...`
  `include_str!`. The generator in `pleiades-data` cannot see it.
- Corpus CSV schema is `epoch_jd,body,x_km,y_km,z_km` — exactly the
  `SnapshotEntry` shape. Reusable pure-Rust parsers already exist in
  `pleiades-jpl` (`parse_snapshot_entries`, `load_snapshot_from_csv`,
  `parse_snapshot_corpus`, `load_snapshot_corpus_from_paths`).
- Crate layering: `pleiades-jpl` → `pleiades-data` → `pleiades-validate`.
- The packaged artifact covers 11 bodies: the 10 base bodies (Sun, Moon,
  Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto) plus
  `asteroid:433-Eros`.
- Body coverage per corpus slice:
  - `interior`, `boundary`, `holdout`: the 10 base bodies only.
  - `fast_clusters`: Mercury, Moon, Venus.
  - `asteroid_reference` (Tier A, sb441-n16): Ceres, Pallas, Juno, Vesta,
    Hygiea, Psyche, Iris — **no Eros**.
  - `asteroid_constrained` (Tier B, Horizons, 1900–2100): includes
    `asteroid:433-Eros`.
  - `fixture_golden`: 16-row exactness cross-check including Eros and Apophis.
- A body-class span-cap mechanism already exists
  (`packaged_artifact_body_class_span_cap_entries`, `body_segment_span_limit`)
  with a recognized "selected asteroids" class.

## Design

### 1. Typed corpus accessors in `pleiades-jpl`

`pleiades-jpl` owns the CSV files, so it becomes the single source of truth.
Add a corpus module exposing the embedded slices parsed into
`Vec<SnapshotEntry>`, grouped by role and cached in `OnceLock`:

- `production_reference_corpus()` → de440 **fitting** rows
  (`interior ∪ boundary ∪ fast_clusters`).
- `production_holdout_corpus()` → `holdout` rows (kept out of fitting).
- `fixture_golden_corpus()` → `fixture_golden` rows (exactness cross-check).
- Asteroid accessors for `asteroid_reference` and `asteroid_constrained`,
  including a way to obtain the rows for a specific asteroid body
  (e.g. `asteroid:433-Eros`) from `asteroid_constrained`.

Parsing reuses the existing `parse_snapshot_entries`/`load_snapshot_from_csv`
entry points. Accessors validate against the embedded `manifest.txt`
(per-slice row counts and checksums) so a drifted CSV fails closed at load.

### 2. Corpus-backed reference backend

`JplSnapshotBackend` is a unit struct reading the global narrow snapshot and
cannot evaluate positions against arbitrary corpus rows. Extract its
interpolation logic (cubic on four-sample windows, quadratic on three, linear
fallback) into a backend that holds arbitrary `&[SnapshotEntry]`, e.g.
`SnapshotBackend::from_entries(...)`. The generator constructs one over the
reference corpus (and over the Eros constrained rows) for residual/midpoint
evaluation. The existing global `JplSnapshotBackend` remains as a thin wrapper
for backward compatibility so unrelated callers are unaffected.

### 3. Slice → role mapping

- **Fitting/reference:** `interior ∪ boundary ∪ fast_clusters` for the 10 base
  bodies (1600–2600 CE; `boundary` supplies the window-edge anchors so segments
  reach the advertised span).
- **Eros (`asteroid:433-Eros`):** fit from `asteroid_constrained` (Tier B,
  Horizons, 1900–2100). Stored as a constrained body, naturally span-bounded by
  the available corpus rows and consistent with the existing "selected
  asteroids" body-class span cap. The base bodies' 1600–2600 span is unchanged.
- **Hold-out:** `holdout` is excluded from fitting and wired as available for
  comparison. Threshold *enforcement* is a later Phase 2 slice.
- **Separate evidence (not fed to release fitting):** `fixture_golden`
  (exactness), `asteroid_reference` (Tier A), and the remaining
  `asteroid_constrained` bodies. Report evidence classes stay labeled distinctly
  per the plan's "keep fitting/hold-out/boundary/fixture/provenance separate"
  rule.

### 4. Generator rebase

- `regenerate_packaged_artifact()` sources base-body entries from
  `production_reference_corpus()` and Eros entries from the `asteroid_constrained`
  Eros rows, instead of `reference_snapshot()`.
- The per-body grouping and `thread::scope` fan-out in
  `packaged_body_artifacts_from_snapshot` are reused; each body's segments are
  fit against a `SnapshotBackend::from_entries(...)` built over that body's
  corpus rows.
- `validate_packaged_artifact_reference_snapshot_inputs` (which currently
  asserts the input equals `reference_snapshot()`) is re-pointed to validate the
  generation input against the corpus accessors, still fail-closed on
  length/content drift.
- Regenerate the committed artifact bytes deterministically from the corpus so
  `generate-packaged-artifact --check` passes against the new bytes. The
  artifact stays **draft-labeled**; no production thresholds are introduced.

### 5. Deduplicate the corpus embedding in `pleiades-validate`

Replace the `../../../pleiades-jpl/data/corpus/*.csv` `include_str!` block in
`pleiades-validate/src/corpus/production.rs` with the new `pleiades-jpl`
accessors, so one embedding feeds both the generator and the `validate-corpus`
gate. The gate's fail-closed behavior (missing bodies/roles, schema/checksum
drift, malformed/non-finite rows, placeholder SHA, fixture-golden cross-check)
is preserved.

## Error handling and determinism

- Generation remains deterministic and pure-Rust. `interior` is ~24,813 rows
  vs. the tiny fixture, so verify regeneration time and memory stay reasonable
  and the encoded output is byte-stable across runs.
- Corpus accessors fail closed when a slice is missing, mis-counted versus the
  manifest, checksum-drifted, or contains malformed/non-finite rows.
- The input guard fails closed when the generation input diverges from the
  corpus.
- Eros lookups outside the 1900–2100 corpus span return the existing
  out-of-range behavior for a constrained body; the base bodies keep full
  1600–2600 coverage.

## Posture and report churn

Many posture/summary strings and their tests assert the narrow-fixture sourcing
(for example `"input path=checked-in CSV fixtures via include_str!
reference_snapshot.csv"` and the `phase2_corpus_alignment` summary). These are
updated to describe corpus sourcing, with their assertions updated in lockstep.
The artifact remains explicitly draft-grade and asteroid coverage remains
constrained in all release-facing summaries.

## Testing

- Corpus accessors parse successfully and match the manifest's per-slice row
  counts and checksums.
- The generator fits the base bodies from the de440 reference corpus and Eros
  from `asteroid_constrained`.
- `holdout` rows are excluded from fitting.
- Regeneration is deterministic and byte-stable; `generate-packaged-artifact
  --check` passes against the corpus-derived bytes.
- The `validate-corpus` gate still fails closed on drift after the embedding is
  deduplicated.
- Updated posture/summary tests reflect corpus sourcing.

## Risks and mitigations

- **Fitting cost on the broad corpus.** Mitigation: reuse the existing
  per-body `thread::scope` fan-out; measure regen wall-clock and confirm it
  stays acceptable for a maintainer-run command.
- **Eros span narrower than base bodies.** Mitigation: store Eros as a
  constrained, span-capped body; keep all release-facing summaries truthful
  about the 1900–2100 constraint.
- **Wide posture/test churn.** Mitigation: treat string + assertion updates as
  a single mechanical pass; keep evidence-class labels distinct.

## Out-of-band follow-ups (not this slice)

- Prune the stale Phase 1 entries in `plan/status/01` and `plan/status/02`
  (public-data reader and asteroid-kernel adoption are already Met).
- Later Phase 2 slices: define and enforce per-body-class/channel thresholds and
  improve fitting until reference and hold-out errors pass.
