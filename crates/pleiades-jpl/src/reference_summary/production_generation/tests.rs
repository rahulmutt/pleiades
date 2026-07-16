//! Tests for the production_generation module.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use pleiades_backend::{Apparentness, EphemerisErrorKind, EphemerisRequest};
#[allow(unused_imports)]
use pleiades_backend::{CelestialBody, EphemerisBackend, QualityAnnotation};
#[allow(unused_imports)]
use pleiades_types::CoordinateFrame;

#[test]
fn production_generation_snapshot_summary_reports_the_boundary_overlay() {
    let summary = production_generation_snapshot_summary()
        .expect("production-generation snapshot summary should exist");
    summary
        .validate()
        .expect("production-generation snapshot summary should validate");
    assert_eq!(summary.row_count, 277);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.epoch_count, 23);
    assert_eq!(summary.boundary_row_count, 66);
    assert_eq!(summary.boundary_body_count, 16);
    assert_eq!(summary.boundary_epoch_count, 12);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(
        summary.boundary_earliest_epoch.julian_day.days(),
        2_378_498.5
    );
    assert_eq!(summary.boundary_latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(summary.quarter_day_row_count, 8);
    assert_eq!(summary.quarter_day_body_count, 4);
    assert_eq!(summary.quarter_day_epoch_count, 2);
    assert_eq!(
        summary.quarter_day_earliest_epoch.julian_day.days(),
        2_451_915.25
    );
    assert_eq!(
        summary.quarter_day_latest_epoch.julian_day.days(),
        2_451_915.75
    );
}

#[test]
fn production_generation_snapshot_summary_validation_rejects_quarter_day_drift() {
    let mut summary = production_generation_snapshot_summary()
        .expect("production-generation snapshot summary should exist");
    summary.quarter_day_row_count += 1;

    let error = summary
        .validate()
        .expect_err("drifted quarter-day production-generation summary should fail validation");

    assert!(matches!(
        error,
        ProductionGenerationSnapshotSummaryValidationError::DerivedSummaryMismatch
    ));
}

#[test]
fn production_generation_snapshot_window_summary_reports_the_source_windows() {
    let summary = production_generation_snapshot_window_summary()
        .expect("production-generation source window summary should exist");
    summary
        .validate()
        .expect("production-generation source window summary should validate");
    assert_eq!(summary.sample_count, 277);
    assert_eq!(summary.sample_bodies.len(), 16);
    assert_eq!(summary.windows.len(), summary.sample_bodies.len());
    assert_eq!(summary.sample_bodies, reference_bodies());
    assert_eq!(summary.epoch_count, 23);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(summary.windows[0].body, CelestialBody::Ceres);
    assert!(summary.windows[0].sample_count >= 8);
}

#[test]
fn production_generation_snapshot_window_summary_validation_rejects_body_order_drift() {
    let mut summary = production_generation_snapshot_window_summary()
        .expect("production-generation source window summary should exist");
    summary.sample_bodies.swap(0, 1);
    let error = summary
        .validate()
        .expect_err("body order drift should be rejected");
    assert!(matches!(
        error,
        ProductionGenerationSnapshotWindowSummaryValidationError::BodyOrderMismatch { .. }
    ));
}

#[test]
fn production_generation_snapshot_window_summary_validation_rejects_derived_summary_drift() {
    let mut summary = production_generation_snapshot_window_summary()
        .expect("production-generation source window summary should exist");
    summary.sample_count += 1;
    let error = summary
        .validate()
        .expect_err("derived summary drift should be rejected");
    assert_eq!(
        error,
        ProductionGenerationSnapshotWindowSummaryValidationError::DerivedSummaryMismatch
    );
}

#[test]
fn production_generation_snapshot_body_class_coverage_summary_reports_the_split() {
    let summary = production_generation_snapshot_body_class_coverage_summary()
        .expect("production-generation body-class coverage summary should exist");
    summary
        .validate()
        .expect("production-generation body-class coverage summary should validate");
    assert_eq!(summary.row_count, 277);
    assert_eq!(summary.major_bodies.len(), 10);
    assert_eq!(summary.asteroid_bodies.len(), 6);
}

#[test]
fn production_generation_snapshot_body_class_coverage_summary_validation_rejects_major_body_drift()
{
    let mut summary = production_generation_snapshot_body_class_coverage_summary()
        .expect("production-generation body-class coverage summary should exist");
    summary.major_bodies.pop();

    assert_eq!(
        summary.validate(),
        Err(
            ProductionGenerationSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                field: "major_bodies",
            }
        )
    );
}

#[test]
fn production_generation_boundary_summary_reports_the_overlay() {
    let summary = production_generation_boundary_summary()
        .expect("production-generation boundary summary should exist");
    summary
        .validate()
        .expect("production-generation boundary summary should validate");
    assert_eq!(summary.row_count, 66);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.bodies, production_generation_boundary_body_list());
    assert_eq!(summary.epoch_count, 12);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
}

#[test]
fn production_generation_boundary_source_summary_reports_the_overlay_provenance() {
    let boundary_summary = production_generation_boundary_source_summary();
    let holdout_summary = independent_holdout_source_summary();
    boundary_summary
        .validate()
        .expect("production-generation boundary source summary should validate");
    holdout_summary
        .validate()
        .expect("independent hold-out source summary should validate");
    assert_eq!(boundary_summary.source, holdout_summary.source);
    assert_eq!(
        boundary_summary.evidence_class,
        holdout_summary.evidence_class
    );
    assert_eq!(boundary_summary.coverage, holdout_summary.coverage);
    assert_eq!(boundary_summary.columns, holdout_summary.columns);
    assert_eq!(
        boundary_summary.redistribution,
        holdout_summary.redistribution
    );
    assert_eq!(
        boundary_summary.frame_treatment,
        holdout_summary.frame_treatment
    );
    assert_eq!(boundary_summary.time_scale, holdout_summary.time_scale);
}

#[test]
fn production_generation_boundary_window_summary_reports_the_overlay_windows() {
    let summary = production_generation_boundary_window_summary()
        .expect("production-generation boundary window summary should exist");
    assert_eq!(summary.sample_count, 66);
    assert_eq!(summary.sample_bodies.len(), 16);
    assert_eq!(
        summary.sample_bodies,
        production_generation_boundary_body_list().to_vec()
    );
    assert_eq!(summary.epoch_count, 12);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(summary.windows[0].body, CelestialBody::Mars);
    assert_eq!(summary.windows[0].sample_count, 5);
    assert_eq!(summary.windows[0].epoch_count, 5);

    let mut drifted = summary.clone();
    drifted.sample_count += 1;
}

#[test]
fn production_generation_boundary_body_class_coverage_summary_reports_the_overlay_body_classes() {
    let summary = production_generation_boundary_body_class_coverage_summary()
        .expect("production-generation boundary body-class coverage summary should exist");
    summary
        .validate()
        .expect("production-generation boundary body-class coverage summary should validate");
    assert_eq!(summary.row_count, 66);
    assert_eq!(summary.major_body_row_count, 34);
    assert_eq!(summary.major_bodies.len(), 10);
    assert_eq!(
        summary.major_bodies,
        vec![
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
            CelestialBody::Sun,
            CelestialBody::Pluto,
            CelestialBody::Moon,
        ]
    );
    assert_eq!(summary.major_epoch_count, 7);
    assert_eq!(summary.major_windows.len(), 10);
    assert_eq!(summary.asteroid_row_count, 32);
    assert_eq!(summary.asteroid_bodies.len(), 6);
    assert_eq!(summary.asteroid_epoch_count, 7);
    assert_eq!(summary.asteroid_windows.len(), 6);

    let mut drifted = summary.clone();
    drifted.row_count += 1;
}

#[test]
fn production_generation_snapshot_requests_preserve_the_boundary_overlay() {
    let requests = production_generation_snapshot_requests(CoordinateFrame::Ecliptic)
        .expect("production-generation snapshot requests should exist");
    let entries = production_generation_snapshot_entries()
        .expect("production-generation snapshot entries should exist");
    let boundary_entries = production_generation_boundary_entries()
        .expect("production-generation boundary entries should exist");
    let boundary_requests = production_generation_boundary_requests(CoordinateFrame::Equatorial)
        .expect("production-generation boundary requests should exist");

    assert_eq!(requests.len(), entries.len());
    for (request, entry) in requests.iter().zip(entries.iter()) {
        assert_eq!(request.body, entry.body);
        assert_eq!(request.instant.julian_day, entry.epoch.julian_day);
        assert_eq!(request.frame, CoordinateFrame::Ecliptic);
        assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
        assert_eq!(request.apparent, Apparentness::Mean);
        assert!(request.observer.is_none());
    }
    let reference_entries = reference_snapshot();
    let boundary_only_entries: Vec<_> = boundary_entries
        .iter()
        .filter(|entry| {
            !reference_entries
                .iter()
                .any(|reference| reference.body == entry.body && reference.epoch == entry.epoch)
        })
        .cloned()
        .collect();
    assert_eq!(
        &entries[entries.len() - boundary_only_entries.len()..],
        boundary_only_entries.as_slice()
    );
    assert_eq!(boundary_requests.len(), boundary_entries.len());
    for (request, entry) in boundary_requests.iter().zip(boundary_entries.iter()) {
        assert_eq!(request.body, entry.body);
        assert_eq!(request.instant.julian_day, entry.epoch.julian_day);
        assert_eq!(request.frame, CoordinateFrame::Equatorial);
        assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
        assert_eq!(request.apparent, Apparentness::Mean);
        assert!(request.observer.is_none());
    }
    assert_eq!(
        production_generation_boundary_request_corpus(CoordinateFrame::Equatorial),
        production_generation_boundary_requests(CoordinateFrame::Equatorial)
    );
}

#[test]
fn production_generation_snapshot_summary_reports_the_expected_coverage() {
    let summary = production_generation_snapshot_summary()
        .expect("production generation summary should exist");
    assert_eq!(summary.row_count, 277);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.epoch_count, 23);
    assert_eq!(summary.boundary_row_count, 66);
    assert_eq!(summary.boundary_body_count, 16);
    assert_eq!(summary.boundary_epoch_count, 12);
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn production_generation_boundary_request_corpus_summary_reports_the_expected_coverage() {
    let summary = production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic)
        .expect("production generation boundary request corpus summary should exist");
    assert_eq!(summary.request_count, 66);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.epoch_count, 12);
    assert_eq!(summary.frame, CoordinateFrame::Ecliptic);
    assert_eq!(summary.time_scale, TimeScale::Tdb);
    assert_eq!(summary.zodiac_mode, ZodiacMode::Tropical);
    assert_eq!(summary.apparentness, Apparentness::Mean);
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn production_generation_boundary_request_corpus_summary_validation_rejects_field_drift() {
    let mut summary =
        production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic)
            .expect("production generation boundary request corpus summary should exist");

    summary.apparentness = Apparentness::Apparent;
    assert!(matches!(
        summary.validate(),
        Err(
            ProductionGenerationBoundaryRequestCorpusSummaryValidationError::FieldOutOfSync {
                field: "apparentness"
            }
        )
    ));

    let mut summary =
        production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic)
            .expect("production generation boundary request corpus summary should exist");

    summary.time_scale = TimeScale::Utc;
    assert!(matches!(
        summary.validate(),
        Err(
            ProductionGenerationBoundaryRequestCorpusSummaryValidationError::FieldOutOfSync {
                field: "time_scale"
            }
        )
    ));
}

#[test]
fn production_generation_boundary_request_corpus_summary_validation_rejects_body_epoch_and_order_drift(
) {
    let mut body_count_drift =
        production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic)
            .expect("production generation boundary request corpus summary should exist");
    body_count_drift.body_count += 1;
    assert!(matches!(
        body_count_drift.validate(),
        Err(
            ProductionGenerationBoundaryRequestCorpusSummaryValidationError::FieldOutOfSync {
                field: "body_count"
            }
        )
    ));

    let mut body_order_drift =
        production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic)
            .expect("production generation boundary request corpus summary should exist");
    body_order_drift.bodies.swap(0, 1);
    assert!(matches!(
        body_order_drift.validate(),
        Err(
            ProductionGenerationBoundaryRequestCorpusSummaryValidationError::BodyOrderMismatch {
                index: 0,
                ..
            }
        )
    ));

    let mut epoch_count_drift =
        production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic)
            .expect("production generation boundary request corpus summary should exist");
    epoch_count_drift.epoch_count += 1;
    assert!(matches!(
        epoch_count_drift.validate(),
        Err(
            ProductionGenerationBoundaryRequestCorpusSummaryValidationError::FieldOutOfSync {
                field: "epoch_count"
            }
        )
    ));
}
#[test]
fn production_generation_source_summary_documents_the_checked_in_csv_path() {
    let summary = production_generation_source_summary();

    assert!(summary.validate().is_ok());
}
#[test]
fn production_generation_source_revision_summary_validation_rejects_drift() {
    let mut summary = production_generation_source_revision_summary();
    summary.reference_snapshot_checksum ^= 1;

    assert!(matches!(
        summary.validate(),
        Err(
            ProductionGenerationSourceRevisionSummaryValidationError::FieldOutOfSync {
                field: "summary"
            }
        )
    ));
}

#[test]
fn production_generation_manifest_summary_documents_the_current_contract() {
    let summary = production_generation_manifest_summary()
        .expect("production generation manifest summary should exist");

    assert!(summary.validate().is_ok());
}
#[test]
fn production_generation_manifest_summary_validation_rejects_drift() {
    let mut summary = production_generation_manifest_summary()
        .expect("production generation manifest summary should exist");
    summary.boundary_request_corpus_summary.epoch_count += 1;

    assert!(matches!(
        summary.validate(),
        Err(
            ProductionGenerationManifestSummaryValidationError::BoundaryRequestCorpus(
                ProductionGenerationBoundaryRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count"
                }
            )
        )
    ));
}

#[test]
fn production_generation_corpus_shape_summary_documents_the_current_contract() {
    let summary = production_generation_corpus_shape_summary()
        .expect("production generation corpus shape summary should exist");

    assert!(summary.validate().is_ok());
}
#[test]
fn production_generation_corpus_shape_summary_validation_rejects_drift() {
    let mut summary = production_generation_corpus_shape_summary()
        .expect("production generation corpus shape summary should exist");
    summary.boundary_request_corpus_equatorial.apparentness = Apparentness::Apparent;

    assert!(matches!(
        summary.validate(),
        Err(
            ProductionGenerationCorpusShapeSummaryValidationError::BoundaryRequestCorpusEquatorial(
                _
            )
        )
    ));
}

#[test]
fn production_generation_boundary_request_corpus_frame_parity_validation_rejects_drift() {
    let ecliptic = production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic)
        .expect("production generation boundary request corpus summary should exist");
    let mut equatorial =
        production_generation_boundary_request_corpus_summary(CoordinateFrame::Equatorial)
            .expect("production generation boundary request corpus summary should exist");
    equatorial.epoch_count += 1;

    assert!(matches!(
        validate_production_generation_boundary_request_corpus_frame_parity(&ecliptic, &equatorial,),
        Err(
            ProductionGenerationCorpusShapeSummaryValidationError::FieldOutOfSync {
                field: "boundary request corpus parity (epoch_count)"
            }
        )
    ));
}

#[test]
fn production_generation_boundary_request_corpus_frame_parity_validation_rejects_apparentness_drift(
) {
    let ecliptic = production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic)
        .expect("production generation boundary request corpus summary should exist");
    let mut equatorial =
        production_generation_boundary_request_corpus_summary(CoordinateFrame::Equatorial)
            .expect("production generation boundary request corpus summary should exist");
    equatorial.apparentness = Apparentness::Apparent;

    assert!(matches!(
        validate_production_generation_boundary_request_corpus_frame_parity(&ecliptic, &equatorial,),
        Err(
            ProductionGenerationCorpusShapeSummaryValidationError::FieldOutOfSync {
                field: "boundary request corpus parity (apparentness)"
            }
        )
    ));
}
#[test]
fn production_generation_source_summary_validation_rejects_drift() {
    let mut summary = production_generation_source_summary();
    summary.source_revision.reference_snapshot_checksum ^= 1;

    assert!(matches!(
        summary.validate(),
        Err(ProductionGenerationSourceSummaryValidationError::SourceRevisionMismatch)
    ));

    let mut nested_drift = production_generation_source_summary();
    nested_drift.reference_summary.coverage = "drifted coverage".to_string();
    assert!(matches!(
        nested_drift.validate(),
        Err(ProductionGenerationSourceSummaryValidationError::Reference(
            ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync { field: "coverage" }
        ))
    ));

    let mut source_windows_drift = production_generation_source_summary();
    source_windows_drift.source_windows.sample_count += 1;
    assert!(matches!(
        source_windows_drift.validate(),
        Err(
            ProductionGenerationSourceSummaryValidationError::SourceWindows(
                ProductionGenerationSnapshotWindowSummaryValidationError::DerivedSummaryMismatch
            )
        )
    ));
}
