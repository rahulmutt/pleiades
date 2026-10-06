# Event Searches at the Window's Ends Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Crossing, station, aspect and occultation searches in `pleiades-events` scan up to the 1900–2100 window's limits, and a `next_*`/`previous_*` search the window cuts short returns `EventError::OutOfWindow` instead of `Ok(None)` (issue #208).

**Architecture:** Remove the one/two-step clamps in `crossings.rs`, `stations.rs`, `aspects.rs`, `occult.rs`. The crossing scanners in `root.rs` already never sample outside `[lo, hi]`; the level scanner (`root::scan_levels`) gains a window so its look-around samples are clamped to it. Two helpers in `error.rs` build the "window ended" error. The CLI turns that error from a `--next`/`--previous` search into a per-entry note.

**Tech Stack:** Rust (workspace toolchain from `mise.toml`), `cargo nextest`, `mise` tasks.

**Spec:** `docs/superpowers/specs/2026-10-06-event-window-edges-design.md`

## Global Constraints

- Window: `WINDOW_START_JD = 2_415_020.5`, `WINDOW_END_JD = 2_488_069.5` (`crates/pleiades-events/src/error.rs`). Do not change them.
- A forward search that reaches `WINDOW_END_JD` without an event returns `EventError::OutOfWindow { julian_day: WINDOW_END_JD + step }`; a backward one reaching `WINDOW_START_JD` returns `OutOfWindow { julian_day: WINDOW_START_JD - step }`, where `step` is that search's scan step.
- `Ok(None)` stays only for `never_stations` bodies (stations) and `target_never_occultable` targets (occultations).
- Range searches keep returning a list; a range that needs a sample within a light-time of `WINDOW_START_JD` (apparent frame, any body but the Sun and the lunar points) returns `OutOfWindow`.
- Do not change any parity gate's corpus or tolerances.
- No `unwrap`/`expect` in library code. `cargo fmt --all` before every commit (CI gate).
- Do not commit or edit sources while a background `nextest`/`test-full` run is in progress.
- `pleiades-validate` tests: run with `cargo test -p pleiades-validate … -- --include-ignored` for the slow tier, never only `nextest` (it skips `#[ignore]`).
- Commit messages end with `(#208)`.

## Review Focus

1. **A range starting exactly at `WINDOW_START_JD` for the apparent Moon or a planet**: now `OutOfWindow` (was a silent late start). Pinned in Task 1 (crossings) and Task 2 (stations, rewriting `ranges_touching_the_window_edges_work`); a reader should agree this is the contract the spec chose.
2. **`next_*` with `after == WINDOW_END_JD`** (an empty range): must be `OutOfWindow`, not `Ok(None)` and not a panic. Pinned in Tasks 1, 2, 4.
3. **The Sun, mean-of-date and heliocentric frames at `WINDOW_START_JD`**: readable from the first instant, so a range starting there works and a backward search there reports `OutOfWindow` only because the window is exhausted. Pinned in Task 1 (Sun, MEAN).
4. **Aspect look-around at the window's start for the apparent Moon**: the clamped look-around read fails with `OutOfWindow` and must be treated as "no sample", not propagated, when the range itself starts after the light-time sliver. Pinned in Task 3 (unit) and Task 4 (Sun–Moon range starting at `WINDOW_START_JD + 0.01`).
5. **CLI `--next` over several pairs where one never reaches its angle**: the others still print and the exhausted one gets a note. Pinned in Task 6.

---

### Task 1: Crossings reach the window's ends

**Files:**
- Modify: `crates/pleiades-events/src/error.rs` (helpers, `OutOfWindow` doc)
- Modify: `crates/pleiades-events/src/crossings.rs` (clamps at ~127-128, ~197-198, ~253-254; docs; `#[cfg(test)] mod window_edge_tests;`)
- Create: `crates/pleiades-events/src/window_edge_support.rs` (test support)
- Modify: `crates/pleiades-events/src/lib.rs` (`#[cfg(test)] mod window_edge_support;`)
- Create: `crates/pleiades-events/src/crossings/window_edge_tests.rs`
- Modify: `crates/pleiades-events/tests/reference.rs` (`previous_crossing_at_the_window_start_is_none`)

**Interfaces:**
- Produces: `pub(crate) fn past_window_end(step_days: f64) -> EventError` and `pub(crate) fn before_window_start(step_days: f64) -> EventError` in `error.rs`.
- Produces (test-only): `crate::window_edge_support::{tdb(jd: f64) -> Instant, out_of_window_jd<T: Debug>(result: Result<T, EventError>) -> f64}`.

- [ ] **Step 1: Write the test support module**

`crates/pleiades-events/src/window_edge_support.rs`:

```rust
//! Shared helpers for the window-edge regression tests (issue #208).

use crate::error::EventError;
use core::fmt::Debug;
use pleiades_types::{Instant, JulianDay, TimeScale};

/// A TDB instant.
pub(crate) fn tdb(julian_day: f64) -> Instant {
    Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb)
}

/// The Julian day an `OutOfWindow` error names; panics on anything else.
pub(crate) fn out_of_window_jd<T: Debug>(result: Result<T, EventError>) -> f64 {
    match result {
        Err(EventError::OutOfWindow { julian_day }) => julian_day,
        other => panic!("expected OutOfWindow, got {other:?}"),
    }
}
```

Add to `lib.rs` next to the other private modules:

```rust
#[cfg(test)]
mod window_edge_support;
```

- [ ] **Step 2: Write the failing crossing tests**

Add `#[cfg(test)] mod window_edge_tests;` at the end of `crossings.rs` (next to `mod tests`), and create `crates/pleiades-events/src/crossings/window_edge_tests.rs`. The fixtures are constructed: the target is the body's own longitude at an instant inside the old margin, so a crossing exists there by construction.

```rust
//! Issue #208: crossings within one scan step of the window's ends are found,
//! and a search the window cuts short is `OutOfWindow`.

use super::*;
use crate::window_edge_support::{out_of_window_jd, tdb};
use pleiades_data::packaged_backend;

const APPARENT: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const MEAN: CrossingFrame = CrossingFrame::GeocentricMeanOfDate;
/// The Moon's crossing step (`EventEngine::step_days`).
const MOON_STEP: f64 = 0.25;
/// Bisection tolerance plus slack: a returned instant trails its crossing by
/// less than 0.5 s.
const SETTLE_DAYS: f64 = 1.0 / 86_400.0;

fn engine() -> EventEngine<pleiades_data::PackagedDataBackend> {
    EventEngine::new(packaged_backend())
}

/// The Moon's longitude at `jd` in `frame`, as a crossing target.
fn moon_at(frame: CrossingFrame, jd: f64) -> Longitude {
    let degrees = engine()
        .longitude_at(CelestialBody::Moon, frame, tdb(jd))
        .unwrap();
    Longitude::from_degrees(degrees)
}

#[test]
fn a_moon_crossing_inside_the_last_step_is_found() {
    let at = WINDOW_END_JD - 0.1;
    let target = moon_at(APPARENT, at);
    let next = engine()
        .next_longitude_crossing(CelestialBody::Moon, target, APPARENT, tdb(WINDOW_END_JD - 0.2))
        .unwrap()
        .expect("the crossing at WINDOW_END - 0.1 d");
    let found = next.instant.julian_day.days();
    assert!((0.0..SETTLE_DAYS).contains(&(found - at)), "{found} vs {at}");
    let in_range = engine()
        .longitude_crossings_in_range(
            CelestialBody::Moon,
            target,
            APPARENT,
            tdb(WINDOW_END_JD - 0.2),
            tdb(WINDOW_END_JD),
        )
        .unwrap();
    assert_eq!(in_range.len(), 1, "{in_range:?}");
}

#[test]
fn a_moon_crossing_inside_the_first_step_is_found() {
    // Mean of date: readable from the window's first instant.
    let at = WINDOW_START_JD + 0.1;
    let target = moon_at(MEAN, at);
    let previous = engine()
        .previous_longitude_crossing(CelestialBody::Moon, target, MEAN, tdb(WINDOW_START_JD + 0.2))
        .unwrap()
        .expect("the crossing at WINDOW_START + 0.1 d");
    let found = previous.instant.julian_day.days();
    assert!((0.0..SETTLE_DAYS).contains(&(found - at)), "{found} vs {at}");
    let in_range = engine()
        .longitude_crossings_in_range(
            CelestialBody::Moon,
            target,
            MEAN,
            tdb(WINDOW_START_JD),
            tdb(WINDOW_START_JD + 0.2),
        )
        .unwrap();
    assert_eq!(in_range.len(), 1, "{in_range:?}");
}

#[test]
fn a_search_the_window_cuts_short_is_out_of_window() {
    // The Moon takes ~27 d to return to a longitude; from 0.2 d before the
    // end, after passing it at -0.1 d, the window ends first.
    let target = moon_at(APPARENT, WINDOW_END_JD - 0.1);
    let next = engine().next_longitude_crossing(
        CelestialBody::Moon,
        target,
        APPARENT,
        tdb(WINDOW_END_JD - 0.05),
    );
    assert_eq!(out_of_window_jd(next), WINDOW_END_JD + MOON_STEP);
    let at_end = engine().next_longitude_crossing(
        CelestialBody::Moon,
        target,
        APPARENT,
        tdb(WINDOW_END_JD),
    );
    assert_eq!(out_of_window_jd(at_end), WINDOW_END_JD + MOON_STEP);
    let target = moon_at(MEAN, WINDOW_START_JD + 0.1);
    let previous = engine().previous_longitude_crossing(
        CelestialBody::Moon,
        target,
        MEAN,
        tdb(WINDOW_START_JD + 0.05),
    );
    assert_eq!(out_of_window_jd(previous), WINDOW_START_JD - MOON_STEP);
}

#[test]
fn an_apparent_moon_range_from_the_first_instant_is_out_of_window() {
    // The apparent Moon is read 1.3 s earlier, before the window.
    let range = engine().longitude_crossings_in_range(
        CelestialBody::Moon,
        Longitude::from_degrees(0.0),
        APPARENT,
        tdb(WINDOW_START_JD),
        tdb(WINDOW_START_JD + 30.0),
    );
    assert!(matches!(range, Err(EventError::OutOfWindow { .. })), "{range:?}");
    // The Sun needs no light-time read and is served from the first instant.
    let sun = engine()
        .longitude_crossings_in_range(
            CelestialBody::Sun,
            Longitude::from_degrees(290.0),
            APPARENT,
            tdb(WINDOW_START_JD),
            tdb(WINDOW_START_JD + 30.0),
        )
        .unwrap();
    assert_eq!(sun.len(), 1, "{sun:?}");
}
```

`pleiades-data` is already a dev-dependency of `pleiades-events`. The Sun is at about 280.5° on 1900-01-01 and moves about 1.02°/day, so 290° is crossed once, around 10 January, inside the first 30 days.

- [ ] **Step 3: Run the tests to see them fail**

Run: `cargo nextest run -p pleiades-events window_edge`
Expected: the first three tests FAIL (`expect` on `None`, or `Ok(None)` where `OutOfWindow` is expected). `an_apparent_moon_range_from_the_first_instant_is_out_of_window` FAILS on the Moon assertion (today it starts a step later and returns a list).

- [ ] **Step 4: Add the error helpers**

In `error.rs`, below the constants:

```rust
/// The answer of a forward search that reached the window's end without an
/// event: the event may lie past the window, where it cannot be computed.
/// Names the next instant the scan would have sampled.
pub(crate) fn past_window_end(step_days: f64) -> EventError {
    EventError::OutOfWindow {
        julian_day: WINDOW_END_JD + step_days,
    }
}

/// The backward twin of [`past_window_end`].
pub(crate) fn before_window_start(step_days: f64) -> EventError {
    EventError::OutOfWindow {
        julian_day: WINDOW_START_JD - step_days,
    }
}
```

Replace the third paragraph of the `OutOfWindow` doc ("A rise, set or transit search also returns this …") with:

```rust
    /// A search for the next or previous event (rise, set, transit,
    /// longitude crossing, station, exact aspect, occultation) also returns
    /// this when it reaches the window's end, or for a backward search its
    /// start, without finding an event: the event may exist but cannot be
    /// computed. `julian_day` is then the first instant past the window the
    /// search needed. `Ok(None)` from such a search means the engine knows
    /// there is no event.
```

- [ ] **Step 5: Remove the clamps in `crossings.rs`**

`longitude_crossings_in_range`: delete the two clamp lines and their comment, and scan `start_jd..=end_jd`:

```rust
        let roots = crossings_in_range(
            |jd| { /* unchanged closure */ },
            start_jd,
            end_jd,
            step,
        )?;
```

`next_longitude_crossing`: scan `after_jd..=WINDOW_END_JD`, and turn an empty result into the error:

```rust
        let root = first_crossing_after(
            |jd| { /* unchanged closure */ },
            after_jd,
            WINDOW_END_JD,
            step,
        )?;
        let jd = root.ok_or_else(|| past_window_end(step))?;
        Ok(Some(Self::crossing(&body, target, &reference, jd)))
```

(The old `.filter(|&jd| jd > after_jd)` is no longer needed: `first_crossing_after` returns roots in `(lo, hi]`.)

`previous_longitude_crossing`: scan `WINDOW_START_JD..=before_jd`, `root.ok_or_else(|| before_window_start(step))?`, return `Ok(Some(..))`.

Import `past_window_end, before_window_start` from `crate::error`.

Docs to edit in the same file:
- `next_longitude_crossing`: replace "same clamps, same step" with "same step"; add a paragraph: "When the window ends before the next crossing, the result is [`EventError::OutOfWindow`] naming the instant one step past the window's end. Every body crosses every longitude, so this search never returns `Ok(None)`."
- `previous_longitude_crossing`: same edit for the start.
- `longitude_crossings_in_range`: add "The scan runs to both ends of the range, which may be the window's own limits. An apparent place of a body other than the Sun is read a light-time earlier, so a range starting within a light-time of the window's first instant is [`EventError::OutOfWindow`]."
- `next_sun_crossing` / `next_moon_crossing`: no change.

- [ ] **Step 6: Fix the tests that pinned the old answer**

`crates/pleiades-events/tests/reference.rs`, `previous_crossing_at_the_window_start_is_none`: rename to `previous_crossing_at_the_window_start_is_out_of_window`, keep the three `before` values, and assert `matches!(previous, Err(EventError::OutOfWindow { .. }))` (Mars apparent: nothing found before the start, and the start itself cannot be read). Update its comment: "A backward search from the first instants of the window finds nothing and reaches the window's start: it is cut short, so it is `OutOfWindow`."

Run: `cargo nextest run -p pleiades-events`
Any other failure that pins `Ok(None)` at the window's ends for a crossing search is updated the same way (assert `OutOfWindow`); a failure anywhere else is a bug, stop and investigate.

- [ ] **Step 7: Run the crate tests**

Run: `cargo nextest run -p pleiades-events && cargo test -p pleiades-events --doc`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
cargo fmt --all
git add crates/pleiades-events
git commit -m "fix(events): search crossings up to the window's ends; report a search the window cuts short (#208)"
```

---

### Task 2: Stations reach the window's ends

**Files:**
- Modify: `crates/pleiades-events/src/stations.rs` (~217-218, ~281-282, ~353-354; docs; `#[cfg(test)] mod window_edge_tests;`)
- Modify: `crates/pleiades-events/src/window_edge_support.rs` (synthetic backend)
- Create: `crates/pleiades-events/src/stations/window_edge_tests.rs`
- Modify: `crates/pleiades-events/tests/stations.rs` (`previous_station_guards_match_next_station`, `ranges_touching_the_window_edges_work`)

**Interfaces:**
- Consumes: `past_window_end`, `before_window_start`, `tdb`, `out_of_window_jd` (Task 1).
- Produces (test-only): `window_edge_support::StationingMars` (an `EphemerisBackend`) and its consts `FIRST_STATION_JD`, `LAST_STATION_JD`.

No real station lies within one step of either window end (probed 2026-10-06 for Mercury–Pluto, the true node and true apogee: the nearest is Uranus 5.3 d before the end), so the fixture is synthetic.

- [ ] **Step 1: Add the synthetic backend**

Append to `window_edge_support.rs`:

```rust
use pleiades_backend::{
    AccuracyClass, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, BodyClaim, CoordinateFrame, EphemerisBackend, EphemerisError,
    EphemerisRequest, EphemerisResult,
};
use pleiades_types::{CelestialBody, EclipticCoordinates, Latitude, Longitude, Motion, TimeRange};
use crate::error::{WINDOW_END_JD, WINDOW_START_JD};

/// Mars turns retrograde here, one day after the window's start…
pub(crate) const FIRST_STATION_JD: f64 = WINDOW_START_JD + 1.0;
/// …and direct here, one day before its end. Mars's station step is 2 days.
pub(crate) const LAST_STATION_JD: f64 = WINDOW_END_JD - 1.0;
/// Speed slope at each station, in degrees per day per day: large enough
/// that precession to the mean equinox of date (about 3.8e-5 °/day) moves
/// each station by about 3 s.
const SLOPE: f64 = 1.0;

/// Serves Mars on a J2000 longitude whose speed is
/// `k · (t − FIRST) · (t − LAST)`, positive before the first station,
/// negative between, positive after.
pub(crate) struct StationingMars;

impl StationingMars {
    fn k() -> f64 {
        SLOPE / (LAST_STATION_JD - FIRST_STATION_JD)
    }
    /// Longitude (degrees) and speed (degrees per day) at `jd`.
    fn longitude_and_speed(jd: f64) -> (f64, f64) {
        let u = jd - FIRST_STATION_JD;
        let span = LAST_STATION_JD - FIRST_STATION_JD;
        let k = Self::k();
        let longitude = 100.0 + k * (u.powi(3) / 3.0 - span * u.powi(2) / 2.0);
        let speed = k * u * (u - span);
        (longitude, speed)
    }
}

impl EphemerisBackend for StationingMars {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new("stationing-mars"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("test backend with stations at the window's ends"),
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![pleiades_types::TimeScale::Tdb],
            body_claims: vec![BodyClaim::from(CelestialBody::Mars)],
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }
    fn supports_body(&self, body: CelestialBody) -> bool {
        body == CelestialBody::Mars
    }
    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let mut result = EphemerisResult::new(
            BackendId::new("stationing-mars"),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        if req.body == CelestialBody::Mars {
            let (longitude, speed) = Self::longitude_and_speed(req.instant.julian_day.days());
            result.ecliptic = Some(EclipticCoordinates::new(
                Longitude::from_degrees(longitude),
                Latitude::from_degrees(0.0),
                Some(1.5),
            ));
            result.motion = Some(Motion::new(Some(speed), Some(0.0), Some(0.0)));
        }
        Ok(result)
    }
}
```

If `Motion::new` takes a different argument shape, read `crates/pleiades-types/src/motion.rs:66` and adapt; the three channels are longitude, latitude and distance speed.

- [ ] **Step 2: Write the failing station tests**

Add `#[cfg(test)] mod window_edge_tests;` to `stations.rs` (beside `mod tests;`) and create `crates/pleiades-events/src/stations/window_edge_tests.rs`:

```rust
//! Issue #208: stations within one scan step of the window's ends are found,
//! and a search the window cuts short is `OutOfWindow`. The fixture is the
//! synthetic `StationingMars`, with stations one day inside each end, in the
//! mean-of-date frame (readable from the window's first instant).

use super::*;
use crate::window_edge_support::{
    out_of_window_jd, tdb, StationingMars, FIRST_STATION_JD, LAST_STATION_JD,
};

const MEAN: CrossingFrame = CrossingFrame::GeocentricMeanOfDate;
/// Mars's station step (`step_days`).
const MARS_STEP: f64 = 2.0;
/// Precession moves each station by about 3 s (see `SLOPE`).
const NEAR_DAYS: f64 = 10.0 / 86_400.0;

fn engine() -> EventEngine<StationingMars> {
    EventEngine::new(StationingMars)
}

fn jd(station: &Station) -> f64 {
    station.instant.julian_day.days()
}

#[test]
fn stations_inside_the_end_steps_are_found_by_a_range() {
    let late = engine()
        .stations_in_range(CelestialBody::Mars, MEAN, tdb(WINDOW_END_JD - 1.5), tdb(WINDOW_END_JD))
        .unwrap();
    assert_eq!(late.len(), 1, "{late:?}");
    assert!((jd(&late[0]) - LAST_STATION_JD).abs() < NEAR_DAYS, "{late:?}");
    assert_eq!(late[0].kind, StationKind::TurnsDirect);
    let early = engine()
        .stations_in_range(CelestialBody::Mars, MEAN, tdb(WINDOW_START_JD), tdb(WINDOW_START_JD + 1.5))
        .unwrap();
    assert_eq!(early.len(), 1, "{early:?}");
    assert!((jd(&early[0]) - FIRST_STATION_JD).abs() < NEAR_DAYS, "{early:?}");
    assert_eq!(early[0].kind, StationKind::TurnsRetrograde);
}

#[test]
fn stations_inside_the_end_steps_are_found_by_next_and_previous() {
    let next = engine()
        .next_station(CelestialBody::Mars, MEAN, tdb(WINDOW_END_JD - 1.5))
        .unwrap()
        .expect("the last station");
    assert!((jd(&next) - LAST_STATION_JD).abs() < NEAR_DAYS, "{next:?}");
    let previous = engine()
        .previous_station(CelestialBody::Mars, MEAN, tdb(WINDOW_START_JD + 1.5))
        .unwrap()
        .expect("the first station");
    assert!((jd(&previous) - FIRST_STATION_JD).abs() < NEAR_DAYS, "{previous:?}");
}

#[test]
fn a_station_search_the_window_cuts_short_is_out_of_window() {
    let after_last = engine().next_station(CelestialBody::Mars, MEAN, tdb(WINDOW_END_JD - 0.5));
    assert_eq!(out_of_window_jd(after_last), WINDOW_END_JD + MARS_STEP);
    let at_end = engine().next_station(CelestialBody::Mars, MEAN, tdb(WINDOW_END_JD));
    assert_eq!(out_of_window_jd(at_end), WINDOW_END_JD + MARS_STEP);
    let before_first =
        engine().previous_station(CelestialBody::Mars, MEAN, tdb(WINDOW_START_JD + 0.5));
    assert_eq!(out_of_window_jd(before_first), WINDOW_START_JD - MARS_STEP);
}
```

- [ ] **Step 3: Run them to see them fail**

Run: `cargo nextest run -p pleiades-events stations::window_edge`
Expected: FAIL (empty ranges, `None`). If instead a test fails because the engine asks the stub for a body or channel it does not serve (an error naming the stub), extend `StationingMars` to serve it and re-run; do not weaken the assertions.

- [ ] **Step 4: Remove the clamps in `stations.rs`**

- `stations_in_range`: delete the clamp lines and the comment; use `start_jd`/`end_jd` for the `never_stations` sample and the scan.
- `next_station`: `never_stations` sample at `after_jd`, return `Ok(None)` as before; scan `after_jd..=WINDOW_END_JD`; then

```rust
        let jd = root.ok_or_else(|| past_window_end(step))?;
        self.station_at(&body, &reference, jd).map(Some)
```

- `previous_station`: `never_stations` sample at `before_jd`; scan `WINDOW_START_JD..=before_jd`; `root.ok_or_else(|| before_window_start(step))?`.

Docs:
- `stations_in_range`: delete the paragraph "As for the crossings, the scan is clamped one step inside …". Add: "The scan runs to both ends of the range. An apparent place of a body other than the Sun is read a light-time earlier, so a range starting within a light-time of the window's first instant is [`EventError::OutOfWindow`]."
- `next_station`: replace "Any other body that happens never to station … is searched to the end of the 1900–2100 window first." with "Any other body is searched to the end of the 1900–2100 window; when the window ends first, the result is [`EventError::OutOfWindow`] naming the instant one step past it." Delete "The window-edge clamp (a station within one step of either end of the window is not reported)," leaving "The accuracy, step and errors are those of …".
- `previous_station`: the mirror edits.

- [ ] **Step 5: Fix the tests that pinned the old answer**

`crates/pleiades-events/tests/stations.rs`:
- `previous_station_guards_match_next_station`: the last assertion (Mercury apparent, `before = WINDOW_START_JD`) becomes `assert!(matches!(engine.previous_station(CelestialBody::Mercury, GEO, tdb(WINDOW_START_JD)), Err(EventError::OutOfWindow { .. })))`, comment "The window's first instant has nothing before it: the search is cut short."
- `ranges_touching_the_window_edges_work`: the early range uses `MEAN` (readable from the first instant) and keeps `>= 4`; add an assertion that the same range in `GEO` is `Err(EventError::OutOfWindow { .. })` with comment "the apparent place is read a light-time before the first instant"; the final `next_station(Mercury, GEO, WINDOW_END_JD)` asserts `Err(EventError::OutOfWindow { julian_day })` with `julian_day == WINDOW_END_JD + 1.0` (Mercury's step).
- `bodies_that_never_station_return_nothing` and `a_body_that_never_stations_is_answered_without_scanning_the_window`: unchanged; they must still pass (`Ok(None)` for `never_stations`).

Run: `cargo nextest run -p pleiades-events`. Other failures pinning `None` at the ends for a stationing body: update to `OutOfWindow`; anything else is a bug.

- [ ] **Step 6: Run the crate tests and docs**

Run: `cargo nextest run -p pleiades-events && cargo test -p pleiades-events --doc`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
cargo fmt --all
git add crates/pleiades-events
git commit -m "fix(events): search stations up to the window's ends; report a search the window cuts short (#208)"
```

---

### Task 3: The level scanner keeps its look-around samples in the window

**Files:**
- Modify: `crates/pleiades-events/src/root.rs` (`scan_levels` ~268-334, wrappers ~340-418, their docs)
- Modify: `crates/pleiades-events/src/root/level_tests.rs` (every call gains a window argument; new tests)

**Interfaces:**
- Produces: `pub(crate) type Window = (f64, f64);` and the new signatures
  - `level_crossings_in_range(d, levels: &[f64], lo_jd: f64, hi_jd: f64, step_days: f64, window: Window) -> Result<Vec<f64>, EventError>`
  - `first_level_crossing_after(d, levels, lo_jd, hi_jd, step_days, window: Window) -> Result<Option<f64>, EventError>`
  - `last_level_crossing_before(d, levels, lo_jd, hi_jd, step_days, window: Window) -> Result<Option<f64>, EventError>`
- `window` is the closed interval `d` may be sampled in. Callers guarantee `window.0 <= lo_jd` and `hi_jd <= window.1`.

- [ ] **Step 1: Add the `window` parameter (ignored for now) and update the callers**

In `root.rs` add `pub(crate) type Window = (f64, f64);` with the doc "The closed interval of Julian days a scanned function may be sampled in." Add `window: Window` as the last parameter of `scan_levels`, `level_crossings_in_range`, `first_level_crossing_after` and `last_level_crossing_before`; the wrappers pass it through (`last_level_crossing_before` to each chunk's `scan_levels`), and `scan_levels` ignores it for now (`let _ = window;`, removed in Step 4).

In `aspects.rs`, the three call sites pass `(WINDOW_START_JD - 2.0 * search.step, WINDOW_END_JD + 2.0 * search.step)`, today's sampling envelope, so behaviour is unchanged until Task 4, with the comment `// Today's sampling envelope; Task 4 of #208 replaces it with the window.` (Task 4 removes both).

In `level_tests.rs` add `const UNBOUNDED: Window = (f64::NEG_INFINITY, f64::INFINITY);`, import `Window`, and append `, UNBOUNDED` as the last argument of every `level_crossings_in_range`, `first_level_crossing_after` and `last_level_crossing_before` call (about 30 calls; `grep -n "level_crossing\|level_crossings" crates/pleiades-events/src/root/level_tests.rs`). They must pass unchanged once Step 4 lands: an unbounded window is today's behaviour.

- [ ] **Step 2: Write the failing window tests**

Append to `level_tests.rs`:

```rust
/// `−(t − centre)² + half_width²`: a turning point at `centre` and, at level
/// 0, two crossings at `centre ± half_width`.
fn dome(centre: f64, half_width: f64) -> impl Fn(f64) -> Result<f64, EventError> + Copy {
    move |t| Ok(half_width.powi(2) - (t - centre).powi(2))
}

/// Wraps `d` so a sample outside `window` panics: the scanner must never ask.
fn fenced<F>(mut d: F, window: Window) -> impl FnMut(f64) -> Result<f64, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    move |t| {
        assert!(
            (window.0..=window.1).contains(&t),
            "sampled {t} outside {window:?}"
        );
        d(t)
    }
}

#[test]
fn no_sample_is_taken_outside_the_window() {
    let window = (T0, T0 + 30.0);
    // Ranges touching each limit, and the whole window.
    for (lo, hi) in [(T0, T0 + 10.0), (T0 + 20.0, T0 + 30.0), (T0, T0 + 30.0)] {
        let d = dome(T0 + 15.3, 1.0);
        level_crossings_in_range(fenced(d, window), &[0.0], lo, hi, 2.0, window).unwrap();
        first_level_crossing_after(fenced(d, window), &[0.0], lo, hi, 2.0, window).unwrap();
        last_level_crossing_before(fenced(d, window), &[0.0], lo, hi, 2.0, window).unwrap();
    }
}

#[test]
fn a_clamped_look_around_sample_splits_the_step_beside_a_limit() {
    let window = (T0, T0 + 30.0);
    // Range [T0 + 1, …], step 2: the look-around sample one step before the
    // range (T0 − 1) is clamped to the window's start, T0. Samples at T0,
    // T0 + 1 and T0 + 3 read −3.96, −0.96, −0.96: the turning point at
    // T0 + 2 shows, and splits the first step's two crossings apart.
    let near_start = level_crossings_in_range(
        fenced(dome(T0 + 2.0, 0.2), window),
        &[0.0],
        T0 + 1.0,
        T0 + 20.0,
        2.0,
        window,
    )
    .unwrap();
    assert_eq!(near_start.len(), 2, "{near_start:?}");
    assert!((near_start[0] - (T0 + 1.8)).abs() < 1e-4, "{near_start:?}");
    assert!((near_start[1] - (T0 + 2.2)).abs() < 1e-4, "{near_start:?}");
    // Range [T0, T0 + 29], step 2: the grid's last point past the range
    // (T0 + 30) is the window's end. Samples at T0 + 26, T0 + 28 and T0 + 30
    // show the turning point at T0 + 28.5, inside the last step
    // [T0 + 28, T0 + 29].
    let near_end = level_crossings_in_range(
        fenced(dome(T0 + 28.5, 0.2), window),
        &[0.0],
        T0,
        T0 + 29.0,
        2.0,
        window,
    )
    .unwrap();
    assert_eq!(near_end.len(), 2, "{near_end:?}");
    assert!((near_end[0] - (T0 + 28.3)).abs() < 1e-4, "{near_end:?}");
    assert!((near_end[1] - (T0 + 28.7)).abs() < 1e-4, "{near_end:?}");
}

#[test]
fn a_turning_point_in_the_step_at_a_limit_is_the_documented_blind_spot() {
    // The range starts AT the window's start: no sample exists before it,
    // so two crossings inside the first step (T0 + 0.8, T0 + 1.2) are not
    // seen. Pins the limit the docs state, so a change to it is deliberate.
    let window = (T0, T0 + 30.0);
    let roots = level_crossings_in_range(
        fenced(dome(T0 + 1.0, 0.2), window),
        &[0.0],
        T0,
        T0 + 20.0,
        2.0,
        window,
    )
    .unwrap();
    assert!(roots.is_empty(), "{roots:?}");
}

#[test]
fn an_out_of_window_look_around_read_is_no_sample() {
    // `d` cannot be read in the first 0.01 d of the window (a light-time),
    // and the range starts after that: the look-around read there fails and
    // must not fail the scan.
    let window = (T0, T0 + 30.0);
    let blind = |t: f64| {
        if t < T0 + 0.01 {
            Err(EventError::OutOfWindow { julian_day: t })
        } else {
            dome(T0 + 15.3, 1.0)(t)
        }
    };
    let roots = level_crossings_in_range(blind, &[0.0], T0 + 0.5, T0 + 30.0, 2.0, window).unwrap();
    assert_eq!(roots.len(), 2, "{roots:?}");
    // A failing read AT the range start still propagates.
    let at_start = level_crossings_in_range(blind, &[0.0], T0, T0 + 30.0, 2.0, window);
    assert!(matches!(at_start, Err(EventError::OutOfWindow { .. })), "{at_start:?}");
}
```

- [ ] **Step 3: Run them to see them fail**

Run: `cargo nextest run -p pleiades-events root::level_tests`
Expected: the new tests fail to compile until the signatures change (Step 1 already added the argument). Once they compile against the unchanged body (the `window` parameter accepted but ignored), `no_sample_is_taken_outside_the_window` and `a_clamped_look_around_sample_splits_the_step_beside_a_limit` panic "sampled … outside", and `an_out_of_window_look_around_read_is_no_sample` fails with the propagated error. `a_turning_point_in_the_step_at_a_limit_is_the_documented_blind_spot` panics too (today's scan samples T0 − 2).

- [ ] **Step 4: Implement the window in `scan_levels`**

Add the helper next to `turning_point`:

```rust
/// A look-around sample of `d` at `jd`: one taken only to see a turning
/// point. `None` when the read is `OutOfWindow` (an apparent place within a
/// light-time of the window's start); any other error propagates.
fn look_around<F>(d: &mut F, jd: f64) -> Result<Option<Sample>, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    match d(jd) {
        Ok(value) => Ok(Some((jd, value))),
        Err(EventError::OutOfWindow { .. }) => Ok(None),
        Err(error) => Err(error),
    }
}
```

Replace the body of `scan_levels` from `let mut pending = vec![anchor];` to the end with (remove the `let _ = window;` from Step 1):

```rust
    let mut pending = vec![anchor];
    // One sample before the range, to see a turning point in the first step:
    // clamped to the window, and none when the range starts at its limit.
    let before = at(-1).max(window.0);
    let mut older = if before < lo_jd {
        look_around(&mut d, before)?
    } else {
        None
    };
    let mut newer = anchor;
    // Up to one sample past the grid, to see a turning point in the last
    // step, clamped to the window's end.
    for k in 1..=intervals + 1 {
        let jd = at(k).min(window.1);
        if jd <= newer.0 {
            // `newer` is already the window's end, and `hi_jd` is in
            // `pending`: no sample is left to take.
            break;
        }
        let sample = if jd <= hi_jd {
            Some((jd, d(jd)?))
        } else {
            look_around(&mut d, jd)?
        };
        if let (Some(older), Some(sample)) = (older, sample) {
            if let Some(turn) = turning_point(&mut d, older, newer, sample)? {
                let inside = pending
                    .first()
                    .is_some_and(|earliest| turn.0 > earliest.0)
                    && turn.0 < hi_jd;
                if inside {
                    let index = pending.partition_point(|point| point.0 < turn.0);
                    pending.insert(index, turn);
                }
            }
        }
        if k < intervals || jd == hi_jd {
            // In the range, so read with `d(jd)?` above.
            pending.extend(sample);
        } else if k == intervals {
            pending.push((hi_jd, d(hi_jd)?));
        }
        // No later sample can add a breakpoint before `newer`.
        drain_until(&mut d, levels, &mut pending, newer.0, &mut out)?;
        if first_only && !out.is_empty() {
            return Ok(out);
        }
        let Some(sample) = sample else {
            // Past the range and unreadable: nothing later to compare.
            break;
        };
        older = Some(newer);
        newer = sample;
    }
    drain_until(&mut d, levels, &mut pending, hi_jd, &mut out)?;
    Ok(out)
```

Why the early `break` is safe: for `k < intervals`, `at(k) < hi_jd <= window.1`, so `jd` only reaches `window.1` at `k >= intervals`, after `hi_jd` has been pushed. With an unbounded window both `min`/`max` are no-ops, `look_around` only differs from `d(jd)?` on `OutOfWindow`, and the loop is today's.

Update the docs: `scan_levels` ("The turning points are still looked for on the uncut grid, one sample either side of the range" → "…one sample either side of the range, clamped to `window`; where the window leaves no sample, or the sample cannot be read, no turning point is looked for in that step"), `level_crossings_in_range` ("`d` is sampled only in `[lo_jd − step_days, hi_jd + 2·step_days)`" → "`d` is sampled only in that interval intersected with `window`"), and add to the Limits paragraph: "Next to a limit of `window`, a turning point in the step beside it is seen only if the window leaves a sample past it, so two crossings within the last step before the window's end (or the first after its start) may go unseen." Same for `last_level_crossing_before`.

- [ ] **Step 5: Run the tests**

Run: `cargo nextest run -p pleiades-events root`
Expected: PASS, including all pre-existing level tests with `UNBOUNDED`.

Run: `cargo nextest run -p pleiades-events`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add crates/pleiades-events
git commit -m "refactor(events): give the level scanner a window for its look-around samples (#208)"
```

---

### Task 4: Aspects reach the window's ends

**Files:**
- Modify: `crates/pleiades-events/src/aspects.rs` (`Search` ~118-126, `aspect_search` ~150-157, the three finders, docs; `#[cfg(test)] mod window_edge_tests;` — `aspects/tests.rs` exists, so the new file is `aspects/window_edge_tests.rs`)
- Create: `crates/pleiades-events/src/aspects/window_edge_tests.rs`
- Modify: `crates/pleiades-events/tests/aspects.rs` (`previous_aspect_returns_nothing_for_a_pair_that_never_reaches_the_angle`, `previous_aspect_guards_match_next_aspect`, any `next_aspect` "never reaches" test)

**Interfaces:**
- Consumes: `Window`, the windowed level-scanner signatures (Task 3); `past_window_end`, `before_window_start`; `tdb`, `out_of_window_jd`.

- [ ] **Step 1: Write the failing aspect tests**

```rust
//! Issue #208: exact aspects within two scan steps of the window's ends are
//! found, and a search the window cuts short is `OutOfWindow`. Fixtures are
//! constructed: the angle is the Sun–Moon separation at an instant inside
//! the old margin.

use super::*;
use crate::window_edge_support::{out_of_window_jd, tdb};
use pleiades_data::packaged_backend;

const APPARENT: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const MEAN: CrossingFrame = CrossingFrame::GeocentricMeanOfDate;
/// The Sun–Moon search step (the Moon's).
const STEP: f64 = 0.25;
const SETTLE_DAYS: f64 = 1.0 / 86_400.0;

fn engine() -> EventEngine<pleiades_data::PackagedDataBackend> {
    EventEngine::new(packaged_backend())
}

/// The unsigned Sun–Moon separation at `jd`, in degrees.
fn separation_at(frame: CrossingFrame, jd: f64) -> Angle {
    let e = engine();
    let sun = e.longitude_at(CelestialBody::Sun, frame, tdb(jd)).unwrap();
    let moon = e.longitude_at(CelestialBody::Moon, frame, tdb(jd)).unwrap();
    Angle::from_degrees(wrap180(moon - sun).abs())
}

#[test]
fn an_aspect_inside_the_last_two_steps_is_found() {
    let at = WINDOW_END_JD - 0.2;
    let angle = separation_at(APPARENT, at);
    let next = engine()
        .next_aspect(CelestialBody::Sun, CelestialBody::Moon, angle, APPARENT, tdb(WINDOW_END_JD - 0.4))
        .unwrap()
        .expect("the aspect at WINDOW_END - 0.2 d");
    let found = next.instant.julian_day.days();
    assert!((0.0..SETTLE_DAYS).contains(&(found - at)), "{found} vs {at}");
    let in_range = engine()
        .aspects_in_range(
            CelestialBody::Sun,
            CelestialBody::Moon,
            angle,
            APPARENT,
            tdb(WINDOW_END_JD - 0.4),
            tdb(WINDOW_END_JD),
        )
        .unwrap();
    assert_eq!(in_range.len(), 1, "{in_range:?}");
}

#[test]
fn an_aspect_inside_the_first_two_steps_is_found() {
    let at = WINDOW_START_JD + 0.2;
    let angle = separation_at(MEAN, at);
    let previous = engine()
        .previous_aspect(CelestialBody::Sun, CelestialBody::Moon, angle, MEAN, tdb(WINDOW_START_JD + 0.4))
        .unwrap()
        .expect("the aspect at WINDOW_START + 0.2 d");
    let found = previous.instant.julian_day.days();
    assert!((0.0..SETTLE_DAYS).contains(&(found - at)), "{found} vs {at}");
    let in_range = engine()
        .aspects_in_range(
            CelestialBody::Sun,
            CelestialBody::Moon,
            angle,
            MEAN,
            tdb(WINDOW_START_JD),
            tdb(WINDOW_START_JD + 0.4),
        )
        .unwrap();
    assert_eq!(in_range.len(), 1, "{in_range:?}");
}

#[test]
fn an_apparent_range_just_after_the_light_time_sliver_scans() {
    // The look-around sample one step before this start is clamped to the
    // first instant, where the apparent Moon cannot be read; that is no
    // sample, not an error.
    let found = engine().aspects_in_range(
        CelestialBody::Sun,
        CelestialBody::Moon,
        Angle::from_degrees(90.0),
        APPARENT,
        tdb(WINDOW_START_JD + 0.01),
        tdb(WINDOW_START_JD + 30.0),
    );
    assert!(found.is_ok(), "{found:?}");
    let at_start = engine().aspects_in_range(
        CelestialBody::Sun,
        CelestialBody::Moon,
        Angle::from_degrees(90.0),
        APPARENT,
        tdb(WINDOW_START_JD),
        tdb(WINDOW_START_JD + 30.0),
    );
    assert!(matches!(at_start, Err(EventError::OutOfWindow { .. })), "{at_start:?}");
}

#[test]
fn an_aspect_search_the_window_cuts_short_is_out_of_window() {
    // Mercury is never 60 degrees from the Sun; Mercury's pair step is 1 day.
    let never = engine().next_aspect(
        CelestialBody::Sun,
        CelestialBody::Mercury,
        Angle::from_degrees(60.0),
        APPARENT,
        tdb(WINDOW_END_JD - 30.0),
    );
    assert_eq!(out_of_window_jd(never), WINDOW_END_JD + 1.0);
    let at_end = engine().next_aspect(
        CelestialBody::Sun,
        CelestialBody::Moon,
        Angle::from_degrees(0.0),
        APPARENT,
        tdb(WINDOW_END_JD),
    );
    assert_eq!(out_of_window_jd(at_end), WINDOW_END_JD + STEP);
    let before_start = engine().previous_aspect(
        CelestialBody::Sun,
        CelestialBody::Moon,
        Angle::from_degrees(0.0),
        MEAN,
        tdb(WINDOW_START_JD + 0.05),
    );
    assert_eq!(out_of_window_jd(before_start), WINDOW_START_JD - STEP);
}
```

If the Sun–Moon separation at `at` happens to be within a few degrees of 0 or 180 (where the pair's separation turns and the angle may be met twice in 0.4 d), the `len() == 1` assertions may see 2; move `at` by ±0.05 d rather than loosening the assertion, and say why in a comment.

- [ ] **Step 2: Run them to see them fail**

Run: `cargo nextest run -p pleiades-events aspects::window_edge`
Expected: FAIL (`None`, empty ranges, `Ok(None)` instead of `OutOfWindow`).

- [ ] **Step 3: Implement**

- `Search`: replace `earliest`/`latest` (and their doc) with nothing; the finders use the window directly. Define in `aspects.rs`: `const WINDOW: Window = (WINDOW_START_JD, WINDOW_END_JD);`.
- `aspects_in_range`: `level_crossings_in_range(…, start_jd, end_jd, search.step, WINDOW)`.
- `next_aspect`: `first_level_crossing_after(…, after_jd, WINDOW_END_JD, search.step, WINDOW)?`, then `let jd = root.ok_or_else(|| past_window_end(search.step))?; self.aspect_at(…, jd).map(Some)`.
- `previous_aspect`: `last_level_crossing_before(…, WINDOW_START_JD, before_jd, search.step, WINDOW)?`, `ok_or_else(|| before_window_start(search.step))`.
- Remove the Task 3 temporary envelope and its comment.

Docs:
- `aspects_in_range` Accuracy: replace the third limit bullet ("an event within two steps of either end … is not reported, because the scan keeps its samples inside the window.") with "next to either end of the 1900–2100 window, two exact moments within the step beside it, either side of a turning point of the separation, may go unseen: the scan has no sample past the window to see the turning point." Add after the "empty or inverted range" paragraph: "An apparent place of a body other than the Sun is read a light-time earlier, so a range starting within a light-time of the window's first instant is [`EventError::OutOfWindow`]."
- `next_aspect`: replace "For a pair that never reaches the angle (the Sun and Mercury at 60 degrees) the search runs to the end of the 1900–2100 window before returning `None`." with "When the window ends before the next event, the result is [`EventError::OutOfWindow`] naming the instant one step past the window's end. That includes a pair that never reaches the angle (the Sun and Mercury at 60 degrees), which is searched to the end of the window first: the engine does not know which pairs can reach which angles. This search never returns `Ok(None)`." Same mirror edit in `previous_aspect`.

- [ ] **Step 4: Fix the tests that pinned the old answer**

`crates/pleiades-events/tests/aspects.rs`:
- `previous_aspect_returns_nothing_for_a_pair_that_never_reaches_the_angle`: rename to `previous_aspect_for_a_pair_that_never_reaches_the_angle_is_out_of_window`; the `previous` helper probably unwraps to `Option`, so call `engine.previous_aspect` directly and assert `Err(EventError::OutOfWindow { julian_day })` with `julian_day == WINDOW_START_JD - 1.0`.
- `previous_aspect_guards_match_next_aspect`: the last assertion (Sun–Moon at `WINDOW_START_JD`) becomes `matches!(…, Err(EventError::OutOfWindow { .. }))`, comment "The window's first instant has nothing before it: the search is cut short."
- `grep -n "never reaches\|None)" crates/pleiades-events/tests/aspects.rs` for a `next_aspect` twin and convert it the same way (`WINDOW_END_JD + 1.0`).

Run: `cargo nextest run -p pleiades-events && cargo test -p pleiades-events --doc`
Expected: PASS. Other failures pinning `None` at the ends: convert; anything else is a bug.

- [ ] **Step 5: Run the full aspects gate**

Run: `mise run gate-aspects` (about 15 minutes; foreground, `timeout` 1200000).
Expected: PASS with the same counts as `main` (10359 aspects, 11 pairs). The corpus spans sit five days inside the window, so no event should move; if counts change, stop and investigate.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add crates/pleiades-events
git commit -m "fix(events): search aspects up to the window's ends; report a search the window cuts short (#208)"
```

---

### Task 5: Occultation searches reach the window's ends

**Files:**
- Modify: `crates/pleiades-events/src/occult.rs` (`next_occultation` ~1016-1045, `previous_occultation` ~1069-1098, `next_global_occultation` ~1115-1125 and its loop exits, docs; tests in `when_loc_tests` ~1362 and `when_glob_tests` ~1469)

**Interfaces:**
- Consumes: `past_window_end`, `before_window_start`.

- [ ] **Step 1: Write the failing tests**

In `mod when_loc_tests` (and `when_glob_tests` for the global one) add, reusing that module's existing engine/observer/atmosphere helpers (read its first test for their names):

```rust
#[test]
fn a_local_search_the_window_cuts_short_is_out_of_window() {
    // No conjunction of the Moon and Mars can lie in the last 0.1 d and also
    // give a visible occultation maximum after it; whatever the last one is,
    // the search reaches the window's end.
    let next = engine().next_occultation(
        OccultTarget::Body(CelestialBody::Mars),
        observer(),
        Atmosphere::default(),
        tdb(WINDOW_END_JD - 0.1),
    );
    assert!(
        matches!(next, Err(EventError::OutOfWindow { julian_day }) if julian_day == WINDOW_END_JD + OCC_CONJUNCTION_STEP_DAYS),
        "{next:?}"
    );
    let previous = engine().previous_occultation(
        OccultTarget::Body(CelestialBody::Mars),
        observer(),
        Atmosphere::default(),
        tdb(WINDOW_START_JD + 0.1),
    );
    assert!(matches!(previous, Err(EventError::OutOfWindow { .. })), "{previous:?}");
}

#[test]
fn a_global_search_the_window_cuts_short_is_out_of_window() {
    let next = engine().next_global_occultation(
        OccultTarget::Body(CelestialBody::Mars),
        tdb(WINDOW_END_JD - 0.1),
    );
    assert!(
        matches!(next, Err(EventError::OutOfWindow { julian_day }) if julian_day == WINDOW_END_JD + OCC_CONJUNCTION_STEP_DAYS),
        "{next:?}"
    );
}
```

Before relying on the "no conjunction in the last 0.1 d" premise, check it: `longitude_at(Moon) − longitude_at(Mars)` (apparent) at `WINDOW_END_JD - 0.4` and `WINDOW_END_JD` have the same sign after `wrap180`. If not, use `WINDOW_END_JD - 0.1` only after confirming, or pick Venus/Jupiter, and record the check in the test's comment. The previous-search case at the start is `OutOfWindow` either way (the apparent Moon cannot be read at the first instant, and nothing before it exists). If the existing helpers are named differently, use them; if the module has none, build `EventEngine::new(packaged_backend())` and an `ObserverLocation::new(Latitude::from_degrees(0.0), Longitude::from_degrees(0.0), None)`.

Also keep (or add, if missing) a test that a never-occultable star still returns `Ok(None)` from `next_occultation` (`grep -n "never_occultable\|Ok(None)" crates/pleiades-events/src/occult.rs` for an existing one).

- [ ] **Step 2: Run them to see them fail**

Run: `cargo nextest run -p pleiades-events occult`
Expected: the new tests FAIL with `Ok(None)`.

- [ ] **Step 3: Implement**

In the three searches:
- `scan_start` lower clamp `.max(WINDOW_START_JD + OCC_CONJUNCTION_STEP_DAYS)` → `.max(WINDOW_START_JD)`; `scan_end = WINDOW_END_JD - OCC_CONJUNCTION_STEP_DAYS` → `WINDOW_END_JD`; in `previous_occultation`, `.min(WINDOW_END_JD - …)` → `.min(WINDOW_END_JD)` and `scan_start = WINDOW_START_JD`.
- `let Some(conj_jd) = conj else { return Ok(None) };` → `else { return Err(past_window_end(OCC_CONJUNCTION_STEP_DAYS)) }` in the forward searches, `before_window_start(OCC_CONJUNCTION_STEP_DAYS)` in `previous_occultation`.
- The loop exits `if scan_start >= scan_end { return Ok(None); }` / `if scan_end <= scan_start { return Ok(None); }` → the same errors.
- `target_never_occultable` early returns stay `Ok(None)`.

Docs: `next_occultation` "`None` if none occurs before the window end (or ever, for an un-occultable star)." → "`None` for a target the Moon can never occult (a star too far from the ecliptic). When the window ends before the next visible occultation, the result is [`EventError::OutOfWindow`] naming the instant one conjunction step past the window's end." Mirror in `previous_occultation` and `next_global_occultation`.

- [ ] **Step 4: Run the tests**

Run: `cargo nextest run -p pleiades-events && cargo test -p pleiades-events --doc`
Expected: PASS. Convert any other test pinning `None` at the ends.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add crates/pleiades-events
git commit -m "fix(events): search occultations up to the window's ends; report a search the window cuts short (#208)"
```

---

### Task 6: The CLI notes a search the window cut short

**Files:**
- Modify: `crates/pleiades-cli/src/commands/events.rs` (`render`, `render_stations`, `render_aspects`)
- Modify: `crates/pleiades-cli/src/cli/tests/events.rs`
- Modify: `docs/cli.md` (the `stations`/`aspects` bullets ~line 96)

**Interfaces:**
- Consumes: `EventError::OutOfWindow` from `next_station`, `previous_station`, `next_aspect`, `previous_aspect`.

- [ ] **Step 1: Write the failing CLI tests**

Append to `crates/pleiades-cli/src/cli/tests/events.rs` (reuse its `run` helper):

```rust
#[test]
fn a_next_search_the_window_cuts_short_is_a_note_not_an_error() {
    let rendered = run(&[
        "aspects", "--pair", "Sun,Mercury", "--pair", "Sun,Moon", "--angle", "60", "--next",
        "--at", "2488039.5",
    ]);
    // Sun–Moon 60° happens within days; Sun–Mercury never does.
    assert!(rendered.contains("Sun–Moon 60°"), "{rendered}");
    assert!(
        rendered.contains("Sun–Mercury 60°: none before the window's end (2100-01-01)"),
        "{rendered}"
    );
}

#[test]
fn a_previous_search_the_window_cuts_short_is_a_note() {
    let rendered = run(&[
        "stations", "--body", "Mercury", "--frame", "mean", "--previous", "--at", "2415021.5",
    ]);
    assert!(
        rendered.contains("Mercury: none after the window's start (1900-01-01)"),
        "{rendered}"
    );
}
```

Check the body label the CLI prints (`station.body` / `event.first` use `Display`) so the note matches the event lines' spelling (`Sun`, `Mercury`).

- [ ] **Step 2: Run them to see them fail**

Run: `cargo nextest run -p pleiades-cli events`
Expected: FAIL (the command errors with the `OutOfWindow` message).

- [ ] **Step 3: Implement**

In `events.rs`:

```rust
/// The note for a `--next`/`--previous` search the window cut short.
fn cut_short_note(label: &str, search: &Search) -> String {
    match search {
        Search::Previous(_) => format!("{label}: none after the window's start (1900-01-01)"),
        _ => format!("{label}: none before the window's end (2100-01-01)"),
    }
}

/// Adds a `--next`/`--previous` result to `found`, or its note to `notes`
/// when the window ended first; any other error fails the command.
fn found_or_note<T>(
    result: Result<Option<T>, EventError>,
    found: &mut Vec<T>,
    notes: &mut Vec<String>,
    note: impl FnOnce() -> String,
) -> Result<(), String> {
    match result {
        Ok(event) => {
            found.extend(event);
            Ok(())
        }
        Err(EventError::OutOfWindow { .. }) => {
            notes.push(note());
            Ok(())
        }
        Err(error) => Err(event_error(error)),
    }
}
```

In `render_stations` / `render_aspects`, the `Search::Next` and `Search::Previous` arms call `found_or_note(engine.next_station(…), &mut found, &mut notes, || cut_short_note(&body.to_string(), &search))?` (aspects: label `format!("{first}–{second} {}°", angle.degrees())`, matching `aspect_line`'s spelling). The `Range` arm is unchanged (an `OutOfWindow` from a range is still an error). Clone `body` before it moves into the engine call.

`render` gains `notes: &[String]`: after the event lines (or the `none` line when there are no events and no notes), append each note on its own line. With events: header, events, notes. With no events and some notes: header, notes. With neither: header, `none`.

- [ ] **Step 4: Run the CLI tests**

Run: `cargo nextest run -p pleiades-cli`
Expected: PASS.

- [ ] **Step 5: Update `docs/cli.md`**

After the `--next`/`--previous` bullet add: "- A `--next` or `--previous` search that reaches the end of the 1900–2100 window first prints a note for that body, or pair and angle, (`Mercury: none before the window's end (2100-01-01)`) and the others still print. A pair that never reaches an angle (the Sun and Mercury at 60°) gets that note."

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add crates/pleiades-cli docs/cli.md
git commit -m "feat(cli): note a stations or aspects search the window cuts short (#208)"
```

---

### Task 7: Compatibility profile, README, full validation, PR

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/mod.rs` (`CURRENT_COMPATIBILITY_PROFILE_ID` 0.7.30 → 0.7.31, `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM`, new release-note entry after the #203 entry ~line 123)
- Modify: `crates/pleiades-cli/src/cli/tests/summary_commands.rs`, `crates/pleiades-validate/src/tests/render_request.rs` (profile id pins)
- Modify: `crates/pleiades-events/README.md` (~line 155-157 and any other margin text)

- [ ] **Step 1: Add the release note and bump the profile**

Add after the #203 entry (keep that entry as history):

```rust
            "Crossing, station, aspect and occultation searches at the window's ends (issue #208): longitude_crossings_in_range, next/previous_longitude_crossing, stations_in_range, next/previous_station, aspects_in_range, next/previous_aspect, next/previous_occultation and next_global_occultation now search up to the first and last instant of the 1900-2100 window. They used to stop one scan step short of each end (two for aspects), so an event there was not reported. Behaviour change: a next or previous search that reaches the window's end, or start, without an event returns EventError::OutOfWindow naming the instant one scan step past it, where it returned Ok(None); that includes an aspect pair that never reaches its angle. Ok(None) now means only that the engine knows there is no event: a body that never stations, a star the Moon never occults. A range or search that needs an apparent place within a light-time of the window's first instant is OutOfWindow, as position_at is. The pleiades-cli stations and aspects commands print a note for a search the window cuts short. Compatibility profile bumped to 0.7.31; API stability profile unchanged.",
```

Bump `CURRENT_COMPATIBILITY_PROFILE_ID` to `"pleiades-compatibility-profile/0.7.31"` and the two test pins (`summary_commands.rs`, `render_request.rs`) from `0.7.30` to `0.7.31`.

Run: `cargo nextest run -p pleiades-core compatibility`
Expected: the checksum test FAILS and prints the new checksum. Copy that value into `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM`; re-run, expected PASS. (Check `git log -p -1 5e72ffbab -- crates/pleiades-core/src/compatibility/mod.rs` if the test's name or output differs.)

- [ ] **Step 2: Update the README**

`crates/pleiades-events/README.md`: replace "an event within two steps …" (around line 156) and any other "within one step / two steps of either end" text with the new answers: the searches reach the window's ends; a next/previous search the window cuts short is `OutOfWindow`. `grep -n -i "window" crates/pleiades-events/README.md` to find every place.

- [ ] **Step 3: Commit**

```bash
cargo fmt --all
git add crates/pleiades-core crates/pleiades-cli crates/pleiades-validate crates/pleiades-events/README.md
git commit -m "docs(events,core): document the window-edge answers of the crossing, station, aspect and occultation searches (#208)"
```

- [ ] **Step 4: Full validation (foreground; no edits or commits while running)**

Run, each in the foreground with a 600000 ms timeout:
1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `mise run docs`
4. `mise run ci`
5. `mise run gate-stations`
6. `cargo test -p pleiades-validate -- --include-ignored crossings` (the `validate-crossings` gate drives `next_longitude_crossing`)
7. `mise run test-full`

Expected: all PASS. A gate whose counts change means an event moved into or out of a corpus span: stop and investigate; never change a corpus or tolerance here.

- [ ] **Step 5: Push and open the PR**

```bash
git -c credential.helper= -c credential.helper='!gh auth git-credential' push -u origin worktree-event-window-edges
gh pr create --title "fix(events): search crossings, stations, aspects and occultations up to the window's ends; report a search the window cuts short (#208)" --body "Closes #208. Spec: docs/superpowers/specs/2026-10-06-event-window-edges-design.md. Plan: docs/superpowers/plans/2026-10-06-event-window-edges.md. …"
```

The body lists the behaviour change (Ok(None) → OutOfWindow), the light-time start rule, the CLI notes, the profile bump, and the validation run with results.
