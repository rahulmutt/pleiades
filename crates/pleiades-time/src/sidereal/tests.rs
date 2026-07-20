use super::*;

#[test]
fn gmst_matches_meeus_example_12a() {
    // Meeus Example 12.a: 1987 April 10, 0h UT -> JD 2446895.5,
    // GMST = 13h10m46.3668s = 197.693195 deg.
    let gmst = gmst_degrees(2_446_895.5);
    assert!((gmst - 197.693_195).abs() < 1e-4, "gmst {gmst}");
}

#[test]
fn gmst_is_normalized() {
    for jd in [2_415_020.5_f64, 2_451_545.0, 2_488_069.5] {
        let g = gmst_degrees(jd);
        assert!(
            (0.0..360.0).contains(&g),
            "gmst {g} out of range at jd {jd}"
        );
    }
}

#[test]
fn gmst_raw_matches_meeus_12_4_at_large_t() {
    // Meeus eq. 12.4 evaluated OUTSIDE the code (double precision, same
    // published coefficients) at t = ±4 Julian centuries from J2000
    // (JD 2451545 ± 4·36525). Large |t| makes the quadratic (~6.2e-3°) and
    // cubic (~1.65e-6°) terms visible; the ± pair separates the even
    // quadratic term (same sign at ±t) from the odd cubic term (flips sign).
    // Tolerance 2e-7° is ~27 ulp of the ~5.27e7° raw value — ≥5× below the
    // smallest surviving-mutant displacement (~1.14e-6°) and ≥25× above
    // last-ulp evaluation noise (margins verified in the slice design doc §4.1).
    assert!(
        (gmst_degrees_raw(2_597_645.0) - 52_740_283.547_038_615).abs() < 2e-7,
        "t=+4: {}",
        gmst_degrees_raw(2_597_645.0)
    );
    assert!(
        (gmst_degrees_raw(2_305_445.0) - (-52_739_722.613_388_02)).abs() < 2e-7,
        "t=-4: {}",
        gmst_degrees_raw(2_305_445.0)
    );
}

#[test]
fn gmst_normalized_matches_raw_at_large_t() {
    // Pins the normalized path to the raw polynomial at the new ±4-century
    // epochs (redundant kill — `gmst_degrees`'s own mutants are already
    // caught; this keeps raw and normalized coupled at the epochs the
    // literal test above relies on).
    for jd in [2_597_645.0_f64, 2_305_445.0] {
        assert!(
            (gmst_degrees(jd) - gmst_degrees_raw(jd).rem_euclid(360.0)).abs() < 1e-9,
            "jd {jd}"
        );
    }
}
