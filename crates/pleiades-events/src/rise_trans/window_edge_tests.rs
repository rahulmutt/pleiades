//! Regression tests for issue #203: searches near either end of the
//! ephemeris window. An event inside the window is found, and a search the
//! window cuts short says so with `OutOfWindow`. `Ok(None)` is left to mean
//! only that the search span holds no event (a circumpolar target).
//!
//! Fixture: the issue's own evidence, the Sun over
//! `CompositeBackend<ElpBackend, Vsop87Backend>` with default
//! `RiseSetOptions`, query instants tagged `Tt`. The expected instants are
//! Swiss Ephemeris's (pyswisseph 2.10, Moshier, `swe.rise_trans` with its
//! default `atpress = 0, attemp = 0`, so the searches use
//! `Atmosphere::SE_DEFAULT_CALL`), converted to TT with `swe.deltat`.

use super::*;
use pleiades_backend::CompositeBackend;
use pleiades_elp::ElpBackend;
use pleiades_types::CelestialBody;
use pleiades_vsop87::Vsop87Backend;

const HOUR: f64 = 1.0 / 24.0;
const SECOND: f64 = 1.0 / 86_400.0;

/// Swiss Ephemeris: Shanghai's sunrise 1.10 h before the window's end.
const SHANGHAI_LAST_SUNRISE_TT: f64 = 2_488_069.454_09;
/// Swiss Ephemeris: the Sun's lower transit at 20°E 1.25 h before the end.
const LAST_LOWER_TRANSIT_TT: f64 = 2_488_069.447_71;
/// Swiss Ephemeris: Dhaka's sunrise 0.68 h after the window's start.
const DHAKA_FIRST_SUNRISE_TT: f64 = 2_415_020.528_35;

/// Agreement asked of the 1900 instant. Measured 2026-10-08 (issue #242):
/// the engine is 0.6 s before Swiss Ephemeris.
const START_PARITY_TOLERANCE_DAYS: f64 = 5.0 * SECOND;
/// Agreement asked of the 2100 instants. Measured 2026-10-08 (issue #242):
/// the sunrise is 51.1 s and the lower transit 51.2 s after Swiss Ephemeris.
/// An offset shared by a rising and a transit is one of Earth rotation: the
/// engine's Delta T is extrapolated beyond 2020, and differs from Swiss
/// Ephemeris's by 2100.
const END_PARITY_TOLERANCE_DAYS: f64 = 60.0 * SECOND;

type Found = Result<Option<f64>, EventError>;

/// One observer, with the searches reduced to Julian Days.
struct At {
    engine: EventEngine<CompositeBackend<ElpBackend, Vsop87Backend>>,
    observer: ObserverLocation,
}

impl At {
    fn new(latitude_deg: f64, longitude_deg: f64) -> Self {
        Self {
            engine: EventEngine::new(CompositeBackend::new(
                ElpBackend::new(),
                Vsop87Backend::new(),
            )),
            observer: ObserverLocation::new(
                Latitude::from_degrees(latitude_deg),
                Longitude::from_degrees(longitude_deg),
                None,
            ),
        }
    }

    fn shanghai() -> Self {
        Self::new(31.23, 121.47)
    }

    fn london() -> Self {
        Self::new(51.5, -0.13)
    }

    fn dhaka() -> Self {
        Self::new(23.81, 90.41)
    }

    fn next_of(&self, target: RiseSetTarget, event: RiseSetEvent, after_jd: f64) -> Found {
        self.engine
            .next_rise_set(
                target,
                event,
                self.observer.clone(),
                Atmosphere::SE_DEFAULT_CALL,
                RiseSetOptions::default(),
                tt(after_jd),
            )
            .map(|found| found.map(|event| event.instant.julian_day.days()))
    }

    fn next(&self, event: RiseSetEvent, after_jd: f64) -> Found {
        self.next_of(sun(), event, after_jd)
    }

    fn previous(&self, event: RiseSetEvent, before_jd: f64) -> Found {
        self.engine
            .previous_rise_set(
                sun(),
                event,
                self.observer.clone(),
                Atmosphere::SE_DEFAULT_CALL,
                RiseSetOptions::default(),
                tt(before_jd),
            )
            .map(|found| found.map(|event| event.instant.julian_day.days()))
    }

    fn in_range(
        &self,
        event: RiseSetEvent,
        start_jd: f64,
        end_jd: f64,
    ) -> Result<Vec<f64>, EventError> {
        self.engine
            .rise_sets_in_range(
                sun(),
                event,
                self.observer.clone(),
                Atmosphere::SE_DEFAULT_CALL,
                RiseSetOptions::default(),
                tt(start_jd),
                tt(end_jd),
            )
            .map(|found| {
                found
                    .iter()
                    .map(|event| event.instant.julian_day.days())
                    .collect()
            })
    }
}

fn sun() -> RiseSetTarget {
    RiseSetTarget::Body(CelestialBody::Sun)
}

fn tt(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

fn found(result: Found) -> f64 {
    match result {
        Ok(Some(jd)) => jd,
        other => panic!("expected an event, got {other:?}"),
    }
}

/// The Julian Day an `OutOfWindow` error names.
fn out_of_window_jd<T: std::fmt::Debug>(result: Result<T, EventError>) -> f64 {
    match result {
        Err(EventError::OutOfWindow { julian_day }) => julian_day,
        other => panic!("expected OutOfWindow, got {other:?}"),
    }
}

fn assert_close(got: f64, want: f64, tolerance: f64) {
    assert!(
        (got - want).abs() <= tolerance,
        "JD {got} is {:.1} s from JD {want}",
        (got - want) / SECOND
    );
}

#[test]
fn a_sunrise_in_the_last_two_hours_of_the_window_is_found() {
    let shanghai = At::shanghai();
    let rise = found(shanghai.next(RiseSetEvent::Rise, WINDOW_END_JD - 0.5));
    assert_close(rise, SHANGHAI_LAST_SUNRISE_TT, END_PARITY_TOLERANCE_DAYS);
    // The backward search from the window's last instant returns that same
    // sunrise, not the one a day before it.
    let back = found(shanghai.previous(RiseSetEvent::Rise, WINDOW_END_JD));
    assert_close(back, rise, SECOND);
    let back = found(shanghai.previous(RiseSetEvent::Rise, WINDOW_END_JD - 0.5 * HOUR));
    assert_close(back, rise, SECOND);
    // And a range that ends with the window holds it.
    let rises = shanghai
        .in_range(RiseSetEvent::Rise, WINDOW_END_JD - 1.0, WINDOW_END_JD)
        .unwrap();
    assert_eq!(rises.len(), 1, "{rises:?}");
    assert_close(rises[0], rise, SECOND);
}

#[test]
fn a_transit_in_the_last_two_hours_of_the_window_is_found() {
    let transit = found(At::new(45.0, 20.0).next(RiseSetEvent::LowerTransit, WINDOW_END_JD - 0.5));
    assert_close(transit, LAST_LOWER_TRANSIT_TT, END_PARITY_TOLERANCE_DAYS);
}

#[test]
fn a_search_that_runs_past_the_windows_end_reports_out_of_window() {
    // London's next sunrise is 8.1 h past the end.
    let london = At::london();
    for after in [WINDOW_END_JD - 0.5, WINDOW_END_JD - HOUR, WINDOW_END_JD] {
        let needed = out_of_window_jd(london.next(RiseSetEvent::Rise, after));
        assert!(needed > WINDOW_END_JD, "after {after}: named JD {needed}");
        assert!(
            needed <= WINDOW_END_JD + HOUR + SECOND,
            "after {after}: named JD {needed}"
        );
    }
    // The sunset before it is inside the window and is still found.
    let set = found(london.next(RiseSetEvent::Set, WINDOW_END_JD - 0.5));
    assert_close(set, 2_488_069.169_39, SECOND);
    // No transit follows the Sun's last one before the end.
    let last_upper = found(london.previous(RiseSetEvent::UpperTransit, WINDOW_END_JD));
    let needed = out_of_window_jd(london.next(RiseSetEvent::UpperTransit, last_upper));
    assert!(needed > WINDOW_END_JD, "named JD {needed}");
}

#[test]
fn a_sunrise_in_the_first_hour_of_the_window_is_found() {
    // The Sun's apparent place needs no read at a retarded instant, so it is
    // readable from the window's first instant on.
    let dhaka = At::dhaka();
    let rise = found(dhaka.next(RiseSetEvent::Rise, WINDOW_START_JD));
    assert_close(rise, DHAKA_FIRST_SUNRISE_TT, START_PARITY_TOLERANCE_DAYS);
    // From half an hour in, the guard sample would fall before the window.
    let again = found(dhaka.next(RiseSetEvent::Rise, WINDOW_START_JD + 0.5 * HOUR));
    assert_close(again, rise, SECOND);
    // The backward search from six hours in returns that same sunrise.
    let back = found(dhaka.previous(RiseSetEvent::Rise, WINDOW_START_JD + 0.25));
    assert_close(back, rise, SECOND);
    // And a range that starts with the window holds it.
    let rises = dhaka
        .in_range(RiseSetEvent::Rise, WINDOW_START_JD, WINDOW_START_JD + 1.0)
        .unwrap();
    assert_eq!(rises.len(), 1, "{rises:?}");
    assert_close(rises[0], rise, SECOND);
}

#[test]
fn a_search_that_runs_into_the_windows_start_reports_out_of_window() {
    let dhaka = At::dhaka();
    // A quarter of an hour into the window the first sunrise is still to
    // come: the previous one is on 1899-12-31.
    let needed =
        out_of_window_jd(dhaka.previous(RiseSetEvent::Rise, WINDOW_START_JD + 0.25 * HOUR));
    assert!(needed < WINDOW_START_JD, "named JD {needed}");
    assert!(needed >= WINDOW_START_JD - HOUR, "named JD {needed}");
    // So is the Sun's previous upper transit, asked 2.4 h in.
    let needed =
        out_of_window_jd(dhaka.previous(RiseSetEvent::UpperTransit, WINDOW_START_JD + 0.1));
    assert!(needed < WINDOW_START_JD, "named JD {needed}");
}

#[test]
fn a_planet_within_light_time_of_the_windows_start_reports_out_of_window() {
    // A planet's apparent place is read at the retarded instant, which falls
    // before the window for a query at its start. The search has no sample
    // to anchor on, and must not answer with a later event or with none.
    let mars = || RiseSetTarget::Body(CelestialBody::Mars);
    let dhaka = At::dhaka();
    out_of_window_jd(dhaka.next_of(mars(), RiseSetEvent::Rise, WINDOW_START_JD));
    // A day in, it is searched as anywhere else.
    let rise = found(dhaka.next_of(mars(), RiseSetEvent::Rise, WINDOW_START_JD + 1.0));
    assert!(rise > WINDOW_START_JD + 1.0 && rise < WINDOW_START_JD + 2.1);
}

#[test]
fn a_circumpolar_sun_is_still_no_event() {
    // Tromsø, polar night, 2026-12-20 12:00 TT.
    let tromso = At::new(69.65, 18.96);
    assert_eq!(tromso.next(RiseSetEvent::Rise, 2_461_395.0).unwrap(), None);
    assert_eq!(
        tromso.previous(RiseSetEvent::Rise, 2_461_395.0).unwrap(),
        None
    );
}
