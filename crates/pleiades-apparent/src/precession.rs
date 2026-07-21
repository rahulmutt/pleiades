//! Precession of ecliptic coordinates from the J2000 mean equinox/ecliptic to
//! the mean equinox/ecliptic of date. IAU-1976 equatorial precession angles
//! (Meeus 20.3 / 21.4) are bridged through the ecliptic↔equatorial rotation
//! (Meeus 13.x): convert ecliptic-J2000 -> equatorial-J2000 with the J2000
//! obliquity, precess the equatorial coordinates, then convert back to ecliptic
//! using the mean obliquity of date. The forward transform's result is
//! referred to the mean equinox and ecliptic of date; the inverse
//! (`precess_ecliptic_date_to_j2000`) returns J2000 coordinates.

use crate::error::ApparentPlaceError;
use crate::nutation::mean_obliquity_degrees;
use pleiades_types::OBLIQUITY_J2000_DEG;

fn julian_centuries(jd_tt: f64) -> f64 {
    (jd_tt - 2_451_545.0) / 36_525.0
}

/// Ecliptic longitude and latitude in the caller-selected frame (mean equinox
/// of date or J2000), degrees (longitude normalized to 0–360).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrecessedEcliptic {
    /// Ecliptic longitude in the caller-selected frame (mean equinox of date or J2000), degrees [0, 360).
    pub longitude_deg: f64,
    /// Ecliptic latitude in the caller-selected frame (mean ecliptic of date or J2000), degrees.
    pub latitude_deg: f64,
}

/// Precesses geocentric ecliptic coordinates from the mean equinox/ecliptic of
/// date `jd_tt` back to the J2000 mean equinox/ecliptic. Algebraic inverse of
/// [`precess_ecliptic_j2000_to_date`] (round-trips to < 1e-6° at ~1 century
/// from J2000; verified at 1900).
pub fn precess_ecliptic_date_to_j2000(
    lambda_deg: f64,
    beta_deg: f64,
    jd_tt: f64,
) -> Result<PrecessedEcliptic, ApparentPlaceError> {
    let t = julian_centuries(jd_tt);
    let zeta = (2306.2181 * t + 0.30188 * t * t + 0.017998 * t * t * t) / 3600.0;
    let z = (2306.2181 * t + 1.09468 * t * t + 0.018203 * t * t * t) / 3600.0;
    let theta = (2004.3109 * t - 0.42665 * t * t - 0.041833 * t * t * t) / 3600.0;

    // ecliptic (of date) -> equatorial (of date), Meeus 13.3/13.4, ε_date.
    let eps = mean_obliquity_degrees(jd_tt).to_radians();
    let lambda = lambda_deg.to_radians();
    let beta = beta_deg.to_radians();
    let alpha_d = (lambda.sin() * eps.cos() - beta.tan() * eps.sin()).atan2(lambda.cos());
    let delta_d = (beta.sin() * eps.cos() + beta.cos() * eps.sin() * lambda.sin())
        .clamp(-1.0, 1.0)
        .asin();

    // precess equatorial (of date) -> equatorial (J2000): inverse rotation,
    // ζ→−z, z→−ζ, θ→−θ (Meeus 21.4 reduction-to-J2000 form).
    let zeta_r = zeta.to_radians();
    let z_r = z.to_radians();
    let theta_r = theta.to_radians();
    let a = delta_d.cos() * (alpha_d - z_r).sin();
    let b = theta_r.cos() * delta_d.cos() * (alpha_d - z_r).cos() + theta_r.sin() * delta_d.sin();
    let c = -theta_r.sin() * delta_d.cos() * (alpha_d - z_r).cos() + theta_r.cos() * delta_d.sin();
    let alpha0 = a.atan2(b) - zeta_r;
    let delta0 = c.clamp(-1.0, 1.0).asin();

    // equatorial (J2000) -> ecliptic (J2000), Meeus 13.1/13.2, ε₀.
    let eps0 = OBLIQUITY_J2000_DEG.to_radians();
    let lon = (alpha0.sin() * eps0.cos() + delta0.tan() * eps0.sin()).atan2(alpha0.cos());
    let lat = (delta0.sin() * eps0.cos() - delta0.cos() * eps0.sin() * alpha0.sin())
        .clamp(-1.0, 1.0)
        .asin();

    let longitude_deg = lon.to_degrees().rem_euclid(360.0);
    let latitude_deg = lat.to_degrees();
    if !longitude_deg.is_finite() || !latitude_deg.is_finite() {
        return Err(ApparentPlaceError::NonFiniteCorrection {
            stage: "precession",
        });
    }
    Ok(PrecessedEcliptic {
        longitude_deg,
        latitude_deg,
    })
}

/// Precesses geocentric ecliptic coordinates from the J2000 mean equinox/ecliptic
/// to the mean equinox/ecliptic of date `jd_tt`.
pub fn precess_ecliptic_j2000_to_date(
    lambda_deg: f64,
    beta_deg: f64,
    jd_tt: f64,
) -> Result<PrecessedEcliptic, ApparentPlaceError> {
    let t = julian_centuries(jd_tt);
    // IAU-1976 precession angles for a J2000 starting epoch (Meeus 20.3),
    // arcseconds -> degrees.
    let zeta = (2306.2181 * t + 0.30188 * t * t + 0.017998 * t * t * t) / 3600.0;
    let z = (2306.2181 * t + 1.09468 * t * t + 0.018203 * t * t * t) / 3600.0;
    let theta = (2004.3109 * t - 0.42665 * t * t - 0.041833 * t * t * t) / 3600.0;

    // ecliptic (J2000) -> equatorial (J2000), Meeus 13.3/13.4.
    let eps0 = OBLIQUITY_J2000_DEG.to_radians();
    let lambda = lambda_deg.to_radians();
    let beta = beta_deg.to_radians();
    let alpha0 = (lambda.sin() * eps0.cos() - beta.tan() * eps0.sin()).atan2(lambda.cos());
    let delta0 = (beta.sin() * eps0.cos() + beta.cos() * eps0.sin() * lambda.sin())
        .clamp(-1.0, 1.0)
        .asin();

    // precess equatorial (J2000) -> equatorial (of date), Meeus 21.4.
    let zeta_r = zeta.to_radians();
    let z_r = z.to_radians();
    let theta_r = theta.to_radians();
    let a = delta0.cos() * (alpha0 + zeta_r).sin();
    let b = theta_r.cos() * delta0.cos() * (alpha0 + zeta_r).cos() - theta_r.sin() * delta0.sin();
    let c = theta_r.sin() * delta0.cos() * (alpha0 + zeta_r).cos() + theta_r.cos() * delta0.sin();
    let alpha = a.atan2(b) + z_r;
    let delta = c.clamp(-1.0, 1.0).asin();

    // equatorial (of date) -> ecliptic (of date), Meeus 13.1/13.2, using the
    // mean obliquity OF DATE.
    let eps = mean_obliquity_degrees(jd_tt).to_radians();
    let lon = (alpha.sin() * eps.cos() + delta.tan() * eps.sin()).atan2(alpha.cos());
    let lat = (delta.sin() * eps.cos() - delta.cos() * eps.sin() * alpha.sin())
        .clamp(-1.0, 1.0)
        .asin();

    let longitude_deg = lon.to_degrees().rem_euclid(360.0);
    let latitude_deg = lat.to_degrees();
    if !longitude_deg.is_finite() || !latitude_deg.is_finite() {
        return Err(ApparentPlaceError::NonFiniteCorrection {
            stage: "precession",
        });
    }
    Ok(PrecessedEcliptic {
        longitude_deg,
        latitude_deg,
    })
}

#[cfg(test)]
mod tests;
