use super::*;
use pleiades_types::{Instant, JulianDay, Longitude, TimeScale};

fn j2000() -> Instant {
    Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt)
}

#[test]
fn gmst_at_j2000_is_about_280_46_degrees() {
    let gmst = greenwich_mean_sidereal_time_degrees(2_451_545.0);
    // GMST at J2000.0 ≈ 280.4606°.
    assert!(
        (gmst.rem_euclid(360.0) - 280.4606).abs() < 1e-3,
        "got {gmst}"
    );
}

#[test]
fn local_apparent_equals_gast_plus_east_longitude() {
    let st = sidereal_time(j2000(), Longitude::from_degrees(90.0));
    let expected = (st.gast_deg + 90.0).rem_euclid(360.0);
    assert!((st.local_apparent_deg - expected).abs() < 1e-9, "{st:?}");
}

#[test]
fn all_fields_normalized_and_hours_consistent() {
    let st = sidereal_time(j2000(), Longitude::from_degrees(-123.4));
    for v in [
        st.gmst_deg,
        st.gast_deg,
        st.local_mean_deg,
        st.local_apparent_deg,
    ] {
        assert!((0.0..360.0).contains(&v), "not normalized: {v}");
    }
    assert!((st.gmst_hours() - st.gmst_deg / 15.0).abs() < 1e-12);
}

#[test]
fn equation_of_equinoxes_is_small() {
    // EE is at most a couple of arcseconds ≈ a few×1e-4 degrees.
    assert!(equation_of_equinoxes_degrees(2_451_545.0).abs() < 0.01);
}

#[test]
fn equation_of_equinoxes_helper_matches_formula() {
    let delta_psi_deg: f64 = 0.001_234;
    let true_obliquity_deg: f64 = 23.44;
    let expected = delta_psi_deg * true_obliquity_deg.to_radians().cos();
    assert!((equation_of_equinoxes(delta_psi_deg, true_obliquity_deg) - expected).abs() < 1e-15);
}

#[test]
fn equation_of_equinoxes_degrees_uses_shared_helper() {
    // The jd-driven wrapper must equal the helper fed the same nutation inputs.
    let jd = 2_451_545.0;
    let n = crate::nutation::nutation(jd).expect("nutation table available in tests");
    let delta_psi_deg = n.delta_psi_arcsec / 3600.0;
    let true_obl_deg = crate::nutation::mean_obliquity_degrees(jd) + n.delta_eps_arcsec / 3600.0;
    assert!(
        (equation_of_equinoxes_degrees(jd) - equation_of_equinoxes(delta_psi_deg, true_obl_deg))
            .abs()
            < 1e-15
    );
}

#[test]
fn apparent_gmst_matches_pleiades_time_source() {
    for jd in [
        2_415_020.5_f64,
        2_433_283.0,
        2_451_545.0,
        2_469_807.0,
        2_488_069.5,
    ] {
        let apparent = greenwich_mean_sidereal_time_degrees(jd);
        // Un-normalized apparent value must equal the pleiades-time source exactly.
        assert_eq!(apparent, pleiades_time::gmst_degrees_raw(jd), "raw jd {jd}");
        // Reducing to [0,360) must match the normalized public fn.
        assert!(
            (apparent.rem_euclid(360.0) - pleiades_time::gmst_degrees(jd)).abs() < 1e-9,
            "normalized jd {jd}"
        );
    }
}

#[test]
fn sidereal_time_fields_match_independent_recomposition() {
    // Pinned epoch/longitude: the Meeus ch. 12 epoch (JD 2446895.5) and
    // lon = +52.5° east. Non-degeneracy, each property load-bearing
    // (design doc §4.2): lon ≠ 0 separates local from Greenwich fields and
    // `norm(gmst + lon)` from the `-`/`*` mutants; EE ≠ 0 separates mean
    // from apparent fields; no field's value collides with the accessor
    // mutants (deg/15 ∉ {deg%15, deg·15, 0.0, 1.0, −1.0}).
    let jd = 2_446_895.5;
    let lon_deg = 52.5;
    let st = sidereal_time(
        Instant::new(JulianDay::from_days(jd), TimeScale::Ut1),
        Longitude::from_degrees(lon_deg),
    );

    // Expected values recomposed from independently-invoked pieces — never
    // read back from `st`. `Angle::normalized_0_360` is rem_euclid(360.0),
    // and the recomposition mirrors production's operation order, so the
    // comparison is bit-level up to the 1e-12 tolerance.
    let gmst = pleiades_time::gmst_degrees_raw(jd);
    let n = crate::nutation::nutation(jd).expect("nutation table available in tests");
    let ee = equation_of_equinoxes(
        n.delta_psi_arcsec / 3600.0,
        crate::nutation::mean_obliquity_degrees(jd) + n.delta_eps_arcsec / 3600.0,
    );
    let expected_deg = [
        gmst.rem_euclid(360.0),
        (gmst + ee).rem_euclid(360.0),
        (gmst + lon_deg).rem_euclid(360.0),
        (gmst + ee + lon_deg).rem_euclid(360.0),
    ];
    let actual_deg = [
        st.gmst_deg,
        st.gast_deg,
        st.local_mean_deg,
        st.local_apparent_deg,
    ];
    let actual_hours = [
        st.gmst_hours(),
        st.gast_hours(),
        st.local_mean_hours(),
        st.local_apparent_hours(),
    ];
    let field = ["gmst", "gast", "local_mean", "local_apparent"];
    for i in 0..4 {
        assert!(
            (actual_deg[i] - expected_deg[i]).abs() < 1e-12,
            "{}_deg: {} vs {}",
            field[i],
            actual_deg[i],
            expected_deg[i]
        );
        assert!(
            (actual_hours[i] - expected_deg[i] / 15.0).abs() < 1e-12,
            "{}_hours: {} vs {}",
            field[i],
            actual_hours[i],
            expected_deg[i] / 15.0
        );
    }
}

#[test]
fn gast_matches_meeus_example_12b() {
    // External-authority anchor (kills no additional mutant, by design —
    // spec §4.2): Meeus Example 12.b, 1987 April 10, 0h UT (JD 2446895.5),
    // apparent sidereal time = 13h10m46.1351s = 197.692230°. The 1e-4°
    // tolerance covers Meeus's 1980-nutation worked example vs the crate's
    // nutation model (~1e-3″ difference).
    let st = sidereal_time(
        Instant::new(JulianDay::from_days(2_446_895.5), TimeScale::Ut1),
        Longitude::from_degrees(0.0),
    );
    assert!(
        (st.gast_deg - 197.692_230).abs() < 1e-4,
        "gast {}",
        st.gast_deg
    );
}
