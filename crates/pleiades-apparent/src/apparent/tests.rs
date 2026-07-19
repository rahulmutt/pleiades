use super::*;
use pleiades_types::{JulianDay, TimeScale};

fn fixed(lon: f64, lat: f64, dist: f64) -> EclipticCoordinates {
    EclipticCoordinates::new(
        Longitude::from_degrees(lon),
        Latitude::from_degrees(lat),
        Some(dist),
    )
}

#[test]
fn at_j2000_only_aberration_and_nutation_shift_longitude() {
    // At J2000 precession is the identity, so the shift from mean is only
    // (Δψ + Δλ)/3600, < ~0.01°, and the precession provenance is ~0.
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let out = apparent_position::<_, ApparentPlaceError>(instant, 280.0, 8, |_| {
        Ok(fixed(100.0, 0.0, 1.0))
    })
    .unwrap();
    let shift_arcsec = (out.ecliptic.longitude.degrees() - 100.0) * 3600.0;
    assert!(shift_arcsec.abs() < 40.0, "shift {shift_arcsec}\"");
    assert!(
        out.provenance.precession_longitude_arcsec.abs() < 1.0,
        "precession should be ~0 at J2000"
    );
    assert!(out.provenance.corrections.precession);
    assert!(out.provenance.corrections.nutation_longitude);
    assert!(out.provenance.iterations >= 1);
}

#[test]
fn precession_dominates_far_from_j2000() {
    // One century from J2000, precession shifts longitude by ~1.4° (≫ the
    // arcsec aberration/nutation), and the provenance records ~5029".
    let jd = 2_451_545.0 + 36_525.0;
    let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
    let out = apparent_position::<_, ApparentPlaceError>(instant, 280.0, 8, |_| {
        Ok(fixed(100.0, 0.0, 1.0))
    })
    .unwrap();
    let shift_deg = out.ecliptic.longitude.degrees() - 100.0;
    assert!((shift_deg - 1.397).abs() < 0.02, "shift {shift_deg}°");
    assert!(
        (out.provenance.precession_longitude_arcsec - 5029.0).abs() < 80.0,
        "precession_lon {}\"",
        out.provenance.precession_longitude_arcsec
    );
}

#[test]
fn latitude_moves_by_precession_and_aberration_only() {
    // At J2000, Δψ does not change latitude; only aberration's sub-arcsec Δβ does.
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let out = apparent_position::<_, ApparentPlaceError>(instant, 280.0, 8, |_| {
        Ok(fixed(100.0, 5.0, 1.0))
    })
    .unwrap();
    let dlat_arcsec = (out.ecliptic.latitude.degrees() - 5.0) * 3600.0;
    assert!(dlat_arcsec.abs() < 1.0, "Δβ {dlat_arcsec}\"");
}

#[test]
fn sun_applies_aberration_once_no_light_time_requery() {
    // At J2000, precession ≈ identity. The Sun routine must apply aberration
    // exactly once and NOT re-query light-time. Compare against a hand-built
    // single-aberration reference: precess (≈identity here) + Δψ + one annual
    // aberration term with ⊙ = λ.
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let sun_j2000 = fixed(280.0, 0.0, 0.983);
    let out = apparent_sun_position(instant, sun_j2000).unwrap();

    // Reference: same math, applied once.
    let jd = 2_451_545.0_f64;
    let p = crate::precession::precess_ecliptic_j2000_to_date(280.0, 0.0, jd).unwrap();
    let lambda = p.longitude_deg;
    let beta = p.latitude_deg;
    let ab = crate::aberration::annual_aberration(lambda, beta, lambda, jd);
    let nut = crate::nutation::nutation(jd).unwrap();
    let expected_lon =
        (lambda + (ab.d_lambda_arcsec + nut.delta_psi_arcsec) / 3600.0).rem_euclid(360.0);

    let diff_arcsec = (out.ecliptic.longitude.degrees() - expected_lon) * 3600.0;
    assert!(
        diff_arcsec.abs() < 1e-6,
        "Sun apparent lon off by {diff_arcsec}\""
    );

    // Distance is passed through unchanged (no re-query).
    assert_eq!(out.ecliptic.distance_au, Some(0.983));

    // Provenance: aberration once, no light-time iteration.
    assert!(
        !out.provenance.corrections.light_time,
        "light_time must be false for Sun"
    );
    assert!(out.provenance.corrections.annual_aberration);
    assert!(out.provenance.corrections.precession);
    assert!(out.provenance.corrections.nutation_longitude);
    assert_eq!(out.provenance.light_time_days, 0.0);
    assert_eq!(out.provenance.iterations, 0);
    // The single applied aberration term, recorded (≈ -20" for the Sun).
    assert!((out.provenance.aberration_longitude_arcsec - ab.d_lambda_arcsec).abs() < 1e-9);
}

#[test]
fn apsis_position_is_precession_and_nutation_only_no_aberration() {
    // At J2000 precession ≈ identity, so the only shift is Δψ/3600 (a few
    // arcsec). There must be NO ~20" annual-aberration term and NO change to
    // latitude (aberration is what would move β; precession at J2000 does not).
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let j2000 = fixed(100.0, 5.0, 0.0025);
    let out = apparent_apsis_position(instant, j2000).unwrap();

    let dlon_arcsec = (out.ecliptic.longitude.degrees() - 100.0) * 3600.0;
    assert!(
        dlon_arcsec.abs() < 20.0,
        "lon shift {dlon_arcsec}\" must be precession+nutation only"
    );
    // Lower bound: nutation Δψ at J2000 ≈ -17"; a no-Δψ regression (precession ≈ 0 here)
    // would leave dlon ≈ 0 and pass the upper bound silently.
    assert!(
        dlon_arcsec.abs() > 1.0,
        "nutation Δψ not applied: lon shift only {dlon_arcsec}\""
    );
    assert!(
        out.provenance.nutation_longitude_arcsec.abs() > 1.0,
        "provenance nutation arcsec implausibly small: {}",
        out.provenance.nutation_longitude_arcsec
    );
    assert!(
        (out.ecliptic.latitude.degrees() - 5.0).abs() < 1e-6,
        "latitude must be unchanged by aberration"
    );
    assert_eq!(out.provenance.aberration_longitude_arcsec, 0.0);
    assert!(!out.provenance.corrections.annual_aberration);
    assert!(!out.provenance.corrections.light_time);
    assert!(out.provenance.corrections.precession);
    assert!(out.provenance.corrections.nutation_longitude);
    assert_eq!(out.provenance.iterations, 0);
}

#[test]
fn combine_apparent_applies_all_terms_with_correct_scaling() {
    // Crafted, mutually-distinct arcsec terms so no term-swap aliases another,
    // and no wrap is triggered (result stays in-range). Expected computed by
    // hand: lon = 100 + (12 + 5)/3600 ; lat = 20 + (-9)/3600.
    let (lon, lat) = combine_apparent(100.0, 20.0, 12.0, -9.0, 5.0, "t").unwrap();
    assert!((lon - (100.0 + 17.0 / 3600.0)).abs() < 1e-9, "lon {lon}");
    assert!((lat - (20.0 - 9.0 / 3600.0)).abs() < 1e-9, "lat {lat}");
}

#[test]
fn combine_apparent_normalizes_longitude_above_360() {
    // λ + corrections crosses 360 -> must wrap into [0, 360).
    let (lon, _) = combine_apparent(359.999, 0.0, 7200.0, 0.0, 0.0, "t").unwrap();
    // 359.999 + 7200/3600 = 361.999 -> 1.999
    assert!((lon - 1.999).abs() < 1e-9, "lon {lon}");
    assert!((0.0..360.0).contains(&lon), "lon out of range: {lon}");
}

#[test]
fn combine_apparent_normalizes_longitude_below_zero() {
    // λ + corrections goes negative -> rem_euclid must return a positive angle.
    let (lon, _) = combine_apparent(0.001, 0.0, -7200.0, 0.0, 0.0, "t").unwrap();
    // 0.001 - 2.0 = -1.999 -> 358.001
    assert!((lon - 358.001).abs() < 1e-9, "lon {lon}");
    assert!((0.0..360.0).contains(&lon), "lon out of range: {lon}");
}

#[test]
fn combine_apparent_fails_closed_on_non_finite_with_stage() {
    let err = combine_apparent(f64::NAN, 0.0, 0.0, 0.0, 0.0, "stage-x").unwrap_err();
    assert!(
        matches!(
            err,
            ApparentPlaceError::NonFiniteCorrection { stage: "stage-x" }
        ),
        "unexpected error: {err:?}"
    );
}

#[test]
fn precession_shift_no_wrap_midrange() {
    // Small positive shift, neither branch taken: (100.5 - 100.0) * 3600.
    let s = precession_shift_arcsec(100.5, 100.0);
    assert!((s - 0.5 * 3600.0).abs() < 1e-6, "shift {s}");
}

#[test]
fn precession_shift_wraps_large_positive_raw() {
    // Raw shift 359.9 - 0.1 = 359.8 (> 180) -> -0.2 deg -> -720".
    let s = precession_shift_arcsec(359.9, 0.1);
    assert!((s - (-0.2 * 3600.0)).abs() < 1e-6, "shift {s}");
}

#[test]
fn precession_shift_wraps_large_negative_raw() {
    // Raw shift 0.1 - 359.9 = -359.8 (< -180) -> +0.2 deg -> +720".
    let s = precession_shift_arcsec(0.1, 359.9);
    assert!((s - (0.2 * 3600.0)).abs() < 1e-6, "shift {s}");
}

#[test]
fn precession_shift_at_positive_180_boundary_does_not_wrap() {
    // Raw shift exactly +180.0 stays in-range (interval is (-180, 180]): no
    // subtraction. Kills the `> 180.0` -> `>= 180.0` comparison-swap mutant,
    // which would wrap 180 -> -180. Behavior-preserving: pins current output.
    let s = precession_shift_arcsec(280.0, 100.0); // 280 - 100 = 180.0 exactly
    assert!((s - 180.0 * 3600.0).abs() < 1e-6, "shift {s}");
}

#[test]
fn precession_shift_at_negative_180_boundary_does_not_wrap() {
    // Raw shift exactly -180.0: the guard is `shift < -180.0` (strict), so
    // -180.0 does NOT wrap and stays -180.0. Kills the `< -180.0` -> `<= -180.0`
    // comparison-swap mutant, which would wrap -180 -> +180. This pins the
    // helper's CURRENT boundary behavior; do NOT change the helper.
    let s = precession_shift_arcsec(100.0, 280.0); // 100 - 280 = -180.0 exactly
    assert!((s - (-180.0 * 3600.0)).abs() < 1e-6, "shift {s}");
}

#[test]
fn apparent_position_provenance_is_fully_specified() {
    let jd = 2_451_545.0 + 36_525.0; // one century from J2000, precession resolvable
    let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
    let (l0, b0) = (100.0, 5.0);
    let out =
        apparent_position::<_, ApparentPlaceError>(instant, 280.0, 8, |_| Ok(fixed(l0, b0, 1.0)))
            .unwrap();

    // Independent recomposition of the corrections.
    let p = crate::precession::precess_ecliptic_j2000_to_date(l0, b0, jd).unwrap();
    let ab = crate::aberration::annual_aberration(p.longitude_deg, p.latitude_deg, 280.0, jd);
    let nut = crate::nutation::nutation(jd).unwrap();

    assert!((out.provenance.nutation_longitude_arcsec - nut.delta_psi_arcsec).abs() < 1e-12);
    assert!((out.provenance.aberration_longitude_arcsec - ab.d_lambda_arcsec).abs() < 1e-12);
    assert!(
        (out.provenance.precession_longitude_arcsec - precession_shift_arcsec(p.longitude_deg, l0))
            .abs()
            < 1e-9
    );
    assert!(
        out.provenance.light_time_days > 0.0,
        "light-time must be applied"
    );
    assert!(out.provenance.iterations >= 1);
    assert_eq!(
        out.ecliptic.distance_au,
        Some(1.0),
        "distance passes through"
    );
    // CorrectionSet — every flag pinned.
    assert!(out.provenance.corrections.light_time);
    assert!(out.provenance.corrections.precession);
    assert!(out.provenance.corrections.annual_aberration);
    assert!(out.provenance.corrections.nutation_longitude);
    assert!(!out.provenance.corrections.diurnal_parallax);
    assert!(!out.provenance.corrections.diurnal_aberration);
}

#[test]
fn apparent_sun_position_provenance_is_fully_specified() {
    let jd = 2_451_545.0;
    let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
    let sun_j2000 = fixed(280.0, 0.0, 0.983);
    let out = apparent_sun_position(instant, sun_j2000).unwrap();

    let p = crate::precession::precess_ecliptic_j2000_to_date(280.0, 0.0, jd).unwrap();
    // Sun is its own aberration argument: ⊙ = λ.
    let ab =
        crate::aberration::annual_aberration(p.longitude_deg, p.latitude_deg, p.longitude_deg, jd);
    let nut = crate::nutation::nutation(jd).unwrap();

    assert!((out.provenance.nutation_longitude_arcsec - nut.delta_psi_arcsec).abs() < 1e-12);
    assert!((out.provenance.aberration_longitude_arcsec - ab.d_lambda_arcsec).abs() < 1e-12);
    assert!(
        (out.provenance.precession_longitude_arcsec
            - precession_shift_arcsec(p.longitude_deg, 280.0))
        .abs()
            < 1e-9
    );
    assert_eq!(
        out.provenance.light_time_days, 0.0,
        "Sun applies no light-time"
    );
    assert_eq!(out.provenance.iterations, 0);
    assert_eq!(out.ecliptic.distance_au, Some(0.983));
    // CorrectionSet — light_time false, the Sun-specific difference.
    assert!(!out.provenance.corrections.light_time);
    assert!(out.provenance.corrections.precession);
    assert!(out.provenance.corrections.annual_aberration);
    assert!(out.provenance.corrections.nutation_longitude);
    assert!(!out.provenance.corrections.diurnal_parallax);
    assert!(!out.provenance.corrections.diurnal_aberration);
}

#[test]
fn apparent_apsis_position_provenance_is_fully_specified() {
    let jd = 2_451_545.0;
    let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
    let j2000 = fixed(100.0, 5.0, 0.0025);
    let out = apparent_apsis_position(instant, j2000).unwrap();

    let nut = crate::nutation::nutation(jd).unwrap();
    let p = crate::precession::precess_ecliptic_j2000_to_date(100.0, 5.0, jd).unwrap();
    assert!((out.provenance.nutation_longitude_arcsec - nut.delta_psi_arcsec).abs() < 1e-12);
    assert!(
        (out.provenance.precession_longitude_arcsec
            - precession_shift_arcsec(p.longitude_deg, 100.0))
        .abs()
            < 1e-9
    );
    // Apse line carries NO aberration term.
    assert_eq!(out.provenance.aberration_longitude_arcsec, 0.0);
    assert_eq!(out.provenance.light_time_days, 0.0);
    assert_eq!(out.provenance.iterations, 0);
    assert_eq!(out.ecliptic.distance_au, Some(0.0025));
    // CorrectionSet — annual_aberration false, the apsis-specific difference.
    assert!(!out.provenance.corrections.light_time);
    assert!(out.provenance.corrections.precession);
    assert!(!out.provenance.corrections.annual_aberration);
    assert!(out.provenance.corrections.nutation_longitude);
    assert!(!out.provenance.corrections.diurnal_parallax);
    assert!(!out.provenance.corrections.diurnal_aberration);
}

#[test]
fn apparent_position_equals_independent_recomposition() {
    let jd = 2_451_545.0 + 36_525.0;
    let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
    let (l0, b0) = (100.0, 5.0);
    let sun = 280.0;
    let out =
        apparent_position::<_, ApparentPlaceError>(instant, sun, 8, |_| Ok(fixed(l0, b0, 1.0)))
            .unwrap();

    let p = crate::precession::precess_ecliptic_j2000_to_date(l0, b0, jd).unwrap();
    let ab = crate::aberration::annual_aberration(p.longitude_deg, p.latitude_deg, sun, jd);
    let nut = crate::nutation::nutation(jd).unwrap();
    let (exp_lon, exp_lat) = combine_apparent(
        p.longitude_deg,
        p.latitude_deg,
        ab.d_lambda_arcsec,
        ab.d_beta_arcsec,
        nut.delta_psi_arcsec,
        "apparent-combine",
    )
    .unwrap();

    let dlon = (out.ecliptic.longitude.degrees() - exp_lon) * 3600.0;
    let dlat = (out.ecliptic.latitude.degrees() - exp_lat) * 3600.0;
    assert!(dlon.abs() < 1e-6, "lon off by {dlon}\"");
    assert!(dlat.abs() < 1e-6, "lat off by {dlat}\"");
}

#[test]
fn apparent_position_propagates_non_finite_query() {
    // A NaN longitude out of `query` is rejected by
    // `precess_ecliptic_j2000_to_date` BEFORE it ever reaches the
    // `apparent-combine` guard in `combine_apparent`, so the actually-produced
    // stage is "precession", not "apparent-combine". This still pins the
    // fail-closed behavior of `apparent_position` end-to-end.
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let err = apparent_position::<_, ApparentPlaceError>(instant, 280.0, 8, |_| {
        Ok(fixed(f64::NAN, 0.0, 1.0))
    })
    .unwrap_err();
    assert!(
        matches!(
            err,
            ApparentLightTimeError::Apparent(ApparentPlaceError::NonFiniteCorrection {
                stage: "precession"
            })
        ),
        "unexpected error: {err:?}"
    );
}

#[test]
fn apparent_sun_position_propagates_non_finite_input() {
    // As above: precession rejects the NaN input longitude first, so the
    // actual stage is "precession" rather than "apparent-sun-combine".
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let err = apparent_sun_position(instant, fixed(f64::NAN, 0.0, 0.983)).unwrap_err();
    assert!(
        matches!(
            err,
            ApparentPlaceError::NonFiniteCorrection {
                stage: "precession"
            }
        ),
        "unexpected error: {err:?}"
    );
}

#[test]
fn apparent_apsis_position_propagates_non_finite_input() {
    // As above: precession rejects the NaN input longitude first, so the
    // actual stage is "precession" rather than "apparent-apsis-combine".
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let err = apparent_apsis_position(instant, fixed(f64::NAN, 0.0, 0.0025)).unwrap_err();
    assert!(
        matches!(
            err,
            ApparentPlaceError::NonFiniteCorrection {
                stage: "precession"
            }
        ),
        "unexpected error: {err:?}"
    );
}
