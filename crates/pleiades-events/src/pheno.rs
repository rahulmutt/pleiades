//! Illumination phenomena — Swiss Ephemeris `swe_pheno` analogue: phase angle,
//! illuminated fraction, elongation, apparent disc diameter, and apparent
//! magnitude. See [`EventEngine::pheno`](crate::EventEngine::pheno).

use pleiades_types::CelestialBody;

/// Phase, phase-angle, elongation, disc-diameter and magnitude of a body, as
/// computed by [`EventEngine::pheno`](crate::EventEngine::pheno). Geocentric
/// apparent-of-date.
#[derive(Clone, Debug, PartialEq)]
pub struct PhenoData {
    /// Sun–body–Earth phase angle, degrees in `[0, 180]`. Zero for the Sun.
    pub phase_angle_deg: f64,
    /// Illuminated fraction of the disc, `(1 + cos phase_angle) / 2` in
    /// `[0, 1]`. Zero for the Sun (Swiss Ephemeris leaves it unset — see the
    /// SP-5 plan §E2).
    pub phase_fraction: f64,
    /// Sun–Earth–body elongation, degrees in `[0, 180]`. Zero for the Sun.
    pub elongation_deg: f64,
    /// Apparent angular diameter of the disc, **degrees** (Swiss Ephemeris
    /// `attr[3]`; see the SP-5 plan §E1). Zero for bodies with no disc datum.
    pub apparent_diameter_deg: f64,
    /// Apparent visual magnitude. `Some` for the ten majors (Sun, Moon,
    /// Mercury–Pluto); `None` for any other body (no photometric model).
    pub apparent_magnitude: Option<f64>,
    /// The body served.
    pub body: CelestialBody,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::CelestialBody;

    #[test]
    fn pheno_data_is_constructible_and_holds_optional_magnitude() {
        let d = PhenoData {
            phase_angle_deg: 12.0,
            phase_fraction: 0.98,
            elongation_deg: 40.0,
            apparent_diameter_deg: 0.53,
            apparent_magnitude: Some(-4.4),
            body: CelestialBody::Venus,
        };
        assert_eq!(d.body, CelestialBody::Venus);
        assert_eq!(d.apparent_magnitude, Some(-4.4));
        let none = PhenoData {
            apparent_magnitude: None,
            ..d
        };
        assert!(none.apparent_magnitude.is_none());
    }
}
