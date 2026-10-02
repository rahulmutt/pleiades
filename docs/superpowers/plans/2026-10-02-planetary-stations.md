# Planetary Station Finder Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `EventEngine::stations_in_range` and `EventEngine::next_station` find the instants a body's longitude speed changes sign, gated against a Swiss Ephemeris speed-zero corpus (issue #85).

**Architecture:** A station is a sign change of the longitude speed `EventEngine::position_at` already reports. A new `stations.rs` module in `pleiades-events` feeds that speed to the existing step-bracket and settled-bisection scanners in `root.rs`. A new reference tool bisects Swiss Ephemeris's own speed to zero and commits a CSV; a new `validate-stations` gate compares the engine's stations to it, exactly for planets and by a separated-station rule for the true node.

**Tech Stack:** Rust (workspace toolchain from `mise.toml`), `cargo nextest`, `libswisseph-sys` in an out-of-workspace tool built under `devenv shell`.

**Spec:** `docs/superpowers/specs/2026-10-02-planetary-stations-design.md` (read it, including the amendment at the end, before starting any task).

## Global Constraints

- Branch: `feat/planetary-stations`. Every commit message ends with `(#85)`.
- Scope is exactly two finders plus the gate. No `previous_station`, no user-facing CLI stations command, no asteroids in the gate.
- A station is a sign change of `position_at(body, reference, t).motion.longitude_deg_per_day`. Do not introduce a second speed (no differenced longitude).
- `root.rs` is reused unchanged. `EventEngine::longitude_at` must return bit-identical values; `validate-crossings` and the `crossings-golden` manifest must not change. A diff in either is a defect, never something to regenerate.
- A missing or non-finite longitude speed returns `EventError::MissingSpeed`. It is never read as "no station".
- Bracketing steps: 0.25 day for `Moon`, `MeanNode`, `TrueNode`, `MeanApogee`, `TrueApogee`, `MeanPerigee`, `TruePerigee`; 1.0 day for `Sun`, `Mercury`, `Venus`; 2.0 days for everything else.
- No `unwrap`/`expect`/panic in library paths. Tests and the reference tool may use them.
- `CHANGELOG.md` files are written by release-plz. Do not edit them and do not bump versions.
- Run `cargo fmt --all` before every commit. Before each commit run at minimum `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and the task's own tests.
- All verification commands run in the foreground. Do not edit sources or commit while a test run is in progress.

## Review Focus

1. **A body the backend reports no speed for** — the caller gets `MissingSpeed`, not an empty list. Pinned in Task 2 (`a_backend_without_speed_is_an_error_not_an_empty_list`).
2. **A range touching a window edge (JD 2415020.5 or 2488069.5), or `after` at the window end** — stations are returned or `None`, never `OutOfWindow` or a backend range error. Pinned in Task 2 (`ranges_touching_the_window_edges_work`).
3. **An empty or inverted range (`start == end`, `start > end`)** — an empty list, not a panic or an error. Pinned in Task 2 (`empty_and_inverted_ranges_give_no_stations`).
4. **A body the backend does not serve at all (Ceres on the packaged backend)** — a typed error, not a panic and not an empty list. Pinned in Task 2 (`a_body_the_backend_does_not_serve_is_an_error`).
5. **A non-finite speed from a backend** — `MissingSpeed`, because `NaN <= 0.0` is false and would otherwise read as "direct". Pinned in Task 2 (`non_finite_speed_is_missing`).

---

### Task 1: `EventError::MissingSpeed` and a non-exhaustive `EventError`

**Files:**
- Modify: `crates/pleiades-events/src/error.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: `EventError::MissingSpeed { body_label: &'static str, julian_day: f64 }`; `EventError` is `#[non_exhaustive]`.

- [ ] **Step 1: Write the failing test**

Append to the `tests` module at the bottom of `crates/pleiades-events/src/error.rs`:

```rust
    #[test]
    fn missing_speed_names_the_body_and_the_julian_day() {
        let err = EventError::MissingSpeed {
            body_label: "Mercury",
            julian_day: 2_451_545.0,
        };
        let text = err.to_string();
        assert!(text.contains("Mercury"), "{text}");
        assert!(text.contains("2451545"), "{text}");
        assert!(text.contains("longitude speed"), "{text}");
    }
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo nextest run -p pleiades-events missing_speed_names`
Expected: compile error, `no variant named MissingSpeed`.

- [ ] **Step 3: Add the variant and the attribute**

In `crates/pleiades-events/src/error.rs`, add `#[non_exhaustive]` to the enum and the new variant after `UnsupportedTimeScale`:

```rust
/// Errors returned by the event engine; all variants fail closed.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum EventError {
```

```rust
    /// The backend reported no finite longitude speed for a body, so its
    /// stations cannot be found.
    MissingSpeed {
        /// Human-readable label of the body (e.g. `"Mercury"`).
        body_label: &'static str,
        /// The Julian Day at which the speed was requested.
        julian_day: f64,
    },
```

And in `Display`, after the `UnsupportedTimeScale` arm:

```rust
            EventError::MissingSpeed {
                body_label,
                julian_day,
            } => write!(
                f,
                "backend reported no finite longitude speed for {body_label} at JD {julian_day}"
            ),
```

- [ ] **Step 4: Run the test and build the workspace**

Run: `cargo nextest run -p pleiades-events missing_speed_names`
Expected: PASS.

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: clean. If any crate outside `pleiades-events` now fails on a non-exhaustive match of `EventError`, add a wildcard arm there that preserves the existing fallback behaviour of that match, and include the file in the commit.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add crates
git commit -m "feat(events)!: EventError::MissingSpeed, and EventError is non-exhaustive (#85)"
```

---

### Task 2: The station finders

**Files:**
- Modify: `crates/pleiades-events/src/position.rs` (add `place_and_motion`, use it in `position_at`)
- Create: `crates/pleiades-events/src/stations.rs`
- Create: `crates/pleiades-events/src/stations/tests.rs`
- Create: `crates/pleiades-events/tests/stations.rs`
- Modify: `crates/pleiades-events/src/lib.rs` (module, re-exports, crate doc)

**Interfaces:**
- Consumes: `EventError::MissingSpeed` (Task 1); `root::crossings_in_range`, `root::first_crossing_after`; `reference::check_supported`; `crossings::body_label`.
- Produces:
  - `pub struct Station { pub body: CelestialBody, pub instant: Instant, pub longitude: Longitude, pub kind: StationKind, pub frame: CrossingFrame, pub zodiac: ZodiacMode }`
  - `pub enum StationKind { TurnsRetrograde, TurnsDirect }`
  - `EventEngine::stations_in_range(&self, body: CelestialBody, reference: impl Into<CrossingReference>, start: Instant, end: Instant) -> Result<Vec<Station>, EventError>`
  - `EventEngine::next_station(&self, body: CelestialBody, reference: impl Into<CrossingReference>, after: Instant) -> Result<Option<Station>, EventError>`
  - All four re-exported from `pleiades_events`.

- [ ] **Step 1: Write the white-box unit tests**

Create `crates/pleiades-events/src/stations/tests.rs`:

```rust
//! White-box checks of the station finder's pure pieces.

use super::{checked_speed, kind_of, step_days, StationKind};
use crate::error::EventError;
use pleiades_types::CelestialBody;

#[test]
fn step_is_scaled_to_the_body() {
    for body in [
        CelestialBody::Moon,
        CelestialBody::MeanNode,
        CelestialBody::TrueNode,
        CelestialBody::MeanApogee,
        CelestialBody::TrueApogee,
        CelestialBody::MeanPerigee,
        CelestialBody::TruePerigee,
    ] {
        assert_eq!(step_days(&body), 0.25, "{body:?}");
    }
    for body in [
        CelestialBody::Sun,
        CelestialBody::Mercury,
        CelestialBody::Venus,
    ] {
        assert_eq!(step_days(&body), 1.0, "{body:?}");
    }
    for body in [
        CelestialBody::Mars,
        CelestialBody::Pluto,
        CelestialBody::Ceres,
    ] {
        assert_eq!(step_days(&body), 2.0, "{body:?}");
    }
}

// `root::bisect` counts zero as the negative side, so the settled instant of
// a turn to retrograde can carry a speed of exactly zero.
#[test]
fn kind_follows_the_sign_of_the_settled_speed() {
    assert_eq!(kind_of(1e-12), StationKind::TurnsDirect);
    assert_eq!(kind_of(0.0), StationKind::TurnsRetrograde);
    assert_eq!(kind_of(-1e-12), StationKind::TurnsRetrograde);
}

#[test]
fn non_finite_speed_is_missing() {
    let body = CelestialBody::Mars;
    assert_eq!(checked_speed(Some(0.25), &body, 2_451_545.0), Ok(0.25));
    for speed in [None, Some(f64::NAN), Some(f64::INFINITY), Some(f64::NEG_INFINITY)] {
        assert_eq!(
            checked_speed(speed, &body, 2_451_545.0),
            Err(EventError::MissingSpeed {
                body_label: "Mars",
                julian_day: 2_451_545.0,
            }),
            "{speed:?}"
        );
    }
}
```

- [ ] **Step 2: Write the public-API tests**

Create `crates/pleiades-events/tests/stations.rs`:

```rust
//! `EventEngine::stations_in_range` and `next_station`: settled instants,
//! chaining, bodies that never station, guards and window edges.

use pleiades_backend::test_backend::LinearSunMoon;
use pleiades_data::packaged_backend;
use pleiades_events::{
    CrossingFrame, CrossingReference, EventEngine, EventError, Station, StationKind,
    WINDOW_END_JD, WINDOW_START_JD,
};
use pleiades_types::{Ayanamsa, CelestialBody, Instant, JulianDay, TimeScale, ZodiacMode};

const GEO: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const MEAN: CrossingFrame = CrossingFrame::GeocentricMeanOfDate;
const HELIO: CrossingFrame = CrossingFrame::Heliocentric;
const J2000: f64 = 2_451_545.0;
/// Two seconds, in days: four times the bisection tolerance.
const TWO_SECONDS: f64 = 2.0 / 86_400.0;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn stations(
    body: CelestialBody,
    reference: impl Into<CrossingReference>,
    start_jd: f64,
    end_jd: f64,
) -> Vec<Station> {
    EventEngine::new(packaged_backend())
        .stations_in_range(body, reference, tdb(start_jd), tdb(end_jd))
        .expect("stations")
}

fn speed(body: CelestialBody, reference: impl Into<CrossingReference>, jd: f64) -> f64 {
    EventEngine::new(packaged_backend())
        .position_at(body, reference, tdb(jd))
        .expect("position")
        .motion
        .longitude_deg_per_day
        .expect("speed")
}

fn jd(station: &Station) -> f64 {
    station.instant.julian_day.days()
}

fn assert_alternating(found: &[Station]) {
    for pair in found.windows(2) {
        assert_ne!(pair[0].kind, pair[1].kind, "{pair:?}");
        assert!(jd(&pair[0]) < jd(&pair[1]), "{pair:?}");
    }
}

// Mercury was retrograde three times in 2000: 21 Feb – 14 Mar, 23 Jun – 17 Jul
// and 18 Oct – 8 Nov.
#[test]
fn mercury_stations_of_2000() {
    let found = stations(CelestialBody::Mercury, GEO, J2000, J2000 + 366.0);
    assert_eq!(found.len(), 6, "{found:?}");
    assert_eq!(found[0].kind, StationKind::TurnsRetrograde);
    assert_alternating(&found);
    // 21 February 2000 is about 51 days after J2000.
    let first = jd(&found[0]) - J2000;
    assert!((49.0..54.0).contains(&first), "first station at +{first} d");
    for station in &found {
        assert_eq!(station.body, CelestialBody::Mercury);
        assert_eq!(station.frame, GEO);
        assert_eq!(station.zodiac, ZodiacMode::Tropical);
        assert_eq!(station.instant.scale, TimeScale::Tdb);
    }
}

// The returned instant is the later end of the final bisection bracket: the
// engine's own speed already has the post-station sign there, and still had
// the pre-station sign two seconds earlier.
#[test]
fn returned_instants_are_settled() {
    let engine = EventEngine::new(packaged_backend());
    let mut found = stations(CelestialBody::Mercury, GEO, J2000, J2000 + 366.0);
    found.extend(stations(CelestialBody::Mars, GEO, J2000, J2000 + 1100.0));
    assert!(found.len() >= 8, "{}", found.len());
    for station in &found {
        let at = speed(station.body.clone(), GEO, jd(station));
        let before = speed(station.body.clone(), GEO, jd(station) - TWO_SECONDS);
        match station.kind {
            StationKind::TurnsDirect => {
                assert!(at > 0.0 && before <= 0.0, "{station:?}: {before} -> {at}")
            }
            StationKind::TurnsRetrograde => {
                assert!(at <= 0.0 && before > 0.0, "{station:?}: {before} -> {at}")
            }
            other => panic!("unexpected kind {other:?}"),
        }
        let longitude = engine
            .longitude_at(station.body.clone(), GEO, station.instant)
            .expect("longitude");
        assert_eq!(station.longitude, longitude, "{station:?}");
    }
}

#[test]
fn next_station_is_the_first_in_range_and_chains() {
    let engine = EventEngine::new(packaged_backend());
    let in_range = stations(CelestialBody::Mercury, GEO, J2000, J2000 + 366.0);
    let mut after = tdb(J2000);
    for expected in &in_range {
        let next = engine
            .next_station(CelestialBody::Mercury, GEO, after)
            .expect("next_station")
            .expect("Mercury stations six times a year");
        assert_eq!(&next, expected);
        // Handing a returned instant back finds the following station.
        after = next.instant;
    }
}

#[test]
fn bodies_that_never_station_return_nothing() {
    for body in [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::MeanNode,
    ] {
        let found = stations(body.clone(), GEO, J2000, J2000 + 730.0);
        assert!(found.is_empty(), "{body:?}: {found:?}");
    }
    let engine = EventEngine::new(packaged_backend());
    let next = engine
        .next_station(CelestialBody::Sun, GEO, tdb(WINDOW_END_JD - 400.0))
        .expect("next_station");
    assert_eq!(next, None);
}

#[test]
fn nothing_stations_heliocentrically() {
    for body in [
        CelestialBody::Mercury,
        CelestialBody::Mars,
        CelestialBody::Pluto,
    ] {
        let found = stations(body.clone(), HELIO, J2000, J2000 + 1100.0);
        assert!(found.is_empty(), "{body:?}: {found:?}");
    }
}

// The true node is retrograde on average and turns briefly direct about
// every two weeks.
#[test]
fn the_true_node_stations_often() {
    let found = stations(CelestialBody::TrueNode, GEO, J2000, J2000 + 365.0);
    assert!(found.len() >= 10, "{}", found.len());
    assert_alternating(&found);
}

// A sidereal longitude speed is the tropical one less the ayanamsa's rate
// (about 3.8e-5 deg/day), so the speed reaches zero later on the way up and
// earlier on the way down. The mean-of-date frame is used because the
// apparent frame's sidereal speed also drops the nutation rate, whose sign
// varies.
#[test]
fn a_sidereal_station_is_shifted_by_the_ayanamsa_rate() {
    let lahiri = CrossingReference::sidereal(MEAN, Ayanamsa::Lahiri);
    let tropical = stations(CelestialBody::Saturn, MEAN, J2000, J2000 + 730.0);
    let sidereal = stations(CelestialBody::Saturn, lahiri, J2000, J2000 + 730.0);
    assert!(tropical.len() >= 3, "{tropical:?}");
    assert_eq!(tropical.len(), sidereal.len());
    for (t, s) in tropical.iter().zip(&sidereal) {
        assert_eq!(t.kind, s.kind);
        let shift = jd(s) - jd(t);
        let expected_sign = match t.kind {
            StationKind::TurnsDirect => 1.0,
            _ => -1.0,
        };
        assert!(
            (0.002..1.0).contains(&(shift * expected_sign)),
            "{:?}: sidereal - tropical = {shift} d",
            t.kind
        );
    }
}

#[test]
fn a_backend_without_speed_is_an_error_not_an_empty_list() {
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let err = engine
        .stations_in_range(CelestialBody::Sun, GEO, tdb(J2000), tdb(J2000 + 30.0))
        .unwrap_err();
    assert!(
        matches!(err, EventError::MissingSpeed { body_label: "Sun", .. }),
        "{err:?}"
    );
    let err = engine
        .next_station(CelestialBody::Sun, GEO, tdb(J2000))
        .unwrap_err();
    assert!(matches!(err, EventError::MissingSpeed { .. }), "{err:?}");
}

#[test]
fn guards_match_position_at() {
    let engine = EventEngine::new(packaged_backend());
    let err = engine
        .stations_in_range(CelestialBody::Mars, GEO, tdb(2_000_000.0), tdb(J2000))
        .unwrap_err();
    assert!(matches!(err, EventError::OutOfWindow { .. }), "{err:?}");
    let err = engine
        .stations_in_range(CelestialBody::Mars, GEO, tdb(J2000), tdb(2_500_000.0))
        .unwrap_err();
    assert!(matches!(err, EventError::OutOfWindow { .. }), "{err:?}");
    let err = engine
        .next_station(CelestialBody::Mars, GEO, tdb(2_000_000.0))
        .unwrap_err();
    assert!(matches!(err, EventError::OutOfWindow { .. }), "{err:?}");
    let err = engine
        .next_station(CelestialBody::Sun, HELIO, tdb(J2000))
        .unwrap_err();
    assert!(matches!(err, EventError::UnsupportedFrame { .. }), "{err:?}");
    let err = engine
        .stations_in_range(
            CelestialBody::Mars,
            CrossingReference::sidereal(HELIO, Ayanamsa::Lahiri),
            tdb(J2000),
            tdb(J2000 + 30.0),
        )
        .unwrap_err();
    assert!(matches!(err, EventError::UnsupportedFrame { .. }), "{err:?}");
}

#[test]
fn ranges_touching_the_window_edges_work() {
    let engine = EventEngine::new(packaged_backend());
    let early = stations(
        CelestialBody::Mercury,
        GEO,
        WINDOW_START_JD,
        WINDOW_START_JD + 400.0,
    );
    assert!(early.len() >= 4, "{early:?}");
    let late = stations(
        CelestialBody::Mercury,
        GEO,
        WINDOW_END_JD - 400.0,
        WINDOW_END_JD,
    );
    assert!(late.len() >= 4, "{late:?}");
    let next = engine
        .next_station(CelestialBody::Mercury, GEO, tdb(WINDOW_END_JD))
        .expect("next_station at the window end");
    assert_eq!(next, None);
}

#[test]
fn empty_and_inverted_ranges_give_no_stations() {
    assert!(stations(CelestialBody::Mercury, GEO, J2000, J2000).is_empty());
    assert!(stations(CelestialBody::Mercury, GEO, J2000 + 366.0, J2000).is_empty());
}

#[test]
fn a_body_the_backend_does_not_serve_is_an_error() {
    let engine = EventEngine::new(packaged_backend());
    let result =
        engine.stations_in_range(CelestialBody::Ceres, GEO, tdb(J2000), tdb(J2000 + 30.0));
    assert!(result.is_err(), "{result:?}");
}

#[cfg(feature = "serde")]
#[test]
fn a_station_round_trips_through_serde() {
    let found = stations(CelestialBody::Mercury, GEO, J2000, J2000 + 100.0);
    let json = serde_json::to_string(&found[0]).expect("serialize");
    let back: Station = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, found[0]);
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo nextest run -p pleiades-events stations`
Expected: compile errors (`stations_in_range`, `Station`, `StationKind` not found).

- [ ] **Step 4: Expose the place and speed without the guards**

In `crates/pleiades-events/src/position.rs`, add this function directly above `impl<B: EphemerisBackend> EventEngine<B>`:

```rust
/// The place and speed [`EventEngine::position_at`] reports, without its
/// window and frame guards. The station finder root-finds on the speed.
pub(crate) fn place_and_motion<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    reference: &CrossingReference,
    julian_day: f64,
) -> Result<(EclipticCoordinates, Motion), EventError> {
    let centre = sample(backend, body, reference, julian_day)?;
    let motion = motion(backend, body, reference, julian_day, &centre);
    Ok((centre.corrected, motion))
}
```

and replace the last lines of `position_at` (from `let centre = sample(...)` to the end of the function) with:

```rust
        let (ecliptic, motion) = place_and_motion(&self.backend, &body, &reference, jd)?;
        Ok(EclipticPosition {
            body,
            frame: reference.frame,
            zodiac: reference.zodiac,
            instant,
            ecliptic,
            motion,
        })
```

- [ ] **Step 5: Write the station module**

Create `crates/pleiades-events/src/stations.rs`:

```rust
//! Station finder: the instants a body's longitude speed changes sign.
//!
//! A station is a sign change of the longitude speed
//! [`EventEngine::position_at`] reports, so it is exactly where the engine's
//! own direction flips, in every frame and zodiac.

use crate::crossings::{body_label, CrossingFrame, EventEngine};
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use crate::position::place_and_motion;
use crate::reference::{check_supported, CrossingReference};
use crate::root::{crossings_in_range, first_crossing_after};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale, ZodiacMode};

/// Which way a body turns at a station.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum StationKind {
    /// Direct before the station, retrograde after it.
    TurnsRetrograde,
    /// Retrograde before the station, direct after it.
    TurnsDirect,
}

/// A station: an instant at which a body's longitude speed changes sign.
//
// `CelestialBody` is not `Copy`, so neither is this.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Station {
    /// The body that stations.
    pub body: CelestialBody,
    /// Instant of the station (TDB). It trails the sign change by less than
    /// the 0.5 s bisection tolerance and never precedes it.
    pub instant: Instant,
    /// Longitude at `instant`, in `frame` and `zodiac`.
    pub longitude: Longitude,
    /// Which way the body turns.
    pub kind: StationKind,
    /// The frame the speed was measured in.
    pub frame: CrossingFrame,
    /// The zodiac the speed and `longitude` are read in.
    pub zodiac: ZodiacMode,
}

/// Step used to bracket stations: well under the shortest interval between
/// two stations of the body. Mercury's shortest retrograde is about 19 days;
/// the lunar points' speeds oscillate within a month.
fn step_days(body: &CelestialBody) -> f64 {
    match body {
        CelestialBody::Moon
        | CelestialBody::MeanNode
        | CelestialBody::TrueNode
        | CelestialBody::MeanApogee
        | CelestialBody::TrueApogee
        | CelestialBody::MeanPerigee
        | CelestialBody::TruePerigee => 0.25,
        CelestialBody::Sun | CelestialBody::Mercury | CelestialBody::Venus => 1.0,
        _ => 2.0,
    }
}

/// The settled instant carries the post-station sign; `root::bisect` counts
/// zero as the negative side.
fn kind_of(settled_speed: f64) -> StationKind {
    if settled_speed > 0.0 {
        StationKind::TurnsDirect
    } else {
        StationKind::TurnsRetrograde
    }
}

/// A finite speed, or [`EventError::MissingSpeed`]. A NaN must not reach the
/// sign test, where it would read as "direct".
fn checked_speed(
    speed: Option<f64>,
    body: &CelestialBody,
    julian_day: f64,
) -> Result<f64, EventError> {
    speed
        .filter(|speed| speed.is_finite())
        .ok_or(EventError::MissingSpeed {
            body_label: body_label(body),
            julian_day,
        })
}

fn longitude_speed<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    reference: &CrossingReference,
    julian_day: f64,
) -> Result<f64, EventError> {
    let (_, motion) = place_and_motion(backend, body, reference, julian_day)?;
    checked_speed(motion.longitude_deg_per_day, body, julian_day)
}

impl<B: EphemerisBackend> EventEngine<B> {
    fn station_at(
        &self,
        body: &CelestialBody,
        reference: &CrossingReference,
        julian_day: f64,
    ) -> Result<Station, EventError> {
        let (ecliptic, motion) = place_and_motion(&self.backend, body, reference, julian_day)?;
        let speed = checked_speed(motion.longitude_deg_per_day, body, julian_day)?;
        Ok(Station {
            body: body.clone(),
            instant: Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb),
            longitude: ecliptic.longitude,
            kind: kind_of(speed),
            frame: reference.frame,
            zodiac: reference.zodiac.clone(),
        })
    }

    /// All stations of `body` in `[start, end]` (TDB), ascending.
    ///
    /// A station is a sign change of the longitude speed
    /// [`EventEngine::position_at`] reports in `reference`, so
    /// `position_at` just before and at a returned instant always disagree
    /// in direction. `reference` is a [`CrossingFrame`] (tropical zodiac) or
    /// a [`CrossingReference`] carrying a sidereal zodiac. The zodiac
    /// matters: a sidereal speed is lower by the ayanamsa's rate, which
    /// moves a slow planet's station by minutes to hours.
    ///
    /// A body that never stations in `reference` (the Sun, the Moon, the
    /// mean node, every body in the heliocentric frame) returns an empty
    /// list. An empty or inverted range returns an empty list.
    ///
    /// # Accuracy
    ///
    /// The search steps by 0.25 day for the Moon and the lunar points, 1 day
    /// for the Sun, Mercury and Venus, and 2 days otherwise. Two stations
    /// closer together than the step are not reported. That happens only
    /// for the osculating lunar points: the true node's speed touches zero
    /// about every two weeks, and whether a touch crosses zero for a few
    /// hours depends on the ephemeris.
    ///
    /// The 0.5 s bisection tolerance bounds how well the engine locates the
    /// zero of its own speed, not how well that zero matches another
    /// ephemeris: near a station the speed changes slowly, so a small speed
    /// difference is a large time difference. See the crate README for the
    /// measured agreement with Swiss Ephemeris per body.
    ///
    /// # Errors
    ///
    /// The same as [`EventEngine::position_at`] —
    /// [`EventError::OutOfWindow`], [`EventError::UnsupportedFrame`],
    /// [`EventError::MissingCoordinates`], [`EventError::Backend`] — and
    /// [`EventError::MissingSpeed`] when the backend reports no finite
    /// longitude speed. A missing speed is never read as "no station".
    pub fn stations_in_range(
        &self,
        body: CelestialBody,
        reference: impl Into<CrossingReference>,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<Station>, EventError> {
        let reference = reference.into();
        let start_jd = start.julian_day.days();
        let end_jd = end.julian_day.days();
        self.check_window(start_jd)?;
        self.check_window(end_jd)?;
        check_supported(&body, &reference, start_jd, "stations are")?;
        let step = step_days(&body);
        // Clamp like the crossings: keep the bracketing samples in-window.
        let scan_start = start_jd.max(WINDOW_START_JD + step);
        let scan_end = end_jd.min(WINDOW_END_JD - step);
        let roots = crossings_in_range(
            |jd| longitude_speed(&self.backend, &body, &reference, jd),
            scan_start,
            scan_end,
            step,
        )?;
        roots
            .into_iter()
            .map(|jd| self.station_at(&body, &reference, jd))
            .collect()
    }

    /// The first station of `body` strictly after `after`, or `None`.
    ///
    /// Identical to the first element of
    /// `stations_in_range(body, reference, after, WINDOW_END)` that is
    /// strictly after `after`, but stops at the first station found. A
    /// returned [`Station::instant`] can be handed back as `after`: the
    /// search then returns the following station, not the same one.
    ///
    /// For a body that never stations the search runs to the end of the
    /// 1900–2100 window before returning `None`.
    ///
    /// Accuracy, step and errors are those of
    /// [`EventEngine::stations_in_range`].
    ///
    /// ```
    /// use pleiades_data::packaged_backend;
    /// use pleiades_events::{CrossingFrame, EventEngine, StationKind};
    /// use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
    ///
    /// // Mercury's first station of 2000: it turns retrograde on 21 February.
    /// let engine = EventEngine::new(packaged_backend());
    /// let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    /// let station = engine
    ///     .next_station(CelestialBody::Mercury, CrossingFrame::GeocentricApparentOfDate, after)
    ///     .unwrap()
    ///     .expect("Mercury stations six times a year");
    /// assert_eq!(station.kind, StationKind::TurnsRetrograde);
    /// let days = station.instant.julian_day.days() - 2_451_545.0;
    /// assert!((49.0..54.0).contains(&days), "{days}");
    /// ```
    pub fn next_station(
        &self,
        body: CelestialBody,
        reference: impl Into<CrossingReference>,
        after: Instant,
    ) -> Result<Option<Station>, EventError> {
        let reference = reference.into();
        let after_jd = after.julian_day.days();
        self.check_window(after_jd)?;
        check_supported(&body, &reference, after_jd, "stations are")?;
        let step = step_days(&body);
        // Same clamps as `stations_in_range` over `[after, WINDOW_END]`.
        let scan_start = after_jd.max(WINDOW_START_JD + step);
        let scan_end = WINDOW_END_JD - step;
        let root = first_crossing_after(
            |jd| longitude_speed(&self.backend, &body, &reference, jd),
            scan_start,
            scan_end,
            step,
        )?;
        root.filter(|&jd| jd > after_jd)
            .map(|jd| self.station_at(&body, &reference, jd))
            .transpose()
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 6: Register the module and exports**

In `crates/pleiades-events/src/lib.rs`: add `mod stations;` after `mod state_vector;`, add `pub use stations::{Station, StationKind};` after `pub use rise_trans::{...};`, and insert this paragraph in the crate doc directly before the `//! ## Example` line:

```rust
//! Planetary stations — the instants a body's longitude speed changes sign —
//! are found by [`EventEngine::stations_in_range`] and
//! [`EventEngine::next_station`], in any [`CrossingFrame`] and zodiac.
//!
```

- [ ] **Step 7: Run the tests**

Run: `cargo nextest run -p pleiades-events stations`
Expected: all PASS (`a_station_round_trips_through_serde` is not built without the feature).

Run: `cargo nextest run -p pleiades-events --features serde stations`
Expected: all PASS including `a_station_round_trips_through_serde`.

Run: `cargo test --doc -p pleiades-events next_station`
Expected: PASS.

If `mercury_stations_of_2000` finds a count other than 6 or a first station outside 49–54 days, do not adjust the assertion to the output: Mercury's 2000 retrogrades (21 Feb – 14 Mar, 23 Jun – 17 Jul, 18 Oct – 8 Nov) are published facts, so a mismatch is a finder defect.

- [ ] **Step 8: Confirm nothing else moved**

Run: `cargo nextest run -p pleiades-events`
Expected: PASS (the `position_at` tests cover the `place_and_motion` refactor).

Run: `cargo nextest run -p pleiades-cli crossings`
Expected: PASS; then `git status --short` shows no modified file under any `crossings-golden` path.

- [ ] **Step 9: Commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
git add crates/pleiades-events
git commit -m "feat(events): station finders stations_in_range and next_station (#85)"
```

---

### Task 3: Swiss Ephemeris station reference tool and corpus

**Files:**
- Create: `tools/se-stations-reference/Cargo.toml`
- Create: `tools/se-stations-reference/src/main.rs`
- Create: `tools/se-stations-reference/Cargo.lock` (generated by cargo)
- Create: `tools/se-stations-reference/LICENSE-NOTES.md`
- Create: `crates/pleiades-validate/data/stations-corpus/stations.csv` (generated)
- Modify: `Cargo.toml` (workspace `exclude` list)

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces: a committed CSV whose data rows are `group,body,jd_tt,lon_deg,kind` with `group` in `geo|mean|sid`, `body` in `Mercury|Venus|Mars|Jupiter|Saturn|Uranus|Neptune|Pluto|TrueNode`, `kind` in `R|D` (turns retrograde, turns direct), rows ascending in `jd_tt` within each `(group, body)`. Spans: `geo` planets JD 2415025.5–2488064.5; everything else JD 2447892.5–2462502.5 (1990-01-01 to 2030-01-01).

- [ ] **Step 1: Create the tool crate**

`tools/se-stations-reference/Cargo.toml`:

```toml
[package]
name = "se-stations-reference"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
swisseph = "0.1.1"
libswisseph-sys = "0.1.2"
```

`tools/se-stations-reference/LICENSE-NOTES.md`: copy `tools/se-helio-reference/LICENSE-NOTES.md` and replace its heading with `# License notes — \`se-stations-reference\`` and its third paragraph's purpose sentence with:

```markdown
Its sole purpose is to link Swiss Ephemeris (via `swisseph` / `libswisseph-sys`)
to **generate a planetary station reference corpus** (the instants at which
Swiss Ephemeris's own longitude speed changes sign, for Mercury–Pluto and the
true lunar node) used to validate the pure-Rust engine's
`EventEngine::stations_in_range`. It runs the Moshier ephemeris (`SEFLG_MOSEPH`), so no Swiss
Ephemeris data files are bundled or distributed.
```

In the root `Cargo.toml`, append `"tools/se-stations-reference"` to the `[workspace] exclude` array.

- [ ] **Step 2: Write the tool**

`tools/se-stations-reference/src/main.rs`:

```rust
//! Emits a Swiss Ephemeris planetary-station reference corpus to STDOUT as
//! CSV. Swiss Ephemeris has no station finder, so a station is located here
//! by scanning the longitude speed of `swe_calc(jd_tt, body, iflag | SEFLG_SPEED)`
//! on a fixed grid and bisecting each sign change to 1e-7 day (Swiss
//! Ephemeris' "ET" argument is TT).
//!
//! Groups:
//!   - `geo`:  geocentric apparent, tropical, true equinox of date (SE default
//!     flags). Mercury–Pluto over the pleiades-events window less five days at
//!     each end; the true node over 1990–2030.
//!   - `mean`: geocentric geometric place in the mean ecliptic and equinox of
//!     date (SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_NONUT).
//!     Mercury, Mars, Saturn over 1990–2030.
//!   - `sid`:  geocentric apparent, sidereal Lahiri (SEFLG_SIDEREAL after
//!     `swe_set_sid_mode(SE_SIDM_LAHIRI)`). Mercury, Mars, Saturn over 1990–2030.
//!
//! Grid: 0.25 day for the planets (their closest stations are 19 days
//! apart). 0.005 day for the true node, whose speed touches zero about every
//! two weeks and can cross it for only a few hours.
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
//!    --manifest-path tools/se-stations-reference/Cargo.toml \
//!    > crates/pleiades-validate/data/stations-corpus/stations.csv`

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::{swe_calc, swe_set_sid_mode};

const SEFLG_MOSEPH: c_int = 4;
const SEFLG_TRUEPOS: c_int = 16; // geometric: no light-time
const SEFLG_NONUT: c_int = 64; // mean equinox of date
const SEFLG_SPEED: c_int = 256;
const SEFLG_NOGDEFL: c_int = 512; // no gravitational deflection
const SEFLG_NOABERR: c_int = 1024; // no annual aberration
const SEFLG_SIDEREAL: c_int = 64 * 1024;

const SE_SIDM_LAHIRI: c_int = 1;
const SE_TRUE_NODE: c_int = 11;

const BASE: c_int = SEFLG_MOSEPH | SEFLG_SPEED;
const MEAN_OF_DATE: c_int = SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_NONUT;

/// (Swiss Ephemeris body id, name as written to the CSV).
const PLANETS: [(c_int, &str); 8] = [
    (2, "Mercury"),
    (3, "Venus"),
    (4, "Mars"),
    (5, "Jupiter"),
    (6, "Saturn"),
    (7, "Uranus"),
    (8, "Neptune"),
    (9, "Pluto"),
];
const SUBSET: [(c_int, &str); 3] = [(2, "Mercury"), (4, "Mars"), (6, "Saturn")];

/// The pleiades-events window (JD 2415020.5–2488069.5) less five days at each
/// end, so neither side of the comparison meets the engine's edge clamp.
const FULL_SPAN: (f64, f64) = (2_415_025.5, 2_488_064.5);
/// 1990-01-01 to 2030-01-01.
const SHORT_SPAN: (f64, f64) = (2_447_892.5, 2_462_502.5);

const PLANET_GRID_DAYS: f64 = 0.25;
const TRUE_NODE_GRID_DAYS: f64 = 0.005;
const BISECT_TOLERANCE_DAYS: f64 = 1e-7;

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
        "non-finite SE result for ipl={ipl} at jd_tt={jd_tt}"
    );
    (xx[0].rem_euclid(360.0), xx[3])
}

/// Prints one row per sign change of the longitude speed in `[lo, hi]`.
fn scan(group: &str, name: &str, ipl: c_int, iflag: c_int, (lo, hi): (f64, f64), grid: f64) {
    let speed = |jd: f64| state(jd, ipl, iflag).1;
    let steps = ((hi - lo) / grid).floor() as u64;
    let mut prev_jd = lo;
    let mut prev = speed(lo);
    for k in 1..=steps {
        let jd = lo + k as f64 * grid;
        let cur = speed(jd);
        if (prev <= 0.0) != (cur <= 0.0) {
            let (mut a, mut b, mut f_a) = (prev_jd, jd, prev);
            while b - a > BISECT_TOLERANCE_DAYS {
                let mid = 0.5 * (a + b);
                let f_mid = speed(mid);
                if (f_a <= 0.0) == (f_mid <= 0.0) {
                    a = mid;
                    f_a = f_mid;
                } else {
                    b = mid;
                }
            }
            let root = 0.5 * (a + b);
            let (lon, _) = state(root, ipl, iflag);
            let kind = if cur > 0.0 { "D" } else { "R" };
            println!("{group},{name},{root:.7},{lon:.9},{kind}");
        }
        prev_jd = jd;
        prev = cur;
    }
}

fn main() {
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), Moshier (SEFLG_MOSEPH, no data files).");
    println!("# A row is a sign change of the longitude speed of swe_calc(jd_tt, body, iflag|SEFLG_SPEED),");
    println!("# bracketed on a 0.25-day grid (0.005 day for TrueNode) and bisected to 1e-7 day. jd_tt is TT.");
    println!("# geo: apparent, tropical, true equinox of date (default flags); planets JD 2415025.5-2488064.5,");
    println!("#   TrueNode JD 2447892.5-2462502.5. mean: SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_NONUT,");
    println!("#   JD 2447892.5-2462502.5. sid: SEFLG_SIDEREAL, SE_SIDM_LAHIRI, JD 2447892.5-2462502.5.");
    println!("# kind: R = turns retrograde, D = turns direct. lon_deg is the longitude at the station.");
    println!("group,body,jd_tt,lon_deg,kind");
    for (ipl, name) in PLANETS {
        scan("geo", name, ipl, BASE, FULL_SPAN, PLANET_GRID_DAYS);
    }
    scan("geo", "TrueNode", SE_TRUE_NODE, BASE, SHORT_SPAN, TRUE_NODE_GRID_DAYS);
    for (ipl, name) in SUBSET {
        scan("mean", name, ipl, BASE | MEAN_OF_DATE, SHORT_SPAN, PLANET_GRID_DAYS);
    }
    unsafe { swe_set_sid_mode(SE_SIDM_LAHIRI, 0.0, 0.0) };
    for (ipl, name) in SUBSET {
        scan("sid", name, ipl, BASE | SEFLG_SIDEREAL, SHORT_SPAN, PLANET_GRID_DAYS);
    }
}
```

- [ ] **Step 3: Generate the corpus**

```bash
mkdir -p crates/pleiades-validate/data/stations-corpus
devenv shell -- env CFLAGS=-std=gnu17 cargo run --release \
  --manifest-path tools/se-stations-reference/Cargo.toml \
  > crates/pleiades-validate/data/stations-corpus/stations.csv
```

Open the CSV and delete any devenv banner line above `# Source:`. The file must begin with `# Source:`.

If `devenv` cannot build the tool in this environment, stop and report; do not hand-write or approximate the corpus.

- [ ] **Step 4: Sanity-check the corpus**

Run: `grep -c '^geo,Mercury,' crates/pleiades-validate/data/stations-corpus/stations.csv`
Expected: between 1240 and 1280 (Mercury stations about 6.3 times a year for 200 years).

Run: `grep '^geo,Mercury,24515[5-9]' crates/pleiades-validate/data/stations-corpus/stations.csv | head -2`
Expected: the first row is kind `R` with `jd_tt` between 2451594 and 2451599 (21 February 2000).

Run: `grep -c '^geo,TrueNode,' crates/pleiades-validate/data/stations-corpus/stations.csv`
Expected: between 1500 and 2600.

Run: `cut -d, -f1,2 crates/pleiades-validate/data/stations-corpus/stations.csv | grep -v '^#' | sort | uniq -c`
Expected: 15 `(group, body)` series plus the header line, none empty; `geo` outer planets about 400 rows each.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml tools/se-stations-reference crates/pleiades-validate/data/stations-corpus/stations.csv
git commit -m "test(validate): Swiss Ephemeris station reference tool and corpus (#85)"
```

Do not commit `tools/se-stations-reference/target/`; confirm with `git status --short` that it is ignored.

---

### Task 4: Gate parsing and comparison rules

**Files:**
- Create: `crates/pleiades-validate/src/stations_thresholds.rs`
- Create: `crates/pleiades-validate/src/stations_validation.rs`
- Create: `crates/pleiades-validate/src/stations_validation/tests.rs`
- Modify: `crates/pleiades-validate/src/lib.rs` (two `mod` lines)

**Interfaces:**
- Consumes: the CSV row format from Task 3.
- Produces (crate-private, used by Task 5):
  - `struct Found { jd: f64, lon_deg: f64, kind: StationKind }`
  - `struct Series { group: Group, body: CelestialBody, body_name: &'static str, stations: Vec<Found> }`
  - `fn parse_corpus(csv: &str) -> Result<Vec<Series>, StationsError>`
  - `fn parse_manifest(manifest: &str) -> Result<(usize, u64), StationsError>`
  - `fn compare_exact(label: &str, engine: &[Found], corpus: &[Found], ceilings: Ceilings) -> Result<Residuals, StationsError>`
  - `fn compare_separated(label: &str, engine: &[Found], corpus: &[Found], ceilings: Ceilings) -> Result<Residuals, StationsError>`
  - `struct Residuals { matched: usize, max_time_s: f64, max_lon_arcsec: f64, sum_signed_time_s: f64 }`
  - `stations_thresholds::{Ceilings { time_s, lon_arcsec }, SEPARATION_DAYS, ceilings_for(body_name) -> Option<Ceilings>, MIN_ROWS_VALIDATED}`
  - `pub enum StationsError`

- [ ] **Step 1: Write the thresholds module with unmeasured ceilings**

`crates/pleiades-validate/src/stations_thresholds.rs`:

```rust
//! Measured-basis ceilings for the `validate-stations` gate.

/// Ceilings for one body: time between the engine's station and the
/// reference's, and the longitude difference at the station.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ceilings {
    pub(crate) time_s: f64,
    pub(crate) lon_arcsec: f64,
}

/// A true-node station is "separated" when its nearest neighbouring station
/// in its own list is at least this many days away. Closer pairs are grazes
/// of the speed against zero whose existence depends on the ephemeris, and
/// are not compared.
pub(crate) const SEPARATION_DAYS: f64 = 2.0;

/// Fail-closed floor on compared stations. Set in Task 5 from the measured
/// count.
pub(crate) const MIN_ROWS_VALIDATED: usize = 0;

/// Not yet measured: Task 5 replaces every value from the gate's own output.
const UNMEASURED: Ceilings = Ceilings {
    time_s: f64::INFINITY,
    lon_arcsec: f64::INFINITY,
};

/// Ceilings by corpus body name, or `None` for a body the gate does not cover.
pub(crate) fn ceilings_for(body_name: &str) -> Option<Ceilings> {
    match body_name {
        "Mercury" | "Venus" | "Mars" | "Jupiter" | "Saturn" | "Uranus" | "Neptune" | "Pluto"
        | "TrueNode" => Some(UNMEASURED),
        _ => None,
    }
}
```

- [ ] **Step 2: Write the failing tests**

`crates/pleiades-validate/src/stations_validation/tests.rs`:

```rust
use super::*;

const D: StationKind = StationKind::TurnsDirect;
const R: StationKind = StationKind::TurnsRetrograde;
const CEILINGS: Ceilings = Ceilings {
    time_s: 600.0,
    lon_arcsec: 5.0,
};
const SECOND: f64 = 1.0 / 86_400.0;

fn found(jd: f64, lon_deg: f64, kind: StationKind) -> Found {
    Found { jd, lon_deg, kind }
}

/// A retrograde loop every 100 days: R at +0, D at +20.
fn loops(count: usize) -> Vec<Found> {
    (0..count)
        .flat_map(|i| {
            let t = 2_451_545.0 + 100.0 * i as f64;
            [found(t, 10.0, R), found(t + 20.0, 2.0, D)]
        })
        .collect()
}

#[test]
fn identical_lists_compare_exactly() {
    let corpus = loops(3);
    let residuals = compare_exact("geo Mars", &corpus, &corpus, CEILINGS).unwrap();
    assert_eq!(residuals.matched, 6);
    assert_eq!(residuals.max_time_s, 0.0);
    assert_eq!(residuals.max_lon_arcsec, 0.0);
}

#[test]
fn exact_comparison_rejects_a_missing_or_extra_station() {
    let corpus = loops(3);
    let mut engine = corpus.clone();
    engine.pop();
    assert!(matches!(
        compare_exact("geo Mars", &engine, &corpus, CEILINGS),
        Err(StationsError::CountMismatch { got: 5, want: 6, .. })
    ));
    assert!(matches!(
        compare_exact("geo Mars", &corpus, &engine, CEILINGS),
        Err(StationsError::CountMismatch { got: 6, want: 5, .. })
    ));
}

#[test]
fn exact_comparison_rejects_a_kind_mismatch() {
    let corpus = loops(2);
    let mut engine = corpus.clone();
    engine[1].kind = R;
    assert!(matches!(
        compare_exact("geo Mars", &engine, &corpus, CEILINGS),
        Err(StationsError::KindMismatch { index: 1, .. })
    ));
}

#[test]
fn exact_comparison_enforces_both_ceilings() {
    let corpus = loops(2);
    let mut late = corpus.clone();
    late[2].jd += 601.0 * SECOND;
    assert!(matches!(
        compare_exact("geo Mars", &late, &corpus, CEILINGS),
        Err(StationsError::CeilingExceeded { kind: "time_seconds", .. })
    ));
    let mut shifted = corpus.clone();
    shifted[2].lon_deg += 6.0 / 3600.0;
    assert!(matches!(
        compare_exact("geo Mars", &shifted, &corpus, CEILINGS),
        Err(StationsError::CeilingExceeded { kind: "longitude_arcsec", .. })
    ));
    let mut nan = corpus.clone();
    nan[0].lon_deg = f64::NAN;
    assert!(matches!(
        compare_exact("geo Mars", &nan, &corpus, CEILINGS),
        Err(StationsError::CeilingExceeded { kind: "longitude_arcsec", .. })
    ));
}

#[test]
fn longitude_residual_wraps_across_zero() {
    let corpus = vec![found(2_451_545.0, 359.999_9, R)];
    let engine = vec![found(2_451_545.0, 0.000_1, R)];
    let residuals = compare_exact("geo Mars", &engine, &corpus, CEILINGS).unwrap();
    assert!((residuals.max_lon_arcsec - 0.72).abs() < 1e-6, "{residuals:?}");
}

#[test]
fn residuals_record_the_maximum_and_the_signed_sum() {
    let corpus = loops(1);
    let mut engine = corpus.clone();
    engine[0].jd += 30.0 * SECOND;
    engine[1].jd -= 10.0 * SECOND;
    let residuals = compare_exact("geo Mars", &engine, &corpus, CEILINGS).unwrap();
    assert!((residuals.max_time_s - 30.0).abs() < 1e-3, "{residuals:?}");
    assert!((residuals.sum_signed_time_s - 20.0).abs() < 1e-3, "{residuals:?}");
}

// A graze: a D/R pair 0.3 day apart that only one side has. Neither station
// is separated, so the lists still agree.
#[test]
fn separated_comparison_ignores_a_close_pair_on_either_side() {
    let corpus = loops(3);
    let mut engine = corpus.clone();
    let graze = 2_451_545.0 + 60.0;
    engine.insert(2, found(graze, 5.0, R));
    engine.insert(3, found(graze + 0.3, 5.0, D));
    engine.sort_by(|a, b| a.jd.total_cmp(&b.jd));
    let residuals = compare_separated("geo TrueNode", &engine, &corpus, CEILINGS).unwrap();
    assert_eq!(residuals.matched, 6);
    let residuals = compare_separated("geo TrueNode", &corpus, &engine, CEILINGS).unwrap();
    assert_eq!(residuals.matched, 6);
}

#[test]
fn separated_comparison_rejects_a_missing_separated_station() {
    let corpus = loops(3);
    let mut engine = corpus.clone();
    engine.remove(4);
    assert!(matches!(
        compare_separated("geo TrueNode", &engine, &corpus, CEILINGS),
        Err(StationsError::Unmatched { side: "corpus", .. })
    ));
    assert!(matches!(
        compare_separated("geo TrueNode", &corpus, &engine, CEILINGS),
        Err(StationsError::Unmatched { side: "engine", .. })
    ));
}

#[test]
fn separated_comparison_requires_the_same_kind_within_the_time_ceiling() {
    let corpus = loops(2);
    let mut wrong_kind = corpus.clone();
    wrong_kind[2].kind = D;
    assert!(matches!(
        compare_separated("geo TrueNode", &wrong_kind, &corpus, CEILINGS),
        Err(StationsError::Unmatched { .. })
    ));
    let mut late = corpus.clone();
    late[2].jd += 601.0 * SECOND;
    assert!(matches!(
        compare_separated("geo TrueNode", &late, &corpus, CEILINGS),
        Err(StationsError::Unmatched { .. })
    ));
    let mut shifted = corpus.clone();
    shifted[2].lon_deg += 6.0 / 3600.0;
    assert!(matches!(
        compare_separated("geo TrueNode", &shifted, &corpus, CEILINGS),
        Err(StationsError::CeilingExceeded { kind: "longitude_arcsec", .. })
    ));
}

#[test]
fn corpus_rows_parse_into_series() {
    let csv = "# comment\ngroup,body,jd_tt,lon_deg,kind\n\
               geo,Mercury,2451596.1234567,17.5,R\n\
               geo,Mercury,2451617.7,2.25,D\n\
               sid,Saturn,2451800.5,60.0,D\n";
    let series = parse_corpus(csv).unwrap();
    assert_eq!(series.len(), 2);
    assert_eq!(series[0].group, Group::Geo);
    assert_eq!(series[0].body_name, "Mercury");
    assert_eq!(
        series[0].stations,
        vec![
            found(2_451_596.123_456_7, 17.5, R),
            found(2_451_617.7, 2.25, D)
        ]
    );
    assert_eq!(series[1].group, Group::Sid);
    assert_eq!(series[1].body, CelestialBody::Saturn);
}

#[test]
fn malformed_rows_are_rejected() {
    for bad in [
        "geo,Mercury,2451596.1,17.5",
        "geo,Mercury,2451596.1,17.5,R,extra",
        "helio,Mercury,2451596.1,17.5,R",
        "geo,Vulcan,2451596.1,17.5,R",
        "geo,Mercury,NaN,17.5,R",
        "geo,Mercury,2451596.1,inf,R",
        "geo,Mercury,2451596.1,17.5,X",
        "geo,Mercury,2451617.7,2.25,D\ngeo,Mercury,2451596.1,17.5,R",
    ] {
        assert!(
            matches!(parse_corpus(bad), Err(StationsError::MalformedRow(_))),
            "{bad}"
        );
    }
}

#[test]
fn manifest_parses_rows_and_checksum() {
    assert_eq!(
        parse_manifest("slice stations file=stations.csv role=stations rows=12 checksum=99")
            .unwrap(),
        (12, 99)
    );
    for bad in ["rows=12 checksum=99", "slice stations rows=12", "slice stations checksum=9"] {
        assert!(
            matches!(parse_manifest(bad), Err(StationsError::MalformedManifest(_))),
            "{bad}"
        );
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Add to `crates/pleiades-validate/src/lib.rs`, next to the `helio_position_*` module lines:

```rust
mod stations_thresholds;
mod stations_validation;
```

Run: `cargo nextest run -p pleiades-validate stations_validation`
Expected: compile errors (`stations_validation.rs` does not exist yet).

- [ ] **Step 4: Write the parsing and comparison code**

`crates/pleiades-validate/src/stations_validation.rs`:

```rust
//! Fail-closed gate: `EventEngine::stations_in_range` on the packaged backend
//! vs the committed Swiss Ephemeris speed-zero reference corpus (issue #85).
//!
//! Swiss Ephemeris has no station finder; the corpus holds the sign changes
//! of its own longitude speed (`tools/se-stations-reference`). Planets must
//! match the corpus station for station. The true node is compared only on
//! separated stations: its speed touches zero about every two weeks, and
//! whether a touch crosses zero for a few hours depends on the ephemeris.
//! See `stations_thresholds` for the basis of the ceilings.

use crate::stations_thresholds::{Ceilings, SEPARATION_DAYS};
use pleiades_events::StationKind;
use pleiades_types::CelestialBody;

const SECONDS_PER_DAY: f64 = 86_400.0;

/// The frame and zodiac a corpus group was generated in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Group {
    /// Geocentric apparent, tropical.
    Geo,
    /// Geocentric mean of date, tropical.
    Mean,
    /// Geocentric apparent, sidereal Lahiri.
    Sid,
}

impl Group {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "geo" => Self::Geo,
            "mean" => Self::Mean,
            "sid" => Self::Sid,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Self::Geo => "geo",
            Self::Mean => "mean",
            Self::Sid => "sid",
        }
    }
}

/// One station, from either side of the comparison.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Found {
    jd: f64,
    lon_deg: f64,
    kind: StationKind,
}

/// The corpus stations of one body in one group, ascending.
#[derive(Clone, Debug)]
struct Series {
    group: Group,
    body: CelestialBody,
    body_name: &'static str,
    stations: Vec<Found>,
}

#[derive(Debug)]
pub enum StationsError {
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
    /// The engine and the corpus disagree on how many stations a planet has.
    CountMismatch {
        series: String,
        got: usize,
        want: usize,
    },
    /// The engine and the corpus disagree on which way a planet turns.
    KindMismatch {
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
    /// A separated true-node station with no counterpart of the same kind
    /// within the time ceiling. `side` names the list the station is in.
    Unmatched {
        series: String,
        side: &'static str,
        jd_tt: f64,
    },
}

impl std::fmt::Display for StationsError {
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
                write!(f, "only {validated} stations validated, floor is {floor}")
            }
            Self::CalculationFailed { series, reason } => {
                write!(f, "{series} station search failed: {reason}")
            }
            Self::CountMismatch { series, got, want } => write!(
                f,
                "{series}: engine found {got} stations, corpus has {want}"
            ),
            Self::KindMismatch {
                series,
                index,
                jd_tt,
            } => write!(
                f,
                "{series}: station {index} near jd_tt={jd_tt} turns the other way in the corpus"
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
            Self::Unmatched {
                series,
                side,
                jd_tt,
            } => write!(
                f,
                "{series}: separated {side} station at jd_tt={jd_tt} has no counterpart of the same kind within the time ceiling"
            ),
        }
    }
}

impl std::error::Error for StationsError {}

fn body_from_name(name: &str) -> Option<(CelestialBody, &'static str)> {
    Some(match name {
        "Mercury" => (CelestialBody::Mercury, "Mercury"),
        "Venus" => (CelestialBody::Venus, "Venus"),
        "Mars" => (CelestialBody::Mars, "Mars"),
        "Jupiter" => (CelestialBody::Jupiter, "Jupiter"),
        "Saturn" => (CelestialBody::Saturn, "Saturn"),
        "Uranus" => (CelestialBody::Uranus, "Uranus"),
        "Neptune" => (CelestialBody::Neptune, "Neptune"),
        "Pluto" => (CelestialBody::Pluto, "Pluto"),
        "TrueNode" => (CelestialBody::TrueNode, "TrueNode"),
        _ => return None,
    })
}

fn parse_corpus(csv: &str) -> Result<Vec<Series>, StationsError> {
    let malformed = |what: String| StationsError::MalformedRow(what);
    let mut all: Vec<Series> = Vec::new();
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("group,") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 5 {
            return Err(malformed(format!(
                "expected 5 fields, got {} in {line}",
                fields.len()
            )));
        }
        let group = Group::from_name(fields[0])
            .ok_or_else(|| malformed(format!("unknown group {} in {line}", fields[0])))?;
        let (body, body_name) = body_from_name(fields[1])
            .ok_or_else(|| malformed(format!("unknown body {} in {line}", fields[1])))?;
        let num = |i: usize| -> Result<f64, StationsError> {
            fields[i]
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| malformed(format!("field {i} is not a finite number in {line}")))
        };
        let kind = match fields[4] {
            "R" => StationKind::TurnsRetrograde,
            "D" => StationKind::TurnsDirect,
            other => return Err(malformed(format!("unknown kind {other} in {line}"))),
        };
        let station = Found {
            jd: num(2)?,
            lon_deg: num(3)?,
            kind,
        };
        match all
            .iter_mut()
            .find(|s| s.group == group && s.body_name == body_name)
        {
            Some(series) => {
                if series.stations.last().is_some_and(|last| last.jd >= station.jd) {
                    return Err(malformed(format!("rows are not ascending at {line}")));
                }
                series.stations.push(station);
            }
            None => all.push(Series {
                group,
                body,
                body_name,
                stations: vec![station],
            }),
        }
    }
    Ok(all)
}

fn parse_manifest(manifest: &str) -> Result<(usize, u64), StationsError> {
    let malformed = |what: String| StationsError::MalformedManifest(what);
    let line = manifest
        .lines()
        .find(|l| l.trim_start().starts_with("slice"))
        .ok_or_else(|| malformed("no slice line".into()))?;
    let mut rows = None;
    let mut checksum = None;
    for tok in line.split_whitespace() {
        if let Some(v) = tok.strip_prefix("rows=") {
            rows = Some(v.parse::<usize>().map_err(|e| malformed(format!("rows: {e}")))?);
        } else if let Some(v) = tok.strip_prefix("checksum=") {
            checksum = Some(v.parse::<u64>().map_err(|e| malformed(format!("checksum: {e}")))?);
        }
    }
    Ok((
        rows.ok_or_else(|| malformed("rows= missing".into()))?,
        checksum.ok_or_else(|| malformed("checksum= missing".into()))?,
    ))
}

/// What one series' comparison measured.
#[derive(Clone, Copy, Debug, Default)]
struct Residuals {
    /// Stations compared.
    matched: usize,
    max_time_s: f64,
    max_lon_arcsec: f64,
    /// Sum of engine − corpus time over the compared stations; a mean far
    /// from zero means the two speeds differ by convention, not by noise.
    sum_signed_time_s: f64,
}

fn lon_residual_arcsec(got_deg: f64, want_deg: f64) -> f64 {
    ((got_deg - want_deg + 180.0).rem_euclid(360.0) - 180.0).abs() * 3600.0
}

/// Checks one engine/corpus pair against both ceilings and records it. A NaN
/// residual fails closed.
fn record(
    label: &str,
    engine: &Found,
    corpus: &Found,
    ceilings: Ceilings,
    residuals: &mut Residuals,
) -> Result<(), StationsError> {
    let signed_time_s = (engine.jd - corpus.jd) * SECONDS_PER_DAY;
    let checks = [
        ("time_seconds", signed_time_s.abs(), ceilings.time_s),
        (
            "longitude_arcsec",
            lon_residual_arcsec(engine.lon_deg, corpus.lon_deg),
            ceilings.lon_arcsec,
        ),
    ];
    for (kind, residual, ceiling) in checks {
        if residual.is_nan() || residual > ceiling {
            return Err(StationsError::CeilingExceeded {
                series: label.to_string(),
                jd_tt: corpus.jd,
                kind,
                residual,
                ceiling,
            });
        }
    }
    residuals.matched += 1;
    residuals.max_time_s = residuals.max_time_s.max(checks[0].1);
    residuals.max_lon_arcsec = residuals.max_lon_arcsec.max(checks[1].1);
    residuals.sum_signed_time_s += signed_time_s;
    Ok(())
}

/// Planets: the two lists must agree station for station.
fn compare_exact(
    label: &str,
    engine: &[Found],
    corpus: &[Found],
    ceilings: Ceilings,
) -> Result<Residuals, StationsError> {
    if engine.len() != corpus.len() {
        return Err(StationsError::CountMismatch {
            series: label.to_string(),
            got: engine.len(),
            want: corpus.len(),
        });
    }
    let mut residuals = Residuals::default();
    for (index, (got, want)) in engine.iter().zip(corpus).enumerate() {
        if got.kind != want.kind {
            return Err(StationsError::KindMismatch {
                series: label.to_string(),
                index,
                jd_tt: want.jd,
            });
        }
        record(label, got, want, ceilings, &mut residuals)?;
    }
    Ok(residuals)
}

/// Whether `list[index]`'s nearest neighbour is at least
/// [`SEPARATION_DAYS`] away.
fn is_separated(list: &[Found], index: usize) -> bool {
    let here = list[index].jd;
    let far = |other: Option<&Found>| {
        other.is_none_or(|other| (other.jd - here).abs() >= SEPARATION_DAYS)
    };
    far(index.checked_sub(1).and_then(|i| list.get(i))) && far(list.get(index + 1))
}

/// The station of `kind` in `list` nearest in time to `jd`, if it is within
/// `time_s` seconds.
fn nearest_of_kind(list: &[Found], jd: f64, kind: StationKind, time_s: f64) -> Option<&Found> {
    list.iter()
        .filter(|candidate| candidate.kind == kind)
        .min_by(|a, b| (a.jd - jd).abs().total_cmp(&(b.jd - jd).abs()))
        .filter(|nearest| (nearest.jd - jd).abs() * SECONDS_PER_DAY <= time_s)
}

/// The true node: every separated station on either side must have a
/// counterpart of the same kind within the time ceiling; stations in closer
/// pairs are unconstrained.
fn compare_separated(
    label: &str,
    engine: &[Found],
    corpus: &[Found],
    ceilings: Ceilings,
) -> Result<Residuals, StationsError> {
    let unmatched = |side: &'static str, jd_tt: f64| StationsError::Unmatched {
        series: label.to_string(),
        side,
        jd_tt,
    };
    let mut residuals = Residuals::default();
    for (index, want) in corpus.iter().enumerate() {
        if !is_separated(corpus, index) {
            continue;
        }
        let got = nearest_of_kind(engine, want.jd, want.kind, ceilings.time_s)
            .ok_or_else(|| unmatched("corpus", want.jd))?;
        record(label, got, want, ceilings, &mut residuals)?;
    }
    for (index, got) in engine.iter().enumerate() {
        if is_separated(engine, index)
            && nearest_of_kind(corpus, got.jd, got.kind, ceilings.time_s).is_none()
        {
            return Err(unmatched("engine", got.jd));
        }
    }
    Ok(residuals)
}

#[cfg(test)]
mod tests;
```

`Group::name` and the `ChecksumMismatch`, `ManifestDrift`, `TooFewRowsValidated` and `CalculationFailed` variants are used by Task 5. Until then, put `#![allow(dead_code)]` with the comment `// Consumed by the gate entry point added next.` at the top of the file below the module doc, and remove it in Task 5.

- [ ] **Step 5: Run the tests**

Run: `cargo nextest run -p pleiades-validate stations_validation`
Expected: all 12 PASS.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
git add crates/pleiades-validate/src
git commit -m "test(validate): station gate parsing and comparison rules (#85)"
```

---

### Task 5: The `validate-stations` gate, measured ceilings and wiring

**Files:**
- Modify: `crates/pleiades-validate/src/stations_validation.rs` (entry point, report)
- Modify: `crates/pleiades-validate/src/stations_validation/tests.rs`
- Modify: `crates/pleiades-validate/src/stations_thresholds.rs` (measured values)
- Create: `crates/pleiades-validate/data/stations-corpus/manifest.txt`
- Modify: `crates/pleiades-validate/src/lib.rs` (`pub use`)
- Modify: `crates/pleiades-validate/src/render/cli.rs` (`run_all_numeric_gates`, command arm, help text)
- Modify: `crates/pleiades-cli/src/cli.rs` (routing arm)

**Interfaces:**
- Consumes: everything Task 4 produces; `EventEngine::stations_in_range`, `Station`, `StationKind` (Task 2); the corpus (Task 3).
- Produces: `pub fn validate_stations_corpus() -> Result<StationsReport, StationsError>`; `pub struct StationsReport { pub rows_validated: usize, .. }` with `pub fn summary_line(&self) -> &str`; CLI commands `validate-stations` and alias `stations-gate`.

- [ ] **Step 1: Write the manifest with a placeholder checksum**

Count the data rows:

Run: `grep -c -E '^(geo|mean|sid),' crates/pleiades-validate/data/stations-corpus/stations.csv`

Create `crates/pleiades-validate/data/stations-corpus/manifest.txt` with that count as `rows=` and `checksum=0`:

```
slice stations file=stations.csv role=stations rows=<count from the command above> checksum=0
```

- [ ] **Step 2: Write the failing tests**

Append to `crates/pleiades-validate/src/stations_validation/tests.rs`:

```rust
#[test]
fn stations_gate_passes_within_ceilings() {
    let report = validate_stations_corpus().expect("stations gate passes");
    eprintln!("{}", report.summary_line());
    for line in report.series_lines() {
        eprintln!("{line}");
    }
    assert!(report.rows_validated >= MIN_ROWS_VALIDATED);
}

#[test]
fn tampered_corpus_fails_the_checksum() {
    let tampered = CORPUS_CSV.replacen("geo,Mercury,", "geo,Mercury, ", 1);
    assert!(matches!(
        validate(&tampered, MANIFEST),
        Err(StationsError::ChecksumMismatch { .. })
    ));
}

#[test]
fn manifest_row_count_drift_fails_closed() {
    let (rows, checksum) = parse_manifest(MANIFEST).unwrap();
    let manifest = format!(
        "slice stations file=stations.csv role=stations rows={} checksum={checksum}",
        rows - 1
    );
    assert!(matches!(
        validate(CORPUS_CSV, &manifest),
        Err(StationsError::ManifestDrift { .. })
    ));
}

#[test]
fn a_corpus_missing_a_station_fails_the_count() {
    // Drop Mars's first mean-of-date station: the engine then finds one more
    // than the corpus has.
    let line = CORPUS_CSV
        .lines()
        .find(|l| l.starts_with("mean,Mars,"))
        .expect("a mean Mars row");
    let csv: String = CORPUS_CSV
        .lines()
        .filter(|l| l.starts_with("mean,Mars,") && *l != line)
        .map(|l| format!("{l}\n"))
        .collect();
    let rows = csv.lines().count();
    let manifest = format!("slice x rows={rows} checksum={}", fnv1a64(&csv));
    assert!(matches!(
        validate(&csv, &manifest),
        Err(StationsError::CountMismatch { .. })
    ));
}

#[test]
fn a_shifted_reference_instant_exceeds_the_time_ceiling() {
    // Move Mars's first mean-of-date station by 0.5 day, far above any
    // planetary time ceiling.
    let line = CORPUS_CSV
        .lines()
        .find(|l| l.starts_with("mean,Mars,"))
        .expect("a mean Mars row");
    let mut fields: Vec<String> = line.split(',').map(str::to_string).collect();
    let jd: f64 = fields[2].parse().unwrap();
    fields[2] = format!("{:.7}", jd + 0.5);
    let csv: String = CORPUS_CSV
        .lines()
        .filter(|l| l.starts_with("mean,Mars,"))
        .map(|l| {
            if l == line {
                format!("{}\n", fields.join(","))
            } else {
                format!("{l}\n")
            }
        })
        .collect();
    let rows = csv.lines().count();
    let manifest = format!("slice x rows={rows} checksum={}", fnv1a64(&csv));
    assert!(matches!(
        validate(&csv, &manifest),
        Err(StationsError::CeilingExceeded {
            kind: "time_seconds",
            ..
        })
    ));
}

#[test]
fn an_empty_corpus_validates_too_few_rows() {
    let csv = "group,body,jd_tt,lon_deg,kind\n";
    let manifest = format!("slice x rows=0 checksum={}", fnv1a64(csv));
    assert!(matches!(
        validate(csv, &manifest),
        Err(StationsError::TooFewRowsValidated { validated: 0, .. })
    ));
}
```

`a_shifted_reference_instant_exceeds_the_time_ceiling` cannot pass until the ceilings are measured in Step 6 (they are infinite until then); that is expected.

- [ ] **Step 3: Write the gate entry point**

In `crates/pleiades-validate/src/stations_validation.rs`: remove the `#![allow(dead_code)]` line and its comment, replace the `use` block with

```rust
use crate::stations_thresholds::{ceilings_for, Ceilings, MIN_ROWS_VALIDATED, SEPARATION_DAYS};
use pleiades_apparent::fnv1a64;
use pleiades_data::packaged_backend;
use pleiades_events::{CrossingFrame, CrossingReference, EventEngine, StationKind};
use pleiades_types::{Ayanamsa, CelestialBody, Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/stations-corpus/stations.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/stations-corpus/manifest.txt"
));

/// The spans `tools/se-stations-reference` scanned (Julian days, TT). The
/// full span is the engine's window less five days at each end, so neither
/// side of the comparison meets the engine's edge clamp.
const FULL_SPAN: (f64, f64) = (2_415_025.5, 2_488_064.5);
/// 1990-01-01 to 2030-01-01.
const SHORT_SPAN: (f64, f64) = (2_447_892.5, 2_462_502.5);
```

add to `impl Group`:

```rust
    fn reference(self) -> CrossingReference {
        match self {
            Self::Geo => CrossingFrame::GeocentricApparentOfDate.into(),
            Self::Mean => CrossingFrame::GeocentricMeanOfDate.into(),
            Self::Sid => CrossingReference::sidereal(
                CrossingFrame::GeocentricApparentOfDate,
                Ayanamsa::Lahiri,
            ),
        }
    }
```

and add, above `#[cfg(test)] mod tests;`:

```rust
#[derive(Debug)]
pub struct StationsReport {
    /// Stations compared against the corpus.
    pub rows_validated: usize,
    series_lines: Vec<String>,
    summary_line: String,
}

impl StationsReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }

    /// One line per corpus series with its measured maxima; the basis for
    /// the ceilings in `stations_thresholds`.
    pub fn series_lines(&self) -> &[String] {
        &self.series_lines
    }
}

fn span(series: &Series) -> (f64, f64) {
    if series.group == Group::Geo && series.body != CelestialBody::TrueNode {
        FULL_SPAN
    } else {
        SHORT_SPAN
    }
}

fn validate(csv: &str, manifest: &str) -> Result<StationsReport, StationsError> {
    let (manifest_rows, manifest_checksum) = parse_manifest(manifest)?;
    let got_checksum = fnv1a64(csv);
    if got_checksum != manifest_checksum {
        return Err(StationsError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let all = parse_corpus(csv)?;
    let rows_csv: usize = all.iter().map(|series| series.stations.len()).sum();
    if rows_csv != manifest_rows {
        return Err(StationsError::ManifestDrift {
            rows_csv,
            rows_manifest: manifest_rows,
        });
    }

    let engine = EventEngine::new(packaged_backend());
    // The corpus epoch is TT; the engine reads the Julian day as TDB. The two
    // differ by under 2 ms, far below every ceiling here.
    let tdb = |jd: f64| Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
    let mut validated = 0usize;
    let mut series_lines = Vec::new();
    let (mut max_time_s, mut max_lon_arcsec) = (0.0_f64, 0.0_f64);
    for series in &all {
        let label = format!("{} {}", series.group.name(), series.body_name);
        let failed = |reason: String| StationsError::CalculationFailed {
            series: label.clone(),
            reason,
        };
        let ceilings = ceilings_for(series.body_name)
            .ok_or_else(|| failed("no ceilings for this body".into()))?;
        let (start, end) = span(series);
        let found: Vec<Found> = engine
            .stations_in_range(
                series.body.clone(),
                series.group.reference(),
                tdb(start),
                tdb(end),
            )
            .map_err(|e| failed(e.to_string()))?
            .into_iter()
            .map(|station| Found {
                jd: station.instant.julian_day.days(),
                lon_deg: station.longitude.degrees(),
                kind: station.kind,
            })
            .collect();
        let residuals = if series.body == CelestialBody::TrueNode {
            compare_separated(&label, &found, &series.stations, ceilings)?
        } else {
            compare_exact(&label, &found, &series.stations, ceilings)?
        };
        validated += residuals.matched;
        max_time_s = max_time_s.max(residuals.max_time_s);
        max_lon_arcsec = max_lon_arcsec.max(residuals.max_lon_arcsec);
        let mean_signed_s = if residuals.matched == 0 {
            0.0
        } else {
            residuals.sum_signed_time_s / residuals.matched as f64
        };
        series_lines.push(format!(
            "{label}: {} compared (engine {}, corpus {}), max time {:.1} s, mean signed time {:+.1} s, max lon {:.3}\"",
            residuals.matched,
            found.len(),
            series.stations.len(),
            residuals.max_time_s,
            mean_signed_s,
            residuals.max_lon_arcsec,
        ));
    }
    let floor = MIN_ROWS_VALIDATED.max(1);
    if validated < floor {
        return Err(StationsError::TooFewRowsValidated { validated, floor });
    }
    let summary_line = format!(
        "Stations gate: {validated} stations validated across {} series vs Swiss Ephemeris speed-zero corpus \
         (planets station-for-station; true node on stations separated by >= {SEPARATION_DAYS} d), \
         max time {max_time_s:.1} s, max lon {max_lon_arcsec:.3}\"",
        all.len(),
    );
    Ok(StationsReport {
        rows_validated: validated,
        series_lines,
        summary_line,
    })
}

pub fn validate_stations_corpus() -> Result<StationsReport, StationsError> {
    validate(CORPUS_CSV, MANIFEST)
}
```

In `crates/pleiades-validate/src/lib.rs`, next to the `helio_position_validation` re-export:

```rust
pub use stations_validation::{validate_stations_corpus, StationsError, StationsReport};
```

- [ ] **Step 4: Fill in the manifest checksum**

Run: `cargo nextest run -p pleiades-validate stations_gate_passes_within_ceilings --no-capture`
Expected: FAIL with `corpus checksum mismatch: got <N> want 0`.

Replace `checksum=0` in `crates/pleiades-validate/data/stations-corpus/manifest.txt` with `checksum=<N>`.

- [ ] **Step 5: Measure**

Run: `cargo nextest run -p pleiades-validate stations_gate_passes_within_ceilings --no-capture`
Expected: PASS in roughly two to four minutes (the ceilings are still infinite), printing the summary line and fifteen series lines. Save the output; it is the measurement record.

Read the series lines before setting any ceiling:

- **A `CountMismatch` or `KindMismatch` for a planet is a defect, not something to tune.** Investigate it (print the engine's and the corpus's stations around the first disagreement). The one benign cause is a station within the time residual of a span edge, present on one side only; if that is what happened, say so in the commit message, nudge that end of the span by one day in both `tools/se-stations-reference/src/main.rs` and the gate's span constant, and regenerate the corpus (Task 3 Step 3) and the manifest.
- **An `Unmatched` true-node station** means a separated station on one side has no counterpart. With infinite ceilings this cannot occur; it can occur after Step 6. If it does, do not raise `SEPARATION_DAYS` or the time ceiling to make it pass without first printing the engine's and the corpus's stations within five days of it and reporting what they show.
- **A planet whose `mean signed time` is comparable to its `max time`** (the engine is consistently early or consistently late) indicates the two speeds differ by convention. Report it and stop before setting that body's ceiling.
- **A planetary `max lon` above 5″** is larger than the known Moshier-versus-DE440 difference. Report it and stop before setting that body's ceiling.

- [ ] **Step 6: Set the ceilings and the floor from the measurement**

In `crates/pleiades-validate/src/stations_thresholds.rs`, delete `UNMEASURED` and give every body its own arm in `ceilings_for`. For each body take the largest `max time` and the largest `max lon` over all of that body's series (`geo`, and `mean` and `sid` where present), multiply by 1.5 and round up to two significant figures. Record the measured value and the series it came from in a trailing comment, in the style of `helio_position_thresholds.rs`:

```rust
        // measured max 41.3 s (geo), 0.812" (sid)
        "Mercury" => Some(Ceilings {
            time_s: 62.0,
            lon_arcsec: 1.3,
        }),
```

(The numbers above show the format only; use the measured ones.)

Replace the `ceilings_for` doc comment's basis with a module-level comment stating: the measurement date, that the residual is the difference between the packaged backend (DE440-derived) and Swiss Ephemeris Moshier speeds divided by the body's longitude acceleration at the station, that this is why slow bodies have large time ceilings, and the 1.5× rule.

Set `MIN_ROWS_VALIDATED` to the `rows_validated` the run printed, and replace its doc comment's second sentence with the measured count and date.

- [ ] **Step 7: Run the gate tests**

Run: `cargo nextest run -p pleiades-validate stations_validation`
Expected: all PASS, including `a_shifted_reference_instant_exceeds_the_time_ceiling` and `an_empty_corpus_validates_too_few_rows`.

If `a_shifted_reference_instant_exceeds_the_time_ceiling` fails because Mars's time ceiling is above 43 200 s (0.5 day), that ceiling is not a meaningful gate: report it rather than enlarging the shift.

- [ ] **Step 8: Wire the gate into the CLI and the release battery**

In `crates/pleiades-validate/src/render/cli.rs`:

1. In `run_all_numeric_gates`, after the `validate_helio_position_corpus` call:

```rust
    crate::validate_stations_corpus().map_err(|e| format!("stations gate failed: {e}"))?;
```

2. After the `Some("validate-helio-position") | Some("helio-position-gate")` arm:

```rust
        Some("validate-stations") | Some("stations-gate") => {
            ensure_no_extra_args(&args[1..], "validate-stations")?;
            crate::validate_stations_corpus()
                .map(|report| report.summary_line().to_string())
                .map_err(|e| e.to_string())
        }
```

3. In the help text string (search for `helio-position-gate       Alias for validate-helio-position\n`), insert directly after that alias line:

```
  validate-stations         Run the fail-closed planetary station gate (Swiss Ephemeris speed-zero reference: station instants and longitudes, Mercury-Pluto station for station, true node on separated stations) over the committed stations corpus\n  stations-gate             Alias for validate-stations\n
```

In `crates/pleiades-cli/src/cli.rs`, after the `Some("validate-helio-position") | Some("helio-position-gate") => validate_render_cli(args),` arm:

```rust
        Some("validate-stations") | Some("stations-gate") => validate_render_cli(args),
```

- [ ] **Step 9: Run the gate end to end**

Run: `cargo run -q -p pleiades-validate -- validate-stations`
Expected: exit 0, printing the `Stations gate: ...` summary line.

Run: `cargo run -q -p pleiades-cli -- stations-gate`
Expected: the same line.

Run: `cargo nextest run -p pleiades-validate -E 'test(help) or test(usage)'`
Expected: PASS. If a help-text test pins the full command list, add the two new lines to its expectation.

Record the wall time of the `validate-stations` run; Task 6 writes it into `docs/follow-ups.md`. If it exceeds 300 s, report it before continuing: the spec's fallback (move the full span out of the blocking path) is a decision for the user.

- [ ] **Step 10: Commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
git add crates/pleiades-validate crates/pleiades-cli
git commit -m "feat(validate): validate-stations gate against the Swiss Ephemeris speed-zero corpus (#85)"
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
- Modify: `spec/astrology-domain.md`
- Modify: `docs/superpowers/specs/2026-10-02-planetary-stations-design.md` (status line)

**Interfaces:**
- Consumes: the measured maxima, validated count and gate wall time from Task 5.
- Produces: no code interfaces.

- [ ] **Step 1: Crate README**

In `crates/pleiades-events/README.md`, change the first paragraph's ending from `and their ecliptic positions and speeds.` to `their ecliptic positions and speeds, and their stations.`, and append this section at the end of the file, replacing each `<...>` with the Task 5 measurement:

```markdown
## Stations

`EventEngine::stations_in_range(body, reference, start, end)` and
`EventEngine::next_station(body, reference, after)` find the instants a body's
longitude speed changes sign. A `Station` carries the TDB instant, the
longitude there, and a `StationKind` (`TurnsRetrograde` or `TurnsDirect`).

A station is a sign change of the speed `position_at` reports in the same
frame and zodiac, so `position_at` just before and at a returned instant
always disagree in direction. A returned instant can be handed back to
`next_station`, which then returns the following station. The zodiac matters:
a sidereal speed is lower by the ayanamsa's rate, which moves a slow planet's
station by minutes to hours.

The Sun, the Moon, the mean node and every body in the heliocentric frame
never station and return nothing. A backend that reports no longitude speed is
`EventError::MissingSpeed`, never an empty list.

A station's instant is soft: near a station the speed changes slowly, so a
small speed difference between two ephemerides is a large time difference.
`validate-stations` compares the engine with the sign changes of Swiss
Ephemeris's own longitude speed (Moshier), 1900–2100 for the planets:

| Body | Largest time difference | Largest longitude difference |
|---|---|---|
| Mercury | <measured> s | <measured>″ |
| Venus | <measured> s | <measured>″ |
| Mars | <measured> s | <measured>″ |
| Jupiter | <measured> s | <measured>″ |
| Saturn | <measured> s | <measured>″ |
| Uranus | <measured> s | <measured>″ |
| Neptune | <measured> s | <measured>″ |
| Pluto | <measured> s | <measured>″ |
| True node (1990–2030, separated stations) | <measured> s | <measured>″ |

The search steps by 0.25 day for the Moon and the lunar points, 1 day for the
Sun, Mercury and Venus, and 2 days otherwise; two stations closer together
than the step are not reported. That happens only for the osculating lunar
points. The true node is retrograde on average and its speed touches zero
about every two weeks; whether a touch crosses zero for a few hours depends on
the ephemeris, so the gate compares only true-node stations at least 2 days
from their neighbours. Stations of asteroids, fictitious bodies and the
osculating apogee are found but not gated.
```

- [ ] **Step 2: Workspace README and crate description**

In `README.md`, add this row to the capability table directly below the `Ecliptic position & speed` row (use the largest planetary time difference from Task 5, rounded to a whole unit, for the last cell):

```markdown
| Planetary stations (geocentric apparent or mean of date; tropical or sidereal) | [`pleiades-events`](crates/pleiades-events) | `validate-stations` | arcsecond-class longitude; timing within <measured> of Swiss Ephemeris |
```

In `crates/pleiades-events/Cargo.toml`, change the `description` to:

```toml
description = "Ephemeris event-finding for the pleiades astrology workspace: longitude crossings (solcross / mooncross / general-body / heliocentric helio_cross), ecliptic positions and planetary stations, derived from pleiades' validated body positions."
```

- [ ] **Step 3: Compatibility profile**

In `crates/pleiades-core/src/compatibility/mod.rs`:

- Line 26: change `pleiades-compatibility-profile/0.7.18` to `pleiades-compatibility-profile/0.7.19`.
- Append this entry to the same string list that ends with the `Mean-place and sidereal crossings (issue #88) additions: ...` entry, directly after it:

```rust
            "Planetary stations (issue #85) additions: EventEngine::stations_in_range and EventEngine::next_station find the instants a body's longitude speed changes sign, in any CrossingFrame and zodiac, returning a Station (TDB instant, longitude, and a StationKind of TurnsRetrograde or TurnsDirect). A station is a sign change of the speed EventEngine::position_at reports, so it is where the engine's own direction flips; a backend with no longitude speed is EventError::MissingSpeed. Swiss Ephemeris has no station finder, so this goes beyond parity; the validate-stations gate compares against the sign changes of Swiss Ephemeris's own longitude speed, station for station for Mercury-Pluto over 1900-2100 and on stations at least 2 days from their neighbours for the true node over 1990-2030.",
```

In `crates/pleiades-cli/src/cli/tests/summary_commands.rs:440` and `crates/pleiades-validate/src/tests/render_request.rs:333`, change `0.7.18` to `0.7.19`.

Run: `cargo nextest run -p pleiades-core compatibility`
Expected: PASS. If a test pins a checksum or a line count of the profile text, update the pinned value to the one the failure message reports, and include that file in the commit.

Run: `cargo nextest run -p pleiades-validate render_request`
Expected: PASS.

- [ ] **Step 4: Follow-ups and spec**

Append to `docs/follow-ups.md`, replacing each `<...>` from the Task 5 output:

```markdown
---

## FU-21: Planetary station finder (issue #85)

**Status:** resolved (2026-10-02) · Spec
`docs/superpowers/specs/2026-10-02-planetary-stations-design.md`, plan
`docs/superpowers/plans/2026-10-02-planetary-stations.md`.

`EventEngine::stations_in_range` and `EventEngine::next_station`
(`pleiades-events` `src/stations.rs`) find sign changes of the longitude speed
`position_at` reports. `EventError` gained `MissingSpeed` and is now
`#[non_exhaustive]` (breaking).

**Gate:** `validate-stations` (<rows>-row Swiss Ephemeris speed-zero corpus,
`tools/se-stations-reference`). Measured 2026-10-02:

```
<the summary line and the fifteen series lines printed in Task 5 Step 5>
```

Gate wall time: <measured> s.

**Open items:**

- **(a) The true node grazes zero.** Its speed touches zero about every two
  weeks; over two years the engine finds 96, 98, 98 and 102 stations at steps
  of 0.5, 0.25, 0.1 and 0.02 day. Pairs closer than the 0.25-day step are not
  reported, and the gate compares only stations at least 2 days from their
  neighbours. A caller who needs every graze has no way to ask for a finer
  step.
- **(b) `previous_station`** is not provided; it would inherit FU-13's
  backward-search caveat.
- **(c) No user-facing CLI stations command.**
- **(d) Ungated bodies.** Asteroids, fictitious bodies and the osculating
  apogee are accepted by the finders but have no reference corpus (FU-7
  records asteroid speed defects).
- **(e) Cost for bodies that never station.** `next_station` scans to the end
  of the window before returning `None` (about 0.3 ms per step).

**Severity:** (a) documented limit, (b)–(d) feature gaps, (e) performance ·
**Opened:** 2026-10-02
```

In `spec/astrology-domain.md`, change the bullet `- retrograde and stationary classification` to:

```markdown
- retrograde and stationary classification, and the instants of stations (`pleiades-events` `EventEngine::stations_in_range` / `next_station`)
```

In `docs/superpowers/specs/2026-10-02-planetary-stations-design.md`, change the status line to `**Status:** implemented (2026-10-02) ·`.

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
git commit -m "docs: planetary stations in the READMEs, compatibility profile and follow-ups (#85)"
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

- [ ] **Step 2: Run the validation crate's station and battery tests**

Run: `cargo nextest run -p pleiades-validate -E 'test(stations) or test(run_all_numeric_gates)'`
Expected: PASS.

- [ ] **Step 3: Confirm the crossings surface did not move**

Run: `cargo run -q -p pleiades-validate -- validate-crossings`
Expected: exit 0. The summary line must equal the one `main` prints; if in doubt, run the same command in a temporary `git worktree` of `main` and compare the two lines.

Run: `git diff --stat main -- '*crossings-golden*' crates/pleiades-validate/data/crossings-corpus`
Expected: empty.

- [ ] **Step 4: Confirm the branch contents**

Run: `git status --short`
Expected: clean.

Run: `git log --oneline main..HEAD`
Expected: the two spec commits, the plan commit, and one commit per Task 1–6, each ending with `(#85)`.

- [ ] **Step 5: Report**

Summarise: the measured per-body maxima, the validated row count, the gate wall time, any investigation Task 5 Step 5 triggered and its outcome, and the open items recorded in FU-21.
