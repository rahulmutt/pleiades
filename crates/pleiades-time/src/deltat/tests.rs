use super::*;

#[test]
fn pinned_checksum() {
    assert_eq!(
        fnv1a64(DELTA_T_CSV),
        DELTA_T_CSV_CHECKSUM,
        "checksum = {}",
        fnv1a64(DELTA_T_CSV)
    );
}

#[test]
fn observed_spot_values() {
    // 2000-01-01 12:00 -> node 2000 -> 63.8, Observed
    let (dt, q) = delta_t(2451545.0).unwrap();
    assert!((dt - 63.8).abs() < 0.5, "got {dt}");
    assert_eq!(q, DeltaTQuality::Observed);
    // 1900 node -> -2.8
    let (dt, _) = delta_t(2415020.5).unwrap();
    assert!((dt - (-2.8)).abs() < 0.5, "got {dt}");
}

#[test]
fn boundary_at_observed_through_jd() {
    assert_eq!(
        delta_t(OBSERVED_THROUGH_JD).unwrap().1,
        DeltaTQuality::Predicted
    );
    assert_eq!(
        delta_t(OBSERVED_THROUGH_JD - 1.0).unwrap().1,
        DeltaTQuality::Observed
    );
}

#[test]
fn future_is_predicted() {
    // 2080-ish: past the 2020 observed node -> Predicted
    let (dt, q) = delta_t(2480000.0).unwrap();
    assert_eq!(q, DeltaTQuality::Predicted);
    assert!(dt > 69.0, "got {dt}");
}

#[test]
fn extrapolated_delta_t_matches_published_polynomial() {
    // JD 2480765.0 = 2451545 + 365.25 * 80 exactly (representable), so
    // decimal_year is exactly 2080.0 and t = 80. Espenak-Meeus 2005-2050
    // polynomial evaluated outside the code (see the design doc's
    // Appendix script): 62.92 + 0.32217*80 + 0.005589*80*80 = 124.4632.
    // Smallest mutant displacement at t = 80 is 25.77 s (spec §4.2), a
    // ~2.6e10x margin over the 1e-9 s tolerance.
    let (dt, q) = delta_t(2_480_765.0).unwrap();
    assert_eq!(q, DeltaTQuality::Predicted);
    assert!((dt - 124.4632).abs() < 1e-9, "got {dt}");
}
