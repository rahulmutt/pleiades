
use super::*;
use pleiades_backend::{Apparentness, EphemerisErrorKind, EphemerisRequest};

#[test]
fn reference_snapshot_covers_the_expected_bodies_and_epochs() {
    let metadata = JplSnapshotBackend::new().metadata();
    assert!(metadata
        .body_coverage
        .contains(&pleiades_backend::CelestialBody::Sun));
    assert!(metadata
        .body_coverage
        .contains(&pleiades_backend::CelestialBody::Moon));
    assert!(metadata
        .body_coverage
        .contains(&pleiades_backend::CelestialBody::Pluto));
    assert!(metadata
        .body_coverage
        .contains(&pleiades_backend::CelestialBody::Ceres));
    assert!(metadata
        .body_coverage
        .contains(&pleiades_backend::CelestialBody::Pallas));
    assert!(metadata
        .body_coverage
        .contains(&pleiades_backend::CelestialBody::Juno));
    assert!(metadata
        .body_coverage
        .contains(&pleiades_backend::CelestialBody::Vesta));
    assert!(metadata
        .body_coverage
        .contains(&pleiades_backend::CelestialBody::Custom(CustomBodyId::new(
            "asteroid", "433-Eros"
        ))));
    assert_eq!(
        reference_asteroids(),
        [
            pleiades_backend::CelestialBody::Ceres,
            pleiades_backend::CelestialBody::Pallas,
            pleiades_backend::CelestialBody::Juno,
            pleiades_backend::CelestialBody::Vesta,
            pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
            pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "99942-Apophis")),
        ]
    );
    assert!(metadata.nominal_range.start.is_some());
    assert!(metadata.nominal_range.end.is_some());
    let start = metadata
        .nominal_range
        .start
        .expect("start epoch should exist");
    let end = metadata.nominal_range.end.expect("end epoch should exist");
    assert!(start.julian_day.days() < end.julian_day.days());
    assert_eq!(reference_epochs().len(), 31);
    assert_eq!(
        reference_snapshot()
            .iter()
            .filter(|entry| entry.epoch.julian_day.days() == 2_400_000.0)
            .count(),
        10
    );
    assert_eq!(
        reference_snapshot()
            .iter()
            .filter(|entry| entry.epoch.julian_day.days() == 2_500_000.0)
            .count(),
        16
    );
    assert_eq!(
        reference_snapshot()
            .iter()
            .filter(|entry| entry.epoch.julian_day.days() == 2_600_000.0)
            .count(),
        1
    );
}

#[test]
fn reference_snapshot_summary_reports_the_expected_coverage() {
    let summary = reference_snapshot_summary().expect("reference snapshot summary should exist");
    summary
        .validate()
        .expect("reference snapshot summary should validate");
    assert_eq!(summary.row_count, 357);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.bodies, reference_bodies());
    assert_eq!(summary.epoch_count, 31);
    assert_eq!(summary.asteroid_row_count, 95);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_268_932.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(
            summary.summary_line(),
            format!(
                "Reference snapshot coverage: 357 rows across 16 bodies and 31 epochs (95 asteroid rows; JD 2268932.5 (TDB)..JD 2634167.0 (TDB)); bodies: {}",
                format_bodies(reference_bodies())
            )
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    let report = reference_snapshot_summary_for_report();
    assert!(report.contains(summary.summary_line().as_str()));
    assert!(report.contains(&reference_snapshot_source_summary_for_report()));
    assert!(report.contains(&reference_snapshot_source_window_summary_for_report()));
    assert!(report.contains(&reference_snapshot_major_body_bridge_summary_for_report()));
    assert!(report.contains(&reference_asteroid_evidence_summary_for_report()));
    assert!(report.contains(&reference_asteroid_equatorial_evidence_summary_for_report()));
    assert!(report.contains(&reference_asteroid_source_window_summary_for_report()));
    assert!(report.contains(&reference_snapshot_1749_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2360233_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_early_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_1750_selected_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_1800_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2378499_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_lunar_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_high_curvature_summary_for_report()));
    assert!(report.contains(&reference_snapshot_high_curvature_window_summary_for_report()));
    assert!(report.contains(&reference_snapshot_high_curvature_epoch_coverage_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2400000_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451545_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2360234_major_body_interior_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451910_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451911_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451912_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451913_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451914_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451914_major_body_pre_bridge_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451914_major_body_bridge_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451915_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451915_major_body_bridge_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451917_major_body_bridge_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451917_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451916_major_body_interior_summary_for_report()));
    assert!(
        report.contains(&reference_snapshot_2451916_major_body_dense_boundary_summary_for_report())
    );
    assert!(report.contains(&reference_snapshot_2451916_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_dense_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_mars_jupiter_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451919_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451920_major_body_interior_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2453000_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2500_major_body_boundary_summary_for_report()));
    assert!(report.contains(&selected_asteroid_boundary_summary_for_report()));
    assert!(report.contains(&selected_asteroid_bridge_summary_for_report()));
    assert!(report.contains(&selected_asteroid_dense_boundary_summary_for_report()));
    assert!(report.contains(&selected_asteroid_terminal_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2200_selected_body_boundary_summary_for_report()));
    assert!(
        report.contains(&reference_snapshot_2524593_selected_body_boundary_summary_for_report())
    );
    assert!(report.contains(&reference_snapshot_2500_selected_body_boundary_summary_for_report()));
    assert!(
        report.contains(&reference_snapshot_2634167_selected_body_boundary_summary_for_report())
    );
}

#[test]
fn reference_snapshot_exact_j2000_evidence_reports_the_expected_slice() {
    let summary = reference_snapshot_exact_j2000_evidence_summary()
        .expect("reference snapshot exact J2000 evidence should exist");
    summary
        .validate()
        .expect("reference snapshot exact J2000 evidence should validate");
    assert_eq!(summary.sample_count, 16);
    assert_eq!(summary.sample_bodies, reference_bodies());
    assert_eq!(summary.epoch.julian_day.days(), 2_451_545.0);
    assert_eq!(summary.summary_line(), format!("Reference snapshot exact J2000 evidence: 16 exact J2000 samples at JD 2451545.0 (TDB) ({})", format_bodies(reference_bodies())));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        validated_reference_snapshot_exact_j2000_evidence_summary_for_report(),
        Ok(summary.summary_line())
    );
    assert_eq!(
        reference_snapshot_exact_j2000_evidence_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_exact_j2000_body_class_coverage_reports_the_expected_slice() {
    let summary = reference_snapshot_exact_j2000_body_class_coverage_summary()
        .expect("reference snapshot exact J2000 body-class coverage should exist");
    summary
        .validate()
        .expect("reference snapshot exact J2000 body-class coverage should validate");
    assert_eq!(summary.major_body_row_count, 10);
    assert_eq!(summary.major_bodies, reference_bodies()[..10].to_vec());
    assert_eq!(summary.asteroid_row_count, 6);
    assert_eq!(summary.asteroid_bodies, reference_asteroids().to_vec());
    assert_eq!(summary.epoch.julian_day.days(), 2_451_545.0);
    assert_eq!(summary.summary_line(), format!("Reference snapshot exact J2000 body-class coverage: 10 major-body samples across 10 bodies and 1 epoch ({}); 6 selected-asteroid samples across 6 bodies and 1 epoch ({})", format_bodies(&summary.major_bodies), format_bodies(&summary.asteroid_bodies)));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        validated_reference_snapshot_exact_j2000_body_class_coverage_summary_for_report(),
        Ok(summary.summary_line())
    );
    assert_eq!(
        reference_snapshot_exact_j2000_body_class_coverage_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_lunar_boundary_summary_reports_the_expected_window() {
    let summary = reference_snapshot_lunar_boundary_summary()
        .expect("reference lunar boundary summary should exist");
    assert_eq!(summary.sample_count, 2);
    assert_eq!(summary.epoch_count, 2);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_911.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_912.5);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference lunar boundary evidence: 2 exact Moon samples at JD 2451911.5 (TDB)..JD 2451912.5 (TDB); high-curvature interpolation window"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_lunar_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_high_curvature_summary_reports_the_expected_window() {
    let summary = reference_snapshot_high_curvature_summary()
        .expect("reference high-curvature summary should exist");
    assert_eq!(summary.sample_count, 50);
    assert_eq!(summary.body_count, 10);
    assert_eq!(summary.bodies.len(), 10);
    assert_eq!(summary.epoch_count, 5);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_911.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_916.5);
    assert_eq!(summary.bodies[0], pleiades_backend::CelestialBody::Sun);
    assert_eq!(summary.bodies[9], pleiades_backend::CelestialBody::Jupiter);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference major-body high-curvature evidence: 50 exact samples across 10 bodies and 5 epochs (JD 2451911.5 (TDB)..JD 2451916.5 (TDB)); bodies: Sun, Moon, Mercury, Venus, Saturn, Uranus, Neptune, Pluto, Mars, Jupiter; high-curvature interpolation window"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_high_curvature_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_boundary_epoch_coverage_summary_reports_the_sparse_epochs() {
    let summary = reference_snapshot_boundary_epoch_coverage_summary()
        .expect("reference snapshot boundary epoch coverage summary should exist");
    assert_eq!(summary.sample_count, 183);
    assert_eq!(summary.epoch_count, 14);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_912.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_919.5);
    assert_eq!(summary.windows.len(), 14);
    assert_eq!(summary.windows[0].body_count, 15);
    assert_eq!(summary.windows[3].body_count, 15);
    assert_eq!(
        summary.windows[3].bodies[0],
        pleiades_backend::CelestialBody::Ceres
    );
    assert_eq!(
        summary.windows[3].bodies[14],
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert!(
            summary
                .summary_line()
                .contains("Reference snapshot boundary epoch coverage: 183 exact samples across 14 epochs (JD 2451912.5 (TDB)..JD 2451919.5 (TDB)); epochs:")
        );
    assert!(summary.summary_line().contains(
            "JD 2451914.0 (TDB): 15 bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros)"
        ));
    assert!(summary
            .summary_line()
            .contains("JD 2451915.5 (TDB): 16 bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"));
    assert!(summary
            .summary_line()
            .contains("JD 2451916.0 (TDB): 10 bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto)"));
    assert!(summary.summary_line().contains(
            "JD 2451919.5 (TDB): 16 bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"
        ));

    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_boundary_epoch_coverage_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_boundary_epoch_coverage_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_boundary_epoch_coverage_summary()
        .expect("reference snapshot boundary epoch coverage summary should exist");
    summary.windows[2].body_count += 1;

    let error = summary
        .validate()
        .expect_err("drifted boundary epoch coverage summary should fail validation");

    assert!(matches!(
        error,
        ReferenceSnapshotBoundaryEpochCoverageSummaryValidationError::FieldOutOfSync {
            field: "windows"
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_boundary_epoch_coverage_summary_for_report(),
        reference_snapshot_boundary_epoch_coverage_summary()
            .expect("reference snapshot boundary epoch coverage summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_pre_bridge_boundary_summary_reports_the_pre_bridge_day() {
    let summary = reference_snapshot_pre_bridge_boundary_summary()
        .expect("reference snapshot pre-bridge boundary summary should exist");
    assert_eq!(summary.sample_count, 15);
    assert_eq!(summary.sample_bodies.len(), 15);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_914.5);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference snapshot pre-bridge boundary day: 15 exact samples at JD 2451914.5 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); pre-bridge boundary day"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_pre_bridge_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_bridge_day_summary_reports_the_bridge_day() {
    let summary = reference_snapshot_bridge_day_summary()
        .expect("reference snapshot bridge day summary should exist");
    assert_eq!(summary.sample_count, 15);
    assert_eq!(summary.sample_bodies.len(), 15);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_914.0);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference snapshot bridge day: 15 exact samples at JD 2451914.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); bridge sample across the reference boundary window"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_bridge_day_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
        validated_reference_snapshot_bridge_day_summary_for_report(),
        Ok(summary.summary_line())
    );
    assert_eq!(
        reference_snapshot_2451914_bridge_day_summary(),
        Some(summary.clone())
    );
    assert_eq!(
        reference_snapshot_2451914_bridge_day_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
        reference_snapshot_2451914_bridge_day_summary(),
        Some(summary.clone())
    );
    assert_eq!(
        reference_snapshot_2451914_major_body_bridge_day_summary(),
        Some(summary.clone())
    );
    assert_eq!(
        reference_snapshot_2451914_major_body_bridge_summary(),
        Some(summary.clone())
    );
    assert_eq!(
            reference_snapshot_2451914_major_body_bridge_day_summary_for_report(),
            "Reference 2451914 major-body bridge-day evidence: 15 exact samples at JD 2451914.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); 2451914 major-body bridge-day sample"
        );
}

#[test]
fn reference_snapshot_bridge_day_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_bridge_day_summary()
        .expect("reference snapshot bridge day summary should exist");
    summary.sample_count += 1;

    let error = summary
        .validate()
        .expect_err("drifted bridge day summary should fail validation");

    assert!(matches!(
        error,
        ReferenceSnapshotBridgeDaySummaryValidationError::SampleCountMismatch {
            sample_count: 16,
            derived_sample_count: 15
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_bridge_day_summary_for_report(),
        reference_snapshot_bridge_day_summary()
            .expect("reference snapshot bridge day summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_bridge_day_summary_validation_rejects_body_drift() {
    let mut summary = reference_snapshot_bridge_day_summary()
        .expect("reference snapshot bridge day summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted bridge day summary should fail validation");

    assert!(matches!(
        error,
        ReferenceSnapshotBridgeDaySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_bridge_day_summary_for_report(),
        reference_snapshot_bridge_day_summary()
            .expect("reference snapshot bridge day summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_sparse_boundary_summary_reports_the_asteroid_only_day() {
    let summary = reference_snapshot_sparse_boundary_summary()
        .expect("reference snapshot sparse boundary summary should exist");
    assert_eq!(summary.sample_count, 16);
    assert_eq!(summary.sample_bodies.len(), 16);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_915.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[1],
        pleiades_backend::CelestialBody::Moon
    );
    assert_eq!(
        summary.sample_bodies[2],
        pleiades_backend::CelestialBody::Mercury
    );
    assert_eq!(
        summary.sample_bodies[3],
        pleiades_backend::CelestialBody::Venus
    );
    assert_eq!(
        summary.sample_bodies[4],
        pleiades_backend::CelestialBody::Mars
    );
    assert_eq!(
        summary.sample_bodies[5],
        pleiades_backend::CelestialBody::Jupiter
    );
    assert_eq!(
        summary.sample_bodies[6],
        pleiades_backend::CelestialBody::Saturn
    );
    assert_eq!(
        summary.sample_bodies[7],
        pleiades_backend::CelestialBody::Uranus
    );
    assert_eq!(
        summary.sample_bodies[8],
        pleiades_backend::CelestialBody::Neptune
    );
    assert_eq!(
        summary.sample_bodies[9],
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(
        summary.sample_bodies[10],
        pleiades_backend::CelestialBody::Ceres
    );
    assert_eq!(
        summary.sample_bodies[11],
        pleiades_backend::CelestialBody::Pallas
    );
    assert_eq!(
        summary.sample_bodies[12],
        pleiades_backend::CelestialBody::Juno
    );
    assert_eq!(
        summary.sample_bodies[13],
        pleiades_backend::CelestialBody::Vesta
    );
    assert_eq!(
        summary.sample_bodies[14],
        pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
    );
    assert_eq!(
        summary.sample_bodies[15],
        pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "99942-Apophis"))
    );
    assert!(summary.missing_bodies.is_empty());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference snapshot boundary day: 16 exact samples at JD 2451915.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_sparse_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_sparse_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_sparse_boundary_summary()
        .expect("reference snapshot sparse boundary summary should exist");
    summary.sample_bodies.swap(0, 1);

    let error = summary
        .validate()
        .expect_err("drifted sparse boundary summary should fail validation");

    assert!(matches!(
        error,
        ReferenceSnapshotSparseBoundarySummaryValidationError::BodyOrderMismatch { index: 0, .. }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_sparse_boundary_summary_for_report(),
        reference_snapshot_sparse_boundary_summary()
            .expect("reference snapshot sparse boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_sparse_boundary_summary_validation_rejects_missing_body_drift() {
    let mut summary = reference_snapshot_sparse_boundary_summary()
        .expect("reference snapshot boundary day summary should exist");
    summary
        .missing_bodies
        .push(pleiades_backend::CelestialBody::Mercury);

    let error = summary
        .validate()
        .expect_err("drifted boundary day summary should fail validation");

    assert!(error
        .to_string()
        .contains("reference snapshot boundary day"));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn reference_snapshot_dense_boundary_summary_reports_the_dense_boundary_day() {
    let summary = reference_snapshot_dense_boundary_summary()
        .expect("reference snapshot dense boundary summary should exist");
    assert_eq!(summary.sample_count, 15);
    assert_eq!(summary.sample_bodies.len(), 15);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_916.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[14],
        pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference snapshot dense boundary day: 15 exact samples at JD 2451916.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); dense boundary day"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_dense_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_dense_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_dense_boundary_summary()
        .expect("reference snapshot dense boundary summary should exist");
    summary.sample_bodies.swap(0, 1);

    let error = summary
        .validate()
        .expect_err("drifted dense boundary summary should fail validation");

    assert!(matches!(
        error,
        ReferenceSnapshotDenseBoundarySummaryValidationError::BodyOrderMismatch { index: 0, .. }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_dense_boundary_summary_for_report(),
        reference_snapshot_dense_boundary_summary()
            .expect("reference snapshot dense boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_1500_selected_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_1500_selected_body_boundary_summary()
        .expect("reference 1500 selected-body boundary summary should exist");
    assert_eq!(summary.sample_count, 4);
    assert_eq!(summary.sample_bodies.len(), 4);
    assert_eq!(summary.epoch.julian_day.days(), 2_268_932.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[1],
        pleiades_backend::CelestialBody::Moon
    );
    assert_eq!(
        summary.sample_bodies[2],
        pleiades_backend::CelestialBody::Mercury
    );
    assert_eq!(
        summary.sample_bodies[3],
        pleiades_backend::CelestialBody::Venus
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 1500 selected-body boundary evidence: 4 exact samples at JD 2268932.5 (TDB) (Sun, Moon, Mercury, Venus); 1500-01-01 selected-body boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_1500_selected_body_boundary_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
        reference_snapshot_2268932_selected_body_boundary_summary()
            .expect("reference 2268932 selected-body boundary summary should exist")
            .summary_line(),
        summary.summary_line()
    );
    assert_eq!(
        reference_snapshot_2268932_selected_body_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_1600_selected_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_1600_selected_body_boundary_summary()
        .expect("reference 1600 selected-body boundary summary should exist");
    assert_eq!(summary.sample_count, 8);
    assert_eq!(summary.sample_bodies.len(), 8);
    assert_eq!(summary.epoch.julian_day.days(), 2_305_457.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[1],
        pleiades_backend::CelestialBody::Moon
    );
    assert_eq!(
        summary.sample_bodies[2],
        pleiades_backend::CelestialBody::Mercury
    );
    assert_eq!(
        summary.sample_bodies[3],
        pleiades_backend::CelestialBody::Venus
    );
    assert_eq!(
        summary.sample_bodies[4],
        pleiades_backend::CelestialBody::Mars
    );
    assert_eq!(
        summary.sample_bodies[5],
        pleiades_backend::CelestialBody::Jupiter
    );
    assert_eq!(
        summary.sample_bodies[6],
        pleiades_backend::CelestialBody::Uranus
    );
    assert_eq!(
        summary.sample_bodies[7],
        pleiades_backend::CelestialBody::Neptune
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 1600 selected-body boundary evidence: 8 exact samples at JD 2305457.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Uranus, Neptune); 1600-01-11 selected-body boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_1600_selected_body_boundary_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
        reference_snapshot_2305457_selected_body_boundary_summary()
            .expect("reference 2305457 selected-body boundary summary should exist")
            .summary_line(),
        summary.summary_line()
    );
    assert_eq!(
        reference_snapshot_2305457_selected_body_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_1900_selected_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_1900_selected_body_boundary_summary()
        .expect("reference 1900 selected-body boundary summary should exist");
    assert_eq!(summary.sample_count, 4);
    assert_eq!(summary.sample_bodies.len(), 4);
    assert_eq!(summary.epoch.julian_day.days(), 2_415_020.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[1],
        pleiades_backend::CelestialBody::Moon
    );
    assert_eq!(
        summary.sample_bodies[2],
        pleiades_backend::CelestialBody::Mercury
    );
    assert_eq!(
        summary.sample_bodies[3],
        pleiades_backend::CelestialBody::Venus
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 1900 selected-body boundary evidence: 4 exact samples at JD 2415020.5 (TDB) (Sun, Moon, Mercury, Venus); 1900-01-01 selected-body boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_1900_selected_body_boundary_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
            reference_snapshot_2415020_selected_body_boundary_summary_for_report(),
            "Reference 2415020 selected-body boundary evidence: 4 exact samples at JD 2415020.5 (TDB) (Sun, Moon, Mercury, Venus); 1900-01-01 selected-body boundary sample"
        );
}

#[test]
fn reference_snapshot_2360234_major_body_interior_summary_reports_the_interior_day() {
    let summary = reference_snapshot_2360234_major_body_interior_summary()
        .expect("reference 2360234 major-body interior comparison summary should exist");
    assert_eq!(summary.sample_count, 9);
    assert_eq!(summary.sample_bodies.len(), 9);
    assert_eq!(summary.epoch.julian_day.days(), 2_360_234.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[1],
        pleiades_backend::CelestialBody::Moon
    );
    assert_eq!(
        summary.sample_bodies[2],
        pleiades_backend::CelestialBody::Mercury
    );
    assert_eq!(
        summary.sample_bodies[3],
        pleiades_backend::CelestialBody::Venus
    );
    assert_eq!(
        summary.sample_bodies[4],
        pleiades_backend::CelestialBody::Mars
    );
    assert_eq!(
        summary.sample_bodies[5],
        pleiades_backend::CelestialBody::Jupiter
    );
    assert_eq!(
        summary.sample_bodies[6],
        pleiades_backend::CelestialBody::Saturn
    );
    assert_eq!(
        summary.sample_bodies[7],
        pleiades_backend::CelestialBody::Uranus
    );
    assert_eq!(
        summary.sample_bodies[8],
        pleiades_backend::CelestialBody::Neptune
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2360234 major-body interior comparison evidence: 9 exact samples at JD 2360234.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune); 1750-01-01 interior comparison sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2360234_major_body_interior_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_1750_major_body_interior_summary_reports_the_interior_day() {
    let summary = reference_snapshot_1750_major_body_interior_summary()
        .expect("reference 1750 major-body interior summary should exist");
    assert_eq!(summary.sample_count, 9);
    assert_eq!(summary.sample_bodies.len(), 9);
    assert_eq!(summary.epoch.julian_day.days(), 2_360_234.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[1],
        pleiades_backend::CelestialBody::Moon
    );
    assert_eq!(
        summary.sample_bodies[2],
        pleiades_backend::CelestialBody::Mercury
    );
    assert_eq!(
        summary.sample_bodies[3],
        pleiades_backend::CelestialBody::Venus
    );
    assert_eq!(
        summary.sample_bodies[4],
        pleiades_backend::CelestialBody::Mars
    );
    assert_eq!(
        summary.sample_bodies[5],
        pleiades_backend::CelestialBody::Jupiter
    );
    assert_eq!(
        summary.sample_bodies[6],
        pleiades_backend::CelestialBody::Saturn
    );
    assert_eq!(
        summary.sample_bodies[7],
        pleiades_backend::CelestialBody::Uranus
    );
    assert_eq!(
        summary.sample_bodies[8],
        pleiades_backend::CelestialBody::Neptune
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 1750 major-body interior comparison evidence: 9 exact samples at JD 2360234.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune); 1750-01-01 interior comparison sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_1750_major_body_interior_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_2200_selected_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_2200_selected_body_boundary_summary()
        .expect("reference 2200 selected-body boundary summary should exist");
    assert_eq!(summary.sample_count, 4);
    assert_eq!(summary.sample_bodies.len(), 4);
    assert_eq!(summary.epoch.julian_day.days(), 2_524_593.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[1],
        pleiades_backend::CelestialBody::Moon
    );
    assert_eq!(
        summary.sample_bodies[2],
        pleiades_backend::CelestialBody::Mercury
    );
    assert_eq!(
        summary.sample_bodies[3],
        pleiades_backend::CelestialBody::Venus
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2200 selected-body boundary evidence: 4 exact samples at JD 2524593.5 (TDB) (Sun, Moon, Mercury, Venus); 2200-01-01 selected-body boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2200_selected_body_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_2500_selected_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_2500_selected_body_boundary_summary()
        .expect("reference 2500 selected-body boundary summary should exist");
    assert_eq!(summary.sample_count, 5);
    assert_eq!(summary.sample_bodies.len(), 5);
    assert_eq!(summary.epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Mars
    );
    assert_eq!(
        summary.sample_bodies[1],
        pleiades_backend::CelestialBody::Mercury
    );
    assert_eq!(
        summary.sample_bodies[2],
        pleiades_backend::CelestialBody::Moon
    );
    assert_eq!(
        summary.sample_bodies[3],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[4],
        pleiades_backend::CelestialBody::Venus
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2500 selected-body boundary evidence: 5 exact samples at JD 2634167.0 (TDB) (Mars, Mercury, Moon, Sun, Venus); 2500-01-01 selected-body boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2500_selected_body_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_2634167_selected_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_2634167_selected_body_boundary_summary()
        .expect("reference 2634167 selected-body boundary summary should exist");
    assert_eq!(summary.sample_count, 5);
    assert_eq!(summary.sample_bodies.len(), 5);
    assert_eq!(summary.epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Mars
    );
    assert_eq!(
        summary.sample_bodies[1],
        pleiades_backend::CelestialBody::Mercury
    );
    assert_eq!(
        summary.sample_bodies[2],
        pleiades_backend::CelestialBody::Moon
    );
    assert_eq!(
        summary.sample_bodies[3],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[4],
        pleiades_backend::CelestialBody::Venus
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2500 selected-body boundary evidence: 5 exact samples at JD 2634167.0 (TDB) (Mars, Mercury, Moon, Sun, Venus); 2500-01-01 selected-body boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
            reference_snapshot_2634167_selected_body_boundary_summary_for_report(),
            "Reference 2634167 selected-body boundary evidence: 5 exact samples at JD 2634167.0 (TDB) (Mars, Mercury, Moon, Sun, Venus); 2500-01-01 selected-body boundary sample"
        );
}

#[test]
fn reference_snapshot_1749_major_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_1749_major_body_boundary_summary()
        .expect("reference 1749 major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 9);
    assert_eq!(summary.sample_bodies.len(), 9);
    assert_eq!(summary.epoch.julian_day.days(), 2_360_233.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[1],
        pleiades_backend::CelestialBody::Moon
    );
    assert_eq!(
        summary.sample_bodies[2],
        pleiades_backend::CelestialBody::Mercury
    );
    assert_eq!(
        summary.sample_bodies[3],
        pleiades_backend::CelestialBody::Venus
    );
    assert_eq!(
        summary.sample_bodies[4],
        pleiades_backend::CelestialBody::Mars
    );
    assert_eq!(
        summary.sample_bodies[5],
        pleiades_backend::CelestialBody::Jupiter
    );
    assert_eq!(
        summary.sample_bodies[6],
        pleiades_backend::CelestialBody::Saturn
    );
    assert_eq!(
        summary.sample_bodies[7],
        pleiades_backend::CelestialBody::Uranus
    );
    assert_eq!(
        summary.sample_bodies[8],
        pleiades_backend::CelestialBody::Neptune
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 1749 major-body boundary evidence: 9 exact samples at JD 2360233.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune); 1749-12-31 boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_1749_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_2360233_major_body_boundary_summary_alias_uses_explicit_2360233_wording() {
    let boundary_2360233 = reference_snapshot_2360233_major_body_boundary_summary_for_report();
    let boundary_generic = reference_snapshot_1749_major_body_boundary_summary_for_report();
    let summary = reference_snapshot_2360233_major_body_boundary_summary()
        .expect("reference 2360233 major-body boundary summary should exist");

    assert!(boundary_2360233.contains("Reference 2360233 major-body boundary evidence:"));
    assert!(boundary_2360233.contains("JD 2360233.5 (TDB)"));
    assert_eq!(
        boundary_2360233,
        summary.summary_line().replacen(
            "Reference 1749 major-body boundary evidence",
            "Reference 2360233 major-body boundary evidence",
            1
        )
    );
    assert_eq!(
        summary.summary_line(),
        reference_snapshot_1749_major_body_boundary_summary()
            .expect("reference 1749 major-body boundary summary should exist")
            .summary_line()
    );
    assert_ne!(boundary_2360233, boundary_generic);
}

#[test]
fn reference_snapshot_major_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_major_body_boundary_summary()
        .expect("reference major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_917.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[1],
        pleiades_backend::CelestialBody::Moon
    );
    assert_eq!(
        summary.sample_bodies[2],
        pleiades_backend::CelestialBody::Mercury
    );
    assert_eq!(
        summary.sample_bodies[3],
        pleiades_backend::CelestialBody::Venus
    );
    assert_eq!(
        summary.sample_bodies[4],
        pleiades_backend::CelestialBody::Mars
    );
    assert_eq!(
        summary.sample_bodies[5],
        pleiades_backend::CelestialBody::Jupiter
    );
    assert_eq!(
        summary.sample_bodies[6],
        pleiades_backend::CelestialBody::Saturn
    );
    assert_eq!(
        summary.sample_bodies[7],
        pleiades_backend::CelestialBody::Uranus
    );
    assert_eq!(
        summary.sample_bodies[8],
        pleiades_backend::CelestialBody::Neptune
    );
    assert_eq!(
        summary.sample_bodies[9],
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference major-body boundary evidence: 10 exact samples at JD 2451917.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-08 boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_major_body_bridge_summary_reports_the_bridge_day() {
    let summary = reference_snapshot_major_body_bridge_summary()
        .expect("reference major-body bridge summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_915.0);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference major-body bridge evidence: 10 exact samples at JD 2451915.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); bridge sample across the major-body boundary window"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_major_body_bridge_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_major_body_bridge_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_major_body_bridge_summary()
        .expect("reference major-body bridge summary should exist");
    summary.sample_count += 1;

    let error = summary
        .validate()
        .expect_err("drifted major-body bridge summary should fail validation");

    assert!(matches!(
        error,
        ReferenceMajorBodyBridgeSummaryValidationError::SampleCountMismatch {
            sample_count: 11,
            derived_sample_count: 10
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_major_body_bridge_summary_for_report(),
        reference_snapshot_major_body_bridge_summary()
            .expect("reference major-body bridge summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_1749_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_1749_major_body_boundary_summary()
        .expect("reference 1749 major-body boundary summary should exist");
    summary.sample_count += 1;

    let error = summary
        .validate()
        .expect_err("drifted 1749 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        Reference1749MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
            sample_count: 10,
            derived_sample_count: 9
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_1749_major_body_boundary_summary_for_report(),
        reference_snapshot_1749_major_body_boundary_summary()
            .expect("reference 1749 major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_major_body_boundary_summary()
        .expect("reference major-body boundary summary should exist");
    summary.sample_count += 1;

    let error = summary
        .validate()
        .expect_err("drifted major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        ReferenceMajorBodyBoundarySummaryValidationError::SampleCountMismatch {
            sample_count: 11,
            derived_sample_count: 10
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_major_body_boundary_summary_for_report(),
        reference_snapshot_major_body_boundary_summary()
            .expect("reference major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_major_body_boundary_summary_validation_rejects_body_drift() {
    let mut summary = reference_snapshot_major_body_boundary_summary()
        .expect("reference major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        ReferenceMajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_major_body_boundary_summary_for_report(),
        reference_snapshot_major_body_boundary_summary()
            .expect("reference major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_mars_jupiter_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_mars_jupiter_boundary_summary()
        .expect("reference Mars/Jupiter boundary summary should exist");
    assert_eq!(summary.sample_count, 16);
    assert_eq!(summary.sample_bodies.len(), 16);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_918.5);
    assert_eq!(
        summary.sample_bodies,
        vec![
            pleiades_backend::CelestialBody::Sun,
            pleiades_backend::CelestialBody::Moon,
            pleiades_backend::CelestialBody::Mercury,
            pleiades_backend::CelestialBody::Venus,
            pleiades_backend::CelestialBody::Mars,
            pleiades_backend::CelestialBody::Jupiter,
            pleiades_backend::CelestialBody::Saturn,
            pleiades_backend::CelestialBody::Uranus,
            pleiades_backend::CelestialBody::Neptune,
            pleiades_backend::CelestialBody::Pluto,
            pleiades_backend::CelestialBody::Ceres,
            pleiades_backend::CelestialBody::Pallas,
            pleiades_backend::CelestialBody::Juno,
            pleiades_backend::CelestialBody::Vesta,
            pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
            pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "99942-Apophis")),
        ]
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference Mars/Jupiter boundary evidence: 16 exact samples at JD 2451918.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); 2001-01-09 boundary sample"
        );
    assert_eq!(
            reference_snapshot_2451918_major_body_boundary_summary_for_report(),
            "Reference 2451918 major-body boundary evidence: 16 exact samples at JD 2451918.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis); 2001-01-09 boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_mars_jupiter_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_mars_jupiter_boundary_summary_validation_rejects_body_drift() {
    let mut summary = reference_snapshot_mars_jupiter_boundary_summary()
        .expect("reference Mars/Jupiter boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted Mars/Jupiter boundary summary should fail validation");

    assert!(matches!(
        error,
        ReferenceMarsJupiterBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_mars_jupiter_boundary_summary_for_report(),
        reference_snapshot_mars_jupiter_boundary_summary()
            .expect("reference Mars/Jupiter boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2451918_major_body_boundary_summary_alias_uses_explicit_2451918_wording() {
    let boundary_2451918 = reference_snapshot_2451918_major_body_boundary_summary_for_report();
    let boundary_generic = reference_snapshot_mars_jupiter_boundary_summary_for_report();
    let summary = reference_snapshot_2451918_major_body_boundary_summary()
        .expect("reference 2451918 major-body boundary summary should exist");

    assert!(boundary_2451918.contains("Reference 2451918 major-body boundary evidence:"));
    assert!(boundary_2451918.contains("JD 2451918.5 (TDB)"));
    assert_eq!(
        boundary_2451918,
        summary.summary_line().replacen(
            "Reference Mars/Jupiter boundary evidence",
            "Reference 2451918 major-body boundary evidence",
            1
        )
    );
    assert_eq!(
        summary.summary_line(),
        reference_snapshot_mars_jupiter_boundary_summary()
            .expect("reference Mars/Jupiter boundary summary should exist")
            .summary_line()
    );
    assert_ne!(boundary_2451918, boundary_generic);
}

#[test]
fn reference_snapshot_2451914_and_2451915_boundary_aliases_match_the_generic_reports() {
    assert_eq!(
        reference_snapshot_2451914_major_body_pre_bridge_summary_for_report(),
        reference_snapshot_pre_bridge_boundary_summary_for_report()
    );
    assert_eq!(
        reference_snapshot_2451914_major_body_pre_bridge_summary()
            .expect("reference 2451914 major-body pre-bridge summary should exist")
            .summary_line(),
        reference_snapshot_pre_bridge_boundary_summary()
            .expect("reference pre-bridge boundary summary should exist")
            .summary_line()
    );

    assert_eq!(
        reference_snapshot_2451914_major_body_bridge_summary_for_report(),
        reference_snapshot_bridge_day_summary_for_report()
    );
    assert_eq!(
        reference_snapshot_2451914_major_body_bridge_summary()
            .expect("reference 2451914 major-body bridge summary should exist")
            .summary_line(),
        reference_snapshot_bridge_day_summary()
            .expect("reference bridge day summary should exist")
            .summary_line()
    );

    let summary = reference_snapshot_2451915_major_body_bridge_summary()
        .expect("reference 2451915 major-body bridge summary should exist");
    assert_eq!(
            reference_snapshot_2451915_major_body_bridge_summary_for_report(),
            format!(
                "Reference 2451915 major-body bridge evidence: {} exact samples at {} ({}); 2451915 major-body bridge sample",
                summary.sample_count,
                format_instant(summary.epoch),
                format_bodies(&summary.sample_bodies),
            )
        );
    assert_eq!(
        summary.summary_line(),
        reference_snapshot_major_body_bridge_summary()
            .expect("reference major-body bridge summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2451918_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2451918_major_body_boundary_summary()
        .expect("reference 2451918 major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2451918 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        ReferenceMarsJupiterBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2451918_major_body_boundary_summary_for_report(),
        reference_snapshot_2451918_major_body_boundary_summary()
            .expect("reference 2451918 major-body boundary summary should exist")
            .summary_line()
            .replacen(
                "Reference Mars/Jupiter boundary evidence",
                "Reference 2451918 major-body boundary evidence",
                1
            )
    );
}

#[test]
fn reference_snapshot_early_major_body_boundary_summary_reports_the_early_boundary_day() {
    let summary = reference_snapshot_early_major_body_boundary_summary()
        .expect("reference early major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[9],
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference early major-body boundary evidence: 10 exact samples at JD 2378498.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto)"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_early_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_2378498_major_body_boundary_summary_alias_uses_explicit_2378498_wording() {
    let boundary_2378498 = reference_snapshot_2378498_major_body_boundary_summary_for_report();
    let boundary_generic = reference_snapshot_early_major_body_boundary_summary_for_report();
    let summary = reference_snapshot_2378498_major_body_boundary_summary()
        .expect("reference 2378498 major-body boundary summary should exist");

    assert!(boundary_2378498.contains("Reference 2378498 major-body boundary evidence:"));
    assert!(boundary_2378498.contains("JD 2378498.5 (TDB)"));
    assert_eq!(
        boundary_2378498,
        summary.summary_line().replacen(
            "Reference early major-body boundary evidence",
            "Reference 2378498 major-body boundary evidence",
            1
        )
    );
    assert_eq!(
        summary.summary_line(),
        reference_snapshot_early_major_body_boundary_summary()
            .expect("reference early major-body boundary summary should exist")
            .summary_line()
    );
    assert_ne!(boundary_2378498, boundary_generic);
}

#[test]
fn reference_snapshot_early_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_early_major_body_boundary_summary()
        .expect("reference early major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted early major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        ReferenceEarlyMajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_early_major_body_boundary_summary_for_report(),
        reference_snapshot_early_major_body_boundary_summary()
            .expect("reference early major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_1800_major_body_boundary_summary_reports_the_1800_boundary_day() {
    let summary = reference_snapshot_1800_major_body_boundary_summary()
        .expect("reference 1800 major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_378_499.0);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Mars
    );
    assert_eq!(
        summary.sample_bodies[4],
        pleiades_backend::CelestialBody::Venus
    );
    assert_eq!(
        summary.sample_bodies[5],
        pleiades_backend::CelestialBody::Jupiter
    );
    assert_eq!(
        summary.sample_bodies[9],
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 1800 major-body boundary evidence: 10 exact samples at JD 2378499.0 (TDB) (Mars, Mercury, Moon, Sun, Venus, Jupiter, Saturn, Uranus, Neptune, Pluto); 1800-01-03 boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_1800_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_2378499_major_body_boundary_summary_alias_uses_explicit_2378499_wording() {
    let boundary_2378499 = reference_snapshot_2378499_major_body_boundary_summary_for_report();
    let boundary_generic = reference_snapshot_1800_major_body_boundary_summary_for_report();
    let summary = reference_snapshot_2378499_major_body_boundary_summary()
        .expect("reference 2378499 major-body boundary summary should exist");

    assert!(boundary_2378499.contains("Reference 2378499 major-body boundary evidence:"));
    assert!(boundary_2378499.contains("JD 2378499.0 (TDB)"));
    assert_eq!(
        boundary_2378499,
        summary.summary_line().replacen(
            "Reference 1800 major-body boundary evidence",
            "Reference 2378499 major-body boundary evidence",
            1
        )
    );
    assert_eq!(
        summary.summary_line(),
        reference_snapshot_1800_major_body_boundary_summary()
            .expect("reference 1800 major-body boundary summary should exist")
            .summary_line()
    );
    assert_ne!(boundary_2378499, boundary_generic);
}

#[test]
fn reference_snapshot_1800_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_1800_major_body_boundary_summary()
        .expect("reference 1800 major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 1800 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        Reference1800MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Mars,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_1800_major_body_boundary_summary_for_report(),
        reference_snapshot_1800_major_body_boundary_summary()
            .expect("reference 1800 major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2500_major_body_boundary_summary_reports_the_terminal_boundary_day() {
    let summary = reference_snapshot_2500_major_body_boundary_summary()
        .expect("reference 2500 major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_500_000.0);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[9],
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2500 major-body boundary evidence: 10 exact samples at JD 2500000.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2500-01-01 boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2500_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_2500_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2500_major_body_boundary_summary()
        .expect("reference 2500 major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2500 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        Reference2500MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2500_major_body_boundary_summary_for_report(),
        reference_snapshot_2500_major_body_boundary_summary()
            .expect("reference 2500 major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2451910_major_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_2451910_major_body_boundary_summary()
        .expect("reference 2451910 major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_910.5);
    assert_eq!(
        summary.sample_bodies,
        vec![
            pleiades_backend::CelestialBody::Sun,
            pleiades_backend::CelestialBody::Moon,
            pleiades_backend::CelestialBody::Mercury,
            pleiades_backend::CelestialBody::Venus,
            pleiades_backend::CelestialBody::Mars,
            pleiades_backend::CelestialBody::Jupiter,
            pleiades_backend::CelestialBody::Saturn,
            pleiades_backend::CelestialBody::Uranus,
            pleiades_backend::CelestialBody::Neptune,
            pleiades_backend::CelestialBody::Pluto,
        ]
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2451910 major-body boundary evidence: 10 exact samples at JD 2451910.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-01 boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2451910_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
    assert!(reference_snapshot_summary_for_report().contains(summary.summary_line().as_str()));
}

#[test]
fn reference_snapshot_2451910_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2451910_major_body_boundary_summary()
        .expect("reference 2451910 major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2451910 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        Reference2451910MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2451910_major_body_boundary_summary_for_report(),
        reference_snapshot_2451910_major_body_boundary_summary()
            .expect("reference 2451910 major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2451911_major_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_2451911_major_body_boundary_summary()
        .expect("reference 2451911 major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_911.5);
    assert_eq!(
        summary.sample_bodies,
        vec![
            pleiades_backend::CelestialBody::Sun,
            pleiades_backend::CelestialBody::Moon,
            pleiades_backend::CelestialBody::Mercury,
            pleiades_backend::CelestialBody::Venus,
            pleiades_backend::CelestialBody::Saturn,
            pleiades_backend::CelestialBody::Uranus,
            pleiades_backend::CelestialBody::Neptune,
            pleiades_backend::CelestialBody::Pluto,
            pleiades_backend::CelestialBody::Mars,
            pleiades_backend::CelestialBody::Jupiter,
        ]
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        reference_snapshot_2451911_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
    assert!(reference_snapshot_summary_for_report().contains(summary.summary_line().as_str()));
}

#[test]
fn reference_snapshot_2451911_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2451911_major_body_boundary_summary()
        .expect("reference 2451911 major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2451911 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        Reference2451911MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2451911_major_body_boundary_summary_for_report(),
        reference_snapshot_2451911_major_body_boundary_summary()
            .expect("reference 2451911 major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2451912_major_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_2451912_major_body_boundary_summary()
        .expect("reference 2451912 major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_912.5);
    assert_eq!(
        summary.sample_bodies,
        vec![
            pleiades_backend::CelestialBody::Sun,
            pleiades_backend::CelestialBody::Moon,
            pleiades_backend::CelestialBody::Mercury,
            pleiades_backend::CelestialBody::Venus,
            pleiades_backend::CelestialBody::Saturn,
            pleiades_backend::CelestialBody::Uranus,
            pleiades_backend::CelestialBody::Neptune,
            pleiades_backend::CelestialBody::Pluto,
            pleiades_backend::CelestialBody::Mars,
            pleiades_backend::CelestialBody::Jupiter,
        ]
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2451912 major-body boundary evidence: 10 exact samples at JD 2451912.5 (TDB) (Sun, Moon, Mercury, Venus, Saturn, Uranus, Neptune, Pluto, Mars, Jupiter); 2001-01-03 boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2451912_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_2451912_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2451912_major_body_boundary_summary()
        .expect("reference 2451912 major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2451912 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        Reference2451912MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2451912_major_body_boundary_summary_for_report(),
        reference_snapshot_2451912_major_body_boundary_summary()
            .expect("reference 2451912 major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2451913_major_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_2451913_major_body_boundary_summary()
        .expect("reference 2451913 major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_913.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[9],
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2451913 major-body boundary evidence: 10 exact samples at JD 2451913.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-04 boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2451913_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
    assert!(reference_snapshot_summary_for_report().contains(summary.summary_line().as_str()));
}

#[test]
fn reference_snapshot_2451913_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2451913_major_body_boundary_summary()
        .expect("reference 2451913 major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2451913 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        Reference2451913MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2451913_major_body_boundary_summary_for_report(),
        reference_snapshot_2451913_major_body_boundary_summary()
            .expect("reference 2451913 major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2451914_major_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_2451914_major_body_boundary_summary()
        .expect("reference 2451914 major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_914.5);
    assert_eq!(
        summary.sample_bodies,
        vec![
            pleiades_backend::CelestialBody::Sun,
            pleiades_backend::CelestialBody::Moon,
            pleiades_backend::CelestialBody::Mercury,
            pleiades_backend::CelestialBody::Venus,
            pleiades_backend::CelestialBody::Mars,
            pleiades_backend::CelestialBody::Jupiter,
            pleiades_backend::CelestialBody::Saturn,
            pleiades_backend::CelestialBody::Uranus,
            pleiades_backend::CelestialBody::Neptune,
            pleiades_backend::CelestialBody::Pluto,
        ]
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2451914 major-body boundary evidence: 10 exact samples at JD 2451914.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-05 boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2451914_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
    assert!(reference_snapshot_summary_for_report().contains(summary.summary_line().as_str()));
}

#[test]
fn reference_snapshot_2451914_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2451914_major_body_boundary_summary()
        .expect("reference 2451914 major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2451914 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        Reference2451914MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2451914_major_body_boundary_summary_for_report(),
        reference_snapshot_2451914_major_body_boundary_summary()
            .expect("reference 2451914 major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2451915_major_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_2451915_major_body_boundary_summary()
        .expect("reference 2451915 major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_915.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[9],
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2451915 major-body boundary evidence: 10 exact samples at JD 2451915.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-06 boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2451915_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
    assert!(reference_snapshot_summary_for_report().contains(summary.summary_line().as_str()));
}

#[test]
fn reference_snapshot_2451915_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2451915_major_body_boundary_summary()
        .expect("reference 2451915 major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2451915 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        Reference2451915MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2451915_major_body_boundary_summary_for_report(),
        reference_snapshot_2451915_major_body_boundary_summary()
            .expect("reference 2451915 major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2400000_major_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_2400000_major_body_boundary_summary()
        .expect("reference 2400000 major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_400_000.0);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[9],
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2400000 major-body boundary evidence: 10 exact samples at JD 2400000.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2400000.0 boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2400000_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
    assert!(reference_snapshot_summary_for_report().contains(summary.summary_line().as_str()));
}

#[test]
fn reference_snapshot_2400000_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2400000_major_body_boundary_summary()
        .expect("reference 2400000 major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2400000 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        Reference2400000MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2400000_major_body_boundary_summary_for_report(),
        reference_snapshot_2400000_major_body_boundary_summary()
            .expect("reference 2400000 major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2451545_major_body_boundary_summary_reports_the_j2000_reference_day() {
    let summary = reference_snapshot_2451545_major_body_boundary_summary()
        .expect("reference 2451545 major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_545.0);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Jupiter
    );
    assert_eq!(
        summary.sample_bodies[9],
        pleiades_backend::CelestialBody::Venus
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2451545 major-body boundary evidence: 10 exact samples at JD 2451545.0 (TDB) (Jupiter, Mars, Mercury, Moon, Neptune, Pluto, Saturn, Sun, Uranus, Venus); J2000 reference sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2451545_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
    assert!(reference_snapshot_summary_for_report().contains(summary.summary_line().as_str()));
}

#[test]
fn reference_snapshot_2451545_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2451545_major_body_boundary_summary()
        .expect("reference 2451545 major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2451545 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        Reference2451545MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Jupiter,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2451545_major_body_boundary_summary_for_report(),
        reference_snapshot_2451545_major_body_boundary_summary()
            .expect("reference 2451545 major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2453000_major_body_boundary_summary_reports_the_late_boundary_day() {
    let summary = reference_snapshot_2453000_major_body_boundary_summary()
        .expect("reference 2453000 major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_453_000.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[9],
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2453000 major-body boundary evidence: 10 exact samples at JD 2453000.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2453000.5 boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2453000_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
    assert!(reference_snapshot_summary_for_report().contains(summary.summary_line().as_str()));
}

#[test]
fn reference_snapshot_2500000_major_body_boundary_summary_reports_the_terminal_boundary_day() {
    let summary = reference_snapshot_2500000_major_body_boundary_summary()
        .expect("reference 2500000 major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_500_000.0);
    assert_eq!(
        summary.sample_bodies,
        vec![
            pleiades_backend::CelestialBody::Sun,
            pleiades_backend::CelestialBody::Moon,
            pleiades_backend::CelestialBody::Mercury,
            pleiades_backend::CelestialBody::Venus,
            pleiades_backend::CelestialBody::Mars,
            pleiades_backend::CelestialBody::Jupiter,
            pleiades_backend::CelestialBody::Saturn,
            pleiades_backend::CelestialBody::Uranus,
            pleiades_backend::CelestialBody::Neptune,
            pleiades_backend::CelestialBody::Pluto,
        ]
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2500000 major-body boundary evidence: 10 exact samples at JD 2500000.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2500000.0 boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2500000_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
    assert!(reference_snapshot_summary_for_report().contains(summary.summary_line().as_str()));
}

#[test]
fn reference_snapshot_2500000_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2500000_major_body_boundary_summary()
        .expect("reference 2500000 major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2500000 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        Reference2500000MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2500000_major_body_boundary_summary_for_report(),
        reference_snapshot_2500000_major_body_boundary_summary()
            .expect("reference 2500000 major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2451917_major_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_2451917_major_body_boundary_summary()
        .expect("reference 2451917 major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_917.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[9],
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2451917 major-body boundary evidence: 10 exact samples at JD 2451917.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-08 boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2451917_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_2451917_major_body_bridge_summary_reports_the_bridge_day() {
    let summary = reference_snapshot_2451917_major_body_bridge_summary()
        .expect("reference 2451917 major-body bridge summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_917.0);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2451917 major-body bridge evidence: 10 exact samples at JD 2451917.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); bridge sample across the major-body boundary window"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2451917_major_body_bridge_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_2451917_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2451917_major_body_boundary_summary()
        .expect("reference 2451917 major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2451917 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        Reference2451917MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2451917_major_body_boundary_summary_for_report(),
        reference_snapshot_2451917_major_body_boundary_summary()
            .expect("reference 2451917 major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2451916_major_body_interior_summary_reports_the_interior_day() {
    let summary = reference_snapshot_2451916_major_body_interior_summary()
        .expect("reference 2451916 major-body interior summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_916.0);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[9],
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2451916 major-body interior evidence: 10 exact samples at JD 2451916.0 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-06 interior reference sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2451916_major_body_interior_summary_for_report(),
        summary.summary_line()
    );
    assert!(reference_snapshot_summary_for_report().contains(summary.summary_line().as_str()));
}

#[test]
fn reference_snapshot_2451916_major_body_boundary_summary_aliases_the_dense_boundary_day() {
    let dense_summary = reference_snapshot_2451916_major_body_dense_boundary_summary()
        .expect("reference 2451916 major-body dense boundary summary should exist");
    let boundary_summary = reference_snapshot_2451916_major_body_boundary_summary()
        .expect("reference 2451916 major-body boundary summary should exist");
    assert_eq!(boundary_summary, dense_summary);
    assert_eq!(boundary_summary.validate(), Ok(()));
    assert_eq!(
        boundary_summary.validated_summary_line(),
        Ok(boundary_summary.summary_line())
    );
    assert_eq!(
        boundary_summary.summary_line(),
        dense_summary.summary_line()
    );
    assert_eq!(
            reference_snapshot_2451916_major_body_dense_boundary_summary_for_report(),
            "Reference 2451916 major-body dense boundary evidence: 15 exact samples at JD 2451916.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); dense boundary day"
        );
    assert_eq!(
            reference_snapshot_2451916_major_body_boundary_summary_for_report(),
            "Reference 2451916 major-body boundary evidence: 15 exact samples at JD 2451916.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros); dense boundary day"
        );
}

#[test]
fn reference_snapshot_2451916_major_body_interior_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2451916_major_body_interior_summary()
        .expect("reference 2451916 major-body interior summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2451916 major-body interior summary should fail validation");

    assert!(matches!(
        error,
        Reference2451916MajorBodyInteriorSummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2451916_major_body_interior_summary_for_report(),
        reference_snapshot_2451916_major_body_interior_summary()
            .expect("reference 2451916 major-body interior summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2451919_major_body_boundary_summary_reports_the_boundary_day() {
    let summary = reference_snapshot_2451919_major_body_boundary_summary()
        .expect("reference 2451919 major-body boundary summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_919.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[9],
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2451919 major-body boundary evidence: 10 exact samples at JD 2451919.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-10 boundary sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2451919_major_body_boundary_summary_for_report(),
        summary.summary_line()
    );
    assert!(reference_snapshot_summary_for_report().contains(summary.summary_line().as_str()));
}

#[test]
fn reference_snapshot_2451919_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2451919_major_body_boundary_summary()
        .expect("reference 2451919 major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2451919 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        Reference2451919MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2451919_major_body_boundary_summary_for_report(),
        reference_snapshot_2451919_major_body_boundary_summary()
            .expect("reference 2451919 major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_2451920_major_body_interior_summary_reports_the_interior_day() {
    let summary = reference_snapshot_2451920_major_body_interior_summary()
        .expect("reference 2451920 major-body interior summary should exist");
    assert_eq!(summary.sample_count, 10);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch.julian_day.days(), 2_451_920.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[9],
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference 2451920 major-body interior evidence: 10 exact samples at JD 2451920.5 (TDB) (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto); 2001-01-13 interior reference sample"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_2451920_major_body_interior_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_2451920_major_body_interior_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2451920_major_body_interior_summary()
        .expect("reference 2451920 major-body interior summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2451920 major-body interior summary should fail validation");

    assert!(matches!(
        error,
        Reference2451920MajorBodyInteriorSummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2451920_major_body_interior_summary_for_report(),
        reference_snapshot_2451920_major_body_interior_summary()
            .expect("reference 2451920 major-body interior summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_summary_for_report_highlights_recent_reference_slices() {
    let report = reference_snapshot_summary_for_report();
    assert!(report.contains(&reference_snapshot_1500_selected_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_1600_selected_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_source_summary_for_report()));
    assert!(report.contains(&reference_snapshot_source_window_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_2451917_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_2453000_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_2500000_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_2634167_summary_for_report()));
    assert!(report.contains(&reference_snapshot_major_body_bridge_summary_for_report()));
    assert!(report.contains(&reference_snapshot_equatorial_parity_summary_for_report()));
    assert!(report.contains(&reference_snapshot_batch_parity_summary_for_report()));
    assert!(report.contains(&reference_snapshot_1749_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2360233_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_early_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_1750_selected_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_1800_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2378499_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_1900_selected_body_boundary_summary_for_report()));
    assert!(
        report.contains(&reference_snapshot_2415020_selected_body_boundary_summary_for_report())
    );
    assert!(report.contains(&reference_snapshot_lunar_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_high_curvature_summary_for_report()));
    assert!(report.contains(&reference_snapshot_high_curvature_window_summary_for_report()));
    assert!(report.contains(&reference_snapshot_high_curvature_epoch_coverage_summary_for_report()));
    assert!(report.contains(&reference_snapshot_exact_j2000_evidence_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2360234_major_body_interior_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451910_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451911_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451912_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451913_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451914_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451914_major_body_pre_bridge_summary_for_report()));
    assert!(report.contains(&reference_snapshot_bridge_day_summary_for_report()));
    assert!(report.contains(&reference_snapshot_major_body_bridge_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451914_bridge_day_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451914_major_body_bridge_day_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451914_major_body_bridge_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451915_major_body_bridge_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451915_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451917_major_body_bridge_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451917_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451918_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451916_major_body_interior_summary_for_report()));
    assert!(report.contains(&selected_asteroid_boundary_summary_for_report()));
    assert!(report.contains(&selected_asteroid_bridge_summary_for_report()));
    assert!(report.contains(&selected_asteroid_dense_boundary_summary_for_report()));
    assert!(report.contains(&selected_asteroid_terminal_boundary_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_evidence_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_window_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2400000_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451545_major_body_boundary_summary_for_report()));
    assert!(
        report.contains(&reference_snapshot_2451916_major_body_dense_boundary_summary_for_report())
    );
    assert!(report.contains(&reference_snapshot_2451916_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_sparse_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_pre_bridge_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_boundary_epoch_coverage_summary_for_report()));
    assert!(report.contains(&reference_snapshot_major_body_boundary_window_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451919_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451920_major_body_interior_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2500000_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_mars_outer_boundary_summary_for_report()));
    assert!(!report.contains("JPL independent hold-out:"));
    assert!(!report.contains("Reference/hold-out overlap:"));
}

#[test]
fn reference_snapshot_2453000_major_body_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_2453000_major_body_boundary_summary()
        .expect("reference 2453000 major-body boundary summary should exist");
    summary.sample_bodies[0] = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted 2453000 major-body boundary summary should fail validation");

    assert!(matches!(
        error,
        Reference2453000MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: pleiades_backend::CelestialBody::Sun,
            found: pleiades_backend::CelestialBody::Moon
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_2453000_major_body_boundary_summary_for_report(),
        reference_snapshot_2453000_major_body_boundary_summary()
            .expect("reference 2453000 major-body boundary summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_high_curvature_window_summary_reports_the_expected_windows() {
    let summary = reference_snapshot_high_curvature_window_summary()
        .expect("reference high-curvature window summary should exist");
    assert_eq!(summary.sample_count, 50);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch_count, 5);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_451_911.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_451_916.5);
    assert_eq!(
        summary.sample_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.sample_bodies[9],
        pleiades_backend::CelestialBody::Jupiter
    );
    assert_eq!(summary.windows.len(), summary.sample_bodies.len());
    assert_eq!(
        summary.windows[0].body,
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(summary.windows[0].sample_count, 5);
    assert_eq!(summary.windows[0].epoch_count, 5);
    assert_eq!(
        summary.windows[9].body,
        pleiades_backend::CelestialBody::Jupiter
    );
    assert_eq!(summary.windows[9].sample_count, 5);
    assert_eq!(summary.windows[9].epoch_count, 5);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference major-body high-curvature windows: 50 source-backed samples across 10 bodies and 5 epochs (JD 2451911.5 (TDB)..JD 2451916.5 (TDB)); windows: Sun: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Moon: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Mercury: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Venus: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Saturn: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Uranus: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Neptune: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Pluto: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Mars: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB); Jupiter: 5 samples across 5 epochs at JD 2451911.5 (TDB)..JD 2451916.5 (TDB)"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_snapshot_high_curvature_window_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_high_curvature_window_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_high_curvature_window_summary()
        .expect("reference high-curvature window summary should exist");
    summary.sample_count += 1;

    let error = summary
        .validate()
        .expect_err("drifted high-curvature window summary should fail validation");

    assert!(matches!(
        error,
        ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
            field: "sample_count"
        }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_high_curvature_window_summary_for_report(),
        reference_snapshot_high_curvature_window_summary()
            .expect("reference high-curvature window summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_high_curvature_window_summary_validation_rejects_window_drift() {
    let mut summary = reference_snapshot_high_curvature_window_summary()
        .expect("reference high-curvature window summary should exist");
    summary.windows[0].body = pleiades_backend::CelestialBody::Moon;

    let error = summary
        .validate()
        .expect_err("drifted high-curvature window summary should fail validation");

    assert!(matches!(
        error,
        ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync { field: "windows" }
    ));
    assert!(summary.validated_summary_line().is_err());
    assert_eq!(
        reference_snapshot_high_curvature_window_summary_for_report(),
        reference_snapshot_high_curvature_window_summary()
            .expect("reference high-curvature window summary should exist")
            .summary_line()
    );
}

#[test]
fn reference_snapshot_lunar_boundary_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_lunar_boundary_summary()
        .expect("reference lunar boundary summary should exist");
    summary.sample_count += 1;

    let error = summary
        .validate()
        .expect_err("drifted lunar boundary summary should fail validation");

    assert!(matches!(
        error,
        ReferenceLunarBoundarySummaryValidationError::FieldOutOfSync {
            field: "sample_count"
        }
    ));
}

#[test]
fn reference_snapshot_high_curvature_summary_validation_rejects_drift() {
    let mut summary = reference_snapshot_high_curvature_summary()
        .expect("reference high-curvature summary should exist");
    summary.body_count += 1;

    let error = summary
        .validate()
        .expect_err("drifted high-curvature summary should fail validation");

    assert!(matches!(
        error,
        ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync {
            field: "body_count"
        }
    ));
}

#[test]
fn production_generation_snapshot_summary_reports_the_boundary_overlay() {
    let summary = production_generation_snapshot_summary()
        .expect("production-generation snapshot summary should exist");
    summary
        .validate()
        .expect("production-generation snapshot summary should validate");
    assert_eq!(summary.row_count, 357);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.bodies, reference_bodies());
    assert_eq!(summary.epoch_count, 31);
    assert_eq!(summary.boundary_row_count, 84);
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
            CelestialBody::Moon,
            CelestialBody::Pluto,
            CelestialBody::Ceres,
            CelestialBody::Pallas,
            CelestialBody::Juno,
            CelestialBody::Vesta,
            CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
            CelestialBody::Custom(CustomBodyId::new("asteroid", "99942-Apophis")),
        ]
    );
    assert_eq!(summary.boundary_epoch_count, 14);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_268_932.5);
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
                "Production generation coverage: 357 rows across 16 bodies and 31 epochs (JD 2268932.5 (TDB)..JD 2634167.0 (TDB)); bodies: {}; boundary overlay (Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2451915.25, 2451915.75, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2451915.25, 2451915.75, 2451915.5, 2500000, and 2634167, plus Moon at 2451545, 2451915.25, 2451915.75, 2451915.5, 2500000, and 2634167, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus Pluto at 2451545 and 2500000, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 84 rows across 16 bodies and 14 epochs.): 84 rows across 16 bodies and 14 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); boundary bodies: {}; quarter-day boundary samples: 8 rows across 4 bodies and 2 epochs (JD 2451915.25 (TDB)..JD 2451915.75 (TDB)); quarter-day bodies: {}",
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
        .contains("source windows=357 source-backed samples across 16 bodies and 31 epochs"));
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
    assert_eq!(summary.sample_count, 357);
    assert_eq!(summary.sample_bodies.len(), 16);
    assert_eq!(summary.windows.len(), summary.sample_bodies.len());
    assert_eq!(summary.sample_bodies, reference_bodies());
    assert_eq!(summary.epoch_count, 31);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_268_932.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(summary.windows[0].body, CelestialBody::Sun);
    assert!(summary.windows[0].sample_count >= 8);
    assert!(summary.windows[0].summary_line().starts_with("Sun: "));
    assert!(summary.summary_line().starts_with(
            "Production generation source windows: 357 source-backed samples across 16 bodies and 31 epochs (JD 2268932.5 (TDB)..JD 2634167.0 (TDB)); windows: "
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
    assert_eq!(summary.row_count, 357);
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
    assert_eq!(summary.row_count, 84);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.bodies, production_generation_boundary_body_list());
    assert_eq!(summary.epoch_count, 14);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(
            summary.summary_line(),
            "Production generation boundary overlay: 84 rows across 16 bodies and 14 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies: Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Moon, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis"
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
    assert_eq!(summary.sample_count, 84);
    assert_eq!(summary.sample_bodies.len(), 16);
    assert_eq!(
        summary.sample_bodies,
        production_generation_boundary_body_list().to_vec()
    );
    assert_eq!(summary.epoch_count, 14);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(summary.windows[0].body, CelestialBody::Mars);
    assert_eq!(summary.windows[0].sample_count, 8);
    assert_eq!(summary.windows[0].epoch_count, 8);
    assert_eq!(
        summary.windows[0].summary_line(),
        format!(
            "Mars: 8 samples across 8 epochs at {}..{}",
            format_instant(summary.windows[0].earliest_epoch),
            format_instant(summary.windows[0].latest_epoch)
        )
    );
    assert!(summary.summary_line().starts_with("Production generation boundary windows: 84 source-backed samples across 16 bodies and 14 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); windows: "));
    assert!(summary.summary_line().contains("Mars: 8 samples across 8 epochs at JD 2451545.0 (TDB)..JD 2634167.0 (TDB); Jupiter: 7 samples across 7 epochs at JD 2400000.0 (TDB)..JD 2500000.0 (TDB)"));
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
    assert_eq!(summary.row_count, 84);
    assert_eq!(summary.major_body_row_count, 52);
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
            CelestialBody::Moon,
            CelestialBody::Pluto,
        ]
    );
    assert_eq!(summary.major_epoch_count, 11);
    assert_eq!(summary.major_windows.len(), 10);
    assert_eq!(summary.asteroid_row_count, 32);
    assert_eq!(summary.asteroid_bodies.len(), 6);
    assert_eq!(summary.asteroid_epoch_count, 7);
    assert_eq!(summary.asteroid_windows.len(), 6);
    assert!(summary.summary_line().starts_with(
            "Production generation boundary body-class coverage: major bodies: 52 rows across 10 bodies and 11 epochs; major windows: "
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
fn reference_snapshot_summary_validation_rejects_body_count_drift() {
    let mut summary =
        reference_snapshot_summary().expect("reference snapshot summary should exist");
    summary.body_count += 1;

    let error = summary
        .validate()
        .expect_err("body-count drift should be rejected");
    assert_eq!(error.label(), "body count mismatch");
    assert!(error
        .to_string()
        .contains("body count 17 does not match body list length 16"));
}

#[test]
fn reference_snapshot_summary_validation_rejects_body_order_drift() {
    let summary = reference_snapshot_summary().expect("reference snapshot summary should exist");
    let mut bodies = reference_bodies().to_vec();
    bodies.swap(0, 1);
    let leaked_bodies: &'static [pleiades_backend::CelestialBody] =
        Box::leak(bodies.into_boxed_slice());
    let summary = ReferenceSnapshotSummary {
        bodies: leaked_bodies,
        ..summary
    };

    let error = summary
        .validate()
        .expect_err("body-order drift should be rejected");
    assert_eq!(error.label(), "body order mismatch");
    assert!(error.to_string().contains("index 0"));
    assert!(error
        .to_string()
        .contains(&reference_bodies()[0].to_string()));
    assert!(error
        .to_string()
        .contains(&reference_bodies()[1].to_string()));
}

#[test]
fn reference_snapshot_summary_validation_rejects_row_count_drift() {
    let mut summary =
        reference_snapshot_summary().expect("reference snapshot summary should exist");
    summary.row_count += 1;

    assert_eq!(
        summary.validate(),
        Err(ReferenceSnapshotSummaryValidationError::DerivedSummaryMismatch)
    );
}

#[test]
fn reference_snapshot_summary_validation_rejects_epoch_count_drift() {
    let mut summary =
        reference_snapshot_summary().expect("reference snapshot summary should exist");
    summary.epoch_count += 1;

    assert!(matches!(
        summary.validate(),
        Err(
            ReferenceSnapshotSummaryValidationError::EpochCountMismatch {
                epoch_count: 32,
                derived_epoch_count: 31,
            }
        )
    ));
}

#[test]
fn reference_snapshot_summary_validation_rejects_asteroid_row_count_drift() {
    let mut summary =
        reference_snapshot_summary().expect("reference snapshot summary should exist");
    summary.asteroid_row_count += 1;

    assert!(matches!(
        summary.validate(),
        Err(
            ReferenceSnapshotSummaryValidationError::AsteroidRowCountMismatch {
                asteroid_row_count: 96,
                derived_asteroid_row_count: 95,
            }
        )
    ));
}

#[test]
fn reference_snapshot_equatorial_parity_summary_reports_the_expected_coverage() {
    let summary = reference_snapshot_equatorial_parity_summary()
        .expect("reference snapshot equatorial parity summary should exist");
    assert_eq!(summary.row_count, 357);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.bodies, reference_bodies());
    assert_eq!(summary.epoch_count, 31);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_268_932.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(
            summary.summary_line(),
            format!(
                "JPL reference snapshot equatorial parity: 357 rows across 16 bodies and 31 epochs (JD 2268932.5 (TDB)..JD 2634167.0 (TDB)); bodies: {}; mean-obliquity transform against the checked-in ecliptic fixture",
                format_bodies(reference_bodies())
            )
        );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        reference_snapshot_equatorial_parity_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_snapshot_equatorial_parity_summary_validation_rejects_body_count_drift() {
    let summary = ReferenceSnapshotEquatorialParitySummary {
        row_count: 2,
        body_count: 3,
        bodies: reference_bodies(),
        epoch_count: 1,
        earliest_epoch: reference_instant(),
        latest_epoch: reference_instant(),
    };

    assert!(matches!(
        summary.validate(),
        Err(
            ReferenceSnapshotEquatorialParitySummaryValidationError::Snapshot(
                ReferenceSnapshotSummaryValidationError::BodyCountMismatch {
                    body_count: 3,
                    bodies_len: 16,
                }
            )
        )
    ));
}

#[test]
fn reference_snapshot_batch_parity_summary_validation_rejects_derived_summary_drift() {
    let mut summary = reference_snapshot_batch_parity_summary()
        .expect("reference snapshot batch parity summary should exist");
    summary.snapshot.asteroid_row_count += 1;

    assert!(matches!(
        summary.validate(),
        Err(
            ReferenceSnapshotBatchParitySummaryValidationError::Snapshot(
                ReferenceSnapshotSummaryValidationError::AsteroidRowCountMismatch {
                    asteroid_row_count: 96,
                    derived_asteroid_row_count: 95,
                }
            )
        )
    ));
}

#[test]
fn reference_snapshot_batch_parity_summary_reports_the_expected_coverage() {
    let summary = reference_snapshot_batch_parity_summary()
        .expect("reference snapshot batch parity summary should exist");
    assert_eq!(summary.snapshot.row_count, 357);
    assert_eq!(summary.snapshot.body_count, 16);
    assert_eq!(summary.snapshot.bodies, reference_bodies());
    assert_eq!(summary.snapshot.epoch_count, 31);
    assert_eq!(
        summary.snapshot.earliest_epoch.julian_day.days(),
        2_268_932.5
    );
    assert_eq!(summary.snapshot.latest_epoch.julian_day.days(), 2_634_167.0);
    assert!(summary.ecliptic_request_count > 0);
    assert!(summary.equatorial_request_count > 0);
    assert_eq!(summary.exact_count, 357);
    assert_eq!(summary.interpolated_count, 0);
    assert_eq!(summary.approximate_count, 0);
    assert_eq!(summary.unknown_count, 0);
    assert_eq!(summary.validate(), Ok(()));
    assert!(summary
            .summary_line()
            .contains("JPL reference snapshot batch parity: 357 rows across 16 bodies and 31 epochs (JD 2268932.5 (TDB)..JD 2634167.0 (TDB)); bodies: "));
    assert!(summary
            .summary_line()
            .contains("quality counts: Exact=357, Interpolated=0, Approximate=0, Unknown=0; batch/single parity preserved"));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        validated_reference_snapshot_batch_parity_summary_for_report(),
        Ok(summary.summary_line())
    );
    assert_eq!(
        reference_snapshot_batch_parity_summary_for_report(),
        summary.summary_line()
    );
    assert!(jpl_snapshot_evidence_summary_for_report().contains(
            "JPL reference snapshot batch parity: 357 rows across 16 bodies and 31 epochs (JD 2268932.5 (TDB)..JD 2634167.0 (TDB)); bodies:"
        ));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_early_major_body_boundary_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_2451910_major_body_boundary_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_2451911_major_body_boundary_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_2451912_major_body_boundary_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_2451913_major_body_boundary_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_2451914_major_body_boundary_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_2451915_major_body_boundary_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_bridge_day_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_2451914_major_body_bridge_day_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_2451914_major_body_bridge_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_1750_major_body_interior_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_2451916_major_body_interior_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_2451916_major_body_dense_boundary_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_2451917_major_body_boundary_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_2451917_major_body_bridge_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_2451918_major_body_boundary_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_2451919_major_body_boundary_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_2451920_major_body_interior_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&production_generation_snapshot_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&production_generation_boundary_source_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&production_generation_boundary_window_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_source_window_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&production_generation_boundary_request_corpus_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_boundary_epoch_coverage_summary_for_report()));
    assert!(jpl_snapshot_evidence_summary_for_report()
        .contains(&reference_snapshot_sparse_boundary_summary_for_report()));
}

#[test]
fn reference_snapshot_batch_parity_summary_validation_rejects_request_count_mismatches() {
    let mut summary = reference_snapshot_batch_parity_summary()
        .expect("reference snapshot batch parity summary should exist");
    summary.equatorial_request_count += 1;

    assert!(matches!(
        summary.validate(),
        Err(ReferenceSnapshotBatchParitySummaryValidationError::RequestCountMismatch { .. })
    ));
}

#[test]
fn production_generation_snapshot_summary_reports_the_expected_coverage() {
    let summary = production_generation_snapshot_summary()
        .expect("production generation summary should exist");
    assert_eq!(summary.row_count, 357);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.epoch_count, 31);
    assert_eq!(summary.boundary_row_count, 84);
    assert_eq!(summary.boundary_body_count, 16);
    assert_eq!(summary.boundary_epoch_count, 14);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        production_generation_snapshot_summary_for_report(),
        summary.summary_line()
    );
    assert!(summary.summary_line().contains("boundary overlay (Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2451915.25, 2451915.75, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2451915.25, 2451915.75, 2451915.5, 2500000, and 2634167, plus Moon at 2451545, 2451915.25, 2451915.75, 2451915.5, 2500000, and 2634167, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus Pluto at 2451545 and 2500000, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 84 rows across 16 bodies and 14 epochs.): 84 rows across 16 bodies and 14 epochs"));
}

#[test]
fn production_generation_boundary_request_corpus_summary_reports_the_expected_coverage() {
    let summary = production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic)
        .expect("production generation boundary request corpus summary should exist");
    assert_eq!(summary.request_count, 84);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.epoch_count, 14);
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
        .contains("observerless) across 16 bodies and 14 epochs"));
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
fn comparison_snapshot_summary_reports_the_expected_coverage() {
    let summary = comparison_snapshot_summary().expect("comparison snapshot summary should exist");
    assert_eq!(summary.row_count, 232);
    assert_eq!(summary.body_count, 10);
    assert_eq!(summary.epoch_count, 28);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_268_932.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(summary.bodies.as_slice(), comparison_bodies());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Comparison snapshot coverage: 232 rows across 10 bodies and 28 epochs (JD 2268932.5 (TDB)..JD 2634167.0 (TDB)); bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Uranus, Neptune, Saturn, Pluto"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        comparison_snapshot_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn comparison_snapshot_body_class_coverage_summary_reports_the_expected_windows() {
    let summary = comparison_snapshot_body_class_coverage_summary()
        .expect("comparison snapshot body-class coverage summary should exist");

    assert_eq!(summary.row_count, 232);
    assert_eq!(summary.bodies.as_slice(), comparison_bodies());
    assert_eq!(summary.epoch_count, 28);
    assert_eq!(summary.windows.len(), summary.bodies.len());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        comparison_snapshot_body_class_coverage_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
        validated_comparison_snapshot_body_class_coverage_summary_for_report(),
        Ok(summary.summary_line())
    );
    assert!(summary
            .summary_line()
            .starts_with("Comparison snapshot body-class coverage: 232 rows across 10 bodies and 28 epochs; bodies: "));
    assert!(summary.summary_line().contains("windows: Sun:"));
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
    assert_eq!(summary.snapshot.row_count, 232);
    assert_eq!(summary.snapshot.body_count, 10);
    assert_eq!(summary.snapshot.epoch_count, 28);
    assert_eq!(
        summary.snapshot.earliest_epoch.julian_day.days(),
        2_268_932.5
    );
    assert_eq!(summary.snapshot.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(summary.snapshot.bodies.as_slice(), comparison_bodies());
    assert_eq!(summary.ecliptic_request_count, 116);
    assert_eq!(summary.equatorial_request_count, 116);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            format!(
                "JPL comparison snapshot batch parity: 232 rows across 10 bodies and 28 epochs (JD 2268932.5 (TDB)..JD 2634167.0 (TDB)); bodies: {}; frame mix: 116 ecliptic, 116 equatorial; quality counts: Exact=232, Interpolated=0, Approximate=0, Unknown=0; batch/single parity preserved",
                format_bodies(comparison_bodies())
            )
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        comparison_snapshot_batch_parity_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
        validated_comparison_snapshot_batch_parity_summary_for_report(),
        Ok(summary.summary_line())
    );
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
    assert!(matches!(
        summary.validated_summary_line(),
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
    assert_eq!(
        summary.validated_summary_line(),
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
    assert_eq!(
        summary.validated_summary_line(),
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
    assert!(matches!(
        summary.validated_summary_line(),
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
    assert!(matches!(
        summary.validated_summary_line(),
        Err(ComparisonSnapshotSummaryValidationError::BodyOrderMismatch {
            index: 0,
            expected: actual_expected,
            found: actual_found,
        }) if actual_expected == expected && actual_found == found
    ));
}

#[test]
fn reference_asteroid_evidence_summary_reports_the_expected_coverage() {
    let summary = reference_asteroid_evidence_summary()
        .expect("reference asteroid evidence summary should exist");
    summary
        .validate()
        .expect("reference asteroid evidence summary should validate");
    assert_eq!(
            summary.summary_line(),
            "Selected asteroid evidence: 6 exact J2000 samples at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis)"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        reference_asteroid_evidence_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_asteroid_source_window_summary_reports_the_expanded_coverage() {
    let summary = reference_asteroid_source_window_summary()
        .expect("reference asteroid source window summary should exist");
    assert_eq!(summary.windows.len(), summary.sample_bodies.len());
    assert_eq!(summary.sample_count, 95);
    assert_eq!(summary.epoch_count, 17);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
            summary.summary_line(),
            "Reference asteroid source windows: 95 source-backed samples across 6 bodies and 17 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); evidence class=source-backed; frame=geocentric ecliptic J2000; time scale=TDB; windows: Ceres: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Pallas: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Juno: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); Vesta: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:433-Eros: 17 samples across 17 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB); asteroid:99942-Apophis: 10 samples across 10 epochs at JD 2378498.5 (TDB)..JD 2634167.0 (TDB)"
        );
    assert_eq!(
        summary.summary_line(),
        reference_asteroid_source_window_summary_for_report()
    );
    assert_eq!(
        validated_reference_asteroid_source_window_summary_for_report(),
        Ok(summary.summary_line())
    );
}

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
fn reference_asteroid_source_window_summary_validation_rejects_custom_body_drift() {
    let mut summary = reference_asteroid_source_window_summary()
        .expect("reference asteroid source window summary should exist");
    summary.sample_bodies[4] = pleiades_backend::CelestialBody::Ceres;

    assert!(matches!(
        summary.validate(),
        Err(
            ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn reference_asteroid_source_window_summary_validation_rejects_sample_body_order_drift() {
    let mut summary = reference_asteroid_source_window_summary()
        .expect("reference asteroid source window summary should exist");
    summary.sample_bodies.swap(0, 1);

    assert!(matches!(
        summary.validate(),
        Err(
            ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn reference_asteroid_source_window_summary_validation_rejects_window_order_drift() {
    let mut summary = reference_asteroid_source_window_summary()
        .expect("reference asteroid source window summary should exist");
    summary.windows.swap(0, 1);

    assert!(matches!(
        summary.validate(),
        Err(
            ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "windows"
            }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn reference_snapshot_source_window_summary_validation_rejects_sample_body_order_drift() {
    let mut summary = reference_snapshot_source_window_summary()
        .expect("reference snapshot source window summary should exist");
    summary.sample_bodies.swap(0, 1);

    assert!(matches!(
        summary.validate(),
        Err(
            ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn reference_snapshot_source_window_summary_validated_report_matches_summary_line() {
    let summary = reference_snapshot_source_window_summary()
        .expect("reference snapshot source window summary should exist");

    assert_eq!(
        validated_reference_snapshot_source_window_summary_for_report().unwrap(),
        summary.summary_line()
    );
    assert_eq!(
        reference_snapshot_source_window_summary_for_report(),
        summary.summary_line()
    );
    assert!(
        format_validated_reference_snapshot_source_window_summary_for_report(&summary)
            .contains("Reference snapshot source windows: ")
    );
}

#[test]
fn reference_snapshot_source_window_summary_validated_report_falls_back_on_drift() {
    let mut summary = reference_snapshot_source_window_summary()
        .expect("reference snapshot source window summary should exist");
    summary.windows.swap(0, 1);

    assert!(
        format_validated_reference_snapshot_source_window_summary_for_report(&summary)
            .starts_with("Reference snapshot source windows: unavailable (")
    );
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
fn reference_asteroid_evidence_summary_validation_rejects_body_order_drift() {
    let mut summary = reference_asteroid_evidence_summary()
        .expect("reference asteroid evidence summary should exist");
    summary.sample_bodies.swap(0, 1);

    assert!(matches!(
        summary.validate(),
        Err(ReferenceAsteroidEvidenceSummaryValidationError::BodyOrderMismatch { index: 0, .. })
    ));
}

#[test]
fn reference_asteroid_evidence_validation_rejects_body_order_drift() {
    let mut evidence = reference_asteroid_evidence().to_vec();
    evidence.swap(0, 1);

    assert!(matches!(
        validate_reference_asteroid_evidence(&evidence),
        Err(ReferenceAsteroidEvidenceValidationError::BodyOrderMismatch { index: 0, .. })
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
    assert_eq!(
            source_summary.summary_line(),
            format!(
                "Comparison snapshot source: NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.; redistribution=repository-checked regression fixtures, not a broad public corpus.; columns=body, x_km, y_km, z_km; checksum=0x{:016x}",
                comparison_snapshot_source_checksum()
            )
        );
    assert_eq!(source_summary.to_string(), source_summary.summary_line());
    assert_eq!(source_summary.validate(), Ok(()));
    assert_eq!(
        source_summary.validated_summary_line(),
        Ok(source_summary.summary_line())
    );
    assert_eq!(
        format_comparison_snapshot_source_summary(&source_summary),
        source_summary.summary_line()
    );
    assert_eq!(
        comparison_snapshot_source_summary_for_report(),
        source_summary.summary_line()
    );
    assert_eq!(
        validated_comparison_snapshot_source_summary_for_report(),
        Ok(source_summary.summary_line())
    );
    let source_window_summary = comparison_snapshot_source_window_summary()
        .expect("comparison snapshot source window summary should exist");
    assert_eq!(
        source_window_summary.summary_line(),
        comparison_snapshot_source_window_summary_for_report()
    );
    assert_eq!(
        source_window_summary.to_string(),
        source_window_summary.summary_line()
    );
    assert_eq!(source_window_summary.validate(), Ok(()));
    assert_eq!(
        source_window_summary.validated_summary_line(),
        Ok(source_window_summary.summary_line())
    );
    assert_eq!(
        comparison_snapshot_source_window_summary_for_report(),
        source_window_summary.summary_line()
    );
    assert_eq!(
        format_comparison_snapshot_source_window_summary(&source_window_summary),
        source_window_summary.summary_line()
    );
    assert_eq!(
        format_validated_comparison_snapshot_source_summary_for_report(&source_summary, manifest,),
        source_summary.summary_line()
    );
    let invalid_manifest = SnapshotManifest {
        title: Some("Example snapshot.".to_string()),
        source: Some(" ".to_string()),
        coverage: Some("coverage".to_string()),
        redistribution: None,
        columns: vec!["body".to_string()],
    };
    assert_eq!(
        format_validated_comparison_snapshot_source_summary_for_report(
            &source_summary,
            &invalid_manifest,
        ),
        "Comparison snapshot source: unavailable (missing source)"
    );
    assert_eq!(
            manifest.summary_line("Comparison snapshot manifest"),
            "Comparison snapshot manifest: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.; columns=body, x_km, y_km, z_km; redistribution=repository-checked regression fixtures, not a broad public corpus."
        );
    assert_eq!(
            manifest.to_string(),
            "Snapshot manifest: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.; columns=body, x_km, y_km, z_km; redistribution=repository-checked regression fixtures, not a broad public corpus."
        );
    let comparison_summary = comparison_snapshot_manifest_summary();
    assert_eq!(
            comparison_summary.summary_line(),
            "Comparison snapshot manifest: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.; coverage=Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.; columns=body, x_km, y_km, z_km; redistribution=repository-checked regression fixtures, not a broad public corpus."
        );
    assert_eq!(
        comparison_summary.to_string(),
        comparison_summary.summary_line()
    );
    assert_eq!(
        comparison_snapshot_manifest_summary_for_report(),
        comparison_summary.summary_line()
    );
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
    assert_eq!(summary.sample_count, 232);
    assert_eq!(summary.sample_bodies.len(), 10);
    assert_eq!(summary.epoch_count, 28);
    assert_eq!(summary.windows.len(), 10);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert!(summary.summary_line().contains("Comparison snapshot source windows: 232 source-backed samples across 10 bodies and 28 epochs"));
    assert!(summary
        .summary_line()
        .contains("Mars: 23 samples across 23 epochs at JD 2305457.5 (TDB)..JD 2634167.0 (TDB)"));
    assert!(summary
        .summary_line()
        .contains("Pluto: 18 samples across 18 epochs at JD 2378499.0 (TDB)..JD 2500000.0 (TDB)"));
    assert_eq!(
        comparison_snapshot_source_window_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
        validated_comparison_snapshot_source_window_summary_for_report(),
        Ok(summary.summary_line())
    );
    assert_eq!(
        format_comparison_snapshot_source_window_summary(&summary),
        summary.summary_line()
    );
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
    assert_eq!(
        summary.validated_summary_line(),
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
    assert_eq!(
        summary.validated_summary_line(),
        Err(
            ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "sample_bodies"
            }
        )
    );
}

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
    assert_eq!(
            manifest.summary_line("Example manifest"),
            "Example manifest: Example snapshot.; source=Example source; coverage=unknown; columns=body"
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
    assert_eq!(
            manifest.summary_line("Example manifest"),
            "Example manifest: Example snapshot.; source=Example source; coverage=unknown; columns=body"
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
    assert_eq!(
            manifest.summary_line("Example manifest"),
            "Example manifest: Example snapshot.; source=Example source; coverage=unknown; columns=body, , x_km"
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
    assert_eq!(
            manifest.summary_line("Example manifest"),
            "Example manifest: Example snapshot.; source=Example source; coverage=unknown; columns=body, x_km, body"
        );
}

#[test]
fn manifest_summary_validated_summary_line_returns_the_rendered_line() {
    let summary = SnapshotManifestSummary {
        label: "Example manifest",
        manifest: SnapshotManifest {
            title: Some("Example snapshot.".to_string()),
            source: Some("Example source".to_string()),
            coverage: Some("Example coverage".to_string()),
            redistribution: None,
            columns: vec!["body".to_string(), "x_km".to_string()],
        },
        source_fallback: "unknown",
        coverage_fallback: "unknown",
    };

    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(summary.to_string(), summary.summary_line());
}

#[test]
fn manifest_summary_validated_summary_line_rejects_columns_drift() {
    let summary = SnapshotManifestSummary {
        label: "Reference snapshot manifest",
        manifest: SnapshotManifest {
            title: Some("Reference snapshot.".to_string()),
            source: Some("NASA/JPL Horizons API".to_string()),
            coverage: Some("Example coverage".to_string()),
            redistribution: None,
            columns: vec![
                "body".to_string(),
                "x_km".to_string(),
                "y_km".to_string(),
                "z_km".to_string(),
            ],
        },
        source_fallback: "unknown",
        coverage_fallback: "unknown",
    };

    assert_eq!(
        summary.validated_summary_line_with_expected_columns(&[
            "epoch_jd", "body", "x_km", "y_km", "z_km",
        ]),
        Err(SnapshotManifestSummaryValidationError::ColumnsMismatch {
            expected: "epoch_jd, body, x_km, y_km, z_km".to_string(),
            found: "body, x_km, y_km, z_km".to_string(),
        })
    );
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
fn manifest_summary_for_report_reports_validation_errors() {
    let manifest = SnapshotManifest {
        title: Some("Example snapshot.".to_string()),
        source: Some("Example source".to_string()),
        coverage: Some("".to_string()),
        redistribution: None,
        columns: vec!["body".to_string()],
    };

    assert_eq!(
        format_manifest_summary_for_report("Example manifest", &manifest),
        "Example manifest: unavailable (blank coverage)"
    );
}

#[test]
fn validated_source_summary_for_report_reports_validation_errors() {
    let manifest = SnapshotManifest {
        title: Some("Example snapshot.".to_string()),
        source: None,
        coverage: Some("Example coverage".to_string()),
        redistribution: None,
        columns: vec!["body".to_string()],
    };

    assert_eq!(
        format_validated_source_summary_for_report("Example snapshot source", &manifest, || {
            "should not render".to_string()
        },),
        "Example snapshot source: unavailable (missing source)"
    );
}

#[test]
fn reference_asteroid_equatorial_evidence_summary_reports_the_expected_coverage() {
    let summary = reference_asteroid_equatorial_evidence_summary()
        .expect("reference asteroid equatorial evidence summary should exist");
    summary
        .validate()
        .expect("reference asteroid equatorial evidence summary should validate");
    assert_eq!(
            summary.summary_line(),
            "Selected asteroid equatorial evidence: 6 exact J2000 samples at JD 2451545.0 (TDB) (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) using a mean-obliquity equatorial transform"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        reference_asteroid_equatorial_evidence_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn reference_asteroid_equatorial_evidence_summary_validation_rejects_transform_drift() {
    let mut summary = reference_asteroid_equatorial_evidence_summary()
        .expect("reference asteroid equatorial evidence summary should exist");
    summary.transform_note = "broken transform";

    assert!(matches!(
        summary.validate(),
        Err(
            ReferenceAsteroidEquatorialEvidenceSummaryValidationError::TransformNoteMismatch {
                expected: "mean-obliquity equatorial transform",
                found: "broken transform",
            }
        )
    ));
}

#[test]
fn reference_asteroid_equatorial_evidence_validation_rejects_transform_drift() {
    let mut evidence = reference_asteroid_equatorial_evidence().to_vec();
    let shifted_right_ascension = pleiades_types::Angle::from_degrees(
        evidence[0].equatorial.right_ascension.degrees() + 0.01,
    );
    evidence[0].equatorial = pleiades_types::EquatorialCoordinates::new(
        shifted_right_ascension,
        evidence[0].equatorial.declination,
        evidence[0].equatorial.distance_au,
    );

    assert!(matches!(
        validate_reference_asteroid_equatorial_evidence(&evidence),
        Err(
            ReferenceAsteroidEquatorialEvidenceValidationError::RightAscensionMismatch {
                index: 0,
                ..
            }
        )
    ));
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
fn reference_snapshot_source_summary_reports_the_expected_provenance() {
    let summary = reference_snapshot_source_summary();

    assert_eq!(
        summary.source,
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables."
    );
    assert_eq!(
            summary.coverage,
            "selected bodies sampled at 1500-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 1600-01-11 for Sun, Moon, Mercury, Venus, Mars, Jupiter, Uranus, Neptune; major bodies sampled at 1749-12-31 for Sun through Neptune; selected bodies sampled at 1750-01-01 for Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; inner planets sampled across 1800-2500; major bodies sampled at 1800-01-03 for Sun through Pluto; selected bodies sampled at 1900-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2200-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2451915.25 and 2451915.75 for Sun, Moon, Mercury, Venus; major bodies sampled at 2400000, 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, 2453000.5, and 2500000; major bodies sampled at 2451915.5 for Sun through Pluto; Mars sampled at 2600000 and 2634167 for outer boundary coverage; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2378498.5, 2451910.5 through 2451919.5, with 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 boundary coverage, 1800-01-03, 2003-12-27, 2132-08-31, 2500-01-01, and 2634167; asteroid:99942-Apophis is now also sampled at 2378498.5 and 2451917.5 to complete the selected-asteroid bridge."
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
    assert_eq!(body_class_summary.major_body_row_count, 262);
    assert_eq!(body_class_summary.major_bodies.len(), 10);
    assert_eq!(body_class_summary.major_epoch_count, 31);
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
            .contains("Reference snapshot body-class coverage: major bodies: 262 rows across 10 bodies and 31 epochs; major windows: "));
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

    let sun_window = &window_summary.windows[0];
    assert_eq!(sun_window.body, pleiades_backend::CelestialBody::Sun);
    assert_eq!(sun_window.sample_count, 30);
    assert_eq!(sun_window.epoch_count, 30);
    assert_eq!(sun_window.earliest_epoch.julian_day.days(), 2_268_932.5);
    assert_eq!(sun_window.latest_epoch.julian_day.days(), 2_634_167.0);

    let jupiter_window = &window_summary.windows[5];
    assert_eq!(
        jupiter_window.body,
        pleiades_backend::CelestialBody::Jupiter
    );
    assert_eq!(jupiter_window.sample_count, 24);
    assert_eq!(jupiter_window.epoch_count, 24);
    assert_eq!(jupiter_window.earliest_epoch.julian_day.days(), 2_305_457.5);
    assert_eq!(jupiter_window.latest_epoch.julian_day.days(), 2_500_000.0);

    let pluto_window = &window_summary.windows[9];
    assert_eq!(pluto_window.body, pleiades_backend::CelestialBody::Pluto);
    assert_eq!(pluto_window.sample_count, 21);
    assert_eq!(pluto_window.epoch_count, 21);
    assert_eq!(pluto_window.earliest_epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(pluto_window.latest_epoch.julian_day.days(), 2_500_000.0);

    let asteroid_window = &window_summary.windows[14];
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
fn reference_snapshot_body_class_coverage_summary_reports_the_expected_body_classes() {
    let summary = reference_snapshot_body_class_coverage_summary()
        .expect("reference snapshot body-class coverage summary should exist");

    assert_eq!(summary.major_body_row_count, 262);
    assert_eq!(summary.major_bodies.len(), 10);
    assert_eq!(
        summary.major_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.major_bodies[9],
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.major_epoch_count, 31);
    assert_eq!(summary.asteroid_row_count, 95);
    assert_eq!(summary.asteroid_bodies.len(), 6);
    assert_eq!(
        summary.asteroid_bodies[0],
        pleiades_backend::CelestialBody::Ceres
    );
    assert_eq!(
        summary.asteroid_bodies[4],
        pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
    );
    assert_eq!(
        summary.asteroid_bodies[5],
        pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "99942-Apophis"))
    );
    assert_eq!(summary.asteroid_epoch_count, 17);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        reference_snapshot_body_class_coverage_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(summary.to_string(), summary.summary_line());
}

#[test]
fn reference_snapshot_source_window_summary_reports_the_current_boundary_windows() {
    let summary = reference_snapshot_source_window_summary()
        .expect("reference snapshot source window summary should exist");

    assert_eq!(summary.sample_count, 357);
    assert_eq!(summary.sample_bodies.len(), 16);
    assert_eq!(summary.epoch_count, 31);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        reference_snapshot_source_window_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
        summary.windows[0].body,
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(summary.windows[0].sample_count, 30);
    assert_eq!(summary.windows[0].epoch_count, 30);
    assert_eq!(
        summary.windows[0].earliest_epoch.julian_day.days(),
        2_268_932.5
    );
    assert_eq!(
        summary.windows[0].latest_epoch.julian_day.days(),
        2_634_167.0
    );
    assert_eq!(
        summary.windows[5].body,
        pleiades_backend::CelestialBody::Jupiter
    );
    assert_eq!(summary.windows[5].sample_count, 24);
    assert_eq!(summary.windows[5].epoch_count, 24);
    assert_eq!(
        summary.windows[5].earliest_epoch.julian_day.days(),
        2_305_457.5
    );
    assert_eq!(
        summary.windows[5].latest_epoch.julian_day.days(),
        2_500_000.0
    );
    assert_eq!(
        summary.windows[9].body,
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.windows[9].sample_count, 21);
    assert_eq!(summary.windows[9].epoch_count, 21);
    assert_eq!(
        summary.windows[9].earliest_epoch.julian_day.days(),
        2_378_498.5
    );
    assert_eq!(
        summary.windows[9].latest_epoch.julian_day.days(),
        2_500_000.0
    );
    assert_eq!(
        summary.windows[15].body,
        pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "99942-Apophis"))
    );
    assert_eq!(summary.windows[15].sample_count, 10);
    assert_eq!(summary.windows[15].epoch_count, 10);
    assert_eq!(
        summary.windows[15].earliest_epoch.julian_day.days(),
        2_378_498.5
    );
    assert_eq!(
        summary.windows[15].latest_epoch.julian_day.days(),
        2_634_167.0
    );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        summary.summary_line(),
        reference_snapshot_source_window_summary_for_report()
    );
}

#[test]
fn reference_snapshot_body_class_coverage_summary_validation_rejects_row_count_drift() {
    let mut summary = reference_snapshot_body_class_coverage_summary()
        .expect("reference snapshot body-class coverage summary should exist");
    summary.major_body_row_count += 1;

    assert_eq!(
        summary.validate(),
        Err(
            ReferenceSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                field: "major_body_row_count",
            }
        )
    );
}

#[test]
fn reference_holdout_overlap_summary_reports_the_current_overlap() {
    let summary = reference_holdout_overlap_summary()
        .expect("reference/hold-out overlap summary should exist");

    assert_eq!(summary.shared_sample_count, 84);
    assert_eq!(summary.shared_epoch_count, 14);
    assert_eq!(summary.shared_bodies.len(), 16);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        reference_holdout_overlap_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
        validated_reference_holdout_overlap_summary_for_report(),
        Ok(summary.summary_line())
    );
    assert_eq!(
            summary.summary_line(),
            format!(
                "Reference/hold-out overlap: 84 shared body-epoch pairs across 16 bodies and 14 epochs; bodies: {}",
                format_bodies(&summary.shared_bodies)
            )
        );
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

    let reference = snapshot_keys(include_str!("../data/reference_snapshot.csv"));
    let holdout = snapshot_keys(include_str!("../data/independent_holdout_snapshot.csv"));

    assert_eq!(reference.row_count, 357);
    assert_eq!(reference.row_count, reference.pairs.len());
    assert_eq!(reference.bodies.len(), 16);
    assert_eq!(reference.epochs.len(), 31);
    assert_eq!(holdout.row_count, 84);
    assert_eq!(holdout.row_count, holdout.pairs.len());
    assert_eq!(holdout.bodies.len(), 16);
    assert_eq!(holdout.epochs.len(), 14);

    assert_eq!(
        reference_holdout_overlap_summary().map(|summary| summary.shared_sample_count),
        Some(84)
    );
    assert_eq!(
        reference.pairs.intersection(&holdout.pairs).count(),
        84,
        "reference and hold-out corpora should retain the documented 84 shared body-epoch pairs"
    );
    assert_eq!(reference.bodies.intersection(&holdout.bodies).count(), 16);
    assert_eq!(reference.epochs.intersection(&holdout.epochs).count(), 14);
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
        report.contains("source windows=357 source-backed samples across 16 bodies and 31 epochs")
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

#[test]
fn reference_snapshot_source_window_summary_validation_rejects_window_order_drift() {
    let mut summary = reference_snapshot_source_window_summary()
        .expect("reference snapshot source window summary should exist");
    summary.windows.swap(0, 1);

    assert!(matches!(
        summary.validate(),
        Err(
            ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "windows",
            }
        )
    ));
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn reference_snapshot_source_summary_validation_reports_blank_fields() {
    let blank_source = ReferenceSnapshotSourceSummary {
        source: " ".to_string(),
        evidence_class: REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
        coverage: "coverage".to_string(),
        columns: REFERENCE_SNAPSHOT_COLUMNS.to_string(),
        redistribution: REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK.to_string(),
        checksum: reference_snapshot_source_checksum(),
        frame_treatment: "geocentric ecliptic J2000".to_string(),
        time_scale: REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
        reference_epoch: reference_instant(),
    };
    assert_eq!(
        blank_source.validate(),
        Err(ReferenceSnapshotSourceSummaryValidationError::BlankSource)
    );

    let blank_coverage = ReferenceSnapshotSourceSummary {
        source: REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
        evidence_class: REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
        coverage: "\n".to_string(),
        columns: REFERENCE_SNAPSHOT_COLUMNS.to_string(),
        redistribution: REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK.to_string(),
        checksum: reference_snapshot_source_checksum(),
        frame_treatment: REFERENCE_SNAPSHOT_FRAME_TREATMENT.to_string(),
        time_scale: REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
        reference_epoch: reference_instant(),
    };
    assert_eq!(
        blank_coverage.validate(),
        Err(ReferenceSnapshotSourceSummaryValidationError::BlankCoverage)
    );

    let padded_coverage = ReferenceSnapshotSourceSummary {
        source: REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
        evidence_class: REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
        coverage: " coverage ".to_string(),
        columns: REFERENCE_SNAPSHOT_COLUMNS.to_string(),
        redistribution: REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK.to_string(),
        checksum: reference_snapshot_source_checksum(),
        frame_treatment: REFERENCE_SNAPSHOT_FRAME_TREATMENT.to_string(),
        time_scale: REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
        reference_epoch: reference_instant(),
    };
    assert_eq!(
        padded_coverage.validate(),
        Err(
            ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                field: "coverage",
            }
        )
    );

    let multiline_source = ReferenceSnapshotSourceSummary {
        source: "source\nline".to_string(),
        evidence_class: REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
        coverage: REFERENCE_SNAPSHOT_COVERAGE_FALLBACK.to_string(),
        columns: REFERENCE_SNAPSHOT_COLUMNS.to_string(),
        redistribution: REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK.to_string(),
        checksum: reference_snapshot_source_checksum(),
        frame_treatment: REFERENCE_SNAPSHOT_FRAME_TREATMENT.to_string(),
        time_scale: REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
        reference_epoch: reference_instant(),
    };
    assert_eq!(
        multiline_source.validate(),
        Err(
            ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                field: "source",
            }
        )
    );

    let blank_columns = ReferenceSnapshotSourceSummary {
        source: REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
        evidence_class: REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
        coverage: REFERENCE_SNAPSHOT_COVERAGE_FALLBACK.to_string(),
        columns: "\t".to_string(),
        redistribution: REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK.to_string(),
        checksum: reference_snapshot_source_checksum(),
        frame_treatment: REFERENCE_SNAPSHOT_FRAME_TREATMENT.to_string(),
        time_scale: REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
        reference_epoch: reference_instant(),
    };
    assert_eq!(
        blank_columns.validate(),
        Err(ReferenceSnapshotSourceSummaryValidationError::BlankColumns)
    );

    let blank_redistribution = ReferenceSnapshotSourceSummary {
        source: REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
        evidence_class: REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
        coverage: REFERENCE_SNAPSHOT_COVERAGE_FALLBACK.to_string(),
        columns: REFERENCE_SNAPSHOT_COLUMNS.to_string(),
        redistribution: "\n".to_string(),
        checksum: reference_snapshot_source_checksum(),
        frame_treatment: REFERENCE_SNAPSHOT_FRAME_TREATMENT.to_string(),
        time_scale: REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
        reference_epoch: reference_instant(),
    };
    assert_eq!(
        blank_redistribution.validate(),
        Err(ReferenceSnapshotSourceSummaryValidationError::BlankRedistribution)
    );

    let blank_frame_treatment = ReferenceSnapshotSourceSummary {
        source: REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
        evidence_class: REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
        coverage: REFERENCE_SNAPSHOT_COVERAGE_FALLBACK.to_string(),
        columns: REFERENCE_SNAPSHOT_COLUMNS.to_string(),
        redistribution: REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK.to_string(),
        checksum: reference_snapshot_source_checksum(),
        frame_treatment: "\n".to_string(),
        time_scale: REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
        reference_epoch: reference_instant(),
    };
    assert_eq!(
        blank_frame_treatment.validate(),
        Err(ReferenceSnapshotSourceSummaryValidationError::BlankFrameTreatment)
    );

    let padded_frame_treatment = ReferenceSnapshotSourceSummary {
        source: REFERENCE_SNAPSHOT_SOURCE_EXPECTED.to_string(),
        evidence_class: REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
        coverage: REFERENCE_SNAPSHOT_COVERAGE_FALLBACK.to_string(),
        columns: REFERENCE_SNAPSHOT_COLUMNS.to_string(),
        redistribution: REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK.to_string(),
        checksum: reference_snapshot_source_checksum(),
        frame_treatment: " geocentric ecliptic J2000 ".to_string(),
        time_scale: REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
        reference_epoch: reference_instant(),
    };
    assert_eq!(
        padded_frame_treatment.validate(),
        Err(
            ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                field: "frame_treatment",
            }
        )
    );
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
            "Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2451915.25, 2451915.75, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2451915.25, 2451915.75, 2451915.5, 2500000, and 2634167, plus Moon at 2451545, 2451915.25, 2451915.75, 2451915.5, 2500000, and 2634167, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus Pluto at 2451545 and 2500000, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 84 rows across 16 bodies and 14 epochs."
        );
    assert_eq!(summary.evidence_class, INDEPENDENT_HOLDOUT_EVIDENCE_CLASS);
    assert_eq!(summary.columns, "epoch_jd, body, x_km, y_km, z_km");
    assert_eq!(summary.frame_treatment, INDEPENDENT_HOLDOUT_FRAME_TREATMENT);
    assert_eq!(summary.time_scale, INDEPENDENT_HOLDOUT_TIME_SCALE);
    assert!(summary.summary_line().contains("time scale=TDB"));
    assert_eq!(
        summary.redistribution,
        "repository-checked regression fixtures, not a broad public corpus."
    );
    assert!(summary.summary_line().contains("evidence class=hold-out"));
    assert!(summary.summary_line().contains(
        "redistribution=repository-checked regression fixtures, not a broad public corpus."
    ));
    assert!(summary
        .summary_line()
        .contains(&format!("checksum=0x{:016x}", summary.checksum)));
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        independent_holdout_source_summary_for_report(),
        summary.summary_line()
    );
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
    assert_eq!(summary.row_count, 84);
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
            "Moon",
            "Pluto",
            "Ceres",
            "Pallas",
            "Juno",
            "Vesta",
            "asteroid:433-Eros",
            "asteroid:99942-Apophis",
        ]
    );
    assert_eq!(summary.epoch_count, 14);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(
            summary.summary_line(),
            "Independent hold-out coverage: 84 rows across 16 bodies and 14 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies: Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Moon, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis"
        );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        independent_holdout_snapshot_summary_for_report(),
        summary.summary_line()
    );
}

#[test]
fn independent_holdout_snapshot_source_window_summary_reports_the_expected_windows() {
    let summary = independent_holdout_snapshot_source_window_summary()
        .expect("independent hold-out source window summary should exist");
    assert_eq!(summary.sample_count, 84);
    assert_eq!(summary.sample_bodies.len(), 16);
    assert_eq!(summary.sample_bodies, independent_holdout_bodies().to_vec());
    assert_eq!(summary.epoch_count, 14);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(summary.windows.len(), 16);
    assert_eq!(
        summary.windows[0].body,
        pleiades_backend::CelestialBody::Mars
    );
    assert_eq!(summary.windows[0].sample_count, 8);
    assert_eq!(summary.windows[0].epoch_count, 8);
    assert_eq!(
        summary.windows[0].earliest_epoch.julian_day.days(),
        2_451_545.0
    );
    assert_eq!(
        summary.windows[0].latest_epoch.julian_day.days(),
        2_634_167.0
    );
    assert_eq!(
        summary.windows[9].body,
        pleiades_backend::CelestialBody::Pluto
    );
    assert_eq!(summary.windows[9].sample_count, 3);
    assert_eq!(summary.windows[9].epoch_count, 3);
    assert_eq!(
        summary.windows[9].earliest_epoch.julian_day.days(),
        2_451_545.0
    );
    assert_eq!(
        summary.windows[9].latest_epoch.julian_day.days(),
        2_500_000.0
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        independent_holdout_snapshot_source_window_summary_for_report(),
        summary.summary_line()
    );
    assert!(summary.summary_line().contains(
            "Independent hold-out source windows: 84 source-backed samples across 16 bodies and 14 epochs"
        ));
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
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        independent_holdout_snapshot_quarter_day_boundary_summary_for_report(),
        summary.summary_line()
    );
    assert!(summary.summary_line().contains(
        "Independent hold-out quarter-day boundary samples: 8 rows across 4 bodies and 2 epochs"
    ));
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
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        independent_holdout_high_curvature_summary_for_report(),
        summary.summary_line()
    );
    assert!(summary.summary_line().contains(
            "JPL independent hold-out high-curvature evidence: 8 exact samples across 4 bodies and 2 epochs"
        ));
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
    assert!(summary.validated_summary_line().is_err());
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
    assert!(summary.validated_summary_line().is_err());
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
    assert_eq!(summary.row_count, 84);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.epoch_count, 14);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(
            summary.summary_line(),
            "JPL independent hold-out equatorial parity: 84 rows across 16 bodies and 14 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); mean-obliquity transform against the checked-in ecliptic fixture"
        );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        independent_holdout_snapshot_equatorial_parity_summary_for_report(),
        summary.summary_line()
    );
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
    assert_eq!(summary.sample_count, 84);
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
            "Moon",
            "Pluto",
            "Ceres",
            "Pallas",
            "Juno",
            "Vesta",
            "asteroid:433-Eros",
            "asteroid:99942-Apophis",
        ]
    );
    assert_eq!(summary.epoch_count, 14);
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

    assert_eq!(summary.to_string(), summary.summary_line());

    let rendered = format_jpl_independent_holdout_summary(&summary);
    assert!(rendered.contains("JPL independent hold-out:"));
    assert!(rendered.contains(
            "84 exact rows across 16 bodies (Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Moon, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) and 14 epochs"
        ));
    assert!(rendered.contains("p95 Δlon="));
    assert!(rendered.contains("p95 Δlat="));
    assert!(rendered.contains("p95 Δdist="));
    assert!(rendered.contains("transparency evidence only, not a production tolerance envelope"));
    assert!(
        rendered.contains("independent JPL Horizons rows held out from the main snapshot corpus")
    );
    assert!(rendered.contains(&format!(
        "({} @ {}",
        summary.max_longitude_error_body,
        format_instant(summary.max_longitude_error_epoch)
    )));
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
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.snapshot.row_count, 84);
    assert_eq!(summary.snapshot.body_count, 16);
    assert_eq!(summary.tt_request_count, 42);
    assert_eq!(summary.tdb_request_count, 42);
    assert!(summary.parity_preserved);
    assert_eq!(
        summary.exact_count
            + summary.interpolated_count
            + summary.approximate_count
            + summary.unknown_count,
        summary.snapshot.row_count,
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        validated_independent_holdout_snapshot_batch_parity_summary_for_report(),
        Ok(summary.summary_line())
    );

    let rendered = format_independent_holdout_snapshot_batch_parity_summary(&summary);
    assert!(rendered.contains("JPL independent hold-out batch parity:"));
    assert!(rendered.contains(
            "84 requests across 16 bodies (Mars, Jupiter, Mercury, Venus, Saturn, Uranus, Neptune, Sun, Moon, Pluto, Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) and 14 epochs"
        ));
    assert!(rendered.contains("TT requests=42, TDB requests=42"));
    assert!(rendered.contains("quality counts:"));
    assert!(rendered.contains("order=preserved, single-query parity=preserved"));
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
fn reference_snapshot_manifest_parses_the_documented_header_comments() {
    let manifest = reference_snapshot_manifest();
    assert_eq!(
        manifest.title.as_deref(),
        Some("JPL Horizons reference snapshot.")
    );
    assert_eq!(
        manifest.source.as_deref(),
        Some("NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.")
    );
    assert_eq!(manifest.coverage.as_deref(), Some("selected bodies sampled at 1500-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 1600-01-11 for Sun, Moon, Mercury, Venus, Mars, Jupiter, Uranus, Neptune; major bodies sampled at 1749-12-31 for Sun through Neptune; selected bodies sampled at 1750-01-01 for Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; inner planets sampled across 1800-2500; major bodies sampled at 1800-01-03 for Sun through Pluto; selected bodies sampled at 1900-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2200-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2451915.25 and 2451915.75 for Sun, Moon, Mercury, Venus; major bodies sampled at 2400000, 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, 2453000.5, and 2500000; major bodies sampled at 2451915.5 for Sun through Pluto; Mars sampled at 2600000 and 2634167 for outer boundary coverage; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2378498.5, 2451910.5 through 2451919.5, with 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 boundary coverage, 1800-01-03, 2003-12-27, 2132-08-31, 2500-01-01, and 2634167; asteroid:99942-Apophis is now also sampled at 2378498.5 and 2451917.5 to complete the selected-asteroid bridge."));
    assert_eq!(
        manifest.redistribution.as_deref(),
        Some("repository-checked regression fixtures, not a broad public corpus.")
    );
    assert_eq!(
        manifest.columns,
        ["epoch_jd", "body", "x_km", "y_km", "z_km"]
    );
    assert_eq!(manifest.validate(), Ok(()));
    assert_eq!(
            manifest.summary_line("Reference snapshot manifest"),
            "Reference snapshot manifest: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=selected bodies sampled at 1500-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 1600-01-11 for Sun, Moon, Mercury, Venus, Mars, Jupiter, Uranus, Neptune; major bodies sampled at 1749-12-31 for Sun through Neptune; selected bodies sampled at 1750-01-01 for Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; inner planets sampled across 1800-2500; major bodies sampled at 1800-01-03 for Sun through Pluto; selected bodies sampled at 1900-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2200-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2451915.25 and 2451915.75 for Sun, Moon, Mercury, Venus; major bodies sampled at 2400000, 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, 2453000.5, and 2500000; major bodies sampled at 2451915.5 for Sun through Pluto; Mars sampled at 2600000 and 2634167 for outer boundary coverage; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2378498.5, 2451910.5 through 2451919.5, with 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 boundary coverage, 1800-01-03, 2003-12-27, 2132-08-31, 2500-01-01, and 2634167; asteroid:99942-Apophis is now also sampled at 2378498.5 and 2451917.5 to complete the selected-asteroid bridge.; columns=epoch_jd, body, x_km, y_km, z_km; redistribution=repository-checked regression fixtures, not a broad public corpus."
        );
}

#[test]
fn reference_snapshot_manifest_summary_rejects_metadata_drift() {
    let summary = reference_snapshot_manifest_summary();
    let error = summary
            .validate_with_expected_metadata(
                "wrong title",
                "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
                "selected bodies sampled at 1500-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 1600-01-11 for Sun, Moon, Mercury, Venus, Mars, Jupiter, Uranus, Neptune; major bodies sampled at 1749-12-31 for Sun through Neptune; selected bodies sampled at 1750-01-01 for Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; inner planets sampled across 1800-2500; major bodies sampled at 1800-01-03 for Sun through Pluto; selected bodies sampled at 1900-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2200-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2451915.25 and 2451915.75 for Sun, Moon, Mercury, Venus; major bodies sampled at 2400000, 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, 2453000.5, and 2500000; major bodies sampled at 2451915.5 for Sun through Pluto; Mars sampled at 2600000 and 2634167 for outer boundary coverage; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2378498.5, 2451910.5 through 2451919.5, with 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 boundary coverage, 1800-01-03, 2003-12-27, 2132-08-31, 2500-01-01, and 2634167; asteroid:99942-Apophis is now also sampled at 2378498.5 and 2451917.5 to complete the selected-asteroid bridge.",
                &["epoch_jd", "body", "x_km", "y_km", "z_km"],
            )
            .expect_err("reference snapshot manifest summary should reject title drift");

    assert!(matches!(
        error,
        SnapshotManifestSummaryValidationError::MetadataMismatch { field: "title", .. }
    ));
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
            Some("Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2451915.25, 2451915.75, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2451915.25, 2451915.75, 2451915.5, 2500000, and 2634167, plus Moon at 2451545, 2451915.25, 2451915.75, 2451915.5, 2500000, and 2634167, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus Pluto at 2451545 and 2500000, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 84 rows across 16 bodies and 14 epochs."),
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
    assert_eq!(
            manifest.summary_line("Independent hold-out manifest"),
            "Independent hold-out manifest: Independent JPL Horizons hold-out snapshot used only for interpolation validation.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Jupiter at 2400000, 2451545, and 2500000, plus Mercury and Venus at 2451545, 2451915.25, 2451915.75, 2500000, and 2634167, plus Saturn at 2400000, 2451545, and 2500000, plus Uranus and Neptune at 2451545 and 2500000, plus Mars at 2451545, 2500000, 2600000, and 2634167, plus Sun at 2451545, 2451915.25, 2451915.75, 2451915.5, 2500000, and 2634167, plus Moon at 2451545, 2451915.25, 2451915.75, 2451915.5, 2500000, and 2634167, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus Pluto at 2451545 and 2500000, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 84 rows across 16 bodies and 14 epochs.; columns=epoch_jd, body, x_km, y_km, z_km; redistribution=repository-checked regression fixtures, not a broad public corpus."
        );
}

#[test]
fn snapshot_manifest_footprint_validation_matches_the_current_reference_and_holdout_corpora() {
    assert_eq!(
        validate_snapshot_manifest_footprint("reference snapshot", snapshot_entries(), 357, 16, 31,),
        Ok(())
    );
    assert_eq!(
        validate_snapshot_manifest_footprint(
            "independent hold-out snapshot",
            independent_holdout_snapshot_entries(),
            84,
            16,
            14,
        ),
        Ok(())
    );
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
        },
        SnapshotEntry {
            body: CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(2.0), TimeScale::Tdb),
            x_km: 4.0,
            y_km: 5.0,
            z_km: 6.0,
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

    assert_eq!(
            manifest.summary_line_with_defaults(
                "Example manifest",
                "example source",
                "example coverage",
            ),
            "Example manifest: Example manifest.; source=example source; coverage=example coverage; columns=none"
        );
    assert_eq!(manifest.source_or("fallback source"), "fallback source");
    assert_eq!(
        manifest.coverage_or("fallback coverage"),
        "fallback coverage"
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
    assert_eq!(
        summary.summary_line(),
        validated_comparison_snapshot_manifest_summary_for_report()
            .expect("comparison snapshot manifest summary should validate")
    );
    assert_eq!(summary.to_string(), summary.summary_line());
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

#[test]
fn reference_snapshot_summary_validation_rejects_duplicate_bodies() {
    let summary = ReferenceSnapshotSummary {
        row_count: 2,
        body_count: 2,
        bodies: &[
            pleiades_backend::CelestialBody::Sun,
            pleiades_backend::CelestialBody::Sun,
        ],
        epoch_count: 1,
        asteroid_row_count: 0,
        earliest_epoch: reference_instant(),
        latest_epoch: reference_instant(),
    };

    assert!(matches!(
        summary.validate(),
        Err(ReferenceSnapshotSummaryValidationError::DuplicateBody {
            first_index: 0,
            second_index: 1,
            body,
        }) if body == "Sun"
    ));
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
    };
    let b = SnapshotEntry {
        body: pleiades_backend::CelestialBody::Moon,
        epoch: Instant::new(JulianDay::from_days(1.0), TimeScale::Tdb),
        x_km: 1.0,
        y_km: 6.0,
        z_km: 5.0,
    };
    let c = SnapshotEntry {
        body: pleiades_backend::CelestialBody::Moon,
        epoch: Instant::new(JulianDay::from_days(2.0), TimeScale::Tdb),
        x_km: 4.0,
        y_km: 15.0,
        z_km: 10.0,
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
    };
    let b = SnapshotEntry {
        body: pleiades_backend::CelestialBody::Moon,
        epoch: Instant::new(JulianDay::from_days(1.0), TimeScale::Tdb),
        x_km: 1.0,
        y_km: 2.0,
        z_km: 3.0,
    };
    let c = SnapshotEntry {
        body: pleiades_backend::CelestialBody::Moon,
        epoch: Instant::new(JulianDay::from_days(2.0), TimeScale::Tdb),
        x_km: 8.0,
        y_km: 9.0,
        z_km: 10.0,
    };
    let d = SnapshotEntry {
        body: pleiades_backend::CelestialBody::Moon,
        epoch: Instant::new(JulianDay::from_days(3.0), TimeScale::Tdb),
        x_km: 27.0,
        y_km: 28.0,
        z_km: 29.0,
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
        },
        SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(1.0), TimeScale::Tdb),
            x_km: 1.0,
            y_km: 2.0,
            z_km: 3.0,
        },
        SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(2.0), TimeScale::Tdb),
            x_km: 8.0,
            y_km: 9.0,
            z_km: 10.0,
        },
        SnapshotEntry {
            body: pleiades_backend::CelestialBody::Moon,
            epoch: Instant::new(JulianDay::from_days(3.0), TimeScale::Tdb),
            x_km: 27.0,
            y_km: 28.0,
            z_km: 29.0,
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
fn reference_snapshot_mixed_time_scale_batch_parity_requests_preserve_the_ecliptic_slice() {
    let requests = reference_snapshot_mixed_time_scale_batch_parity_requests()
        .expect("reference snapshot mixed TT/TDB batch parity requests should exist");
    let entries = reference_snapshot();

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
    }
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
fn reference_snapshot_mixed_time_scale_batch_parity_summary_reports_the_mixed_request_slice() {
    let summary = reference_snapshot_mixed_time_scale_batch_parity_summary()
        .expect("reference snapshot mixed TT/TDB batch parity summary should exist");

    assert_eq!(summary.request_count, summary.snapshot.row_count);
    assert_eq!(summary.body_count, summary.snapshot.body_count);
    assert!(summary.tt_request_count > 0);
    assert!(summary.tdb_request_count > 0);
    assert_eq!(
        summary.tt_request_count + summary.tdb_request_count,
        summary.request_count
    );
    assert_eq!(
        summary.exact_count
            + summary.interpolated_count
            + summary.approximate_count
            + summary.unknown_count,
        summary.request_count
    );
    assert!(summary.order_preserved);
    assert!(summary.single_query_parity_preserved);
    assert!(summary.validate().is_ok());
    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        validated_reference_snapshot_mixed_time_scale_batch_parity_summary_for_report(),
        Ok(summary.summary_line())
    );
    assert_eq!(
        reference_snapshot_mixed_time_scale_batch_parity_summary_for_report(),
        summary.summary_line()
    );
    assert!(
        reference_snapshot_mixed_time_scale_batch_parity_summary_for_report()
            .starts_with("JPL reference snapshot mixed TT/TDB batch parity: ")
    );
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
fn snapshot_backend_resolves_a_later_epoch() {
    let backend = JplSnapshotBackend;
    let request = EphemerisRequest {
        body: pleiades_backend::CelestialBody::Mars,
        instant: Instant::new(JulianDay::from_days(2_634_167.0), TimeScale::Tt),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };

    let result = backend
        .position(&request)
        .expect("reference fixture should resolve at the later epoch");
    assert_eq!(result.quality, QualityAnnotation::Exact);
    let ecliptic = result
        .ecliptic
        .expect("reference fixture should include ecliptic coordinates");
    assert!(ecliptic.longitude.degrees().is_finite());
    assert!(ecliptic.latitude.degrees().is_finite());
}

#[test]
fn snapshot_backend_interpolates_between_fixture_epochs() {
    let backend = JplSnapshotBackend;
    let request = EphemerisRequest {
        body: pleiades_backend::CelestialBody::Mars,
        instant: Instant::new(JulianDay::from_days(2_415_022.0), TimeScale::Tdb),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };

    let result = backend
        .position(&request)
        .expect("reference fixture should interpolate between Mars samples");
    assert_eq!(result.quality, QualityAnnotation::Interpolated);
    let ecliptic = result
        .ecliptic
        .expect("interpolated fixture should include ecliptic coordinates");
    assert!(ecliptic.longitude.degrees().is_finite());
    assert!(ecliptic.latitude.degrees().is_finite());
    assert!(ecliptic
        .distance_au
        .expect("distance should exist")
        .is_finite());
}

#[test]
fn interpolation_quality_samples_are_reportable() {
    let samples = interpolation_quality_samples();
    assert_eq!(samples.len(), 293);
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
        .any(|sample| sample.epoch.julian_day.days() == 2_400_000.0));
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
        .any(|sample| sample.epoch.julian_day.days() == 2_524_593.5));
    assert!(samples
        .iter()
        .any(|sample| sample.epoch.julian_day.days() == 2_500_000.0));
    assert!(samples
        .iter()
        .any(|sample| sample.epoch.julian_day.days() == 2_600_000.0));
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
    assert_eq!(summary.sample_count, 293);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.epoch_count, 27);
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
    assert!(rendered.contains("293 samples across 16 bodies and 27 epochs"));
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
    assert_eq!(coverage.sample_count, 293);
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
    assert!(rendered.contains("293 samples across 16 bodies ["));
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
    assert_eq!(summary.request_count, 293);
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
    assert_eq!(summary.sample_count, 293);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.epoch_count, 27);
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
    assert_eq!(
        summary.validated_summary_line(),
        Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch)
    );
    assert_eq!(
            format_jpl_independent_holdout_summary(&summary),
            "JPL independent hold-out: unavailable (summary no longer matches the derived interpolation evidence)"
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
fn snapshot_backend_resolves_mars_at_2600000() {
    let backend = JplSnapshotBackend;
    let request = EphemerisRequest {
        body: pleiades_backend::CelestialBody::Mars,
        instant: Instant::new(JulianDay::from_days(2_600_000.0), TimeScale::Tdb),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };

    let result = backend
        .position(&request)
        .expect("reference snapshot should resolve the Mars outer-boundary anchor");
    assert_eq!(result.quality, QualityAnnotation::Exact);
    let ecliptic = result
        .ecliptic
        .expect("reference snapshot should include ecliptic coordinates");
    assert!((ecliptic.longitude.degrees() - 56.24824943387116).abs() < 1e-12);
    assert!((ecliptic.latitude.degrees() - (-0.18908796740844558)).abs() < 1e-12);
    assert!(
        (ecliptic.distance_au.expect("distance should exist") - 2.3186132195308553).abs() < 1e-12
    );
}

#[test]
fn snapshot_backend_resolves_mars_at_2634167() {
    let backend = JplSnapshotBackend;
    let request = EphemerisRequest {
        body: pleiades_backend::CelestialBody::Mars,
        instant: Instant::new(JulianDay::from_days(2_634_167.0), TimeScale::Tdb),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };

    let result = backend
        .position(&request)
        .expect("reference snapshot should resolve the Mars outer-boundary epoch");
    assert_eq!(result.quality, QualityAnnotation::Exact);
    assert_eq!(result.instant, request.instant);

    let ecliptic = result
        .ecliptic
        .expect("reference snapshot should include ecliptic coordinates");
    let entry = reference_snapshot()
        .iter()
        .find(|entry| {
            entry.body == pleiades_backend::CelestialBody::Mars
                && entry.epoch.julian_day.days() == 2_634_167.0
        })
        .expect("reference snapshot should include the Mars outer-boundary row");
    assert_eq!(ecliptic, entry.ecliptic());
}

#[test]
fn snapshot_backend_resolves_mars_at_2600000_matches_reference_snapshot() {
    let backend = JplSnapshotBackend;
    let request = EphemerisRequest {
        body: pleiades_backend::CelestialBody::Mars,
        instant: Instant::new(JulianDay::from_days(2_600_000.0), TimeScale::Tdb),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };

    let result = backend
        .position(&request)
        .expect("reference snapshot should resolve the Mars outer-boundary anchor");
    assert_eq!(result.quality, QualityAnnotation::Exact);
    assert_eq!(result.instant, request.instant);

    let ecliptic = result
        .ecliptic
        .expect("reference snapshot should include ecliptic coordinates");
    let entry = reference_snapshot()
        .iter()
        .find(|entry| {
            entry.body == pleiades_backend::CelestialBody::Mars
                && entry.epoch.julian_day.days() == 2_600_000.0
        })
        .expect("reference snapshot should include the Mars outer-boundary anchor row");
    assert_eq!(ecliptic, entry.ecliptic());
}

#[test]
fn snapshot_backend_resolves_major_bodies_at_1749_boundary() {
    let backend = JplSnapshotBackend;
    let epoch = Instant::new(JulianDay::from_days(2_360_233.5), TimeScale::Tdb);
    let entries = reference_snapshot()
        .iter()
        .filter(|entry| entry.epoch == epoch)
        .collect::<Vec<_>>();

    assert_eq!(entries.len(), 9);
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.body.clone())
            .collect::<Vec<_>>(),
        vec![
            pleiades_backend::CelestialBody::Sun,
            pleiades_backend::CelestialBody::Moon,
            pleiades_backend::CelestialBody::Mercury,
            pleiades_backend::CelestialBody::Venus,
            pleiades_backend::CelestialBody::Mars,
            pleiades_backend::CelestialBody::Jupiter,
            pleiades_backend::CelestialBody::Saturn,
            pleiades_backend::CelestialBody::Uranus,
            pleiades_backend::CelestialBody::Neptune,
        ]
    );

    for entry in entries {
        let request = EphemerisRequest {
            body: entry.body.clone(),
            instant: entry.epoch,
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("reference snapshot should resolve the 1749-12-31 boundary row");
        assert_eq!(result.quality, QualityAnnotation::Exact);
        assert_eq!(result.instant, request.instant);
        assert_eq!(result.body, request.body);
        assert_eq!(result.ecliptic, Some(entry.ecliptic()));
    }
}

#[test]
fn snapshot_backend_resolves_major_bodies_at_1800_boundary() {
    let backend = JplSnapshotBackend;
    let epoch = Instant::new(JulianDay::from_days(2_378_498.5), TimeScale::Tdb);
    let entries = reference_snapshot()
        .iter()
        .filter(|entry| entry.epoch == epoch)
        .collect::<Vec<_>>();

    assert_eq!(entries.len(), 16);
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.body.clone())
            .collect::<Vec<_>>(),
        vec![
            pleiades_backend::CelestialBody::Sun,
            pleiades_backend::CelestialBody::Moon,
            pleiades_backend::CelestialBody::Mercury,
            pleiades_backend::CelestialBody::Venus,
            pleiades_backend::CelestialBody::Mars,
            pleiades_backend::CelestialBody::Jupiter,
            pleiades_backend::CelestialBody::Saturn,
            pleiades_backend::CelestialBody::Uranus,
            pleiades_backend::CelestialBody::Neptune,
            pleiades_backend::CelestialBody::Pluto,
            pleiades_backend::CelestialBody::Ceres,
            pleiades_backend::CelestialBody::Pallas,
            pleiades_backend::CelestialBody::Juno,
            pleiades_backend::CelestialBody::Vesta,
            pleiades_backend::CelestialBody::Custom(pleiades_types::CustomBodyId::new(
                "asteroid", "433-Eros"
            )),
            pleiades_backend::CelestialBody::Custom(pleiades_types::CustomBodyId::new(
                "asteroid",
                "99942-Apophis",
            )),
        ]
    );

    for entry in entries {
        let request = EphemerisRequest {
            body: entry.body.clone(),
            instant: entry.epoch,
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: ZodiacMode::Tropical,
            apparent: Apparentness::Mean,
        };

        let result = backend
            .position(&request)
            .expect("reference snapshot should resolve the 1800-01-03 boundary row");
        assert_eq!(result.quality, QualityAnnotation::Exact);
        assert_eq!(result.instant, request.instant);
        assert_eq!(result.body, request.body);
        assert_eq!(result.ecliptic, Some(entry.ecliptic()));
    }
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
fn batch_query_preserves_reference_asteroid_order_and_values() {
    let backend = JplSnapshotBackend;
    let evidence = reference_asteroid_evidence();
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
        .expect("batch query should preserve the asteroid reference order");

    assert_eq!(results.len(), evidence.len());
    for (sample, result) in evidence.iter().zip(results.iter()) {
        assert_eq!(result.body, sample.body);
        assert_eq!(result.quality, QualityAnnotation::Exact);
        let ecliptic = result
            .ecliptic
            .expect("reference snapshot should include ecliptic coordinates");
        assert!((ecliptic.longitude.degrees() - sample.longitude_deg).abs() < 1e-12);
        assert!((ecliptic.latitude.degrees() - sample.latitude_deg).abs() < 1e-12);
        assert!(
            (ecliptic.distance_au.expect("distance should exist") - sample.distance_au).abs()
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
fn reference_asteroid_evidence_exposes_exact_j2000_samples() {
    let evidence = reference_asteroid_evidence();
    assert_eq!(evidence.len(), 6);
    assert_eq!(reference_asteroids().len(), evidence.len());
    assert!(evidence.iter().all(|sample| {
        sample.epoch.julian_day.days() == REFERENCE_EPOCH_JD
            && sample.longitude_deg.is_finite()
            && sample.latitude_deg.is_finite()
            && sample.distance_au.is_finite()
    }));
    assert_eq!(evidence[0].body, pleiades_backend::CelestialBody::Ceres);
    assert_eq!(evidence[1].body, pleiades_backend::CelestialBody::Pallas);
    assert_eq!(evidence[2].body, pleiades_backend::CelestialBody::Juno);
    assert_eq!(evidence[3].body, pleiades_backend::CelestialBody::Vesta);
    assert_eq!(
        evidence[4].body,
        pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
    );
    assert_eq!(
        evidence[5].body,
        pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "99942-Apophis"))
    );
    assert!((evidence[0].longitude_deg - 184.459642854516).abs() < 1e-12);
    assert!((evidence[4].distance_au - 1.854402724550437).abs() < 1e-12);
}

#[test]
fn reference_asteroid_requests_preserve_the_exact_j2000_slice() {
    let backend = JplSnapshotBackend;
    let requests = reference_asteroid_requests(CoordinateFrame::Equatorial)
        .expect("selected asteroid requests should exist");
    let results = backend
        .positions(&requests)
        .expect("selected asteroid batch query should preserve the exact J2000 slice");

    assert_eq!(results.len(), requests.len());
    for ((sample, result), request) in reference_asteroid_evidence()
        .iter()
        .zip(results.iter())
        .zip(requests.iter())
    {
        assert_eq!(result.body, sample.body);
        assert_eq!(result.instant, sample.epoch);
        assert_eq!(result.frame, request.frame);
        assert_eq!(result.quality, QualityAnnotation::Exact);

        let ecliptic = result
            .ecliptic
            .expect("selected asteroid batch rows should include ecliptic coordinates");
        assert!((ecliptic.longitude.degrees() - sample.longitude_deg).abs() < 1e-12);
        assert!((ecliptic.latitude.degrees() - sample.latitude_deg).abs() < 1e-12);
        assert!(
            (ecliptic.distance_au.expect("distance should exist") - sample.distance_au).abs()
                < 1e-12
        );

        let equatorial = result
            .equatorial
            .expect("selected asteroid batch rows should include equatorial coordinates");
        let expected_equatorial = ecliptic.to_equatorial(result.instant.mean_obliquity());
        assert_eq!(equatorial, expected_equatorial);
    }
}

#[test]
fn reference_asteroid_batch_parity_requests_preserve_the_selected_j2000_slice() {
    let backend = JplSnapshotBackend;
    let requests = reference_asteroid_batch_parity_requests()
        .expect("selected asteroid batch parity requests should exist");
    let results = backend
        .positions(&requests)
        .expect("mixed-frame selected asteroid batch query should preserve the exact slice");

    assert_eq!(results.len(), requests.len());
    for ((sample, result), request) in reference_asteroid_evidence()
        .iter()
        .zip(results.iter())
        .zip(requests.iter())
    {
        assert_eq!(result.body, sample.body);
        assert_eq!(result.instant, sample.epoch);
        assert_eq!(result.frame, request.frame);
        assert_eq!(result.quality, QualityAnnotation::Exact);

        let ecliptic = result
            .ecliptic
            .expect("selected asteroid batch rows should include ecliptic coordinates");
        let expected = EclipticCoordinates::new(
            Longitude::from_degrees(sample.longitude_deg),
            Latitude::from_degrees(sample.latitude_deg),
            Some(sample.distance_au),
        );
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

        let equatorial = result
            .equatorial
            .expect("selected asteroid batch rows should include equatorial coordinates");
        assert_eq!(
            equatorial,
            ecliptic.to_equatorial(result.instant.mean_obliquity())
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
