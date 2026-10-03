use super::*;
use crate::leap;

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
    // The 2020 node itself is the first day served by the leap-second bound.
    assert_eq!(
        delta_t(OBSERVED_THROUGH_JD).unwrap().1,
        DeltaTQuality::LeapSecondBound
    );
    assert_eq!(
        delta_t(OBSERVED_THROUGH_JD - 1.0).unwrap().1,
        DeltaTQuality::Observed
    );
}

#[test]
fn leap_second_bound_is_tt_minus_utc() {
    // 2022-01-01 00:00 (JD 2459580.5): TAI − UTC = 37 s since 2017, so
    // ΔT = 32.184 + 37 = 69.184 s exactly (DUT1 taken as zero).
    let (dt, q) = delta_t(2_459_580.5).unwrap();
    assert_eq!(q, DeltaTQuality::LeapSecondBound);
    assert!((dt - 69.184).abs() < 1e-12, "got {dt}");
}

#[test]
fn boundary_at_leap_horizon() {
    // The horizon is exclusive: the day before is leap-bound, the horizon
    // itself is the first Predicted instant.
    let (before, q_before) = delta_t(leap::VALID_THROUGH_JD - 1.0).unwrap();
    let (at, q_at) = delta_t(leap::VALID_THROUGH_JD).unwrap();
    let (past, q_past) = delta_t(leap::VALID_THROUGH_JD + 1.0).unwrap();
    assert_eq!(q_before, DeltaTQuality::LeapSecondBound);
    assert_eq!(q_at, DeltaTQuality::Predicted);
    assert_eq!(q_past, DeltaTQuality::Predicted);
    // Anchored extrapolation: ΔT at the horizon is the leap-second bound
    // itself, and the polynomial's slope at 2027 is ≈0.63 s/yr, so one day
    // past the horizon moves ΔT by ≈0.0017 s, not by the ≈6.8 s jump the
    // unanchored polynomial would produce.
    assert!((before - 69.184).abs() < 1e-12, "before {before}");
    assert!((at - 69.184).abs() < 1e-12, "at {at}");
    assert!((past - at).abs() < 0.01, "at {at}, past {past}");
    assert!(past > at, "at {at}, past {past}");
}

#[test]
fn future_is_predicted() {
    // 2080-ish: past the leap horizon -> Predicted
    let (dt, q) = delta_t(2480000.0).unwrap();
    assert_eq!(q, DeltaTQuality::Predicted);
    assert!(dt > 69.0, "got {dt}");
}

#[test]
fn extrapolated_delta_t_is_the_anchored_published_polynomial() {
    // JD 2480765.0 = 2451545 + 365.25 * 80 exactly (representable), so
    // decimal_year is exactly 2080.0 and t = 80. Espenak-Meeus 2005-2050
    // polynomial P(t) = 62.92 + 0.32217 t + 0.005589 t²; P(80) = 124.4632.
    // The extrapolation is anchored at the leap horizon (2027-07-01, decimal
    // year 2027.494866529774, P = 76.00312454410142) to the leap-bound
    // value 69.184, so ΔT(2080) = 69.184 + 124.4632 − 76.00312454410142
    // = 117.64407545589857 (evaluated outside the code with Python). Dropping
    // the anchor displaces this by 6.82 s, dropping either polynomial term by
    // 20 s or more, so the 1e-6 s tolerance leaves a >1e6x margin.
    let (dt, q) = delta_t(2_480_765.0).unwrap();
    assert_eq!(q, DeltaTQuality::Predicted);
    assert!((dt - 117.644_075_456).abs() < 1e-6, "got {dt}");
}
