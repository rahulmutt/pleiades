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

const BASE_JD: f64 = 2_451_545.0;

/// A synthetic body moving smoothly in longitude, latitude and distance:
/// `distance_rate` AU/day sets how far each light-time step moves the epoch.
fn moving_body(jd: f64, distance_au: f64, distance_rate: f64) -> EclipticCoordinates {
    let dt = jd - BASE_JD;
    EclipticCoordinates::new(
        Longitude::from_degrees(100.0 + 0.1 * dt),
        Latitude::from_degrees(1.0 + 0.05 * dt),
        Some(distance_au + distance_rate * dt),
    )
}

/// The light-time loop as it ran before #247: every retarded epoch is queried.
/// Returns the position, the retardation and the number of queries.
fn queried_every_step(
    distance_au: f64,
    distance_rate: f64,
    max_iterations: u8,
) -> (EclipticCoordinates, f64, usize) {
    let mut tau = 0.0;
    let mut last = moving_body(BASE_JD, distance_au, distance_rate);
    // The first query is at the unretarded instant; step `n` has made `n` queries.
    for queries in 1..=usize::from(max_iterations) {
        let new_tau = last.distance_au.unwrap() * LIGHT_TIME_DAYS_PER_AU;
        if (new_tau - tau).abs() < CONVERGENCE_DAYS {
            return (last, new_tau, queries);
        }
        tau = new_tau;
        last = moving_body(BASE_JD - tau, distance_au, distance_rate);
    }
    panic!("reference loop did not converge");
}

/// Runs the production loop on [`moving_body`], counting backend queries.
fn light_time_counting_queries(
    distance_au: f64,
    distance_rate: f64,
    max_iterations: u8,
) -> (LightTimePosition, usize) {
    let instant = Instant::new(JulianDay::from_days(BASE_JD), TimeScale::Tt);
    let mut queries = 0;
    let out = apparent_via_light_time::<_, ApparentPlaceError>(instant, max_iterations, |i| {
        queries += 1;
        Ok(moving_body(i.julian_day.days(), distance_au, distance_rate))
    })
    .unwrap();
    (out, queries)
}

fn assert_matches_queried_reference(
    out: &LightTimePosition,
    reference: &EclipticCoordinates,
    tolerance: f64,
) {
    let d_lon = out.ecliptic.longitude.degrees() - reference.longitude.degrees();
    let d_lat = out.ecliptic.latitude.degrees() - reference.latitude.degrees();
    let d_dist = out.ecliptic.distance_au.unwrap() - reference.distance_au.unwrap();
    assert!(d_lon.abs() < tolerance, "longitude off by {d_lon}°");
    assert!(d_lat.abs() < tolerance, "latitude off by {d_lat}°");
    // The chord's radial error is the same angular error times the distance.
    let distance_tolerance = tolerance.to_radians() * reference.distance_au.unwrap();
    assert!(
        d_dist.abs() < distance_tolerance,
        "distance off by {d_dist} AU"
    );
}

#[test]
fn outer_planet_third_step_is_interpolated_not_queried() {
    // Saturn-like: 9 AU receding at 0.017 AU/day (the Earth's orbital speed).
    // The second step moves τ by k·ṙ·τ₁ ≈ 5e-6 d, above the 5e-7 d threshold,
    // so a third step is taken — on the line through the two queried
    // positions, not by a third query (#247).
    let (out, queries) = light_time_counting_queries(9.0, 0.017, 8);
    let (reference, reference_tau, reference_queries) = queried_every_step(9.0, 0.017, 8);
    assert_eq!(
        reference_queries, 3,
        "precondition: the old loop queried three times"
    );
    assert_eq!(queries, 2);
    assert_eq!(out.iterations, 3);
    // Interpolation error ½·a·τ₁·δ with a ≈ 2ṙω/r ≈ 7e-5 °/d², τ₁ ≈ 0.052 d,
    // δ ≈ 5e-6 d: about 1e-11°.
    assert_matches_queried_reference(&out, &reference, 1e-9);
    assert!((out.light_time_days - reference_tau).abs() < 1e-12);
}

#[test]
fn successive_steps_keep_interpolating_on_the_queried_line() {
    // A distant fast-receding body (150 AU at 0.15 AU/day) has k·ṙ ≈ 8.7e-4:
    // each step shrinks by that factor, so two interpolated steps follow the
    // two queries before τ settles.
    let (out, queries) = light_time_counting_queries(150.0, 0.15, 8);
    let (reference, _, reference_queries) = queried_every_step(150.0, 0.15, 8);
    assert_eq!(
        reference_queries, 4,
        "precondition: the old loop queried four times"
    );
    assert_eq!(queries, 2);
    assert_eq!(out.iterations, 4);
    // This body is deliberately extreme: τ₁ ≈ 0.87 d and δ ≈ 7.5e-4 d, with
    // a ≈ 2ṙω/r ≈ 2e-4 °/d², so ½·a·τ₁·δ ≈ 6.5e-8° (measured 6.4e-8°).
    assert_matches_queried_reference(&out, &reference, 1e-7);
}

#[test]
fn large_retardation_step_is_queried_not_extrapolated() {
    // At 5 AU approaching at 30 AU/day, k·ṙ ≈ 0.17: each step is far beyond
    // MAX_INTERPOLATION_STEP_RATIO of the queried spacing, so every step
    // queries the backend and the result is the old loop's.
    let (out, queries) = light_time_counting_queries(5.0, -30.0, 20);
    let (reference, reference_tau, reference_queries) = queried_every_step(5.0, -30.0, 20);
    assert_eq!(queries, reference_queries);
    assert_eq!(queries, usize::from(out.iterations));
    assert_eq!(out.ecliptic, reference);
    assert_eq!(out.light_time_days, reference_tau);
}
