use super::support::*;
use crate::systems::*;
use pleiades_types::{Angle, CustomHouseSystem, Latitude};

#[test]
fn placidian_houses_report_invalid_latitude_at_the_pole() {
    // 90°N exceeds the Placidus 66° bound, so the strict check fires before
    // the iterative cusp solver can produce a zero-derivative failure.
    let mut request =
        sample_request(HouseSystem::Placidus).with_obliquity(Angle::from_degrees(0.0));
    request.observer.latitude = Latitude::from_degrees(90.0);
    request.observer.longitude = Longitude::from_degrees(0.0);

    let error = calculate_houses(&request).expect_err("polar Placidus should be rejected");
    assert_eq!(error.kind, crate::error::HouseErrorKind::InvalidLatitude);
    assert!(error
        .message
        .contains("Placidus is undefined beyond |latitude| 66"));
}

#[test]
fn custom_house_systems_are_reported_explicitly_when_unsupported() {
    let mut custom = CustomHouseSystem::new("My Custom Houses");
    custom.aliases.push("MCH".to_string());
    custom.notes = Some("user-defined formula".to_string());

    let error = calculate_houses(&sample_request(HouseSystem::Custom(custom)))
        .expect_err("custom house systems should still be rejected");
    assert_eq!(
        error.kind,
        crate::error::HouseErrorKind::UnsupportedHouseSystem
    );
    assert_eq!(
        error.to_string(),
        "UnsupportedHouseSystem: house placement for custom house system My Custom Houses [aliases: MCH] (user-defined formula) is not implemented yet"
    );
}

#[test]
fn baseline_quadrant_systems_are_implemented() {
    for system in [
        HouseSystem::Placidus,
        HouseSystem::Koch,
        HouseSystem::Regiomontanus,
        HouseSystem::Campanus,
        HouseSystem::Carter,
        HouseSystem::Alcabitius,
        HouseSystem::Meridian,
        HouseSystem::Axial,
        HouseSystem::Morinus,
        HouseSystem::Topocentric,
        HouseSystem::KrusinskiPisaGoelzer,
    ] {
        let snapshot = calculate_houses(&sample_request(system.clone()))
            .expect("baseline quadrant system should calculate");
        assert_eq!(snapshot.cusps.len(), 12);
    }
}

#[test]
fn albategnius_and_pullen_release_systems_are_available() {
    for system in [
        HouseSystem::Albategnius,
        HouseSystem::PullenSd,
        HouseSystem::PullenSr,
    ] {
        let snapshot = calculate_houses(&sample_request(system.clone()))
            .expect("release house system should calculate");
        assert_eq!(snapshot.cusps.len(), 12);
        assert_eq!(snapshot.cusps[9], snapshot.angles.midheaven);
        assert_eq!(
            (snapshot.cusps[6].degrees() - snapshot.cusps[0].degrees()).rem_euclid(360.0),
            180.0
        );
    }
}

#[test]
fn sunshine_release_system_anchors_the_documented_axes() {
    let snapshot = calculate_houses(&sample_request(HouseSystem::Sunshine))
        .expect("sunshine houses should work");
    assert_eq!(snapshot.cusps.len(), 12);
    assert!(
        snapshot.cusps[0] == snapshot.angles.ascendant
            || snapshot.cusps[0] == longitude_opposite(snapshot.angles.ascendant)
    );
    assert!(
        snapshot.cusps[9] == snapshot.angles.midheaven
            || snapshot.cusps[9] == longitude_opposite(snapshot.angles.midheaven)
    );
    assert_eq!(snapshot.cusps[3], longitude_opposite(snapshot.cusps[9]));
    assert_eq!(snapshot.cusps[6], longitude_opposite(snapshot.cusps[0]));
}

#[test]
fn horizon_and_apc_release_systems_are_available() {
    let horizon = calculate_houses(&sample_request(HouseSystem::Horizon))
        .expect("horizon houses should work");
    assert_eq!(horizon.cusps.len(), 12);
    assert_eq!(horizon.cusps[9], horizon.angles.midheaven);
    assert_ne!(horizon.cusps[0], horizon.angles.ascendant);

    let apc = calculate_houses(&sample_request(HouseSystem::Apc)).expect("apc houses should work");
    assert_eq!(apc.cusps.len(), 12);
    assert_eq!(apc.cusps[0], apc.angles.ascendant);
    assert_eq!(apc.cusps[9], apc.angles.midheaven);
}

#[test]
fn placidus_beyond_bound_is_rejected_strictly() {
    // 80°N is above the polar circle; Placidus carries a 66° bound.
    let observer = ObserverLocation::new(
        Latitude::from_degrees(80.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let request = HouseRequest::new(
        Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        ),
        observer,
        HouseSystem::Placidus,
    );
    let err = calculate_houses(&request).expect_err("must reject beyond-bound latitude");
    assert_eq!(err.kind, crate::error::HouseErrorKind::InvalidLatitude);
}

#[test]
fn placidus_within_bound_is_accepted() {
    // 55°N is well within the 66° Placidus bound.
    let observer = ObserverLocation::new(
        Latitude::from_degrees(55.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let request = HouseRequest::new(
        Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        ),
        observer,
        HouseSystem::Placidus,
    );
    calculate_houses(&request).expect("in-band latitude must succeed");
}

#[test]
fn house_assignment_respects_wraparound() {
    let cusps = [
        Longitude::from_degrees(330.0),
        Longitude::from_degrees(0.0),
        Longitude::from_degrees(30.0),
        Longitude::from_degrees(60.0),
        Longitude::from_degrees(90.0),
        Longitude::from_degrees(120.0),
        Longitude::from_degrees(150.0),
        Longitude::from_degrees(180.0),
        Longitude::from_degrees(210.0),
        Longitude::from_degrees(240.0),
        Longitude::from_degrees(270.0),
        Longitude::from_degrees(300.0),
    ];
    let snapshot_angles = HouseAngles {
        ascendant: Longitude::from_degrees(15.0),
        descendant: Longitude::from_degrees(195.0),
        midheaven: Longitude::from_degrees(75.0),
        imum_coeli: Longitude::from_degrees(255.0),
    };
    let snapshot = HouseSnapshot {
        system: HouseSystem::Equal,
        instant: sample_request(HouseSystem::Equal).instant,
        observer: observer(),
        obliquity: Angle::from_degrees(23.4),
        angles: snapshot_angles,
        asc_mc: test_asc_mc(snapshot_angles),
        cusps: cusps.to_vec(),
    };

    assert_eq!(
        house_for_longitude(Longitude::from_degrees(359.0), &cusps),
        1
    );
    assert_eq!(house_for_longitude(Longitude::from_degrees(0.0), &cusps), 2);
    assert_eq!(
        snapshot.house_for_longitude(Longitude::from_degrees(15.0)),
        2
    );
    assert_eq!(
        snapshot.house_for_longitude(Longitude::from_degrees(29.999)),
        2
    );
    assert_eq!(
        snapshot.house_for_longitude(Longitude::from_degrees(30.0)),
        3
    );
    assert_eq!(
        snapshot.house_for_longitude(Longitude::from_degrees(44.999)),
        3
    );
}

#[test]
fn strict_policy_is_the_default() {
    assert_eq!(HighLatitudePolicy::default(), HighLatitudePolicy::Strict);
}

#[test]
fn se_compat_fallback_substitutes_porphyry_beyond_bound() {
    let observer = ObserverLocation::new(
        Latitude::from_degrees(80.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let request = HouseRequest::new(
        Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        ),
        observer.clone(),
        HouseSystem::Placidus,
    )
    .with_high_latitude_policy(HighLatitudePolicy::SwissEphemerisFallback);
    let snapshot =
        calculate_houses(&request).expect("SE-compat fallback must succeed beyond bound");

    // Same instant/observer under Porphyry directly:
    let porphyry = calculate_houses(&HouseRequest::new(
        Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        ),
        observer,
        HouseSystem::Porphyry,
    ))
    .expect("porphyry is defined at all latitudes");

    assert_eq!(
        snapshot.cusps, porphyry.cusps,
        "fallback cusps must equal Porphyry cusps"
    );
}

#[test]
fn se_compat_fallback_rejects_gauquelin_beyond_bound() {
    // Porphyry yields 12 quadrant cusps — a valid high-latitude substitute only
    // for 12-cusp systems. It cannot represent the 36-sector Gauquelin system,
    // and no validated high-latitude Gauquelin reference exists, so the SE-compat
    // fallback must reject cleanly with InvalidLatitude rather than emit a
    // dimensionally-invalid snapshot (which previously failed validation with a
    // confusing NumericalFailure "produced 12 cusps (expected 36)" error).
    let observer = ObserverLocation::new(
        Latitude::from_degrees(80.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let request = HouseRequest::new(
        Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        ),
        observer,
        HouseSystem::Gauquelin,
    )
    .with_high_latitude_policy(HighLatitudePolicy::SwissEphemerisFallback);
    let error = calculate_houses(&request)
        .expect_err("Gauquelin has no Porphyry-style high-latitude fallback");
    assert_eq!(error.kind, crate::error::HouseErrorKind::InvalidLatitude);
}
