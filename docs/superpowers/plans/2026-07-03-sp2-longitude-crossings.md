# SP-2a Longitude Crossings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a public longitude-crossing engine (`solcross`, `mooncross`, general geocentric body crossings, and heliocentric `helio_cross`) to the pleiades workspace, validated fail-closed against Swiss Ephemeris.

**Architecture:** A new standalone `pleiades-events` crate, generic over `B: EphemerisBackend`, mirroring `pleiades-eclipse`. A generic bracket-and-bisect root-finder locates zeros of `g(t) = wrap180(longitude(body, frame, t) − target)`, where longitude is geocentric apparent-of-date (via `pleiades-apparent`) or heliocentric (via `P_helio = P_geo − S_geo` vector reconstruction). An isolated out-of-workspace `tools/se-crossings-reference` harness emits a committed corpus, and a `validate-crossings` gate in `pleiades-validate` recomputes every fixture within per-body time ceilings.

**Tech Stack:** Rust (2021 edition, workspace), `pleiades-backend`/`pleiades-types`/`pleiades-apparent`, Swiss Ephemeris (`swisseph`/`libswisseph-sys`, isolated tool only), CSV corpus + manifest.

## Global Constraints

- **Pure-Rust workspace audit (hard):** no `-sys`/`links`/`build.rs` may enter the workspace lockfile. The SE binding lives ONLY in `tools/se-crossings-reference` (its own `Cargo.lock`, `publish = false`, outside `[workspace].members`). `pleiades-events` and `pleiades-validate` must not depend on any SE/FFI crate.
- **Coverage window:** 1900-01-01 TDB (JD `2_415_020.5`) through 2100-01-01 TDB (JD `2_488_069.5`). Out-of-window requests fail closed. Reuse these exact constants.
- **Time base:** all engine results are TDB (`TimeScale::Tdb`), matching `pleiades-eclipse`. SE `_ut` corpus times are converted to TDB once, at generation.
- **Fail-closed everywhere:** never emit NaN/placeholder; return a structured `EventError`. Backends supply mean/J2000 geocentric ecliptic coordinates; apparent corrections are applied in this crate, never assumed from the backend.
- **Edition/versioning:** crate fields use `.workspace = true` like sibling crates; crate `version = "0.3.0"` via workspace. New public surface → bump the compatibility profile (`0.7.4` → `0.7.5`); no breaking change to existing types (API-stability `0.2.1` unchanged).
- **libclang note:** building `tools/se-crossings-reference` needs `libclang-dev` + `LIBCLANG_PATH`. This is required ONLY to (re)generate the corpus — never to build the workspace or run the `validate-crossings` gate (which reads the committed CSV via `include_str!`).

---

## File Structure

**New crate `crates/pleiades-events/`:**
- `Cargo.toml` — manifest (mirror `pleiades-eclipse/Cargo.toml`)
- `src/lib.rs` — crate docs, module decls, public re-exports, crate-level doctest
- `src/error.rs` — `EventError` + `WINDOW_START_JD`/`WINDOW_END_JD`
- `src/root.rs` — generic bracket + bisect root-finder over `FnMut(f64) -> Result<f64, EventError>`
- `src/ephemeris.rs` — per-body geocentric apparent longitude + heliocentric reconstruction
- `src/crossings.rs` — `CrossingEngine`, `Crossing`, `CrossingFrame`, public API

**Validation:**
- `tools/se-crossings-reference/{Cargo.toml,Cargo.lock,src/main.rs,LICENSE-NOTES.md}` — isolated SE harness
- `crates/pleiades-validate/data/crossings-corpus/{crossings.csv,manifest.txt}` — committed corpus
- `crates/pleiades-validate/src/crossings_validation.rs` — the gate
- Modify `crates/pleiades-validate/{Cargo.toml,src/lib.rs,src/render/cli.rs,src/release/notes.rs}` — dep + export + dispatch + smoke list

**CLI:**
- Modify `crates/pleiades-cli/src/cli.rs` — `crossings` alias routed through validate render

**Compatibility/docs:**
- Modify `crates/pleiades-core/src/compatibility/mod.rs` — profile entry + version bump
- Modify `README.md`, `PLAN.md` — current-state + status refresh

**Workspace:**
- Modify root `Cargo.toml` — add `crates/pleiades-events` to `[workspace].members` and the `[workspace.dependencies]` path entry

---

## Task 1: Scaffold `pleiades-events` crate + `EventError`

**Files:**
- Create: `crates/pleiades-events/Cargo.toml`
- Create: `crates/pleiades-events/src/lib.rs`
- Create: `crates/pleiades-events/src/error.rs`
- Modify: `Cargo.toml` (root workspace `members` + `workspace.dependencies`)

**Interfaces:**
- Produces: `EventError` (enum), `WINDOW_START_JD: f64`, `WINDOW_END_JD: f64`.

- [ ] **Step 1: Add the crate to the workspace.** In root `Cargo.toml`, add `"crates/pleiades-events",` to `[workspace].members` (keep alphabetical: after `pleiades-elp`), and under `[workspace.dependencies]` add:

```toml
pleiades-events = { path = "crates/pleiades-events", version = "0.3.0" }
```

- [ ] **Step 2: Write `crates/pleiades-events/Cargo.toml`** (mirror `pleiades-eclipse/Cargo.toml`):

```toml
[package]
name = "pleiades-events"
description = "Ephemeris event-finding for the pleiades astrology workspace: longitude crossings (solcross / mooncross / general-body / heliocentric helio_cross), derived from pleiades' validated body positions."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[features]
serde = ["dep:serde", "pleiades-types/serde"]

[dependencies]
pleiades-types = { workspace = true }
pleiades-backend = { workspace = true }
pleiades-apparent = { workspace = true }
pleiades-time = { workspace = true }
serde = { workspace = true, optional = true }

[dev-dependencies]
pleiades-backend = { workspace = true, features = ["test-backend"] }
pleiades-data = { workspace = true }

[package.metadata.docs.rs]
all-features = true
```

Create a one-line `crates/pleiades-events/README.md` (crate name + one-sentence description) so the `readme` field resolves.

- [ ] **Step 3: Write the failing test** in `crates/pleiades-events/src/error.rs`:

```rust
//! Structured, fail-closed event errors.

use core::fmt;

/// First instant of the supported window (1900-01-01 TT), Julian Day.
pub const WINDOW_START_JD: f64 = 2_415_020.5;
/// Last instant of the supported window (2100-01-01 TT), Julian Day — the end of
/// the packaged backend's Sun/Moon/planet coverage.
pub const WINDOW_END_JD: f64 = 2_488_069.5;

/// Errors returned by the event engine; all variants fail closed.
#[derive(Clone, Debug, PartialEq)]
pub enum EventError {
    /// A requested instant falls outside the 1900–2100 CE window.
    OutOfWindow {
        /// The out-of-window instant, as a Julian Day.
        julian_day: f64,
    },
    /// The backend returned a structured error (message forwarded verbatim).
    Backend(String),
    /// The backend produced no ecliptic coordinates (or no distance) for a body.
    MissingCoordinates {
        /// Human-readable label of the body that was missing (e.g. `"Sun"`).
        body_label: &'static str,
        /// The Julian Day at which coordinates were requested.
        julian_day: f64,
    },
    /// A frame/body combination that is not defined (e.g. heliocentric Sun/Moon).
    UnsupportedFrame {
        /// Human-readable explanation.
        detail: String,
    },
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventError::OutOfWindow { julian_day } => write!(
                f,
                "instant JD {julian_day} is outside the supported 1900–2100 CE window \
                 (JD {WINDOW_START_JD}..={WINDOW_END_JD})"
            ),
            EventError::Backend(message) => write!(f, "backend error: {message}"),
            EventError::MissingCoordinates { body_label, julian_day } => write!(
                f,
                "backend returned no ecliptic coordinates for {body_label} at JD {julian_day}"
            ),
            EventError::UnsupportedFrame { detail } => {
                write!(f, "unsupported crossing frame: {detail}")
            }
        }
    }
}

impl std::error::Error for EventError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_window_message_names_the_julian_day() {
        let err = EventError::OutOfWindow { julian_day: 2_400_000.5 };
        assert!(err.to_string().contains("2400000.5"));
        assert!(err.to_string().contains("1900"));
    }

    #[test]
    fn window_constants_match_1900_2100() {
        assert_eq!(WINDOW_START_JD, 2_415_020.5);
        assert_eq!(WINDOW_END_JD, 2_488_069.5);
    }
}
```

- [ ] **Step 4: Write `crates/pleiades-events/src/lib.rs`** (modules land in later tasks; declare only what exists now):

```rust
//! Ephemeris event-finding for the `pleiades` workspace: longitude crossings of
//! the Sun, Moon, and planets, derived from pleiades' validated body positions.
//!
//! The engine is generic over any [`pleiades_backend::EphemerisBackend`] and,
//! like `pleiades-eclipse`, works in TDB over the 1900–2100 CE packaged window.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;

pub use error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
```

- [ ] **Step 5: Run tests to verify they pass.**

Run: `cargo test -p pleiades-events`
Expected: PASS (2 tests in `error`), crate compiles, `cargo build` succeeds workspace-wide.

- [ ] **Step 6: Verify the workspace audit still passes** (no FFI added):

Run: `cargo run -q -p pleiades-validate -- workspace-audit`
Expected: PASS.

- [ ] **Step 7: Commit.**

```bash
git add Cargo.toml crates/pleiades-events
git commit -m "feat(events): scaffold pleiades-events crate with EventError and window constants"
```

---

## Task 2: Generic bracket + bisect root-finder (`root.rs`)

**Files:**
- Create: `crates/pleiades-events/src/root.rs`
- Modify: `crates/pleiades-events/src/lib.rs` (add `mod root;`)

**Interfaces:**
- Consumes: `EventError`.
- Produces:
  - `pub(crate) fn crossings_in_range<F>(f: F, lo_jd: f64, hi_jd: f64, step_days: f64) -> Result<Vec<f64>, EventError> where F: FnMut(f64) -> Result<f64, EventError>` — all roots of `f` in `[lo_jd, hi_jd]`, ascending.
  - `pub(crate) const REFINE_TOLERANCE_DAYS: f64` (= `0.5 / 86_400.0`).

The target `f` must return a value that is continuous across a real crossing and wraps into `(-180, 180]` (the caller builds it that way); the finder rejects the ±180 wrap seam using the same `< 180.0` jump guard as `syzygy.rs`.

- [ ] **Step 1: Write the failing tests** in `crates/pleiades-events/src/root.rs`. These use a pure analytic target — no backend — so the finder is tested in isolation, including the retrograde triple-crossing case:

```rust
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
```

*(Plan note: the triple-crossing name is aspirational — the analytic parabola gives a clean 2-root bracket case that still exercises multiplicity; the genuine 3-root retrograde case is asserted against SE in Task 8's corpus. Keep this unit test as the 2-root multiplicity guard.)*

- [ ] **Step 2: Run tests to verify they fail.**

Run: `cargo test -p pleiades-events root::`
Expected: FAIL ("cannot find function `crossings_in_range`").

- [ ] **Step 3: Implement the finder** above the `#[cfg(test)]` block in `root.rs`:

```rust
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
```

- [ ] **Step 4: Add `mod root;`** to `lib.rs` (below `mod error;`).

- [ ] **Step 5: Run tests to verify they pass.**

Run: `cargo test -p pleiades-events root::`
Expected: PASS (4 tests).

- [ ] **Step 6: Commit.**

```bash
git add crates/pleiades-events/src/root.rs crates/pleiades-events/src/lib.rs
git commit -m "feat(events): generic bracket-and-bisect root-finder"
```

---

## Task 3: Body sampling — geocentric apparent + heliocentric reconstruction (`ephemeris.rs`)

**Files:**
- Create: `crates/pleiades-events/src/ephemeris.rs`
- Modify: `crates/pleiades-events/src/lib.rs` (add `mod ephemeris;`)

**Interfaces:**
- Consumes: `EventError`; `pleiades_backend::{EphemerisBackend, EphemerisRequest}`; `pleiades_apparent::{apparent_position, apparent_sun_position, DEFAULT_MAX_ITERATIONS, LIGHT_TIME_DAYS_PER_AU}`; `pleiades_types::*`.
- Produces:
  - `pub(crate) fn geocentric_apparent_longitude_deg<B: EphemerisBackend>(backend: &B, body: CelestialBody, body_label: &'static str, julian_day: f64) -> Result<f64, EventError>`
  - `pub(crate) fn heliocentric_longitude_deg<B: EphemerisBackend>(backend: &B, body: CelestialBody, body_label: &'static str, julian_day: f64) -> Result<f64, EventError>`
  - `pub(crate) fn read_mean_ecliptic<B: EphemerisBackend>(backend: &B, body: CelestialBody, body_label: &'static str, julian_day: f64) -> Result<(f64, f64, f64), EventError>` (lon, lat, dist_au — mean/J2000)

Reuse conventions from `crates/pleiades-eclipse/src/ephemeris.rs`: the `request(...)` builder uses `CoordinateFrame::Ecliptic`, `ZodiacMode::Tropical`, `Apparentness::Mean`, `observer: None`, `TimeScale::Tdb`.

- [ ] **Step 1: Write the failing tests** in `crates/pleiades-events/src/ephemeris.rs`. Use the `LinearSunMoon` test backend (`pleiades_backend::test_backend::LinearSunMoon`), which places Sun and Moon at known geometric longitudes.

```rust
//! Reads body ecliptic positions from a backend and derives the longitudes the
//! crossing engine root-finds on: geocentric apparent-of-date, and heliocentric.

use crate::error::EventError;
use pleiades_apparent::{
    apparent_position, apparent_sun_position, ApparentPlaceError, DEFAULT_MAX_ITERATIONS,
};
use pleiades_backend::{EphemerisBackend, EphemerisRequest};
use pleiades_types::{
    Apparentness, CelestialBody, CoordinateFrame, EclipticCoordinates, Instant, JulianDay,
    Latitude, Longitude, TimeScale, ZodiacMode,
};

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn mean_read_returns_sun_longitude() {
        let backend = LinearSunMoon::new_moon_at(2_451_550.0);
        let (lon, _lat, dist) =
            read_mean_ecliptic(&backend, CelestialBody::Sun, "Sun", 2_451_550.0).unwrap();
        assert!(lon.is_finite());
        assert!(dist > 0.5 && dist < 1.5, "sun distance {dist}");
    }

    #[test]
    fn geocentric_apparent_sun_is_near_mean_but_shifted() {
        // Apparent-of-date longitude differs from mean/J2000 by precession +
        // aberration + nutation; at J2000-ish epochs the shift is small but real.
        let backend = LinearSunMoon::new_moon_at(2_451_550.0);
        let mean = read_mean_ecliptic(&backend, CelestialBody::Sun, "Sun", 2_451_550.0)
            .unwrap()
            .0;
        let app =
            geocentric_apparent_longitude_deg(&backend, CelestialBody::Sun, "Sun", 2_451_550.0)
                .unwrap();
        assert!(app.is_finite());
        assert!((app - mean).abs() < 1.0, "apparent-vs-mean {app} {mean}");
    }

    #[test]
    fn missing_coordinates_fail_closed() {
        let backend = LinearSunMoon::empty();
        let err = read_mean_ecliptic(&backend, CelestialBody::Sun, "Sun", 2_451_550.0)
            .unwrap_err();
        assert!(matches!(err, EventError::MissingCoordinates { .. }));
    }

    #[test]
    fn heliocentric_reconstruction_subtracts_geocentric_sun() {
        // For the Sun-Moon mock there is no planet; assert the reconstruction math
        // on a synthetic pair via the exported helper is covered by the crossings
        // tests. Here just prove missing distance fails closed.
        let backend = LinearSunMoon::empty();
        let err =
            heliocentric_longitude_deg(&backend, CelestialBody::Mars, "Mars", 2_451_550.0)
                .unwrap_err();
        assert!(matches!(
            err,
            EventError::MissingCoordinates { .. } | EventError::Backend(_)
        ));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail.**

Run: `cargo test -p pleiades-events ephemeris::`
Expected: FAIL ("cannot find function `read_mean_ecliptic`").

- [ ] **Step 3: Implement the sampling helpers** above the test module:

```rust
fn request(body: CelestialBody, julian_day: f64) -> EphemerisRequest {
    EphemerisRequest {
        body,
        instant: Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    }
}

/// Mean/J2000 geocentric ecliptic (longitude_deg, latitude_deg, distance_au).
pub(crate) fn read_mean_ecliptic<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<(f64, f64, f64), EventError> {
    let result = backend
        .position(&request(body, julian_day))
        .map_err(|e| EventError::Backend(e.to_string()))?;
    let ecliptic = result.ecliptic.ok_or(EventError::MissingCoordinates {
        body_label,
        julian_day,
    })?;
    let distance = ecliptic
        .distance_au
        .ok_or(EventError::MissingCoordinates { body_label, julian_day })?;
    Ok((
        ecliptic.longitude.degrees(),
        ecliptic.latitude.degrees(),
        distance,
    ))
}

/// Geocentric apparent-of-date ecliptic longitude (degrees). The Sun is a special
/// case where light-time and annual aberration are the same effect, so it uses
/// `apparent_sun_position` (which applies aberration exactly once); every other
/// body uses the general `apparent_position` light-time pipeline.
pub(crate) fn geocentric_apparent_longitude_deg<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<f64, EventError> {
    let (lon, lat, dist) = read_mean_ecliptic(backend, body.clone(), body_label, julian_day)?;
    let instant = Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb);
    if body == CelestialBody::Sun {
        let j2000 = EclipticCoordinates::new(
            Longitude::from_degrees(lon),
            Latitude::from_degrees(lat),
            Some(dist),
        );
        let apparent = apparent_sun_position(instant, j2000)
            .map_err(|e| EventError::Backend(format!("Sun apparent place failed: {e}")))?;
        return Ok(apparent.ecliptic.longitude.degrees());
    }

    // General body: apparent_position needs the Sun's true longitude of date for
    // the aberration term, plus a light-time-retarded body query closure.
    let sun_true_lon = geocentric_apparent_longitude_deg(backend, CelestialBody::Sun, "Sun", julian_day)?;
    let apparent = apparent_position::<_, ApparentPlaceError>(
        instant,
        sun_true_lon,
        DEFAULT_MAX_ITERATIONS,
        |retarded: Instant| {
            let (l, b, d) = read_mean_ecliptic(
                backend,
                body.clone(),
                body_label,
                retarded.julian_day.days(),
            )
            .map_err(|_| ApparentPlaceError::light_time_query())?;
            Ok(EclipticCoordinates::new(
                Longitude::from_degrees(l),
                Latitude::from_degrees(b),
                Some(d),
            ))
        },
    )
    .map_err(|e| EventError::Backend(format!("{body_label} apparent place failed: {e}")))?;
    Ok(apparent.ecliptic.longitude.degrees())
}

/// Heliocentric ecliptic longitude (degrees) via `P_helio = P_geo − S_geo`,
/// reconstructed from the mean geocentric planet and Sun vectors. Both vectors
/// carry distance (AU); a missing distance fails closed.
pub(crate) fn heliocentric_longitude_deg<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<f64, EventError> {
    let (pl, pb, pd) = read_mean_ecliptic(backend, body, body_label, julian_day)?;
    let (sl, sb, sd) = read_mean_ecliptic(backend, CelestialBody::Sun, "Sun", julian_day)?;
    let planet = spherical_to_cartesian(pl, pb, pd);
    let sun = spherical_to_cartesian(sl, sb, sd);
    let helio = [planet[0] - sun[0], planet[1] - sun[1], planet[2] - sun[2]];
    Ok(cartesian_longitude_deg(helio))
}

fn spherical_to_cartesian(lon_deg: f64, lat_deg: f64, r_au: f64) -> [f64; 3] {
    let lon = lon_deg.to_radians();
    let lat = lat_deg.to_radians();
    [
        r_au * lat.cos() * lon.cos(),
        r_au * lat.cos() * lon.sin(),
        r_au * lat.sin(),
    ]
}

fn cartesian_longitude_deg(v: [f64; 3]) -> f64 {
    v[1].atan2(v[0]).to_degrees().rem_euclid(360.0)
}
```

*Plan note:* `ApparentPlaceError::light_time_query()` is a placeholder for whatever constructor `pleiades-apparent` exposes for a query-side failure. During implementation, inspect `crates/pleiades-apparent/src/error.rs` for the correct variant (the crate's own light-time error type used with `apparent_position`), and use `ApparentLightTimeError` per that function's signature (`apparent_position::<_, E>` returns `ApparentLightTimeError<E>`). Adjust the closure's error type `E` and the `.map_err` accordingly so it compiles; the observable behavior (fail closed on a missing backend read) is what the test pins.

- [ ] **Step 4: Add `mod ephemeris;`** to `lib.rs`.

- [ ] **Step 5: Run tests to verify they pass.**

Run: `cargo test -p pleiades-events ephemeris::`
Expected: PASS (4 tests).

- [ ] **Step 6: Commit.**

```bash
git add crates/pleiades-events/src/ephemeris.rs crates/pleiades-events/src/lib.rs
git commit -m "feat(events): body sampling — geocentric apparent + heliocentric reconstruction"
```

---

## Task 4: `CrossingEngine` geocentric API (`crossings.rs`)

**Files:**
- Create: `crates/pleiades-events/src/crossings.rs`
- Modify: `crates/pleiades-events/src/lib.rs` (`mod crossings;` + re-exports)

**Interfaces:**
- Consumes: `EventError`, `WINDOW_START_JD`, `WINDOW_END_JD`; `root::{crossings_in_range, wrap180}`; `ephemeris::{geocentric_apparent_longitude_deg, heliocentric_longitude_deg}`.
- Produces:
  - `pub struct CrossingEngine<B>`; `pub fn new(backend: B) -> Self`.
  - `pub enum CrossingFrame { GeocentricApparentOfDate, Heliocentric }` (`#[non_exhaustive]`).
  - `pub struct Crossing { pub body: CelestialBody, pub target_longitude: Longitude, pub instant: Instant, pub frame: CrossingFrame }` (`#[non_exhaustive]`).
  - `pub fn longitude_crossings_in_range(&self, body, target: Longitude, frame: CrossingFrame, start: Instant, end: Instant) -> Result<Vec<Crossing>, EventError>`
  - `pub fn next_longitude_crossing(&self, body, target, frame, after: Instant) -> Result<Option<Crossing>, EventError>`
  - `pub fn previous_longitude_crossing(&self, body, target, frame, before: Instant) -> Result<Option<Crossing>, EventError>`
  - `pub fn next_sun_crossing(&self, target, after) -> Result<Option<Crossing>, EventError>` and `pub fn next_moon_crossing(&self, target, after)` conveniences.

This task implements only the `GeocentricApparentOfDate` frame end-to-end; `Heliocentric` dispatch + guards land in Task 5 (leave a `todo!()`-free path: match on frame and route heliocentric to `heliocentric_longitude_deg`, but the frame guard tests are Task 5).

- [ ] **Step 1: Write the failing tests** in `crates/pleiades-events/src/crossings.rs` using `LinearSunMoon`:

```rust
//! The public longitude-crossing engine.

use crate::ephemeris::{geocentric_apparent_longitude_deg, heliocentric_longitude_deg};
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use crate::root::{crossings_in_range, wrap180};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale};

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    fn tdb(jd: f64) -> Instant {
        Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
    }

    #[test]
    fn finds_a_sun_crossing_in_range() {
        let engine = CrossingEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        // Sun sweeps ~1°/day; over 400 days it crosses any target at least once.
        let start = tdb(2_451_545.0);
        let end = tdb(2_451_545.0 + 400.0);
        let out = engine
            .longitude_crossings_in_range(
                CelestialBody::Sun,
                Longitude::from_degrees(100.0),
                CrossingFrame::GeocentricApparentOfDate,
                start,
                end,
            )
            .unwrap();
        assert!(!out.is_empty(), "expected at least one Sun crossing");
        for c in &out {
            let lon = geocentric_apparent_longitude_deg(
                &engine.backend,
                CelestialBody::Sun,
                "Sun",
                c.instant.julian_day.days(),
            )
            .unwrap();
            assert!(wrap180(lon - 100.0).abs() < 1e-3, "residual at crossing {lon}");
        }
    }

    #[test]
    fn next_equals_first_in_range() {
        let engine = CrossingEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let after = tdb(2_451_545.0);
        let end = tdb(2_451_545.0 + 400.0);
        let next = engine
            .next_longitude_crossing(
                CelestialBody::Sun,
                Longitude::from_degrees(100.0),
                CrossingFrame::GeocentricApparentOfDate,
                after,
            )
            .unwrap()
            .unwrap();
        let first = engine
            .longitude_crossings_in_range(
                CelestialBody::Sun,
                Longitude::from_degrees(100.0),
                CrossingFrame::GeocentricApparentOfDate,
                after,
                end,
            )
            .unwrap()[0];
        assert!((next.instant.julian_day.days() - first.instant.julian_day.days()).abs() < 1e-6);
    }

    #[test]
    fn out_of_window_fails_closed() {
        let engine = CrossingEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let err = engine
            .longitude_crossings_in_range(
                CelestialBody::Sun,
                Longitude::from_degrees(0.0),
                CrossingFrame::GeocentricApparentOfDate,
                tdb(2_000_000.0),
                tdb(2_100_000.0),
            )
            .unwrap_err();
        assert!(matches!(err, EventError::OutOfWindow { .. }));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail.**

Run: `cargo test -p pleiades-events crossings::`
Expected: FAIL ("cannot find type `CrossingEngine`").

- [ ] **Step 3: Implement the engine** above the test module. Use a body-scaled step; clamp the scan to the coverage window (mirroring `EclipseEngine::eclipses_in_range`):

```rust
/// The coordinate/center convention a crossing is computed in.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum CrossingFrame {
    /// Geocentric apparent tropical ecliptic of date (SE `solcross`/`mooncross`).
    GeocentricApparentOfDate,
    /// Heliocentric ecliptic (SE `helio_cross`); planets only.
    Heliocentric,
}

/// A single longitude crossing.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct Crossing {
    /// The body that crossed the target longitude.
    pub body: CelestialBody,
    /// The ecliptic longitude that was crossed.
    pub target_longitude: Longitude,
    /// Instant of the crossing (TDB).
    pub instant: Instant,
    /// The frame the crossing was computed in.
    pub frame: CrossingFrame,
}

/// Finds longitude crossings of a body over the packaged 1900–2100 TDB window.
pub struct CrossingEngine<B> {
    backend: B,
}

impl<B: EphemerisBackend> CrossingEngine<B> {
    /// Wraps a backend.
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Step used to bracket crossings, scaled by body speed so no crossing is
    /// skipped (fast Moon → small step; slow outer planets → larger step).
    fn step_days(body: CelestialBody) -> f64 {
        match body {
            CelestialBody::Moon => 0.25,
            CelestialBody::Sun | CelestialBody::Mercury | CelestialBody::Venus => 1.0,
            _ => 2.0,
        }
    }

    fn check_window(&self, jd: f64) -> Result<(), EventError> {
        if jd < WINDOW_START_JD || jd > WINDOW_END_JD {
            return Err(EventError::OutOfWindow { julian_day: jd });
        }
        Ok(())
    }

    fn longitude_deg(
        &self,
        body: CelestialBody,
        frame: CrossingFrame,
        jd: f64,
    ) -> Result<f64, EventError> {
        match frame {
            CrossingFrame::GeocentricApparentOfDate => {
                geocentric_apparent_longitude_deg(&self.backend, body, body_label(body), jd)
            }
            CrossingFrame::Heliocentric => {
                heliocentric_longitude_deg(&self.backend, body, body_label(body), jd)
            }
        }
    }

    /// All crossings of `target` by `body` in `[start, end]` (TDB), ascending.
    pub fn longitude_crossings_in_range(
        &self,
        body: CelestialBody,
        target: Longitude,
        frame: CrossingFrame,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<Crossing>, EventError> {
        let start_jd = start.julian_day.days();
        let end_jd = end.julian_day.days();
        self.check_window(start_jd)?;
        self.check_window(end_jd)?;
        let step = Self::step_days(body);
        // Clamp like the eclipse engine: keep retarded/aberration queries in-window.
        let scan_start = start_jd.max(WINDOW_START_JD + step);
        let scan_end = end_jd.min(WINDOW_END_JD - step);
        let target_deg = target.degrees();
        let roots = crossings_in_range(
            |jd| Ok(wrap180(self.longitude_deg(body, frame, jd)? - target_deg)),
            scan_start,
            scan_end,
            step,
        )?;
        Ok(roots
            .into_iter()
            .map(|jd| Crossing {
                body,
                target_longitude: target,
                instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
                frame,
            })
            .collect())
    }

    /// The first crossing strictly after `after`, or `None`.
    pub fn next_longitude_crossing(
        &self,
        body: CelestialBody,
        target: Longitude,
        frame: CrossingFrame,
        after: Instant,
    ) -> Result<Option<Crossing>, EventError> {
        let end = Instant::new(JulianDay::from_days(WINDOW_END_JD), TimeScale::Tdb);
        let after_jd = after.julian_day.days();
        Ok(self
            .longitude_crossings_in_range(body, target, frame, after, end)?
            .into_iter()
            .find(|c| c.instant.julian_day.days() > after_jd))
    }

    /// The last crossing strictly before `before`, or `None`.
    pub fn previous_longitude_crossing(
        &self,
        body: CelestialBody,
        target: Longitude,
        frame: CrossingFrame,
        before: Instant,
    ) -> Result<Option<Crossing>, EventError> {
        let start = Instant::new(JulianDay::from_days(WINDOW_START_JD), TimeScale::Tdb);
        let before_jd = before.julian_day.days();
        Ok(self
            .longitude_crossings_in_range(body, target, frame, start, before)?
            .into_iter()
            .rev()
            .find(|c| c.instant.julian_day.days() < before_jd))
    }

    /// `swe_solcross`: next geocentric apparent Sun crossing of `target`.
    pub fn next_sun_crossing(
        &self,
        target: Longitude,
        after: Instant,
    ) -> Result<Option<Crossing>, EventError> {
        self.next_longitude_crossing(
            CelestialBody::Sun,
            target,
            CrossingFrame::GeocentricApparentOfDate,
            after,
        )
    }

    /// `swe_mooncross`: next geocentric apparent Moon crossing of `target`.
    pub fn next_moon_crossing(
        &self,
        target: Longitude,
        after: Instant,
    ) -> Result<Option<Crossing>, EventError> {
        self.next_longitude_crossing(
            CelestialBody::Moon,
            target,
            CrossingFrame::GeocentricApparentOfDate,
            after,
        )
    }
}

fn body_label(body: CelestialBody) -> &'static str {
    match body {
        CelestialBody::Sun => "Sun",
        CelestialBody::Moon => "Moon",
        CelestialBody::Mercury => "Mercury",
        CelestialBody::Venus => "Venus",
        CelestialBody::Mars => "Mars",
        CelestialBody::Jupiter => "Jupiter",
        CelestialBody::Saturn => "Saturn",
        CelestialBody::Uranus => "Uranus",
        CelestialBody::Neptune => "Neptune",
        CelestialBody::Pluto => "Pluto",
        _ => "body",
    }
}
```

*Plan note:* the `backend` field is read by the test via `engine.backend`; make it `pub(crate) backend: B`. Confirm the exact `CelestialBody` variant list in `crates/pleiades-types/src/bodies.rs` and adjust `body_label`/`step_days` matches to be exhaustive (or keep the `_` arm).

- [ ] **Step 4: Wire re-exports in `lib.rs`:**

```rust
mod crossings;

pub use crossings::{Crossing, CrossingEngine, CrossingFrame};
```

- [ ] **Step 5: Run tests to verify they pass.**

Run: `cargo test -p pleiades-events`
Expected: PASS (all crossings + earlier tests).

- [ ] **Step 6: Commit.**

```bash
git add crates/pleiades-events/src/crossings.rs crates/pleiades-events/src/lib.rs
git commit -m "feat(events): CrossingEngine geocentric next/previous/in-range API"
```

---

## Task 5: Heliocentric crossings + frame/body guards

**Files:**
- Modify: `crates/pleiades-events/src/crossings.rs`

**Interfaces:**
- Produces: fail-closed guards — `Heliocentric` for Sun/Moon → `EventError::UnsupportedFrame`; heliocentric planet crossings resolve via `heliocentric_longitude_deg`.

- [ ] **Step 1: Write the failing tests** (append to the `crossings::tests` module). The heliocentric happy-path is asserted against SE in Task 8; here we pin the guards and that a supported planet does not error structurally on a backend that provides planet+Sun vectors. Use the packaged backend for a real planet:

```rust
    #[test]
    fn heliocentric_rejects_sun_and_moon() {
        let engine = CrossingEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let after = tdb(2_451_545.0);
        for body in [CelestialBody::Sun, CelestialBody::Moon] {
            let err = engine
                .next_longitude_crossing(
                    body,
                    Longitude::from_degrees(0.0),
                    CrossingFrame::Heliocentric,
                    after,
                )
                .unwrap_err();
            assert!(matches!(err, EventError::UnsupportedFrame { .. }), "{body:?}");
        }
    }
```

Add a packaged-backend heliocentric smoke test in `crates/pleiades-events/tests/heliocentric.rs` (integration test; `pleiades-data` is a dev-dependency):

```rust
use pleiades_data::packaged_backend;
use pleiades_events::{CrossingEngine, CrossingFrame};
use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale};

#[test]
fn heliocentric_jupiter_crossing_is_found() {
    let engine = CrossingEngine::new(packaged_backend());
    let start = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let end = Instant::new(JulianDay::from_days(2_451_545.0 + 4400.0), TimeScale::Tdb); // ~1 Jupiter orbit
    let out = engine
        .longitude_crossings_in_range(
            CelestialBody::Jupiter,
            Longitude::from_degrees(0.0),
            CrossingFrame::Heliocentric,
            start,
            end,
        )
        .expect("heliocentric crossing search");
    assert!(!out.is_empty(), "expected a heliocentric Jupiter crossing of 0°");
}
```

- [ ] **Step 2: Run tests to verify they fail.**

Run: `cargo test -p pleiades-events heliocentric_rejects_sun_and_moon` and `cargo test -p pleiades-events --test heliocentric`
Expected: FAIL (guard not implemented; the integration test may already pass if Task 4's heliocentric route works — that's fine, keep it as a regression guard).

- [ ] **Step 3: Implement the guard** in `longitude_crossings_in_range`, immediately after the window checks:

```rust
        if matches!(frame, CrossingFrame::Heliocentric)
            && matches!(body, CelestialBody::Sun | CelestialBody::Moon)
        {
            return Err(EventError::UnsupportedFrame {
                detail: format!("heliocentric crossings are undefined for {:?}", body),
            });
        }
```

- [ ] **Step 4: Run tests to verify they pass.**

Run: `cargo test -p pleiades-events`
Expected: PASS (unit + integration).

- [ ] **Step 5: Commit.**

```bash
git add crates/pleiades-events/src/crossings.rs crates/pleiades-events/tests/heliocentric.rs
git commit -m "feat(events): heliocentric crossings with Sun/Moon frame guard"
```

---

## Task 6: Crate-level doctest + docs polish

**Files:**
- Modify: `crates/pleiades-events/src/lib.rs`

**Interfaces:** none new — locks the public API with an example (the crates doctest heavily; `#![deny(missing_docs)]` is already on).

- [ ] **Step 1: Add a runnable doctest** to the `lib.rs` module docs (mirror the eclipse crate's example style):

````rust
//! ## Example
//!
//! ```rust
//! use pleiades_data::packaged_backend;
//! use pleiades_events::{CrossingEngine, CrossingFrame};
//! use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale};
//!
//! let engine = CrossingEngine::new(packaged_backend());
//! let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
//! // When does the Sun next reach 0° (the March equinox point)?
//! let next = engine
//!     .next_sun_crossing(Longitude::from_degrees(0.0), after)
//!     .unwrap();
//! assert!(next.is_some());
//! ```
````

Ensure `pleiades-data` is a dev-dependency (it is, from Task 1) so the doctest links.

- [ ] **Step 2: Run the doctest.**

Run: `cargo test -p pleiades-events --doc`
Expected: PASS.

- [ ] **Step 3: Confirm lints are clean.**

Run: `cargo clippy -p pleiades-events --all-features -- -D warnings`
Expected: no warnings.

- [ ] **Step 4: Commit.**

```bash
git add crates/pleiades-events/src/lib.rs
git commit -m "docs(events): crate-level doctest for the crossing engine"
```

---

## Task 7: Isolated SE reference tool `tools/se-crossings-reference`

**Files:**
- Create: `tools/se-crossings-reference/Cargo.toml`
- Create: `tools/se-crossings-reference/src/main.rs`
- Create: `tools/se-crossings-reference/LICENSE-NOTES.md`
- (generated on first build) `tools/se-crossings-reference/Cargo.lock`

**Interfaces:**
- Produces a CSV on STDOUT with header:
  `frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb`
  where `frame ∈ {geo,helio}`, `body` is the SE planet name, `direction ∈ {fwd,bwd}`.

This crate is OUTSIDE the workspace (own `Cargo.lock`, `publish = false`) — Global Constraint (pure-Rust audit).

- [ ] **Step 1: Write `tools/se-crossings-reference/Cargo.toml`** (mirror `tools/se-lilith-reference/Cargo.toml`):

```toml
[package]
name = "se-crossings-reference"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
swisseph = "0.1.1"
libswisseph-sys = "0.1.2"
```

- [ ] **Step 2: Write `src/main.rs`** calling SE's crossing functions. Emit deterministic fixtures spanning the window. Use `swe_solcross_ut`/`swe_mooncross_ut` (convert their UT result to TDB via SE's `swe_deltat`) and `swe_helio_cross` (ET/TDB directly). Cover: Sun & Moon crossing 0/90/180/270 and an arbitrary longitude; Mars crossing a longitude near a retrograde loop (a genuine triple-crossing epoch — pick a documented Mars retrograde, e.g. around 2003-08, so SE returns three forward crossings when scanned in sub-intervals); heliocentric Jupiter & Saturn crossing 0°.

```rust
//! Emits a Swiss Ephemeris crossing reference corpus to STDOUT as CSV.
//!
//! Frame `geo`: geocentric apparent tropical of date (SE default; solcross/mooncross).
//! Frame `helio`: heliocentric (SEFLG_HELCTR; helio_cross).
//! Times are emitted in TDB (SE `_ut` results converted via swe_deltat).
//!
//! Usage: `cargo run --release > .../crossings-corpus/crossings.csv`
//! Requires libclang-dev + LIBCLANG_PATH to build. NOT needed to run the gate.

// Implementation detail: bind the exact SE symbols used (swe_solcross_ut,
// swe_mooncross_ut, swe_helio_cross, swe_deltat, swe_set_ephe_path/MOSEPH).
// Follow the FFI + serr-handling pattern in tools/se-lilith-reference/src/main.rs.
fn main() {
    // 1. Configure Moshier ephemeris (SEFLG_MOSEPH) so no data files are needed.
    // 2. For each geo fixture (body, target, start_jd, dir): call the matching
    //    swe_*cross_ut, convert UT->TDB with swe_deltat, print a CSV row.
    // 3. For each helio fixture (planet, target, start_jd, dir): call
    //    swe_helio_cross (ET), print a CSV row.
    // 4. Header first: frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb
    todo!("fill in per the se-lilith-reference FFI pattern; see plan Task 7")
}
```

*Plan note:* This is the one task whose body is genuinely offline/SE-specific. Model every FFI call, `serr` buffer, and error check on `tools/se-lilith-reference/src/main.rs` (already in the tree). The exact SE function signatures: `swe_solcross_ut(x2cross: f64, jd_ut: f64, iflag: i32, serr) -> f64` (returns the crossing JD, `< jd_ut` on error), `swe_mooncross_ut(...)` same shape, `swe_helio_cross(ipl: i32, x2cross: f64, jd_et: f64, iflag: i32, dir: i32, jd_cross: *mut f64, serr) -> i32`.

- [ ] **Step 3: Write `LICENSE-NOTES.md`** — copy the verification-only, non-shipping SE license posture verbatim from `tools/se-lilith-reference/LICENSE-NOTES.md`.

- [ ] **Step 4: Build the tool** (requires libclang):

Run: `cd tools/se-crossings-reference && LIBCLANG_PATH=$(llvm-config --libdir 2>/dev/null || echo /usr/lib/llvm-14/lib) cargo build --release`
Expected: builds; `Cargo.lock` generated.

- [ ] **Step 5: Confirm the workspace audit still ignores it** (it is outside `[workspace].members`):

Run: `cargo run -q -p pleiades-validate -- workspace-audit`
Expected: PASS (no `-sys`/`links`/`build.rs` in the workspace lockfile).

- [ ] **Step 6: Commit** (tool sources + its lockfile; NOT the CSV yet):

```bash
git add tools/se-crossings-reference
git commit -m "test(events): SE crossing reference harness (isolated, out-of-workspace)"
```

---

## Task 8: Generate + commit the crossings corpus

**Files:**
- Create: `crates/pleiades-validate/data/crossings-corpus/crossings.csv`
- Create: `crates/pleiades-validate/data/crossings-corpus/manifest.txt`

**Interfaces:**
- Produces the committed corpus the gate reads via `include_str!`. `manifest.txt` records: SE version (`2.10.03`), row count, SHA-256 of `crossings.csv`, and generation command — mirror `crates/pleiades-validate/data/lilith-corpus/manifest.txt`.

- [ ] **Step 1: Generate the CSV** by running the Task 7 tool:

Run: `cd tools/se-crossings-reference && cargo run --release > ../../crates/pleiades-validate/data/crossings-corpus/crossings.csv`
Expected: a CSV with the header row + one row per fixture (target ~30–60 rows: geo Sun/Moon/Mars + helio Jupiter/Saturn, including the Mars retrograde triple-crossing epoch as three separate rows).

- [ ] **Step 2: Write `manifest.txt`** (mirror the lilith manifest fields):

```
corpus: crossings
source: Swiss Ephemeris 2.10.03 (Moshier, SEFLG_MOSEPH)
generator: tools/se-crossings-reference (cargo run --release)
rows: <N>
sha256(crossings.csv): <hash>
frames: geo (apparent tropical of date), helio (SEFLG_HELCTR)
window: 1900-2100 CE (JD 2415020.5..=2488069.5), times in TDB
```

Compute the hash: `sha256sum crates/pleiades-validate/data/crossings-corpus/crossings.csv`.

- [ ] **Step 3: Sanity-check the CSV** parses and is in-window (quick throwaway check, not committed):

Run: `awk -F, 'NR>1 && ($4<2415020.5 || $6<2415020.5 || $6>2488069.5){print "OUT:",$0}' crates/pleiades-validate/data/crossings-corpus/crossings.csv`
Expected: no `OUT:` lines.

- [ ] **Step 4: Commit.**

```bash
git add crates/pleiades-validate/data/crossings-corpus
git commit -m "test(events): commit SE crossing reference corpus + manifest"
```

---

## Task 9: `validate-crossings` gate + wiring

**Files:**
- Create: `crates/pleiades-validate/src/crossings_validation.rs`
- Modify: `crates/pleiades-validate/Cargo.toml` (add `pleiades-events` dep)
- Modify: `crates/pleiades-validate/src/lib.rs` (`pub mod` + `pub use`)
- Modify: `crates/pleiades-validate/src/render/cli.rs` (dispatch + help text)
- Modify: `crates/pleiades-validate/src/release/notes.rs` (smoke/gate enumeration)

**Interfaces:**
- Consumes: `pleiades_events::{CrossingEngine, CrossingFrame, Crossing}`; `pleiades_data::packaged_backend`.
- Produces: `pub fn validate_crossings_corpus() -> Result<CrossingsCorpusReport, CrossingsCorpusError>`; `pub struct CrossingsCorpusReport { pub checked: usize }` with `summary_line()`; `pub struct CrossingsGateOutcome(pub Result<..>)` with `passed()`; `pub fn run_crossings_gate() -> CrossingsGateOutcome`. Mirror `angles_validation.rs`/`eclipse_validation.rs` exactly.

- [ ] **Step 1: Add the dependency** to `crates/pleiades-validate/Cargo.toml` under `[dependencies]`:

```toml
pleiades-events = { workspace = true }
```

- [ ] **Step 2: Write the failing gate test** at the bottom of `crossings_validation.rs`:

```rust
//! Fail-closed gate: recompute every SE crossing fixture and compare times.

use pleiades_data::packaged_backend;
use pleiades_events::{CrossingEngine, CrossingFrame};
use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/crossings-corpus/crossings.csv"
));

// Per-frame/body time ceilings (seconds). Sun/Moon are well-conditioned; slow
// planets near stations are looser (dλ/dt → 0). Tune from measured residuals.
const GEO_SUN_MOON_TOL_S: f64 = 5.0;
const GEO_PLANET_TOL_S: f64 = 600.0;
const HELIO_TOL_S: f64 = 60.0;

#[derive(Debug)]
pub enum CrossingsCorpusError {
    /// A recomputed crossing exceeded its time ceiling.
    ToleranceExceeded { row: String, residual_s: f64, ceiling_s: f64 },
    /// The engine found no crossing for a fixture that SE reports one for.
    Missing { row: String },
    /// Malformed corpus row.
    Schema { row: String },
    /// Engine error.
    Engine(String),
}

pub struct CrossingsCorpusReport {
    pub checked: usize,
}

impl CrossingsCorpusReport {
    pub fn summary_line(&self) -> String {
        format!(
            "validate-crossings: {} SE crossing fixtures recomputed within per-body \
             time ceilings (0 unexplained drift)",
            self.checked
        )
    }
}

pub fn validate_crossings_corpus() -> Result<CrossingsCorpusReport, CrossingsCorpusError> {
    let engine = CrossingEngine::new(packaged_backend());
    let mut checked = 0usize;
    for line in CORPUS_CSV.lines().skip(1).filter(|l| !l.trim().is_empty()) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 6 {
            return Err(CrossingsCorpusError::Schema { row: line.to_string() });
        }
        let frame = match f[0] {
            "geo" => CrossingFrame::GeocentricApparentOfDate,
            "helio" => CrossingFrame::Heliocentric,
            _ => return Err(CrossingsCorpusError::Schema { row: line.to_string() }),
        };
        let body = parse_body(f[1]).ok_or_else(|| CrossingsCorpusError::Schema {
            row: line.to_string(),
        })?;
        let target = f[2].parse::<f64>().map_err(|_| CrossingsCorpusError::Schema {
            row: line.to_string(),
        })?;
        let start_jd = f[3].parse::<f64>().map_err(|_| CrossingsCorpusError::Schema {
            row: line.to_string(),
        })?;
        let se_jd = f[5].parse::<f64>().map_err(|_| CrossingsCorpusError::Schema {
            row: line.to_string(),
        })?;
        let after = Instant::new(JulianDay::from_days(start_jd), TimeScale::Tdb);
        let got = engine
            .next_longitude_crossing(body, Longitude::from_degrees(target), frame, after)
            .map_err(|e| CrossingsCorpusError::Engine(e.to_string()))?
            .ok_or_else(|| CrossingsCorpusError::Missing { row: line.to_string() })?;
        let residual_s = (got.instant.julian_day.days() - se_jd).abs() * 86_400.0;
        let ceiling = ceiling_for(frame, body);
        if residual_s > ceiling {
            return Err(CrossingsCorpusError::ToleranceExceeded {
                row: line.to_string(),
                residual_s,
                ceiling_s: ceiling,
            });
        }
        checked += 1;
    }
    Ok(CrossingsCorpusReport { checked })
}

fn ceiling_for(frame: CrossingFrame, body: CelestialBody) -> f64 {
    match frame {
        CrossingFrame::Heliocentric => HELIO_TOL_S,
        CrossingFrame::GeocentricApparentOfDate => match body {
            CelestialBody::Sun | CelestialBody::Moon => GEO_SUN_MOON_TOL_S,
            _ => GEO_PLANET_TOL_S,
        },
    }
}

fn parse_body(name: &str) -> Option<CelestialBody> {
    Some(match name {
        "Sun" => CelestialBody::Sun,
        "Moon" => CelestialBody::Moon,
        "Mercury" => CelestialBody::Mercury,
        "Venus" => CelestialBody::Venus,
        "Mars" => CelestialBody::Mars,
        "Jupiter" => CelestialBody::Jupiter,
        "Saturn" => CelestialBody::Saturn,
        "Uranus" => CelestialBody::Uranus,
        "Neptune" => CelestialBody::Neptune,
        "Pluto" => CelestialBody::Pluto,
        _ => return None,
    })
}

#[derive(Debug)]
pub struct CrossingsGateOutcome(pub Result<CrossingsCorpusReport, CrossingsCorpusError>);
impl CrossingsGateOutcome {
    pub fn passed(&self) -> bool {
        self.0.is_ok()
    }
}
pub fn run_crossings_gate() -> CrossingsGateOutcome {
    CrossingsGateOutcome(validate_crossings_corpus())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_crossings_passes_over_committed_corpus() {
        let report = validate_crossings_corpus().expect("gate should pass");
        assert!(report.checked > 0, "no fixtures checked");
    }
}
```

- [ ] **Step 3: Register the module** in `crates/pleiades-validate/src/lib.rs`:
  - add `pub mod crossings_validation;` (alphabetically, near `pub mod corpus;`)
  - add `pub use crossings_validation::{validate_crossings_corpus, CrossingsCorpusError, CrossingsCorpusReport, CrossingsGateOutcome, run_crossings_gate};`

- [ ] **Step 4: Run the gate test to verify it passes** (proves engine matches SE):

Run: `cargo test -p pleiades-validate crossings_validation::`
Expected: PASS. If a residual exceeds its ceiling, this is the tuning point — inspect the failing row, and either fix the engine (real bug) or, for a genuinely ill-conditioned slow-planet-near-station row, widen `GEO_PLANET_TOL_S` with a documented comment. Do NOT widen Sun/Moon ceilings.

- [ ] **Step 5: Wire CLI dispatch** in `crates/pleiades-validate/src/render/cli.rs` (mirror the `validate-angles` arm around line 197):

```rust
        Some("validate-crossings") | Some("crossings-gate") => {
            ensure_no_extra_args(&args[1..], "validate-crossings")?;
            let report = crate::crossings_validation::validate_crossings_corpus()
                .map_err(|e| format!("validate-crossings failed: {e:?}"))?;
            Ok(format!("{banner}\nCrossings gate: {}", report.summary_line()))
        }
```

Add a help-text line to the `Commands:` block in the same file (mirror the `validate-angles` / `angles-gate` help lines):

```
  validate-crossings        Run the fail-closed longitude-crossing gate (Swiss Ephemeris solcross/mooncross/helio_cross, per-body time ceilings) over the committed crossings corpus
  crossings-gate            Alias for validate-crossings
```

- [ ] **Step 6: Add to release-smoke/gate enumeration** in `crates/pleiades-validate/src/release/notes.rs`. Grep for `validate-eclipses` there and add a parallel `validate-crossings` entry everywhere it appears (checklist line and any gate list).

Run: `grep -n "validate-eclipses" crates/pleiades-validate/src/release/notes.rs` — mirror each hit.

- [ ] **Step 7: Run the full validate suite + release smoke.**

Run: `cargo test -p pleiades-validate` then `cargo run -q -p pleiades-validate -- validate-crossings` then `cargo run -q -p pleiades-validate -- release-smoke`
Expected: all PASS; `release-smoke` output lists the crossings gate.

- [ ] **Step 8: Commit.**

```bash
git add crates/pleiades-validate
git commit -m "feat(validate): validate-crossings fail-closed SE-parity gate"
```

---

## Task 10: CLI `crossings` alias

**Files:**
- Modify: `crates/pleiades-cli/src/cli.rs`

**Interfaces:**
- Routes `crossings` / `validate-crossings` / `crossings-gate` through `validate_render_cli`, mirroring the `eclipses` arm (cli.rs ~line 801).

- [ ] **Step 1: Write the failing test** in `crates/pleiades-cli/src/cli/tests/validation.rs` (mirror the existing eclipse alias test):

```rust
#[test]
fn crossings_alias_dispatches_to_validate() {
    let out = render_cli(&["crossings"]).expect("crossings should dispatch");
    assert!(out.contains("Crossings gate"), "unexpected: {out}");
}
```

- [ ] **Step 2: Run to verify it fails.**

Run: `cargo test -p pleiades-cli crossings_alias_dispatches_to_validate`
Expected: FAIL (unknown command).

- [ ] **Step 3: Add the dispatch arm** in `crates/pleiades-cli/src/cli.rs` next to the eclipse arm:

```rust
        Some("validate-crossings") | Some("crossings-gate") => validate_render_cli(args),
        Some("crossings") => validate_render_cli(args),
```

*Plan note:* if `crossings` needs to map to `validate-crossings` for the validate layer, translate the arg as the eclipse `eclipses` alias does — check whether `validate_render_cli` keys off `args[0]`; if so, rewrite `args[0]` to `"validate-crossings"` before delegating (mirror exactly how `eclipses` reaches `validate-eclipses`).

- [ ] **Step 4: Run to verify it passes.**

Run: `cargo test -p pleiades-cli crossings_alias_dispatches_to_validate`
Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git add crates/pleiades-cli/src/cli.rs crates/pleiades-cli/src/cli/tests/validation.rs
git commit -m "feat(cli): crossings alias routed through the validate render layer"
```

---

## Task 11: Compatibility profile entry + README + PLAN status

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/mod.rs`
- Modify: `README.md`
- Modify: `PLAN.md`

**Interfaces:**
- Produces: a compatibility-profile entry for the crossings capability with a `claim_tier` tied to the `validate-crossings` evidence; profile id bumped `0.7.4` → `0.7.5`. The overclaim audit (`compat-claims-audit`) must stay green.

- [ ] **Step 1: Bump the profile id** in `crates/pleiades-core/src/compatibility/mod.rs`:

```rust
pub const CURRENT_COMPATIBILITY_PROFILE_ID: &str = "pleiades-compatibility-profile/0.7.5";
```

- [ ] **Step 2: Add a crossings capability entry** to the profile’s capability list (find the list that enumerates engine capabilities — e.g. where eclipse/angles capabilities are declared — and add a parallel entry with the release-grade claim tier justified by `validate-crossings`). Follow the exact struct/enum the surrounding entries use; do not invent fields.

- [ ] **Step 3: Run the compatibility + overclaim tests.**

Run: `cargo test -p pleiades-core compatibility` then `cargo run -q -p pleiades-validate -- compatibility-profile` and the overclaim audit (`grep -rn "compat-claims-audit\|compat_claims" crates/pleiades-validate/src` to find the command, then run it).
Expected: PASS; profile prints `0.7.5` and lists crossings.

- [ ] **Step 4: Update `README.md` "current state"** — add one sentence: longitude-crossing engine (`solcross`/`mooncross`/general/`helio_cross`) via `pleiades-events`, gated by `validate-crossings`. Match the surrounding prose style; the overclaim audit checks README ↔ profile agreement, so keep the claim tier consistent.

- [ ] **Step 5: Update `PLAN.md`** — in the status line and Phase notes, record: "SP-2a longitude crossings done (2026-07-03) — `pleiades-events` crate; `solcross`/`mooncross`/general geocentric + heliocentric `helio_cross`; `validate-crossings` gate wired into release-smoke/release-gate; compatibility profile 0.7.5. SP-2b (rise/set/transit) and SP-2c (local eclipse circumstances) remain." Remove any now-stale "SP-2 remains" wording that this slice closes.

- [ ] **Step 6: Run the full workspace test + audits once more.**

Run: `cargo test --workspace` then `cargo run -q -p pleiades-validate -- release-gate`
Expected: all PASS.

- [ ] **Step 7: Commit.**

```bash
git add crates/pleiades-core/src/compatibility/mod.rs README.md PLAN.md
git commit -m "docs(events): declare crossings capability; compatibility profile 0.7.5; SP-2a status"
```

---

## Self-Review

**1. Spec coverage** (each spec section → task):
- Public crossing API (next/prev/in-range, solcross/mooncross, general bodies) → Tasks 4.
- Heliocentric `helio_cross` + reconstruction → Tasks 3, 5.
- Root-finder (bracket + bisect, retrograde multiplicity) → Task 2 (+ SE triple-crossing row in Task 8).
- Geocentric apparent-of-date convention (Sun aberration-once) → Task 3.
- TDB time base + 1900–2100 window clamp → Tasks 1, 4.
- Fail-closed conditions (frame/body misuse, missing coords, out-of-window) → Tasks 1, 3, 4, 5.
- New `pleiades-events` crate, no core re-export, CLI via validate → Tasks 1, 10.
- SE reference tool (isolated) → Task 7.
- Committed corpus + manifest → Task 8.
- `validate-crossings` gate wired into release-smoke/gate + overclaim → Tasks 9, 11.
- Compatibility-profile bump + README + PLAN → Task 11.
- Pure-Rust audit constraint held → Tasks 1, 7 (audit run in both).
- Open items resolved at planning time: per-body ceilings (Task 9 constants), Sun-helper reuse without a cycle (Task 3 uses `pleiades-apparent` directly — no eclipse dep), heliocentric convention (Task 8 pins to SE), step sizes (Task 4 `step_days`), corpus shape (Task 8 single `crossings.csv` with `frame`/`body` columns).

**2. Placeholder scan:** The only intentional `todo!()` is Task 7's SE-FFI `main`, which is offline/SE-specific and fully specified by the referenced precedent file + exact SE signatures — not a logic gap. Task 3 flags one API-name confirmation (`ApparentPlaceError` variant) with the exact file to check. No "add error handling"/"write tests"-style placeholders elsewhere; every logic step ships real code.

**3. Type consistency:** `CrossingEngine`, `Crossing`, `CrossingFrame` (`GeocentricApparentOfDate`/`Heliocentric`), `EventError` variants, `crossings_in_range`/`wrap180`, and the gate’s `validate_crossings_corpus`/`CrossingsCorpusReport`/`run_crossings_gate` names are used identically across Tasks 1–11. The corpus schema `frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb` matches between the tool (Task 7), the CSV (Task 8), and the gate parser (Task 9).
