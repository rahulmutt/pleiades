//! Annual aberration in ecliptic coordinates (Meeus ch. 23, eq. 23.2).
//! Pure function: the caller supplies the body's ecliptic position and the
//! Sun's true longitude; this crate has no ephemeris of its own, apart from the low-precision Meeus Sun
//! below, which is accurate enough only for the aberration argument.

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

/// Sun's geometric (true) ecliptic longitude of date, degrees, via the Meeus
/// low-precision solar theory (Astronomical Algorithms, ch. 25).
///
/// Computes the geometric mean longitude `L0`, the mean anomaly `M`, and the
/// equation of the center `C`, then returns the true longitude `L0 + C`.
/// Accuracy is about 0.01°, which is far more than enough for the argument of
/// [`annual_aberration`]: the offset is at most κ ≈ 20.5″ and its sensitivity
/// to the Sun's longitude is a fraction of that per degree. Being
/// backend-free, it costs nothing next to an ephemeris query, so callers that
/// need the Sun only as the aberration argument (the fixed-star path, and the
/// provenance-only aberration estimate of an apparent body place, issue #128)
/// use it instead of querying a backend. `jd` is the (TT/TDB) Julian Day of
/// date.
pub fn sun_true_longitude_of_date_deg(jd: f64) -> f64 {
    let t = (jd - 2_451_545.0) / 36_525.0; // Julian centuries since J2000.0
                                           // Geometric mean longitude of the Sun (Meeus 25.2).
    let l0 = 280.466_46 + 36_000.769_83 * t + 0.000_303_2 * t * t;
    // Mean anomaly of the Sun (Meeus 25.3).
    let m = (357.529_11 + 35_999.050_29 * t - 0.000_153_7 * t * t).to_radians();
    // Equation of the center (Meeus, ch. 25).
    let c = (1.914_602 - 0.004_817 * t - 0.000_014 * t * t) * m.sin()
        + (0.019_993 - 0.000_101 * t) * (2.0 * m).sin()
        + 0.000_289 * (3.0 * m).sin();
    (l0 + c).rem_euclid(360.0)
}

/// The Sun's distance from the Earth, AU, via the same Meeus low-precision
/// theory (Astronomical Algorithms 25.5): `R = 1.000001018 (1 − e²) / (1 + e cos ν)`
/// with ν the true anomaly. Accurate to about 1e-5 AU, which is all the
/// light-deflection scale (∝ 1/R) needs. `jd` is the TT/TDB Julian Day.
pub fn sun_radius_vector_au_of_date(jd: f64) -> f64 {
    let t = (jd - 2_451_545.0) / 36_525.0;
    let m_deg = 357.529_11 + 35_999.050_29 * t - 0.000_153_7 * t * t;
    let m = m_deg.to_radians();
    let c = (1.914_602 - 0.004_817 * t - 0.000_014 * t * t) * m.sin()
        + (0.019_993 - 0.000_101 * t) * (2.0 * m).sin()
        + 0.000_289 * (3.0 * m).sin();
    let (e, _) = earth_orbit_elements(t);
    let nu = (m_deg + c).to_radians();
    1.000_001_018 * (1.0 - e * e) / (1.0 + e * nu.cos())
}

#[cfg(test)]
mod tests;
