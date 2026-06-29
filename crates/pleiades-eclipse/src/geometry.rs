//! Geocentric eclipse shadow-cone geometry and type classification.

// Items here are pub(crate) for upcoming eclipse-engine tasks; silence
// dead_code lint until those consumers land.
#![allow(dead_code)]

use crate::ephemeris::SunMoonSample;
use crate::types::SolarEclipseType;

pub(crate) mod constants {
    pub const R_SUN_KM: f64 = 696_000.0;
    pub const R_MOON_KM: f64 = 1_737.4;
    pub const R_EARTH_KM: f64 = 6_378.137;
    pub const AU_KM: f64 = 149_597_870.7;
    /// Geometric enlargement of Earth's shadow used by the NASA canon.
    pub const SHADOW_INFLATION: f64 = 1.02;
}

use constants::*;

const HYBRID_BAND_RAD: f64 = 0.000_03;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SolarCircumstances {
    pub eclipse_type: SolarEclipseType,
    pub magnitude: f64,
    pub gamma: f64,
}

/// Great-circle separation (radians) between Sun and Moon centers.
fn separation_rad(sample: &SunMoonSample) -> f64 {
    let (l1, b1) = (
        sample.sun_longitude_deg.to_radians(),
        sample.sun_latitude_deg.to_radians(),
    );
    let (l2, b2) = (
        sample.moon_longitude_deg.to_radians(),
        sample.moon_latitude_deg.to_radians(),
    );
    let cos_sep = (b1.sin() * b2.sin() + b1.cos() * b2.cos() * (l1 - l2).cos()).clamp(-1.0, 1.0);
    cos_sep.acos()
}

pub(crate) fn classify_solar(sample: &SunMoonSample) -> Option<SolarCircumstances> {
    let s = (R_SUN_KM / (sample.sun_distance_au * AU_KM)).asin();
    let m = (R_MOON_KM / (sample.moon_distance_au * AU_KM)).asin();
    let parallax = (R_EARTH_KM / (sample.moon_distance_au * AU_KM)).asin();
    let sigma = separation_rad(sample);

    if sigma >= parallax + s + m {
        return None; // penumbra misses Earth — no eclipse
    }

    let gamma = (sigma / parallax) * sample.moon_latitude_deg.signum();
    let magnitude = ((s + m - sigma) / (2.0 * s)).max(0.0);

    let central = sigma <= parallax;
    let eclipse_type = if !central {
        SolarEclipseType::Partial
    } else if (m - s).abs() < HYBRID_BAND_RAD {
        SolarEclipseType::Hybrid
    } else if m >= s {
        SolarEclipseType::Total
    } else {
        SolarEclipseType::Annular
    };

    Some(SolarCircumstances {
        eclipse_type,
        magnitude,
        gamma,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ephemeris::SunMoonSample;

    fn sample(moon_lat_deg: f64, moon_dist_au: f64, sun_dist_au: f64) -> SunMoonSample {
        SunMoonSample {
            sun_longitude_deg: 100.0,
            sun_latitude_deg: 0.0,
            sun_distance_au: sun_dist_au,
            moon_longitude_deg: 100.0,
            moon_latitude_deg: moon_lat_deg,
            moon_distance_au: moon_dist_au,
        }
    }

    #[test]
    fn central_close_moon_is_total() {
        // Moon near perigee (close → large disk), on the node → total.
        let c = classify_solar(&sample(0.0, 0.00238, 1.000)).unwrap();
        assert_eq!(c.eclipse_type, SolarEclipseType::Total);
        assert!(c.magnitude >= 1.0);
        assert!(c.gamma.abs() < 0.1);
    }

    #[test]
    fn central_far_moon_is_annular() {
        // Moon near apogee (far → small disk), on the node → annular.
        let c = classify_solar(&sample(0.0, 0.00271, 1.000)).unwrap();
        assert_eq!(c.eclipse_type, SolarEclipseType::Annular);
        assert!(c.magnitude < 1.0);
    }

    #[test]
    fn far_from_node_is_no_eclipse() {
        assert!(classify_solar(&sample(1.5, 0.00257, 1.000)).is_none());
    }
}
