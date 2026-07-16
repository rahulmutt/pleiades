//! Tests for the backend module.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use pleiades_backend::{Apparentness, EphemerisErrorKind, EphemerisRequest};
#[allow(unused_imports)]
use pleiades_backend::{CelestialBody, EphemerisBackend, QualityAnnotation};
#[allow(unused_imports)]
use pleiades_types::CoordinateFrame;

#[test]
fn snapshot_manifest_validation_reports_missing_required_metadata() {
    let manifest = SnapshotManifest {
        title: Some(" ".to_string()),
        source: None,
        coverage: Some("ignored".to_string()),
        redistribution: None,
        columns: vec!["body".to_string(), "".to_string()],
    };

    assert_eq!(
        manifest.validate(),
        Err(SnapshotManifestValidationError::MissingTitle)
    );

    let manifest = SnapshotManifest {
        title: Some("Example snapshot.".to_string()),
        source: Some(" ".to_string()),
        coverage: None,
        redistribution: None,
        columns: vec!["body".to_string()],
    };

    assert_eq!(
        manifest.validate(),
        Err(SnapshotManifestValidationError::MissingSource)
    );

    let manifest = SnapshotManifest {
        title: Some("Example snapshot.".to_string()),
        source: Some("Example source".to_string()),
        coverage: None,
        redistribution: None,
        columns: Vec::new(),
    };

    assert_eq!(
        manifest.validate(),
        Err(SnapshotManifestValidationError::MissingColumns)
    );

    let manifest = SnapshotManifest {
        title: Some("Example snapshot.".to_string()),
        source: Some("Example source".to_string()),
        coverage: Some(" ".to_string()),
        redistribution: None,
        columns: vec!["body".to_string()],
    };

    assert_eq!(
        manifest.validate(),
        Err(SnapshotManifestValidationError::BlankCoverage)
    );
    assert_eq!(
        SnapshotManifestValidationError::BlankCoverage.to_string(),
        "blank coverage"
    );
    let manifest = SnapshotManifest {
        title: Some("Example snapshot.".to_string()),
        source: Some("Example source".to_string()),
        coverage: None,
        redistribution: Some(" ".to_string()),
        columns: vec!["body".to_string()],
    };

    assert_eq!(
        manifest.validate(),
        Err(SnapshotManifestValidationError::BlankRedistribution)
    );
    assert_eq!(
        SnapshotManifestValidationError::BlankRedistribution.to_string(),
        "blank redistribution"
    );

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
    assert_eq!(
        SnapshotManifestValidationError::SurroundedByWhitespace { field: "title" }.to_string(),
        "title contains surrounding whitespace"
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

    let manifest = SnapshotManifest {
        title: Some("Example snapshot.".to_string()),
        source: Some("Example source".to_string()),
        coverage: None,
        redistribution: None,
        columns: vec!["body".to_string(), "".to_string()],
    };

    assert_eq!(
        manifest.validate(),
        Err(SnapshotManifestValidationError::BlankColumn { index: 1 })
    );
    assert_eq!(
        SnapshotManifestValidationError::BlankColumn { index: 1 }.to_string(),
        "blank column at index 1"
    );

    let manifest = SnapshotManifest {
        title: Some("Example snapshot.".to_string()),
        source: Some("Example source".to_string()),
        coverage: None,
        redistribution: None,
        columns: vec!["body".to_string(), "body".to_string()],
    };

    assert_eq!(
        manifest.validate(),
        Err(SnapshotManifestValidationError::DuplicateColumn {
            first_index: 0,
            second_index: 1,
            name: "body".to_string(),
        })
    );
    assert_eq!(
        SnapshotManifestValidationError::DuplicateColumn {
            first_index: 0,
            second_index: 1,
            name: "body".to_string(),
        }
        .to_string(),
        "duplicate column 'body' at index 1 (first seen at index 0)"
    );
}

#[test]
fn parsed_manifest_preserves_blank_coverage_for_validation() {
    let manifest = parse_snapshot_manifest(
        "# Example snapshot.\n# Source: Example source\n# Coverage:   \n# Columns: body\n",
    );

    assert_eq!(manifest.title.as_deref(), Some("Example snapshot."));
    assert_eq!(manifest.source.as_deref(), Some("Example source"));
    assert_eq!(manifest.coverage.as_deref(), Some(""));
    assert_eq!(manifest.columns, ["body"]);
    assert_eq!(
        manifest.validate(),
        Err(SnapshotManifestValidationError::BlankCoverage)
    );
}

#[test]
fn parse_snapshot_entries_and_manifest_round_trip_the_checked_in_reference_snapshot() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/reference_snapshot.csv"
    ));

    assert_eq!(
        parse_snapshot_manifest(source),
        reference_snapshot_manifest().clone()
    );
    assert_eq!(
        parse_snapshot_entries(source).unwrap(),
        reference_snapshot()
    );
}

#[test]
fn parse_snapshot_corpus_round_trips_the_checked_in_reference_snapshot() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/reference_snapshot.csv"
    ));

    let corpus = parse_snapshot_corpus(source).expect("reference snapshot corpus should parse");
    assert_eq!(corpus.manifest, reference_snapshot_manifest().clone());
    assert_eq!(corpus.entries, reference_snapshot());
    assert_eq!(
        load_snapshot_from_csv(source).unwrap(),
        reference_snapshot()
    );
    assert_eq!(
        corpus.into_parts(),
        (
            reference_snapshot_manifest().clone(),
            reference_snapshot().to_vec()
        )
    );
}

#[test]
fn parse_snapshot_corpus_from_sources_round_trips_split_manifest_and_rows() {
    let manifest_source = "\
# Split JPL snapshot.
# Source: Example source
# Coverage: Example coverage
# Redistribution: Example redistribution
# Columns: epoch_jd,body,x_km,y_km,z_km
";
    let rows_source = "\
2451545.0,Sun,1.0,2.0,3.0
2451546.0,Moon,4.0,5.0,6.0
";

    let corpus = parse_snapshot_corpus_from_sources(manifest_source, rows_source)
        .expect("split snapshot corpus should parse");
    assert_eq!(
        corpus.manifest.title.as_deref(),
        Some("Split JPL snapshot.")
    );
    assert_eq!(corpus.manifest.source.as_deref(), Some("Example source"));
    assert_eq!(
        corpus.manifest.coverage.as_deref(),
        Some("Example coverage")
    );
    assert_eq!(
        corpus.manifest.redistribution.as_deref(),
        Some("Example redistribution")
    );
    assert_eq!(
        corpus.manifest.columns,
        ["epoch_jd", "body", "x_km", "y_km", "z_km"]
    );
    assert_eq!(corpus.entries.len(), 2);
    assert_eq!(corpus.entries[0].body, pleiades_backend::CelestialBody::Sun);
    assert_eq!(
        corpus.entries[1].body,
        pleiades_backend::CelestialBody::Moon
    );

    let sources = SnapshotCorpusSources {
        manifest: manifest_source,
        entries: rows_source,
    };
    assert_eq!(sources.parse().expect("split sources should parse"), corpus);
}

#[test]
fn load_snapshot_corpus_from_paths_round_trips_split_manifest_and_rows() {
    let temp_root = std::env::temp_dir().join(format!(
        "pleiades-jpl-snapshot-corpus-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after UNIX epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_root).expect("temp dir should be created");

    let manifest_path = temp_root.join("manifest.csv");
    let rows_path = temp_root.join("rows.csv");
    let manifest_source = "\
# Split JPL snapshot.
# Source: Example source
# Coverage: Example coverage
# Redistribution: Example redistribution
# Columns: epoch_jd,body,x_km,y_km,z_km
";
    let rows_source = "\
2451545.0,Sun,1.0,2.0,3.0
2451546.0,Moon,4.0,5.0,6.0
";

    std::fs::write(&manifest_path, manifest_source).expect("manifest file should be written");
    std::fs::write(&rows_path, rows_source).expect("rows file should be written");

    let loaded = load_snapshot_corpus_from_paths(&manifest_path, &rows_path)
        .expect("path-backed split snapshot corpus should load");
    let parsed = parse_snapshot_corpus_from_sources(manifest_source, rows_source)
        .expect("split snapshot corpus should parse");
    assert_eq!(loaded, parsed);

    let path_sources = SnapshotCorpusPathSources {
        manifest: manifest_path.clone(),
        entries: rows_path.clone(),
    };
    assert_eq!(
        path_sources.load().expect("split sources should load"),
        parsed
    );

    std::fs::remove_dir_all(&temp_root).expect("temp dir should be removed");
}

#[test]
fn load_snapshot_corpus_from_paths_reports_io_errors() {
    let temp_root = std::env::temp_dir().join(format!(
        "pleiades-jpl-snapshot-corpus-missing-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after UNIX epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_root).expect("temp dir should be created");

    let manifest_path = temp_root.join("manifest.csv");
    let rows_path = temp_root.join("rows.csv");
    std::fs::write(
        &manifest_path,
        "# Split JPL snapshot.\n# Columns: epoch_jd,body,x_km,y_km,z_km\n",
    )
    .expect("manifest file should be written");

    let error = load_snapshot_corpus_from_paths(&manifest_path, &rows_path)
        .expect_err("missing rows file should fail to load");
    assert!(matches!(error, SnapshotCorpusLoadError::EntriesIo { .. }));
    assert!(format!("{error}").contains("failed to read snapshot rows"));

    std::fs::remove_dir_all(&temp_root).expect("temp dir should be removed");
}

#[test]
fn parse_snapshot_entries_rejects_duplicate_rows() {
    let source = "# Example snapshot.\n# Source: Example source\n# Coverage: Example coverage\n# Columns: epoch_jd,body,x_km,y_km,z_km\n2451545.0,Sun,1,2,3\n2451545.0,Sun,4,5,6\n";

    let error = parse_snapshot_entries(source).unwrap_err();
    assert_eq!(error.line_number(), 6);
    assert!(matches!(
        error.kind(),
        SnapshotLoadErrorKind::DuplicateEntry { .. }
    ));
    assert_eq!(error.kind().label(), "duplicate entry");
}

#[test]
fn parsed_manifest_preserves_blank_columns_for_validation() {
    let manifest = parse_snapshot_manifest(
        "# Example snapshot.\n# Source: Example source\n# Columns: body, , x_km\n",
    );

    assert_eq!(manifest.title.as_deref(), Some("Example snapshot."));
    assert_eq!(manifest.source.as_deref(), Some("Example source"));
    assert_eq!(manifest.columns, ["body", "", "x_km"]);
    assert_eq!(
        manifest.validate(),
        Err(SnapshotManifestValidationError::BlankColumn { index: 1 })
    );
}

#[test]
fn parsed_manifest_preserves_duplicate_columns_for_validation() {
    let manifest = parse_snapshot_manifest(
        "# Example snapshot.\n# Source: Example source\n# Columns: body, x_km, body\n",
    );

    assert_eq!(manifest.title.as_deref(), Some("Example snapshot."));
    assert_eq!(manifest.source.as_deref(), Some("Example source"));
    assert_eq!(manifest.columns, ["body", "x_km", "body"]);
    assert_eq!(
        manifest.validate(),
        Err(SnapshotManifestValidationError::DuplicateColumn {
            first_index: 0,
            second_index: 2,
            name: "body".to_string(),
        })
    );
}
#[test]
fn manifest_summary_validated_summary_line_rejects_columns_drift() {
    assert_eq!(
            SnapshotManifestSummaryValidationError::ColumnsMismatch {
                expected: "epoch_jd, body, x_km, y_km, z_km".to_string(),
                found: "body, x_km, y_km, z_km".to_string(),
            }
            .to_string(),
            "column schema mismatch: expected epoch_jd, body, x_km, y_km, z_km but found body, x_km, y_km, z_km"
        );
}
#[test]
fn batch_query_preserves_reference_snapshot_order_and_equatorial_values() {
    let backend = JplSnapshotBackend;
    let requests = reference_snapshot_requests(CoordinateFrame::Equatorial)
        .expect("reference snapshot requests should exist");

    let results = backend
        .positions(&requests)
        .expect("batch query should preserve the reference snapshot order");

    assert_eq!(results.len(), requests.len());
    for (entry, result) in reference_snapshot().iter().zip(results.iter()) {
        assert_eq!(result.body, entry.body);
        assert_eq!(result.instant, entry.epoch);
        assert_eq!(result.frame, CoordinateFrame::Equatorial);
        assert_eq!(result.quality, QualityAnnotation::Exact);

        let ecliptic = result
            .ecliptic
            .expect("reference snapshot entries should include ecliptic coordinates");
        assert_eq!(ecliptic, entry.ecliptic());

        let expected_equatorial = ecliptic.to_equatorial(result.instant.mean_obliquity());
        let equatorial = result
            .equatorial
            .expect("equatorial coordinates should be present for equatorial batch requests");
        assert_eq!(equatorial, expected_equatorial);
        assert!(equatorial.right_ascension.degrees().is_finite());
        assert!(equatorial.declination.degrees().is_finite());
    }
}

#[test]
fn batch_query_preserves_mixed_frame_requests_and_values() {
    let backend = JplSnapshotBackend;
    let requests = reference_snapshot_batch_parity_requests()
        .expect("reference snapshot batch parity requests should exist");

    let results = backend
        .positions(&requests)
        .expect("mixed frame batch query should preserve the reference snapshot order");

    assert_eq!(results.len(), requests.len());
    for ((request, result), entry) in requests
        .iter()
        .zip(results.iter())
        .zip(reference_snapshot().iter())
    {
        assert_eq!(result.body, entry.body);
        assert_eq!(result.instant, entry.epoch);
        assert_eq!(result.frame, request.frame);

        let ecliptic = result
            .ecliptic
            .expect("reference snapshot entries should include ecliptic coordinates");
        assert_eq!(ecliptic, entry.ecliptic());

        let expected_equatorial = ecliptic.to_equatorial(result.instant.mean_obliquity());
        let equatorial = result
            .equatorial
            .expect("equatorial coordinates should be present for mixed frame batch requests");
        assert_eq!(equatorial, expected_equatorial);
        assert!(equatorial.right_ascension.degrees().is_finite());
        assert!(equatorial.declination.degrees().is_finite());
    }
}

#[test]
fn snapshot_manifest_header_structure_validation_rejects_comment_block_drift() {
    let duplicate_comment_block = "\
# JPL Horizons reference snapshot.
# Source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.
# Redistribution: repository-checked regression fixtures, not a broad public corpus
# Coverage: major bodies sampled at 1749-12-31 for Sun through Neptune
# Columns: epoch_jd,body,x_km,y_km,z_km
# Coverage: duplicate
2451545.0,Sun,1,2,3
";
    assert!(matches!(
        validate_snapshot_manifest_header_structure(
            duplicate_comment_block,
            "JPL Horizons reference snapshot.",
            "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
            "major bodies sampled at 1749-12-31 for Sun through Neptune",
            Some("repository-checked regression fixtures, not a broad public corpus."),
            &["epoch_jd", "body", "x_km", "y_km", "z_km"],
        ),
        Err(SnapshotManifestHeaderStructureError::CommentCountMismatch {
            expected: 5,
            found: 6,
        })
    ));

    let swapped_comment_block = "\
# JPL Horizons reference snapshot.
# Coverage: major bodies sampled at 1749-12-31 for Sun through Neptune
# Source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.
# Redistribution: repository-checked regression fixtures, not a broad public corpus
# Columns: epoch_jd,body,x_km,y_km,z_km
2451545.0,Sun,1,2,3
";
    assert!(matches!(
        validate_snapshot_manifest_header_structure(
            swapped_comment_block,
            "JPL Horizons reference snapshot.",
            "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
            "major bodies sampled at 1749-12-31 for Sun through Neptune",
            Some("repository-checked regression fixtures, not a broad public corpus."),
            &["epoch_jd", "body", "x_km", "y_km", "z_km"],
        ),
        Err(SnapshotManifestHeaderStructureError::CommentMismatch {
            index: 1,
            field: "source",
            ..
        })
    ));
}

#[test]
fn snapshot_manifest_footprint_validation_rejects_count_drift() {
    let entries = [
        SnapshotEntry {
            body: CelestialBody::Sun,
            epoch: Instant::new(JulianDay::from_days(1.0), TimeScale::Tdb),
            x_km: 1.0,
            y_km: 2.0,
            z_km: 3.0,
            vx_km_s: None,
            vy_km_s: None,
            vz_km_s: None,
        },
        SnapshotEntry {
            body: CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(2.0), TimeScale::Tdb),
            x_km: 4.0,
            y_km: 5.0,
            z_km: 6.0,
            vx_km_s: None,
            vy_km_s: None,
            vz_km_s: None,
        },
    ];

    assert!(matches!(
        validate_snapshot_manifest_footprint("example snapshot", Some(&entries), 3, 2, 2),
        Err(SnapshotManifestFootprintValidationError::RowCountMismatch {
            label: "example snapshot",
            expected: 3,
            found: 2,
        })
    ));
}

#[test]
fn snapshot_manifest_summary_line_uses_provided_defaults() {
    let manifest = SnapshotManifest {
        title: Some("Example manifest.".to_string()),
        ..Default::default()
    };

    assert_eq!(manifest.source_or("fallback source"), "fallback source");
    assert_eq!(
        manifest.coverage_or("fallback coverage"),
        "fallback coverage"
    );
}

#[test]
fn parser_reports_malformed_rows_without_panicking() {
    let error = load_snapshot_from_str("2451545.0,Sun,1.0,2.0\n")
        .expect_err("missing columns should be reported");
    assert!(format!("{error}").contains("missing z"));

    let error = load_snapshot_from_str("2451545.0,Comet,1.0,2.0,3.0\n")
        .expect_err("unsupported bodies should be reported");
    assert!(format!("{error}").contains("unsupported body 'Comet'"));

    let error = load_snapshot_from_str("2451545.0,,1.0,2.0,3.0\n")
        .expect_err("blank bodies should be reported");
    assert!(format!("{error}").contains("blank body"));
}

#[test]
fn parser_rejects_duplicate_body_epoch_rows() {
    let error = load_snapshot_from_str("2451545.0,Sun,1.0,2.0,3.0\n2451545.0,Sun,4.0,5.0,6.0\n")
        .expect_err("duplicate body/epoch pairs should be reported");
    assert!(format!("{error}").contains("line 2"));
    assert!(format!("{error}").contains("duplicate row for body 'Sun'"));
    assert!(format!("{error}").contains("first seen at line 1"));
    assert!(format!("{error}").contains("JD 2451545.0 (TDB)"));
}

#[test]
fn parser_accepts_custom_catalog_bodies() {
    let snapshot = load_snapshot_from_str("2451545.0,asteroid:433-Eros,-1.0,-2.0,-3.0\n")
        .expect("custom catalog bodies should parse");
    assert_eq!(snapshot.len(), 1);
    assert_eq!(
        snapshot[0].body,
        pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
    );
}

#[test]
fn quadratic_interpolation_matches_a_known_parabola() {
    let a = SnapshotEntry {
        body: pleiades_backend::CelestialBody::Moon,
        epoch: Instant::new(JulianDay::from_days(0.0), TimeScale::Tdb),
        x_km: 0.0,
        y_km: 1.0,
        z_km: 2.0,
        vx_km_s: None,
        vy_km_s: None,
        vz_km_s: None,
    };
    let b = SnapshotEntry {
        body: pleiades_backend::CelestialBody::Moon,
        epoch: Instant::new(JulianDay::from_days(1.0), TimeScale::Tdb),
        x_km: 1.0,
        y_km: 6.0,
        z_km: 5.0,
        vx_km_s: None,
        vy_km_s: None,
        vz_km_s: None,
    };
    let c = SnapshotEntry {
        body: pleiades_backend::CelestialBody::Moon,
        epoch: Instant::new(JulianDay::from_days(2.0), TimeScale::Tdb),
        x_km: 4.0,
        y_km: 15.0,
        z_km: 10.0,
        vx_km_s: None,
        vy_km_s: None,
        vz_km_s: None,
    };

    let interpolated = SnapshotEntry::interpolate_quadratic(&a, &b, &c, 1.5);
    assert!((interpolated.x_km - 2.25).abs() < 1e-12);
    assert!((interpolated.y_km - 10.0).abs() < 1e-12);
    assert!((interpolated.z_km - 7.25).abs() < 1e-12);
}

#[test]
fn cubic_interpolation_matches_a_known_cubic() {
    let a = SnapshotEntry {
        body: pleiades_backend::CelestialBody::Moon,
        epoch: Instant::new(JulianDay::from_days(0.0), TimeScale::Tdb),
        x_km: 0.0,
        y_km: 1.0,
        z_km: 2.0,
        vx_km_s: None,
        vy_km_s: None,
        vz_km_s: None,
    };
    let b = SnapshotEntry {
        body: pleiades_backend::CelestialBody::Moon,
        epoch: Instant::new(JulianDay::from_days(1.0), TimeScale::Tdb),
        x_km: 1.0,
        y_km: 2.0,
        z_km: 3.0,
        vx_km_s: None,
        vy_km_s: None,
        vz_km_s: None,
    };
    let c = SnapshotEntry {
        body: pleiades_backend::CelestialBody::Moon,
        epoch: Instant::new(JulianDay::from_days(2.0), TimeScale::Tdb),
        x_km: 8.0,
        y_km: 9.0,
        z_km: 10.0,
        vx_km_s: None,
        vy_km_s: None,
        vz_km_s: None,
    };
    let d = SnapshotEntry {
        body: pleiades_backend::CelestialBody::Moon,
        epoch: Instant::new(JulianDay::from_days(3.0), TimeScale::Tdb),
        x_km: 27.0,
        y_km: 28.0,
        z_km: 29.0,
        vx_km_s: None,
        vy_km_s: None,
        vz_km_s: None,
    };

    let interpolated = SnapshotEntry::interpolate_cubic(&a, &b, &c, &d, 1.5);
    assert!((interpolated.x_km - 3.375).abs() < 1e-12);
    assert!((interpolated.y_km - 4.375).abs() < 1e-12);
    assert!((interpolated.z_km - 5.375).abs() < 1e-12);
}

#[test]
fn interpolation_uses_a_cubic_window_when_four_points_are_available() {
    let entries = [
        SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(0.0), TimeScale::Tdb),
            x_km: 0.0,
            y_km: 1.0,
            z_km: 2.0,
            vx_km_s: None,
            vy_km_s: None,
            vz_km_s: None,
        },
        SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(1.0), TimeScale::Tdb),
            x_km: 1.0,
            y_km: 2.0,
            z_km: 3.0,
            vx_km_s: None,
            vy_km_s: None,
            vz_km_s: None,
        },
        SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(2.0), TimeScale::Tdb),
            x_km: 8.0,
            y_km: 9.0,
            z_km: 10.0,
            vx_km_s: None,
            vy_km_s: None,
            vz_km_s: None,
        },
        SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(3.0), TimeScale::Tdb),
            x_km: 27.0,
            y_km: 28.0,
            z_km: 29.0,
            vx_km_s: None,
            vy_km_s: None,
            vz_km_s: None,
        },
    ];

    let interpolated =
        interpolate_fixture_state(&entries, pleiades_backend::CelestialBody::Moon, 1.5)
            .expect("four fixture points should produce an interpolated state");
    assert!((interpolated.x_km - 3.375).abs() < 1e-12);
    assert!((interpolated.y_km - 4.375).abs() < 1e-12);
    assert!((interpolated.z_km - 5.375).abs() < 1e-12);
}

#[test]
fn j2000_sun_position_is_finite() {
    let backend = JplSnapshotBackend;
    let request = EphemerisRequest {
        body: pleiades_backend::CelestialBody::Sun,
        instant: reference_instant(),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };

    let result = backend
        .position(&request)
        .expect("reference snapshot should resolve");
    let ecliptic = result
        .ecliptic
        .expect("reference snapshot should include ecliptic coordinates");
    assert!(ecliptic.longitude.degrees().is_finite());
    assert!(ecliptic.latitude.degrees().is_finite());
    assert!(ecliptic
        .distance_au
        .expect("distance should be present")
        .is_finite());
}

#[test]
fn j2000_equatorial_request_is_supported() {
    let backend = JplSnapshotBackend;
    let request = EphemerisRequest {
        body: pleiades_backend::CelestialBody::Sun,
        instant: reference_instant(),
        observer: None,
        frame: CoordinateFrame::Equatorial,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };

    assert!(backend
        .metadata()
        .supported_frames
        .contains(&CoordinateFrame::Equatorial));

    let result = backend
        .position(&request)
        .expect("equatorial frame request should resolve");
    let ecliptic = result
        .ecliptic
        .expect("equatorial requests should still populate ecliptic coordinates");
    let expected = ecliptic.to_equatorial(request.instant.mean_obliquity());
    let equatorial = result
        .equatorial
        .expect("equatorial coordinates should be present");

    assert_eq!(result.frame, CoordinateFrame::Equatorial);
    assert_eq!(equatorial, expected);
    assert!(equatorial.right_ascension.degrees().is_finite());
    assert!(equatorial.declination.degrees().is_finite());
}

#[test]
fn observer_requests_are_rejected_explicitly() {
    let backend = JplSnapshotBackend;
    let request = EphemerisRequest {
        body: pleiades_backend::CelestialBody::Sun,
        instant: reference_instant(),
        observer: Some(pleiades_backend::ObserverLocation::new(
            pleiades_backend::Latitude::from_degrees(51.5),
            pleiades_backend::Longitude::from_degrees(0.0),
            None,
        )),
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };

    let error = backend
        .position(&request)
        .expect_err("reference snapshot should reject topocentric requests");

    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
}

#[test]
fn apparent_requests_are_rejected_explicitly() {
    let backend = JplSnapshotBackend;
    let request = EphemerisRequest {
        body: pleiades_backend::CelestialBody::Sun,
        instant: reference_instant(),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Apparent,
    };

    let error = backend
        .position(&request)
        .expect_err("reference snapshot should reject apparent-place requests");

    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
}

#[test]
fn batch_query_rejects_unsupported_time_scales_explicitly() {
    let backend = JplSnapshotBackend;
    let requests = vec![EphemerisRequest {
        body: pleiades_backend::CelestialBody::Sun,
        instant: Instant::new(JulianDay::from_days(REFERENCE_EPOCH_JD), TimeScale::Utc),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    }];

    let error = backend
        .positions(&requests)
        .expect_err("reference snapshot should reject unsupported batch time-scale requests");

    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
}

#[test]
fn batch_query_rejects_observer_requests_explicitly() {
    let backend = JplSnapshotBackend;
    let requests = vec![EphemerisRequest {
        body: pleiades_backend::CelestialBody::Sun,
        instant: reference_instant(),
        observer: Some(pleiades_backend::ObserverLocation::new(
            pleiades_backend::Latitude::from_degrees(51.5),
            pleiades_backend::Longitude::from_degrees(0.0),
            None,
        )),
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    }];

    let error = backend
        .positions(&requests)
        .expect_err("reference snapshot should reject topocentric batch requests");

    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
}

#[test]
fn batch_query_rejects_apparent_requests_explicitly() {
    let backend = JplSnapshotBackend;
    let requests = vec![EphemerisRequest {
        body: pleiades_backend::CelestialBody::Sun,
        instant: reference_instant(),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Apparent,
    }];

    let error = backend
        .positions(&requests)
        .expect_err("reference snapshot should reject apparent batch requests");

    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
}

#[test]
fn batch_query_preserves_mixed_time_scales_across_the_reference_snapshot() {
    let backend = JplSnapshotBackend;
    let requests = reference_snapshot_mixed_time_scale_batch_parity_requests()
        .expect("reference snapshot mixed TT/TDB batch parity requests should exist");

    let results = backend
        .positions(&requests)
        .expect("reference snapshot should preserve mixed TT/TDB batch requests");

    assert_eq!(results.len(), requests.len());
    for (request, result) in requests.iter().zip(results.iter()) {
        assert_eq!(result.body, request.body);
        assert_eq!(result.instant.scale, request.instant.scale);
        let single = backend
            .position(request)
            .expect("single mixed-scale query should match the batch result");
        assert_eq!(single.body, result.body);
        assert_eq!(single.instant.scale, request.instant.scale);
        assert_eq!(single.quality, result.quality);

        let ecliptic = result
            .ecliptic
            .expect("reference snapshot should include ecliptic coordinates");
        let single_ecliptic = single
            .ecliptic
            .expect("single-query reference snapshot should include ecliptic coordinates");
        assert_eq!(
            ecliptic.longitude.degrees(),
            single_ecliptic.longitude.degrees()
        );
        assert_eq!(
            ecliptic.latitude.degrees(),
            single_ecliptic.latitude.degrees()
        );
        assert_eq!(
            ecliptic.distance_au.expect("distance should exist"),
            single_ecliptic
                .distance_au
                .expect("single-query distance should exist")
        );
    }
}

#[test]
fn snapshot_data_matches_the_known_j2000_sun_longitude() {
    let entry = reference_snapshot()
        .iter()
        .find(|entry| {
            entry.body == pleiades_backend::CelestialBody::Sun
                && entry.epoch.julian_day.days() == REFERENCE_EPOCH_JD
        })
        .expect("sun entry should exist at J2000");

    let longitude = entry.ecliptic().longitude.degrees();
    assert!((longitude - 280.3778227681435).abs() < 1e-9);
}

#[test]
fn snapshot_backend_distinguishes_unsupported_body_from_out_of_range() {
    let backend = JplSnapshotBackend;
    let unsupported = EphemerisRequest {
        body: pleiades_backend::CelestialBody::MeanNode,
        instant: reference_instant(),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };
    let error = backend
        .position(&unsupported)
        .expect_err("missing bodies should not be reported as date-range errors");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);

    let out_of_range = EphemerisRequest {
        body: pleiades_backend::CelestialBody::Ceres,
        instant: Instant::new(JulianDay::from_days(2_634_168.0), TimeScale::Tdb),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };
    let error = backend
        .position(&out_of_range)
        .expect_err("single-epoch bodies should report out-of-range requests");
    assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant);
}

#[test]
fn batch_query_distinguishes_unsupported_body_from_out_of_range() {
    let backend = JplSnapshotBackend;
    let supported = EphemerisRequest {
        body: pleiades_backend::CelestialBody::Ceres,
        instant: reference_instant(),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };
    let unsupported = EphemerisRequest {
        body: pleiades_backend::CelestialBody::MeanNode,
        ..supported.clone()
    };
    let unsupported_error = backend
        .positions(&[supported.clone(), unsupported])
        .expect_err("batch queries should preserve unsupported-body failures");
    assert_eq!(unsupported_error.kind, EphemerisErrorKind::UnsupportedBody);

    let out_of_range = EphemerisRequest {
        body: pleiades_backend::CelestialBody::Ceres,
        instant: Instant::new(JulianDay::from_days(2_634_168.0), TimeScale::Tdb),
        ..supported
    };
    let out_of_range_error = backend
        .positions(&[out_of_range])
        .expect_err("batch queries should preserve out-of-range failures");
    assert_eq!(
        out_of_range_error.kind,
        EphemerisErrorKind::OutOfRangeInstant
    );
}

#[test]
fn snapshot_backend_resolves_ceres_at_j2000() {
    let backend = JplSnapshotBackend;
    let request = EphemerisRequest {
        body: pleiades_backend::CelestialBody::Ceres,
        instant: reference_instant(),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };

    let result = backend
        .position(&request)
        .expect("reference snapshot should resolve the asteroid entry");
    let ecliptic = result
        .ecliptic
        .expect("reference snapshot should include ecliptic coordinates");
    assert!((ecliptic.longitude.degrees() - 184.459642854516).abs() < 1e-12);
    assert!((ecliptic.latitude.degrees() - 11.838531252961646).abs() < 1e-12);
    assert!(
        (ecliptic.distance_au.expect("distance should exist") - 2.2568850705531642).abs() < 1e-12
    );
}

#[test]
fn snapshot_backend_resolves_named_asteroids_at_j2000() {
    let backend = JplSnapshotBackend;
    let cases = [
        (
            pleiades_backend::CelestialBody::Pallas,
            134.04575066840783,
            -48.35108149430447,
            1.4371532489145409,
        ),
        (
            pleiades_backend::CelestialBody::Juno,
            278.008461932084,
            9.450859010610209,
            4.084400792647673,
        ),
        (
            pleiades_backend::CelestialBody::Vesta,
            245.98418908965346,
            4.251902812654469,
            2.898586893865609,
        ),
        (
            pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
            236.28757472178148,
            -7.734019866618642,
            1.854402724550437,
        ),
    ];

    for (body, expected_longitude_deg, expected_latitude_deg, expected_distance_au) in cases {
        let request = EphemerisRequest {
            body,
            instant: reference_instant(),
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("reference snapshot should resolve the asteroid entry");
        assert_eq!(result.quality, QualityAnnotation::Exact);

        let ecliptic = result
            .ecliptic
            .expect("reference snapshot should include ecliptic coordinates");
        assert!((ecliptic.longitude.degrees() - expected_longitude_deg).abs() < 1e-12);
        assert!((ecliptic.latitude.degrees() - expected_latitude_deg).abs() < 1e-12);
        assert!(
            (ecliptic.distance_au.expect("distance should exist") - expected_distance_au).abs()
                < 1e-12
        );
    }
}

#[test]
fn batch_query_preserves_equatorial_frame_and_values() {
    let backend = JplSnapshotBackend;
    let evidence = reference_asteroid_equatorial_evidence();
    let requests = evidence
        .iter()
        .map(|sample| EphemerisRequest {
            body: sample.body.clone(),
            instant: sample.epoch,
            observer: None,
            frame: CoordinateFrame::Equatorial,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        })
        .collect::<Vec<_>>();

    let results = backend
        .positions(&requests)
        .expect("batch equatorial query should preserve the asteroid reference order");

    assert_eq!(results.len(), evidence.len());
    for (sample, result) in evidence.iter().zip(results.iter()) {
        assert_eq!(result.body, sample.body);
        assert_eq!(result.frame, CoordinateFrame::Equatorial);
        let equatorial = result
            .equatorial
            .expect("reference snapshot should include equatorial coordinates");

        assert_eq!(equatorial, sample.equatorial);
        assert!(equatorial.right_ascension.degrees().is_finite());
        assert!(equatorial.declination.degrees().is_finite());
    }
}

#[test]
fn batch_query_preserves_reference_snapshot_order_and_ecliptic_values() {
    let backend = JplSnapshotBackend;
    let evidence = reference_snapshot();
    let requests = evidence
        .iter()
        .map(|sample| EphemerisRequest {
            body: sample.body.clone(),
            instant: sample.epoch,
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        })
        .collect::<Vec<_>>();

    let results = backend
        .positions(&requests)
        .expect("batch ecliptic query should preserve the reference snapshot order");

    assert_eq!(results.len(), evidence.len());
    for (sample, result) in evidence.iter().zip(results.iter()) {
        assert_eq!(result.body, sample.body);
        assert_eq!(result.instant, sample.epoch);
        assert_eq!(result.frame, CoordinateFrame::Ecliptic);
        assert_eq!(result.quality, QualityAnnotation::Exact);

        let ecliptic = result
            .ecliptic
            .expect("reference snapshot should include ecliptic coordinates");
        let expected = sample.ecliptic();
        assert!((ecliptic.longitude.degrees() - expected.longitude.degrees()).abs() < 1e-12);
        assert!((ecliptic.latitude.degrees() - expected.latitude.degrees()).abs() < 1e-12);
        assert!(
            (ecliptic.distance_au.expect("distance should exist")
                - expected
                    .distance_au
                    .expect("expected distance should exist"))
            .abs()
                < 1e-12
        );
    }
}

#[test]
fn snapshot_backend_resolves_custom_asteroid_at_j2000() {
    let backend = JplSnapshotBackend;
    let request = EphemerisRequest {
        body: pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
        instant: reference_instant(),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };

    let result = backend
        .position(&request)
        .expect("reference snapshot should resolve the custom asteroid entry");
    assert_eq!(result.quality, QualityAnnotation::Exact);
    let ecliptic = result
        .ecliptic
        .expect("reference snapshot should include ecliptic coordinates");
    assert!((ecliptic.longitude.degrees() - 236.28757472178148).abs() < 1e-12);
    assert!((ecliptic.latitude.degrees() - (-7.734019866618642)).abs() < 1e-12);
    assert!(
        (ecliptic.distance_au.expect("distance should exist") - 1.854402724550437).abs() < 1e-12
    );
}

#[test]
fn snapshot_corpus_backend_resolves_exact_corpus_epoch() {
    // Two adjacent Sun samples so exact lookup has a defined window.
    let target_entry = SnapshotEntry {
        body: CelestialBody::Sun,
        epoch: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb),
        x_km: 1.0,
        y_km: 2.0,
        z_km: 3.0,
        vx_km_s: None,
        vy_km_s: None,
        vz_km_s: None,
    };
    let adjacent_entry = SnapshotEntry {
        body: CelestialBody::Sun,
        epoch: Instant::new(JulianDay::from_days(2_451_546.0), TimeScale::Tdb),
        x_km: 4.0,
        y_km: 5.0,
        z_km: 6.0,
        vx_km_s: None,
        vy_km_s: None,
        vz_km_s: None,
    };
    // Compute the expected ecliptic from the held entry directly — this is what
    // the backend should return for an exact-epoch hit.
    let expected_ecliptic = target_entry.ecliptic();

    let backend = SnapshotCorpusBackend::from_entries(vec![target_entry, adjacent_entry]);
    let req = EphemerisRequest {
        body: CelestialBody::Sun,
        instant: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };
    let result = backend
        .position(&req)
        .expect("exact corpus epoch should resolve");
    let ecliptic = result.ecliptic.expect("ecliptic output");

    // Exact-epoch lookup must return the stored entry's own ecliptic — proving
    // the result comes from the held entries, not a global snapshot.
    assert!(
        (ecliptic.longitude.degrees() - expected_ecliptic.longitude.degrees()).abs() < 1e-9,
        "longitude mismatch: got {} expected {}",
        ecliptic.longitude.degrees(),
        expected_ecliptic.longitude.degrees()
    );
    assert!(
        (ecliptic.latitude.degrees() - expected_ecliptic.latitude.degrees()).abs() < 1e-9,
        "latitude mismatch: got {} expected {}",
        ecliptic.latitude.degrees(),
        expected_ecliptic.latitude.degrees()
    );

    // A body not present in the held entries must return an error — proving the
    // backend is bound to its held entries and does not fall through to a global source.
    let mars_req = EphemerisRequest {
        body: CelestialBody::Mars,
        ..req
    };
    let err = backend
        .position(&mars_req)
        .expect_err("body absent from held entries should return an error");
    assert_eq!(err.kind, EphemerisErrorKind::UnsupportedBody);
}

#[test]
fn parses_eight_field_row_with_velocity() {
    let csv = "#Columns:epoch_jd,body,x_km,y_km,z_km,vx_km_s,vy_km_s,vz_km_s\n\
               2451545.0,Mars,1.0,2.0,3.0,0.1,0.2,0.3\n";
    let rows = parse_snapshot_entries(csv).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].vx_km_s, Some(0.1));
    assert_eq!(rows[0].vz_km_s, Some(0.3));
}

#[test]
fn parses_five_field_row_without_velocity() {
    let csv = "#Columns:epoch_jd,body,x_km,y_km,z_km\n2451545.0,Mars,1.0,2.0,3.0\n";
    let rows = parse_snapshot_entries(csv).unwrap();
    assert_eq!(rows[0].vx_km_s, None);
}
