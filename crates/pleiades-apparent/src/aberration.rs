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

/// Earth's orbital eccentricity and longitude of perihelion ϖ (degrees),
/// both of date. Meeus 25.4.
///
/// Extracted from `annual_aberration` so the polynomial coefficients have a
/// direct test seam: they reach the public output only through the ~0.34″
/// `e κ cos(ϖ - λ)` term, where a coefficient error moves the result by
/// ~0.001-0.006″ — far below any tolerance the model's own accuracy justifies.
fn earth_orbit_elements(t: f64) -> (f64, f64) {
    let e = 0.016_708_634 - 0.000_042_037 * t - 0.000_000_126_7 * t * t;
    let pi_deg = 102.937_35 + 1.719_46 * t + 0.000_46 * t * t;
    (e, pi_deg)
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
    let (e, pi_deg) = earth_orbit_elements(julian_centuries(jd_tt));

    let lambda = lambda_deg.to_radians();
    let beta = beta_deg.to_radians();
    let sun = sun_true_longitude_deg.to_radians();
    let pi = pi_deg.to_radians();

    let cos_beta = beta.cos();
    let d_lambda =
        (-KAPPA_ARCSEC * (sun - lambda).cos() + e * KAPPA_ARCSEC * (pi - lambda).cos()) / cos_beta;
    let d_beta = -KAPPA_ARCSEC * beta.sin() * ((sun - lambda).sin() - e * (pi - lambda).sin());

    AberrationOffset {
        d_lambda_arcsec: d_lambda,
        d_beta_arcsec: d_beta,
    }
}

#[cfg(test)]
mod tests;
