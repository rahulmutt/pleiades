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
    assert_eq!(summary.bodies, reference_bodies());
    assert_eq!(summary.epoch_count, 23);
    assert_eq!(summary.boundary_row_count, 66);
    assert_eq!(summary.boundary_body_count, 16);
    assert_eq!(
        summary.boundary_bodies,
        &[
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
            CelestialBody::Ceres,
            CelestialBody::Pallas,
            CelestialBody::Juno,
            CelestialBody::Vesta,
            CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
            CelestialBody::Custom(CustomBodyId::new("asteroid", "99942-Apophis")),
        ]
    );
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
    assert_eq!(
        summary.quarter_day_bodies,
        &[
            CelestialBody::Sun,
            CelestialBody::Moon,
            CelestialBody::Mercury,
            CelestialBody::Venus
        ]
    );
    assert_eq!(summary.quarter_day_epoch_count, 2);
    assert_eq!(
        summary.quarter_day_earliest_epoch.julian_day.days(),
        2_451_915.25
    );
    assert_eq!(
        summary.quarter_day_latest_epoch.julian_day.days(),
        2_451_915.75
    );
    let reference_bodies = format_bodies(reference_bodies());
    let boundary_bodies = format_bodies(summary.boundary_bodies);
    let quarter_day_bodies = format_bodies(summary.quarter_day_bodies);
    assert_eq!(
            summary.summary_line(),
            format!(
                "Production generation coverage: 277 rows across 16 bodies and 23 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies: {}; boundary overlay (major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs.): 66 rows across 16 bodies and 12 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); boundary bodies: {}; quarter-day boundary samples: 8 rows across 4 bodies and 2 epochs (JD 2451915.25 (TDB)..JD 2451915.75 (TDB)); quarter-day bodies: {}",
                reference_bodies,
                boundary_bodies,
                quarter_day_bodies
            )
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        production_generation_snapshot_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
            production_generation_quarter_day_boundary_summary_for_report(),
            "Production generation quarter-day boundary samples: 8 rows across 4 bodies and 2 epochs (JD 2451915.25 (TDB)..JD 2451915.75 (TDB)); bodies: Sun, Moon, Mercury, Venus"
        );
    let production_generation_source_summary = production_generation_source_summary_for_report();
    assert!(
        production_generation_source_summary.contains("strategy=documented hybrid fixture corpus")
    );
    assert!(production_generation_source_summary.contains(
        "redistribution=repository-checked regression fixtures, not a broad public corpus."
    ));
    assert!(production_generation_source_summary
        .contains("source windows=277 source-backed samples across 16 bodies and 23 epochs"));
    assert!(production_generation_source_summary
        .contains("evidence classes=reference, hold-out, boundary overlay, provenance-only"));
    assert!(production_generation_source_summary
        .contains("generation command=generate-packaged-artifact --check"));
    assert!(production_generation_source_summary.contains("frame=geocentric ecliptic J2000"));
    assert!(production_generation_source_summary.contains("time scale=TDB"));
    assert!(production_generation_source_summary.contains("parser=pure-Rust and deterministic"));
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
    assert!(summary.windows[0].summary_line().starts_with("Ceres: "));
    assert!(summary.summary_line().starts_with(
            "Production generation source windows: 277 source-backed samples across 16 bodies and 23 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); windows: "
        ));
    assert!(summary.summary_line().contains("Mars:"));
    assert!(summary.summary_line().contains("Jupiter:"));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        production_generation_snapshot_window_summary_for_report(),
        summary.summary_line()
    );
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
    assert!(summary
        .summary_line()
        .starts_with("Production generation body-class coverage: major bodies: "));
    assert!(summary.summary_line().contains("selected asteroids: "));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        production_generation_snapshot_body_class_coverage_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
        validated_production_generation_snapshot_body_class_coverage_summary_for_report(),
        Ok(summary.summary_line())
    );
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
    assert_eq!(
            summary.summary_line(),
            "Production generation boundary overlay: 66 rows across 16 bodies and 12 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies: Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Pluto, Moon, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        production_generation_boundary_summary_for_report(),
        summary.summary_line()
    );
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
    assert_eq!(
        format_production_generation_boundary_source_summary(&boundary_summary),
        production_generation_boundary_source_summary_for_report()
    );
    assert!(production_generation_boundary_source_summary_for_report().contains(
            "Production generation boundary overlay source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables."
        ));
    assert!(production_generation_boundary_source_summary_for_report().contains(
            "selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167"
        ));
    assert!(production_generation_boundary_source_summary_for_report()
        .contains("asteroid:99942-Apophis now also appears at 2378498.5"));
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
    assert_eq!(
        summary.windows[0].summary_line(),
        format!(
            "Mars: 5 samples across 5 epochs at {}..{}",
            format_instant(summary.windows[0].earliest_epoch),
            format_instant(summary.windows[0].latest_epoch)
        )
    );
    assert!(summary.summary_line().starts_with("Production generation boundary windows: 66 source-backed samples across 16 bodies and 12 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); windows: "));
    assert!(summary.summary_line().contains("Mars: 5 samples across 5 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB); Jupiter: 5 samples across 5 epochs at JD 2451545.0 (TDB)..JD 2451915.5 (TDB)"));
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        production_generation_boundary_window_summary_for_report(),
        summary.summary_line()
    );

    let mut drifted = summary.clone();
    drifted.sample_count += 1;
    assert!(drifted.validated_summary_line().is_err());
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
    assert!(summary.summary_line().starts_with(
            "Production generation boundary body-class coverage: major bodies: 34 rows across 10 bodies and 7 epochs; major windows: "
        ));
    assert!(summary
        .summary_line()
        .contains(&summary.major_windows[0].summary_line()));
    assert!(summary
        .summary_line()
        .contains(&summary.major_windows[2].summary_line()));
    assert!(summary
        .summary_line()
        .contains("selected asteroids: 32 rows across 6 bodies and 7 epochs; asteroid windows: "));
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        production_generation_boundary_body_class_coverage_summary_for_report(),
        summary.summary_line()
    );

    let mut drifted = summary.clone();
    drifted.row_count += 1;
    assert!(drifted.validated_summary_line().is_err());
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
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        production_generation_snapshot_summary_for_report(),
        summary.summary_line()
    );
    assert!(summary.summary_line().contains("boundary overlay (major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs.): 66 rows across 16 bodies and 12 epochs"));
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
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        production_generation_boundary_request_corpus_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
        production_generation_boundary_request_corpus_equatorial_summary_for_report(),
        production_generation_boundary_request_corpus_summary(CoordinateFrame::Equatorial)
            .expect("production generation boundary request corpus equatorial summary should exist")
            .summary_line()
    );
    assert_eq!(
        validated_production_generation_boundary_request_corpus_equatorial_summary_for_report(),
        Ok(
            production_generation_boundary_request_corpus_summary(CoordinateFrame::Equatorial)
                .expect(
                    "production generation boundary request corpus equatorial summary should exist"
                )
                .summary_line()
        )
    );
    assert!(summary
        .summary_line()
        .contains("observerless) across 16 bodies and 12 epochs"));
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
fn production_generation_source_summary_validation_rejects_frame_and_time_scale_text_drift() {
    let summary = production_generation_source_summary();
    let frame_drift = summary.summary_line().replace(
        "frame=geocentric ecliptic J2000",
        "frame=geocentric equatorial J2000",
    );

    assert!(matches!(
        validate_production_generation_source_summary_text(&summary, &frame_drift),
        Err(
            ProductionGenerationSourceSummaryValidationError::RenderedSummaryOutOfSync {
                field: "frame"
            }
        )
    ));

    let summary = production_generation_source_summary();
    let time_scale_drift = summary
        .summary_line()
        .replace("time scale=TDB", "time scale=UTC");

    assert!(matches!(
        validate_production_generation_source_summary_text(&summary, &time_scale_drift),
        Err(
            ProductionGenerationSourceSummaryValidationError::RenderedSummaryOutOfSync {
                field: "time scale"
            }
        )
    ));
}

#[test]
fn production_generation_source_summary_documents_the_checked_in_csv_path() {
    let summary = production_generation_source_summary();
    let report = production_generation_source_summary_for_report();

    assert!(summary.validate().is_ok());
    assert!(report.contains("strategy=documented hybrid fixture corpus"));
    assert!(report.contains(
            "input path=checked-in CSV fixtures via include_str! reference_snapshot.csv and independent_holdout_snapshot.csv"
        ));
    assert!(report.contains("file format=comma-separated values"));
    assert!(report.contains("frame=geocentric ecliptic J2000"));
    assert!(report.contains("time scale=TDB"));
    assert!(report.contains("apparentness=Mean"));
    assert!(report.contains("parser=pure-Rust and deterministic"));
    assert!(report.contains("source revision=reference_snapshot.csv checksum=0x"));
    assert!(report.contains("evidence class=reference"));
    assert!(report.contains("reference snapshot exact J2000 evidence=16 exact J2000 samples at"));
    assert!(
        report.contains("evidence classes=reference, hold-out, boundary overlay, provenance-only")
    );
    assert!(report.contains("independent_holdout_snapshot.csv checksum=0x"));
    assert!(
        report.contains("source windows=277 source-backed samples across 16 bodies and 23 epochs")
    );
    assert!(report.contains("license posture=public-source provenance only; checked-in fixtures remain repository-local regression data"));
    assert!(report.contains("generation command=generate-packaged-artifact --check"));
    assert!(report.contains("checksum expectation=byte-identical fixture contents"));
    let expected_cadence = format!(
        "cadence={} reference epochs and {} boundary epochs",
        summary.source_windows.epoch_count,
        production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic)
            .expect("production generation boundary request corpus should exist")
            .epoch_count,
    );
    assert!(report.contains(&expected_cadence));
    let body_class_coverage = production_generation_snapshot_body_class_coverage_summary()
        .expect("production generation body-class coverage should exist");
    let boundary_body_class_coverage = production_generation_boundary_body_class_coverage_summary()
        .expect("production generation boundary body-class coverage should exist");
    let expected_body_class_cadence = format!(
            "body-class cadence=reference major bodies: {} epochs; reference selected asteroids: {} epochs; boundary major bodies: {} epochs; boundary selected asteroids: {} epochs",
            body_class_coverage.major_epoch_count,
            body_class_coverage.asteroid_epoch_count,
            boundary_body_class_coverage.major_epoch_count,
            boundary_body_class_coverage.asteroid_epoch_count,
        );
    assert!(report.contains(&expected_body_class_cadence));
    assert!(report.contains("reference and hold-out rows remain separate"));
    assert!(report.contains("schema=epoch_jd, body, x_km, y_km, z_km"));
    assert!(report.contains("columns=epoch_jd, body, x_km, y_km, z_km"));
    assert!(report.contains(
        "redistribution posture=repository-checked regression fixtures, not a broad public corpus"
    ));
}

#[test]
fn production_generation_source_summary_validated_report_matches_current_rendering() {
    let report = production_generation_source_summary_for_report();
    let validated = validated_production_generation_source_summary_for_report()
        .expect("validated production generation source summary should exist");

    assert_eq!(validated, report);
}

#[test]
fn production_generation_source_cadence_fragment_rejects_boundary_epoch_count_drift() {
    let error = production_generation_source_cadence_fragment_from_counts(31, 13, 12)
        .expect_err("mismatched boundary epoch counts should be rejected");

    assert!(matches!(
        error,
        ProductionGenerationSourceSummaryValidationError::BoundaryRequestCorpusEpochCountMismatch {
            ecliptic_epoch_count: 13,
            equatorial_epoch_count: 12,
        }
    ));
    assert_eq!(
        error.to_string(),
        "boundary request corpus epoch counts differ: ecliptic=13, equatorial=12"
    );
}

#[test]
fn production_generation_source_summary_validation_rejects_body_class_cadence_drift() {
    let summary = production_generation_source_summary();
    let rendered = summary.summary_line();
    let tampered = rendered.replace(
        "body-class cadence=reference major bodies: ",
        "body-class cadence=drifted major bodies: ",
    );

    let error = validate_production_generation_source_summary_text(&summary, &tampered)
        .expect_err("body-class cadence drift should fail closed");
    assert!(matches!(
        error,
        ProductionGenerationSourceSummaryValidationError::RenderedSummaryOutOfSync {
            field: "body-class cadence"
        }
    ));
    assert_eq!(
        error.to_string(),
        "rendered production-generation source summary field `body-class cadence` is out of sync"
    );
}

#[test]
fn production_generation_source_revision_summary_documents_fixture_checksums() {
    let summary = production_generation_source_revision_summary();
    let report = production_generation_source_revision_summary_for_report();
    let validated = validated_production_generation_source_revision_summary_for_report()
        .expect("validated production generation source revision summary should exist");

    assert_eq!(report, summary.summary_line());
    assert_eq!(validated, report);
    assert!(report.contains("reference_snapshot.csv checksum=0x"));
    assert!(report.contains("independent_holdout_snapshot.csv checksum=0x"));
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
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn production_generation_manifest_summary_documents_the_current_contract() {
    let summary = production_generation_manifest_summary()
        .expect("production generation manifest summary should exist");
    let report = production_generation_manifest_summary_for_report();

    assert!(summary.validate().is_ok());
    assert_eq!(summary.validated_summary_line().unwrap(), report);
    assert!(report.contains("Production generation manifest: coverage="));
    assert!(report.contains("source="));
    assert!(report.contains("body-class coverage="));
    assert!(report.contains("boundary overlay="));
    assert!(report.contains("boundary windows="));
    assert!(report.contains("boundary request corpus="));
}

#[test]
fn production_generation_manifest_summary_validated_report_matches_current_rendering() {
    assert_eq!(
        validated_production_generation_manifest_summary_for_report()
            .expect("validated production generation manifest summary should exist"),
        production_generation_manifest_summary_for_report(),
    );
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
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn production_generation_corpus_shape_summary_documents_the_current_contract() {
    let summary = production_generation_corpus_shape_summary()
        .expect("production generation corpus shape summary should exist");
    let report = production_generation_corpus_shape_summary_for_report();

    assert!(summary.validate().is_ok());
    assert!(report.contains("Production generation corpus shape: source="));
    assert!(report.contains("boundary request corpora: ecliptic="));
    assert!(report.contains("equatorial="));
    assert!(report.contains(
        "validated fields=body order, epochs, frame, time scale, columns, apparentness, checksums"
    ));
    assert!(report.contains("columns=epoch_jd, body, x_km, y_km, z_km"));
    assert!(report.contains("frame=geocentric ecliptic J2000"));
    assert!(report.contains("time scale=TDB"));
    assert!(report.contains("apparentness=Mean"));
}

#[test]
fn production_generation_corpus_shape_summary_validated_report_matches_current_rendering() {
    let report = production_generation_corpus_shape_summary_for_report();
    let validated = validated_production_generation_corpus_shape_summary_for_report()
        .expect("validated production generation corpus shape summary should exist");

    assert_eq!(validated, report);
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
fn production_generation_source_summary_validation_rejects_rendered_text_drift() {
    let summary = production_generation_source_summary();
    let drifted = summary.summary_line().replace(
        "columns=epoch_jd, body, x_km, y_km, z_km",
        "columns=epoch_jd, body, x_km, y_km",
    );

    assert!(matches!(
        validate_production_generation_source_summary_text(&summary, &drifted),
        Err(
            ProductionGenerationSourceSummaryValidationError::RenderedSummaryOutOfSync {
                field: "columns"
            }
        )
    ));
}

#[test]
fn production_generation_source_summary_validation_rejects_checksum_text_drift() {
    let summary = production_generation_source_summary();
    let drifted = summary.summary_line().replace(
            &summary.source_revision.summary_line(),
            "source revision=reference_snapshot.csv checksum=0x0000000000000000; independent_holdout_snapshot.csv checksum=0x0000000000000000",
        );

    assert!(matches!(
        validate_production_generation_source_summary_text(&summary, &drifted),
        Err(
            ProductionGenerationSourceSummaryValidationError::RenderedSummaryOutOfSync {
                field: "source revision"
            }
        )
    ));
}

#[test]
fn production_generation_source_summary_validation_rejects_apparentness_text_drift() {
    let summary = production_generation_source_summary();
    let drifted = summary
        .summary_line()
        .replace("apparentness=Mean", "apparentness=Apparent");

    assert!(matches!(
        validate_production_generation_source_summary_text(&summary, &drifted),
        Err(
            ProductionGenerationSourceSummaryValidationError::RenderedSummaryOutOfSync {
                field: "apparentness"
            }
        )
    ));
}

#[test]
fn production_generation_source_summary_validation_rejects_evidence_class_text_drift() {
    let summary = production_generation_source_summary();
    let drifted = summary.summary_line().replace(
        "evidence classes=reference, hold-out, boundary overlay, provenance-only",
        "evidence classes=reference, hold-out, boundary overlay",
    );

    assert!(matches!(
        validate_production_generation_source_summary_text(&summary, &drifted),
        Err(
            ProductionGenerationSourceSummaryValidationError::RenderedSummaryOutOfSync {
                field: "evidence classes"
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
    assert!(summary.validated_summary_line().is_err());

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

    let exactness_drift = production_generation_source_summary();
    let drifted = exactness_drift.summary_line().replace(
        "reference snapshot exact J2000 evidence=16 exact J2000 samples at",
        "reference snapshot exact J2000 evidence=drifted exactness evidence",
    );
    assert!(matches!(
        validate_production_generation_source_summary_text(&exactness_drift, &drifted),
        Err(
            ProductionGenerationSourceSummaryValidationError::RenderedSummaryOutOfSync {
                field: "reference snapshot exact J2000 evidence"
            }
        )
    ));
}
