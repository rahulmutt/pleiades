//! White-box unit tests for the topocentric module.
//!
//! Relocated out of `topocentric.rs` per AGENTS.md ("keep large inline test
//! suites out of the file under test"). These remain white-box unit tests with
//! access to the module's internals — they are deliberately not converted to
//! black-box integration tests.

use super::*;

fn ecl(lon: f64, lat: f64, dist: f64) -> EclipticCoordinates {
    EclipticCoordinates::new(
        Longitude::from_degrees(lon),
        Latitude::from_degrees(lat),
        Some(dist),
    )
}

fn observer(lat: f64) -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(lat),
        Longitude::from_degrees(0.0),
        Some(0.0),
    )
}

#[test]
fn moon_parallax_is_about_one_degree() {
    // Moon at ~0.00257 AU (60.3 Earth radii). For an observer with the Moon
    // near the horizon the parallax approaches ~0.95°. Assert it is large.
    let out = topocentric_position(ecl(100.0, 0.0, 0.002_57), &observer(0.0), 100.0, 23.4).unwrap();
    let shift = out
        .provenance
        .parallax_longitude_arcsec
        .hypot(out.provenance.parallax_latitude_arcsec)
        / 3600.0;
    assert!(shift > 0.3, "moon parallax {shift}° too small");
}

#[test]
fn distant_body_parallax_is_negligible() {
    // A body at 30 AU: parallax < 1".
    let out = topocentric_position(ecl(100.0, 0.0, 30.0), &observer(0.0), 100.0, 23.4).unwrap();
    let shift = out
        .provenance
        .parallax_longitude_arcsec
        .hypot(out.provenance.parallax_latitude_arcsec);
    assert!(shift < 1.0, "distant parallax {shift}\" too large");
}

#[test]
fn missing_distance_errors() {
    let no_dist = EclipticCoordinates::new(
        Longitude::from_degrees(100.0),
        Latitude::from_degrees(0.0),
        None,
    );
    let err = topocentric_position(no_dist, &observer(0.0), 100.0, 23.4).unwrap_err();
    assert_eq!(err, ApparentPlaceError::MissingDistance);
}

#[test]
fn diurnal_aberration_is_sub_arcsec() {
    let out = topocentric_position(ecl(100.0, 0.0, 1.0), &observer(0.0), 100.0, 23.4).unwrap();
    assert!(
        out.provenance.diurnal_aberration_arcsec < 0.36,
        "diurnal aberration {}\"",
        out.provenance.diurnal_aberration_arcsec
    );
}
