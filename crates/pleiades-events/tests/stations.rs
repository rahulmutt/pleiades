//! `EventEngine::stations_in_range` and `next_station`: settled instants,
//! chaining, bodies that never station, guards and window edges.

use pleiades_backend::test_backend::LinearSunMoon;
use pleiades_backend::CompositeBackend;
use pleiades_data::packaged_backend;
use pleiades_elp::ElpBackend;
use pleiades_events::{
    CrossingFrame, CrossingReference, EventEngine, EventError, Station, StationKind, WINDOW_END_JD,
    WINDOW_START_JD,
};
use pleiades_types::{Ayanamsa, CelestialBody, Instant, JulianDay, TimeScale, ZodiacMode};
use pleiades_vsop87::Vsop87Backend;

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

/// Short direct spells of the true node from issue #108: Swiss Ephemeris
/// (`swe_calc(SE_TRUE_NODE, SEFLG_SWIEPH | SEFLG_SPEED)`, pyswisseph 2.10.03)
/// turns direct at `start` for `hours` and peaks at `peak` arcseconds a day.
const TRUE_NODE_DIRECT_SPELLS: [(f64, f64, f64); 6] = [
    (2_457_892.495, 2.6, 0.11),
    (2_458_422.645, 13.4, 3.29),
    (2_459_102.895, 13.9, 2.97),
    (2_459_796.955, 9.8, 1.66),
    (2_460_327.085, 13.9, 3.55),
    (2_461_020.815, 13.7, 3.12),
];

// The true node's speed is the derivative of a fast-oscillating point. A
// one-day central difference read it about 3.2"/day low at these peaks, so the
// shallower spells never turned direct (#108). The peak must match Swiss
// Ephemeris, whose Moon differs from the packaged DE440 one by far less.
#[test]
fn true_node_speed_peaks_match_swiss_ephemeris_in_short_direct_spells() {
    let engine = EventEngine::new(packaged_backend());
    for (start, hours, se_peak) in TRUE_NODE_DIRECT_SPELLS {
        let end = start + hours / 24.0;
        let mut peak = f64::NEG_INFINITY;
        let mut jd = start - 0.25;
        while jd <= end + 0.25 {
            let speed = engine
                .position_at(CelestialBody::TrueNode, GEO, tdb(jd))
                .expect("position")
                .motion
                .longitude_deg_per_day
                .expect("speed");
            peak = peak.max(speed * 3600.0);
            jd += 0.01;
        }
        assert!(
            (peak - se_peak).abs() < 0.1,
            "spell at {start}: peak {peak:.3}\"/day, Swiss Ephemeris {se_peak}\"/day"
        );
    }
}

// Every spell longer than the 0.25-day scan step is found as a station pair.
#[test]
fn true_node_short_direct_spells_are_station_pairs() {
    for (start, hours, _) in TRUE_NODE_DIRECT_SPELLS {
        if hours / 24.0 <= 0.25 {
            continue;
        }
        let end = start + hours / 24.0;
        let found = stations(CelestialBody::TrueNode, GEO, start - 0.5, end + 0.5);
        let kinds: Vec<StationKind> = found.iter().map(|s| s.kind).collect();
        assert_eq!(
            kinds,
            [StationKind::TurnsDirect, StationKind::TurnsRetrograde],
            "spell at {start} ({hours} h): {found:?}"
        );
        // Within an hour of Swiss Ephemeris at each end.
        assert!((jd(&found[0]) - start).abs() < 1.0 / 24.0, "{found:?}");
        assert!((jd(&found[1]) - end).abs() < 1.0 / 24.0, "{found:?}");
    }
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

// Issue #140: the algorithmic backends differenced the position over ±0.5 day,
// which biased Mercury's speed by up to 7″/day and put its stations on the
// VSOP87/ELP composite up to 265 s from the zero of the true speed. Over 2000
// the composite then sat up to 239 s from the packaged backend; it now sits
// within 0.33 s (and within the same 0.33 s over 2000–2010).
#[test]
fn composite_mercury_stations_match_the_packaged_backend() {
    let composite = EventEngine::new(CompositeBackend::new(
        ElpBackend::new(),
        Vsop87Backend::new(),
    ));
    let (start, end) = (J2000, J2000 + 366.0);
    let found = composite
        .stations_in_range(CelestialBody::Mercury, GEO, tdb(start), tdb(end))
        .expect("stations");
    let packaged = stations(CelestialBody::Mercury, GEO, start, end);
    assert_eq!(found.len(), 6, "{found:?}");
    assert_eq!(found.len(), packaged.len());
    let mut worst_seconds = 0.0_f64;
    for (composite, packaged) in found.iter().zip(&packaged) {
        assert_eq!(composite.kind, packaged.kind);
        worst_seconds = worst_seconds.max(((jd(composite) - jd(packaged)) * 86_400.0).abs());
    }
    assert!(
        worst_seconds < 5.0,
        "composite - packaged reaches {worst_seconds} s"
    );
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
    // An unserved body is a backend failure, not a missing speed.
    assert!(
        !matches!(result, Err(EventError::MissingSpeed { .. })),
        "{result:?}"
    );
    assert!(matches!(result, Err(EventError::Backend(_))), "{result:?}");
}

#[cfg(feature = "serde")]
#[test]
fn a_station_round_trips_through_serde() {
    let found = stations(CelestialBody::Mercury, GEO, J2000, J2000 + 100.0);
    let json = serde_json::to_string(&found[0]).expect("serialize");
    let back: Station = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, found[0]);
}
