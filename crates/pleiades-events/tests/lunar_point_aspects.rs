//! Aspects of the lunar points on the packaged backend that the Swiss
//! Ephemeris aspect corpora cannot pin on their own (issue #168).
//!
//! - (a) The one corpus event pair a plain sign-change scan would miss: the
//!   osculating apogee's 1.84″ graze of the Jupiter sextile, whose two exact
//!   moments share one 0.25-day step. The gate compares it only by parity,
//!   inside its graze window, so this test pins it.
//! - (f) The perigees are the apogees' antipodes, so their aspects at an angle
//!   are the apogees' at 180° less that angle, and the gated apogee corpus
//!   covers them.

use pleiades_data::packaged_backend;
use pleiades_events::{AspectEvent, CrossingFrame, EventEngine};
use pleiades_types::{Angle, CelestialBody, Instant, JulianDay, TimeScale};

const GEO: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const MEAN: CrossingFrame = CrossingFrame::GeocentricMeanOfDate;
/// The aspect angles the gate compares; the set is its own image under
/// `θ → 180° − θ`.
const ANGLES_DEG: [f64; 5] = [0.0, 60.0, 90.0, 120.0, 180.0];
/// Two seconds, in days: four times the bisection tolerance.
const TWO_SECONDS: f64 = 2.0 / 86_400.0;
/// The lunar points' scan step (`pleiades_events` steps lunar points by 0.25 day).
const LUNAR_STEP_DAYS: f64 = 0.25;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn wrap180(deg: f64) -> f64 {
    (deg + 180.0).rem_euclid(360.0) - 180.0
}

fn jd(event: &AspectEvent) -> f64 {
    event.instant.julian_day.days()
}

#[test]
fn the_apogee_jupiter_sextile_graze_holds_two_events_in_one_step() {
    // Swiss Ephemeris SWIEPH puts the pair at JD 2420750.7899 and 2420750.8545
    // (lunar-points.csv), the separation turning 1.84" short of +60 degrees
    // between them. From a range starting at 2420750.5 the step grid is
    // .5, .75, 1.0, so both fall in the step [.75, 1.0].
    let engine = EventEngine::new(packaged_backend());
    let start = 2_420_750.5;
    let events = engine
        .aspects_in_range(
            CelestialBody::TrueApogee,
            CelestialBody::Jupiter,
            Angle::from_degrees(60.0),
            GEO,
            tdb(start),
            tdb(start + 1.0),
        )
        .expect("aspects");
    assert_eq!(events.len(), 2, "{events:?}");
    let step_of = |jd: f64| ((jd - start) / LUNAR_STEP_DAYS).floor();
    assert_eq!(step_of(jd(&events[0])), 1.0);
    assert_eq!(step_of(jd(&events[1])), 1.0);
    for (event, se_jd) in events
        .iter()
        .zip([2_420_750.789_911_8, 2_420_750.854_536_2])
    {
        let separation =
            wrap180(event.first_longitude.degrees() - event.second_longitude.degrees());
        assert!((separation - 60.0).abs() < 1e-4, "{event:?}");
        // Near a turn the instant is ill-conditioned; a 0.01-day bound still
        // separates the two events (0.065 day apart).
        assert!((jd(event) - se_jd).abs() < 0.01, "{event:?} vs SE {se_jd}");
    }
    // The step's ends lie on one side of the level, so a scan that only looked
    // for a sign change between grid samples would find neither event.
    let offset = |jd: f64| {
        let lon = |body| {
            engine
                .longitude_at(body, GEO, tdb(jd))
                .expect("longitude")
                .degrees()
        };
        wrap180(lon(CelestialBody::TrueApogee) - lon(CelestialBody::Jupiter)) - 60.0
    };
    let (at_lo, at_hi) = (offset(start + 0.75), offset(start + 1.0));
    assert_eq!(at_lo > 0.0, at_hi > 0.0, "{at_lo} {at_hi}");
}

/// The (perigee, apogee) pairs of the packaged backend.
const APSIDES: [(CelestialBody, CelestialBody); 2] = [
    (CelestialBody::TruePerigee, CelestialBody::TrueApogee),
    (CelestialBody::MeanPerigee, CelestialBody::MeanApogee),
];

#[test]
fn each_perigee_is_its_apogee_plus_180_degrees_all_window() {
    let engine = EventEngine::new(packaged_backend());
    // Every 500 days over 1900-2100, in both geocentric frames.
    for k in 0..=146 {
        let jd = 2_415_025.5 + 500.0 * f64::from(k);
        for frame in [GEO, MEAN] {
            for (perigee, apogee) in &APSIDES {
                let lon = |body: &CelestialBody| {
                    engine
                        .longitude_at(body.clone(), frame, tdb(jd))
                        .expect("longitude")
                        .degrees()
                };
                let off = wrap180(lon(perigee) - lon(apogee) - 180.0).abs();
                assert!(off < 1e-9, "{perigee:?} at jd {jd}: {off} deg off");
            }
        }
    }
}

#[test]
fn perigee_aspects_are_apogee_aspects_at_the_supplementary_angle() {
    let engine = EventEngine::new(packaged_backend());
    let angles = ANGLES_DEG.map(Angle::from_degrees);
    let (start, end) = (2_451_545.0, 2_451_545.0 + 365.25);
    let events = |first: &CelestialBody, second: CelestialBody| {
        engine
            .aspects_in_range_for_angles(first.clone(), second, &angles, GEO, tdb(start), tdb(end))
            .expect("aspects")
    };
    for ((perigee, apogee), other) in APSIDES
        .iter()
        .zip([CelestialBody::Jupiter, CelestialBody::Sun])
    {
        let of_perigee = events(perigee, other.clone());
        let of_apogee = events(apogee, other.clone());
        assert!(!of_perigee.is_empty(), "{perigee:?}");
        for angle in ANGLES_DEG {
            let at = |list: &[AspectEvent], deg: f64| -> Vec<f64> {
                list.iter()
                    .filter(|e| e.angle.degrees() == deg)
                    .map(jd)
                    .collect()
            };
            let got = at(&of_perigee, angle);
            let want = at(&of_apogee, 180.0 - angle);
            assert_eq!(
                got.len(),
                want.len(),
                "{perigee:?}-{other:?} at {angle} vs {apogee:?} at {}",
                180.0 - angle
            );
            for (g, w) in got.iter().zip(&want) {
                assert!(
                    (g - w).abs() < TWO_SECONDS,
                    "{perigee:?} at {angle}: {g} vs {w}"
                );
            }
        }
    }
}
