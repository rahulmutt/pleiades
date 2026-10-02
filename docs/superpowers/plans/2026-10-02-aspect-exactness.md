# Exact-Aspect Event Finder Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `EventEngine::aspects_in_range` and `EventEngine::next_aspect` find the instants the ecliptic separation of two bodies equals a given angle, gated against a Swiss Ephemeris corpus (issue #84).

**Architecture:** An event is a sign change of the signed separation `wrap180(lon(first) − lon(second))` less `+angle` or `−angle`. A new level-crossing scanner in `root.rs` splits each scan step at the turning points of the separation, so two exact moments inside one step are both found; a new `aspects.rs` module feeds it the separation. A new reference tool bisects Swiss Ephemeris's own longitudes and commits a CSV; a new `validate-aspects` gate compares the engine to it event for event.

**Tech Stack:** Rust (workspace toolchain from `mise.toml`), `cargo nextest`, `libswisseph-sys` in an out-of-workspace tool built under `devenv shell`.

**Spec:** `docs/superpowers/specs/2026-10-02-aspect-exactness-design.md` (read it, including the three amendments at the end, before starting any task; where an amendment and the body disagree, the amendment wins).

## Global Constraints

- Branch: `feat/aspect-exactness-84`. Every commit message ends with `(#84)`.
- Scope is exactly two finders plus the gate. No `previous_aspect`, no user-facing CLI aspects command, no batch call over several angles or pairs, no applying/separating flag, no side field.
- `angle` is an unsigned separation in [0°, 180°]. With `d = wrap180(lon(first) − lon(second))`, an event is a sign change of `wrap180(d − angle)` or `wrap180(d + angle)`; 0° and 180° have one target.
- Turning points come from the separation's own samples. Do not use longitude speed in the engine; there is no `MissingSpeed` case for aspects.
- No existing function in `root.rs` is changed. `EventEngine::longitude_at` must return bit-identical values; `validate-crossings`, `validate-stations` and the `crossings-golden` manifest must not change. A diff in any of them is a defect, never something to regenerate.
- Scan step: the smaller of `stations::step_days` for the two bodies (0.25 day for the Moon and the lunar points, 1.0 day for the Sun, Mercury and Venus, 2.0 days otherwise). The scan is clamped two steps inside each end of the 1900–2100 window.
- The new error variant is `EventError::InvalidAspect { detail: String }`. `EventError` is already `#[non_exhaustive]`, so the change is `feat(events)`, not breaking.
- The gate has no graze zones. The reference tool fails without writing a corpus if any turning point of a corpus pair's separation comes within 30″ of one of its levels.
- No `unwrap`/`expect`/panic in library paths. Tests and the reference tool may use them.
- `CHANGELOG.md` files are written by release-plz. Do not edit them and do not bump versions.
- Run `cargo fmt --all` before every commit. Before each commit run at minimum `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and the task's own tests.
- All verification commands run in the foreground. Do not edit sources or commit while a test run is in progress.

## Review Focus

1. **A pair that never reaches the angle (Sun–Mercury at 60°), asked through `next_aspect`** — the caller gets `Ok(None)`, not an error, a panic or a wrong event. Pinned in Task 2 (`a_pair_that_never_reaches_the_angle_gives_nothing`).
2. **A range touching a window edge (JD 2415020.5 or 2488069.5), or `after` at the window end** — events are returned or `None`, never `OutOfWindow` or a backend range error, even though the scanner samples one step before its start and two past its end. Pinned in Task 1 (`samples_stay_within_one_step_before_and_two_after_the_range`) and Task 2 (`ranges_touching_the_window_edges_work`).
3. **An empty or inverted range (`start == end`, `start > end`)** — an empty list, not a panic or an error. Pinned in Task 2 (`empty_and_inverted_ranges_give_no_events`).
4. **A body the backend does not serve (Ceres on the packaged backend), as either `first` or `second`** — a typed error, not a panic and not an empty list. Pinned in Task 2 (`a_body_the_backend_does_not_serve_is_an_error`).
5. **Edge angles: `-0.0`, exactly `180.0`, a tiny angle such as `1e-6`, NaN, infinity, a negative angle** — the first three work (one target for ±0° and 180°, two targets a hair apart for the tiny angle); the rest are `InvalidAspect`, checked before the window so the caller learns about the angle first. Pinned in Task 2 (`edge_angles_work`, `invalid_angles_and_a_repeated_body_are_invalid_aspect`).

---

### Task 1: Level-crossing scanner in `root.rs`

**Files:**
- Modify: `crates/pleiades-events/src/root.rs` (add after `last_crossing_before`, before the existing `#[cfg(test)] mod tests`)
- Create: `crates/pleiades-events/src/root/level_tests.rs`

**Interfaces:**
- Consumes: the existing `wrap180`, `bisect`, `REFINE_TOLERANCE_DAYS` and `EventError` in `root.rs`.
- Produces, both `pub(crate)` in `crate::root`:
  - `level_crossings_in_range<F>(d: F, levels: &[f64], lo_jd: f64, hi_jd: f64, step_days: f64) -> Result<Vec<f64>, EventError>` where `F: FnMut(f64) -> Result<f64, EventError>` — every instant in `[lo_jd, hi_jd]` at which the wrapped-degree function `d` equals one of `levels`, ascending, each a settled instant.
  - `first_level_crossing_after<F>(d: F, levels: &[f64], lo_jd: f64, hi_jd: f64, step_days: f64) -> Result<Option<f64>, EventError>` — the first element of the above, stopping early.
  - Sampling contract: `d` is evaluated only at Julian days in `[lo_jd − step_days, hi_jd + 2·step_days)`.

- [ ] **Step 1: Write the failing tests**

Create `crates/pleiades-events/src/root/level_tests.rs`:

```rust
//! The level-crossing scanner on synthetic functions.

use super::{
    crossings_in_range, first_level_crossing_after, level_crossings_in_range, wrap180,
    REFINE_TOLERANCE_DAYS,
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
    assert!(level_crossings_in_range(d, &[0.0], T0 + 20.0, T0 + 20.0, 1.0)
        .unwrap()
        .is_empty());
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
```

In `crates/pleiades-events/src/root.rs`, add this line directly above the existing `#[cfg(test)]` / `mod tests {` at the bottom of the file:

```rust
#[cfg(test)]
mod level_tests;
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo nextest run -p pleiades-events level_tests`
Expected: a compile error, `cannot find function level_crossings_in_range` (and `first_level_crossing_after`).

- [ ] **Step 3: Write the scanner**

In `crates/pleiades-events/src/root.rs`, insert this block after the closing brace of `last_crossing_before` and before `#[cfg(test)] mod level_tests;`:

```rust
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
        drain_until(&mut d, levels, &mut pending, newer.0, lo_jd, hi_jd, &mut out)?;
        if first_only && !out.is_empty() {
            return Ok(out);
        }
        older = newer;
        newer = sample;
    }
    drain_until(&mut d, levels, &mut pending, at(intervals), lo_jd, hi_jd, &mut out)?;
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
```

This code was prototyped against the Step 1 tests before the plan was written; all twelve passed.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo nextest run -p pleiades-events level_tests`
Expected: 12 passed.

Run: `cargo nextest run -p pleiades-events root`
Expected: PASS, including every pre-existing `root::tests` test (they prove the existing scanners are untouched).

- [ ] **Step 5: Lint and commit**

Run: `cargo fmt --all && cargo clippy -p pleiades-events --all-targets --all-features -- -D warnings`
Expected: no warnings. `root.rs` already carries `#![allow(dead_code)]`, so the two new `pub(crate)` functions do not warn before Task 2 uses them.

```bash
git add crates/pleiades-events/src/root.rs crates/pleiades-events/src/root/level_tests.rs
git commit -m "feat(events): level-crossing scanner that splits steps at turning points (#84)"
```

---

### Task 2: `EventError::InvalidAspect` and the aspect finders

**Files:**
- Modify: `crates/pleiades-events/src/error.rs` (variant, `Display` arm, one test)
- Modify: `crates/pleiades-events/src/stations.rs:51` (`fn step_days` becomes `pub(crate) fn step_days`)
- Create: `crates/pleiades-events/src/aspects.rs`
- Create: `crates/pleiades-events/src/aspects/tests.rs`
- Create: `crates/pleiades-events/tests/aspects.rs`
- Modify: `crates/pleiades-events/src/lib.rs` (crate doc paragraph, `mod aspects;`, `pub use`)

**Interfaces:**
- Consumes: `root::level_crossings_in_range` and `root::first_level_crossing_after` (Task 1, signatures above); `reference::ecliptic_in(backend, &body, &reference, jd) -> Result<(f64, f64, f64), EventError>`; `reference::check_supported(&body, &reference, jd, what) -> Result<(), EventError>`; `EventEngine::check_window(jd)`; `crossings::body_label(&body) -> &'static str`.
- Produces, public in `pleiades_events`:
  - `EventEngine::aspects_in_range(&self, first: CelestialBody, second: CelestialBody, angle: Angle, reference: impl Into<CrossingReference>, start: Instant, end: Instant) -> Result<Vec<AspectEvent>, EventError>`
  - `EventEngine::next_aspect(&self, first: CelestialBody, second: CelestialBody, angle: Angle, reference: impl Into<CrossingReference>, after: Instant) -> Result<Option<AspectEvent>, EventError>`
  - `pub struct AspectEvent { pub first: CelestialBody, pub second: CelestialBody, pub angle: Angle, pub instant: Instant, pub first_longitude: Longitude, pub second_longitude: Longitude, pub frame: CrossingFrame, pub zodiac: ZodiacMode }` (`#[non_exhaustive]`, `Clone + Debug + PartialEq`, serde under the `serde` feature)
  - `EventError::InvalidAspect { detail: String }`

- [ ] **Step 1: Add the error variant with its test**

In `crates/pleiades-events/src/error.rs`, add this variant directly after `MissingSpeed { .. }` in `enum EventError`:

```rust
    /// An aspect request that is not defined: a non-finite angle, an angle
    /// outside 0–180 degrees, or the same body twice.
    InvalidAspect {
        /// Human-readable explanation.
        detail: String,
    },
```

Add this arm at the end of the `match` in `impl fmt::Display for EventError`:

```rust
            EventError::InvalidAspect { detail } => write!(f, "invalid aspect: {detail}"),
```

Add this test at the end of `mod tests` in the same file:

```rust
    #[test]
    fn invalid_aspect_renders_its_detail() {
        let err = EventError::InvalidAspect {
            detail: "got 200".into(),
        };
        assert_eq!(err.to_string(), "invalid aspect: got 200");
    }
```

In `crates/pleiades-events/src/stations.rs`, change `fn step_days(body: &CelestialBody) -> f64 {` to `pub(crate) fn step_days(body: &CelestialBody) -> f64 {`.

Run: `cargo nextest run -p pleiades-events error`
Expected: PASS, including `invalid_aspect_renders_its_detail`.

- [ ] **Step 2: Write the failing white-box tests**

Create `crates/pleiades-events/src/aspects/tests.rs`:

```rust
//! White-box checks of the aspect finder's pure pieces.

use super::{finite_longitude, levels_for, search_step};
use crate::error::EventError;
use pleiades_types::{Angle, CelestialBody};

fn levels(degrees: f64) -> Result<Vec<f64>, EventError> {
    levels_for(Angle::from_degrees(degrees))
}

#[test]
fn an_angle_strictly_inside_the_range_has_a_level_on_each_side() {
    assert_eq!(levels(90.0), Ok(vec![90.0, -90.0]));
    assert_eq!(levels(1e-6), Ok(vec![1e-6, -1e-6]));
}

#[test]
fn conjunction_and_opposition_have_one_level() {
    assert_eq!(levels(0.0), Ok(vec![0.0]));
    assert_eq!(levels(-0.0), Ok(vec![-0.0]));
    assert_eq!(levels(180.0), Ok(vec![180.0]));
}

#[test]
fn angles_outside_the_range_are_invalid() {
    for degrees in [
        -1e-9,
        180.000_001,
        360.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        let result = levels(degrees);
        assert!(
            matches!(result, Err(EventError::InvalidAspect { .. })),
            "{degrees}: {result:?}"
        );
    }
    let message = levels(200.0).unwrap_err().to_string();
    assert!(message.contains("200"), "{message}");
    assert!(message.contains("0 and 180"), "{message}");
}

#[test]
fn the_step_is_the_smaller_of_the_two_bodies() {
    use CelestialBody::{Jupiter, Mars, Mercury, Moon, Saturn, Sun, TrueNode};
    assert_eq!(search_step(&Sun, &Moon), 0.25);
    assert_eq!(search_step(&Moon, &Saturn), 0.25);
    assert_eq!(search_step(&Mars, &TrueNode), 0.25);
    assert_eq!(search_step(&Mercury, &Saturn), 1.0);
    assert_eq!(search_step(&Saturn, &Sun), 1.0);
    assert_eq!(search_step(&Mars, &Jupiter), 2.0);
}

// A NaN longitude would make every sign test false and read as "no event".
#[test]
fn a_non_finite_longitude_is_missing_coordinates() {
    let body = CelestialBody::Mars;
    assert_eq!(finite_longitude(123.5, &body, 2_451_545.0), Ok(123.5));
    for degrees in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            finite_longitude(degrees, &body, 2_451_545.0),
            Err(EventError::MissingCoordinates {
                body_label: "Mars",
                julian_day: 2_451_545.0,
            }),
            "{degrees}"
        );
    }
}
```

- [ ] **Step 3: Write the failing engine tests**

Create `crates/pleiades-events/tests/aspects.rs`. Every expected instant below was measured on the packaged backend with a prototype of the scanner on 2026-10-02.

```rust
//! `EventEngine::aspects_in_range` and `next_aspect`: both sides of an
//! angle, retrograde loops, settled instants, chaining, guards and window
//! edges.

use pleiades_data::packaged_backend;
use pleiades_events::{
    AspectEvent, CrossingFrame, CrossingReference, EventEngine, EventError, WINDOW_END_JD,
    WINDOW_START_JD,
};
use pleiades_types::{Angle, Ayanamsa, CelestialBody, Instant, JulianDay, TimeScale, ZodiacMode};

const GEO: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const HELIO: CrossingFrame = CrossingFrame::Heliocentric;
const J2000: f64 = 2_451_545.0;
/// Two seconds, in days: four times the bisection tolerance.
const TWO_SECONDS: f64 = 2.0 / 86_400.0;
/// The Moon moves 7.6e-5 degree in the 0.5 s bisection tolerance.
const EXACT_DEG: f64 = 2e-4;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn aspects(
    first: CelestialBody,
    second: CelestialBody,
    angle_deg: f64,
    reference: impl Into<CrossingReference>,
    start_jd: f64,
    end_jd: f64,
) -> Vec<AspectEvent> {
    EventEngine::new(packaged_backend())
        .aspects_in_range(
            first,
            second,
            Angle::from_degrees(angle_deg),
            reference,
            tdb(start_jd),
            tdb(end_jd),
        )
        .expect("aspects")
}

fn jd(event: &AspectEvent) -> f64 {
    event.instant.julian_day.days()
}

/// `first − second`, wrapped into (-180, 180].
fn separation(event: &AspectEvent) -> f64 {
    let d = event.first_longitude.degrees() - event.second_longitude.degrees();
    (d + 180.0).rem_euclid(360.0) - 180.0
}

/// Every event sits at the angle, on one side or the other, and the list
/// ascends.
fn assert_exact(events: &[AspectEvent], angle_deg: f64) {
    for event in events {
        let off = (separation(event).abs() - angle_deg).abs();
        assert!(off < EXACT_DEG, "{off} deg off at {event:?}");
    }
    for pair in events.windows(2) {
        assert!(jd(&pair[0]) <= jd(&pair[1]), "{pair:?}");
    }
}

fn assert_near(event: &AspectEvent, days_from_j2000: f64) {
    let got = jd(event) - J2000;
    assert!(
        (got - days_from_j2000).abs() < 0.02,
        "got J2000 {got:+.4}, want {days_from_j2000:+.4}"
    );
}

// Jupiter and Saturn met three times through Jupiter's retrograde loop:
// 31 December 1980, 4 March 1981 and 24 July 1981.
#[test]
fn jupiter_saturn_triple_conjunction_of_1981() {
    let found = aspects(
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        0.0,
        GEO,
        2_444_391.5,
        2_444_970.5,
    );
    assert_eq!(found.len(), 3, "{found:?}");
    assert_exact(&found, 0.0);
    assert_near(&found[0], -6939.6077);
    assert_near(&found[1], -6876.7034);
    assert_near(&found[2], -6735.3218);
    assert_eq!(found[0].first, CelestialBody::Jupiter);
    assert_eq!(found[0].second, CelestialBody::Saturn);
    assert_eq!(found[0].angle, Angle::from_degrees(0.0));
    assert_eq!(found[0].frame, GEO);
    assert_eq!(found[0].zodiac, ZodiacMode::Tropical);
}

// The great conjunction of 28 May 2000 was a single meeting.
#[test]
fn jupiter_saturn_single_conjunction_of_2000() {
    let found = aspects(
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        0.0,
        GEO,
        J2000,
        J2000 + 400.0,
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert_exact(&found, 0.0);
    assert_near(&found[0], 148.1701);
}

// New moons of 6 January and 5 February 2000; full moons of 21 January and
// 19 February. Both targets sit on a wrap: 0 where the difference changes
// sign, 180 where it jumps from +180 to -180.
#[test]
fn a_lunation_has_one_conjunction_and_one_opposition() {
    let new_moons = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        0.0,
        GEO,
        J2000,
        J2000 + 60.0,
    );
    assert_eq!(new_moons.len(), 2, "{new_moons:?}");
    assert_exact(&new_moons, 0.0);
    assert_near(&new_moons[0], 5.2602);
    assert_near(&new_moons[1], 35.0447);

    let full_moons = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        180.0,
        GEO,
        J2000,
        J2000 + 60.0,
    );
    assert_eq!(full_moons.len(), 2, "{full_moons:?}");
    assert_exact(&full_moons, 180.0);
    assert_near(&full_moons[0], 19.6955);
    assert_near(&full_moons[1], 49.1859);
}

// First quarter (14 January 2000, the Moon 90 degrees ahead of the Sun) and
// last quarter (28 January, 90 degrees behind) come from one call.
#[test]
fn a_square_is_found_on_both_sides_in_time_order() {
    let found = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        J2000,
        J2000 + 30.0,
    );
    assert_eq!(found.len(), 2, "{found:?}");
    assert_exact(&found, 90.0);
    assert_near(&found[0], 13.0661);
    assert_near(&found[1], 26.8318);
    assert!(separation(&found[0]) < 0.0, "{found:?}");
    assert!(separation(&found[1]) > 0.0, "{found:?}");
}

// Mercury never gets more than 28 degrees from the Sun.
#[test]
fn a_pair_that_never_reaches_the_angle_gives_nothing() {
    let engine = EventEngine::new(packaged_backend());
    assert!(aspects(
        CelestialBody::Sun,
        CelestialBody::Mercury,
        60.0,
        GEO,
        J2000,
        J2000 + 1826.0
    )
    .is_empty());
    let next = engine
        .next_aspect(
            CelestialBody::Sun,
            CelestialBody::Mercury,
            Angle::from_degrees(60.0),
            GEO,
            tdb(WINDOW_END_JD - 400.0),
        )
        .expect("a pair that never perfects is not an error");
    assert_eq!(next, None);
    // The same pair does meet: three conjunctions in the first 130 days of 2000.
    let conjunctions = aspects(
        CelestialBody::Sun,
        CelestialBody::Mercury,
        0.0,
        GEO,
        J2000,
        J2000 + 130.0,
    );
    assert_eq!(conjunctions.len(), 3, "{conjunctions:?}");
    assert_exact(&conjunctions, 0.0);
}

#[test]
fn next_aspect_is_the_first_in_range_and_chains() {
    let engine = EventEngine::new(packaged_backend());
    let next = |after_jd: f64| {
        engine
            .next_aspect(
                CelestialBody::Sun,
                CelestialBody::Moon,
                Angle::from_degrees(90.0),
                GEO,
                tdb(after_jd),
            )
            .expect("next_aspect")
            .expect("the Moon squares the Sun twice a month")
    };
    let in_range = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        J2000,
        J2000 + 60.0,
    );
    assert_eq!(in_range.len(), 4, "{in_range:?}");
    // Same anchor, same grid, same brackets: the same event exactly.
    assert_eq!(next(J2000), in_range[0]);
    // A returned instant handed back as `after` gives the following event.
    let mut after = J2000;
    for expected in &in_range {
        let found = next(after);
        assert!(jd(&found) > after, "{found:?}");
        assert!((jd(&found) - jd(expected)).abs() < TWO_SECONDS, "{found:?}");
        after = jd(&found);
    }
}

#[test]
fn swapping_the_bodies_swaps_the_longitudes() {
    let sun_moon = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        J2000,
        J2000 + 30.0,
    );
    let moon_sun = aspects(
        CelestialBody::Moon,
        CelestialBody::Sun,
        90.0,
        GEO,
        J2000,
        J2000 + 30.0,
    );
    assert_eq!(sun_moon.len(), moon_sun.len());
    for (a, b) in sun_moon.iter().zip(&moon_sun) {
        assert!((jd(a) - jd(b)).abs() < TWO_SECONDS, "{a:?} {b:?}");
        let first = (a.first_longitude.degrees() - b.second_longitude.degrees()).abs();
        let second = (a.second_longitude.degrees() - b.first_longitude.degrees()).abs();
        assert!(first < 1e-3 && second < 1e-3, "{a:?} {b:?}");
        assert_eq!(b.first, CelestialBody::Moon);
        assert_eq!(b.second, CelestialBody::Sun);
    }
}

// The ayanamsa comes off both longitudes at the same instant and cancels in
// the separation.
#[test]
fn a_sidereal_zodiac_moves_the_longitudes_not_the_instants() {
    let lahiri = CrossingReference::sidereal(GEO, Ayanamsa::Lahiri);
    let tropical = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        J2000,
        J2000 + 30.0,
    );
    let sidereal = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        lahiri.clone(),
        J2000,
        J2000 + 30.0,
    );
    assert_eq!(tropical.len(), 2);
    assert_eq!(sidereal.len(), 2);
    for (t, s) in tropical.iter().zip(&sidereal) {
        assert!((jd(t) - jd(s)).abs() < TWO_SECONDS, "{t:?} {s:?}");
        let shift_first = (t.first_longitude.degrees() - s.first_longitude.degrees()).rem_euclid(360.0);
        let shift_second =
            (t.second_longitude.degrees() - s.second_longitude.degrees()).rem_euclid(360.0);
        // Lahiri was 23.85 degrees in January 2000.
        assert!((23.8..23.9).contains(&shift_first), "{shift_first}");
        assert!((shift_first - shift_second).abs() < 1e-3, "{shift_first} {shift_second}");
        assert_eq!(s.zodiac, lahiri.zodiac);
    }
}

// From the Sun, Mars laps Jupiter every 2.24 years and never turns back.
#[test]
fn heliocentric_conjunctions_come_once_per_synodic_period() {
    let found = aspects(
        CelestialBody::Mars,
        CelestialBody::Jupiter,
        0.0,
        HELIO,
        J2000,
        J2000 + 3652.0,
    );
    assert_eq!(found.len(), 5, "{found:?}");
    assert_exact(&found, 0.0);
    assert_near(&found[0], 72.2396);
    for pair in found.windows(2) {
        let gap = jd(&pair[1]) - jd(&pair[0]);
        assert!((790.0..850.0).contains(&gap), "{gap}");
    }
    assert_eq!(found[0].frame, HELIO);
}

#[test]
fn edge_angles_work() {
    let conjunctions = |angle_deg: f64| {
        aspects(
            CelestialBody::Sun,
            CelestialBody::Moon,
            angle_deg,
            GEO,
            J2000,
            J2000 + 60.0,
        )
    };
    assert_eq!(conjunctions(-0.0).len(), 2);
    assert_eq!(conjunctions(180.0).len(), 2);
    // A hair off conjunction: both sides, a fraction of a second apart.
    let hair = conjunctions(1e-6);
    assert_eq!(hair.len(), 4, "{hair:?}");
    assert_exact(&hair, 1e-6);
}

#[test]
fn invalid_angles_and_a_repeated_body_are_invalid_aspect() {
    let engine = EventEngine::new(packaged_backend());
    for degrees in [f64::NAN, f64::INFINITY, -1.0, 180.5] {
        // Out of the window as well: the angle is reported first.
        let result = engine.aspects_in_range(
            CelestialBody::Mars,
            CelestialBody::Saturn,
            Angle::from_degrees(degrees),
            GEO,
            tdb(WINDOW_START_JD - 10.0),
            tdb(J2000),
        );
        assert!(
            matches!(result, Err(EventError::InvalidAspect { .. })),
            "{degrees}: {result:?}"
        );
        let result = engine.next_aspect(
            CelestialBody::Mars,
            CelestialBody::Saturn,
            Angle::from_degrees(degrees),
            GEO,
            tdb(J2000),
        );
        assert!(
            matches!(result, Err(EventError::InvalidAspect { .. })),
            "{degrees}: {result:?}"
        );
    }
    let twice = engine.aspects_in_range(
        CelestialBody::Mars,
        CelestialBody::Mars,
        Angle::from_degrees(0.0),
        GEO,
        tdb(J2000),
        tdb(J2000 + 30.0),
    );
    match twice {
        Err(EventError::InvalidAspect { detail }) => assert!(detail.contains("Mars"), "{detail}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn out_of_window_instants_and_undefined_frames_fail_closed() {
    let engine = EventEngine::new(packaged_backend());
    let square = Angle::from_degrees(90.0);
    let range = |reference: CrossingReference, first, second, start: f64, end: f64| {
        engine.aspects_in_range(first, second, square, reference, tdb(start), tdb(end))
    };
    assert!(matches!(
        range(
            GEO.into(),
            CelestialBody::Mars,
            CelestialBody::Saturn,
            WINDOW_START_JD - 1.0,
            J2000
        ),
        Err(EventError::OutOfWindow { .. })
    ));
    assert!(matches!(
        range(
            GEO.into(),
            CelestialBody::Mars,
            CelestialBody::Saturn,
            J2000,
            WINDOW_END_JD + 1.0
        ),
        Err(EventError::OutOfWindow { .. })
    ));
    assert!(matches!(
        engine.next_aspect(
            CelestialBody::Mars,
            CelestialBody::Saturn,
            square,
            GEO,
            tdb(WINDOW_END_JD + 1.0)
        ),
        Err(EventError::OutOfWindow { .. })
    ));
    // The Sun and the Moon have no heliocentric place, in either position.
    for (first, second) in [
        (CelestialBody::Sun, CelestialBody::Mars),
        (CelestialBody::Mars, CelestialBody::Moon),
    ] {
        let result = range(HELIO.into(), first, second, J2000, J2000 + 30.0);
        assert!(
            matches!(result, Err(EventError::UnsupportedFrame { .. })),
            "{result:?}"
        );
    }
    let sidereal_helio = CrossingReference::sidereal(HELIO, Ayanamsa::Lahiri);
    assert!(matches!(
        range(
            sidereal_helio,
            CelestialBody::Mars,
            CelestialBody::Saturn,
            J2000,
            J2000 + 30.0
        ),
        Err(EventError::UnsupportedFrame { .. })
    ));
}

#[test]
fn ranges_touching_the_window_edges_work() {
    let engine = EventEngine::new(packaged_backend());
    let early = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        WINDOW_START_JD,
        WINDOW_START_JD + 40.0,
    );
    assert!(early.len() >= 2, "{early:?}");
    assert_exact(&early, 90.0);
    let late = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        WINDOW_END_JD - 40.0,
        WINDOW_END_JD,
    );
    assert!(late.len() >= 2, "{late:?}");
    assert_exact(&late, 90.0);
    let next = engine
        .next_aspect(
            CelestialBody::Sun,
            CelestialBody::Moon,
            Angle::from_degrees(90.0),
            GEO,
            tdb(WINDOW_END_JD),
        )
        .expect("next_aspect at the window end");
    assert_eq!(next, None);
}

#[test]
fn empty_and_inverted_ranges_give_no_events() {
    let sun_moon = |start: f64, end: f64| {
        aspects(
            CelestialBody::Sun,
            CelestialBody::Moon,
            90.0,
            GEO,
            start,
            end,
        )
    };
    assert!(sun_moon(J2000, J2000).is_empty());
    assert!(sun_moon(J2000 + 30.0, J2000).is_empty());
    assert!(sun_moon(WINDOW_END_JD, WINDOW_END_JD).is_empty());
    assert!(sun_moon(WINDOW_START_JD, WINDOW_START_JD).is_empty());
}

#[test]
fn a_body_the_backend_does_not_serve_is_an_error() {
    let engine = EventEngine::new(packaged_backend());
    for (first, second) in [
        (CelestialBody::Ceres, CelestialBody::Mars),
        (CelestialBody::Mars, CelestialBody::Ceres),
    ] {
        let result = engine.aspects_in_range(
            first,
            second,
            Angle::from_degrees(0.0),
            GEO,
            tdb(J2000),
            tdb(J2000 + 30.0),
        );
        assert!(matches!(result, Err(EventError::Backend(_))), "{result:?}");
    }
}

#[cfg(feature = "serde")]
#[test]
fn an_aspect_event_round_trips_through_serde() {
    let found = aspects(
        CelestialBody::Sun,
        CelestialBody::Moon,
        90.0,
        GEO,
        J2000,
        J2000 + 30.0,
    );
    let json = serde_json::to_string(&found[0]).expect("serialize");
    let back: AspectEvent = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, found[0]);
}
```

- [ ] **Step 4: Run the tests to verify they fail**

Run: `cargo nextest run -p pleiades-events aspects`
Expected: a compile error, `unresolved import pleiades_events::AspectEvent` (the module does not exist yet).

- [ ] **Step 5: Write the module**

Create `crates/pleiades-events/src/aspects.rs`:

```rust
//! Aspect finder: the instants the ecliptic separation of two bodies equals
//! a given angle.
//!
//! With `d = wrap180(lon(first) − lon(second))`, an event is a sign change
//! of `d − angle` or of `d + angle`. The scan splits each step at the
//! turning points of `d`, so two exact moments inside one step are both
//! found.

use crate::crossings::{body_label, CrossingFrame, EventEngine};
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use crate::reference::{check_supported, ecliptic_in, CrossingReference};
use crate::root::{first_level_crossing_after, level_crossings_in_range, wrap180};
use crate::stations::step_days;
use pleiades_backend::EphemerisBackend;
use pleiades_types::{
    Angle, CelestialBody, Instant, JulianDay, Longitude, TimeScale, ZodiacMode,
};

/// An exact aspect: an instant at which the ecliptic separation of two
/// bodies equals the requested angle.
///
/// The two longitudes say which body is ahead: `first_longitude −
/// second_longitude` is `+angle` or `−angle`, to within the bodies' motion
/// over the 0.5 s bisection tolerance.
//
// `CelestialBody` is not `Copy`, so neither is this.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct AspectEvent {
    /// The first body of the pair, as passed to the finder.
    pub first: CelestialBody,
    /// The second body of the pair, as passed to the finder.
    pub second: CelestialBody,
    /// The requested separation, in 0–180 degrees.
    pub angle: Angle,
    /// Instant the aspect is exact (TDB). It trails the exact moment by less
    /// than the 0.5 s bisection tolerance and never precedes it.
    pub instant: Instant,
    /// Longitude of `first` at `instant`, in `frame` and `zodiac`.
    pub first_longitude: Longitude,
    /// Longitude of `second` at `instant`, in `frame` and `zodiac`.
    pub second_longitude: Longitude,
    /// The frame the longitudes are measured in.
    pub frame: CrossingFrame,
    /// The zodiac the longitudes are read in.
    pub zodiac: ZodiacMode,
}

fn invalid(detail: String) -> EventError {
    EventError::InvalidAspect { detail }
}

/// The values of the signed separation at which the aspect is exact: one for
/// a conjunction or an opposition, otherwise one on each side.
fn levels_for(angle: Angle) -> Result<Vec<f64>, EventError> {
    let degrees = angle.degrees();
    // A NaN is in no range.
    if !(0.0..=180.0).contains(&degrees) {
        return Err(invalid(format!(
            "the angle must be a separation between 0 and 180 degrees, got {degrees}"
        )));
    }
    if degrees == 0.0 || degrees == 180.0 {
        Ok(vec![degrees])
    } else {
        Ok(vec![degrees, -degrees])
    }
}

/// Step used to bracket aspects: the smaller of the two bodies' station
/// steps, because the separation turns where either body's speed changes.
fn search_step(first: &CelestialBody, second: &CelestialBody) -> f64 {
    step_days(first).min(step_days(second))
}

/// A finite longitude, or [`EventError::MissingCoordinates`]. A NaN must not
/// reach the sign tests, where it would read as "no event".
fn finite_longitude(
    degrees: f64,
    body: &CelestialBody,
    julian_day: f64,
) -> Result<f64, EventError> {
    if degrees.is_finite() {
        Ok(degrees)
    } else {
        Err(EventError::MissingCoordinates {
            body_label: body_label(body),
            julian_day,
        })
    }
}

fn longitude<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    reference: &CrossingReference,
    julian_day: f64,
) -> Result<f64, EventError> {
    let (degrees, _, _) = ecliptic_in(backend, body, reference, julian_day)?;
    finite_longitude(degrees, body, julian_day)
}

/// `lon(first) − lon(second)`, wrapped into (-180, 180].
fn separation<B: EphemerisBackend>(
    backend: &B,
    first: &CelestialBody,
    second: &CelestialBody,
    reference: &CrossingReference,
    julian_day: f64,
) -> Result<f64, EventError> {
    let first_deg = longitude(backend, first, reference, julian_day)?;
    let second_deg = longitude(backend, second, reference, julian_day)?;
    Ok(wrap180(first_deg - second_deg))
}

/// What both finders settle before scanning.
struct Search {
    levels: Vec<f64>,
    step: f64,
    /// Earliest and latest Julian day a scan may start and end at: two steps
    /// inside the window, because the scanner samples one step before its
    /// start and up to two past its end.
    earliest: f64,
    latest: f64,
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Validates an aspect request. The angle is checked first, then the
    /// bodies, the window, and the frame and zodiac for each body.
    fn aspect_search(
        &self,
        first: &CelestialBody,
        second: &CelestialBody,
        angle: Angle,
        reference: &CrossingReference,
        instants_jd: [f64; 2],
    ) -> Result<Search, EventError> {
        let levels = levels_for(angle)?;
        if first == second {
            return Err(invalid(format!(
                "an aspect needs two different bodies, got {} twice",
                body_label(first)
            )));
        }
        for jd in instants_jd {
            self.check_window(jd)?;
        }
        check_supported(first, reference, instants_jd[0], "aspects are")?;
        check_supported(second, reference, instants_jd[0], "aspects are")?;
        let step = search_step(first, second);
        Ok(Search {
            levels,
            step,
            earliest: WINDOW_START_JD + 2.0 * step,
            latest: WINDOW_END_JD - 2.0 * step,
        })
    }

    fn aspect_at(
        &self,
        first: &CelestialBody,
        second: &CelestialBody,
        angle: Angle,
        reference: &CrossingReference,
        julian_day: f64,
    ) -> Result<AspectEvent, EventError> {
        let first_deg = longitude(&self.backend, first, reference, julian_day)?;
        let second_deg = longitude(&self.backend, second, reference, julian_day)?;
        Ok(AspectEvent {
            first: first.clone(),
            second: second.clone(),
            angle,
            instant: Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb),
            first_longitude: Longitude::from_degrees(first_deg),
            second_longitude: Longitude::from_degrees(second_deg),
            frame: reference.frame,
            zodiac: reference.zodiac.clone(),
        })
    }

    /// All instants in `[start, end]` (TDB) at which the ecliptic separation
    /// of `first` and `second` equals `angle`, ascending.
    ///
    /// `angle` is an unsigned separation in 0–180 degrees. An angle strictly
    /// between the two is found on both sides: asking for 90 degrees returns
    /// the moments `first` is 90 degrees ahead of `second` and the moments
    /// it is 90 degrees behind. The two longitudes of each [`AspectEvent`]
    /// say which. The moment a pair enters a 3-degree orb of a square is the
    /// exact moment of the 87-degree (or 93-degree) separation.
    ///
    /// `reference` is a [`CrossingFrame`] (tropical zodiac) or a
    /// [`CrossingReference`] carrying a sidereal zodiac. The zodiac changes
    /// the reported longitudes only: the ayanamsa comes off both longitudes
    /// and cancels in the separation.
    ///
    /// A pair that approaches the angle and turns back before reaching it
    /// returns no event; that is not an error. An empty or inverted range
    /// returns an empty list, but the scan still samples both bodies at its
    /// start, so a body the backend cannot serve still returns its error.
    ///
    /// # Accuracy
    ///
    /// The search steps by the smaller of the two bodies' steps: 0.25 day
    /// for the Moon and the lunar points, 1 day for the Sun, Mercury and
    /// Venus, and 2 days otherwise. Each step is split at the turning points
    /// of the separation, so two exact moments inside one step, around a
    /// station of either body, are both found. Three limits remain:
    ///
    /// - two turning points of the separation within two steps of each other
    ///   may go unseen, and with them a pair of exact moments between them;
    /// - a pair whose separation passes the angle by less than the noise of
    ///   the ephemeris may be found or not;
    /// - an event within two steps of either end of the 1900–2100 window is
    ///   not reported, because the scan keeps its samples inside the window.
    ///
    /// The 0.5 s bisection tolerance bounds how well the engine locates the
    /// exact moment in its own longitudes, not how well that moment matches
    /// another ephemeris. The time difference is the difference in the
    /// separation divided by the pair's relative speed, and the relative
    /// speed falls to zero where the separation turns: an aspect instant is
    /// firm for a fast pair and soft for a slow pair near a station. See the
    /// crate README for the figures measured against Swiss Ephemeris.
    ///
    /// # Errors
    ///
    /// [`EventError::InvalidAspect`] for a non-finite angle, an angle outside
    /// 0–180 degrees, or the same body twice; otherwise the same as
    /// [`EventEngine::longitude_at`] for either body:
    /// [`EventError::OutOfWindow`], [`EventError::UnsupportedFrame`],
    /// [`EventError::MissingCoordinates`], [`EventError::Backend`].
    pub fn aspects_in_range(
        &self,
        first: CelestialBody,
        second: CelestialBody,
        angle: Angle,
        reference: impl Into<CrossingReference>,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<AspectEvent>, EventError> {
        let reference = reference.into();
        let start_jd = start.julian_day.days();
        let end_jd = end.julian_day.days();
        let search = self.aspect_search(&first, &second, angle, &reference, [start_jd, end_jd])?;
        let roots = level_crossings_in_range(
            |jd| separation(&self.backend, &first, &second, &reference, jd),
            &search.levels,
            start_jd.max(search.earliest),
            end_jd.min(search.latest),
            search.step,
        )?;
        roots
            .into_iter()
            .map(|jd| self.aspect_at(&first, &second, angle, &reference, jd))
            .collect()
    }

    /// The first instant strictly after `after` at which the ecliptic
    /// separation of `first` and `second` equals `angle`, or `None`.
    ///
    /// Identical to the first element of
    /// `aspects_in_range(first, second, angle, reference, after, WINDOW_END)`
    /// that is strictly after `after`, but stops at the first event found. A
    /// returned [`AspectEvent::instant`] can be handed back as `after`: the
    /// search then returns the following event, not the same one.
    ///
    /// For a pair that never reaches the angle (the Sun and Mercury at 60
    /// degrees) the search runs to the end of the 1900–2100 window before
    /// returning `None`.
    ///
    /// The meaning of `angle`, the accuracy, the limits and the errors are
    /// those of [`EventEngine::aspects_in_range`].
    ///
    /// ```
    /// use pleiades_data::packaged_backend;
    /// use pleiades_events::{CrossingFrame, EventEngine};
    /// use pleiades_types::{Angle, CelestialBody, Instant, JulianDay, TimeScale};
    ///
    /// // The Jupiter–Saturn great conjunction of 28 May 2000.
    /// let engine = EventEngine::new(packaged_backend());
    /// let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    /// let conjunction = engine
    ///     .next_aspect(
    ///         CelestialBody::Jupiter,
    ///         CelestialBody::Saturn,
    ///         Angle::from_degrees(0.0),
    ///         CrossingFrame::GeocentricApparentOfDate,
    ///         after,
    ///     )
    ///     .unwrap()
    ///     .expect("Jupiter and Saturn meet every twenty years");
    /// let days = conjunction.instant.julian_day.days() - 2_451_545.0;
    /// assert!((148.0..148.4).contains(&days), "{days}");
    /// let apart = conjunction.first_longitude.degrees() - conjunction.second_longitude.degrees();
    /// assert!(apart.abs() < 1e-4, "{apart}");
    /// ```
    pub fn next_aspect(
        &self,
        first: CelestialBody,
        second: CelestialBody,
        angle: Angle,
        reference: impl Into<CrossingReference>,
        after: Instant,
    ) -> Result<Option<AspectEvent>, EventError> {
        let reference = reference.into();
        let after_jd = after.julian_day.days();
        let search = self.aspect_search(&first, &second, angle, &reference, [after_jd, after_jd])?;
        let root = first_level_crossing_after(
            |jd| separation(&self.backend, &first, &second, &reference, jd),
            &search.levels,
            after_jd.max(search.earliest),
            search.latest,
            search.step,
        )?;
        root.filter(|&jd| jd > after_jd)
            .map(|jd| self.aspect_at(&first, &second, angle, &reference, jd))
            .transpose()
    }
}

#[cfg(test)]
mod tests;
```

In `crates/pleiades-events/src/lib.rs`:

- Add this paragraph to the crate doc, directly after the "Planetary stations — …" paragraph and its blank `//!` line:

```rust
//! Exact aspects — the instants the ecliptic separation of two bodies equals
//! a given angle — are found by [`EventEngine::aspects_in_range`] and
//! [`EventEngine::next_aspect`], in any [`CrossingFrame`] and zodiac.
//!
```

- Add `mod aspects;` as the first line of the `mod` list (above `mod crossings;`).
- Add `pub use aspects::AspectEvent;` as the first line of the `pub use` list (above `pub use crossings::{...};`).

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo nextest run -p pleiades-events aspects`
Expected: PASS — 5 white-box tests in `aspects::tests` and 15 in `tests/aspects.rs`.

If a date assertion (`assert_near`) fails while the `assert_exact` assertions pass, print the events and compare them with the comment above the test before changing anything; the expected values were measured on this backend. Never loosen `EXACT_DEG` or `TWO_SECONDS` to make a test pass.

Run: `cargo nextest run -p pleiades-events --features serde aspects`
Expected: PASS, including `an_aspect_event_round_trips_through_serde`.

Run: `cargo test -p pleiades-events --doc aspects`
Expected: PASS (the `next_aspect` doctest).

Run: `cargo nextest run -p pleiades-events`
Expected: PASS for the whole crate; no crossings, stations or rise/set test changes.

- [ ] **Step 7: Lint and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: no warnings.

```bash
git add crates/pleiades-events
git commit -m "feat(events): find the instants an aspect between two bodies is exact (#84)"
```

---

### Task 3: Swiss Ephemeris aspect reference tool and corpus

**Files:**
- Create: `tools/se-aspects-reference/Cargo.toml`
- Create: `tools/se-aspects-reference/Cargo.lock`
- Create: `tools/se-aspects-reference/LICENSE-NOTES.md`
- Create: `tools/se-aspects-reference/src/main.rs`
- Modify: `Cargo.toml:22` (root; add the tool to `[workspace].exclude`)
- Create: `crates/pleiades-validate/data/aspects-corpus/aspects.csv` (generated)

**Interfaces:**
- Consumes: nothing from earlier tasks. The tool is independent of the engine by design: it finds turning points from Swiss Ephemeris's own relative longitude speed, not from three samples.
- Produces: `aspects.csv`. Comment lines start with `#`; the header row is `group,first,second,angle_deg,jd_tt,first_lon_deg,second_lon_deg,rel_speed_deg_per_day`; data rows are grouped by `(group, first, second, angle_deg)` and ascend in `jd_tt` within each group. `group` is `geo`, `mean` or `helio`; `angle_deg` is one of `0`, `60`, `90`, `120`, `180`; `rel_speed_deg_per_day` is the longitude speed of `first` less that of `second` at the event.

- [ ] **Step 1: Create the tool crate**

Create `tools/se-aspects-reference/Cargo.toml`:

```toml
[package]
name = "se-aspects-reference"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
swisseph = "0.1.1"
libswisseph-sys = "0.1.2"
```

Copy the lockfile from the sibling tool and rename the package in it:

```bash
cp tools/se-stations-reference/Cargo.lock tools/se-aspects-reference/Cargo.lock
sed -i 's/name = "se-stations-reference"/name = "se-aspects-reference"/' tools/se-aspects-reference/Cargo.lock
```

Create `tools/se-aspects-reference/LICENSE-NOTES.md`:

```markdown
# License notes — `se-aspects-reference`

This crate is a **build-time verification harness only**. It is **not shipped**
and is deliberately kept **outside the Cargo workspace** (its own `Cargo.lock`,
`publish = false`, listed in the root `[workspace].exclude`). Nothing in the
shipped `pleiades-*` crates depends on it, and the workspace lockfile therefore
stays pure-Rust (no `-sys`/FFI), which the `workspace-audit` gate enforces.

Its sole purpose is to link Swiss Ephemeris (via `swisseph` / `libswisseph-sys`)
to **generate an exact-aspect reference corpus** (the instants at which the
difference of two Swiss Ephemeris longitudes equals 0, 60, 90, 120 or 180
degrees) used to validate the pure-Rust engine's
`EventEngine::aspects_in_range`. It runs the Moshier ephemeris (`SEFLG_MOSEPH`),
so no Swiss Ephemeris data files are bundled or distributed.

## Swiss Ephemeris licensing

Swiss Ephemeris (© Astrodienst AG) is dual-licensed: AGPL, or a separate
commercial/professional license. Because this tool is used **only internally to
produce verification fixtures** and is **never distributed as part of the
product**, no Swiss Ephemeris code, binaries, or data files enter the shipped
artifacts. The generated CSV corpus contains numeric reference values only, not
Swiss Ephemeris source or data.

Anyone building this tool locally must have libclang available
(`LIBCLANG_PATH`) and is responsible for their own compliance with the Swiss
Ephemeris license terms for their use. See the sibling `se-stations-reference`
tool, which follows the same isolated, verification-only posture.
```

In the root `Cargo.toml`, line 22, append `, "tools/se-aspects-reference"` inside the `exclude = [...]` list, directly after `"tools/se-stations-reference"`.

- [ ] **Step 2: Write the generator**

Create `tools/se-aspects-reference/src/main.rs`:

```rust
//! Emits a Swiss Ephemeris exact-aspect reference corpus to STDOUT as CSV.
//! Swiss Ephemeris has no aspect finder, so an exact aspect is located here
//! from `swe_calc(jd_tt, body, iflag | SEFLG_SPEED)` for the two bodies: the
//! signed separation `wrap180(lon(first) − lon(second))` is scanned on a
//! fixed grid, each grid step is split at the zero of the relative longitude
//! speed (the separation's turning point), and each sign change of
//! `separation − level` is bisected to 1e-7 day. The levels of an angle are
//! `+angle` and `−angle` (one level for 0 and 180). Swiss Ephemeris' "ET"
//! argument is TT.
//!
//! Groups:
//!   - `geo`:   geocentric apparent, tropical, true equinox of date (SE
//!     default flags). Seven planet pairs over the pleiades-events window
//!     less five days at each end; Sun–Moon over 1990–2030.
//!   - `mean`:  geocentric geometric place in the mean ecliptic and equinox
//!     of date (SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_NONUT).
//!     Mercury–Venus and Mars–Saturn over 1990–2030.
//!   - `helio`: geometric heliocentric place (SEFLG_HELCTR | SEFLG_TRUEPOS),
//!     the place `se-helio-reference` uses. Mars–Jupiter over the full span.
//!
//! Grid: 0.05 day; 0.01 day for a pair with the Moon.
//!
//! Grazes: the corpus must hold no model-dependent event. If any turning
//! point of a pair's separation comes within 30 arcseconds of one of its
//! levels, on either side, the tool panics and names the pair, angle and
//! instant, and no corpus is written. Counts per pair and angle go to STDERR.
//!
//! Ephemeris: Moshier (SEFLG_MOSEPH), no data files needed.
//!
//! Two build/run caveats: under devenv's gcc the build needs `CFLAGS=-std=gnu17`
//! (libswisseph-sys otherwise fails with a conflicting `getenv` declaration), and
//! `devenv shell` prints a banner line to stdout that must be removed from the
//! top of the CSV (the file must start with the `# Source:` line).
//!
//! Build inside `devenv shell` (provides clang/libclang/LIBCLANG_PATH):
//! `devenv shell -- env CFLAGS=-std=gnu17 cargo run --release \
//!    --manifest-path tools/se-aspects-reference/Cargo.toml \
//!    > crates/pleiades-validate/data/aspects-corpus/aspects.csv`

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::swe_calc;

const SEFLG_MOSEPH: c_int = 4;
const SEFLG_HELCTR: c_int = 8;
const SEFLG_TRUEPOS: c_int = 16; // geometric: no light-time
const SEFLG_NONUT: c_int = 64; // mean equinox of date
const SEFLG_SPEED: c_int = 256;
const SEFLG_NOGDEFL: c_int = 512; // no gravitational deflection
const SEFLG_NOABERR: c_int = 1024; // no annual aberration

const GEO: c_int = SEFLG_MOSEPH | SEFLG_SPEED;
const MEAN: c_int = GEO | SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_NONUT;
const HELIO: c_int = GEO | SEFLG_HELCTR | SEFLG_TRUEPOS;

/// (Swiss Ephemeris body id, name as written to the CSV).
type Body = (c_int, &'static str);
const SUN: Body = (0, "Sun");
const MOON: Body = (1, "Moon");
const MERCURY: Body = (2, "Mercury");
const VENUS: Body = (3, "Venus");
const MARS: Body = (4, "Mars");
const JUPITER: Body = (5, "Jupiter");
const SATURN: Body = (6, "Saturn");
const PLUTO: Body = (9, "Pluto");

/// The pleiades-events window (JD 2415020.5–2488069.5) less five days at each
/// end, so neither side of the comparison meets the engine's edge clamp.
const FULL_SPAN: (f64, f64) = (2_415_025.5, 2_488_064.5);
/// 1990-01-01 to 2030-01-01.
const SHORT_SPAN: (f64, f64) = (2_447_892.5, 2_462_502.5);

const GRID_DAYS: f64 = 0.05;
const MOON_GRID_DAYS: f64 = 0.01;
const BISECT_TOLERANCE_DAYS: f64 = 1e-7;
/// 30 arcseconds: several times the largest difference between the Swiss
/// Ephemeris (Moshier) and pleiades separations.
const GRAZE_MARGIN_DEG: f64 = 30.0 / 3600.0;
const ANGLES: [f64; 5] = [0.0, 60.0, 90.0, 120.0, 180.0];

/// `(longitude_deg in [0, 360), longitude_speed_deg_per_day)`.
fn state(jd_tt: f64, ipl: c_int, iflag: c_int) -> (f64, f64) {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(
            jd_tt,
            ipl,
            iflag,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        let msg = unsafe { CStr::from_ptr(serr.as_ptr() as *const c_char) }
            .to_string_lossy()
            .into_owned();
        panic!("swe_calc(ipl={ipl}, iflag={iflag}) failed at jd_tt={jd_tt}: {msg}");
    }
    assert!(
        xx[0].is_finite() && xx[3].is_finite(),
        "non-finite SE longitude or speed for ipl={ipl} at jd_tt={jd_tt}"
    );
    (xx[0].rem_euclid(360.0), xx[3])
}

fn wrap180(d: f64) -> f64 {
    (d + 180.0).rem_euclid(360.0) - 180.0
}

/// The pair at one instant.
#[derive(Clone, Copy)]
struct Sample {
    jd: f64,
    /// `wrap180(first_lon − second_lon)`.
    separation: f64,
    /// Longitude speed of the first body less that of the second.
    rel_speed: f64,
    first_lon: f64,
    second_lon: f64,
}

fn sample(jd: f64, first: Body, second: Body, iflag: c_int) -> Sample {
    let (first_lon, first_speed) = state(jd, first.0, iflag);
    let (second_lon, second_speed) = state(jd, second.0, iflag);
    Sample {
        jd,
        separation: wrap180(first_lon - second_lon),
        rel_speed: first_speed - second_speed,
        first_lon,
        second_lon,
    }
}

/// The values of the signed separation at which an angle is exact.
fn levels(angle: f64) -> Vec<f64> {
    if angle == 0.0 || angle == 180.0 {
        vec![angle]
    } else {
        vec![angle, -angle]
    }
}

/// Bisects a sign change of `f` over `[a, b]` and returns the midpoint of
/// the final bracket.
fn bisect(f: impl Fn(f64) -> f64, mut a: f64, mut f_a: f64, mut b: f64) -> f64 {
    while b - a > BISECT_TOLERANCE_DAYS {
        let mid = 0.5 * (a + b);
        let f_mid = f(mid);
        if (f_a <= 0.0) == (f_mid <= 0.0) {
            a = mid;
            f_a = f_mid;
        } else {
            b = mid;
        }
    }
    0.5 * (a + b)
}

/// Prints every exact aspect of the pair in `[lo, hi]`, for each angle.
fn scan(group: &str, first: Body, second: Body, iflag: c_int, (lo, hi): (f64, f64), grid: f64) {
    let at = |jd: f64| sample(jd, first, second, iflag);
    let mut events: Vec<Vec<Sample>> = vec![Vec::new(); ANGLES.len()];
    let steps = ((hi - lo) / grid).floor() as u64;
    let mut prev = at(lo);
    for k in 1..=steps {
        let cur = at(lo + k as f64 * grid);
        // Split the step at the separation's turning point, so that each
        // bracket is monotone and holds at most one event per level.
        let mut points = vec![prev];
        if (prev.rel_speed <= 0.0) != (cur.rel_speed <= 0.0) {
            let turn = at(bisect(
                |jd| at(jd).rel_speed,
                prev.jd,
                prev.rel_speed,
                cur.jd,
            ));
            for angle in ANGLES {
                for level in levels(angle) {
                    let miss = wrap180(turn.separation - level).abs();
                    assert!(
                        miss >= GRAZE_MARGIN_DEG,
                        "graze: {group},{},{} turns {:.2} arcsec from the {angle} degree level \
                         at jd_tt={:.5}; an event here would depend on the ephemeris",
                        first.1,
                        second.1,
                        miss * 3600.0,
                        turn.jd
                    );
                }
            }
            points.push(turn);
        }
        points.push(cur);
        for bracket in points.windows(2) {
            let (a, b) = (bracket[0], bracket[1]);
            for (index, angle) in ANGLES.iter().enumerate() {
                for level in levels(*angle) {
                    let f_a = wrap180(a.separation - level);
                    let f_b = wrap180(b.separation - level);
                    // The second test rejects the jump across the +-180 seam.
                    if (f_a <= 0.0) != (f_b <= 0.0) && (f_a - f_b).abs() < 180.0 {
                        let root = bisect(|jd| wrap180(at(jd).separation - level), a.jd, f_a, b.jd);
                        events[index].push(at(root));
                    }
                }
            }
        }
        prev = cur;
    }
    for (index, angle) in ANGLES.iter().enumerate() {
        events[index].sort_by(|x, y| x.jd.total_cmp(&y.jd));
        eprintln!(
            "{group},{},{},{angle:.0}: {} events",
            first.1,
            second.1,
            events[index].len()
        );
        for event in &events[index] {
            println!(
                "{group},{},{},{angle:.0},{:.7},{:.9},{:.9},{:.9}",
                first.1, second.1, event.jd, event.first_lon, event.second_lon, event.rel_speed
            );
        }
    }
}

fn main() {
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), Moshier (SEFLG_MOSEPH, no data files).");
    println!("# A row is an instant at which wrap180(lon(first) - lon(second)) of swe_calc(jd_tt, body, iflag|SEFLG_SPEED)");
    println!("# equals +angle or -angle, scanned on a 0.05-day grid (0.01 day with the Moon), each step split at the zero");
    println!("# of the relative longitude speed, and bisected to 1e-7 day. jd_tt is TT.");
    println!("# geo: apparent, tropical, true equinox of date (default flags); JD 2415025.5-2488064.5, Sun-Moon JD 2447892.5-2462502.5.");
    println!("# mean: SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_NONUT, JD 2447892.5-2462502.5.");
    println!("# helio: SEFLG_HELCTR|SEFLG_TRUEPOS, JD 2415025.5-2488064.5.");
    println!("# rel_speed_deg_per_day is the longitude speed of first less that of second at the row's instant.");
    println!("# No turning point of any pair's separation is within 30 arcsec of one of its levels (the tool fails otherwise).");
    println!("group,first,second,angle_deg,jd_tt,first_lon_deg,second_lon_deg,rel_speed_deg_per_day");
    scan("geo", SUN, MOON, GEO, SHORT_SPAN, MOON_GRID_DAYS);
    for (first, second) in [
        (SUN, MERCURY),
        (MERCURY, VENUS),
        (VENUS, MARS),
        (MARS, JUPITER),
        (MARS, SATURN),
        (JUPITER, SATURN),
        (SATURN, PLUTO),
    ] {
        scan("geo", first, second, GEO, FULL_SPAN, GRID_DAYS);
    }
    for (first, second) in [(MERCURY, VENUS), (MARS, SATURN)] {
        scan("mean", first, second, MEAN, SHORT_SPAN, GRID_DAYS);
    }
    scan("helio", MARS, JUPITER, HELIO, FULL_SPAN, GRID_DAYS);
}
```

- [ ] **Step 3: Generate the corpus**

```bash
mkdir -p crates/pleiades-validate/data/aspects-corpus
devenv shell -- env CFLAGS=-std=gnu17 cargo run --release \
  --manifest-path tools/se-aspects-reference/Cargo.toml \
  > crates/pleiades-validate/data/aspects-corpus/aspects.csv \
  2> /tmp/se-aspects-counts.txt
```

Run it in the foreground. Expected: exit 0.

If the tool panics with `graze:`, stop. Do not raise or lower `GRAZE_MARGIN_DEG` and do not drop the pair: report the message to the user. It is a design question (the spec's first amendment).

Then remove the `devenv shell` banner if one was written: the file must start with `# Source:`.

Run: `head -1 crates/pleiades-validate/data/aspects-corpus/aspects.csv`
Expected: `# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), Moshier (SEFLG_MOSEPH, no data files).` If any line precedes it, delete those lines and re-run this check.

- [ ] **Step 4: Cross-check the counts against the engine probe**

Run: `grep ' events$' /tmp/se-aspects-counts.txt`

Expected, for the `geo` group (0°, 60°, 90°, 120°, 180°), the counts the pre-plan probe measured on the packaged backend:

| Pair | 0° | 60° | 90° | 120° | 180° |
|---|---|---|---|---|---|
| Sun–Moon | 494 | 990 | 990 | 990 | 495 |
| Sun–Mercury | 1261 | 0 | 0 | 0 | 0 |
| Mercury–Venus | 496 | 476 | 0 | 0 | 0 |
| Venus–Mars | 167 | 281 | 268 | 236 | 94 |
| Mars–Jupiter | 91 | 209 | 209 | 201 | 132 |
| Mars–Saturn | 101 | 213 | 219 | 223 | 116 |
| Jupiter–Saturn | 14 | 42 | 52 | 66 | 36 |
| Saturn–Pluto | 10 | 28 | 26 | 30 | 15 |

Every count must match. A difference of one at a span edge is possible only if an event falls within a minute of a span end; any other difference means the tool or the engine misses or invents events. Investigate before continuing; do not edit the corpus by hand. The `mean` and `helio` counts have no prior measurement; record them.

Run: `grep -c -E '^(geo|mean|helio),' crates/pleiades-validate/data/aspects-corpus/aspects.csv`
Expected: the sum of all the counts printed to `/tmp/se-aspects-counts.txt`. Record the number; Task 5 writes it to the manifest.

- [ ] **Step 5: Confirm the workspace still excludes the tool**

Run: `cargo metadata --no-deps --format-version 1 | grep -c se-aspects-reference`
Expected: `0`.

Run: `mise run audit`
Expected: PASS. If the workspace audit names a list of reference tools that must include the new one, make the addition it names and include it in the commit.

- [ ] **Step 6: Commit**

`tools/se-aspects-reference/target/` must not be committed; it is covered by the repository's ignore rules, as for the sibling tools. Check with `git status --short tools/se-aspects-reference` that only the four source files are listed.

```bash
git add Cargo.toml tools/se-aspects-reference crates/pleiades-validate/data/aspects-corpus/aspects.csv
git commit -m "test(validate): Swiss Ephemeris exact-aspect reference tool and corpus (#84)"
```

---

### Task 4: The gate module: parsing and comparison rules

**Files:**
- Create: `crates/pleiades-validate/src/aspects_thresholds.rs`
- Create: `crates/pleiades-validate/src/aspects_validation.rs`
- Create: `crates/pleiades-validate/src/aspects_validation/tests.rs`
- Create: `crates/pleiades-validate/data/aspects-corpus/manifest.txt`
- Modify: `crates/pleiades-validate/src/lib.rs` (two `mod` lines)

**Interfaces:**
- Consumes: `EventEngine::aspects_in_range` and `AspectEvent` (Task 2); `aspects.csv` and its row count (Task 3); `pleiades_apparent::fnv1a64(&str) -> u64`.
- Produces, in `crate::aspects_validation`:
  - `pub fn validate_aspects_corpus() -> Result<AspectsReport, AspectsError>`
  - `pub fn validate_aspects_corpus_subset() -> Result<AspectsReport, AspectsError>`
  - `pub struct AspectsReport { pub rows_validated: usize, .. }` with `pub fn summary_line(&self) -> &str` and `pub fn pair_lines(&self) -> &[String]`
  - `pub enum AspectsError`
  - crate-private, used by the tests: `validate(csv, manifest)`, `validate_scoped(csv, manifest, scope)`, `Scope`, `parse_corpus`, `parse_manifest`, `compare_exact`, `Found`, `Expected`, `Residuals`, `check_floor`, `CORPUS_CSV`, `MANIFEST`.
- Produces, in `crate::aspects_thresholds`: `Ceilings { sep_arcsec, lon_arcsec }`, `ceilings_for(pair: &str) -> Option<Ceilings>`, `MIN_ROWS_VALIDATED`, `MIN_ROWS_VALIDATED_MEAN_SUBSET`. This task writes them unmeasured (infinite ceilings, floors of 1); Task 5 replaces them with measured values.

- [ ] **Step 1: Write the manifest with a placeholder checksum**

Create `crates/pleiades-validate/data/aspects-corpus/manifest.txt`, with the row count recorded in Task 3 Step 4 as `rows=`:

```
slice aspects file=aspects.csv role=aspects rows=<count from Task 3 Step 4> checksum=0
```

Task 5 replaces `checksum=0` with the real value.

- [ ] **Step 2: Write the thresholds module, unmeasured**

Create `crates/pleiades-validate/src/aspects_thresholds.rs`:

```rust
//! Ceilings for the `validate-aspects` gate.
//!
//! Not yet measured: every ceiling is infinite and both floors are 1.

/// Ceilings for one pair, over all of its groups and angles.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ceilings {
    /// The time between the engine's event and the reference's, multiplied
    /// by the pair's relative longitude speed at the event: the time
    /// residual expressed as an angle.
    pub(crate) sep_arcsec: f64,
    /// The longitude difference of either body at the event.
    pub(crate) lon_arcsec: f64,
}

/// Fail-closed floor on compared events for the full gate.
pub(crate) const MIN_ROWS_VALIDATED: usize = 1;

/// Fail-closed floor for the release-battery subset (the `mean` group).
pub(crate) const MIN_ROWS_VALIDATED_MEAN_SUBSET: usize = 1;

/// Ceilings by corpus pair name (`"Mars-Saturn"`), or `None` for a pair the
/// gate does not cover.
pub(crate) fn ceilings_for(pair: &str) -> Option<Ceilings> {
    let unmeasured = Ceilings {
        sep_arcsec: f64::INFINITY,
        lon_arcsec: f64::INFINITY,
    };
    match pair {
        "Sun-Moon" | "Sun-Mercury" | "Mercury-Venus" | "Venus-Mars" | "Mars-Jupiter"
        | "Mars-Saturn" | "Jupiter-Saturn" | "Saturn-Pluto" => Some(unmeasured),
        _ => None,
    }
}
```

- [ ] **Step 3: Write the failing tests**

Create `crates/pleiades-validate/src/aspects_validation/tests.rs`:

```rust
use super::*;

const CEILINGS: Ceilings = Ceilings {
    sep_arcsec: 10.0,
    lon_arcsec: 5.0,
};
const SECOND: f64 = 1.0 / 86_400.0;
const ARCSEC: f64 = 1.0 / 3600.0;

fn found(jd: f64, first_lon_deg: f64, second_lon_deg: f64) -> Found {
    Found {
        jd,
        first_lon_deg,
        second_lon_deg,
    }
}

fn expected(jd: f64, first_lon_deg: f64, second_lon_deg: f64, rel_speed: f64) -> Expected {
    Expected {
        found: found(jd, first_lon_deg, second_lon_deg),
        rel_speed_deg_per_day: rel_speed,
    }
}

/// A square every 100 days, alternately first ahead and first behind,
/// closing at 0.5 deg/day.
fn squares(count: usize) -> Vec<Expected> {
    (0..count)
        .map(|i| {
            let jd = 2_451_545.0 + 100.0 * i as f64;
            if i % 2 == 0 {
                expected(jd, 100.0, 10.0, 0.5)
            } else {
                expected(jd, 10.0, 100.0, -0.5)
            }
        })
        .collect()
}

fn engine_of(corpus: &[Expected]) -> Vec<Found> {
    corpus.iter().map(|event| event.found).collect()
}

#[test]
fn identical_lists_compare_exactly() {
    let corpus = squares(4);
    let residuals =
        compare_exact("geo A-B 90", 90.0, &engine_of(&corpus), &corpus, CEILINGS).unwrap();
    assert_eq!(residuals.matched, 4);
    assert_eq!(residuals.max_sep_arcsec, 0.0);
    assert_eq!(residuals.max_time_s, 0.0);
    assert_eq!(residuals.max_lon_arcsec, 0.0);
}

#[test]
fn two_empty_lists_compare_exactly() {
    let residuals = compare_exact("geo Sun-Mercury 60", 60.0, &[], &[], CEILINGS).unwrap();
    assert_eq!(residuals.matched, 0);
}

#[test]
fn a_missing_or_extra_event_is_a_count_mismatch() {
    let corpus = squares(4);
    let mut engine = engine_of(&corpus);
    engine.pop();
    assert!(matches!(
        compare_exact("geo A-B 90", 90.0, &engine, &corpus, CEILINGS),
        Err(AspectsError::CountMismatch {
            got: 3,
            want: 4,
            ..
        })
    ));
    assert!(matches!(
        compare_exact("geo A-B 90", 90.0, &engine_of(&corpus), &corpus[..3], CEILINGS),
        Err(AspectsError::CountMismatch {
            got: 4,
            want: 3,
            ..
        })
    ));
    // An engine event where the corpus has none: the "never perfects" case.
    assert!(matches!(
        compare_exact("geo Sun-Mercury 60", 60.0, &engine, &[], CEILINGS),
        Err(AspectsError::CountMismatch {
            got: 3,
            want: 0,
            ..
        })
    ));
}

#[test]
fn an_event_on_the_other_side_is_a_side_mismatch() {
    let corpus = squares(2);
    let mut engine = engine_of(&corpus);
    engine[1] = found(engine[1].jd, 100.0, 10.0);
    assert!(matches!(
        compare_exact("geo A-B 90", 90.0, &engine, &corpus, CEILINGS),
        Err(AspectsError::SideMismatch { index: 1, .. })
    ));
}

// At a conjunction the sign of first − second is noise; at an opposition it
// flips at the wrap. Neither has a side.
#[test]
fn conjunctions_and_oppositions_have_no_side() {
    let conjunction = [expected(2_451_545.0, 100.0, 100.0 + 0.1 * ARCSEC, 0.5)];
    let engine = [found(2_451_545.0, 100.0 + 0.1 * ARCSEC, 100.0)];
    compare_exact("geo A-B 0", 0.0, &engine, &conjunction, CEILINGS).unwrap();
    let opposition = [expected(2_451_545.0, 280.0, 100.0 + 0.1 * ARCSEC, 0.5)];
    let engine = [found(2_451_545.0, 280.0 + 0.1 * ARCSEC, 100.0)];
    compare_exact("geo A-B 180", 180.0, &engine, &opposition, CEILINGS).unwrap();
}

// The separation residual is the time residual times the relative speed:
// 1000 s at 0.5 deg/day is 20.8 arcsec, 100 s is 2.08 arcsec.
#[test]
fn the_separation_ceiling_scales_time_by_relative_speed() {
    let corpus = squares(1);
    let late = |seconds: f64| vec![found(corpus[0].found.jd + seconds * SECOND, 100.0, 10.0)];
    let residuals = compare_exact("geo A-B 90", 90.0, &late(100.0), &corpus, CEILINGS).unwrap();
    assert!((residuals.max_sep_arcsec - 2.083).abs() < 0.01, "{residuals:?}");
    assert!((residuals.max_time_s - 100.0).abs() < 0.01, "{residuals:?}");
    assert!((residuals.sum_signed_time_s - 100.0).abs() < 0.01, "{residuals:?}");
    match compare_exact("geo A-B 90", 90.0, &late(1000.0), &corpus, CEILINGS) {
        Err(AspectsError::CeilingExceeded { kind, residual, .. }) => {
            assert_eq!(kind, "separation_arcsec");
            assert!((residual - 20.83).abs() < 0.1, "{residual}");
        }
        other => panic!("{other:?}"),
    }
    // The same 1000 s at a slow pair's 0.01 deg/day is 0.42 arcsec: within.
    let slow = [expected(corpus[0].found.jd, 100.0, 10.0, 0.01)];
    compare_exact("geo A-B 90", 90.0, &late(1000.0), &slow, CEILINGS).unwrap();
}

#[test]
fn either_longitude_can_exceed_its_ceiling() {
    let corpus = squares(1);
    for (first, second, kind) in [
        (100.0 + 6.0 * ARCSEC, 10.0, "first_longitude_arcsec"),
        (100.0, 10.0 - 6.0 * ARCSEC, "second_longitude_arcsec"),
    ] {
        let engine = [found(corpus[0].found.jd, first, second)];
        match compare_exact("geo A-B 90", 90.0, &engine, &corpus, CEILINGS) {
            Err(AspectsError::CeilingExceeded { kind: got, .. }) => assert_eq!(got, kind),
            other => panic!("{other:?}"),
        }
    }
    let within = [found(corpus[0].found.jd, 100.0 + 4.0 * ARCSEC, 10.0)];
    let residuals = compare_exact("geo A-B 90", 90.0, &within, &corpus, CEILINGS).unwrap();
    assert!((residuals.max_lon_arcsec - 4.0).abs() < 1e-6, "{residuals:?}");
}

#[test]
fn longitude_residual_wraps_across_zero() {
    assert!((lon_residual_arcsec(0.0001, 359.9999) - 0.72).abs() < 1e-6);
    assert!((lon_residual_arcsec(359.9999, 0.0001) - 0.72).abs() < 1e-6);
}

#[test]
fn a_nan_residual_fails_closed() {
    let corpus = squares(1);
    let engine = [found(f64::NAN, 100.0, 10.0)];
    assert!(matches!(
        compare_exact("geo A-B 90", 90.0, &engine, &corpus, CEILINGS),
        Err(AspectsError::CeilingExceeded { .. })
    ));
}

#[test]
fn residuals_absorb_maxima_and_sums() {
    let mut total = Residuals {
        matched: 2,
        max_sep_arcsec: 1.0,
        max_time_s: 50.0,
        max_lon_arcsec: 0.2,
        sum_signed_time_s: -10.0,
    };
    total.absorb(Residuals {
        matched: 3,
        max_sep_arcsec: 0.5,
        max_time_s: 80.0,
        max_lon_arcsec: 0.1,
        sum_signed_time_s: 4.0,
    });
    assert_eq!(total.matched, 5);
    assert_eq!(total.max_sep_arcsec, 1.0);
    assert_eq!(total.max_time_s, 80.0);
    assert_eq!(total.max_lon_arcsec, 0.2);
    assert_eq!(total.sum_signed_time_s, -6.0);
}

const SMALL_CSV: &str = "\
# a comment
group,first,second,angle_deg,jd_tt,first_lon_deg,second_lon_deg,rel_speed_deg_per_day
geo,Mars,Saturn,0,2451600.5000000,10.000000000,10.000000001,0.500000000
geo,Mars,Saturn,0,2452300.2500000,200.000000000,200.000000000,0.480000000
geo,Mars,Saturn,90,2451700.5000000,100.000000000,10.000000000,0.510000000
mean,Mercury,Venus,60,2451650.0000000,70.000000000,10.000000000,-0.900000000
";

#[test]
fn corpus_rows_parse_with_their_pair_and_angle() {
    let rows = parse_corpus(SMALL_CSV).unwrap();
    assert_eq!(rows.len(), 4);
    let pair = |row: &Row| (PAIRS[row.pair].group, PAIRS[row.pair].first, PAIRS[row.pair].second);
    assert_eq!(pair(&rows[0]), (Group::Geo, "Mars", "Saturn"));
    assert_eq!(ANGLES_DEG[rows[0].angle], 0.0);
    assert_eq!(rows[0].expected.found.jd, 2_451_600.5);
    assert_eq!(rows[0].expected.rel_speed_deg_per_day, 0.5);
    assert_eq!(ANGLES_DEG[rows[2].angle], 90.0);
    assert_eq!(pair(&rows[3]), (Group::Mean, "Mercury", "Venus"));
    assert_eq!(ANGLES_DEG[rows[3].angle], 60.0);
    assert_eq!(rows[3].expected.found.first_lon_deg, 70.0);
}

#[test]
fn malformed_rows_are_rejected() {
    let header = "group,first,second,angle_deg,jd_tt,first_lon_deg,second_lon_deg,rel_speed_deg_per_day\n";
    for bad in [
        "geo,Mars,Saturn,0,2451600.5,10.0,10.0\n",             // 7 fields
        "sid,Mars,Saturn,0,2451600.5,10.0,10.0,0.5\n",         // unknown group
        "geo,Mars,Venus,0,2451600.5,10.0,10.0,0.5\n",          // pair not in the corpus plan
        "geo,Saturn,Mars,0,2451600.5,10.0,10.0,0.5\n",         // pair in the wrong order
        "mean,Sun,Moon,0,2451600.5,10.0,10.0,0.5\n",           // pair not in this group
        "geo,Mars,Saturn,45,2451600.5,10.0,10.0,0.5\n",        // angle not in the corpus plan
        "geo,Mars,Saturn,0,nope,10.0,10.0,0.5\n",              // not a number
        "geo,Mars,Saturn,0,2451600.5,NaN,10.0,0.5\n",          // not finite
        "geo,Mars,Saturn,0,2451600.5,10.0,10.0,0.5\ngeo,Mars,Saturn,0,2451600.5,10.0,10.0,0.5\n", // not ascending
    ] {
        let csv = format!("{header}{bad}");
        assert!(
            matches!(parse_corpus(&csv), Err(AspectsError::MalformedRow(_))),
            "{bad}"
        );
    }
}

#[test]
fn manifest_parses_rows_and_checksum() {
    assert_eq!(
        parse_manifest("slice aspects file=aspects.csv role=aspects rows=12 checksum=345\n").unwrap(),
        (12, 345)
    );
    for bad in [
        "",
        "slice aspects rows=12\n",
        "slice aspects checksum=345\n",
        "slice aspects rows=x checksum=345\n",
    ] {
        assert!(
            matches!(parse_manifest(bad), Err(AspectsError::MalformedManifest(_))),
            "{bad}"
        );
    }
}

#[test]
fn too_few_validated_rows_fail_the_floor() {
    assert!(check_floor(10, 10).is_ok());
    assert!(matches!(
        check_floor(9, 10),
        Err(AspectsError::TooFewRowsValidated {
            validated: 9,
            floor: 10
        })
    ));
    // A floor of zero still demands one row.
    assert!(matches!(
        check_floor(0, 0),
        Err(AspectsError::TooFewRowsValidated {
            validated: 0,
            floor: 1
        })
    ));
}

#[test]
fn the_corpus_plan_matches_the_reference_tool() {
    assert_eq!(PAIRS.len(), 11);
    assert_eq!(PAIRS.iter().filter(|pair| pair.group == Group::Geo).count(), 8);
    assert_eq!(PAIRS.iter().filter(|pair| pair.group == Group::Mean).count(), 2);
    assert_eq!(PAIRS.iter().filter(|pair| pair.group == Group::Helio).count(), 1);
    for pair in &PAIRS {
        assert!(body_from_name(pair.first).is_some(), "{}", pair.first);
        assert!(body_from_name(pair.second).is_some(), "{}", pair.second);
        let name = format!("{}-{}", pair.first, pair.second);
        assert!(ceilings_for(&name).is_some(), "{name}");
    }
    assert!(Scope::Full.includes(Group::Geo));
    assert!(Scope::MeanSubset.includes(Group::Mean));
    assert!(!Scope::MeanSubset.includes(Group::Geo));
    assert!(!Scope::MeanSubset.includes(Group::Helio));
}
```

- [ ] **Step 4: Run the tests to verify they fail**

Add to `crates/pleiades-validate/src/lib.rs`, in the alphabetical `mod` list (directly above `mod angles_validation;`, or wherever `mod a…` lines sort):

```rust
mod aspects_thresholds;
mod aspects_validation;
```

Run: `cargo nextest run -p pleiades-validate aspects_validation`
Expected: a compile error, `file not found for module aspects_validation`.

- [ ] **Step 5: Write the gate module**

Create `crates/pleiades-validate/src/aspects_validation.rs`:

```rust
//! Fail-closed gate: `EventEngine::aspects_in_range` on the packaged backend
//! vs the committed Swiss Ephemeris exact-aspect reference corpus (issue #84).
//!
//! Swiss Ephemeris has no aspect finder; the corpus holds the instants at
//! which the difference of its own longitudes equals 0, 60, 90, 120 or 180
//! degrees (`tools/se-aspects-reference`). The engine must match it event
//! for event, including the pair-and-angle series in which neither side has
//! an event. The reference tool refuses to write a corpus in which a
//! separation turns within 30 arcseconds of an angle, so no corpus event
//! depends on the ephemeris. See `aspects_thresholds` for the ceilings.

use crate::aspects_thresholds::{
    ceilings_for, Ceilings, MIN_ROWS_VALIDATED, MIN_ROWS_VALIDATED_MEAN_SUBSET,
};
use pleiades_apparent::fnv1a64;
use pleiades_data::packaged_backend;
use pleiades_events::{CrossingFrame, CrossingReference, EventEngine};
use pleiades_types::{Angle, CelestialBody, Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/aspects.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/aspects-corpus/manifest.txt"
));

/// The spans `tools/se-aspects-reference` scanned (Julian days, TT). The
/// full span is the engine's window less five days at each end, so neither
/// side of the comparison meets the engine's edge clamp.
const FULL_SPAN: (f64, f64) = (2_415_025.5, 2_488_064.5);
/// 1990-01-01 to 2030-01-01.
const SHORT_SPAN: (f64, f64) = (2_447_892.5, 2_462_502.5);

/// The angles the reference tool scanned for every pair.
const ANGLES_DEG: [f64; 5] = [0.0, 60.0, 90.0, 120.0, 180.0];

const SECONDS_PER_DAY: f64 = 86_400.0;
const ARCSEC_PER_DEG: f64 = 3600.0;

/// Which corpus groups a run compares. The checksum and the manifest row
/// count are always verified against the whole corpus.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Scope {
    /// Every pair: `validate-aspects` and the gate test (nightly `test-full`).
    Full,
    /// The `mean` group only, for the release battery
    /// (`run_all_numeric_gates`), where the full 1900–2100 scans are too slow.
    MeanSubset,
}

impl Scope {
    fn includes(self, group: Group) -> bool {
        match self {
            Self::Full => true,
            Self::MeanSubset => group == Group::Mean,
        }
    }

    fn floor(self) -> usize {
        match self {
            Self::Full => MIN_ROWS_VALIDATED,
            Self::MeanSubset => MIN_ROWS_VALIDATED_MEAN_SUBSET,
        }
    }

    /// How the summary line names the run.
    fn title(self) -> &'static str {
        match self {
            Self::Full => "Aspects gate",
            Self::MeanSubset => "Aspects gate (mean subset)",
        }
    }
}

/// The frame a corpus group was generated in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Group {
    /// Geocentric apparent, tropical.
    Geo,
    /// Geocentric mean of date, tropical.
    Mean,
    /// Heliocentric, geometric.
    Helio,
}

impl Group {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "geo" => Self::Geo,
            "mean" => Self::Mean,
            "helio" => Self::Helio,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Self::Geo => "geo",
            Self::Mean => "mean",
            Self::Helio => "helio",
        }
    }

    fn reference(self) -> CrossingReference {
        match self {
            Self::Geo => CrossingFrame::GeocentricApparentOfDate.into(),
            Self::Mean => CrossingFrame::GeocentricMeanOfDate.into(),
            Self::Helio => CrossingFrame::Heliocentric.into(),
        }
    }
}

/// One pair the reference tool scanned, at every angle in [`ANGLES_DEG`].
#[derive(Clone, Copy, Debug)]
struct Pair {
    group: Group,
    first: &'static str,
    second: &'static str,
    span: (f64, f64),
}

const fn pair(group: Group, first: &'static str, second: &'static str, span: (f64, f64)) -> Pair {
    Pair {
        group,
        first,
        second,
        span,
    }
}

/// The corpus plan, mirroring `main` of `tools/se-aspects-reference`. A
/// pair-and-angle series with no corpus rows is still compared: the engine
/// must find nothing there either.
const PAIRS: [Pair; 11] = [
    pair(Group::Geo, "Sun", "Moon", SHORT_SPAN),
    pair(Group::Geo, "Sun", "Mercury", FULL_SPAN),
    pair(Group::Geo, "Mercury", "Venus", FULL_SPAN),
    pair(Group::Geo, "Venus", "Mars", FULL_SPAN),
    pair(Group::Geo, "Mars", "Jupiter", FULL_SPAN),
    pair(Group::Geo, "Mars", "Saturn", FULL_SPAN),
    pair(Group::Geo, "Jupiter", "Saturn", FULL_SPAN),
    pair(Group::Geo, "Saturn", "Pluto", FULL_SPAN),
    pair(Group::Mean, "Mercury", "Venus", SHORT_SPAN),
    pair(Group::Mean, "Mars", "Saturn", SHORT_SPAN),
    pair(Group::Helio, "Mars", "Jupiter", FULL_SPAN),
];

fn body_from_name(name: &str) -> Option<CelestialBody> {
    Some(match name {
        "Sun" => CelestialBody::Sun,
        "Moon" => CelestialBody::Moon,
        "Mercury" => CelestialBody::Mercury,
        "Venus" => CelestialBody::Venus,
        "Mars" => CelestialBody::Mars,
        "Jupiter" => CelestialBody::Jupiter,
        "Saturn" => CelestialBody::Saturn,
        "Pluto" => CelestialBody::Pluto,
        _ => return None,
    })
}

/// One exact aspect, from either side of the comparison.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Found {
    jd: f64,
    first_lon_deg: f64,
    second_lon_deg: f64,
}

/// A corpus event: the reference instant and longitudes, and the pair's
/// relative longitude speed there.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Expected {
    found: Found,
    rel_speed_deg_per_day: f64,
}

/// One corpus row, with its pair and angle as indices into [`PAIRS`] and
/// [`ANGLES_DEG`].
#[derive(Clone, Copy, Debug)]
struct Row {
    pair: usize,
    angle: usize,
    expected: Expected,
}

#[derive(Debug)]
pub enum AspectsError {
    MalformedRow(String),
    MalformedManifest(String),
    ChecksumMismatch {
        got: u64,
        want: u64,
    },
    ManifestDrift {
        rows_csv: usize,
        rows_manifest: usize,
    },
    TooFewRowsValidated {
        validated: usize,
        floor: usize,
    },
    CalculationFailed {
        series: String,
        reason: String,
    },
    /// The engine and the corpus disagree on how many times an aspect is exact.
    CountMismatch {
        series: String,
        got: usize,
        want: usize,
    },
    /// The engine and the corpus disagree on which body is ahead.
    SideMismatch {
        series: String,
        index: usize,
        jd_tt: f64,
    },
    CeilingExceeded {
        series: String,
        jd_tt: f64,
        kind: &'static str,
        residual: f64,
        ceiling: f64,
    },
}

impl std::fmt::Display for AspectsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedRow(s) => write!(f, "malformed corpus row: {s}"),
            Self::MalformedManifest(s) => write!(f, "malformed manifest: {s}"),
            Self::ChecksumMismatch { got, want } => {
                write!(f, "corpus checksum mismatch: got {got} want {want}")
            }
            Self::ManifestDrift {
                rows_csv,
                rows_manifest,
            } => write!(
                f,
                "manifest drift: csv has {rows_csv} rows, manifest says {rows_manifest}"
            ),
            Self::TooFewRowsValidated { validated, floor } => {
                write!(f, "only {validated} aspects validated, floor is {floor}")
            }
            Self::CalculationFailed { series, reason } => {
                write!(f, "{series} aspect search failed: {reason}")
            }
            Self::CountMismatch { series, got, want } => write!(
                f,
                "{series}: engine found {got} exact aspects, corpus has {want}"
            ),
            Self::SideMismatch {
                series,
                index,
                jd_tt,
            } => write!(
                f,
                "{series}: aspect {index} near jd_tt={jd_tt} is on the other side in the corpus"
            ),
            Self::CeilingExceeded {
                series,
                jd_tt,
                kind,
                residual,
                ceiling,
            } => write!(
                f,
                "{series} {kind} ceiling exceeded at jd_tt={jd_tt}: residual {residual:.6e} > ceiling {ceiling:.6e}"
            ),
        }
    }
}

impl std::error::Error for AspectsError {}

fn parse_corpus(csv: &str) -> Result<Vec<Row>, AspectsError> {
    let malformed = |what: String| AspectsError::MalformedRow(what);
    let mut rows = Vec::new();
    // The last instant seen in each pair-and-angle series.
    let mut last_jd = [[f64::NEG_INFINITY; ANGLES_DEG.len()]; PAIRS.len()];
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("group,") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 8 {
            return Err(malformed(format!(
                "expected 8 fields, got {} in {line}",
                fields.len()
            )));
        }
        let group = Group::from_name(fields[0])
            .ok_or_else(|| malformed(format!("unknown group {} in {line}", fields[0])))?;
        let pair = PAIRS
            .iter()
            .position(|p| p.group == group && p.first == fields[1] && p.second == fields[2])
            .ok_or_else(|| {
                malformed(format!(
                    "pair {}-{} is not in the {} corpus plan in {line}",
                    fields[1], fields[2], fields[0]
                ))
            })?;
        let num = |i: usize| -> Result<f64, AspectsError> {
            fields[i]
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| malformed(format!("field {i} is not a finite number in {line}")))
        };
        let angle_deg = num(3)?;
        let angle = ANGLES_DEG
            .iter()
            .position(|known| *known == angle_deg)
            .ok_or_else(|| malformed(format!("angle {angle_deg} is not in the corpus plan in {line}")))?;
        let expected = Expected {
            found: Found {
                jd: num(4)?,
                first_lon_deg: num(5)?,
                second_lon_deg: num(6)?,
            },
            rel_speed_deg_per_day: num(7)?,
        };
        if last_jd[pair][angle] >= expected.found.jd {
            return Err(malformed(format!("rows are not ascending at {line}")));
        }
        last_jd[pair][angle] = expected.found.jd;
        rows.push(Row {
            pair,
            angle,
            expected,
        });
    }
    Ok(rows)
}

fn parse_manifest(manifest: &str) -> Result<(usize, u64), AspectsError> {
    let malformed = |what: String| AspectsError::MalformedManifest(what);
    let line = manifest
        .lines()
        .find(|l| l.trim_start().starts_with("slice"))
        .ok_or_else(|| malformed("no slice line".into()))?;
    let mut rows = None;
    let mut checksum = None;
    for tok in line.split_whitespace() {
        if let Some(v) = tok.strip_prefix("rows=") {
            rows = Some(
                v.parse::<usize>()
                    .map_err(|e| malformed(format!("rows: {e}")))?,
            );
        } else if let Some(v) = tok.strip_prefix("checksum=") {
            checksum = Some(
                v.parse::<u64>()
                    .map_err(|e| malformed(format!("checksum: {e}")))?,
            );
        }
    }
    Ok((
        rows.ok_or_else(|| malformed("rows= missing".into()))?,
        checksum.ok_or_else(|| malformed("checksum= missing".into()))?,
    ))
}

/// What a comparison measured.
#[derive(Clone, Copy, Debug, Default)]
struct Residuals {
    /// Events compared.
    matched: usize,
    max_sep_arcsec: f64,
    max_time_s: f64,
    max_lon_arcsec: f64,
    /// Sum of engine − corpus time over the compared events.
    sum_signed_time_s: f64,
}

impl Residuals {
    fn absorb(&mut self, other: Residuals) {
        self.matched += other.matched;
        self.max_sep_arcsec = self.max_sep_arcsec.max(other.max_sep_arcsec);
        self.max_time_s = self.max_time_s.max(other.max_time_s);
        self.max_lon_arcsec = self.max_lon_arcsec.max(other.max_lon_arcsec);
        self.sum_signed_time_s += other.sum_signed_time_s;
    }

    fn mean_signed_time_s(&self) -> f64 {
        if self.matched == 0 {
            0.0
        } else {
            self.sum_signed_time_s / self.matched as f64
        }
    }
}

fn wrap180(degrees: f64) -> f64 {
    (degrees + 180.0).rem_euclid(360.0) - 180.0
}

fn lon_residual_arcsec(got_deg: f64, want_deg: f64) -> f64 {
    wrap180(got_deg - want_deg).abs() * ARCSEC_PER_DEG
}

/// Whether the first body is ahead of the second.
fn first_is_ahead(found: &Found) -> bool {
    wrap180(found.first_lon_deg - found.second_lon_deg) > 0.0
}

/// The two lists must agree event for event: the same length and, for an
/// angle strictly between 0 and 180 degrees, the same side in order; then
/// every residual within its ceiling. A NaN residual fails closed.
fn compare_exact(
    label: &str,
    angle_deg: f64,
    engine: &[Found],
    corpus: &[Expected],
    ceilings: Ceilings,
) -> Result<Residuals, AspectsError> {
    if engine.len() != corpus.len() {
        return Err(AspectsError::CountMismatch {
            series: label.to_string(),
            got: engine.len(),
            want: corpus.len(),
        });
    }
    let has_side = angle_deg > 0.0 && angle_deg < 180.0;
    let mut residuals = Residuals::default();
    for (index, (got, want)) in engine.iter().zip(corpus).enumerate() {
        if has_side && first_is_ahead(got) != first_is_ahead(&want.found) {
            return Err(AspectsError::SideMismatch {
                series: label.to_string(),
                index,
                jd_tt: want.found.jd,
            });
        }
        let signed_time_s = (got.jd - want.found.jd) * SECONDS_PER_DAY;
        let sep_arcsec =
            (got.jd - want.found.jd).abs() * want.rel_speed_deg_per_day.abs() * ARCSEC_PER_DEG;
        let first_arcsec = lon_residual_arcsec(got.first_lon_deg, want.found.first_lon_deg);
        let second_arcsec = lon_residual_arcsec(got.second_lon_deg, want.found.second_lon_deg);
        let checks = [
            ("separation_arcsec", sep_arcsec, ceilings.sep_arcsec),
            ("first_longitude_arcsec", first_arcsec, ceilings.lon_arcsec),
            ("second_longitude_arcsec", second_arcsec, ceilings.lon_arcsec),
        ];
        for (kind, residual, ceiling) in checks {
            if residual.is_nan() || residual > ceiling {
                return Err(AspectsError::CeilingExceeded {
                    series: label.to_string(),
                    jd_tt: want.found.jd,
                    kind,
                    residual,
                    ceiling,
                });
            }
        }
        residuals.absorb(Residuals {
            matched: 1,
            max_sep_arcsec: sep_arcsec,
            max_time_s: signed_time_s.abs(),
            max_lon_arcsec: first_arcsec.max(second_arcsec),
            sum_signed_time_s: signed_time_s,
        });
    }
    Ok(residuals)
}

fn check_floor(validated: usize, floor: usize) -> Result<(), AspectsError> {
    let floor = floor.max(1);
    if validated < floor {
        return Err(AspectsError::TooFewRowsValidated { validated, floor });
    }
    Ok(())
}

#[derive(Debug)]
pub struct AspectsReport {
    /// Exact aspects compared against the corpus.
    pub rows_validated: usize,
    pair_lines: Vec<String>,
    summary_line: String,
}

impl AspectsReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }

    /// One line per corpus pair with its counts per angle and its measured
    /// maxima; the basis for the ceilings in `aspects_thresholds`.
    pub fn pair_lines(&self) -> &[String] {
        &self.pair_lines
    }
}

fn validate(csv: &str, manifest: &str) -> Result<AspectsReport, AspectsError> {
    validate_scoped(csv, manifest, Scope::Full)
}

fn validate_scoped(csv: &str, manifest: &str, scope: Scope) -> Result<AspectsReport, AspectsError> {
    let (manifest_rows, manifest_checksum) = parse_manifest(manifest)?;
    let got_checksum = fnv1a64(csv);
    if got_checksum != manifest_checksum {
        return Err(AspectsError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let rows = parse_corpus(csv)?;
    if rows.len() != manifest_rows {
        return Err(AspectsError::ManifestDrift {
            rows_csv: rows.len(),
            rows_manifest: manifest_rows,
        });
    }

    let engine = EventEngine::new(packaged_backend());
    // The corpus epoch is TT; the engine reads the Julian day as TDB. The two
    // differ by under 2 ms, far below every ceiling here.
    let tdb = |jd: f64| Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
    let mut total = Residuals::default();
    let mut pair_lines = Vec::new();
    let in_scope = PAIRS
        .iter()
        .enumerate()
        .filter(|(_, pair)| scope.includes(pair.group));
    for (pair_index, pair) in in_scope {
        let name = format!("{}-{}", pair.first, pair.second);
        let pair_label = format!("{} {name}", pair.group.name());
        let failed = |series: &str, reason: String| AspectsError::CalculationFailed {
            series: series.to_string(),
            reason,
        };
        let ceilings = ceilings_for(&name)
            .ok_or_else(|| failed(&pair_label, "no ceilings for this pair".into()))?;
        let body = |body_name: &str| {
            body_from_name(body_name)
                .ok_or_else(|| failed(&pair_label, format!("unknown body {body_name}")))
        };
        let (first, second) = (body(pair.first)?, body(pair.second)?);
        let mut residuals = Residuals::default();
        let mut counts = Vec::new();
        for (angle_index, angle_deg) in ANGLES_DEG.iter().enumerate() {
            let label = format!("{pair_label} {angle_deg:.0}");
            let corpus: Vec<Expected> = rows
                .iter()
                .filter(|row| row.pair == pair_index && row.angle == angle_index)
                .map(|row| row.expected)
                .collect();
            let found: Vec<Found> = engine
                .aspects_in_range(
                    first.clone(),
                    second.clone(),
                    Angle::from_degrees(*angle_deg),
                    pair.group.reference(),
                    tdb(pair.span.0),
                    tdb(pair.span.1),
                )
                .map_err(|e| failed(&label, e.to_string()))?
                .into_iter()
                .map(|event| Found {
                    jd: event.instant.julian_day.days(),
                    first_lon_deg: event.first_longitude.degrees(),
                    second_lon_deg: event.second_longitude.degrees(),
                })
                .collect();
            residuals.absorb(compare_exact(&label, *angle_deg, &found, &corpus, ceilings)?);
            counts.push(format!("{angle_deg:.0}°: {}", found.len()));
        }
        pair_lines.push(format!(
            "{pair_label}: {} compared ({}), max sep {:.3}\", max time {:.1} s, mean signed time {:+.1} s, max lon {:.3}\"",
            residuals.matched,
            counts.join(", "),
            residuals.max_sep_arcsec,
            residuals.max_time_s,
            residuals.mean_signed_time_s(),
            residuals.max_lon_arcsec,
        ));
        total.absorb(residuals);
    }
    check_floor(total.matched, scope.floor())?;
    let summary_line = format!(
        "{}: {} exact aspects validated across {} pairs vs Swiss Ephemeris corpus (event for event at 0, 60, 90, 120 and 180 degrees), max separation residual {:.3}\", max time {:.1} s, max lon {:.3}\"",
        scope.title(),
        total.matched,
        pair_lines.len(),
        total.max_sep_arcsec,
        total.max_time_s,
        total.max_lon_arcsec,
    );
    Ok(AspectsReport {
        rows_validated: total.matched,
        pair_lines,
        summary_line,
    })
}

/// The full gate: every corpus pair at every angle, floor
/// `MIN_ROWS_VALIDATED`. Run by `validate-aspects` and by the nightly
/// `test-full` tier.
pub fn validate_aspects_corpus() -> Result<AspectsReport, AspectsError> {
    validate(CORPUS_CSV, MANIFEST)
}

/// The release-battery subset: verifies the checksum and row count of the
/// whole corpus, then compares only the `mean` group (Mercury–Venus and
/// Mars–Saturn over 1990–2030), floor `MIN_ROWS_VALIDATED_MEAN_SUBSET`.
/// Fail-closed like the full gate.
pub fn validate_aspects_corpus_subset() -> Result<AspectsReport, AspectsError> {
    validate_scoped(CORPUS_CSV, MANIFEST, Scope::MeanSubset)
}

#[cfg(test)]
mod tests;
```

`parse_manifest` duplicates the one in `stations_validation.rs` on purpose: every gate module in this crate is self-contained and returns its own error type.

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo nextest run -p pleiades-validate aspects_validation`
Expected: 15 passed. None of these tests runs the engine.

- [ ] **Step 7: Lint and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: no warnings. `validate_aspects_corpus`, `validate_aspects_corpus_subset`, `validate` and the `MANIFEST` constant are not yet used outside tests; if clippy reports dead code for them, add `#[allow(dead_code)] // wired in the next commit (#84)` on exactly the items it names, and remove every such attribute in Task 5 Step 5.

```bash
git add crates/pleiades-validate/src/aspects_thresholds.rs crates/pleiades-validate/src/aspects_validation.rs crates/pleiades-validate/src/aspects_validation crates/pleiades-validate/src/lib.rs crates/pleiades-validate/data/aspects-corpus/manifest.txt
git commit -m "test(validate): exact-aspect gate parsing and event-for-event comparison (#84)"
```

---

### Task 5: The `validate-aspects` gate, measured ceilings and wiring

**Files:**
- Modify: `crates/pleiades-validate/data/aspects-corpus/manifest.txt` (checksum)
- Modify: `crates/pleiades-validate/src/aspects_validation/tests.rs` (engine tests)
- Modify: `crates/pleiades-validate/src/aspects_thresholds.rs` (measured values)
- Modify: `crates/pleiades-validate/src/aspects_validation.rs` (doc comments with measured times; remove any `#[allow(dead_code)]` from Task 4)
- Modify: `crates/pleiades-validate/src/lib.rs` (`pub use`)
- Modify: `crates/pleiades-validate/src/render/cli.rs` (`run_all_numeric_gates`, command arm, help text)
- Modify: `crates/pleiades-cli/src/cli.rs` (routing arm)

**Interfaces:**
- Consumes: everything Task 4 produces.
- Produces: CLI commands `validate-aspects` and alias `aspects-gate`; `pleiades_validate::{validate_aspects_corpus, validate_aspects_corpus_subset, AspectsError, AspectsReport}`; the subset in `run_all_numeric_gates`.

- [ ] **Step 1: Write the failing engine tests**

Append to `crates/pleiades-validate/src/aspects_validation/tests.rs`:

```rust
/// A manifest that matches `csv`, so a test can change the corpus and get
/// past the checksum to the rule it is testing.
fn manifest_for(csv: &str) -> String {
    let rows = csv
        .lines()
        .filter(|line| ["geo,", "mean,", "helio,"].iter().any(|g| line.starts_with(g)))
        .count();
    format!(
        "slice aspects file=aspects.csv role=aspects rows={rows} checksum={}",
        fnv1a64(csv)
    )
}

/// The first corpus line of the mean Mars–Saturn conjunction series.
fn first_mean_mars_saturn_conjunction() -> &'static str {
    CORPUS_CSV
        .lines()
        .find(|line| line.starts_with("mean,Mars,Saturn,0,"))
        .expect("the corpus has a mean Mars-Saturn conjunction")
}

#[test]
fn aspects_gate_passes_within_ceilings() {
    let started = std::time::Instant::now();
    let report = validate_aspects_corpus().expect("aspects gate passes");
    eprintln!("{}", report.summary_line());
    for line in report.pair_lines() {
        eprintln!("{line}");
    }
    eprintln!("full gate: {:.1} s", started.elapsed().as_secs_f64());
    assert!(report.rows_validated >= MIN_ROWS_VALIDATED);
    assert_eq!(report.pair_lines().len(), 11);
}

#[test]
fn mean_subset_passes_and_compares_only_the_mean_group() {
    let started = std::time::Instant::now();
    let report = validate_aspects_corpus_subset().expect("aspects subset passes");
    eprintln!("{}", report.summary_line());
    for line in report.pair_lines() {
        eprintln!("{line}");
    }
    eprintln!("mean subset: {:.1} s", started.elapsed().as_secs_f64());
    assert_eq!(report.rows_validated, MIN_ROWS_VALIDATED_MEAN_SUBSET);
    assert!(
        report
            .summary_line()
            .starts_with("Aspects gate (mean subset): "),
        "{}",
        report.summary_line()
    );
    let lines = report.pair_lines();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines.iter().all(|line| line.starts_with("mean ")), "{lines:?}");
}

#[test]
fn a_tampered_corpus_fails_the_checksum_in_either_scope() {
    // A geo row, which the subset does not compare: the whole-corpus
    // checksum must still catch it.
    let tampered = CORPUS_CSV.replacen("geo,Sun,Moon,", "geo,Sun,Moon, ", 1);
    assert_ne!(tampered, CORPUS_CSV);
    assert!(matches!(
        validate(&tampered, MANIFEST),
        Err(AspectsError::ChecksumMismatch { .. })
    ));
    assert!(matches!(
        validate_scoped(&tampered, MANIFEST, Scope::MeanSubset),
        Err(AspectsError::ChecksumMismatch { .. })
    ));
}

#[test]
fn manifest_row_count_drift_fails_closed() {
    let (rows, checksum) = parse_manifest(MANIFEST).unwrap();
    let manifest = format!(
        "slice aspects file=aspects.csv role=aspects rows={} checksum={checksum}",
        rows + 1
    );
    assert!(matches!(
        validate_scoped(CORPUS_CSV, &manifest, Scope::MeanSubset),
        Err(AspectsError::ManifestDrift { .. })
    ));
}

#[test]
fn a_corpus_missing_an_event_fails_the_count() {
    let line = first_mean_mars_saturn_conjunction();
    let without = CORPUS_CSV.replacen(&format!("{line}\n"), "", 1);
    assert_ne!(without, CORPUS_CSV);
    match validate_scoped(&without, &manifest_for(&without), Scope::MeanSubset) {
        Err(AspectsError::CountMismatch { series, got, want }) => {
            assert_eq!(series, "mean Mars-Saturn 0");
            assert_eq!(got, want + 1);
        }
        other => panic!("{other:?}"),
    }
}

// 0.05 day at Mars–Saturn's relative speed is tens of arcseconds, far over
// the measured ceiling.
#[test]
fn a_shifted_reference_instant_exceeds_the_separation_ceiling() {
    let line = first_mean_mars_saturn_conjunction();
    let mut fields: Vec<String> = line.split(',').map(str::to_string).collect();
    let jd: f64 = fields[4].parse().unwrap();
    fields[4] = format!("{:.7}", jd + 0.05);
    let shifted = CORPUS_CSV.replacen(line, &fields.join(","), 1);
    assert_ne!(shifted, CORPUS_CSV);
    match validate_scoped(&shifted, &manifest_for(&shifted), Scope::MeanSubset) {
        Err(AspectsError::CeilingExceeded { series, kind, .. }) => {
            assert_eq!(series, "mean Mars-Saturn 0");
            assert_eq!(kind, "separation_arcsec");
        }
        other => panic!("{other:?}"),
    }
}
```

- [ ] **Step 2: Run the gate to verify it fails on the checksum, and take the checksum**

Run: `cargo nextest run -p pleiades-validate aspects_gate_passes_within_ceilings --no-capture`
Expected: FAIL with `corpus checksum mismatch: got <N> want 0`.

Replace `checksum=0` in `crates/pleiades-validate/data/aspects-corpus/manifest.txt` with `checksum=<N>`.

- [ ] **Step 3: Measure with infinite ceilings**

Run, in the foreground, with no edits while it runs:

`cargo nextest run -p pleiades-validate -E 'test(aspects_gate_passes_within_ceilings) or test(mean_subset_passes_and_compares_only_the_mean_group)' --no-capture`

Expected: `aspects_gate_passes_within_ceilings` PASSES (ceilings are infinite, floor is 1) and prints the summary line, eleven pair lines and `full gate: <T> s`. `mean_subset_passes_and_compares_only_the_mean_group` FAILS on `assert_eq!(report.rows_validated, MIN_ROWS_VALIDATED_MEAN_SUBSET)` (the floor is still 1) after printing its two pair lines and `mean subset: <T> s`. Save the whole output; Steps 4 and 6 and Task 6 use it.

Three outcomes stop the task and go to the user as a design question; do not work around any of them:

1. **A `CountMismatch` or `SideMismatch`.** The engine and Swiss Ephemeris disagree on which events exist. Report the series, both counts and the instants of the unmatched events (print both lists for that series).
2. **The full gate takes more than 600 s in this (dev) profile.** The nightly `test-full` tier runs it in the dev profile; the stations gate takes 339 s there.
3. **The mean subset takes more than 60 s in this (dev) profile.** It runs in blocking `release-smoke`; the stations subset takes 24 s there.

Also check every pair line before going on: a `max sep` above 10″, or a `max lon` above 5″ for a planet pair, is larger than the difference between the two ephemerides explains (the stations gate's largest planet longitude residual is 2.314″). Investigate the largest offending event (print it, compare both longitudes and the relative speed) and report what you find before setting a ceiling over it.

- [ ] **Step 4: Write the measured thresholds**

Rewrite `crates/pleiades-validate/src/aspects_thresholds.rs` with measured values. For each pair name, take the largest `max sep` and the largest `max lon` over that pair's lines (Mercury–Venus, Mars–Saturn and Mars–Jupiter each have two lines); the ceiling is that value times 1.5, rounded up to two significant figures. `MIN_ROWS_VALIDATED` is the full gate's `rows_validated`; `MIN_ROWS_VALIDATED_MEAN_SUBSET` is the subset's. Record the measured value and its group in a comment beside each ceiling. The file keeps this shape:

```rust
//! Measured-basis ceilings for the `validate-aspects` gate.
//!
//! Measured on <date> by running the gate with infinite ceilings against
//! the committed corpus; each ceiling is the largest value over the pair's
//! corpus groups and angles times 1.5, rounded up to two significant
//! figures. The measured value and its group are recorded beside each
//! ceiling.
//!
//! What the residuals are. The corpus holds the instants at which the
//! difference of two Swiss Ephemeris (Moshier) longitudes equals an angle;
//! the engine finds the same instants in the packaged (DE440-derived)
//! backend's longitudes. A difference between the two separations moves the
//! instant by that difference divided by the pair's relative longitude
//! speed, which falls to zero where the separation turns. A time ceiling
//! would therefore mean nothing for a slow pair near a station, so the gate
//! bounds the time residual multiplied by the corpus's relative speed at
//! the event: the separation residual, in arcseconds. The largest time
//! residual per pair is reported by the gate and recorded in
//! `docs/follow-ups.md` (FU-22), with no ceiling of its own.
//!
//! The longitude ceiling bounds the difference of either body's longitude
//! at the event. It includes the body's motion over the time residual.

/// Ceilings for one pair, over all of its groups and angles.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ceilings {
    /// The time between the engine's event and the reference's, multiplied
    /// by the pair's relative longitude speed at the event: the time
    /// residual expressed as an angle.
    pub(crate) sep_arcsec: f64,
    /// The longitude difference of either body at the event.
    pub(crate) lon_arcsec: f64,
}

/// Fail-closed floor on compared events: the count the gate compared on
/// <date> (<count>).
pub(crate) const MIN_ROWS_VALIDATED: usize = <count>;

/// Fail-closed floor for the release-battery subset (the `mean` group, see
/// `validate_aspects_corpus_subset`): the count that subset compared on
/// <date> (<count>).
pub(crate) const MIN_ROWS_VALIDATED_MEAN_SUBSET: usize = <count>;

/// Ceilings by corpus pair name (`"Mars-Saturn"`), or `None` for a pair the
/// gate does not cover.
pub(crate) fn ceilings_for(pair: &str) -> Option<Ceilings> {
    match pair {
        // measured max <x>" (<group>), <y>" (<group>)
        "Sun-Moon" => Some(Ceilings {
            sep_arcsec: <x * 1.5, rounded up>,
            lon_arcsec: <y * 1.5, rounded up>,
        }),
        // ...one arm per pair, in the order of PAIRS: Sun-Mercury,
        // Mercury-Venus, Venus-Mars, Mars-Jupiter, Mars-Saturn,
        // Jupiter-Saturn, Saturn-Pluto...
        _ => None,
    }
}
```

Every `<...>` above is filled from the Step 3 output; the committed file contains none, and has all eight arms written out.

- [ ] **Step 5: Wire the gate**

In `crates/pleiades-validate/src/lib.rs`, add beside the `pub use stations_validation::{...};` block:

```rust
pub use aspects_validation::{
    validate_aspects_corpus, validate_aspects_corpus_subset, AspectsError, AspectsReport,
};
```

In `crates/pleiades-validate/src/render/cli.rs`:

- In `run_all_numeric_gates`, directly after the `crate::validate_stations_corpus_subset()` call and its `.map_err(...)?;` line, add:

```rust
    // The full aspects gate takes minutes; it runs as `validate-aspects`
    // and in nightly `test-full`. The battery compares the mean subset.
    crate::validate_aspects_corpus_subset()
        .map_err(|e| format!("aspects gate (mean subset) failed: {e}"))?;
```

- In `render_cli`, directly after the `Some("validate-stations") | Some("stations-gate") => { ... }` arm, add:

```rust
        Some("validate-aspects") | Some("aspects-gate") => {
            ensure_no_extra_args(&args[1..], "validate-aspects")?;
            crate::validate_aspects_corpus()
                .map(|report| report.summary_line().to_string())
                .map_err(|e| e.to_string())
        }
```

- In the help text (the long string literal starting `"{banner}\nCommands:\n`), find `  stations-gate             Alias for validate-stations\n` and insert directly after it:

```
  validate-aspects          Run the fail-closed exact-aspect gate (Swiss Ephemeris reference: the instants two bodies' ecliptic separation equals 0, 60, 90, 120 or 180 degrees, event for event) over the committed aspects corpus\n  aspects-gate              Alias for validate-aspects\n
```

In `crates/pleiades-cli/src/cli.rs`, directly after the line `Some("validate-stations") | Some("stations-gate") => validate_render_cli(args),` add:

```rust
        Some("validate-aspects") | Some("aspects-gate") => validate_render_cli(args),
```

In `crates/pleiades-validate/src/aspects_validation.rs`: remove any `#[allow(dead_code)]` added in Task 4 Step 7, and extend the doc comments of `validate_aspects_corpus` and `validate_aspects_corpus_subset` with the measured event counts and wall times from Step 3 (for example "floor `MIN_ROWS_VALIDATED` (N events). About T s in the dev profile (date)").

- [ ] **Step 6: Run the gate tests and the command**

Run, in the foreground: `cargo nextest run -p pleiades-validate aspects`
Expected: PASS — the 15 Task 4 tests and the 6 engine tests. The four subset-scope tests each scan the `mean` group, so this takes several times the subset's wall time.

Run: `cargo run -q --release -p pleiades-validate -- validate-aspects`
Expected: exit 0 and the `Aspects gate: ...` summary line. Record the release-profile wall time (`time` the command).

Run: `cargo run -q -p pleiades-validate -- aspects-gate extra`
Expected: a non-zero exit and a message that `validate-aspects` takes no extra arguments.

Run: `cargo run -q -p pleiades-cli -- validate-aspects` (use the CLI crate's binary name if it differs; `grep -n '^name' crates/pleiades-cli/Cargo.toml`)
Expected: exit 0 and the same summary line.

Run: `time mise run release-smoke`
Expected: PASS. Record the wall time. The stations plan measured 150 s for `release-smoke`; if this run is more than 60 s over that figure, stop and report to the user before committing.

Run: `cargo nextest run -p pleiades-validate -E 'test(help) or test(render_cli) or test(commands)'`
Expected: PASS. If a test pins the help text or a command list, update the pinned text with the two new command lines and include that file in the commit.

- [ ] **Step 7: Lint and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: no warnings.

```bash
git add crates/pleiades-validate crates/pleiades-cli
git commit -m "feat(validate): validate-aspects gate against the Swiss Ephemeris exact-aspect corpus (#84)"
```

---

### Task 6: Documentation, compatibility profile and follow-ups

**Files:**
- Modify: `crates/pleiades-events/README.md`
- Modify: `README.md` (capability table)
- Modify: `crates/pleiades-events/Cargo.toml` (`description`)
- Modify: `crates/pleiades-core/src/compatibility/mod.rs`
- Modify: `crates/pleiades-cli/src/cli/tests/summary_commands.rs:440`
- Modify: `crates/pleiades-validate/src/tests/render_request.rs:333`
- Modify: `docs/follow-ups.md`
- Modify: `spec/astrology-domain.md:82`
- Modify: `docs/superpowers/specs/2026-10-02-aspect-exactness-design.md` (status line)

**Interfaces:**
- Consumes: the pair lines, validated counts and wall times from Task 5.
- Produces: no code interfaces.

- [ ] **Step 1: Crate README**

In `crates/pleiades-events/README.md`, change the first paragraph's ending from `their ecliptic positions and speeds, and their stations.` to `their ecliptic positions and speeds, their stations, and the exact moments of the aspects between them.`, and append this section at the end of the file, replacing each `<...>` with the Task 5 measurement for that pair's `geo` line:

```markdown
## Aspects

`EventEngine::aspects_in_range(first, second, angle, reference, start, end)`
and `EventEngine::next_aspect(first, second, angle, reference, after)` find
the instants the ecliptic separation of two bodies equals an angle. An
`AspectEvent` carries the TDB instant and both longitudes.

The angle is an unsigned separation from 0° to 180°. An angle strictly
between the two is found on both sides: asking for 90° returns the moments
the first body is 90° ahead of the second and the moments it is 90° behind,
and the two longitudes say which. The moment a pair enters a 3° orb of a
square is the exact moment of the 87° or 93° separation. A returned instant
can be handed back to `next_aspect`, which then returns the following event.

A pair that approaches an angle and turns back before reaching it returns
nothing. A sidereal zodiac changes the reported longitudes, not the instants:
the ayanamsa cancels in the separation.

The search steps by the smaller of the two bodies' steps (0.25 day for the
Moon and the lunar points, 1 day for the Sun, Mercury and Venus, 2 days
otherwise) and splits each step where the separation turns, so two exact
moments inside one step, around a station, are both found. Two turning points
within two steps of each other may go unseen, and an event within two steps
of either end of the 1900–2100 window is not reported.

An aspect's instant is firm for a fast pair and soft for a slow pair near a
station, where the pair's relative speed is close to zero and a small
difference between two ephemerides is a large time difference.
`validate-aspects` compares the engine, event for event at 0°, 60°, 90°, 120°
and 180°, with the instants at which the difference of two Swiss Ephemeris
(Moshier) longitudes equals the angle, 1900–2100 for the planet pairs. The
separation difference is the time difference multiplied by the pair's
relative speed:

| Pair | Events | Largest separation difference | Largest time difference | Largest longitude difference |
|---|---|---|---|---|
| Sun–Moon (1990–2030) | <count> | <measured>″ | <measured> s | <measured>″ |
| Sun–Mercury | <count> | <measured>″ | <measured> s | <measured>″ |
| Mercury–Venus | <count> | <measured>″ | <measured> s | <measured>″ |
| Venus–Mars | <count> | <measured>″ | <measured> s | <measured>″ |
| Mars–Jupiter | <count> | <measured>″ | <measured> s | <measured>″ |
| Mars–Saturn | <count> | <measured>″ | <measured> s | <measured>″ |
| Jupiter–Saturn | <count> | <measured>″ | <measured> s | <measured>″ |
| Saturn–Pluto | <count> | <measured>″ | <measured> s | <measured>″ |

The gate also covers Mercury–Venus and Mars–Saturn in the mean place and
Mars–Jupiter from the Sun. Aspects of the lunar points, asteroids and
fictitious bodies are found but not gated.
```

- [ ] **Step 2: Workspace README and crate description**

In `README.md`, add this row to the capability table directly below the `Planetary stations` row (use the largest `geo` separation difference from Task 5, rounded up to a whole arcsecond, for the last cell):

```markdown
| Exact aspects between two bodies (geocentric apparent or mean of date, heliocentric; tropical or sidereal) | [`pleiades-events`](crates/pleiades-events) | `validate-aspects` | event for event with Swiss Ephemeris; separation within <measured>″ at the exact moment |
```

In `crates/pleiades-events/Cargo.toml`, change the `description` to:

```toml
description = "Ephemeris event-finding for the pleiades astrology workspace: longitude crossings (solcross / mooncross / general-body / heliocentric helio_cross), ecliptic positions, planetary stations and exact aspects, derived from pleiades' validated body positions."
```

- [ ] **Step 3: Compatibility profile**

In `crates/pleiades-core/src/compatibility/mod.rs`:

- Line 26: change `pleiades-compatibility-profile/0.7.19` to `pleiades-compatibility-profile/0.7.20`.
- Append this entry to the string list that ends with the `Planetary stations (issue #85) additions: ...` entry, directly after it:

```rust
            "Exact aspects (issue #84) additions: EventEngine::aspects_in_range and EventEngine::next_aspect find the instants the ecliptic separation of two bodies equals an angle, in any CrossingFrame and zodiac, returning an AspectEvent (TDB instant and both longitudes). The angle is an unsigned separation from 0 to 180 degrees and is found on both sides; a non-finite or out-of-range angle, or the same body twice, is EventError::InvalidAspect. Each scan step is split where the separation turns, so two exact moments inside one step are both found. Swiss Ephemeris has no aspect finder, so this goes beyond parity; the validate-aspects gate compares event for event, at 0, 60, 90, 120 and 180 degrees, against the instants at which the difference of two Swiss Ephemeris longitudes equals the angle, for eight geocentric pairs (seven over 1900-2100, Sun-Moon over 1990-2030), two mean-of-date pairs and one heliocentric pair.",
```

In `crates/pleiades-cli/src/cli/tests/summary_commands.rs:440` and `crates/pleiades-validate/src/tests/render_request.rs:333`, change `0.7.19` to `0.7.20`.

Run: `cargo nextest run -p pleiades-core compatibility`
Expected: PASS. If a test pins a checksum or a line count of the profile text, update the pinned value to the one the failure message reports, and include that file in the commit.

Run: `cargo nextest run -p pleiades-validate render_request`
Expected: PASS.

Run: `cargo nextest run -p pleiades-cli summary_commands`
Expected: PASS.

- [ ] **Step 4: Follow-ups and spec**

Append to `docs/follow-ups.md`, replacing each `<...>` from the Task 5 output:

````markdown
---

## FU-22: Exact-aspect event finder (issue #84)

**Status:** resolved (<date>) · Spec
`docs/superpowers/specs/2026-10-02-aspect-exactness-design.md`, plan
`docs/superpowers/plans/2026-10-02-aspect-exactness.md`.

`EventEngine::aspects_in_range` and `EventEngine::next_aspect`
(`pleiades-events` `src/aspects.rs`) find the instants the ecliptic
separation of two bodies equals an unsigned angle, on both sides. The scan
(`root.rs` `level_crossings_in_range`) splits each step at the turning points
of the separation. `EventError` gained `InvalidAspect` (not breaking).

**Gate:** `validate-aspects` (<rows>-row Swiss Ephemeris exact-aspect corpus,
`tools/se-aspects-reference`). Measured <date>:

```
<the summary line and the eleven pair lines printed in Task 5 Step 3>
```

The gate runs in two tiers:

- **Release battery** (`run_all_numeric_gates`: blocking `release-smoke`, and
  the battery tests in nightly `test-full`): `validate_aspects_corpus_subset`
  verifies the whole corpus's checksum and row count, then compares the
  `mean` group only (<count> events), about <measured> s in the dev profile.
  `release-smoke` went from <before> s to <after> s with this change.
- **Full gate** (`validate-aspects`, and the
  `aspects_gate_passes_within_ceilings` test in nightly `test-full`, which
  `release-gate` depends on): all 11 pairs, <count> events, <measured> s in
  release and <measured> s in the dev profile.

**Open items:**

- **(a) The turning-point split is untested by the corpus.** Over the eight
  geocentric corpus pairs and 200 years, a plain sign-change scan and the
  turning-point scan return the same events; the closest two events are 1.59
  days apart (Mercury–Venus conjunctions) against a 1-day step. The split is
  covered by the synthetic `root::level_tests` only.
- **(b) Two turning points within two steps of each other** may go unseen,
  and with them a pair of exact moments between them. No planetary pair does
  this; an oscillating lunar point (true node, osculating apogee) paired with
  a slow body might.
- **(c) No graze in the corpus.** The reference tool fails if a separation
  turns within 30″ of an angle, so a near-tangent pair of events, whose
  existence would depend on the ephemeris, is never compared. The closest
  turn in the corpus pairs is 102″ (Venus–Mars at 60°). Adding a pair or an
  angle that grazes needs a gate rule for it first.
- **(d) `previous_aspect`** is not provided; it would inherit FU-13's
  backward-search caveat.
- **(e) No user-facing CLI aspects command**, and no batch call over several
  angles or pairs. The gate scans each pair once per angle for that reason.
- **(f) Ungated bodies.** The lunar points, asteroids and fictitious bodies
  are accepted by the finders but have no reference corpus.
- **(g) Cost for a pair that never reaches the angle.** `next_aspect` scans to
  the end of the window before returning `None` (about 0.15 ms per step).

**Severity:** (a)–(c) documented limits, (d)–(f) feature gaps, (g)
performance · **Opened:** <date>
````

In `spec/astrology-domain.md`, change the bullet `- aspects and orb-ready angular separations` to:

```markdown
- aspects and orb-ready angular separations, and the instants an aspect between two bodies is exact (`pleiades-events` `EventEngine::aspects_in_range` / `next_aspect`)
```

In `docs/superpowers/specs/2026-10-02-aspect-exactness-design.md`, change the status line to `**Status:** implemented (<date>) ·`.

- [ ] **Step 5: Verify the docs build and the audits pass**

Run: `mise run docs`
Expected: no rustdoc warnings.

Run: `mise run audit`
Expected: PASS. If the workspace audit lists reference tools or README claims that must mention the new tool or gate, make the addition it names and include it in the commit.

Run: `mise run claims-audit`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add README.md crates docs spec
git commit -m "docs: exact aspects in the READMEs, compatibility profile and follow-ups (#84)"
```

---

### Task 7: Final verification

**Files:** none modified unless a check fails.

**Interfaces:**
- Consumes: every earlier task.
- Produces: a branch ready for review.

- [ ] **Step 1: Run the blocking tier**

Run: `mise run ci`
Expected: PASS. Run it in the foreground and make no edits or commits while it runs.

- [ ] **Step 2: Run the validation crate's aspect and battery tests**

Run: `cargo nextest run -p pleiades-validate -E 'test(aspects) or test(run_all_numeric_gates)'`
Expected: PASS.

- [ ] **Step 3: Confirm the existing event surfaces did not move**

Run: `cargo run -q -p pleiades-validate -- validate-crossings`
Expected: exit 0. The summary line must equal the one `main` prints; if in doubt, run the same command in a temporary `git worktree` of `main` and compare the two lines.

Run: `cargo run -q --release -p pleiades-validate -- validate-stations`
Expected: exit 0, and the summary line recorded under FU-21 in `docs/follow-ups.md` (5542 stations, max time 117467.6 s, max lon 51.338″).

Run: `git diff --stat main -- '*crossings-golden*' crates/pleiades-validate/data/crossings-corpus crates/pleiades-validate/data/stations-corpus`
Expected: empty.

Run: `git diff main -- crates/pleiades-events/src/root.rs | grep '^-' | grep -v '^---'`
Expected: empty (the diff to `root.rs` only adds lines).

- [ ] **Step 4: Confirm the branch contents**

Run: `git status --short`
Expected: clean.

Run: `git log --oneline main..HEAD`
Expected: the three spec commits, the plan commit, and one commit per Task 1–6, each ending with `(#84)`.

- [ ] **Step 5: Report**

Summarise: the measured per-pair maxima, the validated event counts, the gate wall times (full and subset, dev and release) and the `release-smoke` time before and after, any investigation Task 3 Step 4 or Task 5 Step 3 triggered and its outcome, and the open items recorded in FU-22.
