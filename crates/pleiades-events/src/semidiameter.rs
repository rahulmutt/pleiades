//! Apparent semidiameter of a target's disc, for rise/set limb conventions.

use crate::rise_trans::RiseSetTarget;
use pleiades_types::CelestialBody;

/// Mean physical radii in AU-consistent units (radius_km / AU_km), matching the
/// Swiss Ephemeris disc-radius table. Sun and Moon dominate; planets are small
/// but nonzero for parity.
fn radius_au(body: &CelestialBody) -> f64 {
    // radius_km / 149_597_870.7
    match body {
        CelestialBody::Sun => 696_000.0 / 149_597_870.7,
        CelestialBody::Moon => 1_737.4 / 149_597_870.7,
        CelestialBody::Mercury => 2_439.7 / 149_597_870.7,
        CelestialBody::Venus => 6_051.8 / 149_597_870.7,
        CelestialBody::Mars => 3_389.5 / 149_597_870.7,
        CelestialBody::Jupiter => 69_911.0 / 149_597_870.7,
        CelestialBody::Saturn => 58_232.0 / 149_597_870.7,
        CelestialBody::Uranus => 25_362.0 / 149_597_870.7,
        CelestialBody::Neptune => 24_622.0 / 149_597_870.7,
        CelestialBody::Pluto => 1_188.3 / 149_597_870.7,
        _ => 0.0,
    }
}

/// Apparent semidiameter (degrees). `distance_au` is the topocentric distance;
/// `fixed` freezes SD at a mean 1-AU-normalised value (`SE_BIT_FIXED_DISC_SIZE`).
pub(crate) fn semidiameter_deg(target: &RiseSetTarget, distance_au: f64, fixed: bool) -> f64 {
    let r = match target {
        RiseSetTarget::Body(b) => radius_au(b),
        _ => 0.0, // ecliptic points and stars have no disc
    };
    if r == 0.0 {
        return 0.0;
    }
    let d = if fixed { 1.0 } else { distance_au.max(1e-9) };
    // Guard the asin domain: a pathologically tiny distance_au could push r/d
    // above 1.0 despite r/d << 1 for all real bodies. Clamp to keep this
    // never-NaN per the crate's fail-closed convention.
    (r / d).clamp(-1.0, 1.0).asin().to_degrees()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::CelestialBody;

    #[test]
    fn sun_semidiameter_is_about_16_arcmin_at_1_au() {
        let sd = semidiameter_deg(&RiseSetTarget::Body(CelestialBody::Sun), 1.0, false);
        assert!((sd - 0.2666).abs() < 0.01, "sun SD {sd} deg"); // ~16'
    }

    #[test]
    fn star_semidiameter_is_zero() {
        let sd = semidiameter_deg(&RiseSetTarget::FixedStar("Sirius".into()), 1.0, false);
        assert_eq!(sd, 0.0);
    }
}
