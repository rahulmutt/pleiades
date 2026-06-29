//! Reads geocentric apparent Sun and Moon ecliptic positions from a backend.

// Items here are pub(crate) for upcoming eclipse-engine tasks; silence
// dead_code lint until those consumers land.
#![allow(dead_code)]

use crate::error::EclipseError;
use pleiades_backend::{EphemerisBackend, EphemerisRequest};
use pleiades_types::{
    Apparentness, CelestialBody, CoordinateFrame, Instant, JulianDay, TimeScale, ZodiacMode,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct SunMoonSample {
    pub sun_longitude_deg: f64,
    pub sun_latitude_deg: f64,
    pub sun_distance_au: f64,
    pub moon_longitude_deg: f64,
    pub moon_latitude_deg: f64,
    pub moon_distance_au: f64,
}

fn request(body: CelestialBody, julian_day: f64) -> EphemerisRequest {
    EphemerisRequest {
        body,
        instant: Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    }
}

fn read<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<(f64, f64, f64), EclipseError> {
    let result = backend
        .position(&request(body, julian_day))
        .map_err(|e| EclipseError::Backend(e.to_string()))?;
    let ecliptic = result.ecliptic.ok_or(EclipseError::MissingCoordinates {
        body_label,
        julian_day,
    })?;
    let distance = ecliptic
        .distance_au
        .ok_or(EclipseError::MissingCoordinates {
            body_label,
            julian_day,
        })?;
    Ok((
        ecliptic.longitude.degrees(),
        ecliptic.latitude.degrees(),
        distance,
    ))
}

pub(crate) fn sample_sun_moon<B: EphemerisBackend>(
    backend: &B,
    julian_day: f64,
) -> Result<SunMoonSample, EclipseError> {
    let (sun_longitude_deg, sun_latitude_deg, sun_distance_au) =
        read(backend, CelestialBody::Sun, "Sun", julian_day)?;
    let (moon_longitude_deg, moon_latitude_deg, moon_distance_au) =
        read(backend, CelestialBody::Moon, "Moon", julian_day)?;
    Ok(SunMoonSample {
        sun_longitude_deg,
        sun_latitude_deg,
        sun_distance_au,
        moon_longitude_deg,
        moon_latitude_deg,
        moon_distance_au,
    })
}

/// Signed Moon−Sun ecliptic longitude, wrapped into `(-180, 180]` degrees.
pub(crate) fn elongation_deg(sample: &SunMoonSample) -> f64 {
    let mut d = sample.moon_longitude_deg - sample.sun_longitude_deg;
    d = ((d + 180.0).rem_euclid(360.0)) - 180.0;
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::EclipseError;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn elongation_is_zero_at_new_moon_epoch() {
        // LinearSunMoon places Sun and Moon at equal longitude at jd0.
        let backend = LinearSunMoon::new_moon_at(2_451_550.0);
        let sample = sample_sun_moon(&backend, 2_451_550.0).unwrap();
        assert!(elongation_deg(&sample).abs() < 1e-6);
    }

    #[test]
    fn missing_coordinates_fail_closed() {
        let backend = LinearSunMoon::empty();
        let err = sample_sun_moon(&backend, 2_451_550.0).unwrap_err();
        assert!(matches!(err, EclipseError::MissingCoordinates { .. }));
    }
}
