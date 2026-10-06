//! The level-crossing scanner on synthetic functions.

use super::{
    crossings_in_range, first_level_crossing_after, last_level_crossing_before,
    level_crossings_in_range, wrap180, LEVEL_CHUNK_STEPS, REFINE_TOLERANCE_DAYS,
};
use crate::error::EventError;
use std::cell::Cell;

const T0: f64 = 2_451_545.0;

/// A returned instant is settled: at or after the root, by less than the
/// bisection tolerance. The slack covers Julian-day rounding (ulp 4.7e-10).
fn assert_settled(got: f64, want: f64) {
    let late = got - want;
    assert!(
        (-1e-8..=REFINE_TOLERANCE_DAYS + 1e-8).contains(&late),
        "got {got}, want {want}, late by {late} d"
    );
}

fn assert_roots(got: &[f64], want: &[f64]) {
    assert_eq!(got.len(), want.len(), "got {got:?}, want {want:?}");
    for (g, w) in got.iter().zip(want) {
        assert_settled(*g, *w);
    }
}

/// A separation that peaks at `peak_deg` at `centre` and falls away as
/// 0.01 deg/day^2: a pair that closes on a level and turns back.
fn parabola(centre: f64, peak_deg: f64) -> impl Fn(f64) -> Result<f64, EventError> {
    move |jd| Ok(peak_deg - 0.01 * (jd - centre).powi(2))
}

#[test]
fn finds_a_single_crossing() {
    let d = |jd: f64| Ok(wrap180(jd - T0 - 10.0));
    let roots = level_crossings_in_range(d, &[0.0], T0, T0 + 30.0, 1.0).unwrap();
    assert_roots(&roots, &[T0 + 10.0]);
}

#[test]
fn a_turn_back_short_of_the_level_is_no_event() {
    let d = parabola(T0 + 10.7, -0.001);
    assert!(level_crossings_in_range(d, &[0.0], T0, T0 + 30.0, 2.0)
        .unwrap()
        .is_empty());
}

// The overshoot is 0.0004 deg, so the two roots are 0.2 day either side of
// the peak, inside one 2-day step. The plain sign-change scan sees the same
// sign at both ends of that step. A peak midway between two grid points
// (T0 + 11) leaves equal samples either side of it.
#[test]
fn finds_two_crossings_inside_one_step() {
    for centre in [T0 + 11.0, T0 + 10.7, T0 + 10.25, T0 + 11.75] {
        let plain = crossings_in_range(parabola(centre, 0.0004), T0, T0 + 30.0, 2.0).unwrap();
        assert!(plain.is_empty(), "centre {centre}: {plain:?}");
        let roots =
            level_crossings_in_range(parabola(centre, 0.0004), &[0.0], T0, T0 + 30.0, 2.0).unwrap();
        assert_roots(&roots, &[centre - 0.2, centre + 0.2]);
    }
}

#[test]
fn a_turning_point_on_a_grid_point_gives_no_duplicates() {
    let centre = T0 + 10.0;
    let roots =
        level_crossings_in_range(parabola(centre, 0.0004), &[0.0], T0, T0 + 30.0, 2.0).unwrap();
    assert_roots(&roots, &[centre - 0.2, centre + 0.2]);
}

// A retrograde loop: the separation crosses the level, turns, and crosses it
// twice more 0.3 day apart inside one step. The two turning points are 6.8
// days apart, in different steps.
#[test]
fn finds_three_crossings_when_two_share_a_step() {
    let centre = T0 + 20.5;
    let d = move |jd: f64| {
        let x = jd - centre;
        Ok(1e-3 * (x + 10.0) * x * (x - 0.3))
    };
    let plain = crossings_in_range(d, T0, T0 + 30.0, 2.0).unwrap();
    assert_eq!(plain.len(), 1, "{plain:?}");
    let roots = level_crossings_in_range(d, &[0.0], T0, T0 + 30.0, 2.0).unwrap();
    assert_roots(&roots, &[centre - 10.0, centre, centre + 0.3]);
}

#[test]
fn a_level_at_the_wrap_seam_is_found_and_the_seam_is_not_a_root() {
    // 170 deg at T0, rising 1 deg/day: reaches 180 at T0 + 10 and wraps.
    let d = |jd: f64| Ok(wrap180(170.0 + (jd - T0)));
    let at_seam = level_crossings_in_range(d, &[180.0], T0, T0 + 30.0, 1.0).unwrap();
    assert_roots(&at_seam, &[T0 + 10.0]);
    let at_zero = level_crossings_in_range(d, &[0.0], T0, T0 + 30.0, 1.0).unwrap();
    assert!(at_zero.is_empty(), "{at_zero:?}");
}

#[test]
fn two_levels_come_back_in_time_order() {
    let d = |jd: f64| Ok(wrap180(-95.0 + (jd - T0)));
    let roots = level_crossings_in_range(d, &[90.0, -90.0], T0, T0 + 200.0, 1.0).unwrap();
    assert_roots(&roots, &[T0 + 5.0, T0 + 185.0]);
}

#[test]
fn two_levels_crossed_in_one_bracket_come_back_in_time_order() {
    let d = |jd: f64| Ok(jd - T0 - 10.5);
    let roots = level_crossings_in_range(d, &[0.3, -0.3], T0, T0 + 30.0, 2.0).unwrap();
    assert_roots(&roots, &[T0 + 10.2, T0 + 10.8]);
}

#[test]
fn first_after_is_the_first_in_range() {
    let centre = T0 + 10.7;
    let all =
        level_crossings_in_range(parabola(centre, 0.0004), &[0.0], T0, T0 + 30.0, 2.0).unwrap();
    let first =
        first_level_crossing_after(parabola(centre, 0.0004), &[0.0], T0, T0 + 30.0, 2.0).unwrap();
    assert_eq!(first, all.first().copied());
    let none =
        first_level_crossing_after(parabola(centre, -0.001), &[0.0], T0, T0 + 30.0, 2.0).unwrap();
    assert_eq!(none, None);
}

#[test]
fn a_root_past_the_end_of_the_range_is_not_returned() {
    let d = |jd: f64| Ok(jd - T0 - 10.5);
    let roots = level_crossings_in_range(d, &[0.0], T0, T0 + 10.2, 2.0).unwrap();
    assert!(roots.is_empty(), "{roots:?}");
}

#[test]
fn empty_and_inverted_ranges_give_nothing_but_still_sample_the_start() {
    let d = |jd: f64| Ok(wrap180(jd - T0 - 10.0));
    assert!(
        level_crossings_in_range(d, &[0.0], T0 + 20.0, T0 + 20.0, 1.0)
            .unwrap()
            .is_empty()
    );
    assert!(level_crossings_in_range(d, &[0.0], T0 + 20.0, T0, 1.0)
        .unwrap()
        .is_empty());
    let failing = |_: f64| Err(EventError::Backend("boom".into()));
    assert_eq!(
        level_crossings_in_range(failing, &[0.0], T0 + 20.0, T0, 1.0),
        Err(EventError::Backend("boom".into()))
    );
}

// The scan looks one step behind its start and less than two steps past its
// end; callers clamp their range two steps inside the window on that basis.
#[test]
fn samples_stay_within_one_step_before_and_two_after_the_range() {
    let (lo, hi, step) = (T0, T0 + 29.3, 2.0);
    let earliest = Cell::new(f64::INFINITY);
    let latest = Cell::new(f64::NEG_INFINITY);
    let d = |jd: f64| {
        earliest.set(earliest.get().min(jd));
        latest.set(latest.get().max(jd));
        parabola(T0 + 28.9, 0.0004)(jd)
    };
    level_crossings_in_range(d, &[0.0], lo, hi, step).unwrap();
    assert_eq!(earliest.get(), lo - step);
    assert!(latest.get() < hi + 2.0 * step, "{}", latest.get());
}

// The backward search (issue #168 (d)) runs the forward scanner over chunks
// of `LEVEL_CHUNK_STEPS` steps, latest first.

/// A sawtooth separation climbing 1 deg/day through 0 at `T0 + 10 + 360k`.
fn sawtooth(jd: f64) -> Result<f64, EventError> {
    Ok(wrap180(jd - T0 - 10.0))
}

#[test]
fn last_level_crossing_is_the_last_one_in_range() {
    // A range of many chunks holding three roots.
    let (lo, hi, step) = (T0, T0 + 1000.0, 1.0);
    let all = level_crossings_in_range(sawtooth, &[0.0], lo, hi, step).unwrap();
    assert_roots(&all, &[T0 + 10.0, T0 + 370.0, T0 + 730.0]);
    let last = last_level_crossing_before(sawtooth, &[0.0], lo, hi, step).unwrap();
    assert_settled(last.expect("three roots in range"), T0 + 730.0);
    // Ending before a root finds the one before it; before the first, none.
    let earlier = last_level_crossing_before(sawtooth, &[0.0], lo, T0 + 729.0, step).unwrap();
    assert_settled(earlier.expect("two roots in range"), T0 + 370.0);
    assert_eq!(
        last_level_crossing_before(sawtooth, &[0.0], lo, T0 + 9.0, step).unwrap(),
        None
    );
}

#[test]
fn last_level_crossing_finds_both_sides_of_a_turn_back() {
    // The pair overshoots the level by 0.0004 deg inside one step: two roots
    // 0.2 day either side of the peak, in the second chunk back (the seam is
    // at T0 + 12). The later one is the last.
    let d = parabola(T0 + 6.9, 0.0004);
    let last = last_level_crossing_before(d, &[0.0], T0, T0 + 140.0, 2.0).unwrap();
    assert_settled(last.expect("two roots in range"), T0 + 7.1);
}

#[test]
fn last_level_crossing_is_not_lost_at_a_chunk_seam() {
    // Put a chunk seam within the bisection tolerance of the root, on either
    // side of it and exactly on it. A root just below a seam settles just
    // above it, in neither chunk's own range.
    let step = 1.0;
    let root = T0 + 10.0;
    let chunk = LEVEL_CHUNK_STEPS * step;
    for seam_offset_s in [-0.4, -0.2, -0.01, 0.0, 0.01, 0.2, 0.4] {
        for chunks_above in [1.0, 2.0] {
            let seam = root + seam_offset_s / 86_400.0;
            let hi = seam + chunks_above * chunk;
            let last = last_level_crossing_before(sawtooth, &[0.0], T0, hi, step).unwrap();
            assert_settled(
                last.unwrap_or_else(|| panic!("root lost with a seam {seam_offset_s} s from it")),
                root,
            );
        }
    }
}

#[test]
fn last_level_crossing_samples_stay_within_the_forward_scanners_bounds() {
    let (lo, hi, step) = (T0, T0 + 500.3, 2.0);
    let earliest = Cell::new(f64::INFINITY);
    let latest = Cell::new(f64::NEG_INFINITY);
    let d = |jd: f64| {
        earliest.set(earliest.get().min(jd));
        latest.set(latest.get().max(jd));
        // Never reaches the level: the search walks the whole range.
        Ok(5.0)
    };
    assert_eq!(
        last_level_crossing_before(d, &[0.0], lo, hi, step).unwrap(),
        None
    );
    assert_eq!(earliest.get(), lo - step);
    assert!(latest.get() < hi + 2.0 * step, "{}", latest.get());
}

#[test]
fn last_level_crossing_on_an_empty_range_still_samples_its_start() {
    assert_eq!(
        last_level_crossing_before(sawtooth, &[0.0], T0 + 20.0, T0, 1.0).unwrap(),
        None
    );
    let failing = |_: f64| Err(EventError::Backend("boom".into()));
    assert_eq!(
        last_level_crossing_before(failing, &[0.0], T0 + 20.0, T0, 1.0),
        Err(EventError::Backend("boom".into()))
    );
}

// Both ends of a range decide membership by the sign of `d − level` there,
// as `crossings_in_range` does (issue #168): comparing a settled root with
// an end drops a crossing that settles just past it.

#[test]
fn ranges_sharing_an_end_hold_each_crossing_once() {
    // Put the shared end within the bisection tolerance of the root, on
    // either side of it and exactly on it.
    let root = T0 + 10.0;
    for offset_s in [-0.6, -0.4, -0.2, -0.01, 0.0, 0.01, 0.2, 0.4, 0.6] {
        let shared = root + offset_s / 86_400.0;
        let below = level_crossings_in_range(sawtooth, &[0.0], T0, shared, 1.0).unwrap();
        let above = level_crossings_in_range(sawtooth, &[0.0], shared, T0 + 30.0, 1.0).unwrap();
        assert_eq!(
            below.len() + above.len(),
            1,
            "end {offset_s} s from the root: {below:?} {above:?}"
        );
        // The crossing has happened by the shared end exactly when the
        // separation is past the level there.
        let happened = sawtooth(shared).unwrap() > 0.0;
        assert_eq!(below.len() == 1, happened, "end {offset_s} s from the root");
        for &found in below.iter().chain(&above) {
            assert_settled(found, root);
        }
        assert!(below.iter().all(|&found| found <= shared), "{below:?}");
    }
}

#[test]
fn a_settled_instant_as_an_end_keeps_its_crossing_below_it() {
    // Anchors off the forward grid, so the two searches bisect differently.
    let (lo, hi, step) = (T0 + 0.3, T0 + 1000.0, 1.0);
    let roots = [T0 + 10.0, T0 + 370.0, T0 + 730.0];
    let all = level_crossings_in_range(sawtooth, &[0.0], T0, hi, step).unwrap();
    assert_roots(&all, &roots);
    for (index, &settled) in all.iter().enumerate() {
        let last = last_level_crossing_before(sawtooth, &[0.0], lo, settled, step).unwrap();
        let last = last.unwrap_or_else(|| panic!("root {index} lost at its settled instant"));
        assert_settled(last, roots[index]);
        assert!(last <= settled, "{last} is past {settled}");
        let next = first_level_crossing_after(sawtooth, &[0.0], settled, hi, step).unwrap();
        match roots.get(index + 1) {
            Some(&following) => assert_settled(next.expect("a following root"), following),
            None => assert_eq!(next, None),
        }
    }
}

#[test]
fn an_end_inside_a_turn_back_keeps_the_crossings_before_it() {
    // Roots 0.2 day either side of the peak, inside one 2-day step that the
    // range ends in.
    let centre = T0 + 10.7;
    for (end_offset, want) in [(-0.3, 0), (-0.1, 1), (0.0, 1), (0.1, 1), (0.3, 2)] {
        let end = centre + end_offset;
        let roots =
            level_crossings_in_range(parabola(centre, 0.0004), &[0.0], T0, end, 2.0).unwrap();
        assert_eq!(roots.len(), want, "end {end_offset} d from the peak");
        assert!(roots.iter().all(|&found| found <= end), "{roots:?}");
        // The same turn back with the range starting inside it.
        let rest = level_crossings_in_range(parabola(centre, 0.0004), &[0.0], end, T0 + 30.0, 2.0)
            .unwrap();
        assert_eq!(
            roots.len() + rest.len(),
            2,
            "end {end_offset} d from the peak"
        );
    }
}
