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

mod long_term;

pub use long_term::{
    long_term_mean_obliquity_radians, precess_ecliptic_vector_date_to_j2000_long_term,
};

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
    let alpha_d = libm::atan2(
        libm::sin(lambda) * libm::cos(eps) - libm::tan(beta) * libm::sin(eps),
        libm::cos(lambda),
    );
    let delta_d = libm::asin(
        (libm::sin(beta) * libm::cos(eps) + libm::cos(beta) * libm::sin(eps) * libm::sin(lambda))
            .clamp(-1.0, 1.0),
    );

    // precess equatorial (of date) -> equatorial (J2000): inverse rotation,
    // ζ→−z, z→−ζ, θ→−θ (Meeus 21.4 reduction-to-J2000 form).
    let zeta_r = zeta.to_radians();
    let z_r = z.to_radians();
    let theta_r = theta.to_radians();
    let a = libm::cos(delta_d) * libm::sin(alpha_d - z_r);
    let b = libm::cos(theta_r) * libm::cos(delta_d) * libm::cos(alpha_d - z_r)
        + libm::sin(theta_r) * libm::sin(delta_d);
    let c = -libm::sin(theta_r) * libm::cos(delta_d) * libm::cos(alpha_d - z_r)
        + libm::cos(theta_r) * libm::sin(delta_d);
    let alpha0 = libm::atan2(a, b) - zeta_r;
    let delta0 = libm::asin(c.clamp(-1.0, 1.0));

    // equatorial (J2000) -> ecliptic (J2000), Meeus 13.1/13.2, ε₀.
    let eps0 = OBLIQUITY_J2000_DEG.to_radians();
    let lon = libm::atan2(
        libm::sin(alpha0) * libm::cos(eps0) + libm::tan(delta0) * libm::sin(eps0),
        libm::cos(alpha0),
    );
    let lat = libm::asin(
        (libm::sin(delta0) * libm::cos(eps0)
            - libm::cos(delta0) * libm::sin(eps0) * libm::sin(alpha0))
        .clamp(-1.0, 1.0),
    );

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
    let alpha0 = libm::atan2(
        libm::sin(lambda) * libm::cos(eps0) - libm::tan(beta) * libm::sin(eps0),
        libm::cos(lambda),
    );
    let delta0 = libm::asin(
        (libm::sin(beta) * libm::cos(eps0) + libm::cos(beta) * libm::sin(eps0) * libm::sin(lambda))
            .clamp(-1.0, 1.0),
    );

    // precess equatorial (J2000) -> equatorial (of date), Meeus 21.4.
    let zeta_r = zeta.to_radians();
    let z_r = z.to_radians();
    let theta_r = theta.to_radians();
    let a = libm::cos(delta0) * libm::sin(alpha0 + zeta_r);
    let b = libm::cos(theta_r) * libm::cos(delta0) * libm::cos(alpha0 + zeta_r)
        - libm::sin(theta_r) * libm::sin(delta0);
    let c = libm::sin(theta_r) * libm::cos(delta0) * libm::cos(alpha0 + zeta_r)
        + libm::cos(theta_r) * libm::sin(delta0);
    let alpha = libm::atan2(a, b) + z_r;
    let delta = libm::asin(c.clamp(-1.0, 1.0));

    // equatorial (of date) -> ecliptic (of date), Meeus 13.1/13.2, using the
    // mean obliquity OF DATE.
    let eps = mean_obliquity_degrees(jd_tt).to_radians();
    let lon = libm::atan2(
        libm::sin(alpha) * libm::cos(eps) + libm::tan(delta) * libm::sin(eps),
        libm::cos(alpha),
    );
    let lat = libm::asin(
        (libm::sin(delta) * libm::cos(eps) - libm::cos(delta) * libm::sin(eps) * libm::sin(alpha))
            .clamp(-1.0, 1.0),
    );

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

/// Rotates a Cartesian ecliptic vector `[x, y, z]` from the J2000 mean
/// equinox/ecliptic into the mean equinox/ecliptic of date `jd_tt`, preserving
/// its magnitude.
///
/// Precession is a rotation, so the same map applies to a position and to a
/// velocity vector alike; this is what lets an osculating lunar orbit be formed
/// in the of-date plane from a J2000 state (position and velocity differenced
/// in J2000, then both rotated once). The zero vector is returned unchanged.
///
/// # Errors
///
/// Returns [`ApparentPlaceError::NonFiniteCorrection`] if any component is
/// non-finite or the direction cannot be precessed.
pub fn precess_ecliptic_vector_j2000_to_date(
    v: [f64; 3],
    jd_tt: f64,
) -> Result<[f64; 3], ApparentPlaceError> {
    let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if !r.is_finite() {
        return Err(ApparentPlaceError::NonFiniteCorrection {
            stage: "precession",
        });
    }
    if r == 0.0 {
        return Ok(v);
    }
    let lon_deg = libm::atan2(v[1], v[0]).to_degrees().rem_euclid(360.0);
    let lat_deg = libm::asin((v[2] / r).clamp(-1.0, 1.0)).to_degrees();
    let p = precess_ecliptic_j2000_to_date(lon_deg, lat_deg, jd_tt)?;
    let (sl, cl) = libm::sincos(p.longitude_deg.to_radians());
    let (sb, cb) = libm::sincos(p.latitude_deg.to_radians());
    Ok([r * cb * cl, r * cb * sl, r * sb])
}

#[cfg(test)]
mod tests;
