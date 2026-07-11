use super::*;

#[test]
fn source_body_evidence_summary_matches_the_canonical_body_evidence() {
    let evidence = canonical_epoch_body_evidence().expect("evidence should exist");
    let summary = source_body_evidence_summary().expect("summary should exist");

    assert_eq!(summary.sample_count, evidence.len());
    assert_eq!(
        summary.sample_bodies,
        evidence
            .iter()
            .map(|row| row.body.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(summary.within_interim_limits_count, evidence.len());
    assert_eq!(summary.vendored_full_file_count, 0);
    assert_eq!(summary.generated_binary_count, evidence.len());
    assert_eq!(summary.truncated_count, 0);
    assert_eq!(summary.outside_interim_limit_count, 0);
    assert!(summary.outside_interim_limit_bodies.is_empty());
    assert!(evidence.iter().all(|row| row.within_interim_limits));
}

#[test]
fn source_body_evidence_validated_summary_line_rejects_drift() {
    let mut summary = source_body_evidence_summary().expect("summary should exist");
    summary.sample_count += 1;

    let error = summary
        .validated_summary_line()
        .expect_err("drifted body evidence summary should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body evidence summary field `sample_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_reports_the_measured_classes() {
    let summary = source_body_class_evidence_summary().expect("summary should exist");
    assert_eq!(summary.len(), 2);
    assert_eq!(summary[0].class, Vsop87SourceBodyClass::Luminary);
    assert_eq!(summary[0].sample_count, 1);
    assert_eq!(summary[0].sample_bodies, vec![CelestialBody::Sun]);
    assert_eq!(summary[0].validate(), Ok(()));
    assert_eq!(summary[1].class, Vsop87SourceBodyClass::MajorPlanet);
    assert_eq!(summary[1].sample_count, 7);
    assert_eq!(
        summary[1].sample_bodies,
        vec![
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ]
    );
    assert_eq!(summary[1].validate(), Ok(()));
    assert_eq!(summary[0].summary_line(), summary[0].to_string());
    assert_eq!(summary[1].summary_line(), summary[1].to_string());
    assert_eq!(
        summary[0].validated_summary_line(),
        Ok(summary[0].summary_line())
    );
    assert_eq!(
        summary[1].validated_summary_line(),
        Ok(summary[1].summary_line())
    );
}

#[test]
fn source_body_evidence_summary_validation_rejects_count_drift() {
    let mut summary = source_body_evidence_summary().expect("summary should exist");
    summary.sample_count += 1;

    let error = summary
        .validate()
        .expect_err("count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body evidence summary field `sample_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_evidence_summary_validation_rejects_duplicate_sample_bodies() {
    let mut summary = source_body_evidence_summary().expect("summary should exist");
    let duplicated_body = summary.sample_bodies[0].clone();
    summary.sample_bodies.push(duplicated_body);
    summary.sample_count += 1;
    summary.within_interim_limits_count += 1;
    summary.generated_binary_count += 1;

    let error = summary
        .validate()
        .expect_err("duplicate sample bodies should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body evidence summary field `sample_bodies` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_evidence_summary_validation_rejects_sample_order_drift() {
    let mut summary = source_body_evidence_summary().expect("summary should exist");
    summary.sample_bodies.reverse();

    let error = summary
        .validate()
        .expect_err("sample bodies must preserve canonical order");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body evidence summary field `sample_bodies` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_evidence_summary_validation_rejects_within_limit_count_drift() {
    let mut summary = source_body_evidence_summary().expect("summary should exist");
    summary.within_interim_limits_count += 1;

    let error = summary
        .validate()
        .expect_err("within-limit count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body evidence summary field `within_interim_limits_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_evidence_summary_validation_rejects_outside_limit_count_drift() {
    let mut summary = source_body_evidence_summary().expect("summary should exist");
    summary.outside_interim_limit_count += 1;
    summary.outside_interim_limit_bodies = vec![CelestialBody::Moon];

    let error = summary
        .validate()
        .expect_err("outside-limit count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body evidence summary field `outside_interim_limit_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_evidence_summary_validation_rejects_empty_summary() {
    let summary = Vsop87SourceBodyEvidenceSummary {
        sample_count: 0,
        sample_bodies: Vec::new(),
        within_interim_limits_count: 0,
        vendored_full_file_count: 0,
        generated_binary_count: 0,
        truncated_count: 0,
        outside_interim_limit_count: 0,
        outside_interim_limit_bodies: Vec::new(),
    };

    let error = summary
        .validate()
        .expect_err("empty evidence summaries should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body evidence summary field `sample_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_count_drift() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .next()
        .expect("at least one class summary should exist");
    summary.sample_count += 1;

    let error = summary
        .validate()
        .expect_err("count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `sample_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_duplicate_sample_bodies() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.sample_bodies[1] = summary.sample_bodies[0].clone();

    let error = summary
        .validate()
        .expect_err("duplicate sample bodies should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `sample_bodies` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_sample_order_drift() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.sample_bodies.reverse();

    let error = summary
        .validate()
        .expect_err("sample bodies must preserve canonical order");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `sample_bodies` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_within_limit_count_drift() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.within_interim_limits_count += 1;

    let error = summary
        .validate()
        .expect_err("within-limit count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `within_interim_limits_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_outside_limit_count_drift() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.outside_interim_limit_count += 1;
    summary.outside_interim_limit_bodies = vec![CelestialBody::Moon];

    let error = summary
        .validate()
        .expect_err("outside-limit count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `outside_interim_limit_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_blank_peak_source_file() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.max_longitude_delta_source_file = "";

    let error = summary
        .validate()
        .expect_err("blank peak source file should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `max_longitude_delta_source_file` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_source_kind_drift() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.max_longitude_delta_source_kind = Vsop87BodySourceKind::VendoredVsop87b;

    let error = summary
        .validate()
        .expect_err("source-kind drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `max_longitude_delta_source_kind` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_source_file_drift() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.max_latitude_delta_source_file = "VSOP87B.ear";

    let error = summary
        .validate()
        .expect_err("source-file drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 source-backed body-class evidence summary field `max_latitude_delta_source_file` is out of sync with the current canonical evidence"
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_non_finite_metric() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.mean_distance_delta_au = f64::NAN;

    let error = summary
        .validate()
        .expect_err("non-finite metrics should fail validation");
    assert_eq!(
        error,
        Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
            field: "mean_distance_delta_au",
        }
    );
}

#[test]
fn source_body_class_evidence_summary_validation_rejects_metric_order_drift() {
    let mut summary = source_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.median_longitude_delta_deg = summary.percentile_longitude_delta_deg + 1e-9;

    let error = summary
        .validate()
        .expect_err("metric ordering should fail validation");
    assert_eq!(
        error,
        Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
            field: "median_longitude_delta_deg",
        }
    );
}

#[test]
fn canonical_equatorial_body_class_evidence_summary_reports_the_measured_classes() {
    let summary =
        canonical_epoch_equatorial_body_class_evidence_summary().expect("summary should exist");
    assert_eq!(summary.len(), 2);
    assert_eq!(summary[0].class, Vsop87SourceBodyClass::Luminary);
    assert_eq!(summary[0].sample_count, 1);
    assert_eq!(summary[0].sample_bodies, vec![CelestialBody::Sun]);
    assert_eq!(summary[0].validate(), Ok(()));
    assert_eq!(summary[1].class, Vsop87SourceBodyClass::MajorPlanet);
    assert_eq!(summary[1].sample_count, 7);
    assert_eq!(
        summary[1].sample_bodies,
        vec![
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ]
    );
    assert_eq!(summary[1].validate(), Ok(()));
    assert_eq!(summary[0].summary_line(), summary[0].to_string());
    assert_eq!(summary[1].summary_line(), summary[1].to_string());
}

#[test]
fn canonical_equatorial_body_class_evidence_summary_validation_rejects_count_drift() {
    let mut summary = canonical_epoch_equatorial_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .next()
        .expect("at least one class summary should exist");
    summary.sample_count += 1;

    let error = summary
        .validate()
        .expect_err("count drift should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 equatorial body-class evidence summary field `sample_count` is out of sync with the current canonical evidence"
    );
}

#[test]
fn canonical_equatorial_body_class_evidence_summary_validation_rejects_blank_peak_source_file() {
    let mut summary = canonical_epoch_equatorial_body_class_evidence_summary()
        .expect("summary should exist")
        .into_iter()
        .find(|summary| summary.class == Vsop87SourceBodyClass::MajorPlanet)
        .expect("major-planet summary should exist");
    summary.max_distance_delta_source_file = "";

    let error = summary
        .validate()
        .expect_err("blank peak source file should fail validation");
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 equatorial body-class evidence summary field `max_distance_delta_source_file` is blank"
    );
}

#[test]
fn canonical_evidence_summary_has_a_displayable_summary_line() {
    let summary = canonical_epoch_evidence_summary().expect("summary should exist");
    assert_eq!(summary.summary_line(), summary.to_string());
}

#[test]
fn canonical_body_evidence_row_validation_and_summary_line_are_stable() {
    let row = canonical_epoch_body_evidence()
        .expect("evidence should exist")
        .into_iter()
        .next()
        .expect("at least one evidence row should exist");

    assert_eq!(row.summary_line(), row.to_string());
    assert!(row.summary_line().contains("kind="));
    assert!(row.summary_line().contains("source=VSOP87B.ear"));
    assert!(row.summary_line().contains("provenance="));
    assert!(row.summary_line().contains("status within interim limits"));
    assert_eq!(row.validate(), Ok(()));
}

#[test]
fn canonical_body_evidence_validation_rejects_source_file_drift() {
    let mut row = canonical_epoch_body_evidence()
        .expect("evidence should exist")
        .into_iter()
        .next()
        .expect("at least one evidence row should exist");
    row.source_file = "VSOP87B.synthetic";
    let body = row.body.clone();

    let error = row
        .validate()
        .expect_err("source file drift should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalBodyEvidenceValidationError::SourceFileMismatch {
            body,
            expected: "VSOP87B.ear",
            found: "VSOP87B.synthetic",
        }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 source-backed body evidence row for Sun expects source file `VSOP87B.ear` but found `VSOP87B.synthetic`"
    );
}

#[test]
fn canonical_body_evidence_validation_rejects_interim_limit_status_drift() {
    let mut row = canonical_epoch_body_evidence()
        .expect("evidence should exist")
        .into_iter()
        .next()
        .expect("at least one evidence row should exist");
    row.within_interim_limits = !row.within_interim_limits;
    let body = row.body.clone();

    let error = row
        .validate()
        .expect_err("interim-limit status drift should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalBodyEvidenceValidationError::InterimLimitStatusMismatch { body }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 source-backed body evidence row for Sun has a mismatched interim-limit status"
    );
}

#[test]
fn canonical_evidence_summary_validation_rejects_duplicate_bodies() {
    let mut summary = canonical_epoch_evidence_summary().expect("summary should exist");
    let duplicated_body = summary.sample_bodies[0].clone();
    summary.sample_bodies[1] = duplicated_body.clone();

    let error = summary
        .validate()
        .expect_err("duplicate bodies should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::DuplicateBody {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            body: duplicated_body,
        }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 source-backed evidence summary lists body `Sun` more than once"
    );
}

#[test]
fn canonical_evidence_summary_validation_rejects_peak_source_file_drift() {
    let mut summary = canonical_epoch_evidence_summary().expect("summary should exist");
    summary.max_longitude_delta_source_file = "VSOP87B.synthetic";

    let error = summary
        .validate()
        .expect_err("peak source file drift should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            field: "max_longitude_delta_source_file",
        }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 source-backed evidence summary field `max_longitude_delta_source_file` is out of sync with the current canonical evidence"
    );
}

#[test]
fn canonical_evidence_summary_validation_rejects_non_finite_metric() {
    let mut summary = canonical_epoch_evidence_summary().expect("summary should exist");
    summary.mean_distance_delta_au = f64::INFINITY;

    let error = summary
        .validate()
        .expect_err("non-finite metrics should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            field: "mean_distance_delta_au",
        }
    );
}

#[test]
fn canonical_evidence_summary_validation_rejects_metric_order_drift() {
    let mut summary = canonical_epoch_evidence_summary().expect("summary should exist");
    summary.median_longitude_delta_deg = summary.percentile_longitude_delta_deg + 1e-9;

    let error = summary
        .validate()
        .expect_err("metric ordering should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            field: "median_longitude_delta_deg",
        }
    );
}

#[test]
fn canonical_evidence_summary_validation_rejects_body_evidence_drift() {
    let mut summary = canonical_epoch_evidence_summary().expect("summary should exist");
    summary.mean_distance_delta_au += 1e-12;

    let error = summary
        .validate()
        .expect_err("body-evidence drift should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            field: "mean_distance_delta_au",
        }
    );
}

#[test]
fn canonical_equatorial_evidence_summary_validation_rejects_non_finite_metric() {
    let mut summary = canonical_epoch_equatorial_evidence_summary().expect("summary should exist");
    summary.rms_distance_delta_au = f64::NAN;

    let error = summary
        .validate()
        .expect_err("non-finite metrics should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            field: "rms_distance_delta_au",
        }
    );
}

#[test]
fn canonical_equatorial_evidence_summary_validation_rejects_metric_order_drift() {
    let mut summary = canonical_epoch_equatorial_evidence_summary().expect("summary should exist");
    summary.percentile_declination_delta_deg = summary.max_declination_delta_deg + 1e-9;

    let error = summary
        .validate()
        .expect_err("metric ordering should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            field: "percentile_declination_delta_deg",
        }
    );
}

#[test]
fn canonical_equatorial_evidence_summary_validation_rejects_body_evidence_drift() {
    let mut summary = canonical_epoch_equatorial_evidence_summary().expect("summary should exist");
    summary.mean_right_ascension_delta_deg += 1e-12;

    let error = summary
        .validate()
        .expect_err("body-evidence drift should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            field: "mean_right_ascension_delta_deg",
        }
    );
}

#[test]
fn canonical_equatorial_evidence_summary_validation_rejects_peak_source_kind_drift() {
    let mut summary = canonical_epoch_equatorial_evidence_summary().expect("summary should exist");
    summary.max_right_ascension_delta_source_kind = Vsop87BodySourceKind::MeanOrbitalElements;

    let error = summary
        .validate()
        .expect_err("peak source kind drift should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            field: "max_right_ascension_delta_source_kind",
        }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 equatorial companion evidence summary field `max_right_ascension_delta_source_kind` is out of sync with the current canonical evidence"
    );
}

#[test]
fn canonical_j2000_batch_parity_summary_has_a_displayable_summary_line() {
    let summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.sample_count, canonical_epoch_samples().len());
    assert_eq!(
        summary.sample_bodies,
        canonical_epoch_samples()
            .iter()
            .map(|sample| sample.body.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(summary.frame, CoordinateFrame::Ecliptic);
    assert_eq!(summary.reference_epoch.julian_day.days(), J2000);
    assert_eq!(summary.reference_epoch.scale, TimeScale::Tt);
}

#[test]
fn canonical_j2000_batch_parity_requests_preserve_the_source_backed_batch_slice() {
    let requests = canonical_j2000_batch_parity_requests();

    assert_eq!(requests.len(), canonical_epoch_samples().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        source_backed_body_order()
    );
    assert!(requests.iter().all(|request| {
        request.instant.julian_day.days() == J2000
            && request.instant.scale == TimeScale::Tt
            && request.frame == CoordinateFrame::Ecliptic
    }));
}

#[test]
fn canonical_epoch_requests_remain_a_compatibility_alias() {
    assert_eq!(
        canonical_epoch_requests(),
        canonical_j2000_batch_parity_requests()
    );
}

#[test]
fn canonical_epoch_request_corpus_remains_a_compatibility_alias() {
    assert_eq!(canonical_epoch_request_corpus(), canonical_epoch_requests());
}

#[test]
fn canonical_j2000_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        canonical_j2000_batch_parity_request_corpus(),
        canonical_j2000_batch_parity_requests()
    );
}

#[test]
fn canonical_j2000_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        canonical_j2000_request_corpus(),
        canonical_j2000_batch_parity_requests()
    );
}

#[test]
fn canonical_epoch_batch_parity_requests_remain_a_compatibility_alias() {
    assert_eq!(
        canonical_epoch_batch_parity_requests(),
        canonical_j2000_batch_parity_requests()
    );
}

#[test]
fn canonical_epoch_batch_parity_request_corpus_remains_a_compatibility_alias() {
    assert_eq!(
        canonical_epoch_batch_parity_request_corpus(),
        canonical_epoch_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j2000_batch_parity_requests_remain_the_explicit_alias() {
    assert_eq!(
        source_backed_body_j2000_batch_parity_requests(),
        canonical_j2000_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j2000_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        source_backed_body_j2000_batch_parity_request_corpus(),
        source_backed_body_j2000_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j2000_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        source_backed_body_j2000_request_corpus(),
        source_backed_body_j2000_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j2000_ecliptic_batch_parity_requests_preserve_the_source_backed_body_order() {
    let requests = source_backed_body_j2000_ecliptic_batch_parity_request_corpus();

    assert_eq!(requests.len(), source_backed_body_order().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        source_backed_body_order()
    );
    assert!(requests.iter().all(|request| {
        request.instant.julian_day.days() == J2000
            && request.instant.scale == TimeScale::Tt
            && request.frame == CoordinateFrame::Ecliptic
    }));
}

#[test]
fn source_backed_body_j2000_ecliptic_batch_parity_requests_remain_the_explicit_alias() {
    assert_eq!(
        source_backed_body_j2000_ecliptic_batch_parity_requests(),
        source_backed_body_j2000_batch_parity_requests()
    );
    assert_eq!(
        source_backed_body_j2000_ecliptic_batch_parity_requests(),
        source_backed_body_j2000_ecliptic_batch_parity_request_corpus()
    );
}

#[test]
fn source_backed_body_request_corpus_aliases_remain_the_frame_specific_canonical_slices() {
    assert_eq!(
        source_backed_body_j2000_ecliptic_request_corpus(),
        source_backed_body_j2000_ecliptic_batch_parity_requests()
    );
    assert_eq!(
        source_backed_body_j2000_equatorial_request_corpus(),
        source_backed_body_j2000_equatorial_batch_parity_requests()
    );
    assert_eq!(
        source_backed_body_j1900_ecliptic_request_corpus(),
        source_backed_body_j1900_ecliptic_batch_parity_requests()
    );
    assert_eq!(
        source_backed_body_j1900_equatorial_request_corpus(),
        source_backed_body_j1900_equatorial_batch_parity_requests()
    );
}

#[test]
fn canonical_j1900_equatorial_batch_parity_requests_remain_the_explicit_alias() {
    assert_eq!(
        canonical_j1900_equatorial_batch_parity_requests(),
        canonical_j1900_batch_parity_requests()
    );
}

#[test]
fn canonical_j1900_equatorial_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        canonical_j1900_equatorial_batch_parity_request_corpus(),
        canonical_j1900_equatorial_batch_parity_requests()
    );
}

#[test]
fn canonical_j1900_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        canonical_j1900_batch_parity_request_corpus(),
        canonical_j1900_batch_parity_requests()
    );
}

#[test]
fn canonical_j1900_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        canonical_j1900_request_corpus(),
        canonical_j1900_batch_parity_requests()
    );
}

#[test]
fn canonical_j1900_batch_parity_requests_preserve_the_supported_body_order() {
    let requests = canonical_j1900_batch_parity_requests();

    assert_eq!(requests.len(), Vsop87Backend::supported_bodies().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        Vsop87Backend::supported_bodies().to_vec()
    );
    assert!(requests.iter().all(|request| {
        request.instant.julian_day.days() == J1900
            && request.instant.scale == TimeScale::Tdb
            && request.frame == CoordinateFrame::Equatorial
    }));
}

#[test]
fn supported_body_j2000_equatorial_batch_parity_requests_preserve_the_supported_body_order() {
    let requests = supported_body_j2000_equatorial_batch_parity_requests();

    assert_eq!(requests.len(), Vsop87Backend::supported_bodies().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        Vsop87Backend::supported_bodies().to_vec()
    );
    assert!(requests.iter().all(|request| {
        request.instant.julian_day.days() == J2000
            && request.instant.scale == TimeScale::Tdb
            && request.frame == CoordinateFrame::Equatorial
    }));
}

#[test]
fn supported_body_j2000_equatorial_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        supported_body_j2000_equatorial_batch_parity_request_corpus(),
        supported_body_j2000_equatorial_batch_parity_requests()
    );
}

#[test]
fn supported_body_j2000_equatorial_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        supported_body_j2000_equatorial_request_corpus(),
        supported_body_j2000_equatorial_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j2000_equatorial_batch_parity_requests_remain_the_explicit_alias() {
    assert_eq!(
        source_backed_body_j2000_equatorial_batch_parity_requests(),
        supported_body_j2000_equatorial_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j2000_equatorial_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        source_backed_body_j2000_equatorial_batch_parity_request_corpus(),
        source_backed_body_j2000_equatorial_batch_parity_requests()
    );
}

#[test]
fn supported_body_j2000_ecliptic_batch_parity_requests_preserve_the_supported_body_order() {
    let requests = supported_body_j2000_ecliptic_batch_parity_request_corpus();

    assert_eq!(requests.len(), Vsop87Backend::supported_bodies().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        Vsop87Backend::supported_bodies().to_vec()
    );
    assert!(requests.iter().all(|request| {
        request.instant.julian_day.days() == J2000
            && request.instant.scale == TimeScale::Tdb
            && request.frame == CoordinateFrame::Ecliptic
    }));
}

#[test]
fn supported_body_j2000_ecliptic_batch_parity_requests_remain_the_explicit_alias() {
    assert_eq!(
        supported_body_j2000_ecliptic_batch_parity_requests(),
        supported_body_j2000_ecliptic_batch_parity_request_corpus()
    );
}

#[test]
fn supported_body_j2000_ecliptic_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        supported_body_j2000_ecliptic_request_corpus(),
        supported_body_j2000_ecliptic_batch_parity_request_corpus()
    );
}

#[test]
fn supported_body_request_corpus_remains_the_ecliptic_aliases() {
    assert_eq!(
        supported_body_j2000_request_corpus(),
        supported_body_j2000_ecliptic_request_corpus()
    );
    assert_eq!(
        supported_body_j1900_request_corpus(),
        supported_body_j1900_ecliptic_request_corpus()
    );
}

#[test]
fn supported_body_j1900_ecliptic_batch_parity_requests_preserve_the_supported_body_order() {
    let requests = supported_body_j1900_ecliptic_batch_parity_requests();

    assert_eq!(requests.len(), Vsop87Backend::supported_bodies().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        Vsop87Backend::supported_bodies().to_vec()
    );
    assert!(requests.iter().all(|request| {
        request.instant.julian_day.days() == J1900
            && request.instant.scale == TimeScale::Tdb
            && request.frame == CoordinateFrame::Ecliptic
    }));
}

#[test]
fn source_backed_body_j1900_ecliptic_batch_parity_requests_preserve_the_supported_body_order() {
    assert_eq!(
        source_backed_body_j1900_ecliptic_batch_parity_requests(),
        supported_body_j1900_ecliptic_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j1900_ecliptic_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        source_backed_body_j1900_ecliptic_batch_parity_request_corpus(),
        source_backed_body_j1900_ecliptic_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j1900_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        source_backed_body_j1900_request_corpus(),
        source_backed_body_j1900_ecliptic_request_corpus()
    );
}

#[test]
fn source_backed_body_j1900_equatorial_batch_parity_requests_preserve_the_supported_body_order() {
    assert_eq!(
        source_backed_body_j1900_equatorial_batch_parity_requests(),
        supported_body_j1900_equatorial_batch_parity_requests()
    );
}

#[test]
fn source_backed_body_j1900_equatorial_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        source_backed_body_j1900_equatorial_batch_parity_request_corpus(),
        source_backed_body_j1900_equatorial_batch_parity_requests()
    );
}

#[test]
fn supported_body_j1900_ecliptic_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        supported_body_j1900_ecliptic_batch_parity_request_corpus(),
        supported_body_j1900_ecliptic_batch_parity_requests()
    );
}

#[test]
fn supported_body_j1900_ecliptic_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        supported_body_j1900_ecliptic_request_corpus(),
        supported_body_j1900_ecliptic_batch_parity_requests()
    );
}

#[test]
fn supported_body_j1900_equatorial_batch_parity_requests_preserve_the_supported_body_order() {
    let requests = supported_body_j1900_equatorial_batch_parity_requests();

    assert_eq!(requests.len(), Vsop87Backend::supported_bodies().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        Vsop87Backend::supported_bodies().to_vec()
    );
    assert!(requests.iter().all(|request| {
        request.instant.julian_day.days() == J1900
            && request.instant.scale == TimeScale::Tdb
            && request.frame == CoordinateFrame::Equatorial
    }));
}

#[test]
fn supported_body_j1900_equatorial_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        supported_body_j1900_equatorial_batch_parity_request_corpus(),
        supported_body_j1900_equatorial_batch_parity_requests()
    );
}

#[test]
fn supported_body_j1900_equatorial_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        supported_body_j1900_equatorial_request_corpus(),
        supported_body_j1900_equatorial_batch_parity_requests()
    );
}

#[test]
fn supported_body_canonical_batch_parity_summary_matches_the_backend_helpers() {
    let summary = supported_body_canonical_batch_parity_summary()
        .expect("supported-body canonical batch matrix should exist");
    let requests = supported_body_canonical_batch_parity_requests();

    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(
        summary.supported_body_count,
        Vsop87Backend::supported_bodies().len()
    );
    assert_eq!(
        summary.j2000_ecliptic.sample_count,
        summary.supported_body_count
    );
    assert_eq!(
        summary.j2000_equatorial.sample_count,
        summary.supported_body_count
    );
    assert_eq!(
        summary.j1900_ecliptic.sample_count,
        summary.supported_body_count
    );
    assert_eq!(
        summary.j1900_equatorial.sample_count,
        summary.supported_body_count
    );
    assert_eq!(
        summary.j2000_ecliptic.sample_bodies,
        Vsop87Backend::supported_bodies().to_vec()
    );
    assert_eq!(
        summary.j2000_ecliptic.sample_bodies,
        summary.j2000_equatorial.sample_bodies
    );
    assert_eq!(
        summary.j2000_ecliptic.sample_bodies,
        summary.j1900_ecliptic.sample_bodies
    );
    assert_eq!(
        summary.j2000_ecliptic.sample_bodies,
        summary.j1900_equatorial.sample_bodies
    );
    assert_eq!(summary.j2000_ecliptic.frame, CoordinateFrame::Ecliptic);
    assert_eq!(summary.j2000_equatorial.frame, CoordinateFrame::Equatorial);
    assert_eq!(summary.j1900_ecliptic.frame, CoordinateFrame::Ecliptic);
    assert_eq!(summary.j1900_equatorial.frame, CoordinateFrame::Equatorial);
    assert_eq!(requests.len(), summary.supported_body_count * 4);
    assert_eq!(
        requests[..summary.supported_body_count]
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        summary.j2000_ecliptic.sample_bodies
    );
    assert_eq!(
        requests[summary.supported_body_count..summary.supported_body_count * 2]
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        summary.j2000_equatorial.sample_bodies
    );
    assert_eq!(
        requests[summary.supported_body_count * 2..summary.supported_body_count * 3]
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        summary.j1900_ecliptic.sample_bodies
    );
    assert_eq!(
        requests[summary.supported_body_count * 3..]
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        summary.j1900_equatorial.sample_bodies
    );
}

#[test]
fn supported_body_canonical_batch_parity_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        supported_body_canonical_batch_parity_request_corpus(),
        supported_body_canonical_batch_parity_requests()
    );
}

#[test]
fn supported_body_canonical_batch_matrix_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        supported_body_canonical_batch_matrix_request_corpus(),
        supported_body_canonical_batch_parity_request_corpus()
    );
}

#[test]
fn supported_body_canonical_batch_matrix_requests_remain_the_alias() {
    assert_eq!(
        supported_body_canonical_batch_matrix_requests(),
        supported_body_canonical_batch_matrix_request_corpus()
    );
}

#[test]
fn supported_body_canonical_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        supported_body_canonical_request_corpus(),
        supported_body_canonical_batch_matrix_requests()
    );
}

#[test]
fn canonical_mixed_time_scale_batch_parity_requests_preserve_the_canonical_slice() {
    let requests = canonical_mixed_time_scale_batch_parity_requests();

    assert_eq!(requests.len(), canonical_epoch_samples().len());
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        canonical_epoch_samples()
            .iter()
            .map(|sample| sample.body.clone())
            .collect::<Vec<_>>()
    );
    assert!(requests.iter().enumerate().all(|(index, request)| {
        request.instant.julian_day.days() == J2000
            && request.instant.scale
                == if index % 2 == 0 {
                    TimeScale::Tt
                } else {
                    TimeScale::Tdb
                }
            && request.frame == CoordinateFrame::Ecliptic
    }));
}

#[test]
fn canonical_mixed_time_scale_batch_parity_summary_has_a_displayable_summary_line() {
    let summary = canonical_mixed_time_scale_batch_parity_summary()
        .expect("mixed batch summary should exist");
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.sample_count, canonical_epoch_samples().len());
    assert_eq!(
        summary.sample_bodies,
        canonical_epoch_samples()
            .iter()
            .map(|sample| sample.body.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(summary.frame, CoordinateFrame::Ecliptic);
    assert_eq!(summary.reference_epoch.julian_day.days(), J2000);
    assert_eq!(summary.reference_epoch.scale, TimeScale::Tt);
    assert_eq!(summary.tt_request_count, summary.sample_count.div_ceil(2));
    assert_eq!(summary.tdb_request_count, summary.sample_count / 2);
}

#[test]
fn canonical_mixed_time_scale_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        canonical_mixed_time_scale_batch_parity_request_corpus(),
        canonical_mixed_time_scale_batch_parity_requests()
    );
}

#[test]
fn canonical_mixed_tt_tdb_batch_parity_requests_remain_the_explicit_alias() {
    assert_eq!(
        canonical_mixed_tt_tdb_batch_parity_requests(),
        canonical_mixed_time_scale_batch_parity_requests()
    );
}

#[test]
fn canonical_mixed_tt_tdb_batch_parity_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        canonical_mixed_tt_tdb_batch_parity_request_corpus(),
        canonical_mixed_time_scale_batch_parity_requests()
    );
}

#[test]
fn canonical_mixed_time_scale_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        canonical_mixed_time_scale_request_corpus(),
        canonical_mixed_time_scale_batch_parity_requests()
    );
}

#[test]
fn canonical_mixed_tt_tdb_request_corpus_remains_the_plain_alias() {
    assert_eq!(
        canonical_mixed_tt_tdb_request_corpus(),
        canonical_mixed_time_scale_batch_parity_requests()
    );
}

#[test]
fn canonical_j2000_batch_parity_summary_validation_rejects_body_order_drift() {
    let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
    summary.sample_bodies.reverse();

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    );
}

#[test]
fn canonical_j2000_batch_parity_summary_validation_rejects_frame_drift() {
    let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
    summary.frame = CoordinateFrame::Equatorial;

    assert_eq!(
        summary.validate(),
        Err(Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" })
    );
}

#[test]
fn canonical_j1900_batch_parity_summary_has_a_displayable_summary_line() {
    let summary = canonical_j1900_batch_parity_summary().expect("batch summary should exist");
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.sample_count, summary.sample_bodies.len());
    assert_eq!(
        summary.sample_bodies,
        Vsop87Backend::supported_bodies().to_vec()
    );
    assert_eq!(summary.frame, CoordinateFrame::Equatorial);
    assert_eq!(summary.reference_epoch.julian_day.days(), J1900);
    assert_eq!(summary.reference_epoch.scale, TimeScale::Tdb);
    assert_eq!(
        summary.sample_count,
        summary.exact_count
            + summary.interpolated_count
            + summary.approximate_count
            + summary.unknown_count
    );
}

#[test]
fn canonical_batch_parity_summary_validation_rejects_count_drift() {
    let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
    summary.sample_count += 1;

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "sample_count"
            }
        )
    );
}

#[test]
fn canonical_batch_parity_summary_validation_rejects_quality_count_drift() {
    let mut summary = canonical_j1900_batch_parity_summary().expect("batch summary should exist");
    if summary.exact_count > 0 {
        summary.exact_count -= 1;
        summary.unknown_count += 1;
    } else if summary.interpolated_count > 0 {
        summary.interpolated_count -= 1;
        summary.exact_count += 1;
    } else if summary.approximate_count > 0 {
        summary.approximate_count -= 1;
        summary.exact_count += 1;
    } else {
        summary.unknown_count -= 1;
        summary.exact_count += 1;
    }

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "quality_counts"
            }
        )
    );
}

#[test]
fn canonical_j2000_batch_parity_summary_validation_rejects_quality_count_drift() {
    let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
    if summary.exact_count > 0 {
        summary.exact_count -= 1;
        summary.unknown_count += 1;
    } else if summary.interpolated_count > 0 {
        summary.interpolated_count -= 1;
        summary.exact_count += 1;
    } else if summary.approximate_count > 0 {
        summary.approximate_count -= 1;
        summary.exact_count += 1;
    } else {
        summary.unknown_count -= 1;
        summary.exact_count += 1;
    }

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "quality_counts"
            }
        )
    );
}

#[test]
fn canonical_j2000_batch_parity_summary_validation_rejects_reference_epoch_drift() {
    let mut summary = canonical_j2000_batch_parity_summary().expect("batch summary should exist");
    summary.reference_epoch = Instant::new(
        pleiades_types::JulianDay::from_days(J2000 + 1.0),
        TimeScale::Tt,
    );

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "reference_epoch"
            }
        )
    );
}

#[test]
fn canonical_j1900_batch_parity_summary_validation_rejects_reference_epoch_drift() {
    let mut summary = canonical_j1900_batch_parity_summary().expect("batch summary should exist");
    summary.reference_epoch = Instant::new(
        pleiades_types::JulianDay::from_days(J1900 + 1.0),
        TimeScale::Tdb,
    );

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "reference_epoch"
            }
        )
    );
}

#[test]
fn canonical_j1900_batch_parity_summary_validation_rejects_frame_drift() {
    let mut summary = canonical_j1900_batch_parity_summary().expect("batch summary should exist");
    summary.frame = CoordinateFrame::Ecliptic;

    assert_eq!(
        summary.validate(),
        Err(Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" })
    );
}

#[test]
fn canonical_j1900_batch_parity_summary_validation_rejects_body_order_drift() {
    let mut summary = canonical_j1900_batch_parity_summary().expect("batch summary should exist");
    summary.sample_bodies.reverse();

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    );
}

#[test]
fn canonical_evidence_outlier_note_reports_the_current_interim_status() {
    let summary = canonical_epoch_outlier_summary().expect("outlier summary should exist");

    assert_eq!(
        summary.summary_line(),
        "VSOP87 canonical J2000 interim outliers: none"
    );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
}

#[test]
fn canonical_evidence_outlier_summary_validation_rejects_drift() {
    let mut summary = canonical_epoch_outlier_summary().expect("outlier summary should exist");
    summary.outlier_bodies.push(CelestialBody::Sun);

    assert_eq!(
        summary.validate(),
        Err(
            Vsop87CanonicalOutlierSummaryValidationError::FieldOutOfSync {
                field: "outlier_bodies"
            }
        )
    );
}

#[test]
fn frame_treatment_summary_has_a_displayable_summary_line() {
    let summary = frame_treatment_summary_details();

    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        summary.summary_line(),
        "VSOP87 frame treatment: J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform"
    );
    assert_eq!(frame_treatment_summary(), summary.summary_line());
    assert!(summary.summary_line().contains("mean-obliquity transform"));
}

#[test]
fn canonical_equatorial_evidence_summary_has_a_displayable_summary_line() {
    let summary =
        canonical_epoch_equatorial_evidence_summary().expect("equatorial summary should exist");
    assert_eq!(summary.summary_line(), summary.to_string());
}

#[test]
fn canonical_equatorial_evidence_summary_validation_rejects_peak_body_drift() {
    let mut summary =
        canonical_epoch_equatorial_evidence_summary().expect("equatorial summary should exist");
    summary.max_distance_delta_body = CelestialBody::Pluto;

    let error = summary
        .validate()
        .expect_err("peak body drift should fail validation");
    assert_eq!(
        error,
        Vsop87CanonicalEvidenceSummaryValidationError::PeakBodyNotInSamples {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            field: "max_distance_delta_body",
            body: CelestialBody::Pluto,
        }
    );
    assert_eq!(
        error.to_string(),
        "the VSOP87 canonical J2000 equatorial companion evidence summary field `max_distance_delta_body` points at body `Pluto` which is absent from the sample body list"
    );
}

#[test]
fn source_manifest_pairs_bodies_with_source_files_in_release_order() {
    let manifest = source_manifest();
    let expected_manifest = source_specifications()
        .into_iter()
        .map(|spec| (spec.body, spec.source_file))
        .collect::<Vec<_>>();

    assert_eq!(manifest, expected_manifest);
    assert_eq!(
        supported_source_files(),
        expected_manifest
            .iter()
            .map(|(_, source_file)| *source_file)
            .collect::<Vec<_>>()
    );
    assert_eq!(validate_source_manifest(&manifest), Ok(()));
    for (body, source_file) in &manifest {
        assert!(
            checked_in_generated_vsop87b_table_bytes_for_source_file(source_file).is_some(),
            "supported source file {source_file} should have a checked-in generated blob for {body}"
        );
    }
}

#[test]
fn source_manifest_validation_rejects_entry_drift() {
    let mut manifest = source_manifest();
    manifest.swap(0, 1);

    let error = validate_source_manifest(&manifest)
        .expect_err("drifted source manifests should fail validation");

    assert_eq!(
        error.to_string(),
        "the VSOP87 source manifest entry 0 is out of sync with the current source catalog (expected Sun / VSOP87B.ear, got Mercury / VSOP87B.mer)"
    );
}

#[test]
fn source_manifest_validation_rejects_length_drift_with_manifest_details() {
    let mut manifest = source_manifest();
    manifest.pop();

    let error = validate_source_manifest(&manifest)
        .expect_err("truncated source manifests should fail validation");

    assert_eq!(
        error.to_string(),
        "the VSOP87 source manifest length is out of sync with the current source catalog (expected 8 entries [Sun / VSOP87B.ear, Mercury / VSOP87B.mer, Venus / VSOP87B.ven, Mars / VSOP87B.mar, Jupiter / VSOP87B.jup, Saturn / VSOP87B.sat, Uranus / VSOP87B.ura, Neptune / VSOP87B.nep], got 7 entries [Sun / VSOP87B.ear, Mercury / VSOP87B.mer, Venus / VSOP87B.ven, Mars / VSOP87B.mar, Jupiter / VSOP87B.jup, Saturn / VSOP87B.sat, Uranus / VSOP87B.ura])"
    );
}

#[test]
fn regenerated_binary_tables_match_the_checked_in_artifacts() {
    for spec in source_specifications() {
        let regenerated = generated_vsop87b_table_bytes_for_source_file(spec.source_file)
            .expect("source-backed tables should regenerate");
        let expected = checked_in_generated_vsop87b_table_bytes_for_source_file(spec.source_file)
            .expect("supported source files should have a checked-in generated blob");
        assert_eq!(
            regenerated.as_slice(),
            expected,
            "regenerated blob should match {}",
            spec.source_file
        );
    }
}

#[test]
fn checked_in_generated_tables_cover_the_supported_source_file_set() {
    for source_file in supported_source_files() {
        assert!(
            checked_in_generated_vsop87b_table_bytes_for_source_file(source_file).is_some(),
            "supported source file {source_file} should have a checked-in generated blob"
        );
    }
    assert!(checked_in_generated_vsop87b_table_bytes_for_source_file("VSOP87B.plu").is_none());
}

#[test]
fn supported_source_files_are_exposed_for_reproducibility_tooling() {
    assert_eq!(
        supported_source_files(),
        source_documentation_summary().source_files
    );
}

#[test]
fn request_corpus_helper_preserves_body_order_and_defaults() {
    let instant = Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb);
    let bodies = vec![
        CelestialBody::Mars,
        CelestialBody::Sun,
        CelestialBody::Neptune,
    ];
    let requests = requests_for_bodies_at(bodies.clone(), instant, CoordinateFrame::Equatorial);

    assert_eq!(
        requests
            .iter()
            .map(|request| request.body.clone())
            .collect::<Vec<_>>(),
        bodies
    );
    assert!(requests.iter().all(|request| {
        request.instant == instant
            && request.frame == CoordinateFrame::Equatorial
            && request.zodiac_mode == ZodiacMode::Tropical
            && request.apparent == Apparentness::Mean
            && request.observer.is_none()
    }));
}

#[test]
fn source_backed_and_fallback_body_profiles_are_exposed_for_reproducibility_tooling() {
    let source_backed_profiles = source_backed_body_profiles();
    let fallback_profiles = fallback_body_profiles();
    let summary = source_documentation_summary();

    assert_eq!(
        source_backed_profiles.len(),
        summary.source_backed_profile_count
    );
    assert_eq!(fallback_profiles.len(), summary.fallback_profile_count);
    assert_eq!(source_backed_body_order(), summary.source_backed_bodies);
    assert_eq!(
        source_backed_profiles
            .iter()
            .map(|profile| profile.body.clone())
            .collect::<Vec<_>>(),
        summary.source_backed_bodies
    );
    assert_eq!(
        fallback_profiles
            .iter()
            .map(|profile| profile.body.clone())
            .collect::<Vec<_>>(),
        summary.fallback_bodies
    );
    assert!(fallback_profiles
        .iter()
        .all(|profile| profile.kind == Vsop87BodySourceKind::MeanOrbitalElements));
    assert!(source_backed_profiles
        .iter()
        .all(|profile| profile.kind != Vsop87BodySourceKind::MeanOrbitalElements));
    assert_eq!(
        source_backed_profiles.len() + fallback_profiles.len(),
        body_source_profiles().len()
    );
}

#[test]
fn regeneration_helper_reports_unknown_source_files_explicitly() {
    let error = try_generated_vsop87b_table_bytes_for_source_file("VSOP87B.plu")
        .expect_err("unsupported source files should be rejected");

    assert_eq!(
        error,
        Vsop87TableGenerationError::UnknownSourceFile {
            source_file: "VSOP87B.plu".to_string(),
            supported_source_files: vec![
                "VSOP87B.ear",
                "VSOP87B.mer",
                "VSOP87B.ven",
                "VSOP87B.mar",
                "VSOP87B.jup",
                "VSOP87B.sat",
                "VSOP87B.ura",
                "VSOP87B.nep",
            ],
        }
    );
    assert!(error
        .to_string()
        .contains("no vendored VSOP87B source text found for VSOP87B.plu"));
}

#[test]
fn unified_body_catalog_keeps_profiles_specs_and_samples_aligned() {
    let catalog = body_catalog_entries();
    assert_eq!(catalog.len(), Vsop87Backend::supported_bodies().len());

    let source_backed = catalog
        .iter()
        .filter(|entry| {
            matches!(
                entry.source_profile.kind,
                Vsop87BodySourceKind::TruncatedVsop87b
                    | Vsop87BodySourceKind::VendoredVsop87b
                    | Vsop87BodySourceKind::GeneratedBinaryVsop87b
            )
        })
        .count();
    let fallback = catalog
        .iter()
        .filter(|entry| entry.source_profile.kind == Vsop87BodySourceKind::MeanOrbitalElements)
        .count();
    assert_eq!(source_backed, 8);
    assert_eq!(fallback, 1);

    let pluto = catalog
        .iter()
        .find(|entry| entry.source_profile.body == CelestialBody::Pluto)
        .expect("Pluto entry should exist");
    assert!(pluto.source_specification.is_none());
    assert!(pluto.canonical_sample.is_none());

    let sun = catalog
        .iter()
        .find(|entry| entry.source_profile.body == CelestialBody::Sun)
        .expect("Sun entry should exist");
    assert_eq!(
        sun.source_profile.kind,
        Vsop87BodySourceKind::GeneratedBinaryVsop87b
    );
    assert!(sun.source_specification.is_some());
    assert!(sun.canonical_sample.is_some());
}

#[test]
fn signed_longitude_delta_wraps_across_zero_aries() {
    assert_eq!(signed_longitude_delta_degrees(359.5, 0.5), 1.0);
    assert_eq!(signed_longitude_delta_degrees(0.5, 359.5), -1.0);
}
