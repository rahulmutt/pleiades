# SP-2b Rise/Set/Transit + Horizontal Coordinates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add observer-local event finding to the pleiades workspace — horizontal coordinates (`swe_azalt`/`swe_azalt_rev`) and full-flag rise/set/meridian-transit (`swe_rise_trans`), for the release-grade bodies, an arbitrary ecliptic point, and a curated fixed-star set — validated fail-closed against Swiss Ephemeris.

**Architecture:** Extend the existing standalone `pleiades-events` crate. A new `refraction` module in `pleiades-apparent` adds the (previously omitted) atmospheric-refraction correction in both directions. `pleiades-events` gains `horizontal` (azalt/azalt_rev on top of the existing topocentric + sidereal pipeline), `fixstar` (curated apparent-place path), and `rise_trans` (rise/set/transit root-finding on the shared `root.rs` bracket-and-bisect). `CrossingEngine` is renamed `EventEngine` (deprecated alias retained). An isolated out-of-workspace `tools/se-rise-trans-reference` harness emits a committed corpus, and a two-tier `validate-rise-trans` gate in `pleiades-validate` re-checks self-consistency and SE parity.

**Tech Stack:** Rust (2021 edition, workspace), `pleiades-backend`/`pleiades-types`/`pleiades-apparent`, Swiss Ephemeris (`swisseph`/`libswisseph-sys`, isolated tool only), CSV corpus + manifest.

## Global Constraints

- **Pure-Rust workspace audit (hard):** no `-sys`/`links`/`build.rs` may enter the workspace lockfile. The SE binding lives ONLY in `tools/se-rise-trans-reference` (its own `Cargo.lock`, `publish = false`, outside `[workspace].members`). `pleiades-events`, `pleiades-apparent`, and `pleiades-validate` must not depend on any SE/FFI crate. The curated `fixstars-catalog.csv` carries Hipparcos-derived astrometry (public source data), not SE-distributed files.
- **Coverage window:** 1900-01-01 TDB (JD `2_415_020.5`) through 2100-01-01 TDB (JD `2_488_069.5`). Out-of-window requests fail closed. Reuse the existing `WINDOW_START_JD`/`WINDOW_END_JD` constants from `pleiades-events`.
- **Time base:** all engine results are TDB (`TimeScale::Tdb`), matching crossings. SE `_ut` corpus times are converted to TDB once, at generation.
- **Fail-closed everywhere:** never emit NaN/placeholder; return a structured `EventError`. Circumpolar / never-rises is `Ok(None)`, distinct from an error. Backends supply mean/J2000 geocentric ecliptic coordinates; apparent/topocentric corrections are applied in-crate.
- **Azimuth convention:** match `swe_azalt` exactly — azimuth measured from **south, increasing westward**, degrees `[0,360)`. Pin against the corpus. Document the convention on `Horizontal::azimuth`.
- **Atmosphere defaults:** `Atmosphere::default()` = `1013.25` mbar, `15.0` °C (SE defaults), so plain callers match SE without setting anything.
- **Edition/versioning:** crate fields use `.workspace = true`. New public surface + refraction provenance flip → bump compatibility profile (`0.7.6` → `0.7.7`). The `CrossingEngine` → `EventEngine` rename is a one-cycle deprecation → bump API-stability profile (`0.2.1` → `0.2.2`).
- **libclang note:** building `tools/se-rise-trans-reference` needs `libclang-dev` + `LIBCLANG_PATH`. Required ONLY to (re)generate the corpus — never to build the workspace or run `validate-rise-trans` (which reads the committed CSV via `include_str!`).
- **Do not perturb passing gates:** `validate-eclipses`, `validate-crossings`, `validate-angles`, `validate-houses` must still pass unchanged. The refraction addition and the engine rename ship without touching their code paths (the rename is aliased).

---

## File Structure

**Refraction (existing crate `crates/pleiades-apparent/`):**
- Create `src/refraction.rs` — `Atmosphere`, `apparent_from_true`, `true_from_apparent`
- Modify `src/lib.rs` — `pub mod refraction;` + re-exports
- Modify `src/provenance.rs` — flip the "atmospheric refraction omitted" line (final bookkeeping task, gated)

**Events (existing crate `crates/pleiades-events/`):**
- Modify `src/crossings.rs` → rename `CrossingEngine` to `EventEngine`; keep `pub type CrossingEngine`
- Modify `src/lib.rs` — new modules + re-exports; update crate doctest
- Create `src/horizontal.rs` — `Horizontal`, `HorizontalInput`, azalt / azalt_rev methods
- Create `src/fixstar.rs` — curated catalog reader + `fixed_star_apparent`
- Create `src/rise_trans.rs` — `RiseSetEvent`, `DiscMode`, `RiseSetTarget`, `RiseSetOptions`, `Atmosphere` re-export, `RiseSet`, engine methods
- Create `src/semidiameter.rs` — per-body apparent semidiameter + SE disc-radius table
- Modify `src/error.rs` — add `UnknownFixedStar`, `InvalidObserver`, `InvalidAtmosphere` variants
- Create `data/fixstars-catalog.csv` — curated ~30-star J2000 astrometry
- Modify `Cargo.toml` — add `pleiades-apparent` refraction use (already a dep)

**Validation:**
- `tools/se-rise-trans-reference/{Cargo.toml,Cargo.lock,src/main.rs,LICENSE-NOTES.md}` — isolated SE harness
- `crates/pleiades-validate/data/rise-trans-corpus/{rise-trans.csv,azalt.csv,manifest.txt}` — committed corpus
- `crates/pleiades-validate/src/rise_trans_validation.rs` — the two-tier gate
- `crates/pleiades-validate/src/rise_trans_thresholds.rs` — per-row ceilings
- Modify `crates/pleiades-validate/{Cargo.toml,src/lib.rs,src/render/cli.rs,src/release/notes.rs}` — dep + export + dispatch + smoke list

**CLI:**
- Modify `crates/pleiades-cli/src/cli.rs` — `rise-trans` + `azalt` aliases routed through validate render

**Compatibility/docs:**
- Modify `crates/pleiades-core/src/compatibility/mod.rs` — profile entries + version bump
- Modify `README.md`, `PLAN.md`, `plan/status/*` — current-state + status refresh

---

## Phase 1 — Atmospheric refraction (`pleiades-apparent`)

Independently landable: adds a reusable correction with its own unit tests. No public behavior change elsewhere until rise/set consumes it.

## Task 1: `Atmosphere` + true→apparent refraction (Bennett)

**Files:**
- Create: `crates/pleiades-apparent/src/refraction.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs`
- Test: inline `#[cfg(test)]` in `refraction.rs`

**Interfaces:**
- Produces: `pub struct Atmosphere { pub pressure_mbar: f64, pub temperature_c: f64 }` with `impl Default` (1013.25 / 15.0); `pub fn apparent_from_true(true_alt_deg: f64, atmos: Atmosphere) -> f64`.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_atmosphere_is_se_standard() {
        let a = Atmosphere::default();
        assert_eq!(a.pressure_mbar, 1013.25);
        assert_eq!(a.temperature_c, 15.0);
    }

    #[test]
    fn refraction_at_horizon_is_about_34_arcmin() {
        // Bennett at h=0 with standard atmosphere ≈ 28.7' true→apparent lift is
        // computed on APPARENT altitude; at true h=0 the raise is ~34'. Assert the
        // apparent altitude sits ~34'(=0.567°) above 0 within a loose band.
        let app = apparent_from_true(0.0, Atmosphere::default());
        assert!((app - 0.4752).abs() < 0.05, "apparent horizon altitude {app}");
    }

    #[test]
    fn refraction_vanishes_at_zenith() {
        let app = apparent_from_true(90.0, Atmosphere::default());
        assert!((app - 90.0).abs() < 1e-4, "zenith {app}");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-apparent refraction::tests -- --nocapture`
Expected: FAIL — `apparent_from_true` / `Atmosphere` not found.

- [ ] **Step 3: Write minimal implementation**

```rust
//! Atmospheric refraction (Bennett 1982 / Saemundsson 1986), pressure- and
//! temperature-scaled. Historically omitted from the apparent-place pipeline;
//! rise/set and horizontal coordinates require it. Matches Swiss Ephemeris
//! `swe_refrac` conventions so `validate-rise-trans` can prove parity.

/// Observer atmosphere used to scale refraction. Defaults are the SE standard
/// (`1013.25` mbar, `15` °C).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Atmosphere {
    /// Atmospheric pressure at the observer, millibars.
    pub pressure_mbar: f64,
    /// Atmospheric temperature at the observer, degrees Celsius.
    pub temperature_c: f64,
}

impl Default for Atmosphere {
    fn default() -> Self {
        Self { pressure_mbar: 1013.25, temperature_c: 15.0 }
    }
}

fn scale(atmos: Atmosphere) -> f64 {
    (atmos.pressure_mbar / 1010.0) * (283.0 / (273.0 + atmos.temperature_c))
}

/// True (geometric) altitude → apparent altitude, degrees. Bennett (1982):
/// `R = 1.02 / tan(h + 10.3/(h + 5.11))` arcmin, evaluated on the true altitude,
/// pressure/temperature scaled. `apparent = true + R`.
pub fn apparent_from_true(true_alt_deg: f64, atmos: Atmosphere) -> f64 {
    let h = true_alt_deg;
    let r_arcmin = scale(atmos) * 1.02
        / ((h + 10.3 / (h + 5.11)).to_radians().tan());
    true_alt_deg + r_arcmin / 60.0
}
```

Add to `crates/pleiades-apparent/src/lib.rs` after the other `pub mod` lines:

```rust
pub mod refraction;
pub use refraction::{apparent_from_true, true_from_apparent, Atmosphere};
```

(`true_from_apparent` lands in Task 2; add its name to the re-export now and stub it as `pub fn true_from_apparent(apparent_alt_deg: f64, atmos: Atmosphere) -> f64 { apparent_alt_deg }` so the crate compiles — Task 2 replaces the body.)

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-apparent refraction::tests`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-apparent/src/refraction.rs crates/pleiades-apparent/src/lib.rs
git commit -m "feat(apparent): Bennett true->apparent atmospheric refraction + Atmosphere"
```

## Task 2: apparent→true refraction (Saemundsson) + round-trip

**Files:**
- Modify: `crates/pleiades-apparent/src/refraction.rs`
- Test: inline

**Interfaces:**
- Produces: `pub fn true_from_apparent(apparent_alt_deg: f64, atmos: Atmosphere) -> f64` (replaces the Task 1 stub).

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn saemundsson_inverts_bennett_within_a_few_arcsec() {
    // Round-trip: for altitudes above the horizon the two formulae are near-inverses.
    for h in [5.0, 15.0, 45.0, 80.0] {
        let app = apparent_from_true(h, Atmosphere::default());
        let back = true_from_apparent(app, Atmosphere::default());
        assert!((back - h).abs() < 0.01, "round-trip h={h} back={back}");
    }
}

#[test]
fn true_from_apparent_at_horizon_is_about_negative_34_arcmin() {
    // A body seen ON the apparent horizon (h_app=0) is geometrically ~34' below it.
    let t = true_from_apparent(0.0, Atmosphere::default());
    assert!((t + 0.5667).abs() < 0.02, "true altitude at apparent horizon {t}");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-apparent refraction::tests::saemundsson_inverts_bennett_within_a_few_arcsec`
Expected: FAIL — stub returns the input unchanged.

- [ ] **Step 3: Write minimal implementation**

```rust
/// Apparent altitude → true (geometric) altitude, degrees. Saemundsson (1986):
/// `R = 1.0 / tan(h + 7.31/(h + 4.4))` arcmin, evaluated on the apparent
/// altitude, pressure/temperature scaled. `true = apparent - R`.
pub fn true_from_apparent(apparent_alt_deg: f64, atmos: Atmosphere) -> f64 {
    let h = apparent_alt_deg;
    let r_arcmin = scale(atmos) * 1.0
        / ((h + 7.31 / (h + 4.4)).to_radians().tan());
    apparent_alt_deg - r_arcmin / 60.0
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-apparent refraction::tests`
Expected: PASS (5 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-apparent/src/refraction.rs
git commit -m "feat(apparent): Saemundsson apparent->true refraction inverse"
```

> **Open item pinned during the gate (Task 17):** the below-horizon branch (`h < -5°`) is refined to match SE's exact clamp/blend once corpus rows exercise it. Until then `true_from_apparent`/`apparent_from_true` are used only for `h ≥ 0` inside rise/set (the standard-altitude target is near 0), so the naive formula is adequate for phases 1–4 and tightened in phase 5.

---

## Phase 2 — Engine rename + horizontal coordinates (`pleiades-events`)

## Task 3: Rename `CrossingEngine` → `EventEngine` (deprecated alias)

**Files:**
- Modify: `crates/pleiades-events/src/crossings.rs`
- Modify: `crates/pleiades-events/src/lib.rs`
- Modify: `crates/pleiades-events/README.md`
- Modify: `crates/pleiades-cli/src/cli.rs` (any `CrossingEngine` reference in the crossings alias)
- Test: existing crossings tests must pass unchanged

**Interfaces:**
- Produces: `pub struct EventEngine<B>`; `#[deprecated] pub type CrossingEngine<B> = EventEngine<B>;`. All existing crossing methods now hang off `EventEngine`.

- [ ] **Step 1: Rename the struct and add the alias**

In `crates/pleiades-events/src/crossings.rs` rename the struct and its `impl`:

```rust
/// Finds ephemeris events (longitude crossings today; rise/set/transit and
/// horizontal coordinates in sibling modules) over the packaged 1900–2100 TDB
/// window.
pub struct EventEngine<B> {
    pub(crate) backend: B,
}

impl<B: EphemerisBackend> EventEngine<B> { /* ...unchanged body... */ }
```

At the bottom of the module add:

```rust
/// Deprecated alias for [`EventEngine`]. Kept one release cycle for the SP-2a
/// crossing API; migrate to `EventEngine`.
#[deprecated(since = "0.3.1", note = "renamed to EventEngine")]
pub type CrossingEngine<B> = EventEngine<B>;
```

- [ ] **Step 2: Update re-exports and doctests**

In `crates/pleiades-events/src/lib.rs`:

```rust
pub use crossings::{Crossing, CrossingEngine, CrossingFrame, EventEngine};
```

Update the crate-level doctest and `longitude_at`'s doctest to construct `EventEngine::new(...)` (the `#[deprecated]` alias would otherwise warn under `#![deny(warnings)]` in doctests). Leave the in-module unit tests using `EventEngine` too (find/replace `CrossingEngine::new` → `EventEngine::new` in `crossings.rs` tests).

- [ ] **Step 3: Run the crossings suite + doctests**

Run: `cargo test -p pleiades-events && cargo test -p pleiades-events --doc`
Expected: PASS — same test count as before the rename, no deprecation warnings.

- [ ] **Step 4: Confirm downstream compiles**

Run: `cargo build -p pleiades-cli`
Expected: builds; if the crossings CLI alias referenced `CrossingEngine`, switch it to `EventEngine`.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events crates/pleiades-cli/src/cli.rs
git commit -m "refactor(events): rename CrossingEngine to EventEngine (deprecated alias kept)"
```

## Task 4: `horizontal` — azalt (equatorial + ecliptic input)

**Files:**
- Create: `crates/pleiades-events/src/horizontal.rs`
- Modify: `crates/pleiades-events/src/lib.rs`
- Modify: `crates/pleiades-events/src/error.rs`
- Test: inline

**Interfaces:**
- Consumes: `pleiades_apparent::{apparent_equatorial_of_date, true_obliquity_degrees, sidereal_time, topocentric_position, apparent_from_true, Atmosphere}`; `EventEngine`.
- Produces:
  ```rust
  pub struct Horizontal { pub azimuth: f64, pub true_altitude: f64, pub apparent_altitude: f64 }
  pub enum HorizontalInput { Ecliptic(Longitude, Latitude), Equatorial(Angle /*RA*/, Latitude /*Dec*/) }
  impl<B: EphemerisBackend> EventEngine<B> {
      pub fn horizontal(&self, input: HorizontalInput, observer: ObserverLocation,
                        atmos: Atmosphere, at: Instant) -> Result<Horizontal, EventError>;
  }
  ```
- Adds to `EventError`: `InvalidObserver { detail: String }`.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_apparent::Atmosphere;
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{Angle, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};

    fn tdb(jd: f64) -> Instant { Instant::new(JulianDay::from_days(jd), TimeScale::Tdb) }
    fn greenwich() -> ObserverLocation {
        ObserverLocation::new(Latitude::from_degrees(51.48), Longitude::from_degrees(0.0), None)
    }

    #[test]
    fn object_on_local_meridian_has_azimuth_zero_or_180() {
        // A body whose RA equals the local apparent sidereal time is on the meridian:
        // hour angle 0 → azimuth 0 (south) if it is south of zenith.
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let at = tdb(2_451_545.0);
        let st = pleiades_apparent::sidereal_time(at, Longitude::from_degrees(0.0));
        let ra = Angle::from_degrees(st.local_apparent_deg);
        let dec = Latitude::from_degrees(10.0); // south of a 51°N observer's zenith
        let h = engine
            .horizontal(HorizontalInput::Equatorial(ra, dec), greenwich(), Atmosphere::default(), at)
            .unwrap();
        assert!(h.azimuth.abs() < 1e-3 || (h.azimuth - 360.0).abs() < 1e-3, "az {}", h.azimuth);
        assert!(h.apparent_altitude >= h.true_altitude, "refraction lifts the body");
    }

    #[test]
    fn altitude_never_exceeds_ninety() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let at = tdb(2_451_545.0);
        let h = engine
            .horizontal(
                HorizontalInput::Equatorial(Angle::from_degrees(0.0), Latitude::from_degrees(51.48)),
                greenwich(), Atmosphere::default(), at)
            .unwrap();
        assert!(h.true_altitude <= 90.0 + 1e-9);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-events horizontal::tests`
Expected: FAIL — module/types not defined.

- [ ] **Step 3: Write minimal implementation**

```rust
//! Horizontal coordinates (`swe_azalt` / `swe_azalt_rev`): azimuth and altitude
//! of a target for a topocentric observer. Azimuth is measured from SOUTH,
//! increasing WESTWARD, degrees `[0,360)`, matching Swiss Ephemeris.

use crate::crossings::EventEngine;
use crate::error::EventError;
use pleiades_apparent::{
    apparent_from_true, sidereal_time, true_obliquity_degrees, Atmosphere,
};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{
    Angle, EclipticCoordinates, Instant, Latitude, Longitude, ObserverLocation,
};

/// Azimuth (from south, westward) plus true (geometric) and apparent (refracted)
/// altitude, all degrees.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct Horizontal {
    /// Azimuth measured from south, increasing westward, `[0,360)` degrees.
    pub azimuth: f64,
    /// Geometric (unrefracted) altitude, degrees.
    pub true_altitude: f64,
    /// Apparent (refracted) altitude, degrees.
    pub apparent_altitude: f64,
}

/// Input coordinate for [`EventEngine::horizontal`].
#[derive(Clone, Copy, Debug)]
pub enum HorizontalInput {
    /// Tropical apparent ecliptic of date (`SE_ECL2HOR`): longitude, latitude.
    Ecliptic(Longitude, Latitude),
    /// Apparent equatorial of date (`SE_EQU2HOR`): right ascension, declination.
    Equatorial(Angle, Latitude),
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Azimuth/altitude of `input` for `observer` at `at` (TDB).
    pub fn horizontal(
        &self,
        input: HorizontalInput,
        observer: ObserverLocation,
        atmos: Atmosphere,
        at: Instant,
    ) -> Result<Horizontal, EventError> {
        observer
            .validate()
            .map_err(|e| EventError::InvalidObserver { detail: e.to_string() })?;
        let jd = at.julian_day.days();
        let eps = true_obliquity_degrees(jd)
            .map_err(|e| EventError::Backend(format!("obliquity failed: {e}")))?;
        // Resolve to apparent equatorial RA/Dec (degrees).
        let (ra_deg, dec_deg) = match input {
            HorizontalInput::Equatorial(ra, dec) => (ra.degrees(), dec.degrees()),
            HorizontalInput::Ecliptic(lon, lat) => {
                let ecl = EclipticCoordinates::new(lon, lat, None);
                let equ = ecl.to_equatorial(Angle::from_degrees(eps));
                (equ.right_ascension.degrees(), equ.declination.degrees())
            }
        };
        // Local apparent sidereal time → local hour angle H = LST − RA.
        let lst = sidereal_time(at, observer.longitude).local_apparent_deg;
        let h_deg = lst - ra_deg;
        let (h, dec, phi) = (
            h_deg.to_radians(),
            dec_deg.to_radians(),
            observer.latitude.degrees().to_radians(),
        );
        // Standard equatorial → horizontal rotation (azimuth from south, west +).
        let sin_alt = phi.sin() * dec.sin() + phi.cos() * dec.cos() * h.cos();
        let alt = sin_alt.asin();
        let az = (h.sin()).atan2(h.cos() * phi.sin() - dec.tan() * phi.cos());
        let true_altitude = alt.to_degrees();
        Ok(Horizontal {
            azimuth: az.to_degrees().rem_euclid(360.0),
            true_altitude,
            apparent_altitude: apparent_from_true(true_altitude, atmos),
        })
    }
}
```

Add to `crates/pleiades-events/src/error.rs` (`EventError` enum + `Display`):

```rust
    /// The observer location failed validation (non-finite / out of range).
    InvalidObserver {
        /// Human-readable detail.
        detail: String,
    },
```

```rust
    EventError::InvalidObserver { detail } => write!(f, "invalid observer: {detail}"),
```

Add to `crates/pleiades-events/src/lib.rs`:

```rust
mod horizontal;
pub use horizontal::{Horizontal, HorizontalInput};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-events horizontal::tests`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/horizontal.rs crates/pleiades-events/src/lib.rs crates/pleiades-events/src/error.rs
git commit -m "feat(events): swe_azalt horizontal coordinates on EventEngine"
```

## Task 5: `horizontal_to_equatorial` (azalt_rev) + round-trip

**Files:**
- Modify: `crates/pleiades-events/src/horizontal.rs`
- Test: inline

**Interfaces:**
- Produces:
  ```rust
  impl<B: EphemerisBackend> EventEngine<B> {
      pub fn horizontal_to_equatorial(&self, azimuth_deg: f64, altitude_deg: f64,
          is_apparent: bool, observer: ObserverLocation, atmos: Atmosphere, at: Instant)
          -> Result<(Angle /*RA*/, Latitude /*Dec*/), EventError>;
  }
  ```

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn azalt_round_trips_through_equatorial() {
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let at = tdb(2_451_545.0);
    let ra_in = Angle::from_degrees(123.0);
    let dec_in = Latitude::from_degrees(17.0);
    let h = engine
        .horizontal(HorizontalInput::Equatorial(ra_in, dec_in), greenwich(), Atmosphere::default(), at)
        .unwrap();
    // Feed the TRUE altitude back (is_apparent = false) to invert the pure rotation.
    let (ra, dec) = engine
        .horizontal_to_equatorial(h.azimuth, h.true_altitude, false, greenwich(), Atmosphere::default(), at)
        .unwrap();
    let dra = pleiades_events::__test_wrap180(ra.degrees() - 123.0);
    assert!(dra.abs() < 1e-6, "ra back {}", ra.degrees());
    assert!((dec.degrees() - 17.0).abs() < 1e-6, "dec back {}", dec.degrees());
}
```

(Expose the existing `root::wrap180` for tests via `#[doc(hidden)] pub fn __test_wrap180(d: f64) -> f64 { crate::root::wrap180(d) }` in `lib.rs`, or inline the wrap in the test — pick the inline wrap if you prefer no test-only surface.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-events horizontal::tests::azalt_round_trips_through_equatorial`
Expected: FAIL — method not found.

- [ ] **Step 3: Write minimal implementation**

```rust
impl<B: EphemerisBackend> EventEngine<B> {
    /// Inverse of [`EventEngine::horizontal`] (`swe_azalt_rev`): horizontal →
    /// apparent equatorial of date. When `is_apparent` is true the altitude is
    /// de-refracted first.
    pub fn horizontal_to_equatorial(
        &self,
        azimuth_deg: f64,
        altitude_deg: f64,
        is_apparent: bool,
        observer: ObserverLocation,
        atmos: Atmosphere,
        at: Instant,
    ) -> Result<(Angle, Latitude), EventError> {
        observer
            .validate()
            .map_err(|e| EventError::InvalidObserver { detail: e.to_string() })?;
        let alt_deg = if is_apparent {
            pleiades_apparent::true_from_apparent(altitude_deg, atmos)
        } else {
            altitude_deg
        };
        let (az, alt, phi) = (
            azimuth_deg.to_radians(),
            alt_deg.to_radians(),
            observer.latitude.degrees().to_radians(),
        );
        let sin_dec = phi.sin() * alt.sin() + phi.cos() * alt.cos() * az.cos();
        let dec = sin_dec.asin();
        let h = (az.sin()).atan2(az.cos() * phi.sin() - alt.tan() * phi.cos());
        let lst = sidereal_time(at, observer.longitude).local_apparent_deg;
        let ra = (lst - h.to_degrees()).rem_euclid(360.0);
        Ok((Angle::from_degrees(ra), Latitude::from_degrees(dec.to_degrees())))
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-events horizontal::tests`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/horizontal.rs crates/pleiades-events/src/lib.rs
git commit -m "feat(events): swe_azalt_rev horizontal->equatorial inverse"
```

---

## Phase 3 — Curated fixed-star apparent place (`pleiades-events`)

## Task 6: Curated `fixstars-catalog.csv` + reader + unknown-star fail-closed

**Files:**
- Create: `crates/pleiades-events/data/fixstars-catalog.csv`
- Create: `crates/pleiades-events/src/fixstar.rs`
- Modify: `crates/pleiades-events/src/lib.rs`
- Modify: `crates/pleiades-events/src/error.rs`
- Test: inline

**Interfaces:**
- Produces:
  ```rust
  pub struct FixedStarEntry { pub name: &'static str, pub ra_j2000_deg: f64, pub dec_j2000_deg: f64,
      pub pm_ra_mas_yr: f64, pub pm_dec_mas_yr: f64, pub parallax_mas: f64, pub rv_km_s: f64 }
  pub fn fixed_star_entry(name: &str) -> Result<&'static FixedStarEntry, EventError>;
  ```
- Adds to `EventError`: `UnknownFixedStar { name: String }`.

**CSV header + provenance** (first lines of `data/fixstars-catalog.csv`; values pinned to SE `sefstars.txt` in the gate task — commit the file with real Hipparcos figures, not zeros):

```csv
# source: Hipparcos (ESA 1997) astrometry, matched to Swiss Ephemeris sefstars.txt names; epoch J2000.0 ICRS
# columns: name,ra_deg,dec_deg,pm_ra_mas_yr,pm_dec_mas_yr,parallax_mas,rv_km_s
name,ra_deg,dec_deg,pm_ra_mas_yr,pm_dec_mas_yr,parallax_mas,rv_km_s
Aldebaran,68.980163,16.509302,62.78,-189.36,48.94,54.26
Regulus,152.092962,11.967209,-249.40,4.91,41.13,5.9
Spica,201.298247,-11.161319,-42.50,-31.73,13.06,1.0
Antares,247.351915,-26.432003,-10.16,-23.21,5.89,-3.4
Fomalhaut,344.412693,-29.622237,329.22,-164.22,129.81,6.5
Sirius,101.287155,-16.716116,-546.01,-1223.07,379.21,-5.5
```

(Complete the file to the ~30-star roster from the spec — Algol, Betelgeuse, Rigel, Pollux, Deneb, Vega, Altair, Alcyone, Bellatrix, Capella, Arcturus, Procyon, Alphard, Zubenelgenubi, Zubeneschamali, Aselli/Praesepe, Castor, Denebola, Markab, Scheat, Alphecca, Vindemiatrix, Sirrah/Alpheratz, Menkar, Hamal, Mirfak — pin each row's figures against `sefstars.txt` during the gate task.)

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_star_resolves() {
        let e = fixed_star_entry("Aldebaran").unwrap();
        assert!((e.ra_j2000_deg - 68.98).abs() < 0.1);
        assert!((e.dec_j2000_deg - 16.51).abs() < 0.1);
    }

    #[test]
    fn unknown_star_fails_closed() {
        let err = fixed_star_entry("Nonesuch").unwrap_err();
        assert!(matches!(err, EventError::UnknownFixedStar { .. }));
    }

    #[test]
    fn catalog_is_nonempty_and_finite() {
        assert!(CATALOG.len() >= 25);
        for e in CATALOG {
            assert!(e.ra_j2000_deg.is_finite() && (0.0..360.0).contains(&e.ra_j2000_deg));
            assert!((-90.0..=90.0).contains(&e.dec_j2000_deg));
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-events fixstar::tests`
Expected: FAIL — module not defined.

- [ ] **Step 3: Write minimal implementation**

```rust
//! Curated fixed-star apparent place. A bounded set of astrologically-used
//! stars, astrometry matched to Swiss Ephemeris `sefstars.txt`, parsed once from
//! a committed CSV at build time.

use crate::error::EventError;

/// One catalog row: J2000 ICRS position + space motion.
#[derive(Clone, Copy, Debug)]
pub struct FixedStarEntry {
    /// SE-compatible star name.
    pub name: &'static str,
    /// Right ascension at J2000, degrees.
    pub ra_j2000_deg: f64,
    /// Declination at J2000, degrees.
    pub dec_j2000_deg: f64,
    /// Proper motion in RA (mas/yr, already `·cosδ`-free per sefstars convention).
    pub pm_ra_mas_yr: f64,
    /// Proper motion in Dec (mas/yr).
    pub pm_dec_mas_yr: f64,
    /// Parallax, milliarcseconds.
    pub parallax_mas: f64,
    /// Radial velocity, km/s.
    pub rv_km_s: f64,
}

const RAW: &str = include_str!("../data/fixstars-catalog.csv");

// Parsed at first use; the file is tiny (~30 rows). Build a static via a parse fn
// invoked in a `LazyLock` (std, no extra deps).
use std::sync::LazyLock;
pub(crate) static CATALOG: LazyLock<Vec<FixedStarEntry>> = LazyLock::new(parse_catalog);

fn parse_catalog() -> Vec<FixedStarEntry> {
    RAW.lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("name") && !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            FixedStarEntry {
                name: Box::leak(f[0].to_string().into_boxed_str()),
                ra_j2000_deg: f[1].parse().unwrap(),
                dec_j2000_deg: f[2].parse().unwrap(),
                pm_ra_mas_yr: f[3].parse().unwrap(),
                pm_dec_mas_yr: f[4].parse().unwrap(),
                parallax_mas: f[5].parse().unwrap(),
                rv_km_s: f[6].parse().unwrap(),
            }
        })
        .collect()
}

/// Looks up a curated fixed star by SE-compatible name (case-insensitive).
pub fn fixed_star_entry(name: &str) -> Result<&'static FixedStarEntry, EventError> {
    CATALOG
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| EventError::UnknownFixedStar { name: name.to_string() })
}
```

Add to `error.rs`:

```rust
    /// A fixed-star name not present in the curated catalog.
    UnknownFixedStar {
        /// The requested name.
        name: String,
    },
```
```rust
    EventError::UnknownFixedStar { name } => write!(f, "unknown fixed star: {name}"),
```

Add to `lib.rs`:

```rust
mod fixstar;
pub use fixstar::{fixed_star_entry, fixed_star_apparent, FixedStarEntry};
```

(`fixed_star_apparent` lands in Task 7; add its re-export now with a stub or land Task 7 before re-exporting — prefer landing Task 7 first if you want a clean compile at this step, otherwise drop `fixed_star_apparent` from this re-export line until Task 7.)

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-events fixstar::tests`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/data/fixstars-catalog.csv crates/pleiades-events/src/fixstar.rs crates/pleiades-events/src/lib.rs crates/pleiades-events/src/error.rs
git commit -m "feat(events): curated fixed-star catalog + reader"
```

## Task 7: `fixed_star_apparent` — apparent equatorial of a star

**Files:**
- Modify: `crates/pleiades-events/src/fixstar.rs`
- Test: inline

**Interfaces:**
- Consumes: `pleiades_apparent::{precess_ecliptic_j2000_to_date, nutation, annual_aberration}`.
- Produces:
  ```rust
  pub fn fixed_star_apparent(name: &str, at: Instant) -> Result<EquatorialCoordinates, EventError>;
  ```

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn aldebaran_apparent_is_near_catalog_and_of_date() {
    use pleiades_types::{Instant, JulianDay, TimeScale};
    let at = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb); // J2000
    let equ = fixed_star_apparent("Aldebaran", at).unwrap();
    // At J2000 the of-date apparent RA/Dec differ from the catalog J2000 values
    // only by nutation + aberration (< ~40"). Assert the small, bounded shift.
    assert!((equ.right_ascension.degrees() - 68.98).abs() < 0.02, "ra {}", equ.right_ascension.degrees());
    assert!((equ.declination.degrees() - 16.51).abs() < 0.02, "dec {}", equ.declination.degrees());
}

#[test]
fn unknown_star_apparent_fails_closed() {
    use pleiades_types::{Instant, JulianDay, TimeScale};
    let at = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    assert!(matches!(
        fixed_star_apparent("Nope", at).unwrap_err(),
        EventError::UnknownFixedStar { .. }
    ));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-events fixstar::tests::aldebaran_apparent_is_near_catalog_and_of_date`
Expected: FAIL — function not found.

- [ ] **Step 3: Write minimal implementation**

```rust
use pleiades_apparent::{annual_aberration, nutation, precess_ecliptic_j2000_to_date};
use pleiades_types::{Angle, EclipticCoordinates, EquatorialCoordinates, Instant, Latitude, Longitude};

const J2000_JD: f64 = 2_451_545.0;
const MEAN_OBLIQUITY_J2000_DEG: f64 = 23.439_291_1;

/// Apparent equatorial of date for a curated fixed star. Applies space motion to
/// epoch, precession (IAU-1976), nutation (IAU-1980), and annual aberration — no
/// light-time or light-deflection, matching `swe_fixstar` default flags.
pub fn fixed_star_apparent(name: &str, at: Instant) -> Result<EquatorialCoordinates, EventError> {
    let e = fixed_star_entry(name)?;
    let jd = at.julian_day.days();
    let years = (jd - J2000_JD) / 365.25;

    // 1. Space motion to epoch (RA/Dec, degrees). PM in mas/yr; RA pm is on the
    //    great circle (sefstars convention already divides by cosδ where needed).
    let dec0 = e.dec_j2000_deg.to_radians();
    let ra = e.ra_j2000_deg + (e.pm_ra_mas_yr / 3_600_000.0) * years / dec0.cos();
    let dec = e.dec_j2000_deg + (e.pm_dec_mas_yr / 3_600_000.0) * years;

    // 2. Equatorial J2000 → ecliptic J2000 for the precession/nutation helpers,
    //    which operate in ecliptic coordinates.
    let equ_j2000 = EquatorialCoordinates::new(
        Angle::from_degrees(ra.rem_euclid(360.0)),
        Latitude::from_degrees(dec),
        None,
    );
    let ecl_j2000 = equ_j2000.to_ecliptic(Angle::from_degrees(MEAN_OBLIQUITY_J2000_DEG));

    // 3. Precession J2000 → of date (ecliptic).
    let precessed = precess_ecliptic_j2000_to_date(
        ecl_j2000.longitude.degrees(),
        ecl_j2000.latitude.degrees(),
        jd,
    )
    .map_err(|e| EventError::Backend(format!("star precession failed: {e}")))?;

    // 4. Nutation in longitude (mean → true equinox) + annual aberration.
    let nut = nutation(jd).map_err(|e| EventError::Backend(format!("star nutation failed: {e}")))?;
    let aberr = annual_aberration(jd, precessed.longitude_deg, precessed.latitude_deg)
        .map_err(|e| EventError::Backend(format!("star aberration failed: {e}")))?;
    let lon_true = precessed.longitude_deg + nut.delta_psi_arcsec / 3600.0 + aberr.delta_longitude_deg;
    let lat_true = precessed.latitude_deg + aberr.delta_latitude_deg;

    // 5. True ecliptic of date → apparent equatorial of date.
    let eps = MEAN_OBLIQUITY_J2000_DEG; // replaced by true_obliquity_degrees(jd) — see note
    let ecl_true = EclipticCoordinates::new(
        Longitude::from_degrees(lon_true.rem_euclid(360.0)),
        Latitude::from_degrees(lat_true),
        None,
    );
    Ok(ecl_true.to_equatorial(Angle::from_degrees(
        pleiades_apparent::true_obliquity_degrees(jd)
            .map_err(|e| EventError::Backend(format!("obliquity failed: {e}")))?,
    )))
    // NOTE: `eps` local above is illustrative; the returned rotation uses
    // `true_obliquity_degrees(jd)`. Delete the unused `eps` binding.
}
```

(Confirm the exact `annual_aberration` return shape and `to_ecliptic`/`to_equatorial` availability on the coordinate types during implementation; adapt field names to match. `annual_aberration`'s signature is in `crates/pleiades-apparent/src/aberration.rs:29`.)

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-events fixstar::tests`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/fixstar.rs
git commit -m "feat(events): fixed-star apparent equatorial of date"
```

---

## Phase 4 — Rise/set/transit (`pleiades-events`)

## Task 8: rise/set/transit types + semidiameter table

**Files:**
- Create: `crates/pleiades-events/src/semidiameter.rs`
- Create: `crates/pleiades-events/src/rise_trans.rs` (types only this task)
- Modify: `crates/pleiades-events/src/lib.rs`
- Test: inline in both files

**Interfaces:**
- Produces:
  ```rust
  pub enum RiseSetEvent { Rise, Set, UpperTransit, LowerTransit }
  pub enum DiscMode { Center, UpperLimb, LowerLimb }
  pub enum RiseSetTarget { Body(CelestialBody), EclipticPoint(Longitude, Latitude), FixedStar(String) }
  pub struct RiseSetOptions { pub disc: DiscMode, pub refraction: bool, pub no_ecl_lat: bool,
      pub hindu: bool, pub fixed_disc_size: bool, pub horizon_altitude_deg: Option<f64> }
  impl Default for RiseSetOptions
  pub struct RiseSet { pub event: RiseSetEvent, pub instant: Instant, pub target: RiseSetTarget }
  // semidiameter.rs:
  pub(crate) fn semidiameter_deg(target: &RiseSetTarget, distance_au: f64, fixed: bool) -> f64;
  ```

- [ ] **Step 1: Write the failing test**

`semidiameter.rs` test:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::CelestialBody;

    #[test]
    fn sun_semidiameter_is_about_16_arcmin_at_1_au() {
        let sd = semidiameter_deg(&RiseSetTarget::Body(CelestialBody::Sun), 1.0, false);
        assert!((sd - 0.2666).abs() < 0.01, "sun SD {sd} deg"); // ~16'
    }

    #[test]
    fn star_semidiameter_is_zero() {
        let sd = semidiameter_deg(&RiseSetTarget::FixedStar("Sirius".into()), 1.0, false);
        assert_eq!(sd, 0.0);
    }
}
```

`rise_trans.rs` type test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options_are_upper_limb_refracted() {
        let o = RiseSetOptions::default();
        assert!(matches!(o.disc, DiscMode::UpperLimb));
        assert!(o.refraction);
        assert!(!o.hindu && !o.no_ecl_lat && !o.fixed_disc_size);
        assert!(o.horizon_altitude_deg.is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-events semidiameter::tests rise_trans::tests::default_options_are_upper_limb_refracted`
Expected: FAIL — items not defined.

- [ ] **Step 3: Write minimal implementation**

`semidiameter.rs`:

```rust
//! Apparent semidiameter of a target's disc, for rise/set limb conventions.
use crate::rise_trans::RiseSetTarget;
use pleiades_types::CelestialBody;

/// Mean physical radii in AU-consistent units (radius_km / AU_km), matching the
/// Swiss Ephemeris disc-radius table. Sun and Moon dominate; planets are small
/// but nonzero for parity.
fn radius_au(body: &CelestialBody) -> f64 {
    // radius_km / 149_597_870.7
    match body {
        CelestialBody::Sun => 696_000.0 / 149_597_870.7,
        CelestialBody::Moon => 1_737.4 / 149_597_870.7,
        CelestialBody::Mercury => 2_439.7 / 149_597_870.7,
        CelestialBody::Venus => 6_051.8 / 149_597_870.7,
        CelestialBody::Mars => 3_389.5 / 149_597_870.7,
        CelestialBody::Jupiter => 69_911.0 / 149_597_870.7,
        CelestialBody::Saturn => 58_232.0 / 149_597_870.7,
        CelestialBody::Uranus => 25_362.0 / 149_597_870.7,
        CelestialBody::Neptune => 24_622.0 / 149_597_870.7,
        CelestialBody::Pluto => 1_188.3 / 149_597_870.7,
        _ => 0.0,
    }
}

/// Apparent semidiameter (degrees). `distance_au` is the topocentric distance;
/// `fixed` freezes SD at a mean 1-AU-normalised value (`SE_BIT_FIXED_DISC_SIZE`).
pub(crate) fn semidiameter_deg(target: &RiseSetTarget, distance_au: f64, fixed: bool) -> f64 {
    let r = match target {
        RiseSetTarget::Body(b) => radius_au(b),
        _ => 0.0, // ecliptic points and stars have no disc
    };
    if r == 0.0 {
        return 0.0;
    }
    let d = if fixed { 1.0 } else { distance_au.max(1e-9) };
    (r / d).asin().to_degrees()
}
```

`rise_trans.rs` (types):

```rust
//! Rise, set, and meridian-transit finding (`swe_rise_trans`, full-flag).
use pleiades_types::{CelestialBody, Instant, Latitude, Longitude};

/// Which observer-local event to find.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiseSetEvent {
    /// Body crosses the horizon upward.
    Rise,
    /// Body crosses the horizon downward.
    Set,
    /// Upper (meridian) transit — hour angle 0.
    UpperTransit,
    /// Lower transit — hour angle ±12ʰ.
    LowerTransit,
}

/// Which point of the disc defines the event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscMode {
    /// Disc center.
    Center,
    /// Upper limb (SE default for rise/set).
    UpperLimb,
    /// Lower limb (`SE_BIT_DISC_BOTTOM`).
    LowerLimb,
}

/// The object whose event is sought.
#[derive(Clone, Debug)]
pub enum RiseSetTarget {
    /// A release-grade body.
    Body(CelestialBody),
    /// An arbitrary ecliptic point (longitude, latitude); pair with `no_ecl_lat`
    /// to force latitude 0 (rising of a zodiac degree).
    EclipticPoint(Longitude, Latitude),
    /// A curated fixed star by name.
    FixedStar(String),
}

/// `swe_rise_trans` flag bundle.
#[derive(Clone, Debug)]
pub struct RiseSetOptions {
    /// Disc convention.
    pub disc: DiscMode,
    /// Apply atmospheric refraction (`false` = `SE_BIT_NO_REFRACTION`).
    pub refraction: bool,
    /// Force ecliptic latitude 0 (`SE_BIT_GEOCTR_NO_ECL_LAT`).
    pub no_ecl_lat: bool,
    /// Hindu rising = `DISC_CENTER | NO_REFRACTION | GEOCTR_NO_ECL_LAT`.
    pub hindu: bool,
    /// Freeze semidiameter at mean distance (`SE_BIT_FIXED_DISC_SIZE`).
    pub fixed_disc_size: bool,
    /// Custom local horizon altitude, degrees (`swe_rise_trans_true_hor`).
    pub horizon_altitude_deg: Option<f64>,
}

impl Default for RiseSetOptions {
    fn default() -> Self {
        Self {
            disc: DiscMode::UpperLimb,
            refraction: true,
            no_ecl_lat: false,
            hindu: false,
            fixed_disc_size: false,
            horizon_altitude_deg: None,
        }
    }
}

impl RiseSetOptions {
    /// Resolves `hindu` into its component flags (SE composition).
    pub(crate) fn effective(&self) -> Self {
        if self.hindu {
            Self { disc: DiscMode::Center, refraction: false, no_ecl_lat: true, ..self.clone() }
        } else {
            self.clone()
        }
    }
}

/// A located rise/set/transit event (TDB).
#[derive(Clone, Debug)]
pub struct RiseSet {
    /// Which event this is.
    pub event: RiseSetEvent,
    /// Instant of the event (TDB).
    pub instant: Instant,
    /// The target the event is for.
    pub target: RiseSetTarget,
}
```

Add to `lib.rs`:

```rust
mod rise_trans;
mod semidiameter;
pub use rise_trans::{RiseSet, RiseSetEvent, RiseSetOptions, RiseSetTarget, DiscMode};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-events semidiameter::tests rise_trans::tests`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/semidiameter.rs crates/pleiades-events/src/rise_trans.rs crates/pleiades-events/src/lib.rs
git commit -m "feat(events): rise/set/transit types + semidiameter table"
```

## Task 9: altitude evaluator + standard-altitude `h0` assembly

**Files:**
- Modify: `crates/pleiades-events/src/rise_trans.rs`
- Test: inline

**Interfaces:**
- Consumes: `crate::ephemeris` (geocentric apparent path), `crate::horizontal`, `semidiameter::semidiameter_deg`, `pleiades_apparent::{sidereal_time, true_obliquity_degrees, topocentric_position, apparent_from_true, true_from_apparent, Atmosphere}`.
- Produces (private to the engine):
  ```rust
  impl<B: EphemerisBackend> EventEngine<B> {
      fn target_apparent_altitude(&self, target: &RiseSetTarget, observer: &ObserverLocation,
          opts: &RiseSetOptions, atmos: Atmosphere, jd: f64) -> Result<f64, EventError>;
      fn standard_altitude(&self, target: &RiseSetTarget, observer: &ObserverLocation,
          opts: &RiseSetOptions, atmos: Atmosphere, jd: f64) -> Result<f64, EventError>;
  }
  ```

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn standard_altitude_sun_upper_limb_is_about_negative_0p833() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(Latitude::from_degrees(40.0), Longitude::from_degrees(0.0), None);
    let opts = RiseSetOptions::default(); // upper limb + refraction
    let h0 = engine
        .standard_altitude(&RiseSetTarget::Body(pleiades_types::CelestialBody::Sun),
            &obs, &opts, pleiades_apparent::Atmosphere::default(),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb).julian_day.days())
        .unwrap();
    // −(34' refraction) − (16' semidiameter) ≈ −0.833°.
    assert!((h0 + 0.833).abs() < 0.05, "sun standard altitude {h0}");
}

#[test]
fn standard_altitude_no_refraction_center_is_zero() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(Latitude::from_degrees(40.0), Longitude::from_degrees(0.0), None);
    let opts = RiseSetOptions { disc: DiscMode::Center, refraction: false, ..RiseSetOptions::default() };
    let h0 = engine
        .standard_altitude(&RiseSetTarget::FixedStar("Sirius".into()), &obs, &opts,
            pleiades_apparent::Atmosphere::default(),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb).julian_day.days())
        .unwrap();
    assert!(h0.abs() < 1e-9, "no-refraction center h0 {h0}");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-events rise_trans::tests::standard_altitude_sun_upper_limb_is_about_negative_0p833`
Expected: FAIL — methods not defined.

- [ ] **Step 3: Write minimal implementation**

```rust
use crate::crossings::EventEngine;
use crate::error::EventError;
use crate::ephemeris::read_mean_ecliptic;
use crate::horizontal::HorizontalInput;
use crate::semidiameter::semidiameter_deg;
use pleiades_apparent::{apparent_from_true, sidereal_time, true_obliquity_degrees, Atmosphere};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{CelestialBody, EclipticCoordinates, Instant, JulianDay, ObserverLocation, TimeScale};

impl<B: EphemerisBackend> EventEngine<B> {
    /// Apparent (refracted, when `opts.refraction`) topocentric altitude of the
    /// target at `jd` (TDB), in degrees. This is the function rise/set root-finds.
    pub(crate) fn target_apparent_altitude(
        &self,
        target: &RiseSetTarget,
        observer: &ObserverLocation,
        opts: &RiseSetOptions,
        atmos: Atmosphere,
        jd: f64,
    ) -> Result<f64, EventError> {
        let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        // Resolve apparent equatorial RA/Dec of the target (topocentric parallax
        // applied for bodies with a distance).
        let (ra_deg, dec_deg) = self.target_equatorial(target, observer, opts, jd)?;
        let phi = observer.latitude.degrees().to_radians();
        let lst = sidereal_time(at, observer.longitude).local_apparent_deg;
        let ha = (lst - ra_deg).to_radians();
        let dec = dec_deg.to_radians();
        let sin_alt = phi.sin() * dec.sin() + phi.cos() * dec.cos() * ha.cos();
        let true_alt = sin_alt.asin().to_degrees();
        Ok(if opts.refraction { apparent_from_true(true_alt, atmos) } else { true_alt })
    }

    /// The standard altitude `h0` the event is defined at: horizon geometry minus
    /// disc/refraction/parallax/dip terms, plus any custom horizon.
    pub(crate) fn standard_altitude(
        &self,
        target: &RiseSetTarget,
        observer: &ObserverLocation,
        opts: &RiseSetOptions,
        _atmos: Atmosphere,
        jd: f64,
    ) -> Result<f64, EventError> {
        // Distance (AU) for semidiameter + parallax; 0 for points/stars.
        let distance_au = match target {
            RiseSetTarget::Body(b) => {
                read_mean_ecliptic(&self.backend, b.clone(), "body", jd)?.2
            }
            _ => 0.0,
        };
        let mut h0 = 0.0_f64;
        // Refraction at the horizon (Bennett at h=0 ≈ 34'); dropped if refraction off.
        if opts.refraction {
            h0 -= apparent_from_true(0.0, _atmos); // ≈ 0.567° raise → subtract as depression
        }
        // Disc term.
        let sd = semidiameter_deg(target, distance_au.max(1e-9), opts.fixed_disc_size);
        h0 += match opts.disc {
            DiscMode::UpperLimb => -sd,
            DiscMode::LowerLimb => sd,
            DiscMode::Center => 0.0,
        };
        // Horizon dip from observer elevation (metres): dip ≈ 1.76' * sqrt(h_m).
        if let Some(elev) = observer.elevation_m {
            if elev > 0.0 {
                h0 -= (1.76 / 60.0) * elev.sqrt();
            }
        }
        // Custom local horizon altitude.
        if let Some(hor) = opts.horizon_altitude_deg {
            h0 += hor;
        }
        Ok(h0)
    }
}
```

(`target_equatorial` is introduced in Task 10; for this task's tests, add a minimal private `target_equatorial` that handles `Body` via the existing geocentric-apparent path + `topocentric_position` and `FixedStar` via `fixed_star_apparent`. Land the full version in Task 10; a thin version here keeps the standard-altitude tests compiling. Alternatively reorder: implement `target_equatorial` first. Prefer implementing `target_equatorial` in this task since `standard_altitude` needs a distance read only, and `target_apparent_altitude` needs `target_equatorial` — see Task 10.)

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-events rise_trans::tests`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/rise_trans.rs
git commit -m "feat(events): rise/set standard-altitude assembly + apparent-altitude evaluator"
```

## Task 10: `target_equatorial` — RA/Dec for body / point / star

**Files:**
- Modify: `crates/pleiades-events/src/rise_trans.rs`
- Modify: `crates/pleiades-events/src/ephemeris.rs` (expose a geocentric apparent ecliptic (lon,lat,dist) helper if not already `pub(crate)`)
- Test: inline

**Interfaces:**
- Produces:
  ```rust
  impl<B: EphemerisBackend> EventEngine<B> {
      fn target_equatorial(&self, target: &RiseSetTarget, observer: &ObserverLocation,
          opts: &RiseSetOptions, jd: f64) -> Result<(f64 /*ra_deg*/, f64 /*dec_deg*/), EventError>;
  }
  ```
- Consumes: `geocentric_apparent_ecliptic` (a new `pub(crate)` in `ephemeris.rs` returning `(lon,lat,dist)` apparent-of-date), `topocentric_position`, `fixed_star_apparent`.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn target_equatorial_matches_horizontal_for_a_star() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(Latitude::from_degrees(40.0), Longitude::from_degrees(0.0), None);
    let jd = 2_451_545.0;
    let (ra, dec) = engine
        .target_equatorial(&RiseSetTarget::FixedStar("Aldebaran".into()), &obs, &RiseSetOptions::default(), jd)
        .unwrap();
    let equ = pleiades_events::fixed_star_apparent(
        "Aldebaran", Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)).unwrap();
    assert!((ra - equ.right_ascension.degrees()).abs() < 1e-9);
    assert!((dec - equ.declination.degrees()).abs() < 1e-9);
}

#[test]
fn ecliptic_point_no_ecl_lat_forces_latitude_zero() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{Latitude, Longitude, ObserverLocation};
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(Latitude::from_degrees(40.0), Longitude::from_degrees(0.0), None);
    let opts = RiseSetOptions { no_ecl_lat: true, ..RiseSetOptions::default() };
    // A point at longitude 90° latitude 30° with no_ecl_lat behaves like latitude 0.
    let (_, dec_forced) = engine.target_equatorial(
        &RiseSetTarget::EclipticPoint(Longitude::from_degrees(90.0), Latitude::from_degrees(30.0)),
        &obs, &opts, 2_451_545.0).unwrap();
    let (_, dec_zero) = engine.target_equatorial(
        &RiseSetTarget::EclipticPoint(Longitude::from_degrees(90.0), Latitude::from_degrees(0.0)),
        &obs, &opts, 2_451_545.0).unwrap();
    assert!((dec_forced - dec_zero).abs() < 1e-9, "no_ecl_lat should ignore supplied latitude");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-events rise_trans::tests::target_equatorial_matches_horizontal_for_a_star`
Expected: FAIL — method not defined.

- [ ] **Step 3: Write minimal implementation**

In `ephemeris.rs` add a `pub(crate)` apparent-ecliptic helper (mirrors `geocentric_apparent_longitude_deg` but returns lon+lat+dist so topocentric parallax can be applied):

```rust
/// Geocentric apparent-of-date ecliptic (lon_deg, lat_deg, dist_au) for a body.
pub(crate) fn geocentric_apparent_ecliptic<B: EphemerisBackend>(
    backend: &B, body: CelestialBody, body_label: &'static str, julian_day: f64,
) -> Result<(f64, f64, f64), EventError> {
    // Same pipeline as geocentric_apparent_longitude_deg but keep lat + dist.
    // (Refactor geocentric_apparent_longitude_deg to call this and take `.0`.)
    // ... implementation applies apparent_sun_position / apparent_position and
    //     returns ecliptic.longitude, ecliptic.latitude, distance_au.
    todo!("factor out of geocentric_apparent_longitude_deg — return (lon,lat,dist)")
}
```

> Replace the `todo!` by extracting the body of `geocentric_apparent_longitude_deg` so it returns `(lon,lat,dist)` and have the existing `_longitude_deg` call `.0`. The apparent pipeline already computes latitude and distance; surface them.

In `rise_trans.rs`:

```rust
use crate::ephemeris::geocentric_apparent_ecliptic;
use crate::fixstar::fixed_star_apparent;
use pleiades_apparent::topocentric_position;
use pleiades_types::{Angle, Latitude, Longitude};

impl<B: EphemerisBackend> EventEngine<B> {
    pub(crate) fn target_equatorial(
        &self,
        target: &RiseSetTarget,
        observer: &ObserverLocation,
        opts: &RiseSetOptions,
        jd: f64,
    ) -> Result<(f64, f64), EventError> {
        let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        let eps = true_obliquity_degrees(jd)
            .map_err(|e| EventError::Backend(format!("obliquity failed: {e}")))?;
        let lst = sidereal_time(at, observer.longitude).local_apparent_deg;
        match target {
            RiseSetTarget::FixedStar(name) => {
                let equ = fixed_star_apparent(name, at)?;
                Ok((equ.right_ascension.degrees(), equ.declination.degrees()))
            }
            RiseSetTarget::EclipticPoint(lon, lat) => {
                let lat = if opts.no_ecl_lat { Latitude::from_degrees(0.0) } else { *lat };
                let equ = EclipticCoordinates::new(*lon, lat, None)
                    .to_equatorial(Angle::from_degrees(eps));
                Ok((equ.right_ascension.degrees(), equ.declination.degrees()))
            }
            RiseSetTarget::Body(b) => {
                let (lon, lat, dist) =
                    geocentric_apparent_ecliptic(&self.backend, b.clone(), "body", jd)?;
                let lat = if opts.no_ecl_lat { 0.0 } else { lat };
                let ecl = EclipticCoordinates::new(
                    Longitude::from_degrees(lon), Latitude::from_degrees(lat), Some(dist));
                // Topocentric parallax (diurnal) then equatorial.
                let topo = topocentric_position(ecl, observer, lst, eps)
                    .map_err(|e| EventError::Backend(format!("topocentric failed: {e}")))?;
                let equ = topo.ecliptic.to_equatorial(Angle::from_degrees(eps));
                Ok((equ.right_ascension.degrees(), equ.declination.degrees()))
            }
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-events rise_trans::tests && cargo test -p pleiades-events` (crossings regressions still green)
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/rise_trans.rs crates/pleiades-events/src/ephemeris.rs
git commit -m "feat(events): topocentric RA/Dec for body/ecliptic-point/star targets"
```

## Task 11: rise/set root-finding (`next_rise_set` + `rise_sets_in_range`)

**Files:**
- Modify: `crates/pleiades-events/src/rise_trans.rs`
- Test: inline

**Interfaces:**
- Consumes: `crate::root::{first_crossing_after, crossings_in_range}`.
- Produces:
  ```rust
  impl<B: EphemerisBackend> EventEngine<B> {
      pub fn next_rise_set(&self, target: RiseSetTarget, event: RiseSetEvent,
          observer: ObserverLocation, atmos: Atmosphere, opts: RiseSetOptions, after: Instant)
          -> Result<Option<RiseSet>, EventError>;
      pub fn rise_sets_in_range(&self, target: RiseSetTarget, event: RiseSetEvent,
          observer: ObserverLocation, atmos: Atmosphere, opts: RiseSetOptions,
          start: Instant, end: Instant) -> Result<Vec<RiseSet>, EventError>;
  }
  ```

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn sun_rises_and_sets_within_a_day() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(Latitude::from_degrees(40.0), Longitude::from_degrees(0.0), None);
    let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let rise = engine.next_rise_set(
        RiseSetTarget::Body(CelestialBody::Sun), RiseSetEvent::Rise,
        obs, Atmosphere::default(), RiseSetOptions::default(), after).unwrap();
    let set = engine.next_rise_set(
        RiseSetTarget::Body(CelestialBody::Sun), RiseSetEvent::Set,
        obs, Atmosphere::default(), RiseSetOptions::default(), after).unwrap();
    let rise = rise.expect("a rise within the window");
    let set = set.expect("a set within the window");
    // At the rise instant the apparent altitude equals the standard altitude.
    let jd = rise.instant.julian_day.days();
    let alt = engine.target_apparent_altitude(
        &RiseSetTarget::Body(CelestialBody::Sun), &obs, &RiseSetOptions::default(),
        Atmosphere::default(), jd).unwrap();
    let h0 = engine.standard_altitude(
        &RiseSetTarget::Body(CelestialBody::Sun), &obs, &RiseSetOptions::default(),
        Atmosphere::default(), jd).unwrap();
    assert!((alt - h0).abs() < 1e-3, "altitude {alt} vs h0 {h0} at rise");
    assert!(set.instant.julian_day.days() != rise.instant.julian_day.days());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-events rise_trans::tests::sun_rises_and_sets_within_a_day`
Expected: FAIL — methods not defined.

- [ ] **Step 3: Write minimal implementation**

```rust
use crate::root::{crossings_in_range, first_crossing_after};

const RISE_SET_STEP_DAYS: f64 = 2.0 / 1440.0; // 2-minute scan: separates fast-Moon grazes

impl<B: EphemerisBackend> EventEngine<B> {
    fn horizon_residual(
        &self, target: &RiseSetTarget, observer: &ObserverLocation,
        opts: &RiseSetOptions, atmos: Atmosphere, jd: f64,
    ) -> Result<f64, EventError> {
        let alt = self.target_apparent_altitude(target, observer, opts, atmos, jd)?;
        let h0 = self.standard_altitude(target, observer, opts, atmos, jd)?;
        Ok(alt - h0)
    }

    /// Next rise/set/transit strictly after `after`, or `None`.
    pub fn next_rise_set(
        &self, target: RiseSetTarget, event: RiseSetEvent, observer: ObserverLocation,
        atmos: Atmosphere, opts: RiseSetOptions, after: Instant,
    ) -> Result<Option<RiseSet>, EventError> {
        observer.validate().map_err(|e| EventError::InvalidObserver { detail: e.to_string() })?;
        let opts = opts.effective();
        let after_jd = after.julian_day.days();
        self.check_window(after_jd)?;
        match event {
            RiseSetEvent::Rise | RiseSetEvent::Set => {
                let scan_start = after_jd.max(WINDOW_START_JD + RISE_SET_STEP_DAYS);
                let scan_end = WINDOW_END_JD - RISE_SET_STEP_DAYS;
                let want_upward = matches!(event, RiseSetEvent::Rise);
                // Wrap the residual so only the correct crossing DIRECTION counts:
                // rise = − → + (ascending); set = + → − (descending). Encode by
                // sign-flipping for set so first_crossing_after finds an ascending zero.
                let root = first_crossing_after(
                    |jd| {
                        let r = self.horizon_residual(&target, &observer, &opts, atmos, jd)?;
                        Ok(if want_upward { r } else { -r })
                    },
                    scan_start, scan_end, RISE_SET_STEP_DAYS,
                )?;
                Ok(root.filter(|&jd| jd > after_jd).map(|jd| RiseSet {
                    event, target: target.clone(),
                    instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
                }))
            }
            RiseSetEvent::UpperTransit | RiseSetEvent::LowerTransit => {
                self.next_transit(target, event, observer, opts, after) // Task 12
            }
        }
    }

    /// All rise/set/transit events of `event` kind in `[start, end]`, ascending.
    pub fn rise_sets_in_range(
        &self, target: RiseSetTarget, event: RiseSetEvent, observer: ObserverLocation,
        atmos: Atmosphere, opts: RiseSetOptions, start: Instant, end: Instant,
    ) -> Result<Vec<RiseSet>, EventError> {
        observer.validate().map_err(|e| EventError::InvalidObserver { detail: e.to_string() })?;
        let opts = opts.effective();
        let start_jd = start.julian_day.days();
        let end_jd = end.julian_day.days();
        self.check_window(start_jd)?;
        self.check_window(end_jd)?;
        match event {
            RiseSetEvent::Rise | RiseSetEvent::Set => {
                let want_upward = matches!(event, RiseSetEvent::Rise);
                let scan_start = start_jd.max(WINDOW_START_JD + RISE_SET_STEP_DAYS);
                let scan_end = end_jd.min(WINDOW_END_JD - RISE_SET_STEP_DAYS);
                let roots = crossings_in_range(
                    |jd| {
                        let r = self.horizon_residual(&target, &observer, &opts, atmos, jd)?;
                        Ok(if want_upward { r } else { -r })
                    },
                    scan_start, scan_end, RISE_SET_STEP_DAYS,
                )?;
                Ok(roots.into_iter().map(|jd| RiseSet {
                    event, target: target.clone(),
                    instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
                }).collect())
            }
            _ => self.transits_in_range(target, event, observer, opts, start, end), // Task 12
        }
    }
}
```

> `check_window`, `WINDOW_START_JD`, `WINDOW_END_JD` are already in scope via `crossings`/`error`. The residual passed to `crossings_in_range` uses altitude (bounded ±90°), so its wrap-seam `< 180.0` guard never mis-fires. `next_transit`/`transits_in_range` are Task 12 — add empty stubs returning `Ok(None)`/`Ok(vec![])` so this compiles, then Task 12 fills them.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-events rise_trans::tests::sun_rises_and_sets_within_a_day`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/rise_trans.rs
git commit -m "feat(events): rise/set root-finding on apparent-altitude residual"
```

## Task 12: meridian transit (upper + lower)

**Files:**
- Modify: `crates/pleiades-events/src/rise_trans.rs`
- Test: inline

**Interfaces:**
- Produces: private `next_transit`, `transits_in_range` (called by Task 11's public methods).

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn upper_transit_puts_body_on_the_meridian() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(Latitude::from_degrees(40.0), Longitude::from_degrees(0.0), None);
    let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let t = engine.next_rise_set(
        RiseSetTarget::Body(CelestialBody::Sun), RiseSetEvent::UpperTransit,
        obs, Atmosphere::default(), RiseSetOptions::default(), after).unwrap().expect("a transit");
    // At upper transit the local hour angle H = LST − RA ≈ 0.
    let jd = t.instant.julian_day.days();
    let (ra, _dec) = engine.target_equatorial(
        &RiseSetTarget::Body(CelestialBody::Sun), &obs, &RiseSetOptions::default(), jd).unwrap();
    let lst = pleiades_apparent::sidereal_time(
        Instant::new(JulianDay::from_days(jd), TimeScale::Tdb), Longitude::from_degrees(0.0)).local_apparent_deg;
    let ha = crate::root::wrap180(lst - ra);
    assert!(ha.abs() < 0.05, "hour angle at upper transit {ha} deg");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-events rise_trans::tests::upper_transit_puts_body_on_the_meridian`
Expected: FAIL — transit stubs return `None`.

- [ ] **Step 3: Write minimal implementation**

```rust
use crate::root::{first_crossing_after, crossings_in_range, wrap180};

const TRANSIT_STEP_DAYS: f64 = 5.0 / 1440.0; // 5-minute scan on hour angle

impl<B: EphemerisBackend> EventEngine<B> {
    // Hour-angle residual, wrapped to (−180,180]. Upper transit: zero of H.
    // Lower transit: zero of wrap180(H − 180).
    fn hour_angle_residual(
        &self, target: &RiseSetTarget, observer: &ObserverLocation, opts: &RiseSetOptions,
        lower: bool, jd: f64,
    ) -> Result<f64, EventError> {
        let (ra, _dec) = self.target_equatorial(target, observer, opts, jd)?;
        let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        let lst = sidereal_time(at, observer.longitude).local_apparent_deg;
        let ha = lst - ra;
        Ok(if lower { wrap180(ha - 180.0) } else { wrap180(ha) })
    }

    pub(crate) fn next_transit(
        &self, target: RiseSetTarget, event: RiseSetEvent, observer: ObserverLocation,
        opts: RiseSetOptions, after: Instant,
    ) -> Result<Option<RiseSet>, EventError> {
        let lower = matches!(event, RiseSetEvent::LowerTransit);
        let after_jd = after.julian_day.days();
        let scan_start = after_jd.max(WINDOW_START_JD + TRANSIT_STEP_DAYS);
        let scan_end = WINDOW_END_JD - TRANSIT_STEP_DAYS;
        let root = first_crossing_after(
            |jd| self.hour_angle_residual(&target, &observer, &opts, lower, jd),
            scan_start, scan_end, TRANSIT_STEP_DAYS,
        )?;
        Ok(root.filter(|&jd| jd > after_jd).map(|jd| RiseSet {
            event, target: target.clone(),
            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
        }))
    }

    pub(crate) fn transits_in_range(
        &self, target: RiseSetTarget, event: RiseSetEvent, observer: ObserverLocation,
        opts: RiseSetOptions, start: Instant, end: Instant,
    ) -> Result<Vec<RiseSet>, EventError> {
        let lower = matches!(event, RiseSetEvent::LowerTransit);
        let scan_start = start.julian_day.days().max(WINDOW_START_JD + TRANSIT_STEP_DAYS);
        let scan_end = end.julian_day.days().min(WINDOW_END_JD - TRANSIT_STEP_DAYS);
        let roots = crossings_in_range(
            |jd| self.hour_angle_residual(&target, &observer, &opts, lower, jd),
            scan_start, scan_end, TRANSIT_STEP_DAYS,
        )?;
        Ok(roots.into_iter().map(|jd| RiseSet {
            event, target: target.clone(),
            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
        }).collect())
    }
}
```

> The hour-angle residual jumps ±360° once per sidereal day; `wrap180` maps that to a smooth (−180,180] sawtooth whose only genuine zero-crossings are the transits. The `< 180.0` wrap-seam guard in `crossings_in_range` rejects the sawtooth wrap, exactly as it does for longitude crossings.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-events rise_trans::tests::upper_transit_puts_body_on_the_meridian`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/rise_trans.rs
git commit -m "feat(events): upper/lower meridian transit via hour-angle root-finding"
```

## Task 13: circumpolar / fail-closed matrix + doctests

**Files:**
- Modify: `crates/pleiades-events/src/rise_trans.rs`
- Modify: `crates/pleiades-events/src/lib.rs` (crate doctest showing a Sun rise)
- Test: inline

**Interfaces:** no new public surface; hardens edge behavior and adds `Atmosphere` validation.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn circumpolar_high_latitude_returns_none() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    // Near the pole a mid-declination body may never cross the horizon in a day.
    let obs = ObserverLocation::new(Latitude::from_degrees(89.9), Longitude::from_degrees(0.0), None);
    let start = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let end = Instant::new(JulianDay::from_days(2_451_545.5), TimeScale::Tdb);
    let out = engine.rise_sets_in_range(
        RiseSetTarget::Body(CelestialBody::Sun), RiseSetEvent::Rise,
        obs, Atmosphere::default(), RiseSetOptions::default(), start, end).unwrap();
    assert!(out.is_empty(), "circumpolar: no rise expected, got {}", out.len());
}

#[test]
fn out_of_window_and_bad_atmosphere_fail_closed() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(Latitude::from_degrees(40.0), Longitude::from_degrees(0.0), None);
    let err = engine.next_rise_set(
        RiseSetTarget::Body(CelestialBody::Sun), RiseSetEvent::Rise,
        obs, Atmosphere::default(), RiseSetOptions::default(),
        Instant::new(JulianDay::from_days(2_000_000.0), TimeScale::Tdb)).unwrap_err();
    assert!(matches!(err, EventError::OutOfWindow { .. }));

    let bad = Atmosphere { pressure_mbar: f64::NAN, temperature_c: 15.0 };
    let err = engine.next_rise_set(
        RiseSetTarget::Body(CelestialBody::Sun), RiseSetEvent::Rise,
        obs, bad, RiseSetOptions::default(),
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb)).unwrap_err();
    assert!(matches!(err, EventError::InvalidAtmosphere { .. }));
}

#[test]
fn unknown_star_target_fails_closed() {
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
    let obs = ObserverLocation::new(Latitude::from_degrees(40.0), Longitude::from_degrees(0.0), None);
    let err = engine.next_rise_set(
        RiseSetTarget::FixedStar("Nope".into()), RiseSetEvent::Rise,
        obs, Atmosphere::default(), RiseSetOptions::default(),
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb)).unwrap_err();
    assert!(matches!(err, EventError::UnknownFixedStar { .. }));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-events rise_trans::tests::out_of_window_and_bad_atmosphere_fail_closed`
Expected: FAIL — `InvalidAtmosphere` not defined; atmosphere not validated.

- [ ] **Step 3: Write minimal implementation**

Add `InvalidAtmosphere` to `error.rs`:

```rust
    /// Atmosphere parameters were non-finite.
    InvalidAtmosphere {
        /// Human-readable detail.
        detail: String,
    },
```
```rust
    EventError::InvalidAtmosphere { detail } => write!(f, "invalid atmosphere: {detail}"),
```

Add a guard used at the top of both public rise/set methods (and `horizontal`):

```rust
fn check_atmosphere(atmos: Atmosphere) -> Result<(), EventError> {
    if !atmos.pressure_mbar.is_finite() || !atmos.temperature_c.is_finite() {
        return Err(EventError::InvalidAtmosphere {
            detail: format!("pressure={} temp={}", atmos.pressure_mbar, atmos.temperature_c),
        });
    }
    Ok(())
}
```

Call `check_atmosphere(atmos)?;` after `observer.validate()` in `next_rise_set`, `rise_sets_in_range`, and `horizontal`/`horizontal_to_equatorial`. The circumpolar `None`/empty result already falls out of the root-finder (no sign change → no root); this task just proves it and adds a crate doctest:

Add to `lib.rs` crate docs a runnable rise example:

```rust
//! // When does the Sun next rise for a mid-latitude observer?
//! use pleiades_data::packaged_backend;
//! use pleiades_events::{EventEngine, RiseSetEvent, RiseSetOptions, RiseSetTarget};
//! use pleiades_apparent::Atmosphere;
//! use pleiades_types::{CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
//! let engine = EventEngine::new(packaged_backend());
//! let obs = ObserverLocation::new(Latitude::from_degrees(40.0), Longitude::from_degrees(-74.0), None);
//! let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
//! let rise = engine.next_rise_set(RiseSetTarget::Body(CelestialBody::Sun), RiseSetEvent::Rise,
//!     obs, Atmosphere::default(), RiseSetOptions::default(), after).unwrap();
//! assert!(rise.is_some());
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-events && cargo test -p pleiades-events --doc`
Expected: PASS (all rise/set + crossings + doctests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/rise_trans.rs crates/pleiades-events/src/error.rs crates/pleiades-events/src/lib.rs
git commit -m "feat(events): circumpolar None + atmosphere/star/window fail-closed + rise doctest"
```

---

## Phase 5 — Validation gate, CLI surfacing, bookkeeping

## Task 14: `tools/se-rise-trans-reference` harness

**Files:**
- Create: `tools/se-rise-trans-reference/{Cargo.toml,Cargo.lock,src/main.rs,LICENSE-NOTES.md}`

**Interfaces:** stand-alone binary; not a workspace member. Mirror `tools/se-crossings-reference` exactly (same deps, same `publish = false`, same `LICENSE-NOTES.md` posture).

- [ ] **Step 1: Copy the sibling tool's skeleton**

```bash
cp -r tools/se-crossings-reference tools/se-rise-trans-reference
```
Edit `tools/se-rise-trans-reference/Cargo.toml` `name = "se-rise-trans-reference"`. Confirm it is NOT added to the root `[workspace].members` (isolated lockfile).

- [ ] **Step 2: Write `src/main.rs` fixtures + SE calls**

Emit two CSVs. For `rise-trans.csv`, over a fixture set of `(object, event, lat, lon, elev, flags, atpress, attemp, start_jd_ut)` call `swe_rise_trans` / `swe_rise_trans_true_hor` (stars via `swe_fixstar`), converting the returned UT JD to TDB with the same ΔT the other harnesses use. For `azalt.csv`, over `(lon_ecl, lat_ecl, lat, lon, elev, atpress, attemp, jd_ut)` call `swe_azalt` (SE_ECL2HOR) and record azimuth + true + apparent altitude.

Fixture matrix (keep it representative, not exhaustive — the gate re-checks each row):
- objects: Sun, Moon, Mars, an ecliptic point (lon 90°, lat 0°), Aldebaran, Regulus.
- events: Rise, Set, UpperTransit, LowerTransit.
- flags spread: {upper-limb+refraction (default), disc-center+no-refraction, lower-limb, fixed-disc, hindu, custom horizon +5°}.
- observers: (40°N, −74°), (0°, 0°), (66.5°N, 25°) including a circumpolar-in-winter case, (-33°S, 151°).

Write the manifest with row counts, the SE version string (`2.10.03`), and an fnv1a64 checksum of each CSV's bytes (reuse `pleiades_apparent::fnv1a64` semantics — recompute the same way the crossings manifest does).

- [ ] **Step 3: Build the harness (libclang required)**

Run: `cd tools/se-rise-trans-reference && LIBCLANG_PATH=$(llvm-config --libdir) cargo build`
Expected: builds; produces the reference binary. (This step needs `libclang-dev`; it is NOT part of the workspace build or the gate.)

- [ ] **Step 4: Smoke-run it to stdout**

Run: `./tools/se-rise-trans-reference/target/debug/se-rise-trans-reference --dry-run | head`
Expected: CSV rows print; no panics.

- [ ] **Step 5: Commit**

```bash
git add tools/se-rise-trans-reference
git commit -m "test(events): isolated SE rise/set/transit + azalt reference harness"
```

## Task 15: generate + commit the corpus

**Files:**
- Create: `crates/pleiades-validate/data/rise-trans-corpus/{rise-trans.csv,azalt.csv,manifest.txt}`

- [ ] **Step 1: Generate the corpus**

Run: `./tools/se-rise-trans-reference/target/debug/se-rise-trans-reference --out crates/pleiades-validate/data/rise-trans-corpus`
Expected: writes `rise-trans.csv`, `azalt.csv`, `manifest.txt` with real SE values + checksums.

- [ ] **Step 2: Eyeball a few rows for sanity**

Read `rise-trans.csv`: Sun rise at 40°N should be a plausible morning JD; the 66.5°N winter row should be flagged as no-event (empty crossing column) so the gate can assert `None`.
Expected: values sane; the no-event row present and marked.

- [ ] **Step 3: Add a corpus-shape unit test**

In `rise_trans_validation.rs` (created next task) or a small standalone test now, assert the committed CSVs parse and the manifest checksums match the file bytes. (This step is folded into Task 16's gate; commit the data here.)

- [ ] **Step 4: Verify the data is tool-free readable**

Run: `git stash --include-untracked -- tools/ 2>/dev/null; cargo build -p pleiades-validate; git stash pop 2>/dev/null || true`
Expected: `pleiades-validate` builds without the tool present (reads committed CSVs via `include_str!`).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/data/rise-trans-corpus
git commit -m "test(validate): commit SE rise/set/transit + azalt corpus"
```

## Task 16: `validate-rise-trans` two-tier gate + thresholds

**Files:**
- Create: `crates/pleiades-validate/src/rise_trans_validation.rs`
- Create: `crates/pleiades-validate/src/rise_trans_thresholds.rs`
- Modify: `crates/pleiades-validate/src/lib.rs` (module + export)
- Modify: `crates/pleiades-validate/Cargo.toml` (`pleiades-events`, `pleiades-apparent` deps if not present)
- Test: inline gate tests

**Interfaces:**
- Produces: `pub fn validate_rise_trans() -> ValidationOutcome;` (match the existing gate return type used by `validate_crossings` — inspect `crossings_validation.rs` for the exact type/signature and mirror it).

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_passes_on_committed_corpus() {
        let outcome = validate_rise_trans();
        assert!(outcome.passed(), "rise-trans gate failed: {outcome:?}");
    }

    #[test]
    fn manifest_checksum_drift_fails_closed() {
        // Recompute the fnv1a64 of the embedded rise-trans.csv and compare to the
        // manifest value; a mismatch must fail the gate.
        assert_eq!(embedded_checksum("rise-trans.csv"), manifest_checksum("rise-trans.csv"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-validate rise_trans_validation`
Expected: FAIL — module not defined.

- [ ] **Step 3: Write the gate**

`rise_trans_thresholds.rs` — per-event ceilings (documented; loosened for grazing/high-latitude/fast-Moon):

```rust
//! Per-row parity ceilings for `validate-rise-trans`, mirroring the crossings/
//! angles thresholds modules. Time ceilings are seconds; angle ceilings arcsec.

/// Rise/set time-parity ceiling (seconds) for a well-conditioned row.
pub const RISE_SET_SECONDS_TIGHT: f64 = 2.0;
/// Loosened ceiling near grazing / high latitude / fast Moon (dalt/dt → 0).
pub const RISE_SET_SECONDS_GRAZING: f64 = 30.0;
/// Meridian-transit time ceiling (seconds).
pub const TRANSIT_SECONDS: f64 = 1.0;
/// azalt angle-parity ceiling (arcseconds).
pub const AZALT_ARCSEC: f64 = 5.0;
/// Self-consistency: azalt round-trip and refraction-inverse ceilings.
pub const SELF_CONSISTENCY_ARCSEC: f64 = 0.1;
```

`rise_trans_validation.rs` — two tiers:

```rust
//! Fail-closed two-tier `validate-rise-trans` gate: Tier 1 self-consistency
//! (tool-free goldens), Tier 2 SE parity over the committed corpus.

use crate::rise_trans_thresholds::*;
use pleiades_apparent::{apparent_from_true, true_from_apparent, Atmosphere};
use pleiades_events::{EventEngine, HorizontalInput, RiseSetEvent, RiseSetOptions, RiseSetTarget};
// ... plus the crate's ValidationOutcome type + packaged backend accessor used by
//     crossings_validation.rs (mirror those imports).

const RISE_TRANS_CSV: &str = include_str!("../data/rise-trans-corpus/rise-trans.csv");
const AZALT_CSV: &str = include_str!("../data/rise-trans-corpus/azalt.csv");
const MANIFEST: &str = include_str!("../data/rise-trans-corpus/manifest.txt");

pub fn validate_rise_trans() -> ValidationOutcome {
    let mut outcome = ValidationOutcome::new("validate-rise-trans");

    // Provenance / checksum drift (fail-closed).
    check_manifest_checksums(&mut outcome, MANIFEST, &[("rise-trans.csv", RISE_TRANS_CSV), ("azalt.csv", AZALT_CSV)]);

    // Tier 1 — self-consistency (no corpus values needed).
    tier1_self_consistency(&mut outcome);

    // Tier 2 — SE parity: for each rise-trans row, recompute with EventEngine and
    // compare the TDB instant to SE within the row's ceiling (grazing rows use the
    // loosened band); for each no-event row assert the engine returns empty/None.
    tier2_rise_trans_parity(&mut outcome, RISE_TRANS_CSV);
    // For each azalt row, compare azimuth + true + apparent altitude within AZALT_ARCSEC.
    tier2_azalt_parity(&mut outcome, AZALT_CSV);

    outcome
}
```

Implement the three helpers with real parsing + comparisons (mirror `crossings_validation.rs`'s row parsing, packaged-backend construction, and `ValidationOutcome` accumulation exactly). Tier 1 asserts: `horizontal_to_equatorial(horizontal(x)) ≈ x` (< `SELF_CONSISTENCY_ARCSEC`), `true_from_apparent(apparent_from_true(h)) ≈ h`, and a transit ⇒ hour-angle-zero check.

Wire into `lib.rs`:

```rust
pub mod rise_trans_thresholds;
pub mod rise_trans_validation;
pub use rise_trans_validation::validate_rise_trans;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-validate rise_trans_validation`
Expected: PASS (gate green on the committed corpus; checksum test green).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/rise_trans_validation.rs crates/pleiades-validate/src/rise_trans_thresholds.rs crates/pleiades-validate/src/lib.rs crates/pleiades-validate/Cargo.toml
git commit -m "feat(validate): two-tier validate-rise-trans SE-parity gate"
```

## Task 17: pin the below-horizon refraction branch + tighten grazing rows

**Files:**
- Modify: `crates/pleiades-apparent/src/refraction.rs`
- Modify: `crates/pleiades-validate/src/rise_trans_thresholds.rs` (only if measured residuals require)
- Test: inline refraction test + re-run the gate

**Interfaces:** no signature change; refines `true_from_apparent`/`apparent_from_true` below the horizon to match SE, proven by the gate.

- [ ] **Step 1: Add the below-horizon regression test**

```rust
#[test]
fn refraction_matches_se_below_horizon() {
    // SE clamps/blends refraction for apparent altitudes below ~ −5°. Lock the
    // engine's value against the SE reference figure captured in the corpus for a
    // deep-negative altitude row (value pinned once from se-rise-trans-reference).
    let r = apparent_from_true(-1.0, Atmosphere::default());
    assert!(r.is_finite() && r < 0.0, "below-horizon refraction {r}");
}
```

- [ ] **Step 2: Run the gate to see if any parity row is over its ceiling**

Run: `cargo test -p pleiades-validate rise_trans_validation::tests::gate_passes_on_committed_corpus -- --nocapture`
Expected: if a low-altitude / grazing row exceeds its ceiling, the failure names the row; use it to (a) correct the refraction branch or (b) justify and set the grazing ceiling from the measured residual.

- [ ] **Step 3: Refine the refraction branch**

Adjust `apparent_from_true`/`true_from_apparent` for `h` below the horizon to match SE (SE holds refraction roughly constant below a threshold rather than letting `tan` blow up). Only touch the low-altitude branch; the `h ≥ 0` behavior tested in Tasks 1–2 must not regress.

- [ ] **Step 4: Re-run refraction + gate**

Run: `cargo test -p pleiades-apparent refraction && cargo test -p pleiades-validate rise_trans_validation`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-apparent/src/refraction.rs crates/pleiades-validate/src/rise_trans_thresholds.rs
git commit -m "fix(apparent): pin below-horizon refraction branch to SE; set grazing ceilings"
```

## Task 18: wire the gate into release-smoke / release-gate

**Files:**
- Modify: `crates/pleiades-validate/src/render/cli.rs`
- Modify: `crates/pleiades-validate/src/release/notes.rs` (or wherever the release gate set is listed — mirror where `validate-crossings` is registered)
- Test: existing release-gate test + a new membership assertion

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn release_gate_set_includes_rise_trans() {
    let names = release_gate_names(); // the fn the crossings test uses
    assert!(names.contains(&"validate-rise-trans"), "gate set: {names:?}");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-validate release_gate_set_includes_rise_trans`
Expected: FAIL — not registered.

- [ ] **Step 3: Register the gate**

Add `validate-rise-trans` next to `validate-crossings` in the release-smoke / release-gate list and the render-layer dispatch (`render/cli.rs`), exactly mirroring the crossings wiring.

- [ ] **Step 4: Run the full release-gate suite**

Run: `cargo test -p pleiades-validate`
Expected: PASS — new gate runs inside the set; `validate-eclipses`/`validate-crossings`/`validate-angles`/`validate-houses` unaffected.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/render/cli.rs crates/pleiades-validate/src/release/notes.rs
git commit -m "feat(validate): wire validate-rise-trans into release-smoke/release-gate"
```

## Task 19: `rise-trans` + `azalt` CLI aliases

**Files:**
- Modify: `crates/pleiades-cli/src/cli.rs`
- Test: CLI integration test mirroring the `crossings` alias test

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn rise_trans_alias_renders() {
    // Mirror the `crossings` alias test: invoke the subcommand and assert it
    // routes through the validate render layer and prints a table without error.
    let out = run_cli(&["rise-trans", "--sun", "--observer", "40,-74"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-cli rise_trans_alias_renders`
Expected: FAIL — alias not defined.

- [ ] **Step 3: Add the aliases**

Route `rise-trans` and `azalt` through `pleiades-validate`'s render layer exactly as the `crossings` alias is routed. Reuse the existing observer/argument parsing patterns already present for chart/house commands.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-cli`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-cli/src/cli.rs
git commit -m "feat(cli): rise-trans + azalt aliases via validate render layer"
```

## Task 20: compatibility profile, API-stability, provenance flip, docs

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/mod.rs`
- Modify: `crates/pleiades-apparent/src/provenance.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs` (doc lines that say refraction is omitted)
- Modify: `README.md`, `PLAN.md`, `plan/status/01-current-execution-frontier.md`, `plan/status/02-next-slice-candidates.md`
- Test: the overclaim audit + compatibility profile tests

- [ ] **Step 1: Write / update the failing test**

The overclaim audit and compatibility-profile tests pin the version and the claim set. Update the expected compatibility profile to `0.7.7`, API-stability to `0.2.2`, and add the rise/set/transit + horizontal-coords + curated-fixed-star entries (claim tier tied to `validate-rise-trans`). Run them to see the expected/actual mismatch first.

Run: `cargo test -p pleiades-core compatibility`
Expected: FAIL — profile version/entries not yet bumped.

- [ ] **Step 2: Flip the refraction provenance**

In `provenance.rs` change `"... atmospheric refraction omitted"` to reflect that refraction is now available for horizontal/rise-set (keep the apparent-place default unchanged if the of-date longitude path still omits it — describe precisely which surface applies it). Update the `lib.rs` doc lines likewise. This is gated: it may only land because `validate-rise-trans` proves the refraction path.

- [ ] **Step 3: Bump profiles + add claim entries**

Set compatibility profile `0.7.6` → `0.7.7`; API-stability `0.2.1` → `0.2.2` (the `EventEngine` rename). Add the three capability entries with claim tiers tied to `validate-rise-trans`.

- [ ] **Step 4: Update docs + run the audits**

Add a README "current state" line (rise/set/transit + azalt + curated fixed stars) and refresh `PLAN.md` / `plan/status/*` (SP-2b done; SP-2c remains). Then:

Run: `cargo test -p pleiades-core && cargo test -p pleiades-validate` (overclaim audit + compatibility profile green)
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-core/src/compatibility/mod.rs crates/pleiades-apparent/src/provenance.rs crates/pleiades-apparent/src/lib.rs README.md PLAN.md plan/status
git commit -m "docs(events): declare SP-2b rise/set/transit + horizontal-coords capability; profile 0.7.7"
```

## Task 21: full-workspace regression + gate sweep

**Files:** none (verification task).

- [ ] **Step 1: Build the whole workspace**

Run: `cargo build --workspace`
Expected: clean build; no SE/FFI crate in `Cargo.lock` (workspace-audit clean).

- [ ] **Step 2: Run the whole test suite**

Run: `cargo test --workspace`
Expected: PASS — including `validate-eclipses`, `validate-crossings`, `validate-angles`, `validate-houses`, and the new `validate-rise-trans`.

- [ ] **Step 3: Run the workspace audit gate**

Run: `cargo test -p pleiades-validate workspace_audit` (or the command that enforces the pure-Rust lockfile)
Expected: PASS — `tools/se-rise-trans-reference` is not in the workspace lockfile.

- [ ] **Step 4: Run doctests**

Run: `cargo test --workspace --doc`
Expected: PASS — the new public items' doctests run.

- [ ] **Step 5: Final commit (if any doc/lint touch-ups)**

```bash
git add -A
git commit -m "test: full-workspace regression sweep for SP-2b" || echo "nothing to commit"
```

---

## Notes for the implementer

- **Confirm exact upstream signatures before coding each task.** `annual_aberration` (`aberration.rs:29`), `to_ecliptic`/`to_equatorial` on the coordinate types, the `ValidationOutcome` type and `release_gate_names` helper (from `crossings_validation.rs`), and the CLI alias plumbing are referenced by name here — read them and match field names exactly rather than assuming.
- **The `ephemeris.rs` refactor (Task 10) is load-bearing:** extracting `(lon,lat,dist)` from `geocentric_apparent_longitude_deg` must not change the longitude that `validate-crossings` already checks. Keep `geocentric_apparent_longitude_deg` returning the identical value (`geocentric_apparent_ecliptic(...).0`) and re-run `validate-crossings` after the refactor.
- **Azimuth convention** is the most likely parity mismatch. If `validate-rise-trans` azalt rows are off by a constant 180°, flip the azimuth origin (south vs north) — pin it against the corpus, don't guess.
- **Grazing/high-latitude ceilings** are set from measured residuals in Task 17, not invented up front — the two-tier gate's tight ceiling should hold for well-conditioned rows.
