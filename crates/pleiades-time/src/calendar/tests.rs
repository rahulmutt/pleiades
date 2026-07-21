use super::*;

#[test]
fn j2000_noon_is_jd_2451545() {
    let jd = CivilDateTime::new(2000, 1, 1, 12, 0, 0.0)
        .to_julian_day()
        .unwrap();
    assert!((jd.days() - 2451545.0).abs() < 1e-6, "got {}", jd.days());
}

#[test]
fn epoch_anchors_match_known_jds() {
    let cases = [
        (1900, 1, 1, 2415020.5),
        (1972, 1, 1, 2441317.5),
        (2100, 1, 1, 2488069.5),
    ];
    for (y, m, d, expected) in cases {
        let jd = CivilDateTime::new(y, m, d, 0, 0, 0.0)
            .to_julian_day()
            .unwrap();
        assert!(
            (jd.days() - expected).abs() < 1e-6,
            "{y}-{m}-{d}: got {}",
            jd.days()
        );
    }
}

#[test]
fn round_trips_within_a_millisecond() {
    let original = CivilDateTime::new(1987, 4, 10, 19, 21, 0.0);
    let jd = original.to_julian_day().unwrap();
    let back = CivilDateTime::from_julian_day(jd);
    assert_eq!(back.year, 1987);
    assert_eq!(back.month, 4);
    assert_eq!(back.day, 10);
    assert_eq!(back.hour, 19);
    assert_eq!(back.minute, 21);
    assert!(back.second < 0.001, "second: {}", back.second);
}

#[test]
fn round_trips_nonzero_seconds_without_minute_corruption() {
    let original = CivilDateTime::new(1987, 4, 10, 19, 20, 30.0);
    let jd = original.to_julian_day().unwrap();
    let back = CivilDateTime::from_julian_day(jd);
    assert_eq!(back.hour, 19);
    assert_eq!(back.minute, 20);
    assert!(
        (back.second - 30.0).abs() < 0.001,
        "second drifted: {}",
        back.second
    );
}

#[test]
fn rejects_bad_fields() {
    assert_eq!(
        CivilDateTime::new(2000, 13, 1, 0, 0, 0.0).to_julian_day(),
        Err(CivilTimeError::InvalidCivilDate { field: "month" })
    );
    assert_eq!(
        CivilDateTime::new(2000, 1, 1, 0, 0, f64::NAN).to_julian_day(),
        Err(CivilTimeError::InvalidCivilDate { field: "second" })
    );
    assert_eq!(
        CivilDateTime::new(2000, 1, 0, 0, 0, 0.0).to_julian_day(),
        Err(CivilTimeError::InvalidCivilDate { field: "day" })
    );
    assert_eq!(
        CivilDateTime::new(2000, 1, 1, 24, 0, 0.0).to_julian_day(),
        Err(CivilTimeError::InvalidCivilDate { field: "hour" })
    );
    assert_eq!(
        CivilDateTime::new(2000, 1, 1, 0, 60, 0.0).to_julian_day(),
        Err(CivilTimeError::InvalidCivilDate { field: "minute" })
    );
}

#[test]
fn near_midnight_does_not_overflow_hour() {
    let jd = CivilDateTime::new(2000, 1, 1, 23, 59, 59.9996)
        .to_julian_day()
        .unwrap();
    let back = CivilDateTime::from_julian_day(jd);
    assert!(back.hour <= 23, "hour overflowed: {}", back.hour);
}

#[test]
fn rejects_negative_and_oversized_seconds() {
    // The second field's contract is [0.0, 61.0): negative and >= 61 are
    // invalid; [60, 61) is deliberately accepted (leap seconds).
    assert_eq!(
        CivilDateTime::new(2000, 1, 1, 0, 0, -1.0).to_julian_day(),
        Err(CivilTimeError::InvalidCivilDate { field: "second" })
    );
    assert_eq!(
        CivilDateTime::new(2000, 1, 1, 0, 0, 61.0).to_julian_day(),
        Err(CivilTimeError::InvalidCivilDate { field: "second" })
    );
    assert!(CivilDateTime::new(2016, 12, 31, 23, 59, 60.5)
        .to_julian_day()
        .is_ok());
}

#[test]
fn from_julian_day_reconstructs_2100_new_year() {
    // JD 2488069.5 = 2100-01-01 00:00:00 (proleptic Gregorian; the known
    // epoch anchor already pinned in epoch_anchors_match_known_jds).
    // Discriminating properties (spec §4.4): alpha = 16, where
    // floor(alpha/4) = 4 differs from alpha % 4 = 0, and January, where
    // e = 14 exercises the e-13 month branch and the e < 14 boundary.
    let back = CivilDateTime::from_julian_day(JulianDay::from_days(2_488_069.5));
    assert_eq!(back.year, 2100);
    assert_eq!(back.month, 1);
    assert_eq!(back.day, 1);
    assert_eq!(back.hour, 0);
    assert_eq!(back.minute, 0);
    assert!(back.second.abs() < 0.001, "second: {}", back.second);
}

#[test]
fn from_julian_day_reconstructs_gregorian_leap_day() {
    // JD 2451604.0 = 2000-02-29 12:00:00 — February is the only month
    // class where e = 15 (separating e - 13 = 2 from e / 13 = 1) and
    // month == 2.0 (the only value where > vs >= differ in the year
    // branch), on the Gregorian 400-year-exception leap day.
    let back = CivilDateTime::from_julian_day(JulianDay::from_days(2_451_604.0));
    assert_eq!(back.year, 2000);
    assert_eq!(back.month, 2);
    assert_eq!(back.day, 29);
    assert_eq!(back.hour, 12);
    assert_eq!(back.minute, 0);
    assert!(back.second.abs() < 0.001, "second: {}", back.second);
}
