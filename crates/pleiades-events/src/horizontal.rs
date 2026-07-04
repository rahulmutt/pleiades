//! Horizontal coordinates (`swe_azalt` / `swe_azalt_rev`): azimuth and altitude
//! of a target for a topocentric observer. Azimuth is measured from SOUTH,
//! increasing WESTWARD, degrees `[0,360)`, matching Swiss Ephemeris.

use crate::crossings::EventEngine;
use crate::error::EventError;
use pleiades_apparent::{apparent_from_true, sidereal_time, true_obliquity_degrees, Atmosphere};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{Angle, EclipticCoordinates, Instant, Latitude, Longitude, ObserverLocation};

/// Azimuth (from south, westward) plus true (geometric) and apparent (refracted)
/// altitude, all degrees.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct Horizontal {
    /// Azimuth measured from south, increasing westward, `[0,360)` degrees.
    pub azimuth: f64,
    /// Geometric (unrefracted) altitude, degrees.
    pub true_altitude: f64,
    /// Apparent (refracted) altitude, degrees.
    pub apparent_altitude: f64,
}

/// Input coordinate for [`EventEngine::horizontal`].
#[derive(Clone, Copy, Debug)]
pub enum HorizontalInput {
    /// Tropical apparent ecliptic of date (`SE_ECL2HOR`): longitude, latitude.
    Ecliptic(Longitude, Latitude),
    /// Apparent equatorial of date (`SE_EQU2HOR`): right ascension, declination.
    Equatorial(Angle, Latitude),
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Azimuth/altitude of `input` for `observer` at `at` (TDB).
    pub fn horizontal(
        &self,
        input: HorizontalInput,
        observer: ObserverLocation,
        atmos: Atmosphere,
        at: Instant,
    ) -> Result<Horizontal, EventError> {
        observer
            .validate()
            .map_err(|e| EventError::InvalidObserver {
                detail: e.to_string(),
            })?;
        let jd = at.julian_day.days();
        let eps = true_obliquity_degrees(jd)
            .map_err(|e| EventError::Backend(format!("obliquity failed: {e}")))?;
        // Resolve to apparent equatorial RA/Dec (degrees).
        let (ra_deg, dec_deg) = match input {
            HorizontalInput::Equatorial(ra, dec) => (ra.degrees(), dec.degrees()),
            HorizontalInput::Ecliptic(lon, lat) => {
                let ecl = EclipticCoordinates::new(lon, lat, None);
                let equ = ecl.to_equatorial(Angle::from_degrees(eps));
                (equ.right_ascension.degrees(), equ.declination.degrees())
            }
        };
        // Local apparent sidereal time → local hour angle H = LST − RA.
        let lst = sidereal_time(at, observer.longitude).local_apparent_deg;
        let h_deg = lst - ra_deg;
        let (h, dec, phi) = (
            h_deg.to_radians(),
            dec_deg.to_radians(),
            observer.latitude.degrees().to_radians(),
        );
        // Standard equatorial → horizontal rotation (azimuth from south, west +).
        let sin_alt = phi.sin() * dec.sin() + phi.cos() * dec.cos() * h.cos();
        let alt = sin_alt.asin();
        let az = (h.sin()).atan2(h.cos() * phi.sin() - dec.tan() * phi.cos());
        let true_altitude = alt.to_degrees();
        Ok(Horizontal {
            azimuth: az.to_degrees().rem_euclid(360.0),
            true_altitude,
            apparent_altitude: apparent_from_true(true_altitude, atmos),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_apparent::Atmosphere;
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{
        Angle, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
    };

    fn tdb(jd: f64) -> Instant {
        Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
    }
    fn greenwich() -> ObserverLocation {
        ObserverLocation::new(
            Latitude::from_degrees(51.48),
            Longitude::from_degrees(0.0),
            None,
        )
    }

    #[test]
    fn object_on_local_meridian_has_azimuth_zero_or_180() {
        // A body whose RA equals the local apparent sidereal time is on the meridian:
        // hour angle 0 → azimuth 0 (south) if it is south of zenith.
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let at = tdb(2_451_545.0);
        let st = pleiades_apparent::sidereal_time(at, Longitude::from_degrees(0.0));
        let ra = Angle::from_degrees(st.local_apparent_deg);
        let dec = Latitude::from_degrees(10.0); // south of a 51°N observer's zenith
        let h = engine
            .horizontal(
                HorizontalInput::Equatorial(ra, dec),
                greenwich(),
                Atmosphere::default(),
                at,
            )
            .unwrap();
        assert!(
            h.azimuth.abs() < 1e-3 || (h.azimuth - 360.0).abs() < 1e-3,
            "az {}",
            h.azimuth
        );
        assert!(
            h.apparent_altitude >= h.true_altitude,
            "refraction lifts the body"
        );
    }

    #[test]
    fn altitude_never_exceeds_ninety() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let at = tdb(2_451_545.0);
        let h = engine
            .horizontal(
                HorizontalInput::Equatorial(
                    Angle::from_degrees(0.0),
                    Latitude::from_degrees(51.48),
                ),
                greenwich(),
                Atmosphere::default(),
                at,
            )
            .unwrap();
        assert!(h.true_altitude <= 90.0 + 1e-9);
    }
}
