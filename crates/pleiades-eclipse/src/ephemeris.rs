//! Reads geocentric Sun and Moon ecliptic positions from a backend.

// Items here are pub(crate) for upcoming eclipse-engine tasks; silence
// dead_code lint until those consumers land.
#![allow(dead_code)]

use crate::error::EclipseError;
use pleiades_apparent::{
    apparent_position, precess_ecliptic_j2000_to_date, DEFAULT_MAX_ITERATIONS,
};
use pleiades_backend::{EphemerisBackend, EphemerisRequest};
use pleiades_types::{
    Apparentness, CelestialBody, CoordinateFrame, EclipticCoordinates, Instant, JulianDay,
    Latitude, Longitude, TimeScale, ZodiacMode,
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

/// Builds a Mean-mode ecliptic request for the packaged backend.
///
/// # Design contract
///
/// All geometric sampling (separation, classification, sub-shadow point) uses
/// `Apparentness::Mean` because the packaged backend (`packaged_backend()`) only
/// supports mean geometric coordinates. This is valid for eclipse timing and type
/// classification because the Sun–Moon angular separation is frame-invariant: both
/// bodies shift together under precession and aberration, so the minimum is
/// unchanged. The `apparent: Mean` assumption binds this crate to backends that
/// return Mean/J2000 geocentric ecliptic coordinates; a backend that already
/// corrects to apparent-of-date would be mishandled. See also
/// `apparent_sun_longitude_deg`, which adds the apparent-place correction
/// to `eclipsed_longitude` via `pleiades-apparent` rather than the backend.
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

/// Computes the apparent geocentric solar ecliptic longitude of date (degrees).
///
/// Mirrors the validated apparent-place path in `pleiades-core` (see
/// `ChartEngine::query_sun_longitude_of_date` + `apparent_position` call):
///
/// 1. Query the Sun's Mean/J2000 geocentric ecliptic from the backend.
/// 2. Precess the J2000 longitude to of-date (→ Sun's true longitude of date)
///    for the annual aberration term.
/// 3. Run `apparent_position` (light-time, J2000→of-date precession, aberration,
///    nutation Δψ) with a query closure that hits the backend for the Sun's
///    Mean/J2000 position at each light-time-retarded epoch.
///
/// The result is the apparent ecliptic-of-date solar longitude, correcting for
/// ~20.5″ aberration, ~±17″ nutation, and sub-arcsecond light-time, so that
/// `Eclipse::eclipsed_longitude` meets the ≤1″ gate checked in Task 10.
///
/// **Assumption**: the backend returns Mean/J2000 geocentric coordinates
/// (as `packaged_backend()` does). A backend that already applies apparent
/// corrections would double-count them.
pub(crate) fn apparent_sun_longitude_deg<B: EphemerisBackend>(
    backend: &B,
    julian_day: f64,
) -> Result<f64, EclipseError> {
    let instant = Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb);

    // Step 1+2: get Sun's J2000 mean ecliptic and precess to of-date for the aberration term.
    let (sun_lon_j2000, sun_lat_j2000, _) = read(backend, CelestialBody::Sun, "Sun", julian_day)?;
    let precessed = precess_ecliptic_j2000_to_date(sun_lon_j2000, sun_lat_j2000, julian_day)
        .map_err(|e| EclipseError::Backend(format!("Sun precession failed: {e}")))?;
    let sun_true_longitude_of_date_deg = precessed.longitude_deg;

    // Step 3: compute full apparent place via pleiades-apparent (light-time, precession,
    // aberration, nutation). The query closure supplies the Sun's Mean/J2000 ecliptic at
    // each retarded epoch — exactly what packaged_backend() provides.
    let apparent = apparent_position::<_, EclipseError>(
        instant,
        sun_true_longitude_of_date_deg,
        DEFAULT_MAX_ITERATIONS,
        |instant_q| {
            let (lon, lat, dist) = read(
                backend,
                CelestialBody::Sun,
                "Sun",
                instant_q.julian_day.days(),
            )?;
            Ok(EclipticCoordinates::new(
                Longitude::from_degrees(lon),
                Latitude::from_degrees(lat),
                Some(dist),
            ))
        },
    )
    .map_err(|e| EclipseError::Backend(format!("apparent Sun position failed: {e}")))?;

    Ok(apparent.ecliptic.longitude.degrees())
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
