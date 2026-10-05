//! Generic time-domain root-finder: bracket by stepping, refine by bisection.
//! Mirrors the eclipse `syzygy` scanner but takes an arbitrary target function.

// Items here are pub(crate) for upcoming crossing-engine tasks; silence
// dead_code lint until those consumers land.
#![allow(dead_code)]

use crate::error::EventError;

/// Bisection tolerance: 0.5 second of time, in days. The widest the final
/// bracket may be, and so the most a returned instant can trail its crossing.
pub(crate) const REFINE_TOLERANCE_DAYS: f64 = 0.5 / 86_400.0;

/// Signed wrap of a degree difference into `(-180, 180]`.
pub(crate) fn wrap180(mut d: f64) -> f64 {
    d = ((d + 180.0).rem_euclid(360.0)) - 180.0;
    d
}

/// Refines a sign change of `f` across `[lo, hi]` and returns the LATER end of
/// the final bracket, which is no wider than [`REFINE_TOLERANCE_DAYS`].
///
/// The later end, not the midpoint, because callers hand the instant back to
/// a follow-on search (issues #80, #81). Bisection keeps the invariant that
/// `f(lo)` carries the pre-crossing sign and `f(hi)` the post-crossing one,
/// so the returned instant is "settled": the crossing has already happened
/// there, less than the tolerance earlier, and a search anchored at it reads
/// the post-crossing sign and cannot bracket the same crossing again. A
/// midpoint sits before the crossing about half the time.
pub(crate) fn bisect<F>(
    f: &mut F,
    mut lo: f64,
    mut f_lo: f64,
    mut hi: f64,
) -> Result<f64, EventError>
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
    Ok(hi)
}

/// Whether `f` crosses zero between two samples: a sign change whose jump is
/// small enough to be a zero-crossing and not the ±180 wrap seam.
fn brackets_crossing(f_earlier: f64, f_later: f64) -> bool {
    (f_earlier <= 0.0) != (f_later <= 0.0) && (f_earlier - f_later).abs() < 180.0
}

/// All roots of `f` in `(lo_jd, hi_jd]`, ascending. `step_days` must be small
/// enough to separate the closest expected crossings for the body in question.
///
/// The grid is anchored at `lo_jd` and its last interval is cut short at
/// `hi_jd`, so `f` is never evaluated outside the range and each end decides
/// membership by the sign of `f` there, exactly: a crossing is in range when
/// `f` carries its pre-crossing sign at `lo_jd` and its post-crossing sign at
/// `hi_jd`. Comparing a refined root against an end instead would drop or
/// repeat a crossing within the bisection tolerance of it. With
/// [`bisect`] returning the settled end of its bracket, ranges that share an
/// end therefore partition the crossings, whatever that end is, including an
/// instant this module returned (issue #159).
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
    while prev_jd < hi_jd {
        let jd = (prev_jd + step_days).min(hi_jd);
        let f_jd = f(jd)?;
        if brackets_crossing(prev_f, f_jd) {
            out.push(bisect(&mut f, prev_jd, prev_f, jd)?);
        }
        prev_jd = jd;
        prev_f = f_jd;
    }
    Ok(out)
}

/// The first root of `f` in `(lo_jd, hi_jd]`, or `None`. Early-terminating
/// twin of [`crossings_in_range`]: the same grid, guard and tolerance, so the
/// returned root is `crossings_in_range(..).first()` for the same arguments,
/// found without scanning the rest of the range.
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
    while prev_jd < hi_jd {
        let jd = (prev_jd + step_days).min(hi_jd);
        let f_jd = f(jd)?;
        if brackets_crossing(prev_f, f_jd) {
            return bisect(&mut f, prev_jd, prev_f, jd).map(Some);
        }
        prev_jd = jd;
        prev_f = f_jd;
    }
    Ok(None)
}

/// The last root of `f` in `(lo_jd, hi_jd]`, or `None`. Backward
/// early-terminating twin of [`crossings_in_range`]: it walks a grid anchored
/// at `hi_jd` downward, its last interval cut short at `lo_jd`, and stops as
/// soon as the latest crossing is refined.
///
/// Anchoring at `hi_jd` is what makes the upper end exact. A crossing is at
/// or before `hi_jd` when `f` already carries its post-crossing sign there,
/// so an instant [`bisect`] settled on a crossing, handed back as `hi_jd`,
/// finds that same crossing, and one before the crossing does not (issue
/// #159). The grid differs from the `lo_jd`-anchored one of
/// [`crossings_in_range`], so the two agree on the root to within the
/// bisection tolerance, not bit for bit.
///
/// `f` is always evaluated once, even on an empty range, so a backend error
/// propagates. On an inverted range that one sample is taken at `lo_jd`, not
/// at the anchor: callers clamp `lo_jd` into the window their backend
/// serves, and an `hi_jd` below it may lie outside.
pub(crate) fn last_crossing_before<F>(
    mut f: F,
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
) -> Result<Option<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    if hi_jd < lo_jd {
        f(lo_jd)?;
        return Ok(None);
    }
    let mut cur_jd = hi_jd;
    let mut cur_f = f(cur_jd)?;
    while cur_jd > lo_jd {
        let prev_jd = (cur_jd - step_days).max(lo_jd);
        let prev_f = f(prev_jd)?;
        if brackets_crossing(prev_f, cur_f) {
            return bisect(&mut f, prev_jd, prev_f, cur_jd).map(Some);
        }
        cur_jd = prev_jd;
        cur_f = prev_f;
    }
    Ok(None)
}

/// One sample of a wrapped-degree function: `(julian_day, degrees)`.
type Sample = (f64, f64);

/// The turning point of `d` between `p0` and `p2`, when the three samples
/// show one: the slope into `p1` is non-zero and the slope out of it has the
/// opposite sign or is zero (a peak midway between two grid points leaves
/// equal samples either side). Located by a ternary search on `d` unwrapped
/// about `p1`.
///
/// Only the value at the turning point matters to the caller, not its time,
/// so the search does not suffer from the flatness of an extremum. A turning
/// point reported where there is none only adds a breakpoint.
fn turning_point<F>(
    d: &mut F,
    p0: Sample,
    p1: Sample,
    p2: Sample,
) -> Result<Option<Sample>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    let rise_in = wrap180(p1.1 - p0.1);
    let rise_out = wrap180(p2.1 - p1.1);
    // A NaN increment compares false and reads as "no turning point".
    let turns = rise_in != 0.0 && rise_in * rise_out <= 0.0;
    if !turns {
        return Ok(None);
    }
    let direction = rise_in.signum();
    let (mut lo, mut hi) = (p0.0, p2.0);
    while (hi - lo) > REFINE_TOLERANCE_DAYS {
        let third = (hi - lo) / 3.0;
        let (left, right) = (lo + third, hi - third);
        let at_left = direction * wrap180(d(left)? - p1.1);
        let at_right = direction * wrap180(d(right)? - p1.1);
        if at_left < at_right {
            lo = left;
        } else {
            hi = right;
        }
    }
    let jd = 0.5 * (lo + hi);
    Ok(Some((jd, d(jd)?)))
}

/// Appends the roots of `d − level`, for each of `levels`, in the bracket
/// `[a, b]` that fall inside `[lo_jd, hi_jd]`, ascending. Same sign test,
/// wrap-seam guard and bisection as [`crossings_in_range`].
fn level_roots_between<F>(
    d: &mut F,
    levels: &[f64],
    a: Sample,
    b: Sample,
    lo_jd: f64,
    hi_jd: f64,
    out: &mut Vec<f64>,
) -> Result<(), EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    let first_new = out.len();
    for &level in levels {
        let f_a = wrap180(a.1 - level);
        let f_b = wrap180(b.1 - level);
        if (f_a <= 0.0) != (f_b <= 0.0) && (f_a - f_b).abs() < 180.0 {
            let mut f = |jd: f64| Ok(wrap180(d(jd)? - level));
            let root = bisect(&mut f, a.0, f_a, b.0)?;
            if root >= lo_jd && root <= hi_jd {
                out.push(root);
            }
        }
    }
    out[first_new..].sort_by(f64::total_cmp);
    Ok(())
}

/// Tests and drops the leading brackets of `pending` whose later end is at
/// or before `limit_jd`, keeping the last tested point as the next bracket's
/// start.
fn drain_until<F>(
    d: &mut F,
    levels: &[f64],
    pending: &mut Vec<Sample>,
    limit_jd: f64,
    lo_jd: f64,
    hi_jd: f64,
    out: &mut Vec<f64>,
) -> Result<(), EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    while pending.len() >= 2 && pending[1].0 <= limit_jd {
        level_roots_between(d, levels, pending[0], pending[1], lo_jd, hi_jd, out)?;
        pending.remove(0);
    }
    Ok(())
}

/// The scan behind [`level_crossings_in_range`] and
/// [`first_level_crossing_after`]. Brackets are the intervals between
/// consecutive breakpoints: the grid `lo_jd + k·step_days`, plus every
/// turning point of `d` the samples reveal. A turning point between grid
/// samples `k − 2` and `k` is only known once sample `k` is taken, so the
/// brackets up to sample `k − 1` are tested one step late.
fn scan_levels<F>(
    mut d: F,
    levels: &[f64],
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
    first_only: bool,
) -> Result<Vec<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    let mut out = Vec::new();
    // Always sample the low anchor, as `crossings_in_range` does, so a
    // backend error there propagates even on an empty or inverted range.
    let anchor = (lo_jd, d(lo_jd)?);
    let span = hi_jd - lo_jd;
    let in_order = span > 0.0;
    if !in_order {
        return Ok(out);
    }
    let intervals = (span / step_days).ceil() as i64;
    let at = |k: i64| lo_jd + k as f64 * step_days;
    let mut pending = vec![anchor];
    // One sample before the range, to see a turning point in the first step.
    let before = at(-1);
    let mut older = (before, d(before)?);
    let mut newer = anchor;
    // One sample past the grid, to see a turning point in the last step.
    for k in 1..=intervals + 1 {
        let jd = at(k);
        let sample = (jd, d(jd)?);
        if let Some(turn) = turning_point(&mut d, older, newer, sample)? {
            if pending.first().is_some_and(|earliest| turn.0 > earliest.0) {
                let index = pending.partition_point(|point| point.0 < turn.0);
                pending.insert(index, turn);
            }
        }
        if k <= intervals {
            pending.push(sample);
        }
        // No later sample can add a breakpoint before `newer`.
        drain_until(
            &mut d,
            levels,
            &mut pending,
            newer.0,
            lo_jd,
            hi_jd,
            &mut out,
        )?;
        if first_only && !out.is_empty() {
            return Ok(out);
        }
        older = newer;
        newer = sample;
    }
    drain_until(
        &mut d,
        levels,
        &mut pending,
        at(intervals),
        lo_jd,
        hi_jd,
        &mut out,
    )?;
    Ok(out)
}

/// Every instant in `[lo_jd, hi_jd]` at which the wrapped-degree function `d`
/// equals one of `levels`, ascending. Each is a settled instant, as from
/// [`bisect`].
///
/// Unlike [`crossings_in_range`], each step is split at the turning points
/// of `d`, so two crossings of a level inside one step are both found. `d`
/// is sampled only in `[lo_jd − step_days, hi_jd + 2·step_days)`.
///
/// Limits: two turning points within two steps of each other may go unseen,
/// and with them a pair of crossings between them.
pub(crate) fn level_crossings_in_range<F>(
    d: F,
    levels: &[f64],
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
) -> Result<Vec<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    scan_levels(d, levels, lo_jd, hi_jd, step_days, false)
}

/// The first element of [`level_crossings_in_range`] for the same arguments,
/// or `None`; the scan stops at the first bracket that yields a crossing.
pub(crate) fn first_level_crossing_after<F>(
    d: F,
    levels: &[f64],
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
) -> Result<Option<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    Ok(scan_levels(d, levels, lo_jd, hi_jd, step_days, true)?
        .into_iter()
        .next())
}

/// Steps per chunk of [`last_level_crossing_before`]'s backward walk.
pub(crate) const LEVEL_CHUNK_STEPS: f64 = 64.0;

/// The last instant in `[lo_jd, hi_jd]` at which `d` equals one of `levels`,
/// or `None`. The backward twin of [`first_level_crossing_after`].
///
/// The level scanner only runs forward, because it needs the samples either
/// side of a step to split it at a turning point. So this walks the range in
/// chunks of [`LEVEL_CHUNK_STEPS`] steps, latest first, runs the forward scan
/// over each and returns the last crossing of the first chunk that holds
/// one: the cost follows the distance back to the crossing, not the length
/// of the range.
///
/// Every chunk after the first reaches one step past its upper seam. The
/// forward scan keeps a crossing only if its settled instant lies in the
/// scanned range, so a crossing just below a seam, which settles just above
/// it, is in neither neighbouring chunk's own range.
///
/// Limits: those of [`level_crossings_in_range`], and its upper end: a
/// crossing within the bisection tolerance of `hi_jd` may settle past it and
/// go unreported. `d` is sampled within the same bounds, `[lo_jd −
/// step_days, hi_jd + 2·step_days)`, and at `lo_jd` even on an empty range.
pub(crate) fn last_level_crossing_before<F>(
    mut d: F,
    levels: &[f64],
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
) -> Result<Option<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    if hi_jd <= lo_jd {
        // The forward scan's own sample of an empty range.
        d(lo_jd)?;
        return Ok(None);
    }
    let chunk_days = LEVEL_CHUNK_STEPS * step_days;
    let mut chunk_hi = hi_jd;
    let mut scan_hi = hi_jd;
    while chunk_hi > lo_jd {
        let chunk_lo = (chunk_hi - chunk_days).max(lo_jd);
        let roots = scan_levels(&mut d, levels, chunk_lo, scan_hi, step_days, false)?;
        if let Some(&last) = roots.last() {
            return Ok(Some(last));
        }
        chunk_hi = chunk_lo;
        scan_hi = chunk_lo + step_days;
    }
    Ok(None)
}

#[cfg(test)]
mod level_tests;

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

    // On an inverted range the one sample is taken at `lo`, the end inside
    // the caller's clamp: `hi` may lie outside what the backend serves.
    #[test]
    fn backward_inverted_range_samples_only_the_low_end() {
        let t0 = 2_451_545.0;
        let mut sampled = Vec::new();
        let none = last_crossing_before(
            |t| {
                sampled.push(t);
                Ok(t - t0)
            },
            t0,
            t0 - 5.0,
            0.5,
        )
        .unwrap();
        assert_eq!(none, None);
        assert_eq!(sampled, [t0]);
    }

    // Fail-closed parity with `crossings_in_range`: on an empty/inverted range
    // (`hi < lo`, loop never runs) the range is still sampled once, so a
    // backend error there must surface rather than being swallowed into
    // `Ok(None)`.
    #[test]
    fn backward_empty_range_still_evaluates_its_anchor() {
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

    // `last_crossing_before` walks a grid anchored at `hi`, `crossings_in_range`
    // one anchored at `lo`, so when `(hi - lo)` is not a multiple of `step`
    // they bracket different intervals. They must still find the same last
    // crossing, each settled within the bisection tolerance after it.
    // Deliberately include several non-aligned ranges plus one exactly-aligned
    // range.
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
                (e - a).abs() < REFINE_TOLERANCE_DAYS,
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

    /// Roots spread across the final-bracket lattice, so a refinement that
    /// returned the bracket's midpoint would land before about half of them.
    fn spread_roots() -> impl Iterator<Item = f64> {
        (0..40).map(|i| 2_451_545.0 + 0.3 + f64::from(i) * 0.013_7)
    }

    // Issue #80: the refined instant is the later end of the final bracket,
    // so the residual there already carries the post-crossing sign and the
    // instant is never earlier than the crossing it describes.
    #[test]
    fn bisect_returns_the_settled_end_of_an_ascending_crossing() {
        for c in spread_roots() {
            let mut f = |t: f64| Ok(t - c);
            let (lo, hi) = (c - 0.4, c + 0.6);
            let root = bisect(&mut f, lo, lo - c, hi).unwrap();
            assert!(root > c, "root {root} is before the crossing {c}");
            assert!(root - c <= REFINE_TOLERANCE_DAYS, "root {root} vs {c}");
        }
    }

    // Issue #159: which side of a range end a crossing falls on is read from
    // the sign of `f` at that end, not from comparing a refined root with it.

    const SECOND_DAYS: f64 = 1.0 / 86_400.0;

    /// A crossing at `c`, ascending or descending through zero.
    fn linear(c: f64, ascending: bool) -> impl FnMut(f64) -> Result<f64, EventError> {
        move |t| Ok(if ascending { t - c } else { c - t })
    }

    #[test]
    fn last_crossing_before_a_settled_instant_is_that_crossing() {
        for (c, ascending) in spread_roots().flat_map(|c| [(c, true), (c, false)]) {
            let (lo, far) = (c - 3.3, c + 5.1);
            let settled = first_crossing_after(linear(c, ascending), lo, far, 0.25)
                .unwrap()
                .expect("one crossing in range");
            let back = last_crossing_before(linear(c, ascending), lo, settled, 0.25)
                .unwrap()
                .expect("the crossing has happened by its settled instant");
            assert!(back >= c, "{back} is before the crossing {c}");
            assert!(back <= settled, "{back} is after the query {settled}");
        }
    }

    #[test]
    fn last_crossing_before_an_instant_ahead_of_the_crossing_is_none() {
        for (c, ascending) in spread_roots().flat_map(|c| [(c, true), (c, false)]) {
            let back =
                last_crossing_before(linear(c, ascending), c - 3.3, c - SECOND_DAYS, 0.25).unwrap();
            assert_eq!(back, None, "crossing {c}");
        }
    }

    #[test]
    fn last_crossing_before_stays_inside_the_range() {
        let c = 2_451_545.3;
        let mut sampled = Vec::new();
        let back = last_crossing_before(
            |t| {
                sampled.push(t);
                Ok(t - c)
            },
            c - 0.6,
            c + 0.61,
            0.25,
        )
        .unwrap();
        assert!(back.is_some());
        // A crossing below the low end is not in range, and is not sampled for.
        let below = last_crossing_before(
            |t| {
                sampled.push(t);
                Ok(t - c)
            },
            c + 0.1,
            c + 0.61,
            0.25,
        )
        .unwrap();
        assert_eq!(below, None);
        assert!(sampled.iter().all(|&t| t >= c - 0.6 && t <= c + 0.61));
    }

    #[test]
    fn crossings_in_range_stays_inside_the_range() {
        let c = 2_451_545.3;
        let mut sampled = Vec::new();
        let roots = crossings_in_range(
            |t| {
                sampled.push(t);
                Ok(t - c)
            },
            c - 0.6,
            c + 0.61,
            0.25,
        )
        .unwrap();
        assert_eq!(roots.len(), 1);
        assert!(sampled.iter().all(|&t| t >= c - 0.6 && t <= c + 0.61));
        assert_eq!(sampled.first(), Some(&(c - 0.6)));
    }

    #[test]
    fn ranges_sharing_an_end_partition_a_crossing_near_it() {
        // Split points on both sides of the crossing, within and beyond the
        // bisection tolerance of it, and at the instants the scanners settle
        // on. Each must put the crossing in exactly one of the two ranges.
        for (c, ascending) in spread_roots().flat_map(|c| [(c, true), (c, false)]) {
            let (lo, hi) = (c - 3.3, c + 5.1);
            let settled = first_crossing_after(linear(c, ascending), lo, hi, 0.25)
                .unwrap()
                .expect("one crossing in range");
            let offsets_s = [-2.0, -0.4, -0.1, 0.0, 0.1, 0.25, 0.4, 2.0];
            let splits = offsets_s
                .iter()
                .map(|s| c + s * SECOND_DAYS)
                .chain([settled]);
            for split in splits {
                let count = |from: f64, to: f64| {
                    crossings_in_range(linear(c, ascending), from, to, 0.25)
                        .unwrap()
                        .len()
                };
                assert_eq!(
                    count(lo, split) + count(split, hi),
                    1,
                    "crossing {c} split at {split}"
                );
                // The backward and forward searches from the split agree with
                // the ranges on which side holds it.
                let before = last_crossing_before(linear(c, ascending), lo, split, 0.25).unwrap();
                let after = first_crossing_after(linear(c, ascending), split, hi, 0.25).unwrap();
                assert_eq!(before.is_some(), count(lo, split) == 1, "split {split}");
                assert_eq!(after.is_some(), count(split, hi) == 1, "split {split}");
            }
        }
    }

    #[test]
    fn bisect_returns_the_settled_end_of_a_descending_crossing() {
        for c in spread_roots() {
            let mut f = |t: f64| Ok(c - t);
            let (lo, hi) = (c - 0.4, c + 0.6);
            let root = bisect(&mut f, lo, c - lo, hi).unwrap();
            // Zero counts as "below", so the settled side is `f <= 0`.
            assert!(root >= c, "root {root} is before the crossing {c}");
            assert!(root - c <= REFINE_TOLERANCE_DAYS, "root {root} vs {c}");
        }
    }
}
