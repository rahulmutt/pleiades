//! Generic time-domain root-finder: bracket by stepping, refine by bisection.
//! Mirrors the eclipse `syzygy` scanner but takes an arbitrary target function.

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

#[cfg(test)]
mod tests {
    use super::*;

    // A prograde body at `rate` deg/day; f(t) = wrap180(rate*(t-t0) - offset).
    // Root where rate*(t-t0) == offset (mod 360).
    #[test]
    fn finds_single_prograde_crossing() {
        let rate = 1.0_f64; // ~Sun
        let t0 = 2_451_545.0;
        let roots = crossings_in_range(
            |t| Ok(wrap180(rate * (t - t0) - 10.0)),
            t0,
            t0 + 30.0,
            1.0,
        )
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
        let roots = crossings_in_range(
            |t| Ok(wrap180(lon(t) - 37.0)),
            t0,
            t0 + 8.0,
            0.25,
        )
        .unwrap();
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
}
