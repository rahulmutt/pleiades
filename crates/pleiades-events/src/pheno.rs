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

use crate::crossings::{body_label, EventEngine};
use crate::ephemeris::{geocentric_apparent_ecliptic, spherical_to_cartesian};
use crate::error::EventError;
use crate::magnitude::{apparent_magnitude, diameter_deg, MagInputs, AUNIT_M, CLIGHT_M_S};
use pleiades_backend::EphemerisBackend;
use pleiades_types::Instant;

/// `(longitude_deg, latitude_deg)` of a Cartesian ecliptic vector.
fn lon_lat_deg(v: [f64; 3]) -> (f64, f64) {
    let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let lon = v[1].atan2(v[0]).to_degrees().rem_euclid(360.0);
    let lat = if r == 0.0 {
        0.0
    } else {
        (v[2] / r).clamp(-1.0, 1.0).asin().to_degrees()
    };
    (lon, lat)
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Phase, phase angle, elongation, apparent disc diameter, and apparent
    /// magnitude of a body — Swiss Ephemeris `swe_pheno` analogue. Geocentric
    /// apparent-of-date. Full five-output parity for the ten majors (Sun, Moon,
    /// Mercury–Pluto); other backend-served bodies get the four geometric
    /// outputs with `apparent_magnitude = None`.
    ///
    /// Fail-closed: out-of-window instants return [`EventError::OutOfWindow`];
    /// missing backend coordinates return [`EventError::Backend`] /
    /// [`EventError::MissingCoordinates`]. Never returns NaN.
    pub fn pheno(&self, body: CelestialBody, instant: Instant) -> Result<PhenoData, EventError> {
        let jd = instant.julian_day.days();
        self.check_window(jd)?;

        // Geocentric apparent-of-date body and Sun.
        let (blon, blat, delta) =
            geocentric_apparent_ecliptic(&self.backend, body.clone(), body_label(&body), jd)?;
        let (slon, slat, r_sun) =
            geocentric_apparent_ecliptic(&self.backend, CelestialBody::Sun, "Sun", jd)?;

        // Heliocentric body = geocentric body − geocentric Sun (§E6).
        let bvec = spherical_to_cartesian(blon, blat, delta);
        let svec = spherical_to_cartesian(slon, slat, r_sun);
        let hvec = [bvec[0] - svec[0], bvec[1] - svec[1], bvec[2] - svec[2]];
        let r = (hvec[0] * hvec[0] + hvec[1] * hvec[1] + hvec[2] * hvec[2]).sqrt();
        let (hlon, hlat) = lon_lat_deg(hvec);

        let diameter = diameter_deg(&body, delta);
        let dt = delta * AUNIT_M / CLIGHT_M_S / 86_400.0;
        let mag_inputs = MagInputs {
            phase_angle_deg: 0.0, // set per-branch below
            r_helio_au: r,
            delta_geo_au: delta,
            diameter_deg: diameter,
            geo_lon_deg: blon,
            geo_lat_deg: blat,
            helio_lon_deg: hlon,
            helio_lat_deg: hlat,
            jd_tdb: jd,
            light_time_days: dt,
        };

        // The Sun is special: SE leaves phase/phase-angle/elongation at 0 (§E2).
        if body == CelestialBody::Sun {
            return Ok(PhenoData {
                phase_angle_deg: 0.0,
                phase_fraction: 0.0,
                elongation_deg: 0.0,
                apparent_diameter_deg: diameter,
                apparent_magnitude: apparent_magnitude(&body, &mag_inputs),
                body,
            });
        }

        // Phase angle and elongation via law of cosines (§E4).
        let cos_alpha =
            ((r * r + delta * delta - r_sun * r_sun) / (2.0 * r * delta)).clamp(-1.0, 1.0);
        let phase_angle_deg = cos_alpha.acos().to_degrees();
        let phase_fraction = (1.0 + cos_alpha) / 2.0;
        let cos_elong =
            ((delta * delta + r_sun * r_sun - r * r) / (2.0 * delta * r_sun)).clamp(-1.0, 1.0);
        let elongation_deg = cos_elong.acos().to_degrees();

        let magnitude_inputs = MagInputs {
            phase_angle_deg,
            ..mag_inputs
        };
        Ok(PhenoData {
            phase_angle_deg,
            phase_fraction,
            elongation_deg,
            apparent_diameter_deg: diameter,
            apparent_magnitude: apparent_magnitude(&body, &magnitude_inputs),
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crossings::EventEngine;
    use crate::error::EventError;
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

    fn tdb(jd: f64) -> Instant {
        Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
    }

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

    #[test]
    fn sun_reports_zero_phase_and_a_magnitude() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_545.0));
        let d = engine.pheno(CelestialBody::Sun, tdb(2_451_545.0)).unwrap();
        assert_eq!(d.phase_angle_deg, 0.0);
        assert_eq!(d.phase_fraction, 0.0); // SE leaves it 0 for the Sun (§E2)
        assert_eq!(d.elongation_deg, 0.0);
        assert!(d.apparent_diameter_deg > 0.4 && d.apparent_diameter_deg < 0.6);
        assert!(d.apparent_magnitude.unwrap() < -20.0);
    }

    #[test]
    fn moon_phase_fraction_is_in_unit_interval() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_545.0));
        let d = engine.pheno(CelestialBody::Moon, tdb(2_451_545.0)).unwrap();
        assert!(
            (0.0..=1.0).contains(&d.phase_fraction),
            "phase {}",
            d.phase_fraction
        );
        assert!((0.0..=180.0).contains(&d.phase_angle_deg));
        assert!(d.apparent_magnitude.is_some());
    }

    #[test]
    fn out_of_window_fails_closed() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_545.0));
        let err = engine
            .pheno(CelestialBody::Sun, tdb(2_400_000.5))
            .unwrap_err();
        assert!(matches!(err, EventError::OutOfWindow { .. }));
    }
}
