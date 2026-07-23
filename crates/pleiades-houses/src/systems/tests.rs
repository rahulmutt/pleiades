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

// --- FU-9 Foundation: shared geometry primitives ---

#[test]
fn spherical_cotrans_matches_independent_x_axis_rotation() {
    // Independent reference (houses-reference.py `spherical_cotrans`): a pure
    // x-axis rotation of (lon,lat,r) -> Cartesian -> rotate by `angle` -> back.
    // Geometry avoids every degeneracy (no 0°/90° angle, non-unit radius) so
    // each `*`/`+` term is observable. Cross-validated to 1e-12 vs the crate.
    let mut coord = [40.0_f64, 25.0, 2.0];
    spherical_cotrans(&mut coord, 15.0);
    assert!(
        (coord[0] - 44.070_120_506_012).abs() < 1e-9,
        "lon' = {}",
        coord[0]
    );
    assert!(
        (coord[1] - 14.918_178_485_226).abs() < 1e-9,
        "lat' = {}",
        coord[1]
    );
    assert!(
        (coord[2] - 2.000_000_000_000).abs() < 1e-9,
        "r' = {}",
        coord[2]
    );
}

#[test]
fn asc2_matches_independent_swehouse_kernel() {
    // Independent reference (houses-reference.py `asc2`, swehouse.c Asc2) at
    // pole height 52°, obliquity sine/cosine. Four x values, one per asc1
    // quadrant; each takes the normal atan branch. Cross-validated to 1e-12.
    let eps = 23.4366_f64;
    let (sine, cose) = (eps.to_radians().sin(), eps.to_radians().cos());
    let cases = [
        (30.0_f64, 60.273_411_210_075),
        (120.0, 138.177_359_444_927),
        (210.0, 20.983_664_735_370),
        (300.0, 86.674_198_092_798),
    ];
    for (x, expected) in cases {
        let got = asc2(x, 52.0, sine, cose);
        assert!((got - expected).abs() < 1e-9, "asc2({x}) = {got}");
    }
}

#[test]
fn asc1_dispatches_each_quadrant_to_independent_reference() {
    // houses-reference.py `asc1` (swehouse.c Asc1): quadrant fold into Asc2.
    // 30° -> Q1, 120° -> Q2, 210° -> Q3, 300° -> Q4 exercise all four match
    // arms and the ±pole / (180-x)/(x-180)/(360-x) argument folding.
    let eps = 23.4366_f64;
    let (sine, cose) = (eps.to_radians().sin(), eps.to_radians().cos());
    let cases = [
        (30.0_f64, 60.273_411_210_075),
        (120.0, 138.177_359_444_927),
        (210.0, 200.983_664_735_370),
        (300.0, 266.674_198_092_798),
    ];
    for (x, expected) in cases {
        let got = asc1(x, 52.0, sine, cose).degrees();
        assert!((got - expected).abs() < 1e-9, "asc1({x}) = {got}");
    }
}

#[test]
fn asc_mc_from_pins_all_points_across_pole_and_flip_branches() {
    // Independent reference (houses-reference.py `asc_mc_from`, swehouse.c
    // swe_houses_armc). obl = 23.4366°. Three geometries cover:
    //   G1 lat>obl  -> f_pole = 90-lat, vertex flip inactive
    //   G2 0<lat<=obl -> flip branch active (vemc>0 path)
    //   G3 lat<0    -> f_pole = -90-lat branch
    // Every literal cross-validated to 1e-12 against the crate.
    let eps = 23.4366_f64;
    let check = |armc: f64, lat: f64, exp: [f64; 7]| {
        let p = asc_mc_from(armc, lat, eps).expect("finite");
        let got = [
            p.ascendant.degrees(),
            p.midheaven.degrees(),
            p.vertex.degrees(),
            p.equatorial_ascendant.degrees(),
            p.coascendant_koch.degrees(),
            p.coascendant_munkasey.degrees(),
            p.polar_ascendant.degrees(),
        ];
        for (i, (g, e)) in got.iter().zip(exp.iter()).enumerate() {
            assert!(
                (g - e).abs() < 1e-8,
                "armc={armc} lat={lat} point[{i}] = {g}, want {e}"
            );
        }
    };
    // G1: armc=45, lat=52 (> obl) — non-flip, f_pole = 38.
    check(
        45.0,
        52.0,
        [
            148.587_249_395_771,
            47.463_595_280_938,
            295.549_781_631_009,
            132.536_404_719_062,
            101.175_335_496_703,
            143.611_940_830_436,
            281.175_335_496_703,
        ],
    );
    // G2: armc=200, lat=10 (0<lat<=obl) — vertex flip active.
    check(
        200.0,
        10.0,
        [
            284.537_224_659_332,
            201.638_102_932_963,
            159.911_766_495_904,
            288.466_379_243_755,
            292.223_701_037_155,
            205.822_995_524_640,
            112.223_701_037_155,
        ],
    );
    // G3: armc=100, lat=-33 (< 0) — f_pole = -90-lat = -57.
    check(
        100.0,
        -33.0,
        [
            195.061_993_029_707,
            99.189_697_612_154,
            6.534_310_124_205,
            190.878_573_217_375,
            188.500_387_274_068,
            210.816_655_151_624,
            8.500_387_274_068,
        ],
    );
}

#[test]
fn interpolate_longitude_wraps_and_scales() {
    // span = (end-start).rem_euclid(360) = 30; start + span*frac = 357.5.
    // start=350,end=20,frac=0.25 keeps every mutant (- -> +//, + -> *//-,
    // * -> +//) observably wrong. Hand-computed.
    let got = interpolate_longitude(
        Longitude::from_degrees(350.0),
        Longitude::from_degrees(20.0),
        0.25,
    )
    .degrees();
    assert!((got - 357.5).abs() < 1e-9, "interp = {got}");
}

#[test]
fn porphyry_houses_trisect_each_quadrant() {
    // asc=100, mc=10 -> desc=280, ic=190. Each quadrant spans 90°, trisected
    // at 30°/60°. Independent hand arithmetic (houses-reference.py `porphyry`)
    // makes the 1/3 and 2/3 fractions observable (mutating / -> % or *).
    let cusps = porphyry_houses(HouseAngles::new(
        Longitude::from_degrees(100.0),
        Longitude::from_degrees(10.0),
    ));
    let expected = [
        100.0, 130.0, 160.0, 190.0, 220.0, 250.0, 280.0, 310.0, 340.0, 10.0, 40.0, 70.0,
    ];
    for (i, e) in expected.iter().enumerate() {
        assert!(
            (cusps[i].degrees() - e).abs() < 1e-9,
            "cusp[{i}] = {}, want {e}",
            cusps[i].degrees()
        );
    }
}

#[test]
fn signed_longitude_difference_both_branches() {
    // delta<180 branch: (10-350).rem_euclid(360)=20 -> 20.
    assert!((signed_longitude_difference(10.0, 350.0) - 20.0).abs() < 1e-9);
    // delta>=180 branch: (200-10).rem_euclid(360)=190 -> 190-360 = -170.
    assert!((signed_longitude_difference(200.0, 10.0) + 170.0).abs() < 1e-9);
}

#[test]
fn right_ascension_from_ecliptic_longitude_matches_reference() {
    // atan2(sinλ·cosε, cosλ) at λ=60°, ε=23.4366°. Independent reference.
    let eps = 23.4366_f64;
    let got =
        right_ascension_from_ecliptic_longitude(Longitude::from_degrees(60.0), eps.to_radians());
    assert!((got - 57.819_266_732_173).abs() < 1e-9, "ra = {got}");
}

#[test]
fn whole_sign_first_cusp_floors_to_sign_boundary() {
    // asc=95° -> first cusp floor(95/30)*30 = 90°. The `* 30` mutant (-> /30)
    // collapses the cusp to 0.1; pin the first two cusps.
    let cusps = whole_sign_houses(Longitude::from_degrees(95.0));
    assert!(
        (cusps[0].degrees() - 90.0).abs() < 1e-9,
        "c0 = {}",
        cusps[0].degrees()
    );
    assert!(
        (cusps[1].degrees() - 120.0).abs() < 1e-9,
        "c1 = {}",
        cusps[1].degrees()
    );
}

#[test]
fn longitude_in_arc_handles_wraparound() {
    // Wraparound arc [350,10): membership is `lon>=350 || lon<10`. A point at
    // 355 is in via the first disjunct only, so || -> && flips it to false.
    assert!(longitude_in_arc(355.0, 350.0, 10.0), "355 in [350,10)");
    assert!(longitude_in_arc(5.0, 350.0, 10.0), "5 in [350,10)");
    // Non-wrap arc [10,20): 15 in, 25 out.
    assert!(longitude_in_arc(15.0, 10.0, 20.0));
    assert!(!longitude_in_arc(25.0, 10.0, 20.0));
}

#[test]
fn longitude_opposite_is_the_antipode() {
    // `longitude_opposite(x) = from_degrees(x + 180)`. NOTE: the cargo-mutants
    // survivor `+ -> -` here is a DOCUMENTED EQUIVALENT MUTANT, not a coverage
    // hole: from_degrees normalizes mod 360 and x+180 ≡ x-180 (mod 360) for all
    // x, so no reachable input distinguishes `+` from `-`. It is left visible
    // (no #[mutants::skip]) per FU-9 posture. This test pins the antipode
    // intent; it cannot and does not claim to kill the equivalent mutant.
    assert!((longitude_opposite(Longitude::from_degrees(50.0)).degrees() - 230.0).abs() < 1e-9);
    assert!((longitude_opposite(Longitude::from_degrees(300.0)).degrees() - 120.0).abs() < 1e-9);
}

#[test]
fn asc2_degenerate_sinx_zero_branch_pins() {
    // The four quadrant inputs in `asc2_matches_independent_swehouse_kernel`
    // all take the normal `atan(sinx/value)` path, leaving asc2's degenerate
    // guard branch (`sinx.abs() < 1e-12`) uncovered. x on the sinx~0 axis
    // reaches it. Independent reference (houses-reference.py `asc2`):
    //   asc2(0)   -> value>0, so the branch returns the +1e-12 sentinel exactly.
    //   asc2(180) -> value<0, so it returns -1e-12, folded by `+180` to ~180.
    // Kills the sinx~0 guard/sign mutants: 1811 `< -> ==`, 1812 `< -> ==`/`>`,
    // and `delete -` on the -1e-12 sentinel (1813).
    let eps = 23.4366_f64;
    let (sine, cose) = (eps.to_radians().sin(), eps.to_radians().cos());
    // asc2(0) is the +1e-12 sentinel exactly; tolerance below the sentinel so a
    // mutant that skips the branch (returns 0.0) or flips the sign (returns
    // ~180) fails.
    let a0 = asc2(0.0, 52.0, sine, cose);
    assert!((a0 - 1e-12).abs() < 1e-13, "asc2(0) = {a0}");
    // asc2(180) folds -1e-12 to ~180; mutants that skip the branch return ~1e-12.
    let a180 = asc2(180.0, 52.0, sine, cose);
    assert!(
        (a180 - 179.999_999_999_999).abs() < 1e-9,
        "asc2(180) = {a180}"
    );
}

#[test]
fn asc_mc_from_vertex_flip_actually_fires() {
    // G2 in `asc_mc_from_pins_all_points_across_pole_and_flip_branches` enters
    // the `|lat| <= obl` flip block but vemc <= 0, so the vertex never rotates
    // and the flip guards/arithmetic stay uncovered. This geometry
    // (armc=15, lat=5) makes vemc > 0 so the flip FIRES, rotating the vertex by
    // 180. Independent reference (houses-reference.py `asc_mc_from`, G4). Kills
    // the flip-trigger mutants: 202 `<= -> >`, 207 `> -> ==`, 208 `+ -> *`.
    let eps = 23.4366_f64;
    let p = asc_mc_from(15.0, 5.0, eps).expect("finite");
    let got = [
        p.ascendant.degrees(),
        p.midheaven.degrees(),
        p.vertex.degrees(),
        p.equatorial_ascendant.degrees(),
        p.coascendant_koch.degrees(),
        p.coascendant_munkasey.degrees(),
        p.polar_ascendant.degrees(),
    ];
    let exp = [
        105.741_462_639_643,
        16.280_047_054_689,
        12.635_804_712_026,
        103.811_888_609_499,
        101.849_837_950_278,
        168.584_056_229_968,
        281.849_837_950_278,
    ];
    for (i, (g, e)) in got.iter().zip(exp.iter()).enumerate() {
        assert!((g - e).abs() < 1e-8, "point[{i}] = {g}, want {e}");
    }
}

#[test]
fn asc_mc_from_equator_pole_asymmetry_kills_tan_periodicity_mutants() {
    // Final-review fix (2026-07-23): the original Foundation PR's equivalence
    // sweep sampled lat in [-66,-50,-33,-20,-10,-5,5,10,20,23,33,50,66] and
    // never tried lat = 0. At lat = 0 the pole height `f_pole` used by the
    // vertex/coascendant_munkasey branch is exactly +-90 deg, where `tan` is
    // NOT 180-periodic in f64 (`tan(90 deg) = +1.633...e16` vs
    // `tan(-90 deg) = -1.633...e16`, not equal). That makes several mutants
    // that were documented as "180-periodicity of tan" equivalents actually
    // observable at the equator. Independent reference (houses-reference.py
    // `asc_mc_from`), cross-validated to 1e-12.
    let eps = 23.4366_f64;

    // C1: kills mod.rs `lat_deg >= 0.0 -> < 0.0` (the f_pole branch select).
    // At lat=0.0 HEAD takes the `>=` arm (f_pole = 90-lat = 90); the mutant
    // takes the `<` arm's `else` (f_pole = -90-lat = -90), giving tan(pole)
    // the opposite huge-magnitude sign and moving coascendant_munkasey from
    // 180.0 to ~6.2e-15 -- a ~180 degree miss.
    let c1 = asc_mc_from(45.0, 0.0, eps).expect("finite");
    let got = c1.coascendant_munkasey.degrees();
    assert!(
        signed_longitude_difference(got, 180.0).abs() < 1e-9,
        "coascendant_munkasey = {got}, want ~180.0"
    );

    // C2: kills mod.rs `vemc > 180.0 -> >= 180.0`. At armc=0, lat=0 the raw
    // vemc is exactly 180.0. HEAD's strict `>` leaves it unfolded and the
    // vertex then flips (vemc > 0.0) to 0.0; the mutant's `>=` folds it to
    // -180.0 first, so the flip guard sees a negative vemc and never fires,
    // leaving the vertex at 180.0 -- a ~180 degree miss.
    let c2 = asc_mc_from(0.0, 0.0, eps).expect("finite");
    let got = c2.vertex.degrees();
    assert!(
        signed_longitude_difference(got, 0.0).abs() < 1e-9,
        "vertex = {got}, want ~0.0"
    );

    // C3: kills mod.rs `vemc > 0.0 -> >= 0.0`. At armc=180, lat=0 the raw vemc
    // is exactly 0.0. HEAD's strict `>` does not flip, leaving vertex at
    // 180.0; the mutant's `>=` flips it to 0.0 -- a ~180 degree miss.
    let c3 = asc_mc_from(180.0, 0.0, eps).expect("finite");
    let got = c3.vertex.degrees();
    assert!(
        signed_longitude_difference(got, 180.0).abs() < 1e-9,
        "vertex = {got}, want ~180.0"
    );

    // I1: kills mod.rs `delete -` turning `-90.0 - lat` into `90.0 - lat` in
    // the southern (`lat < 0`) branch. lat = -1e-16 is a hair below zero, so
    // it takes that branch: HEAD computes f_pole = -90.0 - (-1e-16) ~ -90,
    // giving coascendant_munkasey ~6.2e-15; the mutant computes
    // f_pole = 90.0 - (-1e-16) ~ +90, giving ~180.0 -- again a ~180 degree
    // miss, mirroring C1 but through the southern branch.
    let i1 = asc_mc_from(45.0, -1e-16, eps).expect("finite");
    let got = i1.coascendant_munkasey.degrees();
    assert!(
        signed_longitude_difference(got, 0.0).abs() < 1e-9,
        "coascendant_munkasey = {got}, want ~0.0"
    );
}

#[test]
fn asc2_value_zero_guard_reachable_at_exact_equality() {
    // Final-review fix (2026-07-23): the old equivalence writeup claimed
    // "no representable input hits equality" for asc2's `1.0e-12` guard
    // thresholds. That is wrong: the guard at mod.rs `value.abs() < 1.0e-12`
    // ASSIGNS `value = 0.0`, and pole = 90 - EPS = 66.5634 (the f_pole of an
    // observer at latitude == obliquity) drives `value` to exactly 0.0 at
    // x = 0, making the downstream `value < 0.0` comparison reachable at
    // equality too. Independent reference (houses-reference.py `asc2`),
    // cross-validated to 1e-12: both calls below equal 1e-12 exactly.
    let eps = 23.4366_f64;
    let (sine, cose) = (eps.to_radians().sin(), eps.to_radians().cos());

    // Kills mod.rs `value < 0.0 -> <= 0.0`: value is exactly 0.0 here, so
    // HEAD's strict `<` takes the `else` arm (+1e-12 sentinel); the mutant's
    // `<=` takes the `if` arm (-1e-12), which the final `< 0.0 -> += 180.0`
    // fold turns into ~180.0 -- a ~180 degree miss.
    let at_pole = asc2(0.0, 66.5634, sine, cose);
    assert!(
        (at_pole - 1e-12).abs() < 1e-13,
        "asc2(0, 66.5634) = {at_pole}"
    );

    // Kills mod.rs `value.abs() < 1.0e-12 -> == 1.0e-12`: at this neighboring
    // pole, the raw `value` is a tiny unrounded epsilon, not exactly 1e-12,
    // so the mutant guard never fires and value is never snapped to 0.0. The
    // sign of that unrounded epsilon leaks through to the same ~180 degree
    // miss as above.
    let near_pole = asc2(0.0, 66.5634000000001, sine, cose);
    assert!(
        (near_pole - 1e-12).abs() < 1e-13,
        "asc2(0, 66.5634000000001) = {near_pole}"
    );
}

#[test]
fn asc_geometry_equivalent_mutants_are_documented() {
    // The measured cargo-mutants residual for the Foundation functions is a
    // set of DOCUMENTED EQUIVALENT MUTANTS, left visible (no
    // `#[mutants::skip]`) per the FU-9 posture.
    //
    // CORRECTION (final-review fix, 2026-07-23): an earlier version of this
    // test claimed six of these were equivalent by "180-periodicity of tan in
    // the pole height" and "unreachable exact-equality boundaries". Both
    // claims were wrong — the equivalence sweep never sampled lat = 0 (pole
    // height exactly +-90, where f64 `tan` is NOT 180-periodic) and never
    // noticed that asc2's `value.abs() < 1e-12` guard EXPLICITLY assigns
    // `value = 0.0`, making the downstream `value < 0.0` reachable at
    // equality. Those six mutants (mod.rs 192, 195, 204, 207, 1807, 1812) are
    // now KILLED by `asc_mc_from_equator_pole_asymmetry_kills_tan_periodicity_mutants`
    // and `asc2_value_zero_guard_reachable_at_exact_equality` above, and are
    // no longer documented as equivalent here. The remaining residual splits
    // into two honestly-distinguished categories:
    //
    // All 13 measured survivors are enumerated below, in three honestly
    // distinguished categories.
    //
    // (A) STRUCTURALLY UNREACHABLE / BIT-IDENTICAL — no representable input can
    // distinguish the operators, independent of tolerance. [5 mutants]
    //   * asc2 1818 `< -> ==`, `< -> >`, `< -> <=`, and 1819 `delete -`: this
    //     `else if value == 0.0` arm is reached ONLY when `sinx.abs() >= 1e-12`
    //     (the 1811 guard consumed the small-sinx case), so `sinx` is never 0
    //     here — the arm is instead reached because the 1807 guard ASSIGNED
    //     `value = 0.0`. Whichever way these mutants steer the `sinx < 0.0`
    //     test, the result is `-90.0` or `+90.0`, and the 1826 fold maps
    //     `-90.0 + 180.0` to exactly `90.0`. So every variant returns bit-
    //     identical `90.0`. (NOTE: an earlier revision of this comment stated
    //     the inverted premise "the 1811 guard already forced sinx == 0" — the
    //     conclusion held but the reason did not.)
    //   * asc2 1826 `< -> <=` (final `longitude < 0.0` fold): `longitude ==
    //     0.0` is unreachable from all three producing branches.
    //
    // (B) BELOW THE 1e-9 PARITY TOLERANCE, BUT MEASURABLY NOT BIT-IDENTICAL
    // (I2 correction — do not claim "differ by exactly 360" or "no
    // representable input hits equality" here; state the measured magnitude).
    // Each magnitude is a sweep MAXIMUM, not a proven bound. [6 mutants]
    //   * asc1 `delete match arm 3` (1799): arm 3 and the `_` arm are
    //     ALGEBRAICALLY identical, but not f64-identical — `(180-u)*pi/180`
    //     and `pi - u*pi/180` differ in the last bits, so the trig arguments
    //     differ. Measured max diff ~5.68e-14 (at x1 ~ 180.315, pole -52).
    //     This was previously mis-filed under (A) as "bit-identical".
    //   * asc1 arm-3 `x1 - 180 -> x1 + 180` (360-periodicity of asc2 in x):
    //     measured max diff ~2.56e-13 over a sweep, not exactly 0.
    //   * asc_mc `armc - 180 -> + 180` at 201 and 215: measured max circular
    //     diff ~1.31e-12 over a lat/armc sweep, not exactly 0.
    //   * asc_mc vertex flip `+180 -> -180` (208) and `longitude_opposite`'s
    //     `+ -> -` (1833): measured max circular diff ~5.68e-14 over a sweep,
    //     not exactly 0.
    //
    // (C) asc2's REMAINING 1e-12 GUARD THRESHOLDS — below tolerance under
    // generic inputs; explicitly NOT a strict-unreachability claim. [2 mutants]
    //   * asc2 1807 `< -> <=` (`value.abs() < 1e-12`) and 1811 `< -> <=`
    //     (`sinx.abs() < 1e-12`): under generic (non-adversarial) inputs the
    //     reachable boundary difference is ~1.4e-10 — below the 1e-9 parity
    //     tolerance. An adversarial input sitting exactly on the threshold
    //     could exceed it, so these two are best read as "not proven
    //     equivalent, not currently killable", and are flagged as such in the
    //     docs/follow-ups.md note rather than asserted to be unreachable.
    //
    // 5 + 6 + 2 = 13, matching the measured `mutants.out/missed.txt`.
    //
    // All measured magnitudes above are independently confirmed by
    // `houses-reference.py`'s sanity-check sweeps, never by running the
    // crate. The identities below are asserted so the reasoning itself is
    // regression-tested.
    let eps = 23.4366_f64;
    let (sine, cose) = (eps.to_radians().sin(), eps.to_radians().cos());

    // 360-periodicity of asc2 in x (asc1 arm-3 `x1-180 -> x1+180`): below
    // tolerance, not bit-identical (see comment above).
    assert!(
        (asc2(30.0 - 180.0, -52.0, sine, cose) - asc2(30.0 + 180.0, -52.0, sine, cose)).abs()
            < 1e-9
    );

    // 180-periodicity of tan(pole) holds well away from the poles (asc_mc
    // pole-branch 90-lat vs -90-lat, both far from +-90):
    assert!((asc2(70.0, 38.0, sine, cose) - asc2(70.0, 38.0 - 180.0, sine, cose)).abs() < 1e-9);
    // ...but the identity FAILS exactly at the degenerate pole (lat = 0,
    // pole = +-90): tan(90 deg) and tan(-90 deg) are not equal in f64, so the
    // difference blows well past the parity tolerance instead of vanishing.
    // This is precisely the case
    // `asc_mc_from_equator_pole_asymmetry_kills_tan_periodicity_mutants`
    // exercises to kill mutants 192/195/204/207 above — it is not
    // re-documented as equivalent here.
    assert!(
        (asc2(0.0, 90.0, sine, cose) - asc2(0.0, -90.0, sine, cose)).abs() > 1.0,
        "the tan(pole) periodicity identity must FAIL at pole = +-90"
    );

    // Vertex-fold `+180 -> -180`: exact for these particular literals (pure
    // rem_euclid arithmetic on 200.0, no trig involved)...
    assert_eq!(
        (200.0_f64 + 180.0).rem_euclid(360.0),
        (200.0_f64 - 180.0).rem_euclid(360.0)
    );
    // ...but not universally bit-identical once trig-derived vertex values are
    // involved: x = 332.3 is a measured near-worst-case (~5.68e-14 diff),
    // still comfortably below the 1e-9 tolerance.
    assert!(
        ((332.3_f64 + 180.0).rem_euclid(360.0) - (332.3_f64 - 180.0).rem_euclid(360.0)).abs()
            < 1e-9
    );

    // asc_mc `armc - 180 -> + 180` (201/215): below tolerance, not exact.
    // armc=88.9, lat=66.0 is a measured near-worst-case (~1.3e-12 diff).
    let obl = eps.to_radians();
    let a = ascendant_for(88.9 - 180.0, 66.0, obl).degrees();
    let b = ascendant_for(88.9 + 180.0, 66.0, obl).degrees();
    assert!(
        signed_longitude_difference(a, b).abs() < 1e-9,
        "a={a} b={b}"
    );
}

// ===== FU-9 Great-circle PR: apc_sector / apc_houses / horizon / krusinski =====

#[test]
fn apc_sector_pins_all_twelve_against_independent_reference() {
    // Independent reference (houses-reference.py `apc_sector`, published APC
    // algorithm) at lat=52°, obl=23.4366°, sidereal=45° — non-degenerate, so
    // every `*`/`+`/`-`/`/` swap and the `n < 8` split is observable. Pinning
    // all 12 sectors kills all 58 arith survivors (measured 59/59 caught).
    let lat = 52.0_f64.to_radians();
    let obl = 23.4366_f64.to_radians();
    let sid = 45.0_f64.to_radians();
    let expected = [
        148.587_249_395_771, 166.495_240_772_036, 189.747_228_099_578,
        227.463_595_280_938, 275.481_343_990_138, 308.273_382_675_614,
        328.587_249_395_771, 350.866_340_446_180, 14.729_289_455_955,
        47.463_595_280_938, 88.169_411_590_291, 122.556_859_248_324,
    ];
    for (i, e) in expected.iter().enumerate() {
        let got = apc_sector(i + 1, lat, obl, sid).degrees();
        assert!((got - e).abs() < 1e-9, "apc_sector({}) = {got}, want {e}", i + 1);
    }
}
