//! `EventEngine::aspects_in_range`, `next_aspect` and `previous_aspect`: both sides of an
//! angle, retrograde loops, settled instants, chaining, guards and window
//! edges.

use pleiades_data::packaged_backend;
use pleiades_events::{
    AspectEvent, CrossingFrame, CrossingReference, EventEngine, EventError, WINDOW_END_JD,
    WINDOW_START_JD,
};
use pleiades_types::{Angle, Ayanamsa, CelestialBody, Instant, JulianDay, TimeScale, ZodiacMode};

const GEO: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const HELIO: CrossingFrame = CrossingFrame::Heliocentric;
const J2000: f64 = 2_451_545.0;
/// Two seconds, in days: four times the bisection tolerance.
const TWO_SECONDS: f64 = 2.0 / 86_400.0;
/// The Moon moves 7.6e-5 degree in the 0.5 s bisection tolerance.
const EXACT_DEG: f64 = 2e-4;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn aspects(
    first: CelestialBody,
    second: CelestialBody,
    angle_deg: f64,
    reference: impl Into<CrossingReference>,
    start_jd: f64,
    end_jd: f64,
) -> Vec<AspectEvent> {
    EventEngine::new(packaged_backend())
        .aspects_in_range(
            first,
            second,
            Angle::from_degrees(angle_deg),
            reference,
            tdb(start_jd),
            tdb(end_jd),
        )
        .expect("aspects")
}

fn jd(event: &AspectEvent) -> f64 {
    event.instant.julian_day.days()
}

/// `first − second`, wrapped into (-180, 180].
fn separation(event: &AspectEvent) -> f64 {
    let d = event.first_longitude.degrees() - event.second_longitude.degrees();
    (d + 180.0).rem_euclid(360.0) - 180.0
}

/// Every event sits at the angle, on one side or the other, and the list
/// ascends.
fn assert_exact(events: &[AspectEvent], angle_deg: f64) {
    for event in events {
        let off = (separation(event).abs() - angle_deg).abs();
        assert!(off < EXACT_DEG, "{off} deg off at {event:?}");
    }
    for pair in events.windows(2) {
        assert!(jd(&pair[0]) <= jd(&pair[1]), "{pair:?}");
    }
}

fn assert_near(event: &AspectEvent, days_from_j2000: f64) {
    let got = jd(event) - J2000;
    assert!(
        (got - days_from_j2000).abs() < 0.02,
        "got J2000 {got:+.4}, want {days_from_j2000:+.4}"
    );
}

// Jupiter and Saturn met three times through Jupiter's retrograde loop:
// 31 December 1980, 4 March 1981 and 24 July 1981.
#[test]
fn jupiter_saturn_triple_conjunction_of_1981() {
    let found = aspects(
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        0.0,
        GEO,
        2_444_391.5,
        2_444_970.5,
    );
    assert_eq!(found.len(), 3, "{found:?}");
    assert_exact(&found, 0.0);
    assert_near(&found[0], -6939.6077);
    assert_near(&found[1], -6876.7034);
    assert_near(&found[2], -6735.3218);
    assert_eq!(found[0].first, CelestialBody::Jupiter);
    assert_eq!(found[0].second, CelestialBody::Saturn);
    assert_eq!(found[0].angle, Angle::from_degrees(0.0));
    assert_eq!(found[0].frame, GEO);
    assert_eq!(found[0].zodiac, ZodiacMode::Tropical);
}

// The great conjunction of 28 May 2000 was a single meeting.
#[test]
fn jupiter_saturn_single_conjunction_of_2000() {
    let found = aspects(
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        0.0,
        GEO,
        J2000,
        J2000 + 400.0,
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert_exact(&found, 0.0);
    assert_near(&found[0], 148.1701);
}

// New moons of 6 January and 5 February 2000; full moons of 21 January and
// 19 February. Both targets sit on a wrap: 0 where the difference changes
// sign, 180 where it jumps from +180 to -180.
#[test]
fn a_lunation_has_one_conjunction_and_one_opposition() {
    let new_moons = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        0.0,
        GEO,
        J2000,
        J2000 + 60.0,
    );
    assert_eq!(new_moons.len(), 2, "{new_moons:?}");
    assert_exact(&new_moons, 0.0);
    assert_near(&new_moons[0], 5.2602);
    assert_near(&new_moons[1], 35.0447);

    let full_moons = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        180.0,
        GEO,
        J2000,
        J2000 + 60.0,
    );
    assert_eq!(full_moons.len(), 2, "{full_moons:?}");
    assert_exact(&full_moons, 180.0);
    assert_near(&full_moons[0], 19.6955);
    assert_near(&full_moons[1], 49.1859);
}

// First quarter (14 January 2000, the Moon 90 degrees ahead of the Sun) and
// last quarter (28 January, 90 degrees behind) come from one call.
#[test]
fn a_square_is_found_on_both_sides_in_time_order() {
    let found = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        J2000,
        J2000 + 30.0,
    );
    assert_eq!(found.len(), 2, "{found:?}");
    assert_exact(&found, 90.0);
    assert_near(&found[0], 13.0661);
    assert_near(&found[1], 26.8318);
    assert!(separation(&found[0]) < 0.0, "{found:?}");
    assert!(separation(&found[1]) > 0.0, "{found:?}");
}

// Mercury never gets more than 28 degrees from the Sun.
#[test]
fn a_pair_that_never_reaches_the_angle_gives_nothing() {
    let engine = EventEngine::new(packaged_backend());
    assert!(aspects(
        CelestialBody::Sun,
        CelestialBody::Mercury,
        60.0,
        GEO,
        J2000,
        J2000 + 1826.0
    )
    .is_empty());
    let next = engine
        .next_aspect(
            CelestialBody::Sun,
            CelestialBody::Mercury,
            Angle::from_degrees(60.0),
            GEO,
            tdb(WINDOW_END_JD - 400.0),
        )
        .expect("a pair that never perfects is not an error");
    assert_eq!(next, None);
    // The same pair does meet: three conjunctions in the first 130 days of 2000.
    let conjunctions = aspects(
        CelestialBody::Sun,
        CelestialBody::Mercury,
        0.0,
        GEO,
        J2000,
        J2000 + 130.0,
    );
    assert_eq!(conjunctions.len(), 3, "{conjunctions:?}");
    assert_exact(&conjunctions, 0.0);
}

#[test]
fn next_aspect_is_the_first_in_range_and_chains() {
    let engine = EventEngine::new(packaged_backend());
    let next = |after_jd: f64| {
        engine
            .next_aspect(
                CelestialBody::Sun,
                CelestialBody::Moon,
                Angle::from_degrees(90.0),
                GEO,
                tdb(after_jd),
            )
            .expect("next_aspect")
            .expect("the Moon squares the Sun twice a month")
    };
    let in_range = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        J2000,
        J2000 + 60.0,
    );
    assert_eq!(in_range.len(), 4, "{in_range:?}");
    // Same anchor, same grid, same brackets: the same event exactly.
    assert_eq!(next(J2000), in_range[0]);
    // A returned instant handed back as `after` gives the following event.
    let mut after = J2000;
    for expected in &in_range {
        let found = next(after);
        assert!(jd(&found) > after, "{found:?}");
        assert!((jd(&found) - jd(expected)).abs() < TWO_SECONDS, "{found:?}");
        after = jd(&found);
    }
}

// Issue #168 (d).

fn previous(
    first: CelestialBody,
    second: CelestialBody,
    angle_deg: f64,
    before_jd: f64,
) -> Option<AspectEvent> {
    EventEngine::new(packaged_backend())
        .previous_aspect(
            first,
            second,
            Angle::from_degrees(angle_deg),
            GEO,
            tdb(before_jd),
        )
        .expect("previous_aspect")
}

/// Walks `expected` backward from `before_jd`, stepping two seconds behind
/// each event found.
fn assert_walks_backward(
    first: CelestialBody,
    second: CelestialBody,
    angle_deg: f64,
    expected: &[AspectEvent],
    mut before_jd: f64,
) {
    for want in expected.iter().rev() {
        let found = previous(first.clone(), second.clone(), angle_deg, before_jd)
            .unwrap_or_else(|| panic!("nothing before {before_jd}, want {want:?}"));
        assert!(jd(&found) <= before_jd, "{found:?}");
        assert!(
            (jd(&found) - jd(want)).abs() < TWO_SECONDS,
            "{found:?} vs {want:?}"
        );
        assert_exact(std::slice::from_ref(&found), angle_deg);
        before_jd = jd(&found) - TWO_SECONDS;
    }
}

#[test]
fn previous_aspect_walks_a_year_of_lunar_squares_backward() {
    // 24 events, 16 days to a search chunk: every chunk seam is crossed.
    let (start, end) = (J2000, J2000 + 366.0);
    let in_range = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        start,
        end,
    );
    assert_eq!(in_range.len(), 24, "{in_range:?}");
    assert_walks_backward(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        &in_range,
        end,
    );
}

#[test]
fn previous_aspect_walks_a_retrograde_loop_backward() {
    // The Jupiter–Saturn triple conjunction of 1980–81.
    let (start, end) = (J2000 - 7_200.0, J2000 - 6_500.0);
    let in_range = aspects(
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        0.0,
        GEO,
        start,
        end,
    );
    assert_eq!(in_range.len(), 3, "{in_range:?}");
    assert_walks_backward(
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        0.0,
        &in_range,
        end,
    );
}

// The scanner decides both ends of a range by the sign of the separation
// there (issue #168), so a returned instant is a clean place to cut.

#[test]
fn previous_aspect_at_a_returned_instant_is_that_event() {
    let in_range = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        J2000,
        J2000 + 120.0,
    );
    assert_eq!(in_range.len(), 8, "{in_range:?}");
    let engine = EventEngine::new(packaged_backend());
    let is = |found: &Option<AspectEvent>, event: &AspectEvent| {
        found
            .as_ref()
            .is_some_and(|found| (jd(found) - jd(event)).abs() < TWO_SECONDS)
    };
    for event in &in_range {
        let found = previous(CelestialBody::Sun, CelestialBody::Moon, 90.0, jd(event));
        assert!(is(&found, event), "{found:?} is not {event:?}");
        assert!(found.is_some_and(|found| jd(&found) <= jd(event)));
        // Any instant, inside the tolerance or not, has the event on exactly
        // one side of it.
        for offset_s in [-0.6, -0.4, -0.2, 0.2] {
            let at = jd(event) + offset_s / 86_400.0;
            let before = previous(CelestialBody::Sun, CelestialBody::Moon, 90.0, at);
            let after = engine
                .next_aspect(
                    CelestialBody::Sun,
                    CelestialBody::Moon,
                    Angle::from_degrees(90.0),
                    GEO,
                    tdb(at),
                )
                .expect("next_aspect");
            assert!(
                is(&before, event) != is(&after, event),
                "{offset_s} s from {event:?}: {before:?} {after:?}"
            );
        }
    }
}

#[test]
fn aspect_ranges_sharing_an_end_hold_each_event_once() {
    let in_range = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        J2000,
        J2000 + 120.0,
    );
    assert_eq!(in_range.len(), 8, "{in_range:?}");
    // A returned instant trails its event by less than 0.5 s, so these
    // shared ends fall on both sides of the event and inside the tolerance.
    // The lower range keeps the grid that found the event, so the shared end
    // falls inside the step that holds it.
    for (index, event) in in_range.iter().enumerate() {
        for offset_s in [-0.6, -0.4, -0.3, -0.2, -0.1, 0.0, 0.2] {
            let shared = jd(event) + offset_s / 86_400.0;
            let around = |start_jd: f64, end_jd: f64| {
                aspects(
                    CelestialBody::Sun,
                    CelestialBody::Moon,
                    90.0,
                    GEO,
                    start_jd,
                    end_jd,
                )
            };
            let below = around(J2000, shared);
            let above = around(shared, shared + 3.0);
            // The events before this one, and this one on one side only.
            assert_eq!(
                below.len() + above.len(),
                index + 1,
                "end {offset_s} s from {event:?}: {below:?} {above:?}"
            );
            assert!(below.iter().all(|found| jd(found) <= shared), "{below:?}");
        }
    }
}

#[test]
fn previous_aspect_returns_nothing_for_a_pair_that_never_reaches_the_angle() {
    // Mercury is never 60 degrees from the Sun.
    assert_eq!(
        previous(
            CelestialBody::Sun,
            CelestialBody::Mercury,
            60.0,
            WINDOW_START_JD + 400.0
        ),
        None
    );
}

#[test]
fn previous_aspect_guards_match_next_aspect() {
    let engine = EventEngine::new(packaged_backend());
    let call = |first: CelestialBody, second: CelestialBody, angle_deg: f64, before_jd: f64| {
        engine.previous_aspect(
            first,
            second,
            Angle::from_degrees(angle_deg),
            GEO,
            tdb(before_jd),
        )
    };
    assert!(matches!(
        call(CelestialBody::Sun, CelestialBody::Moon, 181.0, J2000),
        Err(EventError::InvalidAspect { .. })
    ));
    assert!(matches!(
        call(CelestialBody::Sun, CelestialBody::Sun, 0.0, J2000),
        Err(EventError::InvalidAspect { .. })
    ));
    assert!(matches!(
        call(CelestialBody::Sun, CelestialBody::Moon, 0.0, 2_000_000.0),
        Err(EventError::OutOfWindow { .. })
    ));
    // The window's first instant has nothing before it.
    assert_eq!(
        call(
            CelestialBody::Sun,
            CelestialBody::Moon,
            0.0,
            WINDOW_START_JD
        ),
        Ok(None)
    );
}

#[test]
fn swapping_the_bodies_swaps_the_longitudes() {
    let sun_moon = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        J2000,
        J2000 + 30.0,
    );
    let moon_sun = aspects(
        CelestialBody::Moon,
        CelestialBody::Sun,
        90.0,
        GEO,
        J2000,
        J2000 + 30.0,
    );
    assert_eq!(sun_moon.len(), moon_sun.len());
    for (a, b) in sun_moon.iter().zip(&moon_sun) {
        assert!((jd(a) - jd(b)).abs() < TWO_SECONDS, "{a:?} {b:?}");
        let first = (a.first_longitude.degrees() - b.second_longitude.degrees()).abs();
        let second = (a.second_longitude.degrees() - b.first_longitude.degrees()).abs();
        assert!(first < 1e-3 && second < 1e-3, "{a:?} {b:?}");
        assert_eq!(b.first, CelestialBody::Moon);
        assert_eq!(b.second, CelestialBody::Sun);
    }
}

// The ayanamsa comes off both longitudes at the same instant and cancels in
// the separation.
#[test]
fn a_sidereal_zodiac_moves_the_longitudes_not_the_instants() {
    let lahiri = CrossingReference::sidereal(GEO, Ayanamsa::Lahiri);
    let tropical = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        J2000,
        J2000 + 30.0,
    );
    let sidereal = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        lahiri.clone(),
        J2000,
        J2000 + 30.0,
    );
    assert_eq!(tropical.len(), 2);
    assert_eq!(sidereal.len(), 2);
    for (t, s) in tropical.iter().zip(&sidereal) {
        assert!((jd(t) - jd(s)).abs() < TWO_SECONDS, "{t:?} {s:?}");
        let shift_first =
            (t.first_longitude.degrees() - s.first_longitude.degrees()).rem_euclid(360.0);
        let shift_second =
            (t.second_longitude.degrees() - s.second_longitude.degrees()).rem_euclid(360.0);
        // Lahiri was 23.85 degrees in January 2000.
        assert!((23.8..23.9).contains(&shift_first), "{shift_first}");
        assert!(
            (shift_first - shift_second).abs() < 1e-3,
            "{shift_first} {shift_second}"
        );
        assert_eq!(s.zodiac, lahiri.zodiac);
    }
}

// From the Sun, Mars laps Jupiter every 2.24 years and never turns back.
#[test]
fn heliocentric_conjunctions_come_once_per_synodic_period() {
    let found = aspects(
        CelestialBody::Mars,
        CelestialBody::Jupiter,
        0.0,
        HELIO,
        J2000,
        J2000 + 3652.0,
    );
    assert_eq!(found.len(), 5, "{found:?}");
    assert_exact(&found, 0.0);
    assert_near(&found[0], 72.2396);
    for pair in found.windows(2) {
        let gap = jd(&pair[1]) - jd(&pair[0]);
        assert!((790.0..850.0).contains(&gap), "{gap}");
    }
    assert_eq!(found[0].frame, HELIO);
}

#[test]
fn edge_angles_work() {
    let conjunctions = |angle_deg: f64| {
        aspects(
            CelestialBody::Sun,
            CelestialBody::Moon,
            angle_deg,
            GEO,
            J2000,
            J2000 + 60.0,
        )
    };
    assert_eq!(conjunctions(-0.0).len(), 2);
    assert_eq!(conjunctions(180.0).len(), 2);
    // A hair off conjunction: both sides, a fraction of a second apart.
    let hair = conjunctions(1e-6);
    assert_eq!(hair.len(), 4, "{hair:?}");
    assert_exact(&hair, 1e-6);
}

#[test]
fn invalid_angles_and_a_repeated_body_are_invalid_aspect() {
    let engine = EventEngine::new(packaged_backend());
    for degrees in [f64::NAN, f64::INFINITY, -1.0, 180.5] {
        // Out of the window as well: the angle is reported first.
        let result = engine.aspects_in_range(
            CelestialBody::Mars,
            CelestialBody::Saturn,
            Angle::from_degrees(degrees),
            GEO,
            tdb(WINDOW_START_JD - 10.0),
            tdb(J2000),
        );
        assert!(
            matches!(result, Err(EventError::InvalidAspect { .. })),
            "{degrees}: {result:?}"
        );
        let result = engine.next_aspect(
            CelestialBody::Mars,
            CelestialBody::Saturn,
            Angle::from_degrees(degrees),
            GEO,
            tdb(J2000),
        );
        assert!(
            matches!(result, Err(EventError::InvalidAspect { .. })),
            "{degrees}: {result:?}"
        );
    }
    let twice = engine.aspects_in_range(
        CelestialBody::Mars,
        CelestialBody::Mars,
        Angle::from_degrees(0.0),
        GEO,
        tdb(J2000),
        tdb(J2000 + 30.0),
    );
    match twice {
        Err(EventError::InvalidAspect { detail }) => assert!(detail.contains("Mars"), "{detail}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn out_of_window_instants_and_undefined_frames_fail_closed() {
    let engine = EventEngine::new(packaged_backend());
    let square = Angle::from_degrees(90.0);
    let range = |reference: CrossingReference, first, second, start: f64, end: f64| {
        engine.aspects_in_range(first, second, square, reference, tdb(start), tdb(end))
    };
    assert!(matches!(
        range(
            GEO.into(),
            CelestialBody::Mars,
            CelestialBody::Saturn,
            WINDOW_START_JD - 1.0,
            J2000
        ),
        Err(EventError::OutOfWindow { .. })
    ));
    assert!(matches!(
        range(
            GEO.into(),
            CelestialBody::Mars,
            CelestialBody::Saturn,
            J2000,
            WINDOW_END_JD + 1.0
        ),
        Err(EventError::OutOfWindow { .. })
    ));
    assert!(matches!(
        engine.next_aspect(
            CelestialBody::Mars,
            CelestialBody::Saturn,
            square,
            GEO,
            tdb(WINDOW_END_JD + 1.0)
        ),
        Err(EventError::OutOfWindow { .. })
    ));
    // The Sun and the Moon have no heliocentric place, in either position.
    for (first, second) in [
        (CelestialBody::Sun, CelestialBody::Mars),
        (CelestialBody::Mars, CelestialBody::Moon),
    ] {
        let result = range(HELIO.into(), first, second, J2000, J2000 + 30.0);
        assert!(
            matches!(result, Err(EventError::UnsupportedFrame { .. })),
            "{result:?}"
        );
    }
    let sidereal_helio = CrossingReference::sidereal(HELIO, Ayanamsa::Lahiri);
    assert!(matches!(
        range(
            sidereal_helio,
            CelestialBody::Sun,
            CelestialBody::Mars,
            J2000,
            J2000 + 30.0
        ),
        Err(EventError::UnsupportedFrame { .. })
    ));
}

// The ayanamsa and the nutation come off both longitudes, so a sidereal
// zodiac moves the reported longitudes and not the heliocentric events
// (issue #106).
#[test]
fn heliocentric_sidereal_aspects_are_the_tropical_events() {
    let lahiri = CrossingReference::sidereal(HELIO, Ayanamsa::Lahiri);
    let find = |reference: CrossingReference| {
        aspects(
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            0.0,
            reference,
            J2000,
            J2000 + 3652.0,
        )
    };
    let tropical = find(HELIO.into());
    let sidereal = find(lahiri.clone());
    assert_eq!(sidereal.len(), 5, "{sidereal:?}");
    assert_exact(&sidereal, 0.0);
    for (s, t) in sidereal.iter().zip(&tropical) {
        assert!((jd(s) - jd(t)).abs() * 86_400.0 < 1.0, "{s:?} vs {t:?}");
        let shift = (t.first_longitude.degrees() - s.first_longitude.degrees()).rem_euclid(360.0);
        assert!((23.0..25.0).contains(&shift), "{shift}");
        assert_eq!(s.frame, HELIO);
        assert_eq!(s.zodiac, lahiri.zodiac);
    }
}

#[test]
fn ranges_touching_the_window_edges_work() {
    let engine = EventEngine::new(packaged_backend());
    let early = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        WINDOW_START_JD,
        WINDOW_START_JD + 40.0,
    );
    assert!(early.len() >= 2, "{early:?}");
    assert_exact(&early, 90.0);
    let late = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        WINDOW_END_JD - 40.0,
        WINDOW_END_JD,
    );
    assert!(late.len() >= 2, "{late:?}");
    assert_exact(&late, 90.0);
    let next = engine
        .next_aspect(
            CelestialBody::Sun,
            CelestialBody::Moon,
            Angle::from_degrees(90.0),
            GEO,
            tdb(WINDOW_END_JD),
        )
        .expect("next_aspect at the window end");
    assert_eq!(next, None);

    // A 2-day-step pair exercises the wider scan margins at both edges.
    let mars_saturn = |start: f64, end: f64| {
        aspects(
            CelestialBody::Mars,
            CelestialBody::Saturn,
            90.0,
            GEO,
            start,
            end,
        )
    };
    let early = mars_saturn(WINDOW_START_JD, WINDOW_START_JD + 400.0);
    assert!(!early.is_empty(), "{early:?}");
    assert_exact(&early, 90.0);
    let late = mars_saturn(WINDOW_END_JD - 400.0, WINDOW_END_JD);
    assert!(!late.is_empty(), "{late:?}");
    assert_exact(&late, 90.0);
    let next = engine
        .next_aspect(
            CelestialBody::Mars,
            CelestialBody::Saturn,
            Angle::from_degrees(90.0),
            GEO,
            tdb(WINDOW_END_JD),
        )
        .expect("next_aspect at the window end for a 2-day-step pair");
    assert_eq!(next, None);
}

#[test]
fn empty_and_inverted_ranges_give_no_events() {
    let sun_moon = |start: f64, end: f64| {
        aspects(
            CelestialBody::Sun,
            CelestialBody::Moon,
            90.0,
            GEO,
            start,
            end,
        )
    };
    assert!(sun_moon(J2000, J2000).is_empty());
    assert!(sun_moon(J2000 + 30.0, J2000).is_empty());
    assert!(sun_moon(WINDOW_END_JD, WINDOW_END_JD).is_empty());
    assert!(sun_moon(WINDOW_START_JD, WINDOW_START_JD).is_empty());
}

#[test]
fn a_body_the_backend_does_not_serve_is_an_error() {
    let engine = EventEngine::new(packaged_backend());
    for (first, second) in [
        (CelestialBody::Ceres, CelestialBody::Mars),
        (CelestialBody::Mars, CelestialBody::Ceres),
    ] {
        let result = engine.aspects_in_range(
            first.clone(),
            second.clone(),
            Angle::from_degrees(0.0),
            GEO,
            tdb(J2000),
            tdb(J2000 + 30.0),
        );
        assert!(matches!(result, Err(EventError::Backend(_))), "{result:?}");
        let next = engine.next_aspect(first, second, Angle::from_degrees(0.0), GEO, tdb(J2000));
        assert!(matches!(next, Err(EventError::Backend(_))), "{next:?}");
    }
}

#[cfg(feature = "serde")]
#[test]
fn an_aspect_event_round_trips_through_serde() {
    let found = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        J2000,
        J2000 + 30.0,
    );
    let json = serde_json::to_string(&found[0]).expect("serialize");
    let back: AspectEvent = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, found[0]);
}
