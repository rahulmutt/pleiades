//! The apparent-star correction to a star-anchored ayanamsa (issue #164 (c)).

use pleiades_apparent::nutation::mean_obliquity_degrees;
use pleiades_apparent::{apparent_star_place, polar_projection_deg};
use pleiades_ayanamsa::{anchor_star_mean_place, star_anchor, AnchorProjection};
use pleiades_types::{Angle, Ayanamsa, Instant};

fn wrap180(degrees: f64) -> f64 {
    (degrees + 540.0).rem_euclid(360.0) - 180.0
}

/// What `ayanamsa` gains when read from its anchor star's apparent place
/// instead of its mean place, at `instant` (TT): the star's light deflection
/// and annual aberration as Swiss Ephemeris applies them under
/// `SEFLG_SIDEREAL` with apparent flags, within 0.06″ of it away from the
/// star's conjunction with the Sun and 0.5″ at δ Cnc's passage behind the
/// solar disc. Up to about 22″. `None` for an ayanamsa not anchored to a star.
///
/// [`SiderealStarPlace::Apparent`](pleiades_types::SiderealStarPlace) applies
/// it: the apparent sidereal longitude is the mean-equinox longitude less the
/// mean ayanamsa less this correction.
///
/// ```
/// use pleiades_core::apparent_star_ayanamsa_correction;
/// use pleiades_types::{Ayanamsa, Instant, JulianDay, TimeScale};
///
/// let j2000 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
/// let citra = apparent_star_ayanamsa_correction(&Ayanamsa::TrueCitra, j2000).unwrap();
/// assert!((citra.degrees() * 3600.0 + 4.85).abs() < 0.06); // Swiss Ephemeris: −4.8521″
/// assert!(apparent_star_ayanamsa_correction(&Ayanamsa::Lahiri, j2000).is_none());
/// ```
pub fn apparent_star_ayanamsa_correction(ayanamsa: &Ayanamsa, instant: Instant) -> Option<Angle> {
    let anchor = star_anchor(ayanamsa)?;
    let mean = anchor_star_mean_place(anchor.star, instant)?;
    let jd = instant.julian_day.days();
    let (lambda, beta) = apparent_star_place(mean.longitude_deg, mean.latitude_deg, jd);
    let degrees = match anchor.projection {
        AnchorProjection::EclipticLongitude => wrap180(lambda - mean.longitude_deg),
        AnchorProjection::PolarRightAscension => {
            let eps = mean_obliquity_degrees(jd);
            wrap180(
                polar_projection_deg(lambda, beta, eps)
                    - polar_projection_deg(mean.longitude_deg, mean.latitude_deg, eps),
            )
        }
        _ => return None,
    };
    Some(Angle::from_degrees(degrees))
}

#[cfg(test)]
mod tests;
