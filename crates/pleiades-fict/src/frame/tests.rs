use super::*;
use crate::test_support::assert_xyz;

#[test]
fn j2000_equinox_is_identity() {
    let (x, y, z) = rotate_ecliptic_to_j2000(1.0, 2.0, 0.5, crate::J2000_JD);
    assert!((x - 1.0).abs() < 1.0e-12);
    assert!((y - 2.0).abs() < 1.0e-12);
    assert!((z - 0.5).abs() < 1.0e-12);
}

#[test]
fn precession_preserves_distance() {
    // Ecliptic precession is a pure rotation: |v| is invariant. J1900 = 2415020.0.
    let (x, y, z) = rotate_ecliptic_to_j2000(3.0, 0.0, 0.0, 2_415_020.0);
    let r = (x * x + y * y + z * z).sqrt();
    assert!((r - 3.0).abs() < 1.0e-9, "distance changed: r={r}");
}

#[test]
fn an_equinox_within_a_microday_of_j2000_returns_the_vector_untouched() {
    // The identity branch returns the input as is; the precession path would
    // move it by about 7e-13 AU at +-9e-7 d (just inside the 1e-6 d window),
    // far above the 1e-14 tolerance.
    for offset in [0.0, 9.0e-7, -9.0e-7] {
        let equinox = crate::J2000_JD + offset;
        assert_xyz(
            rotate_ecliptic_to_j2000(1.2, -0.7, 0.4, equinox),
            [1.2, -0.7, 0.4],
            1.0e-14,
            "A",
        );
        assert_xyz(
            rotate_ecliptic_to_j2000(-2.0, 1.5, -1.1, equinox),
            [-2.0, 1.5, -1.1],
            1.0e-14,
            "B",
        );
    }
}

#[test]
fn a_distant_equinox_is_precessed() {
    // Not the identity even for an equinox a day from J2000.
    let (x, y, z) = rotate_ecliptic_to_j2000(1.2, -0.7, 0.4, crate::J2000_JD + 1.0);
    assert!((x, y, z) != (1.2, -0.7, 0.4));
    let (x, y, z) = rotate_ecliptic_to_j2000(1.2, -0.7, 0.4, crate::J2000_JD - 1.0);
    assert!((x, y, z) != (1.2, -0.7, 0.4));
}

/// Reference: Swiss Ephemeris 2.10.03 carrying the vector the way
/// `swi_osc_el_plan` does (`swi_coortrf` by -eps(equinox), `swi_precess`
/// J_TO_J2000, `swi_coortrf` by eps(J2000), default Vondrak 2011 models),
/// called directly from a throwaway probe. Same formulas in the same order, so
/// agreement is at rounding level; 1e-11 AU is about 1e-6 arcsec on these
/// vectors. The IAU-1976 rotation used before misses by about 0.3" at J1900 and
/// 1.4" at Nibiru's equinox, 1e-6 AU and more.
#[test]
fn precession_to_j2000_matches_swiss_ephemeris() {
    const J1900: f64 = 2_415_020.0;
    const NIBIRU_EQUINOX: f64 = 1_856_113.380_954;
    let cases = [
        (
            "J1900 A",
            (1.2, -0.7, 0.4),
            J1900,
            [1.216696534556666, -0.6706358978373615, 0.3998712734489384],
        ),
        (
            "J1900 B",
            (-2.0, 1.5, -1.1),
            J1900,
            [-2.035943318237606, 1.451059251437099, -1.099709895264407],
        ),
        (
            "Nibiru A",
            (1.2, -0.7, 0.4),
            NIBIRU_EQUINOX,
            [1.377120554623878, -0.184403927749807, 0.3994172873860948],
        ),
        (
            "Nibiru C",
            (234.0, 10.0, -3.0),
            NIBIRU_EQUINOX,
            [212.0305159619357, 99.5050168330439, -2.609966660897776],
        ),
    ];
    for (what, (x, y, z), equinox, want) in cases {
        let tol = if what == "Nibiru C" { 1.0e-9 } else { 1.0e-11 };
        assert_xyz(rotate_ecliptic_to_j2000(x, y, z, equinox), want, tol, what);
    }
}

#[test]
fn the_zero_vector_stays_at_the_origin() {
    assert_eq!(
        rotate_ecliptic_to_j2000(0.0, 0.0, 0.0, 2_415_020.0),
        (0.0, 0.0, 0.0)
    );
}
