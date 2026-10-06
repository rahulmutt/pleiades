//! Regression tests for issue #204: a rise/set or transit search reads its
//! body at the few lattice instants its span touches, however many times the
//! residual is evaluated on the way to the event.
//!
//! Fixture: the issue's own, the Sun at Chennai over
//! `CompositeBackend<ElpBackend, Vsop87Backend>` with default options.

use super::test_support::{composite, Composite, CountingBackend};
use super::*;
use pleiades_types::CelestialBody;

/// 2025-06-01 06:00 TT.
const QUERY_JD: f64 = 2_460_827.75;

fn chennai() -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(13.08),
        Longitude::from_degrees(80.27),
        Some(0.0),
    )
}

fn engine() -> EventEngine<CountingBackend<Composite>> {
    EventEngine::new(CountingBackend::new(composite()))
}

fn tt(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

fn search(
    engine: &EventEngine<CountingBackend<Composite>>,
    body: CelestialBody,
    event: RiseSetEvent,
    forward: bool,
    at: Instant,
) -> Instant {
    let target = RiseSetTarget::Body(body);
    let found = if forward {
        engine.next_rise_set(
            target,
            event,
            chennai(),
            Atmosphere::default(),
            RiseSetOptions::default(),
            at,
        )
    } else {
        engine.previous_rise_set(
            target,
            event,
            chennai(),
            Atmosphere::default(),
            RiseSetOptions::default(),
            at,
        )
    };
    found.expect("engine ok").expect("the event exists").instant
}

/// The three searches an observer-local calendar makes per day: the sunrise
/// that began the day, the sunset after it, and the next sunrise. Before the
/// body track each made 42 to 55 reads of the Sun (two per residual
/// evaluation); with it, the lattice samples around a span of at most a day.
#[test]
fn a_daily_sunrise_bracket_reads_the_sun_a_few_times() {
    let engine = engine();
    let rise = search(
        &engine,
        CelestialBody::Sun,
        RiseSetEvent::Rise,
        false,
        tt(QUERY_JD),
    );
    let previous_rise_reads = engine.backend.take_reads();
    let set = search(&engine, CelestialBody::Sun, RiseSetEvent::Set, true, rise);
    let next_set_reads = engine.backend.take_reads();
    search(&engine, CelestialBody::Sun, RiseSetEvent::Rise, true, set);
    let next_rise_reads = engine.backend.take_reads();
    for (label, reads) in [
        ("previous rise", previous_rise_reads),
        ("next set", next_set_reads),
        ("next rise", next_rise_reads),
    ] {
        assert!(
            reads <= MAX_SUN_READS_PER_SEARCH,
            "{label}: {reads} reads of the Sun"
        );
    }
}

/// A search over at most a day touches three 12-hour lattice intervals at
/// most, which is six samples.
const MAX_SUN_READS_PER_SEARCH: usize = 6;

/// The Moon's lattice is 6 hours and each sample costs its light-time
/// iteration, so a search costs more reads than the Sun's, but still a small
/// multiple of the samples its span touches and far fewer than the two
/// light-time iterations per residual evaluation it replaced.
#[test]
fn a_moonrise_search_reads_the_moon_at_its_lattice_samples_only() {
    let engine = engine();
    search(
        &engine,
        CelestialBody::Moon,
        RiseSetEvent::Rise,
        true,
        tt(QUERY_JD),
    );
    let reads = engine.backend.take_reads();
    assert!(
        reads <= MAX_MOON_READS_PER_SEARCH,
        "{reads} reads of the Moon"
    );
}

/// Measured 2026-10-06: 18 reads, six lattice samples of three reads each
/// (the light-time iteration). A search over a lunar day touches at most
/// eight samples.
const MAX_MOON_READS_PER_SEARCH: usize = 24;

/// A transit search shares the track.
#[test]
fn a_transit_search_reads_the_sun_a_few_times() {
    let engine = engine();
    search(
        &engine,
        CelestialBody::Sun,
        RiseSetEvent::UpperTransit,
        true,
        tt(QUERY_JD),
    );
    let reads = engine.backend.take_reads();
    assert!(
        reads <= MAX_SUN_READS_PER_SEARCH,
        "{reads} reads of the Sun"
    );
}
