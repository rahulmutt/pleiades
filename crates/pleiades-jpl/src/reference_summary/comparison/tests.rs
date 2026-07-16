//! Tests for the comparison module.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use pleiades_backend::{Apparentness, EphemerisErrorKind, EphemerisRequest};
#[allow(unused_imports)]
use pleiades_backend::{CelestialBody, EphemerisBackend, QualityAnnotation};
#[allow(unused_imports)]
use pleiades_types::CoordinateFrame;

#[test]
fn comparison_snapshot_summary_reports_the_expected_coverage() {
    let summary = comparison_snapshot_summary().expect("comparison snapshot summary should exist");
    assert_eq!(summary.row_count, 162);
    assert_eq!(summary.body_count, 10);
    assert_eq!(summary.epoch_count, 18);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_415_020.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_453_000.5);
    assert_eq!(summary.bodies.as_slice(), comparison_bodies());
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn comparison_snapshot_body_class_coverage_summary_reports_the_expected_windows() {
    let summary = comparison_snapshot_body_class_coverage_summary()
        .expect("comparison snapshot body-class coverage summary should exist");

    assert_eq!(summary.row_count, 162);
    assert_eq!(summary.bodies.as_slice(), comparison_bodies());
    assert_eq!(summary.epoch_count, 18);
    assert_eq!(summary.windows.len(), summary.bodies.len());
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn comparison_snapshot_requests_preserve_row_order_and_tt_frame() {
    let requests = comparison_snapshot_requests(CoordinateFrame::Ecliptic)
        .expect("comparison snapshot requests should exist");
    let entries = comparison_snapshot();

    assert!(!entries
        .iter()
        .any(|entry| entry.epoch.julian_day.days() == 2_451_913.5));
    assert_eq!(requests.len(), entries.len());
    for (request, entry) in requests.iter().zip(entries.iter()) {
        assert_eq!(request.body, entry.body);
        assert_eq!(request.instant.julian_day, entry.epoch.julian_day);
        assert_eq!(request.instant.scale, TimeScale::Tt);
        assert_eq!(request.frame, CoordinateFrame::Ecliptic);
        assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
        assert_eq!(request.apparent, Apparentness::Mean);
        assert!(request.observer.is_none());
    }
}

#[test]
fn comparison_snapshot_equatorial_parity_requests_remain_the_explicit_alias() {
    assert_eq!(
        comparison_snapshot_equatorial_parity_requests(),
        comparison_snapshot_requests(CoordinateFrame::Equatorial)
    );
    assert_eq!(
        comparison_snapshot_equatorial_request_corpus(),
        comparison_snapshot_equatorial_parity_requests()
    );
    assert_eq!(
        comparison_snapshot_equatorial_requests(),
        comparison_snapshot_equatorial_request_corpus()
    );
    assert_eq!(
        comparison_snapshot_equatorial_parity_request_corpus(),
        comparison_snapshot_equatorial_parity_requests()
    );
}

#[test]
fn comparison_snapshot_batch_parity_summary_reports_the_expected_coverage() {
    let summary = comparison_snapshot_batch_parity_summary()
        .expect("comparison snapshot batch parity summary should exist");
    assert_eq!(summary.snapshot.row_count, 162);
    assert_eq!(summary.snapshot.body_count, 10);
    assert_eq!(summary.snapshot.epoch_count, 18);
    assert_eq!(
        summary.snapshot.earliest_epoch.julian_day.days(),
        2_415_020.5
    );
    assert_eq!(summary.snapshot.latest_epoch.julian_day.days(), 2_453_000.5);
    assert_eq!(summary.snapshot.bodies.as_slice(), comparison_bodies());
    assert_eq!(summary.ecliptic_request_count, 81);
    assert_eq!(summary.equatorial_request_count, 81);
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn comparison_snapshot_batch_parity_summary_validation_rejects_request_count_mismatches() {
    let mut summary = comparison_snapshot_batch_parity_summary()
        .expect("comparison snapshot batch parity summary should exist");
    summary.equatorial_request_count += 1;

    assert!(matches!(
        summary.validate(),
        Err(ComparisonSnapshotBatchParitySummaryValidationError::RequestCountMismatch { .. })
    ));
}

#[test]
fn comparison_snapshot_batch_parity_requests_preserve_the_mixed_frame_slice() {
    let requests = comparison_snapshot_batch_parity_requests()
        .expect("comparison snapshot batch parity requests should exist");
    let entries = comparison_snapshot();

    assert_eq!(requests.len(), entries.len());
    for (index, (request, entry)) in requests.iter().zip(entries.iter()).enumerate() {
        assert_eq!(request.body, entry.body);
        assert_eq!(request.instant.julian_day, entry.epoch.julian_day);
        assert_eq!(request.instant.scale, TimeScale::Tt);
        assert_eq!(
            request.frame,
            if index % 2 == 0 {
                CoordinateFrame::Ecliptic
            } else {
                CoordinateFrame::Equatorial
            }
        );
        assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
        assert_eq!(request.apparent, Apparentness::Mean);
        assert!(request.observer.is_none());
    }
}

#[test]
fn comparison_snapshot_mixed_time_scale_batch_parity_requests_preserve_the_ecliptic_slice() {
    let requests = comparison_snapshot_mixed_time_scale_batch_parity_requests()
        .expect("comparison snapshot mixed TT/TDB batch parity requests should exist");
    let entries = comparison_snapshot();

    assert_eq!(requests.len(), entries.len());
    for (index, (request, entry)) in requests.iter().zip(entries.iter()).enumerate() {
        assert_eq!(request.body, entry.body);
        assert_eq!(request.instant.julian_day, entry.epoch.julian_day);
        assert_eq!(
            request.instant.scale,
            if index % 2 == 0 {
                TimeScale::Tt
            } else {
                TimeScale::Tdb
            }
        );
        assert_eq!(request.frame, CoordinateFrame::Ecliptic);
        assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
        assert_eq!(request.apparent, Apparentness::Mean);
        assert!(request.observer.is_none());
    }
}

#[test]
fn comparison_snapshot_summary_validation_rejects_duplicate_bodies() {
    let summary = ComparisonSnapshotSummary {
        row_count: 2,
        body_count: 2,
        bodies: vec![
            pleiades_backend::CelestialBody::Moon,
            pleiades_backend::CelestialBody::Moon,
        ],
        epoch_count: 1,
        earliest_epoch: reference_instant(),
        latest_epoch: reference_instant(),
    };
    assert!(matches!(
        summary.validate(),
        Err(ComparisonSnapshotSummaryValidationError::DuplicateBody {
            first_index: 0,
            second_index: 1,
            body,
        }) if body == "Moon"
    ));
}

#[test]
fn comparison_snapshot_summary_validation_rejects_missing_rows() {
    let mut summary = comparison_snapshot_summary().expect("comparison summary should exist");
    summary.row_count = 0;

    assert_eq!(
        summary.validate(),
        Err(ComparisonSnapshotSummaryValidationError::MissingRows)
    );
}

#[test]
fn comparison_snapshot_summary_validation_rejects_missing_bodies() {
    let mut summary = comparison_snapshot_summary().expect("comparison summary should exist");
    summary.row_count = 1;
    summary.body_count = 0;
    summary.bodies.clear();

    assert_eq!(
        summary.validate(),
        Err(ComparisonSnapshotSummaryValidationError::MissingBodies)
    );
}

#[test]
fn comparison_snapshot_summary_validation_rejects_invalid_epoch_range() {
    let mut summary = comparison_snapshot_summary().expect("comparison summary should exist");
    summary.earliest_epoch = pleiades_types::Instant::new(
        summary.latest_epoch.julian_day.add_seconds(1.0),
        summary.latest_epoch.scale,
    );

    assert!(matches!(
        summary.validate(),
        Err(ComparisonSnapshotSummaryValidationError::InvalidEpochRange {
            earliest_epoch,
            latest_epoch,
        }) if earliest_epoch == summary.earliest_epoch && latest_epoch == summary.latest_epoch
    ));
}

#[test]
fn comparison_snapshot_summary_validation_rejects_body_order_mismatch() {
    let mut summary = comparison_snapshot_summary().expect("comparison summary should exist");
    summary.bodies.swap(0, 1);
    let expected = comparison_body_list()[0].to_string();
    let found = comparison_body_list()[1].to_string();

    assert!(matches!(
        summary.validate(),
        Err(ComparisonSnapshotSummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: actual_expected,
            found: actual_found,
        }) if actual_expected == expected && actual_found == found
    ));
}

#[test]
fn comparison_snapshot_manifest_parses_the_documented_header_comments() {
    let manifest = comparison_snapshot_manifest();
    let source_summary = comparison_snapshot_source_summary();
    assert_eq!(
        manifest.title.as_deref(),
        Some("JPL Horizons reference snapshot.")
    );
    assert_eq!(
        manifest.source.as_deref(),
        Some("NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.")
    );
    assert_eq!(
            manifest.coverage.as_deref(),
            Some("Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.")
        );
    assert_eq!(
        manifest.redistribution.as_deref(),
        Some("repository-checked regression fixtures, not a broad public corpus.")
    );
    assert_eq!(manifest.columns, ["body", "x_km", "y_km", "z_km"]);
    assert_eq!(manifest.validate(), Ok(()));
    assert_eq!(source_summary.validate(), Ok(()));
    let source_window_summary = comparison_snapshot_source_window_summary()
        .expect("comparison snapshot source window summary should exist");
    assert_eq!(source_window_summary.validate(), Ok(()));
}

#[test]
fn comparison_snapshot_source_summary_validation_reports_blank_fields() {
    let blank_source = ComparisonSnapshotSourceSummary {
        source: " ".to_string(),
        coverage: "coverage".to_string(),
        redistribution: COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED.to_string(),
        columns: "body, x_km, y_km, z_km".to_string(),
        checksum: comparison_snapshot_source_checksum(),
    };
    assert_eq!(
        blank_source.validate(),
        Err(ComparisonSnapshotSourceSummaryValidationError::BlankSource)
    );

    let blank_coverage = ComparisonSnapshotSourceSummary {
        source: COMPARISON_SNAPSHOT_SOURCE_EXPECTED.to_string(),
        coverage: "	".to_string(),
        redistribution: COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED.to_string(),
        columns: COMPARISON_SNAPSHOT_COLUMNS.to_string(),
        checksum: comparison_snapshot_source_checksum(),
    };
    assert_eq!(
        blank_coverage.validate(),
        Err(ComparisonSnapshotSourceSummaryValidationError::BlankCoverage)
    );

    let blank_redistribution = ComparisonSnapshotSourceSummary {
        source: COMPARISON_SNAPSHOT_SOURCE_EXPECTED.to_string(),
        coverage: COMPARISON_SNAPSHOT_COVERAGE_EXPECTED.to_string(),
        redistribution: " 	".to_string(),
        columns: COMPARISON_SNAPSHOT_COLUMNS.to_string(),
        checksum: comparison_snapshot_source_checksum(),
    };
    assert_eq!(
        blank_redistribution.validate(),
        Err(ComparisonSnapshotSourceSummaryValidationError::BlankRedistribution)
    );

    let blank_columns = ComparisonSnapshotSourceSummary {
        source: COMPARISON_SNAPSHOT_SOURCE_EXPECTED.to_string(),
        coverage: COMPARISON_SNAPSHOT_COVERAGE_EXPECTED.to_string(),
        redistribution: COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED.to_string(),
        columns: "  ".to_string(),
        checksum: comparison_snapshot_source_checksum(),
    };
    assert_eq!(
        blank_columns.validate(),
        Err(ComparisonSnapshotSourceSummaryValidationError::BlankColumns)
    );

    let padded_source = ComparisonSnapshotSourceSummary {
        source: " source".to_string(),
        coverage: COMPARISON_SNAPSHOT_COVERAGE_EXPECTED.to_string(),
        redistribution: COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED.to_string(),
        columns: COMPARISON_SNAPSHOT_COLUMNS.to_string(),
        checksum: comparison_snapshot_source_checksum(),
    };
    assert_eq!(
        padded_source.validate(),
        Err(
            ComparisonSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                field: "source",
            }
        )
    );

    let multiline_columns = ComparisonSnapshotSourceSummary {
        source: COMPARISON_SNAPSHOT_SOURCE_EXPECTED.to_string(),
        coverage: COMPARISON_SNAPSHOT_COVERAGE_EXPECTED.to_string(),
        redistribution: COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED.to_string(),
        columns: "body,\nx_km, y_km, z_km".to_string(),
        checksum: comparison_snapshot_source_checksum(),
    };
    assert_eq!(
        multiline_columns.validate(),
        Err(
            ComparisonSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                field: "columns",
            }
        )
    );

    let redistribution_drift = ComparisonSnapshotSourceSummary {
        source: COMPARISON_SNAPSHOT_SOURCE_EXPECTED.to_string(),
        coverage: COMPARISON_SNAPSHOT_COVERAGE_EXPECTED.to_string(),
        redistribution: "repository-checked regression fixtures".to_string(),
        columns: COMPARISON_SNAPSHOT_COLUMNS.to_string(),
        checksum: comparison_snapshot_source_checksum(),
    };
    assert_eq!(
        redistribution_drift.validate(),
        Err(
            ComparisonSnapshotSourceSummaryValidationError::FieldOutOfSync {
                field: "redistribution",
            }
        )
    );

    let checksum_drift = ComparisonSnapshotSourceSummary {
        source: COMPARISON_SNAPSHOT_SOURCE_EXPECTED.to_string(),
        coverage: COMPARISON_SNAPSHOT_COVERAGE_EXPECTED.to_string(),
        redistribution: COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED.to_string(),
        columns: COMPARISON_SNAPSHOT_COLUMNS.to_string(),
        checksum: comparison_snapshot_source_checksum() ^ 0x1,
    };
    assert_eq!(
        checksum_drift.validate(),
        Err(ComparisonSnapshotSourceSummaryValidationError::ChecksumMismatch)
    );
}

#[test]
fn comparison_snapshot_source_window_summary_reports_the_expected_body_windows() {
    let summary = comparison_snapshot_source_window_summary()
        .expect("comparison snapshot source window summary should exist");
    assert_eq!(summary.sample_count, 162);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch_count, 18);
    assert_eq!(summary.windows.len(), 10);
    assert_eq!(summary.validate(), Ok(()));
}

#[test]
fn comparison_snapshot_source_window_summary_validation_rejects_drift() {
    let mut summary = comparison_snapshot_source_window_summary()
        .expect("comparison snapshot source window summary should exist");
    summary.sample_count += 1;
    assert_eq!(
        summary.validate(),
        Err(
            ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "sample_count"
            }
        )
    );
}

#[test]
fn comparison_snapshot_source_window_summary_validation_rejects_body_order_drift() {
    let mut summary = comparison_snapshot_source_window_summary()
        .expect("comparison snapshot source window summary should exist");
    summary.sample_bodies.swap(0, 1);

    assert_eq!(
        summary.validate(),
        Err(
            ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    );
}

#[test]
fn comparison_snapshot_manifest_summary_uses_the_current_manifest() {
    let summary = comparison_snapshot_manifest_summary();

    assert_eq!(
            summary.validate_with_expected_metadata(
                "JPL Horizons reference snapshot.",
                "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.",
                "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.",
                &["body", "x_km", "y_km", "z_km"],
            ),
            Ok(())
        );
}

#[test]
fn comparison_snapshot_manifest_summary_validation_rejects_metadata_drift() {
    let mut summary = comparison_snapshot_manifest_summary();
    summary.manifest.coverage = Some("Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000, plus drift".to_string());

    assert_eq!(
            summary.validate_with_expected_metadata(
                "JPL Horizons reference snapshot.",
                "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.",
                "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.",
                &["body", "x_km", "y_km", "z_km"],
            ),
            Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "coverage",
                expected: "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.".to_string(),
                found: "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000, plus drift".to_string(),
            })
        );
}

#[test]
fn comparison_snapshot_manifest_summary_validation_rejects_redistribution_drift() {
    let mut summary = comparison_snapshot_manifest_summary();
    summary.manifest.redistribution = Some("drifted redistribution posture".to_string());

    assert_eq!(
            summary.validate_with_expected_metadata_and_redistribution(
                "JPL Horizons reference snapshot.",
                "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.",
                "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.",
                COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED,
                &["body", "x_km", "y_km", "z_km"],
            ),
            Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "redistribution",
                expected: COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED.to_string(),
                found: "drifted redistribution posture".to_string(),
            })
        );
}

#[test]
fn comparison_snapshot_manifest_summary_validation_rejects_padded_label() {
    let mut summary = comparison_snapshot_manifest_summary();
    summary.label = " Comparison snapshot manifest ";

    assert_eq!(
        summary.validate(),
        Err(SnapshotManifestSummaryValidationError::SurroundedByWhitespace { field: "label" })
    );

    summary.label = "Comparison snapshot manifest\nrelease";
    assert_eq!(
        summary.validate(),
        Err(SnapshotManifestSummaryValidationError::SurroundedByWhitespace { field: "label" })
    );
}
