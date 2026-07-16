//! Tests for the holdout module.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use pleiades_backend::{Apparentness, EphemerisErrorKind, EphemerisRequest};
#[allow(unused_imports)]
use pleiades_backend::{CelestialBody, EphemerisBackend, QualityAnnotation};
#[allow(unused_imports)]
use pleiades_types::CoordinateFrame;

#[test]
fn reference_holdout_overlap_summary_reports_the_current_overlap() {
    let summary = reference_holdout_overlap_summary()
        .expect("reference/hold-out overlap summary should exist");

    assert_eq!(summary.shared_sample_count, 66);
    assert_eq!(summary.shared_epoch_count, 12);
    assert_eq!(summary.shared_bodies.len(), 16);
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn reference_snapshot_and_holdout_corpora_remain_anchored_to_the_checked_in_csvs() {
    struct SnapshotKeys {
        row_count: usize,
        bodies: BTreeSet<String>,
        epochs: BTreeSet<String>,
        pairs: BTreeSet<(String, String)>,
    }

    fn snapshot_keys(contents: &str) -> SnapshotKeys {
        let mut bodies = BTreeSet::new();
        let mut epochs = BTreeSet::new();
        let mut pairs = BTreeSet::new();
        let mut row_count = 0usize;

        for line in contents
            .lines()
            .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        {
            let mut columns = line.split(',');
            let epoch = columns
                .next()
                .expect("snapshot rows should include an epoch")
                .trim()
                .to_string();
            let body = columns
                .next()
                .expect("snapshot rows should include a body")
                .trim()
                .to_string();

            row_count += 1;
            epochs.insert(epoch.clone());
            bodies.insert(body.clone());
            pairs.insert((body, epoch));
        }

        SnapshotKeys {
            row_count,
            bodies,
            epochs,
            pairs,
        }
    }

    let reference = snapshot_keys(include_str!("../../../data/reference_snapshot.csv"));
    let holdout = snapshot_keys(include_str!(
        "../../../data/independent_holdout_snapshot.csv"
    ));

    assert_eq!(reference.row_count, 277);
    assert_eq!(reference.row_count, reference.pairs.len());
    assert_eq!(reference.bodies.len(), 16);
    assert_eq!(reference.epochs.len(), 23);
    assert_eq!(holdout.row_count, 66);
    assert_eq!(holdout.row_count, holdout.pairs.len());
    assert_eq!(holdout.bodies.len(), 16);
    assert_eq!(holdout.epochs.len(), 12);

    assert_eq!(
        reference_holdout_overlap_summary().map(|summary| summary.shared_sample_count),
        Some(66)
    );
    assert_eq!(
        reference.pairs.intersection(&holdout.pairs).count(),
        66,
        "reference and hold-out corpora should retain the documented 66 shared body-epoch pairs"
    );
    assert_eq!(reference.bodies.intersection(&holdout.bodies).count(), 16);
    assert_eq!(reference.epochs.intersection(&holdout.epochs).count(), 12);
}

#[test]
fn independent_holdout_source_summary_reports_the_expected_provenance() {
    let summary = independent_holdout_source_summary();

    assert_eq!(
        summary.source,
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables."
    );
    assert_eq!(
            summary.coverage,
            "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs."
        );
    assert_eq!(summary.evidence_class, INDEPENDENT_HOLDOUT_EVIDENCE_CLASS);
    assert_eq!(summary.columns, "epoch_jd, body, x_km, y_km, z_km");
    assert_eq!(summary.frame_treatment, INDEPENDENT_HOLDOUT_FRAME_TREATMENT);
    assert_eq!(summary.time_scale, INDEPENDENT_HOLDOUT_TIME_SCALE);
    assert_eq!(
        summary.redistribution,
        "repository-checked regression fixtures, not a broad public corpus."
    );
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn independent_holdout_source_summary_validation_reports_blank_fields() {
    let blank_source = IndependentHoldoutSourceSummary {
        source: " ".to_string(),
        evidence_class: INDEPENDENT_HOLDOUT_EVIDENCE_CLASS.to_string(),
        coverage: "coverage".to_string(),
        columns: "epoch_jd, body, x_km, y_km, z_km".to_string(),
        redistribution: INDEPENDENT_HOLDOUT_REDISTRIBUTION_FALLBACK.to_string(),
        checksum: independent_holdout_source_checksum(),
        frame_treatment: INDEPENDENT_HOLDOUT_FRAME_TREATMENT.to_string(),
        time_scale: INDEPENDENT_HOLDOUT_TIME_SCALE.to_string(),
    };
    assert_eq!(
        blank_source.validate(),
        Err(IndependentHoldoutSourceSummaryValidationError::BlankSource)
    );

    let blank_coverage = IndependentHoldoutSourceSummary {
        source: INDEPENDENT_HOLDOUT_SOURCE_EXPECTED.to_string(),
        evidence_class: INDEPENDENT_HOLDOUT_EVIDENCE_CLASS.to_string(),
        coverage: "\t".to_string(),
        columns: INDEPENDENT_HOLDOUT_COLUMNS.to_string(),
        redistribution: INDEPENDENT_HOLDOUT_REDISTRIBUTION_FALLBACK.to_string(),
        checksum: independent_holdout_source_checksum(),
        frame_treatment: INDEPENDENT_HOLDOUT_FRAME_TREATMENT.to_string(),
        time_scale: INDEPENDENT_HOLDOUT_TIME_SCALE.to_string(),
    };
    assert_eq!(
        blank_coverage.validate(),
        Err(IndependentHoldoutSourceSummaryValidationError::BlankCoverage)
    );

    let blank_columns = IndependentHoldoutSourceSummary {
        source: INDEPENDENT_HOLDOUT_SOURCE_EXPECTED.to_string(),
        evidence_class: INDEPENDENT_HOLDOUT_EVIDENCE_CLASS.to_string(),
        coverage: INDEPENDENT_HOLDOUT_COVERAGE_FALLBACK.to_string(),
        columns: "  ".to_string(),
        redistribution: INDEPENDENT_HOLDOUT_REDISTRIBUTION_FALLBACK.to_string(),
        checksum: independent_holdout_source_checksum(),
        frame_treatment: INDEPENDENT_HOLDOUT_FRAME_TREATMENT.to_string(),
        time_scale: INDEPENDENT_HOLDOUT_TIME_SCALE.to_string(),
    };
    assert_eq!(
        blank_columns.validate(),
        Err(IndependentHoldoutSourceSummaryValidationError::BlankColumns)
    );

    let blank_redistribution = IndependentHoldoutSourceSummary {
        source: INDEPENDENT_HOLDOUT_SOURCE_EXPECTED.to_string(),
        evidence_class: INDEPENDENT_HOLDOUT_EVIDENCE_CLASS.to_string(),
        coverage: INDEPENDENT_HOLDOUT_COVERAGE_FALLBACK.to_string(),
        columns: INDEPENDENT_HOLDOUT_COLUMNS.to_string(),
        redistribution: " ".to_string(),
        checksum: independent_holdout_source_checksum(),
        frame_treatment: INDEPENDENT_HOLDOUT_FRAME_TREATMENT.to_string(),
        time_scale: INDEPENDENT_HOLDOUT_TIME_SCALE.to_string(),
    };
    assert_eq!(
        blank_redistribution.validate(),
        Err(IndependentHoldoutSourceSummaryValidationError::BlankRedistribution)
    );

    let padded_columns = IndependentHoldoutSourceSummary {
        source: INDEPENDENT_HOLDOUT_SOURCE_EXPECTED.to_string(),
        evidence_class: INDEPENDENT_HOLDOUT_EVIDENCE_CLASS.to_string(),
        coverage: INDEPENDENT_HOLDOUT_COVERAGE_FALLBACK.to_string(),
        columns: " epoch_jd, body, x_km, y_km, z_km ".to_string(),
        redistribution: INDEPENDENT_HOLDOUT_REDISTRIBUTION_FALLBACK.to_string(),
        checksum: independent_holdout_source_checksum(),
        frame_treatment: INDEPENDENT_HOLDOUT_FRAME_TREATMENT.to_string(),
        time_scale: INDEPENDENT_HOLDOUT_TIME_SCALE.to_string(),
    };
    assert_eq!(
        padded_columns.validate(),
        Err(
            IndependentHoldoutSourceSummaryValidationError::SurroundedByWhitespace {
                field: "columns",
            }
        )
    );

    let multiline_coverage = IndependentHoldoutSourceSummary {
        source: INDEPENDENT_HOLDOUT_SOURCE_EXPECTED.to_string(),
        evidence_class: INDEPENDENT_HOLDOUT_EVIDENCE_CLASS.to_string(),
        coverage: "coverage\nmore".to_string(),
        columns: INDEPENDENT_HOLDOUT_COLUMNS.to_string(),
        redistribution: INDEPENDENT_HOLDOUT_REDISTRIBUTION_FALLBACK.to_string(),
        checksum: independent_holdout_source_checksum(),
        frame_treatment: INDEPENDENT_HOLDOUT_FRAME_TREATMENT.to_string(),
        time_scale: INDEPENDENT_HOLDOUT_TIME_SCALE.to_string(),
    };
    assert_eq!(
        multiline_coverage.validate(),
        Err(
            IndependentHoldoutSourceSummaryValidationError::SurroundedByWhitespace {
                field: "coverage",
            }
        )
    );

    let mut drifted_summary = independent_holdout_source_summary();
    drifted_summary.source =
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables (drift)."
            .to_string();
    assert_eq!(
        drifted_summary.validate(),
        Err(IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync { field: "source" })
    );

    let mut drifted_evidence_class = independent_holdout_source_summary();
    drifted_evidence_class.evidence_class = "hold-out-drift".to_string();
    assert_eq!(
        drifted_evidence_class.validate(),
        Err(
            IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync {
                field: "evidence_class"
            }
        )
    );

    let mut drifted_coverage = independent_holdout_source_summary();
    drifted_coverage.coverage = "hold-out coverage drift".to_string();
    assert_eq!(
        drifted_coverage.validate(),
        Err(IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync { field: "coverage" })
    );

    let mut drifted_columns = independent_holdout_source_summary();
    drifted_columns.columns = "body, epoch_jd, x_km, y_km, z_km".to_string();
    assert_eq!(
        drifted_columns.validate(),
        Err(IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync { field: "columns" })
    );

    let mut drifted_redistribution = independent_holdout_source_summary();
    drifted_redistribution.redistribution = "fixture redistribution drift".to_string();
    assert_eq!(
        drifted_redistribution.validate(),
        Err(
            IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync {
                field: "redistribution"
            }
        )
    );

    let mut drifted_frame_treatment = independent_holdout_source_summary();
    drifted_frame_treatment.frame_treatment = "geocentric ecliptic J2000 drift".to_string();
    assert_eq!(
        drifted_frame_treatment.validate(),
        Err(
            IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync {
                field: "frame_treatment"
            }
        )
    );

    let mut drifted_time_scale = independent_holdout_source_summary();
    drifted_time_scale.time_scale = "TT".to_string();
    assert_eq!(
        drifted_time_scale.validate(),
        Err(
            IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync {
                field: "time_scale"
            }
        )
    );
}

#[test]
fn independent_holdout_snapshot_summary_reports_the_expected_coverage() {
    let summary =
        independent_holdout_snapshot_summary().expect("independent hold-out summary should exist");
    assert_eq!(summary.row_count, 66);
    assert_eq!(summary.body_count, 16);
    assert_eq!(
        summary.bodies,
        vec![
            "Mars",
            "Jupiter",
            "Mercury",
            "Venus",
            "Saturn",
            "Uranus",
            "Neptune",
            "Sun",
            "Pluto",
            "Moon",
            "Ceres",
            "Pallas",
            "Juno",
            "Vesta",
            "asteroid:433-Eros",
            "asteroid:99942-Apophis",
        ]
    );
    assert_eq!(summary.epoch_count, 12);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn independent_holdout_snapshot_source_window_summary_reports_the_expected_windows() {
    let summary = independent_holdout_snapshot_source_window_summary()
        .expect("independent hold-out source window summary should exist");
    assert_eq!(summary.sample_count, 66);
    assert_eq!(summary.sample_bodies.len(), 16);
    assert_eq!(summary.sample_bodies, independent_holdout_bodies().to_vec());
    assert_eq!(summary.epoch_count, 12);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(summary.windows.len(), 16);
    assert_eq!(
        summary.windows[0].body,
        pleiades_backend::CelestialBody::Mars
    );
    assert_eq!(summary.windows[0].sample_count, 5);
    assert_eq!(summary.windows[0].epoch_count, 5);
    assert_eq!(
        summary.windows[0].earliest_epoch.julian_day.days(),
        2_451_545.0
    );
    assert_eq!(
        summary.windows[0].latest_epoch.julian_day.days(),
        2_451_915.5
    );
    assert_eq!(
        summary.windows[8].body,
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.windows[8].sample_count, 2);
    assert_eq!(summary.windows[8].epoch_count, 2);
    assert_eq!(
        summary.windows[8].earliest_epoch.julian_day.days(),
        2_451_545.0
    );
    assert_eq!(
        summary.windows[8].latest_epoch.julian_day.days(),
        2_451_915.5
    );
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn independent_holdout_quarter_day_boundary_summary_reports_the_expected_window() {
    let summary = independent_holdout_quarter_day_boundary_summary_details()
        .expect("independent hold-out quarter-day boundary summary should exist");
    assert_eq!(summary.row_count, 8);
    assert_eq!(summary.body_count, 4);
    assert_eq!(
        summary.bodies,
        vec![
            pleiades_backend::CelestialBody::Sun,
            pleiades_backend::CelestialBody::Moon,
            pleiades_backend::CelestialBody::Mercury,
            pleiades_backend::CelestialBody::Venus,
        ]
    );
    assert_eq!(summary.epoch_count, 2);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_915.25);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_915.75);
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn independent_holdout_high_curvature_summary_reports_the_expected_window() {
    let summary = independent_holdout_high_curvature_summary()
        .expect("independent hold-out high-curvature summary should exist");
    assert_eq!(summary.sample_count, 8);
    assert_eq!(summary.sample_bodies.len(), 4);
    assert_eq!(
        summary.sample_bodies,
        vec![
            pleiades_backend::CelestialBody::Sun,
            pleiades_backend::CelestialBody::Moon,
            pleiades_backend::CelestialBody::Mercury,
            pleiades_backend::CelestialBody::Venus,
        ]
    );
    assert_eq!(summary.epoch_count, 2);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_915.25);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_915.75);
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn independent_holdout_snapshot_source_window_summary_validation_rejects_sample_body_order_drift() {
    let mut summary = independent_holdout_snapshot_source_window_summary()
        .expect("independent hold-out source window summary should exist");
    summary.sample_bodies.swap(0, 1);

    assert!(matches!(
        summary.validate(),
        Err(
            IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    ));
}

#[test]
fn independent_holdout_snapshot_source_window_summary_validation_rejects_window_order_drift() {
    let mut summary = independent_holdout_snapshot_source_window_summary()
        .expect("independent hold-out source window summary should exist");
    summary.windows.swap(0, 1);

    assert!(matches!(
        summary.validate(),
        Err(
            IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "windows"
            }
        )
    ));
}

#[test]
fn independent_holdout_snapshot_summary_validation_rejects_duplicate_bodies() {
    let summary = IndependentHoldoutSnapshotSummary {
        row_count: 2,
        body_count: 2,
        bodies: vec!["Mars".to_string(), "Mars".to_string()],
        epoch_count: 1,
        earliest_epoch: reference_instant(),
        latest_epoch: reference_instant(),
    };
    assert!(matches!(
        summary.validate(),
        Err(IndependentHoldoutSnapshotSummaryValidationError::DuplicateBody {
            first_index: 0,
            second_index: 1,
            body,
        }) if body == "Mars"
    ));
}

#[test]
fn independent_holdout_snapshot_summary_validation_rejects_body_order_drift() {
    let summary =
        independent_holdout_snapshot_summary().expect("independent hold-out summary should exist");
    let mut bodies = summary.bodies.clone();
    bodies.swap(0, 1);
    let summary = IndependentHoldoutSnapshotSummary { bodies, ..summary };

    assert_eq!(
        summary.validate(),
        Err(IndependentHoldoutSnapshotSummaryValidationError::DerivedSummaryMismatch)
    );
}

#[test]
fn independent_holdout_snapshot_equatorial_parity_summary_reports_the_expected_coverage() {
    let summary = independent_holdout_snapshot_equatorial_parity_summary()
        .expect("independent hold-out equatorial parity summary should exist");
    assert_eq!(summary.row_count, 66);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.epoch_count, 12);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn independent_holdout_snapshot_equatorial_parity_summary_validation_rejects_row_count_drift() {
    let summary = IndependentHoldoutSnapshotEquatorialParitySummary {
        row_count: 1,
        body_count: 2,
        epoch_count: 1,
        earliest_epoch: reference_instant(),
        latest_epoch: reference_instant(),
    };

    assert!(matches!(
            summary.validate(),
            Err(IndependentHoldoutSnapshotEquatorialParitySummaryValidationError::BodyCountExceedsRowCount {
                body_count: 2,
                row_count: 1,
            })
        ));
}

#[test]
fn independent_holdout_summary_reports_the_expected_envelope() {
    let summary =
        jpl_independent_holdout_summary().expect("independent hold-out summary should exist");
    assert_eq!(summary.sample_count, 66);
    assert_eq!(summary.body_count, 16);
    assert_eq!(
        summary.bodies,
        vec![
            "Mars",
            "Jupiter",
            "Mercury",
            "Venus",
            "Saturn",
            "Uranus",
            "Neptune",
            "Sun",
            "Pluto",
            "Moon",
            "Ceres",
            "Pallas",
            "Juno",
            "Vesta",
            "asteroid:433-Eros",
            "asteroid:99942-Apophis",
        ]
    );
    assert_eq!(summary.epoch_count, 12);
    assert!(summary.earliest_epoch.julian_day.days() <= summary.latest_epoch.julian_day.days());
    assert!(summary.max_longitude_error_deg.is_finite());
    assert!(summary.mean_longitude_error_deg.is_finite());
    assert!(summary.median_longitude_error_deg.is_finite());
    assert!(summary.percentile_longitude_error_deg.is_finite());
    assert!(summary.rms_longitude_error_deg.is_finite());
    assert!(summary.max_latitude_error_deg.is_finite());
    assert!(summary.mean_latitude_error_deg.is_finite());
    assert!(summary.median_latitude_error_deg.is_finite());
    assert!(summary.percentile_latitude_error_deg.is_finite());
    assert!(summary.rms_latitude_error_deg.is_finite());
    assert!(summary.max_distance_error_au.is_finite());
    assert!(summary.mean_distance_error_au.is_finite());
    assert!(summary.median_distance_error_au.is_finite());
    assert!(summary.percentile_distance_error_au.is_finite());
    assert!(summary.rms_distance_error_au.is_finite());
    assert!(!summary.max_longitude_error_body.is_empty());
    assert!(!summary.max_latitude_error_body.is_empty());
    assert!(!summary.max_distance_error_body.is_empty());
}

#[test]
fn batch_query_preserves_independent_holdout_order_and_single_query_parity() {
    let backend = JplSnapshotBackend;
    let entries =
        independent_holdout_snapshot_entries().expect("independent hold-out entries should exist");
    let requests = independent_holdout_snapshot_requests(CoordinateFrame::Ecliptic)
        .expect("independent hold-out requests should exist");

    let results = backend
        .positions(&requests)
        .expect("batch query should resolve the independent hold-out rows");

    assert_eq!(results.len(), entries.len());
    for ((entry, request), batch_result) in entries.iter().zip(requests.iter()).zip(results.iter())
    {
        assert_eq!(batch_result.body, entry.body);
        assert_eq!(batch_result.instant, entry.epoch);
        assert_eq!(batch_result.frame, request.frame);
        assert_eq!(batch_result.zodiac_mode, request.zodiac_mode);
        assert_eq!(batch_result.apparent, request.apparent);
        let single = backend
            .position(request)
            .expect("single query should match the independent hold-out batch path");
        assert_eq!(batch_result, &single);
    }
}

#[test]
fn batch_query_preserves_independent_holdout_order_and_mixed_time_scales() {
    let backend = JplSnapshotBackend;
    let entries =
        independent_holdout_snapshot_entries().expect("independent hold-out entries should exist");
    let requests = independent_holdout_snapshot_batch_parity_requests()
        .expect("independent hold-out mixed-scale requests should exist");

    let results = backend
        .positions(&requests)
        .expect("mixed-scale batch query should resolve the independent hold-out rows");

    assert_eq!(results.len(), entries.len());
    for ((entry, request), batch_result) in entries.iter().zip(requests.iter()).zip(results.iter())
    {
        assert_eq!(batch_result.body, entry.body);
        assert_eq!(batch_result.instant, request.instant);
        assert_eq!(batch_result.frame, request.frame);
        assert_eq!(batch_result.zodiac_mode, request.zodiac_mode);
        assert_eq!(batch_result.apparent, request.apparent);
        assert_eq!(batch_result.instant.scale, request.instant.scale);

        let single = backend
            .position(request)
            .expect("single query should match the independent hold-out mixed-scale batch path");
        assert_eq!(batch_result, &single);
    }
}

#[test]
fn batch_query_preserves_independent_holdout_order_and_equatorial_values() {
    let backend = JplSnapshotBackend;
    let entries =
        independent_holdout_snapshot_entries().expect("independent hold-out entries should exist");
    let requests = independent_holdout_snapshot_requests(CoordinateFrame::Equatorial)
        .expect("independent hold-out requests should exist");

    let results = backend
        .positions(&requests)
        .expect("equatorial batch query should resolve the independent hold-out rows");

    assert_eq!(results.len(), entries.len());
    for ((entry, request), batch_result) in entries.iter().zip(requests.iter()).zip(results.iter())
    {
        assert_eq!(batch_result.body, entry.body);
        assert_eq!(batch_result.instant, entry.epoch);
        assert_eq!(batch_result.frame, request.frame);
        assert_eq!(batch_result.zodiac_mode, request.zodiac_mode);
        assert_eq!(batch_result.apparent, request.apparent);
        let expected_equatorial = batch_result
            .ecliptic
            .expect("equatorial requests should still populate ecliptic coordinates")
            .to_equatorial(batch_result.instant.mean_obliquity());
        let equatorial = batch_result
            .equatorial
            .expect("equatorial coordinates should be present for the hold-out rows");
        assert_eq!(equatorial, expected_equatorial);
        assert!(equatorial.right_ascension.degrees().is_finite());
        assert!(equatorial.declination.degrees().is_finite());
        let single = backend
            .position(request)
            .expect("single query should match the independent hold-out equatorial batch path");
        assert_eq!(batch_result, &single);
    }
}

#[test]
fn batch_query_preserves_independent_holdout_mixed_scale_order_and_single_query_parity() {
    let summary = independent_holdout_snapshot_batch_parity_summary()
        .expect("independent hold-out batch parity summary should exist");
    assert_eq!(summary.snapshot.row_count, 66);
    assert_eq!(summary.snapshot.body_count, 16);
    assert_eq!(summary.tt_request_count, 33);
    assert_eq!(summary.tdb_request_count, 33);
    assert!(summary.parity_preserved);
    assert_eq!(
        summary.exact_count
            + summary.interpolated_count
            + summary.approximate_count
            + summary.unknown_count,
        summary.snapshot.row_count,
    );
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn independent_holdout_snapshot_batch_parity_summary_validation_rejects_parity_loss() {
    let mut summary = independent_holdout_snapshot_batch_parity_summary()
        .expect("independent hold-out batch parity summary should exist");
    summary.parity_preserved = false;

    assert!(matches!(
        summary.validate(),
        Err(IndependentHoldoutSnapshotBatchParitySummaryValidationError::ParityNotPreserved)
    ));
}

#[test]
fn independent_holdout_snapshot_batch_parity_summary_validation_rejects_degenerate_time_scale_mix()
{
    let mut summary = independent_holdout_snapshot_batch_parity_summary()
        .expect("independent hold-out batch parity summary should exist");
    summary.tt_request_count = summary.snapshot.row_count;
    summary.tdb_request_count = 0;

    assert!(matches!(
        summary.validate(),
        Err(IndependentHoldoutSnapshotBatchParitySummaryValidationError::TimeScaleMixMissing {
            tt_request_count,
            tdb_request_count,
        }) if tt_request_count == summary.snapshot.row_count && tdb_request_count == 0
    ));
}

#[test]
fn independent_holdout_snapshot_manifest_parses_the_documented_header_comments() {
    let manifest = independent_holdout_snapshot_manifest();
    assert_eq!(
        manifest.title.as_deref(),
        Some("Independent JPL Horizons hold-out snapshot used only for interpolation validation.")
    );
    assert_eq!(
        manifest.source.as_deref(),
        Some("NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables."),
    );
    assert_eq!(
            manifest.coverage.as_deref(),
            Some("major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs."),
        );
    assert_eq!(
        manifest.redistribution.as_deref(),
        Some("repository-checked regression fixtures, not a broad public corpus."),
    );
    assert_eq!(
        manifest.columns,
        ["epoch_jd", "body", "x_km", "y_km", "z_km"],
    );
    assert_eq!(manifest.validate(), Ok(()));
}

#[test]
fn snapshot_manifest_footprint_validation_matches_the_current_reference_and_holdout_corpora() {
    assert_eq!(
        validate_snapshot_manifest_footprint("reference snapshot", snapshot_entries(), 277, 16, 23,),
        Ok(())
    );
    assert_eq!(
        validate_snapshot_manifest_footprint(
            "independent hold-out snapshot",
            independent_holdout_snapshot_entries(),
            66,
            16,
            12,
        ),
        Ok(())
    );
}

#[test]
fn independent_holdout_summary_validation_rejects_inconsistent_ranges() {
    let mut summary = jpl_independent_holdout_summary().expect("summary should exist");
    summary.epoch_count = 0;
    assert_eq!(
        summary.validate(),
        Err(JplInterpolationQualitySummaryValidationError::MissingEpochs)
    );

    let mut summary = jpl_independent_holdout_summary().expect("summary should exist");
    summary.earliest_epoch = Instant::new(JulianDay::from_days(2_600_000.0), TimeScale::Tdb);
    summary.latest_epoch = Instant::new(JulianDay::from_days(2_500_000.0), TimeScale::Tdb);
    assert_eq!(
        summary.validate(),
        Err(
            JplInterpolationQualitySummaryValidationError::InvalidEpochRange {
                earliest_epoch: summary.earliest_epoch,
                latest_epoch: summary.latest_epoch,
            }
        )
    );
}

#[test]
fn independent_holdout_summary_validation_rejects_blank_bodies() {
    let mut summary = jpl_independent_holdout_summary().expect("summary should exist");
    summary.bodies[1].clear();
    assert_eq!(
        summary.validate(),
        Err(JplInterpolationQualitySummaryValidationError::BlankBody { index: 1 })
    );
}

#[test]
fn independent_holdout_summary_validation_rejects_blank_peak_bodies() {
    let mut summary = jpl_independent_holdout_summary().expect("summary should exist");
    summary.max_distance_error_body.clear();
    assert_eq!(
        summary.validate(),
        Err(
            JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                field: "max_distance_error_body",
            }
        )
    );
}

#[test]
fn independent_holdout_summary_validation_rejects_derived_summary_drift() {
    let mut summary = jpl_independent_holdout_summary().expect("summary should exist");
    summary.max_distance_error_au += 1e-12;
    assert_eq!(
        summary.validate(),
        Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
    );
}

#[test]
fn independent_holdout_summary_validated_summary_line_rejects_drift() {
    let mut summary = jpl_independent_holdout_summary().expect("summary should exist");
    summary.sample_count += 1;
}
