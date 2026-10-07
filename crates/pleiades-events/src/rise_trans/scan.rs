//! Directed-crossing scanner for the observer-local events (rise, set, and
//! meridian transit): a coarse time grid anchored at the query instant, with
//! culmination insertion, crossing direction read from the bracket signs, and
//! ITP (`root::refine_itp`) refinement to the shared `root::REFINE_TOLERANCE_DAYS`.
//!
//! Why not `root::first_crossing_after` and friends: those are generic
//! "any sign change" scanners tuned for wrapped longitude residuals, and the
//! rise/set path used them at a 2-minute step so that a fast-Moon graze
//! could not slip between two samples. That made every rise/set search
//! linear in the distance to the event at ~30 samples per hour (issue #70).
//! Their backward twin also walks a window-anchored grid, so whether an event
//! within the refinement tolerance of the query instant counts as before it
//! is decided by comparing two independently refined roots. Here every walk
//! is anchored at the query instant and that question is settled by the
//! residual's sign there (issues #80, #81; see [`walk`]).
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
//!
//! The meridian-transit searches reuse the scanner on the wrapped hour-angle
//! residual, asking for ascending crossings. That residual climbs steadily
//! through its zero and drops 360° at the wrap seam, so the seam is a
//! descending bracket (skipped) and no three same-sign samples ever form an
//! extremum (the culmination search never runs).

use crate::error::EventError;
use crate::root::refine_itp;
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

/// Golden-section search over `[a, b]` for the extremum of `f` that maximises
/// `sign · f`, starting from the best sample already in hand. Returns the best
/// sample seen once the bracket is narrower than
/// [`CULMINATION_TOLERANCE_DAYS`].
fn refine_extremum<F>(
    f: &mut F,
    mut a: f64,
    mut b: f64,
    sign: f64,
    mut best: Sample,
) -> Result<Sample, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
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
    Ok(best)
}

/// Refines the culmination the strict extremum `s1` sits next to, given its
/// neighbours, and returns the refined culmination sample if it lies on the
/// OTHER side of the horizon from the three grid samples — i.e. if it grazes.
/// `None` means no crossing hides between the samples.
///
/// `equally_spaced` says whether `s0`, `s1` and `s2` are one grid step apart.
/// They are not when `s0` or `s2` is a sample taken at a window limit, and
/// then the culmination is always refined: the parabola estimate that skips
/// the refinement assumes equal spacing.
fn refine_grazing_culmination<F>(
    f: &mut F,
    s0: Sample,
    s1: Sample,
    s2: Sample,
    equally_spaced: bool,
) -> Result<Option<Sample>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    if equally_spaced {
        // Parabola through three equally spaced samples: its vertex value is a
        // cheap, evaluation-free estimate of the culmination's residual.
        let curvature = s0.f - 2.0 * s1.f + s2.f; // nonzero for a strict extremum
        let vertex_value = s1.f - (s0.f - s2.f).powi(2) / (8.0 * curvature);
        if below(vertex_value) == below(s1.f) && vertex_value.abs() > GRAZE_MARGIN_DEG {
            return Ok(None);
        }
    }
    // Golden-section search for the extremum over [s0, s2], which holds
    // exactly one culmination by the smoothness argument in the module doc.
    // Maximise `sign * f` so one search serves both maxima and minima.
    let sign = if s1.f > s0.f { 1.0 } else { -1.0 };
    let best = refine_extremum(f, s0.jd, s2.jd, sign, s1)?;
    Ok((below(best.f) != below(s1.f)).then_some(best))
}

/// Looks for a grazing culmination between `inner` and `edge`, where `edge`
/// is the last sample the window allows on that side, and returns the refined
/// culmination if it lies on the other side of the horizon from both.
///
/// Away from the window's ends a culmination shows up as a strict extremum of
/// three grid samples (see [`walk`]). Next to an end there is no third sample
/// beyond `edge`, so a culmination nearer to `edge` than to `inner` leaves
/// only this trace: the residual closes on the horizon towards `edge` and is
/// within [`GRAZE_MARGIN_DEG`] of it there. Half a grid step from its
/// culmination a diurnal residual is less than 0.5° from its peak value, so
/// the margin cannot miss a graze. A culmination nearer to `inner` is a strict
/// extremum at `inner` and is left to the three-sample check, so the two never
/// report the same graze.
fn refine_edge_culmination<F>(
    f: &mut F,
    inner: Sample,
    edge: Sample,
) -> Result<Option<Sample>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    if below(inner.f) != below(edge.f)
        || edge.f.abs() > GRAZE_MARGIN_DEG
        || edge.f.abs() >= inner.f.abs()
    {
        return Ok(None);
    }
    // A graze here is an extremum that reaches across the horizon.
    let sign = if below(edge.f) { 1.0 } else { -1.0 };
    let (earlier, later) = time_ordered(inner, edge);
    let best = refine_extremum(f, earlier.jd, later.jd, sign, edge)?;
    Ok((below(best.f) != below(edge.f)).then_some(best))
}

/// The instants a walk may sample: the ephemeris window. A walk never
/// evaluates the residual outside them.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Limits {
    /// Earliest instant that may be sampled (Julian Day).
    pub(crate) earliest: f64,
    /// Latest instant that may be sampled (Julian Day).
    pub(crate) latest: f64,
}

impl Limits {
    /// No limits, for a residual defined at every instant.
    #[cfg(test)]
    pub(crate) const NONE: Limits = Limits {
        earliest: f64::NEG_INFINITY,
        latest: f64::INFINITY,
    };
}

/// Samples `f` at `jd`, or returns `None` where the window does not reach:
/// `jd` lies outside `limits`, or the residual itself reports
/// [`EventError::OutOfWindow`] (an apparent place within light-time of the
/// window's start reads before it). Any other error propagates.
fn read<F>(f: &mut F, jd: f64, limits: Limits) -> Result<Option<Sample>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    if jd < limits.earliest || jd > limits.latest {
        return Ok(None);
    }
    match f(jd) {
        Ok(value) => Ok(Some(Sample { jd, f: value })),
        Err(EventError::OutOfWindow { .. }) => Ok(None),
        Err(error) => Err(error),
    }
}

/// What a walk covers: the range searched, the grid step, and the window.
#[derive(Clone, Copy, Debug)]
struct Search {
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
    limits: Limits,
}

/// Whether a walk examined the whole range it was given.
#[derive(Clone, Copy, Debug)]
enum Coverage {
    /// Every interval of the range was examined, or the visitor stopped the
    /// walk at a root.
    Complete,
    /// The window ended before the range did. `needed_jd` is the first grid
    /// node the walk could not sample.
    CutShort { needed_jd: f64 },
}

/// The two brackets either side of a grazing culmination, in walk order.
fn graze_brackets(
    left: Sample,
    peak: Sample,
    right: Sample,
    backward: bool,
) -> [(Sample, Sample); 2] {
    if backward {
        [(peak, right), (left, peak)]
    } else {
        [(left, peak), (peak, right)]
    }
}

/// Walks the grid `anchor + k·step` (forward from `lo_jd`, or backward from
/// `hi_jd`), refining every bracket of the wanted direction whose root lies
/// in `[lo_jd, hi_jd]` and handing each root to `visit` in walk order until
/// `visit` breaks or the grid is exhausted.
///
/// # Guard samples
///
/// A culmination shows up as a strict extremum of three consecutive samples,
/// so the sample nearest to it needs a neighbour on both sides. The grid is
/// therefore extended by one guard sample beyond each end of the walk: one
/// step behind the anchor, and one step past the sample that overshoots the
/// far end. Without them a night (or day) shorter than the step goes unseen
/// whenever it falls in the first or the last grid interval with the end
/// sample as its nearest one (issue #81).
///
/// Guards inform culmination detection only. A bracket is refined only if it
/// lies on the walk's side of the anchor and starts before the far end, so a
/// crossing behind the anchor costs no refinement and is never reported.
/// Which side of the anchor a crossing falls on is thus decided by the
/// residual's sign AT the anchor, exactly, not by comparing a refined root
/// against it: together with `root::refine_itp` returning the settled end of its
/// bracket, that is what lets a search anchored at a returned instant step
/// past the event it describes (issue #80).
///
/// Like `root::crossings_in_range`, the anchor is always evaluated once, so
/// a backend error there propagates even on an empty range.
///
/// # The window's ends
///
/// No sample is taken outside `limits` (issue #203). Where the grid would
/// step past a limit before the range is covered, the walk takes one last
/// sample at the limit itself, so the partial interval before it is still
/// bracketed. Where a guard sample cannot be taken, the interval next to the
/// end is checked for a grazing culmination by [`refine_edge_culmination`]
/// in place of the three-sample check. A node the residual cannot be read at
/// ([`read`]) ends the walk the same way, without the sample at the limit.
/// The walk reports [`Coverage::CutShort`] when it ends this way short of the
/// range's far end.
fn walk<F, V>(
    f: &mut F,
    search: Search,
    backward: bool,
    want_ascending: bool,
    mut visit: V,
) -> Result<Coverage, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
    V: FnMut(f64) -> ControlFlow<()>,
{
    let Search {
        lo_jd,
        hi_jd,
        step_days,
        limits,
    } = search;
    let (anchor, direction) = if backward {
        (hi_jd, -1.0)
    } else {
        (lo_jd, 1.0)
    };
    let node = |k: f64| anchor + direction * k * step_days;
    let past_far_end = |jd: f64| {
        if backward {
            jd < lo_jd
        } else {
            jd > hi_jd
        }
    };
    let reaches_far_end = |jd: f64| {
        if backward {
            jd <= lo_jd
        } else {
            jd >= hi_jd
        }
    };
    let far_limit = if backward {
        limits.earliest
    } else {
        limits.latest
    };
    let in_range = |jd: f64| jd >= lo_jd && jd <= hi_jd;
    // Whether a time-ordered bracket lies wholly on the walk's side of the
    // anchor and reaches into the range.
    let searchable = |earlier: &Sample, later: &Sample| {
        if backward {
            later.jd <= hi_jd && later.jd >= lo_jd
        } else {
            earlier.jd >= lo_jd && earlier.jd <= hi_jd
        }
    };

    // Refines one time-ordered bracket and reports its root if wanted and in
    // range. Returns `true` when the visitor asked to stop.
    let mut emit = |f: &mut F, earlier: Sample, later: Sample| -> Result<bool, EventError> {
        if ascending(&earlier, &later) != want_ascending || !searchable(&earlier, &later) {
            return Ok(false);
        }
        let root = refine_itp(f, earlier.jd, earlier.f, later.jd, later.f)?;
        if !in_range(root) {
            return Ok(false);
        }
        Ok(visit(root).is_break())
    };

    let anchor_sample = Sample {
        jd: anchor,
        f: f(anchor)?,
    };
    if lo_jd > hi_jd {
        return Ok(Coverage::Complete);
    }
    // `None` when the window ends within a step behind the anchor.
    let mut prev2 = read(f, node(-1.0), limits)?;
    let mut prev1 = anchor_sample;
    // The first grid node the walk could not sample.
    let mut needed_jd = None;
    let mut k = 1.0_f64;
    loop {
        // `prev2` is the sample two nodes back. Once it is past the far end,
        // the interval behind `prev1` holds nothing in range and `prev1`
        // has already served as the far guard.
        if prev2.is_some_and(|sample| past_far_end(sample.jd)) {
            return Ok(Coverage::Complete);
        }
        let jd = node(k);
        // The next sample: the grid node, or, where the node lies beyond the
        // window with part of the range still uncovered, the limit itself.
        let (next, at_node) = match read(f, jd, limits)? {
            Some(sample) => (Some(sample), true),
            None => {
                let beyond_window = jd < limits.earliest || jd > limits.latest;
                let first_miss = needed_jd.is_none();
                needed_jd.get_or_insert(jd);
                if beyond_window
                    && first_miss
                    && !reaches_far_end(prev1.jd)
                    && prev1.jd != far_limit
                {
                    (read(f, far_limit, limits)?, false)
                } else {
                    (None, false)
                }
            }
        };
        let Some(cur) = next else {
            // The window ends at `prev1`.
            if let Some(inner) = prev2 {
                if let Some(peak) = refine_edge_culmination(f, inner, prev1)? {
                    let (left, right) = time_ordered(inner, prev1);
                    for (earlier, later) in graze_brackets(left, peak, right, backward) {
                        if emit(f, earlier, later)? {
                            return Ok(Coverage::Complete);
                        }
                    }
                }
            }
            return Ok(if reaches_far_end(prev1.jd) {
                Coverage::Complete
            } else {
                Coverage::CutShort {
                    needed_jd: needed_jd.unwrap_or(jd),
                }
            });
        };
        if below(prev1.f) != below(cur.f) {
            let (earlier, later) = time_ordered(prev1, cur);
            if emit(f, earlier, later)? {
                return Ok(Coverage::Complete);
            }
        } else {
            // No sign change across this interval; only a grazing culmination
            // could still hide a pair of crossings here.
            let graze = match prev2 {
                // All three samples share a sign (a sign change between
                // `prev2` and `prev1` was handled on the previous step).
                Some(prev2)
                    if below(prev2.f) == below(prev1.f)
                        && is_strict_extremum(&prev2, &prev1, &cur) =>
                {
                    let (s0, s2) = time_ordered(prev2, cur);
                    refine_grazing_culmination(f, s0, prev1, s2, at_node)?.map(|peak| {
                        if peak.jd < prev1.jd {
                            (s0, peak, prev1)
                        } else {
                            (prev1, peak, s2)
                        }
                    })
                }
                Some(_) => None,
                // `prev1` is the anchor and has no guard behind it.
                None => refine_edge_culmination(f, cur, prev1)?.map(|peak| {
                    let (left, right) = time_ordered(prev1, cur);
                    (left, peak, right)
                }),
            };
            if let Some((left, peak, right)) = graze {
                for (earlier, later) in graze_brackets(left, peak, right, backward) {
                    if emit(f, earlier, later)? {
                        return Ok(Coverage::Complete);
                    }
                }
            }
        }
        prev2 = Some(prev1);
        prev1 = cur;
        k += 1.0;
    }
}

/// The answer of a search for one root: the root, or `OutOfWindow` when the
/// window ended before the search had covered its range and no root was
/// found. `Ok(None)` then means the range was examined and holds no root.
fn found_or_cut_short(found: Option<f64>, coverage: Coverage) -> Result<Option<f64>, EventError> {
    match (found, coverage) {
        (None, Coverage::CutShort { needed_jd }) => Err(EventError::OutOfWindow {
            julian_day: needed_jd,
        }),
        (found, _) => Ok(found),
    }
}

/// Every root of the wanted direction in `[lo_jd, hi_jd]`, ascending in time.
/// A crossing is past `lo_jd` if the residual still carries its pre-crossing
/// sign there, so one that a previous search settled at `lo_jd` is excluded;
/// at `hi_jd` the refined root itself is compared.
///
/// Fails with `OutOfWindow` when `limits` end before `hi_jd`.
pub(crate) fn directed_crossings_in_range<F>(
    mut f: F,
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
    limits: Limits,
    want_ascending: bool,
) -> Result<Vec<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    let mut out = Vec::new();
    let coverage = walk(
        &mut f,
        Search {
            lo_jd,
            hi_jd,
            step_days,
            limits,
        },
        false,
        want_ascending,
        |root| {
            out.push(root);
            ControlFlow::Continue(())
        },
    )?;
    match coverage {
        Coverage::Complete => Ok(out),
        Coverage::CutShort { needed_jd } => Err(EventError::OutOfWindow {
            julian_day: needed_jd,
        }),
    }
}

/// The first root of the wanted direction in `[lo_jd, hi_jd]`, or `None`.
/// Early-terminating: stops as soon as that root is refined. A crossing is
/// past `lo_jd` if the residual still carries its pre-crossing sign there.
///
/// Fails with `OutOfWindow` when `limits` end before `hi_jd` and no root lies
/// before them.
pub(crate) fn first_directed_crossing_after<F>(
    mut f: F,
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
    limits: Limits,
    want_ascending: bool,
) -> Result<Option<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    let mut found = None;
    let coverage = walk(
        &mut f,
        Search {
            lo_jd,
            hi_jd,
            step_days,
            limits,
        },
        false,
        want_ascending,
        |root| {
            found = Some(root);
            ControlFlow::Break(())
        },
    )?;
    found_or_cut_short(found, coverage)
}

/// The last root of the wanted direction in `[lo_jd, hi_jd]`, or `None`.
/// Early-terminating: walks the grid backward from `hi_jd` and stops as soon
/// as that root is refined. A crossing is at or before `hi_jd` if the
/// residual already carries its post-crossing sign there. Its grid is
/// anchored at `hi_jd`, not `lo_jd`, so it brackets different intervals from
/// [`directed_crossings_in_range`]; the two agree on the root to within the
/// refinement tolerance, not bit-for-bit.
///
/// Fails with `OutOfWindow` when `limits` begin after `lo_jd` and no root
/// lies after them.
pub(crate) fn last_directed_crossing_before<F>(
    mut f: F,
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
    limits: Limits,
    want_ascending: bool,
) -> Result<Option<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    let mut found = None;
    let coverage = walk(
        &mut f,
        Search {
            lo_jd,
            hi_jd,
            step_days,
            limits,
        },
        true,
        want_ascending,
        |root| {
            found = Some(root);
            ControlFlow::Break(())
        },
    )?;
    found_or_cut_short(found, coverage)
}

#[cfg(test)]
mod tests;
