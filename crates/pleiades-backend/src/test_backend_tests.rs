//! The fixture's positions against its closed form, `lon = ref + offset +
//! rate · (jd − jd0)`, evaluated by hand. Downstream crates' event tests rely
//! on these longitudes, but cargo-mutants runs only this crate's tests.

use super::LinearSunMoon;
use crate::{EphemerisBackend, EphemerisErrorKind, EphemerisRequest};
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

const JD0: f64 = 2_451_545.0;

/// Ten days after `JD0`, so a sign or operator change in `jd − jd0` or in
/// `rate · dt` moves every longitude.
fn request_at_ten_days(body: CelestialBody) -> EphemerisRequest {
    EphemerisRequest::new(
        body,
        Instant::new(JulianDay::from_days(JD0 + 10.0), TimeScale::Tdb),
    )
}

fn longitude(backend: &LinearSunMoon, body: CelestialBody) -> f64 {
    backend
        .position(&request_at_ten_days(body))
        .expect("the fixture serves the Sun and the Moon")
        .ecliptic
        .expect("the fixture produces coordinates")
        .longitude
        .degrees()
}

fn assert_degrees(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "{actual} != {expected} degrees"
    );
}

#[test]
fn sun_and_moon_advance_at_their_rates() {
    let backend = LinearSunMoon::new_moon_at(JD0);
    // 100 + 0.985647 · 10 and 100 + 13.176396 · 10.
    assert_degrees(longitude(&backend, CelestialBody::Sun), 109.856_47);
    assert_degrees(longitude(&backend, CelestialBody::Moon), 231.763_96);
}

#[test]
fn the_moon_offset_is_added_to_the_moon_alone() {
    // The public constructors offer offsets of 0 and 180 degrees only, and
    // `ref - offset` equals `ref + offset` modulo 360 for both, so a 30-degree
    // offset is built here to tell them apart.
    let backend = LinearSunMoon {
        moon_longitude_offset_deg: 30.0,
        ..LinearSunMoon::new_moon_at(JD0)
    };
    assert_degrees(longitude(&backend, CelestialBody::Sun), 109.856_47);
    // 100 + 30 + 13.176396 · 10.
    assert_degrees(longitude(&backend, CelestialBody::Moon), 261.763_96);
}

#[test]
fn serves_only_the_sun_and_the_moon() {
    let backend = LinearSunMoon::new_moon_at(JD0);
    assert!(backend.supports_body(CelestialBody::Sun));
    assert!(backend.supports_body(CelestialBody::Moon));
    assert!(!backend.supports_body(CelestialBody::Mars));

    let error = backend
        .position(&request_at_ten_days(CelestialBody::Mars))
        .expect_err("Mars is not served");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
}

#[test]
fn only_the_empty_fixture_omits_coordinates() {
    let request = request_at_ten_days(CelestialBody::Sun);
    let empty = LinearSunMoon::empty().position(&request).unwrap();
    assert_eq!(empty.ecliptic, None);
    let full = LinearSunMoon::new_moon_at(JD0).position(&request).unwrap();
    assert!(full.ecliptic.is_some());
}
