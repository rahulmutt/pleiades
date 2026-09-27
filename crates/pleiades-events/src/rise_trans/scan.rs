//! Horizon-crossing scanner for rise/set: a coarse time grid with culmination
//! insertion, crossing direction read from the bracket signs, and bisection
//! to the shared `root::REFINE_TOLERANCE_DAYS`.
//!
//! Why not `root::first_crossing_after` and friends: those are generic
//! "any sign change" scanners tuned for wrapped longitude residuals, and the
//! rise/set path used them at a 2-minute step so that a fast-Moon graze
//! could not slip between two samples. That made every rise/set search
//! linear in the distance to the event at ~30 samples per hour (issue #70).
//!
//! The altitude residual is smooth on the day scale: it is a sinusoid in
//! hour angle with one culmination (maximum) and one anti-culmination
//! (minimum) per sidereal day, plus slow drift for the Moon. Two zeros can
//! therefore only hide inside one grid step when a culmination pokes
//! marginally across the horizon between two samples — a graze. The same
//! smoothness means a discrete local extremum of the grid samples always sits
//! next to the true culmination, so the scanner can (a) step coarsely, (b)
//! notice every candidate graze from three consecutive samples, and (c)
//! refine only those culminations that come within [`GRAZE_MARGIN_DEG`] of the
//! horizon, inserting the refined culmination as an extra grid node whose two
//! neighbouring brackets then carry the graze's rise and set. This is the
//! shape of Swiss Ephemeris's own `swe_rise_trans` (`swecl.c`: a 2-hour grid
//! with refined culmination points inserted into the height array).
//!
//! A bracket's direction is the sign pattern across it — below-to-above is
//! ascending (rise), above-to-below is descending (set) — so callers never
//! probe past a refined root to classify it, and brackets of the unwanted
//! direction are skipped without being refined.

use crate::error::EventError;
use crate::root::bisect;
use std::ops::ControlFlow;

/// A culmination whose parabola-estimated residual lies within this many
/// degrees of the horizon is refined before deciding whether it grazes. The
/// three-sample parabola through a diurnal sinusoid sampled hourly
/// misestimates the peak by well under 0.1°, so 1° is a generous guard;
/// culminations further from the horizon than this cannot hide a crossing.
const GRAZE_MARGIN_DEG: f64 = 1.0;

/// Golden-section refinement of a culmination stops once its bracket is
/// narrower than this (30 s). Near the peak the residual is quadratic in
/// time, so a 30 s position error moves the peak value by a negligible
/// amount; a graze shallower than that is beyond the engine's stated parity
/// anyway.
const CULMINATION_TOLERANCE_DAYS: f64 = 30.0 / 86_400.0;

/// `1 / φ`: the golden-section shrink factor.
const INV_PHI: f64 = 0.618_033_988_749_895;

#[derive(Clone, Copy, Debug)]
struct Sample {
    jd: f64,
    f: f64,
}

/// The sign predicate shared with `root.rs`: zero counts as "below".
fn below(f: f64) -> bool {
    f <= 0.0
}

/// Whether a time-ordered bracket (`earlier.jd < later.jd`) with a sign change
/// across it is ascending (a rise).
fn ascending(earlier: &Sample, later: &Sample) -> bool {
    below(earlier.f) && !below(later.f)
}

/// Orders two samples by time.
fn time_ordered(a: Sample, b: Sample) -> (Sample, Sample) {
    if a.jd <= b.jd {
        (a, b)
    } else {
        (b, a)
    }
}

/// Whether the middle of three consecutive samples is a strict local
/// extremum of the grid.
fn is_strict_extremum(s0: &Sample, s1: &Sample, s2: &Sample) -> bool {
    (s1.f > s0.f && s1.f > s2.f) || (s1.f < s0.f && s1.f < s2.f)
}

/// Refines the culmination the strict extremum `s1` sits next to, given its
/// equally spaced neighbours, and returns the refined culmination sample if
/// it lies on the OTHER side of the horizon from the three grid samples —
/// i.e. if it grazes. `None` means no crossing hides between the samples.
fn refine_grazing_culmination<F>(
    f: &mut F,
    s0: Sample,
    s1: Sample,
    s2: Sample,
) -> Result<Option<Sample>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    // Parabola through three equally spaced samples: its vertex value is a
    // cheap, evaluation-free estimate of the culmination's residual.
    let curvature = s0.f - 2.0 * s1.f + s2.f; // nonzero for a strict extremum
    let vertex_value = s1.f - (s0.f - s2.f).powi(2) / (8.0 * curvature);
    if below(vertex_value) == below(s1.f) && vertex_value.abs() > GRAZE_MARGIN_DEG {
        return Ok(None);
    }
    // Golden-section search for the extremum over [s0, s2], which holds
    // exactly one culmination by the smoothness argument in the module doc.
    // Maximise `sign * f` so one search serves both maxima and minima.
    let sign = if s1.f > s0.f { 1.0 } else { -1.0 };
    let mut best = s1;
    let (mut a, mut b) = (s0.jd, s2.jd);
    let mut x1 = b - INV_PHI * (b - a);
    let mut x2 = a + INV_PHI * (b - a);
    let mut g1 = f(x1)?;
    let mut g2 = f(x2)?;
    loop {
        for (jd, val) in [(x1, g1), (x2, g2)] {
            if sign * val > sign * best.f {
                best = Sample { jd, f: val };
            }
        }
        if (b - a).abs() <= CULMINATION_TOLERANCE_DAYS {
            break;
        }
        if sign * g1 > sign * g2 {
            b = x2;
            x2 = x1;
            g2 = g1;
            x1 = b - INV_PHI * (b - a);
            g1 = f(x1)?;
        } else {
            a = x1;
            x1 = x2;
            g1 = g2;
            x2 = a + INV_PHI * (b - a);
            g2 = f(x2)?;
        }
    }
    Ok((below(best.f) != below(s1.f)).then_some(best))
}

/// Walks the grid `anchor + k·step` (forward from `lo_jd`, or backward from
/// `hi_jd`), refining every bracket of the wanted direction whose root lies
/// in `[lo_jd, hi_jd]` and handing each root to `visit` in walk order until
/// `visit` breaks or the grid is exhausted. Like `root::crossings_in_range`,
/// the anchor is always evaluated once (so a backend error there propagates
/// even on an empty range) and the last sample may overshoot the far end by
/// up to one step; roots outside the range are dropped.
fn walk<F, V>(
    f: &mut F,
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
    backward: bool,
    want_ascending: bool,
    mut visit: V,
) -> Result<(), EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
    V: FnMut(f64) -> ControlFlow<()>,
{
    let (anchor, direction) = if backward {
        (hi_jd, -1.0)
    } else {
        (lo_jd, 1.0)
    };
    let on_grid = |jd: f64| {
        if backward {
            jd >= lo_jd - step_days
        } else {
            jd <= hi_jd + step_days
        }
    };
    let in_range = |jd: f64| jd >= lo_jd && jd <= hi_jd;

    // Refines one time-ordered bracket and reports its root if wanted and in
    // range. Returns `true` when the visitor asked to stop.
    let mut emit = |f: &mut F, earlier: Sample, later: Sample| -> Result<bool, EventError> {
        if ascending(&earlier, &later) != want_ascending {
            return Ok(false);
        }
        let root = bisect(f, earlier.jd, earlier.f, later.jd)?;
        if !in_range(root) {
            return Ok(false);
        }
        Ok(visit(root).is_break())
    };

    let mut prev2: Option<Sample> = None;
    let mut prev1: Option<Sample> = None;
    let mut k = 0.0_f64;
    loop {
        let jd = anchor + direction * k * step_days;
        if k > 0.0 && !on_grid(jd) {
            return Ok(());
        }
        let cur = Sample { jd, f: f(jd)? };
        if let Some(p1) = prev1 {
            if below(p1.f) != below(cur.f) {
                let (earlier, later) = time_ordered(p1, cur);
                if emit(f, earlier, later)? {
                    return Ok(());
                }
            } else if let Some(p2) = prev2 {
                // All three samples share a sign (a sign change between p2
                // and p1 was handled on the previous step); only a grazing
                // culmination could still hide a pair of crossings here.
                if below(p2.f) == below(p1.f) && is_strict_extremum(&p2, &p1, &cur) {
                    let (s0, s2) = time_ordered(p2, cur);
                    if let Some(peak) = refine_grazing_culmination(f, s0, p1, s2)? {
                        let (left, right) = if peak.jd < p1.jd { (s0, p1) } else { (p1, s2) };
                        let brackets = if backward {
                            [(peak, right), (left, peak)]
                        } else {
                            [(left, peak), (peak, right)]
                        };
                        for (earlier, later) in brackets {
                            if emit(f, earlier, later)? {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
        prev2 = prev1;
        prev1 = Some(cur);
        k += 1.0;
    }
}

/// Every root of the wanted direction in `[lo_jd, hi_jd]`, ascending in time.
pub(crate) fn horizon_crossings_in_range<F>(
    mut f: F,
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
    want_ascending: bool,
) -> Result<Vec<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    let mut out = Vec::new();
    walk(
        &mut f,
        lo_jd,
        hi_jd,
        step_days,
        false,
        want_ascending,
        |root| {
            out.push(root);
            ControlFlow::Continue(())
        },
    )?;
    Ok(out)
}

/// The first root of the wanted direction in `[lo_jd, hi_jd]`, or `None`.
/// Early-terminating: stops as soon as that root is refined.
pub(crate) fn first_horizon_crossing_after<F>(
    mut f: F,
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
    want_ascending: bool,
) -> Result<Option<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    let mut found = None;
    walk(
        &mut f,
        lo_jd,
        hi_jd,
        step_days,
        false,
        want_ascending,
        |root| {
            found = Some(root);
            ControlFlow::Break(())
        },
    )?;
    Ok(found)
}

/// The last root of the wanted direction in `[lo_jd, hi_jd]`, or `None`.
/// Early-terminating: walks the grid backward from `hi_jd` and stops as soon
/// as that root is refined. Its grid is anchored at `hi_jd`, not `lo_jd`, so
/// it brackets different intervals from [`horizon_crossings_in_range`]; the
/// two agree on the root to within the bisection tolerance, not bit-for-bit.
pub(crate) fn last_horizon_crossing_before<F>(
    mut f: F,
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
    want_ascending: bool,
) -> Result<Option<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    let mut found = None;
    walk(
        &mut f,
        lo_jd,
        hi_jd,
        step_days,
        true,
        want_ascending,
        |root| {
            found = Some(root);
            ControlFlow::Break(())
        },
    )?;
    Ok(found)
}

#[cfg(test)]
mod tests;
