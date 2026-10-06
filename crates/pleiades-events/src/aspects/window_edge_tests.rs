//! Issue #208: exact aspects within two scan steps of the window's ends are
//! found, and a search the window cuts short is `OutOfWindow`. Fixtures are
//! constructed: the angle is the Sun–Moon separation at an instant inside
//! the old margin.

use super::*;
use crate::window_edge_support::{out_of_window_jd, tdb};
use pleiades_data::packaged_backend;

const APPARENT: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const MEAN: CrossingFrame = CrossingFrame::GeocentricMeanOfDate;
/// The Sun–Moon search step (the Moon's).
const STEP: f64 = 0.25;
const SETTLE_DAYS: f64 = 1.0 / 86_400.0;

fn engine() -> EventEngine<pleiades_data::PackagedDataBackend> {
    EventEngine::new(packaged_backend())
}

/// The unsigned Sun–Moon separation at `jd`, in degrees.
fn separation_at(frame: CrossingFrame, jd: f64) -> Angle {
    let e = engine();
    let sun = e.longitude_at(CelestialBody::Sun, frame, tdb(jd)).unwrap();
    let moon = e.longitude_at(CelestialBody::Moon, frame, tdb(jd)).unwrap();
    Angle::from_degrees(wrap180(moon.degrees() - sun.degrees()).abs())
}

#[test]
fn an_aspect_inside_the_last_two_steps_is_found() {
    let at = WINDOW_END_JD - 0.2;
    let angle = separation_at(APPARENT, at);
    let next = engine()
        .next_aspect(
            CelestialBody::Sun,
            CelestialBody::Moon,
            angle,
            APPARENT,
            tdb(WINDOW_END_JD - 0.4),
        )
        .unwrap()
        .expect("the aspect at WINDOW_END - 0.2 d");
    let found = next.instant.julian_day.days();
    assert!(
        (0.0..SETTLE_DAYS).contains(&(found - at)),
        "{found} vs {at}"
    );
    let in_range = engine()
        .aspects_in_range(
            CelestialBody::Sun,
            CelestialBody::Moon,
            angle,
            APPARENT,
            tdb(WINDOW_END_JD - 0.4),
            tdb(WINDOW_END_JD),
        )
        .unwrap();
    assert_eq!(in_range.len(), 1, "{in_range:?}");
}

#[test]
fn an_aspect_inside_the_first_two_steps_is_found() {
    let at = WINDOW_START_JD + 0.2;
    let angle = separation_at(MEAN, at);
    let previous = engine()
        .previous_aspect(
            CelestialBody::Sun,
            CelestialBody::Moon,
            angle,
            MEAN,
            tdb(WINDOW_START_JD + 0.4),
        )
        .unwrap()
        .expect("the aspect at WINDOW_START + 0.2 d");
    let found = previous.instant.julian_day.days();
    assert!(
        (0.0..SETTLE_DAYS).contains(&(found - at)),
        "{found} vs {at}"
    );
    let in_range = engine()
        .aspects_in_range(
            CelestialBody::Sun,
            CelestialBody::Moon,
            angle,
            MEAN,
            tdb(WINDOW_START_JD),
            tdb(WINDOW_START_JD + 0.4),
        )
        .unwrap();
    assert_eq!(in_range.len(), 1, "{in_range:?}");
}

#[test]
fn an_apparent_range_just_after_the_light_time_sliver_scans() {
    // The look-around sample one step before this start is clamped to the
    // first instant, where the apparent Moon cannot be read; that is no
    // sample, not an error.
    let found = engine().aspects_in_range(
        CelestialBody::Sun,
        CelestialBody::Moon,
        Angle::from_degrees(90.0),
        APPARENT,
        tdb(WINDOW_START_JD + 0.01),
        tdb(WINDOW_START_JD + 30.0),
    );
    assert!(found.is_ok(), "{found:?}");
    let at_start = engine().aspects_in_range(
        CelestialBody::Sun,
        CelestialBody::Moon,
        Angle::from_degrees(90.0),
        APPARENT,
        tdb(WINDOW_START_JD),
        tdb(WINDOW_START_JD + 30.0),
    );
    assert!(
        matches!(at_start, Err(EventError::OutOfWindow { .. })),
        "{at_start:?}"
    );
}

#[test]
fn an_aspect_search_the_window_cuts_short_is_out_of_window() {
    // Mercury is never 60 degrees from the Sun; Mercury's pair step is 1 day.
    let never = engine().next_aspect(
        CelestialBody::Sun,
        CelestialBody::Mercury,
        Angle::from_degrees(60.0),
        APPARENT,
        tdb(WINDOW_END_JD - 30.0),
    );
    assert_eq!(out_of_window_jd(never), WINDOW_END_JD + 1.0);
    let at_end = engine().next_aspect(
        CelestialBody::Sun,
        CelestialBody::Moon,
        Angle::from_degrees(0.0),
        APPARENT,
        tdb(WINDOW_END_JD),
    );
    assert_eq!(out_of_window_jd(at_end), WINDOW_END_JD + STEP);
    let before_start = engine().previous_aspect(
        CelestialBody::Sun,
        CelestialBody::Moon,
        Angle::from_degrees(0.0),
        MEAN,
        tdb(WINDOW_START_JD + 0.05),
    );
    assert_eq!(out_of_window_jd(before_start), WINDOW_START_JD - STEP);
}
