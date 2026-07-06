# SP-2c · Local (Per-Observer) Eclipse Circumstances Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend `pleiades-eclipse`/`EclipseEngine` with per-observer solar and lunar eclipse circumstances — local contact times, magnitude/obscuration, az/alt, and horizon visibility — plus a fail-closed Swiss-Ephemeris parity gate.

**Architecture:** Approach A from the design: add a `local` module to the existing `pleiades-eclipse` crate and new methods to `EclipseEngine`, reusing the crate's own shadow geometry (`geometry.rs`) and Sun/Moon sampling (`ephemeris.rs`) plus `pleiades-apparent`'s topocentric-parallax, sidereal-time, obliquity, and refraction functions (already a dependency — no new crate dependency, no `pleiades-events` dependency; the equatorial→horizontal rotation is inlined). Solar contacts are found by topocentric two-circle tangency root-finding (genuinely observer-dependent); lunar contacts are global shadow-immersion instants with a per-observer horizon-visibility + az/alt layer. `next/previous_local_eclipse` reuse the validated global `next_eclipse`/`previous_eclipse` walk. Validation mirrors SP-2b: a standalone `tools/se-eclipse-local-reference` generator, a committed checksum-guarded corpus, and a two-tier `validate-eclipses-local` gate wired into `run_all_numeric_gates`.

**Tech Stack:** Rust (workspace crates), `pleiades-apparent` (topocentric/refraction/sidereal), `pleiades-eclipse` (geometry/ephemeris), `pleiades-validate` (gates/CLI), Swiss Ephemeris FFI (`swisseph`/`libswisseph-sys`) for the reference tool only.

## Global Constraints

- **Coverage window:** hard 1900-01-01 (JD 2 415 020.5 TDB) through 2100-01-01 (JD 2 488 069.5 TDB). Out-of-window requests fail closed via the existing `EclipseError::OutOfWindow`. Constants: `WINDOW_START_JD`, `WINDOW_END_JD` (from `pleiades_eclipse::error`).
- **Time base:** all instants are `TimeScale::Tdb`, matching `Eclipse::greatest_eclipse`. Same documented ΔT caveat as SP-2b (sidereal time consumes the JD as UT1) — do **not** add a ΔT/UT1 policy change; the gate compares against the SE `_ut` column, exactly like `validate-rise-trans`.
- **Azimuth convention:** measured from **south, increasing westward**, `[0,360)` degrees — identical to SP-2b `Horizontal` and `swe_azalt`. Do not use a from-north convention.
- **Fail-closed always:** clamp every `asin`/`acos` domain to `[-1,1]` before the call (never emit NaN); validate `ObserverLocation` and `Atmosphere` at every public entry point (reuse the existing `ObserverLocation::validate()` and mirror `pleiades_events::rise_trans::check_atmosphere`'s finiteness check).
- **No new crate dependency** on `pleiades-eclipse`. Reuse `pleiades-apparent` (already a dependency). Do not add `pleiades-events`.
- **Versioning:** new additive public surface → bump compatibility profile `pleiades-compatibility-profile/0.7.7` → `0.7.8` (`crates/pleiades-core/src/compatibility/mod.rs`). API-stability profile stays `pleiades-api-stability/0.2.2` (no rename, purely additive).
- **Checksum scheme:** committed corpus CSVs are guarded by `fnv1a64` (offset basis `0xcbf29ce484222325`, prime `0x100000001b3`), identical to `pleiades_apparent::fnv1a64` and the other `se-*` manifests.
- **Style:** match surrounding code — `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` are active in `pleiades-eclipse`; every new public item needs a doc comment. Run `cargo fmt` and `cargo clippy` before each commit.

**Reference signatures this plan consumes (verified in the codebase):**

```rust
// pleiades_eclipse::ephemeris (pub(crate))
struct SunMoonSample { sun_longitude_deg, sun_latitude_deg, sun_distance_au,
                       moon_longitude_deg, moon_latitude_deg, moon_distance_au: f64 }
fn sample_sun_moon<B: EphemerisBackend>(backend: &B, jd: f64) -> Result<SunMoonSample, EclipseError>;

// pleiades_apparent (pub)
struct Atmosphere { pressure_mbar: f64, temperature_c: f64 }   // Default = 1013.25 mbar, 15 C
fn topocentric_position(apparent: EclipticCoordinates, observer: &ObserverLocation,
                        local_sidereal_time_deg: f64, obliquity_deg: f64)
    -> Result<TopocentricPosition, ApparentPlaceError>;         // .ecliptic: EclipticCoordinates
fn sidereal_time(at: Instant, observer_longitude: Longitude) -> SiderealTime; // .local_apparent_deg
fn true_obliquity_degrees(jd: f64) -> Result<f64, _>;
fn apparent_from_true(true_alt_deg: f64, atmos: Atmosphere) -> f64;
fn fnv1a64(text: &str) -> u64;

// pleiades_types
struct EclipticCoordinates; // ::new(Longitude, Latitude, Option<f64> distance_au); .to_equatorial(Angle)
struct EquatorialCoordinates; // .right_ascension: Angle, .declination: Latitude
struct ObserverLocation { latitude: Latitude, longitude: Longitude, elevation_m: Option<f64> } // .validate()
struct Instant { julian_day: JulianDay, .. } // ::new(JulianDay, TimeScale)

// pleiades_eclipse::engine (pub)
EclipseEngine::next_eclipse(after: Instant, filter: EclipseFilter) -> Result<Option<Eclipse>, EclipseError>;
EclipseEngine::previous_eclipse(before: Instant, filter: EclipseFilter) -> Result<Option<Eclipse>, EclipseError>;

// pleiades_eclipse::types (pub)
struct Eclipse { kind: EclipseKind, eclipse_type: EclipseType, greatest_eclipse: Instant,
                 magnitude, gamma: f64, saros_series: u32, eclipsed_longitude: Longitude,
                 near_node: Node, greatest_eclipse_location: Option<GeoLocation> }
enum EclipseKind { Solar, Lunar }
enum SolarEclipseType { Total, Annular, Hybrid, Partial }
enum LunarEclipseType { Penumbral, Partial, Total }
enum EclipseFilter { All, SolarOnly, LunarOnly }
```

Physical constants (copy from `pleiades_eclipse::geometry::constants`, do not redefine with different values): `R_SUN_KM = 696_000.0`, `R_MOON_KM = 1_737.4`, `R_EARTH_KM = 6_378.137`, `AU_KM = 149_597_870.7`, `SHADOW_INFLATION = 1.01`.

---

## File Structure

- `crates/pleiades-eclipse/src/local.rs` — **new**: local circumstance value types (`LocalContact`, `LocalSolarCircumstances`, `LocalLunarCircumstances`, `LocalCircumstances`) and the internal local-circumstance computation (topocentric sampling, two-circle solar contacts, lunar shadow contacts, az/alt + visibility). Split into submodules only if it grows past ~600 lines; start as one file.
- `crates/pleiades-eclipse/src/engine.rs` — **modify**: add `local_circumstances`, `next_local_eclipse`, `previous_local_eclipse` methods (thin wrappers delegating to `local.rs`).
- `crates/pleiades-eclipse/src/lib.rs` — **modify**: `mod local;`, re-export the new public types, flip the "No per-observer local circumstances" scope line.
- `tools/se-eclipse-local-reference/` — **new** standalone Cargo project (excluded from workspace) mirroring `tools/se-rise-trans-reference`: emits `sol-local.csv`, `lun-local.csv`, `manifest.txt`.
- `crates/pleiades-validate/data/eclipses-local-corpus/` — **new**: committed `sol-local.csv`, `lun-local.csv`, `manifest.txt`.
- `crates/pleiades-validate/src/eclipse_local_thresholds.rs` — **new**: per-category parity ceilings (measured), mirroring `rise_trans_thresholds.rs`.
- `crates/pleiades-validate/src/eclipse_local_validation.rs` — **new**: the two-tier `validate_eclipse_local_corpus()` gate + `render_eclipse_local_listing()` for the CLI alias.
- `crates/pleiades-validate/src/lib.rs` — **modify**: `mod`/`pub use` the two new modules.
- `crates/pleiades-validate/src/render/cli.rs` — **modify**: `eclipse-local`, `validate-eclipses-local`/`eclipses-local-gate` dispatch; add to `run_all_numeric_gates`; add help-banner lines.
- `crates/pleiades-core/src/compatibility/mod.rs` — **modify**: profile `0.7.7` → `0.7.8`.
- `crates/pleiades-eclipse/README.md`, `README.md`, `PLAN.md`, `plan/status/01-*.md`, `plan/status/02-*.md`, `docs/superpowers/specs/.../*` claim surfaces — **modify**: mark SP-2c done, leave SP-3 remaining.

---

## Task 1: Local circumstance value types

**Files:**
- Create: `crates/pleiades-eclipse/src/local.rs`
- Modify: `crates/pleiades-eclipse/src/lib.rs`

**Interfaces:**
- Consumes: `pleiades_types::Instant`; `crate::types::{SolarEclipseType, LunarEclipseType}`.
- Produces: `pub struct LocalContact`, `pub struct LocalSolarCircumstances`, `pub struct LocalLunarCircumstances`, `pub enum LocalCircumstances` — the return-value vocabulary every later task uses.

- [ ] **Step 1: Write the failing test**

Add to a new `crates/pleiades-eclipse/src/local.rs`:

```rust
//! Per-observer (local) eclipse circumstances: contact times, magnitude,
//! obscuration, horizontal position, and horizon visibility for a specific
//! observer, extending the crate's global/geocentric eclipse data.

use crate::types::{LunarEclipseType, SolarEclipseType};
use pleiades_types::Instant;

/// One observer-local contact event: its instant plus the eclipsed body's
/// horizontal position and visibility there. A contact that occurs below the
/// horizon is still timed (`instant` present) but flagged `visible == false`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalContact {
    /// Instant of the contact (TDB).
    pub instant: Instant,
    /// Apparent (refracted) altitude of the eclipsed body, degrees.
    pub altitude_degrees: f64,
    /// Azimuth of the eclipsed body, measured from south increasing westward,
    /// `[0,360)` degrees (matches `swe_azalt`).
    pub azimuth_degrees: f64,
    /// Whether the body is above the horizon at this instant.
    pub visible: bool,
}

/// Local circumstances of a solar eclipse for one observer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalSolarCircumstances {
    /// What THIS observer sees (may differ from the global classification).
    pub local_type: SolarEclipseType,
    /// Instant of local greatest eclipse.
    pub maximum: LocalContact,
    /// Covered fraction of the Sun's diameter at local maximum.
    pub magnitude: f64,
    /// Covered fraction of the Sun's area at local maximum.
    pub obscuration: f64,
    /// First contact (C1): partial phase begins.
    pub first_contact: LocalContact,
    /// Second contact (C2): total/annular phase begins (central path only).
    pub second_contact: Option<LocalContact>,
    /// Third contact (C3): total/annular phase ends (central path only).
    pub third_contact: Option<LocalContact>,
    /// Fourth contact (C4): partial phase ends.
    pub fourth_contact: LocalContact,
    /// Whether the Sun is above the horizon during any part of the eclipse.
    pub any_phase_visible: bool,
}

/// Local circumstances of a lunar eclipse for one observer. Contact instants
/// are global (shared by all observers); the local content is horizon
/// visibility and the Moon's az/alt at each contact.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalLunarCircumstances {
    /// Umbral/penumbral classification (identical to the global type).
    pub eclipse_type: LunarEclipseType,
    /// Local greatest eclipse.
    pub maximum: LocalContact,
    /// Umbral magnitude at greatest eclipse.
    pub umbral_magnitude: f64,
    /// Penumbral magnitude at greatest eclipse.
    pub penumbral_magnitude: f64,
    /// P1: penumbral phase begins.
    pub penumbral_begin: LocalContact,
    /// U1: partial (umbral) phase begins; `None` for penumbral-only eclipses.
    pub partial_begin: Option<LocalContact>,
    /// U2: total phase begins; `None` unless total.
    pub total_begin: Option<LocalContact>,
    /// U3: total phase ends; `None` unless total.
    pub total_end: Option<LocalContact>,
    /// U4: partial (umbral) phase ends; `None` for penumbral-only eclipses.
    pub partial_end: Option<LocalContact>,
    /// P4: penumbral phase ends.
    pub penumbral_end: LocalContact,
    /// Whether the Moon is above the horizon during any part of the eclipse.
    pub any_phase_visible: bool,
}

/// A tagged local result: either solar or lunar circumstances.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LocalCircumstances {
    /// Solar eclipse local circumstances.
    Solar(LocalSolarCircumstances),
    /// Lunar eclipse local circumstances.
    Lunar(LocalLunarCircumstances),
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{JulianDay, TimeScale};

    fn contact(jd: f64) -> LocalContact {
        LocalContact {
            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
            altitude_degrees: 30.0,
            azimuth_degrees: 180.0,
            visible: true,
        }
    }

    #[test]
    fn local_circumstances_tags_solar_and_lunar() {
        let solar = LocalCircumstances::Solar(LocalSolarCircumstances {
            local_type: SolarEclipseType::Partial,
            maximum: contact(2_451_545.0),
            magnitude: 0.5,
            obscuration: 0.4,
            first_contact: contact(2_451_544.9),
            second_contact: None,
            third_contact: None,
            fourth_contact: contact(2_451_545.1),
            any_phase_visible: true,
        });
        assert!(matches!(solar, LocalCircumstances::Solar(_)));
    }
}
```

- [ ] **Step 2: Wire the module and run the failing test**

In `crates/pleiades-eclipse/src/lib.rs`, add `mod local;` (alphabetical, after `mod geometry;`) and the re-export line in the `pub use` block:

```rust
pub use local::{
    LocalCircumstances, LocalContact, LocalLunarCircumstances, LocalSolarCircumstances,
};
```

Run: `cargo test -p pleiades-eclipse local::tests::local_circumstances_tags_solar_and_lunar`
Expected: FAIL to compile first if `mod local;` is missing, then PASS once wired — this step is really "make it compile and pass". If it fails to find the module, confirm `mod local;` was added.

- [ ] **Step 3: Run test to verify it passes**

Run: `cargo test -p pleiades-eclipse local::`
Expected: PASS (1 test).

- [ ] **Step 4: Update the crate scope doc line**

In `crates/pleiades-eclipse/src/lib.rs`, change the scope block line:
```rust
//! - **Coverage:** global / geocentric only. No per-observer local circumstances.
```
to:
```rust
//! - **Coverage:** global / geocentric, plus per-observer local circumstances
//!   (contact times, magnitude/obscuration, az/alt, visibility) via
//!   [`EclipseEngine::local_circumstances`] and `next/previous_local_eclipse`.
```
(The linked methods are added in Task 8; the doc link resolves then. If `cargo doc` intra-doc-link checking is on and this step is verified before Task 8, temporarily reference them in prose without the `[` `]` link and restore the link in Task 8.)

- [ ] **Step 5: Commit**

```bash
cargo fmt -p pleiades-eclipse
git add crates/pleiades-eclipse/src/local.rs crates/pleiades-eclipse/src/lib.rs
git commit -m "feat(eclipse): SP-2c local circumstance value types"
```

---

## Task 2: Topocentric Sun/Moon sample helper

**Files:**
- Modify: `crates/pleiades-eclipse/src/local.rs`

**Interfaces:**
- Consumes: `pleiades_eclipse::ephemeris::{SunMoonSample, sample_sun_moon}` (crate-internal); `pleiades_apparent::{topocentric_position, sidereal_time, true_obliquity_degrees}`; `pleiades_types::{EclipticCoordinates, Longitude, Latitude, ObserverLocation, Instant, JulianDay, TimeScale}`.
- Produces: `pub(crate) struct TopoSunMoon { sun_lon_deg, sun_lat_deg, sun_dist_au, moon_lon_deg, moon_lat_deg, moon_dist_au: f64 }` and `pub(crate) fn topo_sun_moon<B: EphemerisBackend>(backend: &B, observer: &ObserverLocation, jd: f64) -> Result<TopoSunMoon, EclipseError>` — the observer-relative Sun/Moon geometry every solar-contact computation consumes.

This applies diurnal parallax to the Moon (dominant local effect; up to ~1°) and Sun (small) using the existing `topocentric_position`, giving the topocentric ecliptic-of-date longitudes/latitudes/distances the two-circle geometry needs.

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-eclipse/src/local.rs`:

```rust
use crate::ephemeris::{sample_sun_moon, SunMoonSample};
use crate::error::EclipseError;
use pleiades_apparent::{sidereal_time, topocentric_position, true_obliquity_degrees};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{
    EclipticCoordinates, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
};

/// Observer-relative (topocentric) Sun and Moon apparent ecliptic-of-date
/// geometry at one instant: the input to the two-circle solar contact geometry.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TopoSunMoon {
    pub sun_lon_deg: f64,
    pub sun_lat_deg: f64,
    pub sun_dist_au: f64,
    pub moon_lon_deg: f64,
    pub moon_lat_deg: f64,
    pub moon_dist_au: f64,
}

/// Applies diurnal parallax to the geocentric Sun/Moon sample for `observer`.
pub(crate) fn topo_sun_moon<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    jd: f64,
) -> Result<TopoSunMoon, EclipseError> {
    let sample: SunMoonSample = sample_sun_moon(backend, jd)?;
    let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
    let eps = true_obliquity_degrees(jd)
        .map_err(|e| EclipseError::Backend(format!("obliquity failed: {e}")))?;
    let lst = sidereal_time(at, observer.longitude).local_apparent_deg;

    let to_topo = |lon: f64, lat: f64, dist: f64| -> Result<(f64, f64, f64), EclipseError> {
        let ecl = EclipticCoordinates::new(
            Longitude::from_degrees(lon),
            Latitude::from_degrees(lat),
            Some(dist),
        );
        let topo = topocentric_position(ecl, observer, lst, eps)
            .map_err(|e| EclipseError::Backend(format!("topocentric failed: {e}")))?;
        Ok((
            topo.ecliptic.longitude.degrees(),
            topo.ecliptic.latitude.degrees(),
            topo.ecliptic.distance_au.unwrap_or(dist),
        ))
    };

    let (sun_lon_deg, sun_lat_deg, sun_dist_au) = to_topo(
        sample.sun_longitude_deg,
        sample.sun_latitude_deg,
        sample.sun_distance_au,
    )?;
    let (moon_lon_deg, moon_lat_deg, moon_dist_au) = to_topo(
        sample.moon_longitude_deg,
        sample.moon_latitude_deg,
        sample.moon_distance_au,
    )?;
    Ok(TopoSunMoon {
        sun_lon_deg,
        sun_lat_deg,
        sun_dist_au,
        moon_lon_deg,
        moon_lat_deg,
        moon_dist_au,
    })
}

#[cfg(test)]
mod topo_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn moon_parallax_shifts_topocentric_longitude() {
        // The analytic test backend places a new moon; an equatorial observer
        // sees the Moon shifted from its geocentric longitude by parallax.
        let backend = LinearSunMoon::new_moon_at(2_451_550.0);
        let observer = ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        );
        let geo = sample_sun_moon(&backend, 2_451_550.0).unwrap();
        let topo = topo_sun_moon(&backend, &observer, 2_451_550.0).unwrap();
        let shift = (topo.moon_lon_deg - geo.moon_longitude_deg).abs();
        assert!(shift > 0.0, "expected a nonzero parallax shift, got {shift}");
        assert!(topo.moon_dist_au.is_finite());
    }
}
```

Note: `EclipseError::Backend(String)` is used here. Verify it exists in `crates/pleiades-eclipse/src/error.rs`; if the variant is named differently (e.g. `EclipseError::Backend { detail: String }`), match the actual signature — read `error.rs` first and adapt the two `.map_err` closures. If no string-carrying backend variant exists, add one:
```rust
/// A backend or correction sub-call failed.
Backend(String),
```
and its `Display` arm, in the same style as the other variants.

- [ ] **Step 2: Run test to verify it fails, then compiles**

Run: `cargo test -p pleiades-eclipse local::topo_tests::moon_parallax_shifts_topocentric_longitude`
Expected: initially FAIL to compile if `EclipseError::Backend` is absent; after adding it, PASS.

- [ ] **Step 3: Run the full crate test suite**

Run: `cargo test -p pleiades-eclipse`
Expected: PASS (existing tests unaffected).

- [ ] **Step 4: Commit**

```bash
cargo fmt -p pleiades-eclipse && cargo clippy -p pleiades-eclipse --all-targets
git add crates/pleiades-eclipse/src/local.rs crates/pleiades-eclipse/src/error.rs
git commit -m "feat(eclipse): SP-2c topocentric Sun/Moon sample helper"
```

---

## Task 3: Solar two-circle instantaneous geometry

**Files:**
- Modify: `crates/pleiades-eclipse/src/local.rs`

**Interfaces:**
- Consumes: `TopoSunMoon` (Task 2); constants copied from `geometry::constants`.
- Produces: `pub(crate) struct SolarGeom { sep_deg, s_sun_deg, s_moon_deg: f64 }` and `pub(crate) fn solar_geom(t: &TopoSunMoon) -> SolarGeom`; plus `pub(crate) fn covered_diameter_fraction(g: &SolarGeom) -> f64` and `pub(crate) fn obscuration_fraction(g: &SolarGeom) -> f64`. These are the pure per-instant quantities the contact root-finder and magnitude assembly consume.

`sep_deg` is the topocentric center-to-center Sun–Moon separation; `s_sun_deg`/`s_moon_deg` are their topocentric angular semidiameters. C1/C4 are where `sep == s_sun + s_moon`; C2/C3 where `sep == |s_moon − s_sun|`. Magnitude (diameter fraction) `= (s_sun + s_moon − sep)/(2·s_sun)` clamped to `≥ 0`. Obscuration is the two-circle overlap **area** fraction of the Sun's disk.

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-eclipse/src/local.rs`:

```rust
/// Physical radii and unit conversion (mirrors `geometry::constants`; kept in
/// sync deliberately — do not diverge these values).
mod solar_consts {
    pub const R_SUN_KM: f64 = 696_000.0;
    pub const R_MOON_KM: f64 = 1_737.4;
    pub const AU_KM: f64 = 149_597_870.7;
}

/// Instantaneous topocentric two-circle geometry of a solar eclipse.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SolarGeom {
    /// Center-to-center Sun–Moon separation, degrees.
    pub sep_deg: f64,
    /// Sun's topocentric angular semidiameter, degrees.
    pub s_sun_deg: f64,
    /// Moon's topocentric angular semidiameter, degrees.
    pub s_moon_deg: f64,
}

/// Great-circle separation (degrees) between two ecliptic points.
fn angular_separation_deg(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let (l1, b1) = (lon1.to_radians(), lat1.to_radians());
    let (l2, b2) = (lon2.to_radians(), lat2.to_radians());
    let cos_sep =
        (b1.sin() * b2.sin() + b1.cos() * b2.cos() * (l1 - l2).cos()).clamp(-1.0, 1.0);
    cos_sep.acos().to_degrees()
}

/// Topocentric two-circle geometry at one instant.
pub(crate) fn solar_geom(t: &TopoSunMoon) -> SolarGeom {
    use solar_consts::*;
    let sep_deg = angular_separation_deg(
        t.sun_lon_deg,
        t.sun_lat_deg,
        t.moon_lon_deg,
        t.moon_lat_deg,
    );
    let s_sun_deg = (R_SUN_KM / (t.sun_dist_au * AU_KM)).asin().to_degrees();
    let s_moon_deg = (R_MOON_KM / (t.moon_dist_au * AU_KM)).asin().to_degrees();
    SolarGeom {
        sep_deg,
        s_sun_deg,
        s_moon_deg,
    }
}

/// Covered fraction of the Sun's diameter (the eclipse "magnitude"), clamped ≥ 0.
pub(crate) fn covered_diameter_fraction(g: &SolarGeom) -> f64 {
    ((g.s_sun_deg + g.s_moon_deg - g.sep_deg) / (2.0 * g.s_sun_deg)).max(0.0)
}

/// Covered fraction of the Sun's disk AREA (obscuration), clamped to [0,1].
/// Standard two-circle lens area with radii `r_sun`, `r_moon` and center
/// distance `d` (all in the same angular units).
pub(crate) fn obscuration_fraction(g: &SolarGeom) -> f64 {
    let (r_s, r_m, d) = (g.s_sun_deg, g.s_moon_deg, g.sep_deg);
    if d >= r_s + r_m {
        return 0.0; // disjoint
    }
    if d <= (r_m - r_s).max(0.0) {
        return 1.0; // Sun fully covered (Moon disk envelops Sun disk)
    }
    if d <= (r_s - r_m).max(0.0) {
        // Moon fully inside the Sun (annular): area ratio (r_m/r_s)^2.
        return ((r_m / r_s).powi(2)).clamp(0.0, 1.0);
    }
    let r_s2 = r_s * r_s;
    let r_m2 = r_m * r_m;
    let a_s = ((d * d + r_s2 - r_m2) / (2.0 * d * r_s)).clamp(-1.0, 1.0).acos();
    let a_m = ((d * d + r_m2 - r_s2) / (2.0 * d * r_m)).clamp(-1.0, 1.0).acos();
    let tri = (0.5
        * ((r_s + r_m + d) * (-r_s + r_m + d) * (r_s - r_m + d) * (r_s + r_m - d))
            .max(0.0)
            .sqrt());
    let lens = r_s2 * a_s + r_m2 * a_m - 2.0 * tri; // note: tri counted once per standard formula
    let overlap = r_s2 * a_s + r_m2 * a_m - (r_s + r_m + d).max(0.0) * 0.0 - 2.0 * (0.5 * ((r_s + r_m + d) * (-r_s + r_m + d) * (r_s - r_m + d) * (r_s + r_m - d)).max(0.0).sqrt()) * 0.0;
    let _ = (lens, overlap); // guard against accidental unused-var during edit
    let lens_area = r_s2 * a_s + r_m2 * a_m
        - 0.5 * ((r_s + r_m + d) * (-r_s + r_m + d) * (r_s - r_m + d) * (r_s + r_m - d)).max(0.0).sqrt();
    (lens_area / (core::f64::consts::PI * r_s2)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod solar_geom_tests {
    use super::*;

    fn geom(sep: f64, s_sun: f64, s_moon: f64) -> SolarGeom {
        SolarGeom { sep_deg: sep, s_sun_deg: s_sun, s_moon_deg: s_moon }
    }

    #[test]
    fn magnitude_is_one_when_centers_coincide_and_moon_larger() {
        let g = geom(0.0, 0.26, 0.28);
        let mag = covered_diameter_fraction(&g);
        assert!(mag >= 1.0, "central total magnitude {mag}");
    }

    #[test]
    fn magnitude_zero_outside_contact() {
        let g = geom(1.0, 0.26, 0.26); // sep > s_sun + s_moon
        assert_eq!(covered_diameter_fraction(&g), 0.0);
    }

    #[test]
    fn obscuration_full_when_sun_fully_covered() {
        let g = geom(0.0, 0.26, 0.30);
        assert!((obscuration_fraction(&g) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn obscuration_zero_when_disjoint() {
        let g = geom(1.0, 0.26, 0.26);
        assert_eq!(obscuration_fraction(&g), 0.0);
    }

    #[test]
    fn obscuration_between_zero_and_one_partial() {
        let g = geom(0.30, 0.26, 0.26);
        let o = obscuration_fraction(&g);
        assert!(o > 0.0 && o < 1.0, "partial obscuration {o}");
    }

    #[test]
    fn annular_obscuration_is_area_ratio() {
        // Moon fully inside Sun (annular): d + r_m <= r_s.
        let g = geom(0.0, 0.28, 0.26);
        let o = obscuration_fraction(&g);
        let expected = (0.26_f64 / 0.28).powi(2);
        assert!((o - expected).abs() < 1e-6, "annular obscuration {o} vs {expected}");
    }
}
```

Note: the `obscuration_fraction` body above must be cleaned up before committing — the intermediate `lens`/`overlap`/`tri` scratch lines were included only to make the standard lens formula explicit; keep **only** the final `lens_area` computation:
```rust
let lens_area = r_s2 * a_s + r_m2 * a_m
    - 0.5 * ((r_s + r_m + d) * (-r_s + r_m + d) * (r_s - r_m + d) * (r_s + r_m - d)).max(0.0).sqrt();
(lens_area / (core::f64::consts::PI * r_s2)).clamp(0.0, 1.0)
```
Delete the `let r_s2 = ...` through `let _ = (lens, overlap);` scratch block and the duplicate `a_s`/`a_m` are computed once. Final function body: the three early-return branches, then `a_s`, `a_m`, `lens_area`, and the clamped ratio.

- [ ] **Step 2: Run tests to verify they fail then pass**

Run: `cargo test -p pleiades-eclipse local::solar_geom_tests`
Expected: PASS (6 tests) after cleanup. If `annular_obscuration_is_area_ratio` fails, re-check the `d <= (r_s - r_m)` branch ordering (annular branch must precede the general lens branch).

- [ ] **Step 3: Clippy clean**

Run: `cargo clippy -p pleiades-eclipse --all-targets`
Expected: no warnings (remove the scratch lines flagged as unused).

- [ ] **Step 4: Commit**

```bash
cargo fmt -p pleiades-eclipse
git add crates/pleiades-eclipse/src/local.rs
git commit -m "feat(eclipse): SP-2c solar two-circle geometry (magnitude + obscuration)"
```

---

## Task 4: Solar local maximum + contact-time root-finding

**Files:**
- Modify: `crates/pleiades-eclipse/src/local.rs`

**Interfaces:**
- Consumes: `topo_sun_moon` (Task 2), `solar_geom`/`covered_diameter_fraction` (Task 3); the eclipse's `greatest_eclipse` instant (seed).
- Produces: `pub(crate) struct SolarContactsJd { max_jd, c1_jd, c2_jd, c3_jd, c4_jd, min_sep_deg, s_sun_at_max, s_moon_at_max: f64, c2_c3_present: bool }` and `pub(crate) fn solar_contacts_jd<B: EphemerisBackend>(backend, observer, greatest_jd) -> Result<Option<SolarContactsJd>, EclipseError>`. `None` when the observer sees no partial phase at all (`min_sep >= s_sun + s_moon`). `c2_jd`/`c3_jd` are meaningful only when `c2_c3_present`.

Local maximum minimizes `sep(t)` over a ±0.25-day bracket around the geocentric greatest eclipse (golden-section to ~0.5 s, exactly like `engine::refine_greatest`). Contacts are bisected on `f(t) = sep(t) − threshold`, searched outward from the maximum in each direction over a bounded ~0.25-day half-window (a partial solar eclipse lasts < ~3.5 h locally).

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-eclipse/src/local.rs`:

```rust
/// Half-width of the solar contact search bracket around local maximum (days).
/// A local solar eclipse's partial phase never exceeds ~3.5 h; 0.25 day (6 h)
/// is a safe superset.
const SOLAR_CONTACT_HALF_WINDOW_DAYS: f64 = 0.25;
/// Root/extremum refinement tolerance: 0.5 s, matching the crate's global
/// `refine_greatest` and the SP-2a root-finder.
const REFINE_TOLERANCE_DAYS: f64 = 0.5 / 86_400.0;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SolarContactsJd {
    pub max_jd: f64,
    pub c1_jd: f64,
    pub c2_jd: f64,
    pub c3_jd: f64,
    pub c4_jd: f64,
    pub min_sep_deg: f64,
    pub s_sun_at_max: f64,
    pub s_moon_at_max: f64,
    pub c2_c3_present: bool,
}

/// Topocentric Sun–Moon separation (degrees) at `jd` for `observer`.
fn solar_sep_deg<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    jd: f64,
) -> Result<f64, EclipseError> {
    Ok(solar_geom(&topo_sun_moon(backend, observer, jd)?).sep_deg)
}

/// Golden-section minimize `sep(t)` in `[a,b]` to `REFINE_TOLERANCE_DAYS`.
fn minimize_sep<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    mut a: f64,
    mut b: f64,
) -> Result<f64, EclipseError> {
    let phi = 0.618_033_988_75_f64;
    let mut c = b - (b - a) * phi;
    let mut d = a + (b - a) * phi;
    let mut fc = solar_sep_deg(backend, observer, c)?;
    let mut fd = solar_sep_deg(backend, observer, d)?;
    while (b - a) > REFINE_TOLERANCE_DAYS {
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - (b - a) * phi;
            fc = solar_sep_deg(backend, observer, c)?;
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + (b - a) * phi;
            fd = solar_sep_deg(backend, observer, d)?;
        }
    }
    Ok(0.5 * (a + b))
}

/// Bisect `sep(t) - threshold` between `lo` and `hi` where it changes sign;
/// returns `None` if it does not change sign across the bracket.
fn bisect_contact<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    threshold: impl Fn(f64) -> f64, // threshold as fn of jd (semidiameters vary slowly)
    mut lo: f64,
    mut hi: f64,
) -> Result<Option<f64>, EclipseError> {
    let f = |jd: f64, s: &mut dyn FnMut(f64) -> Result<f64, EclipseError>| -> Result<f64, EclipseError> {
        Ok(s(jd)? - threshold(jd))
    };
    let mut sep = |jd: f64| solar_sep_deg(backend, observer, jd);
    let mut flo = f(lo, &mut sep)?;
    let mut fhi = f(hi, &mut sep)?;
    if flo.signum() == fhi.signum() {
        return Ok(None);
    }
    while (hi - lo) > REFINE_TOLERANCE_DAYS {
        let mid = 0.5 * (lo + hi);
        let fmid = f(mid, &mut sep)?;
        if fmid.signum() == flo.signum() {
            lo = mid;
            flo = fmid;
        } else {
            hi = mid;
            fhi = fmid;
        }
    }
    let _ = fhi;
    Ok(Some(0.5 * (lo + hi)))
}

/// Full set of solar contact instants for `observer` around `greatest_jd`.
pub(crate) fn solar_contacts_jd<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    greatest_jd: f64,
) -> Result<Option<SolarContactsJd>, EclipseError> {
    let max_jd = minimize_sep(
        backend,
        observer,
        greatest_jd - SOLAR_CONTACT_HALF_WINDOW_DAYS,
        greatest_jd + SOLAR_CONTACT_HALF_WINDOW_DAYS,
    )?;
    let g_max = solar_geom(&topo_sun_moon(backend, observer, max_jd)?);
    let external = g_max.s_sun_deg + g_max.s_moon_deg;
    if g_max.sep_deg >= external {
        return Ok(None); // observer sees no eclipse at all
    }
    // Semidiameter sums vary slowly; freeze them at max for the threshold fn
    // (sub-second contact error), matching the design's closed-form treatment.
    let ext_threshold = move |_jd: f64| external;
    let internal = (g_max.s_moon_deg - g_max.s_sun_deg).abs();
    let int_threshold = move |_jd: f64| internal;

    let lo = max_jd - SOLAR_CONTACT_HALF_WINDOW_DAYS;
    let hi = max_jd + SOLAR_CONTACT_HALF_WINDOW_DAYS;
    let c1 = bisect_contact(backend, observer, ext_threshold, lo, max_jd)?.unwrap_or(max_jd);
    let c4 = bisect_contact(backend, observer, ext_threshold, max_jd, hi)?.unwrap_or(max_jd);

    let c2_c3_present = g_max.sep_deg < internal;
    let (c2, c3) = if c2_c3_present {
        let c2 = bisect_contact(backend, observer, int_threshold, c1, max_jd)?.unwrap_or(max_jd);
        let c3 = bisect_contact(backend, observer, int_threshold, max_jd, c4)?.unwrap_or(max_jd);
        (c2, c3)
    } else {
        (max_jd, max_jd)
    };

    Ok(Some(SolarContactsJd {
        max_jd,
        c1_jd: c1,
        c2_jd: c2,
        c3_jd: c3,
        c4_jd: c4,
        min_sep_deg: g_max.sep_deg,
        s_sun_at_max: g_max.s_sun_deg,
        s_moon_at_max: g_max.s_moon_deg,
        c2_c3_present,
    }))
}

#[cfg(test)]
mod solar_contact_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn contacts_bracket_the_maximum() {
        // Analytic on-node backend → central solar eclipse for an equatorial observer.
        let backend = LinearSunMoon::new_moon_at(2_451_550.0).with_moon_latitude(0.0);
        let observer = ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        );
        let c = solar_contacts_jd(&backend, &observer, 2_451_550.0)
            .unwrap()
            .expect("a local eclipse");
        assert!(c.c1_jd <= c.max_jd + 1e-9 && c.max_jd <= c.c4_jd + 1e-9, "C1<=max<=C4");
        assert!(c.c1_jd < c.c4_jd, "C1 strictly before C4");
        if c.c2_c3_present {
            assert!(c.c2_jd >= c.c1_jd - 1e-9 && c.c3_jd <= c.c4_jd + 1e-9, "C2/C3 inside C1..C4");
            assert!(c.c2_jd <= c.max_jd + 1e-9 && c.max_jd <= c.c3_jd + 1e-9, "C2<=max<=C3");
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails then passes**

Run: `cargo test -p pleiades-eclipse local::solar_contact_tests::contacts_bracket_the_maximum`
Expected: PASS. If `LinearSunMoon::with_moon_latitude` does not exist, check the test-backend API (`crates/pleiades-backend/src/test_backend.rs`) and use whatever on-node constructor the existing eclipse tests use (`engine.rs` uses `LinearSunMoon::new_moon_at(..).with_moon_latitude(0.0)`).

- [ ] **Step 3: Full crate test run**

Run: `cargo test -p pleiades-eclipse`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
cargo fmt -p pleiades-eclipse && cargo clippy -p pleiades-eclipse --all-targets
git add crates/pleiades-eclipse/src/local.rs
git commit -m "feat(eclipse): SP-2c solar local maximum + C1-C4 contact times"
```

---

## Task 5: Horizontal position + visibility helper

**Files:**
- Modify: `crates/pleiades-eclipse/src/local.rs`

**Interfaces:**
- Consumes: `topo_sun_moon` (Task 2); `pleiades_apparent::{sidereal_time, true_obliquity_degrees, apparent_from_true, Atmosphere}`; `pleiades_types::{Angle, EclipticCoordinates, ...}`.
- Produces: `pub(crate) fn body_horizontal<B>(backend, observer, atmos, jd, body: LocalBody) -> Result<(f64 az_deg, f64 apparent_alt_deg, bool visible), EclipseError>` with `pub(crate) enum LocalBody { Sun, Moon }`, and a convenience `pub(crate) fn contact_at<B>(..) -> Result<LocalContact, EclipseError>` that packages an instant + `body_horizontal` into a `LocalContact`.

The az/alt rotation is the exact formula SP-2b uses (azimuth from south, west); visibility is `apparent_altitude > 0`.

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-eclipse/src/local.rs`:

```rust
use pleiades_apparent::{apparent_from_true, Atmosphere};
use pleiades_types::Angle;

/// Which body a horizontal position is computed for.
#[derive(Clone, Copy, Debug)]
pub(crate) enum LocalBody {
    Sun,
    Moon,
}

/// Topocentric azimuth (from south, west, `[0,360)`), apparent (refracted)
/// altitude, and above-horizon visibility of `body` at `jd` for `observer`.
pub(crate) fn body_horizontal<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    atmos: Atmosphere,
    jd: f64,
    body: LocalBody,
) -> Result<(f64, f64, bool), EclipseError> {
    let t = topo_sun_moon(backend, observer, jd)?;
    let (lon, lat, dist) = match body {
        LocalBody::Sun => (t.sun_lon_deg, t.sun_lat_deg, t.sun_dist_au),
        LocalBody::Moon => (t.moon_lon_deg, t.moon_lat_deg, t.moon_dist_au),
    };
    let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
    let eps = true_obliquity_degrees(jd)
        .map_err(|e| EclipseError::Backend(format!("obliquity failed: {e}")))?;
    let equ = EclipticCoordinates::new(
        Longitude::from_degrees(lon),
        Latitude::from_degrees(lat),
        Some(dist),
    )
    .to_equatorial(Angle::from_degrees(eps));
    let ra_deg = equ.right_ascension.degrees();
    let dec_deg = equ.declination.degrees();
    let lst = sidereal_time(at, observer.longitude).local_apparent_deg;
    let ha = (lst - ra_deg).to_radians();
    let dec = dec_deg.to_radians();
    let phi = observer.latitude.degrees().to_radians();
    // Standard equatorial → horizontal (azimuth from south, increasing west).
    let sin_alt = (phi.sin() * dec.sin() + phi.cos() * dec.cos() * ha.cos()).clamp(-1.0, 1.0);
    let true_alt = sin_alt.asin().to_degrees();
    let az = ha.sin().atan2(ha.cos() * phi.sin() - dec.tan() * phi.cos());
    let apparent_alt = apparent_from_true(true_alt, atmos);
    Ok((
        az.to_degrees().rem_euclid(360.0),
        apparent_alt,
        apparent_alt > 0.0,
    ))
}

/// Packages a `jd` + a body's horizontal position into a `LocalContact`.
pub(crate) fn contact_at<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    atmos: Atmosphere,
    jd: f64,
    body: LocalBody,
) -> Result<LocalContact, EclipseError> {
    let (az, alt, visible) = body_horizontal(backend, observer, atmos, jd, body)?;
    Ok(LocalContact {
        instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
        altitude_degrees: alt,
        azimuth_degrees: az,
        visible,
    })
}

#[cfg(test)]
mod horizontal_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn altitude_is_finite_and_azimuth_in_range() {
        let backend = LinearSunMoon::new_moon_at(2_451_550.0);
        let observer = ObserverLocation::new(
            Latitude::from_degrees(40.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        );
        let (az, alt, _vis) =
            body_horizontal(&backend, &observer, Atmosphere::default(), 2_451_545.0, LocalBody::Sun)
                .unwrap();
        assert!(alt.is_finite() && alt <= 90.0 + 1e-9, "alt {alt}");
        assert!((0.0..360.0).contains(&az), "az {az}");
    }
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo test -p pleiades-eclipse local::horizontal_tests::altitude_is_finite_and_azimuth_in_range`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
cargo fmt -p pleiades-eclipse && cargo clippy -p pleiades-eclipse --all-targets
git add crates/pleiades-eclipse/src/local.rs
git commit -m "feat(eclipse): SP-2c horizontal position + visibility helper"
```

---

## Task 6: Assemble solar local circumstances

**Files:**
- Modify: `crates/pleiades-eclipse/src/local.rs`

**Interfaces:**
- Consumes: `solar_contacts_jd` (Task 4), `solar_geom`/`covered_diameter_fraction`/`obscuration_fraction` (Task 3), `contact_at`/`body_horizontal` (Task 5).
- Produces: `pub(crate) fn solar_local<B>(backend, observer, atmos, greatest_jd) -> Result<LocalSolarCircumstances, EclipseError>`. Uses the Sun as the eclipsed body for all contacts. `local_type` is derived from the geometry at maximum; `any_phase_visible` is true if the Sun is above the horizon anywhere in `[C1,C4]`.

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-eclipse/src/local.rs`:

```rust
use crate::types::SolarEclipseType;

/// Classifies what the observer sees at local maximum from the two-circle
/// geometry: total when the Moon's disk fully covers the Sun's
/// (`sep + s_sun <= s_moon`), annular when the Moon is fully inside
/// (`sep + s_moon <= s_sun`), else partial. Hybrid is a global (path-level)
/// distinction, not a single-observer one, so a single observer sees total or
/// annular, never "hybrid".
fn classify_local_solar(c: &SolarContactsJd) -> SolarEclipseType {
    let (sep, s_sun, s_moon) = (c.min_sep_deg, c.s_sun_at_max, c.s_moon_at_max);
    if c.c2_c3_present && sep + s_sun <= s_moon + 1e-9 {
        SolarEclipseType::Total
    } else if c.c2_c3_present && sep + s_moon <= s_sun + 1e-9 {
        SolarEclipseType::Annular
    } else {
        SolarEclipseType::Partial
    }
}

/// Whether the Sun is above the horizon anywhere in `[c1,c4]` (coarse 2-min scan).
fn solar_any_visible<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    atmos: Atmosphere,
    c1_jd: f64,
    c4_jd: f64,
) -> Result<bool, EclipseError> {
    let step = 2.0 / 1440.0;
    let mut jd = c1_jd;
    while jd <= c4_jd + 1e-12 {
        let (_, _, vis) = body_horizontal(backend, observer, atmos, jd, LocalBody::Sun)?;
        if vis {
            return Ok(true);
        }
        jd += step;
    }
    Ok(false)
}

/// Full solar local circumstances for `observer` around `greatest_jd`.
pub(crate) fn solar_local<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    atmos: Atmosphere,
    greatest_jd: f64,
) -> Result<LocalSolarCircumstances, EclipseError> {
    let c = solar_contacts_jd(backend, observer, greatest_jd)?;
    let sun = LocalBody::Sun;
    match c {
        None => {
            // No eclipse for this observer: a degenerate all-at-greatest record,
            // not visible. `next_local_eclipse` filters these out; `local_circumstances`
            // still returns it so a caller can inspect "not visible here".
            let contact = contact_at(backend, observer, atmos, greatest_jd, sun)?;
            let g = solar_geom(&topo_sun_moon(backend, observer, greatest_jd)?);
            Ok(LocalSolarCircumstances {
                local_type: SolarEclipseType::Partial,
                maximum: contact,
                magnitude: covered_diameter_fraction(&g), // 0.0 here
                obscuration: obscuration_fraction(&g),
                first_contact: contact,
                second_contact: None,
                third_contact: None,
                fourth_contact: contact,
                any_phase_visible: contact.visible,
            })
        }
        Some(c) => {
            let g_max = solar_geom(&topo_sun_moon(backend, observer, c.max_jd)?);
            let local_type = classify_local_solar(&c);
            let maximum = contact_at(backend, observer, atmos, c.max_jd, sun)?;
            let first_contact = contact_at(backend, observer, atmos, c.c1_jd, sun)?;
            let fourth_contact = contact_at(backend, observer, atmos, c.c4_jd, sun)?;
            let (second_contact, third_contact) = if c.c2_c3_present {
                (
                    Some(contact_at(backend, observer, atmos, c.c2_jd, sun)?),
                    Some(contact_at(backend, observer, atmos, c.c3_jd, sun)?),
                )
            } else {
                (None, None)
            };
            let any_phase_visible =
                solar_any_visible(backend, observer, atmos, c.c1_jd, c.c4_jd)?;
            Ok(LocalSolarCircumstances {
                local_type,
                maximum,
                magnitude: covered_diameter_fraction(&g_max),
                obscuration: obscuration_fraction(&g_max),
                first_contact,
                second_contact,
                third_contact,
                fourth_contact,
                any_phase_visible,
            })
        }
    }
}

#[cfg(test)]
mod solar_local_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn central_eclipse_has_full_magnitude_and_ordered_contacts() {
        let backend = LinearSunMoon::new_moon_at(2_451_550.0).with_moon_latitude(0.0);
        let observer = ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        );
        let s = solar_local(&backend, &observer, Atmosphere::default(), 2_451_550.0).unwrap();
        assert!(s.magnitude > 0.0, "central magnitude {}", s.magnitude);
        assert!(s.obscuration >= 0.0 && s.obscuration <= 1.0);
        assert!(
            s.first_contact.instant.julian_day.days() <= s.maximum.instant.julian_day.days() + 1e-9
        );
        assert!(
            s.maximum.instant.julian_day.days() <= s.fourth_contact.instant.julian_day.days() + 1e-9
        );
    }
}
```

- [ ] **Step 2: Run test**

Run: `cargo test -p pleiades-eclipse local::solar_local_tests`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
cargo fmt -p pleiades-eclipse && cargo clippy -p pleiades-eclipse --all-targets
git add crates/pleiades-eclipse/src/local.rs
git commit -m "feat(eclipse): SP-2c assemble solar local circumstances"
```

---

## Task 7: Lunar shadow contacts + local circumstances

**Files:**
- Modify: `crates/pleiades-eclipse/src/local.rs`

**Interfaces:**
- Consumes: `sample_sun_moon` (geocentric, for global shadow contacts), the crate's shadow-radius model, `contact_at`/`body_horizontal` (Task 5), `classify_lunar` result via a new lunar-geom helper.
- Produces: `pub(crate) fn lunar_local<B>(backend, observer, atmos, greatest_jd) -> Result<LocalLunarCircumstances, EclipseError>`. Contact instants (P1/U1/U2/U3/U4/P4) are **geocentric/global** (root of the Moon-to-shadow-axis distance crossing each shadow radius), computed once; the Moon is the eclipsed body for az/alt + visibility at each contact.

Reuse the existing lunar shadow geometry: umbral radius `u`, penumbral radius `p`, Moon semidiameter `m_moon`, and the shadow-axis separation `sigma`, all in radians, exactly as `geometry::classify_lunar` computes them. Add a `pub(crate)` accessor in `geometry.rs` returning these per-instant radii so `local.rs` does not duplicate the `SHADOW_INFLATION` model.

- [ ] **Step 1: Add a crate-internal lunar shadow accessor in `geometry.rs`**

In `crates/pleiades-eclipse/src/geometry.rs`, add (near `classify_lunar`):

```rust
/// Per-instant lunar shadow geometry (all radians): shadow-axis separation
/// `sigma`, umbral radius `u`, penumbral radius `p`, and the Moon's
/// semidiameter `m_moon`. Shares the `SHADOW_INFLATION` model with
/// [`classify_lunar`] so local contact-finding does not re-derive it.
pub(crate) struct LunarShadow {
    pub sigma: f64,
    pub u: f64,
    pub p: f64,
    pub m_moon: f64,
}

/// Computes the lunar shadow geometry at a full-moon sample.
pub(crate) fn lunar_shadow(sample: &SunMoonSample) -> LunarShadow {
    let s = (R_SUN_KM / (sample.sun_distance_au * AU_KM)).asin();
    let m_moon = (R_MOON_KM / (sample.moon_distance_au * AU_KM)).asin();
    let pi_moon = (R_EARTH_KM / (sample.moon_distance_au * AU_KM)).asin();
    let pi_sun = (R_EARTH_KM / (sample.sun_distance_au * AU_KM)).asin();
    let earth_shadow = SHADOW_INFLATION * (pi_moon + pi_sun);
    LunarShadow {
        sigma: shadow_axis_separation_rad(sample),
        u: earth_shadow - s,
        p: earth_shadow + s,
        m_moon,
    }
}
```

Refactor `classify_lunar` to call `lunar_shadow` for `s`/`u`/`p`/`sigma`/`m_moon` so the model lives in exactly one place (DRY). Keep `classify_lunar`'s public behavior identical; run `cargo test -p pleiades-eclipse geometry::` to confirm no regression.

- [ ] **Step 2: Write the failing lunar-local test and implementation**

Append to `crates/pleiades-eclipse/src/local.rs`:

```rust
use crate::geometry::lunar_shadow;
use crate::types::LunarEclipseType;

/// Half-window (days) to search for lunar shadow contacts around greatest
/// eclipse. A penumbral lunar eclipse lasts up to ~6 h; 0.25 day is safe.
const LUNAR_CONTACT_HALF_WINDOW_DAYS: f64 = 0.25;

/// Signed residual `dist(t) - radius` for a lunar contact, where `dist` is the
/// Moon-to-shadow-axis separation `sigma` and `radius` is one of the shadow
/// radii (umbral/penumbral), optionally offset by the Moon's semidiameter for
/// the exterior/interior contacts. `kind` selects the residual.
#[derive(Clone, Copy)]
enum LunarContactKind {
    /// P1/P4: sigma == p + m_moon (penumbra first touches / last touches disc).
    Penumbral,
    /// U1/U4: sigma == u + m_moon (umbra first touches / last touches disc).
    UmbralPartial,
    /// U2/U3: sigma == u - m_moon (disc fully enters / begins leaving umbra).
    UmbralTotal,
}

fn lunar_residual<B: EphemerisBackend>(
    backend: &B,
    kind: LunarContactKind,
    jd: f64,
) -> Result<f64, EclipseError> {
    let sh = lunar_shadow(&sample_sun_moon(backend, jd)?);
    let target = match kind {
        LunarContactKind::Penumbral => sh.p + sh.m_moon,
        LunarContactKind::UmbralPartial => sh.u + sh.m_moon,
        LunarContactKind::UmbralTotal => sh.u - sh.m_moon,
    };
    Ok(sh.sigma - target)
}

/// Bisect a lunar contact residual between `lo` and `hi`; `None` if no sign change.
fn bisect_lunar<B: EphemerisBackend>(
    backend: &B,
    kind: LunarContactKind,
    mut lo: f64,
    mut hi: f64,
) -> Result<Option<f64>, EclipseError> {
    let mut flo = lunar_residual(backend, kind, lo)?;
    let fhi = lunar_residual(backend, kind, hi)?;
    if flo.signum() == fhi.signum() {
        return Ok(None);
    }
    while (hi - lo) > REFINE_TOLERANCE_DAYS {
        let mid = 0.5 * (lo + hi);
        let fmid = lunar_residual(backend, kind, mid)?;
        if fmid.signum() == flo.signum() {
            lo = mid;
            flo = fmid;
        } else {
            hi = mid;
        }
    }
    Ok(Some(0.5 * (lo + hi)))
}

/// Full lunar local circumstances. Contact instants are global; visibility is local.
pub(crate) fn lunar_local<B: EphemerisBackend>(
    backend: &B,
    observer: &ObserverLocation,
    atmos: Atmosphere,
    greatest_jd: f64,
) -> Result<LocalLunarCircumstances, EclipseError> {
    let moon = LocalBody::Moon;
    let g = classify_lunar_public(backend, greatest_jd)?; // (type, umbral_mag, penumbral_mag)
    let (eclipse_type, umbral_magnitude, penumbral_magnitude) = g;
    let lo = greatest_jd - LUNAR_CONTACT_HALF_WINDOW_DAYS;
    let hi = greatest_jd + LUNAR_CONTACT_HALF_WINDOW_DAYS;

    let find = |kind: LunarContactKind, a: f64, b: f64| bisect_lunar(backend, kind, a, b);
    // Penumbral contacts always exist for any lunar eclipse.
    let p1 = find(LunarContactKind::Penumbral, lo, greatest_jd)?.unwrap_or(greatest_jd);
    let p4 = find(LunarContactKind::Penumbral, greatest_jd, hi)?.unwrap_or(greatest_jd);
    let u1 = find(LunarContactKind::UmbralPartial, lo, greatest_jd)?;
    let u4 = find(LunarContactKind::UmbralPartial, greatest_jd, hi)?;
    let u2 = find(LunarContactKind::UmbralTotal, lo, greatest_jd)?;
    let u3 = find(LunarContactKind::UmbralTotal, greatest_jd, hi)?;

    let mk = |jd: f64| contact_at(backend, observer, atmos, jd, moon);
    let opt = |o: Option<f64>| -> Result<Option<LocalContact>, EclipseError> {
        match o {
            Some(jd) => Ok(Some(mk(jd)?)),
            None => Ok(None),
        }
    };

    // Visibility across the widest phase present (P1..P4).
    let any_phase_visible = {
        let step = 5.0 / 1440.0;
        let mut jd = p1;
        let mut vis = false;
        while jd <= p4 + 1e-12 {
            let (_, _, v) = body_horizontal(backend, observer, atmos, jd, moon)?;
            if v {
                vis = true;
                break;
            }
            jd += step;
        }
        vis
    };

    Ok(LocalLunarCircumstances {
        eclipse_type,
        maximum: mk(greatest_jd)?,
        umbral_magnitude,
        penumbral_magnitude,
        penumbral_begin: mk(p1)?,
        partial_begin: opt(u1)?,
        total_begin: opt(u2)?,
        total_end: opt(u3)?,
        partial_end: opt(u4)?,
        penumbral_end: mk(p4)?,
        any_phase_visible,
    })
}

#[cfg(test)]
mod lunar_local_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn lunar_contacts_are_ordered_and_penumbra_brackets_umbra() {
        let backend = LinearSunMoon::full_moon_at(2_451_550.0).with_moon_latitude(0.0);
        let observer = ObserverLocation::new(
            Latitude::from_degrees(0.0),
            Longitude::from_degrees(0.0),
            Some(0.0),
        );
        let l = lunar_local(&backend, &observer, Atmosphere::default(), 2_451_550.0).unwrap();
        let p1 = l.penumbral_begin.instant.julian_day.days();
        let p4 = l.penumbral_end.instant.julian_day.days();
        assert!(p1 <= p4, "P1 <= P4");
        if let (Some(u1), Some(u4)) = (l.partial_begin, l.partial_end) {
            assert!(p1 <= u1.instant.julian_day.days() + 1e-9);
            assert!(u4.instant.julian_day.days() <= p4 + 1e-9);
        }
    }
}
```

You must provide `classify_lunar_public`. Add to `local.rs` a thin wrapper that reuses `geometry::classify_lunar` plus its penumbral magnitude. Since `classify_lunar` currently returns one `magnitude` (umbral for total/partial, penumbral for penumbral), extend it or add a sibling that returns both umbral and penumbral magnitudes explicitly:

```rust
/// Returns `(type, umbral_magnitude, penumbral_magnitude)` at greatest eclipse.
fn classify_lunar_public<B: EphemerisBackend>(
    backend: &B,
    greatest_jd: f64,
) -> Result<(LunarEclipseType, f64, f64), EclipseError> {
    let sample = sample_sun_moon(backend, greatest_jd)?;
    let sh = lunar_shadow(&sample);
    let umbral_magnitude = ((sh.u + sh.m_moon - sh.sigma) / (2.0 * sh.m_moon)).max(0.0);
    let penumbral_magnitude = ((sh.p + sh.m_moon - sh.sigma) / (2.0 * sh.m_moon)).max(0.0);
    let eclipse_type = if sh.sigma + sh.m_moon <= sh.u {
        LunarEclipseType::Total
    } else if sh.sigma - sh.m_moon < sh.u {
        LunarEclipseType::Partial
    } else {
        LunarEclipseType::Penumbral
    };
    Ok((eclipse_type, umbral_magnitude, penumbral_magnitude))
}
```

This mirrors `geometry::classify_lunar`'s thresholds exactly (verify against that function; the type boundaries must match to keep the global and local classifications consistent).

- [ ] **Step 3: Run tests**

Run: `cargo test -p pleiades-eclipse local::lunar_local_tests geometry::`
Expected: PASS. If `LinearSunMoon::full_moon_at` does not exist, use the constructor the existing lunar test uses (`geometry.rs` tests build full-moon samples directly; check `test_backend.rs` for a full-moon constructor, else add a minimal one or drive via `new_moon_at` offset by half a synodic month). Adapt the test to whatever on-node full-moon backend exists.

- [ ] **Step 4: Commit**

```bash
cargo fmt -p pleiades-eclipse && cargo clippy -p pleiades-eclipse --all-targets
git add crates/pleiades-eclipse/src/local.rs crates/pleiades-eclipse/src/geometry.rs
git commit -m "feat(eclipse): SP-2c lunar shadow contacts + local circumstances"
```

---

## Task 8: EclipseEngine local methods + when_loc search + exports

**Files:**
- Modify: `crates/pleiades-eclipse/src/engine.rs`
- Modify: `crates/pleiades-eclipse/src/local.rs` (add `local_circumstances_for` dispatch)
- Modify: `crates/pleiades-eclipse/src/lib.rs` (already exports types from Task 1; confirm)

**Interfaces:**
- Consumes: `solar_local` (Task 6), `lunar_local` (Task 7); `EclipseEngine::{next_eclipse, previous_eclipse}`; `Eclipse`, `EclipseKind`, `EclipseFilter`.
- Produces (public API):
  ```rust
  EclipseEngine::local_circumstances(&self, eclipse: &Eclipse, observer: &ObserverLocation, atmosphere: Atmosphere) -> Result<LocalCircumstances, EclipseError>
  EclipseEngine::next_local_eclipse(&self, after: Instant, observer: &ObserverLocation, filter: EclipseFilter, atmosphere: Atmosphere) -> Result<Option<(Eclipse, LocalCircumstances)>, EclipseError>
  EclipseEngine::previous_local_eclipse(&self, before: Instant, observer: &ObserverLocation, filter: EclipseFilter, atmosphere: Atmosphere) -> Result<Option<(Eclipse, LocalCircumstances)>, EclipseError>
  ```

- [ ] **Step 1: Add the dispatch helper in `local.rs`**

Append:

```rust
use crate::types::{Eclipse, EclipseKind};

/// Computes local circumstances for an already-found `eclipse`.
pub(crate) fn local_circumstances_for<B: EphemerisBackend>(
    backend: &B,
    eclipse: &Eclipse,
    observer: &ObserverLocation,
    atmos: Atmosphere,
) -> Result<LocalCircumstances, EclipseError> {
    let greatest_jd = eclipse.greatest_eclipse.julian_day.days();
    match eclipse.kind {
        EclipseKind::Solar => Ok(LocalCircumstances::Solar(solar_local(
            backend,
            observer,
            atmos,
            greatest_jd,
        )?)),
        EclipseKind::Lunar => Ok(LocalCircumstances::Lunar(lunar_local(
            backend,
            observer,
            atmos,
            greatest_jd,
        )?)),
    }
}

/// Whether a computed local result has any above-horizon phase.
pub(crate) fn is_locally_visible(local: &LocalCircumstances) -> bool {
    match local {
        LocalCircumstances::Solar(s) => s.any_phase_visible,
        LocalCircumstances::Lunar(l) => l.any_phase_visible,
    }
}
```

- [ ] **Step 2: Write the failing engine test**

In `crates/pleiades-eclipse/src/engine.rs` tests module, add:

```rust
#[test]
fn local_circumstances_returns_solar_for_a_solar_eclipse() {
    use pleiades_apparent::Atmosphere;
    use pleiades_types::{Latitude, Longitude, ObserverLocation};
    let engine =
        EclipseEngine::new(LinearSunMoon::new_moon_at(2_451_550.0).with_moon_latitude(0.0));
    let eclipse = engine
        .next_eclipse(at(2_451_549.0), EclipseFilter::SolarOnly)
        .unwrap()
        .expect("a solar eclipse");
    let observer = ObserverLocation::new(
        Latitude::from_degrees(0.0),
        Longitude::from_degrees(0.0),
        Some(0.0),
    );
    let local = engine
        .local_circumstances(&eclipse, &observer, Atmosphere::default())
        .unwrap();
    assert!(matches!(
        local,
        crate::LocalCircumstances::Solar(_)
    ));
}
```

- [ ] **Step 3: Implement the three engine methods**

In `crates/pleiades-eclipse/src/engine.rs`, add to `impl<B: EphemerisBackend> EclipseEngine<B>` (after `previous_eclipse`). Add imports at the top: `use crate::local::{is_locally_visible, local_circumstances_for};`, `use crate::types::LocalCircumstances;` (or reference via `crate::LocalCircumstances`), and `use pleiades_apparent::Atmosphere;`.

```rust
/// Local (per-observer) circumstances for an already-found `eclipse`.
///
/// Returns full circumstances even when the eclipse is not visible from
/// `observer` (all contacts below the horizon); inspect `any_phase_visible`
/// (via the returned variant) to test visibility. Solar contact instants are
/// observer-dependent (topocentric); lunar contact instants are global with
/// per-observer visibility.
pub fn local_circumstances(
    &self,
    eclipse: &Eclipse,
    observer: &ObserverLocation,
    atmosphere: Atmosphere,
) -> Result<LocalCircumstances, EclipseError> {
    observer
        .validate()
        .map_err(|e| EclipseError::InvalidObserver { detail: e.to_string() })?;
    check_atmosphere(atmosphere)?;
    local_circumstances_for(&self.backend, eclipse, observer, atmosphere)
}

/// The next eclipse admitted by `filter`, strictly after `after`, that is
/// locally visible from `observer` (any phase above the horizon), paired with
/// its local circumstances. Walks the global `next_eclipse` sequence and
/// returns the first locally-visible one, so the result is a strict refinement
/// of the global engine.
pub fn next_local_eclipse(
    &self,
    after: Instant,
    observer: &ObserverLocation,
    filter: EclipseFilter,
    atmosphere: Atmosphere,
) -> Result<Option<(Eclipse, LocalCircumstances)>, EclipseError> {
    observer
        .validate()
        .map_err(|e| EclipseError::InvalidObserver { detail: e.to_string() })?;
    check_atmosphere(atmosphere)?;
    let mut cursor = after;
    // Bounded walk: no more than MAX_LOCAL_SEARCH global eclipses inspected
    // before giving up (backstop; a locally-visible eclipse always occurs well
    // within the window). ~2 eclipses/year × 200 yr ≈ 1200 global eclipses max.
    for _ in 0..MAX_LOCAL_SEARCH {
        let Some(eclipse) = self.next_eclipse(cursor, filter)? else {
            return Ok(None);
        };
        let local = local_circumstances_for(&self.backend, &eclipse, observer, atmosphere)?;
        if is_locally_visible(&local) {
            return Ok(Some((eclipse, local)));
        }
        cursor = eclipse.greatest_eclipse;
    }
    Ok(None)
}

/// The previous eclipse admitted by `filter`, strictly before `before`, that
/// is locally visible from `observer`, paired with its local circumstances.
pub fn previous_local_eclipse(
    &self,
    before: Instant,
    observer: &ObserverLocation,
    filter: EclipseFilter,
    atmosphere: Atmosphere,
) -> Result<Option<(Eclipse, LocalCircumstances)>, EclipseError> {
    observer
        .validate()
        .map_err(|e| EclipseError::InvalidObserver { detail: e.to_string() })?;
    check_atmosphere(atmosphere)?;
    let mut cursor = before;
    for _ in 0..MAX_LOCAL_SEARCH {
        let Some(eclipse) = self.previous_eclipse(cursor, filter)? else {
            return Ok(None);
        };
        let local = local_circumstances_for(&self.backend, &eclipse, observer, atmosphere)?;
        if is_locally_visible(&local) {
            return Ok(Some((eclipse, local)));
        }
        cursor = eclipse.greatest_eclipse;
    }
    Ok(None)
}
```

Add near the top of `engine.rs`:
```rust
/// Backstop cap on how many global eclipses `next/previous_local_eclipse`
/// inspect before returning `None` (a locally-visible eclipse always occurs
/// far sooner; this only guards against a pathological non-terminating walk).
const MAX_LOCAL_SEARCH: usize = 4000;
```

- [ ] **Step 4: Add the error variants and atmosphere check**

In `crates/pleiades-eclipse/src/error.rs`, add variants if absent:
```rust
/// The supplied observer location was invalid.
InvalidObserver {
    /// Human-readable reason.
    detail: String,
},
/// The supplied atmosphere had a non-finite field.
InvalidAtmosphere {
    /// Human-readable reason.
    detail: String,
},
```
with matching `Display` arms. Add a private `check_atmosphere` to `engine.rs` mirroring `pleiades_events::rise_trans::check_atmosphere`:
```rust
fn check_atmosphere(atmos: Atmosphere) -> Result<(), EclipseError> {
    if !atmos.pressure_mbar.is_finite() || !atmos.temperature_c.is_finite() {
        return Err(EclipseError::InvalidAtmosphere {
            detail: format!("pressure={} temp={}", atmos.pressure_mbar, atmos.temperature_c),
        });
    }
    Ok(())
}
```

- [ ] **Step 5: Run the engine tests**

Run: `cargo test -p pleiades-eclipse`
Expected: PASS (including the new `local_circumstances_returns_solar_for_a_solar_eclipse`).

- [ ] **Step 6: Restore the doc link from Task 1 Step 4**

Ensure the `lib.rs` scope block's `[`EclipseEngine::local_circumstances`]` intra-doc link now resolves. Run: `cargo doc -p pleiades-eclipse --no-deps` and confirm no broken-link warning.

- [ ] **Step 7: Commit**

```bash
cargo fmt -p pleiades-eclipse && cargo clippy -p pleiades-eclipse --all-targets
git add crates/pleiades-eclipse/src/engine.rs crates/pleiades-eclipse/src/local.rs crates/pleiades-eclipse/src/error.rs crates/pleiades-eclipse/src/lib.rs
git commit -m "feat(eclipse): SP-2c EclipseEngine local_circumstances + when_loc search"
```

---

## Task 9: Swiss-Ephemeris reference generator tool

**Files:**
- Create: `tools/se-eclipse-local-reference/Cargo.toml`
- Create: `tools/se-eclipse-local-reference/src/main.rs`
- Create: `tools/se-eclipse-local-reference/LICENSE-NOTES.md`

**Interfaces:**
- Produces (files, when run): `sol-local.csv`, `lun-local.csv`, `manifest.txt` — the reference corpus consumed by Task 10/11. This is a standalone binary, not a workspace member; it links Swiss Ephemeris.

Mirror `tools/se-rise-trans-reference` exactly for structure (standalone `[workspace]` table, `swisseph`/`libswisseph-sys` deps, `SEFLG_MOSEPH`, fnv1a64 manifest, `--dry-run`/write modes, `JD_WINDOW_LO/HI` clamp). Read that tool's `src/main.rs` in full before writing this one and copy its scaffolding (arg parsing, `serr_string`, `se_version`, `fnv1a64`, CSV writing, manifest emission).

- [ ] **Step 1: Create the Cargo manifest**

`tools/se-eclipse-local-reference/Cargo.toml`:
```toml
[package]
name = "se-eclipse-local-reference"
version = "0.0.0"
edition = "2021"
publish = false

# Standalone: own workspace root so a nested build does not attach to the repo
# workspace (see se-rise-trans-reference for the rationale).
[workspace]

[dependencies]
swisseph = "0.1.1"
libswisseph-sys = "0.1.2"
```

- [ ] **Step 2: Write `src/main.rs`**

The binary emits two CSVs. For each curated `(eclipse epoch seed, observer)` case:

- **Solar** (`sol-local.csv`): call `swe_sol_eclipse_when_loc(tjd_start, geopos, ...)` to get the local maximum + C1..C4 UT instants and `attr[]` (magnitude, obscuration); OR, when driving from a known eclipse epoch, call `swe_sol_eclipse_how(tjd_max, geopos, ...)` for magnitude/obscuration and `swe_sol_eclipse_when_loc` for the contact instants. For az/alt at each contact, call `swe_azalt(tjd_ut, SE_EQU2HOR or SE_ECL2HOR, geopos, atpress, attemp, sun_xin)` after `swe_calc_ut` of the Sun, so the azimuth convention (from south, west) matches the engine. Columns:
  `label,lat_deg,lon_deg,elev_m,se_max_jd_ut,se_c1_jd_ut,se_c2_jd_ut,se_c3_jd_ut,se_c4_jd_ut,se_local_type,se_magnitude,se_obscuration,se_max_az_deg,se_max_true_alt_deg,se_max_app_alt_deg,se_any_visible`
  Use empty fields for absent C2/C3 (partial-only) and a sentinel (e.g. `-1`) for below-horizon `se_any_visible=0`. Convert each UT JD to nothing here (keep UT; the gate compares against `_ut`, matching `validate-rise-trans`).
- **Lunar** (`lun-local.csv`): `swe_lun_eclipse_when_loc` + `swe_lun_eclipse_how` for P1/U1/U2/U3/U4/P4 UT instants, umbral/penumbral magnitude, and az/alt of the Moon at greatest eclipse via `swe_azalt`. Columns:
  `label,lat_deg,lon_deg,elev_m,se_max_jd_ut,se_p1_jd_ut,se_u1_jd_ut,se_u2_jd_ut,se_u3_jd_ut,se_u4_jd_ut,se_p4_jd_ut,se_type,se_umbral_mag,se_penumbral_mag,se_max_az_deg,se_max_true_alt_deg,se_max_app_alt_deg,se_any_visible`

Curated cases (choose eclipse seeds inside 1900–2100 and observers spanning the categories):
```rust
// (label, seed_jd_ut_near_eclipse, lat, lon, elev)
// Solar:
//   2017-08-21 total: observer in path (Madras OR, ~44.9N,-123.0), partial (NYC 40.7N,-74.0), below-horizon (Tokyo 35.7N,139.7)
//   2024-04-08 total: in-path (Mazatlan 23.2N,-106.4), partial (Miami 25.8N,-80.2)
//   an annular (2023-10-14): in-path (San Antonio 29.4N,-98.5)
//   a hybrid (2013-11-03): a coastal observer
// Lunar:
//   2018-07-27 total: Moon up (Cape Town -33.9,18.4), Moon down (Los Angeles 34.0,-118.2)
//   a partial (2019-07-16): Moon up (Rome 41.9,12.5)
//   a penumbral (2020-01-10): Moon up (Delhi 28.6,77.2)
```
Target ~30 solar rows and ~20 lunar rows (~50 total), matching the design's corpus size. Skip any case SE reports `SE_ECL_NONVISIBLE` inconsistently; keep rows deterministic.

Write the manifest exactly like `se-rise-trans-reference`'s: per-CSV row count + `fnv1a64` digest + SE version line.

- [ ] **Step 3: Build and dry-run the tool**

Run:
```bash
cd tools/se-eclipse-local-reference
SE_EPHE_PATH=/tmp cargo run -- --dry-run | head -40
```
Expected: prints two CSV blocks + a manifest block, no panics, SE version line present. (Moshier ephemeris needs no `.se1` files; the eclipse functions work with `SEFLG_MOSEPH`.)

- [ ] **Step 4: Commit (tool only; corpus committed in Task 10)**

```bash
cd /workspace
cat > tools/se-eclipse-local-reference/LICENSE-NOTES.md <<'EOF'
# License notes
This standalone tool links Swiss Ephemeris (AGPL / SE professional license).
It is a build-time reference generator, excluded from the published workspace,
and is never linked into any `pleiades-*` crate. See tools/se-rise-trans-reference
for the same arrangement.
EOF
printf 'target/\nCargo.lock\n' > tools/se-eclipse-local-reference/.gitignore
git add tools/se-eclipse-local-reference/Cargo.toml tools/se-eclipse-local-reference/src/main.rs tools/se-eclipse-local-reference/LICENSE-NOTES.md tools/se-eclipse-local-reference/.gitignore
git commit -m "tools: SP-2c Swiss-Ephemeris local-eclipse reference generator"
```
(Match whatever the sibling tool commits — if `se-rise-trans-reference` commits its `Cargo.lock`, do the same for consistency; check `git ls-files tools/se-rise-trans-reference`.)

---

## Task 10: Commit the reference corpus

**Files:**
- Create: `crates/pleiades-validate/data/eclipses-local-corpus/sol-local.csv`
- Create: `crates/pleiades-validate/data/eclipses-local-corpus/lun-local.csv`
- Create: `crates/pleiades-validate/data/eclipses-local-corpus/manifest.txt`
- Create: `crates/pleiades-validate/data/eclipses-local-corpus/MANIFEST.md` (provenance prose, mirroring the eclipses-corpus `MANIFEST.md`)

**Interfaces:**
- Consumes: the Task 9 tool.
- Produces: the committed corpus + a manifest recording SE version, row counts, and `fnv1a64` digests — the ground truth the Task 11 gate loads via `include_str!`.

- [ ] **Step 1: Generate the corpus**

Run:
```bash
cd /workspace/tools/se-eclipse-local-reference
SE_EPHE_PATH=/tmp cargo run -- --out-dir /workspace/crates/pleiades-validate/data/eclipses-local-corpus
```
(Implement `--out-dir` in Task 9 to write the three files, mirroring `se-rise-trans-reference`'s write mode. If that tool writes to a fixed path instead, match its interface.)

- [ ] **Step 2: Verify the files exist and are non-empty**

Run:
```bash
wc -l /workspace/crates/pleiades-validate/data/eclipses-local-corpus/*.csv
head -3 /workspace/crates/pleiades-validate/data/eclipses-local-corpus/sol-local.csv
cat /workspace/crates/pleiades-validate/data/eclipses-local-corpus/manifest.txt
```
Expected: ~30 solar + ~20 lunar data rows (plus headers), a manifest with two `fnv1a64=` lines and an SE-version line.

- [ ] **Step 3: Write `MANIFEST.md`**

Create `crates/pleiades-validate/data/eclipses-local-corpus/MANIFEST.md` describing: generator tool + exact command, SE version + ephemeris (Moshier `SEFLG_MOSEPH`), time base (UT columns, why — same ΔT note as rise-trans), the curated case list, column meanings, and the `fnv1a64` drift-guard scheme. Mirror `crates/pleiades-validate/data/eclipses-corpus/MANIFEST.md`'s structure.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-validate/data/eclipses-local-corpus/
git commit -m "data(validate): SP-2c committed SE local-eclipse reference corpus"
```

---

## Task 11: The `validate-eclipses-local` two-tier gate

**Files:**
- Create: `crates/pleiades-validate/src/eclipse_local_thresholds.rs`
- Create: `crates/pleiades-validate/src/eclipse_local_validation.rs`
- Modify: `crates/pleiades-validate/src/lib.rs`

**Interfaces:**
- Consumes: `pleiades_data::packaged_backend`, `pleiades_eclipse::{EclipseEngine, Eclipse, EclipseFilter, LocalCircumstances, LocalSolarCircumstances, LocalLunarCircumstances}`, `pleiades_apparent::{Atmosphere, fnv1a64}`.
- Produces: `pub fn validate_eclipse_local_corpus() -> Result<EclipseLocalReport, EclipseLocalError>` (with `EclipseLocalReport::summary_line()`), and `pub fn render_eclipse_local_listing(args: &[String]) -> Result<String, String>` for the CLI alias.

Read `crates/pleiades-validate/src/rise_trans_validation.rs` and `rise_trans_thresholds.rs` in full first; this gate follows the same shape (checksum guard → Tier 1 self-consistency → Tier 2 SE parity, per-category ceilings, report struct).

- [ ] **Step 1: Write the thresholds module (measured ceilings placeholder-free)**

`crates/pleiades-validate/src/eclipse_local_thresholds.rs`:
```rust
//! Per-category parity ceilings for `validate-eclipses-local`, mirroring the
//! measured-basis convention of `rise_trans_thresholds`. Each constant is set
//! from the MEASURED maximum residual over the committed corpus (~1.4× the
//! observed max), not guessed. Step 5 of Task 11 records the measurement.

/// Contact/max instant parity ceiling for well-conditioned solar rows (seconds
/// of time). Solar contacts near the central limit widen; see
/// `SOLAR_SECONDS_GRAZING`.
pub const SOLAR_SECONDS: f64 = 0.0; // set in Step 5 from measured max
/// Contact/max instant ceiling for grazing / central-limit solar rows (seconds).
pub const SOLAR_SECONDS_GRAZING: f64 = 0.0; // set in Step 5
/// Lunar contact/max instant ceiling (seconds). Global instants; expect tight.
pub const LUNAR_SECONDS: f64 = 0.0; // set in Step 5
/// Solar magnitude (diameter fraction) absolute ceiling.
pub const MAGNITUDE_ABS: f64 = 0.0; // set in Step 5 (expected ~0.01)
/// Solar obscuration (area fraction) absolute ceiling.
pub const OBSCURATION_ABS: f64 = 0.0; // set in Step 5 (expected ~0.01)
/// Lunar umbral/penumbral magnitude absolute ceiling.
pub const LUNAR_MAGNITUDE_ABS: f64 = 0.0; // set in Step 5
/// Azimuth parity ceiling (arcseconds) — reuse the SP-2b horizontal ceiling.
pub const AZIMUTH_ARCSEC: f64 = 0.0; // set in Step 5 (expected ~0.2-class)
/// Apparent-altitude parity ceiling (arcseconds).
pub const ALTITUDE_ARCSEC: f64 = 0.0; // set in Step 5 (expected ~0.1-class)
```
The `0.0` values are **deliberately failing sentinels**, replaced with real measured numbers in Step 5 (the gate cannot pass until then). This is the exact process SP-2b used (`rise_trans_thresholds` ceilings are "honestly measured").

- [ ] **Step 2: Write the gate skeleton with Tier 1 + Tier 2 (test-first)**

`crates/pleiades-validate/src/eclipse_local_validation.rs`: parse both CSVs, guard their `fnv1a64` against `manifest.txt` (fail closed on mismatch, like `rise_trans_validation`), assert the pinned row counts, then:

- **Tier 1 (self-consistency, no SE reference):** for every recomputed row, assert `C1 ≤ max ≤ C4` (`P1 ≤ U1 ≤ U2 ≤ max ≤ U3 ≤ U4 ≤ P4` where present); `0 ≤ magnitude`; `0 ≤ obscuration ≤ 1`; `obscuration > 0` iff `magnitude > 0`.
- **Tier 2 (SE parity):** recompute each row with `EclipseEngine::local_circumstances` (drive from the eclipse whose `greatest_eclipse` matches the row's `se_max_jd_ut` via `next_eclipse` seeded just before it) and compare each field against the SE column under the Step-1 ceilings. Convert JD-day residuals to seconds (`× 86400`); az/alt residuals to arcseconds (`× 3600`). Use `SOLAR_SECONDS_GRAZING` for rows flagged grazing (a `grazing` column or `|magnitude−1| < 0.02` heuristic); `SOLAR_SECONDS` otherwise. Compare `local_type`/`se_type` exactly; compare `visible`/`se_any_visible` exactly.

Provide `EclipseLocalError` (variants: `Manifest`, `ChecksumMismatch`, `RowCountMismatch`, `ToleranceExceeded { category, label, residual, ceiling }`, `Engine`) and `EclipseLocalReport { solar_rows, lunar_rows, max_solar_seconds, max_lunar_seconds, max_magnitude, max_obscuration, max_az_arcsec, max_alt_arcsec }` with:
```rust
impl EclipseLocalReport {
    pub fn summary_line(&self) -> String {
        format!(
            "validate-eclipses-local: {} solar + {} lunar rows — max residuals: \
             solar {:.2}s, lunar {:.2}s, mag {:.4}, obsc {:.4}, az {:.3}\", alt {:.3}\"",
            self.solar_rows, self.lunar_rows, self.max_solar_seconds, self.max_lunar_seconds,
            self.max_magnitude, self.max_obscuration, self.max_az_arcsec, self.max_alt_arcsec,
        )
    }
}
```

Add a Tier-1-only unit test that runs without SE and asserts ordering/bounds on the committed corpus's own recomputed values (independent of the Step-1 ceilings, so it passes immediately):
```rust
#[test]
fn tier1_self_consistency_holds() {
    // Recompute every row; assert contact ordering and magnitude/obscuration bounds.
    let report = crate::eclipse_local_validation::run_tier1_only()
        .expect("tier-1 self-consistency must hold on the committed corpus");
    assert!(report.solar_rows > 0 && report.lunar_rows > 0);
}
```
Expose `run_tier1_only()` that performs the checksum guard + Tier 1 and returns the report without applying Step-1 ceilings.

- [ ] **Step 3: Wire the modules into `lib.rs`**

In `crates/pleiades-validate/src/lib.rs`, add `mod eclipse_local_thresholds;`, `pub mod eclipse_local_validation;` (match the visibility of `rise_trans_validation`), and re-export `validate_eclipse_local_corpus` if the sibling gates are re-exported at crate root.

- [ ] **Step 4: Run Tier-1 test**

Run: `cargo test -p pleiades-validate eclipse_local`
Expected: the Tier-1 test PASSES; the full `validate_eclipse_local_corpus()` FAILS (ceilings are `0.0`). That failure is expected until Step 5.

- [ ] **Step 5: Measure residuals and set the ceilings**

Add a temporary `#[test]` (or a `--report` path) that runs the full comparison, prints the max residual per category, and does NOT assert. Run it:
```bash
cargo test -p pleiades-validate eclipse_local_measure -- --nocapture
```
Read the printed maxima. Set each constant in `eclipse_local_thresholds.rs` to `1.4 × measured_max`, rounded to a clean value (e.g. measured 3.1 s → `4.5`), exactly as `rise_trans_thresholds` documents its ceilings. If any residual is implausibly large (e.g. a solar contact off by minutes), STOP — that indicates a real engine bug (frame, parallax sign, or seed mismatch); debug via `superpowers:systematic-debugging` before setting the ceiling. Delete the temporary measurement test.

- [ ] **Step 6: Run the full gate**

Run: `cargo test -p pleiades-validate eclipse_local`
Expected: `validate_eclipse_local_corpus()` PASSES with the measured ceilings.

- [ ] **Step 7: Commit**

```bash
cargo fmt -p pleiades-validate && cargo clippy -p pleiades-validate --all-targets
git add crates/pleiades-validate/src/eclipse_local_thresholds.rs crates/pleiades-validate/src/eclipse_local_validation.rs crates/pleiades-validate/src/lib.rs
git commit -m "feat(validate): SP-2c validate-eclipses-local two-tier SE-parity gate"
```

---

## Task 12: CLI wiring + release-gate battery

**Files:**
- Modify: `crates/pleiades-validate/src/render/cli.rs`

**Interfaces:**
- Consumes: `validate_eclipse_local_corpus`, `render_eclipse_local_listing` (Task 11).
- Produces: `eclipse-local` listing alias, `validate-eclipses-local`/`eclipses-local-gate` dispatch, `run_all_numeric_gates` inclusion, help-banner lines.

- [ ] **Step 1: Add gate dispatch**

In `crates/pleiades-validate/src/render/cli.rs`, add next to the `validate-rise-trans` arm:
```rust
Some("validate-eclipses-local") | Some("eclipses-local-gate") => {
    ensure_no_extra_args(&args[1..], "validate-eclipses-local")?;
    crate::validate_eclipse_local_corpus()
        .map(|r| r.summary_line())
        .map_err(|e| e.to_string())
}
```

- [ ] **Step 2: Add the listing alias**

Next to `Some("eclipses") => crate::eclipse_validation::render_eclipses_listing(&args[1..]),` add:
```rust
Some("eclipse-local") => crate::eclipse_local_validation::render_eclipse_local_listing(&args[1..]),
```
Implement `render_eclipse_local_listing` (Task 11 file) to accept observer + optional date args and print the next locally-visible eclipse's circumstances via `EclipseEngine::next_local_eclipse`, formatted like the `eclipses` listing. Keep it thin.

- [ ] **Step 3: Add to `run_all_numeric_gates`**

In the `run_all_numeric_gates` function, after the `validate_rise_trans_corpus` line:
```rust
crate::validate_eclipse_local_corpus()
    .map_err(|e| format!("eclipses-local gate failed: {e}"))?;
```

- [ ] **Step 4: Add help-banner lines**

In the help banner string, after the `rise-trans-gate` lines, add:
```
  validate-eclipses-local   Run the fail-closed two-tier local-eclipse gate (Tier 1 contact-ordering + magnitude/obscuration bounds; Tier 2 Swiss-Ephemeris parity on local contact times, magnitude, obscuration, azimuth/altitude, visibility) over the committed local-eclipse corpus
  eclipses-local-gate       Alias for validate-eclipses-local
  eclipse-local             Print the next locally-visible eclipse's per-observer circumstances
```

- [ ] **Step 5: Verify the gate runs via CLI and is in the battery**

Run:
```bash
cargo run -p pleiades-validate -- validate-eclipses-local
cargo test -p pleiades-validate run_all_numeric_gates
```
Expected: the CLI prints the summary line and exits 0; the `run_all_numeric_gates_includes_*` test (extend or add one asserting the local gate is included and passes) PASSES.

- [ ] **Step 6: Commit**

```bash
cargo fmt -p pleiades-validate && cargo clippy -p pleiades-validate --all-targets
git add crates/pleiades-validate/src/render/cli.rs crates/pleiades-validate/src/eclipse_local_validation.rs
git commit -m "feat(cli): SP-2c wire validate-eclipses-local into gate battery + eclipse-local alias"
```

---

## Task 13: Versioning, claims, and documentation

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/mod.rs`
- Modify: `crates/pleiades-eclipse/README.md`, `README.md`, `PLAN.md`, `plan/status/01-current-execution-frontier.md`, `plan/status/02-next-slice-candidates.md`
- Modify: overclaim-audit claim source (find the descriptor the `compat-claims-audit` reads — grep for where `validate-rise-trans` evidence is registered).

**Interfaces:**
- Consumes: nothing new.
- Produces: bumped compatibility profile, aligned claim surfaces, SP-2c marked done.

- [ ] **Step 1: Bump the compatibility profile**

In `crates/pleiades-core/src/compatibility/mod.rs`:
```rust
pub const CURRENT_COMPATIBILITY_PROFILE_ID: &str = "pleiades-compatibility-profile/0.7.8";
```
Leave `crates/pleiades-core/src/api_stability.rs` at `0.2.2` (additive change, no rename).

- [ ] **Step 2: Update the overclaim-audit claim registration**

Run `grep -rn "rise-trans\|validate-rise-trans\|0.7.7\|claim_tier" crates/pleiades-core/src crates/pleiades-validate/src/claims* 2>/dev/null` to find where SP-2b's surface registered its claim tier ↔ evidence. Add the SP-2c local-eclipse surface analogously (evidence = `validate-eclipses-local`; profile = 0.7.8). Run the audit:
```bash
cargo run -p pleiades-validate -- compat-claims-audit
```
Expected: OK (claims match evidence). If it reports a mismatch, align the descriptor/profile/prose until it passes — do not weaken the audit.

- [ ] **Step 3: Flip the eclipse crate README**

In `crates/pleiades-eclipse/README.md`, update the scope/coverage prose that says global-only to state local circumstances are now supported (`local_circumstances`, `next/previous_local_eclipse`), mirroring the `lib.rs` scope block edit from Task 1/8.

- [ ] **Step 4: Update PLAN.md and status files**

- `PLAN.md`: update the status line — SP-2b done → **SP-2c done** (list the new surface: `EclipseEngine::local_circumstances`/`next_local_eclipse`/`previous_local_eclipse`, solar+lunar, `validate-eclipses-local` gate, compatibility profile 0.7.8); leave **SP-3 remains** as the sole remaining event-engine slice.
- `plan/status/01-current-execution-frontier.md` and `plan/status/02-next-slice-candidates.md`: move SP-2c from "next candidate" to done; SP-3 becomes the next candidate (not yet scoped).
- Top-level `README.md`: if it enumerates event-engine capabilities/compatibility profile version, add local eclipse circumstances and bump the profile reference to 0.7.8.

- [ ] **Step 5: Full workspace verification**

Run:
```bash
cargo test --workspace
cargo run -p pleiades-validate -- validate-eclipses-local
cargo run -p pleiades-validate -- compat-claims-audit
cargo run -p pleiades-validate -- compatibility-profile
```
Expected: all tests pass; the gate passes; the audit is OK; the profile prints `0.7.8`.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add -A
git commit -m "docs(events): declare SP-2c local eclipse circumstances; profile 0.7.8"
```

---

## Self-Review

**1. Spec coverage:**
- Solar `when_loc` + `how` → Tasks 4, 6, 8. Lunar `when_loc` + `how` → Tasks 7, 8. ✓
- Both kinds → Tasks 6 (solar), 7 (lunar), dispatched in 8. ✓
- Data model (`LocalContact`/`LocalSolarCircumstances`/`LocalLunarCircumstances`/`LocalCircumstances`) → Task 1. ✓
- Topocentric solar contacts vs global lunar contacts → Tasks 4/6 (topocentric) and 7 (global instants + local visibility). ✓
- `when_loc` reuses global walk → Task 8 (`next/previous_local_eclipse` iterate `next_eclipse`/`previous_eclipse`). ✓
- Reuse SP-2b machinery, no new dep, inline az/alt rotation → Tasks 2, 5. ✓
- Atmosphere param + refraction horizon threshold → Tasks 2, 5, 8. ✓
- SE reference tool → Task 9; corpus → Task 10; two-tier gate + measured ceilings → Task 11; CLI + battery → Task 12. ✓
- Versioning 0.7.7→0.7.8, API-stability unchanged, docs/claims/PLAN → Task 13. ✓
- Out-of-window fail-closed, TDB base, azimuth-from-south → Global Constraints, enforced per task. ✓

**2. Placeholder scan:** The only numeric "0.0" values are the Task 11 threshold sentinels, which are explicitly failing placeholders replaced by measured values in Task 11 Step 5 (documented SP-2b convention, not a plan gap). The Task 3 `obscuration_fraction` scratch block is explicitly flagged for deletion with the exact final body given. No "TBD"/"handle edge cases"/"similar to Task N" left.

**3. Type consistency:** `LocalContact`/`LocalSolarCircumstances`/`LocalLunarCircumstances`/`LocalCircumstances` field names are identical across Tasks 1, 6, 7, 8, 11. `EclipseError::{Backend, InvalidObserver, InvalidAtmosphere}` introduced in Tasks 2/8 and used consistently. `topo_sun_moon`/`solar_geom`/`solar_contacts_jd`/`solar_local`/`lunar_local`/`local_circumstances_for`/`is_locally_visible` signatures match between definition and call sites. The three engine method signatures match the design's API block exactly.

**Caveats for the implementer (verify against live code, adapt if different):**
- `EclipseError` variant names/shapes (Task 2/8) — read `error.rs` first; the plan adds `Backend(String)`, `InvalidObserver{detail}`, `InvalidAtmosphere{detail}` if absent.
- Test-backend constructors (`LinearSunMoon::with_moon_latitude`, `full_moon_at`) — confirm in `crates/pleiades-backend/src/test_backend.rs`; adapt tests to the actual on-node solar/lunar constructors the existing eclipse tests use.
- `se-rise-trans-reference` write-mode/arg interface (Task 9/10 `--out-dir`/`--dry-run`) — read its `main.rs` and match its exact CLI shape.
- Overclaim-audit registration point (Task 13) — locate via grep; mirror SP-2b's registration.
