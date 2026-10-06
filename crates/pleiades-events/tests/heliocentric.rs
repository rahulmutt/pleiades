use pleiades_data::packaged_backend;
use pleiades_events::{CrossingFrame, EventEngine};
use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale};

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

/// Regression: locks the heliocentric frame to the geometric place on the true
/// equinox of date.
///
/// The Swiss Ephemeris reference (`SEFLG_HELCTR | SEFLG_TRUEPOS`, the
/// `helio,Saturn,0,2426000.5` row of the `validate-crossings` corpus) for
/// heliocentric Saturn crossing 0° after 2426000.5 TDB is 2428751.094132220.
/// The engine lands 263 s after it: 0.37" of longitude at Saturn's 0.0335°/day,
/// the difference between the two ephemerides.
///
/// The 400 s tolerance catches both mistakes this frame has had or could have.
/// A J2000 longitude puts the crossing about 14 days early, the precession
/// accumulated since J2000. A light-time-retarded place puts it 4750 s late,
/// which is what the reference itself carried before issue #163: it was
/// generated without `SEFLG_TRUEPOS`, and this test allowed 6000 s for it.
#[test]
fn heliocentric_saturn_of_date_crossing_matches_se() {
    let engine = EventEngine::new(packaged_backend());
    const SE_REF_JD: f64 = 2_428_751.094_132_22;
    let crossing = engine
        .next_longitude_crossing(
            CelestialBody::Saturn,
            Longitude::from_degrees(0.0),
            CrossingFrame::Heliocentric,
            tdb(2_426_000.5),
        )
        .expect("heliocentric Saturn crossing search")
        .expect("expected a heliocentric Saturn crossing of 0°");
    let residual_s = (crossing.instant.julian_day.days() - SE_REF_JD) * 86_400.0;
    assert!(
        residual_s.abs() < 400.0,
        "helio Saturn of-date residual {residual_s:.1} s exceeds 400 s tolerance"
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
