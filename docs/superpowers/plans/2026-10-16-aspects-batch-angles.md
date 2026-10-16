# Aspects at Several Angles in One Scan: Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Three new `EventEngine` methods that find the exact aspects of one pair at several angles in a single scan, bit-identical to merging the single-angle results. The `validate-aspects` gate and the CLI's range search move to them.

**Architecture:** `root::scan_levels` tags each root with the index of the level it crossed. `aspects.rs` maps level indices back to the caller's angles. The single-angle methods become wrappers over the batch methods with `&[angle]`.

**Tech Stack:** Rust (toolchain from `mise.toml`), `cargo nextest`, the packaged backend (`pleiades-data`) for integration tests.

**Spec:** `docs/superpowers/specs/2026-10-16-aspects-batch-angles-design.md`

## Global Constraints

- No change to the scanner's sampling, bracketing or bisection. Single-angle results stay bit-identical.
- No change to the `validate-aspects` corpus, ceilings or event counts.
- New public methods: `aspects_in_range_for_angles`, `next_aspect_for_angles`, `previous_aspect_for_angles`, with the exact signatures in Task 2.
- Validation order: empty slice → each angle (finite, 0–180) → repeated angle (equal `degrees()`) → same body twice → window → frame/zodiac. All are `EventError::InvalidAspect` up to the body check.
- CLI: only `--from/--to` switches to the batch call. `--next`/`--previous` keep one event per angle. A repeated `--angle` is the usage error `--angle <degrees> given twice`.
- Compatibility profile `pleiades-compatibility-profile/0.7.36 → 0.7.37`. API stability profile unchanged.
- `cargo fmt --all` before every commit. `-D warnings` for clippy and rustdoc.
- Do not edit source while a background test run is in progress (release-bundle tests compare git provenance).

## Review Focus

1. **Angles not in ascending order** (`[180, 0, 90]`): events must still ascend by instant, each tagged with its own angle. Pinned in Task 2's property test (the angle sets are deliberately unsorted).
2. **`0.0` and `-0.0` given together** count as a repeated angle, not two levels at the same place. Pinned in Task 2's validation test.
3. **An empty or inverted range** with several angles is an empty list, not an error. (The anchor read at `start` that surfaces a backend error is unchanged scanner code, already covered by `root::level_tests`.) Pinned in Task 2 (`an_empty_range_is_empty_for_any_angle_set`).
4. **Chaining `previous_aspect_for_angles` at an instant it returned** gives that same event, as the single-angle method does. Pinned in Task 2's next/previous property test.
5. **A NaN angle after valid ones** is `InvalidAspect` for the NaN, not reported as a duplicate (NaN ≠ NaN). Pinned in Task 2's validation test.

---

### Task 1: The level scanner tags each root with its level

**Files:**
- Modify: `crates/pleiades-events/src/root.rs` (`level_roots_between`, `drain_until`, `scan_levels`, `level_crossings_in_range`, `first_level_crossing_after`, `last_level_crossing_before`)
- Modify: `crates/pleiades-events/src/root/level_tests.rs` (import shims + one new test)
- Modify: `crates/pleiades-events/src/aspects.rs` (three call sites drop the tag, temporarily)

**Interfaces:**
- Produces (all `pub(crate)`):
  - `level_crossings_in_range<F>(d: F, levels: &[f64], lo_jd: f64, hi_jd: f64, step_days: f64, window: Window) -> Result<Vec<(f64, usize)>, EventError>`
  - `first_level_crossing_after<F>(...same...) -> Result<Option<(f64, usize)>, EventError>`
  - `last_level_crossing_before<F>(...same...) -> Result<Option<(f64, usize)>, EventError>`
  - The `usize` is the index into `levels` of the level that root crosses.

- [ ] **Step 1: Write the failing test** in `crates/pleiades-events/src/root/level_tests.rs`. Change the `use super::{...}` block to import the tagged scanners under other names, and add local untagged shims so every existing test keeps its `Vec<f64>`/`Option<f64>` view:

```rust
use super::{
    crossings_in_range, first_level_crossing_after as tagged_first_after,
    last_level_crossing_before as tagged_last_before,
    level_crossings_in_range as tagged_in_range, wrap180, Window, LEVEL_CHUNK_STEPS,
    REFINE_TOLERANCE_DAYS,
};
use crate::error::EventError;
use std::cell::Cell;

// The scanners tag each root with its level (issue #168 (e)). Most tests
// here are about instants only, so they go through these untagged views.
fn level_crossings_in_range<F>(
    d: F,
    levels: &[f64],
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
    window: Window,
) -> Result<Vec<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    tagged_in_range(d, levels, lo_jd, hi_jd, step_days, window)
        .map(|roots| roots.into_iter().map(|(jd, _)| jd).collect())
}

fn first_level_crossing_after<F>(
    d: F,
    levels: &[f64],
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
    window: Window,
) -> Result<Option<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    tagged_first_after(d, levels, lo_jd, hi_jd, step_days, window)
        .map(|root| root.map(|(jd, _)| jd))
}

fn last_level_crossing_before<F>(
    d: F,
    levels: &[f64],
    lo_jd: f64,
    hi_jd: f64,
    step_days: f64,
    window: Window,
) -> Result<Option<f64>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    tagged_last_before(d, levels, lo_jd, hi_jd, step_days, window)
        .map(|root| root.map(|(jd, _)| jd))
}
```

Then add, after `two_levels_crossed_in_one_bracket_come_back_in_time_order`:

```rust
#[test]
fn each_root_carries_the_index_of_its_level() {
    // Both levels are crossed inside one 2-day step, the later-listed level
    // first: the tags follow the levels, the order follows time.
    let d = |jd: f64| Ok(jd - T0 - 10.5);
    let levels = [0.3, -0.3];
    let roots = tagged_in_range(d, &levels, T0, T0 + 30.0, 2.0, UNBOUNDED).unwrap();
    assert_eq!(roots.iter().map(|&(_, level)| level).collect::<Vec<_>>(), [1, 0]);
    assert_settled(roots[0].0, T0 + 10.2);
    assert_settled(roots[1].0, T0 + 10.8);

    // A level's instants do not depend on which other levels are scanned
    // with it: the brackets come from `d` alone.
    let alone = tagged_in_range(d, &[0.3], T0, T0 + 30.0, 2.0, UNBOUNDED).unwrap();
    assert_eq!(alone, [(roots[1].0, 0)]);

    let first = tagged_first_after(d, &levels, T0, T0 + 30.0, 2.0, UNBOUNDED).unwrap();
    assert_eq!(first, Some(roots[0]));
    let last = tagged_last_before(d, &levels, T0, T0 + 30.0, 2.0, UNBOUNDED).unwrap();
    assert_eq!(last, Some(roots[1]));
}
```

- [ ] **Step 2: Run it to verify it fails to compile**

Run: `cargo nextest run -p pleiades-events --lib root::level_tests`
Expected: compile error. `tagged_in_range` returns `Vec<f64>`, so `(_, level)` patterns and the `(f64, usize)` comparisons don't type-check.

- [ ] **Step 3: Tag the roots in `root.rs`.** In `level_roots_between`, change `out: &mut Vec<f64>` to `out: &mut Vec<(f64, usize)>`, and change the loop and sort to:

```rust
    let first_new = out.len();
    for (index, &level) in levels.iter().enumerate() {
        let f_a = wrap180(a.1 - level);
        let f_b = wrap180(b.1 - level);
        if brackets_crossing(f_a, f_b) {
            let mut f = |jd: f64| Ok(wrap180(d(jd)? - level));
            out.push((bisect(&mut f, a.0, f_a, b.0)?, index));
        }
    }
    out[first_new..].sort_by(|x, y| x.0.total_cmp(&y.0));
    Ok(())
```

Update its doc's first line to: `/// Appends the roots of d − level, for each of levels, in the bracket [a, b], ascending, each with the index of its level.` (keep the backticks the existing doc uses).

In `drain_until`, change `out: &mut Vec<f64>` to `out: &mut Vec<(f64, usize)>`. In `scan_levels`, change the return type to `Result<Vec<(f64, usize)>, EventError>`. The body is unchanged: `let mut out = Vec::new();` infers the new type.

Change the three wrappers' return types and bodies:

```rust
/// Every instant in `(lo_jd, hi_jd]` at which the wrapped-degree function `d`
/// crosses one of `levels`, ascending, each with the index into `levels` of
/// the level it crosses. Each is a settled instant, as from [`bisect`].
```
(Keep the rest of that doc block as is.) The body stays `scan_levels(d, levels, lo_jd, hi_jd, step_days, false, window)`, and the return type becomes `Result<Vec<(f64, usize)>, EventError>`.

`first_level_crossing_after` returns `Result<Option<(f64, usize)>, EventError>` with an unchanged body (`.into_iter().next()`). `last_level_crossing_before` returns `Result<Option<(f64, usize)>, EventError>`, and its two `if let Some(&last) = roots.last()` lines are unchanged (tuples are `Copy`).

- [ ] **Step 4: Keep `aspects.rs` compiling, behavior unchanged.** In `aspects_in_range`, change `.map(|jd| self.aspect_at(...))` to `.map(|(jd, _)| self.aspect_at(&first, &second, angle, &reference, jd))`. In `next_aspect`:

```rust
        let jd = root
            .map(|(jd, _)| jd)
            .filter(|&jd| jd > after_jd)
            .ok_or_else(|| past_window_end(search.step))?;
```

In `previous_aspect`: `let jd = root.map(|(jd, _)| jd).ok_or_else(|| before_window_start(search.step))?;`

- [ ] **Step 5: Run the events tests**

Run: `cargo fmt --all && cargo nextest run -p pleiades-events`
Expected: all pass, including `each_root_carries_the_index_of_its_level`.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-events/src/root.rs crates/pleiades-events/src/root/level_tests.rs crates/pleiades-events/src/aspects.rs
git commit -m "refactor(events): tag each level-scan root with its level (#168 (e))"
```

---

### Task 2: Batch aspect methods, single-angle wrappers, tests and rustdoc

**Files:**
- Modify: `crates/pleiades-events/src/aspects.rs`
- Create: `crates/pleiades-events/tests/aspects_batch.rs`

**Interfaces:**
- Consumes: the tagged scanners from Task 1.
- Produces (public, on `impl<B: EphemerisBackend> EventEngine<B>`):

```rust
pub fn aspects_in_range_for_angles(&self, first: CelestialBody, second: CelestialBody, angles: &[Angle], reference: impl Into<CrossingReference>, start: Instant, end: Instant) -> Result<Vec<AspectEvent>, EventError>
pub fn next_aspect_for_angles(&self, first: CelestialBody, second: CelestialBody, angles: &[Angle], reference: impl Into<CrossingReference>, after: Instant) -> Result<Option<AspectEvent>, EventError>
pub fn previous_aspect_for_angles(&self, first: CelestialBody, second: CelestialBody, angles: &[Angle], reference: impl Into<CrossingReference>, before: Instant) -> Result<Option<AspectEvent>, EventError>
```

- [ ] **Step 1: Write the failing integration tests.** Create `crates/pleiades-events/tests/aspects_batch.rs`:

```rust
//! `EventEngine::aspects_in_range_for_angles`, `next_aspect_for_angles` and
//! `previous_aspect_for_angles` against the single-angle methods: one scan
//! gives each angle's events bit for bit (issue #168 (e)).

use pleiades_data::packaged_backend;
use pleiades_events::{AspectEvent, CrossingFrame, CrossingReference, EventEngine, EventError};
use pleiades_types::{Angle, Ayanamsa, CelestialBody, Instant, JulianDay, TimeScale};

const GEO: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const HELIO: CrossingFrame = CrossingFrame::Heliocentric;
const J2000: f64 = 2_451_545.0;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn angles(degrees: &[f64]) -> Vec<Angle> {
    degrees.iter().map(|&d| Angle::from_degrees(d)).collect()
}

fn jd(event: &AspectEvent) -> f64 {
    event.instant.julian_day.days()
}

/// One search case: a pair, its angles (deliberately unsorted), a
/// reference and a range.
struct Case {
    first: CelestialBody,
    second: CelestialBody,
    angles: Vec<Angle>,
    reference: CrossingReference,
    start: f64,
    end: f64,
}

fn cases() -> Vec<Case> {
    use CelestialBody::{Jupiter, Mars, Mercury, Moon, Saturn, Sun};
    vec![
        // A fast pair: conjunction, opposition and both quarters.
        Case {
            first: Sun,
            second: Moon,
            angles: angles(&[180.0, 0.0, 90.0]),
            reference: GEO.into(),
            start: J2000,
            end: J2000 + 120.0,
        },
        // A slow pair whose separation turns: the 1980-81 triple conjunction.
        Case {
            first: Jupiter,
            second: Saturn,
            angles: angles(&[60.0, 0.0, 120.0]),
            reference: GEO.into(),
            start: 2_444_239.5,
            end: 2_445_000.5,
        },
        // A bounded pair: 60 degrees is out of reach and must not disturb 20.
        Case {
            first: Sun,
            second: Mercury,
            angles: angles(&[60.0, 20.0]),
            reference: GEO.into(),
            start: J2000,
            end: J2000 + 400.0,
        },
        // Heliocentric and sidereal references.
        Case {
            first: Mars,
            second: Jupiter,
            angles: angles(&[90.0, 180.0, 0.0]),
            reference: HELIO.into(),
            start: J2000,
            end: J2000 + 3000.0,
        },
        Case {
            first: Sun,
            second: Moon,
            angles: angles(&[120.0, 60.0]),
            reference: CrossingReference::sidereal(GEO, Ayanamsa::Lahiri),
            start: J2000,
            end: J2000 + 60.0,
        },
    ]
}

/// The single-angle results merged in time order: what one scan must give.
fn merged_singles(engine: &EventEngine<impl pleiades_backend::EphemerisBackend>, case: &Case) -> Vec<AspectEvent> {
    let mut all = Vec::new();
    for &angle in &case.angles {
        all.extend(
            engine
                .aspects_in_range(
                    case.first.clone(),
                    case.second.clone(),
                    angle,
                    case.reference.clone(),
                    tdb(case.start),
                    tdb(case.end),
                )
                .expect("single-angle aspects"),
        );
    }
    all.sort_by(|a, b| jd(a).total_cmp(&jd(b)));
    all
}

#[test]
fn one_scan_gives_each_angles_events_bit_for_bit() {
    let engine = EventEngine::new(packaged_backend());
    for case in cases() {
        let batch = engine
            .aspects_in_range_for_angles(
                case.first.clone(),
                case.second.clone(),
                &case.angles,
                case.reference.clone(),
                tdb(case.start),
                tdb(case.end),
            )
            .expect("batch aspects");
        let singles = merged_singles(&engine, &case);
        assert!(!singles.is_empty(), "{}-{}: no events", case.first, case.second);
        // `AspectEvent: PartialEq` compares the f64 fields with `==`.
        assert_eq!(batch, singles, "{}-{}", case.first, case.second);
    }
}

#[test]
fn next_and_previous_are_the_nearest_of_the_single_angle_answers() {
    let engine = EventEngine::new(packaged_backend());
    for case in cases() {
        let label = format!("{}-{}", case.first, case.second);
        let singles = merged_singles(&engine, &case);
        // From the range start, then chained from each returned event.
        let mut after = case.start;
        for expected in singles.iter() {
            let next = engine
                .next_aspect_for_angles(
                    case.first.clone(),
                    case.second.clone(),
                    &case.angles,
                    case.reference.clone(),
                    tdb(after),
                )
                .expect("next")
                .expect("an event");
            assert_eq!(&next, expected, "{label} next after {after}");
            // Handed back, a returned instant gives that same event.
            let previous = engine
                .previous_aspect_for_angles(
                    case.first.clone(),
                    case.second.clone(),
                    &case.angles,
                    case.reference.clone(),
                    next.instant,
                )
                .expect("previous")
                .expect("an event");
            assert_eq!(previous, next, "{label} previous at {}", jd(&next));
            after = jd(&next);
        }
    }
}

#[test]
fn every_angle_out_of_reach_answers_as_one_does() {
    let engine = EventEngine::new(packaged_backend());
    let (sun, mercury) = (CelestialBody::Sun, CelestialBody::Mercury);
    let far = angles(&[60.0, 90.0]);
    let range = engine
        .aspects_in_range_for_angles(sun.clone(), mercury.clone(), &far, GEO, tdb(J2000), tdb(J2000 + 400.0))
        .expect("range");
    assert!(range.is_empty());
    let single = engine.next_aspect(sun.clone(), mercury.clone(), far[0], GEO, tdb(J2000));
    let batch = engine.next_aspect_for_angles(sun.clone(), mercury.clone(), &far, GEO, tdb(J2000));
    assert!(matches!(single, Err(EventError::OutOfWindow { .. })), "{single:?}");
    assert_eq!(format!("{batch:?}"), format!("{single:?}"));
    let single = engine.previous_aspect(sun.clone(), mercury.clone(), far[0], GEO, tdb(J2000));
    let batch = engine.previous_aspect_for_angles(sun, mercury, &far, GEO, tdb(J2000));
    assert_eq!(format!("{batch:?}"), format!("{single:?}"));
}

#[test]
fn an_empty_range_is_empty_for_any_angle_set() {
    let engine = EventEngine::new(packaged_backend());
    for (start, end) in [(J2000, J2000), (J2000 + 10.0, J2000)] {
        let events = engine
            .aspects_in_range_for_angles(
                CelestialBody::Sun,
                CelestialBody::Moon,
                &angles(&[0.0, 90.0, 180.0]),
                GEO,
                tdb(start),
                tdb(end),
            )
            .expect("an empty range is not an error");
        assert!(events.is_empty());
    }
}

fn invalid_detail(result: Result<Vec<AspectEvent>, EventError>) -> String {
    match result {
        Err(EventError::InvalidAspect { detail }) => detail,
        other => panic!("expected InvalidAspect, got {other:?}"),
    }
}

#[test]
fn angle_sets_are_validated_in_order() {
    let engine = EventEngine::new(packaged_backend());
    let search = |degrees: &[f64], second: CelestialBody| {
        engine.aspects_in_range_for_angles(
            CelestialBody::Sun,
            second,
            &angles(degrees),
            GEO,
            tdb(J2000),
            tdb(J2000 + 30.0),
        )
    };
    let moon = CelestialBody::Moon;
    assert!(invalid_detail(search(&[], moon.clone())).contains("at least one angle"));
    // The angle check comes before the repeat check: a NaN is not a repeat.
    assert!(invalid_detail(search(&[90.0, f64::NAN, f64::NAN], moon.clone())).contains("between 0 and 180"));
    assert!(invalid_detail(search(&[90.0, 200.0, 90.0], moon.clone())).contains("between 0 and 180"));
    assert!(invalid_detail(search(&[90.0, 0.0, 90.0], moon.clone())).contains("given twice"));
    // 0 and -0 are the same separation.
    assert!(invalid_detail(search(&[0.0, -0.0], moon)).contains("given twice"));
    // Then the bodies.
    assert!(invalid_detail(search(&[90.0, 0.0], CelestialBody::Sun)).contains("two different bodies"));
}
```

(`pleiades-backend` is already a dev-dependency of `pleiades-events`, so `merged_singles` can name the trait.)

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo nextest run -p pleiades-events --test aspects_batch`
Expected: compile error, `no method named aspects_in_range_for_angles`.

- [ ] **Step 3: Implement in `aspects.rs`.** Replace `struct Search` and `aspect_search` with:

```rust
/// What the finders settle before scanning.
struct Search {
    /// The levels of every angle in reach, in the order of the angles.
    levels: Vec<f64>,
    /// For each of `levels`, the index into the caller's angles of the angle
    /// it belongs to.
    owners: Vec<usize>,
    step: f64,
    /// No angle is in reach ([`separation_bound`]): the finders answer after
    /// one read instead of scanning.
    unreachable: bool,
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Validates an aspect request. The angles are checked first (at least
    /// one, each a separation, none repeated), then the bodies, the window,
    /// and the frame and zodiac for each body.
    fn aspect_search(
        &self,
        first: &CelestialBody,
        second: &CelestialBody,
        angles: &[Angle],
        reference: &CrossingReference,
        instants_jd: [f64; 2],
    ) -> Result<Search, EventError> {
        if angles.is_empty() {
            return Err(invalid(
                "an aspect search needs at least one angle".to_string(),
            ));
        }
        let per_angle = angles
            .iter()
            .map(|&angle| levels_for(angle))
            .collect::<Result<Vec<_>, _>>()?;
        for (index, angle) in angles.iter().enumerate() {
            if angles[..index].iter().any(|earlier| earlier.degrees() == angle.degrees()) {
                return Err(invalid(format!(
                    "the angle {} degrees is given twice",
                    angle.degrees()
                )));
            }
        }
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
        let bound = separation_bound(first, second, reference.frame);
        let mut levels = Vec::new();
        let mut owners = Vec::new();
        for (index, (angle, angle_levels)) in angles.iter().zip(per_angle).enumerate() {
            if bound.is_some_and(|bound| angle.degrees() > bound) {
                continue;
            }
            owners.extend(std::iter::repeat_n(index, angle_levels.len()));
            levels.extend(angle_levels);
        }
        let unreachable = levels.is_empty();
        if unreachable {
            // The one read an empty range gets, for the same backend errors.
            separation(&self.backend, first, second, reference, instants_jd[0])?;
        }
        Ok(Search {
            levels,
            owners,
            step,
            unreachable,
        })
    }
```

(`aspect_at` is unchanged.) Replace the three public methods' bodies so that each single-angle method delegates. `aspects_in_range` keeps its whole doc comment and becomes:

```rust
    pub fn aspects_in_range(
        &self,
        first: CelestialBody,
        second: CelestialBody,
        angle: Angle,
        reference: impl Into<CrossingReference>,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<AspectEvent>, EventError> {
        self.aspects_in_range_for_angles(first, second, &[angle], reference, start, end)
    }
```

`next_aspect` and `previous_aspect` likewise become `self.next_aspect_for_angles(first, second, &[angle], reference, after)` and `self.previous_aspect_for_angles(first, second, &[angle], reference, before)`. Their docs and doctests stay. Append to each of the three docs:

```rust
    /// [`EventEngine::aspects_in_range_for_angles`] finds several angles in
    /// one scan.
```

(Use `next_aspect_for_angles` and `previous_aspect_for_angles` in the other two.)

Add the batch methods after `previous_aspect`:

```rust
    /// All instants in `[start, end]` (TDB) at which the ecliptic separation
    /// of `first` and `second` equals any of `angles`, ascending, in one scan
    /// of the pair.
    ///
    /// Each event's [`AspectEvent::angle`] is the angle it belongs to. For
    /// each angle the events are exactly those of
    /// [`EventEngine::aspects_in_range`] for that angle, bit for bit: the
    /// scan's brackets come from the separation alone, and each angle's
    /// moments are refined on their own. Scanning the angles together saves
    /// reading both bodies at every step once per angle.
    ///
    /// An angle beyond the pair's bound (Mercury and Venus seen from the
    /// Earth, see [`EventEngine::aspects_in_range`]) contributes no events
    /// and does not stop the others being found.
    ///
    /// The meaning of an angle, the range, the accuracy and the limits are
    /// those of [`EventEngine::aspects_in_range`].
    ///
    /// ```
    /// use pleiades_data::packaged_backend;
    /// use pleiades_events::{CrossingFrame, EventEngine};
    /// use pleiades_types::{Angle, CelestialBody, Instant, JulianDay, TimeScale};
    ///
    /// // The Moon's phases over January 2000: new, first quarter, full and
    /// // last quarter, in time order.
    /// let engine = EventEngine::new(packaged_backend());
    /// let tdb = |jd: f64| Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
    /// let quarters = [0.0, 90.0, 180.0].map(Angle::from_degrees);
    /// let phases = engine
    ///     .aspects_in_range_for_angles(
    ///         CelestialBody::Sun,
    ///         CelestialBody::Moon,
    ///         &quarters,
    ///         CrossingFrame::GeocentricApparentOfDate,
    ///         tdb(2_451_545.0),
    ///         tdb(2_451_574.0),
    ///     )
    ///     .unwrap();
    /// let angles: Vec<f64> = phases.iter().map(|event| event.angle.degrees()).collect();
    /// assert_eq!(angles, [0.0, 90.0, 180.0, 90.0]);
    /// ```
    ///
    /// # Errors
    ///
    /// [`EventError::InvalidAspect`] for an empty `angles`, for any angle
    /// [`EventEngine::aspects_in_range`] rejects, or for an angle given
    /// twice; otherwise those of [`EventEngine::aspects_in_range`].
    pub fn aspects_in_range_for_angles(
        &self,
        first: CelestialBody,
        second: CelestialBody,
        angles: &[Angle],
        reference: impl Into<CrossingReference>,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<AspectEvent>, EventError> {
        let reference = reference.into();
        let start_jd = start.julian_day.days();
        let end_jd = end.julian_day.days();
        let search =
            self.aspect_search(&first, &second, angles, &reference, [start_jd, end_jd])?;
        if search.unreachable {
            return Ok(Vec::new());
        }
        let roots = level_crossings_in_range(
            |jd| separation(&self.backend, &first, &second, &reference, jd),
            &search.levels,
            start_jd,
            end_jd,
            search.step,
            WINDOW,
        )?;
        roots
            .into_iter()
            .map(|(jd, level)| {
                let angle = angles[search.owners[level]];
                self.aspect_at(&first, &second, angle, &reference, jd)
            })
            .collect()
    }

    /// The first instant strictly after `after` at which the ecliptic
    /// separation of `first` and `second` equals any of `angles`.
    ///
    /// The earliest of the [`EventEngine::next_aspect`] answers for each
    /// angle, bit for bit, found in one scan that stops at the first event of
    /// any angle. [`EventError::OutOfWindow`] only when no angle has an event
    /// before the window's end. The meaning of `after` and the errors are
    /// those of [`EventEngine::next_aspect`] and
    /// [`EventEngine::aspects_in_range_for_angles`].
    pub fn next_aspect_for_angles(
        &self,
        first: CelestialBody,
        second: CelestialBody,
        angles: &[Angle],
        reference: impl Into<CrossingReference>,
        after: Instant,
    ) -> Result<Option<AspectEvent>, EventError> {
        let reference = reference.into();
        let after_jd = after.julian_day.days();
        let search =
            self.aspect_search(&first, &second, angles, &reference, [after_jd, after_jd])?;
        if search.unreachable {
            return Err(past_window_end(search.step));
        }
        let root = first_level_crossing_after(
            |jd| separation(&self.backend, &first, &second, &reference, jd),
            &search.levels,
            after_jd,
            WINDOW_END_JD,
            search.step,
            WINDOW,
        )?;
        let (jd, level) = root
            .filter(|&(jd, _)| jd > after_jd)
            .ok_or_else(|| past_window_end(search.step))?;
        let angle = angles[search.owners[level]];
        self.aspect_at(&first, &second, angle, &reference, jd)
            .map(Some)
    }

    /// The last instant at or before `before` at which the ecliptic
    /// separation of `first` and `second` equals any of `angles`.
    ///
    /// The latest of the [`EventEngine::previous_aspect`] answers for each
    /// angle, bit for bit, found in one backward walk that stops at the first
    /// chunk holding an event of any angle. [`EventError::OutOfWindow`] only
    /// when no angle has an event after the window's start. The meaning of
    /// `before` and the errors are those of [`EventEngine::previous_aspect`]
    /// and [`EventEngine::aspects_in_range_for_angles`].
    pub fn previous_aspect_for_angles(
        &self,
        first: CelestialBody,
        second: CelestialBody,
        angles: &[Angle],
        reference: impl Into<CrossingReference>,
        before: Instant,
    ) -> Result<Option<AspectEvent>, EventError> {
        let reference = reference.into();
        let before_jd = before.julian_day.days();
        let search =
            self.aspect_search(&first, &second, angles, &reference, [before_jd, before_jd])?;
        if search.unreachable {
            return Err(before_window_start(search.step));
        }
        let root = last_level_crossing_before(
            |jd| separation(&self.backend, &first, &second, &reference, jd),
            &search.levels,
            WINDOW_START_JD,
            before_jd,
            search.step,
            WINDOW,
        )?;
        let (jd, level) = root.ok_or_else(|| before_window_start(search.step))?;
        let angle = angles[search.owners[level]];
        self.aspect_at(&first, &second, angle, &reference, jd)
            .map(Some)
    }
```

Add to the module doc (top of `aspects.rs`), after the existing paragraph: `//! Several angles of one pair are found in one scan: each root carries the index of its level, and each level the index of its angle.`

- [ ] **Step 4: Run all events tests and the doctests**

Run: `cargo fmt --all && cargo nextest run -p pleiades-events && cargo test -p pleiades-events --doc aspects`
Expected: all pass, including the five new `aspects_batch` tests and the new doctest. If the doctest's angle sequence differs, check the dates rather than editing the assertion: the new Moon of 6 January 2000 (JD ≈ 2451550.3), first quarter 14 Jan, full 21 Jan and last quarter 28 Jan all fall in JD 2451545–2451574.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/aspects.rs crates/pleiades-events/tests/aspects_batch.rs
git commit -m "feat(events): find aspects at several angles in one scan (#168 (e))"
```

---

### Task 3: The `validate-aspects` gate scans each pair once

**Files:**
- Modify: `crates/pleiades-validate/src/aspects_validation.rs` (the per-pair loop, around lines 590–640)
- Modify: `mise.toml` (the `[tasks.gate-aspects]` timing comment)

**Interfaces:**
- Consumes: `EventEngine::aspects_in_range_for_angles` (Task 2).

- [ ] **Step 1: Measure the gate before the change.** Check `cat /proc/loadavg`. If the 1-minute load is above 3, wait (sibling sessions share this machine). Then, on the commit from Task 2:

Run: `time mise run gate-aspects`
Expected: PASS. Record the wall-clock and the load.

- [ ] **Step 2: Replace the per-angle engine calls** with one call per pair, split by angle. Replace the `for (angle_index, angle_deg) in ANGLES_DEG.iter().enumerate() { ... }` loop with:

```rust
        let angles = ANGLES_DEG.map(Angle::from_degrees);
        let events = engine
            .aspects_in_range_for_angles(
                first.clone(),
                second.clone(),
                &angles,
                pair.group.reference(),
                tdb(pair.span.0),
                tdb(pair.span.1),
            )
            .map_err(|e| failed(&pair_label, e.to_string()))?;
        for (angle_index, angle_deg) in ANGLES_DEG.iter().enumerate() {
            let label = format!("{pair_label} {angle_deg:.0}");
            let corpus: Vec<Expected> = rows
                .iter()
                .filter(|row| row.pair == pair_index && row.angle == angle_index)
                .map(|row| row.expected)
                .collect();
            let found: Vec<Found> = events
                .iter()
                .filter(|event| event.angle.degrees() == *angle_deg)
                .map(|event| Found {
                    jd: event.instant.julian_day.days(),
                    first_lon_deg: event.first_longitude.degrees(),
                    second_lon_deg: event.second_longitude.degrees(),
                })
                .collect();
            residuals.absorb(compare_exact(
                &label, *angle_deg, &found, &corpus, ceilings,
            )?);
            counts.push(format!("{angle_deg:.0}°: {}", found.len()));
        }
```

`event.angle` is the `Angle` built from `*angle_deg`, so the `==` filter is exact. Each angle's events keep their ascending order.

- [ ] **Step 3: Run the gate after the change**, under the same load conditions as Step 1:

Run: `time mise run gate-aspects`
Expected: PASS with the same per-pair `compared (...)` counts as Step 1. Record the wall-clock.

- [ ] **Step 4: Update the `[tasks.gate-aspects]` comment in `mise.toml`.** Add one line after the existing nightly-runner timing: `# One scan per pair over all five angles since #168 (e): <before> s -> <after> s locally (load <x>).` Use the measured values.

- [ ] **Step 5: Run the blocking subset**

Run: `cargo fmt --all && mise run release-smoke`
Expected: PASS (it holds the gate's mean subset).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-validate/src/aspects_validation.rs mise.toml
git commit -m "perf(validate): scan each aspect-gate pair once for all angles (#168 (e))"
```

---

### Task 4: The CLI's range search makes one call per pair; a repeated `--angle` is an error

**Files:**
- Modify: `crates/pleiades-cli/src/commands/events.rs` (`render_aspects`, around lines 419–475)
- Test: `crates/pleiades-cli/src/cli/tests/events.rs`

**Interfaces:**
- Consumes: `EventEngine::aspects_in_range_for_angles` (Task 2).

- [ ] **Step 1: Write the failing test** in `crates/pleiades-cli/src/cli/tests/events.rs`, after `aspects_next_and_previous_give_one_event_per_pair_and_angle`:

```rust
#[test]
fn aspects_rejects_a_repeated_angle() {
    // Before #168 (e) a repeated angle printed every event twice.
    for search in [
        &["--from", J2000, "--to", END_OF_2000][..],
        &["--next", "--at", J2000][..],
    ] {
        let mut args = vec!["aspects", "--pair", "Sun,Moon", "--angle", "90", "--angle", "90"];
        args.extend_from_slice(search);
        assert!(error(&args).contains("--angle 90 given twice"), "{search:?}");
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo nextest run -p pleiades-cli aspects_rejects_a_repeated_angle`
Expected: FAIL with "the command should be rejected" (the range search succeeds today).

- [ ] **Step 3: Implement.** In `render_aspects`'s argument loop, replace the `"--angle"` arm with:

```rust
            "--angle" => {
                let angle = Angle::from_degrees(parse_f64(iter.next(), "--angle")?);
                if angles.iter().any(|seen| seen.degrees() == angle.degrees()) {
                    return Err(format!("--angle {} given twice", angle.degrees()));
                }
                angles.push(angle);
            }
```

Replace the search loop over `pairs` with:

```rust
    for (first, second) in &pairs {
        match &search {
            Search::Range { from, to } => found.extend(
                engine
                    .aspects_in_range_for_angles(
                        first.clone(),
                        second.clone(),
                        &angles,
                        reference.clone(),
                        *from,
                        *to,
                    )
                    .map_err(event_error)?,
            ),
            // One event per angle, as before: the batch call would give one
            // in all.
            Search::Next(at) | Search::Previous(at) => {
                for &angle in &angles {
                    let label = format!("{first}–{second} {}°", angle.degrees());
                    let (first, second, reference) =
                        (first.clone(), second.clone(), reference.clone());
                    if matches!(search, Search::Next(_)) {
                        found_or_note(
                            engine.next_aspect(first, second, angle, reference, *at),
                            Direction::Next,
                            &mut found,
                            &mut notes,
                            &label,
                        )?;
                    } else {
                        found_or_note(
                            engine.previous_aspect(first, second, angle, reference, *at),
                            Direction::Previous,
                            &mut found,
                            &mut notes,
                            &label,
                        )?;
                    }
                }
            }
        }
    }
```

The output order needs no change: `render` sorts the lines by Julian day (`events.rs:267`). Both `Search::Next` and `Search::Previous` hold an `Instant`, so the or-pattern binds `at`. `ASPECTS_USAGE` is unchanged, and the repeated-angle rule is documented in `docs/cli.md` in Task 5.

- [ ] **Step 4: Run the CLI tests**

Run: `cargo fmt --all && cargo nextest run -p pleiades-cli`
Expected: all pass, including `aspects_loops_over_pairs_and_angles_in_time_order`, `aspects_next_and_previous_give_one_event_per_pair_and_angle`, the asteroid gate test and the new test.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-cli/src/commands/events.rs crates/pleiades-cli/src/cli/tests/events.rs
git commit -m "feat(cli): one aspect scan per pair; reject a repeated --angle (#168 (e))"
```

---

### Task 5: README, CLI docs, compatibility notes and the profile bump

**Files:**
- Modify: `crates/pleiades-events/README.md` (the `## Aspects` section)
- Modify: `docs/cli.md` (the `aspects` command, if documented there; `grep -n "aspects" docs/cli.md`)
- Modify: `crates/pleiades-core/src/compatibility/mod.rs` (`CURRENT_COMPATIBILITY_PROFILE_ID` and a new notes entry after the Apophis entry)
- Modify: `crates/pleiades-cli/src/cli/tests/summary_commands.rs`, `crates/pleiades-validate/src/tests/render_request.rs` (pinned profile id)
- Modify: `docs/status.md` if it names the profile (`grep -n "0.7.36" docs/`)

- [ ] **Step 1: README.** In `crates/pleiades-events/README.md`, after the paragraph that ends "...hold each event exactly once.", add:

```markdown
`EventEngine::aspects_in_range_for_angles`, `next_aspect_for_angles` and
`previous_aspect_for_angles` take several angles for one pair and scan the
pair once. Each event's `angle` says which angle it belongs to. For each angle
the events are exactly those of the single-angle call, bit for bit, and next
and previous return the nearest event of any angle. An empty angle list or an
angle given twice is `EventError::InvalidAspect`.
```

- [ ] **Step 2: CLI docs.** Where `docs/cli.md` describes `aspects`, add: `A range search scans each pair once for all its angles. Giving the same --angle twice is an error.` If `docs/cli.md` doesn't describe `aspects`, skip this step and say so in the commit message.

- [ ] **Step 3: Compatibility notes.** In `crates/pleiades-core/src/compatibility/mod.rs`, set `CURRENT_COMPATIBILITY_PROFILE_ID` to `"pleiades-compatibility-profile/0.7.37"`, and add after the Apophis (issue #201) entry:

```rust
            "Aspects at several angles (issue #168 (e)) additions: EventEngine::aspects_in_range_for_angles, next_aspect_for_angles and previous_aspect_for_angles take a slice of angles for one pair and scan it once; each AspectEvent's angle names the angle it belongs to, the range result equals the time-merge of the single-angle results bit for bit, and next/previous return the nearest event of any angle. An empty slice or a repeated angle is EventError::InvalidAspect; an angle beyond the pair's bound contributes no events. The single-angle methods are unchanged. The validate-aspects gate now scans each pair once for its five angles, with unchanged counts and ceilings. Behaviour change: pleiades-cli aspects rejects a repeated --angle, which used to print every event twice. Compatibility profile bumped to 0.7.37; API stability profile unchanged.",
```

- [ ] **Step 4: Update the pinned profile id.**

Run: `git grep -n "compatibility-profile/0.7.36"`
In each hit outside `docs/superpowers/` (`summary_commands.rs`, `render_request.rs`), change `0.7.36` to `0.7.37`. Then run `git grep -n "0\.7\.36"` and update any remaining release-facing hit (for example `docs/status.md`) the same way. Leave historical text such as "bumped to 0.7.36" in older notes entries alone.

- [ ] **Step 5: Run the affected tests** (validate's slow tier is `#[ignore]`d, so use libtest with `--include-ignored` for the files that pin the profile):

Run: `cargo fmt --all && cargo nextest run -p pleiades-core -p pleiades-cli && cargo test -p pleiades-validate --lib -- --include-ignored render_request summary report release`
Expected: PASS. If a release-bundle or checksum test fails on the profile string, update its pinned value the same way and re-run. Never change a checksum without re-generating it through the test's own documented path.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-events/README.md docs/cli.md crates/pleiades-core/src/compatibility/mod.rs crates/pleiades-cli/src/cli/tests/summary_commands.rs crates/pleiades-validate/src/tests/render_request.rs docs/status.md
git commit -m "docs: document aspects at several angles; bump the compatibility profile to 0.7.37 (#168 (e))"
```

(Drop from `git add` any path that wasn't changed.)

---

### Task 6: Whole-branch verification

**Files:** none changed unless a check fails.

- [ ] **Step 1: Static checks**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && mise run docs`
Expected: clean.

- [ ] **Step 2: Full test tier.** Start it and make no edits or commits until it finishes:

Run: `mise run test-full`
Expected: PASS.

- [ ] **Step 3: Gates**

Run: `mise run gate-aspects && mise run release-smoke && mise run audit`
Expected: PASS, with the same aspect counts as Task 3 Step 1.

- [ ] **Step 4:** If anything failed, fix it in the task that owns the code, commit, and re-run this task from Step 1.
