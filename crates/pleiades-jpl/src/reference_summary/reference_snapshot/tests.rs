//! Tests for the reference_snapshot module.

use crate::test_support::backend;
#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use pleiades_backend::{Apparentness, EphemerisErrorKind, EphemerisRequest};
#[allow(unused_imports)]
use pleiades_backend::{CelestialBody, EphemerisBackend, QualityAnnotation};
#[allow(unused_imports)]
use pleiades_types::CoordinateFrame;

#[test]
fn reference_snapshot_covers_the_expected_bodies_and_epochs() {
    let metadata = backend().metadata();
    assert!(metadata
        .supported_bodies()
        .contains(&pleiades_backend::CelestialBody::Sun));
    assert!(metadata
        .supported_bodies()
        .contains(&pleiades_backend::CelestialBody::Moon));
    assert!(metadata
        .supported_bodies()
        .contains(&pleiades_backend::CelestialBody::Pluto));
    assert!(metadata
        .supported_bodies()
        .contains(&pleiades_backend::CelestialBody::Ceres));
    assert!(metadata
        .supported_bodies()
        .contains(&pleiades_backend::CelestialBody::Pallas));
    assert!(metadata
        .supported_bodies()
        .contains(&pleiades_backend::CelestialBody::Juno));
    assert!(metadata
        .supported_bodies()
        .contains(&pleiades_backend::CelestialBody::Vesta));
    assert!(metadata
        .supported_bodies()
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
    assert_eq!(reference_epochs().len(), 23);
    assert_eq!(
        reference_snapshot()
            .iter()
            .filter(|entry| entry.epoch.julian_day.days() == 2_400_000.0)
            .count(),
        0
    );
    assert_eq!(
        reference_snapshot()
            .iter()
            .filter(|entry| entry.epoch.julian_day.days() == 2_500_000.0)
            .count(),
        6
    );
    assert_eq!(
        reference_snapshot()
            .iter()
            .filter(|entry| entry.epoch.julian_day.days() == 2_600_000.0)
            .count(),
        0
    );
}

#[test]
fn reference_snapshot_summary_reports_the_expected_coverage() {
    let summary = reference_snapshot_summary().expect("reference snapshot summary should exist");
    summary
        .validate()
        .expect("reference snapshot summary should validate");
    assert_eq!(summary.row_count, 277);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.bodies, reference_bodies());
    assert_eq!(summary.epoch_count, 23);
    assert_eq!(summary.asteroid_row_count, 95);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(
            summary.summary_line(),
            format!(
                "Reference snapshot coverage: 277 rows across 16 bodies and 23 epochs (95 asteroid rows; JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies: {}",
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
    assert!(report.contains(&reference_snapshot_lunar_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_high_curvature_summary_for_report()));
    assert!(report.contains(&reference_snapshot_high_curvature_window_summary_for_report()));
    assert!(report.contains(&reference_snapshot_high_curvature_epoch_coverage_summary_for_report()));
    assert!(report.contains(&reference_snapshot_2451545_major_body_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_major_body_boundary_summary_for_report()));
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
    assert!(report.contains(&selected_asteroid_boundary_summary_for_report()));
    assert!(report.contains(&selected_asteroid_bridge_summary_for_report()));
    assert!(report.contains(&selected_asteroid_dense_boundary_summary_for_report()));
    assert!(report.contains(&selected_asteroid_terminal_boundary_summary_for_report()));
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
    assert_eq!(
        summary.major_bodies,
        vec![
            pleiades_backend::CelestialBody::Sun,
            pleiades_backend::CelestialBody::Moon,
            pleiades_backend::CelestialBody::Mercury,
            pleiades_backend::CelestialBody::Venus,
            pleiades_backend::CelestialBody::Jupiter,
            pleiades_backend::CelestialBody::Mars,
            pleiades_backend::CelestialBody::Neptune,
            pleiades_backend::CelestialBody::Pluto,
            pleiades_backend::CelestialBody::Saturn,
            pleiades_backend::CelestialBody::Uranus,
        ]
    );
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
    assert!(report.contains(&reference_snapshot_source_summary_for_report()));
    assert!(report.contains(&reference_snapshot_source_window_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_2451917_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_2453000_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_2500000_summary_for_report()));
    assert!(report.contains(&selected_asteroid_source_2634167_summary_for_report()));
    assert!(report.contains(&reference_snapshot_major_body_bridge_summary_for_report()));
    assert!(report.contains(&reference_snapshot_equatorial_parity_summary_for_report()));
    assert!(report.contains(&reference_snapshot_batch_parity_summary_for_report()));
    assert!(report.contains(&reference_snapshot_1900_selected_body_boundary_summary_for_report()));
    assert!(
        report.contains(&reference_snapshot_2415020_selected_body_boundary_summary_for_report())
    );
    assert!(report.contains(&reference_snapshot_lunar_boundary_summary_for_report()));
    assert!(report.contains(&reference_snapshot_high_curvature_summary_for_report()));
    assert!(report.contains(&reference_snapshot_high_curvature_window_summary_for_report()));
    assert!(report.contains(&reference_snapshot_high_curvature_epoch_coverage_summary_for_report()));
    assert!(report.contains(&reference_snapshot_exact_j2000_evidence_summary_for_report()));
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
                epoch_count: 24,
                derived_epoch_count: 23,
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
    assert_eq!(summary.row_count, 277);
    assert_eq!(summary.body_count, 16);
    assert_eq!(summary.bodies, reference_bodies());
    assert_eq!(summary.epoch_count, 23);
    assert_eq!(summary.earliest_epoch.julian_day.days(), 2_378_498.5);
    assert_eq!(summary.latest_epoch.julian_day.days(), 2_634_167.0);
    assert_eq!(
            summary.summary_line(),
            format!(
                "JPL reference snapshot equatorial parity: 277 rows across 16 bodies and 23 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies: {}; mean-obliquity transform against the checked-in ecliptic fixture",
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
    assert_eq!(summary.snapshot.row_count, 277);
    assert_eq!(summary.snapshot.body_count, 16);
    assert_eq!(summary.snapshot.bodies, reference_bodies());
    assert_eq!(summary.snapshot.epoch_count, 23);
    assert_eq!(
        summary.snapshot.earliest_epoch.julian_day.days(),
        2_378_498.5
    );
    assert_eq!(summary.snapshot.latest_epoch.julian_day.days(), 2_634_167.0);
    assert!(summary.ecliptic_request_count > 0);
    assert!(summary.equatorial_request_count > 0);
    assert_eq!(summary.exact_count, 277);
    assert_eq!(summary.interpolated_count, 0);
    assert_eq!(summary.approximate_count, 0);
    assert_eq!(summary.unknown_count, 0);
    assert_eq!(summary.validate(), Ok(()));
    assert!(summary
            .summary_line()
            .contains("JPL reference snapshot batch parity: 277 rows across 16 bodies and 23 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies: "));
    assert!(summary
            .summary_line()
            .contains("quality counts: Exact=277, Interpolated=0, Approximate=0, Unknown=0; batch/single parity preserved"));
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
            "JPL reference snapshot batch parity: 277 rows across 16 bodies and 23 epochs (JD 2378498.5 (TDB)..JD 2634167.0 (TDB)); bodies:"
        ));
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
fn reference_snapshot_body_class_coverage_summary_reports_the_expected_body_classes() {
    let summary = reference_snapshot_body_class_coverage_summary()
        .expect("reference snapshot body-class coverage summary should exist");

    assert_eq!(summary.major_body_row_count, 182);
    assert_eq!(summary.major_bodies.len(), 10);
    assert_eq!(
        summary.major_bodies[0],
        pleiades_backend::CelestialBody::Sun
    );
    assert_eq!(
        summary.major_bodies[9],
        pleiades_backend::CelestialBody::Uranus
    );
    assert_eq!(summary.major_epoch_count, 20);
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

    assert_eq!(summary.sample_count, 277);
    assert_eq!(summary.sample_bodies.len(), 16);
    assert_eq!(summary.epoch_count, 23);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        reference_snapshot_source_window_summary_for_report(),
        summary.summary_line()
    );
    assert_eq!(
        summary.windows[0].body,
        pleiades_backend::CelestialBody::Ceres
    );
    assert_eq!(summary.windows[0].sample_count, 17);
    assert_eq!(summary.windows[0].epoch_count, 17);
    assert_eq!(
        summary.windows[0].earliest_epoch.julian_day.days(),
        2_378_498.5
    );
    assert_eq!(
        summary.windows[0].latest_epoch.julian_day.days(),
        2_634_167.0
    );
    assert_eq!(
        summary.windows[5].body,
        pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "99942-Apophis"))
    );
    assert_eq!(summary.windows[5].sample_count, 10);
    assert_eq!(summary.windows[5].epoch_count, 10);
    assert_eq!(
        summary.windows[5].earliest_epoch.julian_day.days(),
        2_378_498.5
    );
    assert_eq!(
        summary.windows[5].latest_epoch.julian_day.days(),
        2_634_167.0
    );
    assert_eq!(
        summary.windows[9].body,
        pleiades_backend::CelestialBody::Venus
    );
    assert_eq!(summary.windows[9].sample_count, 20);
    assert_eq!(summary.windows[9].epoch_count, 20);
    assert_eq!(
        summary.windows[9].earliest_epoch.julian_day.days(),
        2_415_020.5
    );
    assert_eq!(
        summary.windows[9].latest_epoch.julian_day.days(),
        2_453_000.5
    );
    assert_eq!(
        summary.windows[15].body,
        pleiades_backend::CelestialBody::Uranus
    );
    assert_eq!(summary.windows[15].sample_count, 17);
    assert_eq!(summary.windows[15].epoch_count, 17);
    assert_eq!(
        summary.windows[15].earliest_epoch.julian_day.days(),
        2_451_545.0
    );
    assert_eq!(
        summary.windows[15].latest_epoch.julian_day.days(),
        2_453_000.5
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
    assert_eq!(manifest.coverage.as_deref(), Some("major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; selected bodies sampled at 1900-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2451915.25 and 2451915.75 for Sun, Moon, Mercury, Venus; major bodies sampled at 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, and 2453000.5; major bodies sampled at 2451915.5 for Sun through Pluto; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2378498.5, 2451910.5 through 2451919.5, with 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 boundary coverage, 2003-12-27, 2132-08-31, 2500-01-01, and 2634167; asteroid:99942-Apophis is now also sampled at 2378498.5 and 2451917.5 to complete the selected-asteroid bridge."));
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
            "Reference snapshot manifest: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; selected bodies sampled at 1900-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2451915.25 and 2451915.75 for Sun, Moon, Mercury, Venus; major bodies sampled at 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, and 2453000.5; major bodies sampled at 2451915.5 for Sun through Pluto; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2378498.5, 2451910.5 through 2451919.5, with 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 boundary coverage, 2003-12-27, 2132-08-31, 2500-01-01, and 2634167; asteroid:99942-Apophis is now also sampled at 2378498.5 and 2451917.5 to complete the selected-asteroid bridge.; columns=epoch_jd, body, x_km, y_km, z_km; redistribution=repository-checked regression fixtures, not a broad public corpus."
        );
}

#[test]
fn reference_snapshot_manifest_summary_rejects_metadata_drift() {
    let summary = reference_snapshot_manifest_summary();
    let error = summary
            .validate_with_expected_metadata(
                "wrong title",
                "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
                "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; selected bodies sampled at 1900-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2451915.25 and 2451915.75 for Sun, Moon, Mercury, Venus; major bodies sampled at 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, and 2453000.5; major bodies sampled at 2451915.5 for Sun through Pluto; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2378498.5, 2451910.5 through 2451919.5, with 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 boundary coverage, 2003-12-27, 2132-08-31, 2500-01-01, and 2634167; asteroid:99942-Apophis is now also sampled at 2378498.5 and 2451917.5 to complete the selected-asteroid bridge.",
                &["epoch_jd", "body", "x_km", "y_km", "z_km"],
            )
            .expect_err("reference snapshot manifest summary should reject title drift");

    assert!(matches!(
        error,
        SnapshotManifestSummaryValidationError::MetadataMismatch { field: "title", .. }
    ));
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
