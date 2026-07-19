use super::*;

#[test]
fn pinned_checksum() {
    assert_eq!(
        fnv1a64(NUTATION_CSV),
        NUTATION_CSV_CHECKSUM,
        "checksum = {}",
        fnv1a64(NUTATION_CSV)
    );
}

#[test]
fn meeus_example_22a() {
    // Meeus Example 22.a: 1987 April 10, 0h TD -> JDE 2446895.5.
    // Δψ = -3.788", Δε = +9.443", ε0 = 23°26'27.407" = 23.4409463°.
    let n = nutation(2_446_895.5).unwrap();
    assert!(
        (n.delta_psi_arcsec - (-3.788)).abs() < 0.03,
        "Δψ = {}",
        n.delta_psi_arcsec
    );
    assert!(
        (n.delta_eps_arcsec - 9.443).abs() < 0.03,
        "Δε = {}",
        n.delta_eps_arcsec
    );
    let eps0 = mean_obliquity_degrees(2_446_895.5);
    assert!((eps0 - 23.440946).abs() < 1e-5, "ε0 = {eps0}");
}

#[test]
fn j2000_mean_obliquity_matches_anchor() {
    // At J2000 (t=0) the mean obliquity is the anchor constant used elsewhere.
    assert!((mean_obliquity_degrees(2_451_545.0) - 23.439_291_111_111_11).abs() < 1e-9);
}
