use super::support::*;
use crate::systems::*;
use pleiades_types::{Angle, CustomHouseSystem, Latitude};

#[test]
fn house_request_summary_line_reports_instant_observer_system_and_obliquity() {
    let request = sample_request(HouseSystem::WholeSign);
    assert_eq!(
        request.summary_line(),
        format!(
            "instant={}; observer={}; system={}; obliquity=auto",
            request.instant, request.observer, request.system
        )
    );
    assert_eq!(request.to_string(), request.summary_line());

    let request_with_obliquity = request.with_obliquity(Angle::from_degrees(23.5));
    assert_eq!(
        request_with_obliquity.summary_line(),
        format!(
            "instant={}; observer={}; system={}; obliquity=23.5°",
            request_with_obliquity.instant,
            request_with_obliquity.observer,
            request_with_obliquity.system
        )
    );

    let mut custom = CustomHouseSystem::new("House from custom notes");
    custom.aliases.push("Custom alias".to_string());
    custom.notes = Some("extra custom context".to_string());
    let custom_request = HouseRequest::new(
        Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        ),
        observer(),
        HouseSystem::Custom(custom.clone()),
    );
    assert!(custom_request.summary_line().contains(&custom.to_string()));
}

#[test]
fn house_request_validate_accepts_the_baseline_request() {
    let request = sample_request(HouseSystem::WholeSign);
    assert!(request.validate().is_ok());
}

#[test]
fn house_snapshot_summary_line_reports_angles_and_cusp_count() {
    let snapshot =
        calculate_houses(&sample_request(HouseSystem::Equal)).expect("equal houses should work");
    let summary = snapshot.summary_line();

    assert!(summary.contains("system=Equal"));
    assert!(summary.contains("angles=ASC "));
    assert!(summary.contains("MC "));
    assert!(summary.contains("IC "));
    assert!(summary.contains("DSC "));
    assert!(summary.contains("cusp-count=12"));
    assert_eq!(snapshot.to_string(), summary);
    assert_eq!(snapshot.validated_summary_line().unwrap(), summary);
}

#[test]
fn house_request_validate_rejects_non_finite_obliquity_overrides() {
    let request =
        sample_request(HouseSystem::WholeSign).with_obliquity(Angle::from_degrees(f64::NAN));

    let error = request
        .validate()
        .expect_err("non-finite obliquity should fail fast");
    assert_eq!(error.kind, crate::error::HouseErrorKind::InvalidObliquity);
    assert!(error
        .message
        .contains("house obliquity override must be finite"));
}

#[test]
fn house_request_validate_rejects_non_finite_topocentric_elevation() {
    let mut request = sample_request(HouseSystem::Topocentric);
    request.observer.elevation_m = Some(f64::NAN);

    let error = request
        .validate()
        .expect_err("non-finite elevation should fail fast");
    assert_eq!(error.kind, crate::error::HouseErrorKind::InvalidElevation);
    assert!(error
        .message
        .contains("observer elevation must be finite when provided"));
}

#[test]
fn house_request_validate_rejects_non_finite_elevation_even_without_topocentric_houses() {
    let mut request = sample_request(HouseSystem::Equal);
    request.observer.elevation_m = Some(f64::NAN);

    let error = request
        .validate()
        .expect_err("non-finite elevation should fail fast");
    assert_eq!(error.kind, crate::error::HouseErrorKind::InvalidElevation);
    assert!(error
        .message
        .contains("observer elevation must be finite when provided"));
}

#[test]
fn house_request_validate_rejects_non_finite_observer_longitude() {
    let mut request = sample_request(HouseSystem::Equal);
    request.observer.longitude = Longitude::from_degrees(f64::NAN);

    let error = request
        .validate()
        .expect_err("non-finite longitude should fail fast");
    assert_eq!(error.kind, crate::error::HouseErrorKind::InvalidLongitude);
    assert!(error.message.contains("observer longitude must be finite"));
}

#[test]
fn observer_latitudes_outside_the_valid_range_are_rejected() {
    let mut request = sample_request(HouseSystem::Equal);
    request.observer.latitude = Latitude::from_degrees(90.000_1);

    let error = calculate_houses(&request).expect_err("invalid observer latitude should fail");
    assert_eq!(error.kind, crate::error::HouseErrorKind::InvalidLatitude);
    assert!(error
        .message
        .contains("observer latitude 90.0001° is outside the valid range"));
}

#[test]
fn non_finite_obliquity_overrides_are_rejected() {
    for system in [
        HouseSystem::Equal,
        HouseSystem::Placidus,
        HouseSystem::Topocentric,
    ] {
        for obliquity in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let request =
                sample_request(system.clone()).with_obliquity(Angle::from_degrees(obliquity));

            let error = calculate_houses(&request).expect_err("invalid obliquity should fail");
            assert_eq!(error.kind, crate::error::HouseErrorKind::InvalidObliquity);
            assert!(error.message.contains("obliquity override must be finite"));
        }
    }
}

#[test]
fn house_snapshots_reject_non_finite_derived_values() {
    let mut cusps = vec![Longitude::from_degrees(0.0); 12];
    cusps[4] = Longitude::from_degrees(f64::NAN);

    let angles = HouseAngles {
        ascendant: Longitude::from_degrees(15.0),
        descendant: Longitude::from_degrees(195.0),
        midheaven: Longitude::from_degrees(45.0),
        imum_coeli: Longitude::from_degrees(225.0),
    };
    let snapshot = HouseSnapshot {
        system: HouseSystem::Equal,
        instant: sample_request(HouseSystem::Equal).instant,
        observer: sample_request(HouseSystem::Equal).observer,
        obliquity: Angle::from_degrees(23.4),
        angles,
        asc_mc: test_asc_mc(angles),
        cusps,
    };

    let error = snapshot
        .validate()
        .expect_err("non-finite cusp should fail");
    assert_eq!(error.kind, crate::error::HouseErrorKind::NumericalFailure);
    assert!(error.message.contains("non-finite cusp 5"));
}

#[test]
fn house_snapshots_reject_wrong_cusp_counts() {
    let angles = HouseAngles {
        ascendant: Longitude::from_degrees(15.0),
        descendant: Longitude::from_degrees(195.0),
        midheaven: Longitude::from_degrees(45.0),
        imum_coeli: Longitude::from_degrees(225.0),
    };
    let equal_snapshot = HouseSnapshot {
        system: HouseSystem::Equal,
        instant: sample_request(HouseSystem::Equal).instant,
        observer: observer(),
        obliquity: Angle::from_degrees(23.4),
        angles,
        asc_mc: test_asc_mc(angles),
        cusps: vec![Longitude::from_degrees(0.0); 36],
    };

    let equal_error = equal_snapshot
        .validate()
        .expect_err("wrong cusp count should fail for 12-cusp systems");
    assert_eq!(
        equal_error.kind,
        crate::error::HouseErrorKind::NumericalFailure
    );
    assert!(equal_error
        .message
        .contains("house calculation for Equal produced 36 cusps (expected 12)"));

    let gauquelin_angles = HouseAngles {
        ascendant: Longitude::from_degrees(15.0),
        descendant: Longitude::from_degrees(195.0),
        midheaven: Longitude::from_degrees(45.0),
        imum_coeli: Longitude::from_degrees(225.0),
    };
    let gauquelin_snapshot = HouseSnapshot {
        system: HouseSystem::Gauquelin,
        instant: sample_request(HouseSystem::Gauquelin).instant,
        observer: observer(),
        obliquity: Angle::from_degrees(23.4),
        angles: gauquelin_angles,
        asc_mc: test_asc_mc(gauquelin_angles),
        cusps: vec![Longitude::from_degrees(0.0); 12],
    };

    let gauquelin_error = gauquelin_snapshot
        .validate()
        .expect_err("wrong cusp count should fail for Gauquelin sectors");
    assert_eq!(
        gauquelin_error.kind,
        crate::error::HouseErrorKind::NumericalFailure
    );
    assert!(gauquelin_error
        .message
        .contains("house calculation for Gauquelin sectors produced 12 cusps (expected 36)"));
}

#[test]
fn house_snapshots_reject_inconsistent_angle_pairs() {
    let broken_descendant_angles = HouseAngles {
        ascendant: Longitude::from_degrees(15.0),
        descendant: Longitude::from_degrees(200.0),
        midheaven: Longitude::from_degrees(45.0),
        imum_coeli: longitude_opposite(Longitude::from_degrees(45.0)),
    };
    let broken_descendant_snapshot = HouseSnapshot {
        system: HouseSystem::Equal,
        instant: sample_request(HouseSystem::Equal).instant,
        observer: observer(),
        obliquity: Angle::from_degrees(23.4),
        angles: broken_descendant_angles,
        asc_mc: test_asc_mc(broken_descendant_angles),
        cusps: vec![Longitude::from_degrees(0.0); 12],
    };

    let descendant_error = broken_descendant_snapshot
        .validate()
        .expect_err("a non-opposite descendant should fail validation");
    assert_eq!(
        descendant_error.kind,
        crate::error::HouseErrorKind::NumericalFailure
    );
    assert!(descendant_error.message.contains(
        "house calculation for Equal produced a descendant that is not opposite the ascendant"
    ));

    let broken_ic_angles = HouseAngles {
        ascendant: Longitude::from_degrees(15.0),
        descendant: longitude_opposite(Longitude::from_degrees(15.0)),
        midheaven: Longitude::from_degrees(45.0),
        imum_coeli: Longitude::from_degrees(250.0),
    };
    let broken_ic_snapshot = HouseSnapshot {
        system: HouseSystem::Equal,
        instant: sample_request(HouseSystem::Equal).instant,
        observer: observer(),
        obliquity: Angle::from_degrees(23.4),
        angles: broken_ic_angles,
        asc_mc: test_asc_mc(broken_ic_angles),
        cusps: vec![Longitude::from_degrees(0.0); 12],
    };

    let ic_error = broken_ic_snapshot
        .validate()
        .expect_err("a non-opposite imum coeli should fail validation");
    assert_eq!(
        ic_error.kind,
        crate::error::HouseErrorKind::NumericalFailure
    );
    assert!(ic_error.message.contains(
        "house calculation for Equal produced an imum coeli that is not opposite the midheaven"
    ));
}
