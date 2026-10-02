//! `EventEngine::stations_in_range` and `next_station`: settled instants,
//! chaining, bodies that never station, guards and window edges.

use pleiades_backend::test_backend::LinearSunMoon;
use pleiades_data::packaged_backend;
use pleiades_events::{
    CrossingFrame, CrossingReference, EventEngine, EventError, Station, StationKind, WINDOW_END_JD,
    WINDOW_START_JD,
};
use pleiades_types::{Ayanamsa, CelestialBody, Instant, JulianDay, TimeScale, ZodiacMode};

const GEO: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const MEAN: CrossingFrame = CrossingFrame::GeocentricMeanOfDate;
const HELIO: CrossingFrame = CrossingFrame::Heliocentric;
const J2000: f64 = 2_451_545.0;
/// Two seconds, in days: four times the bisection tolerance.
const TWO_SECONDS: f64 = 2.0 / 86_400.0;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn stations(
    body: CelestialBody,
    reference: impl Into<CrossingReference>,
    start_jd: f64,
    end_jd: f64,
) -> Vec<Station> {
    EventEngine::new(packaged_backend())
        .stations_in_range(body, reference, tdb(start_jd), tdb(end_jd))
        .expect("stations")
}

fn speed(body: CelestialBody, reference: impl Into<CrossingReference>, jd: f64) -> f64 {
    EventEngine::new(packaged_backend())
        .position_at(body, reference, tdb(jd))
        .expect("position")
        .motion
        .longitude_deg_per_day
        .expect("speed")
}

fn jd(station: &Station) -> f64 {
    station.instant.julian_day.days()
}

fn assert_alternating(found: &[Station]) {
    for pair in found.windows(2) {
        assert_ne!(pair[0].kind, pair[1].kind, "{pair:?}");
        assert!(jd(&pair[0]) < jd(&pair[1]), "{pair:?}");
    }
}

// Mercury was retrograde three times in 2000: 21 Feb – 14 Mar, 23 Jun – 17 Jul
// and 18 Oct – 8 Nov.
#[test]
fn mercury_stations_of_2000() {
    let found = stations(CelestialBody::Mercury, GEO, J2000, J2000 + 366.0);
    assert_eq!(found.len(), 6, "{found:?}");
    assert_eq!(found[0].kind, StationKind::TurnsRetrograde);
    assert_alternating(&found);
    // 21 February 2000 is about 51 days after J2000.
    let first = jd(&found[0]) - J2000;
    assert!((49.0..54.0).contains(&first), "first station at +{first} d");
    for station in &found {
        assert_eq!(station.body, CelestialBody::Mercury);
        assert_eq!(station.frame, GEO);
        assert_eq!(station.zodiac, ZodiacMode::Tropical);
        assert_eq!(station.instant.scale, TimeScale::Tdb);
    }
}

// The returned instant is the later end of the final bisection bracket: the
// engine's own speed already has the post-station sign there, and still had
// the pre-station sign two seconds earlier.
#[test]
fn returned_instants_are_settled() {
    let engine = EventEngine::new(packaged_backend());
    let mut found = stations(CelestialBody::Mercury, GEO, J2000, J2000 + 366.0);
    found.extend(stations(CelestialBody::Mars, GEO, J2000, J2000 + 1100.0));
    assert!(found.len() >= 8, "{}", found.len());
    for station in &found {
        let at = speed(station.body.clone(), GEO, jd(station));
        let before = speed(station.body.clone(), GEO, jd(station) - TWO_SECONDS);
        match station.kind {
            StationKind::TurnsDirect => {
                assert!(at > 0.0 && before <= 0.0, "{station:?}: {before} -> {at}")
            }
            StationKind::TurnsRetrograde => {
                assert!(at <= 0.0 && before > 0.0, "{station:?}: {before} -> {at}")
            }
            other => panic!("unexpected kind {other:?}"),
        }
        let longitude = engine
            .longitude_at(station.body.clone(), GEO, station.instant)
            .expect("longitude");
        assert_eq!(station.longitude, longitude, "{station:?}");
    }
}

#[test]
fn next_station_is_the_first_in_range_and_chains() {
    let engine = EventEngine::new(packaged_backend());
    let in_range = stations(CelestialBody::Mercury, GEO, J2000, J2000 + 366.0);
    let mut after = tdb(J2000);
    for expected in &in_range {
        let next = engine
            .next_station(CelestialBody::Mercury, GEO, after)
            .expect("next_station")
            .expect("Mercury stations six times a year");
        assert_eq!(&next, expected);
        // Handing a returned instant back finds the following station.
        after = next.instant;
    }
}

#[test]
fn bodies_that_never_station_return_nothing() {
    for body in [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::MeanNode,
    ] {
        let found = stations(body.clone(), GEO, J2000, J2000 + 730.0);
        assert!(found.is_empty(), "{body:?}: {found:?}");
    }
    let engine = EventEngine::new(packaged_backend());
    let next = engine
        .next_station(CelestialBody::Sun, GEO, tdb(WINDOW_END_JD - 400.0))
        .expect("next_station");
    assert_eq!(next, None);
}

#[test]
fn nothing_stations_heliocentrically() {
    for body in [
        CelestialBody::Mercury,
        CelestialBody::Mars,
        CelestialBody::Pluto,
    ] {
        let found = stations(body.clone(), HELIO, J2000, J2000 + 1100.0);
        assert!(found.is_empty(), "{body:?}: {found:?}");
    }
}

// The true node is retrograde on average and turns briefly direct about
// every two weeks.
#[test]
fn the_true_node_stations_often() {
    let found = stations(CelestialBody::TrueNode, GEO, J2000, J2000 + 365.0);
    assert!(found.len() >= 10, "{}", found.len());
    assert_alternating(&found);
}

// A sidereal longitude speed is the tropical one less the ayanamsa's rate
// (about 3.8e-5 deg/day), so the speed reaches zero later on the way up and
// earlier on the way down. The mean-of-date frame is used because the
// apparent frame's sidereal speed also drops the nutation rate, whose sign
// varies.
#[test]
fn a_sidereal_station_is_shifted_by_the_ayanamsa_rate() {
    let lahiri = CrossingReference::sidereal(MEAN, Ayanamsa::Lahiri);
    let tropical = stations(CelestialBody::Saturn, MEAN, J2000, J2000 + 730.0);
    let sidereal = stations(CelestialBody::Saturn, lahiri, J2000, J2000 + 730.0);
    assert!(tropical.len() >= 3, "{tropical:?}");
    assert_eq!(tropical.len(), sidereal.len());
    for (t, s) in tropical.iter().zip(&sidereal) {
        assert_eq!(t.kind, s.kind);
        let shift = jd(s) - jd(t);
        let expected_sign = match t.kind {
            StationKind::TurnsDirect => 1.0,
            _ => -1.0,
        };
        assert!(
            (0.002..1.0).contains(&(shift * expected_sign)),
            "{:?}: sidereal - tropical = {shift} d",
            t.kind
        );
    }
}

#[test]
fn a_backend_without_speed_is_an_error_not_an_empty_list() {
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let err = engine
        .stations_in_range(CelestialBody::Sun, GEO, tdb(J2000), tdb(J2000 + 30.0))
        .unwrap_err();
    assert!(
        matches!(
            err,
            EventError::MissingSpeed {
                body_label: "Sun",
                ..
            }
        ),
        "{err:?}"
    );
    let err = engine
        .next_station(CelestialBody::Sun, GEO, tdb(J2000))
        .unwrap_err();
    assert!(matches!(err, EventError::MissingSpeed { .. }), "{err:?}");
}

#[test]
fn guards_match_position_at() {
    let engine = EventEngine::new(packaged_backend());
    let err = engine
        .stations_in_range(CelestialBody::Mars, GEO, tdb(2_000_000.0), tdb(J2000))
        .unwrap_err();
    assert!(matches!(err, EventError::OutOfWindow { .. }), "{err:?}");
    let err = engine
        .stations_in_range(CelestialBody::Mars, GEO, tdb(J2000), tdb(2_500_000.0))
        .unwrap_err();
    assert!(matches!(err, EventError::OutOfWindow { .. }), "{err:?}");
    let err = engine
        .next_station(CelestialBody::Mars, GEO, tdb(2_000_000.0))
        .unwrap_err();
    assert!(matches!(err, EventError::OutOfWindow { .. }), "{err:?}");
    let err = engine
        .next_station(CelestialBody::Sun, HELIO, tdb(J2000))
        .unwrap_err();
    assert!(
        matches!(err, EventError::UnsupportedFrame { .. }),
        "{err:?}"
    );
    let err = engine
        .stations_in_range(
            CelestialBody::Mars,
            CrossingReference::sidereal(HELIO, Ayanamsa::Lahiri),
            tdb(J2000),
            tdb(J2000 + 30.0),
        )
        .unwrap_err();
    assert!(
        matches!(err, EventError::UnsupportedFrame { .. }),
        "{err:?}"
    );
}

#[test]
fn ranges_touching_the_window_edges_work() {
    let engine = EventEngine::new(packaged_backend());
    let early = stations(
        CelestialBody::Mercury,
        GEO,
        WINDOW_START_JD,
        WINDOW_START_JD + 400.0,
    );
    assert!(early.len() >= 4, "{early:?}");
    let late = stations(
        CelestialBody::Mercury,
        GEO,
        WINDOW_END_JD - 400.0,
        WINDOW_END_JD,
    );
    assert!(late.len() >= 4, "{late:?}");
    let next = engine
        .next_station(CelestialBody::Mercury, GEO, tdb(WINDOW_END_JD))
        .expect("next_station at the window end");
    assert_eq!(next, None);
}

#[test]
fn empty_and_inverted_ranges_give_no_stations() {
    assert!(stations(CelestialBody::Mercury, GEO, J2000, J2000).is_empty());
    assert!(stations(CelestialBody::Mercury, GEO, J2000 + 366.0, J2000).is_empty());
}

#[test]
fn a_body_the_backend_does_not_serve_is_an_error() {
    let engine = EventEngine::new(packaged_backend());
    let result = engine.stations_in_range(CelestialBody::Ceres, GEO, tdb(J2000), tdb(J2000 + 30.0));
    assert!(result.is_err(), "{result:?}");
}

#[cfg(feature = "serde")]
#[test]
fn a_station_round_trips_through_serde() {
    let found = stations(CelestialBody::Mercury, GEO, J2000, J2000 + 100.0);
    let json = serde_json::to_string(&found[0]).expect("serialize");
    let back: Station = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, found[0]);
}
