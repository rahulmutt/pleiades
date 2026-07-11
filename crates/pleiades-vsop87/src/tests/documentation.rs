use super::*;

#[test]
fn canonical_epoch_error_envelope_validation_rejects_out_of_limit_count_drift() {
    let mut summary = canonical_epoch_evidence_summary().expect("summary should exist");
    summary.out_of_limit_count = 1;

    let error = summary
        .validate()
        .expect_err("drifted canonical evidence summaries should fail validation");

    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 source-backed evidence summary field `out_of_limit_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_specifications_document_variant_frames_units_and_range() {
    let specs = source_specifications();
    assert_eq!(specs.len(), 8);
    assert!(validate_source_specifications(&specs).is_ok());
    assert!(specs.iter().all(|spec| spec.variant == "VSOP87B"));
    assert!(specs
        .iter()
        .all(|spec| spec.frame == "J2000 ecliptic/equinox"));
    assert!(specs
        .iter()
        .all(|spec| spec.units == "degrees and astronomical units"));
    assert!(specs
        .iter()
        .any(|spec| spec.reduction.contains("solar reduction")));
    assert!(specs
        .iter()
        .all(|spec| spec.reduction.contains("geocentric")));
    assert!(specs.iter().all(|spec| {
        spec.truncation_policy
            == "generated binary coefficient table derived from vendored full source file"
    }));
    assert!(!specs
        .iter()
        .any(|spec| spec.truncation_policy == "vendored full source file"));
    assert!(specs.iter().all(|spec| spec
        .date_range
        .contains("full public source file; J2000 canonical reference sample")));
    assert!(specs
        .iter()
        .all(|spec| spec.transform_note.contains("mean-obliquity transform")));
    assert!(specs.iter().any(|spec| spec.source_file == "VSOP87B.nep"));
}

#[test]
fn source_specification_validation_rejects_blank_metadata() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    spec.frame = "   ";

    let error = spec
        .validate()
        .expect_err("blank source-specification fields should fail validation");

    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {} has a blank `frame` field",
            spec.body
        )
    );
}

#[test]
fn source_specification_validation_rejects_blank_date_range() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.date_range = "\t";

    let error = spec
        .validate()
        .expect_err("blank source-specification fields should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::BlankField {
            body: body.clone(),
            field: "date_range",
        }
    );
    assert_eq!(
        error.to_string(),
        format!("the VSOP87 source specification for {body} has a blank `date_range` field")
    );
}

#[test]
fn source_specification_validation_rejects_canonical_metadata_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.frame = "J2000 equatorial";

    let error = spec
        .validate()
        .expect_err("canonical source-specification drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "frame",
            expected: "J2000 ecliptic/equinox",
            found: "J2000 equatorial",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `frame` = `J2000 equatorial`, but expected `J2000 ecliptic/equinox`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_variant_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.variant = "VSOP87C";

    let error = spec
        .validate()
        .expect_err("variant drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "variant",
            expected: "VSOP87B",
            found: "VSOP87C",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `variant` = `VSOP87C`, but expected `VSOP87B`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_coordinate_family_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.coordinate_family = "heliocentric rectangular variables";

    let error = spec
        .validate()
        .expect_err("coordinate-family drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "coordinate_family",
            expected: "heliocentric spherical variables",
            found: "heliocentric rectangular variables",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `coordinate_family` = `heliocentric rectangular variables`, but expected `heliocentric spherical variables`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_units_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.units = "radians and astronomical units";

    let error = spec
        .validate()
        .expect_err("units drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "units",
            expected: "degrees and astronomical units",
            found: "radians and astronomical units",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `units` = `radians and astronomical units`, but expected `degrees and astronomical units`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_reduction_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.reduction = "geocentric experimental reduction";

    let error = spec
        .validate()
        .expect_err("reduction drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "reduction",
            expected: "geocentric solar reduction from Earth coefficients",
            found: "geocentric experimental reduction",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `reduction` = `geocentric experimental reduction`, but expected `geocentric solar reduction from Earth coefficients`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_transform_note_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.transform_note = "J2000 equatorial inputs; equatorial coordinates are derived with a mean-obliquity transform";

    let error = spec
        .validate()
        .expect_err("transform note drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "transform_note",
            expected:
                "J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform",
            found:
                "J2000 equatorial inputs; equatorial coordinates are derived with a mean-obliquity transform",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `transform_note` = `J2000 equatorial inputs; equatorial coordinates are derived with a mean-obliquity transform`, but expected `J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_truncation_policy_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.truncation_policy = "vendored full source file";

    let error = spec
        .validate()
        .expect_err("truncation policy drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "truncation_policy",
            expected: "generated binary coefficient table derived from vendored full source file",
            found: "vendored full source file",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `truncation_policy` = `vendored full source file`, but expected `generated binary coefficient table derived from vendored full source file`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_date_range_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.date_range = "full public source file; J2000 reference sample";

    let error = spec
        .validate()
        .expect_err("date range drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "date_range",
            expected: "full public source file; J2000 canonical reference sample",
            found: "full public source file; J2000 reference sample",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `date_range` = `full public source file; J2000 reference sample`, but expected `full public source file; J2000 canonical reference sample`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_unknown_public_source_file() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.source_file = "VSOP87B.synthetic";

    let error = spec
        .validate()
        .expect_err("unknown public source files should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::UnknownSourceFile {
            body: body.clone(),
            source_file: "VSOP87B.synthetic",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} references unknown public source file `VSOP87B.synthetic`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_body_source_drift() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    let body = spec.body.clone();
    spec.source_file = "VSOP87B.mer";

    let error = spec
        .validate()
        .expect_err("source-file drift should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::FieldOutOfSync {
            body: body.clone(),
            field: "source_file",
            expected: "VSOP87B.ear",
            found: "VSOP87B.mer",
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification for {body} has `source_file` = `VSOP87B.mer`, but expected `VSOP87B.ear`"
        )
    );
}

#[test]
fn source_specification_validation_rejects_unknown_body() {
    let mut spec = source_specifications()
        .into_iter()
        .next()
        .expect("expected at least one VSOP87 source specification");
    spec.body = CelestialBody::Pluto;

    let error = spec
        .validate()
        .expect_err("unknown source-backed bodies should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::UnknownBody {
            body: CelestialBody::Pluto,
        }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 source specification for Pluto is no longer backed by the current source catalog"
    );
}

#[test]
fn source_specification_catalog_rejects_duplicate_public_source_files() {
    let mut specs = source_specifications();
    let duplicated_source_file = specs[0].source_file;
    specs[1].source_file = duplicated_source_file;

    let error = validate_source_specifications(&specs)
        .expect_err("duplicate public source files should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::DuplicateSourceFile {
            source_file: duplicated_source_file,
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification catalog lists public source file `{duplicated_source_file}` more than once"
        )
    );
}

#[test]
fn source_specification_catalog_rejects_whitespace_padded_duplicate_public_source_files() {
    let mut specs = source_specifications();
    let duplicated_source_file = specs[0].source_file;
    specs[1].source_file = "  VSOP87B.ear  ";

    let error = validate_source_specifications(&specs)
        .expect_err("whitespace-padded public source files should fail validation");

    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::DuplicateSourceFile {
            source_file: duplicated_source_file,
        }
    );
    assert_eq!(
        error.to_string(),
        format!(
            "the VSOP87 source specification catalog lists public source file `{duplicated_source_file}` more than once"
        )
    );
}

#[test]
fn source_audit_manifest_tracks_all_vendored_inputs() {
    let audits = source_audits();
    let summary = source_audit_summary();

    assert_eq!(audits.len(), 8);
    assert!(audits.iter().all(|audit| audit.validate().is_ok()));
    assert_eq!(audits[0].summary_line(), audits[0].to_string());
    assert_eq!(summary.source_count, 8);
    assert_eq!(summary.source_bodies, source_backed_body_order());
    assert_eq!(
        summary.source_files,
        source_specifications()
            .iter()
            .map(|spec| spec.source_file)
            .collect::<Vec<_>>()
    );
    assert_eq!(summary.vendored_full_file_count, 8);
    assert_eq!(summary.fingerprint_count, 8);
    assert!(summary.total_term_count > 0);
    assert!(summary.max_byte_length > 0);
    assert!(summary.max_line_count > 0);

    let mut fingerprints = audits
        .iter()
        .map(|audit| audit.fingerprint)
        .collect::<Vec<_>>();
    fingerprints.sort_unstable();
    fingerprints.dedup();
    assert_eq!(fingerprints.len(), audits.len());

    let earth = audits
        .iter()
        .find(|audit| audit.body == CelestialBody::Sun)
        .expect("Sun audit should exist");
    assert_eq!(earth.source_file, "VSOP87B.ear");
    assert_eq!(earth.term_count, 2_564);
}

#[test]
fn source_audit_validation_rejects_drifted_fields() {
    let mut audit = source_audits()[0].clone();
    audit.term_count += 1;

    let error = audit
        .validate()
        .expect_err("drifted source audit records should fail validation");
    assert_eq!(
        error.to_string(),
        "source audit record #1 for Sun and source file `VSOP87B.ear` has a stale `term_count` field"
    );
    assert_eq!(
        validate_source_audits(&[audit]),
        Err(Vsop87SourceAuditValidationError::FieldOutOfSync {
            position: 1,
            body: CelestialBody::Sun,
            source_file: "VSOP87B.ear",
            field: "term_count",
        })
    );
}

#[test]
fn source_audit_summary_has_a_displayable_summary_line() {
    let summary = source_audit_summary();
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validate(), Ok(()));
    let rendered = summary.summary_line();
    assert_eq!(summary.validated_summary_line(), Ok(rendered));
}

#[test]
fn source_audit_summary_validate_rejects_drifted_fields() {
    let mut summary = source_audit_summary();
    summary.fingerprint_count += 1;

    let error = summary
        .validate()
        .expect_err("drifted source audit summaries should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source audit summary field `fingerprint_count` is out of sync with the current manifest"
    );
    assert_eq!(
        summary.validated_summary_line(),
        Err(Vsop87SourceAuditSummaryValidationError::FieldOutOfSync {
            field: "fingerprint_count"
        })
    );
}

#[test]
fn generated_binary_audit_manifest_tracks_all_checked_in_blobs() {
    let audits = generated_binary_audits();
    let summary = generated_binary_audit_summary();

    assert_eq!(audits.len(), 8);
    assert_eq!(summary.blob_count, 8);
    assert_eq!(summary.source_file_count, 8);
    assert_eq!(summary.fingerprint_count, 8);
    assert_eq!(summary.source_bodies, source_backed_body_order());
    assert_eq!(
        summary.source_files,
        source_specifications()
            .iter()
            .map(|spec| spec.source_file)
            .collect::<Vec<_>>()
    );
    assert!(summary.total_byte_length > 0);
    assert!(summary.max_byte_length > 0);
    assert_eq!(
        audits
            .iter()
            .map(|audit| audit.source_file)
            .collect::<Vec<_>>(),
        summary.source_files
    );
    for audit in audits.iter() {
        assert_eq!(audit.validate(), Ok(()));
        assert_eq!(
            validate_generated_binary_audits(std::slice::from_ref(audit)),
            Ok(())
        );
    }

    let mut fingerprints = audits
        .iter()
        .map(|audit| audit.fingerprint)
        .collect::<Vec<_>>();
    fingerprints.sort_unstable();
    fingerprints.dedup();
    assert_eq!(fingerprints.len(), audits.len());

    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validate(), Ok(()));
    let rendered = summary.summary_line();
    assert_eq!(summary.validated_summary_line(), Ok(rendered));
}

#[test]
fn generated_binary_audit_summary_validate_rejects_drifted_fields() {
    let mut summary = generated_binary_audit_summary();
    summary.source_file_count += 1;

    let error = summary
        .validate()
        .expect_err("drifted generated blob summaries should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 generated binary audit summary field `source_file_count` is out of sync with the current manifest"
    );
    assert_eq!(
        summary.validated_summary_line(),
        Err(
            Vsop87GeneratedBlobAuditSummaryValidationError::FieldOutOfSync {
                field: "source_file_count"
            }
        )
    );
}

#[test]
fn generated_binary_audit_validation_rejects_body_source_mismatches() {
    let mut audit = generated_binary_audits()[0].clone();
    audit.body = CelestialBody::Mercury;

    let error = audit
        .validate()
        .expect_err("mismatched generated blob audits should fail validation");
    assert_eq!(
        error.to_string(),
        "generated binary audit record #1 uses source file `VSOP87B.ear`, which belongs to Sun rather than Mercury"
    );
    assert_eq!(
        validate_generated_binary_audits(&[audit]),
        Err(
            Vsop87GeneratedBlobAuditValidationError::BodySourceMismatch {
                position: 1,
                body: CelestialBody::Mercury,
                source_file: "VSOP87B.ear",
                expected_body: CelestialBody::Sun,
            }
        )
    );
}

#[test]
fn generated_binary_audit_builder_rejects_missing_checked_in_blob() {
    let error = build_generated_binary_audits_with_lookup(|source_file| {
        if source_file == "VSOP87B.ear" {
            None
        } else {
            Some(&[])
        }
    })
    .expect_err("missing generated blobs should fail manifest construction");

    assert_eq!(
        error,
        Vsop87GeneratedBlobAuditValidationError::MissingGeneratedBlob {
            position: 1,
            body: CelestialBody::Sun,
            source_file: "VSOP87B.ear",
        }
    );
    assert_eq!(
        error.to_string(),
        "generated binary audit record #1 is missing the checked-in blob for Sun at source file `VSOP87B.ear`"
    );
}

#[test]
fn source_documentation_summary_tracks_catalog_counts() {
    let summary = source_documentation_summary();

    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
        summary.validated_summary_line().unwrap(),
        summary.summary_line()
    );
    assert_eq!(summary.source_specification_count, 8);
    assert_eq!(summary.source_backed_profile_count, 8);
    assert_eq!(
        summary.source_backed_bodies,
        vec![
            CelestialBody::Sun,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ]
    );
    assert_eq!(
        summary.source_files,
        vec![
            "VSOP87B.ear",
            "VSOP87B.mer",
            "VSOP87B.ven",
            "VSOP87B.mar",
            "VSOP87B.jup",
            "VSOP87B.sat",
            "VSOP87B.ura",
            "VSOP87B.nep",
        ]
    );
    assert_eq!(
        summary.generated_binary_bodies,
        summary.source_backed_bodies
    );
    assert_eq!(
        summary.generated_binary_bodies,
        vec![
            CelestialBody::Sun,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ]
    );
    assert_eq!(
        summary.generated_binary_bodies,
        vec![
            CelestialBody::Sun,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ]
    );
    assert!(summary.vendored_full_file_bodies.is_empty());
    assert!(summary.truncated_bodies.is_empty());
    assert_eq!(summary.generated_binary_profile_count, 8);
    assert_eq!(summary.vendored_full_file_profile_count, 0);
    assert_eq!(summary.truncated_profile_count, 0);
    assert_eq!(summary.fallback_profile_count, 1);
    assert_eq!(summary.fallback_bodies, vec![CelestialBody::Pluto]);
    assert_eq!(
        summary.date_ranges,
        vec!["full public source file; J2000 canonical reference sample"]
    );
}

#[test]
fn source_specification_summary_is_typed_and_reusable() {
    let specs = source_specifications();
    let first = &specs[0];
    let expected_joined = specs
        .iter()
        .map(format_source_specification)
        .collect::<Vec<_>>()
        .join(", ");

    assert_eq!(first.summary_line(), first.to_string());
    assert_eq!(
        first.validated_summary_line().unwrap(),
        first.summary_line()
    );
    assert_eq!(format_source_specification(first), first.summary_line());
    assert!(first.summary_line().contains("body=Sun"));
    assert!(first.summary_line().contains("file=VSOP87B.ear"));
    assert!(first.summary_line().contains("variant=VSOP87B"));
    assert!(first
        .summary_line()
        .contains("date range=full public source file; J2000 canonical reference sample"));
    assert_eq!(format_source_specifications(&specs), expected_joined);
}

#[test]
fn source_specification_summary_rejects_drifted_metadata() {
    let spec = Vsop87SourceSpecification {
        body: CelestialBody::Sun,
        source_file: "VSOP87B.synthetic",
        variant: "VSOP87B",
        coordinate_family: "heliocentric spherical variables",
        frame: "J2000 ecliptic/equinox",
        units: "degrees and astronomical units",
        reduction: "geocentric solar reduction from Earth coefficients",
        transform_note:
            "J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform",
        truncation_policy: "generated binary coefficient table derived from vendored full source file",
        date_range: "full public source file; J2000 canonical reference sample",
    };

    let error = spec
        .validated_summary_line()
        .expect_err("unknown source files should be rejected");
    assert_eq!(
        error,
        Vsop87SourceSpecificationValidationError::UnknownSourceFile {
            body: CelestialBody::Sun,
            source_file: "VSOP87B.synthetic",
        }
    );
    assert!(format_source_specification(&spec).starts_with(
        "VSOP87 source specification unavailable (the VSOP87 source specification for Sun references unknown public source file `VSOP87B.synthetic`)"
    ));
}

#[test]
fn source_documentation_health_summary_confirms_catalog_partitioning() {
    let documentation_summary = source_documentation_summary();
    let summary = source_documentation_health_summary();

    assert!(summary.consistent);
    assert!(summary.documentation_consistent);
    assert!(summary.issues.is_empty());
    assert_eq!(summary.source_specification_count, 8);
    assert_eq!(summary.source_file_count, 8);
    assert_eq!(
        summary.source_files,
        vec![
            "VSOP87B.ear",
            "VSOP87B.mer",
            "VSOP87B.ven",
            "VSOP87B.mar",
            "VSOP87B.jup",
            "VSOP87B.sat",
            "VSOP87B.ura",
            "VSOP87B.nep",
        ]
    );
    assert_eq!(summary.source_backed_profile_count, 8);
    assert_eq!(summary.body_profile_count, 9);
    assert_eq!(
        summary.generated_binary_bodies,
        vec![
            CelestialBody::Sun,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ]
    );
    assert!(summary.vendored_full_file_bodies.is_empty());
    assert!(summary.truncated_bodies.is_empty());
    assert_eq!(summary.generated_binary_profile_count, 8);
    assert_eq!(summary.vendored_full_file_profile_count, 0);
    assert_eq!(summary.truncated_profile_count, 0);
    assert_eq!(summary.fallback_profile_count, 1);
    assert!(summary.validate().is_ok());
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
        summary.source_backed_bodies,
        vec![
            CelestialBody::Sun,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ]
    );
    assert_eq!(
        summary.source_backed_partition_bodies,
        summary.source_backed_bodies
    );
    assert_eq!(
        source_documentation_partition_bodies(&documentation_summary),
        documentation_summary.source_backed_bodies
    );
    assert_eq!(summary.fallback_bodies, vec![CelestialBody::Pluto]);
}

#[test]
fn source_documentation_health_summary_lists_issues_when_inconsistent() {
    let summary = Vsop87SourceDocumentationHealthSummary {
        consistent: false,
        documentation_consistent: false,
        issues: vec![
            Vsop87SourceDocumentationHealthIssue::SourceSpecificationFileCountMismatch,
            Vsop87SourceDocumentationHealthIssue::DocumentedFieldMismatch,
        ],
        source_specification_count: 1,
        source_file_count: 2,
        source_files: vec!["VSOP87B.ear"],
        source_backed_profile_count: 1,
        source_backed_bodies: vec![CelestialBody::Sun],
        source_backed_partition_bodies: vec![CelestialBody::Sun],
        generated_binary_bodies: vec![CelestialBody::Sun],
        vendored_full_file_bodies: vec![],
        truncated_bodies: vec![],
        body_profile_count: 2,
        generated_binary_profile_count: 1,
        vendored_full_file_profile_count: 0,
        truncated_profile_count: 0,
        fallback_profile_count: 1,
        fallback_bodies: vec![CelestialBody::Pluto],
    };

    let error = summary
        .validate()
        .expect_err("inconsistent summary should fail validation");
    assert_eq!(error.summary(), &summary);
    assert_eq!(error.summary_line(), summary.summary_line());
    assert_eq!(error.to_string(), summary.summary_line());
}

#[test]
fn source_documentation_health_summary_rejects_partition_order_drift() {
    let mut summary = source_documentation_health_summary();
    assert!(summary.validate().is_ok());

    summary.source_backed_partition_bodies.reverse();

    let error = summary
        .validate()
        .expect_err("partition order drift should fail validation");
    assert_eq!(error.summary(), &summary);
    assert_eq!(error.summary_line(), summary.summary_line());
}

#[test]
fn source_documentation_health_summary_rejects_profile_count_drift() {
    let mut summary = source_documentation_health_summary();
    assert!(summary.validate().is_ok());

    summary.generated_binary_profile_count += 1;

    let error = summary
        .validate()
        .expect_err("profile count drift should fail validation");
    assert_eq!(error.summary(), &summary);
    assert_eq!(error.summary_line(), summary.summary_line());
}

#[test]
fn source_documentation_health_summary_rejects_source_backed_body_duplicates() {
    let mut summary = source_documentation_health_summary();
    assert!(summary.validate().is_ok());

    summary.generated_binary_bodies[1] = CelestialBody::Sun;
    summary.source_backed_partition_bodies = summary
        .generated_binary_bodies
        .iter()
        .chain(summary.vendored_full_file_bodies.iter())
        .chain(summary.truncated_bodies.iter())
        .cloned()
        .collect();
    summary.source_backed_bodies = summary.source_backed_partition_bodies.clone();
    summary.consistent = false;
    summary.issues = vec![Vsop87SourceDocumentationHealthIssue::SourceBackedBodyDuplicate];

    let error = summary
        .validate()
        .expect_err("duplicate source-backed bodies should fail validation");
    assert_eq!(error.summary(), &summary);
    assert_eq!(error.summary_line(), summary.summary_line());
    assert!(error.to_string().contains("source-backed body duplicate"));
}

#[test]
fn source_documentation_health_summary_rejects_source_backed_fallback_overlap() {
    let mut summary = source_documentation_health_summary();
    assert!(summary.validate().is_ok());

    summary.fallback_bodies = vec![CelestialBody::Sun];
    summary.consistent = false;
    summary.issues = vec![Vsop87SourceDocumentationHealthIssue::SourceBackedFallbackBodyOverlap];

    let error = summary
        .validate()
        .expect_err("source-backed/fallback overlap should fail validation");
    assert_eq!(error.summary(), &summary);
    assert_eq!(error.summary_line(), summary.summary_line());
    assert!(error
        .to_string()
        .contains("source-backed/fallback body overlap"));
}

#[test]
fn source_documentation_health_summary_rejects_fallback_body_duplicates() {
    let mut summary = source_documentation_health_summary();
    assert!(summary.validate().is_ok());

    summary.fallback_bodies = vec![CelestialBody::Pluto, CelestialBody::Pluto];
    summary.fallback_profile_count = summary.fallback_bodies.len();
    summary.body_profile_count =
        summary.source_backed_profile_count + summary.fallback_profile_count;
    summary.consistent = false;
    summary.issues = vec![Vsop87SourceDocumentationHealthIssue::FallbackBodyDuplicate];

    let error = summary
        .validate()
        .expect_err("duplicate fallback bodies should fail validation");
    assert_eq!(error.summary(), &summary);
    assert_eq!(error.summary_line(), summary.summary_line());
    assert!(error.to_string().contains("fallback body duplicate"));
}

#[test]
fn source_documentation_health_issue_labels_are_stable() {
    assert_eq!(
        Vsop87SourceDocumentationHealthIssue::SourceSpecificationFileCountMismatch.to_string(),
        "source specification/file count mismatch"
    );
    assert_eq!(
        Vsop87SourceDocumentationHealthIssue::SourceBackedBodyDuplicate.to_string(),
        "source-backed body duplicate"
    );
    assert_eq!(
        Vsop87SourceDocumentationHealthIssue::FallbackBodyDuplicate.to_string(),
        "fallback body duplicate"
    );
    assert_eq!(
        Vsop87SourceDocumentationHealthIssue::SourceBackedFallbackBodyOverlap.to_string(),
        "source-backed/fallback body overlap"
    );
    assert_eq!(
        Vsop87SourceDocumentationHealthIssue::DocumentedFieldMismatch.to_string(),
        "documented field mismatch"
    );
}

#[test]
fn source_documentation_health_issues_detect_partition_order_drift() {
    let mut summary = source_documentation_summary();
    summary.source_backed_bodies.reverse();
    let source_specs = source_specifications();

    let issues = source_documentation_health_issues(
        &summary,
        &source_specs,
        body_catalog_entries().len(),
        summary.source_files.len(),
    );

    assert!(issues.contains(&Vsop87SourceDocumentationHealthIssue::SourceBackedBodyOrderMismatch));
}

#[test]
fn request_policy_summary_tracks_the_public_backend_posture() {
    let policy = vsop87_request_policy();

    assert_eq!(policy.to_string(), policy.summary_line());
    assert_eq!(
        policy.summary_line(),
        "frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
    );
    assert_eq!(
        policy.supported_frames,
        &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial]
    );
    assert_eq!(
        policy.supported_time_scales,
        &[TimeScale::Tt, TimeScale::Tdb]
    );
    assert_eq!(policy.supported_zodiac_modes, &[ZodiacMode::Tropical]);
    assert_eq!(policy.supported_apparentness, &[Apparentness::Mean]);
    assert!(!policy.supports_topocentric_observer);
    assert!(policy.validate().is_ok());
}

#[test]
fn request_policy_summary_validation_rejects_stale_posture() {
    let mut policy = vsop87_request_policy();
    policy.supports_topocentric_observer = true;

    let error = policy
        .validate()
        .expect_err("drifted VSOP87 request-policy summaries should fail validation");

    assert_eq!(
        error,
        Vsop87RequestPolicyValidationError::FieldOutOfSync {
            field: "supports_topocentric_observer"
        }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 request-policy summary field `supports_topocentric_observer` is out of sync with the current posture"
    );
}

#[test]
fn source_kind_display_labels_match_the_release_facing_labels() {
    let cases = [
        (
            Vsop87BodySourceKind::TruncatedVsop87b,
            "truncated VSOP87B slice",
        ),
        (
            Vsop87BodySourceKind::VendoredVsop87b,
            "vendored full-file VSOP87B",
        ),
        (
            Vsop87BodySourceKind::GeneratedBinaryVsop87b,
            "generated binary VSOP87B",
        ),
        (
            Vsop87BodySourceKind::MeanOrbitalElements,
            "mean orbital elements fallback",
        ),
    ];

    for (kind, expected) in cases {
        assert_eq!(kind.label(), expected);
        assert_eq!(kind.to_string(), expected);
    }
}

#[test]
fn source_documentation_summary_has_a_displayable_summary_line() {
    let summary = source_documentation_summary();
    assert_eq!(summary.validate(), Ok(()));
    assert!(source_documentation_health_summary().validate().is_ok());
    assert_eq!(summary.summary_line(), summary.to_string());
}

#[test]
fn source_documentation_summary_validation_rejects_source_file_drift() {
    let mut summary = source_documentation_summary();
    summary.source_specification_count += 1;

    let error = summary
        .validate()
        .expect_err("source specification count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source documentation summary field `source_specification_count` is out of sync with the current source catalog"
    );
}
