//! Tests for the jpl_posture module.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use pleiades_backend::{Apparentness, EphemerisErrorKind, EphemerisRequest};
#[allow(unused_imports)]
use pleiades_backend::{CelestialBody, EphemerisBackend, QualityAnnotation};
#[allow(unused_imports)]
use pleiades_types::CoordinateFrame;

#[test]
fn request_corpus_aliases_preserve_the_current_jpl_batch_shapes() {
    assert_eq!(
        reference_snapshot_request_corpus(CoordinateFrame::Ecliptic),
        reference_snapshot_requests(CoordinateFrame::Ecliptic)
    );
    assert_eq!(
        reference_snapshot_ecliptic_request_corpus(),
        reference_snapshot_requests(CoordinateFrame::Ecliptic)
    );
    assert_eq!(
        reference_snapshot_ecliptic_requests(),
        reference_snapshot_ecliptic_request_corpus()
    );
    assert_eq!(
        reference_snapshot_request_corpus(CoordinateFrame::Equatorial),
        reference_snapshot_requests(CoordinateFrame::Equatorial)
    );
    assert_eq!(
        reference_snapshot_equatorial_parity_requests(),
        reference_snapshot_requests(CoordinateFrame::Equatorial)
    );
    assert_eq!(
        reference_snapshot_equatorial_batch_parity_requests(),
        reference_snapshot_equatorial_parity_requests()
    );
    assert_eq!(
        reference_snapshot_equatorial_batch_parity_request_corpus(),
        reference_snapshot_equatorial_batch_parity_requests()
    );
    assert_eq!(
        reference_snapshot_equatorial_request_corpus(),
        reference_snapshot_equatorial_parity_requests()
    );
    assert_eq!(
        reference_snapshot_equatorial_requests(),
        reference_snapshot_equatorial_request_corpus()
    );
    assert_eq!(
        reference_snapshot_equatorial_parity_request_corpus(),
        reference_snapshot_equatorial_parity_requests()
    );
    assert_eq!(
        reference_snapshot_batch_parity_request_corpus(),
        reference_snapshot_batch_parity_requests()
    );
    assert_eq!(
        reference_snapshot_mixed_time_scale_batch_parity_request_corpus(),
        reference_snapshot_mixed_time_scale_batch_parity_requests()
    );
    assert_eq!(
        reference_snapshot_mixed_tt_tdb_batch_parity_request_corpus(),
        reference_snapshot_mixed_tt_tdb_batch_parity_requests()
    );
    assert_eq!(
        reference_snapshot_mixed_time_scale_request_corpus(),
        reference_snapshot_mixed_time_scale_batch_parity_requests()
    );
    assert_eq!(
        reference_snapshot_mixed_tt_tdb_request_corpus(),
        reference_snapshot_mixed_tt_tdb_batch_parity_requests()
    );
    assert_eq!(
        selected_asteroid_source_request_corpus(CoordinateFrame::Ecliptic),
        selected_asteroid_source_requests(CoordinateFrame::Ecliptic)
    );
    assert_eq!(
        selected_asteroid_source_ecliptic_request_corpus(),
        selected_asteroid_source_requests(CoordinateFrame::Ecliptic)
    );
    assert_eq!(
        selected_asteroid_source_ecliptic_requests(),
        selected_asteroid_source_ecliptic_request_corpus()
    );
    assert_eq!(
        selected_asteroid_source_request_corpus(CoordinateFrame::Equatorial),
        selected_asteroid_source_requests(CoordinateFrame::Equatorial)
    );
    assert_eq!(
        selected_asteroid_source_equatorial_request_corpus(),
        selected_asteroid_source_requests(CoordinateFrame::Equatorial)
    );
    assert_eq!(
        selected_asteroid_source_equatorial_requests(),
        selected_asteroid_source_equatorial_request_corpus()
    );
    assert_eq!(
        selected_asteroid_source_batch_parity_request_corpus(),
        selected_asteroid_source_batch_parity_requests()
    );
    assert_eq!(
        production_generation_boundary_request_corpus(CoordinateFrame::Ecliptic),
        production_generation_boundary_requests(CoordinateFrame::Ecliptic)
    );
    assert_eq!(
        production_generation_boundary_request_corpus(CoordinateFrame::Equatorial),
        production_generation_boundary_requests(CoordinateFrame::Equatorial)
    );
    assert_eq!(
        comparison_snapshot_request_corpus(CoordinateFrame::Ecliptic),
        comparison_snapshot_requests(CoordinateFrame::Ecliptic)
    );
    assert_eq!(
        comparison_snapshot_ecliptic_request_corpus(),
        comparison_snapshot_requests(CoordinateFrame::Ecliptic)
    );
    assert_eq!(
        comparison_snapshot_ecliptic_requests(),
        comparison_snapshot_ecliptic_request_corpus()
    );
    assert_eq!(
        comparison_snapshot_request_corpus(CoordinateFrame::Equatorial),
        comparison_snapshot_requests(CoordinateFrame::Equatorial)
    );
    assert_eq!(
        comparison_snapshot_equatorial_batch_parity_requests(),
        comparison_snapshot_equatorial_parity_requests()
    );
    assert_eq!(
        comparison_snapshot_equatorial_batch_parity_request_corpus(),
        comparison_snapshot_equatorial_batch_parity_requests()
    );
    assert_eq!(
        comparison_snapshot_batch_parity_request_corpus(),
        comparison_snapshot_batch_parity_requests()
    );
    assert_eq!(
        comparison_snapshot_mixed_time_scale_batch_parity_request_corpus(),
        comparison_snapshot_mixed_time_scale_batch_parity_requests()
    );
    assert_eq!(
        comparison_snapshot_mixed_tt_tdb_batch_parity_request_corpus(),
        comparison_snapshot_mixed_tt_tdb_batch_parity_requests()
    );
    assert_eq!(
        comparison_snapshot_mixed_time_scale_request_corpus(),
        comparison_snapshot_mixed_time_scale_batch_parity_requests()
    );
    assert_eq!(
        comparison_snapshot_mixed_tt_tdb_request_corpus(),
        comparison_snapshot_mixed_tt_tdb_batch_parity_requests()
    );
    assert_eq!(
        independent_holdout_snapshot_request_corpus(CoordinateFrame::Ecliptic),
        independent_holdout_snapshot_requests(CoordinateFrame::Ecliptic)
    );
    assert_eq!(
        independent_holdout_snapshot_ecliptic_request_corpus(),
        independent_holdout_snapshot_requests(CoordinateFrame::Ecliptic)
    );
    assert_eq!(
        independent_holdout_snapshot_ecliptic_requests(),
        independent_holdout_snapshot_ecliptic_request_corpus()
    );
    assert_eq!(
        independent_holdout_snapshot_request_corpus(CoordinateFrame::Equatorial),
        independent_holdout_snapshot_requests(CoordinateFrame::Equatorial)
    );
    assert_eq!(
        independent_holdout_snapshot_equatorial_batch_parity_requests(),
        independent_holdout_snapshot_equatorial_parity_requests()
    );
    assert_eq!(
        independent_holdout_snapshot_equatorial_batch_parity_request_corpus(),
        independent_holdout_snapshot_equatorial_batch_parity_requests()
    );
    assert_eq!(
        independent_holdout_snapshot_equatorial_request_corpus(),
        independent_holdout_snapshot_equatorial_parity_requests()
    );
    assert_eq!(
        independent_holdout_snapshot_equatorial_requests(),
        independent_holdout_snapshot_equatorial_request_corpus()
    );
    assert_eq!(
        independent_holdout_snapshot_equatorial_parity_request_corpus(),
        independent_holdout_snapshot_equatorial_parity_requests()
    );
    assert_eq!(
        independent_holdout_snapshot_batch_parity_request_corpus(),
        independent_holdout_snapshot_batch_parity_requests()
    );
    assert_eq!(
        independent_holdout_snapshot_mixed_time_scale_batch_parity_request_corpus(),
        independent_holdout_snapshot_mixed_time_scale_batch_parity_requests()
    );
    assert_eq!(
        independent_holdout_snapshot_mixed_tt_tdb_batch_parity_request_corpus(),
        independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests()
    );
    assert_eq!(
        independent_holdout_snapshot_mixed_time_scale_request_corpus(),
        independent_holdout_snapshot_mixed_time_scale_batch_parity_requests()
    );
    assert_eq!(
        independent_holdout_snapshot_mixed_tt_tdb_request_corpus(),
        independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests()
    );
    assert_eq!(
        reference_asteroid_request_corpus(CoordinateFrame::Ecliptic),
        reference_asteroid_requests(CoordinateFrame::Ecliptic)
    );
    assert_eq!(
        reference_asteroid_request_corpus(CoordinateFrame::Equatorial),
        reference_asteroid_requests(CoordinateFrame::Equatorial)
    );
    assert_eq!(
        reference_asteroid_ecliptic_request_corpus(),
        reference_asteroid_requests(CoordinateFrame::Ecliptic)
    );
    assert_eq!(
        reference_asteroid_equatorial_request_corpus(),
        reference_asteroid_requests(CoordinateFrame::Equatorial)
    );
    assert_eq!(
        reference_asteroid_batch_parity_request_corpus(),
        reference_asteroid_batch_parity_requests()
    );
}

#[test]
fn parsed_manifest_rejects_surrounding_whitespace_in_provenance_fields() {
    let manifest = SnapshotManifest {
        title: Some(" Example snapshot.".to_string()),
        source: Some("Example source".to_string()),
        coverage: None,
        redistribution: None,
        columns: vec!["body".to_string()],
    };

    assert_eq!(
        manifest.validate(),
        Err(SnapshotManifestValidationError::SurroundedByWhitespace { field: "title" })
    );

    let manifest = SnapshotManifest {
        title: Some("Example snapshot.".to_string()),
        source: Some(" Example source".to_string()),
        coverage: None,
        redistribution: None,
        columns: vec!["body".to_string()],
    };

    assert_eq!(
        manifest.validate(),
        Err(SnapshotManifestValidationError::SurroundedByWhitespace { field: "source" })
    );

    let manifest = SnapshotManifest {
        title: Some("Example snapshot.".to_string()),
        source: Some("Example source".to_string()),
        coverage: Some(" Coverage".to_string()),
        redistribution: None,
        columns: vec!["body".to_string()],
    };

    assert_eq!(
        manifest.validate(),
        Err(SnapshotManifestValidationError::SurroundedByWhitespace { field: "coverage" })
    );
}

#[test]
fn checked_in_snapshot_schema_summary_for_report_reports_the_shared_schema() {
    assert_eq!(
        checked_in_snapshot_schema_summary_for_report(),
        "Checked-in snapshot schema: epoch_jd, body, x_km, y_km, z_km"
    );
    assert_eq!(
        validated_checked_in_snapshot_schema_summary_for_report(),
        Ok("epoch_jd, body, x_km, y_km, z_km".to_string())
    );
}

#[test]
fn reference_snapshot_source_summary_reports_the_expected_provenance() {
    let summary = reference_snapshot_source_summary();

    assert_eq!(
        summary.source,
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables."
    );
    assert_eq!(
            summary.coverage,
            "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; selected bodies sampled at 1900-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2451915.25 and 2451915.75 for Sun, Moon, Mercury, Venus; major bodies sampled at 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, and 2453000.5; major bodies sampled at 2451915.5 for Sun through Pluto; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2378498.5, 2451910.5 through 2451919.5, with 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 boundary coverage, 2003-12-27, 2132-08-31, 2500-01-01, and 2634167; asteroid:99942-Apophis is now also sampled at 2378498.5 and 2451917.5 to complete the selected-asteroid bridge."
        );
    assert_eq!(summary.evidence_class, REFERENCE_SNAPSHOT_EVIDENCE_CLASS);
    assert_eq!(summary.columns, REFERENCE_SNAPSHOT_COLUMNS);
    assert_eq!(
        summary.redistribution,
        REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK
    );
    assert!(summary.summary_line().contains("evidence class=reference"));
    assert!(summary.summary_line().contains(
        "redistribution=repository-checked regression fixtures, not a broad public corpus."
    ));
    assert_eq!(summary.frame_treatment, REFERENCE_SNAPSHOT_FRAME_TREATMENT);
    assert_eq!(summary.time_scale, REFERENCE_SNAPSHOT_TIME_SCALE);
    assert!(summary.summary_line().contains("time scale=TDB"));
    assert!(summary.summary_line().contains("2132-08-31"));
    assert_eq!(summary.reference_epoch.julian_day.days(), 2_451_545.0);
    assert_eq!(summary.checksum, reference_snapshot_source_checksum());
    assert!(summary
        .summary_line()
        .contains(&format!("checksum=0x{:016x}", summary.checksum)));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    let mut drifted_summary = summary.clone();
    drifted_summary.reference_epoch = drifted_summary
        .reference_epoch
        .with_time_scale_offset(TimeScale::Tt, 1.0);
    assert_eq!(
        drifted_summary.validate(),
        Err(ReferenceSnapshotSourceSummaryValidationError::ReferenceEpochMismatch)
    );

    let mut drifted_source = summary.clone();
    drifted_source.source =
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables (drift)."
            .to_string();
    assert_eq!(
        drifted_source.validate(),
        Err(ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync { field: "source" })
    );

    let mut drifted_evidence_class = summary.clone();
    drifted_evidence_class.evidence_class = "reference-drift".to_string();
    assert_eq!(
        drifted_evidence_class.validate(),
        Err(
            ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync {
                field: "evidence_class"
            }
        )
    );

    let mut drifted_redistribution = summary.clone();
    drifted_redistribution.redistribution = "fixture redistribution drift".to_string();
    assert_eq!(
        drifted_redistribution.validate(),
        Err(
            ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync {
                field: "redistribution"
            }
        )
    );

    let mut drifted_coverage = summary.clone();
    drifted_coverage.coverage = "major-body coverage drift".to_string();
    assert_eq!(
        drifted_coverage.validate(),
        Err(ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync { field: "coverage" })
    );

    let mut drifted_columns = summary.clone();
    drifted_columns.columns = "epoch_jd, body, x_km, y_km, z_km, extra".to_string();
    assert_eq!(
        drifted_columns.validate(),
        Err(ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync { field: "columns" })
    );

    let mut drifted_frame_treatment = summary.clone();
    drifted_frame_treatment.frame_treatment = "geocentric ecliptic J2000 drift".to_string();
    assert_eq!(
        drifted_frame_treatment.validate(),
        Err(
            ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync {
                field: "frame_treatment"
            }
        )
    );

    let mut drifted_time_scale = summary.clone();
    drifted_time_scale.time_scale = "TT".to_string();
    assert_eq!(
        drifted_time_scale.validate(),
        Err(
            ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync {
                field: "time_scale"
            }
        )
    );
    assert_eq!(
        reference_snapshot_source_summary_for_report(),
        summary.summary_line()
    );

    let body_class_summary = reference_snapshot_body_class_coverage_summary()
        .expect("reference snapshot body-class coverage summary should exist");
    assert_eq!(body_class_summary.major_body_row_count, 182);
    assert_eq!(body_class_summary.major_bodies.len(), 10);
    assert_eq!(body_class_summary.major_epoch_count, 20);
    assert_eq!(body_class_summary.major_windows.len(), 10);
    assert_eq!(body_class_summary.asteroid_row_count, 95);
    assert_eq!(body_class_summary.asteroid_bodies.len(), 6);
    assert_eq!(body_class_summary.asteroid_epoch_count, 17);
    assert_eq!(body_class_summary.asteroid_windows.len(), 6);
    assert_eq!(body_class_summary.validate(), Ok(()));
    assert_eq!(
        body_class_summary.validated_summary_line(),
        Ok(body_class_summary.summary_line())
    );
    assert_eq!(
        reference_snapshot_body_class_coverage_summary_for_report(),
        body_class_summary.summary_line()
    );
    assert!(body_class_summary
            .summary_line()
            .contains("Reference snapshot body-class coverage: major bodies: 182 rows across 10 bodies and 20 epochs; major windows: "));
    assert!(body_class_summary
        .summary_line()
        .contains("selected asteroids: 95 rows across 6 bodies and 17 epochs; asteroid windows: "));

    let window_summary = reference_snapshot_source_window_summary()
        .expect("reference snapshot source window summary should exist");
    assert_eq!(
        window_summary.windows.len(),
        window_summary.sample_bodies.len()
    );
    assert_eq!(
        window_summary.sample_bodies,
        reference_snapshot()
            .iter()
            .map(|entry| entry.body.clone())
            .fold(Vec::new(), |mut bodies, body| {
                if !bodies.contains(&body) {
                    bodies.push(body);
                }
                bodies
            })
    );
    assert_eq!(window_summary.validate(), Ok(()));
    assert_eq!(
        window_summary.validated_summary_line(),
        Ok(window_summary.summary_line())
    );
    assert_eq!(window_summary.to_string(), window_summary.summary_line());
    assert_eq!(
        reference_snapshot_source_window_summary_for_report(),
        window_summary.summary_line()
    );
    assert!(window_summary
        .summary_line()
        .starts_with("Reference snapshot source windows: "));
    assert!(window_summary.summary_line().contains("Moon:"));
    assert!(window_summary.summary_line().contains("Pluto:"));

    let sun_window = &window_summary.windows[6];
    assert_eq!(sun_window.body, pleiades_backend::CelestialBody::Sun);
    assert_eq!(sun_window.sample_count, 20);
    assert_eq!(sun_window.epoch_count, 20);
    assert_eq!(sun_window.earliest_epoch.julian_day.days(), 2_415_020.5);
    assert_eq!(sun_window.latest_epoch.julian_day.days(), 2_453_000.5);

    let jupiter_window = &window_summary.windows[10];
    assert_eq!(
        jupiter_window.body,
        pleiades_backend::CelestialBody::Jupiter
    );
    assert_eq!(jupiter_window.sample_count, 17);
    assert_eq!(jupiter_window.epoch_count, 17);
    assert_eq!(jupiter_window.earliest_epoch.julian_day.days(), 2_451_545.0);
    assert_eq!(jupiter_window.latest_epoch.julian_day.days(), 2_453_000.5);

    let pluto_window = &window_summary.windows[13];
    assert_eq!(pluto_window.body, pleiades_backend::CelestialBody::Pluto);
    assert_eq!(pluto_window.sample_count, 17);
    assert_eq!(pluto_window.epoch_count, 17);
    assert_eq!(pluto_window.earliest_epoch.julian_day.days(), 2_451_545.0);
    assert_eq!(pluto_window.latest_epoch.julian_day.days(), 2_453_000.5);

    let asteroid_window = &window_summary.windows[4];
    assert_eq!(
        asteroid_window.body,
        pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
    );
    assert_eq!(asteroid_window.sample_count, 17);
    assert_eq!(asteroid_window.epoch_count, 17);
    assert_eq!(
        asteroid_window.earliest_epoch.julian_day.days(),
        2_378_498.5
    );
    assert_eq!(asteroid_window.latest_epoch.julian_day.days(), 2_634_167.0);
}

#[test]
fn jpl_snapshot_evidence_summary_combines_the_backend_reports() {
    let report = jpl_snapshot_evidence_summary_for_report();
    let reference_report = reference_snapshot_summary_for_report();
    let holdout_summary = jpl_independent_holdout_summary_for_report();
    let holdout_high_curvature = independent_holdout_high_curvature_summary_for_report();

    assert!(report.contains(&jpl_snapshot_evidence_classification_summary_for_report()));
    assert!(report.contains(&jpl_source_posture_summary_for_report()));
    assert!(report.contains(&jpl_provenance_only_summary_for_report()));
    assert!(report.contains(&reference_snapshot_summary_for_report()));
    assert!(report.contains(&reference_snapshot_body_class_coverage_summary_for_report()));
    assert!(report.contains(&reference_snapshot_equatorial_parity_summary_for_report()));
    assert!(report.contains(&reference_snapshot_source_summary_for_report()));
    assert!(report.contains(&reference_snapshot_source_window_summary_for_report()));
    assert!(report.contains(&reference_snapshot_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_mars_jupiter_boundary_summary_for_report()));
    assert!(report.contains(&reference_asteroid_evidence_summary_for_report()));
    assert!(report.contains(&reference_asteroid_equatorial_evidence_summary_for_report()));
    assert!(report.contains(&reference_asteroid_source_window_summary_for_report()));
    assert!(report.contains(&reference_holdout_overlap_summary_for_report()));
    assert!(report.contains(&independent_holdout_high_curvature_summary_for_report()));
    assert!(report.contains(&reference_snapshot_manifest_summary_for_report()));
    assert!(report.contains(&production_generation_snapshot_summary_for_report()));
    assert!(report.contains(&production_generation_source_summary_for_report()));
    assert!(report.contains(&production_generation_boundary_source_summary_for_report()));
    assert!(report.contains(&production_generation_boundary_window_summary_for_report()));
    assert!(
        report.contains(&production_generation_boundary_body_class_coverage_summary_for_report())
    );
    assert!(report.contains(&production_generation_boundary_request_corpus_summary_for_report()));
    assert!(report
        .contains(&production_generation_boundary_request_corpus_equatorial_summary_for_report()));
    assert!(report.contains(&reference_asteroid_evidence_summary_for_report()));
    assert!(report.contains(&reference_asteroid_equatorial_evidence_summary_for_report()));
    assert!(report.contains(&reference_asteroid_source_window_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_2451917_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_2453000_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_2500000_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_2634167_summary_for_report()));
    assert!(report.contains(&selected_asteroid_boundary_summary_for_report()));
    assert!(report.contains(&selected_asteroid_bridge_summary_for_report()));
    assert!(report.contains(&selected_asteroid_dense_boundary_summary_for_report()));
    assert!(report.contains(&selected_asteroid_terminal_boundary_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_evidence_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_window_summary_for_report()));
    assert!(report.contains(&comparison_snapshot_summary_for_report()));
    assert!(report.contains(&comparison_snapshot_body_class_coverage_summary_for_report()));
    assert!(report.contains(&comparison_snapshot_source_summary_for_report()));
    assert!(report.contains(&comparison_snapshot_source_window_summary_for_report()));
    assert!(report.contains(&comparison_snapshot_manifest_summary_for_report()));
    assert!(report.contains(&independent_holdout_snapshot_summary_for_report()));
    assert!(report.contains(&independent_holdout_snapshot_equatorial_parity_summary_for_report()));
    assert!(report.contains(&independent_holdout_snapshot_batch_parity_summary_for_report()));
    assert!(report.contains(&independent_holdout_source_summary_for_report()));
    assert!(report.contains(&independent_holdout_snapshot_source_window_summary_for_report()));
    assert!(
        report.contains(&independent_holdout_snapshot_quarter_day_boundary_summary_for_report())
    );
    assert!(report.contains(&independent_holdout_manifest_summary_for_report()));
    assert!(report.contains(&holdout_summary));
    assert!(report.contains(&holdout_high_curvature));
    assert!(!reference_report.contains(&holdout_summary));
    assert!(!reference_report.contains(&holdout_high_curvature));
}

#[test]
fn jpl_snapshot_evidence_posture_summaries_validate_and_fail_closed() {
    let classification = jpl_snapshot_evidence_classification_summary_details();
    let posture = jpl_source_posture_summary_details();
    let provenance_only = jpl_provenance_only_summary_details();
    let contract = jpl_source_corpus_contract_summary_details();

    assert_eq!(
        classification.summary_line(),
        jpl_snapshot_evidence_classification_summary_for_report()
    );
    assert_eq!(
        posture.summary_line(),
        jpl_source_posture_summary_for_report()
    );
    assert_eq!(
        provenance_only.summary_line(),
        jpl_provenance_only_summary_for_report()
    );
    assert_eq!(
        contract.summary_line(),
        jpl_source_corpus_contract_summary_for_report()
    );
    assert!(contract.summary_line().contains("reference="));
    assert!(contract.summary_line().contains("hold-out="));
    assert!(contract.summary_line().contains("source windows="));
    assert!(contract.summary_line().contains("source revision="));
    assert!(contract
        .summary_line()
        .contains("boundary request corpora: ecliptic="));
    assert!(contract.summary_line().contains("equatorial="));
    assert_eq!(classification.validate(), Ok(()));
    assert_eq!(posture.validate(), Ok(()));
    assert_eq!(provenance_only.validate(), Ok(()));
    assert_eq!(contract.validate(), Ok(()));

    let drifted_classification = JplSnapshotEvidenceClassificationSummary {
        text: "JPL evidence classification: drifted",
    };
    let drifted_posture = JplSourcePostureSummary {
        text: "JPL source posture: drifted",
    };
    let drifted_provenance_only = JplProvenanceOnlySummary {
        text: "JPL provenance-only evidence: drifted",
    };
    let mut drifted_contract = jpl_source_corpus_contract_summary_details();
    drifted_contract.source_posture = drifted_posture.clone();

    assert!(drifted_classification.validate().is_err());
    assert!(drifted_posture.validate().is_err());
    assert!(drifted_provenance_only.validate().is_err());
    assert!(drifted_contract.validate().is_err());
    assert!(drifted_classification
        .validated_summary_line()
        .expect_err("drifted evidence classification should fail closed")
        .to_string()
        .contains("out of sync"));
    assert!(drifted_posture
        .validated_summary_line()
        .expect_err("drifted source posture should fail closed")
        .to_string()
        .contains("out of sync"));
    assert!(drifted_provenance_only
        .validated_summary_line()
        .expect_err("drifted provenance-only summary should fail closed")
        .to_string()
        .contains("out of sync"));
    assert!(drifted_contract
        .validated_summary_line()
        .expect_err("drifted source corpus contract should fail closed")
        .to_string()
        .contains("out of sync"));
}

#[test]
fn interpolation_quality_samples_are_reportable() {
    let samples = interpolation_quality_samples();
    assert_eq!(samples.len(), 223);
    assert!(samples.iter().all(|sample| {
        let epoch = sample.epoch.julian_day.days();
        let summary_line = sample.summary_line();
        (epoch == 2_378_499.0
            || epoch == 2_400_000.0
            || epoch == 2_415_020.5
            || epoch == REFERENCE_EPOCH_JD
            || epoch == 2_451_910.5
            || epoch == 2_451_911.5
            || epoch == 2_451_912.5
            || epoch == 2_451_913.5
            || epoch == 2_451_914.0
            || epoch == 2_451_914.5
            || epoch == 2_451_915.0
            || epoch == 2_451_915.25
            || epoch == 2_451_915.5
            || epoch == 2_451_915.75
            || epoch == 2_451_916.0
            || epoch == 2_451_916.5
            || epoch == 2_451_917.0
            || epoch == 2_451_918.5
            || epoch == 2_451_919.5
            || epoch == 2_451_920.5
            || epoch == 2_305_457.5
            || epoch == 2_360_233.5
            || epoch == 2_360_234.5
            || epoch == 2_453_000.5
            || epoch == 2_524_593.5
            || epoch == 2_500_000.0
            || epoch == 2_600_000.0)
            && sample.validate().is_ok()
            && summary_line.contains("TDB")
            && summary_line == sample.to_string()
            && sample.bracket_span_days > 0.0
            && sample.longitude_error_deg.is_finite()
            && sample.latitude_error_deg.is_finite()
            && sample.distance_error_au.is_finite()
            && matches!(
                sample.interpolation_kind,
                InterpolationQualityKind::Cubic
                    | InterpolationQualityKind::Quadratic
                    | InterpolationQualityKind::Linear
            )
    }));
    assert!(samples
        .iter()
        .any(|sample| sample.interpolation_kind == InterpolationQualityKind::Cubic));
    assert!(!samples
        .iter()
        .any(|sample| sample.interpolation_kind == InterpolationQualityKind::Quadratic));
    assert!(!samples
        .iter()
        .any(|sample| sample.interpolation_kind == InterpolationQualityKind::Linear));
    assert!(samples
        .iter()
        .any(|sample| sample.epoch.julian_day.days() == 2_451_545.0));
    assert!(samples
        .iter()
        .any(|sample| sample.epoch.julian_day.days() == 2_451_910.5));
    assert!(samples
        .iter()
        .any(|sample| sample.epoch.julian_day.days() == 2_451_911.5));
    assert!(samples
        .iter()
        .any(|sample| sample.epoch.julian_day.days() == 2_451_917.0));
    assert!(samples
        .iter()
        .any(|sample| sample.epoch.julian_day.days() == 2_451_920.5));
    assert!(samples
        .iter()
        .any(|sample| sample.epoch.julian_day.days() == 2_453_000.5));
    assert!(samples
        .iter()
        .any(|sample| sample.epoch.julian_day.days() == 2_500_000.0));
    assert!(samples
        .iter()
        .any(|sample| sample.epoch.julian_day.days() == 2_451_920.5));
    assert!(samples
        .iter()
        .any(|sample| sample.body == pleiades_backend::CelestialBody::Mars));
}

#[test]
fn interpolation_quality_sample_validation_rejects_non_tdb_epochs() {
    let mut sample = interpolation_quality_samples()[0].clone();
    sample.epoch = Instant::new(sample.epoch.julian_day, TimeScale::Tt);

    assert!(matches!(
        sample.validate(),
        Err(InterpolationQualitySampleValidationError::NonTdbEpoch {
            found: TimeScale::Tt,
            ..
        })
    ));
}

#[test]
fn batch_query_preserves_interpolation_quality_samples_and_order() {
    let backend = JplSnapshotBackend;
    let samples = interpolation_quality_samples();
    let requests = interpolation_quality_sample_requests()
        .expect("interpolation-quality sample requests should exist");

    assert_eq!(requests.len(), samples.len());
    for (sample, request) in samples.iter().zip(requests.iter()) {
        assert_eq!(request.body, sample.body);
        assert_eq!(request.instant, sample.epoch);
        assert_eq!(request.frame, CoordinateFrame::Ecliptic);
        assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
        assert_eq!(request.apparent, Apparentness::Mean);
        assert!(request.observer.is_none());
    }

    let results = backend
        .positions(&requests)
        .expect("batch query should resolve the interpolation-quality samples");

    assert_eq!(results.len(), samples.len());
    for (sample, result) in samples.iter().zip(results.iter()) {
        assert_eq!(result.body, sample.body);
        assert_eq!(result.instant, sample.epoch);
        assert_eq!(result.frame, CoordinateFrame::Ecliptic);
        assert_eq!(result.apparent, Apparentness::Mean);
        assert_eq!(result.quality, QualityAnnotation::Exact);
        let ecliptic = result
            .ecliptic
            .expect("batch results should include ecliptic coordinates");
        assert!(ecliptic.longitude.degrees().is_finite());
        assert!(ecliptic.latitude.degrees().is_finite());
        assert!(ecliptic
            .distance_au
            .expect("distance should exist")
            .is_finite());
    }
}

#[test]
fn interpolation_quality_sample_request_corpus_remains_the_explicit_alias() {
    assert_eq!(
        interpolation_quality_sample_request_corpus(),
        interpolation_quality_sample_requests()
    );
}

#[test]
fn interpolation_quality_summary_reports_the_worst_case_labels() {
    let summary = jpl_interpolation_quality_summary().expect("summary should exist");
    assert_eq!(summary.sample_count, 223);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.epoch_count, 19);
    assert!(summary.earliest_epoch.julian_day.days() <= summary.latest_epoch.julian_day.days());
    assert_eq!(
        summary.cubic_sample_count + summary.quadratic_sample_count + summary.linear_sample_count,
        summary.sample_count
    );
    assert!(summary.cubic_sample_count > 0);
    assert_eq!(summary.quadratic_sample_count, 0);
    assert_eq!(summary.linear_sample_count, 0);
    assert!(summary.mean_bracket_span_days.is_finite());
    assert!(summary.median_bracket_span_days.is_finite());
    assert!(summary.percentile_bracket_span_days.is_finite());
    assert!(summary.mean_longitude_error_deg.is_finite());
    assert!(summary.median_longitude_error_deg.is_finite());
    assert!(summary.percentile_longitude_error_deg.is_finite());
    assert!(summary.rms_longitude_error_deg.is_finite());
    assert!(summary.mean_latitude_error_deg.is_finite());
    assert!(summary.median_latitude_error_deg.is_finite());
    assert!(summary.percentile_latitude_error_deg.is_finite());
    assert!(summary.rms_latitude_error_deg.is_finite());
    assert!(summary.mean_distance_error_au.is_finite());
    assert!(summary.median_distance_error_au.is_finite());
    assert!(summary.percentile_distance_error_au.is_finite());
    assert!(summary.rms_distance_error_au.is_finite());
    assert!(!summary.max_bracket_span_body.is_empty());
    assert!(!summary.max_longitude_error_body.is_empty());
    assert!(!summary.max_latitude_error_body.is_empty());
    assert!(!summary.max_distance_error_body.is_empty());

    assert_eq!(summary.to_string(), summary.summary_line());

    let rendered = format_jpl_interpolation_quality_summary(&summary);
    assert!(rendered.contains("cubic"));
    assert!(rendered.contains("quadratic"));
    assert!(rendered.contains("linear"));
    assert!(rendered.contains("223 samples across 16 bodies and 19 epochs"));
    assert!(rendered.contains("epoch window"));
    assert!(rendered.contains("mean bracket span="));
    assert!(rendered.contains("median bracket span="));
    assert!(rendered.contains("p95 bracket span="));
    assert!(rendered.contains("mean Δlon="));
    assert!(rendered.contains("median Δlon="));
    assert!(rendered.contains("p95 Δlon="));
    assert!(rendered.contains("rms Δlon="));
    assert!(rendered.contains("mean Δlat="));
    assert!(rendered.contains("median Δlat="));
    assert!(rendered.contains("p95 Δlat="));
    assert!(rendered.contains("rms Δlat="));
    assert!(rendered.contains("mean Δdist="));
    assert!(rendered.contains("median Δdist="));
    assert!(rendered.contains("p95 Δdist="));
    assert!(rendered.contains("rms Δdist="));
    assert!(rendered.contains(&format!(
        "({} @ {}",
        summary.max_bracket_span_body,
        format_instant(summary.max_bracket_span_epoch)
    )));
    assert!(rendered.contains(&format!(
        "({} @ {}",
        summary.max_longitude_error_body,
        format_instant(summary.max_longitude_error_epoch)
    )));
    assert!(rendered.contains(&format!(
        "({} @ {}",
        summary.max_latitude_error_body,
        format_instant(summary.max_latitude_error_epoch)
    )));
    assert!(rendered.contains(&format!(
        "({} @ {}",
        summary.max_distance_error_body,
        format_instant(summary.max_distance_error_epoch)
    )));
    assert!(rendered.contains("transparency evidence only, not a production tolerance envelope"));
}

#[test]
fn interpolation_quality_kind_coverage_reports_the_distinct_body_breakdown() {
    let coverage = jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
    assert_eq!(coverage.sample_count, 223);
    assert_eq!(coverage.body_count, 16);
    assert_eq!(coverage.bodies.len(), coverage.body_count);
    assert!(!coverage.bodies.is_empty());
    assert!(coverage.cubic_body_count > 0);
    assert_eq!(coverage.quadratic_body_count, 0);
    assert_eq!(coverage.linear_body_count, 0);

    assert_eq!(coverage.to_string(), coverage.summary_line());
    assert_eq!(
        coverage.validated_summary_line(),
        Ok(coverage.summary_line())
    );

    let rendered = format_jpl_interpolation_quality_kind_coverage(&coverage);
    assert!(rendered.contains("JPL interpolation quality kind coverage:"));
    assert!(rendered.contains("223 samples across 16 bodies ["));
    assert!(rendered.contains(&coverage.bodies[0]));
    assert!(rendered.contains("cubic bodies"));
    assert!(rendered.contains("quadratic bodies"));
    assert!(rendered.contains("linear bodies"));
    assert_eq!(
        jpl_interpolation_quality_kind_coverage_for_report(),
        coverage.summary_line()
    );
}

#[test]
fn interpolation_quality_sample_request_corpus_reports_the_explicit_request_slice() {
    let summary = interpolation_quality_sample_request_corpus_summary()
        .expect("sample request corpus should exist");
    assert_eq!(summary.request_count, 223);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.bodies.len(), summary.body_count);
    assert!(!summary.bodies.is_empty());
    assert_eq!(summary.frame, CoordinateFrame::Ecliptic);
    assert_eq!(summary.time_scale, TimeScale::Tdb);
    assert_eq!(summary.zodiac_mode, ZodiacMode::Tropical);
    assert_eq!(summary.apparentness, Apparentness::Mean);
    assert!(summary.earliest_epoch.julian_day.days() <= summary.latest_epoch.julian_day.days());
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        interpolation_quality_sample_request_corpus_summary_for_report(),
        summary.summary_line()
    );
    assert!(
        format_interpolation_quality_sample_request_corpus_summary(&summary)
            .contains("Interpolation-quality sample request corpus:")
    );
    assert!(summary.summary_line().contains("observerless"));
}

#[test]
fn interpolation_quality_sample_request_corpus_summary_validation_rejects_drift() {
    let mut summary = interpolation_quality_sample_request_corpus_summary()
        .expect("sample request corpus should exist");
    summary.request_count += 1;
    assert_eq!(
        summary.validate(),
        Err(
            InterpolationQualitySampleRequestCorpusSummaryValidationError::FieldOutOfSync {
                field: "request_count"
            }
        )
    );
}

#[test]
fn interpolation_quality_summary_for_report_combines_source_summary_summary_and_coverage() {
    let source_summary =
        jpl_interpolation_quality_source_summary().expect("source summary should exist");
    let summary = jpl_interpolation_quality_summary().expect("summary should exist");
    let coverage = jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
    let rendered = format_jpl_interpolation_quality_summary_for_report();

    assert!(rendered.contains(&source_summary.summary_line()));
    assert!(rendered.contains(&format_jpl_interpolation_quality_summary(&summary)));
    assert!(rendered.contains(&format_jpl_interpolation_quality_kind_coverage(&coverage)));
    assert!(rendered.contains(&interpolation_quality_sample_request_corpus_summary_for_report()));
    assert!(rendered.contains(&jpl_interpolation_body_class_error_envelopes_for_report()));
}

#[test]
fn interpolation_body_class_error_envelope_summary_reports_the_expected_body_classes() {
    let summaries =
        jpl_interpolation_body_class_error_envelopes().expect("body-class envelopes should exist");

    assert_eq!(summaries.len(), 4);
    assert_eq!(summaries[0].class, "Luminaries");
    assert_eq!(summaries[1].class, "Major planets");
    assert_eq!(summaries[2].class, "Selected asteroids");
    assert_eq!(summaries[3].class, "Custom bodies");
    assert!(summaries.iter().all(|summary| summary.validate().is_ok()));
    assert!(jpl_interpolation_body_class_error_envelopes_for_report()
        .contains("JPL interpolation body-class error envelopes:"));
    assert!(jpl_interpolation_body_class_error_envelopes_for_report().contains("Luminaries"));
    assert!(jpl_interpolation_body_class_error_envelopes_for_report().contains("Major planets"));
}

#[test]
fn interpolation_body_class_error_envelope_summary_validation_rejects_drift() {
    let mut summary = jpl_interpolation_body_class_error_envelopes()
        .expect("body-class envelopes should exist")
        .into_iter()
        .find(|summary| summary.class == "Luminaries")
        .expect("luminary envelope should exist");

    summary.mean_longitude_error_deg += 1e-12;

    assert_eq!(
        summary.validate(),
        Err(
            JplInterpolationBodyClassErrorEnvelopeSummaryValidationError::FieldOutOfSync {
                class: "Luminaries"
            }
        )
    );
}

#[test]
fn interpolation_posture_summary_reports_the_release_decision() {
    let summary = jpl_interpolation_posture_summary().expect("summary should exist");
    assert_eq!(summary.source, JPL_INTERPOLATION_POSTURE_SOURCE);
    assert_eq!(summary.detail, JPL_INTERPOLATION_POSTURE_DETAIL);
    assert_eq!(summary.envelope, JPL_INTERPOLATION_POSTURE_ENVELOPE);
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        jpl_interpolation_posture_summary_for_report(),
        summary.summary_line()
    );
    assert!(
        format_jpl_interpolation_posture_summary(&summary).contains("JPL interpolation posture:")
    );
    assert!(summary
        .summary_line()
        .contains("transparency evidence only"));
    assert!(summary
        .summary_line()
        .contains("not a production tolerance envelope"));
}

#[test]
fn interpolation_posture_summary_validation_rejects_drift() {
    let mut summary = jpl_interpolation_posture_summary().expect("summary should exist");
    summary.detail = "runtime production tolerance".to_string();
    assert_eq!(
        summary.validate(),
        Err(JplInterpolationPostureSummaryValidationError::FieldOutOfSync { field: "detail" })
    );
}

#[test]
fn interpolation_quality_source_summary_reports_the_expected_provenance() {
    let summary = jpl_interpolation_quality_source_summary().expect("source summary should exist");

    assert_eq!(summary.source, reference_snapshot_source_summary().source);
    assert_eq!(summary.derivation, JPL_INTERPOLATION_QUALITY_DERIVATION);
    assert_eq!(summary.sample_count, 223);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.epoch_count, 19);
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        jpl_interpolation_quality_source_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn interpolation_quality_summary_validated_summary_line_returns_the_rendered_line() {
    let summary = jpl_interpolation_quality_summary().expect("summary should exist");
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
}

#[test]
fn interpolation_quality_summary_validated_summary_line_rejects_drift() {
    let mut summary = jpl_interpolation_quality_summary().expect("summary should exist");
    summary.mean_longitude_error_deg += 1e-12;
    assert_eq!(
        summary.validated_summary_line(),
        Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
    );
}

#[test]
fn interpolation_quality_source_summary_validation_rejects_drift() {
    let mut summary =
        jpl_interpolation_quality_source_summary().expect("source summary should exist");
    summary.epoch_count += 1;
    assert_eq!(
        summary.validate(),
        Err(
            JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                field: "epoch_count"
            }
        )
    );
    assert_eq!(
            JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                field: "epoch_count"
            }
            .to_string(),
            "the JPL interpolation-quality source summary field `epoch_count` is out of sync with the current evidence"
        );
}

#[test]
fn interpolation_quality_kind_coverage_validated_summary_line_rejects_drift() {
    let mut coverage = jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
    coverage.cubic_body_count += 1;
    assert_eq!(
        coverage.validated_summary_line(),
        Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
    );
}

#[test]
fn interpolation_quality_summary_validation_rejects_inconsistent_counts() {
    let mut summary = jpl_interpolation_quality_summary().expect("summary should exist");
    summary.sample_count = 0;
    assert_eq!(
        summary.validate(),
        Err(JplInterpolationQualitySummaryValidationError::MissingSamples)
    );

    let mut summary = jpl_interpolation_quality_summary().expect("summary should exist");
    summary.cubic_sample_count += 1;
    let kind_count =
        summary.cubic_sample_count + summary.quadratic_sample_count + summary.linear_sample_count;
    assert_eq!(
        summary.validate(),
        Err(
            JplInterpolationQualitySummaryValidationError::InterpolationKindCountMismatch {
                sample_count: summary.sample_count,
                kind_count,
            }
        )
    );
}

#[test]
fn interpolation_quality_summary_validation_rejects_non_finite_metrics() {
    let mut summary = jpl_interpolation_quality_summary().expect("summary should exist");
    summary.max_longitude_error_deg = f64::INFINITY;
    assert_eq!(
        summary.validate(),
        Err(
            JplInterpolationQualitySummaryValidationError::MetricOutOfRange {
                field: "max_longitude_error_deg",
            }
        )
    );
}

#[test]
fn interpolation_quality_summary_validation_rejects_blank_peak_bodies() {
    let mut summary = jpl_interpolation_quality_summary().expect("summary should exist");
    summary.max_latitude_error_body.clear();
    assert_eq!(
        summary.validate(),
        Err(
            JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                field: "max_latitude_error_body",
            }
        )
    );
}

#[test]
fn interpolation_quality_summary_validation_rejects_derived_summary_drift() {
    let mut summary = jpl_interpolation_quality_summary().expect("summary should exist");
    summary.mean_longitude_error_deg += 1e-12;
    assert_eq!(
        summary.validate(),
        Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
    );
}

#[test]
fn interpolation_quality_coverage_validation_rejects_inconsistent_bodies() {
    let mut coverage = jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
    coverage.body_count += 1;
    assert_eq!(
        coverage.validate(),
        Err(
            JplInterpolationQualitySummaryValidationError::BodyCountMismatch {
                body_count: coverage.body_count,
                bodies_len: coverage.bodies.len(),
            }
        )
    );
}

#[test]
fn interpolation_quality_coverage_validation_rejects_duplicate_bodies() {
    let mut coverage = jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
    let duplicate = coverage.bodies[0].clone();
    coverage.bodies[1] = duplicate.clone();
    coverage.body_count = coverage.bodies.len();
    assert_eq!(
        coverage.validate(),
        Err(JplInterpolationQualitySummaryValidationError::DuplicateBody { body: duplicate })
    );
}

#[test]
fn interpolation_quality_coverage_validation_rejects_derived_summary_drift() {
    let mut coverage = jpl_interpolation_quality_kind_coverage().expect("coverage should exist");
    coverage.cubic_body_count += 1;
    assert_eq!(
        coverage.validate(),
        Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
    );
}

#[test]
fn frame_treatment_summary_documents_the_shared_mean_obliquity_transform() {
    let summary = frame_treatment_summary_details();

    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
            summary.summary_line(),
            "checked-in ecliptic snapshot; equatorial coordinates are derived with a mean-obliquity transform"
        );
    assert_eq!(frame_treatment_summary(), summary.summary_line());
    assert_eq!(frame_treatment_summary_for_report(), summary.summary_line());
    assert!(summary.summary_line().contains("mean-obliquity transform"));
}

#[test]
fn request_policy_summary_is_displayable() {
    let policy = jpl_snapshot_request_policy();

    assert_eq!(policy.to_string(), policy.summary_line());
    assert_eq!(policy.validated_summary_line(), Ok(policy.summary_line()));
    assert_eq!(
        jpl_snapshot_request_policy_summary_for_report(),
        policy.summary_line()
    );
    assert!(policy
        .summary_line()
        .contains("frames=Ecliptic, Equatorial"));
    assert!(policy.validate().is_ok());
}

#[test]
fn request_policy_summary_validation_rejects_stale_posture() {
    let mut policy = jpl_snapshot_request_policy();
    policy.supports_topocentric_observer = true;

    let error = policy
        .validate()
        .expect_err("drifted JPL request-policy summaries should fail validation");

    assert_eq!(
        error,
        JplSnapshotRequestPolicyValidationError::FieldOutOfSync {
            field: "supports_topocentric_observer"
        }
    );
    assert_eq!(
            error.to_string(),
            "the JPL snapshot request-policy summary field `supports_topocentric_observer` is out of sync with the current posture"
        );
}

#[test]
fn batch_error_taxonomy_request_corpus_matches_the_control_sample() {
    let requests = jpl_snapshot_batch_error_taxonomy_request_corpus();

    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].body, pleiades_backend::CelestialBody::Ceres);
    assert_eq!(requests[1].body, pleiades_backend::CelestialBody::MeanNode);
    assert_eq!(requests[2].body, pleiades_backend::CelestialBody::Ceres);
    assert_eq!(requests[0].instant, reference_instant());
    assert_eq!(requests[1].instant, reference_instant());
    assert_eq!(
        requests[2].instant,
        Instant::new(JulianDay::from_days(2_634_168.0), TimeScale::Tdb)
    );
    assert!(requests.iter().all(|request| request.observer.is_none()));
    assert!(requests
        .iter()
        .all(|request| request.frame == CoordinateFrame::Ecliptic));
    assert!(requests
        .iter()
        .all(|request| request.zodiac_mode == ZodiacMode::Tropical));
    assert!(requests
        .iter()
        .all(|request| request.apparent == Apparentness::Mean));
}

#[test]
fn batch_error_taxonomy_summary_matches_current_backend() {
    let summary = jpl_snapshot_batch_error_taxonomy_summary()
        .expect("the batch taxonomy summary should remain computable");
    assert_eq!(
            summary.summary_line(),
            "JPL batch error taxonomy: supported body Ceres; unsupported body Mean Node -> UnsupportedBody; out-of-range Ceres -> OutOfRangeInstant"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        jpl_snapshot_batch_error_taxonomy_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(
        summary.supported_request_body,
        pleiades_backend::CelestialBody::Ceres
    );
    assert_eq!(
        summary.unsupported_request_body,
        pleiades_backend::CelestialBody::MeanNode
    );
    assert_eq!(
        summary.unsupported_error_kind,
        EphemerisErrorKind::UnsupportedBody
    );
    assert_eq!(
        summary.out_of_range_request_body,
        pleiades_backend::CelestialBody::Ceres
    );
    assert_eq!(
        summary.out_of_range_error_kind,
        EphemerisErrorKind::OutOfRangeInstant
    );
}

#[test]
fn batch_error_taxonomy_summary_validation_rejects_drifted_fields() {
    let summary = JplSnapshotBatchErrorTaxonomySummary {
        supported_request_body: pleiades_backend::CelestialBody::Sun,
        unsupported_request_body: pleiades_backend::CelestialBody::MeanNode,
        unsupported_error_kind: EphemerisErrorKind::UnsupportedBody,
        out_of_range_request_body: pleiades_backend::CelestialBody::Ceres,
        out_of_range_error_kind: EphemerisErrorKind::OutOfRangeInstant,
    };
    assert_eq!(
        summary.validate(),
        Err(
            JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                field: "supported_request_body"
            }
        )
    );
    assert_eq!(
            JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                field: "supported_request_body"
            }
            .to_string(),
            "the JPL batch error-taxonomy summary field `supported_request_body` is out of sync with the current posture"
        );
}
