//! Issue #208: crossings within one scan step of the window's ends are found,
//! and a search the window cuts short is `OutOfWindow`.

use super::*;
use crate::window_edge_support::{out_of_window_jd, tdb};
use pleiades_data::packaged_backend;

const APPARENT: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const MEAN: CrossingFrame = CrossingFrame::GeocentricMeanOfDate;
/// The Moon's crossing step (`EventEngine::step_days`).
const MOON_STEP: f64 = 0.25;
/// Bisection tolerance plus slack: a returned instant trails its crossing by
/// less than 0.5 s.
const SETTLE_DAYS: f64 = 1.0 / 86_400.0;

fn engine() -> EventEngine<pleiades_data::PackagedDataBackend> {
    EventEngine::new(packaged_backend())
}

/// The Moon's longitude at `jd` in `frame`, as a crossing target.
fn moon_at(frame: CrossingFrame, jd: f64) -> Longitude {
    engine()
        .longitude_at(CelestialBody::Moon, frame, tdb(jd))
        .unwrap()
}

#[test]
fn a_moon_crossing_inside_the_last_step_is_found() {
    let at = WINDOW_END_JD - 0.1;
    let target = moon_at(APPARENT, at);
    let next = engine()
        .next_longitude_crossing(
            CelestialBody::Moon,
            target,
            APPARENT,
            tdb(WINDOW_END_JD - 0.2),
        )
        .unwrap()
        .expect("the crossing at WINDOW_END - 0.1 d");
    let found = next.instant.julian_day.days();
    assert!(
        (0.0..SETTLE_DAYS).contains(&(found - at)),
        "{found} vs {at}"
    );
    let in_range = engine()
        .longitude_crossings_in_range(
            CelestialBody::Moon,
            target,
            APPARENT,
            tdb(WINDOW_END_JD - 0.2),
            tdb(WINDOW_END_JD),
        )
        .unwrap();
    assert_eq!(in_range.len(), 1, "{in_range:?}");
}

#[test]
fn a_moon_crossing_inside_the_first_step_is_found() {
    // Mean of date: readable from the window's first instant.
    let at = WINDOW_START_JD + 0.1;
    let target = moon_at(MEAN, at);
    let previous = engine()
        .previous_longitude_crossing(
            CelestialBody::Moon,
            target,
            MEAN,
            tdb(WINDOW_START_JD + 0.2),
        )
        .unwrap()
        .expect("the crossing at WINDOW_START + 0.1 d");
    let found = previous.instant.julian_day.days();
    assert!(
        (0.0..SETTLE_DAYS).contains(&(found - at)),
        "{found} vs {at}"
    );
    let in_range = engine()
        .longitude_crossings_in_range(
            CelestialBody::Moon,
            target,
            MEAN,
            tdb(WINDOW_START_JD),
            tdb(WINDOW_START_JD + 0.2),
        )
        .unwrap();
    assert_eq!(in_range.len(), 1, "{in_range:?}");
}

#[test]
fn a_search_the_window_cuts_short_is_out_of_window() {
    // The Moon takes ~27 d to return to a longitude; from 0.2 d before the
    // end, after passing it at -0.1 d, the window ends first.
    let target = moon_at(APPARENT, WINDOW_END_JD - 0.1);
    let next = engine().next_longitude_crossing(
        CelestialBody::Moon,
        target,
        APPARENT,
        tdb(WINDOW_END_JD - 0.05),
    );
    assert_eq!(out_of_window_jd(next), WINDOW_END_JD + MOON_STEP);
    let at_end =
        engine().next_longitude_crossing(CelestialBody::Moon, target, APPARENT, tdb(WINDOW_END_JD));
    assert_eq!(out_of_window_jd(at_end), WINDOW_END_JD + MOON_STEP);
    let target = moon_at(MEAN, WINDOW_START_JD + 0.1);
    let previous = engine().previous_longitude_crossing(
        CelestialBody::Moon,
        target,
        MEAN,
        tdb(WINDOW_START_JD + 0.05),
    );
    assert_eq!(out_of_window_jd(previous), WINDOW_START_JD - MOON_STEP);
}

#[test]
fn an_apparent_moon_range_from_the_first_instant_is_out_of_window() {
    // The apparent Moon is read 1.3 s earlier, before the window.
    let range = engine().longitude_crossings_in_range(
        CelestialBody::Moon,
        Longitude::from_degrees(0.0),
        APPARENT,
        tdb(WINDOW_START_JD),
        tdb(WINDOW_START_JD + 30.0),
    );
    assert!(
        matches!(range, Err(EventError::OutOfWindow { .. })),
        "{range:?}"
    );
    // The Sun needs no light-time read and is served from the first instant.
    let sun = engine()
        .longitude_crossings_in_range(
            CelestialBody::Sun,
            Longitude::from_degrees(290.0),
            APPARENT,
            tdb(WINDOW_START_JD),
            tdb(WINDOW_START_JD + 30.0),
        )
        .unwrap();
    assert_eq!(sun.len(), 1, "{sun:?}");
}
