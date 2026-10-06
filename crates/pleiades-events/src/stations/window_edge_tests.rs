//! Issue #208: stations within one scan step of the window's ends are found,
//! and a search the window cuts short is `OutOfWindow`. The fixture is the
//! synthetic `StationingMars`, with stations one day inside each end, in the
//! mean-of-date frame (readable from the window's first instant).

use super::*;
use crate::window_edge_support::{
    out_of_window_jd, tdb, StationingMars, FIRST_STATION_JD, LAST_STATION_JD,
};

const MEAN: CrossingFrame = CrossingFrame::GeocentricMeanOfDate;
/// Mars's station step (`step_days`).
const MARS_STEP: f64 = 2.0;
/// Precession moves each station by about 3 s (see `SLOPE`).
const NEAR_DAYS: f64 = 10.0 / 86_400.0;

fn engine() -> EventEngine<StationingMars> {
    EventEngine::new(StationingMars)
}

fn jd(station: &Station) -> f64 {
    station.instant.julian_day.days()
}

#[test]
fn stations_inside_the_end_steps_are_found_by_a_range() {
    let late = engine()
        .stations_in_range(
            CelestialBody::Mars,
            MEAN,
            tdb(WINDOW_END_JD - 1.5),
            tdb(WINDOW_END_JD),
        )
        .unwrap();
    assert_eq!(late.len(), 1, "{late:?}");
    assert!(
        (jd(&late[0]) - LAST_STATION_JD).abs() < NEAR_DAYS,
        "{late:?}"
    );
    assert_eq!(late[0].kind, StationKind::TurnsDirect);
    let early = engine()
        .stations_in_range(
            CelestialBody::Mars,
            MEAN,
            tdb(WINDOW_START_JD),
            tdb(WINDOW_START_JD + 1.5),
        )
        .unwrap();
    assert_eq!(early.len(), 1, "{early:?}");
    assert!(
        (jd(&early[0]) - FIRST_STATION_JD).abs() < NEAR_DAYS,
        "{early:?}"
    );
    assert_eq!(early[0].kind, StationKind::TurnsRetrograde);
}

#[test]
fn stations_inside_the_end_steps_are_found_by_next_and_previous() {
    let next = engine()
        .next_station(CelestialBody::Mars, MEAN, tdb(WINDOW_END_JD - 1.5))
        .unwrap()
        .expect("the last station");
    assert!((jd(&next) - LAST_STATION_JD).abs() < NEAR_DAYS, "{next:?}");
    let previous = engine()
        .previous_station(CelestialBody::Mars, MEAN, tdb(WINDOW_START_JD + 1.5))
        .unwrap()
        .expect("the first station");
    assert!(
        (jd(&previous) - FIRST_STATION_JD).abs() < NEAR_DAYS,
        "{previous:?}"
    );
}

#[test]
fn a_station_search_the_window_cuts_short_is_out_of_window() {
    let after_last = engine().next_station(CelestialBody::Mars, MEAN, tdb(WINDOW_END_JD - 0.5));
    assert_eq!(out_of_window_jd(after_last), WINDOW_END_JD + MARS_STEP);
    let at_end = engine().next_station(CelestialBody::Mars, MEAN, tdb(WINDOW_END_JD));
    assert_eq!(out_of_window_jd(at_end), WINDOW_END_JD + MARS_STEP);
    let before_first =
        engine().previous_station(CelestialBody::Mars, MEAN, tdb(WINDOW_START_JD + 0.5));
    assert_eq!(out_of_window_jd(before_first), WINDOW_START_JD - MARS_STEP);
}
