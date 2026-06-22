//! Annual aberration in ecliptic coordinates (Meeus ch. 23, eq. 23.2).
//! Pure function: the caller supplies the body's ecliptic position and the
//! Sun's true longitude; this crate has no ephemeris of its own.

/// Aberration constant κ, arcseconds.
const KAPPA_ARCSEC: f64 = 20.495_52;

/// Annual-aberration offset in ecliptic longitude and latitude, arcseconds.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AberrationOffset {
    /// Aberration in ecliptic longitude (Δλ), arcseconds.
    pub d_lambda_arcsec: f64,
    /// Aberration in ecliptic latitude (Δβ), arcseconds.
    pub d_beta_arcsec: f64,
}

fn julian_centuries(jd_tt: f64) -> f64 {
    (jd_tt - 2_451_545.0) / 36_525.0
}

/// Annual aberration for an ecliptic position, given the Sun's true longitude ⊙.
///
/// Meeus 23.2:
///   Δλ = (-κ cos(⊙ - λ) + e κ cos(ϖ - λ)) / cos β
///   Δβ = -κ sin β (sin(⊙ - λ) - e sin(ϖ - λ))
/// with e the eccentricity and ϖ the longitude of perihelion of Earth's orbit
/// (Meeus 25.4 / 23.x), both of date.
pub fn annual_aberration(
    lambda_deg: f64,
    beta_deg: f64,
    sun_true_longitude_deg: f64,
    jd_tt: f64,
) -> AberrationOffset {
    let t = julian_centuries(jd_tt);
    let e = 0.016_708_634 - 0.000_042_037 * t - 0.000_000_126_7 * t * t;
    let pi_deg = 102.937_35 + 1.719_46 * t - 0.000_46 * t * t;

    let lambda = lambda_deg.to_radians();
    let beta = beta_deg.to_radians();
    let sun = sun_true_longitude_deg.to_radians();
    let pi = pi_deg.to_radians();

    let cos_beta = beta.cos();
    let d_lambda = (-KAPPA_ARCSEC * (sun - lambda).cos()
        + e * KAPPA_ARCSEC * (pi - lambda).cos())
        / cos_beta;
    let d_beta =
        -KAPPA_ARCSEC * beta.sin() * ((sun - lambda).sin() - e * (pi - lambda).sin());

    AberrationOffset { d_lambda_arcsec: d_lambda, d_beta_arcsec: d_beta }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magnitude_is_bounded_by_kappa_over_cos_beta() {
        // For modest latitudes Δλ stays within a few × κ; never explosive.
        let off = annual_aberration(100.0, 2.0, 280.0, 2_451_545.0);
        assert!(off.d_lambda_arcsec.abs() < 25.0, "Δλ = {}", off.d_lambda_arcsec);
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
        assert!(conj.d_lambda_arcsec < 0.0, "conjunction Δλ should be negative: {}", conj.d_lambda_arcsec);
        assert!((conj.d_lambda_arcsec - (-20.15)).abs() < 0.2, "conjunction Δλ = {}", conj.d_lambda_arcsec);

        // Opposition (body opposite the Sun, ⊙-λ = 180): Δλ ≈ +κ + 0.34 ≈ +20.84″.
        let opp = annual_aberration(100.0, 0.0, 280.0, 2_451_545.0);
        assert!(opp.d_lambda_arcsec > 0.0, "opposition Δλ should be positive: {}", opp.d_lambda_arcsec);
        assert!((opp.d_lambda_arcsec - 20.84).abs() < 0.2, "opposition Δλ = {}", opp.d_lambda_arcsec);

        // Quadrature (⊙-λ = 90): the main term vanishes; only the small e·κ term
        // remains (< 1″), and Δβ stays bounded by κ off the ecliptic.
        let quad = annual_aberration(100.0, 10.0, 190.0, 2_451_545.0);
        assert!(quad.d_lambda_arcsec.abs() < 1.0, "quadrature Δλ = {}", quad.d_lambda_arcsec);
        assert!(quad.d_beta_arcsec.abs() < KAPPA_ARCSEC, "quadrature Δβ = {}", quad.d_beta_arcsec);
    }
}
