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

/// Reference: ecliptic-to-ecliptic precession of Meeus, *Astronomical
/// Algorithms* 2nd ed. ch. 21 (eqs. 21.5 and 21.7: the angles eta, Pi, p
/// from the IAU-1976 constants, applied as spherical rotations), evaluated
/// independently of `pleiades_apparent`, which precesses through the
/// equatorial frame with the 1976 zeta/z/theta polynomials. The two agree to
/// about 7e-11 AU on these vectors (measured on the first failing component
/// with a 1e-12 tolerance, J1900 A x), so the tolerance is 1e-8 AU: still
/// orders of magnitude below any arithmetic mutant.
#[test]
fn precession_to_j2000_matches_the_meeus_ecliptic_reference() {
    const J1900: f64 = 2_415_020.0;
    const B1950: f64 = 2_433_282.423_459_05;
    let cases = [
        (
            "J1900 A",
            (1.2, -0.7, 0.4),
            J1900,
            [1.216697495156814, -0.670634170169941, 0.39987124812820674],
        ),
        (
            "J1900 B",
            (-2.0, 1.5, -1.1),
            J1900,
            [-2.035945398844941, 1.4510563695624994, -1.0997098459476553],
        ),
        (
            "B1950 A",
            (1.2, -0.7, 0.4),
            B1950,
            [1.208439367814481, -0.6853662776962126, 0.3999341942282199],
        ),
        (
            "B1950 B",
            (-2.0, 1.5, -1.1),
            B1950,
            [-2.018124398331545, 1.4756348993249713, -1.0998525159098027],
        ),
    ];
    for (what, (x, y, z), equinox, want) in cases {
        assert_xyz(
            rotate_ecliptic_to_j2000(x, y, z, equinox),
            want,
            1.0e-8,
            what,
        );
    }
}

#[test]
fn the_zero_vector_stays_at_the_origin() {
    assert_eq!(
        rotate_ecliptic_to_j2000(0.0, 0.0, 0.0, 2_415_020.0),
        (0.0, 0.0, 0.0)
    );
}
