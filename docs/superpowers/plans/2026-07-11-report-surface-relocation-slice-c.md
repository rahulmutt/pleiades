# Report-Surface Relocation Slice C Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Relocate `pleiades-data`'s free report-prose renderers into `pleiades-validate/src/posture/data/`, verbatim, and decouple the packaged-data backend metadata from the five report strings it embeds — with byte-identical rendered output.

**Architecture:** Pure relocation. Each `*_for_report` / `format_*_summary(&Struct)` free function moves into a per-module posture file under `pleiades-validate`, `pub(crate)` by default. The `Packaged*Summary` structs, their `*_summary_details()` constructors, their inherent `summary_line()` / `validated_summary_line()` / `validate()` methods, the `&'static str` self-description accessors, and the release-gate data (`accuracy_ceiling`, `PACKAGED_BUDGETS`, the accuracy-baseline measurement core, the regeneration pipeline) all stay in `pleiades-data` as structured data. The one calculation-path consumer — `PackagedDataBackend::metadata()`, which embeds five rendered strings in `BackendProvenance.data_sources` — is decoupled by rebuilding those strings inline from the retained accessors/methods. Consumers in validate and the CLI repoint; release-bundle checksum values are unchanged because text is byte-identical.

**Tech Stack:** Rust workspace (edition 2021, rust-version 1.97.0), `cargo`, `mise` task runner. Test framework: built-in `cargo test`. CI: `mise run ci`.

## Global Constraints

- **Byte-identical output.** Every moved renderer produces exactly the prose it produced before. fnv1a64 release-bundle checksum values must not change. Any drift is a defect in the move, never a checksum to regenerate.
- **No version bumps.** Compatibility profile stays `0.7.13` (and `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` unchanged); API-stability profile stays `0.3.0`. No `Cargo.toml` version edits in this slice. The workspace `0.4.0` bump is deferred to post-Slice-D.
- **No behavior / gate / threshold / corpus changes.** `validate-corpus`, claims-audit, and the numeric gates keep enforcing the same data. `accuracy_ceiling`, `PACKAGED_BUDGETS`, the accuracy-baseline measurement core, and the regeneration pipeline stay in `pleiades-data`.
- **Dependency direction.** Nothing below `pleiades-validate` may gain a dependency on it. The `&'static str` self-description accessors (`packaged_request_policy_summary`, `packaged_frame_treatment_summary`, `packaged_artifact_storage_summary`, `packaged_artifact_access_summary`) and the `Packaged*Summary` structs stay in `pleiades-data` for exactly this reason.
- **Visibility:** moved symbols are `pub(crate)` unless a genuine cross-crate runtime consumer exists (enumerated per task). Posture modules carry `#![allow(dead_code)]` as in Slices A/B.
- **Partition rule:** MOVE free functions that render prose (`…_for_report`, `format_validated_…_for_report`, `format_*_summary(&Struct) -> String`, and `packaged_body_coverage_summary() -> String`). KEEP `*_summary()`/`*_details()` struct constructors, the `&'static str` accessors, inherent `summary_line()`/`validated_summary_line()`/`validate()` methods, and all data structs.
- **Branch:** `feat/report-surface-relocation-slice-c` (create it off `main` at execution start; the spec commit `b479b005` and this plan are on `main` and will be in the branch's history).
- **Commit discipline:** one commit per task. Conventional-commit messages; no `!` (no public signature is removed without a kept structured accessor — the removed `_for_report` symbols are dead public surface replaced by structured data).

## Relocation procedure (applied by every move task)

For each free function `F` being moved from `crates/pleiades-data/src/<mod>.rs` to `crates/pleiades-validate/src/posture/data/<mod>.rs`:

1. Copy `F`'s full body (and any private helper it solely owns, e.g. a `format_validated_*` it wraps) into the destination module, unchanged. Adjust `use` paths so it references `pleiades-data`'s retained public items (`Packaged*Summary` structs, `*_details()` constructors, `&'static str` accessors, inherent methods) via `pleiades_data::…`.
2. Copy `F`'s unit tests (the `#[test]` fns asserting on `F`'s output) into the destination module's `#[cfg(test)] mod tests`, unchanged except import paths.
3. Delete `F` and its moved tests from `pleiades-data`.
4. Repoint every caller (in-crate, validate, CLI) to the new path.
5. Verify: the moved tests pass at the new location, `pleiades-data` and all consumers build, and `F` no longer appears as an exported symbol of `pleiades-data`.

"Verbatim" means the rendered `String` is identical; only module path and visibility change.

## Renderer inventory (the MOVE set — free `pub fn` renderers only)

Struct inherent methods `summary_line(&self)` / `validated_summary_line(&self)` are **not** in this set — they stay with their structs. Only these free top-level functions move:

- **coverage/profile.rs (7):** `packaged_artifact_production_profile_summary_for_report`, `packaged_artifact_generation_manifest_for_report`, `packaged_artifact_generation_manifest_checksum_for_report`, `packaged_artifact_profile_coverage_summary_for_report`, `packaged_artifact_profile_summary_with_output_support_for_report`, `packaged_artifact_output_support_summary_for_report`, `packaged_artifact_speed_policy_summary_for_report`.
- **coverage/fit.rs (1):** `packaged_artifact_fit_channel_outlier_summary_for_report`.
- **coverage/target.rs (5):** `packaged_artifact_target_threshold_state_for_report`, `packaged_artifact_phase2_corpus_alignment_summary_for_report`, `packaged_artifact_target_threshold_summary_for_report`, `packaged_artifact_source_fit_holdout_sync_summary_for_report`, `packaged_artifact_target_threshold_scope_envelopes_for_report`.
- **coverage/threshold.rs (6):** `packaged_artifact_fit_envelope_summary_for_report`, `packaged_artifact_fit_outlier_summary_for_report`, `packaged_artifact_fit_threshold_summary_for_report`, `packaged_artifact_fit_margin_summary_for_report`, `packaged_artifact_fit_threshold_violation_count_for_report`, `packaged_artifact_fit_threshold_violation_summary_for_report`.
- **coverage/regen.rs (2):** `packaged_artifact_regeneration_summary_for_report`, `packaged_artifact_normalized_intermediate_summary_for_report`.
- **coverage/generation.rs (2):** `packaged_artifact_generation_policy_summary_for_report`, `packaged_artifact_generation_residual_bodies_summary_for_report`.
- **coverage/body.rs (2):** `packaged_body_coverage_summary` (the `String` renderer) + its `pub(crate)` helper `format_validated_packaged_body_coverage_summary_for_report`.
- **lookup.rs (7 move + 1 demote):** MOVE `packaged_lookup_epoch_policy_summary_for_report`, `packaged_request_policy_summary_for_report`, `packaged_frame_treatment_summary_for_report`, `packaged_artifact_storage_summary_for_report`, `packaged_artifact_access_summary_for_report`, `packaged_frame_parity_summary_for_report`, `packaged_mixed_tt_tdb_batch_parity_summary_for_report`. DEMOTE (stays, `pub`→`pub(crate)`) `packaged_mixed_frame_batch_parity_summary_for_report`.
- **regenerate.rs (3):** `packaged_artifact_body_class_span_cap_summary_for_report`, `packaged_artifact_body_class_span_cap_entries_for_report`, `packaged_artifact_body_cadence_summary_for_report`.
- **thresholds.rs (1):** `packaged_artifact_thresholds_summary_for_report`.
- **accuracy_baseline.rs (1):** `packaged_artifact_accuracy_baseline_summary_for_report`.

**Demotions (stay in `pleiades-data`, `pub`→`pub(crate)`; 0 non-test consumers confirmed):** `packaged_mixed_frame_batch_parity_summary_for_report` (lookup.rs), `packaged_body_coverage_summary_details` (coverage/body.rs), `packaged_artifact_fit_channel_outlier_summary_details` (coverage/fit.rs), `eros_self_consistency_max_longitude_arcsec` (accuracy_baseline.rs).

## File Structure

```
crates/pleiades-validate/src/posture/
  mod.rs                          MODIFY  — add `pub(crate) mod data;`
  data/
    mod.rs                        CREATE  — `pub(crate) mod coverage; pub(crate) mod lookup; ...`
    coverage/
      mod.rs                      CREATE  — `pub(crate) mod body; ...` (7 submodules)
      profile.rs                  CREATE  — 7 renderers + tests
      fit.rs                      CREATE  — 1 renderer + tests
      target.rs                   CREATE  — 5 renderers + tests
      threshold.rs                CREATE  — 6 renderers + tests
      regen.rs                    CREATE  — 2 renderers + tests
      generation.rs               CREATE  — 2 renderers + tests
      body.rs                     CREATE  — packaged_body_coverage_summary (+ helper) + tests
    lookup.rs                     CREATE  — 7 renderers + tests
    regenerate.rs                 CREATE  — 3 renderers + tests
    thresholds.rs                 CREATE  — 1 renderer + tests
    accuracy_baseline.rs          CREATE  — 1 renderer + tests
```

Source crate modified (renderers + their tests removed; structs/constructors/accessors/methods/gate-data retained):
`pleiades-data/src/{coverage/{profile,fit,target,threshold,regen,generation,body}.rs, lookup.rs, regenerate.rs, thresholds.rs, accuracy_baseline.rs, backend.rs}`.

Consumers repointed: `pleiades-validate/src/**` (renderer call sites + `release/{bundle.rs,bundle_verify_helpers.rs}` checksum pins), `pleiades-cli/src/**` (any `pleiades_data` report-symbol imports + summary-command tests).

---

### Task 1: Scaffold posture/data submodules

**Files:**
- Modify: `crates/pleiades-validate/src/posture/mod.rs`
- Create: `crates/pleiades-validate/src/posture/data/mod.rs`, `.../data/coverage/mod.rs`, and the 12 leaf module files (empty shells).

**Interfaces:**
- Produces: `crate::posture::data::{coverage::{profile,fit,target,threshold,regen,generation,body}, lookup, regenerate, thresholds, accuracy_baseline}` module paths that later tasks fill.

- [ ] **Step 1: Add the `data` module declaration**

In `crates/pleiades-validate/src/posture/mod.rs`, add (keeping the list alphabetical):

```rust
pub(crate) mod data;
```

- [ ] **Step 2: Create `data/mod.rs`**

```rust
//! `pleiades-data` report/summary prose relocated from the functional crate
//! (report-surface relocation program, Slice C). Rendering only — the
//! functional crate keeps the structured data, the `&'static str` accessors,
//! its inherent methods, and all release-gate data.
#![allow(dead_code)]

pub(crate) mod accuracy_baseline;
pub(crate) mod coverage;
pub(crate) mod lookup;
pub(crate) mod regenerate;
pub(crate) mod thresholds;
```

- [ ] **Step 3: Create `data/coverage/mod.rs`**

```rust
//! Coverage/profile/fit/threshold report prose relocated from
//! `pleiades-data::coverage` (report-surface relocation program, Slice C).
#![allow(dead_code)]

pub(crate) mod body;
pub(crate) mod fit;
pub(crate) mod generation;
pub(crate) mod profile;
pub(crate) mod regen;
pub(crate) mod target;
pub(crate) mod threshold;
```

- [ ] **Step 4: Create the 12 leaf shells**

Create `data/coverage/{body,fit,generation,profile,regen,target,threshold}.rs` and `data/{lookup,regenerate,thresholds,accuracy_baseline}.rs`, each containing only:

```rust
//! Relocated from `pleiades-data` (report-surface relocation program, Slice C).
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo build -p pleiades-validate`
Expected: builds clean (empty modules; `#![allow(dead_code)]` suppresses unused-module lints until filled).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-validate/src/posture
git commit -m "refactor(validate): scaffold Slice C posture/data submodules"
```

---

### Task 2: Decouple the backend-metadata coupling (BEFORE any renderer move)

**Files:**
- Modify: `crates/pleiades-data/src/backend.rs` (`PackagedDataBackend::metadata()`, ~line 221-227 `data_sources` vec)
- Add test: `crates/pleiades-data/src/backend.rs` test module (metadata `data_sources` equality fixture)

**Interfaces:**
- Consumes (all STAY in `pleiades-data`): the `&'static str` accessors `packaged_request_policy_summary()`, `packaged_frame_treatment_summary()`, `packaged_artifact_storage_summary()`, `packaged_artifact_access_summary()`; the constructor `packaged_body_coverage_summary_details()` and the struct method `PackagedBodyCoverageSummary::validated_summary_line()`.
- Produces: `metadata()` no longer calls `packaged_body_coverage_summary()`, `packaged_request_policy_summary_for_report()`, `packaged_frame_treatment_summary_for_report()`, `packaged_artifact_storage_summary_for_report()`, or `packaged_artifact_access_summary_for_report()` — freeing all five to move in Tasks 3/4.

**Why this is safe (byte-identity):** each `_for_report()` produces the same happy-path string as its `&'static str` accessor — `packaged_frame_treatment_summary()` is literally `OnceLock::get_or_init(packaged_frame_treatment_summary_for_report)`; `packaged_request_policy_summary()` and the storage/access accessors run the identical `summary.to_string()` / `validated_summary_line()` render as their `_for_report` twins. The fixture in Step 1 proves it regardless.

- [ ] **Step 1: Capture a pre-change golden fixture of the whole `data_sources` vec**

Add to `backend.rs`'s test module (create `#[cfg(test)] mod coupling_fixture_tests { … }` if none suits). First write it asserting against a placeholder to dump the current value:

```rust
#[test]
fn backend_metadata_data_sources_is_stable() {
    use pleiades_backend::EphemerisBackend;
    let metadata = PackagedDataBackend::default().metadata();
    let expected: &[&str] = &[
        "REPLACE_LINE_0",
        "REPLACE_LINE_1",
        "REPLACE_LINE_2",
        "REPLACE_LINE_3",
        "REPLACE_LINE_4",
    ];
    assert_eq!(
        metadata.provenance.data_sources,
        expected,
        "backend metadata data_sources drifted:\n{:#?}",
        metadata.provenance.data_sources
    );
}
```

Run: `cargo test -p pleiades-data backend_metadata_data_sources_is_stable -- --nocapture`
Expected: FAIL, printing the five actual strings. Paste each literal into the matching `REPLACE_LINE_N`. Rerun against unchanged code → PASS. This is the pre-move golden.

- [ ] **Step 2: Rebuild the five strings inline from retained structured data**

In `backend.rs`, replace the `data_sources: vec![ … ]` block (currently the five renderer calls) with:

```rust
data_sources: vec![
    packaged_body_coverage_summary_details()
        .validated_summary_line()
        .unwrap_or_else(|error| format!("Packaged body set: unavailable ({error})")),
    packaged_request_policy_summary().to_string(),
    packaged_frame_treatment_summary().to_string(),
    packaged_artifact_storage_summary().to_string(),
    packaged_artifact_access_summary().to_string(),
],
```

Update `backend.rs`'s imports: remove the five `*_for_report` / `packaged_body_coverage_summary` imports; add `packaged_body_coverage_summary_details`, `packaged_request_policy_summary`, `packaged_frame_treatment_summary`, `packaged_artifact_storage_summary`, `packaged_artifact_access_summary` (all retained `pleiades-data` items).

> If `PackagedBodyCoverageSummary::validated_summary_line()` returns `String` (not `Result`), drop the `.unwrap_or_else(...)` and the fixture will still pin it. Read `coverage/body.rs:61` to confirm the signature before finalizing; match the exact happy/error rendering `packaged_body_coverage_summary()` used (it wrapped `validated_summary_line()` with the `"Packaged body set: unavailable ({error})"` fallback).

- [ ] **Step 3: Verify byte-identity**

Run: `cargo test -p pleiades-data backend_metadata_data_sources_is_stable`
Expected: PASS — the rebuilt vec equals the captured golden. If it fails, the rebuild drifted; fix the rebuild (do not edit the fixture).

Run: `cargo build -p pleiades-data`
Expected: clean (the five renderers are now unused by `backend.rs` but still present/`pub` — they move in Tasks 3/4).

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-data/src/backend.rs
git commit -m "refactor(data): rebuild backend metadata data_sources from retained structured accessors (Slice C coupling)"
```

---

### Task 3: Move coverage/ renderers

**Files:**
- Modify: `crates/pleiades-data/src/coverage/{profile,fit,target,threshold,regen,generation,body}.rs` (remove the free renderers + their tests)
- Modify: `crates/pleiades-validate/src/posture/data/coverage/{profile,fit,target,threshold,regen,generation,body}.rs` (add them)
- Modify: any validate/CLI consumer of a moved coverage renderer (repoint; enumerated in Step 1)

**Interfaces:**
- Consumes (STAY in `pleiades-data`): the coverage `Packaged*Summary` structs, their `*_details()`/`*_summary()` constructors, and their inherent `summary_line()`/`validated_summary_line()` methods.
- Produces: `crate::posture::data::coverage::{profile,fit,target,threshold,regen,generation,body}::<renderer>` for each of the 25 free renderers listed in the inventory (profile 7, fit 1, target 5, threshold 6, regen 2, generation 2, body `packaged_body_coverage_summary` + its `format_validated_packaged_body_coverage_summary_for_report` helper).

Note: `packaged_artifact_generation_residual_bodies_summary_for_report` (generation.rs) calls `pleiades_compression::ArtifactResidualBodyCoverageSummary::validated_summary_line_with_body_count` — that call is unchanged after the move (validate already depends on `pleiades-compression`, and Slice B left that method in place for exactly this consumer). `packaged_body_coverage_summary` and its `pub(crate)` helper move together; the helper uses the retained `PackagedBodyCoverageSummary::validated_summary_line()` method.

- [ ] **Step 1: Enumerate consumers of the coverage renderers**

Run: `grep -rn "packaged_artifact_production_profile_summary_for_report\|packaged_artifact_generation_manifest_for_report\|packaged_artifact_generation_manifest_checksum_for_report\|packaged_artifact_profile_coverage_summary_for_report\|packaged_artifact_profile_summary_with_output_support_for_report\|packaged_artifact_output_support_summary_for_report\|packaged_artifact_speed_policy_summary_for_report\|packaged_artifact_fit_channel_outlier_summary_for_report\|packaged_artifact_target_threshold_state_for_report\|packaged_artifact_phase2_corpus_alignment_summary_for_report\|packaged_artifact_target_threshold_summary_for_report\|packaged_artifact_source_fit_holdout_sync_summary_for_report\|packaged_artifact_target_threshold_scope_envelopes_for_report\|packaged_artifact_fit_envelope_summary_for_report\|packaged_artifact_fit_outlier_summary_for_report\|packaged_artifact_fit_threshold_summary_for_report\|packaged_artifact_fit_margin_summary_for_report\|packaged_artifact_fit_threshold_violation_count_for_report\|packaged_artifact_fit_threshold_violation_summary_for_report\|packaged_artifact_regeneration_summary_for_report\|packaged_artifact_normalized_intermediate_summary_for_report\|packaged_artifact_generation_policy_summary_for_report\|packaged_artifact_generation_residual_bodies_summary_for_report\|packaged_body_coverage_summary\b" crates/pleiades-validate/src crates/pleiades-cli/src --include=*.rs`
Expected: a list of validate render/bundle call sites and CLI/test references. Record each for Step 3. (`packaged_body_coverage_summary`'s former `backend.rs` caller was removed in Task 2, so it now has only validate/CLI/test consumers, if any.)

- [ ] **Step 2: Move renderers module-by-module**

For each of the seven coverage modules, apply the relocation procedure into the matching `posture/data/coverage/<mod>.rs`: copy each free renderer (and any `pub(crate)` helper it exclusively wraps) unchanged, adjusting `use` to `pleiades_data::…` retained items; copy its `#[test]`s into a `#[cfg(test)] mod tests`; delete the renderer + moved tests from the source module. Keep every struct, `*_details()`/`*_summary()` constructor, and inherent method in `pleiades-data`.

> Do NOT move `packaged_body_coverage_summary_details` (coverage/body.rs) or `packaged_artifact_fit_channel_outlier_summary_details` (coverage/fit.rs) — they are `_details()` constructors that STAY (and get demoted in Task 6).

- [ ] **Step 3: Repoint consumers** found in Step 1 to `crate::posture::data::coverage::<mod>::…` (validate) or `pleiades_validate::…` (CLI, if any moved coverage renderer is a public CLI runtime consumer — mark those `pub` in the posture module; all others `pub(crate)`).

- [ ] **Step 4: Verify**

Run: `cargo test -p pleiades-validate posture::data::coverage && cargo build -p pleiades-data -p pleiades-validate -p pleiades-cli`
Expected: moved tests PASS; all three crates build.
Run: `grep -rnE "^\s*pub fn .*_for_report|^pub fn packaged_body_coverage_summary\b" crates/pleiades-data/src/coverage`
Expected: no free `pub fn` renderers remain (indented struct-method `summary_line`/`validated_summary_line` stay — that is correct).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-data crates/pleiades-validate crates/pleiades-cli
git commit -m "refactor(data,validate): relocate coverage report renderers into validate posture"
```

---

### Task 4: Move lookup.rs + regenerate.rs renderers

**Files:**
- Modify: `crates/pleiades-data/src/lookup.rs` (remove 7 free renderers + tests), `crates/pleiades-data/src/regenerate.rs` (remove 3 free renderers + tests)
- Modify: `crates/pleiades-validate/src/posture/data/lookup.rs`, `.../posture/data/regenerate.rs`
- Modify: validate/CLI consumers (repoint; enumerated in Step 1)

**Interfaces:**
- Consumes (STAY): the lookup/regenerate `Packaged*Summary` structs, their constructors, the `&'static str` accessors (`packaged_request_policy_summary`, `packaged_frame_treatment_summary`, `packaged_artifact_storage_summary`, `packaged_artifact_access_summary`), and inherent methods.
- Produces: `crate::posture::data::lookup::{packaged_lookup_epoch_policy_summary_for_report, packaged_request_policy_summary_for_report, packaged_frame_treatment_summary_for_report, packaged_artifact_storage_summary_for_report, packaged_artifact_access_summary_for_report, packaged_frame_parity_summary_for_report, packaged_mixed_tt_tdb_batch_parity_summary_for_report}` and `crate::posture::data::regenerate::{packaged_artifact_body_class_span_cap_summary_for_report, packaged_artifact_body_class_span_cap_entries_for_report, packaged_artifact_body_cadence_summary_for_report}`.

**Do NOT move** `packaged_mixed_frame_batch_parity_summary_for_report` (lookup.rs) — it has 0 consumers and is demoted in Task 6.

- [ ] **Step 1: Enumerate consumers**

Run: `grep -rn "packaged_lookup_epoch_policy_summary_for_report\|packaged_request_policy_summary_for_report\|packaged_frame_treatment_summary_for_report\|packaged_artifact_storage_summary_for_report\|packaged_artifact_access_summary_for_report\|packaged_frame_parity_summary_for_report\|packaged_mixed_tt_tdb_batch_parity_summary_for_report\|packaged_artifact_body_class_span_cap_summary_for_report\|packaged_artifact_body_class_span_cap_entries_for_report\|packaged_artifact_body_cadence_summary_for_report" crates/pleiades-validate/src crates/pleiades-cli/src --include=*.rs`
Expected: validate render/bundle call sites (and any CLI). Record for Step 3. Confirm `backend.rs` no longer appears (Task 2 removed those calls).

- [ ] **Step 2: Move the 10 renderers + tests** per the relocation procedure into `posture/data/lookup.rs` (7) and `posture/data/regenerate.rs` (3), adjusting `use` to `pleiades_data::…` retained accessors/structs. Keep the `&'static str` accessors and the regeneration pipeline (`regenerate_packaged_artifact*`) in `pleiades-data`.

- [ ] **Step 3: Repoint consumers** to `crate::posture::data::{lookup,regenerate}::…` (validate) or `pleiades_validate::…` (CLI). Mark any CLI-runtime renderer `pub`; all others `pub(crate)`.

- [ ] **Step 4: Verify**

Run: `cargo test -p pleiades-validate "posture::data::lookup" && cargo test -p pleiades-validate "posture::data::regenerate" && cargo build -p pleiades-data -p pleiades-validate -p pleiades-cli`
Expected: PASS + clean build.
Run: `grep -rnE "^\s*pub fn .*_for_report" crates/pleiades-data/src/lookup.rs crates/pleiades-data/src/regenerate.rs`
Expected: only `packaged_mixed_frame_batch_parity_summary_for_report` (lookup.rs) may remain here as `pub fn` — it becomes `pub(crate)` in Task 6. All others gone.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-data crates/pleiades-validate crates/pleiades-cli
git commit -m "refactor(data,validate): relocate lookup + regenerate report renderers into validate posture"
```

---

### Task 5: Move thresholds.rs + accuracy_baseline.rs renderer wrappers

**Files:**
- Modify: `crates/pleiades-data/src/thresholds.rs` (remove `packaged_artifact_thresholds_summary_for_report` + tests), `crates/pleiades-data/src/accuracy_baseline.rs` (remove `packaged_artifact_accuracy_baseline_summary_for_report` + tests)
- Modify: `crates/pleiades-validate/src/posture/data/thresholds.rs`, `.../posture/data/accuracy_baseline.rs`
- Modify: validate/CLI consumers (repoint)

**Interfaces:**
- Consumes (STAY): `pleiades_data::thresholds::{accuracy_ceiling, PACKAGED_BUDGETS}` (release-gate data — enforced by gates, NOT moved) and the accuracy-baseline measurement core (`accuracy_baseline_against`, `packaged_artifact_accuracy_baseline`, and the `summary_line(&self)` inherent method at accuracy_baseline.rs:55).
- Produces: `crate::posture::data::thresholds::packaged_artifact_thresholds_summary_for_report`, `crate::posture::data::accuracy_baseline::packaged_artifact_accuracy_baseline_summary_for_report`.

- [ ] **Step 1: Enumerate consumers**

Run: `grep -rn "packaged_artifact_thresholds_summary_for_report\|packaged_artifact_accuracy_baseline_summary_for_report" crates/pleiades-validate/src crates/pleiades-cli/src --include=*.rs`
Expected: the validate/CLI call sites (note `thresholds::accuracy_ceiling` and `PACKAGED_BUDGETS` are separate retained items — leave those imports as `pleiades_data::thresholds::…`).

- [ ] **Step 2: Move the two renderer wrappers + tests** per the relocation procedure. In `posture/data/thresholds.rs`, the moved renderer references `pleiades_data::thresholds::{accuracy_ceiling, PACKAGED_BUDGETS}` for its data. Leave `accuracy_ceiling`, `PACKAGED_BUDGETS`, and the measurement core in `pleiades-data`.

- [ ] **Step 3: Repoint consumers** to `crate::posture::data::{thresholds,accuracy_baseline}::…`.

- [ ] **Step 4: Verify**

Run: `cargo test -p pleiades-validate "posture::data::thresholds" && cargo test -p pleiades-validate "posture::data::accuracy_baseline" && cargo build -p pleiades-data -p pleiades-validate -p pleiades-cli`
Expected: PASS + clean build.
Run: `grep -rnE "^\s*pub fn .*_for_report" crates/pleiades-data/src/thresholds.rs crates/pleiades-data/src/accuracy_baseline.rs`
Expected: none remain.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-data crates/pleiades-validate crates/pleiades-cli
git commit -m "refactor(data,validate): relocate thresholds + accuracy-baseline report wrappers into validate posture"
```

---

### Task 6: Demote the four over-exposed no-consumer items

**Files:**
- Modify: `crates/pleiades-data/src/lookup.rs` (`packaged_mixed_frame_batch_parity_summary_for_report`), `crates/pleiades-data/src/coverage/body.rs` (`packaged_body_coverage_summary_details`), `crates/pleiades-data/src/coverage/fit.rs` (`packaged_artifact_fit_channel_outlier_summary_details`), `crates/pleiades-data/src/accuracy_baseline.rs` (`eros_self_consistency_max_longitude_arcsec`)

**Interfaces:**
- Consumes: nothing external (all four confirmed 0 non-test consumers in validate/CLI).
- Produces: these four symbols become `pub(crate)` — reachable by their own in-crate tests and (for `packaged_body_coverage_summary_details`) by `backend.rs`'s Task-2 rebuild, but no longer public API.

- [ ] **Step 1: Re-confirm zero external consumers**

Run: `grep -rn "packaged_mixed_frame_batch_parity_summary_for_report\|packaged_body_coverage_summary_details\|packaged_artifact_fit_channel_outlier_summary_details\|eros_self_consistency_max_longitude_arcsec" crates --include=*.rs | grep -v "crates/pleiades-data/" | grep -v "//"`
Expected: **no output** (only `pleiades-data`-internal references — its own tests and the `backend.rs` rebuild for `packaged_body_coverage_summary_details`).

- [ ] **Step 2: Change each `pub fn` to `pub(crate) fn`**

Edit each of the four function signatures: `pub fn` → `pub(crate) fn`. (Leave bodies and their tests unchanged.)

- [ ] **Step 3: Verify**

Run: `cargo build -p pleiades-data -p pleiades-validate -p pleiades-cli && cargo test -p pleiades-data`
Expected: clean build + `pleiades-data` tests PASS (the demoted items are still reachable by in-crate tests).

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-data
git commit -m "refactor(data): demote four over-exposed no-consumer report/detail items to pub(crate)"
```

---

### Task 7: Consumer repoint sweep + release-bundle pin verification

**Files:**
- Modify (if any residual): `crates/pleiades-validate/src/release/{bundle.rs,bundle_verify_helpers.rs}`, `crates/pleiades-cli/src/**`

**Interfaces:**
- Consumes: all `crate::posture::data::*` renderers from Tasks 3-5.
- Produces: a workspace with zero remaining references to the moved symbols under their old `pleiades_data::…` paths, and unchanged bundle checksums.

- [ ] **Step 1: Find any straggler references to moved symbols**

Run: `grep -rn "pleiades_data::.*_for_report\|pleiades_data::packaged_body_coverage_summary\b" crates --include=*.rs`
Expected: no hits (every moved renderer now lives under `crate::posture::data::…` / `pleiades_validate::…`). Repoint any straggler. Note: `pleiades_data::thresholds::{accuracy_ceiling, PACKAGED_BUDGETS}`, `pleiades_data::packaged_*_summary` (`&'static str` accessors), and `pleiades_data::*_summary_details`/`*_summary()` constructors are retained — they legitimately remain.

- [ ] **Step 2: Verify release-bundle checksums unchanged**

Run: `mise run release-smoke`
Expected: PASS with no checksum diff. If a fnv1a64 value changed, a moved renderer drifted — fix the move to restore byte-identity; **do not** regenerate the checksum.

- [ ] **Step 3: Full workspace test**

Run: `cargo test --workspace --include-ignored`
Expected: PASS.

- [ ] **Step 4: Commit** (only if Step 1 changed files)

```bash
git add crates
git commit -m "refactor(validate,cli): repoint remaining Slice C renderer consumers to posture paths"
```

---

### Task 8: Close-out — grep gate, CI, CHANGELOG, PLAN.md, PR

**Files:**
- Modify: `CHANGELOG.md`, `PLAN.md`

**Interfaces:**
- Consumes: green workspace from Task 7.
- Produces: documented Slice C close-out; the program moves to Slice D remaining.

- [ ] **Step 1: Assert no `_for_report` exports remain in pleiades-data**

Run: `grep -rn "pub fn .*_for_report" crates/pleiades-data/src`
Expected: **no output.** (This is the slice's structural gate. `pub(crate) fn packaged_mixed_frame_batch_parity_summary_for_report` from Task 6 does not match `pub fn`, and inherent struct-method `summary_line`/`validated_summary_line` remain — both expected.)

- [ ] **Step 2: Run full CI**

Run: `mise run ci`
Expected: PASS — fmt, clippy `-D warnings`, `cargo test --workspace --include-ignored`, `cargo doc -D warnings`, workspace-audit, package-check, release-smoke, claims-audit all green.

- [ ] **Step 3: Update CHANGELOG.md**

Add a Slice C entry under Unreleased, in the style of the Slice A/B entries: `pleiades-data`'s report-prose renderers relocated into `pleiades-validate/src/posture/data/`; the packaged-data backend metadata `data_sources` decoupled from the five embedded report strings (rebuilt from retained `&'static str` accessors + `PackagedBodyCoverageSummary::validated_summary_line`); four over-exposed no-consumer items demoted to `pub(crate)`; byte-identical output (release-smoke checksum parity); no version bump (compatibility `0.7.13`, API-stability `0.3.0`); only Slice D (`pleiades-jpl`) remains before the workspace `0.4.0` release.

- [ ] **Step 4: Refresh PLAN.md status line**

Update the report-surface relocation status (currently "slice B done … only slices C, D remain") to record Slice C delivered and only Slice D remaining before `0.4.0`.

- [ ] **Step 5: Commit**

```bash
git add CHANGELOG.md PLAN.md
git commit -m "docs: report-surface relocation Slice C close-out (CHANGELOG, PLAN status)"
```

- [ ] **Step 6: Open the PR** (mirrors Slice A/B's flow)

```bash
git push -u origin feat/report-surface-relocation-slice-c
gh pr create --title "feat(validate): report-surface relocation Slice C (pleiades-data)" --body "Relocates pleiades-data report prose into pleiades-validate posture modules; decouples the packaged-data backend metadata from its five embedded report strings. Pure relocation — byte-identical output, no version bump. Slice C of the report-surface relocation program (program design: docs/superpowers/specs/2026-07-10-report-surface-relocation-design.md; slice spec: docs/superpowers/specs/2026-07-11-report-surface-relocation-slice-c-design.md)."
```

---

## Self-Review

**Spec coverage** — every spec section maps to a task:
- Stay/move partition → Global Constraints + Partition rule + every move task's "STAY" notes.
- Module map (`posture/data/…`) → Task 1 scaffold + Tasks 3-5 fills.
- Coupling (5 embedded metadata strings) → Task 2, ordered before all renderer moves (Tasks 3-5) per spec.
- Deletions/demotions (4 items) → Task 6, with the 0-consumer re-confirmation.
- Consumer migrations (validate render/bundle, CLI) → per-task Step-1 enumeration + Step-3 repoints + Task 7 sweep.
- Visibility (`pub(crate)` default, `pub` only for genuine cross-crate runtime consumers) → each move task's Step 3.
- Invariants (byte-identity, profile `0.7.13`, no behavior change, dependency direction) → Global Constraints + Task 2 fixture + Task 7 release-smoke + Task 8 CI/grep gate.
- Verification (`mise run ci`, release-smoke, grep assertion) → Tasks 7-8.
- Non-goals (no version bump, no Slice D, no validate split) → Global Constraints.

**Placeholder scan:** No "TBD"/"handle edge cases" placeholders. Task 2 Step 1 intentionally instructs capturing the live rendered strings into the fixture (the canonical text lives in the running code and must be captured, not invented) — inherent to a byte-identity relocation, not under-specification. Task 2 Step 2's note to confirm `validated_summary_line`'s return type before finalizing the `.unwrap_or_else` is a targeted read of one signature, not a deferred decision. Renderer symbol lists are explicit per task.

**Type consistency:** Retained vs moved symbols are named consistently across tasks. The four `&'static str` accessors (`packaged_request_policy_summary`, `packaged_frame_treatment_summary`, `packaged_artifact_storage_summary`, `packaged_artifact_access_summary`) named identically in Task 2's rebuild and Task 4's "STAY" list. `packaged_body_coverage_summary_details` (retained/demoted) is distinct from `packaged_body_coverage_summary` (moved) throughout — Task 2 consumes the former, Task 3 moves the latter, Task 6 demotes the former. The demote-set of four is named identically in the inventory, Task 6, and Task 7's retained-items note. `packaged_mixed_frame_batch_parity_summary_for_report` (demote, stays) is consistently excluded from Task 4's move list and handled in Task 6.
