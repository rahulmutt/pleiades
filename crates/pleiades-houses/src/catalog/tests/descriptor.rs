//! Descriptor construction, summary-line formatting, and release-catalog
//! merge behavior for the house-system catalog.

use crate::catalog::*;

#[test]
fn baseline_catalog_includes_required_milestone_entries() {
    let names: Vec<_> = baseline_house_systems()
        .iter()
        .map(|entry| entry.canonical_name)
        .collect();

    for expected in [
        "Placidus",
        "Koch",
        "Porphyry",
        "Regiomontanus",
        "Campanus",
        "Equal",
        "Whole Sign",
        "Alcabitius",
        "Meridian",
        "Axial",
        "Topocentric",
        "Morinus",
    ] {
        assert!(names.contains(&expected), "missing {expected}");
    }
}

#[test]
fn descriptor_summary_line_includes_aliases_formula_family_latitude_and_notes() {
    let descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &["Alias One", "Alias Two"],
        "Summary note",
        true,
        None,
    );

    let expected =
        "Equal (aliases: Alias One, Alias Two) [formula: Equal] [latitude-sensitive] — Summary note";
    assert_eq!(descriptor.summary_line(), expected);
    assert_eq!(
        descriptor.validated_summary_line(),
        Ok(expected.to_string())
    );
    assert_eq!(descriptor.to_string(), expected);
}

#[test]
fn validated_summary_line_rejects_descriptor_drift() {
    let descriptor = HouseSystemDescriptor::new(
        pleiades_types::HouseSystem::Equal,
        "Equal",
        &["Alias One"],
        " Summary note",
        true,
        None,
    );

    assert_eq!(
        descriptor.validated_summary_line(),
        Err(HouseCatalogValidationError::DescriptorNotesNotNormalized { label: "Equal" })
    );

    let alias = HouseSystemCodeAlias {
        label: " T",
        system: pleiades_types::HouseSystem::Topocentric,
    };

    assert_eq!(
        alias.validated_summary_line(),
        Err(HouseSystemCodeAliasValidationError::LabelNotNormalized { label: " T" })
    );
}

#[test]
fn release_additions_are_merged_into_the_built_in_catalog() {
    let names: Vec<_> = built_in_house_systems()
        .iter()
        .map(|entry| entry.canonical_name)
        .collect();

    for expected in [
        "Equal (MC)",
        "Equal (1=Aries)",
        "Vehlow Equal",
        "Sripati",
        "Carter (poli-equatorial)",
        "Horizon/Azimuth",
        "APC",
        "Krusinski-Pisa-Goelzer",
        "Albategnius",
        "Pullen SD",
        "Pullen SR",
        "Sunshine",
        "Gauquelin sectors",
    ] {
        assert!(names.contains(&expected), "missing {expected}");
    }
}

#[test]
fn release_descriptor_aliases_do_not_repeat_canonical_labels() {
    assert!(built_in_house_systems()
        .iter()
        .all(|entry| { !entry.aliases.contains(&entry.canonical_name) }));
}

#[test]
fn release_grade_numeric_house_set_is_exactly_the_twenty_four_corpus_systems() {
    use pleiades_types::{CompatibilityClaimTier, HouseSystem};

    let release_grade: Vec<HouseSystem> = crate::built_in_house_systems()
        .iter()
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .map(|d| d.system.clone())
        .collect();

    let expected = [
        // Twelve baseline corpus systems.
        HouseSystem::Placidus,
        HouseSystem::Koch,
        HouseSystem::Porphyry,
        HouseSystem::Regiomontanus,
        HouseSystem::Campanus,
        HouseSystem::Equal,
        HouseSystem::WholeSign,
        HouseSystem::Alcabitius,
        HouseSystem::Meridian,
        HouseSystem::Axial,
        HouseSystem::Topocentric,
        HouseSystem::Morinus,
        // Ten standard systems promoted in Phase 6.
        HouseSystem::EqualMidheaven,
        HouseSystem::EqualAries,
        HouseSystem::Vehlow,
        HouseSystem::Sripati,
        HouseSystem::Carter,
        HouseSystem::Apc,
        HouseSystem::KrusinskiPisaGoelzer,
        HouseSystem::Sunshine,
        HouseSystem::PullenSd,
        HouseSystem::PullenSr,
        // Gauquelin promoted in Phase 6 Task 5a: its 36 sectors now match SE
        // via the Placidus semi-arc division (corpus-backed by the sectors slice).
        HouseSystem::Gauquelin,
        // Horizon promoted in Phase 6 Task 5b: the SE 'H' azimuth convention was
        // corrected (+180° post-rotation, single 90° quarter-turn, strict-sign
        // latitude branch); now matches SE within the GreatCircle ceiling.
        HouseSystem::Horizon,
    ];

    assert_eq!(release_grade.len(), expected.len());
    for sys in expected {
        assert!(release_grade.contains(&sys), "missing {sys:?}");
    }
}

#[test]
fn latitude_sensitive_systems_carry_a_latitude_bound() {
    for descriptor in built_in_house_systems() {
        if descriptor.latitude_sensitive {
            assert!(
                descriptor.max_abs_latitude_deg.is_some(),
                "latitude-sensitive system {:?} must declare max_abs_latitude_deg",
                descriptor.system
            );
            let bound = descriptor.max_abs_latitude_deg.unwrap();
            assert!(
                (60.0..=89.0).contains(&bound),
                "{:?} bound {bound} out of expected polar range",
                descriptor.system
            );
        } else {
            assert!(
                descriptor.max_abs_latitude_deg.is_none(),
                "non-latitude-sensitive system {:?} must not declare a bound",
                descriptor.system
            );
        }
    }
}
