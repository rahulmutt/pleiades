//! Gravitational deflection of starlight by the Sun, as Swiss Ephemeris
//! computes it (`sweph.c` `swi_deflect_light`, Explanatory Supplement p. 136)
//! for a source at infinity, with the Sun's position from the Meeus theory.
//! Inside the solar disc the Sun's mass is tapered by [`meff`](meff::meff)
//! so the deflection stays finite, as Swiss Ephemeris does.

use crate::aberration::{sun_radius_vector_au_of_date, sun_true_longitude_of_date_deg};

mod meff;

/// 2·G·M☉ / (c² · 1 AU), radians: the deflection scale at 1 AU
/// (`HELGRAVCONST` 1.32712440017987e20 m³/s², `CLIGHT` 2.99792458e8 m/s,
/// `AUNIT` 1.4959787070e11 m, as in Swiss Ephemeris).
const SCALE_RAD_AT_1_AU: f64 =
    2.0 * 1.327_124_400_179_87e20 / (2.997_924_58e8 * 2.997_924_58e8) / 1.495_978_707_00e11;
/// Solar radius at 1 AU, radians (`SUN_RADIUS`, 959.63″).
const SUN_RADIUS_RAD_AT_1_AU: f64 = 959.63 / 3600.0 * std::f64::consts::PI / 180.0;

/// Deflection offset in ecliptic longitude and latitude, arcseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeflectionOffset {
    /// Δλ, arcseconds.
    pub d_lambda_arcsec: f64,
    /// Δβ, arcseconds.
    pub d_beta_arcsec: f64,
}

fn unit(lambda_deg: f64, beta_deg: f64) -> [f64; 3] {
    let (l, b) = (lambda_deg.to_radians(), beta_deg.to_radians());
    [b.cos() * l.cos(), b.cos() * l.sin(), b.sin()]
}

/// The Sun's deflection of light from a star at ecliptic `(λ, β)` (mean
/// ecliptic and equinox of date, degrees) at `jd_tt`:
/// `u' = u + g1/(1 + u·e) · (e − (u·e) u)`, `e` the Sun-to-Earth unit vector,
/// `g1 = SCALE · meff / R`. The star is pushed away from the Sun by
/// `g1 · cot(ψ/2)`, ψ the elongation: 0.0041″ at quadrature, 1.75″ at the
/// limb.
pub fn gravitational_deflection(lambda_deg: f64, beta_deg: f64, jd_tt: f64) -> DeflectionOffset {
    let r = sun_radius_vector_au_of_date(jd_tt);
    let u = unit(lambda_deg, beta_deg);
    let e = unit(sun_true_longitude_of_date_deg(jd_tt) + 180.0, 0.0);
    let ue = u[0] * e[0] + u[1] * e[1] + u[2] * e[2];
    let sin_elongation = (1.0 - ue * ue).max(0.0).sqrt();
    let sin_sun_radius = SUN_RADIUS_RAD_AT_1_AU / r;
    let meff = if sin_elongation < sin_sun_radius {
        meff::meff(sin_elongation / sin_sun_radius)
    } else {
        1.0
    };
    let g1 = SCALE_RAD_AT_1_AU * meff / r;
    let g2 = 1.0 + ue;
    if g1 == 0.0 || g2 <= 0.0 {
        // At the Sun's centre (meff = 0) nothing moves.
        return DeflectionOffset {
            d_lambda_arcsec: 0.0,
            d_beta_arcsec: 0.0,
        };
    }
    let d: [f64; 3] = std::array::from_fn(|i| u[i] + g1 / g2 * (e[i] - ue * u[i]));
    let lambda = d[1].atan2(d[0]).to_degrees();
    let beta = d[2].atan2(d[0].hypot(d[1])).to_degrees();
    DeflectionOffset {
        d_lambda_arcsec: ((lambda - lambda_deg + 540.0).rem_euclid(360.0) - 180.0) * 3600.0,
        d_beta_arcsec: (beta - beta_deg) * 3600.0,
    }
}

#[cfg(test)]
mod tests;
