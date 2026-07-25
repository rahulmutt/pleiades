//! Catalog-wide validation: error display, round-tripping, and rejection of
//! malformed descriptors.

use crate::catalog::*;

#[test]
fn validation_errors_use_stable_house_system_display_names() {
    let error = HouseCatalogValidationError::LabelDoesNotRoundTrip {
        label: "Equal (MC) table of houses",
        expected_system: pleiades_types::HouseSystem::EqualMidheaven,
    };

    assert_eq!(
        error.to_string(),
        "the house catalog label `Equal (MC) table of houses` does not round-trip to Equal (MC)"
    );
}

#[test]
fn house_catalog_round_trips_all_built_ins_and_aliases() {
    use std::collections::HashSet;

    let built_in = built_in_house_systems();
    let mut unique_names = HashSet::new();

    assert_eq!(
        built_in.len(),
        baseline_house_systems().len() + release_house_systems().len()
    );

    for entry in baseline_house_systems()
        .iter()
        .chain(release_house_systems().iter())
    {
        assert!(
            unique_names.insert(entry.canonical_name),
            "duplicate canonical house-system name {}",
            entry.canonical_name
        );
        assert_eq!(
            descriptor(&entry.system).map(|d| d.canonical_name),
            Some(entry.canonical_name)
        );
        assert_eq!(
            resolve_house_system(entry.canonical_name),
            Some(entry.system.clone())
        );
        for alias in entry.aliases {
            assert_eq!(resolve_house_system(alias), Some(entry.system.clone()));
        }
    }

    for entry in built_in {
        assert!(unique_names.contains(entry.canonical_name));
    }
}

#[test]
fn house_catalog_validation_summary_aggregates_catalog_fields() {
    let summary = house_catalog_validation_summary();

    assert_eq!(summary.entry_count, built_in_house_systems().len());
    assert_eq!(summary.baseline_entry_count, baseline_house_systems().len());
    assert_eq!(summary.release_entry_count, release_house_systems().len());
    assert_eq!(
        house_formula_families(),
        vec![
            HouseFormulaFamily::Equal,
            HouseFormulaFamily::WholeSign,
            HouseFormulaFamily::Quadrant,
            HouseFormulaFamily::EquatorialProjection,
            HouseFormulaFamily::GreatCircle,
            HouseFormulaFamily::SolarArc,
            HouseFormulaFamily::Sector,
        ]
    );
    assert!(summary.validation_result.is_ok());
}

#[test]
fn house_catalog_validation_rejects_duplicate_labels_and_round_trip_mismatches() {
    let duplicate_alias_entries = [HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &["Wang", "wang"],
        "notes",
        false,
        None,
    )];

    assert!(matches!(
        validate_house_catalog_entries(&duplicate_alias_entries),
        Err(HouseCatalogValidationError::DescriptorLabelCollision {
            label: "wang",
            canonical_name: "Equal"
        })
    ));

    let mismatched_entry = [HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Not Equal",
        &[],
        "notes",
        false,
        None,
    )];

    assert!(matches!(
        validate_house_catalog_entries(&mismatched_entry),
        Err(HouseCatalogValidationError::LabelDoesNotRoundTrip {
            label: "Not Equal",
            expected_system: pleiades_types::HouseSystem::Equal,
        })
    ));

    let blank_name_descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "   ",
        &[],
        "notes",
        false,
        None,
    );
    assert!(matches!(
        blank_name_descriptor.validate(),
        Err(HouseCatalogValidationError::DescriptorLabelNotNormalized {
            label: "   ",
            field: "canonical name"
        })
    ));

    let padded_alias_descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &[" Alias "],
        "notes",
        false,
        None,
    );
    assert!(matches!(
        padded_alias_descriptor.validate(),
        Err(HouseCatalogValidationError::DescriptorLabelNotNormalized {
            label: " Alias ",
            field: "alias"
        })
    ));

    let blank_notes_descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &[],
        "   ",
        false,
        None,
    );
    assert!(matches!(
        blank_notes_descriptor.validate(),
        Err(HouseCatalogValidationError::DescriptorNotesNotNormalized { label: "Equal" })
    ));

    let line_break_name_descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equ\nal",
        &[],
        "notes",
        false,
        None,
    );
    assert!(matches!(
        line_break_name_descriptor.validate(),
        Err(HouseCatalogValidationError::DescriptorLabelNotNormalized {
            label: "Equ\nal",
            field: "canonical name"
        })
    ));

    let line_break_alias_descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &["Al\nial"],
        "notes",
        false,
        None,
    );
    assert!(matches!(
        line_break_alias_descriptor.validate(),
        Err(HouseCatalogValidationError::DescriptorLabelNotNormalized {
            label: "Al\nial",
            field: "alias"
        })
    ));

    let line_break_notes_descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &[],
        "notes\nline two",
        false,
        None,
    );
    assert!(matches!(
        line_break_notes_descriptor.validate(),
        Err(HouseCatalogValidationError::DescriptorNotesNotNormalized { label: "Equal" })
    ));

    let duplicate_alias_descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &["Wang", "wang"],
        "notes",
        false,
        None,
    );
    assert!(matches!(
        duplicate_alias_descriptor.validate(),
        Err(HouseCatalogValidationError::DescriptorLabelCollision {
            label: "wang",
            canonical_name: "Equal"
        })
    ));

    let blank_notes_entry = [blank_notes_descriptor];
    assert!(matches!(
        validate_house_catalog_entries(&blank_notes_entry),
        Err(HouseCatalogValidationError::DescriptorNotesNotNormalized { label: "Equal" })
    ));
}

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

/// FU-9 catalog residual: 2 surviving mutants, each an EQUIVALENT MUTANT left
/// visible (no `#[mutants::skip]`), with a per-mutant reachability argument.
///
/// Both are whole-function replacements on a *no-argument* public validator that
/// wraps a private slice-taking entry point over one immutable built-in table.
/// The probe that established equivalence checked, for each: who can supply the
/// entries, whether the table can be reached in an invalid state, and whether
/// any build configuration swaps it.
///
/// --- catalog/mod.rs (2) ---
/// (CAT-1) 475:5 `replace validate_house_system_code_aliases -> Result<(),
///   HouseSystemCodeAliasValidationError> with Ok(())`.
///   - Entries: the function takes no arguments; it validates exactly
///     `house_system_code_aliases()`, a `pub const fn` returning the private
///     `const SWISS_EPHEMERIS_HOUSE_SYSTEM_CODE_ALIASES: &[HouseSystemCodeAlias]`.
///     No caller can substitute entries, because the slice-taking entry point
///     `validate_house_system_code_alias_entries` is module-private and is not
///     in `lib.rs`'s re-export list, so no downstream crate can reach it.
///   - Invalid state: the table is a private `const`; `HouseSystemCodeAlias`
///     holds only `&'static str` plus a `HouseSystem` (no interior mutability),
///     and the crate is `#![forbid(unsafe_code)]`, so no code path can mutate it.
///   - Configuration: `pleiades-houses/Cargo.toml` declares no `[features]` at
///     all, the crate has no non-`cfg(test)` `#[cfg]` attribute, and there is no
///     `build.rs`/`include!` that could generate a different table.
///   - Conclusion: that table validates (asserted below), so HEAD returns
///     `Ok(())` on every reachable input — indistinguishable from the mutant on
///     every path.
///
/// (CAT-2) 637:5 `replace validate_house_catalog -> Result<(),
///   HouseCatalogValidationError> with Ok(())`.
///   - Entries: likewise no arguments; it validates exactly
///     `built_in_house_systems()`, a `pub const fn` returning
///     `&BUILT_IN_HOUSE_SYSTEMS` — a private *immutable* `static
///     [HouseSystemDescriptor; 25]` (a `static`, not a `const`, but not `static
///     mut`). `validate_house_catalog_entries` is likewise module-private and
///     unexported. The one workspace call site that *does* hold a descriptor
///     slice, `pleiades_validate::compatibility::verify_house_system_aliases`,
///     deliberately calls this no-argument wrapper and then checks its own
///     `entries` separately, so its crafted-invalid-descriptor tests assert
///     errors from validate's own loop; nothing anywhere asserts the
///     `"house catalog validation failed: …"` / `"house-code alias validation
///     failed: …"` messages that HEAD's `Err` branch would produce.
///   - Invalid state: `HouseSystemDescriptor` holds `&'static str`, `bool`,
///     `Option<f64>`, `HouseSystem`, and `CompatibilityClaimTier` — no interior
///     mutability anywhere — and `#![forbid(unsafe_code)]` rules out mutating an
///     immutable `static`.
///   - Configuration: same as CAT-1; no feature or codegen seam exists.
///   - Conclusion: HEAD returns `Ok(())` on every reachable input here too.
///
/// This is PR 5's VT-1 shape: a whole-function replacement on a guard whose
/// failure branch is unreachable because its only input is a valid built-in
/// constant. The guards are not dead code — they defend against a future bad
/// catalog edit, and the private slice-taking entry points that do the real work
/// are killed by the Task 4/5 tests, which drive them with crafted invalid
/// slices. Killing these two would require adding a test-only injection seam to
/// production code (a parameter, or a `#[cfg(test)]` table override) purely to
/// observe a guard over a compile-time constant; that is out of scope here and
/// would not increase the behavior under test.
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
