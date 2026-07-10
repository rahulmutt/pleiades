# Report-Surface Relocation — Slice A Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the report/validation prose layer from `pleiades-core` and `pleiades-backend` (moving it into `pleiades-validate`), strip the three global-policy lines from `ChartSnapshot::Display`, decouple core's compatibility rendering from `pleiades-ayanamsa`'s report helper, delete dead report helpers in time/apparent/ayanamsa/houses, and bump the API-stability profile to 0.3.0.

**Architecture:** Slice A of `docs/superpowers/specs/2026-07-10-report-surface-relocation-design.md`. Functional crates keep structured data and the backend contract; all policy/report prose moves to a new `pleiades-validate/src/posture/` module (`pub(crate)` except the 9 helpers `pleiades-cli` calls). Rendered release text stays byte-identical everywhere except the one deliberate `ChartSnapshot::Display` change.

**Tech Stack:** Rust workspace (cargo, mise tasks). Test/CI gates: `cargo test -p <crate>`, `mise run ci` (fmt, clippy `-D warnings`, test-full, docs, workspace-audit, package-check, release-smoke, claims-audit).

## Global Constraints

- **Byte-identity:** every moved renderer must produce exactly the prose it produced before. Compatibility profile id stays `0.7.13`; `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` in `crates/pleiades-core/src/compatibility/mod.rs` must NOT change. If a profile-checksum test fails, the slice has a bug — do not regenerate the checksum.
- **Only deliberate output change:** removal of the `Time-scale policy:` / `Frame policy:` / `Apparentness policy:` lines from `ChartSnapshot`'s `Display` (Task 1). The per-snapshot `Observer policy:` lines stay.
- **Dependency direction:** nothing below `pleiades-validate` may import from it. `FrameTreatmentSummary` + `FrameTreatmentSummaryValidationError` stay in `pleiades-backend`.
- **Backend contract stays:** `validate_request_policy`, `validate_request_observer_location` (pub(crate)), `validate_request_against_metadata`, `validate_requests_against_metadata`, `validate_zodiac_policy`, `validate_observer_policy` remain in `pleiades-backend`.
- **No version bump of workspace crates** in this slice (0.4.0 happens after Slice D). API-stability profile id: `pleiades-api-stability/0.2.2` → `pleiades-api-stability/0.3.0` (Task 7 only).
- Branch: create `feat/report-surface-relocation-slice-a` from `main` before Task 1.
- Every task ends with the named test commands passing and a commit. `#![deny(missing_docs)]` is active in these crates: every new `pub` item needs a doc comment.

---

### Task 1: Strip global-policy lines from `ChartSnapshot::Display`

**Files:**
- Modify: `crates/pleiades-core/src/chart/snapshot.rs:3-5` (imports), `:687-734` (Display impl)
- Modify: `crates/pleiades-core/src/chart/tests.rs` (Display pins)
- Modify: `crates/pleiades-cli/src/cli/tests/validation.rs`, `crates/pleiades-cli/src/cli/tests/summary_commands.rs` (only assertions pinning **chart command output**, not validate report output)
- Modify: `crates/pleiades-validate/src/tests/snapshot_render.rs` (only if it pins chart Display text)

**Interfaces:**
- Produces: `ChartSnapshot`'s `Display` no longer emits the three lines `Time-scale policy: …`, `Frame policy: …`, `Apparentness policy: …`. All other lines unchanged. Later tasks (6) rely on `snapshot.rs` having **zero** `pleiades_backend` policy-summary imports.

- [ ] **Step 1: Write/adjust the failing test.** In `crates/pleiades-core/src/chart/tests.rs`, find the test(s) asserting Display content (grep `"Time-scale policy"` in that file). Change the assertions to the new posture:

```rust
let rendered = snapshot.to_string();
assert!(!rendered.contains("Time-scale policy:"));
assert!(!rendered.contains("Frame policy:"));
assert!(!rendered.contains("Apparentness policy:"));
// unchanged lines still present:
assert!(rendered.contains("Backend: "));
assert!(rendered.contains("Observer policy:"));
assert!(rendered.contains("Zodiac mode:"));
assert!(rendered.contains("Apparentness:"));
```

- [ ] **Step 2: Run to verify it fails.** Run: `cargo test -p pleiades-core --lib chart`. Expected: FAIL on the new `!contains` assertions (lines still rendered).

- [ ] **Step 3: Edit the Display impl.** In `crates/pleiades-core/src/chart/snapshot.rs` delete the three `writeln!` blocks:
  - lines ~695-699 (`"Time-scale policy: {}"` + `time_scale_policy_summary_for_report().summary_line()`)
  - lines ~723-727 (`"Frame policy: {}"` + `validated_frame_policy_summary_for_report()`)
  - lines ~730-734 (`"Apparentness policy: {}"` + `request_policy_summary_for_report().apparentness`)

  Then remove the now-unused imports at lines 3-5 (`request_policy_summary_for_report`, `time_scale_policy_summary_for_report`, `validated_frame_policy_summary_for_report` — keep `Apparentness` if still used elsewhere in the file; check with `grep -n "Apparentness" crates/pleiades-core/src/chart/snapshot.rs`).

- [ ] **Step 4: Run core tests.** Run: `cargo test -p pleiades-core`. Fix any remaining Display pins the compiler/tests surface (only by deleting assertions on the three removed labels). Expected: PASS.

- [ ] **Step 5: Fix downstream pins.** Run: `cargo test -p pleiades-cli -p pleiades-validate 2>&1 | head -50`. For each failure that pins **chart output** (chart subcommand tests, snapshot-render tests), delete/adjust only assertions referencing the three removed labels. Do NOT touch validate's own policy-report renderers (`render/text/policy.rs`, `render/summary/*`) — their `Time-scale policy:` labels are report output and unchanged. Expected after fixes: PASS.

- [ ] **Step 6: Commit.**

```bash
git add -A && git commit -m "feat(core)!: drop global policy prose from ChartSnapshot::Display (report-surface relocation slice A)"
```

---

### Task 2: Delete dead report helpers (time, apparent, ayanamsa, houses)

**Files:**
- Delete: `crates/pleiades-time/src/policy.rs`
- Modify: `crates/pleiades-time/src/lib.rs:39` (`pub mod policy;`), `:51` (`pub use policy::{CivilTimePolicyError, CivilTimePolicySummary};`)
- Delete: `crates/pleiades-apparent/src/policy.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs:127-131` (`pub mod policy;` + the `pub use policy::{…}` block)
- Modify: `crates/pleiades-ayanamsa/src/thresholds.rs:106` (delete `pub fn ayanamsa_thresholds_summary_for_report` + its tests in the same file)
- Modify: `crates/pleiades-houses/src/thresholds.rs:104` (delete `pub fn house_thresholds_summary_for_report` + its tests)

**Interfaces:**
- Consumes: nothing. All four targets were verified to have zero consumers outside their own crate's tests.
- Produces: nothing — pure deletion. Threshold *data* (ceiling constants/structs) in the two `thresholds.rs` files stays.

- [ ] **Step 1: Prove deadness.** Run:

```bash
grep -rn --include=*.rs -w -e CivilTimePolicySummary -e ApparentPlacePolicySummary \
  -e ayanamsa_thresholds_summary_for_report -e house_thresholds_summary_for_report crates \
  | grep -v -e pleiades-time/src -e pleiades-apparent/src \
            -e pleiades-ayanamsa/src/thresholds.rs -e pleiades-houses/src/thresholds.rs
```

Expected: no output. If any hit appears, STOP — that symbol is live; report back instead of deleting.

- [ ] **Step 2: Delete.** Remove the two `policy.rs` files and their `pub mod`/`pub use` lines; delete the two `*_thresholds_summary_for_report` functions and any tests in the same files that call them (keep all other threshold data/tests).

- [ ] **Step 3: Verify.** Run: `cargo test -p pleiades-time -p pleiades-apparent -p pleiades-ayanamsa -p pleiades-houses && cargo build --workspace`. Expected: PASS.

- [ ] **Step 4: Commit.**

```bash
git add -A && git commit -m "chore!: delete dead report helpers (time/apparent policy prose, ayanamsa/houses thresholds summaries)"
```

---

### Task 3: Decouple core's compatibility rendering from `pleiades-ayanamsa`'s report helper

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/mod.rs` (add private builder)
- Modify: `crates/pleiades-core/src/compatibility/profile.rs:196-210` (two methods)
- Modify: `crates/pleiades-core/src/compatibility/report.rs:111-115` (remove tautological validate call)
- Modify: `crates/pleiades-core/src/compatibility/validation.rs:73` (remove `AyanamsaProvenanceSummaryValidationFailed` variant + its Display arm)
- Test: `crates/pleiades-core/src/compatibility/tests.rs`

**Interfaces:**
- Consumes: `pleiades_ayanamsa::provenance_sample_ayanamsas()`, `pleiades_ayanamsa::descriptor()` (structured data — both stay public forever).
- Produces: `pub(crate) fn ayanamsa_provenance_summary_text() -> String` in `compatibility/mod.rs`. `CompatibilityProfile::ayanamsa_provenance_summary_line()` keeps its signature (`&self -> String`); `validated_ayanamsa_provenance_summary_line()` keeps `Result<String, CompatibilityProfileValidationError>`.

- [ ] **Step 1: Write the byte-identity test** in `crates/pleiades-core/src/compatibility/tests.rs`:

```rust
#[test]
fn rebuilt_ayanamsa_provenance_line_matches_ayanamsa_crate_rendering() {
    // Guards the slice-A decoupling: core's rebuilt derivation must be
    // byte-identical to the pleiades-ayanamsa renderer it replaced.
    // (Slice B deletes the ayanamsa renderer; this test then converts to a
    // pinned literal — see the slice-B plan.)
    assert_eq!(
        super::ayanamsa_provenance_summary_text(),
        pleiades_ayanamsa::validated_provenance_summary_for_report()
            .expect("ayanamsa provenance summary should validate")
    );
}
```

- [ ] **Step 2: Run to verify it fails.** Run: `cargo test -p pleiades-core compatibility::tests::rebuilt_ayanamsa`. Expected: FAIL — `ayanamsa_provenance_summary_text` not defined.

- [ ] **Step 3: Add the builder** in `crates/pleiades-core/src/compatibility/mod.rs` (replicates `AyanamsaProvenanceSummary::new().summary_line()` from `crates/pleiades-ayanamsa/src/model.rs:249-289` exactly):

```rust
/// Rebuilds the representative ayanamsa provenance line from structured
/// catalog data. Byte-identical to the retired
/// `pleiades_ayanamsa::validated_provenance_summary_for_report` rendering.
pub(crate) fn ayanamsa_provenance_summary_text() -> String {
    let examples = pleiades_ayanamsa::provenance_sample_ayanamsas()
        .iter()
        .map(|ayanamsa| {
            let descriptor = pleiades_ayanamsa::descriptor(ayanamsa)
                .expect("provenance sample ayanamsa should exist in the built-in catalog");
            format!("{} — {}", descriptor.canonical_name, descriptor.notes)
        })
        .collect::<Vec<_>>()
        .join("; ");
    format!("representative provenance examples: {examples}")
}
```

- [ ] **Step 4: Repoint the two profile methods** in `profile.rs:196-210`:

```rust
    /// Returns the representative ayanamsa provenance payload surfaced in the compatibility profile.
    pub fn ayanamsa_provenance_summary_line(&self) -> String {
        super::ayanamsa_provenance_summary_text()
    }

    /// Returns the representative ayanamsa provenance payload after validating the profile.
    pub fn validated_ayanamsa_provenance_summary_line(
        &self,
    ) -> Result<String, CompatibilityProfileValidationError> {
        self.validate()?;
        Ok(super::ayanamsa_provenance_summary_text())
    }
```

(Adjust the `super::` path to wherever the builder lives relative to `profile.rs`.)

- [ ] **Step 5: Remove the tautological validate hook.** In `report.rs:111-115` delete the `pleiades_ayanamsa::validated_provenance_summary_for_report().map_err(…)?;` block (its `validate()` compares a fresh build against a fresh build — it can never fail). In `validation.rs:73` delete the now-unused `AyanamsaProvenanceSummaryValidationFailed` variant and its `Display` arm; fix any test constructing it (delete that test case — the drift it guarded is impossible by construction).

- [ ] **Step 6: Verify byte-identity and profile stability.** Run: `cargo test -p pleiades-core`. Expected: PASS, including the new equality test and the existing compatibility-profile content-checksum test (the rendered profile text must be unchanged).

- [ ] **Step 7: Commit.**

```bash
git add -A && git commit -m "refactor(core): rebuild ayanamsa provenance line from structured catalog data (decouple from ayanamsa report helper)"
```

---

### Task 4: Relocate unsupported-modes prose into core's compatibility posture

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/mod.rs` (new const), `crates/pleiades-core/src/compatibility/profile.rs` (new method), `crates/pleiades-core/src/compatibility/report.rs:386`
- Modify: `crates/pleiades-validate/src/lib.rs:234` (drop import), `crates/pleiades-validate/src/render/summary/policy.rs:127,135`, `crates/pleiades-validate/src/render/text/compatibility.rs:56,170`, `crates/pleiades-validate/src/tests/compatibility.rs:70,362`
- Test: `crates/pleiades-core/src/compatibility/tests.rs`

**Interfaces:**
- Produces: `CompatibilityProfile::unsupported_modes_summary_line(&self) -> &'static str` (new public method). Task 6 relies on core and validate having **zero** references to `pleiades_backend::unsupported_modes_summary_for_report` / `CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT`.

- [ ] **Step 1: Write the failing test** in `compatibility/tests.rs`:

```rust
#[test]
fn unsupported_modes_line_is_owned_by_the_compatibility_posture() {
    let profile = current_compatibility_profile();
    // Byte-identical to the backend constant this replaces (deleted in Task 6).
    assert_eq!(
        profile.unsupported_modes_summary_line(),
        pleiades_backend::CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT
    );
    assert!(profile
        .to_string()
        .contains(&format!("Unsupported modes: {}", profile.unsupported_modes_summary_line())));
}
```

- [ ] **Step 2: Run to verify it fails.** Run: `cargo test -p pleiades-core unsupported_modes_line_is_owned`. Expected: FAIL — method not defined.

- [ ] **Step 3: Implement.** In `compatibility/mod.rs` add (text copied verbatim from `crates/pleiades-backend/src/policy/mod.rs:42-44`):

```rust
/// Canonical current unsupported-modes summary text used by release reporting.
pub(crate) const CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT: &str =
    "built-in UTC convenience remains out of scope; built-in Delta T remains out of scope; native sidereal backend output remains unsupported unless a backend explicitly advertises it";
```

In `profile.rs` add to `impl CompatibilityProfile`:

```rust
    /// Returns the unsupported-advanced-modes posture line surfaced in release reporting.
    pub const fn unsupported_modes_summary_line(&self) -> &'static str {
        super::CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT
    }
```

In `report.rs:386` replace `pleiades_backend::unsupported_modes_summary_for_report()` with `self.unsupported_modes_summary_line()`.

- [ ] **Step 4: Run core tests.** Run: `cargo test -p pleiades-core`. Expected: PASS (incl. the profile content-checksum test — text unchanged).

- [ ] **Step 5: Migrate validate.** Remove `unsupported_modes_summary_for_report` from the `use pleiades_backend::{…}` block at `crates/pleiades-validate/src/lib.rs:231-238`. At each of the six call sites listed above replace `unsupported_modes_summary_for_report()` with `current_compatibility_profile().unsupported_modes_summary_line()` (add the `pleiades_core::current_compatibility_profile` import where missing; `render/text/compatibility.rs` sites already take a `&CompatibilityProfile` parameter in scope — prefer `profile.unsupported_modes_summary_line()` there).

- [ ] **Step 6: Verify.** Run: `cargo test -p pleiades-validate -p pleiades-core && grep -rn "unsupported_modes_summary_for_report" crates/pleiades-core crates/pleiades-validate`. Expected: tests PASS; grep only matches nothing (zero hits in those two crates).

- [ ] **Step 7: Commit.**

```bash
git add -A && git commit -m "refactor(core): own unsupported-modes posture line in the compatibility profile"
```

---

### Task 5: Delete core's report-wrapper free functions; migrate validate to posture-type methods

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/mod.rs:102-282` (delete the free functions), `crates/pleiades-core/src/release_profiles.rs:157-169` (delete both free functions)
- Modify: `crates/pleiades-core/src/lib.rs:91-126,166-170` (export lists)
- Modify: `crates/pleiades-core/src/compatibility/tests.rs`, `crates/pleiades-core/src/release_profiles.rs` tests, `crates/pleiades-core/src/lib.rs` tests + doctest
- Modify: `crates/pleiades-validate/src/lib.rs:240-262` (import block) and the call sites it feeds: `src/render/text/catalog.rs`, `src/render/text/compatibility.rs:205`, `src/render/summary/release.rs`, `src/render/summary/backend.rs:202-214`, `src/release/notes.rs:619`, `src/compatibility/mod.rs:495,913`
- Create: caveats composer inside `crates/pleiades-validate/src/render/text/compatibility.rs`

**Interfaces:**
- Consumes: `CompatibilityProfile`'s existing methods (`catalog_posture_summary_line`, `validated_catalog_posture_summary_line`, `validated_house_formula_families_summary_line`, `validated_latitude_sensitive_house_systems_summary_line`, `validated_latitude_sensitive_house_constraints_summary_line`, `validated_latitude_sensitive_house_failure_modes_summary_line`, `validated_custom_definition_ayanamsa_labels_summary_line`, `validated_catalog_inventory_summary_line`, `validated_house_code_aliases_summary_line`, `validated_known_gaps_summary_line`, `validated_release_house_system_canonical_names_summary_line`, `validated_release_ayanamsa_canonical_names_summary_line`, `validated_target_house_scope_summary_line`, `validated_target_ayanamsa_scope_summary_line` — all already defined in `profile.rs`) and `ReleaseProfileIdentifiers::{summary_line, validated_summary_line}`.
- Produces: `pleiades-core` exports NO `*_summary_for_report` functions. `validate_custom_definition_labels`, `current_compatibility_profile`, `CompatibilityProfile`, `current_api_stability_profile`, `ApiStabilityProfile`, `current_release_profile_identifiers`, `ReleaseProfileIdentifiers` (+ their ValidationError types, which appear in public signatures) remain exported.

- [ ] **Step 1: Delete the free functions.** In `compatibility/mod.rs` delete every free `pub fn *_summary_for_report` (lines 102-282): the 12 unvalidated wrappers, the 13 `validated_*` wrappers, `catalog_posture_summary_for_report`, and `compatibility_caveats_summary_for_report` — but first copy the full body of `compatibility_caveats_summary_for_report` (mod.rs:191-227) for Step 3. In `release_profiles.rs` delete `release_profile_identifiers_summary_for_report` and `validated_release_profile_identifiers_summary_for_report` (lines 157-169). Update `lib.rs`'s `pub use compatibility::{…}` and `pub use release_profiles::{…}` lists to export only: `current_compatibility_profile`, `current_compatibility_profile_id`, `validate_custom_definition_labels`, `CompatibilityProfile`, `HouseCodeAliasInventorySummary`, `CURRENT_COMPATIBILITY_PROFILE_ID`, `current_release_profile_identifiers`, `ReleaseProfileIdentifiers`, `ReleaseProfileIdentifiersValidationError` (id-item downgrades happen in Step 6).

- [ ] **Step 2: Fix core's own tests.** Run: `cargo test -p pleiades-core 2>&1 | head -40`. Delete the re-export round-trip tests in `lib.rs` (e.g. `compatibility_catalog_summary_helpers_match_the_current_profile` at lib.rs:664-687) and the wrapper tests in `compatibility/tests.rs:364,382,1543,1725,1729` region and `release_profiles.rs:287-300` that exercised deleted functions; where a test pins real rendered text, rewrite it against the profile method instead of deleting (e.g. `current_compatibility_profile().validated_catalog_inventory_summary_line().unwrap()`). Expected: `cargo test -p pleiades-core` PASS.

- [ ] **Step 3: Move the caveats composer into validate.** In `crates/pleiades-validate/src/render/text/compatibility.rs` add the function copied verbatim from the old `compatibility/mod.rs:191-227` body, renamed and privatized:

```rust
/// Composes the compatibility caveats line from the profile and release
/// identifiers. Moved verbatim from pleiades-core (report-surface relocation
/// slice A); rendered text is byte-identical.
fn compatibility_caveats_summary_text(
    profile: &CompatibilityProfile,
    identifiers: &ReleaseProfileIdentifiers,
) -> String {
    // <verbatim body from crates/pleiades-core/src/compatibility/mod.rs:191-227>
}
```

Repoint the sole caller (`render/text/compatibility.rs:205`, currently `core_compatibility_caveats_summary_for_report(…)`).

- [ ] **Step 4: Migrate validate's aliased imports.** In `crates/pleiades-validate/src/lib.rs:240-262` remove every deleted symbol from the `use pleiades_core::{…}` block. Replace call sites using this mapping (mechanical; the compiler lists every site):

| old free function (aliases included) | replacement |
| --- | --- |
| `core_catalog_posture_summary_for_report()` | `current_compatibility_profile().catalog_posture_summary_line()` |
| `core_validated_catalog_posture_summary_for_report()` | `current_compatibility_profile().validated_catalog_posture_summary_line()` (unwrap/`?` exactly as the old call site did) |
| `validated_house_formula_families_summary_for_report()` | `current_compatibility_profile().validated_house_formula_families_summary_line()` |
| `validated_latitude_sensitive_house_systems/constraints/failure_modes_summary_for_report()` | same-named `…_summary_line()` methods |
| `validated_custom_definition_ayanamsa_labels_summary_for_report()` | `current_compatibility_profile().validated_custom_definition_ayanamsa_labels_summary_line()` |
| `core_validated_catalog_inventory_summary_for_report()` | `current_compatibility_profile().validated_catalog_inventory_summary_line()` |
| `core_validated_house_code_aliases_summary_for_report()` | `current_compatibility_profile().validated_house_code_aliases_summary_line()` |
| `validated_known_gaps_summary_for_report()` | `current_compatibility_profile().validated_known_gaps_summary_line()` |
| `core_validated_release_house_system/ayanamsa_canonical_names_summary_for_report()` | matching `…_summary_line()` methods |
| `core_validated_target_house/ayanamsa_scope_summary_for_report()` | matching `…_summary_line()` methods |
| `core_validated_release_profile_identifiers_summary_for_report()` | `current_release_profile_identifiers().validated_summary_line()` |
| `core_compatibility_caveats_summary_for_report(…)` | local `compatibility_caveats_summary_text(…)` (Step 3) |

If a method name in this table doesn't exist on `CompatibilityProfile`, find the real name in `crates/pleiades-core/src/compatibility/profile.rs` (all wrappers were one-line delegations — the method always exists; only naming may differ slightly).

- [ ] **Step 5: Verify.** Run: `cargo test -p pleiades-validate -p pleiades-cli && cargo test -p pleiades-core`. Expected: PASS — all rendered report text byte-identical (bundle-verify tests prove it).

- [ ] **Step 6: Downgrade internal-only pub items.** For each of `CURRENT_COMPATIBILITY_PROFILE_ID`, `CURRENT_API_STABILITY_PROFILE_ID`, `current_compatibility_profile_id`, `current_api_stability_profile_id`, `HouseCodeAliasInventorySummary`: run `grep -rnw --include=*.rs <symbol> crates | grep -v pleiades-core/src` — if no hits, change to `pub(crate)` and remove from `lib.rs` exports; if hits exist (or the compiler raises E0446 private-type-in-public-interface), leave it `pub` and note which consumer pinned it in the commit message. Keep `ApiStabilityProfileValidationError`/`ReleaseProfileIdentifiersValidationError`/`CompatibilityProfileValidationError` public (they appear in public `validate()` signatures).

- [ ] **Step 7: Verify + commit.**

```bash
cargo test -p pleiades-core -p pleiades-validate -p pleiades-cli && \
grep -rn "_summary_for_report" crates/pleiades-core/src/lib.rs ; \
git add -A && git commit -m "feat(core)!: remove report-wrapper free functions; validate consumes posture-type methods"
```

Expected: tests PASS; the grep prints nothing.

---

### Task 6: Move the backend policy-prose layer into `pleiades-validate/src/posture/`

**Files:**
- Create: `crates/pleiades-validate/src/posture/mod.rs`, `crates/pleiades-validate/src/posture/backend_policy.rs`
- Modify: `crates/pleiades-backend/src/policy/mod.rs` (constants out), `crates/pleiades-backend/src/policy/current.rs` (report half out), `crates/pleiades-backend/src/policy/frame.rs` (keep only `FrameTreatmentSummary` + error), `crates/pleiades-backend/src/lib.rs` (exports)
- Delete: `crates/pleiades-backend/src/policy/{apparentness,delta_t,native_sidereal,observer,pluto_fallback,request,time_scale,utc,zodiac}.rs`, `crates/pleiades-backend/src/policy_tests.rs`
- Modify: `crates/pleiades-backend/src/request_tests.rs:787-818` (report assertions out)
- Modify: `crates/pleiades-core/src/lib.rs:132-152,172` (drop policy re-exports), `crates/pleiades-core/src/lib.rs` tests :573-662 (re-export round-trip tests)
- Modify: `crates/pleiades-validate/src/lib.rs` (module decl, re-exports, `use pleiades_backend::{…}` block :231-238), `src/render/text/policy.rs`, `src/render/summary/{policy,backend,report,release}.rs`, `src/release/notes.rs`, `src/release/bundle.rs`, `src/release/bundle_verify_helpers.rs`, `src/render/text/{compatibility,evidence,corpus}.rs`, `src/tests/{compatibility,render_request}.rs` (imports)
- Modify: `crates/pleiades-cli/src/help.rs:3-9`, `crates/pleiades-cli/src/cli/tests/misc.rs` (imports)

**Interfaces:**
- Consumes: nothing new — verbatim relocation. The moved code keeps using `pleiades_types` items it already used.
- Produces:
  - `pleiades-validate/src/posture/backend_policy.rs` hosting, **verbatim**: the 9 `CURRENT_*_POLICY_SUMMARY_TEXT` constants (all from `policy/mod.rs:13-59` EXCEPT `CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT`, which now lives in core — delete backend's copy and its `unsupported_modes_summary_for_report` at `current.rs:24`); the 10 summary structs + ValidationErrors from the nine deleted submodules plus `FramePolicySummary`/`FramePolicySummaryValidationError` cut from `frame.rs`; every report fn from `current.rs` (all `pub fn` EXCEPT the six contract fns listed in Global Constraints — that is: everything matching `*_summary_for_report`, `current_*_policy_summary`, `current_*_summary`, `validated_*_for_report`, `request_semantics_summary_for_report`, `frame_policy_summary_details`, `zodiac_policy_summary_for_report`); `policy_tests.rs` as its `#[cfg(test)] mod tests`.
  - Validate `lib.rs` re-exports (pub, for pleiades-cli): `validated_request_policy_summary_for_report`, `validated_request_semantics_summary_for_report`, `validated_time_scale_policy_summary_for_report`, `validated_utc_convenience_policy_summary_for_report`, `validated_delta_t_policy_summary_for_report`, `validated_observer_policy_summary_for_report`, `validated_apparentness_policy_summary_for_report`, `validated_native_sidereal_policy_summary_for_report`, `validated_frame_policy_summary_for_report`. Everything else in `posture` is `pub(crate)`.
  - `pleiades-backend` public surface afterwards: contract types + the six `validate_*` fns + `FrameTreatmentSummary`/`FrameTreatmentSummaryValidationError`. Nothing matching `_for_report` or `PolicySummary` (except `FrameTreatmentSummary…`).

- [ ] **Step 1: Create the posture module.** `crates/pleiades-validate/src/posture/mod.rs`:

```rust
//! Policy/report posture prose relocated from the functional crates
//! (report-surface relocation program). Rendering only — the functional
//! crates keep the structured data and contract validation.

pub(crate) mod backend_policy;
```

Add `mod posture;` to `crates/pleiades-validate/src/lib.rs` and the nine `pub use posture::backend_policy::{…}` re-exports listed in Interfaces.

- [ ] **Step 2: Move the code.** Copy verbatim into `posture/backend_policy.rs`: the 9 constants, the nine submodule files' structs/impls (concatenated; drop the per-file `use` duplication as needed), the `FramePolicySummary` half of `frame.rs`, and the report fns from `current.rs`. Append `policy_tests.rs` content as `#[cfg(test)] mod tests` (fix `use` paths from `crate::policy::…`/`crate::…` to `super::…` / `pleiades_backend::…`). Where moved code referenced `FrameTreatmentSummary`, import it: `use pleiades_backend::FrameTreatmentSummary;`.

- [ ] **Step 3: Shrink pleiades-backend.** Delete the nine submodule files and `policy_tests.rs`; cut the moved constants from `policy/mod.rs` and the moved fns from `current.rs` (keep the six contract fns and their tests); cut `FramePolicySummary`+error from `frame.rs` (keep `FrameTreatmentSummary`+error); delete backend's `CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT` and `unsupported_modes_summary_for_report`. Update `policy/mod.rs`'s re-export block and `lib.rs:103-141` exports to match. Move the six report assertions at `request_tests.rs:787-818` into `posture/backend_policy.rs`'s test module (they test moved fns). Also update the Task-4 test `unsupported_modes_line_is_owned_by_the_compatibility_posture` in `crates/pleiades-core/src/compatibility/tests.rs`: it compared against the now-deleted `pleiades_backend::CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT` — replace that comparison with the pinned literal:

```rust
    assert_eq!(
        profile.unsupported_modes_summary_line(),
        "built-in UTC convenience remains out of scope; built-in Delta T remains out of scope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
    );
```

- [ ] **Step 4: Shrink pleiades-core.** In `crates/pleiades-core/src/lib.rs` remove from the `pub use pleiades_backend::{…}` block every summary fn/struct (`*_summary_for_report`, `current_*_summary`, `*PolicySummary`, `PlutoFallbackSummary`, `DeltaTPolicySummary`, etc.) and the stray `pub use pleiades_backend::request_semantics_summary_for_report;` at line 172 — keep contract types (`AccuracyClass` … `RoutingBackend` list from the design's "what stays"). Delete the re-export round-trip tests at `lib.rs:573-662` (`utc_convenience_policy_summary_is_reexported_from_backend`, `delta_t_…`, `request_policy_component_…`, `request_semantics_…`, `native_sidereal_…`, `pluto_fallback_…`).

- [ ] **Step 5: Repoint consumers.** Run `cargo build --workspace 2>&1 | head -60` and fix, in this order:
  - `crates/pleiades-validate/src/lib.rs:231-238` and every validate module importing policy summaries from `pleiades_backend` or `pleiades_core` → `use crate::posture::backend_policy::{…};`
  - `crates/pleiades-cli/src/help.rs:3-9` → `use pleiades_validate::{validated_apparentness_policy_summary_for_report, …};` (the nine re-exports)
  - `crates/pleiades-cli/src/cli/tests/misc.rs` → same source.

- [ ] **Step 6: Verify byte-identity and full workspace.** Run: `cargo test --workspace`. Expected: PASS — in particular validate's release-bundle verify tests (checksums over policy text unchanged) and cli help tests (identical help text).

- [ ] **Step 7: Grep gates.** Run:

```bash
grep -rn "_for_report\|PolicySummary" crates/pleiades-backend/src | grep -v FrameTreatmentSummary
grep -rn "_for_report\|PolicySummary" crates/pleiades-core/src | grep -v "^.*compatibility"
```

Expected: first grep prints nothing; second prints nothing outside `compatibility/` (and nothing matching `_for_report` at all).

- [ ] **Step 8: Commit.**

```bash
git add -A && git commit -m "feat(backend,validate)!: relocate backend policy prose layer into pleiades-validate posture module"
```

---

### Task 7: API-stability profile 0.2.2 → 0.3.0

**Files:**
- Modify: `crates/pleiades-core/src/api_stability.rs:13` (id), `:173` (summary), `:177` (stable-surface bullet), `:184-188` (experimental)
- Modify: `crates/pleiades-core/src/lib.rs` doctest (if the asserted phrase changes), `crates/pleiades-validate/src/tests/compatibility.rs` + any test pinning the api-stability text (`cargo test` will list them)

**Interfaces:**
- Produces: `CURRENT_API_STABILITY_PROFILE_ID = "pleiades-api-stability/0.3.0"`. Downstream pins updated. `ReleaseProfileIdentifiers::validate()` keeps passing (it reads the id accessors).

- [ ] **Step 1: Edit the profile.** In `api_stability.rs`:
  - Line 13: `pub const CURRENT_API_STABILITY_PROFILE_ID: &str = "pleiades-api-stability/0.3.0";`
  - Line 173 (summary): keep the existing text and append this sentence before the closing quote:

```text
 The 0.4.0 report-surface relocation removes the report-prose layer from the functional crates as a deliberate breaking change: pleiades-core and pleiades-backend no longer export *_summary_for_report helpers or policy-summary wrappers (report prose now lives in pleiades-validate), and ChartSnapshot's Display output no longer embeds the global time-scale/frame/apparentness policy lines.
```

  - Line 177 (stable-surface bullet): replace `"pleiades-core's ChartEngine, ChartRequest, ChartSnapshot, and compatibility-profile helpers are the stable façade used by consumers. …"` with:

```text
"pleiades-core's ChartEngine, ChartRequest, ChartSnapshot, and the posture types (CompatibilityProfile, ApiStabilityProfile, ReleaseProfileIdentifiers) with their summary methods are the stable façade used by consumers. ChartSnapshot's summary_line helper gives the chart façade a compact release-facing snapshot summary."
```

  - `experimental_surfaces` first entry: append `; relocated policy/report posture prose (posture module) is part of that operational tooling` before the closing quote.

- [ ] **Step 2: Run and fix pins.** Run: `cargo test --workspace 2>&1 | grep -E "FAILED|panicked" | head -20`. Fix every failure by updating the pinned expectation to the new id/text (expected: `api_stability.rs` unit test if any phrase assertion broke, `pleiades-core` lib doctest, `pleiades-validate` tests pinning `pleiades-api-stability/0.2.2` or the stable-surface phrase, release-bundle tests embedding the api-stability summary). Do NOT touch compatibility-profile pins (id stays 0.7.13).

- [ ] **Step 3: Verify.** Run: `cargo test --workspace`. Expected: PASS.

- [ ] **Step 4: Commit.**

```bash
git add -A && git commit -m "chore(release): api-stability profile 0.2.2 -> 0.3.0; declare report-surface relocation posture"
```

---

### Task 8: Bookkeeping, full CI, and slice close-out

**Files:**
- Modify: `CHANGELOG.md` (new unreleased entry), `PLAN.md` (status line addendum), `plan/status/02-next-slice-candidates.md` (note slice A done, B/C/D pending), `docs/superpowers/specs/2026-07-10-report-surface-relocation-design.md` (mark Slice A delivered if the spec tracks status)

**Interfaces:**
- Consumes: everything above merged into the branch.
- Produces: green `mise run ci`; documented migration notes for the 0.4.0 release entry.

- [ ] **Step 1: CHANGELOG.** Add under an `## Unreleased (0.4.0)` heading:

```markdown
- **Breaking (report-surface relocation, slice A):** `pleiades-core` no longer
  exports `*_summary_for_report` wrapper functions or re-exports
  `pleiades-backend` policy summaries — call the `CompatibilityProfile` /
  `ReleaseProfileIdentifiers` methods, or use `pleiades-validate` for report
  rendering. `pleiades-backend`'s policy-prose layer (constants, summary
  structs, report functions) moved to `pleiades-validate`;
  `FrameTreatmentSummary` and the request/metadata `validate_*` contract
  functions remain. `ChartSnapshot`'s `Display` no longer embeds the global
  time-scale/frame/apparentness policy lines. Dead report helpers deleted
  from `pleiades-time`, `pleiades-apparent`, `pleiades-ayanamsa`,
  `pleiades-houses`. API-stability profile 0.2.2 → 0.3.0.
```

- [ ] **Step 2: Update PLAN.md status line and `plan/status/02-next-slice-candidates.md`** with one sentence each: slice A of the report-surface relocation is done (link the spec); slices B (houses/ayanamsa/vsop87/elp/compression), C (pleiades-data), D (pleiades-jpl) remain.

- [ ] **Step 3: Full CI.** Run: `mise run ci`. Expected: PASS (fmt, clippy, test-full, docs, workspace-audit, package-check, release-smoke, claims-audit). Fix anything it surfaces (doc-link breakage from removed exports is the most likely: `cargo doc` runs with `-D warnings`).

- [ ] **Step 4: Final grep gates** (repeat, whole slice):

```bash
grep -rn "_for_report" crates/pleiades-core/src/lib.rs crates/pleiades-backend/src && echo "GATE FAILED" || echo "core+backend clean"
```

Expected: `core+backend clean`.

- [ ] **Step 5: Commit and hand off for review.**

```bash
git add -A && git commit -m "docs: report-surface relocation slice A close-out (CHANGELOG, plan status)"
```

Then use superpowers:finishing-a-development-branch to merge/PR `feat/report-surface-relocation-slice-a`.
