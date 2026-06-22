//! Confirms a civil->TT conversion lands on the same JD a manual TT tag would,
//! so downstream backend lookups see no civil-conversion drift.
use pleiades_time::{tt_from_utc_civil, CivilDateTime};
use pleiades_types::{Instant, JulianDay, TimeScale, SECONDS_PER_DAY};

#[test]
fn civil_to_tt_matches_manual_tt_tag() {
    // 2000-01-01 12:00:00 UTC. TAI-UTC=32 -> TT offset 64.184s.
    let civil = CivilDateTime::new(2000, 1, 1, 12, 0, 0.0);
    let converted = tt_from_utc_civil(civil).unwrap();
    let utc_jd = civil.to_julian_day().unwrap().days();
    let manual = Instant::new(
        JulianDay::from_days(utc_jd + 64.184 / SECONDS_PER_DAY),
        TimeScale::Tt,
    );
    let drift_s =
        (converted.instant.julian_day.days() - manual.julian_day.days()).abs() * SECONDS_PER_DAY;
    assert!(drift_s < 1e-3, "civil->TT drift {drift_s}s exceeds 1 ms");
}
