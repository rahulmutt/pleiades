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

## The transformation recipe (referenced by every family task)

Slice D is a large *mechanical* relocation. Task 2 works one struct end-to-end with real code; **Tasks 4–11 apply this exact recipe** to enumerated symbols. The recipe, per source file `crates/pleiades-jpl/src/<path>.rs` → `crates/pleiades-validate/src/posture/jpl/<path>.rs`:

For each **evidence struct** `S` in the file:

1. **Keep in jpl, unchanged:** the `struct S`, its public fields, `impl S { fn validate(), fn label(), private helpers like body_count() }`, its `*_details()`/`*_summary()` constructors, and `impl Display for SValidationError` (error Display is contract surface, not report prose — it stays).
2. **Remove from jpl:** `impl S { pub fn summary_line(&self) -> String, pub fn validated_summary_line(&self) -> Result<String, …> }` and `impl Display for S`.
3. **Add to validate** `posture/jpl/<path>.rs`:
   - `fn <s_snake>_summary_line(s: &S) -> String { <verbatim body of the old S::summary_line, with self→s; any nested X::summary_line(w) call rewritten to the local free fn <x_snake>_summary_line(w)> }` — `pub(crate)`.
   - The file's existing free `pub fn …_for_report()` functions, **moved verbatim**, except internal `summary.summary_line()` → `<s_snake>_summary_line(&summary)`, `summary.validated_summary_line()` → `{ summary.validate().map_err(|e| e.to_string())?; Ok(<s_snake>_summary_line(&summary)) }` (validate stays in jpl; rendering is local), and `format!("{summary}")`/`summary.to_string()` (which used `Display`) → `<s_snake>_summary_line(&summary)`.
4. **Move the file's report tests** (the ones asserting rendered strings / calling `_for_report`) into the same validate module, verbatim. Leave `validate()`/`_details()`/contract tests in jpl.

Each moved `_for_report` keeps its exact name (validate re-exports by name in Task 13). Byte-identity is guaranteed because the render bodies are copied verbatim; it is *verified* by the moved unit tests (which assert exact strings) plus `release-smoke`.

Naming: `posture/jpl/mod.rs` gains `#[allow(unused_imports)] pub(crate) use <module>::*;` per moved file so re-exports resolve, mirroring `pleiades-jpl/src/reference_summary/mod.rs`.

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

**Files:**
- Create: `crates/pleiades-validate/src/posture/jpl/mod.rs`
- Create: `crates/pleiades-validate/src/posture/jpl/backend.rs`
- Modify: `crates/pleiades-validate/src/posture/mod.rs` (register `jpl`)
- Modify: `crates/pleiades-jpl/src/backend.rs` (strip rendering)
- Test: golden-fixture equality test in `crates/pleiades-validate/src/posture/jpl/backend.rs`

**Interfaces:**
- Consumes: `pleiades_jpl::{InterpolationQualitySample, SnapshotManifestSummary, interpolation_quality_sample_list}` (Task 1).
- Produces: `pub(crate) fn jpl_interpolation_quality_sample_summary_line(&InterpolationQualitySample) -> String`, `pub(crate) fn format_jpl_interpolation_quality_summary_for_report() -> String`, `pub(crate) fn snapshot_manifest_summary_line(&SnapshotManifestSummary) -> String` (+ the other backend renderers enumerated in Step 5), in `crate::posture::jpl::backend`.

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

- [ ] **Step 2: Capture the pre-move golden fixture**

Before touching jpl, capture current output. Create `crates/pleiades-validate/src/posture/jpl/backend.rs` with only the fixture test for now:

```rust
//! Relocated backend-struct renderers (InterpolationQualitySample,
//! SnapshotManifestSummary) from `pleiades-jpl::backend` (Slice D).

#[cfg(test)]
mod golden {
    // Captured from `pleiades-jpl` HEAD before the strip. Any drift after the
    // move is a Slice-D defect, not a value to regenerate.
    #[test]
    fn interpolation_quality_summary_byte_identical() {
        let before = pleiades_jpl::format_jpl_interpolation_quality_summary_for_report();
        let after = super::format_jpl_interpolation_quality_summary_for_report();
        assert_eq!(before, after);
    }
}
```

Run: `cargo test -p pleiades-validate posture::jpl::backend::golden -- --ignored 2>&1 | head` — expect a compile error (`super::format_…` not yet defined). This confirms the fixture references the real pre-move symbol; proceed to define the moved renderer, then this test compares old vs new until Task 13 removes the jpl symbol (at which point convert `before` to an inlined string literal captured from a `cargo run` of the pre-move renderer — see Step 6).

- [ ] **Step 3: Add the stripped renderers to validate**

In `crates/pleiades-validate/src/posture/jpl/backend.rs`, above the test module, add (bodies copied verbatim from the jpl inherent methods, `self`→`s`):

```rust
use pleiades_jpl::{
    interpolation_quality_sample_list, InterpolationQualitySample, SnapshotManifestSummary,
};

/// Compact release-facing summary line for one interpolation-quality sample.
pub(crate) fn jpl_interpolation_quality_sample_summary_line(s: &InterpolationQualitySample) -> String {
    format!(
        "{} at {}: {} interpolation, bracket span {:.1} d, |Δlon|={:.12}°, |Δlat|={:.12}°, |Δdist|={:.12} AU",
        s.body,
        s.epoch.summary_line(),
        s.interpolation_kind.label(),
        s.bracket_span_days,
        s.longitude_error_deg,
        s.latitude_error_deg,
        s.distance_error_au,
    )
}
```

Then move the free `format_jpl_interpolation_quality_summary_for_report()` body verbatim from `pleiades-jpl/src/backend.rs`, rewriting any `sample.summary_line()` / `sample.to_string()` to `jpl_interpolation_quality_sample_summary_line(sample)` and sourcing samples via `interpolation_quality_sample_list()`.

- [ ] **Step 4: Strip the rendering from jpl backend.rs**

In `crates/pleiades-jpl/src/backend.rs` delete: `impl InterpolationQualitySample { pub fn summary_line … }` (keep `validate()`), the `impl fmt::Display for InterpolationQualitySample` (~:167), and the free `format_jpl_interpolation_quality_summary_for_report`. Keep the struct, its public fields, `validate()`, and `InterpolationQualitySampleValidationError` + its Display.

- [ ] **Step 5: Repeat for `SnapshotManifestSummary`**

Apply the recipe to `SnapshotManifestSummary` (~:585–760 in jpl backend.rs) and its free renderers: move `summary_line`/`summary_line_with_defaults` bodies → `snapshot_manifest_summary_line(&SnapshotManifestSummary, …)` free fn(s) in validate; move the free `*_for_report`; strip `Display` + inherent rendering from jpl; keep `validate()` + the `validated_*` contract methods that jpl's own manifest-provenance path calls. (Audit: if `validate()` internally calls `summary_line()`, inline the needed literal there — verify with `grep -n "summary_line" crates/pleiades-jpl/src/backend.rs` returning only staying-method references.)

- [ ] **Step 6: Run the golden fixture, then CI**

Run: `cargo test -p pleiades-validate posture::jpl::backend`
Expected: PASS (old == new). Because Task 13 later deletes the jpl symbol, at that point replace the `before` binding with the exact string the pre-move renderer produced (paste the literal captured now via `cargo test … -- --nocapture` printing `before`).

Run: `mise run ci`
Expected: PASS, including `release-smoke` (checksums unchanged).

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-jpl/src/backend.rs crates/pleiades-validate/src/posture/
git commit -m "refactor(jpl,validate): relocate backend struct renderers into validate posture (Slice D coupling)"
```

---

### Tasks 4–9: reference_summary family moves (apply the Task-2 recipe)

Each task moves one file/subtree of `crates/pleiades-jpl/src/reference_summary/` → `crates/pleiades-validate/src/posture/jpl/`, applying the recipe: strip inherent `summary_line`/`validated_summary_line`/`Display` into `pub(crate)` free render fns, move the free `*_for_report` verbatim (rewiring internal calls), move the report tests, register the module in `posture/jpl/mod.rs`. Structs, `validate()`, `label()`, `*_details()` constructors, and `ValidationError` Display stay in jpl.

**Per-task verification (identical, run at each task's end):**
- `cargo build -p pleiades-jpl && cargo build -p pleiades-validate` — PASS.
- `cargo test -p pleiades-validate posture::jpl::<module>` — PASS (moved report tests assert exact strings ⇒ byte-identity).
- `grep -rn "pub fn .*_for_report\|pub fn summary_line\|pub fn validated_summary_line\|impl fmt::Display for <StructName>" crates/pleiades-jpl/src/reference_summary/<file>.rs` — returns nothing for the moved structs (error-enum Display excluded).
- `mise run ci` — PASS (`release-smoke` checksums unchanged). Commit on green.

Enumerate a file's symbols to move with:
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

**Files:**
- Create: `crates/pleiades-validate/src/posture/jpl/production_generation.rs`
- Modify: `crates/pleiades-jpl/src/reference_summary/production_generation.rs` (strip 17 renderers)
- Modify: `crates/pleiades-jpl/src/production_generation.rs` (strip 7 renderers + inherent methods; keep request-corpus builders)
- Modify: `crates/pleiades-validate/src/posture/jpl/mod.rs`
- Test: golden fixture in the new validate module

**Interfaces:**
- Consumes: `pleiades_jpl::{ProductionGenerationBoundarySummary, ProductionGenerationBoundaryWindow, ProductionGenerationBoundaryWindowSummary, ProductionGenerationBoundaryBodyClassCoverageSummary, ProductionGenerationBoundaryRequestCorpusSummary, IndependentHoldoutSourceSummary, …}` (structs + their `validate()`/`label()`).
- Produces: the 24 `*_for_report` renderers (17 + 7) as `pub(crate)` free fns in `crate::posture::jpl::production_generation`, keeping their exact names.

- [ ] **Step 1: Golden fixture** — capture the 7 top-level renderers' current output (they read request-corpus builders that stay) with an old-vs-new equality test, per Task 2 Step 2, for: `production_generation_boundary_summary_for_report`, `…_boundary_source_summary_for_report`, `…_boundary_window_summary_for_report`, `…_boundary_body_class_coverage_summary_for_report`, `…_boundary_request_corpus_summary_for_report`, `…_boundary_request_corpus_equatorial_summary_for_report`, `validated_…_boundary_request_corpus_equatorial_summary_for_report`.

- [ ] **Step 2: Move `reference_summary/production_generation.rs` (17 renderers)** applying the recipe → `posture/jpl/production_generation.rs`.

- [ ] **Step 3: Move top-level `production_generation.rs` report half (7 renderers + inherent `summary_line`/`validated_summary_line` on the `ProductionGenerationBoundary*Summary` structs)** into the same validate module. KEEP in jpl: `production_generation_snapshot_entries`, `production_generation_snapshot_requests`, `production_generation_snapshot_request_corpus`, `production_generation_boundary_requests`, `production_generation_boundary_request_corpus`, and every struct's `validate()`/`label()`/`ProductionGenerationBoundaryWindow` calc.

- [ ] **Step 4: Register module** in `posture/jpl/mod.rs`; move report tests.

- [ ] **Step 5: Verify + commit**

Run: `cargo test -p pleiades-validate posture::jpl::production_generation` — PASS (golden + moved tests).
Run: `mise run ci` — PASS.
```bash
git add crates/pleiades-jpl/src/production_generation.rs crates/pleiades-jpl/src/reference_summary/production_generation.rs crates/pleiades-validate/src/posture/
git commit -m "refactor(jpl,validate): relocate production-generation renderers (Slice D)"
```

---

### Task 11: selected_asteroid renderers (reference_summary + data/)

**Files:**
- Create: `crates/pleiades-validate/src/posture/jpl/selected_asteroid.rs`
- Modify: `crates/pleiades-jpl/src/reference_summary/selected_asteroid.rs` (16 renderers + 23 methods + 20 Display)
- Modify: `crates/pleiades-jpl/src/data/selected_asteroid_2001.rs` and `crates/pleiades-jpl/src/data/selected_asteroid_2378498.rs` (1 renderer each)
- Modify: `crates/pleiades-validate/src/posture/jpl/mod.rs`

- [ ] **Step 1:** Apply the recipe to `reference_summary/selected_asteroid.rs` → `posture/jpl/selected_asteroid.rs` (16 free + strip inherent methods/Display); move report tests.

- [ ] **Step 2:** Move `selected_asteroid_source_2451917_summary_for_report` (from `data/selected_asteroid_2001.rs`) and `selected_asteroid_source_2378498_summary_for_report` (from `data/selected_asteroid_2378498.rs`) into `posture/jpl/selected_asteroid.rs`, rewiring their `.validated_summary_line()`/`Display` calls per the recipe. KEEP the `SelectedAsteroidSource*Summary` data structs + `*_summary()` constructors in `data/`.

- [ ] **Step 3: Verify + commit**

Run: `cargo test -p pleiades-validate posture::jpl::selected_asteroid` — PASS.
Run: `mise run ci` — PASS.
```bash
git add crates/pleiades-jpl/src/reference_summary/selected_asteroid.rs crates/pleiades-jpl/src/data/ crates/pleiades-validate/src/posture/
git commit -m "refactor(jpl,validate): relocate selected-asteroid renderers (Slice D)"
```

---

### Task 12: pleiades-data re-open — PackagedArtifactPhase2CorpusAlignmentSummary rendering

**Files:**
- Create: `crates/pleiades-validate/src/posture/jpl/data_phase2_alignment.rs`
- Modify: `crates/pleiades-data/src/coverage/target.rs` (drop `summary_line`/`Display` on the struct)
- Modify: `crates/pleiades-validate/src/posture/jpl/mod.rs`
- Test: golden fixture in the new validate module

**Interfaces:**
- Consumes: `pleiades_data::PackagedArtifactPhase2CorpusAlignmentSummary` (with now-`pub` fields it already exposes) + the validate posture/jpl field renderers from Tasks 4–11 (e.g. `reference_snapshot_source_summary_line`, `comparison_snapshot_source_summary_line`, `independent_holdout_source_summary_line`, `selected_asteroid_source_*_summary_line`, the production-generation boundary-source renderer).
- Produces: `pub(crate) fn packaged_artifact_phase2_corpus_alignment_summary_for_report(&PackagedArtifactPhase2CorpusAlignmentSummary) -> String` in `crate::posture::jpl::data_phase2_alignment`.

- [ ] **Step 1: Golden fixture**

Add to the new module (paste the `#[cfg(test)]` old-vs-new pattern), comparing `pleiades_data`'s current `PackagedArtifactPhase2CorpusAlignmentSummary::summary_line()` output for `packaged_artifact_phase2_corpus_alignment_summary_details()` against the new free renderer.

- [ ] **Step 2: Write the relocated renderer**

In `crates/pleiades-validate/src/posture/jpl/data_phase2_alignment.rs`, port the `summary_line()` body from `pleiades-data/src/coverage/target.rs:181-200` verbatim, rewriting each `self.<field>.summary_line()` to the corresponding validate posture/jpl free renderer for that jpl evidence type, and the `pleiades_jpl::format_production_generation_boundary_source_summary(&self.production_generation_boundary_source)` call to the validate posture/jpl production-generation boundary-source renderer. Signature: `pub(crate) fn packaged_artifact_phase2_corpus_alignment_summary_for_report(s: &PackagedArtifactPhase2CorpusAlignmentSummary) -> String`.

- [ ] **Step 3: Strip rendering from pleiades-data**

In `crates/pleiades-data/src/coverage/target.rs` delete `impl PackagedArtifactPhase2CorpusAlignmentSummary { pub fn summary_line … }` (~:181) and `impl fmt::Display for PackagedArtifactPhase2CorpusAlignmentSummary` (~:341). KEEP the struct, its fields, `validate()` (~:204), and `_details()`. Repoint any in-crate/test caller of `.summary_line()`/`Display` on this struct to the validate renderer (audit with `grep -rn "PackagedArtifactPhase2CorpusAlignmentSummary" crates/pleiades-data crates/pleiades-validate crates/pleiades-cli`).

- [ ] **Step 4: Verify + commit**

Run: `cargo test -p pleiades-validate posture::jpl::data_phase2_alignment` — PASS (golden).
Run: `mise run ci` — PASS.
```bash
git add crates/pleiades-data/src/coverage/target.rs crates/pleiades-validate/src/posture/
git commit -m "refactor(data,validate): relocate phase2-corpus-alignment rendering into validate posture (Slice D)"
```

---

### Task 13: Consumer repoint + bundle-pin sweep

**Files:**
- Modify: `crates/pleiades-validate/src/lib.rs` (the `use pleiades_jpl::{ … _for_report … }` block, ~:313-430)
- Modify: `crates/pleiades-validate/src/report.rs` (5 direct jpl-renderer call sites)
- Modify: `crates/pleiades-validate/src/release/{bundle.rs,bundle_verify.rs,bundle_verify_helpers.rs}` (pin repoints — values unchanged)
- Modify: `crates/pleiades-cli/src/cli.rs` (~:170,174,221,331,350,553) and `crates/pleiades-cli/src/cli/tests/*.rs`

**Interfaces:**
- Consumes: every moved renderer, now at `crate::posture::jpl::…` (validate) / `pleiades_validate::…` (cli).

- [ ] **Step 1: Repoint validate imports**

In `crates/pleiades-validate/src/lib.rs`, delete the moved renderers from the `use pleiades_jpl::{…}` blocks and re-export them from `crate::posture::jpl::…` (mirroring the existing `pub(crate) use crate::posture::data::…` lines ~:270-301). KEEP importing from `pleiades_jpl` the staying items: `JplSnapshotBackend`, `interpolation_quality_samples`, `reference_asteroid_evidence`, `reference_asteroids`, `jpl_interpolation_posture_summary` (struct accessor), and any non-renderer data accessor. Preserve every `as <alias>` (e.g. `jpl_frame_treatment_summary_for_report`) so downstream names are unchanged.

- [ ] **Step 2: Repoint report.rs**

In `crates/pleiades-validate/src/report.rs`, the 5 call sites (`comparison_snapshot_summary_for_report()` ~:388, `comparison_snapshot_body_class_coverage_summary_for_report()` ~:392, etc.) resolve via the re-exports from Step 1 — confirm they compile unchanged.

- [ ] **Step 3: Repoint CLI**

In `crates/pleiades-cli/src/cli.rs` and `cli/tests/*.rs`, change `pleiades_jpl::<renderer>_for_report()` → `pleiades_validate::<renderer>_for_report()` for every moved renderer (CLI already depends on validate). Staying jpl symbols keep their `pleiades_jpl::` path.

- [ ] **Step 4: Bundle-pin sweep — confirm checksum VALUES unchanged**

Repoint any `pleiades_jpl::*_for_report` reference in `release/bundle*.rs` to the validate path. Then:

Run: `mise run release-smoke` (or `cargo test -p pleiades-validate release`) 
Expected: PASS — **every fnv1a64 pin value identical to pre-slice**. If any pin value changed, a render body was not copied verbatim: fix the move, do not update the pin.

- [ ] **Step 5: Finalize Task-2/10/12 golden fixtures**

Now that the jpl symbols are gone, replace the `before` bindings in the three golden fixtures with the literal strings captured earlier (they can no longer call the deleted jpl renderer). Keep them as regression pins.

- [ ] **Step 6: Verify + commit**

Run: `mise run ci` — PASS.
```bash
git add crates/pleiades-validate/src crates/pleiades-cli/src
git commit -m "refactor(validate,cli): repoint jpl renderers to validate posture + bundle pins (Slice D)"
```

---

### Task 14: Close-out — grep gates, CHANGELOG, PLAN.md

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `PLAN.md`

- [ ] **Step 1: Grep assertions (must all return nothing / expected)**

```bash
grep -rn "pub fn .*_for_report" crates/pleiades-jpl/src            # -> nothing
grep -rn "    pub fn summary_line\|    pub fn validated_summary_line" crates/pleiades-jpl/src/reference_summary  # -> nothing
grep -rn "pleiades_jpl::.*_for_report\|pleiades_jpl::format_.*summary" crates/pleiades-data/src   # -> nothing
```

Also confirm no `impl fmt::Display` remains on a jpl evidence struct (error-enum Display allowed):
`grep -rn "impl fmt::Display for" crates/pleiades-jpl/src/reference_summary` should list only `*ValidationError` types.

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

**Spec coverage** (design §-by-§):
- Goal / pure-relocation / no-bump → Global Constraints + Tasks 1–14; version bump explicitly excluded (design Non-goals) → Task 14 Step 3 defers to a follow-up. ✓
- Inherent-method strip novelty → the transformation recipe + Task 2 worked example. ✓
- Stay/move partition → recipe Step 1 (stay) / Steps 2–4 (move); enforced by Task 14 grep gates. ✓
- Accessor-promotion prerequisite → Task 1. ✓
- Coupling 1 (backend pair) → Task 2. Coupling 2 (production_generation) → Task 10. Coupling 3 (data/selected_asteroid) → Task 11. Coupling 4 (pleiades-data re-open) → Task 12. Dependency direction → Global Constraints + verified by build. ✓
- Module map (`posture/jpl/`) → Tasks 2,8,9 create the tree. ✓
- Consumer migrations (validate/cli/data) → Tasks 12 (data) + 13 (validate/cli/bundle). ✓
- Invariants: byte-identity → moved tests + release-smoke (every task) + 3 golden fixtures; compat profile stable → release-smoke; no behavior → validate() stays; dependency direction → build. ✓
- Verification (mise run ci, release-smoke, grep, golden fixtures) → per-task + Task 14. ✓
- 129 + 9 = 138 renderers all assigned: comparison 11 (T4), holdout 13 (T5), jpl_posture 16 (T6), reference_asteroid 4 (T7), reference_snapshot core 46 (T8), reference_snapshot boundaries 6 (T9), reference_summary/production_generation 17 + top-level production_generation 7 (T10), selected_asteroid 16 + data selected_asteroid 2 (T11) = 138. ✓

**Placeholder scan:** No "TBD/TODO/add error handling". The family Tasks 4–9 reference the explicit transformation recipe (a fully-specified mechanical procedure with a worked example in Task 2 and per-file grep enumeration) rather than re-transcribing ~24k LOC verbatim — this is precise, not a "similar to Task N" hand-wave. ✓

**Type consistency:** `<s_snake>_summary_line(&S) -> String` render-helper convention is used identically in the recipe, Task 2 (`jpl_interpolation_quality_sample_summary_line`), and Task 12 (rewiring field renderers). Moved `*_for_report` functions keep their exact pre-move names so validate re-exports (Task 13) and the bundle pins resolve unchanged. Task 1's promoted accessor signatures match their consumption in Tasks 2/7/8/9. ✓
