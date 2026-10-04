//! The mean-element Pluto path (issue #119).
//!
//! Pluto is the only body that goes through `heliocentric_coordinates`, so a
//! defect there has no other witness: before this module, nothing pinned a
//! Pluto longitude and the path was 110-114 deg from every published
//! ephemeris.

use super::*;
use crate::elements::OrbitalElements;

/// Swiss Ephemeris 2.10 (Moshier) geometric J2000 ecliptic place of Pluto,
/// quoted in issue #119: `(JD TT, longitude deg, latitude deg, distance AU)`.
///
/// The latitude and distance are the issue's apparent-of-date values; the
/// reduction to apparent place moves them by far less than the tolerances
/// below (precession tilts the latitude by seconds of arc, light-time changes
/// the distance in the fourth decimal).
const SWISS_EPHEMERIS_PLUTO_J2000: [(f64, f64, f64, f64); 2] = [
    (2_451_545.0, 251.46, 10.8552, 31.064),
    (2_460_763.5, 303.16, -3.4414, 35.639),
];

/// What JPL's mean Keplerian elements for Pluto deliver over the 20th-21st
/// centuries; the `Approximate` claim promises nothing tighter.
const LONGITUDE_TOLERANCE_DEG: f64 = 1.0;
const LATITUDE_TOLERANCE_DEG: f64 = 1.0;
const DISTANCE_TOLERANCE_AU: f64 = 0.5;

#[test]
fn mean_element_orbit_longitude_includes_the_argument_of_perihelion() {
    // A circular orbit in the ecliptic plane, node at 0 deg, perihelion at
    // 90 deg, body at perihelion: the heliocentric longitude is 90 deg. The
    // argument of perihelion is the only non-zero angle, so a path that drops
    // or mis-scales it cannot land there.
    let elements = OrbitalElements::new(0.0, 0.0, 90.0, 1.0, 0.0, 0.0);
    let coords = Vsop87Backend::heliocentric_coordinates(elements);
    let longitude = coords.yh.atan2(coords.xh).to_degrees().rem_euclid(360.0);
    assert_degrees_close(longitude, 90.0, 1e-9);
    assert_close(coords.xh.hypot(coords.yh), 1.0, 1e-12);
}

#[test]
fn pluto_mean_elements_land_within_a_degree_of_swiss_ephemeris() {
    let backend = Vsop87Backend::new();
    for (jd_tt, lon, lat, dist) in SWISS_EPHEMERIS_PLUTO_J2000 {
        let instant = Instant::new(pleiades_types::JulianDay::from_days(jd_tt), TimeScale::Tt);
        let result = backend
            .position(&mean_request_at(CelestialBody::Pluto, instant))
            .expect("Pluto query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        assert_eq!(result.quality, QualityAnnotation::Approximate);
        assert_degrees_close(ecliptic.longitude.degrees(), lon, LONGITUDE_TOLERANCE_DEG);
        assert_close(ecliptic.latitude.degrees(), lat, LATITUDE_TOLERANCE_DEG);
        assert_close(
            ecliptic.distance_au.expect("distance should exist"),
            dist,
            DISTANCE_TOLERANCE_AU,
        );
    }
}
