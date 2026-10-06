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
    let roots =
        directed_crossings_in_range(ok(diurnal), T0, T0 + 3.0, HOUR, Limits::NONE, true).unwrap();
    assert_eq!(roots.len(), 3, "roots {roots:?}");
    for (k, r) in roots.iter().enumerate() {
        let want = T0 + k as f64 + a();
        assert!((r - want).abs() < TOL, "root {k}: {r} vs {want}");
    }
}

#[test]
fn range_returns_descending_roots_in_order() {
    let roots =
        directed_crossings_in_range(ok(diurnal), T0, T0 + 3.0, HOUR, Limits::NONE, false).unwrap();
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
    let root = first_directed_crossing_after(ok(diurnal), T0, T0 + 3.0, HOUR, Limits::NONE, false)
        .unwrap()
        .expect("a descending root");
    let want = T0 + 0.5 - a();
    assert!((root - want).abs() < TOL, "{root} vs {want}");
}

#[test]
fn first_after_returns_none_without_a_root_in_range() {
    let none = first_directed_crossing_after(ok(|_| 90.0), T0, T0 + 3.0, HOUR, Limits::NONE, true)
        .unwrap();
    assert!(none.is_none());
    // A range that ends before the first root.
    let none = first_directed_crossing_after(ok(diurnal), T0, T0 + 0.01, HOUR, Limits::NONE, true)
        .unwrap();
    assert!(none.is_none());
}

#[test]
fn last_before_matches_range_last_for_misaligned_ranges() {
    for (i, offset) in [0.0, 0.137, 1.0 / 3.0, 0.061, 0.999].iter().enumerate() {
        let lo = T0 + 0.3 * HOUR * i as f64;
        let hi = lo + 2.3 + offset;
        for want_ascending in [true, false] {
            let expected = directed_crossings_in_range(
                ok(diurnal),
                lo,
                hi,
                HOUR,
                Limits::NONE,
                want_ascending,
            )
            .unwrap()
            .last()
            .copied()
            .expect("range has roots");
            let actual = last_directed_crossing_before(
                ok(diurnal),
                lo,
                hi,
                HOUR,
                Limits::NONE,
                want_ascending,
            )
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
    let none = last_directed_crossing_before(ok(|_| -5.0), T0, T0 + 3.0, HOUR, Limits::NONE, false)
        .unwrap();
    assert!(none.is_none());
    // A range that ends before the first root of the wanted direction.
    let none = last_directed_crossing_before(ok(diurnal), T0, T0 + 0.2, HOUR, Limits::NONE, false)
        .unwrap();
    assert!(none.is_none());
}

#[test]
fn roots_in_the_overshoot_sample_are_dropped() {
    // Ascending linear residual with its root at 3.5 h; the range ends at
    // 3.2 h. The grid's last sample (4 h) overshoots `hi` and brackets the
    // root, which must still be rejected as out of range.
    let f = |t: f64| 100.0 * (t - (T0 + 3.5 * HOUR));
    let hi = T0 + 3.2 * HOUR;
    assert!(
        directed_crossings_in_range(ok(f), T0, hi, HOUR, Limits::NONE, true)
            .unwrap()
            .is_empty()
    );
    assert!(
        first_directed_crossing_after(ok(f), T0, hi, HOUR, Limits::NONE, true)
            .unwrap()
            .is_none()
    );
    assert!(
        last_directed_crossing_before(ok(f), T0, hi, HOUR, Limits::NONE, true)
            .unwrap()
            .is_none()
    );
    // Mirror: root at −0.5 h, range starting at −0.2 h.
    let g = |t: f64| 100.0 * (t - (T0 - 0.5 * HOUR));
    let lo = T0 - 0.2 * HOUR;
    assert!(
        last_directed_crossing_before(ok(g), lo, T0 + 3.0 * HOUR, HOUR, Limits::NONE, true)
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
    let rises = directed_crossings_in_range(ok(graze), lo, hi, HOUR, Limits::NONE, true).unwrap();
    let sets = directed_crossings_in_range(ok(graze), lo, hi, HOUR, Limits::NONE, false).unwrap();
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
    let rise = first_directed_crossing_after(ok(graze), lo, hi, HOUR, Limits::NONE, true)
        .unwrap()
        .expect("rise");
    let set = first_directed_crossing_after(ok(graze), lo, hi, HOUR, Limits::NONE, false)
        .unwrap()
        .expect("set");
    assert!((rise - (T0 + 0.25 - tau)).abs() < TOL);
    assert!((set - (T0 + 0.25 + tau)).abs() < TOL);
    let rise = last_directed_crossing_before(ok(graze), lo, hi, HOUR, Limits::NONE, true)
        .unwrap()
        .expect("rise");
    let set = last_directed_crossing_before(ok(graze), lo, hi, HOUR, Limits::NONE, false)
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
        assert!(
            directed_crossings_in_range(ok(miss), lo, hi, HOUR, Limits::NONE, want)
                .unwrap()
                .is_empty()
        );
        assert!(
            first_directed_crossing_after(ok(miss), lo, hi, HOUR, Limits::NONE, want)
                .unwrap()
                .is_none()
        );
        assert!(
            last_directed_crossing_before(ok(miss), lo, hi, HOUR, Limits::NONE, want)
                .unwrap()
                .is_none()
        );
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
    let rises = directed_crossings_in_range(ok(dip), lo, hi, HOUR, Limits::NONE, true).unwrap();
    let sets = directed_crossings_in_range(ok(dip), lo, hi, HOUR, Limits::NONE, false).unwrap();
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
    let root = first_directed_crossing_after(counted, after, after + 3.0, HOUR, Limits::NONE, true)
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
        directed_crossings_in_range(boom, T0, T0 + 1.0, HOUR, Limits::NONE, true).unwrap_err(),
        EventError::Backend(_)
    ));
    assert!(matches!(
        first_directed_crossing_after(boom, T0, T0 + 1.0, HOUR, Limits::NONE, true).unwrap_err(),
        EventError::Backend(_)
    ));
    assert!(matches!(
        last_directed_crossing_before(boom, T0, T0 + 1.0, HOUR, Limits::NONE, true).unwrap_err(),
        EventError::Backend(_)
    ));
}

// ---- Issues #80 and #81: searches chained from a returned instant, and
// ---- short nights (or days) at either end of the scanned range.

const MINUTE: f64 = 1.0 / 1_440.0;

/// `graze` mirrored: up all day except a ~20-minute night centred on
/// `T0 + 0.25`, setting at `dip_set()` and rising at `dip_rise()`.
fn dip(t: f64) -> f64 {
    -graze(t)
}
fn dip_set() -> f64 {
    T0 + 0.25 - graze_tau()
}
fn dip_rise() -> f64 {
    T0 + 0.25 + graze_tau()
}

/// Search starts spread over a day, so the returned roots fall on both
/// halves of the final bisection bracket.
fn spread_starts() -> impl Iterator<Item = f64> {
    (0..48).map(|i| T0 + 0.021_3 * f64::from(i))
}

#[test]
fn first_after_a_returned_root_finds_the_following_one() {
    for start in spread_starts() {
        for want_ascending in [true, false] {
            let root = first_directed_crossing_after(
                ok(diurnal),
                start,
                start + 3.0,
                HOUR,
                Limits::NONE,
                want_ascending,
            )
            .unwrap()
            .expect("a root");
            let next = first_directed_crossing_after(
                ok(diurnal),
                root,
                root + 3.0,
                HOUR,
                Limits::NONE,
                want_ascending,
            )
            .unwrap()
            .expect("a following root");
            assert!(
                (next - root - 1.0).abs() < TOL,
                "start {start} ascending {want_ascending}: {root} then {next}"
            );
        }
    }
}

#[test]
fn last_before_a_returned_root_returns_that_root() {
    for start in spread_starts() {
        for want_ascending in [true, false] {
            let root = first_directed_crossing_after(
                ok(diurnal),
                start,
                start + 3.0,
                HOUR,
                Limits::NONE,
                want_ascending,
            )
            .unwrap()
            .expect("a root");
            let back = last_directed_crossing_before(
                ok(diurnal),
                root - 3.0,
                root,
                HOUR,
                Limits::NONE,
                want_ascending,
            )
            .unwrap()
            .expect("the same root");
            assert!(
                back <= root && root - back < TOL,
                "start {start} ascending {want_ascending}: {root} then {back}"
            );
        }
    }
}

#[test]
fn range_starting_at_a_returned_root_excludes_it() {
    for start in spread_starts() {
        let root = first_directed_crossing_after(
            ok(diurnal),
            start,
            start + 3.0,
            HOUR,
            Limits::NONE,
            true,
        )
        .unwrap()
        .expect("a root");
        let roots =
            directed_crossings_in_range(ok(diurnal), root, root + 0.5, HOUR, Limits::NONE, true)
                .unwrap();
        assert!(roots.is_empty(), "start {start}: {root} then {roots:?}");
    }
}

#[test]
fn first_after_a_returned_set_finds_the_rise_ending_a_short_night() {
    for start in (0..48).map(|i| dip_set() - 0.9 + 0.017_3 * f64::from(i)) {
        let set =
            first_directed_crossing_after(ok(dip), start, start + 3.0, HOUR, Limits::NONE, false)
                .unwrap()
                .expect("the set");
        assert!((set - dip_set()).abs() < TOL, "start {start}: set {set}");
        let rise = first_directed_crossing_after(ok(dip), set, set + 3.0, HOUR, Limits::NONE, true)
            .unwrap()
            .expect("the rise");
        assert!(
            (rise - dip_rise()).abs() < TOL,
            "start {start}: rise {rise} vs {}",
            dip_rise()
        );
    }
}

#[test]
fn first_after_finds_a_short_night_in_the_first_grid_interval() {
    // Anchored a few minutes before the set: the whole night lies inside the
    // first grid interval and the anchor is the lowest sample, so only a
    // sample before the anchor can reveal the culmination.
    for minutes_before in [1.0, 5.0, 12.0, 18.0] {
        let lo = dip_set() - minutes_before * MINUTE;
        let rise = first_directed_crossing_after(ok(dip), lo, lo + 3.0, HOUR, Limits::NONE, true)
            .unwrap()
            .expect("the rise");
        assert!(
            (rise - dip_rise()).abs() < TOL,
            "{minutes_before} min before: rise {rise} vs {}",
            dip_rise()
        );
        let set = first_directed_crossing_after(ok(dip), lo, lo + 3.0, HOUR, Limits::NONE, false)
            .unwrap()
            .expect("the set");
        assert!(
            (set - dip_set()).abs() < TOL,
            "{minutes_before} min before: set {set} vs {}",
            dip_set()
        );
    }
}

#[test]
fn last_before_finds_a_short_night_in_the_first_grid_interval() {
    // The backward twin: anchored a few minutes after the rise.
    for minutes_after in [1.0, 5.0, 12.0, 18.0] {
        let hi = dip_rise() + minutes_after * MINUTE;
        let set = last_directed_crossing_before(ok(dip), hi - 3.0, hi, HOUR, Limits::NONE, false)
            .unwrap()
            .expect("the set");
        assert!(
            (set - dip_set()).abs() < TOL,
            "{minutes_after} min after: set {set} vs {}",
            dip_set()
        );
        let rise = last_directed_crossing_before(ok(dip), hi - 3.0, hi, HOUR, Limits::NONE, true)
            .unwrap()
            .expect("the rise");
        assert!(
            (rise - dip_rise()).abs() < TOL,
            "{minutes_after} min after: rise {rise} vs {}",
            dip_rise()
        );
    }
}

#[test]
fn a_short_night_in_the_far_grid_interval_is_found() {
    // The night sits in the interval that holds the range's far end, nearer
    // to the overshoot sample than to the last in-range one, so the
    // overshoot sample is the lowest and needs a neighbour beyond it.
    let centre = T0 + 0.25;
    // Forward: nodes at `centre − 40 min` and `centre + 20 min` straddle it.
    let hi = dip_rise() + 5.0 * MINUTE;
    let lo = centre + 20.0 * MINUTE - 6.0 * HOUR;
    let sets = directed_crossings_in_range(ok(dip), lo, hi, HOUR, Limits::NONE, false).unwrap();
    let rises = directed_crossings_in_range(ok(dip), lo, hi, HOUR, Limits::NONE, true).unwrap();
    assert_eq!(sets.len(), 1, "sets {sets:?}");
    assert_eq!(rises.len(), 1, "rises {rises:?}");
    assert!((sets[0] - dip_set()).abs() < TOL, "set {}", sets[0]);
    assert!((rises[0] - dip_rise()).abs() < TOL, "rise {}", rises[0]);
    // Backward: nodes at `centre + 40 min` and `centre − 20 min`.
    let lo = dip_set() - 5.0 * MINUTE;
    let hi = centre - 20.0 * MINUTE + 6.0 * HOUR;
    let set = last_directed_crossing_before(ok(dip), lo, hi, HOUR, Limits::NONE, false)
        .unwrap()
        .expect("the set");
    assert!((set - dip_set()).abs() < TOL, "set {set}");
}

#[test]
fn a_short_day_in_the_first_grid_interval_is_found() {
    // The polar-night mirror of the short night: a ~20-minute day.
    let (rise_at, set_at) = (dip_set(), dip_rise());
    let lo = rise_at - 5.0 * MINUTE;
    let set = first_directed_crossing_after(ok(graze), lo, lo + 3.0, HOUR, Limits::NONE, false)
        .unwrap()
        .expect("the set");
    assert!((set - set_at).abs() < TOL, "set {set} vs {set_at}");
    let hi = set_at + 5.0 * MINUTE;
    let rise = last_directed_crossing_before(ok(graze), hi - 3.0, hi, HOUR, Limits::NONE, true)
        .unwrap()
        .expect("the rise");
    assert!((rise - rise_at).abs() < TOL, "rise {rise} vs {rise_at}");
}

#[test]
fn a_root_behind_the_anchor_is_neither_returned_nor_refined() {
    // Ascending root 12 minutes before the forward anchor: it belongs to the
    // guard interval, which informs culmination detection only.
    let calls = Cell::new(0usize);
    let lo = T0 + 5.0 * HOUR;
    let behind = move |t: f64| 100.0 * (t - (lo - 0.2 * HOUR));
    let counted = |t: f64| {
        calls.set(calls.get() + 1);
        Ok(behind(t))
    };
    let none =
        first_directed_crossing_after(counted, lo, lo + 5.0 * HOUR, HOUR, Limits::NONE, true)
            .unwrap();
    assert!(none.is_none(), "{none:?}");
    assert!(
        calls.get() <= 9,
        "expected grid samples only, got {} evaluations",
        calls.get()
    );
    // Mirror: descending-in-reverse root 12 minutes after the backward anchor.
    let calls = Cell::new(0usize);
    let hi = T0 + 5.0 * HOUR;
    let ahead = move |t: f64| 100.0 * (t - (hi + 0.2 * HOUR));
    let counted = |t: f64| {
        calls.set(calls.get() + 1);
        Ok(ahead(t))
    };
    let none =
        last_directed_crossing_before(counted, hi - 5.0 * HOUR, hi, HOUR, Limits::NONE, true)
            .unwrap();
    assert!(none.is_none(), "{none:?}");
    assert!(
        calls.get() <= 9,
        "expected grid samples only, got {} evaluations",
        calls.get()
    );
}

#[test]
fn an_empty_range_still_evaluates_the_anchor() {
    let boom = |_: f64| Err(EventError::Backend("boom".into()));
    assert!(first_directed_crossing_after(boom, T0 + 1.0, T0, HOUR, Limits::NONE, true).is_err());
    assert!(last_directed_crossing_before(boom, T0 + 1.0, T0, HOUR, Limits::NONE, true).is_err());
    let none =
        first_directed_crossing_after(ok(diurnal), T0 + 1.0, T0, HOUR, Limits::NONE, true).unwrap();
    assert!(none.is_none());
}

/// An hour-angle-like residual (degrees): climbs 360° per day through zero at
/// `T0 + 0.4 + k` and wraps from +180 to −180 half a day later.
fn sawtooth(t: f64) -> f64 {
    (360.0 * (t - T0 - 0.4) + 180.0).rem_euclid(360.0) - 180.0
}

#[test]
fn wrapped_sawtooth_yields_its_ascending_zeros_and_skips_the_seam() {
    // The meridian-transit searches ask for ascending crossings of a wrapped
    // residual: each zero is one, the wrap seam is a descending sign change.
    let calls = Cell::new(0usize);
    let counted = |t: f64| {
        calls.set(calls.get() + 1);
        Ok(sawtooth(t))
    };
    let roots =
        directed_crossings_in_range(counted, T0, T0 + 3.0, HOUR, Limits::NONE, true).unwrap();
    assert_eq!(roots.len(), 3, "roots {roots:?}");
    for (k, r) in roots.iter().enumerate() {
        let want = T0 + 0.4 + k as f64;
        assert!((r - want).abs() < TOL, "root {k}: {r} vs {want}");
    }
    // 72 grid intervals plus guards, and one bisection per zero: the seam
    // costs no refinement and triggers no culmination search.
    assert!(
        calls.get() < 76 + 3 * 14,
        "expected grid samples and three bisections, got {}",
        calls.get()
    );
    let last = last_directed_crossing_before(ok(sawtooth), T0, T0 + 3.0, HOUR, Limits::NONE, true)
        .unwrap()
        .expect("a zero");
    assert!((last - (T0 + 2.4)).abs() < TOL, "{last}");
}

// Issue #203: the window's ends. A walk never samples outside its limits,
// still brackets the partial interval before a limit, still finds a graze
// next to a limit, and says so when a limit cuts it short.

fn until(latest: f64) -> Limits {
    Limits {
        earliest: f64::NEG_INFINITY,
        latest,
    }
}

fn from(earliest: f64) -> Limits {
    Limits {
        earliest,
        latest: f64::INFINITY,
    }
}

/// The Julian Day an `OutOfWindow` error names.
fn out_of_window_jd<T: std::fmt::Debug>(result: Result<T, EventError>) -> f64 {
    match result {
        Err(EventError::OutOfWindow { julian_day }) => julian_day,
        other => panic!("expected OutOfWindow, got {other:?}"),
    }
}

#[test]
fn a_root_in_the_partial_interval_before_the_latest_limit_is_found() {
    // The ascending root is at T0 + 1.16 h. The grid from T0 has nodes at 1 h
    // and 2 h; the limit at 1.5 h replaces the second.
    let limits = until(T0 + 1.5 * HOUR);
    let root = first_directed_crossing_after(ok(diurnal), T0, T0 + 3.0, HOUR, limits, true)
        .unwrap()
        .expect("the root lies before the limit");
    assert!((root - (T0 + a())).abs() < TOL, "{root}");
    let roots =
        directed_crossings_in_range(ok(diurnal), T0, T0 + 1.5 * HOUR, HOUR, limits, true).unwrap();
    assert_eq!(roots.len(), 1, "{roots:?}");
    assert!((roots[0] - (T0 + a())).abs() < TOL, "{roots:?}");
}

#[test]
fn a_root_in_the_partial_interval_after_the_earliest_limit_is_found() {
    // Backward from T0 + 3 h the nodes are 2 h, 1 h, 0 h. A limit at 1.1 h
    // replaces the node at 1 h and still sits before the root at 1.16 h.
    let limits = from(T0 + 1.1 * HOUR);
    let root =
        last_directed_crossing_before(ok(diurnal), T0 - 3.0, T0 + 3.0 * HOUR, HOUR, limits, true)
            .unwrap()
            .expect("the root lies after the limit");
    assert!((root - (T0 + a())).abs() < TOL, "{root}");
}

#[test]
fn a_search_a_limit_cuts_short_reports_out_of_window() {
    // The limit at 1.1 h is before the root at 1.16 h: the search cannot say
    // whether a root follows, and names the first node it could not sample.
    let limits = until(T0 + 1.1 * HOUR);
    let needed = out_of_window_jd(first_directed_crossing_after(
        ok(diurnal),
        T0,
        T0 + 3.0,
        HOUR,
        limits,
        true,
    ));
    assert!((needed - (T0 + 2.0 * HOUR)).abs() < 1e-9, "{needed}");
    let needed = out_of_window_jd(directed_crossings_in_range(
        ok(diurnal),
        T0,
        T0 + 3.0,
        HOUR,
        limits,
        true,
    ));
    assert!((needed - (T0 + 2.0 * HOUR)).abs() < 1e-9, "{needed}");

    // Backward: the limit at 1.3 h is after the root.
    let needed = out_of_window_jd(last_directed_crossing_before(
        ok(diurnal),
        T0 - 3.0,
        T0 + 3.0 * HOUR,
        HOUR,
        from(T0 + 1.3 * HOUR),
        true,
    ));
    assert!((needed - (T0 + HOUR)).abs() < 1e-9, "{needed}");
}

#[test]
fn a_range_that_ends_at_a_limit_is_covered() {
    // Nothing is cut short when the range itself stops at the limit, whether
    // or not it holds a root: only the guard samples are missing.
    let hi = T0 + 1.1 * HOUR;
    let none = first_directed_crossing_after(ok(diurnal), T0, hi, HOUR, until(hi), true).unwrap();
    assert!(none.is_none(), "{none:?}");
    let none =
        last_directed_crossing_before(ok(diurnal), T0, T0 + HOUR, HOUR, from(T0), true).unwrap();
    assert!(none.is_none(), "{none:?}");
    let roots = directed_crossings_in_range(
        ok(diurnal),
        T0,
        T0 + 2.0,
        HOUR,
        Limits {
            earliest: T0,
            latest: T0 + 2.0,
        },
        true,
    )
    .unwrap();
    assert_eq!(roots.len(), 2, "{roots:?}");
}

#[test]
fn a_graze_next_to_the_latest_limit_is_found_without_a_guard() {
    // The graze peaks at T0 + 0.25 d and lasts 20 minutes. The last grid
    // node is 31 minutes before the peak and the limit 14 minutes after it,
    // so the peak is nearest the sample at the limit, which has no neighbour
    // beyond it.
    let lo = T0 + 0.02;
    let hi = T0 + 0.26;
    for (want_ascending, want) in [
        (true, T0 + 0.25 - graze_tau()),
        (false, T0 + 0.25 + graze_tau()),
    ] {
        let roots = directed_crossings_in_range(ok(graze), lo, hi, HOUR, until(hi), want_ascending)
            .unwrap();
        assert_eq!(roots.len(), 1, "ascending {want_ascending}: {roots:?}");
        assert!((roots[0] - want).abs() < TOL, "{roots:?} vs {want}");
        let first =
            first_directed_crossing_after(ok(graze), lo, hi + 1.0, HOUR, until(hi), want_ascending)
                .unwrap()
                .expect("the graze lies before the limit");
        assert!((first - want).abs() < TOL, "{first} vs {want}");
    }
}

#[test]
fn a_graze_next_to_the_anchor_is_found_without_a_guard() {
    // Forward from 14 minutes before the peak, with the window starting at
    // the anchor; backward from 14 minutes after it, with the window ending
    // there. Either way the anchor is the sample nearest the peak.
    let lo = T0 + 0.24;
    let hi = T0 + 0.26;
    for (want_ascending, want) in [
        (true, T0 + 0.25 - graze_tau()),
        (false, T0 + 0.25 + graze_tau()),
    ] {
        let first =
            first_directed_crossing_after(ok(graze), lo, lo + 1.0, HOUR, from(lo), want_ascending)
                .unwrap()
                .expect("the graze follows the anchor");
        assert!((first - want).abs() < TOL, "forward: {first} vs {want}");
        let last =
            last_directed_crossing_before(ok(graze), hi - 1.0, hi, HOUR, until(hi), want_ascending)
                .unwrap()
                .expect("the graze precedes the anchor");
        assert!((last - want).abs() < TOL, "backward: {last} vs {want}");
    }
}

#[test]
fn a_near_miss_next_to_a_limit_yields_nothing() {
    // The same peak 0.06° BELOW the horizon: the edge check refines it and
    // finds no crossing.
    let near_miss = |t: f64| 60.0 * (TAU * (t - T0)).sin() - 60.0 * (1.0 + EPS);
    let hi = T0 + 0.26;
    let roots =
        directed_crossings_in_range(ok(near_miss), T0 + 0.02, hi, HOUR, until(hi), true).unwrap();
    assert!(roots.is_empty(), "{roots:?}");
}

#[test]
fn no_sample_is_taken_outside_the_limits() {
    let limits = Limits {
        earliest: T0 + 0.24,
        latest: T0 + 1.26,
    };
    let seen = Cell::new((f64::INFINITY, f64::NEG_INFINITY));
    let recording = |t: f64| {
        let (lowest, highest) = seen.get();
        seen.set((lowest.min(t), highest.max(t)));
        Ok(graze(t))
    };
    let _ = directed_crossings_in_range(recording, T0 + 0.24, T0 + 1.26, HOUR, limits, true);
    let _ = first_directed_crossing_after(recording, T0 + 0.3, T0 + 4.0, HOUR, limits, false);
    let _ = last_directed_crossing_before(recording, T0 - 4.0, T0 + 1.2, HOUR, limits, false);
    let (lowest, highest) = seen.get();
    assert!(lowest >= limits.earliest, "sampled {lowest}");
    assert!(highest <= limits.latest, "sampled {highest}");
}

/// `diurnal`, unreadable before `readable_from`, as an apparent place is
/// within light-time of the window's start.
fn unreadable_before(readable_from: f64) -> impl FnMut(f64) -> Result<f64, EventError> {
    move |t| {
        if t < readable_from {
            Err(EventError::OutOfWindow { julian_day: t })
        } else {
            Ok(diurnal(t))
        }
    }
}

#[test]
fn an_unreadable_node_ends_the_walk_like_a_limit() {
    // Backward from T0 + 3 h; the root is at 1.16 h. With the node at 1 h
    // readable the root is bracketed; with it unreadable the search is cut
    // short and names that node.
    let last = |readable_from: f64| {
        last_directed_crossing_before(
            unreadable_before(readable_from),
            T0 - 3.0,
            T0 + 3.0 * HOUR,
            HOUR,
            Limits::NONE,
            true,
        )
    };
    let root = last(T0 + 0.9 * HOUR)
        .unwrap()
        .expect("the root is bracketed");
    assert!((root - (T0 + a())).abs() < TOL, "{root}");
    let needed = out_of_window_jd(last(T0 + HOUR + 5.0 * MINUTE));
    assert!((needed - (T0 + HOUR)).abs() < 1e-9, "{needed}");
}

#[test]
fn an_unreadable_guard_does_not_stop_a_forward_search() {
    // The anchor at 1.05 h is readable; the guard an hour behind it is not.
    let root = first_directed_crossing_after(
        unreadable_before(T0 + 0.9 * HOUR),
        T0 + 1.05 * HOUR,
        T0 + 3.0,
        HOUR,
        Limits::NONE,
        true,
    )
    .unwrap()
    .expect("the root follows the anchor");
    assert!((root - (T0 + a())).abs() < TOL, "{root}");
}

#[test]
fn an_unreadable_anchor_is_an_error() {
    let needed = out_of_window_jd(first_directed_crossing_after(
        unreadable_before(T0 + HOUR),
        T0,
        T0 + 3.0,
        HOUR,
        Limits::NONE,
        true,
    ));
    assert_eq!(needed, T0);
}
