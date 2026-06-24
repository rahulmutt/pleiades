use pleiades_types::{Angle, Ayanamsa, CustomAyanamsa, Instant, JulianDay, TimeScale};

use crate::lookup::validate_ayanamsa_catalog_entries;
use crate::{
    ayanamsa_catalog_validation_summary, built_in_ayanamsas, custom_definition_ayanamsa_labels,
    custom_definition_example_ayanamsa_labels, descriptor, metadata_coverage,
    provenance_sample_ayanamsas, provenance_summary, reference_offset_sample_ayanamsas,
    resolve_ayanamsa, sidereal_offset, validate_ayanamsa_catalog, AyanamsaCatalogValidationError,
    AyanamsaCatalogValidationSummaryValidationError, AyanamsaDescriptor,
    AyanamsaMetadataCoverageValidationError, AyanamsaProvenanceExample, AyanamsaProvenanceSummary,
    AyanamsaProvenanceSummaryValidationError,
};

fn assert_close_degrees(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-12,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn descriptor_summary_line_includes_aliases_reference_metadata_and_notes() {
    let d = AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &["Alias One", "Alias Two"],
        "Summary note",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(23.5)),
    );

    let expected =
        "Lahiri (aliases: Alias One, Alias Two) [epoch: JD 2451545] [offset: 23.5°] — Summary note";
    assert_eq!(d.summary_line(), expected);
    assert_eq!(d.validated_summary_line(), Ok(expected.to_string()));
    assert_eq!(d.to_string(), expected);
}

#[test]
fn validated_summary_line_rejects_partial_sidereal_metadata() {
    let d = AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &[],
        "Summary note",
        Some(JulianDay::from_days(2_451_545.0)),
        None,
    );

    assert_eq!(
        d.validated_summary_line(),
        Err(AyanamsaCatalogValidationError::PartialSiderealMetadata { label: "Lahiri" })
    );
}

#[test]
fn reference_epoch_offsets_match_the_documented_baseline_values() {
    for body in [
        Ayanamsa::Lahiri,
        Ayanamsa::Raman,
        Ayanamsa::Krishnamurti,
        Ayanamsa::FaganBradley,
        Ayanamsa::TrueChitra,
    ] {
        let d = descriptor(&body).expect("baseline ayanamsa should resolve");
        let epoch = Instant::new(
            d.epoch.expect("baseline ayanamsa should carry an epoch"),
            TimeScale::Tt,
        );
        let offset = d
            .offset_at(epoch)
            .expect("baseline ayanamsa should carry an offset");
        assert_close_degrees(offset.degrees(), d.offset_degrees.unwrap().degrees());
    }
}

#[test]
fn catalog_validation_summary_reports_catalog_health() {
    let summary = ayanamsa_catalog_validation_summary();
    let expected_custom_definition_only_labels =
        metadata_coverage().custom_definition_only.join(", ");

    assert_eq!(summary.entry_count, built_in_ayanamsas().len());
    assert_eq!(
        summary.baseline_entry_count,
        crate::baseline_ayanamsas().len()
    );
    assert_eq!(
        summary.release_entry_count,
        crate::release_ayanamsas().len()
    );
    assert!(matches!(summary.validation_result, Ok(())));
    assert!(summary
        .summary_line()
        .contains("ayanamsa catalog validation: ok"));
    assert!(summary.summary_line().contains("custom-definition-only="));
    assert!(summary
        .summary_line()
        .contains("implementation posture: 5 baseline entries, 54 release-specific entries, 6 custom-definition-only labels"));
    assert!(summary
        .summary_line()
        .contains(&expected_custom_definition_only_labels));
    assert!(summary.validated_summary_line().is_ok());
    assert_eq!(validate_ayanamsa_catalog(), Ok(()));
}

#[test]
fn catalog_validation_summary_validated_summary_line_rejects_label_count_drift() {
    let mut summary = ayanamsa_catalog_validation_summary();
    summary.label_count += 1;

    let error = summary
        .validated_summary_line()
        .expect_err("label count drift should be rejected");
    assert_eq!(
        error,
        AyanamsaCatalogValidationSummaryValidationError::FieldOutOfSync {
            field: "label_count",
        }
    );
    assert!(error.to_string().contains("label_count"));
}

#[test]
fn reference_offset_sample_ayanamsas_match_the_documented_release_set() {
    assert_eq!(
        reference_offset_sample_ayanamsas(),
        &[
            Ayanamsa::Lahiri,
            Ayanamsa::LahiriIcrc,
            Ayanamsa::Lahiri1940,
            Ayanamsa::UshaShashi,
            Ayanamsa::Raman,
            Ayanamsa::Krishnamurti,
            Ayanamsa::FaganBradley,
            Ayanamsa::TrueChitra,
            Ayanamsa::TrueCitra,
            Ayanamsa::SuryasiddhantaRevati,
            Ayanamsa::SuryasiddhantaCitra,
            Ayanamsa::DeLuce,
            Ayanamsa::Yukteshwar,
            Ayanamsa::PvrPushyaPaksha,
            Ayanamsa::J2000,
            Ayanamsa::J1900,
            Ayanamsa::B1950,
            Ayanamsa::TrueRevati,
            Ayanamsa::TrueMula,
            Ayanamsa::TruePushya,
            Ayanamsa::Udayagiri,
            Ayanamsa::LahiriVP285,
            Ayanamsa::KrishnamurtiVP291,
            Ayanamsa::Sheoran,
            Ayanamsa::TrueSheoran,
            Ayanamsa::Hipparchus,
            Ayanamsa::DjwhalKhul,
            Ayanamsa::GalacticCenter,
            Ayanamsa::GalacticCenterRgilbrand,
            Ayanamsa::GalacticCenterMardyks,
            Ayanamsa::GalacticCenterCochrane,
            Ayanamsa::GalacticCenterMulaWilhelm,
            Ayanamsa::DhruvaGalacticCenterMula,
            Ayanamsa::GalacticEquatorIau1958,
            Ayanamsa::GalacticEquatorFiorenza,
            Ayanamsa::GalacticEquatorTrue,
            Ayanamsa::GalacticEquatorMula,
            Ayanamsa::ValensMoon,
            Ayanamsa::BabylonianBritton,
            Ayanamsa::BabylonianKugler1,
            Ayanamsa::BabylonianKugler2,
            Ayanamsa::BabylonianKugler3,
            Ayanamsa::BabylonianEtaPiscium,
            Ayanamsa::BabylonianAldebaran,
            Ayanamsa::BabylonianHuber,
            Ayanamsa::Aryabhata499,
            Ayanamsa::Sassanian,
            Ayanamsa::JnBhasin,
            Ayanamsa::GalacticEquator,
            Ayanamsa::Suryasiddhanta499,
            Ayanamsa::Suryasiddhanta499MeanSun,
            Ayanamsa::Aryabhata499MeanSun,
            Ayanamsa::Aryabhata522,
        ]
    );
}

#[test]
fn provenance_sample_ayanamsas_match_the_documented_release_set() {
    assert_eq!(
        provenance_sample_ayanamsas(),
        &[
            Ayanamsa::TrueCitra,
            Ayanamsa::TrueRevati,
            Ayanamsa::TrueMula,
            Ayanamsa::TruePushya,
            Ayanamsa::Udayagiri,
            Ayanamsa::TrueSheoran,
            Ayanamsa::BabylonianBritton,
            Ayanamsa::GalacticCenterRgilbrand,
            Ayanamsa::BabylonianKugler1,
            Ayanamsa::GalacticEquator,
            Ayanamsa::Suryasiddhanta499MeanSun,
            Ayanamsa::Aryabhata522,
            Ayanamsa::ValensMoon,
        ]
    );
}

#[test]
fn provenance_summary_reports_the_documented_examples() {
    let summary = provenance_summary();
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert!(summary
        .summary_line()
        .contains("representative provenance examples:"));
    assert!(summary.summary_line().contains("True Citra —"));
    assert!(summary.summary_line().contains("True Revati —"));
    assert!(summary.summary_line().contains("True Mula —"));
    assert!(summary.summary_line().contains("True Pushya —"));
    assert!(summary.summary_line().contains("Udayagiri —"));
    assert!(summary.summary_line().contains("True Sheoran —"));
    assert!(summary.summary_line().contains("Babylonian (Britton) —"));
    assert!(summary
        .summary_line()
        .contains("Galactic Center (Rgilbrand) —"));
    assert!(summary.summary_line().contains("Babylonian (Kugler 1) —"));
    assert!(summary.summary_line().contains("Galactic Equator —"));
    assert!(summary
        .summary_line()
        .contains("Suryasiddhanta (Mean Sun) —"));
    assert!(summary.summary_line().contains("Aryabhata (522 CE) —"));
    assert!(summary.summary_line().contains("Valens Moon —"));
}

#[test]
fn provenance_summary_validation_rejects_example_drift() {
    let summary = AyanamsaProvenanceSummary {
        examples: vec![AyanamsaProvenanceExample {
            canonical_name: "Example",
            provenance_note: "drifted note",
        }],
    };

    assert!(matches!(
        summary.validate(),
        Err(AyanamsaProvenanceSummaryValidationError::FieldOutOfSync { field: "examples" })
    ));
}

#[test]
fn catalog_validation_entries_reject_duplicate_labels() {
    let duplicate_entries = [
        AyanamsaDescriptor::new(
            Ayanamsa::Lahiri,
            "Lahiri",
            &[],
            "Summary note",
            Some(JulianDay::from_days(2_451_545.0)),
            Some(Angle::from_degrees(23.5)),
        ),
        AyanamsaDescriptor::new(
            Ayanamsa::Lahiri,
            "Lahiri",
            &[],
            "Summary note",
            Some(JulianDay::from_days(2_451_545.0)),
            Some(Angle::from_degrees(23.5)),
        ),
    ];

    assert!(matches!(
        validate_ayanamsa_catalog_entries(&duplicate_entries),
        Err(AyanamsaCatalogValidationError::DuplicateLabel { label: "Lahiri" })
    ));

    let blank_name_descriptor = AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        " ",
        &[],
        "Summary note",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(23.5)),
    );
    assert!(matches!(
        blank_name_descriptor.validate(),
        Err(
            AyanamsaCatalogValidationError::DescriptorLabelNotNormalized {
                label: " ",
                field: "canonical name"
            }
        )
    ));

    let padded_alias_descriptor = AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &[" alias "],
        "Summary note",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(23.5)),
    );
    assert!(matches!(
        padded_alias_descriptor.validate(),
        Err(
            AyanamsaCatalogValidationError::DescriptorLabelNotNormalized {
                label: " alias ",
                field: "alias"
            }
        )
    ));

    let blank_notes_descriptor = AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &[],
        " ",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(23.5)),
    );
    assert!(matches!(
        blank_notes_descriptor.validate(),
        Err(AyanamsaCatalogValidationError::DescriptorNotesNotNormalized { label: "Lahiri" })
    ));

    let duplicate_alias_descriptor = AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &["KP", "kp"],
        "Summary note",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(23.5)),
    );
    assert!(matches!(
        duplicate_alias_descriptor.validate(),
        Err(AyanamsaCatalogValidationError::DescriptorLabelCollision {
            label: "kp",
            canonical_name: "Lahiri"
        })
    ));

    let blank_notes_entry = [blank_notes_descriptor];
    assert!(matches!(
        validate_ayanamsa_catalog_entries(&blank_notes_entry),
        Err(AyanamsaCatalogValidationError::DescriptorNotesNotNormalized { label: "Lahiri" })
    ));

    let line_break_name_descriptor = AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "La\nhiri",
        &[],
        "Summary note",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(23.5)),
    );
    assert!(matches!(
        line_break_name_descriptor.validate(),
        Err(
            AyanamsaCatalogValidationError::DescriptorLabelNotNormalized {
                label: "La\nhiri",
                field: "canonical name"
            }
        )
    ));

    let line_break_alias_descriptor = AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &["La\nhiri alias"],
        "Summary note",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(23.5)),
    );
    assert!(matches!(
        line_break_alias_descriptor.validate(),
        Err(
            AyanamsaCatalogValidationError::DescriptorLabelNotNormalized {
                label: "La\nhiri alias",
                field: "alias"
            }
        )
    ));

    let line_break_notes_descriptor = AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &[],
        "Summary note\nline two",
        Some(JulianDay::from_days(2_451_545.0)),
        Some(Angle::from_degrees(23.5)),
    );
    assert!(matches!(
        line_break_notes_descriptor.validate(),
        Err(AyanamsaCatalogValidationError::DescriptorNotesNotNormalized { label: "Lahiri" })
    ));
}

#[test]
fn catalog_validation_entries_reject_partial_sidereal_metadata() {
    let epoch_only_descriptor = AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &[],
        "Summary note",
        Some(JulianDay::from_days(2_451_545.0)),
        None,
    );
    assert!(matches!(
        epoch_only_descriptor.validate(),
        Err(AyanamsaCatalogValidationError::PartialSiderealMetadata { label: "Lahiri" })
    ));
    assert!(matches!(
        validate_ayanamsa_catalog_entries(&[epoch_only_descriptor]),
        Err(AyanamsaCatalogValidationError::PartialSiderealMetadata { label: "Lahiri" })
    ));

    let offset_only_descriptor = AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &[],
        "Summary note",
        None,
        Some(Angle::from_degrees(23.5)),
    );
    assert!(matches!(
        offset_only_descriptor.validate(),
        Err(AyanamsaCatalogValidationError::PartialSiderealMetadata { label: "Lahiri" })
    ));
    assert!(matches!(
        validate_ayanamsa_catalog_entries(&[offset_only_descriptor]),
        Err(AyanamsaCatalogValidationError::PartialSiderealMetadata { label: "Lahiri" })
    ));
}

#[test]
fn aliases_resolve_to_builtin_ayanamsas() {
    assert_eq!(resolve_ayanamsa("KP"), Some(Ayanamsa::Krishnamurti));
    assert_eq!(
        resolve_ayanamsa("Krishnamurti (Swiss)"),
        Some(Ayanamsa::Krishnamurti)
    );
    assert_eq!(
        resolve_ayanamsa("Krishnamurti Paddhati"),
        Some(Ayanamsa::Krishnamurti)
    );
    assert_eq!(
        resolve_ayanamsa("Krishnamurti Ayanamsa"),
        Some(Ayanamsa::Krishnamurti)
    );
    assert_eq!(
        resolve_ayanamsa("Krishnamurti ayanamsa"),
        Some(Ayanamsa::Krishnamurti)
    );
    assert_eq!(
        resolve_ayanamsa("KP ayanamsa"),
        Some(Ayanamsa::Krishnamurti)
    );
    assert_eq!(
        resolve_ayanamsa("fagan-bradley"),
        Some(Ayanamsa::FaganBradley)
    );
    assert_eq!(
        resolve_ayanamsa("Fagan / Bradley"),
        Some(Ayanamsa::FaganBradley)
    );
    assert_eq!(resolve_ayanamsa("Chitra Paksha"), Some(Ayanamsa::Lahiri));
    assert_eq!(resolve_ayanamsa("Chitra-paksha"), Some(Ayanamsa::Lahiri));
    assert_eq!(resolve_ayanamsa("chitrapaksha"), Some(Ayanamsa::Lahiri));
    assert_eq!(resolve_ayanamsa("Lahiri Ayanamsha"), Some(Ayanamsa::Lahiri));
    assert_eq!(resolve_ayanamsa("Lahiri ayanamsa"), Some(Ayanamsa::Lahiri));
    assert_eq!(resolve_ayanamsa("B.V. Raman"), Some(Ayanamsa::Raman));
    assert_eq!(resolve_ayanamsa("B V Raman"), Some(Ayanamsa::Raman));
    assert_eq!(resolve_ayanamsa("Raman Ayanamsha"), Some(Ayanamsa::Raman));
    assert_eq!(resolve_ayanamsa("Raman ayanamsa"), Some(Ayanamsa::Raman));
    assert_eq!(resolve_ayanamsa("J2000.0"), Some(Ayanamsa::J2000));
    assert_eq!(resolve_ayanamsa("J1900.0"), Some(Ayanamsa::J1900));
    assert_eq!(resolve_ayanamsa("B1950.0"), Some(Ayanamsa::B1950));
    assert_eq!(resolve_ayanamsa("True Revati"), Some(Ayanamsa::TrueRevati));
    assert_eq!(resolve_ayanamsa("True Mula"), Some(Ayanamsa::TrueMula));
    assert_eq!(resolve_ayanamsa("True Citra"), Some(Ayanamsa::TrueCitra));
    assert_eq!(
        resolve_ayanamsa("True Chitra ayanamsa"),
        Some(Ayanamsa::TrueChitra)
    );
    assert_eq!(
        resolve_ayanamsa("True Citra ayanamsa"),
        Some(Ayanamsa::TrueCitra)
    );
    assert_eq!(
        resolve_ayanamsa("True Citra Paksha"),
        Some(Ayanamsa::TrueCitra)
    );
    assert_eq!(
        resolve_ayanamsa("True Chitra Paksha"),
        Some(Ayanamsa::TrueCitra)
    );
    assert_eq!(
        resolve_ayanamsa("True Chitrapaksha"),
        Some(Ayanamsa::TrueCitra)
    );
    assert_eq!(
        resolve_ayanamsa("SS Revati"),
        Some(Ayanamsa::SuryasiddhantaRevati)
    );
    assert_eq!(
        resolve_ayanamsa("SS Citra"),
        Some(Ayanamsa::SuryasiddhantaCitra)
    );
    assert_eq!(resolve_ayanamsa("ICRC Lahiri"), Some(Ayanamsa::LahiriIcrc));
    assert_eq!(
        resolve_ayanamsa("Panchanga Darpan Lahiri"),
        Some(Ayanamsa::Lahiri1940)
    );
    assert_eq!(resolve_ayanamsa("Revati"), Some(Ayanamsa::UshaShashi));
    assert_eq!(
        resolve_ayanamsa("Usha Shashi ayanamsa"),
        Some(Ayanamsa::UshaShashi)
    );
    assert_eq!(resolve_ayanamsa("Moon"), Some(Ayanamsa::ValensMoon));
    assert_eq!(resolve_ayanamsa("Aryabhata"), Some(Ayanamsa::Aryabhata499));
    assert_eq!(
        resolve_ayanamsa("Aryabhata 499"),
        Some(Ayanamsa::Aryabhata499)
    );
    assert_eq!(
        resolve_ayanamsa("Aryabhata 499 CE"),
        Some(Ayanamsa::Aryabhata499)
    );
    assert_eq!(
        resolve_ayanamsa("Aryabhata Kaliyuga"),
        Some(Ayanamsa::Aryabhata499)
    );
    assert_eq!(
        resolve_ayanamsa("Suryasiddhanta 499"),
        Some(Ayanamsa::Suryasiddhanta499)
    );
    assert_eq!(
        resolve_ayanamsa("Surya Siddhanta 499"),
        Some(Ayanamsa::Suryasiddhanta499)
    );
    assert_eq!(
        resolve_ayanamsa("Suryasiddhanta 499 CE"),
        Some(Ayanamsa::Suryasiddhanta499)
    );
    assert_eq!(
        resolve_ayanamsa("Surya Siddhanta 499 CE"),
        Some(Ayanamsa::Suryasiddhanta499)
    );
    assert_eq!(resolve_ayanamsa("Zij al-Shah"), Some(Ayanamsa::Sassanian));
    assert_eq!(resolve_ayanamsa("Sasanian"), Some(Ayanamsa::Sassanian));
    assert_eq!(resolve_ayanamsa("De Luce"), Some(Ayanamsa::DeLuce));
    assert_eq!(resolve_ayanamsa("Yukteswar"), Some(Ayanamsa::Yukteshwar));
    assert_eq!(
        resolve_ayanamsa("Yukteshwar ayanamsa"),
        Some(Ayanamsa::Yukteshwar)
    );
    assert_eq!(
        resolve_ayanamsa("Sri Yukteshwar"),
        Some(Ayanamsa::Yukteshwar)
    );
    assert_eq!(
        resolve_ayanamsa("Shri Yukteswar"),
        Some(Ayanamsa::Yukteshwar)
    );
    assert_eq!(
        resolve_ayanamsa("Shri Yukteshwar"),
        Some(Ayanamsa::Yukteshwar)
    );
    assert_eq!(
        resolve_ayanamsa("P.V.R. Narasimha Rao"),
        Some(Ayanamsa::PvrPushyaPaksha)
    );
    assert_eq!(
        resolve_ayanamsa("True Pushya (PVRN Rao)"),
        Some(Ayanamsa::PvrPushyaPaksha)
    );
    assert_eq!(
        resolve_ayanamsa("PVR Pushya Paksha"),
        Some(Ayanamsa::PvrPushyaPaksha)
    );
    assert_eq!(
        resolve_ayanamsa("Pushya-paksha"),
        Some(Ayanamsa::PvrPushyaPaksha)
    );
    assert_eq!(resolve_ayanamsa("Usha/Shashi"), Some(Ayanamsa::UshaShashi));
    assert_eq!(
        resolve_ayanamsa("Usha / Shashi"),
        Some(Ayanamsa::UshaShashi)
    );
    assert_eq!(resolve_ayanamsa("Sunil Sheoran"), Some(Ayanamsa::Sheoran));
    assert_eq!(resolve_ayanamsa("Vedic / Sheoran"), Some(Ayanamsa::Sheoran));
    assert_eq!(
        resolve_ayanamsa("\"Vedic\"/Sheoran"),
        Some(Ayanamsa::Sheoran)
    );
    assert_eq!(resolve_ayanamsa("Hipparchos"), Some(Ayanamsa::Hipparchus));
    assert_eq!(
        resolve_ayanamsa("Babylonian/Kugler 1"),
        Some(Ayanamsa::BabylonianKugler1)
    );
    assert_eq!(
        resolve_ayanamsa("Babylonian/Kugler 2"),
        Some(Ayanamsa::BabylonianKugler2)
    );
    assert_eq!(
        resolve_ayanamsa("Babylonian/Kugler 3"),
        Some(Ayanamsa::BabylonianKugler3)
    );
    assert_eq!(
        resolve_ayanamsa("Babylonian/Huber"),
        Some(Ayanamsa::BabylonianHuber)
    );
    assert_eq!(
        resolve_ayanamsa("Babylonian/Eta Piscium"),
        Some(Ayanamsa::BabylonianEtaPiscium)
    );
    assert_eq!(
        resolve_ayanamsa("Babylonian/Aldebaran = 15 Tau"),
        Some(Ayanamsa::BabylonianAldebaran)
    );
    assert_eq!(
        resolve_ayanamsa("Babylonian/Britton"),
        Some(Ayanamsa::BabylonianBritton)
    );
    assert_eq!(
        resolve_ayanamsa("BABYL_HOUSE"),
        Some(Ayanamsa::BabylonianHouse)
    );
    assert_eq!(
        resolve_ayanamsa("BABYL_SISSY"),
        Some(Ayanamsa::BabylonianSissy)
    );
    assert_eq!(
        resolve_ayanamsa("BABYL_TRUE_GEOC"),
        Some(Ayanamsa::BabylonianTrueGeoc)
    );
    assert_eq!(
        resolve_ayanamsa("BABYL_TRUE_TOPC"),
        Some(Ayanamsa::BabylonianTrueTopc)
    );
    assert_eq!(
        resolve_ayanamsa("BABYL_TRUE_OBS"),
        Some(Ayanamsa::BabylonianTrueObs)
    );
    assert_eq!(
        resolve_ayanamsa("BABYL_HOUSE_OBS"),
        Some(Ayanamsa::BabylonianHouseObs)
    );
    assert_eq!(
        resolve_ayanamsa("Galact. Center = 0 Sag"),
        Some(Ayanamsa::GalacticCenter)
    );
    assert_eq!(
        resolve_ayanamsa("Cochrane (Gal.Center = 0 Cap)"),
        Some(Ayanamsa::GalacticCenterCochrane)
    );
    assert_eq!(
        resolve_ayanamsa("David Cochrane"),
        Some(Ayanamsa::GalacticCenterCochrane)
    );
    assert_eq!(
        resolve_ayanamsa("Galactic Center (Gil Brand)"),
        Some(Ayanamsa::GalacticCenterRgilbrand)
    );
    assert_eq!(
        resolve_ayanamsa("Galactic Center (Rgilbrand)"),
        Some(Ayanamsa::GalacticCenterRgilbrand)
    );
    assert_eq!(
        resolve_ayanamsa("Galactic center"),
        Some(Ayanamsa::GalacticCenter)
    );
    assert_eq!(
        resolve_ayanamsa("Galactic center Rgilbrand"),
        Some(Ayanamsa::GalacticCenterRgilbrand)
    );
    assert_eq!(
        resolve_ayanamsa("Galactic center Mardyks"),
        Some(Ayanamsa::GalacticCenterMardyks)
    );
    assert_eq!(
        resolve_ayanamsa("Galactic center Mula/Wilhelm"),
        Some(Ayanamsa::GalacticCenterMulaWilhelm)
    );
    assert_eq!(
        resolve_ayanamsa("Galactic center Cochrane"),
        Some(Ayanamsa::GalacticCenterCochrane)
    );
    assert_eq!(
        resolve_ayanamsa("Skydram"),
        Some(Ayanamsa::GalacticCenterMardyks)
    );
    assert_eq!(
        resolve_ayanamsa("Skydram/Galactic Alignment"),
        Some(Ayanamsa::GalacticCenterMardyks)
    );
    assert_eq!(
        resolve_ayanamsa("Skydram (Mardyks)"),
        Some(Ayanamsa::GalacticCenterMardyks)
    );
    assert_eq!(
        resolve_ayanamsa("Mula Wilhelm"),
        Some(Ayanamsa::GalacticCenterMulaWilhelm)
    );
    assert_eq!(
        resolve_ayanamsa("Wilhelm"),
        Some(Ayanamsa::GalacticCenterMulaWilhelm)
    );
    assert_eq!(
        resolve_ayanamsa("True Mula (Chandra Hari)"),
        Some(Ayanamsa::TrueMula)
    );
    assert_eq!(
        resolve_ayanamsa("Dhruva/Gal.Center/Mula (Wilhelm)"),
        Some(Ayanamsa::GalacticCenterMulaWilhelm)
    );
    assert_eq!(
        resolve_ayanamsa("Gal. Eq."),
        Some(Ayanamsa::GalacticEquator)
    );
    assert_eq!(
        resolve_ayanamsa("Galactic Equator (IAU1958)"),
        Some(Ayanamsa::GalacticEquatorIau1958)
    );
    assert_eq!(
        resolve_ayanamsa("Galactic Equator mid-Mula"),
        Some(Ayanamsa::GalacticEquatorMula)
    );
    assert_eq!(
        resolve_ayanamsa("Nick Anthony Fiorenza"),
        Some(Ayanamsa::GalacticEquatorFiorenza)
    );
    assert_eq!(resolve_ayanamsa("True Pushya"), Some(Ayanamsa::TruePushya));
    assert_eq!(resolve_ayanamsa("Udayagiri"), Some(Ayanamsa::Udayagiri));
    assert_eq!(resolve_ayanamsa("Djwhal"), Some(Ayanamsa::DjwhalKhul));
    assert_eq!(resolve_ayanamsa("J.N. Bhasin"), Some(Ayanamsa::JnBhasin));
    assert_eq!(resolve_ayanamsa("Bhasin"), Some(Ayanamsa::JnBhasin));
    assert_eq!(
        resolve_ayanamsa("Suryasiddhanta, mean Sun"),
        Some(Ayanamsa::Suryasiddhanta499MeanSun)
    );
    assert_eq!(
        resolve_ayanamsa("Surya Siddhanta, mean Sun"),
        Some(Ayanamsa::Suryasiddhanta499MeanSun)
    );
    assert_eq!(
        resolve_ayanamsa("Surya Siddhanta mean sun"),
        Some(Ayanamsa::Suryasiddhanta499MeanSun)
    );
    assert_eq!(
        resolve_ayanamsa("Aryabhata, mean Sun"),
        Some(Ayanamsa::Aryabhata499MeanSun)
    );
    assert_eq!(
        resolve_ayanamsa("Aryabhata 522"),
        Some(Ayanamsa::Aryabhata522)
    );
    assert_eq!(
        resolve_ayanamsa("Aryabhata 522 CE"),
        Some(Ayanamsa::Aryabhata522)
    );
    assert_eq!(resolve_ayanamsa("VP285"), Some(Ayanamsa::LahiriVP285));
    assert_eq!(resolve_ayanamsa("VP291"), Some(Ayanamsa::KrishnamurtiVP291));
    assert_eq!(
        resolve_ayanamsa("Krishnamurti-Senthilathiban"),
        Some(Ayanamsa::KrishnamurtiVP291)
    );
    assert_eq!(
        resolve_ayanamsa("Vettius Valens"),
        Some(Ayanamsa::ValensMoon)
    );
    assert_eq!(resolve_ayanamsa("Valens"), Some(Ayanamsa::ValensMoon));
    assert_eq!(
        resolve_ayanamsa("Moon sign ayanamsa"),
        Some(Ayanamsa::ValensMoon)
    );
    assert_eq!(resolve_ayanamsa("Moon sign"), Some(Ayanamsa::ValensMoon));
    assert_eq!(
        resolve_ayanamsa("Valens Moon ayanamsa"),
        Some(Ayanamsa::ValensMoon)
    );
    assert_eq!(
        resolve_ayanamsa("True Sheoran"),
        Some(Ayanamsa::TrueSheoran)
    );
}

#[test]
fn sidereal_offset_is_available_for_baseline_ayanamsas() {
    let lahiri = descriptor(&Ayanamsa::Lahiri).expect("Lahiri descriptor");
    assert_eq!(lahiri.epoch, Some(JulianDay::from_days(2_435_553.5)));
    assert_eq!(
        lahiri.offset_degrees,
        Some(Angle::from_degrees(23.245_524_743))
    );

    let instant = Instant::new(JulianDay::from_days(2_435_553.5), TimeScale::Tt);
    let offset = sidereal_offset(&Ayanamsa::Lahiri, instant).expect("offset should exist");
    // At its reference epoch the corrected (precession-drift) value equals the
    // documented anchor to within the gate ceiling; tight residual is checked by
    // validate-ayanamsa against the SE corpus.
    assert!(
        (offset.degrees() - 23.245_524_743).abs() < 0.01,
        "Lahiri at epoch should be ~23.2455°, got {}",
        offset.degrees()
    );
}

#[test]
fn metadata_coverage_reports_remaining_gaps() {
    let coverage = metadata_coverage();
    let expected_custom_definition_only: Vec<_> = [
        "Babylonian (House)",
        "Babylonian (Sissy)",
        "Babylonian (True Geoc)",
        "Babylonian (True Topc)",
        "Babylonian (True Obs)",
        "Babylonian (House Obs)",
    ]
    .into_iter()
    .collect();
    let expected_without: Vec<_> = built_in_ayanamsas()
        .iter()
        .filter(|entry| {
            !entry.has_sidereal_metadata()
                && !crate::lookup::is_custom_definition_only_ayanamsa(entry.canonical_name)
        })
        .map(|entry| entry.canonical_name)
        .collect();

    assert_eq!(coverage.total, built_in_ayanamsas().len());
    assert_eq!(
        coverage.with_sidereal_metadata
            + coverage.custom_definition_only.len()
            + coverage.without_sidereal_metadata.len(),
        coverage.total
    );
    assert_eq!(
        coverage.custom_definition_only,
        expected_custom_definition_only
    );
    assert_eq!(coverage.without_sidereal_metadata, expected_without);
    assert_eq!(
        coverage.summary_line(),
        format!(
            "ayanamsa sidereal metadata: {}/{} entries with both a reference epoch and offset; custom-definition-only={} labels: {}; missing-sidereal-metadata=none",
            coverage.with_sidereal_metadata,
            coverage.total,
            coverage.custom_definition_only.len(),
            coverage.custom_definition_only.join(", "),
        )
    );
    assert_eq!(coverage.to_string(), coverage.summary_line());
    assert_eq!(
        coverage.validated_summary_line(),
        Ok(coverage.summary_line())
    );
    assert!(coverage.validate().is_ok());
    assert!(coverage.is_complete());
    assert!(coverage
        .custom_definition_only
        .iter()
        .all(|name| name.starts_with("Babylonian (")));
    assert!(coverage
        .summary_line()
        .contains("custom-definition-only=6 labels"));
    assert!(coverage.without_sidereal_metadata.is_empty());
}

#[test]
fn metadata_coverage_validate_rejects_count_or_label_drift() {
    let mut count_drift = metadata_coverage();
    count_drift.total += 1;

    let count_error = count_drift
        .validate()
        .expect_err("mismatched counts should fail validation");
    assert!(matches!(
        count_error,
        AyanamsaMetadataCoverageValidationError::CountsDoNotSum { .. }
    ));
    assert!(count_drift.summary_line().contains("unavailable"));
    assert!(count_drift.validated_summary_line().is_err());

    let mut label_drift = metadata_coverage();
    label_drift.with_sidereal_metadata = label_drift.total.saturating_sub(2);
    label_drift.custom_definition_only = vec!["Lahiri"];
    label_drift.without_sidereal_metadata = vec!["Babylonian (House)"];

    let label_error = label_drift
        .validate()
        .expect_err("unexpected custom-definition labels should fail validation");
    assert!(matches!(
        label_error,
        AyanamsaMetadataCoverageValidationError::UnexpectedCustomDefinitionLabel {
            label: "Lahiri"
        }
    ));
    assert!(label_drift.summary_line().contains("unavailable"));
    assert!(label_drift.validated_summary_line().is_err());

    let mut order_drift = metadata_coverage();
    order_drift.custom_definition_only.reverse();

    let order_error = order_drift
        .validate()
        .expect_err("reordered custom-definition labels should fail validation");
    assert!(matches!(
        order_error,
        AyanamsaMetadataCoverageValidationError::CustomDefinitionOnlyLabelsDoNotMatch { .. }
    ));
    assert!(order_drift.summary_line().contains("unavailable"));
    assert!(order_drift.validated_summary_line().is_err());

    let mut missing_drift = metadata_coverage();
    missing_drift.with_sidereal_metadata = missing_drift.with_sidereal_metadata.saturating_sub(1);
    missing_drift.without_sidereal_metadata = vec!["Placeholder"];

    let missing_error = missing_drift
        .validate()
        .expect_err("non-empty missing-metadata labels should fail validation");
    assert!(matches!(
        missing_error,
        AyanamsaMetadataCoverageValidationError::WithoutSiderealMetadataLabelsDoNotMatch { .. }
    ));
    assert!(missing_drift.summary_line().contains("unavailable"));
    assert!(missing_drift.validated_summary_line().is_err());
}

#[test]
fn custom_definition_example_ayanamsa_labels_match_the_release_profile() {
    assert_eq!(
        custom_definition_example_ayanamsa_labels(),
        &["True Balarama", "Aphoric", "Takra"]
    );
    assert_eq!(
        custom_definition_ayanamsa_labels(),
        &[
            "Babylonian (House)",
            "Babylonian (Sissy)",
            "Babylonian (True Geoc)",
            "Babylonian (True Topc)",
            "Babylonian (True Obs)",
            "Babylonian (House Obs)",
            "True Balarama",
            "Aphoric",
            "Takra",
        ]
    );
}

#[test]
fn custom_ayanamsa_uses_explicit_epoch_and_offset_metadata() {
    for (name, offset_degrees) in [
        ("True Balarama", 12.5),
        ("Aphoric", -3.25),
        ("Takra", 0.125),
    ] {
        let custom = Ayanamsa::Custom(CustomAyanamsa {
            name: name.to_owned(),
            description: Some("Custom label for a non-built-in sidereal variant".to_owned()),
            epoch: Some(JulianDay::from_days(2_451_545.0)),
            offset_degrees: Some(Angle::from_degrees(offset_degrees)),
        });
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);

        let offset = sidereal_offset(&custom, instant).expect("custom offset should exist");
        assert_eq!(offset, Angle::from_degrees(offset_degrees));
    }
}

#[test]
fn true_chitra_tracks_a_star_not_a_fixed_offset_from_lahiri() {
    // Before the correction, True Chitra == Lahiri at every instant (same epoch/offset).
    // After it, the two differ and True Chitra is non-linear vs a Lahiri-style linear offset.
    let early = Instant::new(JulianDay::from_days(2_415_020.5), TimeScale::Tt);
    let late = Instant::new(JulianDay::from_days(2_488_070.0), TimeScale::Tt);
    let tc_early = sidereal_offset(&Ayanamsa::TrueChitra, early)
        .unwrap()
        .degrees();
    let tc_late = sidereal_offset(&Ayanamsa::TrueChitra, late)
        .unwrap()
        .degrees();
    let lah_early = sidereal_offset(&Ayanamsa::Lahiri, early).unwrap().degrees();
    let lah_late = sidereal_offset(&Ayanamsa::Lahiri, late).unwrap().degrees();
    // Both increase with time (precession), staying in a sane sidereal range.
    assert!(
        tc_late > tc_early && tc_early > 22.0 && tc_late < 26.0,
        "tc {tc_early}..{tc_late}"
    );
    // True Chitra and Lahiri are close but NOT identical (true-star vs offset model).
    assert!((tc_early - lah_early).abs() < 0.1 && (tc_late - lah_late).abs() < 0.1);
    // Strict check: they must genuinely differ, not be identical.
    assert!(
        (tc_early - lah_early).abs() > 1.0e-6 || (tc_late - lah_late).abs() > 1.0e-6,
        "TrueChitra and Lahiri must differ after correction, got tc_early={tc_early} lah_early={lah_early} tc_late={tc_late} lah_late={lah_late}"
    );
    // Different model shape: true-star cubic drift differs from offset-defined linear-ish drift.
    let tc_diff = tc_late - tc_early;
    let lah_diff = lah_late - lah_early;
    assert!(
        (tc_diff - lah_diff).abs() > 1.0e-4,
        "TrueChitra and Lahiri should accumulate differently, got tc_diff={tc_diff} lah_diff={lah_diff}"
    );
}

#[test]
fn lahiri_drift_is_nonlinear_after_correction() {
    // Equal time steps forward and backward from epoch give unequal deltas
    // (general precession is non-linear); the old constant-rate model gave equal.
    let epoch = 2_435_553.5;
    let step = 36_525.0;
    let at = |jd: f64| {
        sidereal_offset(
            &Ayanamsa::Lahiri,
            Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
        )
        .unwrap()
        .degrees()
    };
    let fwd = at(epoch + step) - at(epoch);
    let bwd = at(epoch) - at(epoch - step);
    assert!((fwd - bwd).abs() > 1.0e-5, "fwd={fwd} bwd={bwd}");
}
