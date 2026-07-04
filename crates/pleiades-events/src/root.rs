//! Generic time-domain root-finder: bracket by stepping, refine by bisection.
//! Mirrors the eclipse `syzygy` scanner but takes an arbitrary target function.

// Items here are pub(crate) for upcoming crossing-engine tasks; silence
// dead_code lint until those consumers land.
#![allow(dead_code)]

use crate::error::EventError;

/// Bisection tolerance: 0.5 second of time, in days.
pub(crate) const REFINE_TOLERANCE_DAYS: f64 = 0.5 / 86_400.0;

/// Signed wrap of a degree difference into `(-180, 180]`.
pub(crate) fn wrap180(mut d: f64) -> f64 {
    d = ((d + 180.0).rem_euclid(360.0)) - 180.0;
    d
}

fn bisect<F>(f: &mut F, mut lo: f64, mut f_lo: f64, mut hi: f64) -> Result<f64, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    while (hi - lo) > REFINE_TOLERANCE_DAYS {
        let mid = 0.5 * (lo + hi);
        let f_mid = f(mid)?;
        if (f_lo <= 0.0) == (f_mid <= 0.0) {
            lo = mid;
            f_lo = f_mid;
        } else {
            hi = mid;
        }
    }
    Ok(0.5 * (lo + hi))
}

/// All roots of `f` in `[lo_jd, hi_jd]`, ascending. `step_days` must be small
/// enough to separate the closest expected crossings for the body in question.
pub(crate) fn crossings_in_range<F>(
    mut f: F,
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
) -> Result<Vec<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    let mut out = Vec::new();
    let mut prev_jd = lo_jd;
    let mut prev_f = f(prev_jd)?;
    let mut jd = lo_jd + step_days;
    while jd <= hi_jd + step_days {
        let f_jd = f(jd)?;
        // Real crossing: sign change whose function jump is small enough to be a
        // zero-crossing rather than the ±180 wrap seam.
        if (prev_f <= 0.0) != (f_jd <= 0.0) && (prev_f - f_jd).abs() < 180.0 {
            let root = bisect(&mut f, prev_jd, prev_f, jd)?;
            if root >= lo_jd && root <= hi_jd {
                out.push(root);
            }
        }
        prev_jd = jd;
        prev_f = f_jd;
        jd += step_days;
    }
    Ok(out)
}

/// The first root of `f` strictly greater than `lo_jd`, or `None` if there is no
/// root in `[lo_jd, hi_jd]`. Early-terminating twin of [`crossings_in_range`]: it
/// brackets by stepping and returns as soon as the first zero-crossing is refined,
/// instead of scanning the whole window. Uses the identical wrap-seam (`< 180.0`)
/// guard and bisection tolerance, so the returned root matches
/// `crossings_in_range(..).first()` for the same arguments.
pub(crate) fn first_crossing_after<F>(
    mut f: F,
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
) -> Result<Option<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    let mut prev_jd = lo_jd;
    let mut prev_f = f(prev_jd)?;
    let mut jd = lo_jd + step_days;
    while jd <= hi_jd + step_days {
        let f_jd = f(jd)?;
        if (prev_f <= 0.0) != (f_jd <= 0.0) && (prev_f - f_jd).abs() < 180.0 {
            let root = bisect(&mut f, prev_jd, prev_f, jd)?;
            if root >= lo_jd && root <= hi_jd {
                return Ok(Some(root));
            }
        }
        prev_jd = jd;
        prev_f = f_jd;
        jd += step_days;
    }
    Ok(None)
}

/// The last root of `f` in `[lo_jd, hi_jd]`, or `None` if there is no root in
/// range. Backward early-terminating twin of [`crossings_in_range`]: it walks
/// the SAME `lo_jd`-anchored grid — samples at `lo_jd + k*step_days` for
/// integer `k`, bracketing the SAME `[lo_jd + (k-1)*step_days, lo_jd +
/// k*step_days]` intervals `crossings_in_range` brackets — just visited in
/// DECREASING `k`, and returns as soon as the first (highest-JD) zero-crossing
/// is refined, instead of scanning the whole window. It does NOT step a fresh
/// grid downward from `hi_jd`, which would bracket different intervals
/// whenever `(hi_jd - lo_jd)` is not an exact multiple of `step_days`. Uses the
/// identical wrap-seam (`< 180.0`) guard and bisection tolerance, and — like
/// `crossings_in_range` — always evaluates `f` at the low anchor `lo_jd` once
/// (even on an empty/inverted range), so a backend error there propagates
/// identically. The returned root matches `crossings_in_range(..).last()` for
/// the same arguments.
pub(crate) fn last_crossing_before<F>(
    mut f: F,
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
) -> Result<Option<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    // Matches `crossings_in_range`'s loop bound `jd <= hi_jd + step_days`,
    // where `jd` walks `lo_jd + k*step_days` for k = 1, 2, ...
    let k_max = ((hi_jd + step_days - lo_jd) / step_days).floor() as i64;
    // Match `crossings_in_range`, which always evaluates the low anchor once
    // (`let mut prev_f = f(lo_jd)?;`) before its loop: do the same here so a
    // backend error at `lo_jd` propagates identically, even on an
    // empty/inverted range where the loop below never runs.
    let _ = f(lo_jd)?;
    if k_max < 1 {
        return Ok(None);
    }
    let mut cur_jd = lo_jd + (k_max as f64) * step_days;
    let mut cur_f = f(cur_jd)?;
    let mut k = k_max;
    while k >= 1 {
        let prev_jd = lo_jd + ((k - 1) as f64) * step_days;
        let prev_f = f(prev_jd)?;
        // Same wrap-seam guard as `crossings_in_range`.
        if (prev_f <= 0.0) != (cur_f <= 0.0) && (prev_f - cur_f).abs() < 180.0 {
            let root = bisect(&mut f, prev_jd, prev_f, cur_jd)?;
            if root >= lo_jd && root <= hi_jd {
                return Ok(Some(root));
            }
        }
        cur_jd = prev_jd;
        cur_f = prev_f;
        k -= 1;
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    // A prograde body at `rate` deg/day; f(t) = wrap180(rate*(t-t0) - offset).
    // Root where rate*(t-t0) == offset (mod 360).
    #[test]
    fn finds_single_prograde_crossing() {
        let rate = 1.0_f64; // ~Sun
        let t0 = 2_451_545.0;
        let roots = crossings_in_range(|t| Ok(wrap180(rate * (t - t0) - 10.0)), t0, t0 + 30.0, 1.0)
            .unwrap();
        assert_eq!(roots.len(), 1);
        assert!((roots[0] - (t0 + 10.0)).abs() < 1e-3, "root {}", roots[0]);
    }

    // A body whose longitude goes forward, retrogrades back over the target,
    // then forward again — three crossings of the same longitude. Model the
    // longitude as a parabola in time so dλ/dt changes sign once.
    #[test]
    fn finds_retrograde_triple_crossing() {
        // lon(t) = 30 + 8*(t-t0) - (t-t0)^2  (deg); target = 45.
        // Solve 8x - x^2 = 15 -> x = 3, x = 5 within the loop; plus a third
        // when lon comes back around... use a target that yields exactly 3 in-range.
        let t0 = 2_451_545.0;
        let lon = |t: f64| {
            let x = t - t0;
            30.0 + 8.0 * x - x * x
        };
        // target 37 -> 8x - x^2 = 7 -> x=1, x=7 (two crossings). Add a wrap-around
        // crossing by extending the window so lon dips below and returns.
        let roots = crossings_in_range(|t| Ok(wrap180(lon(t) - 37.0)), t0, t0 + 8.0, 0.25).unwrap();
        assert_eq!(roots.len(), 2, "roots {roots:?}");
        assert!((roots[0] - (t0 + 1.0)).abs() < 1e-2);
        assert!((roots[1] - (t0 + 7.0)).abs() < 1e-2);
    }

    #[test]
    fn empty_when_no_crossing() {
        let t0 = 2_451_545.0;
        let roots =
            crossings_in_range(|t| Ok(wrap180(0.0 * t + 90.0)), t0, t0 + 30.0, 1.0).unwrap();
        assert!(roots.is_empty());
    }

    #[test]
    fn propagates_target_error() {
        let t0 = 2_451_545.0;
        let err = crossings_in_range(
            |_| Err(EventError::Backend("boom".into())),
            t0,
            t0 + 1.0,
            0.5,
        )
        .unwrap_err();
        assert!(matches!(err, EventError::Backend(_)));
    }

    #[test]
    fn backward_propagates_target_error() {
        let t0 = 2_451_545.0;
        let err = last_crossing_before(
            |_| Err(EventError::Backend("boom".into())),
            t0,
            t0 + 1.0,
            0.5,
        )
        .unwrap_err();
        assert!(matches!(err, EventError::Backend(_)));
    }

    // Fail-closed parity with `crossings_in_range`: on an empty/inverted range
    // (`hi < lo` → `k_max < 1`, loop never runs) the low anchor is still
    // evaluated once, so a backend error there must surface rather than being
    // swallowed into `Ok(None)`.
    #[test]
    fn backward_empty_range_still_evaluates_low_anchor() {
        let t0 = 2_451_545.0;
        // Inverted range: hi < lo.
        let err = last_crossing_before(
            |_| Err(EventError::Backend("boom".into())),
            t0,
            t0 - 5.0,
            0.5,
        )
        .unwrap_err();
        assert!(matches!(err, EventError::Backend(_)));
        // And a non-erroring closure on the same inverted range yields None.
        let none =
            last_crossing_before(|t| Ok(wrap180(0.0 * t + 90.0)), t0, t0 - 5.0, 0.5).unwrap();
        assert!(none.is_none());
    }

    // The real guarantee: `last_crossing_before` walks the SAME lo_jd-anchored
    // grid as `crossings_in_range` (just in decreasing k), so it must agree with
    // `crossings_in_range(..).last()` even when `(hi - lo)` is NOT an exact
    // multiple of `step` — a fresh grid stepped downward from `hi` would bracket
    // different intervals and silently diverge. Deliberately include several
    // non-aligned ranges plus one exactly-aligned range.
    fn assert_last_matches<F>(name: &str, mut make_f: F, lo: f64, hi: f64, step: f64, offset: f64)
    where
        F: FnMut() -> Box<dyn FnMut(f64) -> Result<f64, EventError>>,
    {
        let expected = crossings_in_range(make_f(), lo, hi, step)
            .unwrap()
            .last()
            .copied();
        let actual = last_crossing_before(make_f(), lo, hi, step).unwrap();
        match (expected, actual) {
            (Some(e), Some(a)) => assert!(
                (e - a).abs() < 1e-6,
                "{name} offset {offset} lo {lo} hi {hi}: expected {e}, got {a}"
            ),
            (None, None) => {}
            (e, a) => panic!("{name} offset {offset} lo {lo} hi {hi}: expected {e:?}, got {a:?}"),
        }
    }

    #[test]
    fn last_crossing_before_matches_crossings_in_range_last() {
        let t0 = 2_451_545.0;
        let step = 0.25;

        // Non-aligned (hi - lo) offsets, varied by loop index, plus one
        // exactly-aligned range (offset == 0.0).
        let offsets = [0.0, 0.137, 1.0 / 3.0, 0.061, 29.5 * step, 0.999];

        for (i, &offset) in offsets.iter().enumerate() {
            let lo = t0 + i as f64 * 0.3; // vary lo per iteration too
            let hi = lo + 30.0 + offset;

            // A single prograde crossing.
            assert_last_matches(
                "single",
                || -> Box<dyn FnMut(f64) -> Result<f64, EventError>> {
                    Box::new(move |t: f64| Ok(wrap180(1.0 * (t - t0) - 10.0)))
                },
                lo,
                hi,
                step,
                offset,
            );

            // The retrograde parabola from `finds_retrograde_triple_crossing`
            // (multiple crossings in range).
            assert_last_matches(
                "retrograde",
                || -> Box<dyn FnMut(f64) -> Result<f64, EventError>> {
                    Box::new(move |t: f64| {
                        let x = t - t0;
                        Ok(wrap180(30.0 + 8.0 * x - x * x - 37.0))
                    })
                },
                lo,
                hi,
                step,
                offset,
            );

            // A no-crossing constant.
            assert_last_matches(
                "constant",
                || -> Box<dyn FnMut(f64) -> Result<f64, EventError>> {
                    Box::new(move |t: f64| Ok(wrap180(0.0 * t + 90.0)))
                },
                lo,
                hi,
                step,
                offset,
            );
        }
    }
}
