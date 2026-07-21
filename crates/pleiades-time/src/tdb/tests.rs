use super::*;

#[test]
fn bounded_below_two_milliseconds() {
    for offset in [-36525.0, -18262.0, 0.0, 18262.0, 36525.0] {
        let v = tdb_minus_tt_seconds(2451545.0 + offset);
        assert!(
            v.abs() < 0.002,
            "TDB-TT {v} out of bound at offset {offset}"
        );
    }
}

#[test]
fn tdb_minus_tt_matches_usno_formula_at_pinned_epochs() {
    // USNO two-term model evaluated outside the code with matching
    // operation order (design doc Appendix script). The output is bounded
    // below 2 ms by construction for ANY g, so only value-pinning can
    // constrain the phase; each epoch kills all 9 baseline survivors
    // alone (smallest displacement 2.04e-6 s vs the 1e-9 s tolerance,
    // which in turn sits far above cross-libm sin differences of ~1 ulp).
    let v1 = tdb_minus_tt_seconds(2_451_645.0);
    assert!((v1 - 0.001_645_689_159_645_154_8).abs() < 1e-9, "got {v1}");
    let v2 = tdb_minus_tt_seconds(2_446_895.5);
    assert!((v2 - 0.001_649_315_495_175_222_8).abs() < 1e-9, "got {v2}");
}
