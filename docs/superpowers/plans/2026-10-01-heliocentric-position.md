# Public Ecliptic Position With Latitude and Speed (#89) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `EventEngine::position_at`, returning ecliptic longitude, latitude, distance and their speeds in either `CrossingFrame`, gated against Swiss Ephemeris for the heliocentric frame and against the chart layer for the geocentric frame; and move the toolchain and MSRV to Rust 1.99.0.

**Architecture:** Both frames follow one pattern: a base place with a base `Motion`, a corrected place, and `motion = base motion + d(corrected − base)/dt`, differenced over ±0.5 day. The differencing helper moves from `pleiades-core` into `pleiades-apparent` so the chart and the event engine share it. Heliocentric base rates come from Cartesian velocities (planet minus Sun) built from the backend's spherical rates.

**Tech Stack:** Rust 1.99.0 (stable), cargo-nextest via `mise run test`, Swiss Ephemeris via `libswisseph-sys` for the reference tool only (built under `devenv shell`).

**Spec:** `docs/superpowers/specs/2026-10-01-heliocentric-position-design.md`

## Global Constraints

- Branch: `feat/heliocentric-position`. Every commit message ends with `(#89)`.
- `EventEngine::longitude_at` must return bit-identical values to today. `validate-crossings` and the `crossings-golden` manifest must not change; a diff in either is a defect, never something to regenerate.
- No new `EventError` variants. Missing speeds are `None`, never an error and never a substitute value.
- No `unwrap`/`expect`/panic in library paths. Tests may use them.
- `pleiades-events` must not depend on `pleiades-core` except as a dev-dependency.
- The pinned fuzz nightly (`FUZZ_NIGHTLY` in `mise.toml`) is not touched.
- Run `cargo fmt --all` before every commit. Do not edit sources or commit while a background test run is in progress.
- Before each commit run, at minimum: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and the task's own tests. The final task runs `mise run ci`.
- All verification commands run in the foreground.

## Review Focus

1. **A lunar point or other distance-less body in the geocentric frame** — `position_at` must behave exactly as `longitude_at` does for the same input: same error, or same longitude. Pinned in Task 5 (`position_and_longitude_agree_or_fail_together`).
2. **An instant exactly on a window edge (JD 2415020.5 or 2488069.5)** — the position is returned and the speed is a one-sided difference, not an error and not `None`. Pinned in Task 5 (`window_edges_use_a_one_sided_difference`).
3. **A backend that reports no motion, or only some channels** — the position is valid and each missing channel is `None`; no channel is filled from another place. Pinned in Task 5 (`backend_without_motion_gives_none_speeds`) and Task 4 (`partial_backend_motion_gives_no_heliocentric_velocity`).
4. **A longitude straddling 0°/360° across the ±0.5 day span** — the differenced correction wraps and the speed stays finite and small. Pinned in Task 2 (`correction_rate_wraps_across_zero`) and Task 5 (`speed_is_continuous_across_the_zero_degree_wrap`).
5. **Two correction samples at the same Julian day passed to the now-public helper** — every channel `None`, not NaN or infinity. Pinned in Task 2 (`zero_span_gives_no_speed`).

---

### Task 1: Toolchain and MSRV to Rust 1.99.0

**Files:**
- Modify: `mise.toml` (already modified in the working tree: `rust = { version = "1.99.0", ... }`)
- Modify: `Cargo.toml:28`
- Modify: `README.md:79`

**Interfaces:**
- Consumes: nothing.
- Produces: a workspace that builds and lints clean on Rust 1.99.0.

- [ ] **Step 1: Confirm the pending change and install the toolchain**

Run: `git diff mise.toml`
Expected: the only change is `rust = { version = "1.98.1", ...}` → `"1.99.0"`.

Run: `mise install`
Expected: Rust 1.99.0 installed with `rustfmt` and `clippy`. Then `cargo --version` prints `cargo 1.99.0`.

- [ ] **Step 2: Bump the MSRV**

In `Cargo.toml`, change line 28:

```toml
rust-version = "1.99.0"
```

In `README.md`, change the line that reads `**Minimum supported Rust version: 1.98.1.**` so it reads:

```markdown
**Minimum supported Rust version: 1.99.0.** The MSRV is declared as
```

(Only the version number changes; keep the rest of the sentence.)

Run: `grep -rn "1\.98" --include=*.toml --include=*.md --include=*.yml --include=*.nix . | grep -v "^./target" | grep -v CHANGELOG`
Expected: no output. If a file still names 1.98.x as the toolchain or MSRV, update it the same way. Do not edit any `CHANGELOG.md`.

- [ ] **Step 3: Format, lint and test on the new toolchain**

Run: `cargo fmt --all --check`
Expected: no output. If 1.99.0's rustfmt reformats files, run `cargo fmt --all` and include the result in this commit.

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: finishes with no warnings. If 1.99.0 adds lints that fire, fix the root cause in each reported location (do not add `#[allow]` unless the lint is wrong for that line, and then scope it to the item with a one-line reason).

Run: `mise run test`
Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
git add mise.toml Cargo.toml README.md
git add -u
git commit -m "chore: move the toolchain and MSRV to Rust 1.99.0 (#89)"
```

---

### Task 2: Move the correction-differencing helper into `pleiades-apparent`

**Files:**
- Create: `crates/pleiades-apparent/src/motion.rs`
- Create: `crates/pleiades-apparent/src/motion/tests.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs` (add `pub mod motion;` next to `pub mod nutation;`)
- Delete: `crates/pleiades-core/src/chart/apparent_motion.rs`
- Move: `crates/pleiades-core/src/chart/apparent_motion/tests.rs` → `crates/pleiades-core/src/chart/apparent_motion_tests.rs`
- Modify: `crates/pleiades-core/src/chart/mod.rs:13` and `:56`

**Interfaces:**
- Consumes: `pleiades_types::{EclipticCoordinates, Motion}`.
- Produces, in `pleiades_apparent::motion`:
  - `pub const HALF_SPAN_DAYS: f64 = 0.5;`
  - `pub struct Correction` (private fields) with `pub fn between(corrected: &EclipticCoordinates, base: &EclipticCoordinates) -> Self`
  - `pub struct CorrectionSample { pub julian_day: f64, pub correction: Correction }`
  - `pub fn apparent_motion(base: Motion, earlier: &CorrectionSample, later: &CorrectionSample) -> Motion`

- [ ] **Step 1: Write the failing tests for the pure helper**

Create `crates/pleiades-apparent/src/motion/tests.rs`:

```rust
use super::*;
use pleiades_types::{EclipticCoordinates, Latitude, Longitude, Motion};

fn place(lon: f64, lat: f64, dist: Option<f64>) -> EclipticCoordinates {
    EclipticCoordinates::new(
        Longitude::from_degrees(lon),
        Latitude::from_degrees(lat),
        dist,
    )
}

fn sample(jd: f64, corrected: EclipticCoordinates, base: EclipticCoordinates) -> CorrectionSample {
    CorrectionSample {
        julian_day: jd,
        correction: Correction::between(&corrected, &base),
    }
}

#[test]
fn adds_the_correction_rate_to_each_channel() {
    // Correction grows by (0.2°, -0.1°, 0.01 AU) over one day.
    let earlier = sample(10.0, place(100.1, 1.0, Some(2.0)), place(100.0, 1.0, Some(2.0)));
    let later = sample(11.0, place(100.3, 0.9, Some(2.01)), place(100.0, 1.0, Some(2.0)));
    let out = apparent_motion(Motion::new(Some(1.0), Some(0.5), Some(0.0)), &earlier, &later);
    assert!((out.longitude_deg_per_day.unwrap() - 1.2).abs() < 1e-12);
    assert!((out.latitude_deg_per_day.unwrap() - 0.4).abs() < 1e-12);
    assert!((out.distance_au_per_day.unwrap() - 0.01).abs() < 1e-12);
}

#[test]
fn correction_rate_wraps_across_zero() {
    // Corrected place crosses 0°: 359.9° then 0.1°, base fixed at 359.8°.
    let earlier = sample(10.0, place(359.9, 0.0, None), place(359.8, 0.0, None));
    let later = sample(11.0, place(0.1, 0.0, None), place(359.8, 0.0, None));
    let out = apparent_motion(Motion::new(Some(0.0), Some(0.0), None), &earlier, &later);
    assert!((out.longitude_deg_per_day.unwrap() - 0.2).abs() < 1e-9);
}

#[test]
fn empty_base_channels_stay_empty() {
    let earlier = sample(10.0, place(1.0, 0.0, Some(1.0)), place(0.0, 0.0, Some(1.0)));
    let later = sample(11.0, place(2.0, 0.0, Some(1.0)), place(0.0, 0.0, Some(1.0)));
    let out = apparent_motion(Motion::new(None, Some(0.0), None), &earlier, &later);
    assert_eq!(out.longitude_deg_per_day, None);
    assert_eq!(out.latitude_deg_per_day, Some(0.0));
    assert_eq!(out.distance_au_per_day, None);
}

#[test]
fn distance_speed_is_unchanged_when_a_sample_has_no_distance() {
    let earlier = sample(10.0, place(1.0, 0.0, None), place(0.0, 0.0, Some(1.0)));
    let later = sample(11.0, place(2.0, 0.0, Some(1.5)), place(0.0, 0.0, Some(1.0)));
    let out = apparent_motion(Motion::new(Some(0.0), Some(0.0), Some(0.25)), &earlier, &later);
    assert_eq!(out.distance_au_per_day, Some(0.25));
}

#[test]
fn zero_span_gives_no_speed() {
    let a = sample(10.0, place(1.0, 0.0, Some(1.0)), place(0.0, 0.0, Some(1.0)));
    let out = apparent_motion(Motion::new(Some(1.0), Some(1.0), Some(1.0)), &a, &a);
    assert_eq!(out, Motion::new(None, None, None));
}

#[test]
fn non_finite_span_gives_no_speed() {
    let a = sample(10.0, place(1.0, 0.0, None), place(0.0, 0.0, None));
    let b = sample(f64::NAN, place(1.0, 0.0, None), place(0.0, 0.0, None));
    let out = apparent_motion(Motion::new(Some(1.0), Some(1.0), None), &a, &b);
    assert_eq!(out, Motion::new(None, None, None));
}
```

- [ ] **Step 2: Create the module by moving the file, and run the tests to see the zero-span test fail**

```bash
git mv crates/pleiades-core/src/chart/apparent_motion.rs crates/pleiades-apparent/src/motion.rs
git mv crates/pleiades-core/src/chart/apparent_motion/tests.rs crates/pleiades-core/src/chart/apparent_motion_tests.rs
```

In `crates/pleiades-apparent/src/lib.rs`, add next to `pub mod nutation;`:

```rust
pub mod motion;
```

Replace the whole of `crates/pleiades-apparent/src/motion.rs` with (this is the moved code made public, reworded in base/corrected terms, without the zero-span guard yet):

```rust
//! Speed of a corrected place, from the speed of its base place.
//!
//! A backend reports the speed of the place it serves (the *base* place). A
//! place derived from it by a small, smooth correction — the apparent place
//! (precession, nutation, annual aberration, light-time), or a frame rotation
//! from J2000 to the equinox of date — moves at a slightly different rate. The
//! corrected speed is the base speed plus the rate of that correction
//! (corrected minus base place).
//!
//! Differencing the correction rather than the corrected place itself keeps
//! the accuracy of the backend's own speed: the correction is small and
//! smooth, so its finite difference carries a truncation error far below the
//! one a difference of the full position would (the Moon's longitude speed
//! alone would be off by about 3e-3 deg/day over the same span).

use pleiades_types::{EclipticCoordinates, Motion};

/// Half-span of the correction difference, in days. Matches the span
/// `pleiades-data` and `pleiades-events` use for their differenced speeds.
pub const HALF_SPAN_DAYS: f64 = 0.5;

/// Corrected minus base place at one instant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Correction {
    /// Longitude correction in degrees, wrapped to `[-180, 180)`.
    longitude_deg: f64,
    /// Latitude correction in degrees.
    latitude_deg: f64,
    /// Distance correction in astronomical units, when both places carry a distance.
    distance_au: Option<f64>,
}

impl Correction {
    /// The correction that takes `base` to `corrected`.
    pub fn between(corrected: &EclipticCoordinates, base: &EclipticCoordinates) -> Self {
        Self {
            longitude_deg: wrap_signed(corrected.longitude.degrees() - base.longitude.degrees()),
            latitude_deg: corrected.latitude.degrees() - base.latitude.degrees(),
            distance_au: corrected
                .distance_au
                .zip(base.distance_au)
                .map(|(corrected, base)| corrected - base),
        }
    }
}

/// A correction and the Julian day it was evaluated at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CorrectionSample {
    /// Julian day of the sample.
    pub julian_day: f64,
    /// Corrected minus base place at that Julian day.
    pub correction: Correction,
}

/// Returns the speed of the corrected place: `base` plus the rate of the
/// correction between two samples. Channels `base` leaves empty stay empty,
/// and the distance speed is unchanged when a sample has no distance.
pub fn apparent_motion(
    base: Motion,
    earlier: &CorrectionSample,
    later: &CorrectionSample,
) -> Motion {
    let span_days = later.julian_day - earlier.julian_day;
    let (from, to) = (&earlier.correction, &later.correction);
    let longitude_rate = wrap_signed(to.longitude_deg - from.longitude_deg) / span_days;
    let latitude_rate = (to.latitude_deg - from.latitude_deg) / span_days;
    let distance_rate = to
        .distance_au
        .zip(from.distance_au)
        .map_or(0.0, |(to, from)| (to - from) / span_days);
    Motion::new(
        base.longitude_deg_per_day
            .map(|speed| speed + longitude_rate),
        base.latitude_deg_per_day.map(|speed| speed + latitude_rate),
        base.distance_au_per_day.map(|speed| speed + distance_rate),
    )
}

/// Wraps an angle difference in degrees to `[-180, 180)`.
fn wrap_signed(degrees: f64) -> f64 {
    (degrees + 180.0).rem_euclid(360.0) - 180.0
}

#[cfg(test)]
mod tests;
```

Run: `cargo test -p pleiades-apparent motion::`
Expected: `zero_span_gives_no_speed` and `non_finite_span_gives_no_speed` FAIL (the speeds are NaN/infinite, not `None`); the other four pass.

- [ ] **Step 3: Add the span guard**

In `apparent_motion`, directly after `let span_days = ...;`, insert:

```rust
    if !span_days.is_finite() || span_days == 0.0 {
        return Motion::new(None, None, None);
    }
```

and extend the function's rustdoc with:

```rust
/// The two samples must be at different instants: when their span is zero or
/// not finite the speed is unknown and every channel is `None`.
```

Run: `cargo test -p pleiades-apparent motion::`
Expected: 6 passed.

- [ ] **Step 4: Point `pleiades-core` at the shared module**

In `crates/pleiades-core/src/chart/mod.rs`:

Replace line 13 `mod apparent_motion;` with:

```rust
#[cfg(test)]
mod apparent_motion_tests;
```

Replace line 56 `use apparent_motion::{apparent_motion, Correction, CorrectionSample, HALF_SPAN_DAYS};` with:

```rust
use pleiades_apparent::motion::{apparent_motion, Correction, CorrectionSample, HALF_SPAN_DAYS};
```

Run: `ls crates/pleiades-core/src/chart/apparent_motion 2>/dev/null`
Expected: the directory is empty or missing; remove it if empty (`rmdir`).

Run: `grep -n "super::" crates/pleiades-core/src/chart/apparent_motion_tests.rs`
Expected: no output (the tests import through `crate::chart::...`). If any `super::` path appears, change it to the `crate::chart::` equivalent.

- [ ] **Step 5: Verify core is unchanged**

Run: `cargo test -p pleiades-core`
Expected: all pass, including every test that was in `chart::apparent_motion::tests` (now `chart::apparent_motion_tests`). The test count for that module must equal the count before the move: check with `cargo test -p pleiades-core apparent_motion -- --list | tail -1` on this commit and on `main`.

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add -A crates/pleiades-apparent crates/pleiades-core
git commit -m "refactor(apparent): share the correction-rate speed helper with pleiades-core (#89)"
```

---

### Task 3: Spherical ↔ Cartesian state conversions

**Files:**
- Create: `crates/pleiades-events/src/state_vector.rs`
- Create: `crates/pleiades-events/src/state_vector/tests.rs`
- Modify: `crates/pleiades-events/src/lib.rs` (add `mod state_vector;` after `mod semidiameter;`)

**Interfaces:**
- Consumes: `pleiades_types::Motion`; `crate::ephemeris::spherical_to_cartesian(lon_deg: f64, lat_deg: f64, r_au: f64) -> [f64; 3]` (exists).
- Produces:
  - `pub(crate) fn cartesian_velocity(lon_deg: f64, lat_deg: f64, r_au: f64, motion: Option<Motion>) -> Option<[f64; 3]>` — AU/day; `None` unless all three rates are present.
  - `pub(crate) fn spherical_rates(position: [f64; 3], velocity: [f64; 3]) -> Motion` — deg/day, deg/day, AU/day; a channel that is not finite is `None`.

- [ ] **Step 1: Write the failing tests**

Create `crates/pleiades-events/src/state_vector/tests.rs`:

```rust
use super::*;
use crate::ephemeris::spherical_to_cartesian;
use pleiades_types::Motion;

fn full(lon: f64, lat: f64, dist: f64) -> Option<Motion> {
    Some(Motion::new(Some(lon), Some(lat), Some(dist)))
}

#[test]
fn circular_coplanar_orbit_has_the_analytic_velocity() {
    // r = 2 AU, longitude 90°, moving at 1°/day in the ecliptic plane:
    // the velocity is tangential, -x direction, magnitude r·n.
    let v = cartesian_velocity(90.0, 0.0, 2.0, full(1.0, 0.0, 0.0)).unwrap();
    let speed = 2.0 * 1.0_f64.to_radians();
    assert!((v[0] + speed).abs() < 1e-15, "{v:?}");
    assert!(v[1].abs() < 1e-15, "{v:?}");
    assert!(v[2].abs() < 1e-15, "{v:?}");
}

#[test]
fn rates_round_trip_through_cartesian() {
    let (lon, lat, r) = (217.3, -6.4, 5.2);
    let (lon_rate, lat_rate, r_rate) = (0.083, -0.0021, 0.0004);
    let p = spherical_to_cartesian(lon, lat, r);
    let v = cartesian_velocity(lon, lat, r, full(lon_rate, lat_rate, r_rate)).unwrap();
    let back = spherical_rates(p, v);
    assert!((back.longitude_deg_per_day.unwrap() - lon_rate).abs() < 1e-13);
    assert!((back.latitude_deg_per_day.unwrap() - lat_rate).abs() < 1e-13);
    assert!((back.distance_au_per_day.unwrap() - r_rate).abs() < 1e-13);
}

#[test]
fn rates_match_a_central_difference_on_an_inclined_orbit() {
    // Position as an explicit function of time; velocity by a tiny central
    // difference of the Cartesian position, independent of cartesian_velocity.
    let at = |t: f64| spherical_to_cartesian(40.0 + 0.5 * t, 7.0 * (0.03 * t).sin(), 1.5 + 0.01 * t);
    let h = 1e-4;
    let (a, b, p) = (at(-h), at(h), at(0.0));
    let v = [
        (b[0] - a[0]) / (2.0 * h),
        (b[1] - a[1]) / (2.0 * h),
        (b[2] - a[2]) / (2.0 * h),
    ];
    let rates = spherical_rates(p, v);
    assert!((rates.longitude_deg_per_day.unwrap() - 0.5).abs() < 1e-7);
    // d/dt[7 sin(0.03 t)] at t = 0 is 0.21.
    assert!((rates.latitude_deg_per_day.unwrap() - 0.21).abs() < 1e-7);
    assert!((rates.distance_au_per_day.unwrap() - 0.01).abs() < 1e-7);
}

#[test]
fn missing_rate_channel_gives_no_velocity() {
    assert!(cartesian_velocity(10.0, 1.0, 1.0, None).is_none());
    let partial = Some(Motion::new(Some(1.0), None, Some(0.0)));
    assert!(cartesian_velocity(10.0, 1.0, 1.0, partial).is_none());
}

#[test]
fn pole_has_no_longitude_or_latitude_rate() {
    let rates = spherical_rates([0.0, 0.0, 1.0], [0.01, 0.0, 0.002]);
    assert_eq!(rates.longitude_deg_per_day, None);
    assert_eq!(rates.latitude_deg_per_day, None);
    assert!((rates.distance_au_per_day.unwrap() - 0.002).abs() < 1e-15);
}

#[test]
fn origin_has_no_rates() {
    assert_eq!(
        spherical_rates([0.0; 3], [0.01, 0.0, 0.0]),
        Motion::new(None, None, None)
    );
}
```

Add `mod state_vector;` to `crates/pleiades-events/src/lib.rs` after `mod semidiameter;`, and create `crates/pleiades-events/src/state_vector.rs` containing only:

```rust
//! Conversions between spherical ecliptic rates and Cartesian velocities.

#[cfg(test)]
mod tests;
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p pleiades-events state_vector::`
Expected: FAIL to compile — `cartesian_velocity` and `spherical_rates` not found.

- [ ] **Step 3: Implement**

Replace `crates/pleiades-events/src/state_vector.rs` with:

```rust
//! Conversions between spherical ecliptic rates and Cartesian velocities.
//!
//! Angles are degrees and degrees per day at the interface, distances are
//! astronomical units and AU per day. The frame is whatever frame the
//! spherical coordinates are in; these are pure geometry.

use pleiades_types::Motion;

/// Cartesian velocity (AU/day) of a point at spherical `(lon, lat, r)` moving
/// at the spherical rates in `motion`. `None` unless all three rates are known:
/// a velocity built from a partial rate set would describe a different motion.
pub(crate) fn cartesian_velocity(
    lon_deg: f64,
    lat_deg: f64,
    r_au: f64,
    motion: Option<Motion>,
) -> Option<[f64; 3]> {
    let motion = motion?;
    let lon_rate = motion.longitude_deg_per_day?.to_radians();
    let lat_rate = motion.latitude_deg_per_day?.to_radians();
    let r_rate = motion.distance_au_per_day?;
    let (sin_lon, cos_lon) = lon_deg.to_radians().sin_cos();
    let (sin_lat, cos_lat) = lat_deg.to_radians().sin_cos();
    Some([
        r_rate * cos_lat * cos_lon
            - r_au * sin_lat * cos_lon * lat_rate
            - r_au * cos_lat * sin_lon * lon_rate,
        r_rate * cos_lat * sin_lon - r_au * sin_lat * sin_lon * lat_rate
            + r_au * cos_lat * cos_lon * lon_rate,
        r_rate * sin_lat + r_au * cos_lat * lat_rate,
    ])
}

/// Spherical rates (deg/day, deg/day, AU/day) of a point with Cartesian
/// `position` (AU) and `velocity` (AU/day). A rate that is undefined — the
/// longitude and latitude rates on the polar axis, every rate at the origin —
/// is `None` rather than a non-finite number.
pub(crate) fn spherical_rates(position: [f64; 3], velocity: [f64; 3]) -> Motion {
    let [x, y, z] = position;
    let [vx, vy, vz] = velocity;
    let rho2 = x * x + y * y;
    let r = (rho2 + z * z).sqrt();
    let planar = x * vx + y * vy;
    let lon_rate = ((x * vy - y * vx) / rho2).to_degrees();
    let lat_rate = ((vz * rho2 - z * planar) / (r * r * rho2.sqrt())).to_degrees();
    let r_rate = (planar + z * vz) / r;
    let finite = |value: f64| value.is_finite().then_some(value);
    Motion::new(finite(lon_rate), finite(lat_rate), finite(r_rate))
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p pleiades-events state_vector::`
Expected: 6 passed.

Run: `cargo clippy -p pleiades-events --all-targets --all-features -- -D warnings`
Expected: clean. The functions are not yet used outside tests; if clippy reports `dead_code`, add `#![allow(dead_code)]` at the top of `state_vector.rs` with the comment `// Consumed by position.rs (next task).` and remove it in Task 5.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add crates/pleiades-events/src/state_vector.rs crates/pleiades-events/src/state_vector crates/pleiades-events/src/lib.rs
git commit -m "feat(events): spherical-rate and Cartesian-velocity conversions (#89)"
```

---

### Task 4: Heliocentric place with latitude, distance and velocity

**Files:**
- Modify: `crates/pleiades-events/src/ephemeris.rs` (the `read_mean_ecliptic` function, the `heliocentric_longitude_deg` function, the `tests` module)

**Interfaces:**
- Consumes: `crate::state_vector::cartesian_velocity` (Task 3).
- Produces, all `pub(crate)` in `crate::ephemeris`:
  - `fn read_mean_ecliptic_with_motion<B: EphemerisBackend>(backend: &B, body: CelestialBody, body_label: &'static str, julian_day: f64) -> Result<((f64, f64, f64), Option<Motion>), EventError>`
  - `struct HeliocentricJ2000 { pub(crate) position: [f64; 3], pub(crate) velocity: Option<[f64; 3]> }`
  - `fn heliocentric_j2000<B: EphemerisBackend>(backend: &B, body: CelestialBody, body_label: &'static str, julian_day: f64) -> Result<HeliocentricJ2000, EventError>`
  - `fn j2000_spherical(position: [f64; 3]) -> (f64, f64, f64)` — (lon °, lat °, dist AU)
  - `fn heliocentric_of_date(position: [f64; 3], julian_day: f64) -> Result<(f64, f64, f64), EventError>` — (lon °, lat °, dist AU), true equinox of date
  - `heliocentric_longitude_deg` keeps its signature and returns the same bits as before.

- [ ] **Step 1: Capture the current heliocentric longitudes as a bit-exact pin**

Before changing anything, add this test to the `tests` module at the bottom of `crates/pleiades-events/src/ephemeris.rs`. It prints the bits on the first run.

```rust
    /// The longitude wrapper must not move by a single bit when the
    /// reconstruction is refactored: `validate-crossings` root-finds on it.
    #[test]
    fn heliocentric_longitude_bits_are_pinned() {
        let backend = pleiades_data::packaged_backend();
        let cases = [
            (CelestialBody::Mercury, "Mercury", 2_415_100.25),
            (CelestialBody::Mars, "Mars", 2_451_545.0),
            (CelestialBody::Saturn, "Saturn", 2_439_500.066527),
            (CelestialBody::Pluto, "Pluto", 2_487_900.5),
        ];
        let got: Vec<u64> = cases
            .iter()
            .map(|(body, label, jd)| {
                heliocentric_longitude_deg(&backend, body.clone(), label, *jd)
                    .unwrap()
                    .to_bits()
            })
            .collect();
        eprintln!("PIN {got:?}");
        assert_eq!(got, PINNED_HELIO_LONGITUDE_BITS);
    }
```

and above it, inside the same `tests` module:

```rust
    const PINNED_HELIO_LONGITUDE_BITS: [u64; 4] = [0, 0, 0, 0];
```

Run: `cargo test -p pleiades-events ephemeris::tests::heliocentric_longitude_bits_are_pinned -- --nocapture`
Expected: FAIL, with a line `PIN [a, b, c, d]` in the output. Copy those four integers into `PINNED_HELIO_LONGITUDE_BITS`, remove the `eprintln!` line, and re-run.
Expected: PASS. This is the pre-refactor baseline; do not change these numbers again in this task.

- [ ] **Step 2: Write the failing tests for the new helpers**

Add to the same `tests` module:

```rust
    #[test]
    fn of_date_longitude_matches_the_longitude_wrapper() {
        let backend = pleiades_data::packaged_backend();
        let jd = 2_451_545.0;
        let helio = heliocentric_j2000(&backend, CelestialBody::Mars, "Mars", jd).unwrap();
        let (lon, lat, dist) = heliocentric_of_date(helio.position, jd).unwrap();
        let wrapper = heliocentric_longitude_deg(&backend, CelestialBody::Mars, "Mars", jd).unwrap();
        assert_eq!(lon.to_bits(), wrapper.to_bits());
        // Mars: heliocentric latitude within its 1.85° inclination, distance 1.38–1.67 AU.
        assert!(lat.abs() < 1.9, "latitude {lat}");
        assert!((1.38..1.67).contains(&dist), "distance {dist}");
    }

    #[test]
    fn j2000_and_of_date_share_the_distance() {
        let backend = pleiades_data::packaged_backend();
        let jd = 2_470_000.5;
        let helio = heliocentric_j2000(&backend, CelestialBody::Jupiter, "Jupiter", jd).unwrap();
        let j2000 = j2000_spherical(helio.position);
        let of_date = heliocentric_of_date(helio.position, jd).unwrap();
        assert_eq!(j2000.2.to_bits(), of_date.2.to_bits());
        // 2050: precession has moved the equinox by roughly 0.7°.
        let shift = (of_date.0 - j2000.0 + 180.0).rem_euclid(360.0) - 180.0;
        assert!((0.6..0.8).contains(&shift), "precession shift {shift}");
    }

    #[test]
    fn packaged_backend_gives_a_heliocentric_velocity() {
        let backend = pleiades_data::packaged_backend();
        let helio =
            heliocentric_j2000(&backend, CelestialBody::Venus, "Venus", 2_451_545.0).unwrap();
        let v = helio.velocity.expect("packaged backend reports motion");
        // Venus orbital speed is about 0.0202 AU/day.
        let speed = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((0.0195..0.0210).contains(&speed), "speed {speed}");
    }

    #[test]
    fn partial_backend_motion_gives_no_heliocentric_velocity() {
        // LinearSunMoon serves no planets; the Sun–Moon mock has no full
        // planetary motion, so use the pure helper on a partial rate set.
        use crate::state_vector::cartesian_velocity;
        use pleiades_types::Motion;
        let planet = cartesian_velocity(10.0, 0.0, 1.5, Some(Motion::new(Some(0.5), None, Some(0.0))));
        let sun = cartesian_velocity(100.0, 0.0, 1.0, Some(Motion::new(Some(1.0), Some(0.0), Some(0.0))));
        assert!(combine_velocities(planet, sun).is_none());
    }
```

Run: `cargo test -p pleiades-events ephemeris::tests`
Expected: FAIL to compile — `heliocentric_j2000`, `heliocentric_of_date`, `j2000_spherical`, `combine_velocities` not found.

- [ ] **Step 3: Implement**

In `crates/pleiades-events/src/ephemeris.rs`:

Add `Motion` to the `pleiades_types` import list, and add `use crate::state_vector::cartesian_velocity;`.

Replace `read_mean_ecliptic` with the pair below (same reads, same values; the old function becomes a wrapper):

```rust
/// Mean/J2000 geocentric ecliptic `(longitude_deg, latitude_deg, distance_au)`
/// and the backend's motion for that place, when it reports one.
pub(crate) fn read_mean_ecliptic_with_motion<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<((f64, f64, f64), Option<Motion>), EventError> {
    let result = backend
        .position(&request(body, julian_day))
        .map_err(|e| EventError::Backend(e.to_string()))?;
    let ecliptic = result.ecliptic.ok_or(EventError::MissingCoordinates {
        body_label,
        julian_day,
    })?;
    let distance = ecliptic.distance_au.ok_or(EventError::MissingCoordinates {
        body_label,
        julian_day,
    })?;
    Ok((
        (
            ecliptic.longitude.degrees(),
            ecliptic.latitude.degrees(),
            distance,
        ),
        result.motion,
    ))
}

/// Mean/J2000 geocentric ecliptic (longitude_deg, latitude_deg, distance_au).
pub(crate) fn read_mean_ecliptic<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<(f64, f64, f64), EventError> {
    Ok(read_mean_ecliptic_with_motion(backend, body, body_label, julian_day)?.0)
}
```

Replace the whole `heliocentric_longitude_deg` function (keep its doc comment on the wrapper) with:

```rust
/// Heliocentric J2000 ecliptic state of a body: `P_helio = P_geo − S_geo`,
/// reconstructed from the mean geocentric planet and Sun.
pub(crate) struct HeliocentricJ2000 {
    /// Heliocentric position vector, AU.
    pub(crate) position: [f64; 3],
    /// Heliocentric velocity vector, AU/day, when the backend reports all
    /// three rates for both the body and the Sun.
    pub(crate) velocity: Option<[f64; 3]>,
}

/// Planet-minus-Sun velocity, when both are known.
pub(crate) fn combine_velocities(
    planet: Option<[f64; 3]>,
    sun: Option<[f64; 3]>,
) -> Option<[f64; 3]> {
    let (planet, sun) = planet.zip(sun)?;
    Some([planet[0] - sun[0], planet[1] - sun[1], planet[2] - sun[2]])
}

/// Reads the mean geocentric body and Sun and subtracts them. Both carry
/// distance (AU); a missing distance fails closed.
///
/// The heliocentric place is GEOMETRIC (Sun-centred): no annual aberration or
/// light-time is applied.
pub(crate) fn heliocentric_j2000<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<HeliocentricJ2000, EventError> {
    let ((pl, pb, pd), planet_motion) =
        read_mean_ecliptic_with_motion(backend, body, body_label, julian_day)?;
    let ((sl, sb, sd), sun_motion) =
        read_mean_ecliptic_with_motion(backend, CelestialBody::Sun, "Sun", julian_day)?;
    let planet = spherical_to_cartesian(pl, pb, pd);
    let sun = spherical_to_cartesian(sl, sb, sd);
    Ok(HeliocentricJ2000 {
        position: [planet[0] - sun[0], planet[1] - sun[1], planet[2] - sun[2]],
        velocity: combine_velocities(
            cartesian_velocity(pl, pb, pd, planet_motion),
            cartesian_velocity(sl, sb, sd, sun_motion),
        ),
    })
}

/// J2000 ecliptic `(longitude_deg, latitude_deg, distance_au)` of a vector.
pub(crate) fn j2000_spherical(position: [f64; 3]) -> (f64, f64, f64) {
    let [x, y, z] = position;
    let planar = (x * x + y * y).sqrt();
    (
        y.atan2(x).to_degrees().rem_euclid(360.0),
        z.atan2(planar).to_degrees(),
        (x * x + y * y + z * z).sqrt(),
    )
}

/// Rotates a heliocentric J2000 vector to the **true equinox of date**
/// (precession, then nutation in longitude) to match SE's `SEFLG_HELCTR`:
/// `(longitude_deg, latitude_deg, distance_au)`. Nutation in longitude leaves
/// the ecliptic latitude unchanged, and the rotation leaves the distance
/// unchanged.
pub(crate) fn heliocentric_of_date(
    position: [f64; 3],
    julian_day: f64,
) -> Result<(f64, f64, f64), EventError> {
    let (lon_j2000, lat_j2000, distance) = j2000_spherical(position);

    // J2000 -> mean equinox/ecliptic of date (precession).
    let precessed = precess_ecliptic_j2000_to_date(lon_j2000, lat_j2000, julian_day)
        .map_err(|e| EventError::Backend(format!("helio precession failed: {e}")))?;

    // Mean -> true equinox of date: add nutation in longitude (Δψ).
    let nut = nutation(julian_day)
        .map_err(|e| EventError::Backend(format!("helio nutation failed: {e}")))?;

    Ok((
        (precessed.longitude_deg + nut.delta_psi_arcsec / 3600.0).rem_euclid(360.0),
        precessed.latitude_deg,
        distance,
    ))
}

/// Heliocentric ecliptic longitude (degrees) of the true equinox of date.
/// Thin wrapper over [`heliocentric_j2000`] and [`heliocentric_of_date`]; its
/// return value is byte-identical to before their extraction, which
/// `validate-crossings` depends on.
pub(crate) fn heliocentric_longitude_deg<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<f64, EventError> {
    let helio = heliocentric_j2000(backend, body, body_label, julian_day)?;
    Ok(heliocentric_of_date(helio.position, julian_day)?.0)
}
```

Note on bit identity: the old code computed `lat_j2000` as `helio[2].atan2((helio[0] * helio[0] + helio[1] * helio[1]).sqrt())` and `lon_j2000` as `helio[1].atan2(helio[0]).to_degrees().rem_euclid(360.0)`. `j2000_spherical` evaluates exactly those expressions, so the longitude is unchanged bit for bit.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p pleiades-events ephemeris::tests`
Expected: all pass, including `heliocentric_longitude_bits_are_pinned` with the numbers captured in Step 1. If the pin fails, the refactor changed an evaluation order: restore the exact expressions; do not update the pin.

Run: `cargo test -p pleiades-events --test heliocentric`
Expected: PASS (the Saturn crossing regression is unchanged).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add crates/pleiades-events/src/ephemeris.rs
git commit -m "refactor(events): expose the heliocentric latitude, distance and velocity internally (#89)"
```

---

### Task 5: `EclipticPosition` and `EventEngine::position_at`

**Files:**
- Create: `crates/pleiades-events/src/position.rs`
- Create: `crates/pleiades-events/tests/position.rs`
- Modify: `crates/pleiades-events/src/lib.rs` (add `mod position;` after `mod pheno;`, and `pub use position::EclipticPosition;`)
- Modify: `crates/pleiades-events/Cargo.toml` (dev-dependency on `pleiades-core`)
- Modify: `crates/pleiades-events/src/state_vector.rs` (remove the temporary `#![allow(dead_code)]` if Task 3 added it)

**Interfaces:**
- Consumes:
  - `pleiades_apparent::motion::{apparent_motion, Correction, CorrectionSample, HALF_SPAN_DAYS}` (Task 2)
  - `crate::state_vector::spherical_rates` (Task 3)
  - `crate::ephemeris::{geocentric_apparent_ecliptic, heliocentric_j2000, heliocentric_of_date, j2000_spherical, read_mean_ecliptic_with_motion}` (Task 4)
  - `crate::crossings::{body_label, CrossingFrame, EventEngine}`, `EventEngine::check_window` (exist, `pub(crate)`)
- Produces:
  - `pub struct EclipticPosition { pub body: CelestialBody, pub frame: CrossingFrame, pub instant: Instant, pub ecliptic: EclipticCoordinates, pub motion: Motion }` (`#[non_exhaustive]`)
  - `pub fn EventEngine::position_at(&self, body: CelestialBody, frame: CrossingFrame, instant: Instant) -> Result<EclipticPosition, EventError>`

- [ ] **Step 1: Add the dev-dependency**

In `crates/pleiades-events/Cargo.toml`, under `[dev-dependencies]`, add:

```toml
pleiades-core = { path = "../pleiades-core" }
```

Run: `cargo tree -p pleiades-core | grep pleiades-events`
Expected: no output (core does not depend on events, so there is no cycle).

- [ ] **Step 2: Write the failing integration tests**

Create `crates/pleiades-events/tests/position.rs`:

```rust
//! `EventEngine::position_at`: longitude identity with `longitude_at`, guards,
//! speed behaviour at the window edges and without backend motion, and
//! agreement with the chart layer's apparent place.

use pleiades_backend::{
    BackendMetadata, EphemerisBackend, EphemerisError, EphemerisRequest, EphemerisResult,
};
use pleiades_core::{ChartEngine, ChartRequest};
use pleiades_data::packaged_backend;
use pleiades_events::{
    CrossingFrame, EclipticPosition, EventEngine, EventError, WINDOW_END_JD, WINDOW_START_JD,
};
use pleiades_types::{Apparentness, CelestialBody, Instant, JulianDay, Motion, TimeScale};

const GEO: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const HELIO: CrossingFrame = CrossingFrame::Heliocentric;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn wrap(deg: f64) -> f64 {
    (deg + 180.0).rem_euclid(360.0) - 180.0
}

/// Delegates to an inner backend, optionally dropping the motion it reports.
struct StripMotion<B> {
    inner: B,
}

impl<B: EphemerisBackend> EphemerisBackend for StripMotion<B> {
    fn metadata(&self) -> BackendMetadata {
        self.inner.metadata()
    }
    fn supports_body(&self, body: CelestialBody) -> bool {
        self.inner.supports_body(body)
    }
    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let mut result = self.inner.position(req)?;
        result.motion = None;
        Ok(result)
    }
}

fn position(body: CelestialBody, frame: CrossingFrame, jd: f64) -> EclipticPosition {
    EventEngine::new(packaged_backend())
        .position_at(body, frame, tdb(jd))
        .expect("position")
}

#[test]
fn longitude_is_bit_identical_to_longitude_at() {
    let engine = EventEngine::new(packaged_backend());
    let bodies = [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Mars,
        CelestialBody::Saturn,
        CelestialBody::Pluto,
    ];
    for jd in [2_415_100.25, 2_451_545.0, 2_460_000.5, 2_487_900.5] {
        for body in &bodies {
            for frame in [GEO, HELIO] {
                if frame == HELIO && matches!(body, CelestialBody::Sun | CelestialBody::Moon) {
                    continue;
                }
                let lon = engine.longitude_at(body.clone(), frame, tdb(jd)).unwrap();
                let pos = engine.position_at(body.clone(), frame, tdb(jd)).unwrap();
                assert_eq!(
                    pos.ecliptic.longitude.degrees().to_bits(),
                    lon.degrees().to_bits(),
                    "{body:?} {frame:?} {jd}"
                );
                assert_eq!(pos.body, *body);
                assert_eq!(pos.frame, frame);
                assert_eq!(pos.instant, tdb(jd));
                assert!(pos.ecliptic.distance_au.is_some());
            }
        }
    }
}

#[test]
fn position_and_longitude_agree_or_fail_together() {
    // Lunar points and anything else the longitude evaluator rejects must be
    // rejected by position_at too, and anything it serves must match.
    let engine = EventEngine::new(packaged_backend());
    let bodies = [
        CelestialBody::MeanNode,
        CelestialBody::TrueNode,
        CelestialBody::MeanApogee,
        CelestialBody::TrueApogee,
    ];
    for body in bodies {
        for frame in [GEO, HELIO] {
            let lon = engine.longitude_at(body.clone(), frame, tdb(2_451_545.0));
            let pos = engine.position_at(body.clone(), frame, tdb(2_451_545.0));
            match (lon, pos) {
                (Ok(lon), Ok(pos)) => assert_eq!(
                    pos.ecliptic.longitude.degrees().to_bits(),
                    lon.degrees().to_bits(),
                    "{body:?} {frame:?}"
                ),
                (Err(_), Err(_)) => {}
                (lon, pos) => panic!("{body:?} {frame:?}: longitude_at {lon:?} vs position_at {pos:?}"),
            }
        }
    }
}

#[test]
fn guards_match_longitude_at() {
    let engine = EventEngine::new(packaged_backend());
    for jd in [WINDOW_START_JD - 0.001, WINDOW_END_JD + 0.001] {
        assert!(matches!(
            engine.position_at(CelestialBody::Mars, GEO, tdb(jd)),
            Err(EventError::OutOfWindow { .. })
        ));
    }
    for body in [CelestialBody::Sun, CelestialBody::Moon] {
        assert!(matches!(
            engine.position_at(body, HELIO, tdb(2_451_545.0)),
            Err(EventError::UnsupportedFrame { .. })
        ));
    }
}

#[test]
fn speed_matches_a_small_central_difference_of_the_position() {
    // Independent check of all three channels in both frames: difference the
    // engine's own position over ±0.01 day. Truncation at that step is far
    // below the tolerance; the tolerance absorbs packaged interpolation noise.
    let engine = EventEngine::new(packaged_backend());
    let h = 0.01;
    for (body, frame) in [
        (CelestialBody::Mars, GEO),
        (CelestialBody::Jupiter, GEO),
        (CelestialBody::Mars, HELIO),
        (CelestialBody::Mercury, HELIO),
        (CelestialBody::Neptune, HELIO),
    ] {
        let jd = 2_455_000.5;
        let at = |jd: f64| engine.position_at(body.clone(), frame, tdb(jd)).unwrap().ecliptic;
        let (a, b) = (at(jd - h), at(jd + h));
        let lon_rate = wrap(b.longitude.degrees() - a.longitude.degrees()) / (2.0 * h);
        let lat_rate = (b.latitude.degrees() - a.latitude.degrees()) / (2.0 * h);
        let dist_rate = (b.distance_au.unwrap() - a.distance_au.unwrap()) / (2.0 * h);
        let motion = engine.position_at(body.clone(), frame, tdb(jd)).unwrap().motion;
        // 1e-4 deg/day = 0.36"/day.
        assert!(
            (motion.longitude_deg_per_day.unwrap() - lon_rate).abs() < 1e-4,
            "{body:?} {frame:?} lon {:?} vs {lon_rate}",
            motion.longitude_deg_per_day
        );
        assert!(
            (motion.latitude_deg_per_day.unwrap() - lat_rate).abs() < 1e-4,
            "{body:?} {frame:?} lat {:?} vs {lat_rate}",
            motion.latitude_deg_per_day
        );
        assert!(
            (motion.distance_au_per_day.unwrap() - dist_rate).abs() < 1e-5,
            "{body:?} {frame:?} dist {:?} vs {dist_rate}",
            motion.distance_au_per_day
        );
    }
}

#[test]
fn heliocentric_neptune_speed_is_in_its_orbital_range() {
    // Neptune's mean motion is 0.00598 deg/day; the of-date speed adds general
    // precession (3.82e-5 deg/day) and the nutation rate. This is a coarse
    // sanity bound; whether the precession rate is included is decided by the
    // signed-mean diagnostic of validate-helio-position.
    let motion = position(CelestialBody::Neptune, HELIO, 2_451_545.0).motion;
    let lon_rate = motion.longitude_deg_per_day.unwrap();
    assert!((0.0055..0.0068).contains(&lon_rate), "Neptune helio speed {lon_rate}");
}

#[test]
fn window_edges_use_a_one_sided_difference() {
    for jd in [WINDOW_START_JD, WINDOW_END_JD] {
        for (body, frame) in [(CelestialBody::Mars, HELIO), (CelestialBody::Sun, GEO)] {
            let at_edge = EventEngine::new(packaged_backend()).position_at(body.clone(), frame, tdb(jd));
            let Ok(at_edge) = at_edge else {
                // The backend itself may not serve the exact edge instant in the
                // apparent path (light-time re-query leaves its range). That is
                // longitude_at's behaviour too, checked here.
                assert!(EventEngine::new(packaged_backend())
                    .longitude_at(body.clone(), frame, tdb(jd))
                    .is_err());
                continue;
            };
            let inside = if jd == WINDOW_START_JD { jd + 1.0 } else { jd - 1.0 };
            let near = position(body.clone(), frame, inside);
            let (edge_rate, near_rate) = (
                at_edge.motion.longitude_deg_per_day.expect("one-sided speed, not None"),
                near.motion.longitude_deg_per_day.unwrap(),
            );
            // One day apart, a planet's or the Sun's speed changes by well under 0.05 deg/day.
            assert!(
                (edge_rate - near_rate).abs() < 0.05,
                "{body:?} {frame:?} edge {edge_rate} vs inside {near_rate}"
            );
        }
    }
}

#[test]
fn backend_without_motion_gives_none_speeds() {
    let engine = EventEngine::new(StripMotion { inner: packaged_backend() });
    for (body, frame) in [(CelestialBody::Mars, GEO), (CelestialBody::Mars, HELIO)] {
        let pos = engine.position_at(body, frame, tdb(2_451_545.0)).unwrap();
        assert_eq!(pos.motion, Motion::new(None, None, None));
        assert!(pos.ecliptic.distance_au.is_some());
    }
}

#[test]
fn speed_is_continuous_across_the_zero_degree_wrap() {
    // Find a heliocentric Mars crossing of 0° and evaluate the speed on it:
    // the samples at ±0.5 day sit on either side of the 0°/360° seam.
    let engine = EventEngine::new(packaged_backend());
    let crossing = engine
        .next_longitude_crossing(
            CelestialBody::Mars,
            pleiades_types::Longitude::from_degrees(0.0),
            HELIO,
            tdb(2_451_545.0),
        )
        .unwrap()
        .expect("Mars crosses 0° heliocentric");
    let jd = crossing.instant.julian_day.days();
    let on = position(CelestialBody::Mars, HELIO, jd).motion.longitude_deg_per_day.unwrap();
    let before = position(CelestialBody::Mars, HELIO, jd - 2.0).motion.longitude_deg_per_day.unwrap();
    // Heliocentric Mars moves 0.43–0.64 deg/day and changes slowly.
    assert!((0.4..0.7).contains(&on), "speed on the seam {on}");
    assert!((on - before).abs() < 0.01, "seam {on} vs before {before}");
}

#[test]
fn geocentric_position_and_speed_match_the_chart_layer() {
    // The chart's apparent tropical geocentric placement is gated against JPL
    // Horizons by validate-apparent; position_at must report the same place
    // and the same speed.
    let bodies = [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Mars,
        CelestialBody::Jupiter,
        CelestialBody::Pluto,
    ];
    let engine = EventEngine::new(packaged_backend());
    for jd in [WINDOW_START_JD + 10.0, 2_451_545.0, 2_460_000.5, WINDOW_END_JD - 10.0] {
        let request = ChartRequest::new(tdb(jd))
            .with_bodies(bodies.to_vec())
            .with_apparentness(Apparentness::Apparent);
        let chart = ChartEngine::new(packaged_backend())
            .chart(&request)
            .expect("chart");
        for body in &bodies {
            let placed = &chart.placement_for(body).expect("placed").position;
            assert_eq!(placed.apparent, Apparentness::Apparent, "{body:?} {jd}");
            let chart_ecl = placed.ecliptic.as_ref().expect("chart ecliptic");
            let chart_motion = placed.motion.expect("chart motion");
            let pos = engine.position_at(body.clone(), GEO, tdb(jd)).unwrap();
            let d_lon = wrap(pos.ecliptic.longitude.degrees() - chart_ecl.longitude.degrees());
            let d_lat = pos.ecliptic.latitude.degrees() - chart_ecl.latitude.degrees();
            assert!(d_lon.abs() < 1e-9, "{body:?} {jd} lon diff {d_lon:e}");
            assert!(d_lat.abs() < 1e-9, "{body:?} {jd} lat diff {d_lat:e}");
            let d_dist = pos.ecliptic.distance_au.unwrap() - chart_ecl.distance_au.unwrap();
            assert!(d_dist.abs() < 1e-12, "{body:?} {jd} dist diff {d_dist:e}");
            for (name, got, want) in [
                ("lon", pos.motion.longitude_deg_per_day, chart_motion.longitude_deg_per_day),
                ("lat", pos.motion.latitude_deg_per_day, chart_motion.latitude_deg_per_day),
                ("dist", pos.motion.distance_au_per_day, chart_motion.distance_au_per_day),
            ] {
                let (got, want) = (got.expect(name), want.expect(name));
                assert!((got - want).abs() < 1e-9, "{body:?} {jd} {name} speed {got} vs {want}");
            }
        }
    }
}
```

Run: `cargo test -p pleiades-events --test position`
Expected: FAIL to compile — `EclipticPosition` and `position_at` not found.

- [ ] **Step 3: Implement `position.rs`**

Add to `crates/pleiades-events/src/lib.rs`: `mod position;` after `mod pheno;`, and `pub use position::EclipticPosition;` after the `pub use pheno::PhenoData;` line. If Task 3 added `#![allow(dead_code)]` to `state_vector.rs`, remove it.

Create `crates/pleiades-events/src/position.rs`:

```rust
//! Ecliptic position and speed of a body in a [`CrossingFrame`].
//!
//! Both frames follow one pattern. A *base* place has a speed known from the
//! backend; the reported place is that base place moved by a small, smooth
//! correction; and the reported speed is the base speed plus the rate of the
//! correction, differenced over ±[`HALF_SPAN_DAYS`].
//!
//! | Frame | Base place and speed | Corrected place |
//! |---|---|---|
//! | geocentric apparent of date | backend mean J2000 place and its motion | apparent place of date |
//! | heliocentric | planet minus Sun in J2000, rates from Cartesian velocities | true equinox of date |

use crate::crossings::{body_label, CrossingFrame, EventEngine};
use crate::ephemeris::{
    geocentric_apparent_ecliptic, heliocentric_j2000, heliocentric_of_date, j2000_spherical,
    read_mean_ecliptic_with_motion,
};
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use crate::state_vector::spherical_rates;
use pleiades_apparent::motion::{apparent_motion, Correction, CorrectionSample, HALF_SPAN_DAYS};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{
    CelestialBody, EclipticCoordinates, Instant, Latitude, Longitude, Motion,
};

/// Ecliptic position and speed of a body at one instant.
//
// `CelestialBody` is not `Copy`, so neither is this.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct EclipticPosition {
    /// The body.
    pub body: CelestialBody,
    /// The frame `ecliptic` and `motion` are expressed in.
    pub frame: CrossingFrame,
    /// Instant of the position (TDB).
    pub instant: Instant,
    /// Ecliptic longitude and latitude (degrees) and distance (AU, always
    /// `Some`): from the Earth's centre for the geocentric frame, from the Sun
    /// for the heliocentric frame.
    pub ecliptic: EclipticCoordinates,
    /// Speed of `ecliptic`: longitude and latitude in degrees per day,
    /// distance in AU per day. A channel is `None` when the backend reports no
    /// speed to derive it from.
    pub motion: Motion,
}

const NO_MOTION: Motion = Motion::new(None, None, None);

/// The base and corrected place of a body at one instant, with the base speed.
struct Sample {
    base: EclipticCoordinates,
    corrected: EclipticCoordinates,
    base_motion: Option<Motion>,
}

impl Sample {
    fn correction_at(&self, julian_day: f64) -> CorrectionSample {
        CorrectionSample {
            julian_day,
            correction: Correction::between(&self.corrected, &self.base),
        }
    }
}

fn coordinates((lon_deg, lat_deg, distance_au): (f64, f64, f64)) -> EclipticCoordinates {
    EclipticCoordinates::new(
        Longitude::from_degrees(lon_deg),
        Latitude::from_degrees(lat_deg),
        Some(distance_au),
    )
}

fn sample<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    frame: CrossingFrame,
    julian_day: f64,
) -> Result<Sample, EventError> {
    let label = body_label(body);
    match frame {
        CrossingFrame::GeocentricApparentOfDate => {
            let (mean, base_motion) =
                read_mean_ecliptic_with_motion(backend, body.clone(), label, julian_day)?;
            let apparent = geocentric_apparent_ecliptic(backend, body.clone(), label, julian_day)?;
            Ok(Sample {
                base: coordinates(mean),
                corrected: coordinates(apparent),
                base_motion,
            })
        }
        CrossingFrame::Heliocentric => {
            let helio = heliocentric_j2000(backend, body.clone(), label, julian_day)?;
            let of_date = heliocentric_of_date(helio.position, julian_day)?;
            Ok(Sample {
                base: coordinates(j2000_spherical(helio.position)),
                corrected: coordinates(of_date),
                base_motion: helio
                    .velocity
                    .map(|velocity| spherical_rates(helio.position, velocity)),
            })
        }
    }
}

/// Speed of the corrected place at `julian_day`, whose sample is `centre`.
///
/// The correction is differenced centrally over ±[`HALF_SPAN_DAYS`]. A
/// neighbouring instant outside the engine's window, or one the backend cannot
/// serve, is dropped and the difference becomes one-sided. With no base speed,
/// or with neither neighbour, the speed is unknown: every channel is `None`
/// rather than a speed that describes another place.
fn motion<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    frame: CrossingFrame,
    julian_day: f64,
    centre: &Sample,
) -> Motion {
    let Some(base) = centre.base_motion else {
        return NO_MOTION;
    };
    let neighbour = |jd: f64| {
        if !(WINDOW_START_JD..=WINDOW_END_JD).contains(&jd) {
            return None;
        }
        sample(backend, body, frame, jd)
            .ok()
            .map(|sample| sample.correction_at(jd))
    };
    let centre = centre.correction_at(julian_day);
    let (earlier, later) = match (
        neighbour(julian_day - HALF_SPAN_DAYS),
        neighbour(julian_day + HALF_SPAN_DAYS),
    ) {
        (Some(earlier), Some(later)) => (earlier, later),
        (Some(earlier), None) => (earlier, centre),
        (None, Some(later)) => (centre, later),
        (None, None) => return NO_MOTION,
    };
    apparent_motion(base, &earlier, &later)
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Ecliptic position and speed of `body` in `frame` at `instant` (TDB).
    ///
    /// The longitude is exactly the one [`EventEngine::longitude_at`] returns,
    /// so a position read here is consistent with the crossings the engine
    /// finds in the same frame.
    ///
    /// - [`CrossingFrame::GeocentricApparentOfDate`]: the apparent place
    ///   (light-time with annual aberration, precession, nutation) in the true
    ///   ecliptic of date, from the Earth's centre. The speed is the one a
    ///   `pleiades-core` apparent chart reports.
    /// - [`CrossingFrame::Heliocentric`]: the geometric place from the Sun (no
    ///   light-time, no aberration) in the true ecliptic and equinox of date,
    ///   as Swiss Ephemeris `SEFLG_HELCTR`.
    ///
    /// Longitude and latitude are degrees, distance is AU; speeds are per day.
    /// The speed is the backend's own speed plus the rate of the frame or
    /// apparent-place correction, differenced over ±0.5 day (one-sided at the
    /// edges of the window). A speed channel is `None` when the backend
    /// reports no speed to derive it from.
    ///
    /// # Errors
    ///
    /// The same as [`EventEngine::longitude_at`]:
    /// [`EventError::OutOfWindow`] outside the packaged 1900–2100 window,
    /// [`EventError::UnsupportedFrame`] for a heliocentric Sun or Moon,
    /// [`EventError::MissingCoordinates`] when the backend returns no ecliptic
    /// place or no distance, and [`EventError::Backend`] for a backend failure.
    ///
    /// ```
    /// use pleiades_data::packaged_backend;
    /// use pleiades_events::{CrossingFrame, EventEngine};
    /// use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
    ///
    /// let engine = EventEngine::new(packaged_backend());
    /// let t = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    ///
    /// let helio = engine
    ///     .position_at(CelestialBody::Mars, CrossingFrame::Heliocentric, t)
    ///     .unwrap();
    /// // Mars is 1.38–1.67 AU from the Sun and always moves direct around it.
    /// assert!((1.38..1.67).contains(&helio.ecliptic.distance_au.unwrap()));
    /// assert!(helio.motion.longitude_deg_per_day.unwrap() > 0.4);
    ///
    /// let geo = engine
    ///     .position_at(CelestialBody::Mars, CrossingFrame::GeocentricApparentOfDate, t)
    ///     .unwrap();
    /// let lon = engine
    ///     .longitude_at(CelestialBody::Mars, CrossingFrame::GeocentricApparentOfDate, t)
    ///     .unwrap();
    /// assert_eq!(geo.ecliptic.longitude, lon);
    /// ```
    pub fn position_at(
        &self,
        body: CelestialBody,
        frame: CrossingFrame,
        instant: Instant,
    ) -> Result<EclipticPosition, EventError> {
        let jd = instant.julian_day.days();
        self.check_window(jd)?;
        if matches!(frame, CrossingFrame::Heliocentric)
            && matches!(body, CelestialBody::Sun | CelestialBody::Moon)
        {
            return Err(EventError::UnsupportedFrame {
                detail: format!("heliocentric position is undefined for {:?}", body),
            });
        }
        let centre = sample(&self.backend, &body, frame, jd)?;
        let motion = motion(&self.backend, &body, frame, jd, &centre);
        Ok(EclipticPosition {
            body,
            frame,
            instant,
            ecliptic: centre.corrected,
            motion,
        })
    }
}
```

Compile notes:
- `EventEngine::backend` and `check_window` are already `pub(crate)`; `body_label` is `pub(crate)` in `crossings.rs`.
- If `EclipticCoordinates` or `Motion` do not implement `serde` traits under this crate's `serde` feature, check how `Crossing` gets `Longitude` serialised: `crates/pleiades-events/Cargo.toml` `[features]` forwards `serde` to `pleiades-types/serde`. The same forwarding covers these types.
- If `Longitude` does not implement `PartialEq` (needed by the doctest's `assert_eq!`), compare `.degrees()` instead.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p pleiades-events --test position`
Expected: 10 passed.

If `geocentric_position_and_speed_match_the_chart_layer` fails, do not widen its tolerances. Print the differences and report them: a difference means the two paths compute the apparent place or its speed differently, which is the defect this test exists to catch. The one expected cause is the engine dropping a neighbour the chart keeps (or the reverse) near a window edge; the test's epochs are 10 days inside each edge to avoid that.

Run: `cargo test -p pleiades-events --doc position_at`
Expected: PASS.

Run: `cargo test -p pleiades-events --features serde`
Expected: PASS (the new struct derives serde).

Run: `cargo test -p pleiades-events`
Expected: all pass, including `ephemeris::tests::heliocentric_longitude_bits_are_pinned`.

- [ ] **Step 5: Confirm the crossings golden did not move**

Run: `cargo test -p pleiades-cli crossings`
Expected: PASS with no golden or manifest changes. Then `git status --short` must show no modified file under any `crossings-golden` path.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
git add crates/pleiades-events Cargo.lock
git commit -m "feat(events): position_at returns latitude, distance and speed in both frames (#89)"
```

---

### Task 6: Swiss Ephemeris heliocentric reference tool and corpus

**Files:**
- Create: `tools/se-helio-reference/Cargo.toml`
- Create: `tools/se-helio-reference/src/main.rs`
- Create: `tools/se-helio-reference/Cargo.lock` (generated by cargo)
- Create: `tools/se-helio-reference/LICENSE-NOTES.md` (copy of `tools/se-crossings-reference/LICENSE-NOTES.md`)
- Create: `crates/pleiades-validate/data/helio-position-corpus/helio-position.csv` (generated)
- Modify: `Cargo.toml` (workspace `exclude` list)

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces: a committed CSV whose data rows are
  `jd_tt,body,lon_deg,lat_deg,dist_au,lon_speed_deg_per_day,lat_speed_deg_per_day,dist_speed_au_per_day`
  with `body` one of `Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto`, comment lines starting `#`, and one header line starting `jd_tt`.

- [ ] **Step 1: Write the tool**

`tools/se-helio-reference/Cargo.toml`:

```toml
[package]
name = "se-helio-reference"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
swisseph = "0.1.1"
libswisseph-sys = "0.1.2"
```

`tools/se-helio-reference/src/main.rs`:

```rust
//! Emits a Swiss Ephemeris heliocentric reference corpus (Mercury–Pluto) to
//! STDOUT as CSV: position and speed from
//! `swe_calc(jd_et, body, SEFLG_MOSEPH | SEFLG_HELCTR | SEFLG_SPEED)`.
//!
//! Frame: true ecliptic and equinox of date, nutation on (SE default — no
//! SEFLG_NONUT, no SEFLG_J2000). Heliocentric places are geometric: SE applies
//! no light-time or aberration under SEFLG_HELCTR. Ephemeris: Moshier
//! (SEFLG_MOSEPH), no data files needed.
//!
//! Grid: every 23 days from 1900-01-01 across the packaged window, plus the
//! window's last instant, so the corpus has rows on both window edges.
//!
//! Two build/run caveats: under devenv's gcc the build needs `CFLAGS=-std=gnu17`
//! (libswisseph-sys otherwise fails with a conflicting `getenv` declaration), and
//! `devenv shell` prints a banner line to stdout that must be removed from the
//! top of the CSV (the file must start with the `# Source:` line).
//!
//! Build inside `devenv shell` (provides clang/libclang/LIBCLANG_PATH):
//! `devenv shell -- env CFLAGS=-std=gnu17 cargo run --release \
//!    --manifest-path tools/se-helio-reference/Cargo.toml \
//!    > crates/pleiades-validate/data/helio-position-corpus/helio-position.csv`

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::swe_calc;

const SEFLG_MOSEPH: c_int = 4;
const SEFLG_HELCTR: c_int = 8;
const SEFLG_SPEED: c_int = 256;

/// (Swiss Ephemeris body id, name as written to the CSV).
const BODIES: [(c_int, &str); 8] = [
    (2, "Mercury"),
    (3, "Venus"),
    (4, "Mars"),
    (5, "Jupiter"),
    (6, "Saturn"),
    (7, "Uranus"),
    (8, "Neptune"),
    (9, "Pluto"),
];

const JD_START_TT: f64 = 2_415_020.5; // 1900-01-01, pleiades-events WINDOW_START_JD
const JD_END_TT: f64 = 2_488_069.5; //   pleiades-events WINDOW_END_JD
const STEP_DAYS: f64 = 23.0;

fn se_state(jd_tt: f64, body: c_int, name: &str) -> [f64; 6] {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(
            jd_tt,
            body,
            SEFLG_MOSEPH | SEFLG_HELCTR | SEFLG_SPEED,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        let msg = unsafe { CStr::from_ptr(serr.as_ptr() as *const c_char) }
            .to_string_lossy()
            .into_owned();
        panic!("swe_calc({name}) failed at jd_tt={jd_tt}: {msg}");
    }
    assert!(
        xx.iter().all(|v| v.is_finite()),
        "non-finite SE {name} result at jd_tt={jd_tt}"
    );
    xx[0] = xx[0].rem_euclid(360.0);
    xx
}

fn emit(jd: f64) {
    for (body, name) in BODIES {
        let [lon, lat, dist, lon_speed, lat_speed, dist_speed] = se_state(jd, body, name);
        println!(
            "{jd:.1},{name},{lon:.9},{lat:.9},{dist:.12},{lon_speed:.12},{lat_speed:.12},{dist_speed:.14}"
        );
    }
}

fn main() {
    println!(
        "# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), swe_calc bodies 2..=9 (Mercury..Pluto),"
    );
    println!(
        "# iflag=SEFLG_MOSEPH|SEFLG_HELCTR|SEFLG_SPEED (Moshier, no data files). Frame: heliocentric,"
    );
    println!(
        "# true ecliptic and equinox of date, nutation on, geometric (no light-time, no aberration)."
    );
    println!(
        "# Columns: longitude/latitude (deg), distance (AU), then their speeds (deg/day, deg/day, AU/day)."
    );
    println!(
        "jd_tt,body,lon_deg,lat_deg,dist_au,lon_speed_deg_per_day,lat_speed_deg_per_day,dist_speed_au_per_day"
    );
    let mut jd = JD_START_TT;
    while jd < JD_END_TT {
        emit(jd);
        jd += STEP_DAYS;
    }
    emit(JD_END_TT);
}
```

Copy the license notes: `cp tools/se-crossings-reference/LICENSE-NOTES.md tools/se-helio-reference/LICENSE-NOTES.md`.

In the root `Cargo.toml`, add `"tools/se-helio-reference"` to the `[workspace] exclude = [...]` array, after `"tools/se-mean-lunar-reference"`.

- [ ] **Step 2: Generate the corpus**

```bash
mkdir -p crates/pleiades-validate/data/helio-position-corpus
devenv shell -- env CFLAGS=-std=gnu17 cargo run --release \
  --manifest-path tools/se-helio-reference/Cargo.toml \
  > crates/pleiades-validate/data/helio-position-corpus/helio-position.csv
```

If `devenv` is not available in this environment, STOP and report: the corpus cannot be produced another way, and it must not be hand-written or derived from pleiades itself.

Remove any banner line before `# Source:` so the file starts with `# Source:`.

Run: `head -6 crates/pleiades-validate/data/helio-position-corpus/helio-position.csv`
Expected: four `#` lines, the header line, then a row starting `2415020.5,Mercury,`.

Run: `grep -c "^24" crates/pleiades-validate/data/helio-position-corpus/helio-position.csv`
Expected: `25424` (3177 grid epochs plus the final edge epoch, 3178 × 8 bodies).

Run: `tail -1 crates/pleiades-validate/data/helio-position-corpus/helio-position.csv`
Expected: a row starting `2488069.5,Pluto,`.

- [ ] **Step 3: Sanity-check the reference by hand**

Run: `grep "^2451547.5,Mars\|^2451547.5,Neptune" crates/pleiades-validate/data/helio-position-corpus/helio-position.csv` (if that epoch is not on the grid, use `awk -F, '$1>2451540 && $1<2451565 && ($2=="Mars"||$2=="Neptune")'` instead).
Expected: Mars distance between 1.38 and 1.67 AU with longitude speed between 0.43 and 0.64 deg/day; Neptune distance near 30.1 AU with longitude speed near 0.006 deg/day. All heliocentric longitude speeds in the file are positive: `awk -F, '/^24/ && $6<=0' <file> | wc -l` prints `0`.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml tools/se-helio-reference/Cargo.toml tools/se-helio-reference/Cargo.lock \
  tools/se-helio-reference/src/main.rs tools/se-helio-reference/LICENSE-NOTES.md \
  crates/pleiades-validate/data/helio-position-corpus/helio-position.csv
git commit -m "test(validate): Swiss Ephemeris heliocentric position and speed reference corpus (#89)"
```

Do not commit `tools/se-helio-reference/target/`.

---

### Task 7: `validate-helio-position` gate

**Files:**
- Create: `crates/pleiades-validate/src/helio_position_validation.rs`
- Create: `crates/pleiades-validate/data/helio-position-corpus/manifest.txt`
- Modify: `crates/pleiades-validate/src/lib.rs` (module and re-exports, next to `mod mean_lunar_validation;`)
- Modify: `crates/pleiades-validate/src/render/cli.rs` (release-path call near line 108, command arm near line 320, help text near line 2273)
- Modify: `crates/pleiades-cli/src/cli.rs` (command arm near line 804)
- Modify: `crates/pleiades-validate/Cargo.toml` only if `pleiades-events` is not already a dependency (check with `grep pleiades-events crates/pleiades-validate/Cargo.toml`; `crossings_validation.rs` already uses it)

**Interfaces:**
- Consumes: `pleiades_events::{CrossingFrame, EventEngine, EclipticPosition}`, `EventEngine::position_at` (Task 5); the corpus (Task 6); `pleiades_apparent::fnv1a64`.
- Produces:
  - `pub fn validate_helio_position_corpus() -> Result<HelioPositionReport, HelioPositionError>`
  - `pub struct HelioPositionReport` with `pub fn summary_line(&self) -> &str`, `pub rows_validated: usize`, `pub rows_skipped_oor: usize`, `pub maxima: HelioMaxima`, `pub pluto_maxima: HelioMaxima`
  - `pub struct HelioMaxima { pub lon_arcsec, pub lat_arcsec, pub dist_rel, pub lon_speed_arcsec_per_day, pub lat_speed_arcsec_per_day, pub dist_speed_au_per_day: f64 }`
  - CLI commands `validate-helio-position` and alias `helio-position-gate`.

- [ ] **Step 1: Write the gate with open ceilings and its tests**

Create `crates/pleiades-validate/src/helio_position_validation.rs`:

```rust
//! Fail-closed gate: `EventEngine::position_at(.., CrossingFrame::Heliocentric, ..)`
//! on the packaged backend vs the committed Swiss Ephemeris
//! `SEFLG_HELCTR | SEFLG_SPEED` reference corpus (Mercury–Pluto, 1900–2100),
//! for longitude, latitude, distance and their speeds (issue #89).
//!
//! The longitude residual carries the known light-time signature of
//! reconstructing the heliocentric vector from the backend's geocentric
//! vectors (see `pleiades-events` `tests/heliocentric.rs`), the same floor
//! `validate-crossings` measures for the heliocentric frame.

use pleiades_apparent::fnv1a64;
use pleiades_data::packaged_backend;
use pleiades_events::{CrossingFrame, EventEngine, EventError};
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/helio-position-corpus/helio-position.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/helio-position-corpus/manifest.txt"
));

/// Fail-closed floor on validated rows (corpus is 25424 rows; both window
/// edges are inside the engine's window, so skips should be zero).
const MIN_ROWS_VALIDATED: usize = 25_400;

/// Per-channel ceilings for one body group.
#[derive(Clone, Copy, Debug)]
struct Ceilings {
    lon_arcsec: f64,
    lat_arcsec: f64,
    dist_rel: f64,
    lon_speed_arcsec_per_day: f64,
    lat_speed_arcsec_per_day: f64,
    dist_speed_au_per_day: f64,
}

// Ceilings are set in Step 4 from the measured maxima. Until then they are
// open so the first run reports the maxima instead of failing.
const PLANET_CEILINGS: Ceilings = Ceilings {
    lon_arcsec: f64::INFINITY,
    lat_arcsec: f64::INFINITY,
    dist_rel: f64::INFINITY,
    lon_speed_arcsec_per_day: f64::INFINITY,
    lat_speed_arcsec_per_day: f64::INFINITY,
    dist_speed_au_per_day: f64::INFINITY,
};
const PLUTO_CEILINGS: Ceilings = PLANET_CEILINGS;

#[derive(Clone, Debug)]
struct Row {
    jd_tt: f64,
    body: CelestialBody,
    body_name: &'static str,
    lon_deg: f64,
    lat_deg: f64,
    dist_au: f64,
    lon_speed: f64,
    lat_speed: f64,
    dist_speed: f64,
}

#[derive(Debug)]
pub enum HelioPositionError {
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
        body: &'static str,
        jd_tt: f64,
        reason: String,
    },
    CeilingExceeded {
        body: &'static str,
        jd_tt: f64,
        kind: &'static str,
        got: f64,
        want: f64,
        residual: f64,
        ceiling: f64,
    },
}

impl std::fmt::Display for HelioPositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedRow(s) => write!(f, "malformed corpus row: {s}"),
            Self::MalformedManifest(s) => write!(f, "malformed manifest: {s}"),
            Self::ChecksumMismatch { got, want } => {
                write!(f, "corpus checksum mismatch: got {got} want {want}")
            }
            Self::ManifestDrift { rows_csv, rows_manifest } => {
                write!(f, "manifest drift: csv has {rows_csv} rows, manifest says {rows_manifest}")
            }
            Self::TooFewRowsValidated { validated, floor } => {
                write!(f, "only {validated} rows validated, floor is {floor}")
            }
            Self::CalculationFailed { body, jd_tt, reason } => {
                write!(f, "{body} calculation failed at jd_tt={jd_tt}: {reason}")
            }
            Self::CeilingExceeded { body, jd_tt, kind, got, want, residual, ceiling } => write!(
                f,
                "{body} {kind} ceiling exceeded at jd_tt={jd_tt}: got {got:.12} want {want:.12} residual {residual:.6e} > ceiling {ceiling:.6e}"
            ),
        }
    }
}

impl std::error::Error for HelioPositionError {}

/// Largest absolute residuals seen for one body group across the corpus.
#[derive(Clone, Copy, Debug, Default)]
pub struct HelioMaxima {
    pub lon_arcsec: f64,
    pub lat_arcsec: f64,
    pub dist_rel: f64,
    pub lon_speed_arcsec_per_day: f64,
    pub lat_speed_arcsec_per_day: f64,
    pub dist_speed_au_per_day: f64,
}

#[derive(Debug)]
pub struct HelioPositionReport {
    pub rows_validated: usize,
    /// Rows skipped because the instant is outside the engine's window.
    pub rows_skipped_oor: usize,
    /// Mercury–Neptune.
    pub maxima: HelioMaxima,
    /// Pluto (served from a different source than the other planets).
    pub pluto_maxima: HelioMaxima,
    /// Mean signed longitude-speed residual (arcsec/day) over Jupiter–Neptune.
    /// A value near ±0.137 would mean the reference and the engine disagree on
    /// whether the speed includes the precession rate.
    pub outer_lon_speed_mean_signed_arcsec_per_day: f64,
    summary_line: String,
}

impl HelioPositionReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

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
        _ => return None,
    })
}

fn parse_corpus(csv: &str) -> Result<Vec<Row>, HelioPositionError> {
    let mut rows = Vec::new();
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("jd_tt") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 8 {
            return Err(HelioPositionError::MalformedRow(format!(
                "expected 8 fields, got {} in {line}",
                fields.len()
            )));
        }
        let num = |i: usize| -> Result<f64, HelioPositionError> {
            let v = fields[i].parse::<f64>().map_err(|e| {
                HelioPositionError::MalformedRow(format!("field {i}: {e} in {line}"))
            })?;
            if v.is_finite() {
                Ok(v)
            } else {
                Err(HelioPositionError::MalformedRow(format!(
                    "field {i} is not finite in {line}"
                )))
            }
        };
        let (body, body_name) = body_from_name(fields[1]).ok_or_else(|| {
            HelioPositionError::MalformedRow(format!("unknown body {} in {line}", fields[1]))
        })?;
        let dist_au = num(4)?;
        if dist_au <= 0.0 {
            return Err(HelioPositionError::MalformedRow(format!(
                "non-positive distance in {line}"
            )));
        }
        rows.push(Row {
            jd_tt: num(0)?,
            body,
            body_name,
            lon_deg: num(2)?,
            lat_deg: num(3)?,
            dist_au,
            lon_speed: num(5)?,
            lat_speed: num(6)?,
            dist_speed: num(7)?,
        });
    }
    Ok(rows)
}

fn parse_manifest(manifest: &str) -> Result<(usize, u64), HelioPositionError> {
    let line = manifest
        .lines()
        .find(|l| l.trim_start().starts_with("slice"))
        .ok_or_else(|| HelioPositionError::MalformedManifest("no slice line".into()))?;
    let mut rows = None;
    let mut checksum = None;
    for tok in line.split_whitespace() {
        if let Some(v) = tok.strip_prefix("rows=") {
            rows = Some(
                v.parse::<usize>()
                    .map_err(|e| HelioPositionError::MalformedManifest(format!("rows: {e}")))?,
            );
        } else if let Some(v) = tok.strip_prefix("checksum=") {
            checksum = Some(
                v.parse::<u64>()
                    .map_err(|e| HelioPositionError::MalformedManifest(format!("checksum: {e}")))?,
            );
        }
    }
    Ok((
        rows.ok_or_else(|| HelioPositionError::MalformedManifest("rows= missing".into()))?,
        checksum.ok_or_else(|| HelioPositionError::MalformedManifest("checksum= missing".into()))?,
    ))
}

fn wrap_deg(got_deg: f64, want_deg: f64) -> f64 {
    (got_deg - want_deg + 180.0).rem_euclid(360.0) - 180.0
}

fn validate(csv: &str, manifest: &str) -> Result<HelioPositionReport, HelioPositionError> {
    let (manifest_rows, manifest_checksum) = parse_manifest(manifest)?;
    let got_checksum = fnv1a64(csv);
    if got_checksum != manifest_checksum {
        return Err(HelioPositionError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let rows = parse_corpus(csv)?;
    if rows.len() != manifest_rows {
        return Err(HelioPositionError::ManifestDrift {
            rows_csv: rows.len(),
            rows_manifest: manifest_rows,
        });
    }

    let engine = EventEngine::new(packaged_backend());
    let mut maxima = HelioMaxima::default();
    let mut pluto_maxima = HelioMaxima::default();
    let mut validated = 0usize;
    let mut skipped_oor = 0usize;
    let (mut outer_sum, mut outer_count) = (0.0_f64, 0usize);

    for row in &rows {
        let failed = |reason: String| HelioPositionError::CalculationFailed {
            body: row.body_name,
            jd_tt: row.jd_tt,
            reason,
        };
        // The corpus epoch is TT; the engine reads the Julian day as TDB. The
        // two differ by under 2 ms, far below every ceiling here.
        let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tdb);
        let position = match engine.position_at(row.body.clone(), CrossingFrame::Heliocentric, instant) {
            Ok(position) => position,
            Err(EventError::OutOfWindow { .. }) => {
                skipped_oor += 1;
                continue;
            }
            Err(e) => return Err(failed(e.to_string())),
        };
        let got_dist = position
            .ecliptic
            .distance_au
            .ok_or_else(|| failed("no distance".into()))?;
        let speed = |value: Option<f64>, name: &str| {
            value.ok_or_else(|| failed(format!("no {name} speed")))
        };
        let got_lon_speed = speed(position.motion.longitude_deg_per_day, "longitude")?;
        let got_lat_speed = speed(position.motion.latitude_deg_per_day, "latitude")?;
        let got_dist_speed = speed(position.motion.distance_au_per_day, "distance")?;
        let got_lon = position.ecliptic.longitude.degrees();
        let got_lat = position.ecliptic.latitude.degrees();

        let is_pluto = row.body == CelestialBody::Pluto;
        let ceilings = if is_pluto { PLUTO_CEILINGS } else { PLANET_CEILINGS };
        let lon_speed_signed = (got_lon_speed - row.lon_speed) * 3600.0;
        let checks = [
            ("longitude_arcsec", got_lon, row.lon_deg, (wrap_deg(got_lon, row.lon_deg) * 3600.0).abs(), ceilings.lon_arcsec),
            ("latitude_arcsec", got_lat, row.lat_deg, ((got_lat - row.lat_deg) * 3600.0).abs(), ceilings.lat_arcsec),
            ("distance_rel", got_dist, row.dist_au, ((got_dist - row.dist_au) / row.dist_au).abs(), ceilings.dist_rel),
            ("longitude_speed_arcsec_per_day", got_lon_speed, row.lon_speed, lon_speed_signed.abs(), ceilings.lon_speed_arcsec_per_day),
            ("latitude_speed_arcsec_per_day", got_lat_speed, row.lat_speed, ((got_lat_speed - row.lat_speed) * 3600.0).abs(), ceilings.lat_speed_arcsec_per_day),
            ("distance_speed_au_per_day", got_dist_speed, row.dist_speed, (got_dist_speed - row.dist_speed).abs(), ceilings.dist_speed_au_per_day),
        ];
        for (kind, got, want, residual, ceiling) in checks {
            // A NaN residual must fail closed too.
            if residual.is_nan() || residual > ceiling {
                return Err(HelioPositionError::CeilingExceeded {
                    body: row.body_name,
                    jd_tt: row.jd_tt,
                    kind,
                    got,
                    want,
                    residual,
                    ceiling,
                });
            }
        }
        let group = if is_pluto { &mut pluto_maxima } else { &mut maxima };
        group.lon_arcsec = group.lon_arcsec.max(checks[0].3);
        group.lat_arcsec = group.lat_arcsec.max(checks[1].3);
        group.dist_rel = group.dist_rel.max(checks[2].3);
        group.lon_speed_arcsec_per_day = group.lon_speed_arcsec_per_day.max(checks[3].3);
        group.lat_speed_arcsec_per_day = group.lat_speed_arcsec_per_day.max(checks[4].3);
        group.dist_speed_au_per_day = group.dist_speed_au_per_day.max(checks[5].3);
        if matches!(
            row.body,
            CelestialBody::Jupiter | CelestialBody::Saturn | CelestialBody::Uranus | CelestialBody::Neptune
        ) {
            outer_sum += lon_speed_signed;
            outer_count += 1;
        }
        validated += 1;
    }
    let floor = MIN_ROWS_VALIDATED.min(manifest_rows);
    if validated < floor {
        return Err(HelioPositionError::TooFewRowsValidated { validated, floor });
    }
    let outer_mean = if outer_count == 0 { 0.0 } else { outer_sum / outer_count as f64 };

    let group = |m: &HelioMaxima| {
        format!(
            "lon {:.3}\" lat {:.3}\" dist {:.2e} rel, speed lon {:.4}\"/d lat {:.4}\"/d dist {:.2e} AU/d",
            m.lon_arcsec, m.lat_arcsec, m.dist_rel,
            m.lon_speed_arcsec_per_day, m.lat_speed_arcsec_per_day, m.dist_speed_au_per_day
        )
    };
    let summary_line = format!(
        "Helio-position gate: {validated} rows validated ({skipped_oor} oor-skipped) vs Swiss Ephemeris SEFLG_HELCTR|SEFLG_SPEED, \
         Mercury-Neptune max {}; Pluto max {}; outer-planet mean signed lon speed {:+.4}\"/d",
        group(&maxima),
        group(&pluto_maxima),
        outer_mean,
    );
    Ok(HelioPositionReport {
        rows_validated: validated,
        rows_skipped_oor: skipped_oor,
        maxima,
        pluto_maxima,
        outer_lon_speed_mean_signed_arcsec_per_day: outer_mean,
        summary_line,
    })
}

pub fn validate_helio_position_corpus() -> Result<HelioPositionReport, HelioPositionError> {
    validate(CORPUS_CSV, MANIFEST)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helio_position_gate_passes_within_ceilings() {
        let report = validate_helio_position_corpus().expect("helio-position gate passes");
        assert!(report.rows_validated >= MIN_ROWS_VALIDATED);
        eprintln!("{}", report.summary_line());
    }

    #[test]
    fn tampered_corpus_fails_the_checksum() {
        let tampered = CORPUS_CSV.replacen("2415020.5,", "2415020.5, ", 1);
        assert!(matches!(
            validate(&tampered, MANIFEST),
            Err(HelioPositionError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn manifest_row_count_drift_fails_closed() {
        let checksum = fnv1a64(CORPUS_CSV);
        let manifest = format!(
            "slice helio-position file=helio-position.csv role=helio-position rows=25423 checksum={checksum}"
        );
        assert!(matches!(
            validate(CORPUS_CSV, &manifest),
            Err(HelioPositionError::ManifestDrift { rows_csv: 25424, rows_manifest: 25423 })
        ));
    }

    #[test]
    fn manifest_without_a_slice_line_is_rejected() {
        assert!(matches!(
            validate(CORPUS_CSV, "rows=25424"),
            Err(HelioPositionError::MalformedManifest(_))
        ));
    }

    #[test]
    fn malformed_rows_are_rejected() {
        for bad in [
            "2451545.0,Mars,1.0,0.0",
            "2451545.0,Mars,NaN,0.0,1.5,0.5,0.0,0.0",
            "2451545.0,Vulcan,1.0,0.0,1.5,0.5,0.0,0.0",
            "2451545.0,Mars,1.0,0.0,0.0,0.5,0.0,0.0",
        ] {
            let manifest = format!("slice x rows=1 checksum={}", fnv1a64(bad));
            assert!(
                matches!(validate(bad, &manifest), Err(HelioPositionError::MalformedRow(_))),
                "{bad}"
            );
        }
    }

    /// One real corpus row with field `index` replaced by `f(value)`.
    fn shifted_row(body: &str, index: usize, f: impl Fn(f64) -> f64) -> (String, String) {
        let line = CORPUS_CSV
            .lines()
            .find(|l| l.starts_with("245") && l.split(',').nth(1) == Some(body))
            .expect("a data row near J2000");
        let mut fields: Vec<String> = line.split(',').map(str::to_string).collect();
        let value: f64 = fields[index].parse().unwrap();
        fields[index] = format!("{:.12}", f(value));
        let csv = fields.join(",");
        let manifest = format!("slice x rows=1 checksum={}", fnv1a64(&csv));
        (csv, manifest)
    }

    #[test]
    fn a_shifted_reference_longitude_exceeds_the_ceiling() {
        // 0.1° = 360″, well above the light-time floor.
        let (csv, manifest) = shifted_row("Mars", 2, |v| (v + 0.1).rem_euclid(360.0));
        assert!(matches!(
            validate(&csv, &manifest),
            Err(HelioPositionError::CeilingExceeded { body: "Mars", kind: "longitude_arcsec", .. })
        ));
    }

    #[test]
    fn a_shifted_reference_speed_exceeds_the_ceiling() {
        // 0.01 deg/day = 36″/day.
        let (csv, manifest) = shifted_row("Mars", 5, |v| v + 0.01);
        assert!(matches!(
            validate(&csv, &manifest),
            Err(HelioPositionError::CeilingExceeded {
                body: "Mars",
                kind: "longitude_speed_arcsec_per_day",
                ..
            })
        ));
    }
}
```

In `crates/pleiades-validate/src/lib.rs`, next to `mod mean_lunar_validation;` add `mod helio_position_validation;`, and next to its `pub use` add:

```rust
pub use helio_position_validation::{
    validate_helio_position_corpus, HelioMaxima, HelioPositionError, HelioPositionReport,
};
```

- [ ] **Step 2: Create the manifest with the real checksum**

Create `crates/pleiades-validate/data/helio-position-corpus/manifest.txt` with one line:

```
slice helio-position file=helio-position.csv role=helio-position rows=25424 checksum=0
```

Run: `cargo test -p pleiades-validate helio_position_validation::tests::helio_position_gate_passes_within_ceilings -- --nocapture`
Expected: FAIL with `ChecksumMismatch { got: <N>, want: 0 }`. Put `<N>` in the manifest in place of `0`.

If `rows=25424` does not match the row count from Task 6 Step 2, the corpus is not the one this plan describes: STOP and re-check Task 6 rather than editing the number.

- [ ] **Step 3: Measure**

Run: `cargo test -p pleiades-validate helio_position_validation::tests::helio_position_gate_passes_within_ceilings -- --nocapture`
Expected: PASS (ceilings are open), printing the `Helio-position gate: ...` summary line. The two `a_shifted_reference_*` tests FAIL at this point because the ceilings are infinite; that is expected until Step 4.

Record the full summary line. Then check it against these stop conditions before setting any ceiling:

1. `rows validated` must be 25424 with 0 skipped. Otherwise STOP and report which rows were skipped.
2. Mercury–Neptune longitude maximum must be at most 50″ and Pluto's at most 5″ (the existing `validate-crossings` ceilings for the same quantity). Above that, STOP and report: the spec requires investigation, not a wider ceiling.
3. `outer-planet mean signed lon speed` must be within ±0.03″/day. A value near ±0.137″/day means the engine and Swiss Ephemeris disagree on whether the speed includes the precession rate: STOP and report (spec "Risks: SE speed convention").
4. Latitude, distance and the three speed maxima have no prior measurement. Report them as measured. If any looks out of class with the longitude residual (latitude above 50″; longitude or latitude speed above 5″/day; relative distance above 1e-3), STOP and report before setting a ceiling over it.

- [ ] **Step 4: Set the ceilings from the measurement**

Rule, as for `validate-mean-lunar-points`: each ceiling is 1.5 × the measured maximum of that channel for that group, rounded **up** to two significant figures.

Replace the two open `Ceilings` constants. The shape is fixed; each number comes from the Step 3 summary line by the rule above, and each line carries its measured maximum in a trailing comment:

```rust
// Ceilings — 1.5 × the measured maximum of each channel, rounded up to two
// significant figures, measured <date of the run> over 25424 rows. The
// longitude residual is dominated by the light-time signature of the
// planet-minus-Sun reconstruction, as in validate-crossings.
const PLANET_CEILINGS: Ceilings = Ceilings {
    lon_arcsec: <1.5 × max>,                // measured max <value>"
    lat_arcsec: <1.5 × max>,                // measured max <value>"
    dist_rel: <1.5 × max>,                  // measured max <value>
    lon_speed_arcsec_per_day: <1.5 × max>,  // measured max <value>"/day
    lat_speed_arcsec_per_day: <1.5 × max>,  // measured max <value>"/day
    dist_speed_au_per_day: <1.5 × max>,     // measured max <value> AU/day
};
const PLUTO_CEILINGS: Ceilings = Ceilings {
    // same six fields, from the Pluto maxima
};
```

The angle-bracketed entries are the only values in this plan that cannot be written in advance, because they are measurements; every one is determined by the rule and the Step 3 output. No field may be left as `f64::INFINITY`.

Run: `cargo test -p pleiades-validate helio_position_validation`
Expected: 8 passed, including both `a_shifted_reference_*` tests. If `a_shifted_reference_speed_exceeds_the_ceiling` still fails, the longitude-speed ceiling is above 36″/day, which contradicts stop condition 4: STOP and report.

- [ ] **Step 5: Wire the gate into the CLIs and the release path**

In `crates/pleiades-validate/src/render/cli.rs`:

After the release-path block near line 108

```rust
    crate::validate_mean_lunar_points_corpus()
        .map_err(|e| format!("mean-lunar-points gate failed: {e}"))?;
```

add:

```rust
    crate::validate_helio_position_corpus()
        .map_err(|e| format!("helio-position gate failed: {e}"))?;
```

After the command arm near line 320 for `Some("validate-mean-lunar-points") | Some("mean-lunar-points-gate")`, add:

```rust
        Some("validate-helio-position") | Some("helio-position-gate") => {
            ensure_no_extra_args(&args[1..], "validate-helio-position")?;
            crate::validate_helio_position_corpus()
                .map(|report| report.summary_line().to_string())
                .map_err(|e| e.to_string())
        }
```

In the help text near line 2273, find the `validate-mean-lunar-points` entry inside the long string and add, directly after it and in the same column layout, an entry for `validate-helio-position` reading `Validate heliocentric position and speed against the Swiss Ephemeris corpus`, plus the `helio-position-gate` alias line in the same style the mean-lunar alias uses.

In `crates/pleiades-cli/src/cli.rs`, after the arm near line 804

```rust
        Some("validate-mean-lunar-points") | Some("mean-lunar-points-gate") => {
            validate_render_cli(args)
        }
```

add:

```rust
        Some("validate-helio-position") | Some("helio-position-gate") => validate_render_cli(args),
```

Run: `grep -rn "mean-lunar-points" --include=*.rs crates/pleiades-validate/src crates/pleiades-cli/src | grep -v mean_lunar_validation.rs`
Expected: every listed location that is a command name, alias, help entry or release-path call now has a `helio-position` sibling next to it. Locations that are about the mean lunar points themselves (claims, chart tests) have none.

- [ ] **Step 6: Run the gate through the CLI and the test suites that pin help text**

Run: `cargo run -p pleiades-validate --release -- validate-helio-position`
Expected: prints the `Helio-position gate: 25424 rows validated (0 oor-skipped) ...` summary line, exit code 0.

Run: `cargo run -p pleiades-cli --release -- validate-helio-position`
Expected: the same line.

Run: `cargo test -p pleiades-validate -p pleiades-cli`
Expected: all pass. A test that pins the help text or a command list will fail if it needs the new entry: update that pinned text to include the new command, and nothing else.

- [ ] **Step 7: Commit**

Put the full measured summary line in the commit body.

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
git add crates/pleiades-validate crates/pleiades-cli
git commit -m "feat(validate): validate-helio-position gate over the Swiss Ephemeris heliocentric corpus (#89)" \
  -m "Measured maxima: <the summary line printed in Step 3>"
```

---

### Task 8: Documentation, compatibility profile and full verification

**Files:**
- Modify: `README.md` (capability table, after the `Longitude crossings` row at line 35)
- Modify: `crates/pleiades-events/README.md`
- Modify: `crates/pleiades-apparent/README.md` (only if it lists the crate's modules)
- Modify: `crates/pleiades-core/src/compatibility/mod.rs` (profile id line 26, checksum, release-note entry after the `#90` entry near line 111)
- Modify: `crates/pleiades-cli/src/cli/tests/summary_commands.rs:440`
- Modify: `crates/pleiades-validate/src/tests/render_request.rs:333`
- Modify: `docs/follow-ups.md` (new FU-16 at the end)
- Modify: `docs/superpowers/specs/2026-10-01-heliocentric-position-design.md` (status line)
- Modify: `spec/*.md` only where a document lists the events surface or the validation gates

**Interfaces:**
- Consumes: the measured summary line from Task 7.
- Produces: nothing code-facing.

- [ ] **Step 1: README capability table**

In `README.md`, after the row beginning `| Longitude crossings |`, add:

```markdown
| Ecliptic position & speed (geocentric apparent, heliocentric) | [`pleiades-events`](crates/pleiades-events) | `validate-helio-position` (heliocentric), `validate-apparent` (geocentric) | arcsecond-class geocentric; tens-of-arcseconds heliocentric |
```

If the Task 7 measurement shows the heliocentric longitude maximum below 10″, write `arcsecond-class` for both instead.

- [ ] **Step 2: Crate README**

In `crates/pleiades-events/README.md`, in the section that describes longitude crossings and `longitude_at` (find it with `grep -n "longitude_at\|crossing" crates/pleiades-events/README.md`), add this paragraph after it. If the README has no such section, add it under a new `## Positions` heading placed after the crossings section:

```markdown
`EventEngine::position_at(body, frame, instant)` returns the full ecliptic
position in either `CrossingFrame`: longitude, latitude and distance, with
their speeds per day. Its longitude is exactly the one `longitude_at` returns,
so a position is consistent with the crossings found in the same frame.

- `GeocentricApparentOfDate`: the apparent place in the true ecliptic of date,
  with the same speed a `pleiades-core` apparent chart reports.
- `Heliocentric`: the geometric place from the Sun in the true ecliptic and
  equinox of date (Swiss Ephemeris `SEFLG_HELCTR`), gated by
  `validate-helio-position`. The Sun and Moon are an error in this frame.

A speed channel is `None` when the backend reports no speed to derive it from.
```

- [ ] **Step 3: Compatibility profile**

In `crates/pleiades-core/src/compatibility/mod.rs`:

- Line 26: change `pleiades-compatibility-profile/0.7.16` to `pleiades-compatibility-profile/0.7.17`.
- After the release-note string that begins `"Mean lunar points on the packaged backend (issue #90) additions:` add a new element in the same list:

```rust
            "Ecliptic position with latitude and speed (issue #89) additions: EventEngine::position_at returns an EclipticPosition (longitude, latitude, distance and their speeds per day) in either CrossingFrame — geocentric apparent of date, or heliocentric geometric in the true ecliptic and equinox of date as Swiss Ephemeris SEFLG_HELCTR. Its longitude is bit-identical to EventEngine::longitude_at. Heliocentric values are gated against a Swiss Ephemeris SEFLG_HELCTR|SEFLG_SPEED corpus by validate-helio-position; geocentric values equal the apparent chart placement. Heliocentric Sun and Moon remain unsupported.",
```

- In `crates/pleiades-cli/src/cli/tests/summary_commands.rs:440` and `crates/pleiades-validate/src/tests/render_request.rs:333`, change `0.7.16` to `0.7.17`.

Run: `cargo test -p pleiades-core rendered_profile_matches_pinned_content_checksum`
Expected: FAIL, reporting the new checksum. Set `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` to the reported value (keep the `0x____ ____ ____ ____` underscore grouping), following the instructions in that constant's doc comment, and re-run.
Expected: PASS.

Run: `grep -rn "0\.7\.16" --include=*.rs --include=*.md . | grep -v "^./target" | grep -v "docs/superpowers"`
Expected: no output.

- [ ] **Step 4: Follow-ups entry and spec status**

Append to `docs/follow-ups.md`:

```markdown
## FU-16: A public ecliptic position with latitude and speed (issue #89)

**Status:** resolved (<date of completion, YYYY-MM-DD>) · Spec
`docs/superpowers/specs/2026-10-01-heliocentric-position-design.md`, plan
`docs/superpowers/plans/2026-10-01-heliocentric-position.md`.

**What:** the only public heliocentric read was `EventEngine::longitude_at`,
which returned a longitude alone. `EventEngine::position_at` now returns
longitude, latitude, distance and their speeds in either `CrossingFrame`, with
a longitude bit-identical to `longitude_at`. The speed is the backend's speed
plus the rate of the frame or apparent-place correction, differenced over
±0.5 day, using the helper shared with the chart layer
(`pleiades_apparent::motion`).

**Gate:** `validate-helio-position` (25424-row Swiss Ephemeris
`SEFLG_HELCTR | SEFLG_SPEED` corpus, Mercury–Pluto, 1900–2100; measured:
<the summary line from Task 7 Step 3>). The geocentric frame is pinned to the
apparent chart placement by `crates/pleiades-events/tests/position.rs`.

**Still open:** the heliocentric longitude carries the light-time signature of
the planet-minus-Sun reconstruction (also seen by `validate-crossings`);
removing it would move the crossings golden and is a separate change.

**Also in this change:** toolchain and MSRV moved from Rust 1.98.1 to 1.99.0.
```

Replace the two angle-bracketed entries with the completion date and the measured summary line.

In the spec, change the status line to `**Status:** implemented (<same date>) ·`.

- [ ] **Step 5: Spec documents**

Run: `grep -n -i "longitude_at\|validate-crossings\|validate-mean-lunar-points" SPEC.md spec/*.md docs/*.md | grep -v follow-ups`
Expected: a list of places that enumerate the events surface or the gates. For each list of gates that includes `validate-mean-lunar-points`, add `validate-helio-position` next to it. For each description of the events surface that names `longitude_at`, add `position_at`. If the command prints nothing, no spec document enumerates them and nothing changes.

- [ ] **Step 6: Full verification**

Run: `mise run ci`
Expected: PASS (fmt, clippy, blocking tests, doctests, and the other blocking checks).

Run: `cargo run -p pleiades-validate --release -- validate-crossings`
Expected: PASS, with group maxima equal to those recorded in `crates/pleiades-validate/src/crossings_validation.rs` (geo Sun 0.322″, geo Moon 2.606″, geo planets 0.483″, helio non-Pluto 35.090″). Any change in those figures means `longitude_at` moved: STOP and report.

Run: `cargo test -p pleiades-validate --release helio_position_validation mean_lunar_validation crossings`
Expected: PASS.

Run: `git status --short`
Expected: only the files this task edited are modified; nothing under a `crossings-golden` path, and no stray files under `fuzz/corpus/` or `tools/*/target/`.

- [ ] **Step 7: Commit**

```bash
cargo fmt --all
git add README.md crates/pleiades-events/README.md crates/pleiades-core/src/compatibility/mod.rs \
  crates/pleiades-cli/src/cli/tests/summary_commands.rs \
  crates/pleiades-validate/src/tests/render_request.rs docs/follow-ups.md \
  docs/superpowers/specs/2026-10-01-heliocentric-position-design.md
git add -u spec SPEC.md docs
git commit -m "docs: document position_at and bump the compatibility profile to 0.7.17 (#89)"
```

---

## Self-review record

- **Spec coverage:** Public API → Task 5. Shared speed helper → Task 2. Geocentric and heliocentric position and speed → Tasks 3–5. Errors → Task 5 (`guards_match_longitude_at`). SE tool and corpus → Task 6. Gate, row floor, release path → Task 7. Chart agreement → Task 5. Unit and regression tests → Tasks 2–5. Toolchain and MSRV → Task 1. Documentation and compatibility profile → Task 8. Out-of-scope items have no task.
- **Measured values:** the gate ceilings, the manifest checksum, the pinned longitude bits, the profile checksum and the follow-up's summary line are produced by running the code. Each has an explicit procedure and, where a wrong value is possible, a stop condition.
- **Type consistency:** `apparent_motion(base, &earlier, &later)`, `Correction::between(&corrected, &base)`, `CorrectionSample { julian_day, correction }`, `cartesian_velocity(lon, lat, r, Option<Motion>) -> Option<[f64; 3]>`, `spherical_rates([f64; 3], [f64; 3]) -> Motion`, `heliocentric_j2000(..) -> HeliocentricJ2000 { position, velocity }`, `heliocentric_of_date([f64; 3], f64)`, `j2000_spherical([f64; 3])`, `read_mean_ecliptic_with_motion(..) -> ((f64, f64, f64), Option<Motion>)` are used with the same names and shapes in every task.
