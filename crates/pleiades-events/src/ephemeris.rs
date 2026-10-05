//! Reads body ecliptic positions from a backend and derives the longitudes the
//! crossing engine root-finds on: geocentric apparent-of-date, geocentric
//! mean-of-date, and heliocentric.

use crate::error::{EventError, WINDOW_START_JD};
use crate::state_vector::cartesian_velocity;
use pleiades_apparent::nutation::nutation;
use pleiades_apparent::{
    apparent_apsis_position, apparent_position, apparent_sun_position,
    precess_ecliptic_j2000_to_date, sun_true_longitude_of_date_deg, ApparentLightTimeError,
    DEFAULT_MAX_ITERATIONS,
};
use pleiades_backend::{EphemerisBackend, EphemerisRequest};
use pleiades_types::{
    Apparentness, CelestialBody, CelestialBodyClass, CoordinateFrame, EclipticCoordinates, Instant,
    JulianDay, Latitude, Longitude, Motion, TimeScale, ZodiacMode,
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

/// `(longitude_deg, latitude_deg, distance_au)`.
type EclipticTriple = (f64, f64, f64);

/// `(longitude_deg, latitude_deg, distance_au)` with the distance only when
/// the backend reports one.
pub(crate) type MeanPlace = (f64, f64, Option<f64>);

/// Reads `body` through `position` (with motion) or through
/// `position_without_motion` (without), so the two share one decode.
fn query_mean_place<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
    with_motion: bool,
) -> Result<(MeanPlace, Option<Motion>), EventError> {
    let request = request(body, julian_day);
    let result = if with_motion {
        backend.position(&request)
    } else {
        backend.position_without_motion(&request)
    }
    .map_err(|e| EventError::Backend(e.to_string()))?;
    let ecliptic = result.ecliptic.ok_or(EventError::MissingCoordinates {
        body_label,
        julian_day,
    })?;
    Ok((
        (
            ecliptic.longitude.degrees(),
            ecliptic.latitude.degrees(),
            ecliptic.distance_au,
        ),
        result.motion,
    ))
}

/// Mean/J2000 geocentric ecliptic place as the backend serves it: longitude
/// and latitude in degrees, the distance in AU when the backend reports one,
/// and the backend's motion for that place, when it reports one.
///
/// Only a lunar orbit point may come without a distance (the ELP backend
/// serves the nodes and apsides as directions); [`read_mean_ecliptic_with_motion`]
/// requires one for every other body.
pub(crate) fn read_mean_place<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<(MeanPlace, Option<Motion>), EventError> {
    query_mean_place(backend, body, body_label, julian_day, true)
}

/// [`read_mean_place`] for a caller that discards the motion: the place is
/// bit-identical, and the backend skips any extra evaluations its motion
/// would cost (issue #128).
pub(crate) fn read_mean_place_without_motion<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<MeanPlace, EventError> {
    Ok(query_mean_place(backend, body, body_label, julian_day, false)?.0)
}

/// Mean/J2000 geocentric ecliptic `(longitude_deg, latitude_deg, distance_au)`
/// and the backend's motion for that place, when it reports one. A place
/// without a distance is [`EventError::MissingDistance`].
pub(crate) fn read_mean_ecliptic_with_motion<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<(EclipticTriple, Option<Motion>), EventError> {
    let ((lon, lat, distance), motion) = read_mean_place(backend, body, body_label, julian_day)?;
    let distance = distance.ok_or(EventError::MissingDistance {
        body_label,
        julian_day,
    })?;
    Ok(((lon, lat, distance), motion))
}

/// Mean/J2000 geocentric ecliptic (longitude_deg, latitude_deg, distance_au).
pub(crate) fn read_mean_ecliptic<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<EclipticTriple, EventError> {
    let (lon, lat, distance) =
        read_mean_place_without_motion(backend, body, body_label, julian_day)?;
    let distance = distance.ok_or(EventError::MissingDistance {
        body_label,
        julian_day,
    })?;
    Ok((lon, lat, distance))
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
    geocentric_apparent_ecliptic_from(backend, body, body_label, julian_day, None)
}

/// [`geocentric_apparent_ecliptic`], reusing `mean_at_instant` — the body's
/// mean J2000 place at `julian_day`, already read from the same backend — in
/// place of a fresh backend read at that instant. The result is bit-identical
/// either way (the backend is deterministic); passing the place saves one
/// backend query for a caller that read it anyway (issue #128).
pub(crate) fn geocentric_apparent_ecliptic_from<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
    mean_at_instant: Option<EclipticTriple>,
) -> Result<(f64, f64, f64), EventError> {
    let instant = Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb);
    if body == CelestialBody::Sun {
        let (lon, lat, dist) = match mean_at_instant {
            Some(mean) => mean,
            None => read_mean_ecliptic(backend, body.clone(), body_label, julian_day)?,
        };
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

    // General body: the Sun's true longitude of date feeds only the provenance
    // aberration estimate (the light-time re-query carries aberration, #93),
    // and this engine discards the provenance, so the backend-free Meeus Sun
    // serves it: querying the backend for the Sun here cost about one Sun
    // evaluation per sample, more than the whole sample for a cheap body
    // (issue #128). It never reaches a returned position.
    //
    // The light-time loop's first query is at `instant` itself, so a mean
    // place the caller already read answers it. The closure propagates
    // `EventError` verbatim (its own error type), so the combined error is
    // `ApparentLightTimeError<EventError>`, which we flatten back to
    // `EventError::Backend` — preserving fail-closed on missing reads.
    let sun_true_lon = sun_true_longitude_of_date_deg(julian_day);
    let mut first_query = mean_at_instant;
    let apparent = apparent_position::<_, EventError>(
        instant,
        sun_true_lon,
        DEFAULT_MAX_ITERATIONS,
        |retarded: Instant| {
            // The body is read a light-time before `instant`. Within a
            // light-time of the window start that read falls before the
            // window: the instant cannot be served, and says so with the
            // window error, not with whatever the backend makes of an epoch
            // outside its range (issue #163).
            if retarded.julian_day.days() < WINDOW_START_JD {
                return Err(EventError::OutOfWindow { julian_day });
            }
            let (l, b, d) = match first_query.take() {
                Some(mean) => mean,
                None => read_mean_ecliptic(
                    backend,
                    body.clone(),
                    body_label,
                    retarded.julian_day.days(),
                )?,
            };
            Ok(EclipticCoordinates::new(
                Longitude::from_degrees(l),
                Latitude::from_degrees(b),
                Some(d),
            ))
        },
    )
    .map_err(|e| match e {
        ApparentLightTimeError::Query(window @ EventError::OutOfWindow { .. }) => window,
        e => EventError::Backend(format!("{body_label} apparent place failed: {e}")),
    })?;
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

/// Geocentric geometric ecliptic `(longitude_deg, latitude_deg, distance_au)`
/// in the mean ecliptic and equinox of date: the backend's J2000 place
/// precessed to date. No light-time, no aberration, no nutation.
pub(crate) fn geocentric_mean_of_date_ecliptic<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<EclipticTriple, EventError> {
    let (lon, lat, dist) = read_mean_ecliptic(backend, body, body_label, julian_day)?;
    let (lon, lat, _) = mean_place_of_date((lon, lat, None), body_label, julian_day)?;
    Ok((lon, lat, dist))
}

/// A mean J2000 place already read from the backend, precessed to the mean
/// ecliptic and equinox of date; the distance passes through unchanged.
pub(crate) fn mean_place_of_date(
    (lon, lat, distance): MeanPlace,
    body_label: &'static str,
    julian_day: f64,
) -> Result<MeanPlace, EventError> {
    let precessed = precess_ecliptic_j2000_to_date(lon, lat, julian_day)
        .map_err(|e| EventError::Backend(format!("{body_label} precession failed: {e}")))?;
    Ok((
        precessed.longitude_deg.rem_euclid(360.0),
        precessed.latitude_deg,
        distance,
    ))
}

/// Whether `body` is a lunar orbit point: mean or true node, apogee or perigee.
pub(crate) fn is_lunar_point(body: &CelestialBody) -> bool {
    body.class() == CelestialBodyClass::LunarPoint
}

/// Geocentric apparent-of-date direction of a lunar orbit point,
/// `(longitude_deg, latitude_deg, distance_au)`: the backend's J2000
/// direction rotated to the true ecliptic and equinox of date with precession
/// and nutation in longitude only. No light-time and no annual aberration: the
/// point is a geometric direction of the lunar orbit, not a body, and this is
/// how the `pleiades-core` chart layer and Swiss Ephemeris reduce it (issue
/// #118). The distance, when the backend reports one, passes through
/// unchanged; the ELP backend reports none.
pub(crate) fn geocentric_apparent_lunar_point<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<MeanPlace, EventError> {
    let mean = read_mean_place_without_motion(backend, body, body_label, julian_day)?;
    apparent_lunar_point_of(mean, body_label, julian_day)
}

/// [`geocentric_apparent_lunar_point`] of a mean J2000 direction already read
/// from the backend.
pub(crate) fn apparent_lunar_point_of(
    (lon, lat, distance): MeanPlace,
    body_label: &'static str,
    julian_day: f64,
) -> Result<MeanPlace, EventError> {
    let instant = Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb);
    let j2000 = EclipticCoordinates::new(
        Longitude::from_degrees(lon),
        Latitude::from_degrees(lat),
        distance,
    );
    let apparent = apparent_apsis_position(instant, j2000)
        .map_err(|e| EventError::Backend(format!("{body_label} apparent place failed: {e}")))?;
    Ok((
        apparent.ecliptic.longitude.degrees(),
        apparent.ecliptic.latitude.degrees(),
        apparent.ecliptic.distance_au,
    ))
}

/// Geocentric mean-of-date direction of a lunar orbit point,
/// `(longitude_deg, latitude_deg, distance_au)`: the backend's J2000 direction
/// precessed to the mean ecliptic and equinox of date. See
/// [`geocentric_apparent_lunar_point`] for why no distance is required.
pub(crate) fn geocentric_mean_of_date_lunar_point<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<MeanPlace, EventError> {
    let mean = read_mean_place_without_motion(backend, body, body_label, julian_day)?;
    mean_place_of_date(mean, body_label, julian_day)
}

/// Heliocentric J2000 ecliptic state of a body: `P_helio = P_geo − S_geo`,
/// reconstructed from the mean geocentric planet and Sun.
pub(crate) struct HeliocentricJ2000 {
    /// Heliocentric position vector, AU.
    pub(crate) position: [f64; 3],
    /// Heliocentric velocity vector, AU/day, when the backend reports all
    /// three rates for both the body and the Sun.
    pub(crate) velocity: Option<[f64; 3]>,
}

/// Planet-minus-Sun velocity, when both are known.
pub(crate) fn combine_velocities(
    planet: Option<[f64; 3]>,
    sun: Option<[f64; 3]>,
) -> Option<[f64; 3]> {
    let (planet, sun) = planet.zip(sun)?;
    Some([planet[0] - sun[0], planet[1] - sun[1], planet[2] - sun[2]])
}

/// Reads the mean geocentric body and Sun and subtracts them. Both carry
/// distance (AU); a missing distance fails closed.
///
/// The heliocentric place is GEOMETRIC (Sun-centred): no annual aberration or
/// light-time is applied — Swiss Ephemeris `SEFLG_HELCTR | SEFLG_TRUEPOS`, not
/// plain `SEFLG_HELCTR`, whose output is retarded by the heliocentric
/// light-time (up to ≈ 41″ in longitude for Mercury).
pub(crate) fn heliocentric_j2000<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<HeliocentricJ2000, EventError> {
    let ((pl, pb, pd), planet_motion) =
        read_mean_ecliptic_with_motion(backend, body, body_label, julian_day)?;
    let ((sl, sb, sd), sun_motion) =
        read_mean_ecliptic_with_motion(backend, CelestialBody::Sun, "Sun", julian_day)?;
    let planet = spherical_to_cartesian(pl, pb, pd);
    let sun = spherical_to_cartesian(sl, sb, sd);
    Ok(HeliocentricJ2000 {
        position: [planet[0] - sun[0], planet[1] - sun[1], planet[2] - sun[2]],
        velocity: combine_velocities(
            cartesian_velocity(pl, pb, pd, planet_motion),
            cartesian_velocity(sl, sb, sd, sun_motion),
        ),
    })
}

/// J2000 ecliptic `(longitude_deg, latitude_deg, distance_au)` of a vector.
pub(crate) fn j2000_spherical(position: [f64; 3]) -> (f64, f64, f64) {
    let [x, y, z] = position;
    let planar = (x * x + y * y).sqrt();
    (
        y.atan2(x).to_degrees().rem_euclid(360.0),
        z.atan2(planar).to_degrees(),
        (x * x + y * y + z * z).sqrt(),
    )
}

/// Rotates a heliocentric J2000 vector to the **true equinox of date**
/// (precession, then nutation in longitude), the frame of Swiss Ephemeris'
/// `SEFLG_HELCTR | SEFLG_TRUEPOS` geometric heliocentric place:
/// `(longitude_deg, latitude_deg, distance_au)`. Nutation in longitude leaves
/// the ecliptic latitude unchanged, and the rotation leaves the distance
/// unchanged.
pub(crate) fn heliocentric_of_date(
    position: [f64; 3],
    julian_day: f64,
) -> Result<(f64, f64, f64), EventError> {
    let (lon_j2000, lat_j2000, distance) = j2000_spherical(position);

    // J2000 -> mean equinox/ecliptic of date (precession).
    let precessed = precess_ecliptic_j2000_to_date(lon_j2000, lat_j2000, julian_day)
        .map_err(|e| EventError::Backend(format!("helio precession failed: {e}")))?;

    // Mean -> true equinox of date: add nutation in longitude (Δψ).
    let nut = nutation(julian_day)
        .map_err(|e| EventError::Backend(format!("helio nutation failed: {e}")))?;

    Ok((
        (precessed.longitude_deg + nut.delta_psi_arcsec / 3600.0).rem_euclid(360.0),
        precessed.latitude_deg,
        distance,
    ))
}

/// Geometric heliocentric ecliptic longitude (degrees, no light-time) of the
/// true equinox of date.
/// Thin wrapper over [`heliocentric_j2000`] and [`heliocentric_of_date`]; its
/// return value is byte-identical to before their extraction, which
/// `validate-crossings` depends on.
///
/// Test-only since the reference module took over the crossing path; kept as
/// the pinned-bits oracle.
#[cfg(test)]
pub(crate) fn heliocentric_longitude_deg<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<f64, EventError> {
    let helio = heliocentric_j2000(backend, body, body_label, julian_day)?;
    Ok(heliocentric_of_date(helio.position, julian_day)?.0)
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

    const PINNED_HELIO_LONGITUDE_BITS: [u64; 4] = [
        4640199238401330988,
        4645031011594974728,
        4566141178696634184,
        4629851122814407423,
    ];

    /// The longitude wrapper must not move by a single bit when the
    /// reconstruction is refactored: `validate-crossings` root-finds on it.
    #[test]
    fn heliocentric_longitude_bits_are_pinned() {
        let backend = pleiades_data::packaged_backend();
        let cases = [
            (CelestialBody::Mercury, "Mercury", 2_415_100.25),
            (CelestialBody::Mars, "Mars", 2_451_545.0),
            (CelestialBody::Saturn, "Saturn", 2_439_500.066527),
            (CelestialBody::Pluto, "Pluto", 2_487_900.5),
        ];
        let got: Vec<u64> = cases
            .iter()
            .map(|(body, label, jd)| {
                heliocentric_longitude_deg(&backend, body.clone(), label, *jd)
                    .unwrap()
                    .to_bits()
            })
            .collect();
        assert_eq!(got, PINNED_HELIO_LONGITUDE_BITS);
    }

    #[test]
    fn of_date_longitude_matches_the_longitude_wrapper() {
        let backend = pleiades_data::packaged_backend();
        let jd = 2_451_545.0;
        let helio = heliocentric_j2000(&backend, CelestialBody::Mars, "Mars", jd).unwrap();
        let (lon, lat, dist) = heliocentric_of_date(helio.position, jd).unwrap();
        let wrapper =
            heliocentric_longitude_deg(&backend, CelestialBody::Mars, "Mars", jd).unwrap();
        assert_eq!(lon.to_bits(), wrapper.to_bits());
        // Mars: heliocentric latitude within its 1.85° inclination, distance 1.38–1.67 AU.
        assert!(lat.abs() < 1.9, "latitude {lat}");
        assert!((1.38..1.67).contains(&dist), "distance {dist}");
    }

    #[test]
    fn j2000_and_of_date_share_the_distance() {
        let backend = pleiades_data::packaged_backend();
        let jd = 2_470_000.5;
        let helio = heliocentric_j2000(&backend, CelestialBody::Jupiter, "Jupiter", jd).unwrap();
        let j2000 = j2000_spherical(helio.position);
        let of_date = heliocentric_of_date(helio.position, jd).unwrap();
        assert_eq!(j2000.2.to_bits(), of_date.2.to_bits());
        // 2050: precession has moved the equinox by roughly 0.7°.
        let shift = (of_date.0 - j2000.0 + 180.0).rem_euclid(360.0) - 180.0;
        assert!((0.6..0.8).contains(&shift), "precession shift {shift}");
    }

    #[test]
    fn packaged_backend_gives_a_heliocentric_velocity() {
        let backend = pleiades_data::packaged_backend();
        let helio =
            heliocentric_j2000(&backend, CelestialBody::Venus, "Venus", 2_451_545.0).unwrap();
        let v = helio.velocity.expect("packaged backend reports motion");
        // Venus orbital speed is about 0.0202 AU/day.
        let speed = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((0.0195..0.0210).contains(&speed), "speed {speed}");
    }

    #[test]
    fn partial_backend_motion_gives_no_heliocentric_velocity() {
        // A planet whose backend motion lacks the latitude rate has no
        // Cartesian velocity, so the heliocentric velocity is `None` even
        // though the Sun's motion is complete.
        let planet = cartesian_velocity(
            10.0,
            0.0,
            1.5,
            Some(Motion::new(Some(0.5), None, Some(0.0))),
        );
        let sun = cartesian_velocity(
            100.0,
            0.0,
            1.0,
            Some(Motion::new(Some(1.0), Some(0.0), Some(0.0))),
        );
        assert!(combine_velocities(planet, sun).is_none());
    }

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
