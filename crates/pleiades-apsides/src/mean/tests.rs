//! Unit tests for the mean lunar orbit elements.

use super::*;
use crate::points_from_elements;
use proptest::prelude::*;

const J2000: f64 = 2_451_545.0;

#[test]
fn polynomials_reproduce_their_j2000_constants() {
    assert!((mean_lunar_node_longitude_of_date(J2000) - 125.044_547_9).abs() < 1e-12);
    assert!((mean_lunar_perigee_longitude_of_date(J2000) - 83.353_246_5).abs() < 1e-12);
}

#[test]
fn polynomials_match_hand_evaluation_one_century_out() {
    // t = 1 Julian century: every coefficient contributes.
    let jd = J2000 + 36_525.0;
    assert!((mean_lunar_node_longitude_of_date(jd) - 350.910_336_282_4).abs() < 1e-7);
    assert!((mean_lunar_perigee_longitude_of_date(jd) - 192.356_642_760_9).abs() < 1e-7);
}

#[test]
fn polynomials_match_the_meeus_worked_dates() {
    // Published examples: the mean node passes 0° on 1913-05-27 and 180° on
    // 1959-12-07; the mean perigee is 224.89194° on 2021-03-05 (0h TT).
    assert!(mean_lunar_node_longitude_of_date(2_419_914.5) < 0.1);
    assert!((mean_lunar_node_longitude_of_date(2_436_909.5) - 180.0).abs() < 0.1);
    assert!((mean_lunar_perigee_longitude_of_date(2_459_278.5) - 224.891_94).abs() < 1e-4);
}

#[test]
fn elements_assemble_the_five_mean_values() {
    let e = mean_lunar_elements_of_date(J2000);
    assert_eq!(e.node_deg, mean_lunar_node_longitude_of_date(J2000));
    assert_eq!(e.peri_lon_deg, mean_lunar_perigee_longitude_of_date(J2000));
    assert_eq!(e.incl_deg, MOON_MEAN_INCLINATION_DEG);
    assert_eq!(e.eccentricity, MOON_MEAN_ECCENTRICITY);
    assert_eq!(e.semi_major_au, MOON_MEAN_SEMI_MAJOR_AU);
}

#[test]
fn projected_mean_apogee_at_j2000_is_the_swiss_ephemeris_point() {
    // SE mean apogee at J2000 (nod-aps corpus row): 263.464250479°, +3.419723161°
    // in the true equinox of date; Δψ(J2000) ≈ −0.003868°, so the mean-equinox
    // value is ≈ 263.46812°. The raw element (perigee + 180°) is 263.3532°:
    // this test fails if the point is not projected through the inclined orbit.
    let points = points_from_elements(&mean_lunar_elements_of_date(J2000), false).unwrap();
    assert!((points.aphelion.longitude_deg - 263.468_12).abs() < 1e-4);
    assert!((points.aphelion.latitude_deg - 3.419_722).abs() < 1e-5);
    assert!((points.aphelion.distance_au - 0.002_710_625).abs() < 1e-9);
    assert!((points.perihelion.latitude_deg + 3.419_722).abs() < 1e-5);
    assert!((points.ascending.latitude_deg).abs() < 1e-12);
}

#[test]
fn non_finite_input_does_not_panic() {
    assert!(mean_lunar_node_longitude_of_date(f64::NAN).is_nan());
    assert!(mean_lunar_perigee_longitude_of_date(f64::INFINITY).is_nan());
}

#[test]
fn normalize_degrees_never_returns_360() {
    let d = normalize_degrees(-1e-20);
    assert!((0.0..360.0).contains(&d), "{d}");
    assert!((0.0..360.0).contains(&normalize_degrees(-360.0)));
    assert!(normalize_degrees(f64::NAN).is_nan());
}

#[test]
fn far_but_finite_epochs_stay_normalized() {
    for jd in [-1e9, -100_000.0, 0.0, 1e9] {
        let node = mean_lunar_node_longitude_of_date(jd);
        let peri = mean_lunar_perigee_longitude_of_date(jd);
        assert!((0.0..360.0).contains(&node), "node at {jd}: {node}");
        assert!((0.0..360.0).contains(&peri), "perigee at {jd}: {peri}");
    }
}

#[test]
fn non_finite_and_overflowing_input_yields_nan() {
    for jd in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1e200] {
        assert!(
            mean_lunar_node_longitude_of_date(jd).is_nan(),
            "node at {jd}"
        );
        assert!(
            mean_lunar_perigee_longitude_of_date(jd).is_nan(),
            "perigee at {jd}"
        );
    }
}

proptest! {
    #[test]
    fn longitudes_stay_normalized(jd in 0.0_f64..5_000_000.0) {
        let node = mean_lunar_node_longitude_of_date(jd);
        let peri = mean_lunar_perigee_longitude_of_date(jd);
        prop_assert!((0.0..360.0).contains(&node), "node {node}");
        prop_assert!((0.0..360.0).contains(&peri), "perigee {peri}");
    }
}
