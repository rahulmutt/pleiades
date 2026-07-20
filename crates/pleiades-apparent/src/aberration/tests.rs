//! White-box unit tests for the annual-aberration module.
//!
//! Relocated out of `aberration.rs` per AGENTS.md ("keep large inline test
//! suites out of the file under test"). These remain white-box unit tests with
//! access to the module's private helpers (`julian_centuries`,
//! `earth_orbit_elements`) — they are deliberately not converted into
//! black-box integration tests.

use super::*;

#[test]
fn magnitude_is_bounded_by_kappa_over_cos_beta() {
    // For modest latitudes Δλ stays within a few × κ; never explosive.
    let off = annual_aberration(100.0, 2.0, 280.0, 2_451_545.0);
    assert!(
        off.d_lambda_arcsec.abs() < 25.0,
        "Δλ = {}",
        off.d_lambda_arcsec
    );
    assert!(off.d_beta_arcsec.abs() < 1.0, "Δβ = {}", off.d_beta_arcsec);
}

#[test]
fn sign_and_magnitude_match_known_geometry() {
    // Physically valid sign/magnitude checks at J2000 (t=0), β=0 so cosβ=1.
    // The dominant term is -κ cos(⊙-λ); the e·κ term adds ≈ +0.34″.
    // (The earlier "Venus" example used an impossible geometry — Venus
    // 180° from the Sun — and an inconsistent expected value; replaced with
    // valid geometries. Precision is gated end-to-end against Horizons in
    // Task 14.)

    // Conjunction (body at the Sun's longitude, ⊙-λ = 0): Δλ ≈ -κ + 0.34 ≈ -20.15″.
    let conj = annual_aberration(100.0, 0.0, 100.0, 2_451_545.0);
    assert!(
        conj.d_lambda_arcsec < 0.0,
        "conjunction Δλ should be negative: {}",
        conj.d_lambda_arcsec
    );
    assert!(
        (conj.d_lambda_arcsec - (-20.15)).abs() < 0.2,
        "conjunction Δλ = {}",
        conj.d_lambda_arcsec
    );

    // Opposition (body opposite the Sun, ⊙-λ = 180): Δλ ≈ +κ + 0.34 ≈ +20.84″.
    let opp = annual_aberration(100.0, 0.0, 280.0, 2_451_545.0);
    assert!(
        opp.d_lambda_arcsec > 0.0,
        "opposition Δλ should be positive: {}",
        opp.d_lambda_arcsec
    );
    assert!(
        (opp.d_lambda_arcsec - 20.84).abs() < 0.2,
        "opposition Δλ = {}",
        opp.d_lambda_arcsec
    );

    // Quadrature (⊙-λ = 90): the main term vanishes; only the small e·κ term
    // remains (< 1″), and Δβ stays bounded by κ off the ecliptic.
    let quad = annual_aberration(100.0, 10.0, 190.0, 2_451_545.0);
    assert!(
        quad.d_lambda_arcsec.abs() < 1.0,
        "quadrature Δλ = {}",
        quad.d_lambda_arcsec
    );
    assert!(
        quad.d_beta_arcsec.abs() < KAPPA_ARCSEC,
        "quadrature Δβ = {}",
        quad.d_beta_arcsec
    );
}

#[test]
fn julian_centuries_counts_from_j2000_in_units_of_36525_days() {
    // 2451545.0 is J2000 itself -> t = 0.
    assert!(
        (julian_centuries(2_451_545.0) - 0.0).abs() < 1e-15,
        "t(J2000) = {}",
        julian_centuries(2_451_545.0)
    );

    // 2469807.5 = J2000 + 18262.5 d = J2000 + half a Julian century.
    // A *half* century is deliberate: t = 1.0 would be indistinguishable
    // from the `replace julian_centuries -> 1.0` whole-function mutant,
    // and t = 0 alone is indistinguishable from `-> 0.0`.
    assert!(
        (julian_centuries(2_469_807.5) - 0.5).abs() < 1e-15,
        "t(J2000 + 18262.5 d) = {}",
        julian_centuries(2_469_807.5)
    );
}

#[test]
fn earth_orbit_elements_match_meeus_25_4() {
    // Meeus 25.4, evaluated OUTSIDE this code:
    //   e(t) = 0.016708634 - 0.000042037 t - 0.0000001267 t^2
    //   ϖ(t) = 102.93735   + 1.71946 t     + 0.00046 t^2
    //
    // Three epochs are required, not one. At t = 0 only the lead constants
    // are exercised. Evaluating at both +1 and -1 separates the linear term
    // (which flips sign) from the quadratic (which does not) — that is what
    // distinguishes a mutated quadratic coefficient from a mutated linear one.
    for (t, e_expected, pi_expected) in [
        (0.0, 0.016_708_634, 102.937_35),
        (1.0, 0.016_666_470_3, 104.657_27),
        (-1.0, 0.016_750_544_3, 101.218_35),
    ] {
        let (e, pi_deg) = earth_orbit_elements(t);
        assert!(
            (e - e_expected).abs() < 1e-15,
            "e({t}) = {e}, expected {e_expected}"
        );
        assert!(
            (pi_deg - pi_expected).abs() < 1e-12,
            "ϖ({t}) = {pi_deg}, expected {pi_expected}"
        );
    }
}

#[test]
fn meeus_23_2_matches_independent_evaluation_at_crafted_geometry() {
    // Crafted discriminating geometry (design §6):
    //   λ = 30°, β = 60°, ⊙ = 120°, jd = J2000 (t = 0)
    //
    // Chosen so that:
    //   cos β = 0.5   -> `/ cos_beta` and `* cos_beta` differ by a factor of 4
    //   ⊙ - λ = 90°   -> cos = 0, sin = 1, isolating the e·κ term in Δλ
    //   λ ≠ 0         -> otherwise ϖ + λ ≡ ϖ - λ and the `-` → `+` mutant lives
    //   λ ≠ ϖ         -> otherwise sin(ϖ - λ) = 0, annihilating the whole
    //                    `e sin(ϖ - λ)` subtraction and letting its mutants live
    //   β ≠ 0         -> Δβ is non-zero, so its sign is assertable
    //
    // Expected values evaluated OUTSIDE this code from Meeus 23.2, using the
    // t = 0 elements e = 0.016708634 and ϖ = 102.93735, so ϖ - λ = 72.93735°:
    //   Δλ = (0 + 0.34245214231967996 * 0.29341719999540683) / 0.5
    //      = 0.20096269746373557
    //   Δβ = -20.49552 * sin(60°) * (1 - 0.016708634 * 0.9559844908505867)
    //      = -17.46612250773869
    let off = annual_aberration(30.0, 60.0, 120.0, 2_451_545.0);

    assert!(
        (off.d_lambda_arcsec - 0.200_962_697_463_735_57).abs() < 1e-9,
        "Δλ = {}",
        off.d_lambda_arcsec
    );
    assert!(
        (off.d_beta_arcsec - (-17.466_122_507_738_69)).abs() < 1e-9,
        "Δβ = {}",
        off.d_beta_arcsec
    );
}

#[test]
fn aberration_in_latitude_is_signed_not_merely_bounded() {
    // The pre-existing tests assert only |Δβ| against a bound, which leaves
    // the leading `-` of the Δβ formula completely unconstrained. Pin the
    // sign directly: at β = +60° with ⊙ - λ = +90°, sin β > 0 and the
    // bracket (1 - e sin(ϖ-λ)) > 0, so Δβ must be strictly negative.
    let off = annual_aberration(30.0, 60.0, 120.0, 2_451_545.0);
    assert!(
        off.d_beta_arcsec < -17.0,
        "Δβ should be strictly negative here: {}",
        off.d_beta_arcsec
    );

    // sin β is odd, so mirroring the ecliptic latitude must negate Δβ exactly
    // while leaving Δλ (which depends on β only through the even cos β) alone.
    let mirrored = annual_aberration(30.0, -60.0, 120.0, 2_451_545.0);
    assert!(
        (mirrored.d_beta_arcsec + off.d_beta_arcsec).abs() < 1e-12,
        "Δβ(-β) should negate Δβ(+β): {} vs {}",
        mirrored.d_beta_arcsec,
        off.d_beta_arcsec
    );
    assert!(
        (mirrored.d_lambda_arcsec - off.d_lambda_arcsec).abs() < 1e-12,
        "Δλ should be even in β: {} vs {}",
        mirrored.d_lambda_arcsec,
        off.d_lambda_arcsec
    );
}
