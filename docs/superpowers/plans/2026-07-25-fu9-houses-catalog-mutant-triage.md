# FU-9 houses PR 6 — Catalog + thresholds (crate-completing slice) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive the last 43 surviving mutants in `pleiades-houses` (15 in `catalog/mod.rs`, 28 in `catalog_name`) to zero-or-documented-equivalent, add the crate to the weekly mutants tier, and close the campaign with a measured next-crate roadmap.

**Architecture:** Three surfaces, three different mechanisms. `catalog_name`'s 28 survivors are closed by a **single-source refactor** — it duplicates the catalog descriptor table and is dead through the public API — preceded by a characterization test that both kills the 28 and proves the refactor is a no-op. `catalog/mod.rs`'s 15 fall to **exact-value assertions** (strings, counts, vectors) and **crafted-entry guard flips**, using the private entry-point functions that take a slice. Two whole-function `-> Ok(())` mutants are equivalent-mutant candidates (PR 5's VT-1 shape) and are probed before being documented.

**Tech Stack:** Rust (stable, per `mise.toml`), `cargo-mutants` 27.1.0 with `--test-tool nextest`, `cargo-nextest`, `mise` task runner.

## Global Constraints

Every task's requirements implicitly include this section.

- **No parity gate may be touched.** `validate-houses` / `validate-angles` corpora, tolerances, and gate code are unchanged. If a change seems to require touching one, stop and escalate.
- **The mutants tier stays report-only.** No mutation-score gate is introduced. Surviving mutants (cargo-mutants exit code 2) are an expected result.
- **Documented equivalents are left visible.** Never apply `#[mutants::skip]` — a function-level skip blanket-suppresses that function's other mutants. Each equivalent gets a written per-mutant reachability argument in a `*_equivalent_mutants_are_documented` test.
- **Equivalence is measured, never predicted.** Probe a candidate for reachability *before* writing the claim down. Three of this campaign's five landed PRs had an equivalence claim refuted on review.
- **Per-mutant rows only.** Never aggregate mutants into a single displacement row; enumerate mutant × input and state the true minimum.
- **Run `cargo fmt --all` before every commit.** Array literals in this campaign's tests and docs have broken the CI fmt gate in prior slices.
- **Authoritative mutants command** (per-file):
  `cargo mutants -p pleiades-houses --test-tool nextest --test-workspace=false --baseline run --file <path>`
- **`-F` filters are unanchored in this plan.** `-F 'in (…)$'` structurally excludes whole-function replacement mutants (their description reads `replace <fn> -> <T> with …` and never ends in `in <fn>`). PR 5 lost two mutants to this. Use bare substring filters, and confirm residuals with a whole-file or whole-crate run.
- **Test module conventions:** family/test files use `use crate::<module>::*;` plus `use super::support::*;`. Private items of `catalog` are visible to `catalog::tests::*` because those are descendant modules.
- **Verification command for the whole repo:** `mise run ci` (fmt + clippy `-D warnings` + docs + audit + deny + secrets + claims-audit + test + doctest + release-smoke).

## Measured baseline (2026-07-25, at `348c00ac6`)

| Surface | Tested | Missed | Caught | Unviable |
|---------|--------|--------|--------|----------|
| `catalog/mod.rs` + `thresholds.rs` | 101 (2 min) | **15** | 69 | 17 |
| `catalog_name` (`systems/mod.rs`) | 28 (41 s) | **28** | 0 | 0 |

`thresholds.rs` contributes **no** survivors — its single mutant is caught.

The 15 in `catalog/mod.rs`, by line:

| Line:col | Mutant | Task |
|----------|--------|------|
| 118:13 | `replace \|\| with &&` in `HouseSystemDescriptor::validate` | 5 |
| 160:50 | `replace \|\| with &&` in `HouseSystemDescriptor::validate` | 5 |
| 219:13 | `delete match arm HouseSystem::Custom(_)` in `formula_family` | 5 |
| 245:9 | `replace failure_mode_summary_line -> String with String::new()` | 4 |
| 245:9 | `replace failure_mode_summary_line -> String with "xyzzy".into()` | 4 |
| 428:9 | `replace <impl Display for HouseSystemCodeAliasValidationError>::fmt -> fmt::Result with Ok(Default::default())` | 4 |
| 462:24 | `replace += with *=` in `validate_house_system_code_alias_entries` | 4 |
| 475:5 | `replace validate_house_system_code_aliases -> Result<…> with Ok(())` | 6 |
| 594:24 | `replace += with *=` in `validate_house_catalog_entries` | 4 |
| 610:28 | `replace += with *=` in `validate_house_catalog_entries` | 4 |
| 637:5 | `replace validate_house_catalog -> Result<…> with Ok(())` | 6 |
| 645:49 | `replace \|\| with &&` in `collect_house_formula_families` | 5 |
| 678:5 | `replace latitude_sensitive_house_failure_modes -> Vec<String> with vec![]` | 4 |
| 678:5 | `… with vec![String::new()]` | 4 |
| 678:5 | `… with vec!["xyzzy".into()]` | 4 |

## Reference values (probed 2026-07-25, at `348c00ac6`)

These are the independently-observed runtime values the assertions pin. They were captured by a throwaway probe test that was reverted; **do not** re-derive them from the code under test.

```text
house_catalog_validation_summary().entry_count          = 25
house_catalog_validation_summary().baseline_entry_count = 12
house_catalog_validation_summary().release_entry_count  = 13
house_catalog_validation_summary().label_count          = 181   (25 canonical + 156 aliases)
house_system_code_aliases().len()                       = 22
latitude_sensitive_house_failure_modes().len()          = 8
house_formula_families().len()                          = 7
```

The eight latitude-sensitive failure-mode strings, in order:

```text
Placidus: Quadrant system; can fail or become unstable at extreme latitudes.
Koch: Quadrant system with documented high-latitude pathologies.
Horizon/Azimuth: Azimuthal house system that anchors house 1 due East and house 10 at the MC.
APC: APC (Ram school) houses with non-opposite quadrant pairs and polar adjustments.
Krusinski-Pisa-Goelzer: Great-circle house system centered on the ascendant and zenith; latitude-sensitive near the poles.
Topocentric: Topocentric (Polich-Page) house system with geodetic-to-geocentric latitude correction.
Sunshine: Sunshine house system based on the Sun's diurnal and nocturnal arcs; the 1st house is the Ascendant and the 10th house is the MC.
Gauquelin sectors: Thirty-six sectors used by the Gauquelin-sector family.
```

---

### Task 1: Structure moves (`catalog/tests/` split, `thresholds/tests.rs` relocation)

Two AGENTS.md structure moves, no test body edited. `catalog/tests.rs` is 1,140 lines and Tasks 4–6 add to it — split before adding, not after.

**Files:**
- Create: `crates/pleiades-houses/src/catalog/tests/mod.rs`
- Create: `crates/pleiades-houses/src/catalog/tests/descriptor.rs`
- Create: `crates/pleiades-houses/src/catalog/tests/validation.rs`
- Create: `crates/pleiades-houses/src/catalog/tests/aliases.rs`
- Create: `crates/pleiades-houses/src/catalog/tests/families.rs`
- Delete: `crates/pleiades-houses/src/catalog/tests.rs`
- Create: `crates/pleiades-houses/src/thresholds/tests.rs`
- Modify: `crates/pleiades-houses/src/thresholds.rs` (replace inline `mod tests { … }` with `mod tests;`)

**Interfaces:**
- Produces: the module paths `crate::catalog::tests::{descriptor, validation, aliases, families}` and `crate::thresholds::tests`. Tasks 4–6 add tests to `families.rs`, `validation.rs`, and `aliases.rs`.

- [ ] **Step 1: Capture the pre-move test inventory**

```bash
cd /workspace
cargo nextest list -p pleiades-houses --lib 2>/dev/null | sort > /tmp/nextest-before.txt
wc -l /tmp/nextest-before.txt
```

Expected: a sorted list of every test in the crate. Record the line count — Step 7 compares against it.

- [ ] **Step 2: Create the `catalog/tests/` module root**

Create `crates/pleiades-houses/src/catalog/tests/mod.rs`:

```rust
//! Unit tests for the `catalog` module, split by concern. Relocated from the
//! former monolithic `catalog/tests.rs` per AGENTS.md ("split a large file
//! before adding to it") and the `systems/tests/` precedent.

mod aliases;
mod descriptor;
mod families;
mod validation;
```

- [ ] **Step 3: Move the test bodies into the four files**

Every new file starts with this header (adjust the `//!` line per file):

```rust
//! <one-line description of this file's concern>

use crate::catalog::*;
```

Move each existing test **verbatim** — do not edit a single assertion — into:

| File | Tests moved from `catalog/tests.rs` |
|------|-------------------------------------|
| `descriptor.rs` | `baseline_catalog_includes_required_milestone_entries`, `descriptor_summary_line_includes_aliases_formula_family_latitude_and_notes`, `validated_summary_line_rejects_descriptor_drift`, `release_additions_are_merged_into_the_built_in_catalog`, `release_descriptor_aliases_do_not_repeat_canonical_labels`, `release_grade_numeric_house_set_is_exactly_the_twenty_four_corpus_systems`, `latitude_sensitive_systems_carry_a_latitude_bound` |
| `families.rs` | `formula_family_groups_the_built_in_house_systems_by_shape`, `built_in_house_systems_have_known_formula_families` |
| `validation.rs` | `validation_errors_use_stable_house_system_display_names`, `house_catalog_round_trips_all_built_ins_and_aliases`, `house_catalog_validation_summary_aggregates_catalog_fields`, `house_catalog_validation_rejects_duplicate_labels_and_round_trip_mismatches` |
| `aliases.rs` | `aliases_resolve_to_builtin_systems`, `additional_release_house_aliases_resolve_to_builtin_systems`, `swiss_ephemeris_house_system_code_aliases_are_unique_and_round_trip`, `house_system_code_alias_validation_rejects_duplicate_short_labels`, `house_system_code_alias_validate_rejects_normalization_and_round_trip_drift` |

That is 18 tests, matching the 18 in `catalog/tests.rs`. Then delete `crates/pleiades-houses/src/catalog/tests.rs`.

Note: `release_grade_numeric_house_set_is_exactly_the_twenty_four_corpus_systems` refers to `crate::built_in_house_systems()` (re-exported at the crate root) — keep that path exactly as written.

- [ ] **Step 4: Relocate the `thresholds.rs` tests**

Create `crates/pleiades-houses/src/thresholds/tests.rs` containing the **verbatim** body of the current inline `mod tests`:

```rust
//! Unit tests for the per-formula-family gate ceilings. Relocated from the
//! inline `#[cfg(test)] mod tests` in `thresholds.rs` per AGENTS.md.

use super::*;

#[test]
fn space_division_is_tighter_than_quadrant() {
    let equal = house_family_ceiling(HouseFormulaFamily::Equal);
    let quad = house_family_ceiling(HouseFormulaFamily::Quadrant);
    assert!(equal.cusp_arcsec <= quad.cusp_arcsec);
    assert!(equal.cusp_arcsec > 0.0);
}

#[test]
fn every_family_has_finite_positive_ceilings() {
    for family in [
        HouseFormulaFamily::Equal,
        HouseFormulaFamily::WholeSign,
        HouseFormulaFamily::Quadrant,
        HouseFormulaFamily::EquatorialProjection,
        HouseFormulaFamily::GreatCircle,
        HouseFormulaFamily::SolarArc,
        HouseFormulaFamily::Sector,
        HouseFormulaFamily::Custom,
        HouseFormulaFamily::Unknown,
    ] {
        let c = house_family_ceiling(family);
        assert!(c.cusp_arcsec.is_finite() && c.cusp_arcsec > 0.0);
        assert!(c.angle_arcsec.is_finite() && c.angle_arcsec > 0.0);
    }
}
```

Then in `crates/pleiades-houses/src/thresholds.rs`, replace lines 103–133 (the whole `#[cfg(test)] mod tests { … }` block) with:

```rust
#[cfg(test)]
mod tests;
```

- [ ] **Step 5: Build and run the suite**

Run: `cargo nextest run -p pleiades-houses --lib`
Expected: PASS, same test count as before the move.

- [ ] **Step 6: Format and lint**

```bash
cargo fmt --all
cargo clippy -p pleiades-houses --all-targets --all-features -- -D warnings
```

Expected: no diff from fmt, no clippy warnings.

- [ ] **Step 7: Verify the move is a no-op by inventory comparison**

```bash
cargo nextest list -p pleiades-houses --lib 2>/dev/null | sort > /tmp/nextest-after.txt
diff /tmp/nextest-before.txt /tmp/nextest-after.txt && echo "INVENTORY IDENTICAL"
```

Expected: `INVENTORY IDENTICAL`. If `diff` reports anything, a test was renamed, dropped, or duplicated — fix it before committing. This is the acceptance criterion for the move; do not eyeball it.

- [ ] **Step 8: Commit**

```bash
git add -A crates/pleiades-houses/src/catalog crates/pleiades-houses/src/thresholds.rs crates/pleiades-houses/src/thresholds
git commit -m "test(houses): split catalog tests by concern, relocate thresholds tests

Verified no-op move: cargo nextest list inventory identical before and after.
No test body edited. Per AGENTS.md (split a large file before adding to it)
and the systems/tests/ precedent; PR 6 adds to both files."
```

---

### Task 2: `catalog_name` cross-table characterization test (28 → 0)

Kills all 28 `catalog_name` survivors against `HEAD`, and doubles as the no-op proof for Task 3's refactor.

**Files:**
- Modify: `crates/pleiades-houses/src/systems/tests/dispatch.rs`

**Interfaces:**
- Consumes: `catalog_name(&HouseSystem) -> &'static str` (private, in `crate::systems`); `crate::catalog::{built_in_house_systems, descriptor}`.
- Produces: the test `catalog_name_agrees_with_the_descriptor_table`, which Task 3 deletes and replaces.

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-houses/src/systems/tests/dispatch.rs`:

```rust
/// `catalog_name` carries its own 25-arm name table, byte-for-byte duplicating
/// `HouseSystemDescriptor::canonical_name`. Nothing asserted they agree, so all
/// 28 of its mutants survived (26 arm deletes + 2 return-value replacements) —
/// the function is also dead through the public API, reachable only via the
/// dispatch `_` arm that exists because `HouseSystem` is `#[non_exhaustive]`.
///
/// This test is the cross-table pin AND the no-op proof for the single-source
/// refactor that replaces the match with a `catalog::descriptor` lookup.
#[test]
fn catalog_name_agrees_with_the_descriptor_table() {
    let entries = crate::catalog::built_in_house_systems();
    assert_eq!(entries.len(), 25, "catalog size changed; update this pin");

    for entry in entries {
        assert_eq!(
            catalog_name(&entry.system),
            entry.canonical_name,
            "catalog_name disagrees with the descriptor for {:?}",
            entry.system,
        );
    }

    // `Custom` has no catalog entry, so its name cannot come from the table.
    let custom = HouseSystem::Custom(CustomHouseSystem::new("Probe Houses"));
    assert!(
        crate::catalog::descriptor(&custom).is_none(),
        "no descriptor may claim a Custom system, or the refactor changes behavior",
    );
    assert_eq!(catalog_name(&custom), "Custom");
}
```

- [ ] **Step 2: Run the test to verify it passes against HEAD**

Run: `cargo nextest run -p pleiades-houses --lib catalog_name_agrees_with_the_descriptor_table`
Expected: PASS. (This is a characterization test — it documents existing behavior, so it passes immediately. Its *failure* mode is exercised by the mutants run in Step 3.)

- [ ] **Step 3: Verify it kills all 28 mutants**

```bash
cargo mutants -p pleiades-houses --test-tool nextest --test-workspace=false \
  --baseline run --file crates/pleiades-houses/src/systems/mod.rs -F 'catalog_name' \
  -o /tmp/mut-catname-after
```

Expected: `28 mutants tested … 0 missed, 28 caught`. Baseline before this task was `28 missed`.

- [ ] **Step 4: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-houses --all-targets --all-features -- -D warnings
git add crates/pleiades-houses/src/systems/tests/dispatch.rs
git commit -m "test(houses): FU-9 pin catalog_name against the descriptor table (28 -> 0)

Kills all 28 catalog_name survivors (26 match-arm deletes + 2 return-value
replacements) by asserting the function's private name table agrees with
HouseSystemDescriptor::canonical_name for all 25 built-ins, plus Custom.

Measured: cargo mutants -F 'catalog_name' 28 tested, 0 missed (was 28 missed)."
```

---

### Task 3: `catalog_name` single-source refactor

Behavior-preserving, own commit. Removes 26 match arms from existence and closes the drift seam. **Mutant-surface reduction is not a test kill** — the follow-up note must state both mechanisms separately.

**Files:**
- Modify: `crates/pleiades-houses/src/systems/mod.rs:1879-1907` (the `catalog_name` function)
- Modify: `crates/pleiades-houses/src/systems/tests/dispatch.rs`

**Interfaces:**
- Consumes: `crate::catalog::descriptor(&HouseSystem) -> Option<&'static HouseSystemDescriptor>` (public).
- Produces: `catalog_name` with an unchanged signature `fn catalog_name(system: &HouseSystem) -> &'static str`.

- [ ] **Step 1: Confirm the pre-refactor proof passes**

Run: `cargo nextest run -p pleiades-houses --lib catalog_name_agrees_with_the_descriptor_table`
Expected: PASS. This is the gate for the refactor — if it does not pass, stop.

- [ ] **Step 2: Replace the match body**

In `crates/pleiades-houses/src/systems/mod.rs`, replace the whole `catalog_name` function (currently a 27-arm match) with:

```rust
/// Returns the catalog's canonical name for a house system.
///
/// Single-sourced from `catalog::descriptor` rather than carrying a duplicate
/// name table: the two drifted independently before, with nothing asserting
/// they agreed. `Custom` has no catalog entry, so it keeps an explicit arm;
/// a future `#[non_exhaustive]` variant with no descriptor falls back to
/// `"Unspecified"`.
fn catalog_name(system: &HouseSystem) -> &'static str {
    match system {
        HouseSystem::Custom(_) => "Custom",
        other => crate::catalog::descriptor(other).map_or("Unspecified", |d| d.canonical_name),
    }
}
```

- [ ] **Step 3: Run the proof test to verify the refactor is a no-op**

Run: `cargo nextest run -p pleiades-houses --lib`
Expected: PASS, including `catalog_name_agrees_with_the_descriptor_table` — unchanged and unmodified. That is the no-op evidence.

- [ ] **Step 4: Replace the now-tautological test with the residual pin**

After the refactor, `catalog_name_agrees_with_the_descriptor_table` compares the descriptor table against itself. Delete it from `crates/pleiades-houses/src/systems/tests/dispatch.rs` and add in its place:

```rust
/// Pins the three behaviors that survive the single-source refactor: a known
/// system resolves through the catalog, `Custom` short-circuits before the
/// lookup, and a system with no descriptor falls back to `"Unspecified"`.
///
/// Before the refactor `catalog_name` carried a 25-arm duplicate of
/// `HouseSystemDescriptor::canonical_name`; those 26 arm-delete mutants no
/// longer exist (the arms are gone), they were not suppressed.
#[test]
fn catalog_name_resolves_through_the_catalog_with_custom_and_unknown_fallbacks() {
    // Known systems resolve to the catalog's canonical name. Spot-check the
    // three whose names are least guessable from the enum variant.
    assert_eq!(catalog_name(&HouseSystem::Carter), "Carter (poli-equatorial)");
    assert_eq!(catalog_name(&HouseSystem::Horizon), "Horizon/Azimuth");
    assert_eq!(
        catalog_name(&HouseSystem::KrusinskiPisaGoelzer),
        "Krusinski-Pisa-Goelzer"
    );
    assert_eq!(catalog_name(&HouseSystem::Gauquelin), "Gauquelin sectors");

    // Every built-in resolves to something non-empty and never to the fallback.
    for entry in crate::catalog::built_in_house_systems() {
        let name = catalog_name(&entry.system);
        assert!(!name.is_empty(), "empty name for {:?}", entry.system);
        assert_ne!(
            name, "Unspecified",
            "built-in {:?} fell through to the unknown fallback",
            entry.system,
        );
    }

    // `Custom` short-circuits: it has no descriptor, so without its own arm it
    // would take the "Unspecified" fallback.
    let custom = HouseSystem::Custom(CustomHouseSystem::new("Probe Houses"));
    assert!(crate::catalog::descriptor(&custom).is_none());
    assert_eq!(catalog_name(&custom), "Custom");
}
```

- [ ] **Step 5: Run the tests**

Run: `cargo nextest run -p pleiades-houses --lib`
Expected: PASS.

- [ ] **Step 6: Measure the post-refactor residual**

```bash
cargo mutants -p pleiades-houses --test-tool nextest --test-workspace=false \
  --baseline run --file crates/pleiades-houses/src/systems/mod.rs -F 'catalog_name' \
  -o /tmp/mut-catname-refactored
```

Expected: roughly `3 mutants tested … 0 missed` (the `Custom` arm delete, the `map_or` fallback, and the whole-function replacement). **Record the actual numbers** — they go in the follow-up note. If any mutant is missed, add an assertion to the test above rather than documenting an equivalent; all three of these are reachable from the test's own inputs.

- [ ] **Step 7: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-houses --all-targets --all-features -- -D warnings
git add crates/pleiades-houses/src/systems/mod.rs crates/pleiades-houses/src/systems/tests/dispatch.rs
git commit -m "refactor(houses): single-source catalog_name from the descriptor table

Behavior-preserving, proven no-op by catalog_name_agrees_with_the_descriptor_table
(added in the prior commit, passing unchanged across this one).

catalog_name duplicated HouseSystemDescriptor::canonical_name for all 25
built-ins with nothing asserting they agreed. Delegating to catalog::descriptor
closes that drift seam and removes 26 match-arm mutants from existence — a
mutant-surface reduction, NOT a test kill; the tests-only kill was the prior
commit. The now-tautological cross-table test is replaced by a residual pin
covering the catalog lookup, the Custom short-circuit, and the Unspecified
fallback."
```

---

### Task 4: `catalog/mod.rs` — renderings, vectors, counters (9 mutants)

Kills 245:9 ×2, 428:9, 678:5 ×3, 462:24, 594:24, 610:28.

**Files:**
- Modify: `crates/pleiades-houses/src/catalog/tests/descriptor.rs`
- Modify: `crates/pleiades-houses/src/catalog/tests/validation.rs`
- Modify: `crates/pleiades-houses/src/catalog/tests/aliases.rs`

**Interfaces:**
- Consumes: `latitude_sensitive_house_failure_modes() -> Vec<String>`, `house_catalog_validation_summary() -> HouseCatalogValidationSummary`, `HouseSystemDescriptor::failure_mode_summary_line(&self) -> String`, `HouseSystemCodeAliasValidationError` (public); `validate_house_system_code_alias_entries(&[HouseSystemCodeAlias]) -> Result<usize, HouseSystemCodeAliasValidationError>` and `validate_house_catalog_entries(&[HouseSystemDescriptor]) -> Result<usize, HouseCatalogValidationError>` (private, visible to descendant test modules).

- [ ] **Step 1: Write the failing test for the failure-mode vector and summary line**

Append to `crates/pleiades-houses/src/catalog/tests/descriptor.rs`:

```rust
/// Pins the release-facing latitude-sensitive failure-mode notes exactly.
///
/// Kills `latitude_sensitive_house_failure_modes -> vec![]` /
/// `vec![String::new()]` / `vec!["xyzzy".into()]` (678:5, ×3) and, because the
/// strings are produced by it, `failure_mode_summary_line -> String::new()` /
/// `"xyzzy".into()` (245:9, ×2). These are diagnostics a mutant could silently
/// empty with no other test noticing.
#[test]
fn latitude_sensitive_failure_modes_render_the_documented_notes() {
    let expected = [
        "Placidus: Quadrant system; can fail or become unstable at extreme latitudes.",
        "Koch: Quadrant system with documented high-latitude pathologies.",
        "Horizon/Azimuth: Azimuthal house system that anchors house 1 due East and house 10 at the MC.",
        "APC: APC (Ram school) houses with non-opposite quadrant pairs and polar adjustments.",
        "Krusinski-Pisa-Goelzer: Great-circle house system centered on the ascendant and zenith; latitude-sensitive near the poles.",
        "Topocentric: Topocentric (Polich-Page) house system with geodetic-to-geocentric latitude correction.",
        "Sunshine: Sunshine house system based on the Sun's diurnal and nocturnal arcs; the 1st house is the Ascendant and the 10th house is the MC.",
        "Gauquelin sectors: Thirty-six sectors used by the Gauquelin-sector family.",
    ];

    let actual = latitude_sensitive_house_failure_modes();
    assert_eq!(actual.len(), 8, "expected exactly eight latitude-sensitive systems");
    assert_eq!(actual, expected);
}

/// Pins `failure_mode_summary_line` directly, independent of the vector above,
/// so the `-> String::new()` / `"xyzzy".into()` mutants (245:9) stay dead even
/// if the aggregate ever changes shape. The rendering is `"{canonical}: {notes}"`.
#[test]
fn failure_mode_summary_line_renders_canonical_name_then_notes() {
    let placidus = descriptor(&pleiades_types::HouseSystem::Placidus)
        .expect("Placidus is a built-in");

    assert_eq!(
        placidus.failure_mode_summary_line(),
        "Placidus: Quadrant system; can fail or become unstable at extreme latitudes.",
    );
    assert_eq!(
        placidus.failure_mode_summary_line(),
        format!("{}: {}", placidus.canonical_name, placidus.notes),
    );
}
```

- [ ] **Step 2: Write the failing test for the alias-error `Display`**

Append to `crates/pleiades-houses/src/catalog/tests/aliases.rs`:

```rust
/// Pins every `HouseSystemCodeAliasValidationError` rendering exactly.
///
/// Kills `<impl Display for HouseSystemCodeAliasValidationError>::fmt ->
/// Ok(Default::default())` (428:9): that mutant writes nothing, producing an
/// empty error string. Nothing asserted these renderings before.
#[test]
fn alias_validation_errors_render_stable_diagnostics() {
    use pleiades_types::HouseSystem;

    assert_eq!(
        HouseSystemCodeAliasValidationError::EmptyAliasTable.to_string(),
        "the house-code alias table is empty",
    );
    assert_eq!(
        HouseSystemCodeAliasValidationError::LabelNotNormalized { label: " P " }.to_string(),
        "the house-code alias label ` P ` is blank, contains surrounding whitespace, \
         or contains line breaks",
    );
    assert_eq!(
        HouseSystemCodeAliasValidationError::DuplicateLabel { label: "P" }.to_string(),
        "the house-code alias table contains duplicate label `P`",
    );
    assert_eq!(
        HouseSystemCodeAliasValidationError::LabelDoesNotRoundTrip {
            label: "P",
            expected_system: HouseSystem::Koch,
        }
        .to_string(),
        format!(
            "the house-code alias label `P` does not round-trip to {}",
            HouseSystem::Koch
        ),
    );
}
```

- [ ] **Step 3: Write the failing test for the three label counters**

Append to `crates/pleiades-houses/src/catalog/tests/validation.rs`:

```rust
/// Pins the exact label counts the validators accumulate.
///
/// Kills the three `+= -> *=` mutants. A counter initialised to `0` is
/// invariant under `*=`, so only an exact-count assertion distinguishes them:
///   - 594:24 (canonical labels)  -> mutated total 156 instead of 181
///   - 610:28 (alias labels)      -> mutated total  25 instead of 181
///   - 462:24 (alias-table entries) -> mutated total 0 instead of 22
///
/// The pre-existing `house_catalog_validation_summary_aggregates_catalog_fields`
/// could not catch these: it compares `summary.entry_count` against
/// `built_in_house_systems().len()` — the same source on both sides — and never
/// asserts `label_count` at all.
#[test]
fn catalog_validators_count_every_label_they_check() {
    let summary = house_catalog_validation_summary();

    // 25 canonical names + 156 aliases, counted from the committed catalog.
    assert_eq!(summary.entry_count, 25);
    assert_eq!(summary.baseline_entry_count, 12);
    assert_eq!(summary.release_entry_count, 13);
    assert_eq!(
        summary.label_count, 181,
        "label_count must be 25 canonical + 156 aliases",
    );
    assert!(summary.validation_result.is_ok());

    // The alias-table validator returns its own count, which the public wrapper
    // discards — assert it at the private entry point.
    let alias_labels = validate_house_system_code_alias_entries(house_system_code_aliases())
        .expect("the built-in alias table validates");
    assert_eq!(alias_labels, 22);
    assert_eq!(house_system_code_aliases().len(), 22);

    // And the catalog validator's own return value, likewise discarded.
    let catalog_labels = validate_house_catalog_entries(built_in_house_systems())
        .expect("the built-in catalog validates");
    assert_eq!(catalog_labels, 181);
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo nextest run -p pleiades-houses --lib`
Expected: PASS. If `label_count` is not 181 or the alias count is not 22, the catalog has changed since 2026-07-25 — re-probe the values rather than adjusting the assertion to whatever the code returns.

- [ ] **Step 5: Verify the nine mutants are dead**

```bash
cargo mutants -p pleiades-houses --test-tool nextest --test-workspace=false \
  --baseline run --file crates/pleiades-houses/src/catalog/mod.rs -o /tmp/mut-catalog-t4
grep -E ':(245|428|462|594|610|678):' /tmp/mut-catalog-t4/missed.txt
```

Expected: `grep` finds nothing (exit 1). The run's total should be `6 missed` (down from 15) — the four in Task 5 plus the two in Task 6.

- [ ] **Step 6: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-houses --all-targets --all-features -- -D warnings
git add crates/pleiades-houses/src/catalog/tests
git commit -m "test(houses): FU-9 pin catalog renderings, failure modes, label counts

Kills 9 of catalog/mod.rs's 15 survivors: the latitude-sensitive failure-mode
vector (678:5 x3) and summary line (245:9 x2), the alias-error Display
(428:9), and the three label counters (462:24, 594:24, 610:28).

The counters needed exact totals (181 catalog labels, 22 alias entries): a
counter starting at 0 is invariant under the += -> *= mutation, and the
pre-existing summary test compared entry_count against its own source."
```

---

### Task 5: `catalog/mod.rs` — validation guards and the `Custom` family arm (4 mutants)

Kills 118:13, 160:50, 219:13, 645:49.

**Files:**
- Modify: `crates/pleiades-houses/src/catalog/tests/families.rs`
- Modify: `crates/pleiades-houses/src/catalog/tests/validation.rs`

**Interfaces:**
- Consumes: `HouseSystemDescriptor::new(system, canonical_name, aliases, notes, latitude_sensitive, max_abs_latitude_deg) -> Self` (public const); `HouseSystemDescriptor::validate(&self) -> Result<(), HouseCatalogValidationError>`; `HouseSystemDescriptor::formula_family(&self) -> HouseFormulaFamily`; `collect_house_formula_families(&[HouseSystemDescriptor]) -> Vec<HouseFormulaFamily>` (private); `pleiades_types::CustomHouseSystem::new(&str) -> CustomHouseSystem`.

- [ ] **Step 1: Write the failing test for the `Custom` family arm and the collector guard**

Append to `crates/pleiades-houses/src/catalog/tests/families.rs`:

```rust
use pleiades_types::{CustomHouseSystem, HouseSystem};

/// Builds a descriptor for a user-defined custom system. The built-in catalog
/// contains no `Custom` entry, so this is the only way to reach the
/// `HouseSystem::Custom(_)` arm of `formula_family` and the Custom half of the
/// collector's skip guard.
fn custom_descriptor() -> HouseSystemDescriptor {
    HouseSystemDescriptor::new(
        HouseSystem::Custom(CustomHouseSystem::new("Probe Houses")),
        "Probe Houses",
        &[],
        "A user-defined system used only by tests.",
        false,
        None,
    )
}

/// Kills `delete match arm HouseSystem::Custom(_)` in `formula_family`
/// (219:13). With the arm deleted, a custom system falls through to
/// `_ => HouseFormulaFamily::Unknown`, so only an assertion distinguishing
/// `Custom` from `Unknown` catches it. No test constructed a custom descriptor
/// before.
#[test]
fn custom_systems_report_the_custom_formula_family_not_unknown() {
    let custom = custom_descriptor();

    assert_eq!(custom.formula_family(), HouseFormulaFamily::Custom);
    assert_ne!(
        custom.formula_family(),
        HouseFormulaFamily::Unknown,
        "a custom system must not be indistinguishable from a future built-in",
    );
    assert_eq!(HouseFormulaFamily::Custom.to_string(), "Custom");
}

/// Kills `replace || with &&` in `collect_house_formula_families` (645:49).
///
/// The guard is `family == Custom || family == Unknown -> continue`. Under
/// `&&` a Custom entry satisfies only the left operand, so the mutant stops
/// skipping it and leaks `Custom` into the public family list. The built-in
/// catalog contains no Custom or Unknown entry, so this is unreachable through
/// `house_formula_families()` — it must be driven through the private
/// slice-taking collector.
#[test]
fn the_family_collector_skips_custom_entries() {
    let entries = [custom_descriptor()];
    assert_eq!(
        collect_house_formula_families(&entries),
        Vec::<HouseFormulaFamily>::new(),
        "a Custom entry must not appear in the collected family list",
    );

    // A mixed slice: the real entry survives, the Custom one is dropped.
    let equal = descriptor(&HouseSystem::Equal)
        .expect("Equal is a built-in")
        .clone();
    let mixed = [custom_descriptor(), equal];
    assert_eq!(
        collect_house_formula_families(&mixed),
        vec![HouseFormulaFamily::Equal],
    );
}
```

- [ ] **Step 2: Write the failing test for the two `validate` guards**

Append to `crates/pleiades-houses/src/catalog/tests/validation.rs`:

```rust
/// Kills the two `|| -> &&` mutants in `HouseSystemDescriptor::validate`.
///
/// 118:13 — `canonical_name.trim().is_empty() || has_surrounding_whitespace(..)
///   || contains_line_break(..)`, which parses as `(a || b) || c`. A padded but
///   non-empty, single-line name gives `a=false, b=true, c=false`: HEAD rejects
///   via `(F||T)`, the `&&` mutant computes `(F&&T)||F = false` and accepts.
///
/// 160:50 — `alias == &canonical_name || saw_canonical_case_variant`, inside the
///   alias-collision loop. Reaching it needs TWO case-variant aliases: the first
///   sets the flag, the second is rejected by the right operand alone. HEAD
///   returns `DescriptorLabelCollision`; the `&&` mutant needs both operands and
///   accepts. A single case-variant alias is legal by design, so the existing
///   drift tests never reached this branch.
#[test]
fn descriptor_validation_guards_reject_each_operand_alone() {
    use pleiades_types::HouseSystem;

    // 118:13 — surrounding whitespace alone must be rejected.
    let padded = HouseSystemDescriptor::new(
        HouseSystem::Equal,
        "  Equal  ",
        &[],
        "Padded canonical name.",
        false,
        None,
    );
    assert_eq!(
        padded.validate(),
        Err(HouseCatalogValidationError::DescriptorLabelNotNormalized {
            label: "  Equal  ",
            field: "canonical name",
        }),
        "a padded canonical name must be rejected by the whitespace operand alone",
    );

    // 160:50 — a SECOND case-variant alias must collide via the flag alone.
    let two_case_variants = HouseSystemDescriptor::new(
        HouseSystem::Equal,
        "Equal",
        &["EQUAL", "equal"],
        "Two case variants of the canonical name.",
        false,
        None,
    );
    assert_eq!(
        two_case_variants.validate(),
        Err(HouseCatalogValidationError::DescriptorLabelCollision {
            label: "equal",
            canonical_name: "Equal",
        }),
        "the second case-variant alias must collide via saw_canonical_case_variant alone",
    );

    // Exactly one case-variant alias remains legal — the behavior the guard's
    // left operand protects, and the reason this branch was never reached.
    let one_case_variant = HouseSystemDescriptor::new(
        HouseSystem::Equal,
        "Equal",
        &["EQUAL"],
        "One case variant of the canonical name.",
        false,
        None,
    );
    assert_eq!(one_case_variant.validate(), Ok(()));
}
```

- [ ] **Step 3: Run the tests**

Run: `cargo nextest run -p pleiades-houses --lib`
Expected: PASS.

If `two_case_variants.validate()` does not return `DescriptorLabelCollision`, trace which guard fired first — `validate` checks canonical normalization, alias normalization, notes, and formula family *before* the collision loop, so the fixture's other fields must all be valid.

- [ ] **Step 4: Verify the four mutants are dead**

```bash
cargo mutants -p pleiades-houses --test-tool nextest --test-workspace=false \
  --baseline run --file crates/pleiades-houses/src/catalog/mod.rs -o /tmp/mut-catalog-t5
cat /tmp/mut-catalog-t5/missed.txt
```

Expected: exactly two lines remain — `475:5` and `637:5` (Task 6). Lines 118, 160, 219 and 645 must be absent.

**If 160:50 survives**, it is an equivalence candidate: probe whether any reachable descriptor makes the left operand false and the right true. Do not document it as equivalent without that probe — record the finding and escalate.

- [ ] **Step 5: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-houses --all-targets --all-features -- -D warnings
git add crates/pleiades-houses/src/catalog/tests
git commit -m "test(houses): FU-9 pin catalog validation guards and the Custom family arm

Kills 118:13 and 160:50 (|| -> && in HouseSystemDescriptor::validate), 219:13
(delete match arm Custom(_) in formula_family) and 645:49 (|| -> && in
collect_house_formula_families).

All four needed inputs the built-in catalog cannot supply: a padded canonical
name, a second case-variant alias, and a constructed Custom descriptor driven
through the private slice-taking collector."
```

---

### Task 6: `catalog/mod.rs` — probe and document the two `Ok(())` wrappers

The last two survivors, `475:5` and `637:5`. Both are PR 5's VT-1 shape. **Probe before documenting.**

**Files:**
- Modify: `crates/pleiades-houses/src/catalog/tests/validation.rs`

**Interfaces:**
- Consumes: `validate_house_catalog() -> Result<(), HouseCatalogValidationError>` and `validate_house_system_code_aliases() -> Result<(), HouseSystemCodeAliasValidationError>` (public, no parameters).

- [ ] **Step 1: Probe reachability before writing any equivalence claim**

Both functions take no arguments and wrap a private entry-point over a `const` table:

```rust
pub fn validate_house_catalog() -> Result<(), HouseCatalogValidationError> {
    validate_house_catalog_entries(built_in_house_systems()).map(|_| ())
}
```

Answer these three questions and record the answers in the commit message:

1. Can any caller supply different entries? (Check for callers passing a slice: `rg 'validate_house_catalog\b|validate_house_system_code_aliases\b' crates/`)
2. Is `built_in_house_systems()` reachable in an invalid state — is it `const`, or can any code path mutate it?
3. Does the crate expose a feature flag or config that swaps the table?

If **all three** answers rule out an invalid table, the mutants are equivalent: HEAD and mutant both return `Ok(())` on every reachable input. If **any** answer admits an invalid table, the mutants are killable — write the test instead of the documentation, and record the finding.

- [ ] **Step 2: Write the documentation test (only if Step 1 confirms equivalence)**

Append to `crates/pleiades-houses/src/catalog/tests/validation.rs`:

```rust
/// FU-9 catalog residual: 2 surviving mutants, each an EQUIVALENT MUTANT left
/// visible (no `#[mutants::skip]`), with a per-mutant reachability argument.
///
/// --- catalog/mod.rs (2) ---
/// (CAT-1) 475:5 `replace validate_house_system_code_aliases -> Result<..> with
///   Ok(())`. The function takes no arguments and validates exactly one table,
///   the `const SWISS_EPHEMERIS_HOUSE_SYSTEM_CODE_ALIASES` slice returned by
///   `house_system_code_aliases()`. No caller can supply different entries, the
///   table cannot be mutated at runtime, and no feature flag swaps it. That
///   table validates (asserted below), so HEAD returns `Ok(())` on every
///   reachable input — byte-identical to the mutant on every path.
/// (CAT-2) 637:5 `replace validate_house_catalog -> Result<..> with Ok(())`.
///   The identical argument over `built_in_house_systems()`.
///
/// This is PR 5's VT-1 shape: a whole-function replacement on a guard whose
/// failure branch is unreachable because its only input is a valid constant.
/// The guards are not dead code — they defend against a future bad catalog
/// edit, and the private slice-taking entry points that do the real work are
/// killed by the Task 4/5 tests, which drive them with crafted invalid slices.
#[test]
fn catalog_equivalent_mutants_are_documented() {
    // The live path both operators share: the built-in tables validate.
    assert_eq!(validate_house_catalog(), Ok(()));
    assert_eq!(validate_house_system_code_aliases(), Ok(()));

    // The failure branches ARE reachable at the private entry points, which is
    // why only the no-argument public wrappers are equivalent.
    assert_eq!(
        validate_house_catalog_entries(&[]),
        Err(HouseCatalogValidationError::EmptyCatalog),
    );
    assert_eq!(
        validate_house_system_code_alias_entries(&[]),
        Err(HouseSystemCodeAliasValidationError::EmptyAliasTable),
    );
}
```

- [ ] **Step 3: Run the tests**

Run: `cargo nextest run -p pleiades-houses --lib`
Expected: PASS.

- [ ] **Step 4: Confirm the residual by a whole-file run**

```bash
cargo mutants -p pleiades-houses --test-tool nextest --test-workspace=false \
  --baseline run --file crates/pleiades-houses/src/catalog/mod.rs -o /tmp/mut-catalog-final
cat /tmp/mut-catalog-final/missed.txt
```

Expected: exactly `475:5` and `637:5`, i.e. `2 missed` (was 15). Whole-file, not `-F` — these are function-replacement mutants and a scoped filter would exclude them entirely.

- [ ] **Step 5: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-houses --all-targets --all-features -- -D warnings
git add crates/pleiades-houses/src/catalog/tests/validation.rs
git commit -m "test(houses): FU-9 document the two catalog Ok(()) equivalent mutants

catalog/mod.rs residual: 475:5 and 637:5, both whole-function replacements on
no-argument validators whose only input is a const table. Probed: no caller
supplies different entries, the tables cannot be mutated at runtime, no feature
flag swaps them — so HEAD and mutant return Ok(()) on every reachable path.
PR 5's VT-1 shape. Left visible, not #[mutants::skip]-suppressed.

Measured whole-file: 101 tested, 2 missed (was 15 missed)."
```

---

### Task 7: Withdraw the Sector GQ-1 equivalent classification

PR 3 documented `solve_gauquelin_sector` `1327:21 <` → `==` as equivalent on the premise that "the campaign does not pin error-message text". That premise is false. The existing comment already carries a `SUPERSEDED (PR 5) … the retraction lands in PR 6` note.

**Files:**
- Modify: `crates/pleiades-houses/src/systems/tests/sector.rs`

**Interfaces:**
- Consumes: `solve_gauquelin_sector(ramc_deg: f64, latitude_deg: f64, obliquity_deg: f64, fraction: f64, sign: f64) -> Result<Longitude, HouseError>` (private, in `crate::systems`).

**Geometry (derived during planning, verified against the crate):** the guard `gp.abs() < 1.0e-12` fires on the **first** Newton iteration when `gp` vanishes. At `sign = +1`, `q₀ = fraction × 90`, so `arg = 90°` and `sin(arg) = 1`, making `gp` linear in `tan(lat)`:

```text
gp × (180/π) = -(1/fraction) + tan(lat) · tan(obliquity) · cos(ramc + q₀)
```

Choosing `fraction = 8/9`, `ramc = 280°` puts `alpha = ramc + q₀ = 360° ≡ 0°`, so `cos(alpha) = 1` and the root is `lat = atan((9/8) / tan(23.4366°)) = 68.926_784_442_096_97°`. Measured `|gp| = 3.875e-18`, well inside the `1e-12` guard. Verified: the crate returns `Err(NumericalFailure, "gauquelin sector iteration encountered a zero derivative")` at exactly these inputs.

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-houses/src/systems/tests/sector.rs`:

```rust
/// Kills `solve_gauquelin_sector` 1327:21 `gp.abs() < 1e-12 -> ==`, withdrawing
/// the GQ-1 equivalent classification recorded by PR 3.
///
/// PR 3 argued the two operators were indistinguishable because both exit with
/// `Err(NumericalFailure)` and "the campaign does not pin error-message text".
/// That premise was wrong — the suite pins message text in several places, and
/// PR 5 killed the structurally identical `solve_placidian_cusp` 1741 `<` ->
/// `==` mutant exactly this way. The two exits carry DIFFERENT messages:
/// HEAD trips the zero-derivative guard; the `==` mutant falls through, divides
/// by a ~4e-18 derivative, and exits via the non-convergence branch.
///
/// Geometry: `gp` on the first iteration is linear in `tan(lat)` because
/// `sign = +1` fixes `arg = 90°`, so
///   `gp·(180/π) = -(1/fraction) + tan(lat)·tan(obl)·cos(ramc + fraction·90)`.
/// With `fraction = 8/9` and `ramc = 280°`, `alpha = 360° ≡ 0°` so `cos = 1`,
/// and the root is `lat = atan((9/8)/tan(23.4366°))`. Measured `|gp| = 3.875e-18`.
#[test]
fn solve_gauquelin_sector_fails_closed_on_a_zero_derivative() {
    let err = solve_gauquelin_sector(280.0, 68.926_784_442_096_97, 23.4366, 8.0 / 9.0, 1.0)
        .expect_err("the zero-derivative guard must fire at this geometry");

    assert_eq!(
        err.message, "gauquelin sector iteration encountered a zero derivative",
        "HEAD must exit via the zero-derivative guard, not the non-convergence branch",
    );
    assert_eq!(err.kind, HouseErrorKind::NumericalFailure);
}
```

`HouseError` exposes `message: String` and `kind: HouseErrorKind` as **public fields**, not accessors — `err.message`, not `err.message()`. This matches the existing precedent in `systems/tests/request.rs:118`.

- [ ] **Step 2: Run the test**

Run: `cargo nextest run -p pleiades-houses --lib solve_gauquelin_sector_fails_closed_on_a_zero_derivative`
Expected: PASS.

- [ ] **Step 3: Update the GQ-1 documentation block**

In `sector_equivalent_mutants_are_documented` in the same file, **delete** the whole `(GQ-1)` paragraph (the `1327:21 … -> ==` entry plus its `SUPERSEDED (PR 5)` note) and renumber the remaining two entries. Update the header count and the two references:

- The doc header currently reads `FU-9 Sector residual: 6 surviving mutants` — change `6` to `5`.
- `Measured by the authoritative scoped run: 233 tested, 6 missed, 227 caught` — change to `233 tested, 5 missed, 228 caught` **only after Step 4 confirms that figure**; if the measurement differs, use the measured numbers.
- Renumber `(GQ-2)` → `(GQ-1)` and `(GQ-3)` → `(GQ-2)`, and update the `--- solve_gauquelin_sector (3) ---` header to `(2)`.

- [ ] **Step 4: Measure the new Sector residual**

```bash
cargo mutants -p pleiades-houses --test-tool nextest --test-workspace=false \
  --baseline run --file crates/pleiades-houses/src/systems/mod.rs \
  -F 'solve_gauquelin_sector' -o /tmp/mut-gq
cat /tmp/mut-gq/missed.txt
```

Expected: `1327:21 <` → `==` is gone; the `<=` variants at 1327 and 1335 remain. Use the measured counts to finalize the Step 3 comment.

- [ ] **Step 5: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-houses --all-targets --all-features -- -D warnings
git add crates/pleiades-houses/src/systems/tests/sector.rs
git commit -m "test(houses): FU-9 withdraw Sector GQ-1, kill the zero-derivative mutant

PR 3 classified solve_gauquelin_sector 1327:21 '<' -> '==' equivalent on the
premise that the campaign does not pin error-message text. It does, and PR 5
killed the structurally identical solve_placidian_cusp mutant that way.

Killed by a geometry where gp vanishes on the first Newton iteration: gp is
linear in tan(lat) at sign=+1, so fraction=8/9, ramc=280 gives cos(alpha)=1 and
lat=atan((9/8)/tan(23.4366deg))=68.926784442096970. Measured |gp|=3.875e-18.
HEAD exits via the zero-derivative guard; the mutant via non-convergence, with
a different message.

Sector residual 6 -> 5; houses sub-total 35 -> 34; campaign-wide 44 -> 43."
```

---

### Task 8: Migrate the open-coded corpus closures onto `assert_corpus_cusps`

PR 5 added `assert_corpus_cusps` but deferred migrating the six pre-existing closures. Maintainability only — no mutant target.

**Files:**
- Modify: `crates/pleiades-houses/src/systems/tests/quadrant.rs`
- Modify: `crates/pleiades-houses/src/systems/tests/trivial.rs`

**Interfaces:**
- Consumes: `assert_corpus_cusps(label: &str, system: HouseSystem, latitude_deg: f64, expected: [f64; 12])` from `crates/pleiades-houses/src/systems/tests/support.rs:70`. It builds the request at JD 2451545.0 (TT), longitude 0°, and asserts all twelve cusps within 1.0 arcsec — which matches all six closures exactly.

- [ ] **Step 1: Migrate the six closures**

Each of these tests in `quadrant.rs` currently open-codes a `circ_diff_arcsec` closure, a `tolerance_arcsec` binding, a `HouseRequest`, a `calculate_houses` call, and a comparison loop. Replace each body with a call to `assert_corpus_cusps`, **keeping the existing `[f64; 12]` expected array literal and its doc comment verbatim**. All six use JD 2451545.0 and longitude 0°.

| Test | System | Latitude | Notes |
|------|--------|----------|-------|
| `morinus_cusps_match_swiss_ephemeris_corpus_within_120_arcsec` | `Morinus` | 40.0 | one call |
| `placidus_and_topocentric_cusps_match_swiss_ephemeris_corpus_within_120_arcsec` | `Placidus`, `Topocentric` | 40.0 | **two** calls, one per system, reusing `se_placidus` / `se_topocentric` |
| `koch_cusps_match_swiss_ephemeris_corpus_within_120_arcsec` | `Koch` | 40.0 | one call |
| `campanus_cusps_match_swiss_ephemeris_corpus_within_120_arcsec` | `Campanus` | 40.0 | one call |
| `alcabitius_cusps_match_swiss_ephemeris_corpus_within_120_arcsec` | `Alcabitius` | 40.0 | one call |
| `alcabitius_cusps_c2_lat55_match_swiss_ephemeris_corpus_within_1_arcsec` | `Alcabitius` | 55.0 | one call |

Worked example — `morinus_cusps_match_swiss_ephemeris_corpus_within_120_arcsec` becomes:

```rust
/// Swiss Ephemeris external-reference anchor for the Morinus house system.
///
/// Fixture c1_lat40: JD=2451545.0 (J2000.0), lat=40°N, lon=0°E.
/// SE reference cusps come straight from the houses-corpus
/// (`pleiades-validate/data/houses-corpus/cusps.csv`, system_code=Morinus).
/// Tolerance is 1 arcsec; actual residuals are ~0.02 arcsec after switching
/// to GAST + true obliquity.
#[test]
fn morinus_cusps_match_swiss_ephemeris_corpus_within_1_arcsec() {
    assert_corpus_cusps(
        "Morinus c1_lat40",
        HouseSystem::Morinus,
        40.0,
        [
            9.611_088,
            38.040_522,
            68.849_424,
            101.373_900,
            132.906_648,
            161.960_854,
            189.611_088,
            218.040_522,
            248.849_424,
            281.373_900,
            312.906_648,
            341.960_854,
        ],
    );
}
```

- [ ] **Step 2: Rename the misleadingly-named tests**

Five tests are named `*_within_120_arcsec` but assert a `1.0` arcsec tolerance. Rename each `_within_120_arcsec` suffix to `_within_1_arcsec`:

- `morinus_cusps_match_swiss_ephemeris_corpus_within_120_arcsec`
- `placidus_and_topocentric_cusps_match_swiss_ephemeris_corpus_within_120_arcsec`
- `koch_cusps_match_swiss_ephemeris_corpus_within_120_arcsec`
- `campanus_cusps_match_swiss_ephemeris_corpus_within_120_arcsec`
- `alcabitius_cusps_match_swiss_ephemeris_corpus_within_120_arcsec`

(`alcabitius_cusps_c2_lat55_match_swiss_ephemeris_corpus_within_1_arcsec` is already named correctly.)

- [ ] **Step 3: Check and fix the seventh misnamed test**

`crates/pleiades-houses/src/systems/tests/trivial.rs:106` has `equal_house_angles_match_swiss_ephemeris_corpus_within_120_arcsec`. It asserts **angles**, not cusps, so it is not an `assert_corpus_cusps` candidate. Read its tolerance:

```bash
sed -n '100,140p' crates/pleiades-houses/src/systems/tests/trivial.rs
```

If the asserted tolerance is 1.0 arcsec, rename it to `..._within_1_arcsec` too. If it genuinely asserts 120 arcsec, leave the name alone and add a one-line comment recording that it is correctly named, so a future reader does not re-flag it.

- [ ] **Step 4: Run the tests**

Run: `cargo nextest run -p pleiades-houses --lib`
Expected: PASS. The migrated tests must still pass — `assert_corpus_cusps` uses the same JD, longitude, and 1.0 arcsec tolerance as the closures it replaces.

- [ ] **Step 5: Verify the migration changed no coverage**

```bash
cargo nextest list -p pleiades-houses --lib 2>/dev/null | grep -c 'corpus'
```

Expected: the same number of corpus tests as before (renames change names, not counts). Confirm no test was dropped: the six migrated tests must all still be listed under their new names.

- [ ] **Step 6: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-houses --all-targets --all-features -- -D warnings
git add crates/pleiades-houses/src/systems/tests/quadrant.rs crates/pleiades-houses/src/systems/tests/trivial.rs
git commit -m "test(houses): migrate corpus closures onto assert_corpus_cusps, fix names

Replaces ~270 lines of near-identical arrange blocks in six SE-corpus anchors
(Morinus, Placidus+Topocentric, Koch, Campanus, Alcabitius x2) with calls to
the shared helper PR 5 added but deliberately did not migrate onto.

Also renames the five tests named *_within_120_arcsec that assert a 1.0 arcsec
tolerance. Expected-value literals and doc comments are unchanged."
```

---

### Task 9: Add `pleiades-houses` to the weekly mutants tier

**Files:**
- Modify: `mise.toml` (`[tasks.mutants]`, ~line 127)
- Modify: `.github/workflows/mutants.yml` (the `timeout-minutes` comment, ~line 28)

- [ ] **Step 1: Add the crate to the default mutants task**

In `mise.toml`, in `[tasks.mutants]`, add `-p pleiades-houses` to the package list:

```toml
run = """
cargo mutants \
  --test-tool nextest \
  --test-workspace=false \
  --baseline run \
  -p pleiades-types \
  -p pleiades-time \
  -p pleiades-apparent \
  -p pleiades-houses
"""
```

- [ ] **Step 2: Record the calibration figure in the workflow comment**

The `timeout-minutes: 90` comment in `.github/workflows/mutants.yml` says "the first scheduled run is the calibration point for revisiting this number". That run has happened. Replace the speculative paragraph with the measured figure, keeping `timeout-minutes: 90`:

```yaml
    # Calibrated 2026-07-25: the first scheduled run (2026-07-20) completed in
    # 16m05s on the 4-vCPU runner over the three baseline crates (~1451
    # mutants). Adding pleiades-houses (~1205 mutants after PR 6's catalog_name
    # refactor) roughly doubles the set, projecting ~30-35 min. 90 minutes is
    # retained as headroom for further crate additions; revisit it if a
    # scheduled run exceeds ~60 min.
    timeout-minutes: 90
```

- [ ] **Step 3: Verify the task parses and enumerates the right packages**

```bash
mise tasks info mutants
```

Expected: the task body lists all four `-p` flags. Do **not** run `mise run mutants` here — that is the ~35-minute weekly job; Task 11 runs the scoped whole-crate confirmation instead.

- [ ] **Step 4: Commit**

```bash
git add mise.toml .github/workflows/mutants.yml
git commit -m "ci(mutants): add pleiades-houses to the weekly tier

The crate reaches 0-or-documented-equivalent with PR 6, so the report-only
weekly tier now regression-checks it. Also replaces the mutants.yml timeout
guess with the measured calibration: the 2026-07-20 scheduled run took 16m05s
against a 90-minute budget; ~2x the mutant count projects ~30-35 min."
```

---

### Task 10: Measure roadmap baselines for the four smallest candidate crates

Makes the next-campaign ordering defensible rather than guessed. Bounded: 1,099 mutants, ≈20 min.

**Files:**
- Create: `docs/superpowers/specs/notes/2026-07-25-mutants-roadmap-baseline.md`

- [ ] **Step 1: Measure each crate**

```bash
for c in pleiades-apsides pleiades-backend pleiades-ayanamsa pleiades-fict; do
  echo "=== $c ==="
  cargo mutants -p "$c" --test-tool nextest --test-workspace=false --baseline run \
    -o "/tmp/mut-roadmap-$c" 2>&1 | tail -3
done
```

Expected: four summary lines of the form `N mutants tested in Xm: A missed, B caught, C unviable`. Record each verbatim.

If a crate's baseline **fails to build or its test suite fails** (cargo-mutants exit code 1/3/4), that is a measurement failure, not a result — record it as "could not measure" with the error, and do not substitute a guess.

- [ ] **Step 2: Record the results**

Create `docs/superpowers/specs/notes/2026-07-25-mutants-roadmap-baseline.md`:

```markdown
# FU-9 next-campaign roadmap baseline (2026-07-25)

Measured at the close of the `pleiades-houses` campaign (FU-9 PR 6), so the
next slice's ordering rests on data rather than crate size.

Command per crate:

```bash
cargo mutants -p <crate> --test-tool nextest --test-workspace=false --baseline run
```

## Measured survivor baselines

| Crate | Mutants | Missed | Caught | Unviable | Score |
|-------|---------|--------|--------|----------|-------|
| `pleiades-apsides` | | | | | |
| `pleiades-backend` | | | | | |
| `pleiades-ayanamsa` | | | | | |
| `pleiades-fict` | | | | | |

## Not measured — mutant counts only

Enumerated with `cargo mutants -p <crate> --list | wc -l` on 2026-07-25. These
are **sizing figures, not survivor counts**; no ordering may be inferred from
them.

| Crate | Mutants | Crate | Mutants |
|-------|---------|-------|---------|
| `pleiades-compression` | 607 | `pleiades-elp` | 1,521 |
| `pleiades-eclipse` | 913 | `pleiades-data` | 1,752 |
| `pleiades-core` | 962 | `pleiades-events` | 1,901 |
| `pleiades-vsop87` | 1,493 | `pleiades-jpl` | 3,662 |

~13,900 unmeasured mutants across twelve crates.
```

Fill the first table in from the Step 1 output — every cell, no blanks. Compute `Score` as `caught / (tested - unviable)`, matching how FU-9's baseline note reports it.

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/specs/notes/2026-07-25-mutants-roadmap-baseline.md
git commit -m "docs(plans): FU-9 next-campaign roadmap baseline

Measured survivor baselines for the four smallest unmeasured crates so the
next campaign's ordering is data-backed. The other eight carry mutant counts
only, explicitly labelled as sizing rather than survivor counts."
```

---

### Task 11: Whole-crate confirmation and the FU-9 campaign-closing note

**Files:**
- Modify: `docs/follow-ups.md` (FU-9 section)

- [ ] **Step 1: Run the whole-crate confirmation**

```bash
cargo mutants -p pleiades-houses --test-tool nextest --test-workspace=false \
  --baseline run -o /tmp/mut-houses-final 2>&1 | tail -5
cat /tmp/mut-houses-final/missed.txt
```

Whole-crate, **not** `-F`-scoped: PR 5's closing guidance is that a scoped filter structurally excludes whole-function replacement mutants, and this slice has two of those as its residual.

Expected: roughly `~1,205 mutants tested` (down from 1,231 — the refactor removed 26 arms). The missed list must decompose with **no unaccounted remainder**:

| Bucket | Expected |
|--------|----------|
| Prior-slice documented equivalents (Foundation 13 + Great-circle 8 + Sector 6 + Sunshine 5 + Quadrant 3 = 35, **minus GQ-1**) | 34 |
| PR 6 `catalog/mod.rs` residual (CAT-1, CAT-2) | 2 |
| PR 6 `catalog_name` residual | 0 |
| **Total** | **36** |

If the total differs, **do not adjust the note to match** — find the discrepancy. A survivor outside these buckets means a prior slice regressed or the refactor introduced a new one.

- [ ] **Step 2: Append the PR 6 Progress note to `docs/follow-ups.md`**

In the FU-9 section, after the Quadrant/projection entry and its record-keeping block, add a `**Progress (2026-07-25) — houses Catalog + thresholds:**` entry in the established format. It must state, using the **measured** numbers from Step 1:

- The measured baseline (`catalog/mod.rs` 15, `catalog_name` 28, `thresholds.rs` 0) and the authoritative commands.
- The two mechanisms, **kept separate**: 28 `catalog_name` mutants killed by test, then 26 removed from existence by the single-source refactor — a surface reduction, not a kill.
- That `catalog_name` was dead through the public API (dispatch `_` arm, `#[non_exhaustive]`) and duplicated `HouseSystemDescriptor::canonical_name` with nothing asserting agreement.
- The `catalog/mod.rs` bucket-by-bucket kills and the 2 documented equivalents (CAT-1 `475:5`, CAT-2 `637:5`) with their reachability arguments.
- The GQ-1 withdrawal and its geometry, and the resulting tally arithmetic: Sector 6→5, houses sub-total 35→34, campaign-wide 44→43, then **+2** for CAT-1/CAT-2 = **45**.
- The whole-crate confirmation figures and the no-remainder decomposition.
- `mise.toml` / `mutants.yml` changes; no parity gate touched; tier still report-only; `mise run ci` green.

- [ ] **Step 3: Add the campaign-closing restatement**

Replace the `**Remaining houses PRs:** catalog + thresholds …` line with a closing block stating:

- The `pleiades-houses` campaign is **complete** — six PRs, every file at 0-or-documented-equivalent, crate sub-total 34 (Foundation 13 + Great-circle 8 + Sector 5 + Sunshine 5 + Quadrant 3) plus PR 6's 2 = **36 in-crate**; campaign-wide running tally **45**.
- FU-9 stays **open as a standing posture entry** — the same disposition the three-crate baseline took. There is no remaining slice for either the baseline or the houses campaign.
- The roadmap, linking `docs/superpowers/specs/notes/2026-07-25-mutants-roadmap-baseline.md`: four measured candidate baselines, eight crates sized only, ~13,900 unmeasured mutants across twelve crates. State plainly that the houses campaign covered one crate of thirteen remaining.

- [ ] **Step 4: Run the full CI gate**

```bash
cargo fmt --all
mise run ci
```

Expected: green. This is the acceptance gate for the whole PR.

- [ ] **Step 5: Commit**

```bash
git add docs/follow-ups.md
git commit -m "docs(plans): FU-9 houses Catalog mutant triage + campaign close

Sixth and final PR of the pleiades-houses expansion campaign. catalog/mod.rs
15 -> 2 documented equivalents, catalog_name 28 -> 0 (killed by test, then 26
removed by the single-source refactor), thresholds.rs tests relocated.

Withdraws Sector GQ-1. Adds -p pleiades-houses to the weekly tier. Records a
measured next-campaign roadmap. FU-9 stays open as standing posture with no
remaining slice."
```

---

## Plan self-review

**Spec coverage.** Every section of the PR 6 addendum maps to a task: measured baseline → the baseline tables above; `catalog_name` refactor (a)/(b)/(c) → Tasks 2 and 3; the 15 `catalog/mod.rs` buckets → Tasks 4, 5, 6; structure changes → Tasks 1 and 8; Sector GQ-1 → Task 7; weekly-tier expansion → Task 9; FU-9 disposition and roadmap → Tasks 10 and 11; acceptance criteria → Task 11 Steps 1 and 4.

**Known open items, deliberately not pre-decided.**

- Task 6 documents two equivalents *only if* its Step 1 probe confirms them. If the probe finds a seam, the task writes a test instead — and Task 11's expected total drops from 36 to 34.
- Task 7 Step 1 assumes `HouseError` exposes `message()`. The step says to match `quadrant.rs`'s existing precedent if the accessor differs.
- Task 8 Step 3 leaves the seventh test's rename conditional on its actual tolerance, which the step reads rather than assumes.

**Type consistency.** `assert_corpus_cusps(&str, HouseSystem, f64, [f64; 12])` is used with that signature in Task 8. `catalog_name(&HouseSystem) -> &'static str` keeps its signature across Tasks 2 and 3. `collect_house_formula_families(&[HouseSystemDescriptor]) -> Vec<HouseFormulaFamily>`, `validate_house_catalog_entries(&[HouseSystemDescriptor]) -> Result<usize, HouseCatalogValidationError>` and `validate_house_system_code_alias_entries(&[HouseSystemCodeAlias]) -> Result<usize, HouseSystemCodeAliasValidationError>` are used consistently in Tasks 4, 5 and 6. `HouseSystemDescriptor::new` is called with its six-argument form throughout.
