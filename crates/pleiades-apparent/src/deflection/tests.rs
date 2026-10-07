use super::*;
use crate::aberration::{sun_radius_vector_au_of_date, sun_true_longitude_of_date_deg};

const JD: f64 = 2_451_545.0;
/// 2GM☉ / (c² · 1 AU), in arcseconds: the deflection scale at 1 AU.
const SCALE_ARCSEC: f64 = 0.004_071_9;

fn sun() -> (f64, f64) {
    (
        sun_true_longitude_of_date_deg(JD),
        sun_radius_vector_au_of_date(JD),
    )
}

// At 90° elongation the displacement is the full scale (cot 45° = 1), along
// the ecliptic away from the Sun.
#[test]
fn a_star_at_quadrature_is_pushed_away_from_the_sun_by_the_full_scale() {
    let (sun_lon, r) = sun();
    let east = gravitational_deflection(sun_lon + 90.0, 0.0, JD);
    assert!(
        (east.d_lambda_arcsec - SCALE_ARCSEC / r).abs() < 2e-6,
        "{east:?}"
    );
    assert!(east.d_beta_arcsec.abs() < 1e-9);
    let west = gravitational_deflection(sun_lon - 90.0, 0.0, JD);
    assert!(
        (west.d_lambda_arcsec + SCALE_ARCSEC / r).abs() < 2e-6,
        "{west:?}"
    );
}

// Outside the disc |Δ| = scale · cot(ψ/2) / r.
#[test]
fn the_deflection_grows_as_cot_half_elongation() {
    let (sun_lon, r) = sun();
    for psi in [10.0_f64, 2.0, 0.5] {
        let d = gravitational_deflection(sun_lon + psi, 0.0, JD);
        let expected = SCALE_ARCSEC / r / (psi.to_radians() / 2.0).tan();
        assert!(
            (d.d_lambda_arcsec - expected).abs() < 1e-4 * expected,
            "{psi}: {d:?}"
        );
    }
}

// Inside the disc Swiss Ephemeris scales the mass by `meff` (Stix's solar
// model): at a tenth of the solar radius m_eff = 0.186794.
#[test]
fn inside_the_disc_the_mass_is_tapered_by_meff() {
    let (sun_lon, r) = sun();
    let radius_deg = 959.63 / 3600.0 / r;
    let psi = 0.1 * radius_deg;
    let d = gravitational_deflection(sun_lon + psi, 0.0, JD);
    let expected = 0.186_794 * SCALE_ARCSEC / r / (psi.to_radians() / 2.0).tan();
    assert!(
        (d.d_lambda_arcsec - expected).abs() < 1e-3 * expected,
        "{d:?}"
    );
    // At the centre there is no deflection at all.
    let centre = gravitational_deflection(sun_lon, 0.0, JD);
    assert_eq!(centre.d_lambda_arcsec, 0.0);
}

#[test]
fn meff_matches_the_swiss_ephemeris_table_ends_and_interpolates() {
    assert_eq!(meff::meff(0.0), 0.0);
    assert_eq!(meff::meff(1.0), 1.0);
    assert_eq!(meff::meff(1.5), 1.0);
    assert_eq!(meff::meff(0.5), 0.937_790);
    // Halfway between 0.10 (0.186794) and 0.11 (0.218327).
    assert!((meff::meff(0.105) - 0.202_560_5).abs() < 1e-9);
}
