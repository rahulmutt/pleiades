use pleiades_data::packaged_backend;
use pleiades_events::{CrossingFrame, EventEngine};
use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale};

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

/// Regression: locks the heliocentric of-date fix (precession + nutation).
///
/// SE reference (`SEFLG_HELCTR`, of-date true equinox) for heliocentric Saturn
/// crossing 0° after start 2430000.5 TDB is `crossing_jd = 2439500.066527`.
///
/// Before the fix, the engine returned a J2000 longitude and the crossing landed
/// ~14 days (~1.2M s) early — a pure precession signature growing with distance
/// from J2000. After precession + nutation the residual collapses to ~4740 s
/// (~79 min), which corresponds to only ~6.6" of heliocentric longitude at
/// Saturn's ~0.0335°/day rate. That residual is the geocentric-light-time
/// signature of reconstructing `P_helio = P_geo − S_geo` from the backend's
/// astrometric (light-time-corrected) geocentric vectors; SE's geometric helio
/// place does not carry it, and the task scopes light-time out of this fix.
///
/// The tolerance (6000 s) is set to what the fixed code actually achieves with
/// margin above the documented ~4740 s light-time floor, while being ~200x
/// tighter than the pre-fix error — so any regression of the of-date rotation
/// (dropping precession or nutation) fails this test immediately.
#[test]
fn heliocentric_saturn_of_date_crossing_matches_se() {
    let engine = EventEngine::new(packaged_backend());
    const SE_REF_JD: f64 = 2_439_500.066527;
    let crossing = engine
        .next_longitude_crossing(
            CelestialBody::Saturn,
            Longitude::from_degrees(0.0),
            CrossingFrame::Heliocentric,
            tdb(2_430_000.5),
        )
        .expect("heliocentric Saturn crossing search")
        .expect("expected a heliocentric Saturn crossing of 0°");
    let residual_s = (crossing.instant.julian_day.days() - SE_REF_JD) * 86_400.0;
    assert!(
        residual_s.abs() < 6_000.0,
        "helio Saturn of-date residual {residual_s:.1} s exceeds 6000 s tolerance"
    );
}

/// `next_longitude_crossing` (early-terminating) must return the SAME crossing as
/// `longitude_crossings_in_range(..).first()` filtered strictly after `after`.
#[test]
fn next_equals_first_in_range_heliocentric() {
    let engine = EventEngine::new(packaged_backend());
    let after = tdb(2_451_545.0);
    let end = tdb(2_451_545.0 + 4400.0);
    let next = engine
        .next_longitude_crossing(
            CelestialBody::Jupiter,
            Longitude::from_degrees(0.0),
            CrossingFrame::Heliocentric,
            after,
        )
        .unwrap()
        .unwrap();
    let in_range = engine
        .longitude_crossings_in_range(
            CelestialBody::Jupiter,
            Longitude::from_degrees(0.0),
            CrossingFrame::Heliocentric,
            after,
            end,
        )
        .unwrap();
    let first = in_range
        .iter()
        .find(|c| c.instant.julian_day.days() > after.julian_day.days())
        .expect("range has a crossing after `after`");
    assert!(
        (next.instant.julian_day.days() - first.instant.julian_day.days()).abs() < 1e-9,
        "next {} != first-in-range {}",
        next.instant.julian_day.days(),
        first.instant.julian_day.days()
    );
}

/// A Moon `next_longitude_crossing` (0.25-day step) must return quickly now that
/// the finder terminates on the first root instead of scanning to WINDOW_END.
#[test]
fn moon_next_crossing_returns_quickly() {
    let engine = EventEngine::new(packaged_backend());
    let after = tdb(2_451_545.0);
    let t0 = std::time::Instant::now();
    let crossing = engine
        .next_longitude_crossing(
            CelestialBody::Moon,
            Longitude::from_degrees(0.0),
            CrossingFrame::GeocentricApparentOfDate,
            after,
        )
        .unwrap()
        .expect("Moon crosses 0° within a month");
    let elapsed = t0.elapsed();
    // The first crossing is within ~1 month; the early-return means only a few
    // hundred backend evals, not ~288k. Generous ceiling to avoid CI flakiness.
    assert!(
        elapsed.as_secs_f64() < 5.0,
        "Moon next-crossing took {elapsed:?}"
    );
    assert!(crossing.instant.julian_day.days() > after.julian_day.days());
}

#[test]
fn heliocentric_jupiter_crossing_is_found() {
    let engine = EventEngine::new(packaged_backend());
    let start = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let end = Instant::new(JulianDay::from_days(2_451_545.0 + 4400.0), TimeScale::Tdb); // ~1 Jupiter orbit
    let out = engine
        .longitude_crossings_in_range(
            CelestialBody::Jupiter,
            Longitude::from_degrees(0.0),
            CrossingFrame::Heliocentric,
            start,
            end,
        )
        .expect("heliocentric crossing search");
    assert!(
        !out.is_empty(),
        "expected a heliocentric Jupiter crossing of 0°"
    );
}
