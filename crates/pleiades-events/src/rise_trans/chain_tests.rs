//! Regression tests for issues #80 and #81: a search started from an instant
//! the engine itself returned must treat the event that instant describes as
//! already past, and a night (or day) shorter than the scan step must be
//! found wherever it falls relative to the search start.
//!
//! Fixture: the issues' own evidence, the Sun over
//! `CompositeBackend<ElpBackend, Vsop87Backend>` with default
//! `RiseSetOptions` and `Atmosphere`, query instants tagged `Tt`.

use super::*;
use pleiades_backend::CompositeBackend;
use pleiades_elp::ElpBackend;
use pleiades_types::CelestialBody;
use pleiades_vsop87::Vsop87Backend;

const SECOND: f64 = 1.0 / 86_400.0;
const MINUTE: f64 = 60.0 * SECOND;
/// 1999-01-01 00:00 as a Julian Day number: the first day of #80's sweep.
const JD_1999_01_01: f64 = 2_451_179.5;
/// 2000-06-21 12:00 as a Julian Day number: #81's query instant.
const JD_2000_06_21_NOON: f64 = 2_451_717.0;

/// The Sun for one observer, with the searches reduced to Julian Days.
struct SunAt {
    engine: EventEngine<CompositeBackend<ElpBackend, Vsop87Backend>>,
    observer: ObserverLocation,
}

impl SunAt {
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

    fn next(&self, event: RiseSetEvent, after: Instant) -> Option<Instant> {
        self.engine
            .next_rise_set(
                RiseSetTarget::Body(CelestialBody::Sun),
                event,
                self.observer.clone(),
                Atmosphere::default(),
                RiseSetOptions::default(),
                after,
            )
            .expect("engine ok")
            .map(|found| found.instant)
    }

    fn previous(&self, event: RiseSetEvent, before: Instant) -> Option<Instant> {
        self.engine
            .previous_rise_set(
                RiseSetTarget::Body(CelestialBody::Sun),
                event,
                self.observer.clone(),
                Atmosphere::default(),
                RiseSetOptions::default(),
                before,
            )
            .expect("engine ok")
            .map(|found| found.instant)
    }

    fn in_range(&self, event: RiseSetEvent, start: Instant, end: Instant) -> Vec<Instant> {
        self.engine
            .rise_sets_in_range(
                RiseSetTarget::Body(CelestialBody::Sun),
                event,
                self.observer.clone(),
                Atmosphere::default(),
                RiseSetOptions::default(),
                start,
                end,
            )
            .expect("engine ok")
            .into_iter()
            .map(|found| found.instant)
            .collect()
    }
}

fn tt(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

fn days(instant: Instant) -> f64 {
    instant.julian_day.days()
}

/// Returned instants shifted by a number of days, keeping their `Tdb` tag.
fn shifted(instant: Instant, by_days: f64) -> Instant {
    Instant::new(JulianDay::from_days(days(instant) + by_days), instant.scale)
}

/// #80's two sites, Chennai and Greenwich, as (latitude, east longitude).
const SWEEP_SITES: [(f64, f64); 2] = [(13.0827, 80.2707), (51.4779, 0.0)];

/// #80's probes, thinned to every fifteenth day of 1999: local mean midnight
/// at an observer east longitude. The issue re-found the event in about half
/// of its brackets, so 25 per site and event leave no room for luck.
fn midnight_probes(longitude_deg: f64) -> impl Iterator<Item = f64> {
    (0..366)
        .step_by(15)
        .map(move |day| JD_1999_01_01 + f64::from(day) - longitude_deg / 360.0)
}

/// Every site, probe, and daily event kind of the sweep.
fn sweep() -> Vec<(SunAt, f64, RiseSetEvent)> {
    let mut out = Vec::new();
    for (latitude, longitude) in SWEEP_SITES {
        for probe in midnight_probes(longitude) {
            for event in DAILY_EVENTS {
                out.push((SunAt::new(latitude, longitude), probe, event));
            }
        }
    }
    out
}

const DAILY_EVENTS: [RiseSetEvent; 4] = [
    RiseSetEvent::Rise,
    RiseSetEvent::Set,
    RiseSetEvent::UpperTransit,
    RiseSetEvent::LowerTransit,
];

// Issue #80.
#[test]
fn next_after_a_returned_previous_event_is_the_following_one() {
    for (sun, probe, event) in sweep() {
        let found = sun.previous(event, tt(probe)).expect("a daily event");
        let following = sun.next(event, found).expect("a daily event");
        let gap = days(following) - days(found);
        assert!(
            (0.9..1.1).contains(&gap),
            "{event:?} probe {probe}: {} then {} ({:.3} s later)",
            days(found),
            days(following),
            gap / SECOND
        );
    }
}

// The mirror of #80: `previous` means "at or before", so an instant the
// engine returned finds its own event, not the one a day earlier.
#[test]
fn previous_from_a_returned_next_event_is_that_event() {
    for (sun, probe, event) in sweep() {
        let found = sun.next(event, tt(probe)).expect("a daily event");
        let again = sun.previous(event, found).expect("a daily event");
        let gap = days(found) - days(again);
        assert!(
            (0.0..=SECOND).contains(&gap),
            "{event:?} probe {probe}: {} then {} ({:.3} s earlier)",
            days(found),
            days(again),
            gap / SECOND
        );
    }
}

#[test]
fn range_starting_at_a_returned_event_holds_only_later_ones() {
    for (sun, probe, event) in sweep() {
        let found = sun.next(event, tt(probe)).expect("a daily event");
        let later = sun.in_range(event, found, shifted(found, 1.5));
        assert_eq!(later.len(), 1, "{event:?} probe {probe}: {later:?}");
        let gap = days(later[0]) - days(found);
        assert!((0.9..1.1).contains(&gap), "{event:?} probe {probe}: {gap}");
    }
}

/// #81's sweep: 20 E on 2000-06-21, 65.50 N to 65.68 N in 0.01° steps, where
/// the night lasts between an hour and twelve minutes. 65.69 N has no set.
fn short_night_observers() -> impl Iterator<Item = (f64, SunAt)> {
    (0..19).map(|i| {
        let latitude = 65.50 + 0.01 * f64::from(i);
        (latitude, SunAt::new(latitude, 20.0))
    })
}

/// The sunrise that began the day holding #81's query, the sunset that ended
/// it, and the sunrise after that short night, each located from an instant
/// well clear of all three.
fn short_night(sun: &SunAt) -> (Instant, Instant, Instant) {
    let rise = sun
        .previous(RiseSetEvent::Rise, tt(JD_2000_06_21_NOON))
        .expect("a sunrise");
    let set = sun
        .next(RiseSetEvent::Set, tt(JD_2000_06_21_NOON))
        .expect("a sunset");
    let next_rise = sun
        .previous(RiseSetEvent::Rise, tt(JD_2000_06_21_NOON + 1.0))
        .expect("a sunrise");
    (rise, set, next_rise)
}

#[test]
fn short_night_fixture_is_a_night_under_an_hour_and_a_bit() {
    for (latitude, sun) in short_night_observers() {
        let (rise, set, next_rise) = short_night(&sun);
        let day = days(next_rise) - days(rise);
        let night = days(next_rise) - days(set);
        assert!((0.99..1.01).contains(&day), "{latitude}: day {day}");
        assert!(
            night > 10.0 * MINUTE && night < 61.0 * MINUTE,
            "{latitude}: night {:.1} min",
            night / MINUTE
        );
    }
}

// Issue #81.
#[test]
fn next_rise_after_a_returned_sunset_ends_the_short_night() {
    for (latitude, sun) in short_night_observers() {
        let (_, set, next_rise) = short_night(&sun);
        let found = sun.next(RiseSetEvent::Rise, set).expect("a sunrise");
        assert!(
            (days(found) - days(next_rise)).abs() < SECOND,
            "{latitude}: sunrise {} after the sunset {}, expected {}",
            days(found),
            days(set),
            days(next_rise)
        );
    }
}

// #81 widened: the search start need not be a returned instant.
#[test]
fn next_rise_from_shortly_before_the_sunset_ends_the_short_night() {
    for (latitude, sun) in short_night_observers() {
        let (_, set, next_rise) = short_night(&sun);
        for minutes_before in [5.0, 20.0, 40.0] {
            let found = sun
                .next(RiseSetEvent::Rise, shifted(set, -minutes_before * MINUTE))
                .expect("a sunrise");
            assert!(
                (days(found) - days(next_rise)).abs() < SECOND,
                "{latitude}, {minutes_before} min before the sunset: sunrise {} expected {}",
                days(found),
                days(next_rise)
            );
        }
    }
}

#[test]
fn previous_set_from_shortly_after_the_sunrise_began_the_short_night() {
    for (latitude, sun) in short_night_observers() {
        let (_, set, next_rise) = short_night(&sun);
        for minutes_after in [5.0, 20.0, 40.0] {
            let found = sun
                .previous(
                    RiseSetEvent::Set,
                    shifted(next_rise, minutes_after * MINUTE),
                )
                .expect("a sunset");
            assert!(
                (days(found) - days(set)).abs() < SECOND,
                "{latitude}, {minutes_after} min after the sunrise: sunset {} expected {}",
                days(found),
                days(set)
            );
        }
    }
}

#[test]
fn range_clipped_around_the_short_night_holds_its_set_and_rise() {
    for (latitude, sun) in short_night_observers() {
        let (_, set, next_rise) = short_night(&sun);
        let start = shifted(set, -5.0 * MINUTE);
        let end = shifted(next_rise, 5.0 * MINUTE);
        let sets = sun.in_range(RiseSetEvent::Set, start, end);
        let rises = sun.in_range(RiseSetEvent::Rise, start, end);
        assert_eq!(sets.len(), 1, "{latitude}: sets {sets:?}");
        assert_eq!(rises.len(), 1, "{latitude}: rises {rises:?}");
        assert!((days(sets[0]) - days(set)).abs() < SECOND, "{latitude}");
        assert!(
            (days(rises[0]) - days(next_rise)).abs() < SECOND,
            "{latitude}"
        );
    }
}
