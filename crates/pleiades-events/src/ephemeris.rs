//! Reads body ecliptic positions from a backend and derives the longitudes the
//! crossing engine root-finds on: geocentric apparent-of-date, and heliocentric.

// Sampling helpers are pub(crate) for the crossing engine (Task 4); silence the
// dead_code lint until those consumers land.
#![allow(dead_code)]

use crate::error::EventError;
use pleiades_apparent::nutation::nutation;
use pleiades_apparent::{
    apparent_position, apparent_sun_position, precess_ecliptic_j2000_to_date,
    DEFAULT_MAX_ITERATIONS,
};
use pleiades_backend::{EphemerisBackend, EphemerisRequest};
use pleiades_types::{
    Apparentness, CelestialBody, CoordinateFrame, EclipticCoordinates, Instant, JulianDay,
    Latitude, Longitude, TimeScale, ZodiacMode,
};

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

/// Mean/J2000 geocentric ecliptic (longitude_deg, latitude_deg, distance_au).
pub(crate) fn read_mean_ecliptic<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<(f64, f64, f64), EventError> {
    let result = backend
        .position(&request(body, julian_day))
        .map_err(|e| EventError::Backend(e.to_string()))?;
    let ecliptic = result.ecliptic.ok_or(EventError::MissingCoordinates {
        body_label,
        julian_day,
    })?;
    let distance = ecliptic.distance_au.ok_or(EventError::MissingCoordinates {
        body_label,
        julian_day,
    })?;
    Ok((
        ecliptic.longitude.degrees(),
        ecliptic.latitude.degrees(),
        distance,
    ))
}

/// Mean/J2000 geocentric ecliptic longitude only — for bodies whose backends
/// legitimately omit distance (the mean lunar points: `MeanNode`,
/// `MeanPerigee`). Latitude is read and discarded; distance is not required.
pub(crate) fn read_mean_longitude<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<f64, EventError> {
    let result = backend
        .position(&request(body, julian_day))
        .map_err(|e| EventError::Backend(e.to_string()))?;
    let ecliptic = result.ecliptic.ok_or(EventError::MissingCoordinates {
        body_label,
        julian_day,
    })?;
    Ok(ecliptic.longitude.degrees())
}

/// Geocentric apparent-of-date ecliptic (longitude_deg, latitude_deg, distance_au)
/// for a body. The Sun is a special case where light-time and annual aberration
/// are the same effect, so it uses `apparent_sun_position` (which applies
/// aberration exactly once); every other body uses the general
/// `apparent_position` light-time pipeline.
///
/// The apparent pipeline's `distance_au` is `Option<f64>`; bodies always carry
/// `Some` via the light-time/Sun pipeline, but a `None` is handled fail-closed
/// (returns `EventError::Backend`) rather than unwrapped.
pub(crate) fn geocentric_apparent_ecliptic<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<(f64, f64, f64), EventError> {
    let (lon, lat, dist) = read_mean_ecliptic(backend, body.clone(), body_label, julian_day)?;
    let instant = Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb);
    if body == CelestialBody::Sun {
        let j2000 = EclipticCoordinates::new(
            Longitude::from_degrees(lon),
            Latitude::from_degrees(lat),
            Some(dist),
        );
        let apparent = apparent_sun_position(instant, j2000)
            .map_err(|e| EventError::Backend(format!("Sun apparent place failed: {e}")))?;
        let distance_au = apparent.ecliptic.distance_au.ok_or_else(|| {
            EventError::Backend(format!("{body_label} apparent place missing distance"))
        })?;
        return Ok((
            apparent.ecliptic.longitude.degrees(),
            apparent.ecliptic.latitude.degrees(),
            distance_au,
        ));
    }

    // General body: apparent_position needs the Sun's true longitude of date for
    // the aberration term, plus a light-time-retarded body query closure. The
    // closure propagates `EventError` verbatim (its own error type), so the
    // combined error is `ApparentLightTimeError<EventError>`, which we flatten
    // back to `EventError::Backend` — preserving fail-closed on missing reads.
    let sun_true_lon =
        geocentric_apparent_longitude_deg(backend, CelestialBody::Sun, "Sun", julian_day)?;
    let apparent = apparent_position::<_, EventError>(
        instant,
        sun_true_lon,
        DEFAULT_MAX_ITERATIONS,
        |retarded: Instant| {
            let (l, b, d) = read_mean_ecliptic(
                backend,
                body.clone(),
                body_label,
                retarded.julian_day.days(),
            )?;
            Ok(EclipticCoordinates::new(
                Longitude::from_degrees(l),
                Latitude::from_degrees(b),
                Some(d),
            ))
        },
    )
    .map_err(|e| EventError::Backend(format!("{body_label} apparent place failed: {e}")))?;
    let distance_au = apparent.ecliptic.distance_au.ok_or_else(|| {
        EventError::Backend(format!("{body_label} apparent place missing distance"))
    })?;
    Ok((
        apparent.ecliptic.longitude.degrees(),
        apparent.ecliptic.latitude.degrees(),
        distance_au,
    ))
}

/// Geocentric apparent-of-date ecliptic longitude (degrees). Thin wrapper over
/// [`geocentric_apparent_ecliptic`] — kept so its return value stays
/// byte-identical to before that helper's extraction; `validate-crossings`
/// depends on this exact value.
pub(crate) fn geocentric_apparent_longitude_deg<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<f64, EventError> {
    Ok(geocentric_apparent_ecliptic(backend, body, body_label, julian_day)?.0)
}

/// Heliocentric ecliptic longitude (degrees) via `P_helio = P_geo − S_geo`,
/// reconstructed from the mean geocentric planet and Sun vectors, then rotated
/// from the J2000 ecliptic to the **true equinox of date** (precession +
/// nutation in longitude) to match SE's `SEFLG_HELCTR`. Both vectors carry
/// distance (AU); a missing distance fails closed.
///
/// The heliocentric position is GEOMETRIC (Sun-centered): no annual aberration
/// or light-time is applied — only the frame rotation to of-date.
pub(crate) fn heliocentric_longitude_deg<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<f64, EventError> {
    let (pl, pb, pd) = read_mean_ecliptic(backend, body, body_label, julian_day)?;
    let (sl, sb, sd) = read_mean_ecliptic(backend, CelestialBody::Sun, "Sun", julian_day)?;
    let planet = spherical_to_cartesian(pl, pb, pd);
    let sun = spherical_to_cartesian(sl, sb, sd);
    let helio = [planet[0] - sun[0], planet[1] - sun[1], planet[2] - sun[2]];

    // Heliocentric J2000 ecliptic (longitude, latitude).
    let lon_j2000 = helio[1].atan2(helio[0]).to_degrees().rem_euclid(360.0);
    let lat_j2000 = helio[2]
        .atan2((helio[0] * helio[0] + helio[1] * helio[1]).sqrt())
        .to_degrees();

    // J2000 -> mean equinox/ecliptic of date (precession).
    let precessed = precess_ecliptic_j2000_to_date(lon_j2000, lat_j2000, julian_day)
        .map_err(|e| EventError::Backend(format!("helio precession failed: {e}")))?;

    // Mean -> true equinox of date: add nutation in longitude (Δψ).
    let nut = nutation(julian_day)
        .map_err(|e| EventError::Backend(format!("helio nutation failed: {e}")))?;

    Ok((precessed.longitude_deg + nut.delta_psi_arcsec / 3600.0).rem_euclid(360.0))
}

pub(crate) fn spherical_to_cartesian(lon_deg: f64, lat_deg: f64, r_au: f64) -> [f64; 3] {
    let lon = lon_deg.to_radians();
    let lat = lat_deg.to_radians();
    [
        r_au * lat.cos() * lon.cos(),
        r_au * lat.cos() * lon.sin(),
        r_au * lat.sin(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn mean_read_returns_sun_longitude() {
        let backend = LinearSunMoon::new_moon_at(2_451_550.0);
        let (lon, _lat, dist) =
            read_mean_ecliptic(&backend, CelestialBody::Sun, "Sun", 2_451_550.0).unwrap();
        assert!(lon.is_finite());
        assert!(dist > 0.5 && dist < 1.5, "sun distance {dist}");
    }

    #[test]
    fn geocentric_apparent_sun_is_near_mean_but_shifted() {
        // Apparent-of-date longitude differs from mean/J2000 by precession +
        // aberration + nutation; at J2000-ish epochs the shift is small but real.
        let backend = LinearSunMoon::new_moon_at(2_451_550.0);
        let mean = read_mean_ecliptic(&backend, CelestialBody::Sun, "Sun", 2_451_550.0)
            .unwrap()
            .0;
        let app =
            geocentric_apparent_longitude_deg(&backend, CelestialBody::Sun, "Sun", 2_451_550.0)
                .unwrap();
        assert!(app.is_finite());
        assert!((app - mean).abs() < 1.0, "apparent-vs-mean {app} {mean}");
    }

    #[test]
    fn missing_coordinates_fail_closed() {
        let backend = LinearSunMoon::empty();
        let err = read_mean_ecliptic(&backend, CelestialBody::Sun, "Sun", 2_451_550.0).unwrap_err();
        assert!(matches!(err, EventError::MissingCoordinates { .. }));
    }

    #[test]
    fn heliocentric_reconstruction_subtracts_geocentric_sun() {
        // For the Sun-Moon mock there is no planet; assert the reconstruction math
        // on a synthetic pair via the exported helper is covered by the crossings
        // tests. Here just prove missing distance fails closed.
        let backend = LinearSunMoon::empty();
        let err = heliocentric_longitude_deg(&backend, CelestialBody::Mars, "Mars", 2_451_550.0)
            .unwrap_err();
        assert!(matches!(
            err,
            EventError::MissingCoordinates { .. } | EventError::Backend(_)
        ));
    }
}
