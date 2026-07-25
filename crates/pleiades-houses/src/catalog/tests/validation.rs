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
