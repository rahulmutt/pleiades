//! Reads geocentric Sun and Moon ecliptic positions from a backend.

// Items here are pub(crate) for upcoming eclipse-engine tasks; silence
// dead_code lint until those consumers land.
#![allow(dead_code)]

use crate::error::EclipseError;
use pleiades_apparent::{apparent_sun_position, LIGHT_TIME_DAYS_PER_AU};
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

/// Reads a body's *apparent geometric* (light-time-retarded) geocentric ecliptic
/// position: the direction the body is actually seen, i.e. where it was one
/// light-time ago. NASA computes eclipse circumstances (and the greatest-eclipse
/// instant) from these retarded positions. Because the Sun's light-time (~499 s)
/// is ~390× the Moon's (~1.3 s), the retarded Sun lags its instantaneous place by
/// ~20.5″ while the Moon lags only ~0.7″; the net shifts the apparent conjunction
/// ~39 s earlier than the purely geometric one. Sampling positions this way is
/// what makes the engine's greatest-eclipse instant agree with NASA's to well
/// under the 60 s gate (and keeps `eclipsed_longitude` within 1″). One iteration
/// suffices: a body's distance is essentially constant across its own light-time.
///
/// Note: this is distinct from, and must not be combined with, the annual
/// aberration applied in [`apparent_sun_longitude_deg`] — for the Sun those are
/// the same physical effect, so `eclipsed_longitude` applies it exactly once and
/// uses the *un*-retarded [`read`].
fn read_retarded<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<(f64, f64, f64), EclipseError> {
    let (_, _, dist) = read(backend, body.clone(), body_label, julian_day)?;
    let light_time_days = dist * LIGHT_TIME_DAYS_PER_AU;
    read(backend, body, body_label, julian_day - light_time_days)
}

pub(crate) fn sample_sun_moon<B: EphemerisBackend>(
    backend: &B,
    julian_day: f64,
) -> Result<SunMoonSample, EclipseError> {
    let (sun_longitude_deg, sun_latitude_deg, sun_distance_au) =
        read_retarded(backend, CelestialBody::Sun, "Sun", julian_day)?;
    let (moon_longitude_deg, moon_latitude_deg, moon_distance_au) =
        read_retarded(backend, CelestialBody::Moon, "Moon", julian_day)?;
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

/// Computes the apparent geocentric solar ecliptic longitude of date (degrees)
/// by delegating to [`pleiades_apparent::apparent_sun_position`], which applies
/// annual aberration exactly once (no light-time re-query) — see that function
/// for why light-time and aberration are the same effect for the geocentric Sun.
///
/// # Why aberration is applied only once
///
/// For a planet, light-time retardation (re-querying the body at the epoch the
/// light left it) and annual aberration are two physically independent effects.
/// For the **Sun**, they are the *same* effect: the ~20.5″ displacement caused
/// by Earth's orbital velocity. Light-time retardation moves the Sun's geometric
/// place backward along its apparent path by exactly the annual-aberration amount,
/// so an apparent-place routine that applies a light-time re-query *and* a
/// separate annual-aberration term double-counts ~20.5″.
///
/// **Assumption**: the backend returns Mean/J2000 geocentric coordinates
/// (as `packaged_backend()` does). A backend that already applies apparent
/// corrections would double-count them.
pub(crate) fn apparent_sun_longitude_deg<B: EphemerisBackend>(
    backend: &B,
    julian_day: f64,
) -> Result<f64, EclipseError> {
    // Step 1: Sun's J2000 mean geocentric ecliptic at the true (un-retarded) epoch.
    let (sun_lon_j2000, sun_lat_j2000, sun_dist_au) =
        read(backend, CelestialBody::Sun, "Sun", julian_day)?;

    // Steps 2–4: precess → nutation → aberration ONCE, via the shared routine.
    // For the geocentric Sun, light-time retardation and annual aberration are
    // the same effect, so `apparent_sun_position` performs no light-time re-query.
    let sun_j2000 = EclipticCoordinates::new(
        Longitude::from_degrees(sun_lon_j2000),
        Latitude::from_degrees(sun_lat_j2000),
        Some(sun_dist_au),
    );
    let instant = Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb);
    let apparent = apparent_sun_position(instant, sun_j2000)
        .map_err(|e| EclipseError::Backend(format!("Sun apparent place failed: {e}")))?;
    Ok(apparent.ecliptic.longitude.degrees())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::EclipseError;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn elongation_reflects_light_time_at_geometric_new_moon() {
        // LinearSunMoon places Sun and Moon at equal *geometric* longitude at jd0.
        // `sample_sun_moon` now returns light-time-retarded (apparent) positions:
        // the Sun is retarded ~499 s (sun_rate·1 AU·LT ≈ 0.005694°) and the Moon
        // ~1.3 s (moon_rate·0.00257 AU·LT ≈ 0.000196°). The apparent elongation at
        // the geometric new moon is therefore the differential light-time offset
        // ≈ +0.005498°, not zero. (The apparent new moon itself lands ~39 s earlier
        // — see `syzygy::finds_the_new_moon_near_the_reference_epoch`.)
        let backend = LinearSunMoon::new_moon_at(2_451_550.0);
        let sample = sample_sun_moon(&backend, 2_451_550.0).unwrap();
        let e = elongation_deg(&sample);
        assert!((e - 0.005_498).abs() < 1e-4, "apparent elongation {e}");
    }

    #[test]
    fn missing_coordinates_fail_closed() {
        let backend = LinearSunMoon::empty();
        let err = sample_sun_moon(&backend, 2_451_550.0).unwrap_err();
        assert!(matches!(err, EclipseError::MissingCoordinates { .. }));
    }
}
