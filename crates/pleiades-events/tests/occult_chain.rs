//! Occultation searches select by the maximum instant, and that instant does
//! not depend on where the search started (issue #159): a returned maximum
//! handed back gives the neighbouring occultation, and an instant near a
//! maximum has that occultation on exactly one side.

use pleiades_apparent::Atmosphere;
use pleiades_data::packaged_backend;
use pleiades_events::{EventEngine, GlobalOccultation, LocalOccultation, OccultTarget};
use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};

const MINUTE: f64 = 1.0 / 1440.0;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn madrid() -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(-3.7),
        Some(650.0),
    )
}

fn aldebaran() -> OccultTarget {
    OccultTarget::Star("Aldebaran".into())
}

fn next(after_jd: f64) -> Option<LocalOccultation> {
    EventEngine::new(packaged_backend())
        .next_occultation(aldebaran(), madrid(), Atmosphere::default(), tdb(after_jd))
        .expect("next_occultation")
}

fn previous(before_jd: f64) -> Option<LocalOccultation> {
    EventEngine::new(packaged_backend())
        .previous_occultation(aldebaran(), madrid(), Atmosphere::default(), tdb(before_jd))
        .expect("previous_occultation")
}

fn next_global(after_jd: f64) -> Option<GlobalOccultation> {
    EventEngine::new(packaged_backend())
        .next_global_occultation(aldebaran(), tdb(after_jd))
        .expect("next_global_occultation")
}

fn maximum(occultation: &LocalOccultation) -> f64 {
    occultation.maximum.instant.julian_day.days()
}

/// Four consecutive occultations of Aldebaran seen from Madrid, December
/// 2015 to August 2016, found a day past each maximum so that no search
/// starts near an event. The geocentric conjunction falls before the maximum
/// in some and after it in others.
fn series() -> Vec<LocalOccultation> {
    let mut found = Vec::new();
    let mut after_jd = 2_457_370.0;
    for _ in 0..4 {
        let occultation = next(after_jd).expect("Aldebaran is occulted through 2016");
        after_jd = maximum(&occultation) + 1.0;
        found.push(occultation);
    }
    let days: Vec<f64> = found.iter().map(|o| maximum(o) - 2_457_000.0).collect();
    for (got, want) in days.iter().zip([380.26, 407.66, 516.82, 599.02]) {
        assert!((got - want).abs() < 0.01, "{days:?}");
    }
    found
}

#[test]
fn next_occultation_from_a_returned_maximum_is_the_following_one() {
    let series = series();
    for pair in series.windows(2) {
        assert_eq!(next(maximum(&pair[0])).as_ref(), Some(&pair[1]));
    }
}

#[test]
fn previous_occultation_from_a_returned_maximum_is_the_preceding_one() {
    let series = series();
    for pair in series.windows(2) {
        assert_eq!(previous(maximum(&pair[1])).as_ref(), Some(&pair[0]));
    }
}

#[test]
fn an_instant_near_a_maximum_has_that_occultation_on_one_side() {
    let series = series();
    for (index, occultation) in series.iter().enumerate() {
        let at = maximum(occultation);
        // One representable Julian day either side, then out to an hour:
        // the conjunction is up to an hour or so from the maximum.
        let just_before = f64::from_bits(at.to_bits() - 1);
        let just_after = f64::from_bits(at.to_bits() + 1);
        let before = [
            just_before,
            at - MINUTE,
            at - 10.0 * MINUTE,
            at - 60.0 * MINUTE,
        ];
        let after = [
            just_after,
            at + MINUTE,
            at + 10.0 * MINUTE,
            at + 60.0 * MINUTE,
        ];
        for instant in before {
            assert_eq!(
                next(instant).as_ref(),
                Some(occultation),
                "next, {} s before maximum {index}",
                (at - instant) * 86_400.0
            );
            if index > 0 {
                assert_eq!(previous(instant).as_ref(), Some(&series[index - 1]));
            }
        }
        for instant in after {
            assert_eq!(
                previous(instant).as_ref(),
                Some(occultation),
                "previous, {} s after maximum {index}",
                (instant - at) * 86_400.0
            );
            if let Some(following) = series.get(index + 1) {
                assert_eq!(next(instant).as_ref(), Some(following));
            }
        }
    }
}

#[test]
fn next_global_occultation_selects_by_its_maximum() {
    // Aldebaran was occulted somewhere on Earth every month of 2015.
    let mut series = Vec::new();
    let mut after_jd = 2_457_040.0;
    for _ in 0..3 {
        let occultation = next_global(after_jd).expect("a monthly occultation");
        after_jd = occultation.maximum.julian_day.days() + 1.0;
        series.push(occultation);
    }
    for (index, occultation) in series.iter().enumerate() {
        let at = occultation.maximum.julian_day.days();
        for minutes in [1.0, 30.0, 120.0] {
            assert_eq!(
                next_global(at - minutes * MINUTE).as_ref(),
                Some(occultation),
                "{minutes} min before maximum {index}"
            );
        }
        if let Some(following) = series.get(index + 1) {
            assert_eq!(next_global(at).as_ref(), Some(following));
        }
    }
}
