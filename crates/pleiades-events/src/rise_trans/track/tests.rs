use super::*;
use crate::rise_trans::test_support::{composite, CountingBackend};

const TRACKED: [CelestialBody; 10] = [
    CelestialBody::Sun,
    CelestialBody::Moon,
    CelestialBody::Mercury,
    CelestialBody::Venus,
    CelestialBody::Mars,
    CelestialBody::Jupiter,
    CelestialBody::Saturn,
    CelestialBody::Uranus,
    CelestialBody::Neptune,
    CelestialBody::Pluto,
];

/// Largest difference allowed between a track and a direct read, in either
/// angle. 0.05″ of place is about 3 ms of rise time, against the search's
/// 0.5 s refinement tolerance. Measured 2026-10-06 over 200 instants per
/// body: the Moon 0.0200″, Mercury 0.0031″, Venus 0.0017″, and every other
/// body within 0.0003″.
const ANGLE_CEILING_ARCSEC: f64 = 0.05;

/// Largest relative difference allowed in distance. A body's semidiameter
/// and its diurnal parallax are the only uses of it. Measured: at most
/// 6.1e-8, for the Moon.
const DISTANCE_CEILING: f64 = 1e-6;

/// Longitude (scaled by cos latitude) and latitude differences in
/// arcseconds, and the relative distance difference.
fn difference(got: Place, want: Place) -> (f64, f64, f64) {
    let longitude = ((got.0 - want.0 + 180.0).rem_euclid(360.0) - 180.0).abs()
        * want.1.to_radians().cos()
        * 3600.0;
    let latitude = (got.1 - want.1).abs() * 3600.0;
    let distance = ((got.2 - want.2) / want.2).abs();
    (longitude, latitude, distance)
}

#[test]
fn a_track_reproduces_a_direct_read_for_every_tracked_body() {
    let backend = composite();
    for body in TRACKED {
        let (mut worst_longitude, mut worst_latitude, mut worst_distance) =
            (0.0_f64, 0.0_f64, 0.0_f64);
        // 200 instants a year less a day apart, 1900 to 2099, at no fixed
        // phase of the lattice.
        for sample in 0..200_u32 {
            let jd = WINDOW_START_JD + 80.3 + f64::from(sample) * 364.37;
            let track = BodyTrack::new(&backend, &body).expect("a tracked body");
            let got = track.place(jd).unwrap();
            let want = geocentric_apparent_ecliptic(&backend, body.clone(), "body", jd).unwrap();
            let (longitude, latitude, distance) = difference(got, want);
            worst_longitude = worst_longitude.max(longitude);
            worst_latitude = worst_latitude.max(latitude);
            worst_distance = worst_distance.max(distance);
        }
        assert!(
            worst_longitude <= ANGLE_CEILING_ARCSEC && worst_latitude <= ANGLE_CEILING_ARCSEC,
            "{body}: {worst_longitude:.6}″ in longitude, {worst_latitude:.6}″ in latitude"
        );
        assert!(
            worst_distance <= DISTANCE_CEILING,
            "{body}: {worst_distance:.3e} in distance"
        );
    }
}

#[test]
fn a_track_interpolates_across_the_longitude_seam() {
    // The Sun crosses 0° at JD 2460754.88 (the 2025 March equinox), between
    // the lattice samples at 2460754.5 and 2460755.0.
    let backend = composite();
    let track = BodyTrack::new(&backend, &CelestialBody::Sun).expect("the Sun is tracked");
    for jd in [2_460_754.6, 2_460_754.87, 2_460_754.89, 2_460_754.99] {
        let got = track.place(jd).unwrap();
        let want = geocentric_apparent_ecliptic(&backend, CelestialBody::Sun, "body", jd).unwrap();
        assert!((0.0..360.0).contains(&got.0), "longitude {}", got.0);
        let (longitude, latitude, _) = difference(got, want);
        assert!(
            longitude <= 0.001 && latitude <= 0.001,
            "JD {jd}: {longitude:.6}″, {latitude:.6}″"
        );
    }
}

#[test]
fn a_track_reads_each_lattice_sample_once() {
    let backend = CountingBackend::new(composite());
    let track = BodyTrack::new(&backend, &CelestialBody::Sun).expect("the Sun is tracked");
    // Thirty instants inside one 12-hour lattice interval.
    for step in 0..30_u32 {
        track.place(2_460_000.01 + f64::from(step) * 0.016).unwrap();
    }
    assert_eq!(backend.take_reads(), 4);
    // The next interval shares three of its four samples.
    track.place(2_460_000.6).unwrap();
    assert_eq!(backend.take_reads(), 1);
}

#[test]
fn an_instant_next_to_the_windows_end_is_read_directly() {
    // The lattice sample after the window's last instant cannot be read, so
    // the place comes from a read at the instant itself, bit for bit.
    let backend = composite();
    for body in [CelestialBody::Sun, CelestialBody::Moon] {
        let track = BodyTrack::new(&backend, &body).expect("a tracked body");
        for jd in [WINDOW_END_JD, WINDOW_END_JD - 0.01, WINDOW_START_JD + 0.01] {
            let want = geocentric_apparent_ecliptic(&backend, body.clone(), "body", jd).unwrap();
            assert_eq!(track.place(jd).unwrap(), want, "{body} at JD {jd}");
        }
    }
}

#[test]
fn an_unreadable_instant_keeps_its_window_error() {
    // Mars cannot be read within light-time of the window's start, on the
    // track as off it.
    let backend = composite();
    let track = BodyTrack::new(&backend, &CelestialBody::Mars).expect("Mars is tracked");
    assert!(matches!(
        track.place(WINDOW_START_JD),
        Err(EventError::OutOfWindow { .. })
    ));
}

#[test]
fn a_body_without_a_measured_lattice_is_not_tracked() {
    let backend = composite();
    for body in [
        CelestialBody::TrueNode,
        CelestialBody::MeanNode,
        CelestialBody::Ceres,
    ] {
        assert!(BodyTrack::new(&backend, &body).is_none(), "{body}");
    }
}
