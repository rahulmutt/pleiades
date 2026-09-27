use super::*;
use std::cell::Cell;
use std::f64::consts::TAU;

const T0: f64 = 2_451_545.0;
const HOUR: f64 = 1.0 / 24.0;
/// Root tolerance: 1 s, twice the bisection's own 0.5 s ceiling.
const TOL: f64 = 1.0 / 86_400.0;

/// A diurnal altitude-like residual (degrees): 60° amplitude, 1-day period,
/// crossing zero where `sin = 0.3`. Ascending root at `k + a()`, descending at
/// `k + 0.5 − a()`.
fn diurnal(t: f64) -> f64 {
    60.0 * (TAU * (t - T0)).sin() - 18.0
}
/// `asin(0.3) / TAU`: the ascending root's offset into each day of `diurnal`.
fn a() -> f64 {
    0.3_f64.asin() / TAU
}

/// A graze: the same diurnal curve, shifted so its daily peak (at `T0 + 0.25`)
/// pokes `60·EPS ≈ 0.057°` above zero for ~20 minutes — shorter than the
/// 1-hour grid step used by every test here.
const EPS: f64 = 9.5e-4;
fn graze(t: f64) -> f64 {
    60.0 * (TAU * (t - T0)).sin() - 60.0 * (1.0 - EPS)
}
/// Its roots: `sin(TAU x) = 1 − EPS` → `x = asin(1 − EPS) / TAU` (ascending)
/// and `0.5 − x` (descending), i.e. `0.25 ∓ τ`.
fn graze_tau() -> f64 {
    0.25 - (1.0 - EPS).asin() / TAU
}

fn ok<F: Fn(f64) -> f64>(f: F) -> impl FnMut(f64) -> Result<f64, EventError> {
    move |t| Ok(f(t))
}

#[test]
fn range_returns_ascending_roots_in_order() {
    let roots = horizon_crossings_in_range(ok(diurnal), T0, T0 + 3.0, HOUR, true).unwrap();
    assert_eq!(roots.len(), 3, "roots {roots:?}");
    for (k, r) in roots.iter().enumerate() {
        let want = T0 + k as f64 + a();
        assert!((r - want).abs() < TOL, "root {k}: {r} vs {want}");
    }
}

#[test]
fn range_returns_descending_roots_in_order() {
    let roots = horizon_crossings_in_range(ok(diurnal), T0, T0 + 3.0, HOUR, false).unwrap();
    assert_eq!(roots.len(), 3, "roots {roots:?}");
    for (k, r) in roots.iter().enumerate() {
        let want = T0 + k as f64 + 0.5 - a();
        assert!((r - want).abs() < TOL, "root {k}: {r} vs {want}");
    }
}

#[test]
fn first_after_skips_a_wrong_direction_crossing() {
    // The ascending root at T0 + A comes first; asking for descending must
    // skip it and land on T0 + 0.5 − A.
    let root = first_horizon_crossing_after(ok(diurnal), T0, T0 + 3.0, HOUR, false)
        .unwrap()
        .expect("a descending root");
    let want = T0 + 0.5 - a();
    assert!((root - want).abs() < TOL, "{root} vs {want}");
}

#[test]
fn first_after_returns_none_without_a_root_in_range() {
    let none = first_horizon_crossing_after(ok(|_| 90.0), T0, T0 + 3.0, HOUR, true).unwrap();
    assert!(none.is_none());
    // A range that ends before the first root.
    let none = first_horizon_crossing_after(ok(diurnal), T0, T0 + 0.01, HOUR, true).unwrap();
    assert!(none.is_none());
}

#[test]
fn last_before_matches_range_last_for_misaligned_ranges() {
    for (i, offset) in [0.0, 0.137, 1.0 / 3.0, 0.061, 0.999].iter().enumerate() {
        let lo = T0 + 0.3 * HOUR * i as f64;
        let hi = lo + 2.3 + offset;
        for want_ascending in [true, false] {
            let expected = horizon_crossings_in_range(ok(diurnal), lo, hi, HOUR, want_ascending)
                .unwrap()
                .last()
                .copied()
                .expect("range has roots");
            let actual = last_horizon_crossing_before(ok(diurnal), lo, hi, HOUR, want_ascending)
                .unwrap()
                .expect("a last root");
            assert!(
                (expected - actual).abs() < TOL,
                "offset {offset} ascending {want_ascending}: {expected} vs {actual}"
            );
        }
    }
}

#[test]
fn last_before_returns_none_without_a_root_in_range() {
    let none = last_horizon_crossing_before(ok(|_| -5.0), T0, T0 + 3.0, HOUR, false).unwrap();
    assert!(none.is_none());
    // A range that ends before the first root of the wanted direction.
    let none = last_horizon_crossing_before(ok(diurnal), T0, T0 + 0.2, HOUR, false).unwrap();
    assert!(none.is_none());
}

#[test]
fn roots_in_the_overshoot_sample_are_dropped() {
    // Ascending linear residual with its root at 3.5 h; the range ends at
    // 3.2 h. The grid's last sample (4 h) overshoots `hi` and brackets the
    // root, which must still be rejected as out of range.
    let f = |t: f64| 100.0 * (t - (T0 + 3.5 * HOUR));
    let hi = T0 + 3.2 * HOUR;
    assert!(horizon_crossings_in_range(ok(f), T0, hi, HOUR, true)
        .unwrap()
        .is_empty());
    assert!(first_horizon_crossing_after(ok(f), T0, hi, HOUR, true)
        .unwrap()
        .is_none());
    assert!(last_horizon_crossing_before(ok(f), T0, hi, HOUR, true)
        .unwrap()
        .is_none());
    // Mirror: root at −0.5 h, range starting at −0.2 h.
    let g = |t: f64| 100.0 * (t - (T0 - 0.5 * HOUR));
    let lo = T0 - 0.2 * HOUR;
    assert!(
        last_horizon_crossing_before(ok(g), lo, T0 + 3.0 * HOUR, HOUR, true)
            .unwrap()
            .is_none()
    );
}

#[test]
fn graze_peak_between_samples_yields_rise_then_set() {
    // Grid anchored 18 min past T0, so no sample lands inside the ±10 min
    // above-zero window around the peak at T0 + 0.25.
    let lo = T0 + 0.3 * HOUR;
    let hi = T0 + 1.0;
    let tau = graze_tau();
    let rises = horizon_crossings_in_range(ok(graze), lo, hi, HOUR, true).unwrap();
    let sets = horizon_crossings_in_range(ok(graze), lo, hi, HOUR, false).unwrap();
    assert_eq!(rises.len(), 1, "rises {rises:?}");
    assert_eq!(sets.len(), 1, "sets {sets:?}");
    assert!(
        (rises[0] - (T0 + 0.25 - tau)).abs() < TOL,
        "rise {}",
        rises[0]
    );
    assert!((sets[0] - (T0 + 0.25 + tau)).abs() < TOL, "set {}", sets[0]);
    assert!(rises[0] < sets[0]);
}

#[test]
fn graze_is_found_by_first_after_and_last_before() {
    let lo = T0 + 0.3 * HOUR;
    let hi = T0 + 1.0;
    let tau = graze_tau();
    let rise = first_horizon_crossing_after(ok(graze), lo, hi, HOUR, true)
        .unwrap()
        .expect("rise");
    let set = first_horizon_crossing_after(ok(graze), lo, hi, HOUR, false)
        .unwrap()
        .expect("set");
    assert!((rise - (T0 + 0.25 - tau)).abs() < TOL);
    assert!((set - (T0 + 0.25 + tau)).abs() < TOL);
    let rise = last_horizon_crossing_before(ok(graze), lo, hi, HOUR, true)
        .unwrap()
        .expect("rise");
    let set = last_horizon_crossing_before(ok(graze), lo, hi, HOUR, false)
        .unwrap()
        .expect("set");
    assert!((rise - (T0 + 0.25 - tau)).abs() < TOL);
    assert!((set - (T0 + 0.25 + tau)).abs() < TOL);
}

#[test]
fn near_miss_peak_below_zero_yields_nothing() {
    // Same curve with the peak 0.057° BELOW zero: a discrete local maximum
    // near the horizon that must not be mistaken for a crossing.
    let miss = |t: f64| 60.0 * (TAU * (t - T0)).sin() - 60.0 * (1.0 + EPS);
    let lo = T0 + 0.3 * HOUR;
    let hi = T0 + 1.0;
    for want in [true, false] {
        assert!(horizon_crossings_in_range(ok(miss), lo, hi, HOUR, want)
            .unwrap()
            .is_empty());
        assert!(first_horizon_crossing_after(ok(miss), lo, hi, HOUR, want)
            .unwrap()
            .is_none());
        assert!(last_horizon_crossing_before(ok(miss), lo, hi, HOUR, want)
            .unwrap()
            .is_none());
    }
}

#[test]
fn dip_between_samples_yields_set_then_rise() {
    // The graze curve mirrored: a body that is up all day except for a
    // ~20-minute dip below the horizon between two grid samples.
    let dip = |t: f64| -graze(t);
    let lo = T0 + 0.3 * HOUR;
    let hi = T0 + 1.0;
    let tau = graze_tau();
    let rises = horizon_crossings_in_range(ok(dip), lo, hi, HOUR, true).unwrap();
    let sets = horizon_crossings_in_range(ok(dip), lo, hi, HOUR, false).unwrap();
    assert_eq!(sets.len(), 1, "sets {sets:?}");
    assert_eq!(rises.len(), 1, "rises {rises:?}");
    assert!((sets[0] - (T0 + 0.25 - tau)).abs() < TOL, "set {}", sets[0]);
    assert!(
        (rises[0] - (T0 + 0.25 + tau)).abs() < TOL,
        "rise {}",
        rises[0]
    );
}

#[test]
fn evaluation_count_stays_coarse() {
    // The whole point of the scanner: a root ~23.5 h away costs on the order
    // of one sample per hour plus one bisection, not one sample per 2 min.
    let calls = Cell::new(0usize);
    let counted = |t: f64| {
        calls.set(calls.get() + 1);
        Ok(diurnal(t))
    };
    // Start just after the day-0 ascending root; the next one is ~24 h away.
    let after = T0 + a() + 0.5 * HOUR;
    let root = first_horizon_crossing_after(counted, after, after + 3.0, HOUR, true)
        .unwrap()
        .expect("next ascending root");
    assert!((root - (T0 + 1.0 + a())).abs() < TOL);
    assert!(
        calls.get() < 60,
        "expected ~24 grid + ~13 bisection evaluations, got {}",
        calls.get()
    );
}

#[test]
fn errors_propagate_from_all_entry_points() {
    let boom = |_: f64| Err(EventError::Backend("boom".into()));
    assert!(matches!(
        horizon_crossings_in_range(boom, T0, T0 + 1.0, HOUR, true).unwrap_err(),
        EventError::Backend(_)
    ));
    assert!(matches!(
        first_horizon_crossing_after(boom, T0, T0 + 1.0, HOUR, true).unwrap_err(),
        EventError::Backend(_)
    ));
    assert!(matches!(
        last_horizon_crossing_before(boom, T0, T0 + 1.0, HOUR, true).unwrap_err(),
        EventError::Backend(_)
    ));
}
