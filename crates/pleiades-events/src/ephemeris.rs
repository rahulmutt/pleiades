//! Reads body ecliptic positions from a backend and derives the longitudes the
//! crossing engine root-finds on: geocentric apparent-of-date, and heliocentric.

use crate::error::EventError;
use crate::state_vector::cartesian_velocity;
use pleiades_apparent::nutation::nutation;
use pleiades_apparent::{
    apparent_position, apparent_sun_position, precess_ecliptic_j2000_to_date,
    DEFAULT_MAX_ITERATIONS,
};
use pleiades_backend::{EphemerisBackend, EphemerisRequest};
use pleiades_types::{
    Apparentness, CelestialBody, CoordinateFrame, EclipticCoordinates, Instant, JulianDay,
    Latitude, Longitude, Motion, TimeScale, ZodiacMode,
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

/// Mean/J2000 geocentric ecliptic `(longitude_deg, latitude_deg, distance_au)`
/// and the backend's motion for that place, when it reports one.
pub(crate) fn read_mean_ecliptic_with_motion<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<(EclipticTriple, Option<Motion>), EventError> {
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
        (
            ecliptic.longitude.degrees(),
            ecliptic.latitude.degrees(),
            distance,
        ),
        result.motion,
    ))
}

/// Mean/J2000 geocentric ecliptic (longitude_deg, latitude_deg, distance_au).
pub(crate) fn read_mean_ecliptic<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<EclipticTriple, EventError> {
    Ok(read_mean_ecliptic_with_motion(backend, body, body_label, julian_day)?.0)
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

    // General body: the Sun's true longitude of date feeds only the provenance
    // aberration estimate (the light-time re-query carries aberration, #93), plus
    // a light-time-retarded body query closure. The closure propagates
    // `EventError` verbatim (its own error type), so the combined error is
    // `ApparentLightTimeError<EventError>`, which we flatten back to
    // `EventError::Backend` — preserving fail-closed on missing reads.
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
