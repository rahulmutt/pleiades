//! Descriptor validation guards, metadata helpers, label counting and the
//! exact `Display` text of the catalog's error and summary types.
//!
//! Each guard test builds the smallest descriptor that reaches one branch.

use pleiades_types::{Angle, Ayanamsa, JulianDay};

use crate::lookup::{format_ayanamsa_label_list, validate_ayanamsa_catalog_entries};
use crate::{
    ayanamsa_catalog_validation_summary, built_in_ayanamsas, metadata_coverage,
    AyanamsaCatalogValidationError, AyanamsaCatalogValidationSummary, AyanamsaDescriptor,
    AyanamsaMetadataCoverage, AyanamsaMetadataCoverageValidationError, AyanamsaProvenanceExample,
    AyanamsaProvenanceSummary, AyanamsaProvenanceSummaryValidationError,
};

fn lahiri_with(
    name: &'static str,
    aliases: &'static [&'static str],
    notes: &'static str,
) -> AyanamsaDescriptor {
    AyanamsaDescriptor::new(Ayanamsa::Lahiri, name, aliases, notes, None, None)
}

#[test]
fn validate_rejects_a_canonical_name_with_surrounding_whitespace() {
    // Not blank and no line break: only the surrounding-whitespace test fires.
    assert_eq!(
        lahiri_with(" Lahiri", &[], "note").validate(),
        Err(
            AyanamsaCatalogValidationError::DescriptorLabelNotNormalized {
                label: " Lahiri",
                field: "canonical name",
            }
        )
    );
}

#[test]
fn validate_rejects_notes_with_surrounding_whitespace() {
    // Not blank and no line break: only the padded-notes test fires.
    assert_eq!(
        lahiri_with("Lahiri", &[], " padded note").validate(),
        Err(AyanamsaCatalogValidationError::DescriptorNotesNotNormalized { label: "Lahiri" })
    );
    assert_eq!(
        lahiri_with("Lahiri", &[], "padded note ").validate(),
        Err(AyanamsaCatalogValidationError::DescriptorNotesNotNormalized { label: "Lahiri" })
    );
    assert_eq!(lahiri_with("Lahiri", &[], "clean note").validate(), Ok(()));
}

#[test]
fn validate_rejects_an_alias_identical_to_the_canonical_name() {
    assert_eq!(
        lahiri_with("Lahiri", &["Lahiri"], "note").validate(),
        Err(AyanamsaCatalogValidationError::DescriptorLabelCollision {
            label: "Lahiri",
            canonical_name: "Lahiri",
        })
    );
}

#[test]
fn validate_allows_one_case_variant_of_the_canonical_name_but_not_two() {
    assert_eq!(
        lahiri_with("Lahiri", &["LAHIRI"], "note").validate(),
        Ok(())
    );
    assert_eq!(
        lahiri_with("Lahiri", &["LAHIRI", "lahiri"], "note").validate(),
        Err(AyanamsaCatalogValidationError::DescriptorLabelCollision {
            label: "lahiri",
            canonical_name: "Lahiri",
        })
    );
}

#[test]
fn has_sidereal_metadata_needs_both_epoch_and_offset() {
    let epoch = Some(JulianDay::from_days(2_451_545.0));
    let offset = Some(Angle::from_degrees(1.0));
    let build = |epoch, offset| {
        AyanamsaDescriptor::new(Ayanamsa::Lahiri, "Lahiri", &[], "note", epoch, offset)
    };
    assert!(build(epoch, offset).has_sidereal_metadata());
    assert!(!build(epoch, None).has_sidereal_metadata());
    assert!(!build(None, offset).has_sidereal_metadata());
    assert!(!build(None, None).has_sidereal_metadata());
}

#[test]
fn metadata_coverage_is_complete_only_without_missing_entries() {
    let complete = metadata_coverage();
    assert!(complete.is_complete());

    let incomplete = AyanamsaMetadataCoverage {
        without_sidereal_metadata: vec!["Gap"],
        ..complete
    };
    assert!(!incomplete.is_complete());
}

#[test]
fn format_label_list_renders_none_or_a_comma_separated_list() {
    assert_eq!(format_ayanamsa_label_list(&[]), "none");
    assert_eq!(format_ayanamsa_label_list(&["Alpha"]), "Alpha");
    assert_eq!(
        format_ayanamsa_label_list(&["Alpha", "Beta"]),
        "Alpha, Beta"
    );
}

#[test]
fn entry_validation_counts_every_canonical_name_and_alias() {
    // Lahiri: 1 name + 2 aliases; Raman: 1 name + 1 alias; all resolve to
    // their own built-in entry. 3 + 2 = 5 labels.
    let entries = [
        AyanamsaDescriptor::new(
            Ayanamsa::Lahiri,
            "Lahiri",
            &["Chitra Paksha", "Lahiri Ayanamsha"],
            "note",
            None,
            None,
        ),
        AyanamsaDescriptor::new(
            Ayanamsa::Raman,
            "Raman",
            &["Raman Ayanamsha"],
            "note",
            None,
            None,
        ),
    ];
    assert_eq!(validate_ayanamsa_catalog_entries(&entries), Ok(5));
    assert_eq!(validate_ayanamsa_catalog_entries(&entries[..1]), Ok(3));
}

#[test]
fn built_in_validation_summary_counts_every_label() {
    let expected: usize = built_in_ayanamsas()
        .iter()
        .map(|d| 1 + d.aliases.len())
        .sum();
    let summary = ayanamsa_catalog_validation_summary();
    assert_eq!(summary.label_count, expected);
    assert_eq!(summary.validation_result, Ok(()));
    let text = summary.to_string();
    assert_eq!(text, summary.summary_line());
    assert!(
        text.starts_with(&format!(
            "ayanamsa catalog validation: ok ({} entries, {expected} labels checked;",
            built_in_ayanamsas().len()
        )),
        "{text}"
    );
}

#[test]
fn validation_summary_display_reports_an_error_result() {
    let summary = AyanamsaCatalogValidationSummary {
        entry_count: 3,
        baseline_entry_count: 1,
        release_entry_count: 2,
        label_count: 0,
        metadata_coverage: metadata_coverage(),
        validation_result: Err(AyanamsaCatalogValidationError::EmptyCatalog),
    };
    assert_eq!(
        summary.to_string(),
        "ayanamsa catalog validation: error: catalog is empty (3 entries; baseline=1, release=2)"
    );
}

#[test]
fn provenance_summary_display_joins_examples_with_semicolons() {
    let summary = AyanamsaProvenanceSummary {
        examples: vec![
            AyanamsaProvenanceExample {
                canonical_name: "Alpha",
                provenance_note: "first note",
            },
            AyanamsaProvenanceExample {
                canonical_name: "Beta",
                provenance_note: "second note",
            },
        ],
    };
    assert_eq!(
        summary.to_string(),
        "representative provenance examples: Alpha — first note; Beta — second note"
    );
}

#[test]
fn provenance_validation_error_display_names_the_field() {
    assert_eq!(
        AyanamsaProvenanceSummaryValidationError::FieldOutOfSync { field: "examples" }.to_string(),
        "the ayanamsa provenance summary field `examples` is out of sync with the current posture"
    );
}

#[test]
fn metadata_coverage_error_display_is_exact_for_every_variant() {
    use AyanamsaMetadataCoverageValidationError as E;
    let cases = [
        (
            E::CountsDoNotSum {
                total: 7,
                with_sidereal_metadata: 1,
                custom_definition_only: 2,
                without_sidereal_metadata: 3,
            },
            "coverage counts do not sum to the total (1 + 2 + 3 != 7)",
        ),
        (
            E::UnexpectedCustomDefinitionLabel { label: "X" },
            "label `X` is not a documented custom-definition-only ayanamsa",
        ),
        (
            E::UnexpectedMissingMetadataLabel { label: "Y" },
            "label `Y` is not a documented sidereal-metadata gap",
        ),
        (
            E::CustomDefinitionOnlyLabelsDoNotMatch {
                expected: &["A", "B"],
                actual: vec![],
            },
            "custom-definition-only labels do not match the documented release profile (expected: A, B; actual: none)",
        ),
        (
            E::WithoutSiderealMetadataLabelsDoNotMatch {
                expected: &[],
                actual: vec!["C", "D"],
            },
            "missing sidereal-metadata labels do not match the documented release profile (expected: none; actual: C, D)",
        ),
        (
            E::DuplicateLabel { label: "Z" },
            "label `Z` appears in more than one sidereal-metadata bucket",
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn catalog_error_display_is_exact_for_every_variant() {
    use AyanamsaCatalogValidationError as E;
    let cases = [
        (E::EmptyCatalog, "catalog is empty"),
        (E::DuplicateLabel { label: "Dup" }, "duplicate label 'Dup'"),
        (
            E::LabelDoesNotRoundTrip {
                label: "Lost",
                expected_ayanamsa: Ayanamsa::Lahiri,
            },
            "label 'Lost' should resolve to Lahiri",
        ),
        (
            E::DescriptorLabelCollision {
                label: "a",
                canonical_name: "A",
            },
            "the ayanamsa catalog descriptor label `a` collides with another label on `A`",
        ),
        (
            E::DescriptorLabelNotNormalized {
                label: " a",
                field: "alias",
            },
            "the ayanamsa catalog descriptor alias for ` a` is blank, contains surrounding whitespace, or contains line breaks",
        ),
        (
            E::DescriptorNotesNotNormalized { label: "A" },
            "the ayanamsa catalog descriptor note for `A` is blank, contains surrounding whitespace, or contains line breaks",
        ),
        (
            E::PartialSiderealMetadata { label: "A" },
            "the ayanamsa catalog descriptor for `A` has only one of the reference epoch or offset fields populated",
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}
