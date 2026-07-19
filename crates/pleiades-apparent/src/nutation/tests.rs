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

/// `fundamental_arguments` must reproduce the published Meeus 22.x polynomials
/// exactly. Reference values are an independent evaluation of those published
/// polynomials (NOT captured from this code) at large |t|, where every term —
/// including the cubic — is individually resolvable, so any per-term operator
/// or sign swap moves the result far above the 1e-6° tolerance.
#[test]
fn fundamental_arguments_matches_published_polynomials_large_t() {
    // t = -4.0 (~1600 CE)
    let a = fundamental_arguments(-4.0);
    let expected_m4 = [
        -1_780_770.626_524_977_2,
        -143_638.675_991_466_6,
        -1_908_660.368_594_577_5,
        -1_932_714.857_357_557_4,
        7_861.622_554_577_8,
    ];
    for (i, (got, want)) in a.iter().zip(expected_m4.iter()).enumerate() {
        assert!((got - want).abs() < 1e-6, "t=-4 arg[{i}] = {got}, want {want}");
    }

    // t = +6.0 (~2600 CE)
    let b = fundamental_arguments(6.0);
    let expected_p6 = [
        2_671_900.451_468_798_3,
        216_351.823_269_200_0,
        2_863_328.484_307_199_7,
        2_899_305.245_228_006_0,
        -11_479.698_017_200_0,
    ];
    for (i, (got, want)) in b.iter().zip(expected_p6.iter()).enumerate() {
        assert!((got - want).abs() < 1e-6, "t=+6 arg[{i}] = {got}, want {want}");
    }
}

/// `julian_centuries` is the exact TT-centuries-since-J2000 map. The chosen
/// epochs are exact integer multiples of the Julian century, so the expected
/// values are exact — any operator swap in `(jd - 2451545.0) / 36525.0` diverges.
#[test]
fn julian_centuries_maps_anchor_epochs_exactly() {
    assert_eq!(julian_centuries(2_305_445.0), -4.0);
    assert_eq!(julian_centuries(2_451_545.0), 0.0);
    assert_eq!(julian_centuries(2_670_695.0), 6.0);
}

/// `mean_obliquity_degrees` must reproduce the published Meeus 22.2 polynomial.
/// Reference values are an independent evaluation of that polynomial at large
/// |t|; the J2000 anchor is retained by `j2000_mean_obliquity_matches_anchor`.
#[test]
fn mean_obliquity_matches_published_polynomial_across_range() {
    // jd = 2305445.0 (t = -4)
    assert!(
        (mean_obliquity_degrees(2_305_445.0) - 23.491_272_924_444).abs() < 1e-8,
        "eps(t=-4) = {}",
        mean_obliquity_degrees(2_305_445.0)
    );
    // jd = 2670695.0 (t = +6)
    assert!(
        (mean_obliquity_degrees(2_670_695.0) - 23.361_368_991_111).abs() < 1e-8,
        "eps(t=+6) = {}",
        mean_obliquity_degrees(2_670_695.0)
    );
}

/// `nutation` sums the 19 published IAU-1980 terms. Reference values are an
/// independent evaluation of those same 19 published rows (NOT captured from
/// this code) at large |t|, where the `psi_b * t` / `eps_d * t` rate terms are
/// amplified — so a swapped operator in the accumulation, argument reduction,
/// or the 0.0001 scaling diverges above the 1e-6" tolerance. The Meeus 22.a
/// example (near t=0) is retained by `meeus_example_22a`.
#[test]
fn nutation_series_matches_independent_term_sum_across_range() {
    // jd = 2305445.0 (t = -4)
    let a = nutation(2_305_445.0).unwrap();
    assert!(
        (a.delta_psi_arcsec - 14.651_980_644_9).abs() < 1e-6,
        "Δψ(t=-4) = {}",
        a.delta_psi_arcsec
    );
    assert!(
        (a.delta_eps_arcsec - 4.198_298_187_3).abs() < 1e-6,
        "Δε(t=-4) = {}",
        a.delta_eps_arcsec
    );

    // jd = 2670695.0 (t = +6)
    let b = nutation(2_670_695.0).unwrap();
    assert!(
        (b.delta_psi_arcsec - (-10.499_930_688_4)).abs() < 1e-6,
        "Δψ(t=+6) = {}",
        b.delta_psi_arcsec
    );
    assert!(
        (b.delta_eps_arcsec - 6.406_441_965_2).abs() < 1e-6,
        "Δε(t=+6) = {}",
        b.delta_eps_arcsec
    );
}

#[test]
fn parse_table_accepts_embedded_table() {
    // The wrapper's own input must parse to the full 19-term table.
    let terms = parse_table(NUTATION_CSV, NUTATION_CSV_CHECKSUM).unwrap();
    assert_eq!(terms.len(), 19);
}

#[test]
fn parse_table_rejects_checksum_mismatch() {
    // Any body that does not hash to the expected checksum is refused before parsing.
    let err = parse_table("not the real table", NUTATION_CSV_CHECKSUM).unwrap_err();
    assert!(matches!(
        err,
        ApparentPlaceError::StaleModelData { kind: "nutation" }
    ));
}

#[test]
fn parse_table_rejects_wrong_column_count() {
    // Passes the checksum (computed for this crafted body) but has 3 columns, not 9.
    let body = "h1,h2,h3\n1,2,3";
    let err = parse_table(body, fnv1a64(body)).unwrap_err();
    assert!(matches!(
        err,
        ApparentPlaceError::StaleModelData { kind: "nutation" }
    ));
}

#[test]
fn parse_table_rejects_non_numeric_cell() {
    // Passes the checksum but a data cell is non-numeric.
    let body = "D,M,M1,F,Om,psi_a,psi_b,eps_c,eps_d\n0,0,0,0,1,-171996,-174.2,92025,NOPE";
    let err = parse_table(body, fnv1a64(body)).unwrap_err();
    assert!(matches!(
        err,
        ApparentPlaceError::StaleModelData { kind: "nutation" }
    ));
}
