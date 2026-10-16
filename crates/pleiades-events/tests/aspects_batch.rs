//! `EventEngine::aspects_in_range_for_angles`, `next_aspect_for_angles` and
//! `previous_aspect_for_angles` against the single-angle methods: one scan
//! gives each angle's events bit for bit (issue #168 (e)).

use pleiades_data::packaged_backend;
use pleiades_events::{AspectEvent, CrossingFrame, CrossingReference, EventEngine, EventError};
use pleiades_types::{Angle, Ayanamsa, CelestialBody, Instant, JulianDay, TimeScale};

const GEO: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const HELIO: CrossingFrame = CrossingFrame::Heliocentric;
const J2000: f64 = 2_451_545.0;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn angles(degrees: &[f64]) -> Vec<Angle> {
    degrees.iter().map(|&d| Angle::from_degrees(d)).collect()
}

fn jd(event: &AspectEvent) -> f64 {
    event.instant.julian_day.days()
}

/// One search case: a pair, its angles (deliberately unsorted), a
/// reference and a range.
struct Case {
    first: CelestialBody,
    second: CelestialBody,
    angles: Vec<Angle>,
    reference: CrossingReference,
    start: f64,
    end: f64,
}

fn cases() -> Vec<Case> {
    use CelestialBody::{Jupiter, Mars, Mercury, Moon, Saturn, Sun};
    vec![
        // A fast pair: conjunction, opposition and both quarters.
        Case {
            first: Sun,
            second: Moon,
            angles: angles(&[180.0, 0.0, 90.0]),
            reference: GEO.into(),
            start: J2000,
            end: J2000 + 120.0,
        },
        // A slow pair whose separation turns: the 1980-81 triple conjunction.
        Case {
            first: Jupiter,
            second: Saturn,
            angles: angles(&[60.0, 0.0, 120.0]),
            reference: GEO.into(),
            start: 2_444_239.5,
            end: 2_445_000.5,
        },
        // A bounded pair: 60 degrees is out of reach and must not disturb 20.
        Case {
            first: Sun,
            second: Mercury,
            angles: angles(&[60.0, 20.0]),
            reference: GEO.into(),
            start: J2000,
            end: J2000 + 400.0,
        },
        // Heliocentric and sidereal references.
        Case {
            first: Mars,
            second: Jupiter,
            angles: angles(&[90.0, 180.0, 0.0]),
            reference: HELIO.into(),
            start: J2000,
            end: J2000 + 3000.0,
        },
        Case {
            first: Sun,
            second: Moon,
            angles: angles(&[120.0, 60.0]),
            reference: CrossingReference::sidereal(GEO, Ayanamsa::Lahiri),
            start: J2000,
            end: J2000 + 60.0,
        },
    ]
}

/// The single-angle results merged in time order: what one scan must give.
fn merged_singles<B: pleiades_backend::EphemerisBackend>(
    engine: &EventEngine<B>,
    case: &Case,
) -> Vec<AspectEvent> {
    let mut all = Vec::new();
    for &angle in &case.angles {
        all.extend(
            engine
                .aspects_in_range(
                    case.first.clone(),
                    case.second.clone(),
                    angle,
                    case.reference.clone(),
                    tdb(case.start),
                    tdb(case.end),
                )
                .expect("single-angle aspects"),
        );
    }
    all.sort_by(|a, b| jd(a).total_cmp(&jd(b)));
    all
}

#[test]
fn one_scan_gives_each_angles_events_bit_for_bit() {
    let engine = EventEngine::new(packaged_backend());
    for case in cases() {
        let batch = engine
            .aspects_in_range_for_angles(
                case.first.clone(),
                case.second.clone(),
                &case.angles,
                case.reference.clone(),
                tdb(case.start),
                tdb(case.end),
            )
            .expect("batch aspects");
        let singles = merged_singles(&engine, &case);
        assert!(
            !singles.is_empty(),
            "{}-{}: no events",
            case.first,
            case.second
        );
        // `AspectEvent: PartialEq` compares the f64 fields with `==`.
        assert_eq!(batch, singles, "{}-{}", case.first, case.second);
    }
}

#[test]
fn next_and_previous_are_the_nearest_of_the_single_angle_answers() {
    let engine = EventEngine::new(packaged_backend());
    for case in cases() {
        let label = format!("{}-{}", case.first, case.second);
        let singles = merged_singles(&engine, &case);
        // From the range start, then chained from each returned event.
        let mut after = case.start;
        for expected in singles.iter() {
            let next = engine
                .next_aspect_for_angles(
                    case.first.clone(),
                    case.second.clone(),
                    &case.angles,
                    case.reference.clone(),
                    tdb(after),
                )
                .expect("next")
                .expect("an event");
            assert_eq!(&next, expected, "{label} next after {after}");
            // Handed back, a returned instant gives that same event.
            let previous = engine
                .previous_aspect_for_angles(
                    case.first.clone(),
                    case.second.clone(),
                    &case.angles,
                    case.reference.clone(),
                    next.instant,
                )
                .expect("previous")
                .expect("an event");
            assert_eq!(previous, next, "{label} previous at {}", jd(&next));
            after = jd(&next);
        }
    }
}

#[test]
fn every_angle_out_of_reach_answers_as_one_does() {
    let engine = EventEngine::new(packaged_backend());
    let (sun, mercury) = (CelestialBody::Sun, CelestialBody::Mercury);
    let far = angles(&[60.0, 90.0]);
    let range = engine
        .aspects_in_range_for_angles(
            sun.clone(),
            mercury.clone(),
            &far,
            GEO,
            tdb(J2000),
            tdb(J2000 + 400.0),
        )
        .expect("range");
    assert!(range.is_empty());
    let single = engine.next_aspect(sun.clone(), mercury.clone(), far[0], GEO, tdb(J2000));
    let batch = engine.next_aspect_for_angles(sun.clone(), mercury.clone(), &far, GEO, tdb(J2000));
    assert!(
        matches!(single, Err(EventError::OutOfWindow { .. })),
        "{single:?}"
    );
    assert_eq!(format!("{batch:?}"), format!("{single:?}"));
    let single = engine.previous_aspect(sun.clone(), mercury.clone(), far[0], GEO, tdb(J2000));
    let batch = engine.previous_aspect_for_angles(sun, mercury, &far, GEO, tdb(J2000));
    assert_eq!(format!("{batch:?}"), format!("{single:?}"));
}

#[test]
fn an_empty_range_is_empty_for_any_angle_set() {
    let engine = EventEngine::new(packaged_backend());
    for (start, end) in [(J2000, J2000), (J2000 + 10.0, J2000)] {
        let events = engine
            .aspects_in_range_for_angles(
                CelestialBody::Sun,
                CelestialBody::Moon,
                &angles(&[0.0, 90.0, 180.0]),
                GEO,
                tdb(start),
                tdb(end),
            )
            .expect("an empty range is not an error");
        assert!(events.is_empty());
    }
}

fn invalid_detail(result: Result<Vec<AspectEvent>, EventError>) -> String {
    match result {
        Err(EventError::InvalidAspect { detail }) => detail,
        other => panic!("expected InvalidAspect, got {other:?}"),
    }
}

#[test]
fn angle_sets_are_validated_in_order() {
    let engine = EventEngine::new(packaged_backend());
    let search = |degrees: &[f64], second: CelestialBody| {
        engine.aspects_in_range_for_angles(
            CelestialBody::Sun,
            second,
            &angles(degrees),
            GEO,
            tdb(J2000),
            tdb(J2000 + 30.0),
        )
    };
    let moon = CelestialBody::Moon;
    assert!(invalid_detail(search(&[], moon.clone())).contains("at least one angle"));
    // The angle check comes before the repeat check: a NaN is not a repeat.
    assert!(
        invalid_detail(search(&[90.0, f64::NAN, f64::NAN], moon.clone()))
            .contains("between 0 and 180")
    );
    assert!(
        invalid_detail(search(&[90.0, 200.0, 90.0], moon.clone())).contains("between 0 and 180")
    );
    assert!(invalid_detail(search(&[90.0, 0.0, 90.0], moon.clone())).contains("given twice"));
    // 0 and -0 are the same separation.
    assert!(invalid_detail(search(&[0.0, -0.0], moon)).contains("given twice"));
    // Then the bodies.
    assert!(
        invalid_detail(search(&[90.0, 0.0], CelestialBody::Sun)).contains("two different bodies")
    );
}
