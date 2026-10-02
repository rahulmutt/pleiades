use super::*;

use super::test_support::{at, insertions};

#[test]
fn utc_modern_is_exact() {
    // 2017-01-01 00:00 UTC -> TT. Offset = 37 + 32.184 = 69.184 s.
    let civil = CivilDateTime::new(2017, 1, 1, 0, 0, 0.0);
    let out = tt_from_utc_civil(civil).unwrap();
    assert_eq!(out.instant.scale, TimeScale::Tt);
    assert_eq!(out.provenance.quality, ConversionQuality::Exact);
    assert_eq!(out.provenance.tai_minus_utc, Some(37));
    let expected_jd = civil.to_julian_day().unwrap().days() + 69.184 / SECONDS_PER_DAY;
    assert!((out.instant.julian_day.days() - expected_jd).abs() < 1e-9);
}

#[test]
fn ut1_historical_is_observed() {
    let civil = CivilDateTime::new(1950, 1, 1, 0, 0, 0.0);
    let out = tt_from_ut1_civil(civil).unwrap();
    assert_eq!(out.provenance.quality, ConversionQuality::Observed);
    assert!(out.provenance.delta_t_seconds.unwrap() > 28.0);
}

#[test]
fn ut1_after_observed_table_is_leap_bound_and_reported_observed() {
    // 2022-01-01: past the 2020 observed node, inside the leap table. ΔT is
    // the leap-second bound 32.184 + 37 = 69.184 s, reported through the
    // existing three-tier vocabulary as Observed on the Ut1DeltaT path.
    let civil = CivilDateTime::new(2022, 1, 1, 0, 0, 0.0);
    let out = tt_from_ut1_civil(civil).unwrap();
    assert_eq!(out.provenance.quality, ConversionQuality::Observed);
    assert_eq!(out.provenance.path, ConversionPath::Ut1DeltaT);
    assert!((out.provenance.delta_t_seconds.unwrap() - 69.184).abs() < 1e-12);
}

#[test]
fn ut1_and_utc_agree_where_delta_t_is_leap_bound() {
    // The leap-second bound takes DUT1 = 0, so a civil day tagged UT1 and the
    // same day tagged UTC must land on the same TT instant.
    let civil = CivilDateTime::new(2022, 1, 1, 0, 0, 0.0);
    let from_ut1 = tt_from_ut1_civil(civil).unwrap();
    let from_utc = tt_from_utc_civil(civil).unwrap();
    assert_eq!(
        from_ut1.instant.julian_day.days(),
        from_utc.instant.julian_day.days()
    );
}

#[test]
fn future_utc_is_predicted() {
    let civil = CivilDateTime::new(2090, 6, 1, 0, 0, 0.0);
    let out = tt_from_utc_civil(civil).unwrap();
    assert_eq!(out.provenance.quality, ConversionQuality::Predicted);
    assert_eq!(out.provenance.path, ConversionPath::FutureExtrapolated);
}

#[test]
fn pre_1972_utc_is_rejected() {
    let civil = CivilDateTime::new(1965, 1, 1, 0, 0, 0.0);
    assert_eq!(
        tt_from_utc_civil(civil),
        Err(CivilTimeError::UtcBeforeLeapEpoch)
    );
}

#[test]
fn outside_window_is_rejected() {
    let civil = CivilDateTime::new(1880, 1, 1, 0, 0, 0.0);
    assert!(matches!(
        tt_from_ut1_civil(civil),
        Err(CivilTimeError::BeyondHorizon { .. })
    ));
}

#[test]
fn bad_target_scale_is_rejected() {
    let civil = CivilDateTime::new(2000, 1, 1, 0, 0, 0.0);
    assert_eq!(
        to_terrestrial(civil, TimeScale::Utc, TimeScale::Ut1),
        Err(CivilTimeError::UnsupportedScale {
            source: TimeScale::Utc,
            target: TimeScale::Ut1
        })
    );
}

#[test]
fn end_of_2100_is_accepted() {
    let civil = CivilDateTime::new(2100, 12, 31, 23, 59, 59.0);
    assert!(tt_from_ut1_civil(civil).is_ok());
}

#[test]
fn start_of_2101_ut1_is_rejected() {
    let civil = CivilDateTime::new(2101, 1, 1, 0, 0, 0.0);
    assert!(matches!(
        tt_from_ut1_civil(civil),
        Err(CivilTimeError::BeyondHorizon { .. })
    ));
}

#[test]
fn start_of_2101_utc_is_rejected() {
    let civil = CivilDateTime::new(2101, 1, 1, 0, 0, 0.0);
    assert!(matches!(
        tt_from_utc_civil(civil),
        Err(CivilTimeError::BeyondHorizon { .. })
    ));
}

#[test]
fn tdb_differs_from_tt_sub_millisecond() {
    let civil = CivilDateTime::new(2000, 4, 1, 0, 0, 0.0);
    let tt = tt_from_utc_civil(civil).unwrap().instant.julian_day.days();
    let tdb = tdb_from_utc_civil(civil).unwrap().instant.julian_day.days();
    let diff_s = (tdb - tt).abs() * SECONDS_PER_DAY;
    assert!(diff_s < 0.002 && diff_s > 0.0, "diff {diff_s}s");
}

#[test]
fn ut1_is_earlier_than_tt_by_delta_t() {
    // J2000.0: ΔT ≈ 63.8 s, so UT1 JD < TT JD by ~63.8/86400 days.
    let jd_tt = 2_451_545.0;
    let jd_ut1 = ut1_jd_from_tt(jd_tt).unwrap();
    let diff_seconds = (jd_tt - jd_ut1) * 86_400.0;
    assert!((50.0..80.0).contains(&diff_seconds), "ΔT {diff_seconds}s");
}

#[test]
fn display_vocabulary_is_stable() {
    // The diagnostic vocabulary is release-facing; pin every variant.
    assert_eq!(ConversionPath::UtcLeapSecond.to_string(), "utc-leap-second");
    assert_eq!(ConversionPath::Ut1DeltaT.to_string(), "ut1-delta-t");
    assert_eq!(
        ConversionPath::FutureExtrapolated.to_string(),
        "future-extrapolated"
    );
    assert_eq!(ConversionQuality::Exact.to_string(), "exact");
    assert_eq!(ConversionQuality::Observed.to_string(), "observed");
    assert_eq!(ConversionQuality::Predicted.to_string(), "predicted");

    let exact = ConversionProvenance {
        path: ConversionPath::UtcLeapSecond,
        quality: ConversionQuality::Exact,
        delta_t_seconds: None,
        tai_minus_utc: Some(37),
        sources: "test",
    };
    assert_eq!(
        exact.summary_line(),
        "civil-time path=utc-leap-second quality=exact delta_t=n/a tai_minus_utc=37s"
    );
    let observed = ConversionProvenance {
        path: ConversionPath::Ut1DeltaT,
        quality: ConversionQuality::Observed,
        delta_t_seconds: Some(63.8),
        tai_minus_utc: None,
        sources: "test",
    };
    assert_eq!(
        observed.summary_line(),
        "civil-time path=ut1-delta-t quality=observed delta_t=63.800s tai_minus_utc=n/a"
    );
    assert_eq!(
        observed.to_string(),
        "civil-time path=ut1-delta-t quality=observed delta_t=63.800s tai_minus_utc=n/a"
    );
}

#[test]
fn finite_guard_fails_closed_on_non_finite() {
    // White-box: no public input can reach this guard (the 1900-2101
    // window bounds jd and dT is bounded ~153 s inside it — spec §4.1
    // group B, overflow lens checked), so its fail-closed contract is
    // asserted directly.
    assert!(finite(f64::NAN).is_err());
    assert!(finite(f64::INFINITY).is_err());
    assert!(finite(f64::NEG_INFINITY).is_err());
    assert!(finite(0.0).is_ok());
}

#[test]
fn utc_at_exact_leap_epoch_is_exact() {
    // 1972-01-01 00:00:00 UTC is exactly LEAP_EPOCH_JD (2441317.5): the
    // first instant of the leap-second era belongs to it, not to the
    // pre-1972 rejection. jd_tt = 2441317.5 + (10 + 32.184)/86400,
    // computed outside the code.
    let out = tt_from_utc_civil(CivilDateTime::new(1972, 1, 1, 0, 0, 0.0)).unwrap();
    assert_eq!(out.provenance.path, ConversionPath::UtcLeapSecond);
    assert_eq!(out.provenance.quality, ConversionQuality::Exact);
    assert_eq!(out.provenance.tai_minus_utc, Some(10));
    let jd = out.instant.julian_day.days();
    assert!((jd - 2_441_317.500_488_240_7).abs() < 1e-9, "got {jd}");
}

#[test]
fn future_utc_and_ut1_jd_values_match_hand_computation() {
    // Future-UTC path: 2090-06-01 00:00 UTC -> jd_civil 2484568.5 (past
    // the leap table's VALID_THROUGH_JD, inside the support window).
    // dT = Espenak-Meeus polynomial at decimal_year(2484568.5), anchored at
    // the leap horizon (JD 2461221.5, decimal year 2026.4928131416839) to the
    // leap-second bound 69.184 s, evaluated outside the code with Python:
    // 69.184 + P(2090.4133) − P(2026.4928) = 131.5423132417809 s (the
    // unanchored polynomial gives 137.736 s). Smallest mutant displacement
    // on this path is 3.19e-3 days (spec §4.1 group D).
    let out = tt_from_utc_civil(CivilDateTime::new(2090, 6, 1, 0, 0, 0.0)).unwrap();
    assert_eq!(out.provenance.path, ConversionPath::FutureExtrapolated);
    let dt = out.provenance.delta_t_seconds.unwrap();
    assert!((dt - 131.542_313_241_780_9).abs() < 1e-9, "dt {dt}");
    let jd = out.instant.julian_day.days();
    assert!((jd - 2_484_568.501_522_480_5).abs() < 1e-9, "jd_tt {jd}");

    // UT1 path: 1955-06-15 00:00 UT1 -> jd_civil 2435273.5. dT hand-
    // interpolated between the committed 1950 (29.1 s) and 1960 (33.2 s)
    // decade nodes: 31.334934976043815 s. Smallest mutant displacement
    // on this path is 7.25e-4 days.
    let out = tt_from_ut1_civil(CivilDateTime::new(1955, 6, 15, 0, 0, 0.0)).unwrap();
    assert_eq!(out.provenance.quality, ConversionQuality::Observed);
    let dt = out.provenance.delta_t_seconds.unwrap();
    assert!((dt - 31.334_934_976_043_815).abs() < 1e-9, "dt {dt}");
    let jd = out.instant.julian_day.days();
    assert!((jd - 2_435_273.500_362_673).abs() < 1e-9, "jd_tt {jd}");
}

#[test]
fn tdb_target_applies_positive_periodic_term() {
    // 2000-04-01 sits near the annual peak of TDB - TT (g ~ 87 deg),
    // where the USNO term is +1.6569e-3 s (evaluated outside the code at
    // jd_tt = 2451635.5 + 64.184/86400). The SIGNED assertion pins what
    // the .abs() bound test cannot: the +/- mutant lands at -1.6569e-3 s,
    // a 3.3e-3 s displacement vs the 5e-5 s tolerance (which budgets
    // half-ulp(JD) ~ 2.0e-5 s of JD-grid quantization on the stored TDB
    // day; the observable diff computes to 1.64956e-3 s, 7.3e-6 s from
    // the formula value — inside budget).
    let civil = CivilDateTime::new(2000, 4, 1, 0, 0, 0.0);
    let tt = tt_from_utc_civil(civil).unwrap();
    assert_eq!(tt.provenance.tai_minus_utc, Some(32));
    let jd_tt = tt.instant.julian_day.days();
    let jd_tdb = tdb_from_utc_civil(civil).unwrap().instant.julian_day.days();
    let diff_s = (jd_tdb - jd_tt) * SECONDS_PER_DAY;
    assert!(
        (diff_s - 0.001_656_892_188_342_611_6).abs() < 5e-5,
        "TDB-TT {diff_s}s"
    );
}

fn seconds_between(earlier: &CivilInstant, later: &CivilInstant) -> f64 {
    (later.instant.julian_day.days() - earlier.instant.julian_day.days()) * SECONDS_PER_DAY
}

#[test]
fn insertion_helper_matches_known_dates() {
    // Anchors the table-derived helper to dates known independently.
    let all = insertions();
    assert_eq!(all.len(), 27);
    let first = &all[0];
    assert_eq!(
        (
            first.last_day.year,
            first.last_day.month,
            first.last_day.day
        ),
        (1972, 6, 30)
    );
    assert_eq!(first.offset_before, 10);
    let last = &all[26];
    assert_eq!(
        (last.last_day.year, last.last_day.month, last.last_day.day),
        (2016, 12, 31)
    );
    assert_eq!(
        (last.next_day.year, last.next_day.month, last.next_day.day),
        (2017, 1, 1)
    );
    assert_eq!(last.offset_before, 36);
}

#[test]
fn utc_leap_second_sits_between_its_neighbours() {
    // 2016-12-31T23:59:60.5 is one second after 23:59:59.5 and one second
    // before 2017-01-01T00:00:00.5. Before the fix it aliased the latter.
    let before = tt_from_utc_civil(CivilDateTime::new(2016, 12, 31, 23, 59, 59.5)).unwrap();
    let inside = tt_from_utc_civil(CivilDateTime::new(2016, 12, 31, 23, 59, 60.5)).unwrap();
    let after = tt_from_utc_civil(CivilDateTime::new(2017, 1, 1, 0, 0, 0.5)).unwrap();
    // A Julian day near 2.46e6 resolves ~40 µs; 2e-4 s clears the noise.
    assert!((seconds_between(&before, &inside) - 1.0).abs() < 2e-4);
    assert!((seconds_between(&inside, &after) - 1.0).abs() < 2e-4);
    assert_eq!(inside.provenance.tai_minus_utc, Some(36));
    assert_eq!(inside.provenance.quality, ConversionQuality::Exact);
    assert_eq!(after.provenance.tai_minus_utc, Some(37));
}

#[test]
fn every_inserted_leap_second_is_one_second_long() {
    for insertion in insertions() {
        let last_ordinary = tt_from_utc_civil(at(insertion.last_day, 23, 59, 59.0)).unwrap();
        let leap = tt_from_utc_civil(at(insertion.last_day, 23, 59, 60.0)).unwrap();
        let midnight = tt_from_utc_civil(at(insertion.next_day, 0, 0, 0.0)).unwrap();
        assert!((seconds_between(&last_ordinary, &leap) - 1.0).abs() < 2e-4);
        assert!((seconds_between(&leap, &midnight) - 1.0).abs() < 2e-4);
        assert_eq!(leap.provenance.tai_minus_utc, Some(insertion.offset_before));
        assert_eq!(
            midnight.provenance.tai_minus_utc,
            Some(insertion.offset_before + 1)
        );
    }
}

#[test]
fn leap_second_converts_to_tdb_too() {
    let tt = tt_from_utc_civil(CivilDateTime::new(2016, 12, 31, 23, 59, 60.5)).unwrap();
    let tdb = tdb_from_utc_civil(CivilDateTime::new(2016, 12, 31, 23, 59, 60.5)).unwrap();
    assert_eq!(tdb.instant.scale, TimeScale::Tdb);
    let gap = seconds_between(&tt, &tdb);
    assert!(gap.abs() < 2e-3, "TDB − TT is bounded by 2 ms, got {gap}");
}

#[test]
fn second_sixty_is_rejected_where_no_leap_second_was_inserted() {
    let invalid = Err(CivilTimeError::InvalidCivilDate { field: "second" });
    // A day with no insertion.
    assert_eq!(
        tt_from_utc_civil(CivilDateTime::new(2016, 12, 30, 23, 59, 60.0)),
        invalid
    );
    // The right day, the wrong minute.
    assert_eq!(
        tt_from_utc_civil(CivilDateTime::new(2016, 12, 31, 12, 0, 60.0)),
        invalid
    );
    // 1972-01-01 is the table's epoch, not an insertion.
    assert_eq!(
        tt_from_utc_civil(CivilDateTime::new(1971, 12, 31, 23, 59, 60.0)),
        invalid
    );
    // UT1 has no leap seconds.
    assert_eq!(
        tt_from_ut1_civil(CivilDateTime::new(2016, 12, 31, 23, 59, 60.5)),
        invalid
    );
}

#[test]
fn out_of_range_seconds_are_still_rejected() {
    let invalid = Err(CivilTimeError::InvalidCivilDate { field: "second" });
    assert_eq!(
        tt_from_utc_civil(CivilDateTime::new(2016, 12, 31, 23, 59, 61.0)),
        invalid
    );
    assert_eq!(
        tt_from_utc_civil(CivilDateTime::new(2016, 12, 31, 23, 59, f64::NAN)),
        invalid
    );
}

/// The sub-millisecond seconds at the end of an insertion day's `23:59`
/// that the Julian day cannot keep apart from the next midnight.
const LAST_MICROSECONDS: [f64; 4] = [59.999_98, 59.999_999_99, 60.999_98, 60.999_999_99];

#[test]
fn last_microseconds_of_a_leap_day_keep_the_old_offset() {
    // A Julian day near 2.46e6 resolves ~40 µs, so these seconds once
    // rounded to the next midnight and picked up the new offset (1 s late).
    for insertion in insertions() {
        let anchor = tt_from_utc_civil(at(insertion.last_day, 23, 59, 59.0)).unwrap();
        for second in LAST_MICROSECONDS {
            let out = tt_from_utc_civil(at(insertion.last_day, 23, 59, second)).unwrap();
            let elapsed = seconds_between(&anchor, &out);
            assert!(
                (elapsed - (second - 59.0)).abs() < 2e-4,
                "{:?} second {second}: elapsed {elapsed}",
                insertion.last_day
            );
            assert_eq!(out.provenance.tai_minus_utc, Some(insertion.offset_before));
        }
    }
}

#[test]
fn last_microseconds_of_an_ordinary_day_still_convert() {
    // 2016-12-30 ends without an insertion: `:59.x` converts as before and
    // `:60.x` stays invalid.
    let day = CivilDateTime::new(2016, 12, 30, 0, 0, 0.0);
    for second in LAST_MICROSECONDS {
        let result = tt_from_utc_civil(at(day, 23, 59, second));
        if second < 60.0 {
            assert_eq!(result.unwrap().provenance.tai_minus_utc, Some(36));
        } else {
            assert_eq!(
                result,
                Err(CivilTimeError::InvalidCivilDate { field: "second" })
            );
        }
    }
}

mod properties {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn last_two_seconds_of_a_leap_day_keep_the_old_offset(
            index in 0..insertions().len(),
            second in 59.0f64..61.0,
        ) {
            let insertion = &insertions()[index];
            let anchor = tt_from_utc_civil(at(insertion.last_day, 23, 59, 59.0)).unwrap();
            let out = tt_from_utc_civil(at(insertion.last_day, 23, 59, second)).unwrap();
            let elapsed = seconds_between(&anchor, &out);
            prop_assert!(
                (elapsed - (second - 59.0)).abs() < 2e-4,
                "second {second}: elapsed {elapsed}"
            );
            prop_assert_eq!(out.provenance.tai_minus_utc, Some(insertion.offset_before));
        }
    }
}
