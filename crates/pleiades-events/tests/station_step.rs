//! `StationOptions`: a caller-chosen scan step for the station finder
//! (issue #167 (a)).

use pleiades_data::packaged_backend;
use pleiades_events::{
    CrossingFrame, EventEngine, EventError, Station, StationOptions, MIN_STATION_STEP_DAYS,
};
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

const GEO: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const J2000: f64 = 2_451_545.0;
/// Two seconds, in days: four times the bisection tolerance.
const TWO_SECONDS: f64 = 2.0 / 86_400.0;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn jd(station: &Station) -> f64 {
    station.instant.julian_day.days()
}

fn fine() -> StationOptions {
    StationOptions::default().with_step_days(0.02)
}

#[test]
fn default_options_match_the_plain_methods() {
    let engine = EventEngine::new(packaged_backend());
    let options = StationOptions::default();
    for body in [CelestialBody::Mercury, CelestialBody::TrueNode] {
        let (start, end) = (tdb(J2000), tdb(J2000 + 120.0));
        assert_eq!(
            engine.stations_in_range(body.clone(), GEO, start, end),
            engine.stations_in_range_with(body.clone(), GEO, start, end, options),
            "{body:?}"
        );
        assert_eq!(
            engine.next_station(body.clone(), GEO, start),
            engine.next_station_with(body.clone(), GEO, start, options),
            "{body:?}"
        );
        assert_eq!(
            engine.previous_station(body.clone(), GEO, end),
            engine.previous_station_with(body.clone(), GEO, end, options),
            "{body:?}"
        );
    }
}

/// Fifteen days of 2001 around a true-node direct spell of about 3.7 hours
/// (JD 2452000.83–2452000.99, 1 April 2001; measured 2026-10-07 on the
/// packaged backend),
/// shorter than the default 0.25-day step. Over J2000 + 730 days there are
/// three such spells: 102 stations at the default step, 108 at 0.02 day.
const SHORT_SPELL: (f64, f64) = (J2000 + 450.0, J2000 + 465.0);

#[test]
fn a_finer_step_finds_the_true_node_grazes_the_default_step_misses() {
    let engine = EventEngine::new(packaged_backend());
    let (start, end) = (tdb(SHORT_SPELL.0), tdb(SHORT_SPELL.1));
    let coarse = engine
        .stations_in_range(CelestialBody::TrueNode, GEO, start, end)
        .expect("default step");
    let found = engine
        .stations_in_range_with(CelestialBody::TrueNode, GEO, start, end, fine())
        .expect("fine step");
    // The short spell is one station pair the default step does not see.
    assert_eq!(found.len(), coarse.len() + 2, "{found:?}\n{coarse:?}");
    for pair in found.windows(2) {
        assert_ne!(pair[0].kind, pair[1].kind, "{pair:?}");
        assert!(jd(&pair[0]) < jd(&pair[1]), "{pair:?}");
    }
    // At least one pair is closer together than the default step.
    assert!(
        found
            .windows(2)
            .any(|pair| jd(&pair[1]) - jd(&pair[0]) < 0.25),
        "no sub-step pair found"
    );
    // Every station the default step finds, the fine step finds too.
    for station in &coarse {
        assert!(
            found
                .iter()
                .any(|f| f.kind == station.kind && (jd(f) - jd(station)).abs() < TWO_SECONDS),
            "{station:?} missing from the fine-step list"
        );
    }
}

#[test]
fn next_and_previous_with_a_fine_step_walk_the_fine_range() {
    let engine = EventEngine::new(packaged_backend());
    // Through the short spell, so the walk chains across a sub-step pair.
    let (start_jd, end_jd) = SHORT_SPELL;
    let found = engine
        .stations_in_range_with(
            CelestialBody::TrueNode,
            GEO,
            tdb(start_jd),
            tdb(end_jd),
            fine(),
        )
        .expect("range");
    assert!(found.len() >= 2, "{found:?}");

    let mut after = tdb(start_jd);
    for expected in &found {
        let next = engine
            .next_station_with(CelestialBody::TrueNode, GEO, after, fine())
            .expect("next")
            .expect("a station");
        assert_eq!(next.kind, expected.kind);
        assert!((jd(&next) - jd(expected)).abs() < TWO_SECONDS, "{next:?}");
        after = next.instant;
    }

    let mut before = tdb(end_jd);
    for expected in found.iter().rev() {
        let previous = engine
            .previous_station_with(CelestialBody::TrueNode, GEO, before, fine())
            .expect("previous")
            .expect("a station");
        assert_eq!(previous.kind, expected.kind);
        assert!(
            (jd(&previous) - jd(expected)).abs() < TWO_SECONDS,
            "{previous:?}"
        );
        before = tdb(jd(&previous) - 1.0 / 86_400.0);
    }
}

fn is_invalid_step(result: Result<impl std::fmt::Debug, EventError>, step: f64) -> bool {
    match result {
        Err(EventError::InvalidStationStep { step_days, .. }) => {
            step_days.to_bits() == step.to_bits()
        }
        _ => false,
    }
}

#[test]
fn invalid_steps_are_refused_for_every_search() {
    let engine = EventEngine::new(packaged_backend());
    let (start, end) = (tdb(J2000), tdb(J2000 + 30.0));
    let cases = [
        // The true node's default step is 0.25 day; the Sun's is 1 day.
        (CelestialBody::TrueNode, f64::NAN),
        (CelestialBody::TrueNode, f64::INFINITY),
        (CelestialBody::TrueNode, 0.0),
        (CelestialBody::TrueNode, -0.1),
        (CelestialBody::TrueNode, MIN_STATION_STEP_DAYS / 2.0),
        (CelestialBody::TrueNode, 0.5),
        (CelestialBody::Mercury, 1.5),
        // A body answered without a scan still has its step checked.
        (CelestialBody::Sun, 2.0),
        (CelestialBody::Sun, 0.0),
    ];
    for (body, step) in cases {
        let options = StationOptions::default().with_step_days(step);
        assert!(
            is_invalid_step(
                engine.stations_in_range_with(body.clone(), GEO, start, end, options),
                step
            ),
            "range {body:?} {step}"
        );
        assert!(
            is_invalid_step(
                engine.next_station_with(body.clone(), GEO, start, options),
                step
            ),
            "next {body:?} {step}"
        );
        assert!(
            is_invalid_step(
                engine.previous_station_with(body.clone(), GEO, end, options),
                step
            ),
            "previous {body:?} {step}"
        );
    }
}

#[test]
fn the_step_bounds_are_inclusive() {
    let engine = EventEngine::new(packaged_backend());
    let (start, end) = (tdb(J2000), tdb(J2000 + 1.0));
    for step in [MIN_STATION_STEP_DAYS, 0.25] {
        let options = StationOptions::default().with_step_days(step);
        engine
            .stations_in_range_with(CelestialBody::TrueNode, GEO, start, end, options)
            .unwrap_or_else(|err| panic!("step {step}: {err}"));
    }
}

#[test]
fn the_invalid_step_error_names_the_body_and_the_bounds() {
    let engine = EventEngine::new(packaged_backend());
    let options = StationOptions::default().with_step_days(0.5);
    let err = engine
        .next_station_with(CelestialBody::TrueNode, GEO, tdb(J2000), options)
        .expect_err("coarser than the default");
    let text = err.to_string();
    assert!(text.contains("true node"), "{text}");
    assert!(text.contains("0.5"), "{text}");
    assert!(text.contains("0.25"), "{text}");
}
