use super::*;
use pleiades_types::{Angle, CustomHouseSystem, JulianDay, Latitude, TimeScale};

// --- shared test setup helpers ---

pub(super) fn observer() -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(0.0),
        Longitude::from_degrees(0.0),
        None,
    )
}

pub(super) fn sample_request(system: HouseSystem) -> HouseRequest {
    HouseRequest::new(
        Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        ),
        observer(),
        system,
    )
}

fn assert_close_degrees(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-12,
        "expected {expected}, got {actual}"
    );
}

fn test_asc_mc(angles: HouseAngles) -> AscMc {
    AscMc {
        ascendant: angles.ascendant,
        midheaven: angles.midheaven,
        descendant: angles.descendant,
        imum_coeli: angles.imum_coeli,
        armc: angles.midheaven,
        vertex: angles.ascendant,
        antivertex: angles.descendant,
        equatorial_ascendant: angles.ascendant,
        coascendant_koch: angles.ascendant,
        coascendant_munkasey: angles.ascendant,
        polar_ascendant: angles.descendant,
    }
}

// --- tests ---

#[test]
fn house_request_summary_line_reports_instant_observer_system_and_obliquity() {
    let request = sample_request(HouseSystem::WholeSign);
    assert_eq!(
        request.summary_line(),
        format!(
            "instant={}; observer={}; system={}; obliquity=auto",
            &request.instant, &request.observer, &request.system
        )
    );
    assert_eq!(request.to_string(), request.summary_line());

    let request_with_obliquity = request.with_obliquity(Angle::from_degrees(23.5));
    assert_eq!(
        request_with_obliquity.summary_line(),
        format!(
            "instant={}; observer={}; system={}; obliquity=23.5°",
            &request_with_obliquity.instant,
            &request_with_obliquity.observer,
            &request_with_obliquity.system
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
fn equal_houses_step_in_thirty_degree_increments() {
    let snapshot =
        calculate_houses(&sample_request(HouseSystem::Equal)).expect("equal houses should work");
    assert_eq!(snapshot.cusps.len(), 12);
    assert_eq!(
        snapshot.cusps[0].degrees(),
        snapshot.angles.ascendant.degrees()
    );
    assert_eq!(
        (snapshot.cusps[1].degrees() - snapshot.cusps[0].degrees()).rem_euclid(360.0),
        30.0
    );
    assert_eq!(
        (snapshot.cusps[3].degrees() - snapshot.cusps[0].degrees()).rem_euclid(360.0),
        90.0
    );
}

#[test]
fn whole_sign_houses_start_at_the_rising_sign_boundary() {
    let snapshot = calculate_houses(&sample_request(HouseSystem::WholeSign))
        .expect("whole sign houses should work");
    assert_eq!(snapshot.cusps[0].degrees() % 30.0, 0.0);
    assert!(snapshot.cusps[0].degrees() <= snapshot.angles.ascendant.degrees());
    assert_eq!(
        (snapshot.cusps[1].degrees() - snapshot.cusps[0].degrees()).rem_euclid(360.0),
        30.0
    );
}

#[test]
fn equal_midheaven_and_vehlow_variants_are_available() {
    let mc_snapshot = calculate_houses(&sample_request(HouseSystem::EqualMidheaven))
        .expect("equal (MC) houses should work");
    assert!(
        (mc_snapshot.cusps[9].degrees() - mc_snapshot.angles.midheaven.degrees()).abs() < 1.0e-12
    );
    assert_eq!(
        (mc_snapshot.cusps[1].degrees() - mc_snapshot.cusps[0].degrees()).rem_euclid(360.0),
        30.0
    );
    assert_eq!(
        (mc_snapshot.cusps[0].degrees() - mc_snapshot.angles.midheaven.degrees()).rem_euclid(360.0),
        90.0
    );

    let vehlow_snapshot =
        calculate_houses(&sample_request(HouseSystem::Vehlow)).expect("vehlow houses should work");
    assert_eq!(
        (vehlow_snapshot.angles.ascendant.degrees() - vehlow_snapshot.cusps[0].degrees())
            .rem_euclid(360.0),
        15.0
    );
    assert_eq!(
        (vehlow_snapshot.cusps[1].degrees() - vehlow_snapshot.cusps[0].degrees()).rem_euclid(360.0),
        30.0
    );
}

#[test]
fn sripati_midpoints_follow_porphyry_segments() {
    let snapshot = calculate_houses(&sample_request(HouseSystem::Sripati))
        .expect("sripati houses should work");
    let porphyry = calculate_houses(&sample_request(HouseSystem::Porphyry))
        .expect("porphyry houses should work");
    assert_eq!(
        snapshot.cusps[0],
        midpoint_longitude(porphyry.cusps[11], porphyry.cusps[0])
    );
    assert_eq!(
        snapshot.cusps[3],
        midpoint_longitude(porphyry.cusps[2], porphyry.cusps[3])
    );
    assert_eq!(
        snapshot.cusps[9],
        midpoint_longitude(porphyry.cusps[8], porphyry.cusps[9])
    );
}

#[test]
fn equal_aries_houses_start_at_zero_aries() {
    let snapshot = calculate_houses(&sample_request(HouseSystem::EqualAries))
        .expect("equal Aries houses should work");
    assert_eq!(snapshot.cusps[0].degrees(), 0.0);
    assert_eq!(snapshot.cusps[1].degrees(), 30.0);
    assert_eq!(snapshot.cusps[11].degrees(), 330.0);
}

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
fn topocentric_latitude_uses_geocentric_correction() {
    let sea_level = topocentric_latitude(45.0, None).expect("latitude should convert");
    let mountain = topocentric_latitude(45.0, Some(2_000.0)).expect("latitude should convert");

    assert!((sea_level.degrees() - 44.807_576).abs() < 1.0e-6);
    assert!(mountain.degrees() > sea_level.degrees());
}

#[test]
fn topocentric_latitude_rejects_non_finite_elevation() {
    let error =
        topocentric_latitude(45.0, Some(f64::NAN)).expect_err("non-finite elevation should fail");
    assert_eq!(error.kind, crate::error::HouseErrorKind::InvalidElevation);
    assert!(error
        .message
        .contains("observer elevation must be finite when provided"));
}

#[test]
fn topocentric_house_snapshot_matches_a_frozen_reference_point() {
    // Re-pinned after switching from GMST+mean-obliquity to GAST+true-obliquity
    // (equation of equinoxes applied to local sidereal time). The explicit
    // obliquity override is still honoured (true obliquity is only used for the
    // auto-computed default), so only GAST changes the values here.
    // Cusp 1 equals the Ascendant and cusp 10 the Midheaven, as required.
    let mut request = sample_request(HouseSystem::Topocentric);
    request.observer.latitude = Latitude::from_degrees(45.0);
    request.observer.longitude = Longitude::from_degrees(10.0);
    request.observer.elevation_m = Some(2_000.0);
    request.obliquity = Some(Angle::from_degrees(23.439_291_1));

    let snapshot = calculate_houses(&request).expect("topocentric houses should work");

    assert_eq!(snapshot.cusps.len(), 12);
    assert_close_degrees(snapshot.angles.ascendant.degrees(), 37.117_052_460_292_804);
    assert_close_degrees(snapshot.angles.descendant.degrees(), 217.117_052_460_292_8);
    assert_close_degrees(snapshot.angles.midheaven.degrees(), 288.893_467_921_746_2);
    assert_close_degrees(snapshot.angles.imum_coeli.degrees(), 108.893_467_921_746_21);
    assert_close_degrees(snapshot.cusps[0].degrees(), 37.117_052_460_292_804);
    assert_close_degrees(snapshot.cusps[1].degrees(), 67.530_700_716_702_61);
    assert_close_degrees(snapshot.cusps[9].degrees(), 288.893_467_921_746_2);
}

#[test]
fn topocentric_houses_share_placidus_angles_but_diverge_on_intermediate_cusps() {
    // The Topocentric (Polich-Page) system shares the Ascendant/Midheaven pair
    // with Placidus (cusps 1/4/7/10 are identical), but trisects the diurnal
    // arc with its own house-pole projection, so the intermediate cusps differ
    // from Placidus. This replaces the former (incorrect) invariant that
    // Topocentric equalled Placidus evaluated at the geocentric latitude; that
    // model disagreed with Swiss Ephemeris by thousands of arcseconds.
    let mut topocentric_request = sample_request(HouseSystem::Topocentric);
    topocentric_request.observer.latitude = Latitude::from_degrees(45.0);
    topocentric_request.observer.longitude = Longitude::from_degrees(10.0);
    topocentric_request.observer.elevation_m = Some(2_000.0);

    let topocentric =
        calculate_houses(&topocentric_request).expect("topocentric houses should work");
    assert_eq!(topocentric.cusps.len(), 12);

    let mut placidus_request = topocentric_request.clone();
    placidus_request.system = HouseSystem::Placidus;
    let placidus = calculate_houses(&placidus_request).expect("placidus houses should work");

    // Same angles (1/4/7/10), different intermediate cusps.
    assert_eq!(topocentric.angles, placidus.angles);
    for angle_cusp in [0usize, 3, 6, 9] {
        assert_eq!(topocentric.cusps[angle_cusp], placidus.cusps[angle_cusp]);
    }
    assert_ne!(topocentric.cusps[1], placidus.cusps[1]);
    assert_ne!(topocentric.cusps[10], placidus.cusps[10]);
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
fn regiomontanus_campanus_and_koch_reduce_to_sidereal_phase_spacing_on_the_equator() {
    let request =
        sample_request(HouseSystem::Regiomontanus).with_obliquity(Angle::from_degrees(0.0));
    let regiomontanus = calculate_houses(&request).expect("regiomontanus houses should work");
    let campanus = calculate_houses(
        &sample_request(HouseSystem::Campanus).with_obliquity(Angle::from_degrees(0.0)),
    )
    .expect("campanus houses should work");
    let koch = calculate_houses(
        &sample_request(HouseSystem::Koch).with_obliquity(Angle::from_degrees(0.0)),
    )
    .expect("koch houses should work");

    assert_eq!(regiomontanus.cusps.len(), 12);
    assert_eq!(campanus.cusps.len(), 12);
    assert_eq!(koch.cusps.len(), 12);
    assert_eq!(regiomontanus.cusps[0], regiomontanus.angles.ascendant);
    assert_eq!(regiomontanus.cusps[3], regiomontanus.angles.imum_coeli);
    assert_eq!(regiomontanus.cusps[6], regiomontanus.angles.descendant);
    assert_eq!(regiomontanus.cusps[9], regiomontanus.angles.midheaven);
    // At the equator with zero obliquity Campanus and Regiomontanus are algebraically
    // equivalent (both reduce to sidereal-phase spacing). They may differ at the level of
    // floating-point rounding order (~1e-13°), so compare with a tight numeric tolerance
    // rather than exact bit equality.
    for i in 0..12 {
        assert!(
            (regiomontanus.cusps[i].degrees() - campanus.cusps[i].degrees()).abs() < 1.0e-10,
            "regiomontanus cusp {} ({}) and campanus cusp {} ({}) should agree at equator+zero obliquity",
            i + 1,
            regiomontanus.cusps[i].degrees(),
            i + 1,
            campanus.cusps[i].degrees(),
        );
    }

    let sidereal_time = local_sidereal_time(request.instant, request.observer.longitude).degrees();
    for house in [2usize, 3, 5, 6, 8, 9, 11, 12] {
        let expected = Longitude::from_degrees(sidereal_time + house_phase(house));
        for (name, snapshot) in [
            ("regiomontanus", &regiomontanus),
            ("campanus", &campanus),
            ("koch", &koch),
        ] {
            assert!(
                (snapshot.cusps[house - 1].degrees() - expected.degrees()).abs() < 1.0e-10,
                "{name} house {house} should follow the equatorial sidereal-phase spacing"
            );
        }
    }
}

#[test]
fn carter_houses_follow_ascendant_centered_equatorial_spacing() {
    let request = sample_request(HouseSystem::Carter).with_obliquity(Angle::from_degrees(0.0));
    let snapshot = calculate_houses(&request).expect("carter houses should work");
    assert!((snapshot.cusps[0].degrees() - snapshot.angles.ascendant.degrees()).abs() < 1.0e-10);
    assert_eq!(
        (snapshot.cusps[1].degrees() - snapshot.cusps[0].degrees()).rem_euclid(360.0),
        30.0
    );
}

#[test]
fn meridian_and_axial_share_the_documented_equatorial_projection_layout() {
    let request = sample_request(HouseSystem::Meridian);
    let meridian = calculate_houses(&request).expect("meridian houses should work");
    let axial =
        calculate_houses(&sample_request(HouseSystem::Axial)).expect("axial houses should work");

    assert_eq!(meridian.cusps.len(), 12);
    assert_eq!(meridian.cusps, axial.cusps);
    assert_eq!(meridian.angles, axial.angles);

    let obliquity = meridian.obliquity.degrees().to_radians();
    let sidereal_time = local_sidereal_time(request.instant, request.observer.longitude);
    assert_eq!(
        meridian.cusps[9],
        ecliptic_longitude_from_ra(sidereal_time.degrees(), obliquity)
    );
    assert_eq!(
        meridian.cusps[0],
        ecliptic_longitude_from_ra(sidereal_time.degrees() + 90.0, obliquity)
    );
}

/// Morinus is a distinct system from Meridian/Axial.  It projects equatorial
/// arc endpoints (at RA = RAMC + 90 + n*30°) onto the ecliptic using the
/// full spherical rotation formula for dec = 0, whereas Meridian/Axial use the
/// inverse ecliptic-to-equatorial formula.  The two systems therefore produce
/// different cusp sets.
#[test]
fn morinus_is_distinct_from_meridian_and_produces_12_cusps() {
    let meridian = calculate_houses(&sample_request(HouseSystem::Meridian))
        .expect("meridian houses should work");
    let morinus = calculate_houses(&sample_request(HouseSystem::Morinus))
        .expect("morinus houses should work");

    assert_eq!(morinus.cusps.len(), 12);
    // Morinus and Meridian must NOT be identical (they use different ecliptic
    // projection formulas and would only agree at zero obliquity).
    assert_ne!(
        morinus.cusps, meridian.cusps,
        "Morinus and Meridian should produce different cusp sets at non-zero obliquity"
    );
}

/// Swiss Ephemeris external-reference anchor for the Morinus house system.
///
/// Fixture c1_lat40: JD=2451545.0 (J2000.0), lat=40°N, lon=0°E.
/// SE reference cusps come straight from the houses-corpus
/// (`pleiades-validate/data/houses-corpus/cusps.csv`, system_code=Morinus).
/// Tolerance is 1 arcsec; actual residuals are ~0.02 arcsec after switching
/// to GAST + true obliquity.
#[test]
fn morinus_cusps_match_swiss_ephemeris_corpus_within_120_arcsec() {
    let circ_diff_arcsec = |a: f64, b: f64| -> f64 {
        let diff = (a - b).rem_euclid(360.0);
        let signed = if diff > 180.0 { diff - 360.0 } else { diff };
        signed.abs() * 3600.0
    };
    let tolerance_arcsec = 1.0_f64;

    // c1_lat40 SE corpus Morinus row, cusps c1..c12.
    let se_morinus: [f64; 12] = [
        9.611_088,
        38.040_522,
        68.849_424,
        101.373_900,
        132.906_648,
        161.960_854,
        189.611_088,
        218.040_522,
        248.849_424,
        281.373_900,
        312.906_648,
        341.960_854,
    ];

    let request = HouseRequest::new(
        Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        ),
        ObserverLocation::new(
            Latitude::from_degrees(40.0),
            Longitude::from_degrees(0.0),
            None,
        ),
        HouseSystem::Morinus,
    );
    let snapshot = calculate_houses(&request).expect("Morinus houses should compute");

    for (index, &expected) in se_morinus.iter().enumerate() {
        let diff = circ_diff_arcsec(snapshot.cusps[index].degrees(), expected);
        assert!(
            diff < tolerance_arcsec,
            "Morinus cusp {} = {:.6}° differs from SE {expected:.6}° by {diff:.1} arcsec (limit {tolerance_arcsec})",
            index + 1,
            snapshot.cusps[index].degrees(),
        );
    }
}

/// Morinus is latitude-independent: the same RAMC and obliquity produce
/// identical cusp sets regardless of geographic latitude.
///
/// Verified against the SE corpus rows c0_lat00, c1_lat40, c2_lat55, c3_lat66
/// for JD=2451545 (J2000.0), lon=0°E, which all carry identical Morinus cusps.
#[test]
fn morinus_cusps_are_latitude_invariant() {
    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        pleiades_types::TimeScale::Tt,
    );
    let lon = Longitude::from_degrees(0.0);

    let latitudes = [0.0_f64, 40.0, 55.0, 66.0];
    let snapshots: Vec<_> = latitudes
        .iter()
        .map(|&lat| {
            calculate_houses(&HouseRequest::new(
                instant,
                ObserverLocation::new(Latitude::from_degrees(lat), lon, None),
                HouseSystem::Morinus,
            ))
            .expect("Morinus houses should compute at any latitude")
        })
        .collect();

    // All snapshots must produce bit-identical cusp sets.
    for (i, snapshot) in snapshots[1..].iter().enumerate() {
        assert_eq!(
            snapshot.cusps,
            snapshots[0].cusps,
            "Morinus cusps at lat={} must be identical to cusps at lat=0 (same RAMC and obliquity)",
            latitudes[i + 1],
        );
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

/// SE-anchored regression for the Horizon ('H') azimuth convention fix.
///
/// Before the fix `horizon_houses` disagreed with Swiss-Ephemeris by ~55–115°
/// per cusp (missing the +180° post-rotation, an extra +90° azimuth quarter-turn
/// via `ascendant_for`, and a `>= 0` latitude branch that flipped cusp 1 at the
/// equator). The fixtures below are the exact SE 2.10.03 'H' reference rows from
/// the house corpus (grep ',Horizon,' cusps.csv), covering the equator, mid-
/// latitudes, the lat-66° support bound, and an elevated off-J2000 epoch.
#[test]
fn horizon_houses_match_swiss_ephemeris_reference() {
    // (lat, lon, jd, [c1..c12])
    let fixtures: [(f64, f64, f64, [f64; 12]); 5] = [
        (
            0.0,
            0.0,
            2451545.0,
            [
                180.0, 131.019922, 111.942372, 99.611088, 86.327570, 62.021662, 0.0, 311.019922,
                291.942372, 279.611088, 266.327570, 242.021662,
            ],
        ),
        (
            40.0,
            0.0,
            2451545.0,
            [
                7.512627, 40.957940, 71.860911, 99.611088, 126.477720, 155.300954, 187.512627,
                220.957940, 251.860911, 279.611088, 306.477720, 335.300954,
            ],
        ),
        (
            55.0,
            0.0,
            2451545.0,
            [
                8.738716, 39.436172, 69.870644, 99.611088, 128.927706, 158.489993, 188.738716,
                219.436172, 249.870644, 279.611088, 308.927706, 338.489993,
            ],
        ),
        (
            66.0,
            0.0,
            2451545.0,
            [
                9.545351, 39.500052, 69.532860, 99.611088, 129.656324, 159.623396, 189.545351,
                219.500052, 249.532860, 279.611088, 309.656324, 339.623396,
            ],
        ),
        (
            40.0,
            30.0,
            2433283.0,
            [
                29.044129, 64.514574, 99.194140, 128.147116, 153.317114, 178.927456, 209.044129,
                244.514574, 279.194140, 308.147116, 333.317114, 358.927456,
            ],
        ),
    ];
    let mut worst = 0.0f64;
    for (lat, lon, jd, want) in fixtures {
        let req = HouseRequest::new(
            Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
            ObserverLocation::new(
                Latitude::from_degrees(lat),
                Longitude::from_degrees(lon),
                None,
            ),
            HouseSystem::Horizon,
        );
        let snap = calculate_houses(&req).expect("horizon houses should compute");
        // MC must be the true meridian; cusp 10 is anchored to it directly.
        assert_eq!(snap.cusps[9], snap.angles.midheaven);
        for (i, (cusp, expected)) in snap.cusps.iter().zip(want.iter()).enumerate() {
            let got = cusp.degrees();
            let mut diff = (got - expected).rem_euclid(360.0);
            if diff > 180.0 {
                diff -= 360.0;
            }
            let arcsec = diff.abs() * 3600.0;
            if arcsec > worst {
                worst = arcsec;
            }
            // Tight per-cusp tolerance well inside the GreatCircle 5.0″ ceiling.
            assert!(
                arcsec < 1.0,
                "Horizon lat={lat} cusp {} = {got:.6}° differs from SE {expected:.6}° by {arcsec:.4}\u{2033}",
                i + 1,
            );
        }
    }
    // Whole-corpus Horizon worst residual stays far below the GreatCircle ceiling.
    assert!(
        worst < 1.0,
        "worst Horizon residual {worst:.4}\u{2033} exceeded 1\u{2033}"
    );
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
fn krusinski_pisa_goelzer_release_system_preserves_the_documented_opposite_pairs() {
    let snapshot = calculate_houses(&sample_request(HouseSystem::KrusinskiPisaGoelzer))
        .expect("krusinski-pisa-goelzer houses should work");

    assert_eq!(snapshot.cusps.len(), 12);
    assert!(snapshot.cusps.iter().all(|cusp| cusp.degrees().is_finite()));

    for index in 0..6 {
        let diff = (snapshot.cusps[index + 6].degrees() - snapshot.cusps[index].degrees())
            .rem_euclid(360.0);
        assert!(
            (diff - 180.0).abs() < 1.0e-10,
            "krusinski cusp {} and cusp {} should be opposite (diff {diff:.15} should be ~180°)",
            index + 1,
            index + 7,
        );
    }
}

#[test]
fn gauquelin_release_system_exposes_thirty_six_sectors() {
    let snapshot = calculate_houses(&sample_request(HouseSystem::Gauquelin))
        .expect("gauquelin sectors should work");
    assert_eq!(snapshot.cusps.len(), 36);
    assert_eq!(snapshot.cusps[0], snapshot.angles.ascendant);
    assert_eq!(snapshot.cusps[9], snapshot.angles.midheaven);
    assert_eq!(
        snapshot.cusps[18],
        longitude_opposite(snapshot.angles.ascendant)
    );
    assert_eq!(
        snapshot.cusps[27],
        longitude_opposite(snapshot.angles.midheaven)
    );
    assert_eq!(snapshot.cusp(36), Some(snapshot.cusps[35]));
    // Gauquelin sectors are a Placidus-family semi-arc division, so the lower
    // hemisphere is the exact antipode of the upper hemisphere:
    // s(k + 18) = opposite(s(k)). (Numeric SE agreement is asserted separately
    // in `gauquelin_sectors_match_swiss_ephemeris_reference`.)
    for k in 0..18 {
        let upper = snapshot.cusps[k].degrees();
        let lower = snapshot.cusps[k + 18].degrees();
        let diff = (((upper + 180.0) - lower + 180.0).rem_euclid(360.0)) - 180.0;
        assert!(
            diff.abs() < 1.0e-9,
            "Gauquelin sector {} should be the antipode of sector {}",
            k + 19,
            k + 1
        );
    }
}

#[test]
fn gauquelin_sectors_match_swiss_ephemeris_reference() {
    // Ground-truth Swiss-Ephemeris 36-sector values (sectors.csv fixtures):
    // (jd_ut, lat_deg, lon_deg, [s1..s36]).
    // Gauquelin sectors are a Placidus-family semi-arc division (each diurnal/
    // nocturnal quadrant split into ninths), NOT a longitude lerp. These values
    // exercise the equator, mid-latitudes, and the near-polar (66°) regime.
    struct Fixture {
        id: &'static str,
        jd: f64,
        lat: f64,
        lon: f64,
        sectors: [f64; 36],
    }
    let fixtures = [
        Fixture {
            id: "c0_lat00",
            jd: 2451545.0,
            lat: 0.0,
            lon: 0.0,
            sectors: [
                11.373900, 0.498173, 349.616833, 338.849424, 328.295170, 318.017909, 308.040522,
                298.347719, 288.893685, 279.611088, 270.419362, 261.231657, 251.960854, 242.525543,
                232.856860, 222.906648, 212.656467, 202.125490, 191.373900, 180.498173, 169.616833,
                158.849424, 148.295170, 138.017909, 128.040522, 118.347719, 108.893685, 99.611088,
                90.419362, 81.231657, 71.960854, 62.525543, 52.856860, 42.906648, 32.656467,
                22.125490,
            ],
        },
        Fixture {
            id: "c1_lat40",
            jd: 2451545.0,
            lat: 40.0,
            lon: 0.0,
            sectors: [
                17.706103, 0.736222, 345.587073, 332.464496, 321.131729, 311.221449, 302.382578,
                294.320768, 286.796477, 279.611088, 272.591552, 265.575907, 258.399152, 250.877889,
                242.791680, 233.858979, 223.707487, 211.848972, 197.706103, 180.736222, 165.587073,
                152.464496, 141.131729, 131.221449, 122.382578, 114.320768, 106.796477, 99.611088,
                92.591552, 85.575907, 78.399152, 70.877889, 62.791680, 53.858979, 43.707487,
                31.848972,
            ],
        },
        Fixture {
            id: "c2_lat55",
            jd: 2451545.0,
            lat: 55.0,
            lon: 0.0,
            sectors: [
                28.505186, 1.107822, 340.254713, 325.231349, 313.955164, 305.003978, 297.529337,
                291.013310, 285.119432, 279.611088, 274.305089, 269.042114, 263.663645, 257.987911,
                251.775970, 244.671608, 236.075863, 224.846914, 208.505186, 181.107822, 160.254713,
                145.231349, 133.955164, 125.003978, 117.529337, 111.013310, 105.119432, 99.611088,
                94.305089, 89.042114, 83.663645, 77.987911, 71.775970, 64.671608, 56.075863,
                44.846914,
            ],
        },
        Fixture {
            id: "c3_lat66",
            jd: 2451545.0,
            lat: 66.0,
            lon: 0.0,
            sectors: [
                87.195607, 3.702693, 318.826001, 303.196619, 295.102092, 290.005092, 286.407601,
                283.668075, 281.463357, 279.611088, 277.998875, 276.551926, 275.216522, 273.950574,
                272.717332, 271.480026, 270.195365, 268.802302, 267.195607, 183.702693, 138.826001,
                123.196619, 115.102092, 110.005092, 106.407601, 103.668075, 101.463357, 99.611088,
                97.998875, 96.551926, 95.216522, 93.950574, 92.717332, 91.480026, 90.195365,
                88.802302,
            ],
        },
        Fixture {
            id: "c4_lat40_e2",
            jd: 2433283.0,
            lat: 40.0,
            lon: 30.0,
            sectors: [
                60.830331, 45.921003, 30.402853, 15.112631, 0.775651, 347.777129, 336.178607,
                325.848219, 316.577373, 308.147116, 300.355307, 293.022440, 285.987810, 279.101589,
                272.214688, 265.166046, 257.765393, 249.767458, 240.830331, 225.921003, 210.402853,
                195.112631, 180.775651, 167.777129, 156.178607, 145.848219, 136.577373, 128.147116,
                120.355307, 113.022440, 105.987810, 99.101589, 92.214688, 85.166046, 77.765393,
                69.767458,
            ],
        },
    ];

    // Sector-family ceiling is 2.0″ (see thresholds.rs); the test tolerance is
    // the same so the gate and this unit test agree (measured max is ~0.49″).
    let tolerance_arcsec = 2.0;
    let mut overall_max = 0.0_f64;
    for fx in &fixtures {
        let request = HouseRequest::new(
            Instant::new(
                pleiades_types::JulianDay::from_days(fx.jd),
                pleiades_types::TimeScale::Tt,
            ),
            ObserverLocation::new(
                Latitude::from_degrees(fx.lat),
                Longitude::from_degrees(fx.lon),
                None,
            ),
            HouseSystem::Gauquelin,
        );
        let snapshot = calculate_houses(&request).expect("gauquelin sectors should compute");
        assert_eq!(snapshot.cusps.len(), 36);
        for (i, &want) in fx.sectors.iter().enumerate() {
            let got = snapshot.cusps[i].degrees();
            let resid = (((got - want + 180.0).rem_euclid(360.0)) - 180.0).abs() * 3600.0;
            overall_max = overall_max.max(resid);
            assert!(
                resid <= tolerance_arcsec,
                "{}: sector {} got {got:.6} want {want:.6} resid {resid:.3}\"",
                fx.id,
                i + 1
            );
        }
    }
    eprintln!("gauquelin max residual vs SE = {overall_max:.4} arcsec");
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

/// Swiss Ephemeris external-reference anchor test.
///
/// Fixture: JD=2451545.0 (J2000.0), lat=40°N, lon=0°E (Equal house system).
/// SE reference values: ASC=17.706103°, MC=279.611088°.
/// Tolerance: 1 arcsec; actual residuals are ~0.04 arcsec after switching to
/// GAST + true obliquity (equation of equinoxes applied).
#[test]
fn equal_house_angles_match_swiss_ephemeris_corpus_within_120_arcsec() {
    let observer = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let request = HouseRequest::new(
        Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        ),
        observer,
        HouseSystem::Equal,
    );
    let snapshot = calculate_houses(&request).expect("Equal houses should work");

    let se_asc = 17.706_103_f64;
    let se_mc = 279.611_088_f64;
    let tolerance_arcsec = 1.0_f64;

    let circ_diff_arcsec = |a: f64, b: f64| -> f64 {
        let diff = (a - b).rem_euclid(360.0);
        let signed = if diff > 180.0 { diff - 360.0 } else { diff };
        signed.abs() * 3600.0
    };

    let asc_diff = circ_diff_arcsec(snapshot.angles.ascendant.degrees(), se_asc);
    let mc_diff = circ_diff_arcsec(snapshot.angles.midheaven.degrees(), se_mc);

    assert!(
        asc_diff < tolerance_arcsec,
        "ASC {:.6}° differs from SE {se_asc:.6}° by {asc_diff:.1} arcsec (limit {tolerance_arcsec})",
        snapshot.angles.ascendant.degrees(),
    );
    assert!(
        mc_diff < tolerance_arcsec,
        "MC {:.6}° differs from SE {se_mc:.6}° by {mc_diff:.1} arcsec (limit {tolerance_arcsec})",
        snapshot.angles.midheaven.degrees(),
    );
}

/// Swiss Ephemeris external-reference anchor for the Placidus and Topocentric
/// intermediate cusps.
///
/// Fixture c1_lat40: JD=2451545.0 (J2000.0), lat=40°N, lon=0°E.
/// SE reference cusps come straight from the houses-corpus
/// (`pleiades-validate/data/houses-corpus/cusps.csv`). Tolerance 1 arcsec;
/// actual residuals are ~0.04 arcsec after switching to GAST + true obliquity.
#[test]
fn placidus_and_topocentric_cusps_match_swiss_ephemeris_corpus_within_120_arcsec() {
    let circ_diff_arcsec = |a: f64, b: f64| -> f64 {
        let diff = (a - b).rem_euclid(360.0);
        let signed = if diff > 180.0 { diff - 360.0 } else { diff };
        signed.abs() * 3600.0
    };
    let tolerance_arcsec = 1.0_f64;

    // c1_lat40 SE corpus rows, cusps c1..c12.
    let se_placidus: [f64; 12] = [
        17.706_103,
        53.858_979,
        78.399_152,
        99.611_088,
        122.382_578,
        152.464_496,
        197.706_103,
        233.858_979,
        258.399_152,
        279.611_088,
        302.382_578,
        332.464_496,
    ];
    let se_topocentric: [f64; 12] = [
        17.706_103,
        53.759_507,
        78.270_701,
        99.611_088,
        122.465_089,
        152.483_265,
        197.706_103,
        233.759_507,
        258.270_701,
        279.611_088,
        302.465_089,
        332.483_265,
    ];

    for (system, se) in [
        (HouseSystem::Placidus, se_placidus),
        (HouseSystem::Topocentric, se_topocentric),
    ] {
        let request = HouseRequest::new(
            Instant::new(
                pleiades_types::JulianDay::from_days(2_451_545.0),
                pleiades_types::TimeScale::Tt,
            ),
            ObserverLocation::new(
                Latitude::from_degrees(40.0),
                Longitude::from_degrees(0.0),
                None,
            ),
            system.clone(),
        );
        let snapshot = calculate_houses(&request).expect("houses should compute");

        for (index, &expected) in se.iter().enumerate() {
            let diff = circ_diff_arcsec(snapshot.cusps[index].degrees(), expected);
            assert!(
                diff < tolerance_arcsec,
                "{system:?} cusp {} = {:.6}° differs from SE {expected:.6}° by {diff:.1} arcsec (limit {tolerance_arcsec})",
                index + 1,
                snapshot.cusps[index].degrees(),
            );
        }
    }
}

/// Swiss Ephemeris external-reference anchor for the Koch (GOH / "birthplace")
/// intermediate cusps.
///
/// Fixture c1_lat40: JD=2451545.0 (J2000.0), lat=40°N, lon=0°E.
/// SE reference cusps come straight from the houses-corpus
/// (`pleiades-validate/data/houses-corpus/cusps.csv`). Tolerance 1 arcsec;
/// actual residuals are ~0.03 arcsec after switching to GAST + true obliquity.
#[test]
fn koch_cusps_match_swiss_ephemeris_corpus_within_120_arcsec() {
    let circ_diff_arcsec = |a: f64, b: f64| -> f64 {
        let diff = (a - b).rem_euclid(360.0);
        let signed = if diff > 180.0 { diff - 360.0 } else { diff };
        signed.abs() * 3600.0
    };
    let tolerance_arcsec = 1.0_f64;

    // c1_lat40 SE corpus Koch row, cusps c1..c12.
    let se_koch: [f64; 12] = [
        17.706_103,
        51.954_052,
        78.286_109,
        99.611_088,
        125.345_306,
        158.845_358,
        197.706_103,
        231.954_052,
        258.286_109,
        279.611_088,
        305.345_306,
        338.845_358,
    ];

    let request = HouseRequest::new(
        Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        ),
        ObserverLocation::new(
            Latitude::from_degrees(40.0),
            Longitude::from_degrees(0.0),
            None,
        ),
        HouseSystem::Koch,
    );
    let snapshot = calculate_houses(&request).expect("Koch houses should compute");

    for (index, &expected) in se_koch.iter().enumerate() {
        let diff = circ_diff_arcsec(snapshot.cusps[index].degrees(), expected);
        assert!(
            diff < tolerance_arcsec,
            "Koch cusp {} = {:.6}° differs from SE {expected:.6}° by {diff:.1} arcsec (limit {tolerance_arcsec})",
            index + 1,
            snapshot.cusps[index].degrees(),
        );
    }
}

/// Swiss Ephemeris external-reference anchor for the Campanus (prime-vertical)
/// intermediate cusps.
///
/// Fixture c1_lat40: JD=2451545.0 (J2000.0), lat=40°N, lon=0°E.
/// SE reference cusps come straight from the houses-corpus
/// (`pleiades-validate/data/houses-corpus/cusps.csv`). Tolerance 1 arcsec;
/// actual residuals are ~0.04 arcsec after switching to GAST + true obliquity.
#[test]
fn campanus_cusps_match_swiss_ephemeris_corpus_within_120_arcsec() {
    let circ_diff_arcsec = |a: f64, b: f64| -> f64 {
        let diff = (a - b).rem_euclid(360.0);
        let signed = if diff > 180.0 { diff - 360.0 } else { diff };
        signed.abs() * 3600.0
    };
    let tolerance_arcsec = 1.0_f64;

    // c1_lat40 SE corpus Campanus row, cusps c1..c12.
    let se_campanus: [f64; 12] = [
        17.706_103,
        64.352_912,
        85.435_838,
        99.611_088,
        114.834_455,
        141.116_623,
        197.706_103,
        244.352_912,
        265.435_838,
        279.611_088,
        294.834_455,
        321.116_623,
    ];

    let request = HouseRequest::new(
        Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        ),
        ObserverLocation::new(
            Latitude::from_degrees(40.0),
            Longitude::from_degrees(0.0),
            None,
        ),
        HouseSystem::Campanus,
    );
    let snapshot = calculate_houses(&request).expect("Campanus houses should compute");

    for (index, &expected) in se_campanus.iter().enumerate() {
        let diff = circ_diff_arcsec(snapshot.cusps[index].degrees(), expected);
        assert!(
            diff < tolerance_arcsec,
            "Campanus cusp {} = {:.6}° differs from SE {expected:.6}° by {diff:.1} arcsec (limit {tolerance_arcsec})",
            index + 1,
            snapshot.cusps[index].degrees(),
        );
    }
}

/// Swiss Ephemeris external-reference anchor for the Alcabitius intermediate
/// cusps.
///
/// Fixture c1_lat40: JD=2451545.0 (J2000.0), lat=40°N, lon=0°E.
/// SE reference cusps come straight from the houses-corpus
/// (`pleiades-validate/data/houses-corpus/cusps.csv`). Tolerance 1 arcsec;
/// actual residuals are ~0.01 arcsec after switching to GAST + true obliquity.
#[test]
fn alcabitius_cusps_match_swiss_ephemeris_corpus_within_120_arcsec() {
    let circ_diff_arcsec = |a: f64, b: f64| -> f64 {
        let diff = (a - b).rem_euclid(360.0);
        let signed = if diff > 180.0 { diff - 360.0 } else { diff };
        signed.abs() * 3600.0
    };
    let tolerance_arcsec = 1.0_f64;

    // c1_lat40 SE corpus Alcabitius row, cusps c1..c12.
    let se_alcabitius: [f64; 12] = [
        17.706_103,
        46.835_395,
        73.785_097,
        99.611_088,
        129.969_119,
        163.041_881,
        197.706_103,
        226.835_395,
        253.785_097,
        279.611_088,
        309.969_119,
        343.041_881,
    ];

    let request = HouseRequest::new(
        Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        ),
        ObserverLocation::new(
            Latitude::from_degrees(40.0),
            Longitude::from_degrees(0.0),
            None,
        ),
        HouseSystem::Alcabitius,
    );
    let snapshot = calculate_houses(&request).expect("Alcabitius houses should compute");

    for (index, &expected) in se_alcabitius.iter().enumerate() {
        let diff = circ_diff_arcsec(snapshot.cusps[index].degrees(), expected);
        assert!(
            diff < tolerance_arcsec,
            "Alcabitius cusp {} = {:.6}° differs from SE {expected:.6}° by {diff:.1} arcsec (limit {tolerance_arcsec})",
            index + 1,
            snapshot.cusps[index].degrees(),
        );
    }
}

/// Swiss Ephemeris external-reference anchor for the Alcabitius intermediate
/// cusps at a higher latitude.
///
/// Fixture c2_lat55: JD=2451545.0 (J2000.0), lat=55°N, lon=0°E.
/// SE reference cusps come straight from the houses-corpus
/// (`pleiades-validate/data/houses-corpus/cusps.csv`). Tolerance 1 arcsec;
/// actual residuals are ~0.06 arcsec after switching to GAST + true obliquity.
#[test]
fn alcabitius_cusps_c2_lat55_match_swiss_ephemeris_corpus_within_1_arcsec() {
    let circ_diff_arcsec = |a: f64, b: f64| -> f64 {
        let diff = (a - b).rem_euclid(360.0);
        let signed = if diff > 180.0 { diff - 360.0 } else { diff };
        signed.abs() * 3600.0
    };
    let tolerance_arcsec = 1.0_f64;

    // c2_lat55 SE corpus Alcabitius row, cusps c1..c12.
    let se_alcabitius: [f64; 12] = [
        28.505_186,
        53.528_350,
        76.929_561,
        99.611_088,
        133.334_056,
        170.360_534,
        208.505_186,
        233.528_350,
        256.929_561,
        279.611_088,
        313.334_056,
        350.360_534,
    ];

    let request = HouseRequest::new(
        Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        ),
        ObserverLocation::new(
            Latitude::from_degrees(55.0),
            Longitude::from_degrees(0.0),
            None,
        ),
        HouseSystem::Alcabitius,
    );
    let snapshot = calculate_houses(&request).expect("Alcabitius houses should compute");

    for (index, &expected) in se_alcabitius.iter().enumerate() {
        let diff = circ_diff_arcsec(snapshot.cusps[index].degrees(), expected);
        assert!(
            diff < tolerance_arcsec,
            "Alcabitius (c2_lat55) cusp {} = {:.6}° differs from SE {expected:.6}° by {diff:.1} arcsec (limit {tolerance_arcsec})",
            index + 1,
            snapshot.cusps[index].degrees(),
        );
    }
}

#[test]
fn chart_points_from_armc_mc_is_analytic_at_cardinal_armc() {
    use pleiades_types::{Angle, Latitude, Longitude};
    // With obliquity ε, MC longitude satisfies tan(λ_MC)=tan(ARMC)/cos(ε);
    // at ARMC = 0/90/180/270 the MC equals the ARMC exactly.
    let obl = Angle::from_degrees(23.4392911);
    for armc in [0.0_f64, 90.0, 180.0, 270.0] {
        let pts = chart_points_from_armc(
            Longitude::from_degrees(armc),
            Latitude::from_degrees(40.0),
            obl,
        )
        .expect("defined at 40N");
        let diff = (pts.midheaven.degrees() - armc).rem_euclid(360.0);
        let diff = diff.min(360.0 - diff);
        assert!(diff < 1e-6, "ARMC {armc}: MC {}", pts.midheaven.degrees());
    }
}

#[test]
fn chart_points_from_armc_mc_obliquity_coefficient_is_pinned() {
    use pleiades_types::{Angle, Latitude, Longitude};
    // Independent check of the MC's obliquity dependence at a NON-cardinal ARMC,
    // where the cardinal-value test cannot distinguish cos(ε) from any other
    // coefficient. The MC satisfies tan(λ_MC)·cos(ε) = tan(ARMC); a regression
    // swapping cos(ε) for sin(ε) (or any wrong coefficient) breaks this identity.
    let eps = Angle::from_degrees(23.4392911);
    for armc in [37.0_f64, 123.4, 210.0, 316.7] {
        let pts = chart_points_from_armc(
            Longitude::from_degrees(armc),
            Latitude::from_degrees(40.0),
            eps,
        )
        .expect("defined at 40N");
        let lhs = pts.midheaven.degrees().to_radians().tan() * eps.degrees().to_radians().cos();
        let rhs = armc.to_radians().tan();
        assert!(
            (lhs - rhs).abs() < 1e-9,
            "ARMC {armc}: tan(MC)·cos(ε)={lhs} vs tan(ARMC)={rhs}"
        );
    }
}

#[test]
fn chart_points_invariants_hold() {
    use pleiades_types::{Angle, Latitude, Longitude};
    let pts = chart_points_from_armc(
        Longitude::from_degrees(123.4),
        Latitude::from_degrees(51.5),
        Angle::from_degrees(23.4392911),
    )
    .expect("defined at 51.5N");
    let opp = |a: f64, b: f64| {
        let d = (a - b).rem_euclid(360.0);
        (d - 180.0).abs() < 1e-6
    };
    assert!(opp(pts.ascendant.degrees(), pts.descendant.degrees()));
    assert!(opp(pts.midheaven.degrees(), pts.imum_coeli.degrees()));
    assert!(opp(pts.vertex.degrees(), pts.antivertex.degrees()));
    for p in [
        pts.armc,
        pts.vertex,
        pts.equatorial_ascendant,
        pts.coascendant_koch,
        pts.coascendant_munkasey,
        pts.polar_ascendant,
    ] {
        assert!(
            (0.0..360.0).contains(&p.degrees()),
            "unnormalized {}",
            p.degrees()
        );
    }
    // ARMC round-trips the input.
    let d = (pts.armc.degrees() - 123.4).rem_euclid(360.0);
    assert!(d.min(360.0 - d) < 1e-9);
}

#[test]
fn chart_points_uses_true_obliquity_by_default() {
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let observer = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(-74.0),
        None,
    );
    let inst = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let pts = chart_points(inst, &observer, None).expect("defined");
    // Ascendant matches the value derive_angles produces for the same inputs.
    let req = HouseRequest::new(inst, observer.clone(), HouseSystem::Placidus);
    let snap = calculate_houses(&req).expect("houses");
    assert!((pts.ascendant.degrees() - snap.angles.ascendant.degrees()).abs() < 1e-9);
    assert!((pts.midheaven.degrees() - snap.angles.midheaven.degrees()).abs() < 1e-9);
}

#[test]
fn house_snapshot_carries_asc_mc_consistent_with_angles() {
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let observer = ObserverLocation::new(
        Latitude::from_degrees(48.85),
        Longitude::from_degrees(2.35),
        None,
    );
    let req = HouseRequest::new(
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        observer,
        HouseSystem::Placidus,
    );
    let snap = calculate_houses(&req).expect("houses");
    assert_eq!(snap.asc_mc.ascendant, snap.angles.ascendant);
    assert_eq!(snap.asc_mc.midheaven, snap.angles.midheaven);
    assert!((0.0..360.0).contains(&snap.asc_mc.vertex.degrees()));
}

#[test]
fn porphyry_fallback_snapshot_carries_consistent_asc_mc() {
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    // Latitude 75° is beyond Placidus's polar bound, so with the SE fallback
    // policy calculate_houses takes the early-return Porphyry-fallback branch.
    let observer = ObserverLocation::new(
        Latitude::from_degrees(75.0),
        Longitude::from_degrees(10.0),
        None,
    );
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let req = HouseRequest::new(instant, observer.clone(), HouseSystem::Placidus)
        .with_high_latitude_policy(HighLatitudePolicy::SwissEphemerisFallback);

    let snap = calculate_houses(&req).expect("Porphyry fallback should produce a snapshot");

    // It really took the fallback: Porphyry yields 12 quadrant cusps.
    assert_eq!(snap.cusps.len(), 12);

    // The fallback site's asc_mc must equal an independent recomputation.
    let expected = asc_mc_from(
        local_sidereal_time(instant, observer.longitude).degrees(),
        observer.latitude.degrees(),
        snap.obliquity.degrees(),
    )
    .expect("asc_mc_from");
    assert_eq!(snap.asc_mc, expected);
}
