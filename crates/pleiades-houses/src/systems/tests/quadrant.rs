use super::support::*;
use crate::systems::*;
use pleiades_types::{Angle, Instant, JulianDay, Latitude, TimeScale};

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

/// FU-9: pins the WGS-84 reduction against an independent evaluation of the
/// published datum constants (`houses-reference.py::topocentric_latitude`).
///
/// The elevation MUST be non-zero: with `h = 0` the `(N + h)` term at
/// mod.rs:1681 is degenerate and its `+ -> -` mutant is invisible.
#[test]
fn topocentric_latitude_pins_the_wgs84_reduction_with_elevation() {
    assert_close_degrees(
        topocentric_latitude(40.0, Some(1000.0))
            .expect("finite elevation is accepted")
            .degrees(),
        39.810_640_281_732_304,
    );
    assert_close_degrees(
        topocentric_latitude(-33.0, Some(500.0))
            .expect("finite elevation is accepted")
            .degrees(),
        -32.824_466_106_045_98,
    );
}

/// FU-9: cross-checks the prime-vertical form against a second, genuinely
/// different published formulation, `tan(phi') = (1 - f)^2 * tan(phi)`, which
/// is exact at sea level. Constrains the eccentricity terms at mod.rs:1680
/// independently of the elevation pins above.
#[test]
fn topocentric_latitude_at_sea_level_matches_the_closed_form() {
    let flattening = 1.0 / 298.257_223_563;
    let one_minus_f_squared = (1.0 - flattening) * (1.0 - flattening);
    for latitude in [40.0_f64, 55.0, -33.0, 66.0] {
        let expected = (one_minus_f_squared * latitude.to_radians().tan())
            .atan()
            .to_degrees();
        assert_close_degrees(
            topocentric_latitude(latitude, None)
                .expect("absent elevation is accepted")
                .degrees(),
            expected,
        );
    }
}

/// FU-9: the pre-existing Regiomontanus coverage runs at lat 0, where
/// `sin(lat) = 0` and `cos(lat) = 1` make three of the four factors at
/// mod.rs:947-949 degenerate. These two non-equatorial charts restore them.
#[test]
fn regiomontanus_cusps_match_swiss_ephemeris_corpus() {
    // houses-corpus/cusps.csv row c1_lat40 (JD 2451545.0, lat 40N, lon 0).
    assert_corpus_cusps(
        "Regiomontanus c1_lat40",
        HouseSystem::Regiomontanus,
        40.0,
        [
            17.706_103,
            57.771_262,
            81.547_840,
            99.611_088,
            119.384_226,
            149.836_713,
            197.706_103,
            237.771_262,
            261.547_840,
            279.611_088,
            299.384_226,
            329.836_713,
        ],
    );
    // houses-corpus/cusps.csv row c2_lat55 (JD 2451545.0, lat 55N, lon 0).
    assert_corpus_cusps(
        "Regiomontanus c2_lat55",
        HouseSystem::Regiomontanus,
        55.0,
        [
            28.505_186,
            72.373_244,
            88.608_626,
            99.611_088,
            112.251_819,
            138.090_289,
            208.505_186,
            252.373_244,
            268.608_626,
            279.611_088,
            292.251_819,
            318.090_289,
        ],
    );
}

/// FU-9: Koch is undefined inside the polar circle, where the Midheaven's
/// ascensional difference stops being real, and mod.rs:843 fails closed on
/// `|lat| >= 90 - obliquity`. The catalog caps Koch at |lat| 66°, *below* the
/// 66.56° polar circle, so `calculate_houses` rejects (Strict) or substitutes
/// Porphyry (SwissEphemerisFallback) before this guard is ever reached — the
/// guard is only observable by calling the private function directly.
#[test]
fn koch_houses_fails_closed_inside_the_polar_circle() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let observer = ObserverLocation::new(
        Latitude::from_degrees(70.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let obliquity = Angle::from_degrees(23.4392811);
    let angles = derive_angles(instant, &observer, obliquity);

    let error = koch_houses(instant, &observer, obliquity, angles)
        .expect_err("Koch inside the polar circle must fail closed");
    assert_eq!(error.kind, crate::error::HouseErrorKind::NumericalFailure);
    assert!(
        error.message.contains("undefined within the polar circle"),
        "unexpected message: {}",
        error.message
    );
}

/// FU-9: pins all four solved Placidus cusps against an independent BISECTION
/// root of the published residual (`houses-reference.py`), not against the
/// crate's own Newton output.
///
/// Geometry `RAMC = 90°, lat = 61°, eps = 23.4392811°` was chosen by search to
/// maximise the weakest derivative-mutant displacement. It also kills the two
/// derivative-term mutants at mod.rs:1739 without any divergence hunt: a
/// mutated derivative still converges, but stops at its own `|delta| < 1e-9`
/// leaving an error of order 1e-10, while HEAD's quadratic convergence lands
/// within ~1e-14 of the true root. Tolerance 1e-11 sits between the two.
#[test]
fn solve_placidian_cusp_matches_an_independent_bisection_root() {
    const TOLERANCE_DEG: f64 = 1.0e-11;

    let cases = [
        (11_usize, 129.435_210_158_986_65),
        (12, 158.604_832_372_515_46),
        (2, 201.395_167_627_484_54),
        (3, 230.564_789_841_013_35),
    ];

    for (house, expected) in cases {
        let cusp =
            solve_placidian_cusp(90.0, 61.0, 23.4392811, house).expect("this geometry converges");
        let difference = (cusp.degrees() - expected).abs();
        assert!(
            difference < TOLERANCE_DEG,
            "house {house}: {} differs from the bisection root {expected} by {difference:e}",
            cusp.degrees(),
        );
    }
}

/// FU-9: the zero-derivative guard at mod.rs:1741 fails closed when the Newton
/// derivative vanishes. `gp = (-(1/f)·sin(q/f) + tan(φ)·tan(ε)·cos(α)) / (180/π)`.
/// At the house-11 seed (`q = 30°`, so `q/f = 90°` and `sin = 1`) with
/// `RAMC = 330°` the cosine factor is `cos(360°) = 1`, so `gp` vanishes exactly
/// when `tan(φ)·tan(ε) = 1/f`. Latitude is a free test input, so solving that
/// equation for it gives 81.776_683_964_516_9°, where `|gp| ~ 1.2e-16 < 1e-12`.
///
/// This distinguishes the `<` -> `==` mutant, which disables the guard for
/// every genuinely near-zero derivative: HEAD reports "zero derivative", the
/// mutant falls through to the non-convergence exit and reports "failed to
/// converge". Both are `NumericalFailure`, so only the diagnostic message
/// separates them — this file already pins error-message text in five places
/// (`request.rs`), so pinning it here is the established practice.
#[test]
fn solve_placidian_cusp_fails_closed_on_a_vanishing_derivative() {
    let error = solve_placidian_cusp(330.0, 81.776_683_964_516_9, 23.4392811, 11)
        .expect_err("a vanishing derivative must fail closed");
    assert_eq!(error.kind, crate::error::HouseErrorKind::NumericalFailure);
    assert!(
        error.message.contains("zero derivative"),
        "unexpected message: {}",
        error.message
    );
}

/// FU-9: the fail-closed exit at mod.rs:1756 is `!converged || !q.is_finite()`.
/// At lat 78° the product `|tan(φ)·tan(δ)|` exceeds 1 over much of the range,
/// so `cos(q/f) = -tan(φ)·tan(δ)` has no solution and the iteration oscillates
/// without converging while `q` stays **finite** (~39.6). That combination —
/// `converged == false`, `q` finite — is exactly what the `||` -> `&&` mutant
/// needs to slip through: with `&&` the guard is false and the function returns
/// a garbage `Ok` instead of failing closed.
#[test]
fn solve_placidian_cusp_fails_closed_when_the_iteration_does_not_converge() {
    let error = solve_placidian_cusp(18.0, 78.0, 23.4392811, 11)
        .expect_err("a non-converging geometry must fail closed");
    assert_eq!(error.kind, crate::error::HouseErrorKind::NumericalFailure);
    assert!(
        error.message.contains("failed to converge"),
        "unexpected message: {}",
        error.message
    );
}
