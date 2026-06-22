//! Precession of ecliptic coordinates from the J2000 mean equinox/ecliptic to
//! the mean equinox/ecliptic of date. IAU-1976 equatorial precession angles
//! (Meeus 20.3 / 21.4) are bridged through the ecliptic↔equatorial rotation
//! (Meeus 13.x): convert ecliptic-J2000 -> equatorial-J2000 with the J2000
//! obliquity, precess the equatorial coordinates, then convert back to ecliptic
//! using the mean obliquity OF DATE. The result is referred to the mean
//! equinox and ecliptic of date.

use crate::error::ApparentPlaceError;
use crate::nutation::mean_obliquity_degrees;

/// J2000 mean obliquity of the ecliptic, degrees. Matches `Instant::mean_obliquity`
/// and the value the backend used to produce its J2000 ecliptic coordinates, so
/// the inbound conversion is exactly the inverse of the backend's rotation.
const OBLIQUITY_J2000_DEG: f64 = 23.439_291_111_111_11;

fn julian_centuries(jd_tt: f64) -> f64 {
    (jd_tt - 2_451_545.0) / 36_525.0
}

/// Ecliptic longitude and latitude of date, degrees (longitude normalized to 0–360).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrecessedEcliptic {
    /// Ecliptic longitude referred to the mean equinox of date, degrees [0, 360).
    pub longitude_deg: f64,
    /// Ecliptic latitude referred to the mean ecliptic of date, degrees.
    pub latitude_deg: f64,
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
        return Err(ApparentPlaceError::NonFiniteCorrection { stage: "precession" });
    }
    Ok(PrecessedEcliptic { longitude_deg, latitude_deg })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_at_j2000() {
        // At J2000 the precession angles are zero and the inbound/outbound
        // obliquities are equal, so the transform is the identity.
        let out = precess_ecliptic_j2000_to_date(123.456, 4.5, 2_451_545.0).unwrap();
        assert!((out.longitude_deg - 123.456).abs() < 1e-6, "λ = {}", out.longitude_deg);
        assert!((out.latitude_deg - 4.5).abs() < 1e-6, "β = {}", out.latitude_deg);
    }

    #[test]
    fn general_precession_one_century() {
        // The J2000 vernal-equinox direction (λ=0, β=0) viewed in the
        // equinox-of-date frame one Julian century on has longitude ≈ the general
        // precession in longitude (5029.0966″/cy = 1.39697°). β stays small but
        // NOT exactly zero: the ecliptic plane itself precesses (~47″/cy), so a
        // point in the J2000 ecliptic acquires ≈ +4.4″ (0.00122°) of ecliptic-of-
        // date latitude. This is physically real and matches the rigorous Meeus
        // ch.21 ecliptic-precession result (4.39″) to sub-mas; the bound below is
        // widened from the naive 1e-3° to admit that residual while still catching
        // gross errors (a transcription bug would produce degrees, not arcsec).
        let jd = 2_451_545.0 + 36_525.0;
        let out = precess_ecliptic_j2000_to_date(0.0, 0.0, jd).unwrap();
        assert!((out.longitude_deg - 1.39697).abs() < 5e-3, "λ' = {}", out.longitude_deg);
        assert!(out.latitude_deg.abs() < 2e-3, "β' = {}", out.latitude_deg);
    }

    #[test]
    fn longitude_shifts_by_precession_off_the_ecliptic() {
        // For an off-ecliptic point, longitude still shifts by ≈ the general
        // precession over a century; latitude moves only slightly (ecliptic motion).
        let jd = 2_451_545.0 + 36_525.0;
        let out = precess_ecliptic_j2000_to_date(80.0, 30.0, jd).unwrap();
        let dlon = out.longitude_deg - 80.0;
        assert!((dlon - 1.397).abs() < 0.05, "Δλ = {dlon}");
        assert!((out.latitude_deg - 30.0).abs() < 0.05, "β' = {}", out.latitude_deg);
    }
}
