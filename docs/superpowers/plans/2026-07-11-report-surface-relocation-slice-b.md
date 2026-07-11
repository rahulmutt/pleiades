# Report-Surface Relocation Slice B Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Relocate the free report-prose functions of five functional crates (houses, ayanamsa, vsop87, elp, compression) into `pleiades-validate/src/posture/<crate>/`, verbatim, and resolve the elp backend-metadata coupling — with byte-identical rendered output.

**Architecture:** Pure relocation. Each crate's `*_for_report` / `format_validated_*_for_report` / `format_*_summary(&Struct)` free functions move into a per-crate posture subdirectory under `pleiades-validate`, `pub(crate)` by default. The structs, their constructors, and their inherent `summary_line()` / `validate()` methods stay in the functional crate as structured data. The one calculation-path consumer (elp `EphemerisBackend::metadata`) is decoupled by inlining a 4-line render using the retained struct methods. Consumers in validate and the CLI repoint; release-bundle checksum values are unchanged because text is byte-identical.

**Tech Stack:** Rust workspace (edition 2021, rust-version 1.97.0), `cargo`, `mise` task runner. Test framework: built-in `cargo test`. CI: `mise run ci`.

## Global Constraints

- **Byte-identical output.** Every moved renderer produces exactly the prose it produced before. fnv1a64 release-bundle checksum values must not change. Any drift is a defect in the move, never a checksum to regenerate.
- **No version bumps.** Compatibility profile stays `0.7.13`; API-stability profile stays `0.3.0`. No `Cargo.toml` version edits in this slice.
- **No deletions of live surface, no behavior/gate/threshold/corpus changes.** (The two dead `*_thresholds_summary_for_report` helpers were already removed in Slice A.)
- **Dependency direction.** Nothing below `pleiades-validate` may gain a dependency on it. `pleiades_backend::FrameTreatmentSummary` and `pleiades_elp::lunar_theory_frame_treatment_summary_details` stay put.
- **Visibility:** moved symbols are `pub(crate)` unless they have a genuine cross-crate runtime consumer (enumerated per task). Posture modules carry `#![allow(dead_code)]` as in Slice A's `backend_policy.rs`.
- **Partition rule:** MOVE free functions that render prose (`…_for_report`, `format_validated_…_for_report`, `format_*_summary(&Struct) -> String`). KEEP `*_summary()`/`*_details()` struct constructors, inherent `summary_line()`/`validated_summary_line()`/`validate()` methods, and all data structs.
- **Branch:** `feat/report-surface-relocation-slice-b` (already created; the spec commit is on it).
- **Commit discipline:** one commit per task (or per crate-sub-step where noted). Conventional-commit messages; use `!` only if a public signature is removed without a kept re-export.

## Relocation procedure (applied by every move task)

For each free function `F` being moved from `crates/pleiades-<crate>/src/<mod>.rs` to `crates/pleiades-validate/src/posture/<crate>/<mod>.rs`:

1. Copy `F`'s full body (and any private helper it solely owns, e.g. a `format_validated_*` it wraps) into the destination module, unchanged. Adjust `use` paths so it references the functional crate's retained public items (structs, constructors, methods) via `pleiades_<crate>::…`.
2. Copy `F`'s unit tests (the `#[test]` fns that assert on `F`'s output) into the destination module's `#[cfg(test)] mod tests`, unchanged except import paths.
3. Delete `F` and its moved tests from the source crate.
4. Repoint every caller (in-crate, validate, CLI) to the new path.
5. Verify: the moved tests pass at the new location, the source crate and all consumers build, and `F` no longer appears as an exported symbol of the source crate.

"Verbatim" means the rendered `String` is identical; only module path and visibility change.

## File Structure

```
crates/pleiades-validate/src/posture/
  mod.rs                 MODIFY  — add `mod houses; mod ayanamsa; mod vsop87; mod elp; mod compression;`
  backend_policy.rs      (Slice A — untouched)
  houses/mod.rs          CREATE  — 4 house catalog renderers + tests
  ayanamsa/mod.rs        CREATE  — provenance report renderer + tests
  vsop87/
    mod.rs               CREATE  — `mod spec; mod audit; ...`
    spec.rs              CREATE
    audit.rs             CREATE
    documentation.rs     CREATE
    request_corpus.rs    CREATE
    evidence.rs          CREATE
    batch_parity.rs      CREATE
  elp/
    mod.rs               CREATE  — `mod lib_summaries; mod catalog; mod evidence; mod source;`
    lib_summaries.rs     CREATE  — from elp lib.rs top-level renderers
    catalog.rs           CREATE
    evidence.rs          CREATE
    source.rs            CREATE
  compression/mod.rs     CREATE  — 2 coverage-summary renderings (validate-side recompute)
```

Source crates modified (renderers + their tests removed; structs/constructors/methods retained):
`pleiades-houses/src/catalog/mod.rs`, `pleiades-ayanamsa/src/lookup.rs`,
`pleiades-vsop87/src/source_docs/{spec,audit,documentation,request_corpus,evidence,batch_parity}.rs`,
`pleiades-elp/src/{lib,catalog,evidence,source,backend}.rs`,
`pleiades-compression/src/format.rs`.

Consumers repointed: `pleiades-validate/src/{release/bundle.rs,release/bundle_verify_helpers.rs,render/summary/writers.rs,render/summary/artifact.rs,render/text/catalog.rs,tests/render_catalog.rs}`, `pleiades-cli/src/{cli.rs,cli/tests/summary_commands.rs}`.

---

### Task 1: Scaffold posture submodules

**Files:**
- Modify: `crates/pleiades-validate/src/posture/mod.rs`
- Create: `crates/pleiades-validate/src/posture/{houses/mod.rs, ayanamsa/mod.rs, vsop87/mod.rs, elp/mod.rs, compression/mod.rs}` (empty module shells)

**Interfaces:**
- Produces: `crate::posture::{houses, ayanamsa, vsop87, elp, compression}` module paths that later tasks fill.

- [ ] **Step 1: Add module declarations**

In `crates/pleiades-validate/src/posture/mod.rs`, after the existing `pub(crate) mod backend_policy;`:

```rust
pub(crate) mod ayanamsa;
pub(crate) mod compression;
pub(crate) mod elp;
pub(crate) mod houses;
pub(crate) mod vsop87;
```

- [ ] **Step 2: Create empty submodule shells**

Each new file starts with the Slice-A posture header pattern. For the single-file crates (`houses/mod.rs`, `ayanamsa/mod.rs`, `compression/mod.rs`):

```rust
//! <Crate> report/summary prose relocated from `pleiades-<crate>`
//! (report-surface relocation program, Slice B). Rendering only — the
//! functional crate keeps the structured data and its inherent methods.

// Verbatim relocation of a report-prose surface: some renderers are exercised
// only by this module's own tests or have no current in-crate caller.
#![allow(dead_code)]
```

For `vsop87/mod.rs`:

```rust
//! VSOP87 source-documentation report prose relocated from
//! `pleiades-vsop87::source_docs` (report-surface relocation program, Slice B).
#![allow(dead_code)]

pub(crate) mod audit;
pub(crate) mod batch_parity;
pub(crate) mod documentation;
pub(crate) mod evidence;
pub(crate) mod request_corpus;
pub(crate) mod spec;
```

For `elp/mod.rs`:

```rust
//! ELP lunar-theory report prose relocated from `pleiades-elp`
//! (report-surface relocation program, Slice B).
#![allow(dead_code)]

pub(crate) mod catalog;
pub(crate) mod evidence;
pub(crate) mod lib_summaries;
pub(crate) mod source;
```

Create each referenced submodule file (`vsop87/*.rs`, `elp/*.rs`) with just the header comment for now.

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p pleiades-validate`
Expected: builds clean (empty modules, no warnings — `#![allow(dead_code)]` suppresses the unused-module lint until filled).

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-validate/src/posture
git commit -m "refactor(validate): scaffold Slice B posture submodules (houses/ayanamsa/vsop87/elp/compression)"
```

---

### Task 2: Move pleiades-houses catalog renderers

**Files:**
- Modify: `crates/pleiades-houses/src/catalog/mod.rs` (remove 4 free renderers + their tests)
- Modify: `crates/pleiades-validate/src/posture/houses/mod.rs` (add them)
- Modify: `crates/pleiades-validate/src/render/text/catalog.rs:387` (repoint)

**Interfaces:**
- Consumes: retained `pleiades_houses::{HouseCatalogValidationSummary, house_catalog_validation_summary, house_system_code_aliases (data), HouseCatalogValidationError}` and the catalog descriptor data these renderers read.
- Produces: `crate::posture::houses::{house_system_code_aliases_summary_line, validated_house_system_code_aliases_summary_line, house_formula_families_summary_line, latitude_sensitive_house_failure_modes_summary_line}`.

Symbols to move (all free fns in `catalog/mod.rs`): `house_system_code_aliases_summary_line` (~:412), `validated_house_system_code_aliases_summary_line` (~:421), `house_formula_families_summary_line` (~:750), `latitude_sensitive_house_failure_modes_summary_line` (~:782). STAY: `house_catalog_validation_summary` (:787, constructor) and every struct method.

- [ ] **Step 1: Confirm the current consumer set**

Run: `grep -rn "house_system_code_aliases_summary_line\|house_formula_families_summary_line\|latitude_sensitive_house_failure_modes_summary_line" crates --include=*.rs | grep -v "fn \|//"`
Expected: the only non-definition, non-test runtime caller is `crates/pleiades-validate/src/render/text/catalog.rs:387` (`validated_house_system_code_aliases_summary_line`). Note any others for repointing.

- [ ] **Step 2: Move the four functions and their tests**

Follow the relocation procedure: copy the four functions (and any private helper they exclusively wrap) into `crates/pleiades-validate/src/posture/houses/mod.rs`, adjusting internal references to `pleiades_houses::…`; copy their `#[test]` functions into a `#[cfg(test)] mod tests` there; delete all four (and moved tests) from `catalog/mod.rs`.

- [ ] **Step 3: Repoint the validate consumer**

In `crates/pleiades-validate/src/render/text/catalog.rs:387`, change `pleiades_houses::validated_house_system_code_aliases_summary_line()` to `crate::posture::houses::validated_house_system_code_aliases_summary_line()`.

- [ ] **Step 4: Verify build, tests, and non-export**

Run: `cargo test -p pleiades-validate posture::houses && cargo build -p pleiades-houses -p pleiades-validate -p pleiades-cli`
Expected: moved tests PASS; all three crates build.
Run: `grep -rn "pub fn .*summary_line\|pub fn .*_for_report" crates/pleiades-houses/src/catalog/mod.rs`
Expected: no free `pub fn` renderers remain (struct-method `summary_line` are indented `impl` methods — still present, and that is correct).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-houses crates/pleiades-validate
git commit -m "refactor(houses,validate): relocate house catalog report renderers into validate posture"
```

---

### Task 3: Move pleiades-ayanamsa provenance renderer

**Files:**
- Modify: `crates/pleiades-ayanamsa/src/lookup.rs` (remove `validated_provenance_summary_for_report` + its `format_validated_*` helper + their tests)
- Modify: `crates/pleiades-validate/src/posture/ayanamsa/mod.rs` (add them)

**Interfaces:**
- Consumes: retained `pleiades_ayanamsa::{provenance_summary, AyanamsaProvenanceSummary, provenance_sample_ayanamsas}` and descriptor data.
- Produces: `crate::posture::ayanamsa::validated_provenance_summary_for_report`.

Symbols to move: `validated_provenance_summary_for_report` (`lookup.rs:110`) and any `format_validated_…provenance…_for_report` helper it wraps. STAY: `provenance_summary` (:105, constructor), `ayanamsa_catalog_validation_summary` (:255, constructor). Note: validate already rebuilds its own provenance line from `provenance_sample_ayanamsas()` (coupling 2, Slice A), so this renderer's only remaining callers are its own tests and possibly the release bundle — confirm in Step 1.

- [ ] **Step 1: Confirm consumers**

Run: `grep -rn "validated_provenance_summary_for_report" crates --include=*.rs | grep -v "fn \|//"`
Expected: enumerate callers (its tests, and any bundle/CLI use). If a release-bundle pin references it, note it for Task 9's pin sweep.

- [ ] **Step 2: Move the function + helper + tests** — per the relocation procedure, into `posture/ayanamsa/mod.rs`.

- [ ] **Step 3: Repoint any callers** found in Step 1 to `crate::posture::ayanamsa::validated_provenance_summary_for_report`.

- [ ] **Step 4: Verify**

Run: `cargo test -p pleiades-validate posture::ayanamsa && cargo build -p pleiades-ayanamsa -p pleiades-validate -p pleiades-cli`
Expected: PASS + clean build.
Run: `grep -rn "pub fn .*_for_report" crates/pleiades-ayanamsa/src`
Expected: none remain.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-ayanamsa crates/pleiades-validate
git commit -m "refactor(ayanamsa,validate): relocate provenance report renderer into validate posture"
```

---

### Task 4: Move pleiades-vsop87 source-docs renderers

**Files:**
- Modify: `crates/pleiades-vsop87/src/source_docs/{spec,audit,documentation,request_corpus,evidence,batch_parity}.rs`
- Modify: `crates/pleiades-validate/src/posture/vsop87/{spec,audit,documentation,request_corpus,evidence,batch_parity}.rs`
- Modify: `crates/pleiades-validate/src/render/text/evidence.rs` (only if it calls a moved `_for_report`; it currently takes the retained `Vsop87SourceDocumentation[Health]Summary` structs — likely no change)

**Interfaces:**
- Consumes: retained `pleiades_vsop87` struct constructors (`source_documentation_summary`, `source_audit_summary`, `generated_binary_audit_summary`, `source_manifest_summary`, `source_body_evidence_summary`, `canonical_epoch_*_summary`, `*_batch_parity_summary`, etc.) and their public struct types.
- Produces: `crate::posture::vsop87::<mod>::<renderer>` for each moved `_for_report` / `format_validated_*` / `format_*_summary` function.

Move, per source module, exactly the free functions matching `…_for_report`, `format_validated_…_for_report`, and `format_*_summary(&…Summary) -> String`. KEEP every `*_summary()` that returns a struct. From the inventory:
- **spec.rs:** `source_specifications_for_report`, `frame_treatment_summary_for_report`, `vsop87_request_policy_summary_for_report`. (Keep `frame_treatment_summary() -> &'static str` — it is retained catalog data used elsewhere; confirm in Step 1.)
- **audit.rs:** `format_source_audit_summary`, `format_validated_source_audit_summary_for_report`, `source_audit_summary_for_report`, `format_generated_binary_audit_summary`, `format_validated_generated_binary_audit_summary_for_report`, `generated_binary_audit_summary_for_report`.
- **documentation.rs:** `format_source_documentation_summary`, `format_source_documentation_health_summary`, `format_validated_source_documentation_health_summary_for_report`, `format_validated_source_documentation_summary_for_report`, `source_documentation_summary_for_report`, `source_documentation_health_summary_for_report`.
- **request_corpus.rs:** `format_source_manifest_summary`, `source_manifest_summary_for_report`.
- **evidence.rs:** `format_canonical_epoch_evidence_summary`, `format_validated_canonical_epoch_evidence_summary_for_report`, `canonical_epoch_evidence_summary_for_report`, `canonical_epoch_outlier_note_for_report`, `canonical_epoch_equatorial_evidence_summary_for_report`, `format_canonical_equatorial_body_class_evidence_summary`, `canonical_epoch_equatorial_body_class_evidence_summary_for_report`, `format_canonical_equatorial_evidence_summary`, `format_source_body_evidence_summary`, `source_body_evidence_summary_for_report`.
- **batch_parity.rs:** the eight `*_batch_parity_summary_for_report` + their `format_validated_*_for_report` helpers, plus `format_source_body_class_evidence_summary`, `format_validated_source_body_class_evidence_summary_for_report`, `source_body_class_evidence_summary_for_report`.

- [ ] **Step 1: Confirm which `frame_treatment_summary()` / `*_summary()` are retained data**

Run: `grep -rn "vsop87.*frame_treatment_summary\b\|vsop87.*_summary\b" crates/pleiades-validate/src crates/pleiades-cli/src crates/pleiades-vsop87/src | grep -v "_for_report\|format_"`
Expected: a list of struct-returning constructors / `&'static str` accessors that STAY. Anything a calculation or catalog path consumes stays; only the prose renderers move.

- [ ] **Step 2: Move renderers module-by-module**

For each of the six modules, apply the relocation procedure into the matching `posture/vsop87/<mod>.rs`. Do one module per commit sub-step to keep diffs reviewable. Adjust `use` to reference retained `pleiades_vsop87::…` constructors and struct types. Move each renderer's tests alongside it.

- [ ] **Step 3: Repoint consumers**

Run: `grep -rn "pleiades_vsop87::.*_for_report\|pleiades_vsop87::format_" crates/pleiades-validate/src crates/pleiades-cli/src`
Repoint each hit to `crate::posture::vsop87::<mod>::…` (validate) or `pleiades_validate::…` (CLI, if any moved renderer is a public CLI consumer — vsop87 had no CLI `_for_report` runtime consumer in the survey, so expect validate-only test repoints).

- [ ] **Step 4: Verify**

Run: `cargo test -p pleiades-validate posture::vsop87 && cargo build -p pleiades-vsop87 -p pleiades-validate -p pleiades-cli`
Expected: PASS + clean build.
Run: `grep -rn "pub fn .*_for_report" crates/pleiades-vsop87/src`
Expected: none remain.

- [ ] **Step 5: Commit** (or one commit per module in Step 2)

```bash
git add crates/pleiades-vsop87 crates/pleiades-validate
git commit -m "refactor(vsop87,validate): relocate source-docs report renderers into validate posture"
```

---

### Task 5: Resolve coupling 3 — decouple elp backend metadata (BEFORE Task 6)

**Files:**
- Modify: `crates/pleiades-elp/src/backend.rs:~229` (inline the render using retained struct methods)
- Create test: `crates/pleiades-elp/src/backend.rs` test module (metadata equality fixture) OR extend the existing backend test module

**Interfaces:**
- Consumes: retained `pleiades_elp::{lunar_theory_source_family_summary, LunarTheorySourceFamilySummary}` and its inherent `.validate()` / `.summary_line()` methods (both STAY).
- Produces: `EphemerisBackend::metadata()` no longer calls `lunar_theory_source_family_summary_for_report()`, freeing that free function to move in Task 6.

The current `format_validated_lunar_theory_source_family_summary_for_report` (catalog.rs:581) is exactly:

```rust
match summary.validate() {
    Ok(()) => summary.summary_line(),
    Err(error) => format!("lunar source family: unavailable ({error})"),
}
```

Since `LunarTheorySourceFamilySummary::{validate, summary_line}` are retained inherent methods, `backend.rs` can inline this directly.

- [ ] **Step 1: Capture a pre-change golden fixture**

Add a test to `backend.rs` that pins the current metadata string, so the rebuild is proven byte-identical:

```rust
#[test]
fn backend_metadata_source_family_line_is_stable() {
    use crate::backend::ElpBackend; // unit struct
    use pleiades_backend::EphemerisBackend; // brings `.metadata()` into scope
    let metadata = ElpBackend.metadata();
    let expected = "REPLACE_WITH_CURRENT_RENDERED_LINE";
    assert!(
        metadata.provenance.data_sources.iter().any(|s| s == expected),
        "source-family provenance line drifted:\n{:#?}",
        metadata.provenance.data_sources
    );
}
```

Run it once against the current code to capture the exact string, then paste that literal into `expected`.

Run: `cargo test -p pleiades-elp backend_metadata_source_family_line_is_stable -- --nocapture`
Expected: on first run, read the actual line from the assertion dump; set `expected`; rerun → PASS against unchanged code.

- [ ] **Step 2: Inline the render in `metadata()`**

In `backend.rs`, replace the `lunar_theory_source_family_summary_for_report(),` element of the `data_sources` vec with:

```rust
{
    let family = lunar_theory_source_family_summary();
    match family.validate() {
        Ok(()) => family.summary_line(),
        Err(error) => format!("lunar source family: unavailable ({error})"),
    }
},
```

Ensure `lunar_theory_source_family_summary` is imported in `backend.rs` (it is a retained public fn in `source.rs`).

- [ ] **Step 3: Verify byte-identity**

Run: `cargo test -p pleiades-elp backend_metadata_source_family_line_is_stable`
Expected: PASS — the inlined render equals the captured fixture.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-elp/src/backend.rs
git commit -m "refactor(elp): rebuild backend metadata source-family line from retained struct methods (coupling 3)"
```

---

### Task 6: Move pleiades-elp lunar-theory renderers

**Files:**
- Modify: `crates/pleiades-elp/src/{lib.rs,catalog.rs,evidence.rs,source.rs}` (remove ~38 renderers + tests)
- Modify: `crates/pleiades-validate/src/posture/elp/{lib_summaries,catalog,evidence,source}.rs`
- Modify: `crates/pleiades-validate/src/{release/bundle.rs,release/bundle_verify_helpers.rs,render/summary/writers.rs,tests/render_catalog.rs}` (repoint)
- Modify: `crates/pleiades-cli/src/{cli.rs,cli/tests/summary_commands.rs}` (repoint)

**Interfaces:**
- Consumes: retained elp struct constructors (`lunar_theory_source_summary`, `lunar_theory_source_family_summary`, `lunar_theory_catalog_summary`, `lunar_theory_capability_summary`, `lunar_theory_limitations_summary`, `lunar_theory_catalog_validation_summary`, evidence `*_summary()` constructors), inherent struct methods, `lunar_theory_source_family` accessor, and `lunar_theory_frame_treatment_summary_details` (all STAY).
- Produces: `crate::posture::elp::<mod>::<renderer>`. **`lunar_theory_source_selection_summary_for_report` is re-exported `pub` from validate** (CLI `cli.rs:529` runtime consumer).

Move, by source module (free prose fns only — keep every `*_summary()` struct constructor and `lunar_theory_frame_treatment_summary_details`):
- **lib.rs → posture/elp/lib_summaries.rs:** `lunar_theory_summary_for_report`, `lunar_theory_summary`, `format_lunar_theory_source_summary`, `lunar_theory_source_summary_for_report`, `lunar_theory_request_policy_summary`, `lunar_theory_frame_treatment_summary_for_report`. (Keep `lunar_theory_frame_treatment_summary() -> &'static str` and `…_details()` — confirm consumers in Step 1.)
- **catalog.rs → posture/elp/catalog.rs:** `format_lunar_theory_catalog_validation_summary`, `lunar_theory_catalog_validation_summary_for_report`, `format_validated_lunar_theory_source_selection_for_report`, `lunar_theory_source_selection_summary`, `lunar_theory_source_selection_summary_for_report`, `validated_lunar_theory_source_selection_summary_for_report`, `lunar_theory_source_family_summary_for_report` (+ its `format_validated_…` helper), `format_lunar_theory_catalog_summary`, `format_validated_lunar_theory_catalog_summary_for_report`, `lunar_theory_catalog_summary_for_report`, `format_lunar_theory_capability_summary`, `format_validated_lunar_theory_capability_summary_for_report`, `lunar_theory_capability_summary_for_report`, `format_lunar_theory_limitations_summary`, `format_validated_lunar_theory_limitations_summary_for_report`, `lunar_theory_limitations_summary_for_report`.
- **evidence.rs → posture/elp/evidence.rs:** `lunar_equatorial_reference_batch_parity_summary_for_report`, `format_lunar_equatorial_reference_batch_parity_summary`, `format_lunar_reference_evidence_summary`, `lunar_reference_evidence_summary_for_report`, `format_lunar_source_window_summary`, `validated_lunar_source_window_summary_for_report`, `lunar_source_window_summary_for_report`, `format_lunar_reference_batch_parity_summary`, `lunar_reference_batch_parity_summary_for_report`, `format_lunar_equatorial_reference_evidence_summary`, `lunar_equatorial_reference_evidence_summary_for_report`, `format_lunar_apparent_comparison_summary`, `lunar_apparent_comparison_summary_for_report`, `lunar_equatorial_reference_evidence_envelope_for_report`, `lunar_high_curvature_continuity_evidence_for_report`, `lunar_high_curvature_equatorial_continuity_evidence_for_report`, `lunar_reference_evidence_envelope_for_report`, and the CLI-referenced `lunar_reference_evidence_envelope_for_report` / `lunar_reference_evidence_summary_for_report` (already listed). Keep every `*_summary()` constructor.
- **source.rs:** the two `*_summary()` fns there (`lunar_theory_source_summary`, `lunar_theory_source_family_summary`) are STRUCT CONSTRUCTORS — they STAY. `posture/elp/source.rs` may end up empty; if so, drop it from `elp/mod.rs`.

- [ ] **Step 1: Enumerate runtime vs test consumers**

Run: `grep -rn "pleiades_elp::" crates/pleiades-validate/src crates/pleiades-cli/src | grep -iE "_for_report|_summary|frame_treatment"`
Classify each as: (a) validate runtime (`release/bundle.rs`, `render/summary/writers.rs`) → repoint to `crate::posture::elp::…`; (b) CLI runtime (`cli.rs:529` → `lunar_theory_source_selection_summary_for_report`) → keep that symbol `pub` in validate, repoint CLI to `pleiades_validate::…`; (c) tests (`bundle_verify_helpers.rs`, `tests/render_catalog.rs`, `cli/tests/summary_commands.rs`) → repoint; (d) retained-constructor / `_details` / `frame_treatment` accessor calls → leave as `pleiades_elp::…`.

- [ ] **Step 2: Move renderers module-by-module** into `posture/elp/{lib_summaries,catalog,evidence}.rs`, per the relocation procedure. Mark `lunar_theory_source_selection_summary_for_report` `pub` (all others `pub(crate)`). Move each renderer's tests with it.

- [ ] **Step 3: Repoint validate + CLI consumers** per the Step-1 classification. For CLI `cli.rs`, `use pleiades_validate::posture::elp::…` (or a re-export path validate already exposes) — no `Cargo.toml` change (CLI already depends on validate).

- [ ] **Step 4: Verify**

Run: `cargo test -p pleiades-validate posture::elp && cargo build -p pleiades-elp -p pleiades-validate -p pleiades-cli`
Expected: PASS + clean build.
Run: `cargo test -p pleiades-cli summary_commands`
Expected: PASS (repointed CLI summary-command tests).
Run: `grep -rn "pub fn .*_for_report" crates/pleiades-elp/src`
Expected: none remain.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-elp crates/pleiades-validate crates/pleiades-cli
git commit -m "refactor(elp,validate,cli): relocate lunar-theory report renderers into validate posture"
```

---

### Task 7: Move pleiades-compression coverage rendering (validate-side recompute)

**Files:**
- Modify: `crates/pleiades-compression/src/format.rs` (remove the coverage-summary *rendering* methods that have no remaining consumer; keep fields, `new`, `validate`)
- Modify: `crates/pleiades-validate/src/posture/compression/mod.rs` (add free-function renderers)
- Modify: `crates/pleiades-validate/src/render/summary/artifact.rs:~183` (repoint)

**Interfaces:**
- Consumes: retained `pleiades_compression::{ArtifactProfileCoverageSummary, ArtifactResidualBodyCoverageSummary}` public fields (`bodies`, `body_count`, `profile`), their `validate()` methods, `CompressedArtifact` (+ `residual_bodies()`), and `ArtifactProfile::summary_for_body_count` (retained profile self-description).
- Produces: `crate::posture::compression::{profile_coverage_summary_line, residual_body_coverage_summary_line, validated_residual_body_coverage_summary_line}` (names chosen to mirror the methods they replace).

The methods being replaced (from `format.rs`): `ArtifactProfileCoverageSummary::{summary_line, validated_summary_line, summary_line_with_bodies, validated_summary_line_with_bodies}` and `ArtifactResidualBodyCoverageSummary::{summary_line, validated_summary_line, summary_line_with_body_count, validated_summary_line_with_body_count}`. Their bodies (from the read of `format.rs:449-560`) are small and reference `self.profile.summary_for_body_count(self.bodies.len())`, `crate::join_display(&self.bodies)`, and `self.validate(...)`. Validate reimplements them as free functions over the public fields.

- [ ] **Step 1: Confirm the real consumer set**

Run: `grep -rn "profile_coverage_summary\|residual_body_coverage_summary\|ArtifactProfileCoverageSummary\|ArtifactResidualBodyCoverageSummary\|\.summary_line_with_bodies\|\.validated_summary_line_with_body_count" crates --include=*.rs | grep -v "fn \|struct \|//"`
Expected: the runtime consumer is `crates/pleiades-validate/src/render/summary/artifact.rs` (`validate_packaged_artifact_generation_residual_bodies_summary` → `summary.validated_summary_line_with_body_count(artifact)`). Enumerate any others (compression's own tests move; any other crate consumer stays working via the retained struct if only fields are used).

- [ ] **Step 2: Write validate free-function renderers**

In `posture/compression/mod.rs`, add a small `join_display` helper and the renderers. Copy the exact bodies from `format.rs` (byte-identical output), swapping `self.` field access for the struct parameter:

```rust
use pleiades_compression::{
    ArtifactProfileCoverageSummary, ArtifactResidualBodyCoverageSummary, CompressedArtifact,
    CompressionError,
};
use pleiades_types::CelestialBody;

fn join_display(bodies: &[CelestialBody]) -> String {
    // Match pleiades_compression::join_display exactly (comma-space separated Display).
    bodies.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(", ")
}

pub(crate) fn profile_coverage_summary_line(summary: &ArtifactProfileCoverageSummary) -> String {
    summary.profile.summary_for_body_count(summary.bodies.len())
}

pub(crate) fn residual_body_coverage_summary_line(
    summary: &ArtifactResidualBodyCoverageSummary,
) -> String {
    match summary.bodies.as_slice() {
        [] => "residual bodies: none".to_string(),
        bodies => format!("residual bodies: {}", join_display(bodies)),
    }
}

pub(crate) fn validated_residual_body_coverage_summary_line_with_body_count(
    summary: &ArtifactResidualBodyCoverageSummary,
    artifact: &CompressedArtifact,
) -> Result<String, CompressionError> {
    summary.validate(artifact)?;
    // `summary_line_with_body_count` == "{summary_line}; applies to {body_count_suffix}"
    Ok(format!(
        "{}; applies to {}",
        residual_body_coverage_summary_line(summary),
        residual_body_count_suffix(summary),
    ))
}
```

> During implementation, read `format.rs:550-600` to copy `body_count_suffix`/`summary_line_with_body_count` and the profile-coverage `summary_line_with_bodies` bodies exactly. Verify `pleiades_compression::join_display`'s separator matches the local helper before finalizing (adjust if it differs). Only implement the specific variants the Step-1 consumer set actually needs; do not port unused variants.

- [ ] **Step 3: Repoint the validate consumer**

In `render/summary/artifact.rs`, replace `summary.validated_summary_line_with_body_count(artifact)` with `crate::posture::compression::validated_residual_body_coverage_summary_line_with_body_count(summary, artifact)`.

- [ ] **Step 4: Move the compression rendering tests + remove now-dead methods**

Move the `#[test]`s that assert on these rendering methods into `posture/compression/mod.rs`. Remove from `format.rs` only the rendering methods with no remaining consumer (confirmed in Step 1); keep any variant still used elsewhere. Keep all fields, `new`, `validate`, and the `Display` impls only if a consumer needs them (else remove the `Display` that calls the removed `summary_line`).

- [ ] **Step 5: Verify byte-identity**

Run: `cargo test -p pleiades-validate posture::compression && cargo build -p pleiades-compression -p pleiades-validate -p pleiades-cli`
Expected: PASS + clean build. The moved rendering tests (which assert exact strings) prove byte-identity.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-compression crates/pleiades-validate
git commit -m "refactor(compression,validate): relocate artifact coverage-summary rendering into validate posture"
```

---

### Task 8: Consumer repoint sweep + release-bundle pin verification

**Files:**
- Modify (if any residual): `crates/pleiades-validate/src/release/{bundle.rs,bundle_verify_helpers.rs}`, `crates/pleiades-cli/src/**`

**Interfaces:**
- Consumes: all `crate::posture::*` renderers from Tasks 2–7.
- Produces: a workspace with zero remaining references to the moved symbols under their old `pleiades_<crate>::…` paths, and unchanged bundle checksums.

- [ ] **Step 1: Find any straggler references**

Run: `grep -rn "pleiades_houses::.*summary_line\|pleiades_ayanamsa::.*_for_report\|pleiades_vsop87::.*_for_report\|pleiades_vsop87::format_\|pleiades_elp::.*_for_report\|pleiades_elp::format_\|\.validated_summary_line_with_body_count" crates --include=*.rs`
Expected: no hits except retained-constructor calls. Repoint any straggler to its `crate::posture::…` (validate) or `pleiades_validate::…` (CLI) path.

- [ ] **Step 2: Verify release-bundle checksums unchanged**

Run: `mise run release-smoke`
Expected: PASS with no checksum diff. If a fnv1a64 value changed, a moved renderer drifted — fix the move to restore byte-identity; **do not** regenerate the checksum.

- [ ] **Step 3: Full workspace test**

Run: `cargo test --workspace --include-ignored`
Expected: PASS.

- [ ] **Step 4: Commit** (only if Step 1 changed files)

```bash
git add crates
git commit -m "refactor(validate,cli): repoint remaining Slice B renderer consumers to posture paths"
```

---

### Task 9: Close-out — grep gate, CI, CHANGELOG, PLAN.md

**Files:**
- Modify: `CHANGELOG.md`, `PLAN.md`

**Interfaces:**
- Consumes: green workspace from Task 8.
- Produces: documented Slice B close-out; the program moves to Slices C and D remaining.

- [ ] **Step 1: Assert no `_for_report` exports remain in the five crates**

Run: `grep -rn "pub fn .*_for_report" crates/pleiades-houses/src crates/pleiades-ayanamsa/src crates/pleiades-vsop87/src crates/pleiades-elp/src crates/pleiades-compression/src`
Expected: **no output.** (This is the slice's structural gate. Inherent struct-method `summary_line` remain — expected.)

- [ ] **Step 2: Run full CI**

Run: `mise run ci`
Expected: PASS — fmt, clippy `-D warnings`, `cargo test --workspace --include-ignored`, `cargo doc -D warnings`, workspace-audit, package-check, release-smoke, claims-audit all green.

- [ ] **Step 3: Update CHANGELOG.md**

Add a Slice B entry under Unreleased, in the style of the Slice A entry: report prose for houses/ayanamsa/vsop87/elp/compression relocated into `pleiades-validate` posture modules; elp backend metadata decoupled from the source-family report helper (coupling 3); no output/version changes (compatibility `0.7.13`, API-stability `0.3.0`); Slices C (`pleiades-data`) and D (`pleiades-jpl`) remain before the workspace `0.4.0` release.

- [ ] **Step 4: Refresh PLAN.md status line**

Update the report-surface relocation status (currently "slice A done … slices B … C … D remain") to record Slice B delivered and only C, D remaining.

- [ ] **Step 5: Commit**

```bash
git add CHANGELOG.md PLAN.md
git commit -m "docs: report-surface relocation Slice B close-out (CHANGELOG, PLAN status)"
```

- [ ] **Step 6: Open the PR** (mirrors Slice A's PR #19 flow)

```bash
git push -u origin feat/report-surface-relocation-slice-b
gh pr create --title "feat(validate): report-surface relocation Slice B (houses/ayanamsa/vsop87/elp/compression)" --body "Relocates report prose from five functional crates into pleiades-validate posture modules; resolves coupling 3 (elp backend metadata). Pure relocation — byte-identical output, no version bump. Slice B of the report-surface relocation program (design: docs/superpowers/specs/2026-07-10-report-surface-relocation-design.md; slice spec: docs/superpowers/specs/2026-07-11-report-surface-relocation-slice-b-design.md)."
```

---

## Self-Review

**Spec coverage** — every spec section maps to a task:
- Partition rule → Global Constraints + every move task.
- houses/ayanamsa/vsop87/elp/compression module map → Tasks 2, 3, 4, 6, 7 (+ scaffold Task 1).
- Coupling 3 → Task 5 (ordered before Task 6, per spec).
- Consumer migrations (validate bundle/writers/render, CLI) → Tasks 2–7 inline + Task 8 sweep.
- Visibility (`pub(crate)` default, `pub` for `lunar_theory_source_selection_summary_for_report`) → Task 6.
- Invariants (byte-identity, profile `0.7.13`, no behavior change, dependency direction) → Global Constraints + Task 8 release-smoke + Task 9 CI/grep gate.
- Verification (`mise run ci`, release-smoke, grep assertion) → Tasks 8–9.
- Non-goals (no version bump, no C/D, no validate split) → Global Constraints.

**Placeholder scan:** No "TBD"/"handle edge cases" placeholders. Two steps intentionally instruct reading exact source line ranges before copying verbatim bodies (Task 7 Step 2 `body_count_suffix`; Task 5 Step 1 fixture capture) — this is inherent to a *verbatim relocation* (the canonical text lives in the source file and must be copied, not re-invented), not an under-specification. Symbol lists are explicit per task.

**Type consistency:** Retained vs moved symbols are named consistently across tasks (`lunar_theory_source_family_summary` constructor stays; `lunar_theory_source_family_summary_for_report` moves and is consumed by coupling-3's inline replacement, which uses the retained `.validate()`/`.summary_line()` methods). CLI's `pub` symbol `lunar_theory_source_selection_summary_for_report` is named identically in Task 6 produce-interface and consumer repoint. Compression free-fn names introduced in Task 7 are used identically in its Step 3 repoint and Task 8 straggler grep.
