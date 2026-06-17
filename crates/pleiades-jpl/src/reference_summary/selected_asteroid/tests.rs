//! Tests for the selected_asteroid module.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use pleiades_backend::{Apparentness, EphemerisErrorKind, EphemerisRequest};
#[allow(unused_imports)]
use pleiades_backend::{CelestialBody, EphemerisBackend, QualityAnnotation};
#[allow(unused_imports)]
use pleiades_types::CoordinateFrame;

#[test]
fn selected_asteroid_apophis_samples_match_horizons_fixture() {
    let reference_entries = reference_snapshot();
    let holdout_entries =
        independent_holdout_snapshot_entries().expect("independent hold-out snapshot should exist");

    #[allow(clippy::excessive_precision)]
    let expected_samples = [
        (
            2_451_545.0,
            (
                -1.287724404032539E+08,
                -1.665083325095297E+08,
                -2.616026236697651E+06,
            ),
        ),
        (
            2_451_915.5,
            (
                -4.208617179604869E+07,
                -2.505545978627344E+08,
                3.774323955966830E+06,
            ),
        ),
        (
            2_451_917.5,
            (
                -3.213794076979073E+07,
                -2.513264006732349E+08,
                4.014690688127324E+06,
            ),
        ),
    ];

    for (epoch, (expected_x, expected_y, expected_z)) in expected_samples {
        let reference_entry = reference_entries
            .iter()
            .find(|entry| {
                entry.body
                    == pleiades_backend::CelestialBody::Custom(CustomBodyId::new(
                        "asteroid",
                        "99942-Apophis",
                    ))
                    && entry.epoch.julian_day.days() == epoch
            })
            .unwrap_or_else(|| panic!("missing reference Apophis sample at JD {epoch}"));
        assert_eq!(reference_entry.x_km, expected_x);
        assert_eq!(reference_entry.y_km, expected_y);
        assert_eq!(reference_entry.z_km, expected_z);

        let holdout_entry = holdout_entries
            .iter()
            .find(|entry| {
                entry.body
                    == pleiades_backend::CelestialBody::Custom(CustomBodyId::new(
                        "asteroid",
                        "99942-Apophis",
                    ))
                    && entry.epoch.julian_day.days() == epoch
            })
            .unwrap_or_else(|| panic!("missing hold-out Apophis sample at JD {epoch}"));
        assert_eq!(holdout_entry.x_km, expected_x);
        assert_eq!(holdout_entry.y_km, expected_y);
        assert_eq!(holdout_entry.z_km, expected_z);
    }
}

#[test]
fn selected_asteroid_source_evidence_summary_reports_the_expanded_coverage() {
    let summary = selected_asteroid_source_evidence_summary()
        .expect("selected asteroid source evidence summary should exist");
    assert_eq!(
            summary.summary_line(),
            "Selected asteroid source evidence: 95 source-backed samples across 6 bodies and 17 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; bodies: Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis"
        );
    assert_eq!(
        summary.summary_line(),
        selected_asteroid_source_evidence_summary_for_report()
    );
    assert_eq!(
        validated_selected_asteroid_source_evidence_summary_for_report(),
        Ok(summary.summary_line())
    );
}

#[test]
fn selected_asteroid_source_window_summary_reports_the_body_windows() {
    let summary = selected_asteroid_source_window_summary()
        .expect("selected asteroid source window summary should exist");
    assert_eq!(summary.windows.len(), summary.sample_bodies.len());
    assert_eq!(summary.sample_count, 95);
    assert_eq!(summary.epoch_count, 17);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Selected asteroid source windows: 95 source-backed samples across 6 bodies and 17 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; windows: Ceres: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Pallas: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Juno: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Vesta: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:433-Eros: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:99942-Apophis: 10 samples across 10 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB)"
        );
    assert_eq!(
        summary.summary_line(),
        selected_asteroid_source_window_summary_for_report()
    );
    assert_eq!(
        validated_selected_asteroid_source_window_summary_for_report(),
        Ok(summary.summary_line())
    );
}

#[test]
fn selected_asteroid_source_request_corpus_summary_reports_the_frame_specific_request_slice() {
    let summary = selected_asteroid_source_request_corpus_summary(CoordinateFrame::Ecliptic)
        .expect("selected asteroid source request corpus summary should exist");
    assert_eq!(summary.request_count, 95);
    assert_eq!(summary.body_count, 6);
    assert_eq!(summary.epoch_count, 17);
    assert_eq!(summary.frame, CoordinateFrame::Ecliptic);
    assert_eq!(summary.zodiac_mode, ZodiacMode::Tropical);
    assert_eq!(summary.apparentness, Apparentness::Mean);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        selected_asteroid_source_request_corpus_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
        validated_selected_asteroid_source_request_corpus_summary_for_report(),
        Ok(summary.summary_line())
    );
    assert_eq!(
        selected_asteroid_source_request_corpus_equatorial_summary_for_report(),
        selected_asteroid_source_request_corpus_summary(CoordinateFrame::Equatorial)
            .expect("selected asteroid source request corpus equatorial summary should exist")
            .summary_line()
    );
    assert_eq!(
        validated_selected_asteroid_source_request_corpus_equatorial_summary_for_report(),
        Ok(
            selected_asteroid_source_request_corpus_summary(CoordinateFrame::Equatorial)
                .expect("selected asteroid source request corpus equatorial summary should exist")
                .summary_line()
        )
    );
    assert!(summary
        .summary_line()
        .contains("observerless) across 6 bodies and 17 epochs"));
}

#[test]
fn selected_asteroid_source_request_corpus_summary_validation_rejects_request_count_drift() {
    let mut summary = selected_asteroid_source_request_corpus_summary(CoordinateFrame::Ecliptic)
        .expect("selected asteroid source request corpus summary should exist");
    summary.request_count += 1;

    assert!(matches!(
        summary.validate(),
        Err(
            SelectedAsteroidSourceRequestCorpusSummaryValidationError::FieldOutOfSync {
                field: "request_count"
            }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn selected_asteroid_source_requests_preserve_the_source_slice() {
    let backend = JplSnapshotBackend;
    let requests = selected_asteroid_source_requests(CoordinateFrame::Equatorial)
        .expect("selected asteroid source requests should exist");
    let entries =
        selected_asteroid_source_entries().expect("selected asteroid source entries should exist");
    let results = backend
        .positions(&requests)
        .expect("selected asteroid source batch query should preserve the source slice");

    assert_eq!(results.len(), entries.len());
    for ((request, result), entry) in requests.iter().zip(results.iter()).zip(entries.iter()) {
        assert_eq!(request.body, entry.body);
        assert_eq!(request.instant, entry.epoch);
        assert_eq!(request.frame, CoordinateFrame::Equatorial);
        assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
        assert_eq!(request.apparent, Apparentness::Mean);
        assert!(request.observer.is_none());
        assert_eq!(result.body, entry.body);
        assert_eq!(result.instant, entry.epoch);
        assert_eq!(result.frame, CoordinateFrame::Equatorial);
        assert_eq!(result.quality, QualityAnnotation::Exact);

        let ecliptic = result
            .ecliptic
            .expect("selected asteroid source rows should include ecliptic coordinates");
        assert_eq!(ecliptic, entry.ecliptic());

        let expected_equatorial = ecliptic.to_equatorial(result.instant.mean_obliquity());
        let equatorial = result
            .equatorial
            .expect("selected asteroid source rows should include equatorial coordinates");
        assert_eq!(equatorial, expected_equatorial);
    }
}

#[test]
fn selected_asteroid_source_batch_parity_requests_preserve_the_source_slice() {
    let backend = JplSnapshotBackend;
    let requests = selected_asteroid_source_batch_parity_requests()
        .expect("selected asteroid source batch parity requests should exist");
    let entries =
        selected_asteroid_source_entries().expect("selected asteroid source entries should exist");
    let results = backend.positions(&requests).expect(
        "mixed-frame selected asteroid source batch query should preserve the source slice",
    );

    assert_eq!(results.len(), entries.len());
    for ((index, request), (result, entry)) in requests
        .iter()
        .enumerate()
        .zip(results.iter().zip(entries.iter()))
    {
        assert_eq!(request.body, entry.body);
        assert_eq!(request.instant, entry.epoch);
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
        assert_eq!(result.body, entry.body);
        assert_eq!(result.instant, entry.epoch);
        assert_eq!(result.frame, request.frame);
        assert_eq!(result.quality, QualityAnnotation::Exact);

        let ecliptic = result
            .ecliptic
            .expect("selected asteroid source rows should include ecliptic coordinates");
        assert_eq!(ecliptic, entry.ecliptic());

        let expected_equatorial = ecliptic.to_equatorial(result.instant.mean_obliquity());
        let equatorial = result
            .equatorial
            .expect("selected asteroid source rows should include equatorial coordinates");
        assert_eq!(equatorial, expected_equatorial);
    }
}

#[test]
fn selected_asteroid_source_2453000_summary_reports_the_2003_source_slice() {
    let summary = selected_asteroid_source_2453000_summary()
        .expect("selected asteroid 2003-12-27 source summary should exist");
    assert_eq!(summary.sample_count, 6);
    assert_eq!(
        summary.epoch,
        Instant::new(JulianDay::from_days(2_453_000.5), TimeScale::Tdb)
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference selected-asteroid 2003-12-27 source evidence: 6 exact samples at JD 2453000.5 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); 2003-12-27 source sample"
        );
    assert_eq!(
        summary.summary_line(),
        selected_asteroid_source_2453000_summary_for_report()
    );
}

#[test]
fn selected_asteroid_source_2500000_summary_reports_the_late_boundary_slice() {
    let summary = selected_asteroid_source_2500000_summary()
        .expect("selected asteroid 2500000 source summary should exist");
    assert_eq!(summary.sample_count, 6);
    assert_eq!(
        summary.epoch,
        Instant::new(JulianDay::from_days(2_500_000.0), TimeScale::Tdb)
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference selected-asteroid 2500000 source evidence: 6 exact samples at JD 2500000.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); 2500000 source sample"
        );
    assert_eq!(
        summary.summary_line(),
        selected_asteroid_source_2500000_summary_for_report()
    );
}

#[test]
fn selected_asteroid_source_2634167_summary_reports_the_outer_boundary_slice() {
    let summary = selected_asteroid_source_2634167_summary()
        .expect("selected asteroid 2634167 source summary should exist");
    assert_eq!(summary.sample_count, 6);
    assert_eq!(
        summary.epoch,
        Instant::new(JulianDay::from_days(2_634_167.0), TimeScale::Tdb)
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference selected-asteroid 2634167 source evidence: 6 exact samples at JD 2634167.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); 2634167 source sample"
        );
    assert_eq!(
        summary.summary_line(),
        selected_asteroid_source_2634167_summary_for_report()
    );
}

#[test]
fn selected_asteroid_source_window_summary_validation_rejects_sample_body_order_drift() {
    let mut summary = selected_asteroid_source_window_summary()
        .expect("selected asteroid source window summary should exist");
    summary.sample_bodies.swap(0, 1);

    assert!(matches!(
        summary.validate(),
        Err(
            SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn selected_asteroid_boundary_summary_reports_the_boundary_days() {
    let summary = selected_asteroid_boundary_summary()
        .expect("selected asteroid boundary summary should exist");
    assert_eq!(summary.sample_count, 23);
    assert_eq!(summary.epochs.len(), 4);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Selected asteroid boundary evidence: 23 exact samples across 4 epochs at JD 2451914.5 (TDB)..JD 2451919.5 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"
        );
    assert_eq!(
        summary.summary_line(),
        selected_asteroid_boundary_summary_for_report()
    );
}

#[test]
fn selected_asteroid_bridge_summary_reports_the_bridge_day() {
    let summary =
        selected_asteroid_bridge_summary().expect("selected asteroid bridge summary should exist");
    assert_eq!(summary.sample_count, 6);
    assert_eq!(summary.sample_bodies, reference_asteroids().to_vec());
    assert_eq!(summary.epoch.julian_day.days(), 2_451_915.0);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Selected asteroid bridge evidence: 6 exact samples at JD 2451915.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); bridge sample across the asteroid-only gap"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        selected_asteroid_bridge_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn selected_asteroid_dense_boundary_summary_reports_the_dense_boundary_day() {
    let summary = selected_asteroid_dense_boundary_summary()
        .expect("selected asteroid dense boundary summary should exist");
    assert_eq!(summary.sample_count, 5);
    assert_eq!(summary.sample_bodies, reference_asteroids().to_vec());
    assert_eq!(summary.epoch.julian_day.days(), 2_451_916.5);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Selected asteroid dense boundary evidence: 5 exact samples at JD 2451916.5 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); dense boundary day"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        selected_asteroid_dense_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn selected_asteroid_dense_boundary_summary_validation_rejects_drift() {
    let mut summary = selected_asteroid_dense_boundary_summary()
        .expect("selected asteroid dense boundary summary should exist");
    summary.sample_bodies.swap(0, 1);

    let error = summary
        .validate()
        .expect_err("drifted selected asteroid dense boundary summary should fail validation");

    assert!(matches!(
        error,
        SelectedAsteroidDenseBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Ceres,
            found: pleiades_backend::CelestialBody::Pallas
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        selected_asteroid_dense_boundary_summary_for_report(),
        selected_asteroid_dense_boundary_summary()
            .expect("selected asteroid dense boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn selected_asteroid_bridge_summary_validation_rejects_drift() {
    let mut summary =
        selected_asteroid_bridge_summary().expect("selected asteroid bridge summary should exist");
    summary.sample_bodies.swap(0, 1);

    let error = summary
        .validate()
        .expect_err("drifted selected asteroid bridge summary should fail validation");

    assert!(matches!(
        error,
        SelectedAsteroidBridgeSummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Ceres,
            found: pleiades_backend::CelestialBody::Pallas
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        selected_asteroid_bridge_summary_for_report(),
        selected_asteroid_bridge_summary()
            .expect("selected asteroid bridge summary should exist")
            .summary_line()
    );
}

#[test]
fn selected_asteroid_terminal_boundary_summary_reports_the_terminal_boundary_day() {
    let summary = selected_asteroid_terminal_boundary_summary()
        .expect("selected asteroid terminal boundary summary should exist");
    assert_eq!(summary.sample_count, 6);
    assert_eq!(summary.sample_bodies, reference_asteroids().to_vec());
    assert_eq!(
        summary.epoch,
        Instant::new(JulianDay::from_days(2_500_000.0), TimeScale::Tdb)
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference selected-asteroid terminal boundary evidence: 6 exact samples at JD 2500000.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); 2500-01-01 terminal boundary sample"
        );
    assert_eq!(
        summary.summary_line(),
        selected_asteroid_terminal_boundary_summary_for_report()
    );
}

#[test]
fn selected_asteroid_terminal_boundary_summary_validation_rejects_epoch_drift() {
    let mut summary = selected_asteroid_terminal_boundary_summary()
        .expect("selected asteroid terminal boundary summary should exist");
    summary.epoch = Instant::new(JulianDay::from_days(2_500_001.0), TimeScale::Tdb);

    assert!(matches!(
        summary.validate(),
        Err(
            SelectedAsteroidTerminalBoundarySummaryValidationError::EpochMismatch {
                expected: _,
                found: _
            }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn selected_asteroid_batch_parity_summary_reports_the_expected_coverage() {
    let summary = selected_asteroid_batch_parity_summary()
        .expect("selected asteroid batch parity summary should exist");
    assert_eq!(summary.request_count, 6);
    assert_eq!(summary.sample_bodies, reference_asteroids().to_vec());
    assert_eq!(summary.epoch, reference_asteroid_evidence()[0].epoch);
    assert_eq!(summary.ecliptic_count, 3);
    assert_eq!(summary.equatorial_count, 3);
    assert!(summary.parity_preserved);
    summary
        .validate()
        .expect("selected asteroid batch parity summary should validate");
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Selected asteroid batch parity: 6 requests across 6 bodies at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); frame mix: 3 ecliptic, 3 equatorial; batch/single parity preserved"
        );
    assert_eq!(
        summary.summary_line(),
        selected_asteroid_batch_parity_summary_for_report()
    );
}

#[test]
fn selected_asteroid_source_evidence_summary_validation_rejects_sample_count_drift() {
    let mut summary = selected_asteroid_source_evidence_summary()
        .expect("selected asteroid source evidence summary should exist");
    summary.sample_count += 1;

    assert!(matches!(
        summary.validate(),
        Err(
            SelectedAsteroidSourceSummaryValidationError::FieldOutOfSync {
                field: "sample_count"
            }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn selected_asteroid_source_evidence_summary_validation_rejects_custom_body_drift() {
    let mut summary = selected_asteroid_source_evidence_summary()
        .expect("selected asteroid source evidence summary should exist");
    summary.sample_bodies[4] = pleiades_backend::CelestialBody::Ceres;

    assert!(matches!(
        summary.validate(),
        Err(
            SelectedAsteroidSourceSummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn selected_asteroid_source_evidence_summary_validation_rejects_body_order_drift() {
    let mut summary = selected_asteroid_source_evidence_summary()
        .expect("selected asteroid source evidence summary should exist");
    summary.sample_bodies.swap(0, 1);

    assert!(matches!(
        summary.validate(),
        Err(
            SelectedAsteroidSourceSummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn selected_asteroid_source_window_summary_validation_rejects_sample_count_drift() {
    let mut summary = selected_asteroid_source_window_summary()
        .expect("selected asteroid source window summary should exist");
    summary.sample_count += 1;

    assert!(matches!(
        summary.validate(),
        Err(
            SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "sample_count"
            }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn selected_asteroid_source_window_summary_validation_rejects_epoch_count_drift() {
    let mut summary = selected_asteroid_source_window_summary()
        .expect("selected asteroid source window summary should exist");
    summary.epoch_count += 1;

    assert!(matches!(
        summary.validate(),
        Err(
            SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "epoch_count"
            }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn selected_asteroid_source_window_summary_validation_rejects_custom_body_drift() {
    let mut summary = selected_asteroid_source_window_summary()
        .expect("selected asteroid source window summary should exist");
    summary.sample_bodies[4] = pleiades_backend::CelestialBody::Ceres;

    assert!(matches!(
        summary.validate(),
        Err(
            SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn selected_asteroid_batch_parity_summary_validation_rejects_parity_drift() {
    let mut summary = selected_asteroid_batch_parity_summary()
        .expect("selected asteroid batch parity summary should exist");
    summary.parity_preserved = false;

    assert!(matches!(
        summary.validate(),
        Err(
            SelectedAsteroidBatchParitySummaryValidationError::ParityPreservedMismatch {
                expected: true,
                found: false,
            }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn selected_asteroid_boundary_summary_validation_rejects_body_order_drift() {
    let mut summary = selected_asteroid_boundary_summary()
        .expect("selected asteroid boundary summary should exist");
    summary.sample_bodies.swap(0, 1);

    assert!(matches!(
        summary.validate(),
        Err(SelectedAsteroidBoundarySummaryValidationError::BodyOrderMismatch { index: 0, .. })
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn selected_asteroid_boundary_summary_validation_rejects_epoch_order_drift() {
    let mut summary = selected_asteroid_boundary_summary()
        .expect("selected asteroid boundary summary should exist");
    summary.epochs.swap(0, 1);

    assert!(matches!(
        summary.validate(),
        Err(SelectedAsteroidBoundarySummaryValidationError::EpochOrderMismatch { index: 0, .. })
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn selected_asteroid_source_window_summary_validation_rejects_window_order_drift() {
    let mut summary = selected_asteroid_source_window_summary()
        .expect("selected asteroid source window summary should exist");
    summary.windows.swap(0, 1);

    assert!(matches!(
        summary.validate(),
        Err(
            SelectedAsteroidSourceWindowSummaryValidationError::FieldOutOfSync { field: "windows" }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn constrained_class_report_states_window_and_tiers() {
    let r = selected_asteroid_constrained_class_report();
    assert!(r.contains("1900\u{2013}2100"));
    assert!(r.contains("constrained class"));
    assert!(r.contains("Tier A"));
    assert!(r.contains("Tier B"));
}
