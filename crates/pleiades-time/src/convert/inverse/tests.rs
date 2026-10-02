use super::*;

use crate::convert::test_support::{at, insertions};
use crate::convert::{tdb_from_utc_civil, tt_from_ut1_civil, tt_from_utc_civil};
use crate::deltat::OBSERVED_THROUGH_JD;

fn tt(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

/// `instant` moved by `seconds`, keeping its scale.
fn shifted(instant: Instant, seconds: f64) -> Instant {
    Instant::new(
        JulianDay::from_days(instant.julian_day.days() + seconds / SECONDS_PER_DAY),
        instant.scale,
    )
}

#[test]
fn millisecond_axis_is_exact_at_midnights_and_round_trips() {
    // JD 2457754.5 is 2017-01-01 00:00: day number 2457755, millisecond 0.
    assert_eq!(ms_from_jd(2_457_754.5), 2_457_755 * MS_PER_DAY);
    assert_eq!(jd_from_ms(2_457_755 * MS_PER_DAY), 2_457_754.5);
    // Noon, plus 1 ms.
    let noon_plus = 2_457_755 * MS_PER_DAY + 43_200_001;
    assert_eq!(ms_from_jd(jd_from_ms(noon_plus)), noon_plus);
    assert_eq!(
        civil_from_ms(noon_plus),
        CivilDateTime::new(2017, 1, 1, 12, 0, 0.001)
    );
    // The last millisecond of a day stays on that day.
    assert_eq!(
        civil_from_ms(2_457_755 * MS_PER_DAY - 1),
        CivilDateTime::new(2016, 12, 31, 23, 59, 59.999)
    );
}

#[test]
fn utc_modern_inverts_exactly() {
    let civil = CivilDateTime::new(2017, 1, 1, 0, 0, 0.0);
    let forward = tt_from_utc_civil(civil).unwrap();
    let back = from_terrestrial(forward.instant, TimeScale::Utc).unwrap();
    assert_eq!(back.civil, civil);
    assert_eq!(back.scale, TimeScale::Utc);
    assert_eq!(back.provenance, forward.provenance);
    assert_eq!(back.provenance.quality, ConversionQuality::Exact);
    assert_eq!(back.provenance.tai_minus_utc, Some(37));
}

#[test]
fn tdb_input_inverts_to_the_same_utc() {
    let civil = CivilDateTime::new(2024, 3, 20, 3, 6, 21.25);
    let forward = tdb_from_utc_civil(civil).unwrap();
    let back = utc_civil_from_tdb(forward.instant).unwrap();
    assert_eq!(back.civil, civil);
    assert_eq!(back.provenance, forward.provenance);
}

#[test]
fn every_insertion_round_trips_through_the_leap_second() {
    for insertion in insertions() {
        let before = insertion.offset_before;
        let samples = [
            (at(insertion.last_day, 23, 59, 59.5), before),
            (at(insertion.last_day, 23, 59, 60.0), before),
            (at(insertion.last_day, 23, 59, 60.5), before),
            (at(insertion.next_day, 0, 0, 0.0), before + 1),
            (at(insertion.next_day, 0, 0, 0.5), before + 1),
        ];
        let mut previous_jd: Option<f64> = None;
        for (civil, offset) in samples {
            let forward = tt_from_utc_civil(civil).unwrap();
            let jd = forward.instant.julian_day.days();
            if let Some(previous) = previous_jd {
                let step = (jd - previous) * SECONDS_PER_DAY;
                assert!((step - 0.5).abs() < 2e-4, "{civil:?}: step {step}");
            }
            previous_jd = Some(jd);
            let back = utc_civil_from_tt(forward.instant).unwrap();
            assert_eq!(back.civil, civil);
            assert_eq!(back.provenance.tai_minus_utc, Some(offset));
            assert_eq!(back.provenance, forward.provenance);
        }
    }
}

#[test]
fn leap_second_edges_round_trip() {
    let edges = [
        CivilDateTime::new(2016, 12, 31, 23, 59, 59.999),
        CivilDateTime::new(2016, 12, 31, 23, 59, 60.0),
        CivilDateTime::new(2016, 12, 31, 23, 59, 60.999),
        CivilDateTime::new(2017, 1, 1, 0, 0, 0.0),
    ];
    for civil in edges {
        let forward = tt_from_utc_civil(civil).unwrap();
        let back = utc_civil_from_tt(forward.instant).unwrap();
        assert_eq!(back.civil, civil);
    }
}

#[test]
fn sub_millisecond_below_midnight_rounds_into_the_next_day() {
    // An ordinary midnight: 0.4 ms before it rounds up to 00:00:00.000 of the
    // new day, never to 23:59:60.000 or 24:00.
    let midnight = CivilDateTime::new(2020, 3, 1, 0, 0, 0.0);
    let forward = tt_from_utc_civil(midnight).unwrap();
    let back = utc_civil_from_tt(shifted(forward.instant, -0.0004)).unwrap();
    assert_eq!(back.civil, midnight);
    // 0.6 ms before it rounds down to the last millisecond of the old day.
    let back = utc_civil_from_tt(shifted(forward.instant, -0.0006)).unwrap();
    assert_eq!(back.civil, CivilDateTime::new(2020, 2, 29, 23, 59, 59.999));
}

#[test]
fn utc_before_1972_is_rejected() {
    let epoch = CivilDateTime::new(1972, 1, 1, 0, 0, 0.0);
    let forward = tt_from_utc_civil(epoch).unwrap();
    assert_eq!(utc_civil_from_tt(forward.instant).unwrap().civil, epoch);
    assert_eq!(
        utc_civil_from_tt(shifted(forward.instant, -0.002)),
        Err(CivilTimeError::UtcBeforeLeapEpoch)
    );
    // 1950 is inside the window but before UTC exists.
    assert_eq!(
        utc_civil_from_tt(tt(2_433_282.5)),
        Err(CivilTimeError::UtcBeforeLeapEpoch)
    );
    // UT1 is defined there.
    assert!(ut1_civil_from_tt(tt(2_433_282.5)).is_ok());
}

#[test]
fn leap_horizon_is_continuous() {
    // 2026-06-30 00:00:00 UTC is the last leap-second-exact instant.
    let horizon = CivilDateTime::new(2026, 6, 30, 0, 0, 0.0);
    let forward = tt_from_utc_civil(horizon).unwrap();
    assert_eq!(forward.provenance.quality, ConversionQuality::Exact);
    let back = utc_civil_from_tt(forward.instant).unwrap();
    assert_eq!(back.civil, horizon);
    assert_eq!(back.provenance, forward.provenance);

    let later = CivilDateTime::new(2026, 6, 30, 0, 0, 1.0);
    let forward = tt_from_utc_civil(later).unwrap();
    assert_eq!(forward.provenance.quality, ConversionQuality::Predicted);
    let back = utc_civil_from_tt(forward.instant).unwrap();
    assert_eq!(back.civil, later);
    assert_eq!(back.provenance.path, ConversionPath::FutureExtrapolated);
    assert_eq!(back.provenance.quality, ConversionQuality::Predicted);
    assert_eq!(back.provenance.tai_minus_utc, None);
    let delta_t = back.provenance.delta_t_seconds.unwrap();
    assert!((delta_t - forward.provenance.delta_t_seconds.unwrap()).abs() < 1e-6);
}

#[test]
fn ut1_inverts_with_the_forward_quality_tiers() {
    for (civil, path, quality) in [
        (
            CivilDateTime::new(1950, 1, 1, 0, 0, 0.0),
            ConversionPath::Ut1DeltaT,
            ConversionQuality::Observed,
        ),
        (
            CivilDateTime::new(2022, 1, 1, 6, 30, 15.5),
            ConversionPath::Ut1DeltaT,
            ConversionQuality::Observed,
        ),
        (
            CivilDateTime::new(2090, 6, 1, 0, 0, 0.0),
            ConversionPath::FutureExtrapolated,
            ConversionQuality::Predicted,
        ),
    ] {
        let forward = tt_from_ut1_civil(civil).unwrap();
        let back = from_terrestrial(forward.instant, TimeScale::Ut1).unwrap();
        assert_eq!(back.civil, civil);
        assert_eq!(back.scale, TimeScale::Ut1);
        assert_eq!(back.provenance.path, path);
        assert_eq!(back.provenance.quality, quality);
        assert_eq!(back.provenance.tai_minus_utc, None);
        let delta_t = back.provenance.delta_t_seconds.unwrap();
        assert!((delta_t - forward.provenance.delta_t_seconds.unwrap()).abs() < 1e-6);
    }
}

#[test]
fn ambiguous_window_at_the_2020_node_resolves_after_the_node() {
    // ΔT steps from 69.4 s to 69.184 s at the 2020 node, so TT values in
    // [node + 69.184 s, node + 69.4 s) have a UT1 preimage on each side.
    // The inverse returns the one at or after the node.
    let instant = tt(OBSERVED_THROUGH_JD + 69.3 / SECONDS_PER_DAY);
    let back = ut1_civil_from_tt(instant).unwrap();
    assert_eq!(back.civil, CivilDateTime::new(2020, 1, 1, 0, 0, 0.116));
    assert!((back.provenance.delta_t_seconds.unwrap() - 69.184).abs() < 1e-9);
}

#[test]
fn results_outside_the_window_are_beyond_horizon() {
    let start = tt_from_ut1_civil(CivilDateTime::new(1900, 1, 1, 0, 0, 0.0)).unwrap();
    assert!(ut1_civil_from_tt(start.instant).is_ok());
    assert!(matches!(
        ut1_civil_from_tt(shifted(start.instant, -1.0)),
        Err(CivilTimeError::BeyondHorizon { .. })
    ));
    let end = tt_from_ut1_civil(CivilDateTime::new(2100, 12, 31, 23, 59, 59.0)).unwrap();
    assert!(ut1_civil_from_tt(end.instant).is_ok());
    assert!(matches!(
        ut1_civil_from_tt(shifted(end.instant, 2.0)),
        Err(CivilTimeError::BeyondHorizon { .. })
    ));
    // Before 1900 a UTC target reports the window, not the 1972 epoch.
    assert!(matches!(
        utc_civil_from_tt(tt(2_396_758.5)),
        Err(CivilTimeError::BeyondHorizon { .. })
    ));
}

#[test]
fn absurd_julian_days_are_beyond_horizon() {
    for jd in [1e300, -1e300, f64::MAX, f64::MIN, 0.0] {
        for target in [TimeScale::Utc, TimeScale::Ut1] {
            assert!(
                matches!(
                    from_terrestrial(tt(jd), target),
                    Err(CivilTimeError::BeyondHorizon { .. })
                ),
                "jd {jd} target {target}"
            );
        }
    }
}

#[test]
fn non_finite_instants_are_rejected() {
    for jd in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            from_terrestrial(tt(jd), TimeScale::Utc),
            Err(CivilTimeError::NonFiniteOffset)
        );
    }
}

#[test]
fn unsupported_scales_are_rejected() {
    let jd = JulianDay::from_days(2_451_545.0);
    let utc = Instant::new(jd, TimeScale::Utc);
    assert_eq!(
        from_terrestrial(utc, TimeScale::Utc),
        Err(CivilTimeError::UnsupportedScale {
            source: TimeScale::Utc,
            target: TimeScale::Utc,
        })
    );
    assert_eq!(
        from_terrestrial(tt(2_451_545.0), TimeScale::Tdb),
        Err(CivilTimeError::UnsupportedScale {
            source: TimeScale::Tt,
            target: TimeScale::Tdb,
        })
    );
}

#[test]
fn conveniences_require_their_named_source_scale() {
    let as_tt = tt(2_451_545.0);
    let as_tdb = Instant::new(as_tt.julian_day, TimeScale::Tdb);
    assert!(utc_civil_from_tt(as_tt).is_ok());
    assert!(ut1_civil_from_tt(as_tt).is_ok());
    assert!(utc_civil_from_tdb(as_tdb).is_ok());
    assert!(ut1_civil_from_tdb(as_tdb).is_ok());
    assert_eq!(
        utc_civil_from_tdb(as_tt),
        Err(CivilTimeError::UnsupportedScale {
            source: TimeScale::Tt,
            target: TimeScale::Utc,
        })
    );
    assert_eq!(
        ut1_civil_from_tt(as_tdb),
        Err(CivilTimeError::UnsupportedScale {
            source: TimeScale::Tdb,
            target: TimeScale::Ut1,
        })
    );
}

mod properties {
    use proptest::prelude::*;

    use super::*;
    use crate::convert::to_terrestrial;

    /// Asserts the inverse reproduced the forward's provenance. ΔT is solved
    /// rather than looked up, so it is compared to a microsecond.
    fn assert_same_provenance(
        back: &ConversionProvenance,
        forward: &ConversionProvenance,
    ) -> Result<(), TestCaseError> {
        prop_assert_eq!(back.path, forward.path);
        prop_assert_eq!(back.quality, forward.quality);
        prop_assert_eq!(back.tai_minus_utc, forward.tai_minus_utc);
        match (back.delta_t_seconds, forward.delta_t_seconds) {
            (Some(a), Some(b)) => prop_assert!((a - b).abs() < 1e-6, "ΔT {a} vs {b}"),
            (None, None) => {}
            other => prop_assert!(false, "ΔT presence differs: {other:?}"),
        }
        Ok(())
    }

    /// Orders civil datetimes; `:60` sorts after `:59` on the same minute.
    fn key(civil: CivilDateTime) -> (i32, u8, u8, u8, u8, f64) {
        (
            civil.year,
            civil.month,
            civil.day,
            civil.hour,
            civil.minute,
            civil.second,
        )
    }

    proptest! {
        #[test]
        fn utc_round_trips_through_tt_and_tdb(
            ms in ms_from_jd(leap::LEAP_EPOCH_JD)..ms_from_jd(SUPPORT_END_JD),
        ) {
            let civil = civil_from_ms(ms);
            for target in [TimeScale::Tt, TimeScale::Tdb] {
                let forward = to_terrestrial(civil, TimeScale::Utc, target).unwrap();
                let back = from_terrestrial(forward.instant, TimeScale::Utc).unwrap();
                prop_assert_eq!(back.civil, civil);
                prop_assert_eq!(back.scale, TimeScale::Utc);
                assert_same_provenance(&back.provenance, &forward.provenance)?;
            }
        }

        #[test]
        fn leap_seconds_round_trip_at_every_millisecond(
            index in 0usize..27,
            ms in 0i64..1_000,
        ) {
            let insertion = &insertions()[index];
            let civil = at(insertion.last_day, 23, 59, 60.0 + ms as f64 / 1_000.0);
            for target in [TimeScale::Tt, TimeScale::Tdb] {
                let forward = to_terrestrial(civil, TimeScale::Utc, target).unwrap();
                let back = from_terrestrial(forward.instant, TimeScale::Utc).unwrap();
                prop_assert_eq!(back.civil, civil);
                prop_assert_eq!(back.provenance.tai_minus_utc, Some(insertion.offset_before));
            }
        }

        #[test]
        fn ut1_round_trips_away_from_the_2020_node(
            ms in ms_from_jd(SUPPORT_START_JD)..ms_from_jd(SUPPORT_END_JD),
        ) {
            // The forward is two-to-one in a 0.216 s window at the node.
            let node_ms = ms_from_jd(crate::deltat::OBSERVED_THROUGH_JD);
            prop_assume!((ms - node_ms).abs() > 500);
            let civil = civil_from_ms(ms);
            for target in [TimeScale::Tt, TimeScale::Tdb] {
                let forward = to_terrestrial(civil, TimeScale::Ut1, target).unwrap();
                let back = from_terrestrial(forward.instant, TimeScale::Ut1).unwrap();
                prop_assert_eq!(back.civil, civil);
                prop_assert_eq!(back.scale, TimeScale::Ut1);
                assert_same_provenance(&back.provenance, &forward.provenance)?;
            }
        }

        #[test]
        fn utc_inverse_is_monotonic_and_reenters_the_forward(
            // TT from one minute after the UTC epoch; `gap` reaches past two
            // days so pairs straddle leap seconds and the leap horizon.
            a in (ms_from_jd(leap::LEAP_EPOCH_JD) + 60_000)
                ..(ms_from_jd(SUPPORT_END_JD) - 300_000_000),
            gap in 2i64..200_000_000,
        ) {
            let b = a + gap;
            let civil_a = utc_civil_from_tt(tt(jd_from_ms(a))).unwrap().civil;
            let civil_b = utc_civil_from_tt(tt(jd_from_ms(b))).unwrap().civil;
            prop_assert!(key(civil_a) < key(civil_b), "{civil_a:?} !< {civil_b:?}");
            // Every result is a datetime the forward accepts, and it lands
            // within 1 ms of where it came from.
            let again = to_terrestrial(civil_a, TimeScale::Utc, TimeScale::Tt).unwrap();
            let error = (again.instant.julian_day.days() - jd_from_ms(a)) * SECONDS_PER_DAY;
            prop_assert!(error.abs() < 1e-3, "re-entry error {error} s");
        }
    }
}
