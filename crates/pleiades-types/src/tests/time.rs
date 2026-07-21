use crate::*;
use core::time::Duration;

#[test]
fn instant_mean_obliquity_matches_the_shared_cubic_approximation() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);

    assert_eq!(instant.mean_obliquity().degrees(), 23.439_291_111_111_11);
}

#[test]
fn instant_has_a_compact_display() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);

    assert_eq!(instant.summary_line(), "JD 2451545 TDB");
    assert_eq!(instant.to_string(), "JD 2451545 TDB");
}

#[test]
fn time_scales_have_stable_display_names() {
    assert_eq!(TimeScale::Utc.to_string(), "UTC");
    assert_eq!(TimeScale::Ut1.to_string(), "UT1");
    assert_eq!(TimeScale::Tt.to_string(), "TT");
    assert_eq!(TimeScale::Tdb.to_string(), "TDB");
}

#[test]
fn time_scale_conversion_errors_use_stable_display_labels() {
    let error = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt)
        .tt_from_ut1(Duration::from_secs(1))
        .expect_err("TT is not UT1");

    assert_eq!(
        error.to_string(),
        "time-scale conversion expected UT1, got TT"
    );
}

#[test]
fn caller_supplied_time_scale_offsets_shift_julian_days() {
    let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
    let tt = ut1
        .tt_from_ut1(Duration::from_secs_f64(64.184))
        .expect("UT1 to TT conversion should accept UT1 input");

    assert_eq!(tt.scale, TimeScale::Tt);
    assert!((tt.julian_day.days() - 2_451_545.000_742_870_4).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_utc_to_tt() {
    let utc = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
    let tt = utc
        .tt_from_utc(Duration::from_secs_f64(64.184))
        .expect("UTC to TT conversion should accept UTC input");

    assert_eq!(tt.scale, TimeScale::Tt);
    assert!((tt.julian_day.days() - 2_451_545.000_742_870_4).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_utc_to_tt_with_signed_offset() {
    let utc = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
    let tt = utc
        .tt_from_utc_signed(64.184)
        .expect("UTC to TT conversion should accept signed UTC input");

    assert_eq!(tt.scale, TimeScale::Tt);
    assert!((tt.julian_day.days() - 2_451_545.000_742_870_4).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_tt_to_tdb() {
    let tt = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let tdb = tt
        .tdb_from_tt(Duration::from_secs_f64(0.001_657))
        .expect("TT to TDB conversion should accept TT input");

    assert_eq!(tdb.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + 0.001_657 / 86_400.0;
    assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_tt_to_tdb_with_signed_offset() {
    let tt = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let tdb = tt
        .tdb_from_tt_signed(-0.001_657)
        .expect("TT to TDB conversion should accept signed TT input");

    assert_eq!(tdb.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
    assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_tdb_to_tt() {
    let tdb = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let tt = tdb
        .tt_from_tdb(-0.001_657)
        .expect("TDB to TT conversion should accept TDB input");

    assert_eq!(tt.scale, TimeScale::Tt);
    let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
    assert!((tt.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_tdb_to_tt_with_signed_offset() {
    let tdb = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let tt = tdb
        .tt_from_tdb_signed(-0.001_657)
        .expect("TDB to TT conversion should accept signed TDB input");

    assert_eq!(tt.scale, TimeScale::Tt);
    let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
    assert!((tt.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_utc_to_tdb() {
    let utc = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
    let tdb = utc
        .tdb_from_utc(
            Duration::from_secs_f64(64.184),
            Duration::from_secs_f64(0.001_657),
        )
        .expect("UTC to TDB conversion should accept UTC input");

    assert_eq!(tdb.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + (64.184 + 0.001_657) / 86_400.0;
    assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_utc_to_tdb_with_signed_offset() {
    let utc = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
    let tdb = utc
        .tdb_from_utc_signed(Duration::from_secs_f64(64.184), -0.001_657)
        .expect("UTC to TDB conversion should accept signed UTC input");

    assert_eq!(tdb.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + (64.184 - 0.001_657) / 86_400.0;
    assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_ut1_to_tdb() {
    let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
    let tdb = ut1
        .tdb_from_ut1(
            Duration::from_secs_f64(64.184),
            Duration::from_secs_f64(0.001_657),
        )
        .expect("UT1 to TDB conversion should accept UT1 input");

    assert_eq!(tdb.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + (64.184 + 0.001_657) / 86_400.0;
    assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_ut1_to_tdb_with_signed_offset() {
    let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
    let tdb = ut1
        .tdb_from_ut1_signed(Duration::from_secs_f64(64.184), -0.001_657)
        .expect("UT1 to TDB conversion should accept signed UT1 input");

    assert_eq!(tdb.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + (64.184 - 0.001_657) / 86_400.0;
    assert!((tdb.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn caller_supplied_time_scale_offsets_can_convert_ut1_to_tt_with_signed_offset() {
    let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
    let tt = ut1
        .tt_from_ut1_signed(64.184)
        .expect("UT1 to TT conversion should accept signed UT1 input");

    assert_eq!(tt.scale, TimeScale::Tt);
    let expected = 2_451_545.0 + 64.184 / 86_400.0;
    assert!((tt.julian_day.days() - expected).abs() < 1e-12);
}

#[test]
fn time_scale_helpers_reject_the_wrong_source_scale() {
    let utc = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc);
    let ut1_error = utc
        .tt_from_ut1(Duration::from_secs(64))
        .expect_err("UTC is not UT1");

    assert!(matches!(
        ut1_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Ut1,
            actual: TimeScale::Utc,
        }
    ));

    let tdb_ut1_error = utc
        .tdb_from_ut1(Duration::from_secs(64), Duration::from_secs(1))
        .expect_err("UTC is not UT1 for UT1-to-TDB conversion");

    assert!(matches!(
        tdb_ut1_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Ut1,
            actual: TimeScale::Utc,
        }
    ));

    let tt = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let utc_error = tt
        .tt_from_utc(Duration::from_secs(64))
        .expect_err("TT is not UTC");

    assert!(matches!(
        utc_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Utc,
            actual: TimeScale::Tt,
        }
    ));

    let utc_signed_error = tt
        .tt_from_utc_signed(64.0)
        .expect_err("TT is not UTC for signed UTC-to-TT conversion");

    assert!(matches!(
        utc_signed_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Utc,
            actual: TimeScale::Tt,
        }
    ));

    let tdb_error = tt
        .tdb_from_utc(Duration::from_secs(64), Duration::from_secs(1))
        .expect_err("TT is not UTC for UTC-to-TDB conversion");

    assert!(matches!(
        tdb_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Utc,
            actual: TimeScale::Tt,
        }
    ));

    let tt_error = utc
        .tt_from_tdb(-0.001_657)
        .expect_err("UTC is not TDB for TDB-to-TT conversion");

    assert!(matches!(
        tt_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Tdb,
            actual: TimeScale::Utc,
        }
    ));

    let ut1_signed_error = utc
        .tt_from_ut1_signed(64.0)
        .expect_err("UTC is not UT1 for signed UT1-to-TT conversion");

    assert!(matches!(
        ut1_signed_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Ut1,
            actual: TimeScale::Utc,
        }
    ));

    let wrong_scale_error = tt
        .tt_from_tdb(-0.001_657)
        .expect_err("TT is not TDB for TDB-to-TT conversion");

    assert!(matches!(
        wrong_scale_error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Tdb,
            actual: TimeScale::Tt,
        }
    ));
}

#[test]
fn signed_time_scale_helpers_reject_non_finite_offsets() {
    let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);
    let tt_nan_error = ut1
        .tt_from_ut1_signed(f64::NAN)
        .expect_err("non-finite UT1 offsets should be rejected");

    assert!(matches!(
        tt_nan_error,
        TimeScaleConversionError::NonFiniteOffset
    ));

    let tt = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let tdb_inf_error = tt
        .tdb_from_tt_signed(f64::INFINITY)
        .expect_err("non-finite TDB offsets should be rejected");

    assert!(matches!(
        tdb_inf_error,
        TimeScaleConversionError::NonFiniteOffset
    ));

    let tdb = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let tt_negative_inf_error = tdb
        .tt_from_tdb(f64::NEG_INFINITY)
        .expect_err("non-finite TDB-to-TT offsets should be rejected");

    assert!(matches!(
        tt_negative_inf_error,
        TimeScaleConversionError::NonFiniteOffset
    ));
}

#[test]
fn time_scale_conversion_errors_render_stable_summary_lines() {
    let expected = TimeScaleConversionError::Expected {
        expected: TimeScale::Tt,
        actual: TimeScale::Utc,
    };
    assert_eq!(
        expected.summary_line(),
        "time-scale conversion expected TT, got UTC"
    );
    assert_eq!(expected.to_string(), expected.summary_line());

    let non_finite = TimeScaleConversionError::NonFiniteOffset;
    assert_eq!(
        non_finite.summary_line(),
        "time-scale conversion offset must be finite"
    );
    assert_eq!(non_finite.to_string(), non_finite.summary_line());
}

#[test]
fn time_scale_conversion_policy_can_validate_and_apply_a_caller_supplied_rule() {
    let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tt, 64.184);
    let ut1 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Ut1);

    assert!(policy.validate(ut1).is_ok());
    assert!(ut1.validate_time_scale_conversion(policy).is_ok());

    let converted = policy
        .apply(ut1)
        .expect("caller-supplied policy should convert the source instant");

    assert_eq!(converted.scale, TimeScale::Tt);
    assert!((converted.julian_day.days() - 2_451_545.000_742_870_4).abs() < 1e-12);
    assert_eq!(
        policy.summary_line(),
        "source=UT1; target=TT; offset_seconds=64.184 s"
    );
    assert_eq!(policy.to_string(), policy.summary_line());
}

#[test]
fn time_scale_conversion_policy_renders_signed_offsets_in_summary_lines() {
    let policy = TimeScaleConversion::new(TimeScale::Tdb, TimeScale::Tt, -0.001_657);

    assert_eq!(
        policy.summary_line(),
        "source=TDB; target=TT; offset_seconds=-0.001657 s"
    );
    assert_eq!(policy.to_string(), policy.summary_line());
}

#[test]
fn time_scale_conversion_policy_validated_summary_line_matches_the_plain_rendering() {
    let policy = TimeScaleConversion::new(TimeScale::Tdb, TimeScale::Tt, -0.001_657);
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);

    assert_eq!(
        policy
            .validated_summary_line(instant)
            .expect("policy should validate"),
        policy.summary_line()
    );
}

#[test]
fn time_scale_conversion_policy_accepts_signed_tdb_to_tt_validation() {
    let policy = TimeScaleConversion::new(TimeScale::Tdb, TimeScale::Tt, -0.001_657);
    let tdb = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);

    assert!(policy.validate(tdb).is_ok());
    assert!(tdb.validate_time_scale_conversion(policy).is_ok());

    let converted = policy
        .apply(tdb)
        .expect("signed TDB-to-TT policy should convert the source instant");

    assert_eq!(converted.scale, TimeScale::Tt);
    assert!(
        (converted.julian_day.days() - 2_451_544.999_999_981).abs() < 1e-12,
        "signed TDB-to-TT conversion should apply the caller-supplied offset"
    );
}

#[test]
fn instant_validate_time_scale_conversion_rejects_mismatched_scales_and_non_finite_offsets() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tt, 64.184);

    let error = instant
        .validate_time_scale_conversion(policy)
        .expect_err("policy should reject the wrong source scale");

    assert!(matches!(
        error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Ut1,
            actual: TimeScale::Tt,
        }
    ));

    let non_finite = TimeScaleConversion::new(TimeScale::Tt, TimeScale::Tdb, f64::NAN);
    let error = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt)
        .validate_time_scale_conversion(non_finite)
        .expect_err("policy should reject non-finite offsets");

    assert!(matches!(error, TimeScaleConversionError::NonFiniteOffset));
}

#[test]
fn time_scale_conversion_policy_rejects_mismatched_scales_and_non_finite_offsets() {
    let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tt, 64.184);
    let tt = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let error = policy
        .validate(tt)
        .expect_err("policy should reject the wrong source scale");

    assert!(matches!(
        error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Ut1,
            actual: TimeScale::Tt,
        }
    ));

    let non_finite = TimeScaleConversion::new(TimeScale::Tt, TimeScale::Tdb, f64::NAN);
    let error = non_finite
        .validate(Instant::new(
            JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ))
        .expect_err("policy should reject non-finite offsets");

    assert!(matches!(error, TimeScaleConversionError::NonFiniteOffset));
    assert!(matches!(
        non_finite.apply(Instant::new(
            JulianDay::from_days(2_451_545.0),
            TimeScale::Tt
        )),
        Err(TimeScaleConversionError::NonFiniteOffset)
    ));
}

#[test]
fn instant_with_time_scale_offset_checked_rejects_non_finite_offsets() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let error = instant
        .with_time_scale_offset_checked(TimeScale::Tdb, f64::NEG_INFINITY)
        .expect_err("checked offset conversion should reject non-finite offsets");

    assert!(matches!(error, TimeScaleConversionError::NonFiniteOffset));

    let converted = instant
        .with_time_scale_offset_checked(TimeScale::Tdb, 0.001_657)
        .expect("checked offset conversion should accept finite offsets");

    assert_eq!(converted.scale, TimeScale::Tdb);
    assert!((converted.julian_day.days() - 2_451_545.000_000_019).abs() < 1e-12);
}
