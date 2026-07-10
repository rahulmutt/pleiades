use super::*;
use pleiades_ayanamsa::AyanamsaDescriptor;
use pleiades_houses::{HouseCatalogValidationError, HouseSystemDescriptor};
use pleiades_types::{Ayanamsa, HouseSystem};

#[test]
fn profile_includes_baseline_and_release_catalogs() {
    let profile = current_compatibility_profile();
    assert!(profile
        .house_systems
        .iter()
        .any(|entry| entry.canonical_name == "Equal (MC)"));
    assert!(profile
        .baseline_house_systems
        .iter()
        .any(|entry| entry.canonical_name == "Placidus"));
    assert!(profile
        .release_house_systems
        .iter()
        .any(|entry| entry.canonical_name == "Sripati"));
    assert!(profile
        .release_house_systems
        .iter()
        .any(|entry| entry.canonical_name == "Carter (poli-equatorial)"));
    assert!(profile
        .release_house_systems
        .iter()
        .any(|entry| entry.canonical_name == "Horizon/Azimuth"));
    assert!(profile
        .release_house_systems
        .iter()
        .any(|entry| entry.canonical_name == "APC"));
    assert!(profile
        .release_house_systems
        .iter()
        .any(|entry| entry.canonical_name == "Krusinski-Pisa-Goelzer"));
    assert!(profile
        .release_house_systems
        .iter()
        .any(|entry| entry.canonical_name == "Albategnius"));
    assert!(profile
        .release_house_systems
        .iter()
        .any(|entry| entry.canonical_name == "Pullen SD"));
    assert!(profile
        .release_house_systems
        .iter()
        .any(|entry| entry.canonical_name == "Pullen SR"));
    assert!(profile
        .release_house_systems
        .iter()
        .any(|entry| entry.canonical_name == "Sunshine"));
    assert!(profile
        .release_house_systems
        .iter()
        .any(|entry| entry.canonical_name == "Gauquelin sectors"));
    assert!(profile
        .release_house_systems
        .iter()
        .any(|entry| entry.canonical_name == "Equal (1=Aries)"));
    assert!(profile
        .baseline_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Lahiri"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "J2000"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "DeLuce"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Yukteshwar"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "PVR Pushya-paksha"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Sheoran"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "True Revati"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "True Mula"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Suryasiddhanta (Revati)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Suryasiddhanta (Citra)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Lahiri (ICRC)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Sassanian"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Hipparchus"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Babylonian (Kugler 1)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Babylonian (Kugler 2)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Babylonian (Aldebaran)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Babylonian (House)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Babylonian (Sissy)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Babylonian (True Geoc)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Babylonian (True Topc)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Babylonian (True Obs)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Babylonian (House Obs)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "True Pushya"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Djwhal Khul"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "JN Bhasin"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Suryasiddhanta (Mean Sun)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Aryabhata (Mean Sun)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Babylonian (Britton)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Aryabhata (522 CE)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Lahiri (VP285)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Krishnamurti (VP291)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "True Sheoran"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Galactic Center"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Galactic Center (Rgilbrand)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Galactic Center (Mardyks)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Galactic Center (Mula/Wilhelm)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Dhruva Galactic Center (Middle Mula)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Galactic Center (Cochrane)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Galactic Equator"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Galactic Equator (IAU 1958)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Galactic Equator (True)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Galactic Equator (Mula)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Galactic Equator (Fiorenza)"));
    assert!(profile
        .release_ayanamsas
        .iter()
        .any(|entry| entry.canonical_name == "Valens Moon"));
    assert_eq!(
        profile.house_formula_family_names(),
        vec![
            "Equal".to_string(),
            "Equatorial projection".to_string(),
            "Great-circle".to_string(),
            "Quadrant".to_string(),
            "Sector".to_string(),
            "Solar arc".to_string(),
            "Whole Sign".to_string(),
        ]
    );
    assert_eq!(
        profile.house_systems,
        pleiades_houses::built_in_house_systems()
    );
    assert_eq!(
        profile.baseline_house_systems,
        pleiades_houses::baseline_house_systems()
    );
    assert_eq!(
        profile.release_house_systems,
        pleiades_houses::release_house_systems()
    );
    assert_eq!(profile.ayanamsas, pleiades_ayanamsa::built_in_ayanamsas());
    assert_eq!(
        profile.baseline_ayanamsas,
        pleiades_ayanamsa::baseline_ayanamsas()
    );
    assert_eq!(
        profile.release_ayanamsas,
        pleiades_ayanamsa::release_ayanamsas()
    );
    assert!(profile
        .target_house_scope
        .iter()
        .any(|line| line.contains("Swiss-Ephemeris-class house-system catalog")));
    assert!(profile
        .target_ayanamsa_scope
        .iter()
        .any(|line| line.contains("Swiss-Ephemeris-class ayanamsa catalog")));
    assert!(profile
        .release_notes
        .iter()
        .any(|note| note.contains("Krusinski-Pisa-Goelzer")));
    assert!(profile
        .release_notes
        .iter()
        .any(|note| note.contains("Treindl Sunshine")));
    assert!(profile
        .release_notes
        .iter()
        .any(|note| note.contains("Babylonian/Aldebaran = 15 Tau")));
    assert!(profile
        .release_notes
        .iter()
        .any(|note| note.contains("Krishnamurti-Senthilathiban")));
    assert!(profile
        .release_notes
        .iter()
        .any(|note| note.contains("B. V. Raman")));
    assert!(profile
        .release_notes
        .iter()
        .any(|note| note.contains("Raman Ayanamsha")));
    assert!(profile
        .release_notes
        .iter()
        .any(|note| note.contains("Raman ayanamsa")));
    assert!(profile
        .release_notes
        .iter()
        .any(|note| note.contains("Whole sign houses, 1. house = Aries")));
    assert!(profile
        .validation_reference_points
        .iter()
        .any(|point| point.contains("validation corpus")));
    assert!(profile
        .validation_reference_points
        .iter()
        .any(|point| point.contains("house formulas")));
    assert!(profile
        .known_gaps
        .iter()
        .all(|gap| !gap.contains("validation corpus")));
    assert!(profile
        .known_gaps
        .iter()
        .all(|gap| !gap.contains("house formulas")));
    assert!(profile
        .custom_definition_labels
        .contains(&"Babylonian (House)"));
    assert!(profile
        .custom_definition_labels
        .contains(&"Babylonian (House Obs)"));
    assert!(profile
        .known_gaps
        .iter()
        .all(|gap| !gap.contains("Babylonian (House)")));
    assert!(profile
        .known_gaps
        .iter()
        .all(|gap| !gap.contains("House Obs")));
}

#[test]
fn compatibility_profile_validate_accepts_the_current_profile() {
    current_compatibility_profile()
        .validate()
        .expect("current compatibility profile should validate");
}

#[test]
fn compatibility_profile_target_scope_summary_helpers_render_and_validate() {
    let profile = current_compatibility_profile();

    assert_eq!(
        profile.target_house_scope_summary_line(),
        profile.target_house_scope.join("; ")
    );
    assert_eq!(
        profile
            .validated_target_house_scope_summary_line()
            .expect("target house scope should validate"),
        profile.target_house_scope.join("; ")
    );
    assert_eq!(
        target_house_scope_summary_for_report(),
        profile.target_house_scope_summary_line()
    );
    assert_eq!(
        validated_target_house_scope_summary_for_report(),
        Ok(profile.target_house_scope.join("; "))
    );
    assert_eq!(
        profile.target_ayanamsa_scope_summary_line(),
        profile.target_ayanamsa_scope.join("; ")
    );
    assert_eq!(
        profile
            .validated_target_ayanamsa_scope_summary_line()
            .expect("target ayanamsa scope should validate"),
        profile.target_ayanamsa_scope.join("; ")
    );
    assert_eq!(
        target_ayanamsa_scope_summary_for_report(),
        profile.target_ayanamsa_scope_summary_line()
    );
    assert_eq!(
        validated_target_ayanamsa_scope_summary_for_report(),
        Ok(profile.target_ayanamsa_scope.join("; "))
    );
}

#[test]
fn compatibility_profile_validate_rejects_whitespace_padded_scope_entries() {
    let mut profile = current_compatibility_profile();
    profile.target_house_scope = &["Target house scope: example "];

    let error = profile
        .validate()
        .expect_err("whitespace-padded scope entry should fail validation");

    assert!(matches!(
        error,
        CompatibilityProfileValidationError::WhitespaceTextSectionEntry {
            section_label: "target-house-scope",
            entry: "Target house scope: example "
        }
    ));
}

#[test]
fn compatibility_profile_validate_rejects_case_insensitive_duplicate_within_section() {
    let mut profile = current_compatibility_profile();
    profile.release_notes = &["Release note entry", "release note entry"];

    let error = profile
        .validate()
        .expect_err("case-insensitive duplicates within a section should fail validation");

    assert!(matches!(
        error,
        CompatibilityProfileValidationError::DuplicateTextSectionEntry {
            section_label: "release-note",
            entry: "release note entry"
        }
    ));
}

#[test]
fn compatibility_profile_validate_rejects_case_insensitive_duplicates_across_sections() {
    let mut profile = current_compatibility_profile();
    profile.validation_reference_points = &["Release note entry"];
    profile.known_gaps = &["release note entry"];

    let error = profile
        .validate()
        .expect_err("case-insensitive duplicates across sections should fail validation");

    assert!(matches!(
        error,
        CompatibilityProfileValidationError::DuplicateTextSectionEntryAcrossSections {
            entry: "release note entry",
            first_section: "validation-reference-point",
            second_section: "compatibility-caveat"
        }
    ));
}

#[test]
fn compatibility_profile_validate_rejects_custom_definition_labels_that_resolve_to_house_systems() {
    let mut profile = current_compatibility_profile();
    profile.custom_definition_labels = &["Placidus"];

    let error = profile
        .validate()
        .expect_err("custom-definition labels should not resolve to built-in house systems");

    assert!(matches!(
        error,
        CompatibilityProfileValidationError::CustomDefinitionLabelResolvesToBuiltIn {
            label: "Placidus"
        }
    ));
}

#[test]
fn compatibility_profile_validate_rejects_custom_definition_labels_that_resolve_to_ayanamsas() {
    let mut profile = current_compatibility_profile();
    profile.custom_definition_labels = &["Lahiri"];

    let error = profile
        .validate()
        .expect_err("custom-definition labels should not resolve to built-in ayanamsas");

    assert!(matches!(
        error,
        CompatibilityProfileValidationError::CustomDefinitionLabelResolvesToBuiltIn {
            label: "Lahiri"
        }
    ));
}

#[test]
fn compatibility_profile_custom_definition_ayanamsas_align_with_metadata_coverage() {
    let profile = current_compatibility_profile();
    let coverage = pleiades_ayanamsa::metadata_coverage();

    assert!(coverage
        .custom_definition_only
        .iter()
        .all(|label| profile.custom_definition_labels.contains(label)));
    assert!(profile.custom_definition_labels.contains(&"True Balarama"));
    assert!(profile.custom_definition_labels.contains(&"Aphoric"));
    assert!(profile.custom_definition_labels.contains(&"Takra"));
    assert!(pleiades_ayanamsa::resolve_ayanamsa("True Balarama").is_none());
    assert!(pleiades_ayanamsa::resolve_ayanamsa("Aphoric").is_none());
    assert!(pleiades_ayanamsa::resolve_ayanamsa("Takra").is_none());
    let mut resolved_custom_definition_labels = profile
        .custom_definition_labels
        .iter()
        .copied()
        .filter(|label| pleiades_ayanamsa::resolve_ayanamsa(label).is_some())
        .collect::<Vec<_>>();
    resolved_custom_definition_labels.sort_unstable();
    assert_eq!(
        resolved_custom_definition_labels,
        vec![
            "Babylonian (House Obs)",
            "Babylonian (House)",
            "Babylonian (Sissy)",
            "Babylonian (True Geoc)",
            "Babylonian (True Obs)",
            "Babylonian (True Topc)",
        ]
    );
    assert!(profile
        .custom_definition_labels
        .iter()
        .filter(|label| pleiades_ayanamsa::resolve_ayanamsa(label).is_none())
        .all(|label| {
            !profile
                .release_ayanamsas
                .iter()
                .any(|entry| entry.canonical_name == *label)
        }));
}

#[test]
fn compatibility_profile_validate_rejects_missing_custom_definition_ayanamsa_coverage_labels() {
    let mut profile = current_compatibility_profile();
    profile.custom_definition_labels = &["True Balarama", "Aphoric", "Takra"];

    let error = profile
        .validate()
        .expect_err("custom-definition ayanamsa coverage labels should be required in the profile");

    assert!(matches!(
        error,
        CompatibilityProfileValidationError::MissingCustomDefinitionAyanamsaCoverageLabel {
            label: "Babylonian (House)"
        }
    ));
}

#[test]
fn compatibility_profile_validate_rejects_house_descriptor_metadata_drift() {
    let mut profile = current_compatibility_profile();
    const HOUSE_SYSTEMS: &[HouseSystemDescriptor] = &[HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Placidus",
        &[],
        "Release note with trailing whitespace ",
        false,
        None,
    )];
    profile.house_systems = HOUSE_SYSTEMS;
    profile.baseline_house_systems = HOUSE_SYSTEMS;
    profile.release_house_systems = &[];

    let error = profile
        .validate()
        .expect_err("invalid house descriptors should fail compatibility-profile validation");

    assert!(matches!(
        error,
        CompatibilityProfileValidationError::HouseDescriptorValidationFailed {
            error: HouseCatalogValidationError::DescriptorNotesNotNormalized { label: "Placidus" }
        }
    ));
}

#[test]
fn compatibility_profile_validate_rejects_ayanamsa_descriptor_metadata_drift() {
    let mut profile = current_compatibility_profile();
    const AYANAMSA_SYSTEMS: &[AyanamsaDescriptor] = &[AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &[],
        "Release note with trailing whitespace ",
        None,
        None,
    )];
    profile.ayanamsas = AYANAMSA_SYSTEMS;
    profile.baseline_ayanamsas = AYANAMSA_SYSTEMS;
    profile.release_ayanamsas = &[];

    let error = profile
        .validate()
        .expect_err("invalid ayanamsa descriptors should fail compatibility-profile validation");

    assert!(matches!(
        error,
        CompatibilityProfileValidationError::AyanamsaDescriptorValidationFailed { error }
            if error.contains("descriptor note") && error.contains("Lahiri")
    ));
}

#[test]
fn compatibility_profile_retains_intentional_case_only_alias_variants() {
    let profile = current_compatibility_profile();

    let vehlow_equal = profile
        .release_house_systems
        .iter()
        .find(|entry| entry.canonical_name == "Vehlow Equal")
        .expect("Vehlow Equal should be part of the release catalog");
    assert!(vehlow_equal.aliases.contains(&"Vehlow equal"));

    let galactic_center = profile
        .release_ayanamsas
        .iter()
        .find(|entry| entry.canonical_name == "Galactic Center")
        .expect("Galactic Center should be part of the release catalog");
    assert!(galactic_center.aliases.contains(&"Galactic center"));

    let galactic_equator = profile
        .release_ayanamsas
        .iter()
        .find(|entry| entry.canonical_name == "Galactic Equator")
        .expect("Galactic Equator should be part of the release catalog");
    assert!(galactic_equator.aliases.contains(&"Galactic equator"));

    let galactic_center_cochrane = profile
        .release_ayanamsas
        .iter()
        .find(|entry| entry.canonical_name == "Galactic Center (Cochrane)")
        .expect("Galactic Center (Cochrane) should be part of the release catalog");
    assert!(galactic_center_cochrane
        .aliases
        .contains(&"Galactic center Cochrane"));

    let galactic_equator_true = profile
        .release_ayanamsas
        .iter()
        .find(|entry| entry.canonical_name == "Galactic Equator (True)")
        .expect("Galactic Equator (True) should be part of the release catalog");
    assert!(galactic_equator_true
        .aliases
        .contains(&"Galactic equator true"));

    assert!(profile.validate().is_ok());
}

#[test]
fn compatibility_profile_validate_rejects_case_insensitive_duplicate_house_labels_within_catalog() {
    let mut profile = current_compatibility_profile();
    const TOTAL_HOUSE_SYSTEMS: &[HouseSystemDescriptor] = &[
        HouseSystemDescriptor::new(
            HouseSystem::Placidus,
            "Total Placidus",
            &["Total Placidus alias"],
            "Total house coverage",
            false,
            None,
        ),
        HouseSystemDescriptor::new(
            HouseSystem::Equal,
            "Total Equal",
            &["Total Placidus alias"],
            "Total house coverage",
            false,
            None,
        ),
    ];
    const BASELINE_HOUSE_SYSTEMS: &[HouseSystemDescriptor] = &[HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Total Placidus",
        &["Total Placidus alias"],
        "Baseline house coverage",
        false,
        None,
    )];
    const RELEASE_HOUSE_SYSTEMS: &[HouseSystemDescriptor] = &[HouseSystemDescriptor::new(
        HouseSystem::Equal,
        "Total Equal",
        &["Total Equal alias"],
        "Release house coverage",
        false,
        None,
    )];
    profile.house_systems = TOTAL_HOUSE_SYSTEMS;
    profile.baseline_house_systems = BASELINE_HOUSE_SYSTEMS;
    profile.release_house_systems = RELEASE_HOUSE_SYSTEMS;

    let error = profile
        .validate()
        .expect_err("duplicate house labels should fail validation");

    assert!(matches!(
        error,
        CompatibilityProfileValidationError::CatalogLabelCollision {
            catalog_label: "house-system",
            ..
        }
    ));
    assert!(error.to_string().contains("duplicate label"));
}

#[test]
fn compatibility_profile_validate_rejects_case_insensitive_duplicate_ayanamsa_labels_within_catalog(
) {
    let mut profile = current_compatibility_profile();
    const TOTAL_AYANAMSAS: &[AyanamsaDescriptor] = &[
        AyanamsaDescriptor::new(
            Ayanamsa::Lahiri,
            "Total Lahiri",
            &["Total Lahiri alias"],
            "Total ayanamsa coverage",
            None,
            None,
        ),
        AyanamsaDescriptor::new(
            Ayanamsa::Krishnamurti,
            "Total Krishnamurti",
            &["Total Lahiri alias"],
            "Total ayanamsa coverage",
            None,
            None,
        ),
    ];
    const BASELINE_AYANAMSAS: &[AyanamsaDescriptor] = &[AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Total Lahiri",
        &["Total Lahiri alias"],
        "Baseline ayanamsa coverage",
        None,
        None,
    )];
    const RELEASE_AYANAMSAS: &[AyanamsaDescriptor] = &[AyanamsaDescriptor::new(
        Ayanamsa::Krishnamurti,
        "Total Krishnamurti",
        &["Total Krishnamurti alias"],
        "Release ayanamsa coverage",
        None,
        None,
    )];
    profile.ayanamsas = TOTAL_AYANAMSAS;
    profile.baseline_ayanamsas = BASELINE_AYANAMSAS;
    profile.release_ayanamsas = RELEASE_AYANAMSAS;

    let error = profile
        .validate()
        .expect_err("duplicate ayanamsa labels should fail validation");

    assert!(matches!(
        error,
        CompatibilityProfileValidationError::CatalogLabelCollision {
            catalog_label: "ayanamsa",
            ..
        }
    ));
    assert!(error.to_string().contains("duplicate label"));
}

#[test]
fn compatibility_profile_validate_rejects_case_insensitive_duplicate_house_labels_across_entries() {
    #[derive(Clone, Copy)]
    struct Entry {
        canonical_name: &'static str,
        aliases: &'static [&'static str],
    }

    impl report::AliasProfileEntry for Entry {
        fn canonical_name(&self) -> &'static str {
            self.canonical_name
        }

        fn aliases(&self) -> &'static [&'static str] {
            self.aliases
        }
    }

    let entries = [
        Entry {
            canonical_name: "Placidus",
            aliases: &["Placidus house system"],
        },
        Entry {
            canonical_name: "Koch",
            aliases: &["placidus"],
        },
    ];

    let error = validation::validate_catalog_label_uniqueness("house-system", &entries)
        .expect_err("cross-entry duplicate house labels should fail validation");
    assert!(matches!(
        error,
        CompatibilityProfileValidationError::CatalogLabelCollision {
            catalog_label: "house-system",
            label: "placidus"
        }
    ));
}

#[test]
fn compatibility_profile_validate_rejects_case_insensitive_duplicate_ayanamsa_labels_across_entries(
) {
    #[derive(Clone, Copy)]
    struct Entry {
        canonical_name: &'static str,
        aliases: &'static [&'static str],
    }

    impl report::AliasProfileEntry for Entry {
        fn canonical_name(&self) -> &'static str {
            self.canonical_name
        }

        fn aliases(&self) -> &'static [&'static str] {
            self.aliases
        }
    }

    let entries = [
        Entry {
            canonical_name: "Lahiri",
            aliases: &["Lahiri ayanamsa"],
        },
        Entry {
            canonical_name: "Raman",
            aliases: &["lahiri"],
        },
    ];

    let error = validation::validate_catalog_label_uniqueness("ayanamsa", &entries)
        .expect_err("cross-entry duplicate ayanamsa labels should fail validation");
    assert!(matches!(
        error,
        CompatibilityProfileValidationError::CatalogLabelCollision {
            catalog_label: "ayanamsa",
            label: "lahiri"
        }
    ));
}

#[test]
fn compatibility_profile_validate_rejects_cross_catalog_house_and_ayanamsa_label_collisions() {
    #[derive(Clone, Copy)]
    struct Entry {
        canonical_name: &'static str,
        aliases: &'static [&'static str],
    }

    impl report::AliasProfileEntry for Entry {
        fn canonical_name(&self) -> &'static str {
            self.canonical_name
        }

        fn aliases(&self) -> &'static [&'static str] {
            self.aliases
        }
    }

    let houses = [Entry {
        canonical_name: "Placidus",
        aliases: &["Equal house system"],
    }];
    let ayanamsas = [Entry {
        canonical_name: "Raman",
        aliases: &["equal house system"],
    }];

    let error =
        validation::validate_catalogs_are_disjoint("house-system", &houses, "ayanamsa", &ayanamsas)
            .expect_err("cross-catalog duplicate labels should fail validation");
    assert!(matches!(
        error,
        CompatibilityProfileValidationError::CrossCatalogLabelCollision {
            first_catalog_label: "house-system",
            second_catalog_label: "ayanamsa",
            label: "equal house system"
        }
    ));
}

#[test]
fn compatibility_profile_validate_rejects_overlapping_catalog_partitions() {
    let mut profile = current_compatibility_profile();
    profile.release_house_systems = profile.baseline_house_systems;

    let error = profile
        .validate()
        .expect_err("overlapping catalog partitions should fail validation");

    assert!(matches!(
        error,
        CompatibilityProfileValidationError::CatalogPartitionOverlap {
            catalog_label: "house-system",
            label: _
        }
    ));
}

#[test]
fn compatibility_profile_validate_rejects_inexact_house_catalog_coverage() {
    let mut profile = current_compatibility_profile();
    const TOTAL_HOUSE_SYSTEMS: &[HouseSystemDescriptor] = &[
        HouseSystemDescriptor::new(
            HouseSystem::Placidus,
            "Total Placidus",
            &["Total Placidus alias"],
            "Total house coverage",
            false,
            None,
        ),
        HouseSystemDescriptor::new(
            HouseSystem::Equal,
            "Total Equal",
            &[],
            "Total house coverage",
            false,
            None,
        ),
    ];
    const BASELINE_HOUSE_SYSTEMS: &[HouseSystemDescriptor] = &[HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Baseline Placidus",
        &["Baseline Placidus alias"],
        "Baseline house coverage",
        false,
        None,
    )];
    const RELEASE_HOUSE_SYSTEMS: &[HouseSystemDescriptor] = &[HouseSystemDescriptor::new(
        HouseSystem::Equal,
        "Release Equal",
        &["Release Equal alias"],
        "Release house coverage",
        false,
        None,
    )];
    profile.house_systems = TOTAL_HOUSE_SYSTEMS;
    profile.baseline_house_systems = BASELINE_HOUSE_SYSTEMS;
    profile.release_house_systems = RELEASE_HOUSE_SYSTEMS;

    let error = profile
        .validate()
        .expect_err("inexact catalog coverage should fail validation");

    assert!(matches!(
        error,
        CompatibilityProfileValidationError::CatalogCoverageLabelMismatch {
            catalog_label: "house-system",
            missing_label: "Total Placidus",
            unexpected_label: "Baseline Placidus"
        }
    ));
}

#[test]
fn compatibility_profile_validate_rejects_inexact_ayanamsa_catalog_coverage() {
    let mut profile = current_compatibility_profile();
    const TOTAL_AYANAMSAS: &[AyanamsaDescriptor] = &[
        AyanamsaDescriptor::new(
            Ayanamsa::Lahiri,
            "Total Lahiri",
            &["Total Lahiri alias"],
            "Total ayanamsa coverage",
            None,
            None,
        ),
        AyanamsaDescriptor::new(
            Ayanamsa::Krishnamurti,
            "Total Krishnamurti",
            &[],
            "Total ayanamsa coverage",
            None,
            None,
        ),
    ];
    const BASELINE_AYANAMSAS: &[AyanamsaDescriptor] = &[AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Baseline Lahiri",
        &["Baseline Lahiri alias"],
        "Baseline ayanamsa coverage",
        None,
        None,
    )];
    const RELEASE_AYANAMSAS: &[AyanamsaDescriptor] = &[AyanamsaDescriptor::new(
        Ayanamsa::Krishnamurti,
        "Release Krishnamurti",
        &["Release Krishnamurti alias"],
        "Release ayanamsa coverage",
        None,
        None,
    )];
    profile.ayanamsas = TOTAL_AYANAMSAS;
    profile.baseline_ayanamsas = BASELINE_AYANAMSAS;
    profile.release_ayanamsas = RELEASE_AYANAMSAS;

    let error = profile
        .validate()
        .expect_err("inexact catalog coverage should fail validation");

    assert!(matches!(
        error,
        CompatibilityProfileValidationError::CatalogCoverageLabelMismatch {
            catalog_label: "ayanamsa",
            missing_label: "Total Lahiri",
            unexpected_label: "Baseline Lahiri"
        }
    ));
}

#[test]
fn display_lists_release_sections() {
    let profile = current_compatibility_profile();
    assert!(profile.release_note().contains("David Cochrane"));
    assert!(profile.release_note().contains("Nick Anthony Fiorenza"));
    assert!(profile.release_note().contains("True Sheoran"));
    assert!(profile
        .release_note()
        .contains("Dhruva/Gal.Center/Mula (Wilhelm)"));
    assert!(profile.release_note().contains("Mula Wilhelm"));
    assert!(profile.release_note().contains("Wilhelm"));
    assert!(profile
        .release_note()
        .contains("Equal Midheaven table of houses"));
    assert!(profile.release_note().contains("Equal from MC"));
    assert!(profile.release_note().contains("Polich/Page"));
    assert!(profile.release_note().contains("Polich Page"));
    assert!(profile.release_note().contains("Equal/1=0 Aries"));
    assert!(profile.release_note().contains("Equal (cusp 1 = 0° Aries)"));
    assert!(profile.release_note().contains("Makransky Sunshine"));
    assert!(profile.release_note().contains("Pullen SD table of houses"));
    assert!(profile.release_note().contains("PVR Pushya Paksha"));
    assert!(profile
        .release_note()
        .contains("Pullen SD (Neo-Porphyry) table of houses"));
    assert!(profile.release_note().contains("Pullen SD (Neo-Porphyry)"));
    assert!(profile.release_note().contains("Neo-Porphyry"));
    assert!(profile
        .release_note()
        .contains("Pullen SD (Sinusoidal Delta)"));
    assert!(profile.release_note().contains("Pullen SR table of houses"));
    assert!(profile
        .release_note()
        .contains("Pullen SR (Sinusoidal Ratio) table of houses"));
    assert!(profile
        .release_note()
        .contains("Pullen SR (Sinusoidal Ratio)"));
    assert!(profile
        .release_note()
        .contains("Unsupported modes remain explicit"));

    let rendered = profile.to_string();
    assert!(rendered.contains("Target compatibility catalog:"));
    assert!(rendered.contains(
        "the full Swiss-Ephemeris-class house-system catalog remains the long-term compatibility goal."
    ));
    assert!(rendered.contains("Target ayanamsa catalog:"));
    assert!(rendered.contains(
        "the full Swiss-Ephemeris-class ayanamsa catalog remains the long-term compatibility goal."
    ));
    assert!(rendered.contains("Baseline compatibility milestone:"));
    assert!(rendered.contains("Release-specific coverage beyond baseline:"));
    assert!(rendered.contains("Alias mappings for built-in house systems:"));
    assert!(rendered.contains("Source-label aliases for built-in house systems:"));
    assert!(rendered.contains("Source-label aliases for built-in ayanamsas:"));
    assert!(rendered.contains("Alias mappings for built-in ayanamsas:"));
    assert!(rendered.contains("Coverage summary:"));
    assert!(rendered.contains("Unsupported modes: built-in UTC convenience remains out of scope; built-in Delta T remains out of scope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
    assert!(rendered.contains("house systems:"));
    assert!(rendered.contains("ayanamsas:"));
    assert!(rendered.contains(&format!(
        "house formula families: {}",
        profile.house_formula_families_summary_line()
    )));
    assert!(rendered.contains(&format!(
        "latitude-sensitive house systems: {}",
        profile.latitude_sensitive_house_systems_summary_line()
    )));
    assert!(rendered.contains(&format!(
        "latitude-sensitive house constraints: {}",
        profile.latitude_sensitive_house_constraints_summary_line()
    )));
    assert!(rendered.contains("ayanamsa sidereal metadata:"));
    assert!(rendered.contains(&format!(
        "custom-definition ayanamsas: {} (tracked without sidereal metadata)",
        pleiades_ayanamsa::metadata_coverage()
            .custom_definition_only
            .join(", ")
    )));
    assert!(rendered.contains("no unexpected sidereal-metadata gaps remain."));
    assert!(rendered.contains("custom-definition labels:"));
    assert!(rendered.contains("Validation reference points:"));
    assert!(rendered.contains("The stage-4 validation corpus remains the reference point for tightening house formulas whenever future revisions land."));
    assert!(rendered.contains("Babylonian (House) (aliases: Babylonian House, BABYL_HOUSE)"));
    assert!(rendered.contains("Treindl Sunshine"));
    assert!(rendered.contains("Makransky Sunshine"));
    assert!(rendered.contains("Sunshine table of houses, by Bob Makransky, Makransky Sunshine, Bob Makransky, Treindl Sunshine -> Sunshine"));
    assert!(rendered.contains("Placidus house system, Placidus table of houses -> Placidus"));
    assert!(rendered.contains("Koch houses, Koch house system, house system of the birth place, Koch table of houses, W. Koch, W Koch -> Koch"));
    assert!(rendered.contains("Whole Sign houses, Whole Sign table of houses, Whole-sign, Whole Sign system, Whole Sign house system -> Whole Sign"));
    assert_eq!(
        profile.latitude_sensitive_house_systems(),
        vec![
            "Placidus",
            "Koch",
            "Horizon/Azimuth",
            "APC",
            "Krusinski-Pisa-Goelzer",
            "Topocentric",
            "Sunshine",
            "Gauquelin sectors",
        ]
    );
    assert!(rendered.contains("Pullen SD (Neo-Porphyry) table of houses"));
    assert!(rendered.contains("Pullen SD (Sinusoidal Delta)"));
    assert!(rendered.contains("Pullen SR (Sinusoidal Ratio) table of houses"));
    assert!(rendered.contains("Babylonian/Aldebaran = 15 Tau"));
    assert!(rendered
        .contains("Babylonian (House Obs) (aliases: Babylonian House Obs, BABYL_HOUSE_OBS)"));
    assert!(rendered.contains(
        "Custom-definition-only Babylonian sidereal label (alias BABYL_HOUSE); not a Swiss Ephemeris sidereal mode. Swiss Ephemeris defines no SE_SIDM code for it."
    ));
    assert!(rendered.contains(
        "Custom-definition-only Babylonian sidereal label (alias BABYL_HOUSE_OBS); not a Swiss Ephemeris sidereal mode. Swiss Ephemeris defines no SE_SIDM code for it."
    ));
    assert!(rendered.contains(
        "Babylonian/Kugler 1, Babylonian Kugler 1, Babylonian 1 -> Babylonian (Kugler 1)"
    ));
    assert!(rendered.contains("D equal / MC, Equal from MC, Equal (from MC), Equal (from MC) table of houses, Equal (MC) table of houses, Equal/MC table of houses, Equal (MC) house system, Equal/MC house system, Equal MC, Equal/MC, Equal Midheaven, Equal Midheaven house system, Equal Midheaven table of houses, Equal/MC = 10th -> Equal (MC)"));
    assert!(rendered.contains("Equal (MC) table of houses"));
    assert!(rendered.contains("Equal Midheaven table of houses"));
    assert!(rendered.contains("Equal (MC) house system"));
    assert!(rendered.contains(
        "Equal (MC) table of houses, Equal/MC table of houses, Equal (MC) house system, Equal/MC house system, Equal MC, Equal/MC, Equal Midheaven, Equal Midheaven house system, Equal Midheaven table of houses, Equal/MC = 10th -> Equal (MC)"
    ));
    assert!(rendered.contains(
        "N, N whole sign houses, 1. house = Aries, Equal/1=Aries, Equal Aries, Aries houses, Whole Sign (house 1 = Aries), Whole Sign (house 1 = Aries) table of houses, Equal (1=Aries) table of houses, Equal/1=Aries table of houses, Equal (1=Aries) house system, Equal/1=Aries house system, Whole sign houses, 1. house = Aries, Equal/1=0 Aries, Equal (cusp 1 = 0° Aries) -> Equal (1=Aries)"
    ));
    assert!(rendered.contains("Equal (1=Aries) house system"));
    assert!(rendered.contains(
        "Galactic Center (Gil Brand), Gil Brand, Rgilbrand, Galactic center Rgilbrand -> Galactic Center (Rgilbrand)"
    ));
    assert!(rendered.contains(
        "Skydram, Skydram/Galactic Alignment, Skydram (Mardyks), Mardyks, Galactic center Mardyks -> Galactic Center (Mardyks)"
    ));
    assert!(rendered.contains("Galact. Center = 0 Sag, Gal. Center = 0 Sag -> Galactic Center"));
    assert!(rendered.contains(
        "Cochrane (Gal.Center = 0 Cap), Gal. Center = 0 Cap, Cochrane, Galactic center Cochrane, David Cochrane -> Galactic Center (Cochrane)"
    ));
    assert!(rendered.contains("Galactic equator, Gal. Eq. -> Galactic Equator"));
    assert!(rendered.contains("IAU 1958, Galactic equator IAU 1958 -> Galactic Equator (IAU 1958)"));
    assert!(rendered
        .contains("True galactic equator, Galactic equator true -> Galactic Equator (True)"));
    assert!(rendered.contains(
        "Galactic Equator mid-Mula, Mula galactic equator, Galactic equator Mula -> Galactic Equator (Mula)"
    ));
    assert!(rendered.contains(
        "Fiorenza, Galactic equator Fiorenza, Nick Anthony Fiorenza -> Galactic Equator (Fiorenza)"
    ));
    assert!(rendered.contains("Zij al-Shah, Sasanian -> Sassanian"));
    assert!(
        rendered.contains("Vettius Valens, Valens, Moon, Moon sign, Moon sign ayanamsa, Valens Moon ayanamsa -> Valens Moon")
    );
    assert!(rendered.contains("Suryasiddhanta, mean Sun"));
    assert!(rendered.contains("Surya Siddhanta, mean Sun"));
    assert!(rendered.contains("Surya Siddhanta mean sun"));
    assert!(rendered.contains("Surya Siddhanta mean-sun source forms"));
    assert!(rendered.contains("Aryabhata mean-sun source forms"));
    assert!(rendered.contains("Suryasiddhanta, mean Sun, Surya Siddhanta, mean Sun, Suryasiddhanta mean sun, Surya Siddhanta mean sun, Suryasiddhanta MSUN, Surya Siddhanta MSUN -> Suryasiddhanta (Mean Sun)"));
    assert!(rendered.contains(
        "Aryabhata, mean Sun, Aryabhata mean sun, Aryabhata MSUN -> Aryabhata (Mean Sun)"
    ));
    assert!(rendered.contains(
        "Suryasiddhanta, Surya Siddhanta, Suryasiddhanta 499, Surya Siddhanta 499, Suryasiddhanta 499 CE, Surya Siddhanta 499 CE -> Suryasiddhanta (499 CE)"
    ));
    assert!(rendered.contains(
        "Aryabhata, Aryabhata 499, Aryabhata 499 CE, Aryabhatan Kaliyuga, Aryabhata Kaliyuga -> Aryabhata (499 CE)"
    ));
    assert!(rendered.contains("Aryabhata 522, Aryabhata 522 CE -> Aryabhata (522 CE)"));
    assert!(rendered.contains("J. N. Bhasin, J.N. Bhasin, Bhasin -> JN Bhasin"));
    assert!(rendered.contains("Lahiri VP285, VP285 -> Lahiri (VP285)"));
    assert!(rendered.contains(
        "KP VP291, Krishnamurti VP291, Krishnamurti-Senthilathiban, VP291 -> Krishnamurti (VP291)"
    ));
    assert!(rendered.contains(
        "True Pushya (PVRN Rao), Pushya-paksha, Pushya Paksha, PVR Pushya Paksha, PVR, P.V.R. Narasimha Rao -> PVR Pushya-paksha"
    ));
    assert!(rendered.contains("True Pushya ayanamsa, Pushya -> True Pushya"));
    assert!(rendered
        .contains("True Citra ayanamsa, True Citra Paksha, True Chitra Paksha, True Chitrapaksha -> True Citra"));
    assert!(rendered.contains("Chitra, True Chitra ayanamsa -> True Chitra"));
    assert!(rendered.contains("True Revati ayanamsa -> True Revati"));
    assert!(rendered
        .contains("True Mula (Chandra Hari), True Mula ayanamsa, Chandra Hari -> True Mula"));
    assert!(rendered.contains("Udayagiri ayanamsa -> Udayagiri"));
    assert!(rendered.contains("ICRC Lahiri, Lahiri ICRC -> Lahiri (ICRC)"));
    assert!(rendered.contains("Lahiri original, Panchanga Darpan Lahiri -> Lahiri (1940)"));
    assert!(rendered.contains("De Luce, DeLuce ayanamsa -> DeLuce"));
    assert!(rendered.contains(
        "T, Polich-Page, Polich/Page, Polich Page, Polich-Page \"topocentric\" table of houses, T Polich/Page (\"topocentric\"), T topocentric, Topocentric house system, Topocentric table of houses -> Topocentric"
    ));
    assert!(rendered.contains("Polich-Page \"topocentric\" table of houses"));
    assert!(rendered.contains("T Polich/Page (\"topocentric\")"));
    assert!(rendered.contains(
        "Horizon, Azimuth, Horizontal, Azimuthal, Horizon table of houses, Horizontal table of houses, Azimuthal table of houses, Horizon/Azimuth table of houses, Horizon house system, Horizon/Azimuth house system, Horizontal house system, Azimuth house system, Azimuthal house system, horizon/azimut, horizon/azimuth -> Horizon/Azimuth"
    ));
    assert!(rendered.contains(
        "X, Meridian houses, Meridian table of houses, Meridian house system, ARMC, Axial Rotation, Axial rotation system, Zariel, X axial rotation system/ Meridian houses -> Meridian"
    ));
    assert!(rendered.contains("Axial variants, A -> Axial"));
    assert!(rendered.contains("M, Morinus houses, Morinus house system -> Morinus"));
    assert!(rendered.contains("Whole Sign house system -> Whole Sign"));
    assert!(rendered.contains("Equal table of houses, Whole Sign system, and Morinus house system spellings now called out explicitly in the quick-audit text"));
    assert!(rendered.contains("horizon/azimuth"));
    assert!(rendered.contains("Horizon/Azimuth house system"));
    assert!(rendered.contains("Horizontal house system"));
    assert!(rendered.contains("Horizontal table of houses"));
    assert!(rendered.contains("Azimuth house system"));
    assert!(rendered.contains("Azimuthal table of houses"));
    assert!(rendered.contains("Meridian house system"));
    assert!(rendered.contains("Horizon/Azimuth table of houses"));
    assert!(rendered
        .contains("Y, APC, Ram school, Ram's school, Ramschool, WvA, Y APC houses, APC houses, APC, also known as \u{201c}Ram school\u{201d}, table of houses, APC house system, Ascendant Parallel Circle -> APC"));
    assert!(rendered.contains(
        "Chitra Paksha, Chitrapaksha, Chitra-paksha, Lahiri Ayanamsha, Lahiri ayanamsa -> Lahiri"
    ));
    assert!(rendered.contains("Usha Shashi, Ushashashi, Usha-Shashi, Usha/Shashi, Usha / Shashi, Usha Shashi ayanamsa, Revati -> Usha Shashi"));
    assert!(rendered.contains("Yukteswar, Sri Yukteswar, Sri Yukteshwar, Shri Yukteswar, Shri Yukteshwar, Yukteshwar ayanamsa -> Yukteshwar"));
    assert!(rendered.contains("source-label appendix entries for Lahiri / Chitrapaksha / Chitra Paksha, True Chitra / Chitra, Krishnamurti Ayanamsha / Krishnamurti Ayanamsa / Krishnamurti ayanamsa / Krishnamurti (Swiss) / Krishnamurti Paddhati / KP ayanamsa, Fagan/Bradley Ayanamsha / Fagan/Bradley / Fagan Bradley / Fagan / Bradley / Fagan-Bradley, Usha Shashi / Usha / Shashi, and the Yukteshwar / Sri Yukteshwar / Shri Yukteshwar transliterations"));
    assert!(rendered.contains("source-label appendix entries for P.V.R. Narasimha Rao, Aries houses, and True Mula (Chandra Hari)"));
    assert!(rendered
        .contains("B. V. Raman, B.V. Raman, B V Raman, Raman Ayanamsha, Raman ayanamsa -> Raman"));
    assert!(rendered.contains(
        "Krishnamurti Ayanamsha, Krishnamurti Ayanamsa, Krishnamurti ayanamsa, Krishnamurti (Swiss), Krishnamurti Paddhati, KP ayanamsa -> Krishnamurti"
    ));
    assert!(rendered.contains("Krishnamurti (aliases: KP,"));
    assert!(rendered.contains(
        "Fagan/Bradley Ayanamsha, Fagan/Bradley, Fagan Bradley, Fagan / Bradley, Fagan-Bradley -> Fagan/Bradley"
    ));
    assert!(rendered.contains("Whole Sign (house 1 = Aries), Whole Sign (house 1 = Aries) table of houses, Equal (1=Aries) table of houses, Equal/1=Aries table of houses, Equal (1=Aries) house system, Equal/1=Aries house system, N whole sign houses, 1. house = Aries, Whole sign houses, 1. house = Aries, Equal/1=0 Aries, Equal (cusp 1 = 0° Aries) -> Equal (1=Aries)"));
    assert!(rendered.contains("Equal (1=Aries) table of houses"));
    assert!(rendered.contains("Equal from MC"));
    assert!(rendered.contains(
        "A equal, E equal = A, Equal houses, Equal house system, Equal House, Equal table of houses, Wang, Equal (cusp 1 = Asc) -> Equal"
    ));
    let source_label_section = rendered
        .split("Source-label aliases for built-in house systems:")
        .nth(1)
        .expect("source-label house appendix should be present");
    assert!(source_label_section.contains(
        "A equal, E equal = A, Equal houses, Equal house system, Equal House, Equal table of houses, Wang, Equal (cusp 1 = Asc) -> Equal"
    ));
    assert!(source_label_section.contains(
        "D equal / MC, Equal from MC, Equal (from MC), Equal (from MC) table of houses, Equal (MC) table of houses, Equal/MC table of houses, Equal (MC) house system, Equal/MC house system, Equal MC, Equal/MC, Equal Midheaven, Equal Midheaven house system, Equal Midheaven table of houses, Equal/MC = 10th -> Equal (MC)"
    ));
    assert!(source_label_section.contains(
        "Equal (1=Aries) table of houses, Equal/1=Aries table of houses, Equal (1=Aries) house system, Equal/1=Aries house system, Whole sign houses, 1. house = Aries, Equal/1=0 Aries, Equal (cusp 1 = 0° Aries) -> Equal (1=Aries)"
    ));
    assert!(source_label_section
        .contains("Equal Quadrant, Porphyry house system, Porphyry table of houses -> Porphyry"));
    assert!(source_label_section.contains("Axial variants, A -> Axial"));
    assert!(source_label_section.contains(
        "Regiomontanus houses, Regiomontanus house system, Regiomontanus table of houses -> Regiomontanus"
    ));
    assert!(source_label_section
        .contains("Campanus houses, Campanus house system, Campanus table of houses -> Campanus"));
    assert!(source_label_section.contains(
        "Alcabitius houses, Alcabitius house system, Alcabitius table of houses -> Alcabitius"
    ));
    assert!(rendered.contains("D equal / MC, Equal from MC, Equal (from MC), Equal (from MC) table of houses, Equal (MC) table of houses, Equal/MC table of houses, Equal (MC) house system, Equal/MC house system, Equal MC, Equal/MC, Equal Midheaven, Equal Midheaven house system, Equal Midheaven table of houses, Equal/MC = 10th -> Equal (MC)"));
    assert!(rendered.contains("Equal (MC) table of houses"));
    assert!(rendered.contains("J2000.0 -> J2000"));
    assert!(rendered.contains("J1900.0 -> J1900"));
    assert!(rendered.contains("B1950.0 -> B1950"));
    assert!(
        rendered.contains("Vettius Valens, Valens, Moon, Moon sign, Moon sign ayanamsa, Valens Moon ayanamsa -> Valens Moon")
    );
    assert!(rendered.contains("Equal (MC)"));
    assert!(rendered.contains("Equal (MC) table of houses"));
    assert!(rendered.contains("Equal (1=Aries)"));
    assert!(rendered.contains("Equal (1=Aries) table of houses"));
    assert!(rendered.contains("N, N whole sign houses, 1. house = Aries, Equal/1=Aries, Equal Aries, Aries houses, Whole Sign (house 1 = Aries), Whole Sign (house 1 = Aries) table of houses, Equal (1=Aries) table of houses, Equal/1=Aries table of houses, Equal (1=Aries) house system, Equal/1=Aries house system, Whole sign houses, 1. house = Aries, Equal/1=0 Aries, Equal (cusp 1 = 0° Aries) -> Equal (1=Aries)"));
    assert!(rendered.contains("Equal (1=Aries) table of houses"));
    assert!(
        rendered.contains("V equal Vehlow, Vehlow, Vehlow equal, Vehlow house system, Vehlow Equal house system, Vehlow-equal, Vehlow-equal table of houses, Vehlow Equal table of houses -> Vehlow Equal")
    );
    assert!(rendered.contains("Vehlow-equal table of houses, Vehlow Equal table of houses"));
    assert!(rendered.contains("Vehlow-equal, Vehlow -> Vehlow Equal"));
    assert!(rendered.contains(
        "S, S sripati, Śrīpati, Sripati house system, Sripati table of houses -> Sripati"
    ));
    assert!(rendered.contains("Carter (poli-equatorial)"));
    assert!(rendered.contains("Carter's poli-equatorial"));
    assert!(rendered.contains("Carter, Carter's poli-equatorial, Carter's poli-equatorial table of houses, Poli-Equatorial, Poli-equatorial -> Carter (poli-equatorial)"));
    assert!(rendered.contains("Horizon/Azimuth"));
    assert!(rendered.contains("APC"));
    assert!(rendered.contains("Krusinski-Pisa-Goelzer"));
    assert!(rendered.contains("U, Krusinski, Krusinski-Pisa, Krusinski Pisa, Krusinski/Pisa/Goelzer, Krusinski-Pisa-Goelzer table of houses, U krusinski-pisa-goelzer, Krusinski/Pisa/Goelzer house system, Pisa-Goelzer -> Krusinski-Pisa-Goelzer"));
    assert!(rendered.contains("Albategnius"));
    assert!(rendered.contains("Savard-A, Savard A, Savard's Albategnius -> Albategnius"));
    assert!(rendered.contains("Pullen SD"));
    assert!(rendered.contains("Pullen SD table of houses, Pullen SD (Neo-Porphyry) table of houses, Pullen SD (Neo-Porphyry), Neo-Porphyry, Pullen (Sinusoidal Delta), Pullen SD (Sinusoidal Delta), Pullen sinusoidal delta -> Pullen SD"));
    assert!(rendered.contains("Pullen SD table of houses, Pullen SD (Neo-Porphyry) table of houses, Pullen SD (Neo-Porphyry), Neo-Porphyry, Pullen (Sinusoidal Delta), Pullen SD (Sinusoidal Delta), Pullen SD (Sinusoidal Delta) table of houses, Pullen sinusoidal delta -> Pullen SD"));
    assert!(rendered.contains("Pullen SR"));
    assert!(rendered.contains(
        "Pullen SR table of houses, Pullen SR (Sinusoidal Ratio) table of houses, Pullen SR (Sinusoidal Ratio), Pullen (Sinusoidal Ratio), Pullen sinusoidal ratio -> Pullen SR"
    ));
    assert!(rendered.contains(
        "Babylonian/Kugler 1, Babylonian Kugler 1, Babylonian 1 -> Babylonian (Kugler 1)"
    ));
    assert!(rendered.contains(
        "Babylonian/Kugler 2, Babylonian Kugler 2, Babylonian 2 -> Babylonian (Kugler 2)"
    ));
    assert!(rendered.contains(
        "Babylonian/Kugler 3, Babylonian Kugler 3, Babylonian 3 -> Babylonian (Kugler 3)"
    ));
    assert!(rendered.contains("Babylonian/Huber, Babylonian Huber -> Babylonian (Huber)"));
    assert!(rendered.contains(
        "Aryabhata, Aryabhata 499, Aryabhata 499 CE, Aryabhatan Kaliyuga, Aryabhata Kaliyuga -> Aryabhata (499 CE)"
    ));
    assert!(rendered.contains(
        "I, I sunshine, Sunshine, Sunshine houses, Sunshine house system, Sunshine table of houses, Sunshine table of houses, by Bob Makransky, Makransky Sunshine, Bob Makransky, Treindl Sunshine -> Sunshine"
    ));
    assert!(rendered.contains("I sunshine"));
    assert!(rendered.contains(
        "S, S sripati, Śrīpati, Sripati house system, Sripati table of houses -> Sripati"
    ));
    assert!(rendered.contains("S sripati"));
    assert!(rendered.contains("P/K/R/C/O/E/W/N/V/A/H/B/M/S/I/G"));
    assert!(rendered.contains("plus the additional T/U/X/Y interoperability codes"));
    assert!(rendered.contains("A equal, E equal = A, Equal houses, Equal house system, Equal House, Equal table of houses, Wang, Equal (cusp 1 = Asc) -> Equal"));
    assert!(rendered
        .contains("Equal Quadrant, Porphyry house system, Porphyry table of houses -> Porphyry"));
    assert!(rendered.contains("Regiomontanus houses, Regiomontanus house system, Regiomontanus table of houses -> Regiomontanus"));
    assert!(rendered
        .contains("Campanus houses, Campanus house system, Campanus table of houses -> Campanus"));
    assert!(rendered.contains(
        "Alcabitius houses, Alcabitius house system, Alcabitius table of houses -> Alcabitius"
    ));
    assert!(rendered.contains("D equal / MC, Equal from MC, Equal (from MC), Equal (from MC) table of houses, Equal (MC) table of houses, Equal/MC table of houses, Equal (MC) house system, Equal/MC house system, Equal MC, Equal/MC, Equal Midheaven, Equal Midheaven house system, Equal Midheaven table of houses, Equal/MC = 10th -> Equal (MC)"));
    assert!(rendered.contains("Equal (MC) table of houses"));
    assert!(rendered.contains(
        "W equal, whole sign, Whole Sign houses, Whole Sign table of houses, Whole-sign, Whole Sign system, Whole Sign house system -> Whole Sign"
    ));
    assert!(
        rendered.contains("V equal Vehlow, Vehlow, Vehlow equal, Vehlow house system, Vehlow Equal house system, Vehlow-equal, Vehlow-equal table of houses, Vehlow Equal table of houses -> Vehlow Equal")
    );
    assert!(rendered.contains(
        "X, Meridian houses, Meridian table of houses, Meridian house system, ARMC, Axial Rotation, Axial rotation system, Zariel, X axial rotation system/ Meridian houses -> Meridian"
    ));
    assert!(rendered.contains("Axial variants, A -> Axial"));
    assert!(rendered.contains("Y, APC, Ram school, Ram's school, Ramschool, WvA, Y APC houses, APC houses, APC, also known as \u{201c}Ram school\u{201d}, table of houses, APC house system, Ascendant Parallel Circle -> APC"));
    assert!(rendered.contains("T, Polich-Page, Polich/Page, Polich Page, Polich-Page \"topocentric\" table of houses, T Polich/Page (\"topocentric\"), T topocentric, Topocentric house system, Topocentric table of houses -> Topocentric"));
    assert!(rendered.contains("Gauquelin sectors"));
    assert!(rendered
        .contains("G, Gauquelin, Gauquelin sector, Gauquelin sectors, Gauquelin table of sectors -> Gauquelin sectors"));
    assert!(rendered.contains("J2000"));
    assert!(rendered.contains("DeLuce"));
    assert!(rendered.contains("Yukteshwar"));
    assert!(rendered.contains("PVR Pushya-paksha"));
    assert!(rendered.contains(
        "Sunil Sheoran, Vedic Sheoran, Vedic / Sheoran, Sheoran ayanamsa, Sheoran true, True Sheoran ayanamsa, \"Vedic\"/Sheoran -> Sheoran"
    ));
    assert!(rendered.contains("True Revati"));
    assert!(rendered.contains("True Mula"));
    assert!(rendered.contains("Suryasiddhanta (Revati)"));
    assert!(rendered.contains("Suryasiddhanta (Citra)"));
    assert!(rendered.contains("Lahiri (ICRC)"));
    assert!(rendered.contains("Sassanian"));
    assert!(rendered.contains("Hipparchus"));
    assert!(rendered.contains("Babylonian (Kugler 1)"));
    assert!(rendered.contains("Babylonian (Aldebaran)"));
    assert!(rendered.contains("Babylonian (Eta Piscium)"));
    assert!(rendered.contains("Suryasiddhanta 499 CE"));
    assert!(rendered.contains("Surya Siddhanta 499 CE"));
    assert!(rendered.contains("Babylonian (House)"));
    assert!(rendered.contains("Babylonian (Sissy)"));
    assert!(rendered.contains("Babylonian (True Geoc)"));
    assert!(rendered.contains("Babylonian (True Topc)"));
    assert!(rendered.contains("Babylonian (True Obs)"));
    assert!(rendered.contains("Babylonian (House Obs)"));
    assert!(rendered.contains("Galactic Center"));
    assert!(rendered.contains("Dhruva Galactic Center (Middle Mula)"));
    assert!(rendered.contains("Galactic Equator"));
    assert!(rendered.contains("Compatibility caveats:"));
    assert!(rendered.contains("Placidus house system, Placidus table of houses -> Placidus"));
    assert!(rendered.contains("Porphyry house system, Porphyry table of houses -> Porphyry"));
    assert!(rendered.contains(
        "Regiomontanus houses, Regiomontanus house system, Regiomontanus table of houses -> Regiomontanus"
    ));
    assert!(rendered
        .contains("Campanus houses, Campanus house system, Campanus table of houses -> Campanus"));
    assert!(rendered.contains(
        "Alcabitius houses, Alcabitius house system, Alcabitius table of houses -> Alcabitius"
    ));
    assert!(rendered.contains("Equal (cusp 1 = Asc) -> Equal"));
    assert!(rendered.contains(
        "Koch houses, Koch house system, house system of the birth place, Koch table of houses, W. Koch, W Koch -> Koch"
    ));
    assert!(rendered.contains("Lahiri"));
    assert!(rendered.contains("Custom-definition labels:"));
    assert!(rendered.contains("- True Balarama"));
    assert!(rendered.contains("- Aphoric"));
    assert!(rendered.contains("- Takra"));
    assert!(rendered.contains("custom definitions"));
    let house_alias_count: usize = profile
        .house_systems
        .iter()
        .map(|entry| entry.aliases.len())
        .sum();
    let ayanamsa_alias_count: usize = profile
        .ayanamsas
        .iter()
        .map(|entry| entry.aliases.len())
        .sum();
    let ayanamsa_alias_bearing_entry_count: usize = profile
        .ayanamsas
        .iter()
        .filter(|entry| !entry.aliases.is_empty())
        .count();
    let ayanamsa_provenance = pleiades_ayanamsa::validated_provenance_summary_for_report()
        .expect("ayanamsa provenance summary should validate");
    assert_eq!(
        profile.ayanamsa_provenance_summary_line(),
        ayanamsa_provenance
    );
    assert_eq!(
        profile
            .validated_ayanamsa_provenance_summary_line()
            .expect("ayanamsa provenance summary should validate"),
        ayanamsa_provenance
    );
    assert!(rendered.contains(&format!(
        "Compatibility catalog inventory: house systems={} ({} baseline, {} release-specific, {} aliases); house formula families={}; house latitude-sensitive constraints={}; house-code aliases={}; ayanamsas={} ({} baseline, {} release-specific, {} aliases); custom-definition labels={}; custom-definition ayanamsa labels={} (Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs)); ayanamsa metadata gaps={}; ayanamsa alias-bearing entries={}; catalog posture=house systems={} ({} constrained, {} unconstrained); ayanamsas={} ({} descriptor-only, {} metadata-bearing); custom-only labels={}; custom-only ayanamsa labels={}; ayanamsa provenance={}; known gaps={}; claim audit: baseline catalogs are the published guarantees; release-specific entries are shipped additions; custom-definition labels remain custom-definition territory; descriptor-only ayanamsa entries remain catalog descriptors; constrained house systems stay explicitly flagged; known gaps stay documented",
        profile.house_systems.len(),
        profile.baseline_house_systems.len(),
        profile.release_house_systems.len(),
        house_alias_count,
        profile.house_formula_families_summary_line(),
        profile.latitude_sensitive_house_constraints_summary_line(),
        pleiades_houses::house_system_code_aliases().len(),
        profile.ayanamsas.len(),
        profile.baseline_ayanamsas.len(),
        profile.release_ayanamsas.len(),
        ayanamsa_alias_count,
        profile.custom_definition_labels.len(),
        profile.custom_definition_ayanamsa_labels().len(),
        pleiades_ayanamsa::metadata_coverage().without_sidereal_metadata.len(),
        ayanamsa_alias_bearing_entry_count,
        profile.house_systems.len(),
        profile.constrained_house_system_count(),
        profile
            .house_systems
            .len()
            .saturating_sub(profile.constrained_house_system_count()),
        profile.ayanamsas.len(),
        profile.ayanamsa_descriptor_only_count(),
        profile
            .ayanamsas
            .len()
            .saturating_sub(profile.ayanamsa_descriptor_only_count()),
        profile.custom_definition_labels.len(),
        profile.custom_definition_ayanamsa_labels().len(),
        ayanamsa_provenance,
        profile.known_gaps.len()
    )));
    assert!(rendered.contains("house systems: 25 total"));
    assert!(rendered.contains("ayanamsas: 59 total"));
    assert!(rendered.contains("ayanamsa metadata gaps=0"));
    assert_eq!(
        profile.catalog_posture_summary_line(),
        catalog_posture_summary_for_report()
    );
    assert_eq!(
        profile
            .validated_catalog_posture_summary_line()
            .expect("catalog posture summary should validate"),
        catalog_posture_summary_for_report()
    );
    assert!(rendered.contains("ayanamsa alias-bearing entries="));
}

#[test]
fn canonical_name_helpers_preserve_release_and_baseline_order() {
    let profile = current_compatibility_profile();

    assert_eq!(
        profile.baseline_house_system_canonical_names(),
        profile
            .baseline_house_systems
            .iter()
            .map(|entry| entry.canonical_name)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        profile.release_house_system_canonical_names(),
        profile
            .release_house_systems
            .iter()
            .map(|entry| entry.canonical_name)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        profile.baseline_ayanamsa_canonical_names(),
        profile
            .baseline_ayanamsas
            .iter()
            .map(|entry| entry.canonical_name)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        profile.release_ayanamsa_canonical_names(),
        profile
            .release_ayanamsas
            .iter()
            .map(|entry| entry.canonical_name)
            .collect::<Vec<_>>()
    );
}

#[test]
fn catalog_inventory_summary_line_reports_the_house_code_alias_count() {
    let profile = current_compatibility_profile();
    let rendered = profile.catalog_inventory_summary_line();

    assert!(rendered.contains(&format!(
        "house-code aliases={}",
        profile.house_code_alias_count()
    )));
    assert!(rendered.contains(&format!(
        "custom-definition ayanamsa labels={} (Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs))",
        profile.custom_definition_ayanamsa_labels().len()
    )));
    assert!(rendered.contains("ayanamsa provenance=representative provenance examples:"));
    assert!(rendered.contains(&format!(
        "house latitude-sensitive constraints={}",
        profile.latitude_sensitive_house_constraints_summary_line()
    )));
    assert_eq!(
        profile.custom_definition_ayanamsa_labels(),
        pleiades_ayanamsa::metadata_coverage().custom_definition_only
    );
    assert_eq!(
        profile.custom_definition_ayanamsa_labels_summary_line(),
        "6 (Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs))"
    );
    assert_eq!(
        profile.validated_catalog_inventory_summary_line(),
        Ok(rendered.clone())
    );
    assert_eq!(
        validated_catalog_inventory_summary_for_report(),
        Ok(rendered.clone())
    );
    assert_eq!(
        profile.validated_house_code_aliases_summary_line(),
        Ok(profile.house_code_aliases_summary_line())
    );
}

#[test]
fn catalog_inventory_summary_line_validation_rejects_invalid_profiles() {
    let profile = current_compatibility_profile();
    let invalid_profile = CompatibilityProfile {
        summary: "",
        ..profile
    };

    assert!(invalid_profile
        .validated_catalog_inventory_summary_line()
        .is_err());
    assert!(invalid_profile
        .validated_house_code_aliases_summary_line()
        .is_err());
    assert!(invalid_profile
        .validated_release_house_system_canonical_names_summary_line()
        .is_err());
    assert!(invalid_profile
        .validated_release_ayanamsa_canonical_names_summary_line()
        .is_err());
    assert!(invalid_profile
        .validated_house_formula_families_summary_line()
        .is_err());
    assert!(invalid_profile
        .validated_latitude_sensitive_house_systems_summary_line()
        .is_err());
    assert!(invalid_profile
        .validated_latitude_sensitive_house_constraints_summary_line()
        .is_err());
    assert!(invalid_profile
        .validated_custom_definition_ayanamsa_labels_summary_line()
        .is_err());
    assert_eq!(
        invalid_profile.to_string(),
        "Compatibility profile unavailable (compatibility profile summary is blank)"
    );
}

#[test]
fn compatibility_caveats_summary_for_report_tracks_the_current_profile() {
    let profile = current_compatibility_profile();
    let release_profiles = crate::release_profiles::current_release_profile_identifiers();
    let rendered = compatibility_caveats_summary_for_report(&profile, &release_profiles);

    assert!(rendered.starts_with("Compatibility caveats summary\nProfile: "));
    assert!(rendered.contains(release_profiles.compatibility_profile_id));
    assert!(rendered.contains("House formula families: "));
    assert!(rendered.contains("Latitude-sensitive house systems: "));
    assert!(rendered.contains("Latitude-sensitive house failure modes: "));
    assert!(rendered.contains("Descriptor-only ayanamsa labels: "));
    let expected_prefix = format!(
        "Compatibility caveats summary\nProfile: {}\nCompatibility caveats: {}\nHouse formula families: {}\nLatitude-sensitive house systems: {}\nLatitude-sensitive house constraints: {}\nLatitude-sensitive house failure modes: {}\nDescriptor-only ayanamsa labels: {}\n",
        release_profiles.compatibility_profile_id,
        profile.known_gaps.len(),
        profile.house_formula_families_summary_line(),
        profile.latitude_sensitive_house_systems_summary_line(),
        profile.latitude_sensitive_house_constraints_summary_line(),
        profile.latitude_sensitive_house_failure_modes_summary_line(),
        profile.custom_definition_ayanamsa_labels_summary_line()
    );
    assert!(rendered.starts_with(&expected_prefix));
    for gap in profile.known_gaps {
        assert!(rendered.contains(gap));
    }
}

#[test]
fn house_code_alias_inventory_summary_tracks_the_built_in_table() {
    let profile = current_compatibility_profile();
    let summary = profile.house_code_alias_inventory_summary();

    assert_eq!(
        summary.count(),
        pleiades_houses::house_system_code_aliases().len()
    );
    assert_eq!(
        summary.summary_line(),
        profile.house_code_aliases_summary_line()
    );
    assert_eq!(
        summary.to_string(),
        profile.house_code_aliases_summary_line()
    );
    assert_eq!(
        summary.validated_summary_line(),
        Ok(profile.house_code_aliases_summary_line())
    );
    assert_eq!(
        summary
            .validate()
            .expect("built-in aliases should validate"),
        summary.count()
    );
}

#[test]
fn house_code_alias_inventory_summary_validation_rejects_empty_inventory() {
    let summary = HouseCodeAliasInventorySummary::new(&[]);

    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn compatibility_profile_validation_rejects_release_summary_drift() {
    let mut profile = current_compatibility_profile();
    profile.summary = "Stage 6 release profile: all built-ins are fully complete and the compatibility catalog is already exhaustive";

    let error = profile
        .validate()
        .expect_err("summary overclaim should fail validation");
    assert_eq!(
        error,
        CompatibilityProfileValidationError::SummaryDoesNotDescribeReleaseSplit
    );
    assert!(profile.validated_release_note().is_err());
}

#[test]
fn compatibility_profile_validation_rejects_release_summary_overstatement_even_with_required_phrases(
) {
    let mut profile = current_compatibility_profile();
    profile.summary = "Stage 6 release profile: the baseline catalogs remain published as a routine release artifact while the target Swiss-Ephemeris-class compatibility catalog stays explicit, and all built-ins are fully complete and exhaustive";

    let error = profile
        .validate()
        .expect_err("summary overstatement should fail validation");
    assert_eq!(
        error,
        CompatibilityProfileValidationError::SummaryDoesNotDescribeReleaseSplit
    );
}

#[test]
fn compatibility_profile_validated_release_note_tracks_the_built_in_summary() {
    let profile = current_compatibility_profile();

    assert_eq!(profile.validated_release_note(), Ok(profile.release_note()));
}

#[test]
fn house_code_alias_validation_accepts_the_built_in_table() {
    assert_eq!(
        validation::validate_house_code_aliases(pleiades_houses::house_system_code_aliases())
            .expect("built-in house-code aliases should validate"),
        pleiades_houses::house_system_code_aliases().len()
    );
}

#[test]
fn release_canonical_name_summaries_track_the_built_in_catalogs() {
    let profile = current_compatibility_profile();
    let house_summary =
        report::format_canonical_name_summary(&profile.release_house_system_canonical_names());
    let ayanamsa_summary =
        report::format_canonical_name_summary(&profile.release_ayanamsa_canonical_names());

    assert_eq!(
        profile.release_house_system_canonical_names_summary_line(),
        house_summary
    );
    assert_eq!(
        profile.release_ayanamsa_canonical_names_summary_line(),
        ayanamsa_summary
    );
    assert_eq!(
        profile.validated_release_house_system_canonical_names_summary_line(),
        Ok(house_summary.clone())
    );
    assert_eq!(
        profile.validated_release_ayanamsa_canonical_names_summary_line(),
        Ok(ayanamsa_summary.clone())
    );
    assert_eq!(
        release_house_system_canonical_names_summary_for_report(),
        house_summary
    );
    assert_eq!(
        release_ayanamsa_canonical_names_summary_for_report(),
        ayanamsa_summary
    );
    assert_eq!(
        validated_release_house_system_canonical_names_summary_for_report(),
        Ok(profile.release_house_system_canonical_names_summary_line())
    );
    assert_eq!(
        validated_release_ayanamsa_canonical_names_summary_for_report(),
        Ok(profile.release_ayanamsa_canonical_names_summary_line())
    );
    assert_eq!(
        profile.validated_house_formula_families_summary_line(),
        Ok(profile.house_formula_families_summary_line())
    );
    assert_eq!(
        profile.validated_latitude_sensitive_house_systems_summary_line(),
        Ok(profile.latitude_sensitive_house_systems_summary_line())
    );
    assert_eq!(
        profile.validated_custom_definition_ayanamsa_labels_summary_line(),
        Ok(profile.custom_definition_ayanamsa_labels_summary_line())
    );
}

#[test]
fn house_code_alias_validation_rejects_duplicates_and_round_trip_drift() {
    use pleiades_houses::HouseSystemCodeAlias;

    let duplicate_aliases = [
        HouseSystemCodeAlias {
            label: "P",
            system: HouseSystem::Placidus,
        },
        HouseSystemCodeAlias {
            label: "p",
            system: HouseSystem::Placidus,
        },
    ];
    assert!(matches!(
        validation::validate_house_code_aliases(&duplicate_aliases),
        Err(
            CompatibilityProfileValidationError::DuplicateTextSectionEntry {
                section_label: "house-code-alias",
                entry: "p"
            }
        )
    ));

    let drifted_aliases = [HouseSystemCodeAlias {
        label: "P",
        system: HouseSystem::Koch,
    }];
    assert!(matches!(
        validation::validate_house_code_aliases(&drifted_aliases),
        Err(
            CompatibilityProfileValidationError::HouseCodeAliasDoesNotRoundTrip {
                label: "P",
                expected_system: HouseSystem::Koch,
            }
        )
    ));
}

#[test]
fn rendered_profile_matches_pinned_content_checksum() {
    let rendered = current_compatibility_profile().to_string();
    let actual = pleiades_time::fnv1a64(&rendered);
    assert_eq!(
        actual, CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM,
        "rendered compatibility profile changed (checksum {actual:#018x}); bump \
         CURRENT_COMPATIBILITY_PROFILE_ID and update \
         CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM in the same commit"
    );
}

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
