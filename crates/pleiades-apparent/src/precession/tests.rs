use super::*;

#[test]
fn date_to_j2000_is_the_inverse_of_j2000_to_date() {
    // Round-trip a non-trivial point at a far epoch: J2000 -> date -> J2000.
    let jd = 2_415_025.5; // 1900
    let to_date = precess_ecliptic_j2000_to_date(123.456, 4.5, jd).unwrap();
    let back =
        precess_ecliptic_date_to_j2000(to_date.longitude_deg, to_date.latitude_deg, jd).unwrap();
    assert!(
        (back.longitude_deg - 123.456).abs() < 1e-6,
        "λ round-trip {}",
        back.longitude_deg
    );
    assert!(
        (back.latitude_deg - 4.5).abs() < 1e-6,
        "β round-trip {}",
        back.latitude_deg
    );
}

#[test]
fn identity_at_j2000() {
    // At J2000 the precession angles are zero and the inbound/outbound
    // obliquities are equal, so the transform is the identity.
    let out = precess_ecliptic_j2000_to_date(123.456, 4.5, 2_451_545.0).unwrap();
    assert!(
        (out.longitude_deg - 123.456).abs() < 1e-6,
        "λ = {}",
        out.longitude_deg
    );
    assert!(
        (out.latitude_deg - 4.5).abs() < 1e-6,
        "β = {}",
        out.latitude_deg
    );
}

#[test]
fn general_precession_one_century() {
    // The J2000 vernal-equinox direction (λ=0, β=0) viewed in the
    // equinox-of-date frame one Julian century on has longitude ≈ the general
    // precession in longitude (5029.0966″/cy = 1.39697°). β stays small but
    // NOT exactly zero: the ecliptic plane itself precesses (~47″/cy), so a
    // point in the J2000 ecliptic acquires ≈ +4.4″ (0.00122°) of ecliptic-of-
    // date latitude. This is physically real and matches the rigorous Meeus
    // ch.21 ecliptic-precession result (4.39″) to sub-mas; the bound below is
    // widened from the naive 1e-3° to admit that residual while still catching
    // gross errors (a transcription bug would produce degrees, not arcsec).
    let jd = 2_451_545.0 + 36_525.0;
    let out = precess_ecliptic_j2000_to_date(0.0, 0.0, jd).unwrap();
    assert!(
        (out.longitude_deg - 1.39697).abs() < 5e-3,
        "λ' = {}",
        out.longitude_deg
    );
    assert!(out.latitude_deg.abs() < 2e-3, "β' = {}", out.latitude_deg);
}

#[test]
fn longitude_shifts_by_precession_off_the_ecliptic() {
    // For an off-ecliptic point, longitude still shifts by ≈ the general
    // precession over a century; latitude moves only slightly (ecliptic motion).
    let jd = 2_451_545.0 + 36_525.0;
    let out = precess_ecliptic_j2000_to_date(80.0, 30.0, jd).unwrap();
    let dlon = out.longitude_deg - 80.0;
    assert!((dlon - 1.397).abs() < 0.05, "Δλ = {dlon}");
    assert!(
        (out.latitude_deg - 30.0).abs() < 0.05,
        "β' = {}",
        out.latitude_deg
    );
}

#[test]
fn date_to_j2000_matches_independent_literals_at_pm4_centuries() {
    // Expected values computed OUTSIDE this crate by an independent Python
    // implementation of the published pipeline (Meeus 20.3 precession
    // angles, 21.4 equatorial rotation, 13.x ecliptic<->equatorial bridges,
    // 22.2 mean obliquity), cross-validated against the genuinely different
    // Meeus 21.5 elements + 21.7 direct-ecliptic rotation (agreement
    // ~3e-3″ at t = ±4). See the 2026-07-21 FU-9 slice design doc §6.
    // Epochs sit at t = ±4 Julian centuries (≈ years 2400/1600, inside the
    // 1600–2600 coverage target), far from the |t| ≈ 1 degeneracy where
    // `t*t ≈ t/t` hid the quadratic/cubic `*` -> `/` mutants from the 1900
    // round-trip. Smallest mutant displacement at this geometry: 0.961″ =
    // 2.67e-4°, ~2.7e5× the tolerance below.
    const TOL_DEG: f64 = 1e-9;
    let cases = [
        // (jd_tt, expected λ_J2000, expected β_J2000); inputs are mean-of-date.
        (2_597_645.0, 117.860_897_668_741, 4.456_799_466_404), // t = +4
        (2_305_445.0, 129.041_779_511_373, 4.538_180_014_018), // t = -4
    ];
    for (jd, exp_lon, exp_lat) in cases {
        let out = precess_ecliptic_date_to_j2000(123.456, 4.5, jd).unwrap();
        assert!(
            (out.longitude_deg - exp_lon).abs() < TOL_DEG,
            "λ at jd {jd}: {} vs {exp_lon}",
            out.longitude_deg
        );
        assert!(
            (out.latitude_deg - exp_lat).abs() < TOL_DEG,
            "β at jd {jd}: {} vs {exp_lat}",
            out.latitude_deg
        );
    }
}

#[test]
fn date_to_j2000_identity_at_j2000() {
    // Inverse-direction mirror of `identity_at_j2000` — a genuine intent
    // gap: the inverse's identity property was never asserted. At t = 0 the
    // precession angles vanish and the of-date obliquity equals ε₀, so the
    // inverse transform is also the identity. (Redundant mutant kill: at
    // t = 0 every quadratic/cubic `*` -> `/` mutant divides by zero,
    // producing NaN and a NonFiniteCorrection error instead of Ok.)
    let out = precess_ecliptic_date_to_j2000(123.456, 4.5, 2_451_545.0).unwrap();
    assert!(
        (out.longitude_deg - 123.456).abs() < 1e-9,
        "λ = {}",
        out.longitude_deg
    );
    assert!(
        (out.latitude_deg - 4.5).abs() < 1e-9,
        "β = {}",
        out.latitude_deg
    );
}

#[test]
fn overflow_epoch_fails_closed_in_both_directions() {
    // jd_tt = 7.0e107 puts t ≈ 1.92e103 in the window where θ's cubic term
    // overflows to -inf while ζ, z, and the mean obliquity stay finite (θ's
    // 0.041833 coefficient is the largest of the cubics, so it overflows
    // first as |t| grows). sin/cos(±inf) = NaN then poisons BOTH the b
    // (→ longitude) and c (→ latitude) rotation terms, so both outputs go
    // non-finite together and the guard fails closed. This shared poisoning
    // is also why the `||` -> `&&` guard mutants are documented equivalents:
    // no reachable input makes exactly one output non-finite (design doc §5).
    let err = precess_ecliptic_date_to_j2000(123.456, 4.5, 7.0e107).unwrap_err();
    assert!(
        matches!(
            err,
            ApparentPlaceError::NonFiniteCorrection {
                stage: "precession"
            }
        ),
        "date->J2000: expected NonFiniteCorrection, got {err:?}"
    );
    let err = precess_ecliptic_j2000_to_date(123.456, 4.5, 7.0e107).unwrap_err();
    assert!(
        matches!(
            err,
            ApparentPlaceError::NonFiniteCorrection {
                stage: "precession"
            }
        ),
        "J2000->date: expected NonFiniteCorrection, got {err:?}"
    );
}

#[test]
fn vector_precession_preserves_magnitude_and_matches_the_angular_form() {
    // A Moon-scale vector well off the ecliptic, one century from J2000.
    let v = [0.002_1, -0.001_3, 0.000_2];
    let jd = 2_488_070.0;
    let out = precess_ecliptic_vector_j2000_to_date(v, jd).unwrap();
    let r_in = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let r_out = (out[0] * out[0] + out[1] * out[1] + out[2] * out[2]).sqrt();
    assert!((r_in - r_out).abs() < 1e-15, "magnitude {r_in} -> {r_out}");
    let lon = v[1].atan2(v[0]).to_degrees().rem_euclid(360.0);
    let lat = (v[2] / r_in).asin().to_degrees();
    let p = precess_ecliptic_j2000_to_date(lon, lat, jd).unwrap();
    let lon_out = out[1].atan2(out[0]).to_degrees().rem_euclid(360.0);
    let lat_out = (out[2] / r_out).asin().to_degrees();
    assert!(
        (lon_out - p.longitude_deg).abs() < 1e-9,
        "{lon_out} vs {}",
        p.longitude_deg
    );
    assert!(
        (lat_out - p.latitude_deg).abs() < 1e-9,
        "{lat_out} vs {}",
        p.latitude_deg
    );
    // Precession turns the vector by ~1.4 deg per century, so it must move.
    assert!((lon_out - lon).abs() > 1.0);
}

#[test]
fn vector_precession_is_a_rotation_so_it_commutes_with_scaling() {
    // The same map must serve position and velocity vectors: f(k v) = k f(v).
    let v = [-0.000_4, 0.000_35, 0.000_05];
    let jd = 2_415_020.5;
    let a = precess_ecliptic_vector_j2000_to_date(v, jd).unwrap();
    let b = precess_ecliptic_vector_j2000_to_date([v[0] * 250.0, v[1] * 250.0, v[2] * 250.0], jd)
        .unwrap();
    for i in 0..3 {
        assert!(
            (a[i] * 250.0 - b[i]).abs() < 1e-12,
            "component {i}: {} vs {}",
            a[i] * 250.0,
            b[i]
        );
    }
}

#[test]
fn vector_precession_leaves_the_zero_vector_alone_and_rejects_non_finite() {
    assert_eq!(
        precess_ecliptic_vector_j2000_to_date([0.0; 3], 2_451_545.0).unwrap(),
        [0.0; 3]
    );
    let err = precess_ecliptic_vector_j2000_to_date([f64::NAN, 0.0, 0.0], 2_451_545.0).unwrap_err();
    assert!(matches!(
        err,
        ApparentPlaceError::NonFiniteCorrection {
            stage: "precession"
        }
    ));
}
