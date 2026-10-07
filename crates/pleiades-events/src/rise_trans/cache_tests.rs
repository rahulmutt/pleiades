//! The engine's shared sample cache (issue #204) changes how many reads a
//! search makes and never what it returns: every answer here is compared
//! bit for bit with one from a fresh engine.

use std::sync::Arc;

use super::test_support::{chennai, composite, sun_bracket, tt, Composite, BRACKET_QUERY_JD};
use super::*;
use crate::error::WINDOW_END_JD;
use pleiades_types::CelestialBody;

fn fresh() -> EventEngine<Composite> {
    EventEngine::new(composite())
}

fn fresh_brackets(days: u32) -> Vec<[u64; 3]> {
    (0..days)
        .map(|day| sun_bracket(&fresh(), BRACKET_QUERY_JD + f64::from(day)))
        .collect()
}

#[test]
fn a_reused_engine_returns_the_brackets_of_fresh_engines() {
    let reused = fresh();
    let got: Vec<_> = (0..60)
        .map(|day| sun_bracket(&reused, BRACKET_QUERY_JD + f64::from(day)))
        .collect();
    assert_eq!(got, fresh_brackets(60));
}

#[test]
fn a_tiny_cache_gives_the_same_brackets() {
    // Three entries cannot even hold one stencil, so the table is cleared
    // over and over inside each search.
    let tiny = EventEngine::with_place_cache_capacity(composite(), 3);
    let got: Vec<_> = (0..10)
        .map(|day| sun_bracket(&tiny, BRACKET_QUERY_JD + f64::from(day)))
        .collect();
    assert_eq!(got, fresh_brackets(10));
}

#[test]
fn a_shared_engine_answers_the_same_from_several_threads() {
    let engine = Arc::new(fresh());
    let want = fresh_brackets(10);
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let engine = Arc::clone(&engine);
            std::thread::spawn(move || {
                (0..10)
                    .map(|day| sun_bracket(&engine, BRACKET_QUERY_JD + f64::from(day)))
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    for handle in handles {
        assert_eq!(handle.join().expect("no panic"), want);
    }
}

fn moonrise_bits(engine: &EventEngine<Composite>, at_jd: f64) -> u64 {
    engine
        .next_rise_set(
            RiseSetTarget::Body(CelestialBody::Moon),
            RiseSetEvent::Rise,
            chennai(),
            Atmosphere::default(),
            RiseSetOptions::default(),
            tt(at_jd),
        )
        .expect("engine ok")
        .expect("a moonrise")
        .instant
        .julian_day
        .days()
        .to_bits()
}

#[test]
fn interleaved_sun_and_moon_searches_match_fresh_engines() {
    let shared = fresh();
    for day in 0..10 {
        let jd = BRACKET_QUERY_JD + f64::from(day);
        assert_eq!(
            sun_bracket(&shared, jd),
            sun_bracket(&fresh(), jd),
            "day {day}"
        );
        assert_eq!(
            moonrise_bits(&shared, jd),
            moonrise_bits(&fresh(), jd),
            "day {day}"
        );
    }
}

fn sunrise_near_the_end(engine: &EventEngine<Composite>, at_jd: f64) -> String {
    format!(
        "{:?}",
        engine.next_rise_set(
            RiseSetTarget::Body(CelestialBody::Sun),
            RiseSetEvent::Rise,
            chennai(),
            Atmosphere::default(),
            RiseSetOptions::default(),
            tt(at_jd),
        )
    )
}

#[test]
fn a_search_at_the_windows_end_answers_the_same_on_a_reused_engine() {
    // Searches whose samples run past the window's end leave `None` entries
    // in the cache; asked again on the same engine, each answers as a fresh
    // engine does, whether that is an event or `OutOfWindow`.
    let reused = fresh();
    for offset in [3.0, 1.0, 0.6, 0.3, 0.1] {
        let jd = WINDOW_END_JD - offset;
        let first = sunrise_near_the_end(&reused, jd);
        assert_eq!(sunrise_near_the_end(&reused, jd), first, "offset {offset}");
        assert_eq!(sunrise_near_the_end(&fresh(), jd), first, "offset {offset}");
    }
}

#[test]
fn an_engine_stays_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<EventEngine<Composite>>();
}
