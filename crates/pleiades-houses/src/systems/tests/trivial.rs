//! The non-quadrant, non-projection systems: Equal, EqualAries,
//! EqualMidheaven, Vehlow, WholeSign, and the Porphyry-midpoint family
//! (Porphyry itself and Sripati, which trisects Porphyry's quadrant
//! segments).

use super::support::*;
use crate::systems::*;
use pleiades_types::Latitude;

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

// Issue #180: the sign-anchored systems rebuilt for an ascendant in another
// zodiac.

fn degrees(cusps: [Longitude; 12]) -> [f64; 12] {
    cusps.map(|cusp| cusp.degrees())
}

/// Cusps every 30° from `first_deg`.
fn sign_boundaries_from(first_deg: f64) -> [f64; 12] {
    core::array::from_fn(|house| (first_deg + 30.0 * house as f64) % 360.0)
}

#[test]
fn sign_anchored_whole_sign_cusps_start_at_the_ascendants_sign() {
    for (ascendant_deg, first_deg) in [
        (46.7632, 30.0),
        (0.0, 0.0),
        (29.999_999, 0.0),
        (30.0, 30.0),
        (359.5, 330.0),
    ] {
        let cusps = sign_anchored_cusps(
            &HouseSystem::WholeSign,
            Longitude::from_degrees(ascendant_deg),
        )
        .expect("whole sign is sign-anchored");
        assert_eq!(
            degrees(cusps),
            sign_boundaries_from(first_deg),
            "ascendant {ascendant_deg}"
        );
    }
}

#[test]
fn sign_anchored_equal_aries_cusps_ignore_the_ascendant() {
    for ascendant_deg in [0.0, 46.7632, 359.5] {
        let cusps = sign_anchored_cusps(
            &HouseSystem::EqualAries,
            Longitude::from_degrees(ascendant_deg),
        )
        .expect("equal Aries is sign-anchored");
        assert_eq!(degrees(cusps), sign_boundaries_from(0.0));
    }
}

#[test]
fn sign_anchored_cusps_match_calculate_houses_for_its_own_ascendant() {
    for system in [HouseSystem::WholeSign, HouseSystem::EqualAries] {
        let snapshot =
            calculate_houses(&sample_request(system.clone())).expect("houses should work");
        let cusps = sign_anchored_cusps(&system, snapshot.angles.ascendant)
            .expect("system is sign-anchored");
        assert_eq!(snapshot.cusps, cusps.to_vec(), "{system:?}");
    }
}

#[test]
fn only_whole_sign_and_equal_aries_are_sign_anchored() {
    let ascendant = Longitude::from_degrees(46.7632);
    for system in [
        HouseSystem::Equal,
        HouseSystem::EqualMidheaven,
        HouseSystem::Vehlow,
        HouseSystem::Sripati,
        HouseSystem::Porphyry,
        HouseSystem::Placidus,
        HouseSystem::Koch,
        HouseSystem::Regiomontanus,
        HouseSystem::Campanus,
        HouseSystem::Meridian,
        HouseSystem::Morinus,
        HouseSystem::Gauquelin,
    ] {
        assert!(
            sign_anchored_cusps(&system, ascendant).is_none(),
            "{system:?}"
        );
    }
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

/// Swiss Ephemeris external-reference anchor test.
///
/// Fixture: JD=2451545.0 (J2000.0), lat=40°N, lon=0°E (Equal house system).
/// SE reference values: ASC=17.706103°, MC=279.611088°.
/// Tolerance: 1 arcsec; actual residuals are ~0.04 arcsec after switching to
/// GAST + true obliquity (equation of equinoxes applied).
#[test]
fn equal_house_angles_match_swiss_ephemeris_corpus_within_1_arcsec() {
    let observer = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let request = HouseRequest::new(
        Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Ut1,
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

/// FU-9: `sripati_midpoints_follow_porphyry_segments` compares the crate's
/// Sripati cusps against `midpoint_longitude` — the same function on both
/// sides — so the `-> Default::default()` mutant at mod.rs:1790 makes both
/// sides `0` and still passes. Swiss Ephemeris never calls our function, so
/// this row breaks the circularity.
#[test]
fn sripati_cusps_match_swiss_ephemeris_corpus() {
    // houses-corpus/cusps.csv row c1_lat40 (JD 2451545.0, lat 40N, lon 0).
    assert_corpus_cusps(
        "Sripati c1_lat40",
        HouseSystem::Sripati,
        40.0,
        [
            1.356_934,
            31.356_934,
            58.658_595,
            85.960_257,
            115.960_257,
            148.658_595,
            181.356_934,
            211.356_934,
            238.658_595,
            265.960_257,
            295.960_257,
            328.658_595,
        ],
    );
}
