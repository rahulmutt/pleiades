# Eclipse Subsystem Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `pleiades-eclipse` crate that computes global/geocentric solar and lunar eclipses over 1900–2100 CE from pleiades' own Sun+Moon positions, proven by a fail-closed `validate-eclipses` gate against NASA's Five Millennium Canon.

**Architecture:** A new pure-Rust crate sits above the existing backends. An `EclipseEngine<B: EphemerisBackend>` drives any Sun+Moon backend (in practice `pleiades_data::packaged_backend()`): it finds syzygies by root-finding on the Sun−Moon elongation, classifies each with shadow-cone geometry, and refines the greatest-eclipse instant. It owns no ephemeris data; accuracy inherits from the already-gated positions. A new `validate-eclipses` gate in `pleiades-validate` recomputes an exhaustive committed NASA fixture and fails closed on drift.

**Tech Stack:** Rust (edition 2021, workspace `rust-version = 1.96.0`), pure-Rust no-`unsafe` no-C-deps style, `include_str!`-embedded CSV fixtures, the existing `EphemerisBackend` trait contract.

## Global Constraints

- Rust edition `2021`, MSRV `rust-version = 1.96.0`, crate `version = 0.2.0` — all inherited via `.workspace = true`.
- License `MIT OR Apache-2.0`; every crate carries `keywords`/`categories`/`repository`/`homepage` via workspace inheritance.
- Pure Rust only: no `unsafe`, no C/native build dependencies, no network access in library or gate code (the fixture is committed, not fetched).
- Backend contract is exactly: `fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError>`. `EphemerisRequest { body, instant, observer, frame, zodiac_mode, apparent }`. `EphemerisResult.ecliptic: Option<EclipticCoordinates>` where `EclipticCoordinates { longitude: Longitude, latitude: Latitude, distance_au: Option<f64> }`.
- Time: `Instant::new(julian_day: JulianDay, scale: TimeScale)`, `JulianDay::from_days(f64)`, `Instant::add_seconds(f64)`, `Instant::mean_obliquity() -> Angle`. Angles: `Longitude::from_degrees`, `.degrees()`, `.normalized_0_360()`, `.radians()`.
- Validation gates return `Result<Report, Error>` and **fail closed** (the `validate_house_corpus` / `validate-corpus` pattern). Tolerances for this gate: `greatest_eclipse` ≤ 60 s, `magnitude` ≤ 0.01, `eclipse_type` exact, `saros_series` exact, `eclipsed_longitude` ≤ 1.0″.
- Coverage window: 1900–2100 CE (JD 2415020.5 … 2488069.5). Out-of-window requests return a structured error, never a silent guess.
- Eclipse `eclipsed_longitude` is **tropical ecliptic of date**; never apply ayanamsa in this crate.
- Penumbral lunar eclipses are **included** (the NASA canon lists them).
- Physical constants (use these exact values, km): solar radius `R_sun = 696_000.0`; lunar radius `R_moon = 1_737.4`; Earth equatorial radius `R_earth = 6_378.137`; `AU_KM = 149_597_870.7`. Earth shadow enlargement factor for the umbra/penumbra (Danjon-free geometric, the value the NASA canon uses): inflate Earth's radius by `1.02` when forming the lunar shadow cones.

---

## File Structure

**New crate `crates/pleiades-eclipse/`:**
- `Cargo.toml` — manifest, workspace-inherited fields, deps on `pleiades-types`, `pleiades-backend`, `pleiades-apparent`, `pleiades-time`.
- `src/lib.rs` — module wiring + public re-exports + crate rustdoc.
- `src/types.rs` — `EclipseKind`, `SolarEclipseType`, `LunarEclipseType`, `EclipseType`, `EclipseFilter`, `Node`, `GeoLocation`, `Eclipse`.
- `src/error.rs` — `EclipseError`.
- `src/ephemeris.rs` — thin `SunMoonSample` reader: turns a backend + JD into geocentric Sun/Moon ecliptic longitude, latitude, distance.
- `src/syzygy.rs` — new/full-moon root-finding over a range.
- `src/geometry.rs` — apparent radii, parallax, gamma, solar+lunar classification, magnitude.
- `src/saros.rs` — Saros series numbering.
- `src/engine.rs` — `EclipseEngine<B>`: `eclipses_in_range` / `next_eclipse` / `previous_eclipse`, greatest-eclipse refinement, `near_node`, `greatest_eclipse_location`.
- `README.md` — crate readme (matches the other crates' style).

**`pleiades-validate` additions:**
- `crates/pleiades-validate/data/eclipses-corpus/eclipses.csv` — committed NASA fixture (exhaustive 1900–2100).
- `crates/pleiades-validate/data/eclipses-corpus/MANIFEST.md` — provenance.
- `crates/pleiades-validate/src/eclipse_validation.rs` — `validate_eclipse_corpus()` gate + report types.
- `crates/pleiades-validate/src/lib.rs` — module + `render_cli` dispatch for `validate-eclipses` and `eclipses`; wire into `release-smoke`/`release-gate`.

**Workspace wiring:**
- `Cargo.toml` (root) — add member + `[workspace.dependencies] pleiades-eclipse`.
- `crates/pleiades-cli/src/cli.rs` — forward `validate-eclipses` and `eclipses` to `validate_render_cli` (the existing forwarding pattern).

---

## Task 1: Scaffold the `pleiades-eclipse` crate

**Files:**
- Create: `crates/pleiades-eclipse/Cargo.toml`
- Create: `crates/pleiades-eclipse/src/lib.rs`
- Modify: `Cargo.toml` (root) — add member + workspace dependency
- Test: `crates/pleiades-eclipse/tests/smoke.rs`

**Interfaces:**
- Produces: the crate `pleiades_eclipse` (empty but compiling), buildable as a workspace member.

- [ ] **Step 1: Create the manifest**

`crates/pleiades-eclipse/Cargo.toml`:
```toml
[package]
name = "pleiades-eclipse"
description = "Global geocentric solar and lunar eclipse computation for the pleiades astrology workspace: type, greatest-eclipse time, magnitude, gamma, Saros series, eclipsed longitude, and greatest-eclipse location."
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

[package.metadata.docs.rs]
all-features = true
```

- [ ] **Step 2: Create an empty README and lib.rs**

`crates/pleiades-eclipse/README.md`:
```markdown
# pleiades-eclipse

Global geocentric solar and lunar eclipse computation for the `pleiades`
astrology workspace, derived from validated Sun and Moon positions.
```

`crates/pleiades-eclipse/src/lib.rs`:
```rust
//! Global geocentric solar and lunar eclipse computation, derived entirely from
//! pleiades' validated Sun and Moon positions. Scope: 1900–2100 CE, geocentric
//! circumstances only (no per-observer local circumstances).
#![forbid(unsafe_code)]
```

- [ ] **Step 3: Register the crate in the workspace**

In root `Cargo.toml`, add `"crates/pleiades-eclipse",` to `members` (keep alphabetical: after `pleiades-data`/before `pleiades-elp`). Under `[workspace.dependencies]` add (keep alphabetical near the other `pleiades-*` lines):
```toml
pleiades-eclipse = { path = "crates/pleiades-eclipse", version = "0.2.0" }
```

- [ ] **Step 4: Write the smoke test**

`crates/pleiades-eclipse/tests/smoke.rs`:
```rust
#[test]
fn crate_builds() {
    // Compilation of this test is the assertion.
    assert_eq!(2 + 2, 4);
}
```

- [ ] **Step 5: Run it**

Run: `cargo test -p pleiades-eclipse`
Expected: PASS (1 test).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-eclipse Cargo.toml
git commit -m "feat(eclipse): scaffold pleiades-eclipse crate"
```

---

## Task 2: Eclipse domain types

**Files:**
- Create: `crates/pleiades-eclipse/src/types.rs`
- Modify: `crates/pleiades-eclipse/src/lib.rs`

**Interfaces:**
- Produces:
  - `enum EclipseKind { Solar, Lunar }`
  - `enum SolarEclipseType { Total, Annular, Hybrid, Partial }`
  - `enum LunarEclipseType { Penumbral, Partial, Total }`
  - `enum EclipseType { Solar(SolarEclipseType), Lunar(LunarEclipseType) }` with `fn kind(&self) -> EclipseKind`
  - `enum EclipseFilter { All, SolarOnly, LunarOnly }` with `fn admits(&self, kind: EclipseKind) -> bool`
  - `enum Node { North, South }`
  - `struct GeoLocation { latitude_degrees: f64, longitude_degrees: f64 }`
  - `struct Eclipse { kind, eclipse_type, greatest_eclipse: Instant, magnitude: f64, gamma: f64, saros_series: u32, eclipsed_longitude: Longitude, near_node: Node, greatest_eclipse_location: Option<GeoLocation> }`

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-eclipse/src/types.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eclipse_type_reports_its_kind() {
        assert_eq!(EclipseType::Solar(SolarEclipseType::Total).kind(), EclipseKind::Solar);
        assert_eq!(EclipseType::Lunar(LunarEclipseType::Penumbral).kind(), EclipseKind::Lunar);
    }

    #[test]
    fn filter_admits_expected_kinds() {
        assert!(EclipseFilter::All.admits(EclipseKind::Solar));
        assert!(EclipseFilter::All.admits(EclipseKind::Lunar));
        assert!(EclipseFilter::SolarOnly.admits(EclipseKind::Solar));
        assert!(!EclipseFilter::SolarOnly.admits(EclipseKind::Lunar));
        assert!(EclipseFilter::LunarOnly.admits(EclipseKind::Lunar));
        assert!(!EclipseFilter::LunarOnly.admits(EclipseKind::Solar));
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p pleiades-eclipse types::tests`
Expected: FAIL (compile error — types undefined).

- [ ] **Step 3: Write the types**

Prepend to `crates/pleiades-eclipse/src/types.rs`:
```rust
//! Eclipse domain value types.

use pleiades_types::{Instant, Longitude};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EclipseKind {
    Solar,
    Lunar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolarEclipseType {
    Total,
    Annular,
    Hybrid,
    Partial,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LunarEclipseType {
    Penumbral,
    Partial,
    Total,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EclipseType {
    Solar(SolarEclipseType),
    Lunar(LunarEclipseType),
}

impl EclipseType {
    pub fn kind(&self) -> EclipseKind {
        match self {
            EclipseType::Solar(_) => EclipseKind::Solar,
            EclipseType::Lunar(_) => EclipseKind::Lunar,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EclipseFilter {
    All,
    SolarOnly,
    LunarOnly,
}

impl EclipseFilter {
    pub fn admits(&self, kind: EclipseKind) -> bool {
        match self {
            EclipseFilter::All => true,
            EclipseFilter::SolarOnly => kind == EclipseKind::Solar,
            EclipseFilter::LunarOnly => kind == EclipseKind::Lunar,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Node {
    North,
    South,
}

/// A geocentric sub-shadow point on Earth. Deliberately not `ObserverLocation`:
/// the greatest-eclipse point is a position, not an observing site.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeoLocation {
    pub latitude_degrees: f64,
    pub longitude_degrees: f64,
}

/// A single global/geocentric eclipse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Eclipse {
    pub kind: EclipseKind,
    pub eclipse_type: EclipseType,
    pub greatest_eclipse: Instant,
    pub magnitude: f64,
    pub gamma: f64,
    pub saros_series: u32,
    pub eclipsed_longitude: Longitude,
    pub near_node: Node,
    pub greatest_eclipse_location: Option<GeoLocation>,
}
```

- [ ] **Step 4: Wire the module and re-exports**

In `crates/pleiades-eclipse/src/lib.rs` add after the rustdoc/attribute lines:
```rust
mod types;

pub use types::{
    Eclipse, EclipseFilter, EclipseKind, EclipseType, GeoLocation, LunarEclipseType, Node,
    SolarEclipseType,
};
```

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p pleiades-eclipse`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-eclipse/src
git commit -m "feat(eclipse): add eclipse domain types"
```

---

## Task 3: Eclipse error type

**Files:**
- Create: `crates/pleiades-eclipse/src/error.rs`
- Modify: `crates/pleiades-eclipse/src/lib.rs`

**Interfaces:**
- Produces: `enum EclipseError { OutOfWindow { julian_day: f64 }, Backend(String), MissingCoordinates { body_label: &'static str, julian_day: f64 } }` implementing `Display` + `std::error::Error`, plus `pub const WINDOW_START_JD: f64 = 2_415_020.5;` and `pub const WINDOW_END_JD: f64 = 2_488_069.5;`.

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-eclipse/src/error.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_window_message_names_the_julian_day() {
        let err = EclipseError::OutOfWindow { julian_day: 2_400_000.5 };
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

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p pleiades-eclipse error::tests`
Expected: FAIL (compile error).

- [ ] **Step 3: Write the error type**

Prepend to `crates/pleiades-eclipse/src/error.rs`:
```rust
//! Structured, fail-closed eclipse errors.

use core::fmt;

/// First instant of the supported window (1900-01-01 TT), Julian Day.
pub const WINDOW_START_JD: f64 = 2_415_020.5;
/// Last instant of the supported window (2100-01-01 TT), Julian Day.
pub const WINDOW_END_JD: f64 = 2_488_069.5;

#[derive(Clone, Debug, PartialEq)]
pub enum EclipseError {
    /// A requested instant falls outside the 1900–2100 CE window.
    OutOfWindow { julian_day: f64 },
    /// The backend returned a structured error.
    Backend(String),
    /// The backend produced no ecliptic coordinates for a required body.
    MissingCoordinates { body_label: &'static str, julian_day: f64 },
}

impl fmt::Display for EclipseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EclipseError::OutOfWindow { julian_day } => write!(
                f,
                "instant JD {julian_day} is outside the supported 1900–2100 CE window \
                 (JD {WINDOW_START_JD}..={WINDOW_END_JD})"
            ),
            EclipseError::Backend(message) => write!(f, "backend error: {message}"),
            EclipseError::MissingCoordinates { body_label, julian_day } => write!(
                f,
                "backend returned no ecliptic coordinates for {body_label} at JD {julian_day}"
            ),
        }
    }
}

impl std::error::Error for EclipseError {}
```

- [ ] **Step 4: Wire the module**

In `crates/pleiades-eclipse/src/lib.rs`:
```rust
mod error;

pub use error::{EclipseError, WINDOW_END_JD, WINDOW_START_JD};
```

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p pleiades-eclipse`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-eclipse/src
git commit -m "feat(eclipse): add fail-closed eclipse error type"
```

---

## Task 4: Sun/Moon ephemeris reader

**Files:**
- Create: `crates/pleiades-eclipse/src/ephemeris.rs`
- Modify: `crates/pleiades-eclipse/src/lib.rs`

**Interfaces:**
- Consumes: `EphemerisBackend::position`, `EphemerisRequest`, `EclipseError`.
- Produces:
  - `struct SunMoonSample { sun_longitude_deg: f64, sun_latitude_deg: f64, sun_distance_au: f64, moon_longitude_deg: f64, moon_latitude_deg: f64, moon_distance_au: f64 }`
  - `fn sample_sun_moon<B: EphemerisBackend>(backend: &B, julian_day: f64) -> Result<SunMoonSample, EclipseError>`
  - `fn elongation_deg(sample: &SunMoonSample) -> f64` — signed Moon−Sun longitude wrapped to `(-180, 180]`.

This isolates all backend-request construction (apparent, tropical-of-date, geocentric ecliptic) in one place.

- [ ] **Step 1: Write the failing test (with an analytic mock backend)**

Append to `crates/pleiades-eclipse/src/ephemeris.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon; // defined in Step 3

    #[test]
    fn elongation_is_zero_at_new_moon_epoch() {
        // LinearSunMoon places Sun and Moon at equal longitude at jd0.
        let backend = LinearSunMoon::new_moon_at(2_451_550.0);
        let sample = sample_sun_moon(&backend, 2_451_550.0).unwrap();
        assert!(elongation_deg(&sample).abs() < 1e-6);
    }

    #[test]
    fn missing_coordinates_fail_closed() {
        let backend = LinearSunMoon::empty();
        let err = sample_sun_moon(&backend, 2_451_550.0).unwrap_err();
        assert!(matches!(err, EclipseError::MissingCoordinates { .. }));
    }
}
```

- [ ] **Step 2: Write the reusable analytic mock backend in `pleiades-backend`**

The mock is shared by several tasks, so it lives in `pleiades-backend` behind `#[cfg(any(test, feature = "test-backend"))]`. Add to `crates/pleiades-backend/Cargo.toml` under `[features]`:
```toml
test-backend = []
```
Create `crates/pleiades-backend/src/test_backend.rs`:
```rust
//! Deterministic analytic Sun/Moon backend for downstream tests.
//! NOT for production: circular-orbit longitudes, fixed distances.

use crate::errors::EphemerisError;
use crate::metadata::BackendMetadata;
use crate::request::EphemerisRequest;
use crate::result::EphemerisResult;
use crate::traits::EphemerisBackend;
use pleiades_types::{CelestialBody, EclipticCoordinates, Latitude, Longitude};

/// Sun and Moon move at constant ecliptic rates; both at `ref_longitude` at `jd0`.
#[derive(Clone, Copy, Debug)]
pub struct LinearSunMoon {
    jd0: f64,
    ref_longitude_deg: f64,
    sun_rate_deg_per_day: f64,
    moon_rate_deg_per_day: f64,
    moon_latitude_deg: f64,
    produce_coordinates: bool,
}

impl LinearSunMoon {
    /// New moon (Sun==Moon longitude, Moon on the ecliptic) at `jd0`.
    pub fn new_moon_at(jd0: f64) -> Self {
        Self {
            jd0,
            ref_longitude_deg: 100.0,
            sun_rate_deg_per_day: 0.985_647,
            moon_rate_deg_per_day: 13.176_396,
            moon_latitude_deg: 0.0,
            produce_coordinates: true,
        }
    }

    /// Backend that returns no coordinates (drives the fail-closed path).
    pub fn empty() -> Self {
        let mut s = Self::new_moon_at(2_451_550.0);
        s.produce_coordinates = false;
        s
    }

    /// Set the Moon's (constant) ecliptic latitude in degrees.
    pub fn with_moon_latitude(mut self, degrees: f64) -> Self {
        self.moon_latitude_deg = degrees;
        self
    }
}

impl EphemerisBackend for LinearSunMoon {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata::test_placeholder("linear-sun-moon")
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        matches!(body, CelestialBody::Sun | CelestialBody::Moon)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if !self.produce_coordinates {
            return Ok(EphemerisResult::without_coordinates(req));
        }
        let dt = req.instant.julian_day().days() - self.jd0;
        let (rate, latitude, distance_au) = match req.body {
            CelestialBody::Sun => (self.sun_rate_deg_per_day, 0.0, 1.000_0),
            CelestialBody::Moon => (self.moon_rate_deg_per_day, self.moon_latitude_deg, 0.002_57),
            _ => return Err(EphemerisError::unsupported_body(req.body.clone())),
        };
        let lon = Longitude::from_degrees(self.ref_longitude_deg + rate * dt).normalized_0_360();
        let coords =
            EclipticCoordinates::new(lon, Latitude::from_degrees(latitude), Some(distance_au));
        Ok(EphemerisResult::with_ecliptic(req, coords))
    }
}
```
Wire it in `crates/pleiades-backend/src/lib.rs`:
```rust
#[cfg(any(test, feature = "test-backend"))]
pub mod test_backend;
```
> Implementer note: `BackendMetadata::test_placeholder`, `EphemerisResult::without_coordinates`, `EphemerisResult::with_ecliptic`, `EphemerisError::unsupported_body`, and `Instant::julian_day()` are thin constructors. If any does not already exist with that exact name, add a minimal one in the same commit (check `result.rs`, `metadata.rs`, `errors.rs`, `time.rs`) — match the existing constructor style and keep them `pub`. In `pleiades-eclipse/Cargo.toml` add `pleiades-backend = { workspace = true, features = ["test-backend"] }` under `[dev-dependencies]` (it stays default-off for the library build).

- [ ] **Step 3: Write the reader**

Prepend to `crates/pleiades-eclipse/src/ephemeris.rs`:
```rust
//! Reads geocentric apparent Sun and Moon ecliptic positions from a backend.

use crate::error::EclipseError;
use pleiades_backend::{EphemerisBackend, EphemerisRequest};
use pleiades_types::{
    Apparentness, CelestialBody, CoordinateFrame, Instant, JulianDay, TimeScale, ZodiacMode,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct SunMoonSample {
    pub sun_longitude_deg: f64,
    pub sun_latitude_deg: f64,
    pub sun_distance_au: f64,
    pub moon_longitude_deg: f64,
    pub moon_latitude_deg: f64,
    pub moon_distance_au: f64,
}

fn request(body: CelestialBody, julian_day: f64) -> EphemerisRequest {
    EphemerisRequest {
        body,
        instant: Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Apparent,
    }
}

fn read<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<(f64, f64, f64), EclipseError> {
    let result = backend
        .position(&request(body, julian_day))
        .map_err(|e| EclipseError::Backend(e.to_string()))?;
    let ecliptic = result
        .ecliptic
        .ok_or(EclipseError::MissingCoordinates { body_label, julian_day })?;
    let distance = ecliptic
        .distance_au
        .ok_or(EclipseError::MissingCoordinates { body_label, julian_day })?;
    Ok((
        ecliptic.longitude.degrees(),
        ecliptic.latitude.degrees(),
        distance,
    ))
}

pub(crate) fn sample_sun_moon<B: EphemerisBackend>(
    backend: &B,
    julian_day: f64,
) -> Result<SunMoonSample, EclipseError> {
    let (sun_longitude_deg, sun_latitude_deg, sun_distance_au) =
        read(backend, CelestialBody::Sun, "Sun", julian_day)?;
    let (moon_longitude_deg, moon_latitude_deg, moon_distance_au) =
        read(backend, CelestialBody::Moon, "Moon", julian_day)?;
    Ok(SunMoonSample {
        sun_longitude_deg,
        sun_latitude_deg,
        sun_distance_au,
        moon_longitude_deg,
        moon_latitude_deg,
        moon_distance_au,
    })
}

/// Signed Moon−Sun ecliptic longitude, wrapped into `(-180, 180]` degrees.
pub(crate) fn elongation_deg(sample: &SunMoonSample) -> f64 {
    let mut d = sample.moon_longitude_deg - sample.sun_longitude_deg;
    d = ((d + 180.0).rem_euclid(360.0)) - 180.0;
    d
}
```

Wire in `lib.rs`: `mod ephemeris;`

- [ ] **Step 4: Run the tests**

Run: `cargo test -p pleiades-eclipse ephemeris && cargo test -p pleiades-backend test_backend`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-eclipse crates/pleiades-backend Cargo.toml
git commit -m "feat(eclipse): add Sun/Moon reader and analytic test backend"
```

---

## Task 5: Syzygy search

**Files:**
- Create: `crates/pleiades-eclipse/src/syzygy.rs`
- Modify: `crates/pleiades-eclipse/src/lib.rs`

**Interfaces:**
- Consumes: `sample_sun_moon`, `elongation_deg`, `EclipseError`.
- Produces:
  - `enum Syzygy { NewMoon, FullMoon }`
  - `struct SyzygyEvent { syzygy: Syzygy, julian_day: f64 }`
  - `fn find_syzygies<B: EphemerisBackend>(backend: &B, start_jd: f64, end_jd: f64) -> Result<Vec<SyzygyEvent>, EclipseError>` — every new and full moon with `start_jd <= jd <= end_jd`, time-ordered, each refined to ≤ 1 s.

Algorithm: a new moon is a zero of `elongation_deg`; a full moon is a zero of `elongation_deg ∓ 180` (the wrapped distance from 180). Step in coarse increments of `0.5` day, detect sign changes of each target function, bisect to refine.

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-eclipse/src/syzygy.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn finds_the_new_moon_at_the_reference_epoch() {
        let backend = LinearSunMoon::new_moon_at(2_451_550.0);
        let events = find_syzygies(&backend, 2_451_549.0, 2_451_551.0).unwrap();
        let new_moon = events.iter().find(|e| e.syzygy == Syzygy::NewMoon).unwrap();
        assert!((new_moon.julian_day - 2_451_550.0).abs() < 1.0 / 86_400.0);
    }

    #[test]
    fn events_are_time_ordered() {
        let backend = LinearSunMoon::new_moon_at(2_451_550.0);
        let events = find_syzygies(&backend, 2_451_540.0, 2_451_600.0).unwrap();
        assert!(events.windows(2).all(|w| w[0].julian_day <= w[1].julian_day));
        assert!(events.len() >= 4); // ~2 lunations → ≥2 new + ≥2 full
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p pleiades-eclipse syzygy`
Expected: FAIL (compile error).

- [ ] **Step 3: Write the search**

Prepend to `crates/pleiades-eclipse/src/syzygy.rs`:
```rust
//! Locates new and full moons (eclipse candidates) by root-finding on the
//! Sun−Moon elongation.

use crate::ephemeris::{elongation_deg, sample_sun_moon};
use crate::error::EclipseError;
use pleiades_backend::EphemerisBackend;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Syzygy {
    NewMoon,
    FullMoon,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SyzygyEvent {
    pub syzygy: Syzygy,
    pub julian_day: f64,
}

const STEP_DAYS: f64 = 0.5;
const REFINE_TOLERANCE_DAYS: f64 = 0.5 / 86_400.0; // 0.5 s

/// Target function whose zero marks the syzygy: elongation for a new moon, and
/// the signed distance from 180° for a full moon. Both wrap into (-180, 180].
fn target<B: EphemerisBackend>(
    backend: &B,
    julian_day: f64,
    syzygy: Syzygy,
) -> Result<f64, EclipseError> {
    let sample = sample_sun_moon(backend, julian_day)?;
    let elong = elongation_deg(&sample);
    Ok(match syzygy {
        Syzygy::NewMoon => elong,
        Syzygy::FullMoon => {
            let mut d = elong - 180.0;
            d = ((d + 180.0).rem_euclid(360.0)) - 180.0;
            d
        }
    })
}

fn bisect<B: EphemerisBackend>(
    backend: &B,
    syzygy: Syzygy,
    mut lo: f64,
    mut f_lo: f64,
    mut hi: f64,
) -> Result<f64, EclipseError> {
    while (hi - lo) > REFINE_TOLERANCE_DAYS {
        let mid = 0.5 * (lo + hi);
        let f_mid = target(backend, mid, syzygy)?;
        if (f_lo <= 0.0) == (f_mid <= 0.0) {
            lo = mid;
            f_lo = f_mid;
        } else {
            hi = mid;
        }
    }
    Ok(0.5 * (lo + hi))
}

fn find_one<B: EphemerisBackend>(
    backend: &B,
    syzygy: Syzygy,
    start_jd: f64,
    end_jd: f64,
    out: &mut Vec<SyzygyEvent>,
) -> Result<(), EclipseError> {
    let mut prev_jd = start_jd;
    let mut prev_f = target(backend, prev_jd, syzygy)?;
    let mut jd = start_jd + STEP_DAYS;
    while jd <= end_jd + STEP_DAYS {
        let f = target(backend, jd, syzygy)?;
        // A real syzygy crossing: sign change with the gap small enough that it
        // is the elongation passing through zero (not the ±180 wrap seam).
        if (prev_f <= 0.0) != (f <= 0.0) && (prev_f - f).abs() < 180.0 {
            let root = bisect(backend, syzygy, prev_jd, prev_f, jd)?;
            if root >= start_jd && root <= end_jd {
                out.push(SyzygyEvent { syzygy, julian_day: root });
            }
        }
        prev_jd = jd;
        prev_f = f;
        jd += STEP_DAYS;
    }
    Ok(())
}

pub(crate) fn find_syzygies<B: EphemerisBackend>(
    backend: &B,
    start_jd: f64,
    end_jd: f64,
) -> Result<Vec<SyzygyEvent>, EclipseError> {
    let mut out = Vec::new();
    find_one(backend, Syzygy::NewMoon, start_jd, end_jd, &mut out)?;
    find_one(backend, Syzygy::FullMoon, start_jd, end_jd, &mut out)?;
    out.sort_by(|a, b| a.julian_day.partial_cmp(&b.julian_day).unwrap());
    Ok(out)
}
```

Wire in `lib.rs`: `mod syzygy;`

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p pleiades-eclipse syzygy`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-eclipse/src
git commit -m "feat(eclipse): add syzygy (new/full moon) search"
```

---

## Task 6: Solar eclipse geometry & classification

**Files:**
- Create: `crates/pleiades-eclipse/src/geometry.rs`
- Modify: `crates/pleiades-eclipse/src/lib.rs`

**Interfaces:**
- Consumes: `SunMoonSample`.
- Produces:
  - `pub(crate) mod constants` with `R_SUN_KM`, `R_MOON_KM`, `R_EARTH_KM`, `AU_KM`, `SHADOW_INFLATION`.
  - `struct SolarCircumstances { eclipse_type: SolarEclipseType, magnitude: f64, gamma: f64 }`
  - `fn classify_solar(sample: &SunMoonSample) -> Option<SolarCircumstances>` — `None` when no eclipse occurs at this new moon.

Geometry (geocentric, angular, small-angle in radians):
- Sun angular radius `s = asin(R_SUN_KM / (sun_distance_au * AU_KM))`.
- Moon angular radius `m = asin(R_MOON_KM / (moon_distance_au * AU_KM))`.
- Moon horizontal parallax `π = asin(R_EARTH_KM / (moon_distance_au * AU_KM))`.
- Geocentric Sun–Moon separation `σ`: with the Moon's ecliptic latitude `β` (the Sun's latitude ≈ 0), `σ ≈ |β|` at conjunction (longitudes equal at the syzygy). Use the full great-circle separation from both longitudes/latitudes for robustness.
- `gamma = σ / π` (least shadow-axis distance from Earth center, in Earth radii; signed by the sign of `β`).
- An eclipse occurs when `σ < π + s + m` (penumbral cone touches Earth). Below that:
  - `magnitude = (s + m - σ) / (2 * s)` clamped to `[0, ∞)`; partial if `magnitude < 1`.
  - Central (`σ ≤ π`, axis hits Earth): **Total** if `m >= s` (Moon's disk covers the Sun), **Annular** if `m < s`. **Hybrid** when the umbra/antumbra transition straddles Earth's surface — approximate as `|m - s| < HYBRID_BAND` with `HYBRID_BAND = 0.000_03` rad and `σ ≤ π`.

- [ ] **Step 1: Write the failing tests**

Append to `crates/pleiades-eclipse/src/geometry.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ephemeris::SunMoonSample;

    fn sample(moon_lat_deg: f64, moon_dist_au: f64, sun_dist_au: f64) -> SunMoonSample {
        SunMoonSample {
            sun_longitude_deg: 100.0,
            sun_latitude_deg: 0.0,
            sun_distance_au: sun_dist_au,
            moon_longitude_deg: 100.0,
            moon_latitude_deg: moon_lat_deg,
            moon_distance_au: moon_dist_au,
        }
    }

    #[test]
    fn central_close_moon_is_total() {
        // Moon near perigee (close → large disk), on the node → total.
        let c = classify_solar(&sample(0.0, 0.00238, 1.000)).unwrap();
        assert_eq!(c.eclipse_type, SolarEclipseType::Total);
        assert!(c.magnitude >= 1.0);
        assert!(c.gamma.abs() < 0.1);
    }

    #[test]
    fn central_far_moon_is_annular() {
        // Moon near apogee (far → small disk), on the node → annular.
        let c = classify_solar(&sample(0.0, 0.00271, 1.000)).unwrap();
        assert_eq!(c.eclipse_type, SolarEclipseType::Annular);
        assert!(c.magnitude < 1.0);
    }

    #[test]
    fn far_from_node_is_no_eclipse() {
        assert!(classify_solar(&sample(1.5, 0.00257, 1.000)).is_none());
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p pleiades-eclipse geometry::tests::central`
Expected: FAIL (compile error).

- [ ] **Step 3: Write the geometry**

Prepend to `crates/pleiades-eclipse/src/geometry.rs`:
```rust
//! Geocentric eclipse shadow-cone geometry and type classification.

use crate::ephemeris::SunMoonSample;
use crate::types::SolarEclipseType;

pub(crate) mod constants {
    pub const R_SUN_KM: f64 = 696_000.0;
    pub const R_MOON_KM: f64 = 1_737.4;
    pub const R_EARTH_KM: f64 = 6_378.137;
    pub const AU_KM: f64 = 149_597_870.7;
    /// Geometric enlargement of Earth's shadow used by the NASA canon.
    pub const SHADOW_INFLATION: f64 = 1.02;
}

use constants::*;

const HYBRID_BAND_RAD: f64 = 0.000_03;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SolarCircumstances {
    pub eclipse_type: SolarEclipseType,
    pub magnitude: f64,
    pub gamma: f64,
}

/// Great-circle separation (radians) between Sun and Moon centers.
fn separation_rad(sample: &SunMoonSample) -> f64 {
    let (l1, b1) = (
        sample.sun_longitude_deg.to_radians(),
        sample.sun_latitude_deg.to_radians(),
    );
    let (l2, b2) = (
        sample.moon_longitude_deg.to_radians(),
        sample.moon_latitude_deg.to_radians(),
    );
    let cos_sep =
        (b1.sin() * b2.sin() + b1.cos() * b2.cos() * (l1 - l2).cos()).clamp(-1.0, 1.0);
    cos_sep.acos()
}

pub(crate) fn classify_solar(sample: &SunMoonSample) -> Option<SolarCircumstances> {
    let s = (R_SUN_KM / (sample.sun_distance_au * AU_KM)).asin();
    let m = (R_MOON_KM / (sample.moon_distance_au * AU_KM)).asin();
    let parallax = (R_EARTH_KM / (sample.moon_distance_au * AU_KM)).asin();
    let sigma = separation_rad(sample);

    if sigma >= parallax + s + m {
        return None; // penumbra misses Earth — no eclipse
    }

    let gamma = (sigma / parallax) * sample.moon_latitude_deg.signum();
    let magnitude = ((s + m - sigma) / (2.0 * s)).max(0.0);

    let central = sigma <= parallax;
    let eclipse_type = if !central {
        SolarEclipseType::Partial
    } else if (m - s).abs() < HYBRID_BAND_RAD {
        SolarEclipseType::Hybrid
    } else if m >= s {
        SolarEclipseType::Total
    } else {
        SolarEclipseType::Annular
    };

    Some(SolarCircumstances { eclipse_type, magnitude, gamma })
}
```

Wire in `lib.rs`: `mod geometry;`

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p pleiades-eclipse geometry`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-eclipse/src
git commit -m "feat(eclipse): add solar eclipse geometry and classification"
```

---

## Task 7: Lunar eclipse geometry & classification

**Files:**
- Modify: `crates/pleiades-eclipse/src/geometry.rs`

**Interfaces:**
- Produces:
  - `struct LunarCircumstances { eclipse_type: LunarEclipseType, magnitude: f64, gamma: f64 }`
  - `fn classify_lunar(sample: &SunMoonSample) -> Option<LunarCircumstances>` — `None` when the Moon misses the penumbra at this full moon.

Geometry: at a full moon the Moon is near the antisolar point; its distance from the shadow axis is the great-circle separation between the Moon and the antisolar point (Sun longitude + 180°, latitude negated), call it `σ`. With `π_moon`, `π_sun = asin(R_EARTH_KM/(sun_distance_au*AU_KM))`, `s` (Sun ang. radius):
- Umbral radius `u = SHADOW_INFLATION * (π_moon + π_sun - s)`.
- Penumbral radius `p = SHADOW_INFLATION * (π_moon + π_sun + s)`.
- `m_moon` = Moon angular radius.
- No eclipse if `σ >= p + m_moon`. Else: **Total** if `σ + m_moon <= u`; **Partial** if `σ - m_moon < u` (umbra touched but not fully immersed); otherwise **Penumbral**.
- Umbral magnitude `= (u + m_moon - σ) / (2 * m_moon)`; for penumbral-only eclipses report the penumbral magnitude `(p + m_moon - σ)/(2*m_moon)`.
- `gamma = σ / π_moon` signed by the Moon's latitude sign.

- [ ] **Step 1: Write the failing tests**

Append to the `tests` module in `crates/pleiades-eclipse/src/geometry.rs`:
```rust
    fn full_moon_sample(moon_lat_deg: f64, moon_dist_au: f64) -> SunMoonSample {
        // Full moon: Moon opposite the Sun in longitude.
        SunMoonSample {
            sun_longitude_deg: 100.0,
            sun_latitude_deg: 0.0,
            sun_distance_au: 1.000,
            moon_longitude_deg: 280.0,
            moon_latitude_deg: moon_lat_deg,
            moon_distance_au: moon_dist_au,
        }
    }

    #[test]
    fn central_full_moon_is_total_lunar() {
        let c = classify_lunar(&full_moon_sample(0.0, 0.00257)).unwrap();
        assert_eq!(c.eclipse_type, LunarEclipseType::Total);
        assert!(c.magnitude >= 1.0);
    }

    #[test]
    fn distant_latitude_full_moon_is_no_eclipse() {
        assert!(classify_lunar(&full_moon_sample(1.6, 0.00257)).is_none());
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p pleiades-eclipse geometry::tests::central_full_moon`
Expected: FAIL (compile error — `classify_lunar` undefined).

- [ ] **Step 3: Write the lunar geometry**

Add to `crates/pleiades-eclipse/src/geometry.rs` (after `classify_solar`), and add `use crate::types::LunarEclipseType;` to the imports:
```rust
#[derive(Clone, Copy, Debug)]
pub(crate) struct LunarCircumstances {
    pub eclipse_type: LunarEclipseType,
    pub magnitude: f64,
    pub gamma: f64,
}

/// Separation (radians) of the Moon from the antisolar (shadow-axis) point.
fn shadow_axis_separation_rad(sample: &SunMoonSample) -> f64 {
    let anti_lon = (sample.sun_longitude_deg + 180.0).rem_euclid(360.0);
    let anti = SunMoonSample {
        sun_longitude_deg: anti_lon,
        sun_latitude_deg: -sample.sun_latitude_deg,
        ..*sample
    };
    // Reuse the Sun↔Moon separation routine against the antisolar point.
    separation_rad(&anti)
}

pub(crate) fn classify_lunar(sample: &SunMoonSample) -> Option<LunarCircumstances> {
    let s = (R_SUN_KM / (sample.sun_distance_au * AU_KM)).asin();
    let m_moon = (R_MOON_KM / (sample.moon_distance_au * AU_KM)).asin();
    let pi_moon = (R_EARTH_KM / (sample.moon_distance_au * AU_KM)).asin();
    let pi_sun = (R_EARTH_KM / (sample.sun_distance_au * AU_KM)).asin();

    let u = SHADOW_INFLATION * (pi_moon + pi_sun - s);
    let p = SHADOW_INFLATION * (pi_moon + pi_sun + s);
    let sigma = shadow_axis_separation_rad(sample);

    if sigma >= p + m_moon {
        return None;
    }

    let gamma = (sigma / pi_moon) * sample.moon_latitude_deg.signum();
    let (eclipse_type, magnitude) = if sigma + m_moon <= u {
        (LunarEclipseType::Total, (u + m_moon - sigma) / (2.0 * m_moon))
    } else if sigma - m_moon < u {
        (LunarEclipseType::Partial, (u + m_moon - sigma) / (2.0 * m_moon))
    } else {
        (LunarEclipseType::Penumbral, (p + m_moon - sigma) / (2.0 * m_moon))
    };

    Some(LunarCircumstances { eclipse_type, magnitude, gamma })
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p pleiades-eclipse geometry`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-eclipse/src
git commit -m "feat(eclipse): add lunar eclipse geometry and classification"
```

---

## Task 8: Saros series numbering

**Files:**
- Create: `crates/pleiades-eclipse/src/saros.rs`
- Modify: `crates/pleiades-eclipse/src/lib.rs`

**Interfaces:**
- Produces: `fn saros_series(kind: EclipseKind, greatest_eclipse_jd: f64) -> u32`.

Method: the Saros series number is fixed by the eclipse's lunation number `n` (integer count of synodic months from a reference new moon) via the standard congruence. Use the well-established relation anchored on Saros: with `n` the synodic-month index from the 2000-01-06 new moon (`SYNODIC_REF_JD = 2_451_550.1`, `SYNODIC_MONTH = 29.530_588_861`), the solar Saros series is `s = ((n * 38 + 153) mod 223)` mapped onto the active series window for the era, and the lunar series uses the half-saros-shifted anchor. Because exact series numbering is fiddly, implement it as a **table-anchored** lookup: ship a small const table of `(jd_near, series)` anchors (one per active series in 1900–2100, sourced from the NASA canon) and select the series whose 18.03-year cadence lands nearest `greatest_eclipse_jd`.

> Implementer note: the const anchor table is populated in Task 10 from the same NASA canon used for the fixture (extract `series → one member eclipse JD` while parsing). Here, implement the selection logic and unit-test it against three hand-entered anchors. Keep `SAROS_ANCHORS: &[(EclipseKind, f64, u32)]` `pub(crate)` so Task 10 can extend it.

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-eclipse/src/saros.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EclipseKind;

    #[test]
    fn picks_the_series_one_saros_period_away() {
        // Anchor: solar series 145 has a member at JD 2_451_044.5 (1999-08-11).
        // One saros (~6585.32 d) later is still series 145.
        let jd = 2_451_044.5 + SAROS_PERIOD_DAYS;
        assert_eq!(saros_series(EclipseKind::Solar, jd), 145);
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p pleiades-eclipse saros`
Expected: FAIL (compile error).

- [ ] **Step 3: Write the numbering**

Prepend to `crates/pleiades-eclipse/src/saros.rs`:
```rust
//! Saros series assignment by nearest-cadence match against canon anchors.

use crate::types::EclipseKind;

pub(crate) const SAROS_PERIOD_DAYS: f64 = 6_585.321_3;

/// `(kind, member_jd, series)` anchors taken from the NASA canon. Extended in
/// the validation-fixture build (Task 10) to cover every active 1900–2100 series.
pub(crate) const SAROS_ANCHORS: &[(EclipseKind, f64, u32)] = &[
    (EclipseKind::Solar, 2_451_044.5, 145), // 1999-08-11 total solar
    (EclipseKind::Solar, 2_457_987.27, 145), // 2017-08-21 total solar (same series)
    (EclipseKind::Lunar, 2_458_504.9, 134), // 2019-01-21 total lunar
];

pub(crate) fn saros_series(kind: EclipseKind, greatest_eclipse_jd: f64) -> u32 {
    let mut best: Option<(u32, f64)> = None;
    for &(anchor_kind, anchor_jd, series) in SAROS_ANCHORS {
        if anchor_kind != kind {
            continue;
        }
        // How far is `greatest_eclipse_jd` from an integer number of saros
        // periods away from this anchor?
        let periods = ((greatest_eclipse_jd - anchor_jd) / SAROS_PERIOD_DAYS).round();
        let residual = (greatest_eclipse_jd - (anchor_jd + periods * SAROS_PERIOD_DAYS)).abs();
        if best.map_or(true, |(_, r)| residual < r) {
            best = Some((series, residual));
        }
    }
    best.map(|(series, _)| series).unwrap_or(0)
}
```

Wire in `lib.rs`: `mod saros;`

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p pleiades-eclipse saros`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-eclipse/src
git commit -m "feat(eclipse): add Saros series numbering"
```

---

## Task 9: EclipseEngine assembly

**Files:**
- Create: `crates/pleiades-eclipse/src/engine.rs`
- Modify: `crates/pleiades-eclipse/src/lib.rs`
- Test: `crates/pleiades-eclipse/tests/known_eclipses.rs`

**Interfaces:**
- Consumes: `find_syzygies`, `classify_solar`, `classify_lunar`, `saros_series`, `sample_sun_moon`, all domain types.
- Produces:
  - `struct EclipseEngine<B> { backend: B }` with `pub fn new(backend: B) -> Self`.
  - `pub fn eclipses_in_range(&self, start: Instant, end: Instant, filter: EclipseFilter) -> Result<Vec<Eclipse>, EclipseError>`
  - `pub fn next_eclipse(&self, after: Instant, filter: EclipseFilter) -> Result<Option<Eclipse>, EclipseError>`
  - `pub fn previous_eclipse(&self, before: Instant, filter: EclipseFilter) -> Result<Option<Eclipse>, EclipseError>`

Engine flow per syzygy: refine the greatest-eclipse instant (golden-section minimize separation around the syzygy JD over ±0.25 day), re-sample, classify (solar for new moon, lunar for full moon), and if an eclipse exists build the `Eclipse` with `eclipsed_longitude` = apparent Sun longitude at greatest eclipse, `near_node` from the sign of the Moon's latitude rate, `saros_series`, and (solar only) `greatest_eclipse_location` from the sub-shadow geometry + GMST.

- [ ] **Step 1: Write the failing window-bound test**

`crates/pleiades-eclipse/src/engine.rs` (tests use the analytic backend; the real-ephemeris assertions live in the integration test in Step 5):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{Instant, JulianDay, TimeScale};

    fn at(jd: f64) -> Instant {
        Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
    }

    #[test]
    fn out_of_window_start_fails_closed() {
        let engine = EclipseEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let err = engine
            .eclipses_in_range(at(2_400_000.0), at(2_451_551.0), EclipseFilter::All)
            .unwrap_err();
        assert!(matches!(err, EclipseError::OutOfWindow { .. }));
    }

    #[test]
    fn filter_excludes_lunar() {
        // The on-node analytic backend yields a solar eclipse at every new moon
        // and a lunar one at every full moon; SolarOnly must drop the lunar ones.
        let engine = EclipseEngine::new(
            LinearSunMoon::new_moon_at(2_451_550.0).with_moon_latitude(0.0),
        );
        let solar = engine
            .eclipses_in_range(at(2_451_549.0), at(2_451_551.0), EclipseFilter::SolarOnly)
            .unwrap();
        assert!(solar.iter().all(|e| e.kind == EclipseKind::Solar));
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p pleiades-eclipse engine`
Expected: FAIL (compile error).

- [ ] **Step 3: Write the engine**

Prepend to `crates/pleiades-eclipse/src/engine.rs`:
```rust
//! The public eclipse search engine.

use crate::ephemeris::sample_sun_moon;
use crate::error::{EclipseError, WINDOW_END_JD, WINDOW_START_JD};
use crate::geometry::{classify_lunar, classify_solar};
use crate::saros::saros_series;
use crate::syzygy::{find_syzygies, Syzygy};
use crate::types::{
    Eclipse, EclipseFilter, EclipseKind, EclipseType, GeoLocation, Node,
};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{Instant, JulianDay, Longitude, TimeScale};

pub struct EclipseEngine<B> {
    backend: B,
}

impl<B: EphemerisBackend> EclipseEngine<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn eclipses_in_range(
        &self,
        start: Instant,
        end: Instant,
        filter: EclipseFilter,
    ) -> Result<Vec<Eclipse>, EclipseError> {
        let start_jd = start.julian_day().days();
        let end_jd = end.julian_day().days();
        self.check_window(start_jd)?;
        self.check_window(end_jd)?;

        let mut out = Vec::new();
        for event in find_syzygies(&self.backend, start_jd, end_jd)? {
            if let Some(eclipse) = self.build(event.syzygy, event.julian_day)? {
                if filter.admits(eclipse.kind) {
                    out.push(eclipse);
                }
            }
        }
        Ok(out)
    }

    pub fn next_eclipse(
        &self,
        after: Instant,
        filter: EclipseFilter,
    ) -> Result<Option<Eclipse>, EclipseError> {
        let after_jd = after.julian_day().days();
        let end = Instant::new(JulianDay::from_days(WINDOW_END_JD), TimeScale::Tdb);
        Ok(self
            .eclipses_in_range(after, end, filter)?
            .into_iter()
            .find(|e| e.greatest_eclipse.julian_day().days() > after_jd))
    }

    pub fn previous_eclipse(
        &self,
        before: Instant,
        filter: EclipseFilter,
    ) -> Result<Option<Eclipse>, EclipseError> {
        let before_jd = before.julian_day().days();
        let start = Instant::new(JulianDay::from_days(WINDOW_START_JD), TimeScale::Tdb);
        Ok(self
            .eclipses_in_range(start, before, filter)?
            .into_iter()
            .rev()
            .find(|e| e.greatest_eclipse.julian_day().days() < before_jd))
    }

    fn check_window(&self, jd: f64) -> Result<(), EclipseError> {
        if jd < WINDOW_START_JD || jd > WINDOW_END_JD {
            Err(EclipseError::OutOfWindow { julian_day: jd })
        } else {
            Ok(())
        }
    }

    fn build(&self, syzygy: Syzygy, syzygy_jd: f64) -> Result<Option<Eclipse>, EclipseError> {
        let greatest_jd = self.refine_greatest(syzygy, syzygy_jd)?;
        let sample = sample_sun_moon(&self.backend, greatest_jd)?;
        let greatest_eclipse = Instant::new(JulianDay::from_days(greatest_jd), TimeScale::Tdb);
        let eclipsed_longitude = match syzygy {
            Syzygy::NewMoon => Longitude::from_degrees(sample.sun_longitude_deg),
            Syzygy::FullMoon => {
                Longitude::from_degrees(sample.sun_longitude_deg + 180.0).normalized_0_360()
            }
        };
        // Node: ascending (North) if the Moon's latitude is increasing through 0.
        let later = sample_sun_moon(&self.backend, greatest_jd + 0.01)?;
        let near_node = if later.moon_latitude_deg >= sample.moon_latitude_deg {
            Node::North
        } else {
            Node::South
        };

        let eclipse = match syzygy {
            Syzygy::NewMoon => {
                let Some(c) = classify_solar(&sample) else { return Ok(None) };
                Eclipse {
                    kind: EclipseKind::Solar,
                    eclipse_type: EclipseType::Solar(c.eclipse_type),
                    greatest_eclipse,
                    magnitude: c.magnitude,
                    gamma: c.gamma,
                    saros_series: saros_series(EclipseKind::Solar, greatest_jd),
                    eclipsed_longitude,
                    near_node,
                    greatest_eclipse_location: Some(self.sub_shadow_point(&sample, greatest_jd)),
                }
            }
            Syzygy::FullMoon => {
                let Some(c) = classify_lunar(&sample) else { return Ok(None) };
                Eclipse {
                    kind: EclipseKind::Lunar,
                    eclipse_type: EclipseType::Lunar(c.eclipse_type),
                    greatest_eclipse,
                    magnitude: c.magnitude,
                    gamma: c.gamma,
                    saros_series: saros_series(EclipseKind::Lunar, greatest_jd),
                    eclipsed_longitude,
                    near_node,
                    greatest_eclipse_location: None,
                }
            }
        };
        Ok(Some(eclipse))
    }

    /// Golden-section minimize the Sun–Moon (or Moon–antisolar) separation in a
    /// ±0.25-day bracket around the syzygy to find greatest eclipse.
    fn refine_greatest(&self, syzygy: Syzygy, syzygy_jd: f64) -> Result<f64, EclipseError> {
        use crate::geometry::separation_for;
        let phi = 0.618_033_988_75_f64;
        let (mut a, mut b) = (syzygy_jd - 0.25, syzygy_jd + 0.25);
        let mut c = b - (b - a) * phi;
        let mut d = a + (b - a) * phi;
        let mut fc = separation_for(syzygy, &sample_sun_moon(&self.backend, c)?);
        let mut fd = separation_for(syzygy, &sample_sun_moon(&self.backend, d)?);
        while (b - a) > 0.5 / 86_400.0 {
            if fc < fd {
                b = d;
                d = c;
                fd = fc;
                c = b - (b - a) * phi;
                fc = separation_for(syzygy, &sample_sun_moon(&self.backend, c)?);
            } else {
                a = c;
                c = d;
                fc = fd;
                d = a + (b - a) * phi;
                fd = separation_for(syzygy, &sample_sun_moon(&self.backend, d)?);
            }
        }
        Ok(0.5 * (a + b))
    }

    /// Sub-shadow geographic point at greatest eclipse (solar). Geocentric:
    /// latitude ≈ Moon declination, longitude from GMST and the Moon's RA.
    fn sub_shadow_point(
        &self,
        sample: &crate::ephemeris::SunMoonSample,
        greatest_jd: f64,
    ) -> GeoLocation {
        use crate::geometry::sub_shadow_point;
        sub_shadow_point(sample, greatest_jd)
    }
}
```

- [ ] **Step 4: Add the geometry helpers the engine calls**

Add to `crates/pleiades-eclipse/src/geometry.rs`:
```rust
use crate::syzygy::Syzygy;

/// Separation (radians) that greatest eclipse minimizes, per syzygy type.
pub(crate) fn separation_for(syzygy: Syzygy, sample: &SunMoonSample) -> f64 {
    match syzygy {
        Syzygy::NewMoon => separation_rad(sample),
        Syzygy::FullMoon => shadow_axis_separation_rad(sample),
    }
}

/// Geocentric sub-shadow point: latitude = Moon's equatorial declination,
/// longitude = Moon RA − GMST (east-positive), both from the mean obliquity.
pub(crate) fn sub_shadow_point(sample: &SunMoonSample, greatest_jd: f64) -> crate::types::GeoLocation {
    use pleiades_types::{Angle, EclipticCoordinates, Latitude, Longitude};
    let coords = EclipticCoordinates::new(
        Longitude::from_degrees(sample.moon_longitude_deg),
        Latitude::from_degrees(sample.moon_latitude_deg),
        Some(sample.moon_distance_au),
    );
    // Mean obliquity of date (degrees) via the standard IAU polynomial.
    let t = (greatest_jd - 2_451_545.0) / 36_525.0;
    let eps_deg = 23.439_291 - 0.013_004_2 * t;
    let equatorial = coords.to_equatorial(Angle::from_degrees(eps_deg));
    let declination = equatorial.declination.degrees();
    let ra_deg = equatorial.right_ascension.degrees();
    // GMST in degrees (IAU 1982), then geographic longitude = RA − GMST.
    let gmst_deg = (280.460_618_37 + 360.985_647_366_29 * (greatest_jd - 2_451_545.0))
        .rem_euclid(360.0);
    let mut lon = (ra_deg - gmst_deg + 180.0).rem_euclid(360.0) - 180.0;
    if lon <= -180.0 {
        lon += 360.0;
    }
    crate::types::GeoLocation { latitude_degrees: declination, longitude_degrees: lon }
}
```

Wire `mod engine;` and `pub use engine::EclipseEngine;` in `lib.rs`.

- [ ] **Step 5: Write the real-ephemeris integration test**

`crates/pleiades-eclipse/tests/known_eclipses.rs` (uses the committed packaged backend — kernel-free):
```rust
use pleiades_data::packaged_backend;
use pleiades_eclipse::{EclipseEngine, EclipseFilter, EclipseKind, EclipseType, SolarEclipseType};
use pleiades_types::{Instant, JulianDay, TimeScale};

fn at(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

#[test]
fn finds_the_1999_august_11_total_solar_eclipse() {
    let engine = EclipseEngine::new(packaged_backend());
    // Search a tight window around 1999-08-11.
    let eclipses = engine
        .eclipses_in_range(at(2_451_400.0), at(2_451_410.0), EclipseFilter::SolarOnly)
        .unwrap();
    let e = eclipses
        .iter()
        .find(|e| e.kind == EclipseKind::Solar)
        .expect("a solar eclipse in this window");
    assert_eq!(e.eclipse_type, EclipseType::Solar(SolarEclipseType::Total));
    // Greatest eclipse was 1999-08-11 ~11:03 UT → JD ≈ 2451401.96, within 1 min.
    let jd = e.greatest_eclipse.julian_day().days();
    assert!((jd - 2_451_401.961).abs() < 60.0 / 86_400.0, "jd was {jd}");
}
```
Add `pleiades-data = { workspace = true }` to `pleiades-eclipse/Cargo.toml` `[dev-dependencies]`.

> Implementer note: confirm the exact greatest-eclipse JD for 1999-08-11 from the NASA canon when you write the assertion; if the packaged Moon is off by more than 60 s here, that is a real finding — stop and report it (the spec's accuracy claim depends on it), do not loosen the tolerance.

- [ ] **Step 6: Run all crate tests**

Run: `cargo test -p pleiades-eclipse`
Expected: PASS (unit + integration).

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-eclipse Cargo.toml
git commit -m "feat(eclipse): assemble EclipseEngine with range/next/previous queries"
```

---

## Task 10: NASA canon fixture + `validate-eclipses` gate

**Files:**
- Create: `crates/pleiades-validate/data/eclipses-corpus/eclipses.csv`
- Create: `crates/pleiades-validate/data/eclipses-corpus/MANIFEST.md`
- Create: `crates/pleiades-validate/src/eclipse_validation.rs`
- Modify: `crates/pleiades-validate/Cargo.toml` (add `pleiades-eclipse`, `pleiades-data` deps)
- Modify: `crates/pleiades-validate/src/lib.rs` (module)
- Modify: `crates/pleiades-eclipse/src/saros.rs` (extend `SAROS_ANCHORS` from the canon)

**Interfaces:**
- Produces:
  - `pub fn validate_eclipse_corpus() -> Result<EclipseCorpusReport, EclipseCorpusError>`
  - `pub struct EclipseCorpusReport { checked: usize }` with `summary_line(&self) -> String`
  - `pub enum EclipseCorpusError { TimeExceeded {..}, MagnitudeExceeded {..}, TypeMismatch {..}, SarosMismatch {..}, LongitudeExceeded {..}, Missing {..} }` (fail-closed)

- [ ] **Step 1: Add the committed fixture (provenance first)**

`crates/pleiades-validate/data/eclipses-corpus/MANIFEST.md`:
```markdown
# Eclipse validation corpus

Source: NASA Five Millennium Canon of Solar Eclipses and Five Millennium
Canon of Lunar Eclipses (Espenak & Meeus), restricted to 1900-01-01 …
2100-12-31. Exhaustive: every solar and lunar eclipse in the window.

Columns (eclipses.csv):
kind,type,greatest_eclipse_jd_tt,magnitude,saros,eclipsed_longitude_deg
- kind: solar | lunar
- type: total | annular | hybrid | partial | penumbral
- greatest_eclipse_jd_tt: Julian Day (TT) of greatest eclipse
- magnitude: catalog magnitude (umbral for solar/lunar; penumbral for penumbral lunar)
- saros: Saros series number
- eclipsed_longitude_deg: apparent geocentric solar longitude of date (lunar: +180°), degrees
```

`crates/pleiades-validate/data/eclipses-corpus/eclipses.csv` — header plus every 1900–2100 eclipse. Seed with at least these rows and complete from the canon:
```csv
kind,type,greatest_eclipse_jd_tt,magnitude,saros,eclipsed_longitude_deg
solar,total,2451401.9612,1.029,145,138.42
lunar,total,2458504.9028,1.195,134,121.30
solar,total,2457987.2681,1.031,145,148.78
```
> Implementer note: produce the full CSV by parsing the NASA canon tables (a one-off script outside the crate). Commit the data file only. While building it, also extract one `(kind, member_jd, series)` per active series and paste those into `SAROS_ANCHORS` in `pleiades-eclipse/src/saros.rs` so every fixture row's Saros resolves.

- [ ] **Step 2: Write the failing gate test**

Append to `crates/pleiades-validate/src/eclipse_validation.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_canon_eclipse_recomputes_within_tolerance() {
        // Fail-closed: any drift returns Err and fails the test.
        let report = validate_eclipse_corpus().expect("eclipse corpus must validate");
        assert!(report.checked >= 900, "expected the exhaustive 1900–2100 set");
    }
}
```

- [ ] **Step 3: Write the gate**

Prepend to `crates/pleiades-validate/src/eclipse_validation.rs`:
```rust
//! Fail-closed gate: recompute every NASA-canon eclipse and compare.

use pleiades_data::packaged_backend;
use pleiades_eclipse::{EclipseEngine, EclipseFilter, EclipseType, LunarEclipseType, SolarEclipseType};
use pleiades_types::{Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/eclipses-corpus/eclipses.csv"
));

const TIME_TOLERANCE_SECONDS: f64 = 60.0;
const MAGNITUDE_TOLERANCE: f64 = 0.01;
const LONGITUDE_TOLERANCE_DEG: f64 = 1.0 / 3600.0; // 1 arcsecond

pub struct EclipseCorpusReport {
    pub checked: usize,
}

impl EclipseCorpusReport {
    pub fn summary_line(&self) -> String {
        format!(
            "validate-eclipses: {} NASA-canon eclipses recomputed within \
             ≤{TIME_TOLERANCE_SECONDS}s / ≤{MAGNITUDE_TOLERANCE} mag, type & saros exact",
            self.checked
        )
    }
}

#[derive(Debug)]
pub enum EclipseCorpusError {
    Missing { kind: String, expected_jd: f64 },
    TimeExceeded { expected_jd: f64, actual_jd: f64 },
    MagnitudeExceeded { expected_jd: f64, expected: f64, actual: f64 },
    TypeMismatch { expected_jd: f64, expected: String, actual: String },
    SarosMismatch { expected_jd: f64, expected: u32, actual: u32 },
    LongitudeExceeded { expected_jd: f64, delta_deg: f64 },
    Parse { line: usize },
}

impl std::fmt::Display for EclipseCorpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for EclipseCorpusError {}

fn type_label(t: EclipseType) -> &'static str {
    match t {
        EclipseType::Solar(SolarEclipseType::Total) => "total",
        EclipseType::Solar(SolarEclipseType::Annular) => "annular",
        EclipseType::Solar(SolarEclipseType::Hybrid) => "hybrid",
        EclipseType::Solar(SolarEclipseType::Partial) => "partial",
        EclipseType::Lunar(LunarEclipseType::Total) => "total",
        EclipseType::Lunar(LunarEclipseType::Partial) => "partial",
        EclipseType::Lunar(LunarEclipseType::Penumbral) => "penumbral",
    }
}

pub fn validate_eclipse_corpus() -> Result<EclipseCorpusReport, EclipseCorpusError> {
    let engine = EclipseEngine::new(packaged_backend());
    let mut checked = 0;

    for (i, line) in CORPUS_CSV.lines().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() != 6 {
            return Err(EclipseCorpusError::Parse { line: i + 1 });
        }
        let kind = cols[0];
        let exp_type = cols[1];
        let exp_jd: f64 = cols[2].parse().map_err(|_| EclipseCorpusError::Parse { line: i + 1 })?;
        let exp_mag: f64 = cols[3].parse().map_err(|_| EclipseCorpusError::Parse { line: i + 1 })?;
        let exp_saros: u32 = cols[4].parse().map_err(|_| EclipseCorpusError::Parse { line: i + 1 })?;
        let exp_lon: f64 = cols[5].parse().map_err(|_| EclipseCorpusError::Parse { line: i + 1 })?;

        let filter = match kind {
            "solar" => EclipseFilter::SolarOnly,
            _ => EclipseFilter::LunarOnly,
        };
        // Search a ±1-day bracket around the catalog time and take the match.
        let start = Instant::new(JulianDay::from_days(exp_jd - 1.0), TimeScale::Tdb);
        let end = Instant::new(JulianDay::from_days(exp_jd + 1.0), TimeScale::Tdb);
        let found = engine
            .eclipses_in_range(start, end, filter)
            .map_err(|_| EclipseCorpusError::Missing { kind: kind.into(), expected_jd: exp_jd })?;
        let e = found
            .into_iter()
            .min_by(|a, b| {
                let da = (a.greatest_eclipse.julian_day().days() - exp_jd).abs();
                let db = (b.greatest_eclipse.julian_day().days() - exp_jd).abs();
                da.partial_cmp(&db).unwrap()
            })
            .ok_or(EclipseCorpusError::Missing { kind: kind.into(), expected_jd: exp_jd })?;

        let actual_jd = e.greatest_eclipse.julian_day().days();
        if (actual_jd - exp_jd).abs() * 86_400.0 > TIME_TOLERANCE_SECONDS {
            return Err(EclipseCorpusError::TimeExceeded { expected_jd: exp_jd, actual_jd });
        }
        if type_label(e.eclipse_type) != exp_type {
            return Err(EclipseCorpusError::TypeMismatch {
                expected_jd: exp_jd,
                expected: exp_type.into(),
                actual: type_label(e.eclipse_type).into(),
            });
        }
        if (e.magnitude - exp_mag).abs() > MAGNITUDE_TOLERANCE {
            return Err(EclipseCorpusError::MagnitudeExceeded {
                expected_jd: exp_jd,
                expected: exp_mag,
                actual: e.magnitude,
            });
        }
        if e.saros_series != exp_saros {
            return Err(EclipseCorpusError::SarosMismatch {
                expected_jd: exp_jd,
                expected: exp_saros,
                actual: e.saros_series,
            });
        }
        let dlon = {
            let d = (e.eclipsed_longitude.degrees() - exp_lon).abs() % 360.0;
            d.min(360.0 - d)
        };
        if dlon > LONGITUDE_TOLERANCE_DEG {
            return Err(EclipseCorpusError::LongitudeExceeded { expected_jd: exp_jd, delta_deg: dlon });
        }
        checked += 1;
    }

    Ok(EclipseCorpusReport { checked })
}
```

Add to `crates/pleiades-validate/Cargo.toml` `[dependencies]`:
```toml
pleiades-eclipse = { workspace = true }
pleiades-data = { workspace = true }
```
Add to `crates/pleiades-validate/src/lib.rs`:
```rust
pub mod eclipse_validation;
```

- [ ] **Step 4: Run the gate test**

Run: `cargo test -p pleiades-validate eclipse_validation`
Expected: PASS once the full CSV is committed and Saros anchors are populated. If a row fails, fix the geometry/anchor — do not delete the row or widen tolerances.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate crates/pleiades-eclipse/src/saros.rs
git commit -m "feat(eclipse): add exhaustive validate-eclipses gate against the NASA canon"
```

---

## Task 11: CLI surface + release-gate wiring

**Files:**
- Modify: `crates/pleiades-validate/src/lib.rs` (`render_cli` dispatch + release-gate inclusion)
- Modify: `crates/pleiades-cli/src/cli.rs` (forward the two new subcommands)
- Test: `crates/pleiades-validate/src/tests/` (new render test) and a `pleiades-cli` forward test

**Interfaces:**
- Produces CLI subcommands:
  - `validate-eclipses` → runs the gate, prints `summary_line()`, exits non-zero on `Err`.
  - `eclipses --start <iso-or-jd> --end <iso-or-jd> [--solar|--lunar]` → lists eclipses.
- `release-smoke` / `release-gate` additionally invoke `validate_eclipse_corpus()` and fail closed.

- [ ] **Step 1: Write the failing render test**

In `crates/pleiades-validate/src/lib.rs` test module (or a new `tests/eclipse_cli.rs`):
```rust
#[test]
fn validate_eclipses_command_reports_a_summary() {
    let out = render_cli(&["validate-eclipses"]).expect("gate passes");
    assert!(out.contains("validate-eclipses"));
    assert!(out.contains("NASA-canon"));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p pleiades-validate validate_eclipses_command`
Expected: FAIL (unknown subcommand).

- [ ] **Step 3: Add the dispatch**

In `pleiades-validate`'s `render_cli`, add arms (match the existing arm style returning `Result<String, String>`):
```rust
Some("validate-eclipses") => crate::eclipse_validation::validate_eclipse_corpus()
    .map(|r| r.summary_line())
    .map_err(|e| e.to_string()),
Some("eclipses") => crate::eclipse_validation::render_eclipses_listing(args),
```
Add a small `render_eclipses_listing(args: &[&str]) -> Result<String, String>` to `eclipse_validation.rs` that parses `--start`/`--end`/`--solar`/`--lunar`, builds the engine, and formats one line per eclipse (`<jd> <kind> <type> mag=<m> saros=<s> lon=<deg>`). Keep parsing minimal and reuse the window constants.

In the `release-smoke`/`release-gate` builder, add a step that calls `validate_eclipse_corpus()` and turns `Err` into a release-gate failure alongside the existing `validate_house_corpus` / corpus gates (find where those are invoked and mirror the pattern).

- [ ] **Step 4: Forward from `pleiades-cli`**

In `crates/pleiades-cli/src/cli.rs`, add to the `render_cli` match (alongside the other forwarded `validate-*` arms):
```rust
Some("validate-eclipses") => validate_render_cli(args),
Some("eclipses") => validate_render_cli(args),
```

- [ ] **Step 5: Run the tests**

Run: `cargo test -p pleiades-validate && cargo test -p pleiades-cli`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-validate crates/pleiades-cli
git commit -m "feat(eclipse): wire validate-eclipses + eclipses CLI and release gate"
```

---

## Task 12: Public docs, rustdoc examples, and truthful release claims

**Files:**
- Modify: `crates/pleiades-eclipse/src/lib.rs` (crate-level rustdoc example)
- Modify: `crates/pleiades-eclipse/README.md`
- Modify: `README.md` (root — add eclipse support to "Current state", truthfully scoped)
- Modify: the compatibility-profile prose if it enumerates capabilities (search for where backends/capabilities are listed)

**Interfaces:** none new — documentation and claims only.

- [ ] **Step 1: Add a doctest to the crate root**

Append to `crates/pleiades-eclipse/src/lib.rs`:
```rust
/// # Example
///
/// ```
/// use pleiades_data::packaged_backend;
/// use pleiades_eclipse::{EclipseEngine, EclipseFilter};
/// use pleiades_types::{Instant, JulianDay, TimeScale};
///
/// let engine = EclipseEngine::new(packaged_backend());
/// let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
/// let next = engine.next_eclipse(after, EclipseFilter::All).unwrap();
/// assert!(next.is_some());
/// ```
fn _doc_anchor() {}
```
Add `pleiades-data` and `pleiades-types` to `[dev-dependencies]` if not already present (Task 9 added `pleiades-data`).

- [ ] **Step 2: Run the doctest**

Run: `cargo test -p pleiades-eclipse --doc`
Expected: PASS.

- [ ] **Step 3: Update the root README "Current state"**

Add a bullet, scoped exactly to what shipped (no overclaim of local circumstances):
```markdown
- global/geocentric solar and lunar eclipse data (type, greatest-eclipse time,
  magnitude, gamma, Saros series, eclipsed longitude, and solar greatest-eclipse
  location) for 1900–2100 CE via `pleiades-eclipse`, validated exhaustively
  against NASA's Five Millennium Canon by the fail-closed `validate-eclipses`
  gate; local (per-observer) circumstances are not provided.
```

- [ ] **Step 4: Reconcile any capability/compatibility prose**

Search for where the workspace enumerates shipped capabilities (`grep -rn "validate-houses" crates/pleiades-validate/src` and the compatibility-profile module). Add `validate-eclipses` to any release-gate listing so the gate set stays truthful. Do not claim eclipse support in the per-body backend claims (eclipses are a derived product, not a backend body).

- [ ] **Step 5: Full workspace check**

Run: `cargo test --workspace && cargo fmt --all -- --check && cargo clippy --workspace --all-targets`
Expected: PASS / no warnings (fix formatting and clippy findings before committing).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-eclipse README.md crates/pleiades-validate
git commit -m "docs(eclipse): rustdoc example, READMEs, and truthful release claims"
```

---

## Self-Review (completed against the spec)

**Spec coverage:**
- Global/geocentric solar+lunar, 1900–2100 → Tasks 6–9 + window constants (Task 3). ✓
- Range scan + next/previous, unified `Eclipse` + filter → Task 9. ✓
- Result fields (type, greatest_eclipse, magnitude, gamma, saros, eclipsed_longitude, near_node, solar location) → Task 2 (shape) + Task 9 (population). ✓
- Penumbral lunar included → Task 7 classification + fixture seed (Task 10). ✓
- Approach A (derive from gated Sun/Moon positions; no new data) → Tasks 4–9; engine takes a backend, owns no data. ✓
- `validate-eclipses` exhaustive, fail-closed, ≤60 s / ≤0.01 / exact type & saros, wired into release gates → Tasks 10–11. ✓
- Crate layering (deps on types/backend/apparent/time; CLI; gate in validate) → Tasks 1, 10, 11. ✓
- Tropical-of-date eclipsed_longitude; no ayanamsa → Task 9 builds it from solar longitude only. ✓
- Truthful claims, no local-circumstance overclaim → Task 12. ✓

**Placeholder scan:** Two deliberate data-build steps (the full NASA CSV and the extracted Saros anchors) are flagged as one-off extraction in Task 10 with exact column formats and seed rows — the code that consumes them is complete. No "TODO/handle edge cases" left in code.

**Type consistency:** `EclipseEngine::new`, `eclipses_in_range/next_eclipse/previous_eclipse`, `EclipseFilter::admits`, `EclipseType::kind`, `classify_solar/classify_lunar`, `find_syzygies`, `saros_series`, `validate_eclipse_corpus`/`EclipseCorpusReport.summary_line` are used with identical signatures across tasks. `apparent`/`apparentness` and `EclipticCoordinates { longitude, latitude, distance_au }` match the real types confirmed in the codebase.

**Open verification item for the implementer (Task 4 note):** a handful of thin backend constructors (`BackendMetadata::test_placeholder`, `EphemerisResult::with_ecliptic`/`without_coordinates`, `EphemerisError::unsupported_body`, `Instant::julian_day`) are assumed; if absent, add minimal `pub` versions in the same commit, matching existing style.
</content>
