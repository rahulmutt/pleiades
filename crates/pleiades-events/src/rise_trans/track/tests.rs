use super::*;
use crate::rise_trans::test_support::{composite, CountingBackend, FailingFirstReads};

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
        let cache = PlaceCache::new();
        let (mut worst_longitude, mut worst_latitude, mut worst_distance) =
            (0.0_f64, 0.0_f64, 0.0_f64);
        // 200 instants a year less a day apart, 1900 to 2099, at no fixed
        // phase of the lattice.
        for sample in 0..200_u32 {
            let jd = WINDOW_START_JD + 80.3 + f64::from(sample) * 364.37;
            let track = BodyTrack::new(&backend, &cache, &body).expect("a tracked body");
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
    let cache = PlaceCache::new();
    let track = BodyTrack::new(&backend, &cache, &CelestialBody::Sun).expect("the Sun is tracked");
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
    let cache = PlaceCache::new();
    let track = BodyTrack::new(&backend, &cache, &CelestialBody::Sun).expect("the Sun is tracked");
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
    let cache = PlaceCache::new();
    for body in [CelestialBody::Sun, CelestialBody::Moon] {
        let track = BodyTrack::new(&backend, &cache, &body).expect("a tracked body");
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
    let cache = PlaceCache::new();
    let track = BodyTrack::new(&backend, &cache, &CelestialBody::Mars).expect("Mars is tracked");
    assert!(matches!(
        track.place(WINDOW_START_JD),
        Err(EventError::OutOfWindow { .. })
    ));
}

#[test]
fn a_body_without_a_measured_lattice_is_not_tracked() {
    let backend = composite();
    let cache = PlaceCache::new();
    for body in [
        CelestialBody::TrueNode,
        CelestialBody::MeanNode,
        CelestialBody::Ceres,
    ] {
        assert!(BodyTrack::new(&backend, &cache, &body).is_none(), "{body}");
    }
}

#[test]
fn a_cache_starts_over_when_it_is_full() {
    let cache = PlaceCache::with_capacity(2);
    let place = Some((1.0, 2.0, 3.0));
    cache.insert(&CelestialBody::Sun, 1, place);
    cache.insert(&CelestialBody::Sun, 2, place);
    assert_eq!(cache.len(), 2);
    cache.insert(&CelestialBody::Sun, 3, place);
    assert_eq!(
        cache.len(),
        1,
        "the full table is cleared before the insert"
    );
    assert_eq!(cache.get(&CelestialBody::Sun, 1), None);
    assert_eq!(cache.get(&CelestialBody::Sun, 3), Some(place));
}

#[test]
fn a_cache_keeps_bodies_apart() {
    let cache = PlaceCache::new();
    cache.insert(&CelestialBody::Sun, 7, Some((1.0, 0.0, 1.0)));
    assert_eq!(cache.get(&CelestialBody::Moon, 7), None);
    assert_eq!(
        cache.get(&CelestialBody::Sun, 7),
        Some(Some((1.0, 0.0, 1.0)))
    );
}

#[test]
fn a_cache_reinserting_a_key_does_not_grow_it() {
    let cache = PlaceCache::with_capacity(2);
    cache.insert(&CelestialBody::Sun, 1, None);
    cache.insert(&CelestialBody::Sun, 1, None);
    cache.insert(&CelestialBody::Sun, 2, None);
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.get(&CelestialBody::Sun, 1), Some(None));
}

#[test]
fn a_cache_survives_a_poisoned_lock() {
    let cache = PlaceCache::new();
    cache.insert(&CelestialBody::Sun, 1, Some((1.0, 2.0, 3.0)));
    let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = cache.lock();
        panic!("poison the lock");
    }));
    assert!(poisoned.is_err());
    assert_eq!(
        cache.get(&CelestialBody::Sun, 1),
        Some(Some((1.0, 2.0, 3.0)))
    );
    cache.insert(&CelestialBody::Sun, 2, None);
    assert_eq!(cache.get(&CelestialBody::Sun, 2), Some(None));
}

#[test]
fn tracks_sharing_a_cache_read_each_sample_once() {
    let backend = CountingBackend::new(composite());
    let cache = PlaceCache::new();
    let first = BodyTrack::new(&backend, &cache, &CelestialBody::Sun).expect("tracked");
    first.place(2_460_000.01).unwrap();
    assert_eq!(backend.take_reads(), 4);
    // A second track over the same cache, as the next search of the same
    // engine makes, reads nothing for the same interval.
    let second = BodyTrack::new(&backend, &cache, &CelestialBody::Sun).expect("tracked");
    assert_eq!(
        second.place(2_460_000.01).unwrap(),
        first.place(2_460_000.01).unwrap()
    );
    assert_eq!(backend.take_reads(), 0);
}

#[test]
fn a_sample_the_window_does_not_reach_is_remembered() {
    // Mars at the window's first lattice instant reads before the window
    // (light-time), so that sample is `None` and the place at an instant
    // next to it is read directly. Asked again, the `None` is not re-read:
    // only the direct read is repeated.
    let backend = CountingBackend::new(composite());
    let cache = PlaceCache::new();
    let jd = WINDOW_START_JD + 0.6;
    let track = BodyTrack::new(&backend, &cache, &CelestialBody::Mars).expect("tracked");
    let first = track.place(jd).unwrap();
    backend.take_reads();
    geocentric_apparent_ecliptic(&backend, CelestialBody::Mars, "body", jd).unwrap();
    let direct_reads = backend.take_reads();
    let again = track.place(jd).unwrap();
    assert_eq!(again, first);
    assert_eq!(backend.take_reads(), direct_reads);
}

#[test]
fn a_failed_read_is_not_remembered() {
    let backend = FailingFirstReads::new(composite(), 1);
    let cache = PlaceCache::new();
    let track = BodyTrack::new(&backend, &cache, &CelestialBody::Sun).expect("tracked");
    let jd = 2_460_000.2;
    assert!(matches!(track.place(jd), Err(EventError::Backend(_))));
    let fresh_backend = composite();
    let fresh_cache = PlaceCache::new();
    let fresh = BodyTrack::new(&fresh_backend, &fresh_cache, &CelestialBody::Sun).expect("tracked");
    assert_eq!(track.place(jd).unwrap(), fresh.place(jd).unwrap());
}
