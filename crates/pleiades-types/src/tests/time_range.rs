use crate::*;

#[test]
fn time_range_checks_scale_and_julian_day() {
    let start = Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt);
    let end = Instant::new(JulianDay::from_days(2451546.0), TimeScale::Tt);
    let range = TimeRange::new(Some(start), Some(end));

    assert!(range.contains(Instant::new(JulianDay::from_days(2451545.5), TimeScale::Tt)));
    assert!(!range.contains(Instant::new(
        JulianDay::from_days(2451545.5),
        TimeScale::Utc
    )));
    assert_eq!(
        range.summary_line(),
        "JD 2451545.0 (TT) → JD 2451546.0 (TT)"
    );
    assert_eq!(range.to_string(), range.summary_line());
    assert!(range.validate().is_ok());
    assert_eq!(TimeRange::new(Some(start), None).validate(), Ok(()));
    assert_eq!(
        TimeRange::new(Some(start), None).to_string(),
        "from JD 2451545.0 (TT)"
    );
    assert_eq!(
        TimeRange::new(None, Some(end)).to_string(),
        "through JD 2451546.0 (TT)"
    );
    assert_eq!(TimeRange::new(None, None).to_string(), "unbounded");
}

#[test]
fn time_range_validation_rejects_non_finite_bounds_and_invalid_order() {
    let finite_start = Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt);
    let finite_end = Instant::new(JulianDay::from_days(2451546.0), TimeScale::Tt);

    let error = TimeRange::new(
        Some(Instant::new(
            JulianDay::from_days(f64::INFINITY),
            TimeScale::Tt,
        )),
        Some(finite_end),
    )
    .validate()
    .expect_err("non-finite start bounds should fail validation");
    assert_eq!(
        error.summary_line(),
        "time range bound `start` must be finite: JD inf (TT)"
    );
    assert_eq!(error.to_string(), error.summary_line());

    let error = TimeRange::new(
        Some(finite_start),
        Some(Instant::new(
            JulianDay::from_days(f64::NEG_INFINITY),
            TimeScale::Tt,
        )),
    )
    .validate()
    .expect_err("non-finite end bounds should fail validation");
    assert_eq!(
        error.summary_line(),
        "time range bound `end` must be finite: JD -inf (TT)"
    );
    assert_eq!(error.to_string(), error.summary_line());

    let error = TimeRange::new(
        Some(Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt)),
        Some(Instant::new(
            JulianDay::from_days(2451546.0),
            TimeScale::Tdb,
        )),
    )
    .validate()
    .expect_err("mixed time-scale bounds should fail validation");
    assert_eq!(
            error.summary_line(),
            "time range bounds must use the same time scale: start=JD 2451545.0 (TT); end=JD 2451546.0 (TDB)"
        );
    assert_eq!(error.to_string(), error.summary_line());

    let error = TimeRange::new(Some(finite_end), Some(finite_start))
        .validate()
        .expect_err("out-of-order ranges should fail validation");
    assert_eq!(
        error.summary_line(),
        "time range end must not precede the start: start=JD 2451546.0 (TT); end=JD 2451545.0 (TT)"
    );
    assert_eq!(error.to_string(), error.summary_line());
}

#[test]
fn time_range_contains_and_validate_respect_boundaries() {
    let start = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let end = Instant::new(JulianDay::from_days(2_451_546.0), TimeScale::Tt);
    let range = TimeRange::new(Some(start), Some(end));

    // Same scale, but outside each bound: the && form excludes these; the ||
    // mutant would wrongly include a value that fails exactly one clause.
    let before = Instant::new(JulianDay::from_days(2_451_544.5), TimeScale::Tt);
    let after = Instant::new(JulianDay::from_days(2_451_546.5), TimeScale::Tt);
    assert!(
        !range.contains(before),
        "instant before start must not be contained"
    );
    assert!(
        !range.contains(after),
        "instant after end must not be contained"
    );
    // A same-scale in-range instant IS contained (guards the 37 && overall).
    assert!(range.contains(Instant::new(
        JulianDay::from_days(2_451_545.5),
        TimeScale::Tt
    )));

    // Degenerate range start == end must validate Ok; the >= mutant flags it
    // as out-of-order.
    let point = TimeRange::new(Some(start), Some(start));
    assert_eq!(point.validate(), Ok(()));
}
