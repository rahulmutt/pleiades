use super::*;
use pleiades_types::{Angle, CustomHouseSystem, Latitude};

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
    assert!(error.message.contains("Placidus is undefined beyond |latitude| 66"));
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
    // Re-pinned after the Topocentric (Polich-Page) algorithm was corrected to
    // agree with Swiss Ephemeris (within the mean-vs-apparent sidereal floor).
    // Cusp 1 now equals the Ascendant and cusp 10 the Midheaven, as required.
    let mut request = sample_request(HouseSystem::Topocentric);
    request.observer.latitude = Latitude::from_degrees(45.0);
    request.observer.longitude = Longitude::from_degrees(10.0);
    request.observer.elevation_m = Some(2_000.0);
    request.obliquity = Some(Angle::from_degrees(23.439_291_1));

    let snapshot = calculate_houses(&request).expect("topocentric houses should work");

    assert_eq!(snapshot.cusps.len(), 12);
    assert_close_degrees(snapshot.angles.ascendant.degrees(), 37.122_815_618_733_284);
    assert_close_degrees(snapshot.angles.descendant.degrees(), 217.122_815_618_733_284);
    assert_close_degrees(snapshot.angles.midheaven.degrees(), 288.896_788_004_295_672);
    assert_close_degrees(snapshot.angles.imum_coeli.degrees(), 108.896_788_004_295_672);
    assert_close_degrees(snapshot.cusps[0].degrees(), 37.122_815_618_733_284);
    assert_close_degrees(snapshot.cusps[1].degrees(), 67.534_515_714_748_451);
    assert_close_degrees(snapshot.cusps[9].degrees(), 288.896_788_004_295_672);
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

    let snapshot = HouseSnapshot {
        system: HouseSystem::Equal,
        instant: sample_request(HouseSystem::Equal).instant,
        observer: sample_request(HouseSystem::Equal).observer,
        obliquity: Angle::from_degrees(23.4),
        angles: HouseAngles {
            ascendant: Longitude::from_degrees(15.0),
            descendant: Longitude::from_degrees(195.0),
            midheaven: Longitude::from_degrees(45.0),
            imum_coeli: Longitude::from_degrees(225.0),
        },
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
/// Tolerance is 120 arcsec; actual residuals are ~15 arcsec.
#[test]
fn morinus_cusps_match_swiss_ephemeris_corpus_within_120_arcsec() {
    let circ_diff_arcsec = |a: f64, b: f64| -> f64 {
        let diff = (a - b).rem_euclid(360.0);
        let signed = if diff > 180.0 { diff - 360.0 } else { diff };
        signed.abs() * 3600.0
    };
    let tolerance_arcsec = 120.0_f64;

    // c1_lat40 SE corpus Morinus row, cusps c1..c12.
    let se_morinus: [f64; 12] = [
        9.611_088, 38.040_522, 68.849_424, 101.373_900, 132.906_648, 161.960_854,
        189.611_088, 218.040_522, 248.849_424, 281.373_900, 312.906_648, 341.960_854,
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
            snapshot.cusps, snapshots[0].cusps,
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
        assert_eq!(
            (snapshot.cusps[index + 6].degrees() - snapshot.cusps[index].degrees())
                .rem_euclid(360.0),
            180.0
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
    // Each of the 4 arcs (ASC→MC, MC→DSC, DSC→IC, IC→ASC) is divided into 9 equal steps.
    // Verify uniform spacing within the first arc (ASC to MC).
    let step_01 = (snapshot.cusps[1].degrees() - snapshot.cusps[0].degrees()).rem_euclid(360.0);
    let step_12 = (snapshot.cusps[2].degrees() - snapshot.cusps[1].degrees()).rem_euclid(360.0);
    assert!(
        (step_01 - step_12).abs() < 1.0e-10,
        "Gauquelin sectors should be uniformly spaced within each arc"
    );
    // Each step is ~10° (either clockwise or counter-clockwise depending on ASC/MC geometry).
    let step_deg = step_01.min(360.0 - step_01);
    assert!(
        (step_deg - 10.0).abs() < 1.5,
        "Gauquelin sector angular width should be ~10°, got {step_deg}°"
    );
}

#[test]
fn house_snapshots_reject_wrong_cusp_counts() {
    let equal_snapshot = HouseSnapshot {
        system: HouseSystem::Equal,
        instant: sample_request(HouseSystem::Equal).instant,
        observer: observer(),
        obliquity: Angle::from_degrees(23.4),
        angles: HouseAngles {
            ascendant: Longitude::from_degrees(15.0),
            descendant: Longitude::from_degrees(195.0),
            midheaven: Longitude::from_degrees(45.0),
            imum_coeli: Longitude::from_degrees(225.0),
        },
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

    let gauquelin_snapshot = HouseSnapshot {
        system: HouseSystem::Gauquelin,
        instant: sample_request(HouseSystem::Gauquelin).instant,
        observer: observer(),
        obliquity: Angle::from_degrees(23.4),
        angles: HouseAngles {
            ascendant: Longitude::from_degrees(15.0),
            descendant: Longitude::from_degrees(195.0),
            midheaven: Longitude::from_degrees(45.0),
            imum_coeli: Longitude::from_degrees(225.0),
        },
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
    let broken_descendant_snapshot = HouseSnapshot {
        system: HouseSystem::Equal,
        instant: sample_request(HouseSystem::Equal).instant,
        observer: observer(),
        obliquity: Angle::from_degrees(23.4),
        angles: HouseAngles {
            ascendant: Longitude::from_degrees(15.0),
            descendant: Longitude::from_degrees(200.0),
            midheaven: Longitude::from_degrees(45.0),
            imum_coeli: longitude_opposite(Longitude::from_degrees(45.0)),
        },
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

    let broken_ic_snapshot = HouseSnapshot {
        system: HouseSystem::Equal,
        instant: sample_request(HouseSystem::Equal).instant,
        observer: observer(),
        obliquity: Angle::from_degrees(23.4),
        angles: HouseAngles {
            ascendant: Longitude::from_degrees(15.0),
            descendant: longitude_opposite(Longitude::from_degrees(15.0)),
            midheaven: Longitude::from_degrees(45.0),
            imum_coeli: Longitude::from_degrees(250.0),
        },
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
    let snapshot = HouseSnapshot {
        system: HouseSystem::Equal,
        instant: sample_request(HouseSystem::Equal).instant,
        observer: observer(),
        obliquity: Angle::from_degrees(23.4),
        angles: HouseAngles {
            ascendant: Longitude::from_degrees(15.0),
            descendant: Longitude::from_degrees(195.0),
            midheaven: Longitude::from_degrees(75.0),
            imum_coeli: Longitude::from_degrees(255.0),
        },
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
    let snapshot = calculate_houses(&request)
        .expect("SE-compat fallback must succeed beyond bound");

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

/// Swiss Ephemeris external-reference anchor test.
///
/// Fixture: JD=2451545.0 (J2000.0), lat=40°N, lon=0°E (Equal house system).
/// SE reference values: ASC=17.706103°, MC=279.611088°.
/// Tolerance: 120 arcsec (residual is mean-vs-apparent sidereal time, addressed separately).
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
    let tolerance_arcsec = 120.0_f64;

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
/// (`pleiades-validate/data/houses-corpus/cusps.csv`). Tolerance 120 arcsec is
/// the mean-vs-apparent sidereal floor the already-correct systems achieve.
#[test]
fn placidus_and_topocentric_cusps_match_swiss_ephemeris_corpus_within_120_arcsec() {
    let circ_diff_arcsec = |a: f64, b: f64| -> f64 {
        let diff = (a - b).rem_euclid(360.0);
        let signed = if diff > 180.0 { diff - 360.0 } else { diff };
        signed.abs() * 3600.0
    };
    let tolerance_arcsec = 120.0_f64;

    // c1_lat40 SE corpus rows, cusps c1..c12.
    let se_placidus: [f64; 12] = [
        17.706_103, 53.858_979, 78.399_152, 99.611_088, 122.382_578, 152.464_496, 197.706_103,
        233.858_979, 258.399_152, 279.611_088, 302.382_578, 332.464_496,
    ];
    let se_topocentric: [f64; 12] = [
        17.706_103, 53.759_507, 78.270_701, 99.611_088, 122.465_089, 152.483_265, 197.706_103,
        233.759_507, 258.270_701, 279.611_088, 302.465_089, 332.483_265,
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
/// (`pleiades-validate/data/houses-corpus/cusps.csv`). Tolerance 120 arcsec is
/// the mean-vs-apparent sidereal floor the already-correct systems achieve;
/// Koch (which reuses the same `Asc1` projection as Placidus) sits well inside
/// it at this latitude.
#[test]
fn koch_cusps_match_swiss_ephemeris_corpus_within_120_arcsec() {
    let circ_diff_arcsec = |a: f64, b: f64| -> f64 {
        let diff = (a - b).rem_euclid(360.0);
        let signed = if diff > 180.0 { diff - 360.0 } else { diff };
        signed.abs() * 3600.0
    };
    let tolerance_arcsec = 120.0_f64;

    // c1_lat40 SE corpus Koch row, cusps c1..c12.
    let se_koch: [f64; 12] = [
        17.706_103, 51.954_052, 78.286_109, 99.611_088, 125.345_306, 158.845_358, 197.706_103,
        231.954_052, 258.286_109, 279.611_088, 305.345_306, 338.845_358,
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
/// (`pleiades-validate/data/houses-corpus/cusps.csv`). Tolerance 120 arcsec is
/// the mean-vs-apparent sidereal floor; Campanus sits well inside it at this
/// latitude (max residual ≈ 20 arcsec).
#[test]
fn campanus_cusps_match_swiss_ephemeris_corpus_within_120_arcsec() {
    let circ_diff_arcsec = |a: f64, b: f64| -> f64 {
        let diff = (a - b).rem_euclid(360.0);
        let signed = if diff > 180.0 { diff - 360.0 } else { diff };
        signed.abs() * 3600.0
    };
    let tolerance_arcsec = 120.0_f64;

    // c1_lat40 SE corpus Campanus row, cusps c1..c12.
    let se_campanus: [f64; 12] = [
        17.706_103, 64.352_912, 85.435_838, 99.611_088, 114.834_455, 141.116_623,
        197.706_103, 244.352_912, 265.435_838, 279.611_088, 294.834_455, 321.116_623,
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
/// (`pleiades-validate/data/houses-corpus/cusps.csv`). Tolerance 120 arcsec is
/// the mean-vs-apparent sidereal floor; Alcabitius sits well inside it at
/// lat 40° (max residual < 5 arcsec after the loop-offset fix).
#[test]
fn alcabitius_cusps_match_swiss_ephemeris_corpus_within_120_arcsec() {
    let circ_diff_arcsec = |a: f64, b: f64| -> f64 {
        let diff = (a - b).rem_euclid(360.0);
        let signed = if diff > 180.0 { diff - 360.0 } else { diff };
        signed.abs() * 3600.0
    };
    let tolerance_arcsec = 120.0_f64;

    // c1_lat40 SE corpus Alcabitius row, cusps c1..c12.
    let se_alcabitius: [f64; 12] = [
        17.706_103, 46.835_395, 73.785_097, 99.611_088, 129.969_119, 163.041_881,
        197.706_103, 226.835_395, 253.785_097, 279.611_088, 309.969_119, 343.041_881,
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

