use super::*;

const J2000: f64 = 2_451_545.0;

/// Reference values from Swiss Ephemeris 2.10.03's own Vondrák 2011 routines
/// (`swi_epsiln`, and `swi_coortrf` / `swi_precess(J_TO_J2000)` / `swi_coortrf`
/// for the vector rotation), default astro models, called directly from a
/// throwaway probe linked against `libswisseph-sys` 0.1.2. 1856113.380954 is
/// Nibiru's `seorbel.txt` equinox (about 370 CE); 1000000.5 is about 1975 BCE.
const SE_OBLIQUITY_ARCSEC: [(f64, f64); 5] = [
    (2_451_545.0, 84_381.405_999),
    (2_415_020.0, 84_428.240_610_293_24),
    (1_856_113.380_954, 85_136.242_057_049),
    (2_524_593.5, 84_287.749_641_516_92),
    (1_000_000.5, 86_117.970_590_612_56),
];

type Case = (f64, [f64; 3], [f64; 3]);

#[rustfmt::skip]
const SE_DATE_TO_J2000: [Case; 12] = [
    (2_415_020.0, [1.2, -0.7, 0.4], [1.216696534556666e0, -6.706358978373615e-1, 3.998712734489384e-1]),
    (2_415_020.0, [-2.0, 1.5, -1.1], [-2.035943318237606e0, 1.451059251437099e0, -1.099709895264407e0]),
    (2_415_020.0, [234.0, 10.0, -3.0], [2.336868223232635e2, 1.570087999557012e1, -2.991895690053989e0]),
    (1_856_113.380_954, [1.2, -0.7, 0.4], [1.377120554623878e0, -1.84403927749807e-1, 3.994172873860948e-1]),
    (1_856_113.380_954, [-2.0, 1.5, -1.1], [-2.423754965678148e0, 6.164973878518347e-1, -1.097881066929557e0]),
    (1_856_113.380_954, [234.0, 10.0, -3.0], [2.120305159619357e2, 9.95050168330439e1, -2.609966660897776e0]),
    (2_524_593.5, [1.2, -0.7, 0.4], [1.164457153425969e0, -7.575000998935233e-1, 4.00291314540288e-1]),
    (2_524_593.5, [-2.0, 1.5, -1.1], [-1.924527304451385e0, 1.595240699387085e0, -1.100636981679287e0]),
    (2_524_593.5, [234.0, 10.0, -3.0], [2.34209126559181e2, -1.423487849358481e0, -3.009770544125459e0]),
    (1_000_000.5, [1.2, -0.7, 0.4], [1.261519954830966e0, 5.787897320750359e-1, 4.044376955821303e-1]),
    (1_000_000.5, [-2.0, 1.5, -1.1], [-2.376060643547723e0, -7.696807864316332e-1, -1.105408207487876e0]),
    (1_000_000.5, [234.0, 10.0, -3.0], [1.258622021620169e2, 1.975401330711732e2, -1.265659176373717e0]),
];

#[test]
fn mean_obliquity_matches_swiss_ephemeris() {
    for (jd, expected) in SE_OBLIQUITY_ARCSEC {
        let got = long_term_mean_obliquity_radians(jd).to_degrees() * 3600.0;
        assert!(
            (got - expected).abs() < 1.0e-9,
            "obliquity at {jd}: {got} vs SE {expected}"
        );
    }
}

#[test]
fn date_to_j2000_matches_swiss_ephemeris() {
    // Same formulas, same order of operations: agreement is at rounding level,
    // about 1e-15 relative. 1e-12 relative (2e-7 arcsec) still catches any
    // altered coefficient.
    for (jd, v, expected) in SE_DATE_TO_J2000 {
        let got = precess_ecliptic_vector_date_to_j2000_long_term(v, jd).unwrap();
        let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        for k in 0..3 {
            assert!(
                (got[k] - expected[k]).abs() < 1.0e-12 * r,
                "jd {jd} v {v:?} component {k}: {} vs SE {}",
                got[k],
                expected[k]
            );
        }
    }
}

#[test]
fn j2000_equinox_is_the_identity() {
    let v = [1.2, -0.7, 0.4];
    assert_eq!(
        precess_ecliptic_vector_date_to_j2000_long_term(v, J2000).unwrap(),
        v
    );
}

#[test]
fn rotation_preserves_length() {
    for (jd, v, _) in SE_DATE_TO_J2000 {
        let got = precess_ecliptic_vector_date_to_j2000_long_term(v, jd).unwrap();
        let r0 = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        let r1 = (got[0] * got[0] + got[1] * got[1] + got[2] * got[2]).sqrt();
        assert!((r1 - r0).abs() < 1.0e-13 * r0, "jd {jd}: {r0} -> {r1}");
    }
}

#[test]
fn agrees_with_iau_1976_near_j2000() {
    // A century from J2000 the two models differ mainly by the IAU 2006
    // correction to the general precession rate (5028.796" against IAU-1976's
    // 5029.097" per century): about 0.30" per century in longitude, measured
    // -0.294" at J1900 and +0.306" at J2100, and under 0.005" in latitude.
    const RATE_CORRECTION_ARCSEC_PER_CENTURY: f64 = 0.30;
    let (lon, lat) = (123.456_f64, 4.5_f64);
    let (sl, cl) = libm::sincos(lon.to_radians());
    let (sb, cb) = libm::sincos(lat.to_radians());
    let v = [cb * cl, cb * sl, sb];
    for jd in [2_415_020.0, 2_488_070.0] {
        let lt = precess_ecliptic_vector_date_to_j2000_long_term(v, jd).unwrap();
        let iau = crate::precess_ecliptic_date_to_j2000(lon, lat, jd).unwrap();
        let lt_lon = libm::atan2(lt[1], lt[0]).to_degrees().rem_euclid(360.0);
        let lt_lat = libm::asin(lt[2]).to_degrees();
        let t = (jd - J2000) / 36_525.0;
        let dlon = (lt_lon - iau.longitude_deg) * 3600.0;
        let dlat = (lt_lat - iau.latitude_deg) * 3600.0;
        assert!(
            (dlon - RATE_CORRECTION_ARCSEC_PER_CENTURY * t).abs() < 0.02,
            "lon at {jd}: {dlon}"
        );
        assert!(dlat.abs() < 0.01, "lat at {jd}: {dlat}");
    }
}

#[test]
fn non_finite_input_is_refused() {
    for v in [[f64::NAN, 0.0, 0.0], [0.0, f64::INFINITY, 0.0]] {
        assert!(precess_ecliptic_vector_date_to_j2000_long_term(v, 2_415_020.0).is_err());
    }
    assert!(precess_ecliptic_vector_date_to_j2000_long_term([1.0, 0.0, 0.0], f64::NAN).is_err());
}
