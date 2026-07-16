# Report-surface relocation — Slice D Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Relocate all report prose out of `pleiades-jpl` (and the one dependent `pleiades-data` renderer) into `pleiades-validate/src/posture/jpl/`, byte-identically, promoting the crate-private corpus accessors the moved renderers read.

**Architecture:** `pleiades-jpl` keeps every evidence/summary struct, its `*_details()` constructor, its `validate()`/`label()` methods, and its `ValidationError` enums (structured data + gate logic). It loses all *rendering*: the free `*_for_report` functions move verbatim, and the inherent `summary_line()`/`validated_summary_line()`/`impl Display` on evidence structs are stripped into free `*_for_report` functions in validate that read the structs' public fields. The single below-validate consumer of jpl prose — `pleiades-data`'s `PackagedArtifactPhase2CorpusAlignmentSummary` rendering — follows the jpl rendering into validate. No output changes, no version bump.

**Tech Stack:** Rust workspace (`cargo`), `mise` task runner, `fnv1a64` bundle checksums, `release-smoke` rehearsal gate.

## Global Constraints

- **Byte-identical rendered text.** Every moved/stripped renderer produces exactly the prose it produced before. fnv1a64 release-bundle checksum *values* MUST NOT change — any drift is a defect in the move, never a checksum to regenerate. `release-smoke` proves it.
- **No version bump.** Compatibility profile stays `0.7.13`; API-stability profile stays `0.3.0`; `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` unchanged. The workspace `0.3.0 → 0.4.0` bump + release is a SEPARATE follow-up after this slice merges.
- **No behavior / gate / threshold / corpus changes.** `validate-corpus`, claims-audit, and the numeric gates keep enforcing identical data.
- **Dependency direction.** Nothing below `pleiades-validate` gains a dependency on it. jpl and data keep only structs + constructors + `validate()`; all prose lives in validate.
- **Posture-module convention** (established Slices A–C): one `posture/<crate>/` subdir mirroring the source crate; `pub(crate)` by default with `#![allow(dead_code)]`; `pub` only where a genuine cross-crate runtime consumer exists.
- **CI gate for every task boundary:** `mise run ci` (fmt, clippy `-D warnings`, `cargo test --workspace --include-ignored`, `cargo doc -D warnings`, workspace-audit, package-check, release-smoke, claims-audit). Commit only on green.
- Branch `feat/report-surface-relocation-slice-d` is already checked out with the design committed.

---

## Execution model — EXPAND-then-CONTRACT (supersedes strip-in-place)

> **Amendment (2026-07-13, program-owner approved).** The originally-drafted
> recipe stripped each struct's inherent rendering *in the same family task*.
> That is **mechanically impossible** under the dependency-direction invariant:
> jpl's render layer is a deep call graph that crosses files — free
> `*_for_report` renderers aggregate other files' renderers (e.g.
> `jpl_posture.rs::jpl_snapshot_evidence_summary_for_report` calls
> `comparison_snapshot_summary_for_report`, `reference_snapshot_*`,
> `production_generation_*`, `selected_asteroid_*`), and inherent
> `summary_line()` methods call each other across structs. Deleting a renderer
> or inherent method in an early task breaks a *later* task's jpl renderer that
> still calls it, and its only replacement lives in validate — which jpl may
> not call. So we **copy, never delete, during the family tasks**, and remove
> jpl's entire render layer in ONE final contract sweep.

**Phase structure:**

- **EXPAND (Task 2 + Tasks 4–12): additive copy into validate. Delete NOTHING
  from `pleiades-jpl` or `pleiades-data`.** jpl keeps all its rendering and
  stays fully green (still renders, all its tests pass). Validate gains a
  verbatim copy of the render layer under `posture/jpl/`.
- **REPOINT + SELF-CONTAIN (Task 13):** flip every external consumer
  (validate `lib.rs`/`report.rs`/bundle pins, CLI) to the validate copies, and
  flip every *validate-internal* call that still reaches into jpl rendering to
  its local equivalent — so nothing outside jpl's own crate renders via jpl.
  `release-smoke` proves checksum values unchanged. Delete nothing yet.
- **CONTRACT SWEEP (Task 14):** now that no code outside `pleiades-jpl`'s own
  crate calls its render layer, delete it wholesale — all free `*_for_report`,
  all inherent `summary_line()`/`validated_summary_line()`, all evidence-struct
  `impl Display` — plus the one `pleiades-data` renderer; repoint any
  jpl-internal caller (audit `validate()` paths); the grep gates go green here.

## The transformation recipe (referenced by every family task)

Slice D is a large *mechanical* relocation. Task 2 works one struct end-to-end with real code; **Tasks 4–12 apply this exact recipe** to enumerated symbols. The recipe, per source file `crates/pleiades-jpl/src/<path>.rs` → `crates/pleiades-validate/src/posture/jpl/<path>.rs`:

For each **evidence struct** `S` in the file:

1. **Keep in jpl, UNCHANGED (do not delete or edit its rendering during expand):** everything — the `struct S`, its public fields, `impl S { fn validate(), fn label(), summary_line(), validated_summary_line(), private helpers }`, its `*_details()`/`*_summary()` constructors, `impl Display for S`, and `impl Display for SValidationError`. The family task is **purely additive to validate**; jpl is touched only to promote a field to `pub` if a copied renderer reads a non-`pub` field (Task 1 promoted the accessors; per-field promotions land beside their consumer).
2. **Add to validate** `posture/jpl/<path>.rs`:
   - `pub(crate) fn <s_snake>_summary_line(s: &S) -> String { <verbatim body of jpl's S::summary_line, with self→s; any nested call on ANOTHER jpl evidence struct `w.summary_line()` rewritten to the local free fn `<w_snake>_summary_line(&w)` WHEN that helper already exists locally, else call jpl's still-present `w.summary_line()` (validate→jpl is allowed and byte-identical; Task 13 flips the residual). Nested calls on NON-jpl types (e.g. `Instant::summary_line`, pleiades-time) stay as-is — only jpl's own rendering moves.> }`.
   - The file's free `pub fn …_for_report()` functions, **copied verbatim**, except: `summary.summary_line()` → `<s_snake>_summary_line(&summary)`; `summary.validated_summary_line()` → `{ summary.validate().map_err(|e| e.to_string())?; Ok(<s_snake>_summary_line(&summary)) }` (validate() stays in jpl; rendering is local); `format!("{summary}")`/`summary.to_string()` (Display) → `<s_snake>_summary_line(&summary)`; and any call to a free renderer defined in a **different, not-yet-copied** file → left as `pleiades_jpl::<name>()` (validate→jpl, byte-identical; Task 13 flips it local).
3. **Copy the file's report tests** (the ones asserting rendered strings / calling `_for_report`) into the same validate module, verbatim (pointed at the validate copies via `super::`). **Leave the jpl originals in place** — they still pass against jpl's still-present renderers and are deleted with those renderers in the contract sweep. No test-splitting during expand.

Each copied `_for_report` keeps its exact name (validate re-exports by name in Task 13). Byte-identity is guaranteed because the render bodies are copied verbatim; it is *verified* by the copied unit tests (which assert exact strings) plus `release-smoke`.

Naming: `posture/jpl/mod.rs` gains `#[allow(unused_imports)] pub(crate) use <module>::*;` per copied file so re-exports resolve, mirroring `pleiades-jpl/src/reference_summary/mod.rs`.

**Per-family EXPAND verification (replaces any per-task "jpl grep returns nothing" check — jpl is intentionally still full until the sweep):**
- `cargo build -p pleiades-jpl && cargo build -p pleiades-validate` — PASS.
- `cargo test -p pleiades-validate posture::jpl::<module>` — PASS (copied report tests assert exact strings ⇒ byte-identity of the copy).
- `mise run ci` — PASS (`release-smoke` checksums unchanged; jpl unchanged, so its own tests still pass too). Commit on green.

---

### Task 1: Promote jpl corpus/evidence accessors + boundary constants + renderer-read fields

**Files:**
- Modify: `crates/pleiades-jpl/src/backend.rs` (accessor + const visibility)
- Modify: evidence-struct fields across `crates/pleiades-jpl/src/reference_summary/*` where a moved renderer reads a non-`pub` field (audited in this task)

**Interfaces:**
- Produces: `pub fn snapshot_entries() -> Option<&'static [SnapshotEntry]>`, `pub fn comparison_snapshot_entries() -> &'static [SnapshotEntry]`, `pub fn snapshot_bodies() -> &'static [CelestialBody]`, `pub fn snapshot_instants() -> &'static [Instant]`, `pub fn comparison_body_list() -> &'static [CelestialBody]`, `pub fn reference_asteroid_evidence_list() -> &'static [ReferenceAsteroidEvidence]`, `pub fn interpolation_quality_sample_list() -> &'static [InterpolationQualitySample]`, `pub fn is_comparison_body(&CelestialBody) -> bool`, and `pub const REFERENCE_SNAPSHOT_*_EPOCH_JD: f64` — all consumable from `pleiades-validate`.

- [ ] **Step 1: Promote the 8 accessors**

In `crates/pleiades-jpl/src/backend.rs`, change `pub(crate) fn` → `pub fn` for exactly: `snapshot_entries` (~:1572), `snapshot_bodies` (~:1580), `comparison_snapshot_entries` (~:1622), `comparison_body_list` (~:1878), `reference_asteroid_evidence_list` (~:1932), `interpolation_quality_sample_list` (~:1985), `snapshot_instants` (~:2081), `is_comparison_body` (~:2379). Add a `/// ` doc line to any that lacks one (clippy `missing_docs` on `pub` items).

- [ ] **Step 2: Promote the boundary-epoch constants**

In `backend.rs`, change `pub(crate) const REFERENCE_SNAPSHOT_*_EPOCH_JD: f64` → `pub const …` for all `REFERENCE_SNAPSHOT_*_EPOCH_JD` (~:1597–1606 and any peers). Add `/// ` docs where missing.

- [ ] **Step 3: Verify it builds and nothing else changed**

Run: `cargo build -p pleiades-jpl`
Expected: PASS. No behavior change — these were already reachable in-crate.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-jpl/src/backend.rs
git commit -m "refactor(jpl): promote corpus/evidence accessors + boundary consts to pub (Slice D prereq)"
```

> Field promotions are deferred: Tasks 4–11 promote a specific struct field to `pub` only when that task's moved renderer fails to compile against it, keeping each promotion beside its consumer.

---

### Task 2: backend.rs coupling — the worked recipe (InterpolationQualitySample + SnapshotManifestSummary)

> **EXPAND phase — additive copy only. Delete NOTHING from `pleiades-jpl`.**
> Correction to the original draft: `backend.rs` contains **no free
> `*_for_report` renderers** (`grep -n "fn .*_for_report" crates/pleiades-jpl/src/backend.rs`
> returns nothing) and does **not** define
> `format_jpl_interpolation_quality_summary_for_report` — that free renderer
> lives in `reference_summary/jpl_posture.rs` (Task 6) and composes *other*
> renderers; it does not iterate samples. Task 2's real job is to copy the two
> backend structs' **inherent** rendering into validate free fns. The
> `InterpolationQualitySample::summary_line` body (`backend.rs:114`) is exactly
> the `jpl_interpolation_quality_sample_summary_line` example below.

**Files:**
- Create: `crates/pleiades-validate/src/posture/jpl/mod.rs`
- Create: `crates/pleiades-validate/src/posture/jpl/backend.rs`
- Modify: `crates/pleiades-validate/src/posture/mod.rs` (register `jpl`)
- Test: golden-fixture equality test in `crates/pleiades-validate/src/posture/jpl/backend.rs`
- **`crates/pleiades-jpl/src/backend.rs` is NOT modified in this task** (its rendering is copied, not stripped; it is deleted in the Task 14 contract sweep).

**Interfaces:**
- Consumes: `pleiades_jpl::{InterpolationQualitySample, SnapshotManifestSummary, interpolation_quality_sample_list}` (Task 1) and the `InterpolationQualitySample`/`SnapshotManifestSummary` public fields.
- Produces (in `crate::posture::jpl::backend`, all `pub(crate)`): `interpolation_quality_sample_summary_line(&InterpolationQualitySample) -> String`, and the `SnapshotManifestSummary` render helper(s) matching jpl's inherent signatures — `snapshot_manifest_summary_line(&SnapshotManifestSummary) -> String` delegating to the `SnapshotManifest::summary_line_with_defaults` body copied in (enumerate the exact inherent methods in Step 3).

- [ ] **Step 1: Register the `jpl` posture module**

In `crates/pleiades-validate/src/posture/mod.rs` add (alphabetical): `pub(crate) mod jpl;`

Create `crates/pleiades-validate/src/posture/jpl/mod.rs`:

```rust
//! `pleiades-jpl` report/summary prose relocated from the functional crate
//! (report-surface relocation program, Slice D). Rendering only — the
//! functional crate keeps the structured evidence structs, their
//! `*_details()` constructors, `validate()`/`label()` methods, and all
//! release-gate data.
#![allow(dead_code)]

pub(crate) mod backend;
```

- [ ] **Step 2: Golden fixture (old-vs-new equality)**

Create `crates/pleiades-validate/src/posture/jpl/backend.rs` and, alongside the copied renderers (Step 3), pin byte-identity by comparing jpl's still-present inherent rendering against the validate copy over the real corpus data:

```rust
//! Relocated backend-struct renderers (InterpolationQualitySample,
//! SnapshotManifestSummary) copied from `pleiades-jpl::backend` (Slice D).

#[cfg(test)]
mod golden {
    use pleiades_jpl::interpolation_quality_sample_list;

    // jpl's inherent renderer is still present through the contract sweep
    // (Task 14); this fails closed on any drift in the validate copy. Task 14
    // replaces `before` with the captured literal when the jpl method is
    // deleted.
    #[test]
    fn interpolation_quality_sample_lines_byte_identical() {
        for sample in interpolation_quality_sample_list() {
            let before = sample.summary_line(); // jpl inherent method (still present)
            let after = super::interpolation_quality_sample_summary_line(sample);
            assert_eq!(before, after);
        }
    }
}
```

Add the analogous equality test for the `SnapshotManifestSummary` render helper(s) against jpl's inherent `summary_line`.

- [ ] **Step 3: Copy the inherent renderings into validate**

In `crates/pleiades-validate/src/posture/jpl/backend.rs`, above the test module, add (bodies copied **verbatim** from the jpl inherent methods, `self`→`s`):

```rust
use pleiades_jpl::{InterpolationQualitySample, SnapshotManifestSummary};

/// Compact release-facing summary line for one interpolation-quality sample.
/// Verbatim copy of `InterpolationQualitySample::summary_line` (backend.rs:114).
pub(crate) fn interpolation_quality_sample_summary_line(s: &InterpolationQualitySample) -> String {
    format!(
        "{} at {}: {} interpolation, bracket span {:.1} d, |Δlon|={:.12}°, |Δlat|={:.12}°, |Δdist|={:.12} AU",
        s.body,
        s.epoch.summary_line(), // Instant::summary_line (pleiades-time) — NOT moved, stays
        s.interpolation_kind.label(),
        s.bracket_span_days,
        s.longitude_error_deg,
        s.latitude_error_deg,
        s.distance_error_au,
    )
}
```

Then copy `SnapshotManifestSummary`'s inherent render surface (`summary_line` at `backend.rs:798`, which delegates to `SnapshotManifest::summary_line_with_defaults`; copy the actually-rendering body it reaches, reading public fields) into `snapshot_manifest_summary_line(&SnapshotManifestSummary) -> String` (and any peer render method the struct exposes). Do **not** copy `validate()`/`validated_summary_line()` gate logic — those stay in jpl; a `validated_*` renderer becomes `{ s.validate().map_err(|e| e.to_string())?; Ok(snapshot_manifest_summary_line(s)) }` if validate needs one. Enumerate the exact inherent methods first: `grep -n "    pub fn summary_line\|    pub fn validated_summary_line\|impl fmt::Display for \(InterpolationQualitySample\|SnapshotManifestSummary\)" crates/pleiades-jpl/src/backend.rs`.

- [ ] **Step 4: Register in `posture/jpl/mod.rs`** (already done in Step 1) and confirm the copied consumer path — the sole cross-crate runtime consumer of `InterpolationQualitySample::summary_line` is `pleiades-validate/src/render/summary/writers.rs:369` (`sample.summary_line()` in a loop). Do **not** repoint it yet — that flip happens in Task 13. jpl's inherent method still backs it during expand.

- [ ] **Step 5: Verify + commit**

Run: `cargo test -p pleiades-validate posture::jpl::backend` — PASS (golden equality old==new).
Run: `mise run ci` — PASS, including `release-smoke` (checksums unchanged; jpl untouched so its own tests still pass).

```bash
git add crates/pleiades-validate/src/posture/
git commit -m "refactor(validate): copy backend struct renderers into validate posture (Slice D expand)"
```

---

### Tasks 4–9: reference_summary family moves (apply the Task-2 recipe)

Each task **copies** one file/subtree of `crates/pleiades-jpl/src/reference_summary/` → `crates/pleiades-validate/src/posture/jpl/`, applying the EXPAND recipe: re-home inherent `summary_line`/`validated_summary_line`/`Display` as `pub(crate)` free render fns in validate, copy the free `*_for_report` verbatim (rewiring same-file/already-copied internal calls to local; cross-file-not-yet-copied calls stay `pleiades_jpl::…`), copy the report tests, register the module in `posture/jpl/mod.rs`. **Delete NOTHING from jpl** — structs, `validate()`, `label()`, `*_details()`, `summary_line()`, `Display`, and the jpl-side tests all stay until the Task 14 contract sweep.

**Per-task verification (identical, run at each task's end):**
- `cargo build -p pleiades-jpl && cargo build -p pleiades-validate` — PASS.
- `cargo test -p pleiades-validate posture::jpl::<module>` — PASS (copied report tests assert exact strings ⇒ byte-identity of the copy).
- `mise run ci` — PASS (`release-smoke` checksums unchanged; jpl untouched so its own tests still pass). Commit on green.
- (The "jpl grep returns nothing" gate is deferred to Task 14 — jpl is intentionally still full of renderers during expand.)

Enumerate a file's symbols to copy with:
`grep -n "^pub fn .*_for_report\|    pub fn summary_line\|    pub fn validated_summary_line\|impl fmt::Display for" crates/pleiades-jpl/src/reference_summary/<file>.rs`

- [ ] **Task 4: `comparison.rs`** → `posture/jpl/comparison.rs` — 11 free renderers, 11 inherent methods, 11 Display. Commit: `refactor(jpl,validate): relocate comparison-snapshot renderers (Slice D)`.
- [ ] **Task 5: `holdout.rs`** → `posture/jpl/holdout.rs` — 13 free, 21 methods, 17 Display. Commit: `refactor(jpl,validate): relocate independent-holdout renderers (Slice D)`.
- [ ] **Task 6: `jpl_posture.rs`** → `posture/jpl/jpl_posture.rs` — 16 free, 22 methods, 23 Display. Commit: `refactor(jpl,validate): relocate jpl-posture renderers (Slice D)`.
- [ ] **Task 7: `reference_asteroid.rs`** → `posture/jpl/reference_asteroid.rs` — 4 free, 6 methods, 7 Display. Uses `reference_asteroid_evidence_list()` (Task 1). Commit: `refactor(jpl,validate): relocate reference-asteroid renderers (Slice D)`.
- [ ] **Task 8: `reference_snapshot/core/*`** → `posture/jpl/reference_snapshot/core/{coverage,evidence,general_a,general_b,parity}.rs` — 46 free renderers total (coverage 3, evidence 4, general_a 16, general_b 18, parity 5) plus their inherent methods; move `reference_snapshot/tests.rs` report tests. Uses `snapshot_entries`/`snapshot_bodies`/`snapshot_instants` (Task 1). Mirror the source submodule tree; add `pub(crate) mod reference_snapshot;` to `posture/jpl/mod.rs` and a `reference_snapshot/mod.rs` + `reference_snapshot/core/mod.rs`. Commit: `refactor(jpl,validate): relocate reference-snapshot core renderers (Slice D)`.
- [ ] **Task 9: `reference_snapshot/boundaries/*`** → `posture/jpl/reference_snapshot/boundaries/{era_a,era_b,era_c,era_d}.rs` — 6 free renderers (era_b 2, era_d 4; era_a/era_c carry boundary renderers too — enumerate) plus inherent methods; uses `REFERENCE_SNAPSHOT_*_EPOCH_JD` (Task 1). Add `boundaries/mod.rs`. Commit: `refactor(jpl,validate): relocate reference-snapshot boundary renderers (Slice D)`.

> Task 8 is the largest (~general_a 1473 LOC + general_b 2149 LOC + tests 2650 LOC). If a reviewer prefers, split it per file (coverage/evidence/parity in one commit, general_a and general_b each their own) — the verification block is per-file already.

---

### Task 10: reference_summary/production_generation.rs + top-level production_generation.rs report half

> **EXPAND phase — copy only, delete NOTHING from jpl.** "Move" below means
> *copy into validate*; the jpl originals (17 + 7 renderers + inherent methods)
> stay until the Task 14 contract sweep. Cross-file calls to renderers not yet
> copied stay `pleiades_jpl::…`.

**Files:**
- Create: `crates/pleiades-validate/src/posture/jpl/production_generation.rs`
- Modify: `crates/pleiades-validate/src/posture/jpl/mod.rs`
- Test: golden fixture in the new validate module
- (`crates/pleiades-jpl/src/reference_summary/production_generation.rs` and `crates/pleiades-jpl/src/production_generation.rs` are read for the copy but **not modified** — their renderers/methods are deleted in Task 14.)

**Interfaces:**
- Consumes: `pleiades_jpl::{ProductionGenerationBoundarySummary, ProductionGenerationBoundaryWindow, ProductionGenerationBoundaryWindowSummary, ProductionGenerationBoundaryBodyClassCoverageSummary, ProductionGenerationBoundaryRequestCorpusSummary, IndependentHoldoutSourceSummary, …}` (structs + their `validate()`/`label()`).
- Produces: the 24 `*_for_report` renderers (17 + 7) as `pub(crate)` free fns in `crate::posture::jpl::production_generation`, keeping their exact names.

- [ ] **Step 1: Golden fixture** — capture the 7 top-level renderers' current output (they read request-corpus builders that stay) with an old-vs-new equality test, per Task 2 Step 2, for: `production_generation_boundary_summary_for_report`, `…_boundary_source_summary_for_report`, `…_boundary_window_summary_for_report`, `…_boundary_body_class_coverage_summary_for_report`, `…_boundary_request_corpus_summary_for_report`, `…_boundary_request_corpus_equatorial_summary_for_report`, `validated_…_boundary_request_corpus_equatorial_summary_for_report`.

- [ ] **Step 2: Move `reference_summary/production_generation.rs` (17 renderers)** applying the recipe → `posture/jpl/production_generation.rs`.

- [ ] **Step 3: Move top-level `production_generation.rs` report half (7 renderers + inherent `summary_line`/`validated_summary_line` on the `ProductionGenerationBoundary*Summary` structs)** into the same validate module. KEEP in jpl: `production_generation_snapshot_entries`, `production_generation_snapshot_requests`, `production_generation_snapshot_request_corpus`, `production_generation_boundary_requests`, `production_generation_boundary_request_corpus`, and every struct's `validate()`/`label()`/`ProductionGenerationBoundaryWindow` calc.

- [ ] **Step 4: Register module** in `posture/jpl/mod.rs`; move report tests.

- [ ] **Step 5: Verify + commit**

Run: `cargo test -p pleiades-validate posture::jpl::production_generation` — PASS (golden + copied tests).
Run: `mise run ci` — PASS.
```bash
git add crates/pleiades-validate/src/posture/
git commit -m "refactor(validate): copy production-generation renderers into validate posture (Slice D expand)"
```

---

### Task 11: selected_asteroid renderers (reference_summary + data/)

> **EXPAND phase — copy only, delete NOTHING from jpl.** "Apply the recipe" /
> "Move" below means *copy into validate*; the jpl originals stay until Task 14.

**Files:**
- Create: `crates/pleiades-validate/src/posture/jpl/selected_asteroid.rs`
- Modify: `crates/pleiades-validate/src/posture/jpl/mod.rs`
- (`crates/pleiades-jpl/src/reference_summary/selected_asteroid.rs` — 16 renderers + 23 methods + 20 Display — and `crates/pleiades-jpl/src/data/selected_asteroid_2001.rs` / `selected_asteroid_2378498.rs` (1 renderer each) are read for the copy but **not modified**; deleted in Task 14.)

- [ ] **Step 1:** Apply the recipe to `reference_summary/selected_asteroid.rs` → `posture/jpl/selected_asteroid.rs` (copy the 16 free renderers + re-home inherent methods/Display as validate free fns); copy the report tests.

- [ ] **Step 2:** Copy `selected_asteroid_source_2451917_summary_for_report` (from `data/selected_asteroid_2001.rs`) and `selected_asteroid_source_2378498_summary_for_report` (from `data/selected_asteroid_2378498.rs`) into `posture/jpl/selected_asteroid.rs`, rewiring their `.validated_summary_line()`/`Display` calls per the recipe. The `SelectedAsteroidSource*Summary` data structs + `*_summary()` constructors stay in `data/` permanently; the 2 renderers' jpl originals are deleted in Task 14.

- [ ] **Step 3: Verify + commit**

Run: `cargo test -p pleiades-validate posture::jpl::selected_asteroid` — PASS.
Run: `mise run ci` — PASS.
```bash
git add crates/pleiades-validate/src/posture/
git commit -m "refactor(validate): copy selected-asteroid renderers into validate posture (Slice D expand)"
```

---

### Task 12: pleiades-data re-open — PackagedArtifactPhase2CorpusAlignmentSummary rendering

> **EXPAND phase — copy only.** Runs AFTER Tasks 4–11 (its renderer calls their
> validate copies). Copy the data renderer into validate; **do not** strip
> `pleiades-data` yet — its `summary_line`/`Display` deletion is part of the
> Task 14 contract sweep. The golden fixture compares `pleiades_data`'s
> still-present `summary_line()` against the validate copy.

**Files:**
- Create: `crates/pleiades-validate/src/posture/jpl/data_phase2_alignment.rs`
- Modify: `crates/pleiades-validate/src/posture/jpl/mod.rs`
- Test: golden fixture in the new validate module
- (`crates/pleiades-data/src/coverage/target.rs` is read for the copy but **not modified** here; its rendering is deleted in Task 14. If a field the renderer reads is not `pub`, promote just that field — the one permitted jpl/data edit during expand.)

**Interfaces:**
- Consumes: `pleiades_data::PackagedArtifactPhase2CorpusAlignmentSummary` (with now-`pub` fields it already exposes) + the validate posture/jpl field renderers from Tasks 4–11 (e.g. `reference_snapshot_source_summary_line`, `comparison_snapshot_source_summary_line`, `independent_holdout_source_summary_line`, `selected_asteroid_source_*_summary_line`, the production-generation boundary-source renderer).
- Produces: `pub(crate) fn packaged_artifact_phase2_corpus_alignment_summary_for_report(&PackagedArtifactPhase2CorpusAlignmentSummary) -> String` in `crate::posture::jpl::data_phase2_alignment`.

- [ ] **Step 1: Golden fixture**

Add to the new module (paste the `#[cfg(test)]` old-vs-new pattern), comparing `pleiades_data`'s current `PackagedArtifactPhase2CorpusAlignmentSummary::summary_line()` output for `packaged_artifact_phase2_corpus_alignment_summary_details()` against the new free renderer.

- [ ] **Step 2: Write the relocated renderer**

In `crates/pleiades-validate/src/posture/jpl/data_phase2_alignment.rs`, port the `summary_line()` body from `pleiades-data/src/coverage/target.rs:181-200` verbatim, rewriting each `self.<field>.summary_line()` to the corresponding validate posture/jpl free renderer for that jpl evidence type, and the `pleiades_jpl::format_production_generation_boundary_source_summary(&self.production_generation_boundary_source)` call to the validate posture/jpl production-generation boundary-source renderer. Signature: `pub(crate) fn packaged_artifact_phase2_corpus_alignment_summary_for_report(s: &PackagedArtifactPhase2CorpusAlignmentSummary) -> String`.

- [ ] **Step 3: Verify + commit** (no `pleiades-data` strip here — deferred to Task 14)

Run: `cargo test -p pleiades-validate posture::jpl::data_phase2_alignment` — PASS (golden old==new).
Run: `mise run ci` — PASS.
```bash
git add crates/pleiades-validate/src/posture/
git commit -m "refactor(validate): copy phase2-corpus-alignment rendering into validate posture (Slice D expand)"
```

---

### Task 13: REPOINT + SELF-CONTAIN — external consumers to validate, flip validate-internal residuals

> **Still no deletion from jpl.** This task makes validate's posture/jpl copies
> fully self-contained (no call into jpl rendering) and points every external
> consumer at them, so that after this task **nothing outside `pleiades-jpl`'s
> own crate renders via jpl**. jpl's render layer is still present (dead) — it
> is deleted in Task 14. Prerequisite: Tasks 2 + 4–12 all complete.

**Files:**
- Modify: `crates/pleiades-validate/src/posture/jpl/**` (flip residual `pleiades_jpl::*_for_report` / jpl inherent `.summary_line()` calls to local)
- Modify: `crates/pleiades-validate/src/lib.rs` (the `use pleiades_jpl::{ … _for_report … }` block, ~:313-430)
- Modify: `crates/pleiades-validate/src/report.rs`, `crates/pleiades-validate/src/render/summary/writers.rs:369` (and any other direct jpl-renderer/inherent-method call site)
- Modify: `crates/pleiades-validate/src/release/{bundle.rs,bundle_verify.rs,bundle_verify_helpers.rs}` (pin repoints — values unchanged)
- Modify: `crates/pleiades-cli/src/cli.rs` (~:170,174,221,331,350,553) and `crates/pleiades-cli/src/cli/tests/*.rs`

- [ ] **Step 1: Self-contain the validate copies (flip residuals to local)**

In `crates/pleiades-validate/src/posture/jpl/`, replace every remaining call into jpl rendering with its local equivalent: `pleiades_jpl::<name>_for_report()` → `crate::posture::jpl::…::<name>_for_report()` (or the module-local path), and any `x.summary_line()`/`x.validated_summary_line()`/`format!("{x}")` on a **jpl evidence struct** `x` → the local `<x_snake>_summary_line(&x)` free fn. Enumerate residuals: `grep -rn "pleiades_jpl::.*_for_report\|\.summary_line()\|\.validated_summary_line()" crates/pleiades-validate/src/posture/jpl`. (Calls on non-jpl types — `Instant::summary_line`, etc. — stay.)

- [ ] **Step 2: Repoint validate imports (lib.rs)**

In `crates/pleiades-validate/src/lib.rs`, delete the copied renderers from the `use pleiades_jpl::{…}` blocks and re-export them from `crate::posture::jpl::…` (mirroring the existing `pub(crate) use crate::posture::data::…` lines ~:270-301). KEEP importing from `pleiades_jpl` the staying items: `JplSnapshotBackend`, `interpolation_quality_samples`, `reference_asteroid_evidence`, `reference_asteroids`, `jpl_interpolation_posture_summary` (struct accessor), and any non-renderer data accessor. Preserve every `as <alias>` (e.g. `jpl_frame_treatment_summary_for_report`) so downstream names are unchanged.

- [ ] **Step 3: Repoint report.rs + writers.rs + other direct call sites**

`report.rs`'s 5 call sites resolve via the re-exports from Step 2 — confirm they compile. `render/summary/writers.rs:369` (`sample.summary_line()` on `InterpolationQualitySample`) → `crate::posture::jpl::backend::interpolation_quality_sample_summary_line(sample)`. Grep for any other validate call into jpl rendering and flip it.

- [ ] **Step 4: Repoint CLI**

In `crates/pleiades-cli/src/cli.rs` and `cli/tests/*.rs`, change `pleiades_jpl::<renderer>_for_report()` → `pleiades_validate::<renderer>_for_report()` for every copied renderer (CLI already depends on validate). Staying jpl symbols keep their `pleiades_jpl::` path.

- [ ] **Step 5: Bundle-pin sweep — confirm checksum VALUES unchanged**

Repoint any `pleiades_jpl::*_for_report` reference in `release/bundle*.rs` to the validate path. Then:

Run: `mise run release-smoke` (or `cargo test -p pleiades-validate release`)
Expected: PASS — **every fnv1a64 pin value identical to pre-slice**. If any pin value changed, a render body was not copied verbatim: fix the copy, do not update the pin.

- [ ] **Step 6: Verify + commit**

Run: `mise run ci` — PASS. (jpl still builds — its render layer is now dead but present.)
```bash
git add crates/pleiades-validate/src crates/pleiades-cli/src
git commit -m "refactor(validate,cli): repoint jpl renderers to validate posture + self-contain posture copies (Slice D)"
```

---

### Task 14: CONTRACT SWEEP — delete jpl + data render layer, finalize goldens, grep gates

> **The one deletion task.** After Task 13, no code outside `pleiades-jpl`'s own
> crate renders via jpl, and validate's posture copies are self-contained. Now
> delete jpl's entire render layer and the one `pleiades-data` renderer, and
> flip the three golden fixtures to captured literals (their `before` symbol is
> being deleted here).

**Files:**
- Modify (delete rendering): `crates/pleiades-jpl/src/backend.rs`, `crates/pleiades-jpl/src/production_generation.rs`, `crates/pleiades-jpl/src/reference_summary/**` (all `*.rs` + their `tests.rs`), `crates/pleiades-jpl/src/data/selected_asteroid_2001.rs`, `crates/pleiades-jpl/src/data/selected_asteroid_2378498.rs`
- Modify: `crates/pleiades-data/src/coverage/target.rs` (drop `PackagedArtifactPhase2CorpusAlignmentSummary::summary_line` ~:181 + its `impl Display` ~:341)
- Modify: `crates/pleiades-validate/src/posture/jpl/{backend,production_generation,data_phase2_alignment}.rs` (golden `before` → literal)

- [ ] **Step 1: Delete jpl's render layer**

In `pleiades-jpl`, delete every free `pub fn …_for_report`, every inherent `pub fn summary_line`/`pub fn validated_summary_line` on the evidence structs, and every `impl fmt::Display for <EvidenceStruct>`. **KEEP:** the structs + public fields, `validate()`/`label()`/`*_details()`/`*_summary()` constructors, private calc helpers, the request-corpus/snapshot builders, and every `impl fmt::Display for *ValidationError` (contract surface). Delete the jpl-side report tests that assert rendered strings (they moved to validate); keep `validate()`/`_details()`/contract tests. **Audit jpl-internal callers first:** `grep -rn "\.summary_line()\|\.validated_summary_line()\|_for_report(" crates/pleiades-jpl/src` — if a *staying* `validate()`/constructor/builder calls a rendering method, inline the needed literal or keep the minimal private helper it needs (do not keep the `pub` render surface).

- [ ] **Step 2: Delete the `pleiades-data` renderer**

In `crates/pleiades-data/src/coverage/target.rs` delete `PackagedArtifactPhase2CorpusAlignmentSummary::summary_line` (~:181) and its `impl fmt::Display` (~:341). KEEP the struct, fields, `validate()` (~:204), `_details()`. Repoint any residual in-crate/test caller (`grep -rn "PackagedArtifactPhase2CorpusAlignmentSummary" crates/pleiades-data`) — a data test that asserted the rendered line either moves to validate or drops (the byte-identity assertion now lives in validate's golden).

- [ ] **Step 3: Finalize the three golden fixtures**

In the backend / production_generation / data_phase2_alignment validate modules, replace each golden's `before` binding (which called the now-deleted jpl/data symbol) with the exact literal string it produced, captured via `cargo test … -- --nocapture` before the deletion. Keep them as regression pins.

- [ ] **Step 4: Grep gates (must all return nothing / expected)**

```bash
grep -rn "pub fn .*_for_report" crates/pleiades-jpl/src            # -> nothing
grep -rn "    pub fn summary_line\|    pub fn validated_summary_line" crates/pleiades-jpl/src/reference_summary  # -> nothing
grep -rn "pleiades_jpl::.*_for_report\|pleiades_jpl::format_.*summary" crates/pleiades-data/src crates/pleiades-validate/src/posture/jpl   # -> nothing
```

Also confirm no `impl fmt::Display` remains on a jpl evidence struct (error-enum Display allowed):
`grep -rn "impl fmt::Display for" crates/pleiades-jpl/src/reference_summary` should list only `*ValidationError` types.

- [ ] **Step 5: Verify + commit**

Run: `mise run ci` — PASS (full workspace; `release-smoke` checksums still identical; grep gates green). This is the byte-identity + pure-end-state proof.
```bash
git add crates/pleiades-jpl/src crates/pleiades-data/src crates/pleiades-validate/src
git commit -m "refactor(jpl,data): delete relocated render layer — pure end-state (Slice D contract)"
```

---

### Task 15: Close-out — CHANGELOG, PLAN.md

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `PLAN.md`

- [ ] **Step 1: Re-confirm the grep gates** (Task 14 already made them pass — re-run to be safe)

```bash
grep -rn "pub fn .*_for_report" crates/pleiades-jpl/src            # -> nothing
grep -rn "    pub fn summary_line\|    pub fn validated_summary_line" crates/pleiades-jpl/src/reference_summary  # -> nothing
grep -rn "pleiades_jpl::.*_for_report\|pleiades_jpl::format_.*summary" crates/pleiades-data/src   # -> nothing
```

- [ ] **Step 2: CHANGELOG entry**

Add under the unreleased section, matching the Slice C entry's voice: report prose for `pleiades-jpl` relocated into `pleiades-validate` posture (`posture/jpl/`); corpus/evidence accessors promoted to `pub`; `pleiades-data`'s phase-2 alignment rendering relocated; byte-identical output; no version bump.

- [ ] **Step 3: PLAN.md status refresh**

Update the report-surface relocation status line: Slice D done; **all four slices (A–D) complete**; only the workspace `0.3.0 → 0.4.0` bump + release-plz cut remains before the 0.4.0 release.

- [ ] **Step 4: Final CI + commit**

Run: `mise run ci` — PASS.
```bash
git add CHANGELOG.md PLAN.md
git commit -m "docs: report-surface relocation Slice D close-out (CHANGELOG, PLAN status)"
```

- [ ] **Step 5: Open the PR** (do not merge; release is a separate follow-up)

```bash
git push -u origin feat/report-surface-relocation-slice-d
gh pr create --title "Report-surface relocation Slice D (pleiades-jpl)" --body "Final slice of the report-surface relocation program. Relocates pleiades-jpl report prose (+ one dependent pleiades-data renderer) into pleiades-validate posture/jpl, byte-identical, no version bump. The 0.3.0->0.4.0 bump + release is a separate follow-up."
```

---

## Self-Review

**Spec coverage** (design §-by-§) — **updated for the EXPAND-then-CONTRACT amendment (2026-07-13):**
- Goal / pure-relocation / no-bump → Global Constraints + Tasks 1–15; version bump explicitly excluded (design Non-goals) → Task 15 Step 3 defers to a follow-up. ✓
- Inherent-method relocation novelty → the transformation recipe + Task 2 worked example (copy inherent rendering into validate free fns). ✓
- Stay/move partition → recipe Step 1 (stay) / Step 2 (copy into validate); pure end-state reached + enforced by **Task 14** grep gates (deletion is the contract sweep, not per-family). ✓
- Accessor-promotion prerequisite → Task 1. ✓
- Coupling 1 (backend pair) → Task 2. Coupling 2 (production_generation) → Task 10. Coupling 3 (data/selected_asteroid) → Task 11. Coupling 4 (pleiades-data re-open) → Task 12. Dependency direction → EXPAND copies never delete, so jpl never needs to call validate; verified by build. ✓
- Module map (`posture/jpl/`) → Tasks 2,8,9 create the tree. ✓
- Consumer migrations (validate/cli/data) → **Task 13** (repoint + self-contain external + validate-internal) + **Task 14** (delete jpl + data render layer). ✓
- Invariants: byte-identity → copied tests + release-smoke (every task) + 3 golden fixtures (finalized Task 14); compat profile stable → release-smoke; no behavior → validate() stays; dependency direction → EXPAND additive-only + build. ✓
- Verification (mise run ci, release-smoke, grep, golden fixtures) → per-task + Task 14 (grep gates) + Task 15. ✓
- 129 + 9 = 138 renderers all assigned: comparison 11 (T4), holdout 13 (T5), jpl_posture 16 (T6), reference_asteroid 4 (T7), reference_snapshot core 46 (T8), reference_snapshot boundaries 6 (T9), reference_summary/production_generation 17 + top-level production_generation 7 (T10), selected_asteroid 16 + data selected_asteroid 2 (T11) = 138. ✓

**Placeholder scan:** No "TBD/TODO/add error handling". The family Tasks 4–9 reference the explicit transformation recipe (a fully-specified mechanical procedure with a worked example in Task 2 and per-file grep enumeration) rather than re-transcribing ~24k LOC verbatim — this is precise, not a "similar to Task N" hand-wave. ✓

**Type consistency:** `<s_snake>_summary_line(&S) -> String` render-helper convention is used identically in the recipe, Task 2 (`jpl_interpolation_quality_sample_summary_line`), and Task 12 (rewiring field renderers). Moved `*_for_report` functions keep their exact pre-move names so validate re-exports (Task 13) and the bundle pins resolve unchanged. Task 1's promoted accessor signatures match their consumption in Tasks 2/7/8/9. ✓
