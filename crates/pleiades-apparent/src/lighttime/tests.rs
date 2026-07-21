use super::*;
use pleiades_types::{Latitude, Longitude, TimeScale};

fn at(jd: f64, lon: f64, dist: f64) -> EclipticCoordinates {
    let _ = jd;
    EclipticCoordinates::new(
        Longitude::from_degrees(lon),
        Latitude::from_degrees(0.0),
        Some(dist),
    )
}

#[test]
fn converges_for_a_fixed_distance_body() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    // A body at constant 5 AU: τ should converge to 5 × per-AU on iteration 2.
    let out = apparent_via_light_time::<_, ApparentPlaceError>(instant, 8, |i| {
        Ok(at(i.julian_day.days(), 100.0, 5.0))
    })
    .unwrap();
    assert!((out.light_time_days - 5.0 * LIGHT_TIME_DAYS_PER_AU).abs() < 1e-9);
    assert!(out.iterations <= 3);
}

#[test]
fn missing_distance_is_rejected() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let err = apparent_via_light_time::<_, ApparentPlaceError>(instant, 8, |_| {
        Ok(EclipticCoordinates::new(
            Longitude::from_degrees(0.0),
            Latitude::from_degrees(0.0),
            None,
        ))
    })
    .unwrap_err();
    assert!(matches!(
        err,
        ApparentLightTimeError::Apparent(ApparentPlaceError::MissingDistance)
    ));
}

#[test]
fn absurd_distance_is_rejected_as_non_convergent() {
    // A body returning 50,000 AU (light-time ≈ 289 days) must be rejected
    // by the sanity cap (MAX_PLAUSIBLE_LIGHT_TIME_DAYS = 10 days), not
    // silently returned as a huge retardation.
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let err = apparent_via_light_time::<_, ApparentPlaceError>(instant, 8, |_| {
        Ok(at(0.0, 90.0, 50_000.0))
    })
    .unwrap_err();
    assert!(
        matches!(
            err,
            ApparentLightTimeError::Apparent(ApparentPlaceError::NonConvergentLightTime { .. })
        ),
        "expected NonConvergentLightTime for absurd distance, got: {err:?}"
    );
}

#[test]
fn query_error_is_propagated() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let err = apparent_via_light_time::<_, &str>(instant, 8, |_| Err("backend down")).unwrap_err();
    assert!(matches!(err, ApparentLightTimeError::Query("backend down")));
}

#[test]
fn converged_position_is_queried_at_retarded_epoch() {
    // The point of the light-time iteration is that the returned position
    // is the one evaluated at the RETARDED epoch t − τ. Every other test's
    // query ignores the instant it is given, which is exactly why the
    // retarded-epoch mutants survived the baseline. Here the query's
    // longitude depends on the instant (1000 °/day, constant 5 AU), so the
    // retarded epoch is observable: expected longitude = 100 − 1000τ
    // ≈ 71.1224083°. The expected value is recomputed below from the same
    // crafted constants via the REPRESENTABLE retarded epoch fl(BASE − τ):
    // one ulp at this JD magnitude is 2^-31 ≈ 4.66e-10 days, and the
    // 1000 °/day rate amplifies the JD grid, so the naive hand value
    // 100 − 1000·τ lands up to 2.33e-7° away from the longitude at any
    // representable epoch (measured: 2.14e-7°) — it cannot carry a 1e-9°
    // tolerance. Mutant margins at this geometry (design doc §4.2) dwarf
    // both figures: `-` -> `+` queries base + τ → 128.878° (57.8° off);
    // `-` -> `/` queries jd = base/τ ≈ 8.49e7 → 183.967° (112.8° off);
    // convergence `<` -> `>` converges on iteration 1 with the UNRETARDED
    // position → 100.0° (28.9° off) and iterations == 1.
    const BASE: f64 = 2_451_545.0;
    let tau = 5.0 * LIGHT_TIME_DAYS_PER_AU;
    let instant = Instant::new(JulianDay::from_days(BASE), TimeScale::Tt);
    let out = apparent_via_light_time::<_, ApparentPlaceError>(instant, 8, |i| {
        let lon = 100.0 + 1000.0 * (i.julian_day.days() - BASE);
        Ok(at(i.julian_day.days(), lon, 5.0))
    })
    .unwrap();
    // The same f64 ops the closure performs at the retarded epoch
    // fl(BASE − τ); production subtracts the identical τ product, so the
    // difference is exactly 0.0 and the 1e-9 tolerance is pure headroom.
    let expected_lon = 100.0 + 1000.0 * ((BASE - tau) - BASE);
    assert!(
        (out.ecliptic.longitude.degrees() - expected_lon).abs() < 1e-9,
        "longitude {} should be {expected_lon} (queried at the retarded epoch)",
        out.ecliptic.longitude.degrees()
    );
    assert_eq!(out.iterations, 2);
    // Exact: light_time_days is the same f64 product the test computes.
    assert_eq!(out.light_time_days, tau);
}

#[test]
fn light_time_exactly_at_cap_is_accepted() {
    // The cap's contract is "EXCEEDING this cap" is non-convergent — a
    // light-time exactly AT the cap converges normally, pinning the strict
    // `>`. 1731.4463361669202 AU (0x1.b0dc90c591fc7p+10) is crafted so the
    // f64 product distance × LIGHT_TIME_DAYS_PER_AU is EXACTLY 10.0
    // (design-stage representability check; asserted below as a
    // precondition so a future constant change cannot silently degrade
    // this test into the non-boundary case).
    const D_CAP: f64 = 1_731.446_336_166_920_2;
    assert_eq!(
        D_CAP * LIGHT_TIME_DAYS_PER_AU,
        MAX_PLAUSIBLE_LIGHT_TIME_DAYS
    );
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let out =
        apparent_via_light_time::<_, ApparentPlaceError>(instant, 8, |_| Ok(at(0.0, 90.0, D_CAP)))
            .unwrap();
    assert_eq!(out.light_time_days, MAX_PLAUSIBLE_LIGHT_TIME_DAYS);
    assert_eq!(out.iterations, 2);
}

#[test]
fn convergence_requires_strict_retardation_decrease() {
    // 8.6572316808346e-05 AU is crafted so the first-iteration retardation
    // change |new_tau − 0| is EXACTLY CONVERGENCE_DAYS (5e-7; asserted as a
    // precondition). The strict `<` must NOT declare convergence on
    // iteration 1 — a change equal to the threshold is not yet converged —
    // so convergence lands on iteration 2. (Also re-kills `<` -> `>`,
    // which would never converge here and exhaust max_iterations.)
    const D_CONV: f64 = 8.657_231_680_834_6e-5;
    assert_eq!(D_CONV * LIGHT_TIME_DAYS_PER_AU, CONVERGENCE_DAYS);
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let out =
        apparent_via_light_time::<_, ApparentPlaceError>(instant, 8, |_| Ok(at(0.0, 90.0, D_CONV)))
            .unwrap();
    assert_eq!(out.iterations, 2);
    assert_eq!(out.light_time_days, CONVERGENCE_DAYS);
}
