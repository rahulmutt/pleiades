//! The apparent place of a star at infinity: light deflection by the Sun,
//! then annual aberration, the order Swiss Ephemeris applies them. Used for
//! the anchor star of a star-anchored ayanamsa (issue #164 (c)).

use crate::aberration::{annual_aberration, sun_true_longitude_of_date_deg};
use crate::deflection::gravitational_deflection;

/// Apparent `(λ, β)` (degrees, mean ecliptic and equinox of date, no
/// nutation) of a star whose mean place is `(λ, β)` at `jd_tt`.
pub fn apparent_star_place(lambda_deg: f64, beta_deg: f64, jd_tt: f64) -> (f64, f64) {
    let d = gravitational_deflection(lambda_deg, beta_deg, jd_tt);
    let (l1, b1) = (
        lambda_deg + d.d_lambda_arcsec / 3600.0,
        beta_deg + d.d_beta_arcsec / 3600.0,
    );
    let a = annual_aberration(l1, b1, sun_true_longitude_of_date_deg(jd_tt), jd_tt);
    (
        (l1 + a.d_lambda_arcsec / 3600.0).rem_euclid(360.0),
        b1 + a.d_beta_arcsec / 3600.0,
    )
}

/// The ecliptic longitude whose right ascension equals that of `(λ, β)`,
/// degrees in `[0, 360)`: Swiss Ephemeris's `swi_armc_to_mc` applied to the
/// point's right ascension, `atan2(sin α, cos α · cos ε)`.
pub fn polar_projection_deg(lambda_deg: f64, beta_deg: f64, obliquity_deg: f64) -> f64 {
    let (l, b, eps) = (
        lambda_deg.to_radians(),
        beta_deg.to_radians(),
        obliquity_deg.to_radians(),
    );
    let ra = (l.sin() * eps.cos() - b.tan() * eps.sin()).atan2(l.cos());
    ra.sin()
        .atan2(ra.cos() * eps.cos())
        .to_degrees()
        .rem_euclid(360.0)
}

#[cfg(test)]
mod tests;
