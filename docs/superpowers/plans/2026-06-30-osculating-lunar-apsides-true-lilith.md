# Osculating (true) lunar apsides — "True Lilith" Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Serve `CelestialBody::TrueApogee` and `CelestialBody::TruePerigee` (osculating "True Black Moon Lilith") as ReleaseGrade bodies computed from the release-grade packaged Moon state, gated to Swiss Ephemeris `SE_OSCU_APOG` parity.

**Architecture:** A new pure crate `pleiades-apsides` computes the osculating Kepler apsides from the Moon's geocentric position+velocity. `PackagedDataBackend` builds that state from its own `lookup_ecliptic`+`lookup_motion` (Moon) and returns the apsis as a **mean-J2000 geometric direction**. The chart layer special-cases these two bodies in its apparent stage to apply **precession + nutation only** (no light-time, no aberration), via a new `apparent_apsis_position` in `pleiades-apparent`. A new `tools/se-lilith-reference` emits a committed Swiss Ephemeris reference corpus; a new fail-closed `validate-lilith` gate locks the result in.

**Tech Stack:** Rust (workspace, edition 2021, `resolver = "2"`), `cargo`, vendored Swiss Ephemeris via `libswisseph-sys 0.1.2`, FNV-1a checksums via `pleiades_apparent::fnv1a64`.

## Global Constraints

- Backends are **mean-only, J2000 ecliptic** at the boundary (`capabilities.apparent = false, mean = true`). All of-date / apparent corrections live in the chart layer via `pleiades-apparent`. The apsis backend output MUST be mean-J2000.
- Packaged data accepts **only `TimeScale::Tt` or `TimeScale::Tdb`**, frame `Ecliptic`/`Equatorial`, zodiac `Tropical`, no observer; coverage window **1900–2100 CE**.
- `EclipticCoordinates::new`, `EquatorialCoordinates::new`, `Motion::new` are all `const fn`. `EclipticCoordinates::new(Longitude::from_degrees(..), Latitude::from_degrees(..), Some(dist))`.
- The osculating apsis is a **geometric direction**: Swiss Ephemeris applies **neither light-time nor annual aberration** to `SE_OSCU_APOG`. The output frame is **true ecliptic of date, nutation on** (matches SE default; same frame as the chart's apparent bodies minus light-time/aberration).
- New crate version `0.2.0` to match `workspace.package`; the reference tool is `publish = false`, `version = "0.0.0"`, and lives **outside** the workspace (in `exclude`), mirroring `tools/se-ayanamsa-reference`.
- No `ARTIFACT_VERSION` bump — apsides are **derived at lookup**, not stored; no artifact regeneration.
- TDD, DRY, YAGNI, frequent commits. Run `cargo fmt` before each commit.

---

### Task 1: `pleiades-apsides` — pure osculating-elements crate

**Files:**
- Create: `crates/pleiades-apsides/Cargo.toml`
- Create: `crates/pleiades-apsides/src/lib.rs`
- Modify: `Cargo.toml` (root workspace `members` + `workspace.dependencies`)

**Interfaces:**
- Produces:
  - `pub const MU_EARTH_MOON_AU3_PER_DAY2: f64`
  - `pub struct ApsisPoint { pub longitude_deg: f64, pub latitude_deg: f64, pub distance_au: f64 }`
  - `pub struct Apsides { pub apogee: ApsisPoint, pub perigee: ApsisPoint, pub eccentricity: f64, pub semi_major_au: f64 }`
  - `pub enum ApsidesError { DegenerateOrbit, UnboundOrbit, NonFinite }` (derives `Debug, Clone, Copy, PartialEq, Eq`)
  - `pub fn apsides(pos_au: [f64; 3], vel_au_per_day: [f64; 3], mu: f64) -> Result<Apsides, ApsidesError>`

- [ ] **Step 1: Add the crate to the workspace**

In root `Cargo.toml`, add to `members` (keep alphabetical, after `pleiades-apparent`):
```toml
    "crates/pleiades-apsides",
```
And to `[workspace.dependencies]` (after the `pleiades-apparent` line):
```toml
pleiades-apsides = { path = "crates/pleiades-apsides", version = "0.2.0" }
```

- [ ] **Step 2: Create `crates/pleiades-apsides/Cargo.toml`**

```toml
[package]
name = "pleiades-apsides"
version = "0.2.0"
edition = "2021"
license = "MIT OR Apache-2.0"
description = "Osculating lunar apsides (true apogee/perigee) for the pleiades astrology workspace"
repository = "https://github.com/rahulmutt/pleiades"

[dependencies]
```
(No dependencies — pure `f64` vector algebra over `std`.)

- [ ] **Step 3: Write the failing test**

Create `crates/pleiades-apsides/src/lib.rs` with the tests first (implementation stubs added next step). Add this `tests` module:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // A planar bound orbit with perigee on +x. At perigee the eccentricity
    // vector points to +x, so the perigee longitude is 0° and the apogee is
    // 180° at distance a(1+e).
    fn perigee_on_x_state(a: f64, e: f64, mu: f64) -> ([f64; 3], [f64; 3]) {
        let r_peri = a * (1.0 - e);
        let v_peri = (mu / a * (1.0 + e) / (1.0 - e)).sqrt();
        ([r_peri, 0.0, 0.0], [0.0, v_peri, 0.0])
    }

    #[test]
    fn apogee_is_opposite_perigee_at_correct_distance() {
        let a = 0.00257;
        let e = 0.05;
        let mu = MU_EARTH_MOON_AU3_PER_DAY2;
        let (pos, vel) = perigee_on_x_state(a, e, mu);
        let aps = apsides(pos, vel, mu).unwrap();

        assert!((aps.eccentricity - e).abs() < 1e-9, "ecc {}", aps.eccentricity);
        assert!((aps.semi_major_au - a).abs() < 1e-12, "a {}", aps.semi_major_au);
        assert!((aps.perigee.longitude_deg - 0.0).abs() < 1e-6, "peri lon {}", aps.perigee.longitude_deg);
        assert!((aps.apogee.longitude_deg - 180.0).abs() < 1e-6, "apo lon {}", aps.apogee.longitude_deg);
        assert!((aps.apogee.distance_au - a * (1.0 + e)).abs() < 1e-12, "apo dist {}", aps.apogee.distance_au);
        assert!((aps.perigee.distance_au - a * (1.0 - e)).abs() < 1e-12, "peri dist {}", aps.perigee.distance_au);
        assert!(aps.apogee.latitude_deg.abs() < 1e-9);
    }

    #[test]
    fn near_circular_orbit_is_degenerate() {
        let a = 0.00257;
        let mu = MU_EARTH_MOON_AU3_PER_DAY2;
        let v_circ = (mu / a).sqrt();
        let err = apsides([a, 0.0, 0.0], [0.0, v_circ, 0.0], mu).unwrap_err();
        assert_eq!(err, ApsidesError::DegenerateOrbit);
    }

    #[test]
    fn unbound_orbit_is_rejected() {
        let a = 0.00257;
        let mu = MU_EARTH_MOON_AU3_PER_DAY2;
        // Far above escape velocity → unbound (1/a <= 0).
        let v_escape = (2.0 * mu / a).sqrt() * 1.5;
        let err = apsides([a, 0.0, 0.0], [0.0, v_escape, 0.0], mu).unwrap_err();
        assert_eq!(err, ApsidesError::UnboundOrbit);
    }
}
```

- [ ] **Step 4: Run the test to verify it fails**

Run: `cargo test -p pleiades-apsides`
Expected: FAIL to compile — `apsides`, `Apsides`, `ApsidesError`, `MU_EARTH_MOON_AU3_PER_DAY2` not defined.

- [ ] **Step 5: Write the implementation**

Prepend to `crates/pleiades-apsides/src/lib.rs` (above the tests module):
```rust
//! Osculating lunar apsides — the true (osculating) apogee and perigee of the
//! Moon's instantaneous Kepler ellipse, derived from its geocentric position and
//! velocity. This is Swiss Ephemeris `SE_OSCU_APOG` ("True Black Moon Lilith"),
//! distinct from the smooth mean apogee. Pure two-body geometry; frame-agnostic
//! (output ecliptic longitude/latitude are in the same frame as the input
//! Cartesian state — here, geocentric J2000 mean ecliptic).

/// `G(M⊕ + M☾)` in AU³/day². Derived from GM⊕ = 398600.4418 km³/s² and
/// GM☾ = 4902.800 km³/s² (sum 403503.2418) with 1 AU = 149597870.7 km and
/// 1 day = 86400 s. Starting value; tuned against the `validate-lilith` gate
/// (the apse *direction* depends on μ, so it is the dominant parity knob).
pub const MU_EARTH_MOON_AU3_PER_DAY2: f64 = 8.997_14e-10;

/// One apsis expressed in the input frame's ecliptic longitude/latitude (deg)
/// and geocentric distance (AU).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ApsisPoint {
    pub longitude_deg: f64,
    pub latitude_deg: f64,
    pub distance_au: f64,
}

/// The osculating apogee and perigee plus the shape of the osculating ellipse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Apsides {
    pub apogee: ApsisPoint,
    pub perigee: ApsisPoint,
    pub eccentricity: f64,
    pub semi_major_au: f64,
}

/// Why an osculating apsis could not be formed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApsidesError {
    /// Eccentricity below the conditioning floor (apse direction ill-defined).
    DegenerateOrbit,
    /// Specific orbital energy is non-negative (not an ellipse).
    UnboundOrbit,
    /// A non-finite intermediate value was produced.
    NonFinite,
}

const MIN_ECCENTRICITY: f64 = 1e-6;

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

fn to_ecliptic(p: [f64; 3]) -> Result<ApsisPoint, ApsidesError> {
    let r = norm(p);
    if !r.is_finite() || r == 0.0 {
        return Err(ApsidesError::NonFinite);
    }
    let longitude_deg = p[1].atan2(p[0]).to_degrees().rem_euclid(360.0);
    let latitude_deg = (p[2] / r).asin().to_degrees();
    if !longitude_deg.is_finite() || !latitude_deg.is_finite() {
        return Err(ApsidesError::NonFinite);
    }
    Ok(ApsisPoint {
        longitude_deg,
        latitude_deg,
        distance_au: r,
    })
}

/// Computes the osculating apogee and perigee from a geocentric state vector.
///
/// `pos_au` and `vel_au_per_day` are the geocentric position (AU) and velocity
/// (AU/day) of the Moon in the J2000 mean ecliptic frame; `mu` is `G(M⊕+M☾)` in
/// AU³/day².
pub fn apsides(
    pos_au: [f64; 3],
    vel_au_per_day: [f64; 3],
    mu: f64,
) -> Result<Apsides, ApsidesError> {
    let r = pos_au;
    let v = vel_au_per_day;
    let r_mag = norm(r);
    if !r_mag.is_finite() || r_mag == 0.0 || !mu.is_finite() || mu <= 0.0 {
        return Err(ApsidesError::NonFinite);
    }
    let v2 = dot(v, v);
    let rv = dot(r, v);

    // Eccentricity vector: e = ((v·v − μ/r) r − (r·v) v) / μ. Points to perigee.
    let c1 = (v2 - mu / r_mag) / mu;
    let c2 = rv / mu;
    let e_vec = [
        c1 * r[0] - c2 * v[0],
        c1 * r[1] - c2 * v[1],
        c1 * r[2] - c2 * v[2],
    ];
    let e = norm(e_vec);
    if !e.is_finite() {
        return Err(ApsidesError::NonFinite);
    }
    if e < MIN_ECCENTRICITY {
        return Err(ApsidesError::DegenerateOrbit);
    }

    // Semi-major axis: a = 1 / (2/r − v²/μ). Non-positive inverse ⇒ unbound.
    let inv_a = 2.0 / r_mag - v2 / mu;
    if !inv_a.is_finite() {
        return Err(ApsidesError::NonFinite);
    }
    if inv_a <= 0.0 {
        return Err(ApsidesError::UnboundOrbit);
    }
    let a = 1.0 / inv_a;

    let e_hat = [e_vec[0] / e, e_vec[1] / e, e_vec[2] / e];
    let r_apo = a * (1.0 + e);
    let r_peri = a * (1.0 - e);
    let apo_pos = [-e_hat[0] * r_apo, -e_hat[1] * r_apo, -e_hat[2] * r_apo];
    let peri_pos = [e_hat[0] * r_peri, e_hat[1] * r_peri, e_hat[2] * r_peri];

    Ok(Apsides {
        apogee: to_ecliptic(apo_pos)?,
        perigee: to_ecliptic(peri_pos)?,
        eccentricity: e,
        semi_major_au: a,
    })
}
```

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo fmt -p pleiades-apsides && cargo test -p pleiades-apsides`
Expected: PASS (3 tests).

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml crates/pleiades-apsides
git commit -m "feat(apsides): pure osculating lunar apsides crate (true apogee/perigee)"
```

---

### Task 2: Expose `spherical_state_to_cartesian` from `pleiades-compression`

The backend needs to turn the Moon's spherical position+rates into a Cartesian state. `cartesian_state_to_spherical` is already `pub`; its inverse is `pub(crate)`. Make it `pub` and re-export it (symmetric API).

**Files:**
- Modify: `crates/pleiades-compression/src/frame_recombine.rs:66`
- Modify: `crates/pleiades-compression/src/lib.rs:58-61`

**Interfaces:**
- Produces: `pleiades_compression::spherical_state_to_cartesian(SphericalState) -> CartesianState` (now `pub`, re-exported at crate root alongside `SphericalState`, `CartesianState`).

- [ ] **Step 1: Write the failing test**

Add to the bottom of `crates/pleiades-compression/src/frame_recombine.rs` inside its existing `#[cfg(test)] mod tests` (or add such a module if absent):
```rust
    #[test]
    fn spherical_to_cartesian_is_publicly_reachable_and_round_trips() {
        // Reach it through the crate root to prove the re-export exists.
        let s = crate::SphericalState {
            lon_rad: 1.0,
            lat_rad: 0.1,
            dist_au: 0.0025,
            lon_rate_rad_per_day: 0.2,
            lat_rate_rad_per_day: -0.01,
            dist_rate_au_per_day: 1e-6,
        };
        let c = crate::spherical_state_to_cartesian(s);
        let back = crate::cartesian_state_to_spherical(c);
        assert!((back.lon_rad - s.lon_rad).abs() < 1e-12);
        assert!((back.dist_au - s.dist_au).abs() < 1e-15);
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p pleiades-compression spherical_to_cartesian_is_publicly_reachable`
Expected: FAIL to compile — `crate::spherical_state_to_cartesian` is private (`pub(crate)`, not re-exported).

- [ ] **Step 3: Make it public and re-export it**

In `crates/pleiades-compression/src/frame_recombine.rs:66`, change:
```rust
pub(crate) fn spherical_state_to_cartesian(s: SphericalState) -> CartesianState {
```
to:
```rust
pub fn spherical_state_to_cartesian(s: SphericalState) -> CartesianState {
```

In `crates/pleiades-compression/src/lib.rs:58-61`, add `spherical_state_to_cartesian` to the re-export list:
```rust
pub use frame_recombine::{
    cartesian_au_to_ecliptic, cartesian_state_to_spherical, ecliptic_to_cartesian_au,
    geocentric_from_heliocentric, heliocentric_from_geocentric, spherical_state_to_cartesian,
    CartesianState, SphericalState,
};
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p pleiades-compression spherical_to_cartesian_is_publicly_reachable`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-compression/src/frame_recombine.rs crates/pleiades-compression/src/lib.rs
git commit -m "feat(compression): expose spherical_state_to_cartesian (symmetric with its inverse)"
```

---

### Task 3: `apparent_apsis_position` — precession + nutation only

**Files:**
- Modify: `crates/pleiades-apparent/src/apparent.rs` (add fn after `apparent_sun_position`, ~line 175)
- Modify: `crates/pleiades-apparent/src/lib.rs:42-44` (re-export)
- Test: `crates/pleiades-apparent/src/apparent.rs` (its `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `precess_ecliptic_j2000_to_date`, `nutation`, `ApparentProvenance`, `CorrectionSet`, `MODEL_SOURCES`, `ApparentPosition`, `ApparentPlaceError` (all already in scope in `apparent.rs`).
- Produces: `pub fn apparent_apsis_position(instant: Instant, apsis_geocentric_j2000: EclipticCoordinates) -> Result<ApparentPosition, ApparentPlaceError>`

- [ ] **Step 1: Write the failing test**

Add to `crates/pleiades-apparent/src/apparent.rs`'s `#[cfg(test)] mod tests` (the helper `fixed(..)` already exists there):
```rust
    #[test]
    fn apsis_position_is_precession_and_nutation_only_no_aberration() {
        // At J2000 precession ≈ identity, so the only shift is Δψ/3600 (a few
        // arcsec). There must be NO ~20" annual-aberration term and NO change to
        // latitude (aberration is what would move β; precession at J2000 does not).
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let j2000 = fixed(100.0, 5.0, 0.0025);
        let out = apparent_apsis_position(instant, j2000).unwrap();

        let dlon_arcsec = (out.ecliptic.longitude.degrees() - 100.0) * 3600.0;
        assert!(dlon_arcsec.abs() < 20.0, "lon shift {dlon_arcsec}\" must be precession+nutation only");
        assert!((out.ecliptic.latitude.degrees() - 5.0).abs() < 1e-6, "latitude must be unchanged by aberration");
        assert_eq!(out.provenance.aberration_longitude_arcsec, 0.0);
        assert!(!out.provenance.corrections.annual_aberration);
        assert!(!out.provenance.corrections.light_time);
        assert!(out.provenance.corrections.precession);
        assert!(out.provenance.corrections.nutation_longitude);
        assert_eq!(out.provenance.iterations, 0);
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p pleiades-apparent apsis_position_is_precession_and_nutation_only`
Expected: FAIL to compile — `apparent_apsis_position` not defined.

- [ ] **Step 3: Implement the function**

Insert after `apparent_sun_position` (after line 175) in `crates/pleiades-apparent/src/apparent.rs`:
```rust
/// Computes the of-date ecliptic position of a **derived lunar apsis** (the
/// osculating True Apogee / True Perigee).
///
/// The apse line is a geometric direction, not a body: Swiss Ephemeris applies
/// neither light-time nor annual aberration to `SE_OSCU_APOG`. This routine
/// therefore rotates the J2000 mean direction to the **true ecliptic of date**
/// with precession + nutation in longitude ONLY. Distance passes through
/// unchanged.
pub fn apparent_apsis_position(
    instant: Instant,
    apsis_geocentric_j2000: EclipticCoordinates,
) -> Result<ApparentPosition, ApparentPlaceError> {
    let jd_tt = instant.julian_day.days();
    let lambda_j2000 = apsis_geocentric_j2000.longitude.degrees();
    let beta_j2000 = apsis_geocentric_j2000.latitude.degrees();

    let precessed = precess_ecliptic_j2000_to_date(lambda_j2000, beta_j2000, jd_tt)?;
    let lambda = precessed.longitude_deg;
    let beta = precessed.latitude_deg;
    let nut = nutation(jd_tt)?;

    let apparent_lon = (lambda + nut.delta_psi_arcsec / 3600.0).rem_euclid(360.0);
    let apparent_lat = beta;
    if !apparent_lon.is_finite() || !apparent_lat.is_finite() {
        return Err(ApparentPlaceError::NonFiniteCorrection {
            stage: "apparent-apsis-combine",
        });
    }

    let mut precession_shift = lambda - lambda_j2000;
    if precession_shift > 180.0 {
        precession_shift -= 360.0;
    } else if precession_shift < -180.0 {
        precession_shift += 360.0;
    }

    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(apparent_lon),
        Latitude::from_degrees(apparent_lat),
        apsis_geocentric_j2000.distance_au,
    );
    let provenance = ApparentProvenance {
        light_time_days: 0.0,
        iterations: 0,
        precession_longitude_arcsec: precession_shift * 3600.0,
        nutation_longitude_arcsec: nut.delta_psi_arcsec,
        aberration_longitude_arcsec: 0.0,
        corrections: CorrectionSet {
            light_time: false,
            precession: true,
            annual_aberration: false,
            nutation_longitude: true,
            diurnal_parallax: false,
            diurnal_aberration: false,
        },
        model_sources: MODEL_SOURCES,
    };
    Ok(ApparentPosition {
        ecliptic,
        provenance,
    })
}
```

- [ ] **Step 4: Re-export it**

In `crates/pleiades-apparent/src/lib.rs`, find the `pub use apparent::{...}` line (~42-44) and add `apparent_apsis_position`:
```rust
pub use apparent::{
    apparent_apsis_position, apparent_position, apparent_sun_position, ApparentPosition,
    DEFAULT_MAX_ITERATIONS,
};
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo fmt -p pleiades-apparent && cargo test -p pleiades-apparent apsis_position_is_precession_and_nutation_only`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-apparent/src/apparent.rs crates/pleiades-apparent/src/lib.rs
git commit -m "feat(apparent): apparent_apsis_position (precession+nutation only, no aberration)"
```

---

### Task 4: `PackagedDataBackend` serves the osculating apsides

**Files:**
- Modify: `crates/pleiades-data/Cargo.toml` (add `pleiades-apsides` dep)
- Modify: `crates/pleiades-data/src/lib.rs` (add `apsis_body_claims()`)
- Modify: `crates/pleiades-data/src/backend.rs` (imports, `metadata` claims, `supports_body`, `position` branch, helpers)
- Test: `crates/pleiades-data/src/backend.rs` (its test module) or `crates/pleiades-data/src/tests.rs` if present — use whichever the crate already uses.

**Interfaces:**
- Consumes: `pleiades_apsides::{apsides, MU_EARTH_MOON_AU3_PER_DAY2}`; `pleiades_compression::{SphericalState, spherical_state_to_cartesian}` (Task 2); existing `lookup_ecliptic`/`lookup_motion`.
- Produces: `pleiades_data::apsis_body_claims() -> Vec<pleiades_backend::BodyClaim>`; `PackagedDataBackend` now `supports_body` and serves `TrueApogee`/`TruePerigee`.

- [ ] **Step 1: Add the dependency**

In `crates/pleiades-data/Cargo.toml` under `[dependencies]`:
```toml
pleiades-apsides = { workspace = true }
```
(`pleiades-compression`, `pleiades-backend`, `pleiades-types` are already deps.)

- [ ] **Step 2: Write the failing test**

Add this test to the packaged-backend test module (mirror the file the crate already tests `PackagedDataBackend` in):
```rust
    #[test]
    fn packaged_backend_serves_osculating_true_apsides() {
        use pleiades_backend::{Apparentness, EphemerisBackend, EphemerisRequest};
        use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

        let backend = PackagedDataBackend::new();
        assert!(backend.supports_body(CelestialBody::TrueApogee));
        assert!(backend.supports_body(CelestialBody::TruePerigee));

        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let apo = backend
            .position(&EphemerisRequest::new(CelestialBody::TrueApogee, instant))
            .unwrap();
        let apo_ecl = apo.ecliptic.unwrap();

        // Mean-J2000 geometric direction at the backend boundary.
        assert_eq!(apo.apparent, Apparentness::Mean);
        // Apse line lies in the inclined lunar orbit → |β| up to ~6°.
        assert!(apo_ecl.latitude.degrees().abs() <= 6.5, "β {}", apo_ecl.latitude.degrees());
        // Distance is a Moon-scale apogee (~0.0027 AU).
        let d = apo_ecl.distance_au.unwrap();
        assert!((0.0023..0.0030).contains(&d), "apogee distance {d} AU");
        // Derived motion present.
        assert!(apo.motion.unwrap().longitude_deg_per_day.is_some());

        // Perigee is the opposite apse (~180° away).
        let peri = backend
            .position(&EphemerisRequest::new(CelestialBody::TruePerigee, instant))
            .unwrap()
            .ecliptic
            .unwrap();
        let sep = (apo_ecl.longitude.degrees() - peri.longitude.degrees()).rem_euclid(360.0);
        assert!((sep - 180.0).abs() < 1.0, "apogee/perigee separation {sep}°");

        // The body appears as ReleaseGrade in metadata.
        let claim = backend
            .metadata()
            .body_claims
            .into_iter()
            .find(|c| c.body == CelestialBody::TrueApogee)
            .expect("TrueApogee claim present");
        assert_eq!(claim.tier, pleiades_backend::BodyClaimTier::ReleaseGrade);
    }
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo test -p pleiades-data packaged_backend_serves_osculating_true_apsides`
Expected: FAIL — `supports_body` returns false / `position` errors / no claim.

- [ ] **Step 4: Add `apsis_body_claims()` in `lib.rs`**

In `crates/pleiades-data/src/lib.rs`, after `packaged_body_claims()` (~line 209), add:
```rust
/// Release claims for the derived osculating lunar apsides (True Apogee /
/// Perigee). These are computed from the packaged Moon state at lookup and
/// validated against the Swiss Ephemeris `SE_OSCU_APOG` corpus by the
/// `validate-lilith` gate, so their evidence is `CorpusValidated`.
pub fn apsis_body_claims() -> Vec<pleiades_backend::BodyClaim> {
    use pleiades_backend::{AccuracyClass, BodyClaim, ClaimEvidence};
    let source = "Swiss Ephemeris 2.10.03 SE_OSCU_APOG (validate-lilith)".to_string();
    vec![
        BodyClaim::release_grade(
            CelestialBody::TrueApogee,
            AccuracyClass::High,
            ClaimEvidence::CorpusValidated { source: source.clone() },
        ),
        BodyClaim::release_grade(
            CelestialBody::TruePerigee,
            AccuracyClass::High,
            ClaimEvidence::CorpusValidated { source },
        ),
    ]
}
```
(If `CelestialBody` is not already imported in `lib.rs`, add `use pleiades_types::CelestialBody;` or qualify as `pleiades_types::CelestialBody`.)

- [ ] **Step 5: Wire claims into `metadata()`**

In `crates/pleiades-data/src/backend.rs`, change the `body_claims` block (lines 104-116) to extend with the apsis claims:
```rust
            body_claims: {
                let declared = crate::packaged_body_claims();
                let mut claims: Vec<_> = bodies
                    .iter()
                    .map(|body| {
                        declared
                            .iter()
                            .find(|c| &c.body == body)
                            .cloned()
                            .unwrap_or_else(|| pleiades_backend::BodyClaim::from(body.clone()))
                    })
                    .collect();
                claims.extend(crate::apsis_body_claims());
                claims
            },
```

- [ ] **Step 6: Add imports to `backend.rs`**

Near the existing imports at the top of `crates/pleiades-data/src/backend.rs`, add:
```rust
use pleiades_apsides::{apsides, MU_EARTH_MOON_AU3_PER_DAY2};
use pleiades_compression::{spherical_state_to_cartesian, SphericalState};
use pleiades_types::{JulianDay, Longitude, Latitude, Motion};
```
(Some of these may already be imported — merge into the existing `use pleiades_types::{...}` rather than duplicating. `EphemerisErrorKind`, `EclipticCoordinates`, `EphemerisError`, `BackendId`, `QualityAnnotation`, `Instant`, `CelestialBody` are already in scope.)

- [ ] **Step 7: Extend `supports_body`**

Replace `supports_body` (backend.rs:132-137):
```rust
    fn supports_body(&self, body: CelestialBody) -> bool {
        matches!(
            body,
            CelestialBody::TrueApogee | CelestialBody::TruePerigee
        ) || self.artifact.bodies.iter().any(|series| series.body == body)
    }
```

- [ ] **Step 8: Branch `position` and add the apsis helpers**

In `position` (backend.rs:139), immediately **after** `validate_observer_policy(req, "packaged data", false)?;` (line 158) and **before** `let lookup_instant = ...`, insert:
```rust
        if matches!(
            req.body,
            CelestialBody::TrueApogee | CelestialBody::TruePerigee
        ) {
            return self.osculating_apsis_position(req);
        }
```

Then add these private methods inside `impl PackagedDataBackend` (place them next to `artifact()`, not inside the trait impl):
```rust
    fn osculating_apsis_position(
        &self,
        req: &EphemerisRequest,
    ) -> Result<EphemerisResult, EphemerisError> {
        let ecliptic = self.osculating_apsis_ecliptic(&req.body, req.instant)?;
        let equatorial = ecliptic.to_equatorial(req.instant.mean_obliquity());
        let motion = self.osculating_apsis_motion(&req.body, req.instant)?;

        let mut result = EphemerisResult::new(
            BackendId::new(PACKAGE_NAME),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.ecliptic = Some(ecliptic);
        result.equatorial = Some(equatorial);
        result.motion = Some(motion);
        result.quality = QualityAnnotation::Interpolated;
        Ok(result)
    }

    fn osculating_apsis_ecliptic(
        &self,
        body: &CelestialBody,
        instant: Instant,
    ) -> Result<EclipticCoordinates, EphemerisError> {
        let li = normalize_lookup_instant(instant);
        let ecl = self
            .artifact
            .lookup_ecliptic(&CelestialBody::Moon, li)
            .map_err(map_artifact_error)?;
        let mot = self
            .artifact
            .lookup_motion(&CelestialBody::Moon, li)
            .map_err(map_artifact_error)?;
        let dist = ecl.distance_au.ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "packaged Moon lacks distance for osculating apsis",
            )
        })?;
        let state = SphericalState {
            lon_rad: ecl.longitude.degrees().to_radians(),
            lat_rad: ecl.latitude.degrees().to_radians(),
            dist_au: dist,
            lon_rate_rad_per_day: mot.longitude_deg_per_day.unwrap_or(0.0).to_radians(),
            lat_rate_rad_per_day: mot.latitude_deg_per_day.unwrap_or(0.0).to_radians(),
            dist_rate_au_per_day: mot.distance_au_per_day.unwrap_or(0.0),
        };
        let cart = spherical_state_to_cartesian(state);
        let aps = apsides(cart.pos_au, cart.vel_au_per_day, MU_EARTH_MOON_AU3_PER_DAY2)
            .map_err(|_| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "osculating apsis undefined for the lunar state at this instant",
                )
            })?;
        let point = match body {
            CelestialBody::TrueApogee => aps.apogee,
            CelestialBody::TruePerigee => aps.perigee,
            _ => {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "not an osculating-apsis body",
                ))
            }
        };
        Ok(EclipticCoordinates::new(
            Longitude::from_degrees(point.longitude_deg),
            Latitude::from_degrees(point.latitude_deg),
            Some(point.distance_au),
        ))
    }

    fn osculating_apsis_motion(
        &self,
        body: &CelestialBody,
        instant: Instant,
    ) -> Result<Motion, EphemerisError> {
        const HALF_SPAN_DAYS: f64 = 0.5;
        let shift = |days: f64| {
            Instant::new(
                JulianDay::from_days(instant.julian_day.days() + days),
                instant.scale,
            )
        };
        let before = self.osculating_apsis_ecliptic(body, shift(-HALF_SPAN_DAYS))?;
        let after = self.osculating_apsis_ecliptic(body, shift(HALF_SPAN_DAYS))?;
        let span = 2.0 * HALF_SPAN_DAYS;

        let mut dlon = after.longitude.degrees() - before.longitude.degrees();
        while dlon > 180.0 {
            dlon -= 360.0;
        }
        while dlon < -180.0 {
            dlon += 360.0;
        }
        let dlon_per_day = dlon / span;
        let dlat_per_day = (after.latitude.degrees() - before.latitude.degrees()) / span;
        let ddist_per_day = match (before.distance_au, after.distance_au) {
            (Some(b), Some(a)) => Some((a - b) / span),
            _ => None,
        };
        Ok(Motion::new(Some(dlon_per_day), Some(dlat_per_day), ddist_per_day))
    }
```

- [ ] **Step 9: Run the test to verify it passes**

Run: `cargo fmt -p pleiades-data && cargo test -p pleiades-data packaged_backend_serves_osculating_true_apsides`
Expected: PASS.

- [ ] **Step 10: Run the whole crate to catch regressions**

Run: `cargo test -p pleiades-data`
Expected: PASS (no existing coverage/claims test broke — if a body-count assertion fails, update it to include the two new apsis claims).

- [ ] **Step 11: Commit**

```bash
git add crates/pleiades-data/Cargo.toml crates/pleiades-data/src/lib.rs crates/pleiades-data/src/backend.rs
git commit -m "feat(data): serve osculating true apogee/perigee from the packaged Moon state"
```

---

### Task 5: Chart apparent stage serves `TrueApogee`/`TruePerigee` of-date

**Files:**
- Modify: `crates/pleiades-core/src/chart/mod.rs` (import + apparent-stage branch)
- Test: `crates/pleiades-core/src/chart/tests.rs`

**Interfaces:**
- Consumes: `pleiades_apparent::apparent_apsis_position` (Task 3); `self.query_mean_ecliptic` (existing); `map_apparent_place_error` (existing).

- [ ] **Step 1: Write the failing test**

Add to `crates/pleiades-core/src/chart/tests.rs`. Mirror the construction used by the existing apparent-mode tests in this file (search the file for `Apparentness::Apparent` / the apparent chart-request builder and copy that request setup), requesting `CelestialBody::TrueApogee` and `CelestialBody::TruePerigee` with the production `PackagedDataBackend`:
```rust
    #[test]
    fn chart_serves_apparent_true_apsides_precession_nutation_only() {
        use pleiades_data::PackagedDataBackend;
        use pleiades_types::CelestialBody;

        // Build an apparent-mode chart request at J2000 for both apsides, using
        // the same request-construction helper the other apparent tests in this
        // file use. (Replace `apparent_request_at` with that helper.)
        let request = apparent_request_at(2_451_545.0)
            .with_bodies(vec![CelestialBody::TrueApogee, CelestialBody::TruePerigee]);

        let snapshot = ChartEngine::new(PackagedDataBackend::new())
            .chart(&request)
            .expect("apparent true-apsides chart");

        let apo = snapshot
            .placement(&CelestialBody::TrueApogee)
            .expect("true apogee placement");
        let peri = snapshot
            .placement(&CelestialBody::TruePerigee)
            .expect("true perigee placement");

        // Apse line is inclined → real ecliptic latitude.
        assert!(apo.latitude().abs() <= 6.5);
        // Apogee and perigee stay opposite after the of-date rotation.
        let sep = (apo.longitude() - peri.longitude()).rem_euclid(360.0);
        assert!((sep - 180.0).abs() < 1.0, "apo/peri separation {sep}°");
        // Marked apparent (release-grade, of-date).
        assert_eq!(apo.apparentness(), pleiades_types::Apparentness::Apparent);
    }
```
NOTE: `placement`/`longitude`/`latitude`/`apparentness` accessor names must match `crates/pleiades-core/src/chart/placement.rs` and `snapshot.rs` — confirm against `BodyPlacement` (the reference shows `longitude_speed`/`latitude_speed`; use the sibling longitude/latitude accessors there). Adjust accessor names to the real ones if they differ.

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p pleiades-core chart_serves_apparent_true_apsides`
Expected: FAIL — apsides come back `Mean` (no apparent branch) or the separation/latitude assertions trip because the of-date rotation never ran.

- [ ] **Step 3: Add the import**

In `crates/pleiades-core/src/chart/mod.rs`, extend the `pleiades_apparent` import (mod.rs:39-42) with `apparent_apsis_position`:
```rust
use pleiades_apparent::{
    apparent_apsis_position, apparent_position, apparent_sun_position,
    precess_ecliptic_j2000_to_date, ApparentLightTimeError, ApparentPlaceError,
    DEFAULT_MAX_ITERATIONS,
};
```

- [ ] **Step 4: Add the apsis branch in the apparent stage**

In `crates/pleiades-core/src/chart/mod.rs`, in the per-body `let outcome = if matches!(body, …Sun) { … } else { …planets… };` chain (~mod.rs:313-358), insert a new arm **between** the Sun arm and the planetary `else`:
```rust
        } else if matches!(
            body,
            pleiades_types::CelestialBody::TrueApogee
                | pleiades_types::CelestialBody::TruePerigee
        ) {
            // Osculating apsis: a geometric direction. Apply precession +
            // nutation only (no light-time re-query, no annual aberration).
            // observer = None keeps it geocentric.
            self.query_mean_ecliptic(&body, request.instant, &backend_zodiac_mode, None)
                .and_then(|apsis_j2000| {
                    apparent_apsis_position(request.instant, apsis_j2000)
                        .map_err(map_apparent_place_error)
                })
        } else {
```
(The shared `match outcome { Ok(outcome) => … }` writeback below it consumes the `ApparentPosition` identically to the Sun arm.)

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo fmt -p pleiades-core && cargo test -p pleiades-core chart_serves_apparent_true_apsides`
Expected: PASS.

- [ ] **Step 6: Run the chart suite for regressions**

Run: `cargo test -p pleiades-core`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-core/src/chart/mod.rs crates/pleiades-core/src/chart/tests.rs
git commit -m "feat(chart): serve apparent true apsides via precession+nutation-only path"
```

---

### Task 6: `tools/se-lilith-reference` — Swiss Ephemeris reference corpus

This tool generates the committed golden corpus. It is **outside** the workspace, mirroring `tools/se-ayanamsa-reference`.

**Files:**
- Create: `tools/se-lilith-reference/Cargo.toml`
- Create: `tools/se-lilith-reference/src/main.rs`
- Modify: `Cargo.toml` (root `[workspace] exclude`)
- Create (generated, committed): `crates/pleiades-validate/data/lilith-corpus/lilith.csv`
- Create (committed): `crates/pleiades-validate/data/lilith-corpus/manifest.txt`

**Ephemeris-flag decision (refines the spec's one open item):** the spec named `SEFLG_SWIEPH` as the start. During recon we confirmed `SE_OSCU_APOG` needs the lunar ephemeris, and `SEFLG_SWIEPH` requires Swiss `.se1` files the repo does not commit. So this tool uses **`SEFLG_MOSEPH`** (Moshier analytic theory — **no data files**, fully self-contained and reproducible). The Moshier-vs-DE440 Moon difference is part of the tolerance budget; the documented accuracy upgrade (if the gate band is too loose) is to provision SE/DE440 files and switch to `SEFLG_JPLEPH`. This is recorded in the corpus header and the design's open-item note.

- [ ] **Step 1: Add the tool to the workspace exclude**

In root `Cargo.toml`, extend `exclude`:
```toml
exclude = ["tools/se-house-reference", "tools/se-ayanamsa-reference", "tools/se-lilith-reference"]
```

- [ ] **Step 2: Create `tools/se-lilith-reference/Cargo.toml`**

```toml
[package]
name = "se-lilith-reference"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
swisseph = "0.1.1"
libswisseph-sys = "0.1.2"
```

- [ ] **Step 3: Create `tools/se-lilith-reference/src/main.rs`**

```rust
//! Emits a Swiss Ephemeris reference corpus for the osculating true lunar
//! apogee (`SE_OSCU_APOG`, "True Black Moon Lilith") to STDOUT as CSV.
//!
//! Frame: true ecliptic of date, nutation on (SE default — no SEFLG_NONUT,
//! no SEFLG_J2000). Ephemeris: Moshier (SEFLG_MOSEPH) — no data files needed.
//! The perigee is the opposite apse and is not emitted (SE has no separate
//! perigee body); the gate checks the apogee against SE and the perigee via
//! internal symmetry.
//!
//! Usage: `cargo run --release > .../lilith-corpus/lilith.csv`

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::swe_calc;

const SE_OSCU_APOG: c_int = 13;
const SEFLG_MOSEPH: c_int = 4;

// Deterministic sampling grid across the 1900–2100 packaged window. Step is
// coprime-ish with the ~206-day anomalistic period so successive samples land
// on different orbit phases.
const JD_START_TT: f64 = 2_415_020.5; // 1900-01-01
const JD_END_TT: f64 = 2_488_070.0; //   ~2100-01-01
const STEP_DAYS: f64 = 23.0;

fn se_true_apogee(jd_tt: f64) -> (f64, f64, f64) {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(
            jd_tt,
            SE_OSCU_APOG,
            SEFLG_MOSEPH,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        let msg = unsafe { CStr::from_ptr(serr.as_ptr() as *const c_char) }
            .to_string_lossy()
            .into_owned();
        panic!("swe_calc(SE_OSCU_APOG) failed at jd_tt={jd_tt}: {msg}");
    }
    let (lon, lat, dist) = (xx[0], xx[1], xx[2]);
    assert!(
        lon.is_finite() && lat.is_finite() && dist.is_finite(),
        "non-finite SE result at jd_tt={jd_tt}"
    );
    (lon.rem_euclid(360.0), lat, dist)
}

fn main() {
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), swe_calc SE_OSCU_APOG=13,");
    println!("# iflag=SEFLG_MOSEPH (Moshier, no data files). Frame: true ecliptic of date, nutation on.");
    println!("# Columns: of-date true ecliptic longitude/latitude (deg) and geocentric distance (AU).");
    println!("# Accuracy note: Moshier Moon vs the DE440-sourced packaged Moon is part of the gate budget;");
    println!("# upgrade path is SEFLG_JPLEPH against DE440 if the band is too loose.");
    println!("jd_tt,se_oscu_apogee_lon_deg,se_oscu_apogee_lat_deg,se_oscu_apogee_dist_au");
    let mut jd = JD_START_TT;
    while jd <= JD_END_TT {
        let (lon, lat, dist) = se_true_apogee(jd);
        println!("{jd:.1},{lon:.9},{lat:.9},{dist:.12}");
        jd += STEP_DAYS;
    }
}
```
NOTE: verify the exact symbol name in `libswisseph_sys::raw` is `swe_calc` (signature `swe_calc(tjd_et: f64, ipl: c_int, iflag: c_int, xx: *mut f64, serr: *mut c_char) -> c_int`). If the crate exposes only `swe_calc_ut`, pass `jd_tt` (TT≈ET for this purpose) to it instead — confirm by reading `~/.cargo/registry/src/*/libswisseph-sys-0.1.2/src/raw.rs`.

- [ ] **Step 4: Build the tool**

Run: `cargo build --release --manifest-path tools/se-lilith-reference/Cargo.toml`
Expected: builds. If the linker cannot find `swe_calc`, fix per the NOTE in Step 3.

- [ ] **Step 5: Generate and commit the corpus**

```bash
mkdir -p crates/pleiades-validate/data/lilith-corpus
cargo run --release --manifest-path tools/se-lilith-reference/Cargo.toml \
  > crates/pleiades-validate/data/lilith-corpus/lilith.csv
```
Verify a sane row count (~3176 data rows) and that the file ends without a partial line:
Run: `wc -l crates/pleiades-validate/data/lilith-corpus/lilith.csv`
Expected: ~3182 lines (6 header/comment lines + data).

- [ ] **Step 6: Write the manifest**

Compute the FNV-1a64 checksum of the CSV the same way the gate will (the gate recomputes `pleiades_apparent::fnv1a64(CORPUS_CSV)` over the **entire included file**). The simplest reliable approach: defer the manifest's exact `checksum=`/`rows=` values until Task 7 Step 2, where the gate's own first run prints the computed checksum and row count. For now create the file with placeholders to be filled in Task 7:
```
slice lilith file=lilith.csv role=lilith rows=PENDING checksum=PENDING
```
(Task 7 replaces `PENDING` with the gate-reported values; the gate stays fail-closed until they match.)

- [ ] **Step 7: Commit the tool and corpus**

```bash
git add Cargo.toml tools/se-lilith-reference crates/pleiades-validate/data/lilith-corpus
git commit -m "feat(tools): se-lilith-reference + committed SE_OSCU_APOG reference corpus"
```

---

### Task 7: `validate-lilith` fail-closed gate + CLI wiring

**Files:**
- Create: `crates/pleiades-validate/src/lilith_validation.rs`
- Modify: `crates/pleiades-validate/src/lib.rs` (module + re-export)
- Modify: `crates/pleiades-validate/Cargo.toml` (ensure `pleiades-data`, `pleiades-apparent`, `pleiades-backend`, `pleiades-types` deps)
- Modify: `crates/pleiades-validate/src/render/cli.rs` (subcommand + help)
- Modify: `crates/pleiades-cli/src/cli.rs` (forwarding arm, if the umbrella CLI enumerates subcommands)
- Modify: `crates/pleiades-validate/data/lilith-corpus/manifest.txt` (fill `rows`/`checksum`)

**Interfaces:**
- Consumes: `pleiades_data::PackagedDataBackend`; `pleiades_apparent::{apparent_apsis_position, fnv1a64}`; `pleiades_backend::{EphemerisBackend, EphemerisRequest}`; `pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale}`.
- Produces: `pub fn validate_lilith_corpus() -> Result<LilithCorpusReport, LilithCorpusError>`; `pub struct LilithCorpusReport`; `pub enum LilithCorpusError`.

- [ ] **Step 1: Ensure dependencies**

In `crates/pleiades-validate/Cargo.toml`, confirm (add if missing) under `[dependencies]`:
```toml
pleiades-data = { workspace = true }
pleiades-apparent = { workspace = true }
pleiades-backend = { workspace = true }
pleiades-types = { workspace = true }
```

- [ ] **Step 2: Create the gate (mirrors `ayanamsa_validation.rs`)**

Create `crates/pleiades-validate/src/lilith_validation.rs`:
```rust
//! Fail-closed gate: our of-date osculating True Apogee vs the committed Swiss
//! Ephemeris `SE_OSCU_APOG` reference corpus. Reproduces the exact chart path —
//! packaged-backend mean-J2000 apsis → `apparent_apsis_position` (precession +
//! nutation only) — and compares against SE within published ceilings.

use pleiades_apparent::{apparent_apsis_position, fnv1a64};
use pleiades_backend::{EphemerisBackend, EphemerisRequest};
use pleiades_data::PackagedDataBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/lilith-corpus/lilith.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/lilith-corpus/manifest.txt"
));

// Ceilings. Provisional starting values; tightened to the measured maxima in
// Step 4. The dominant residual is the Moshier-vs-DE440 Moon difference.
const LON_CEILING_ARCSEC: f64 = 600.0;
const LAT_CEILING_ARCSEC: f64 = 600.0;
const DIST_CEILING_REL: f64 = 5e-3;

#[derive(Clone, Copy, Debug)]
struct LilithRow {
    jd_tt: f64,
    lon_deg: f64,
    lat_deg: f64,
    dist_au: f64,
}

#[derive(Debug)]
pub enum LilithCorpusError {
    MalformedRow(String),
    MalformedManifest(String),
    ChecksumMismatch { got: u64, want: u64 },
    ManifestDrift { rows_csv: usize, rows_manifest: usize },
    CalculationFailed { jd_tt: f64, reason: String },
    CeilingExceeded {
        jd_tt: f64,
        kind: &'static str,
        got: f64,
        want: f64,
        residual: f64,
        ceiling: f64,
    },
}

impl std::fmt::Display for LilithCorpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedRow(s) => write!(f, "malformed corpus row: {s}"),
            Self::MalformedManifest(s) => write!(f, "malformed manifest: {s}"),
            Self::ChecksumMismatch { got, want } => {
                write!(f, "corpus checksum mismatch: got {got:016x} want {want:016x}")
            }
            Self::ManifestDrift { rows_csv, rows_manifest } => {
                write!(f, "manifest drift: csv has {rows_csv} rows, manifest says {rows_manifest}")
            }
            Self::CalculationFailed { jd_tt, reason } => {
                write!(f, "calculation failed at jd_tt={jd_tt}: {reason}")
            }
            Self::CeilingExceeded { jd_tt, kind, got, want, residual, ceiling } => write!(
                f,
                "lilith {kind} ceiling exceeded at jd_tt={jd_tt}: got {got:.6} want {want:.6} residual {residual:.4} > ceiling {ceiling:.4}"
            ),
        }
    }
}

impl std::error::Error for LilithCorpusError {}

#[derive(Debug)]
pub struct LilithCorpusReport {
    pub rows_validated: usize,
    pub max_residual_lon_arcsec: f64,
    pub max_residual_lat_arcsec: f64,
    pub max_residual_dist_rel: f64,
    summary_line: String,
}

impl LilithCorpusReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

fn parse_corpus() -> Result<Vec<LilithRow>, LilithCorpusError> {
    let mut rows = Vec::new();
    for line in CORPUS_CSV.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("jd_tt") {
            continue;
        }
        let mut it = line.split(',');
        let mut next = |name: &str| -> Result<f64, LilithCorpusError> {
            it.next()
                .ok_or_else(|| LilithCorpusError::MalformedRow(format!("{name} missing in {line}")))?
                .parse::<f64>()
                .map_err(|e| LilithCorpusError::MalformedRow(format!("{name}: {e} in {line}")))
        };
        rows.push(LilithRow {
            jd_tt: next("jd_tt")?,
            lon_deg: next("lon")?,
            lat_deg: next("lat")?,
            dist_au: next("dist")?,
        });
    }
    Ok(rows)
}

fn parse_manifest_rows() -> Result<(usize, u64), LilithCorpusError> {
    let line = MANIFEST
        .lines()
        .find(|l| l.trim_start().starts_with("slice"))
        .ok_or_else(|| LilithCorpusError::MalformedManifest("no slice line".into()))?;
    let mut rows = None;
    let mut checksum = None;
    for tok in line.split_whitespace() {
        if let Some(v) = tok.strip_prefix("rows=") {
            rows = Some(
                v.parse::<usize>()
                    .map_err(|e| LilithCorpusError::MalformedManifest(format!("rows: {e}")))?,
            );
        } else if let Some(v) = tok.strip_prefix("checksum=") {
            checksum = Some(
                v.parse::<u64>()
                    .map_err(|e| LilithCorpusError::MalformedManifest(format!("checksum: {e}")))?,
            );
        }
    }
    Ok((
        rows.ok_or_else(|| LilithCorpusError::MalformedManifest("rows= missing".into()))?,
        checksum.ok_or_else(|| LilithCorpusError::MalformedManifest("checksum= missing".into()))?,
    ))
}

fn wrap_arcsec(got_deg: f64, want_deg: f64) -> f64 {
    let mut d = got_deg - want_deg;
    while d > 180.0 {
        d -= 360.0;
    }
    while d < -180.0 {
        d += 360.0;
    }
    (d * 3600.0).abs()
}

pub fn validate_lilith_corpus() -> Result<LilithCorpusReport, LilithCorpusError> {
    let (manifest_rows, manifest_checksum) = parse_manifest_rows()?;
    let got_checksum = fnv1a64(CORPUS_CSV);
    if got_checksum != manifest_checksum {
        return Err(LilithCorpusError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let rows = parse_corpus()?;
    if rows.len() != manifest_rows {
        return Err(LilithCorpusError::ManifestDrift {
            rows_csv: rows.len(),
            rows_manifest: manifest_rows,
        });
    }

    let backend = PackagedDataBackend::new();
    let mut max_lon = 0.0_f64;
    let mut max_lat = 0.0_f64;
    let mut max_dist = 0.0_f64;

    for row in &rows {
        let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);
        let mean = backend
            .position(&EphemerisRequest::new(CelestialBody::TrueApogee, instant))
            .map_err(|e| LilithCorpusError::CalculationFailed {
                jd_tt: row.jd_tt,
                reason: e.to_string(),
            })?
            .ecliptic
            .ok_or_else(|| LilithCorpusError::CalculationFailed {
                jd_tt: row.jd_tt,
                reason: "no ecliptic".into(),
            })?;
        let apparent = apparent_apsis_position(instant, mean).map_err(|e| {
            LilithCorpusError::CalculationFailed {
                jd_tt: row.jd_tt,
                reason: format!("{e:?}"),
            }
        })?;

        let our_lon = apparent.ecliptic.longitude.degrees();
        let our_lat = apparent.ecliptic.latitude.degrees();
        let our_dist = apparent.ecliptic.distance_au.unwrap_or(0.0);

        let resid_lon = wrap_arcsec(our_lon, row.lon_deg);
        let resid_lat = ((our_lat - row.lat_deg) * 3600.0).abs();
        let resid_dist = if row.dist_au != 0.0 {
            ((our_dist - row.dist_au) / row.dist_au).abs()
        } else {
            0.0
        };

        if resid_lon > LON_CEILING_ARCSEC {
            return Err(LilithCorpusError::CeilingExceeded {
                jd_tt: row.jd_tt,
                kind: "longitude_arcsec",
                got: our_lon,
                want: row.lon_deg,
                residual: resid_lon,
                ceiling: LON_CEILING_ARCSEC,
            });
        }
        if resid_lat > LAT_CEILING_ARCSEC {
            return Err(LilithCorpusError::CeilingExceeded {
                jd_tt: row.jd_tt,
                kind: "latitude_arcsec",
                got: our_lat,
                want: row.lat_deg,
                residual: resid_lat,
                ceiling: LAT_CEILING_ARCSEC,
            });
        }
        if resid_dist > DIST_CEILING_REL {
            return Err(LilithCorpusError::CeilingExceeded {
                jd_tt: row.jd_tt,
                kind: "distance_rel",
                got: our_dist,
                want: row.dist_au,
                residual: resid_dist,
                ceiling: DIST_CEILING_REL,
            });
        }

        max_lon = max_lon.max(resid_lon);
        max_lat = max_lat.max(resid_lat);
        max_dist = max_dist.max(resid_dist);
    }

    let summary_line = format!(
        "Lilith gate: {} rows vs Swiss Ephemeris SE_OSCU_APOG, max lon {:.3}\" lat {:.3}\" dist {:.2e} rel",
        rows.len(),
        max_lon,
        max_lat,
        max_dist
    );
    Ok(LilithCorpusReport {
        rows_validated: rows.len(),
        max_residual_lon_arcsec: max_lon,
        max_residual_lat_arcsec: max_lat,
        max_residual_dist_rel: max_dist,
        summary_line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lilith_gate_passes_within_ceilings() {
        let report = validate_lilith_corpus().expect("lilith gate passes");
        assert!(report.rows_validated > 0);
        // Print measured maxima so the ceilings can be tightened.
        eprintln!("{}", report.summary_line());
    }
}
```

- [ ] **Step 3: Register the module + re-export**

In `crates/pleiades-validate/src/lib.rs`, add the module declaration (with the other `mod`s) and re-export beside the ayanamsa one (lib.rs:169-170):
```rust
mod lilith_validation;
pub use lilith_validation::{validate_lilith_corpus, LilithCorpusError, LilithCorpusReport};
```

- [ ] **Step 4: Fill the manifest (the checksum/row dance)**

The gate fail-closes on the `PENDING` manifest. Get the real values by printing them once:
```bash
cargo test -p pleiades-validate lilith_gate_passes_within_ceilings -- --nocapture 2>&1 | tee /tmp/lilith.out
```
The first run fails with `ChecksumMismatch { got: <N>, want: ... }` and (after you set checksum) `ManifestDrift { rows_csv: <M>, ... }`. Set `crates/pleiades-validate/data/lilith-corpus/manifest.txt` to:
```
slice lilith file=lilith.csv role=lilith rows=<M> checksum=<N>
```
using the `got` checksum `<N>` and `rows_csv` `<M>` the errors report. Re-run until the gate reaches the assertion and prints the `Lilith gate: …` summary line with the measured maxima.

- [ ] **Step 5: Tighten the ceilings to the measured maxima**

From the printed summary line, set:
- `LON_CEILING_ARCSEC` = `ceil(max_lon_measured * 1.5)`
- `LAT_CEILING_ARCSEC` = `ceil(max_lat_measured * 1.5)`
- `DIST_CEILING_REL` = `max_dist_measured * 1.5`
Add a one-line comment above each constant recording the measured max and date. Re-run:
Run: `cargo test -p pleiades-validate lilith_gate_passes_within_ceilings -- --nocapture`
Expected: PASS, with all residuals comfortably under the tightened ceilings.

(If `max_lon` is implausibly large — many degrees — the μ constant or the SE flag is wrong: re-check `MU_EARTH_MOON_AU3_PER_DAY2` and that the SE corpus used of-date + nutation-on + `SE_OSCU_APOG`. This is the spec's "μ is the dominant parity knob" check.)

- [ ] **Step 6: Wire the CLI subcommand**

In `crates/pleiades-validate/src/render/cli.rs`, add an arm beside `validate-ayanamsa` (~cli.rs:190):
```rust
        Some("validate-lilith") | Some("lilith-gate") => {
            ensure_no_extra_args(&args[1..], "validate-lilith")?;
            crate::validate_lilith_corpus()
                .map(|report| report.summary_line().to_string())
                .map_err(|e| e.to_string())
        }
```
Add a help line next to the other gate help entries (~cli.rs:2044):
```rust
    "  validate-lilith        validate osculating true apsides vs Swiss Ephemeris SE_OSCU_APOG",
```
If `crates/pleiades-cli/src/cli.rs` enumerates the validate subcommands (it forwards to `validate_render_cli`), add `"validate-lilith" | "lilith-gate"` to whatever match decides forwarding (mirror the existing `validate-ayanamsa` handling there).

- [ ] **Step 7: Verify the CLI path**

Run: `cargo run -p pleiades-validate -- validate-lilith`
Expected: prints the `Lilith gate: … rows … max lon …` summary, exit 0.

- [ ] **Step 8: Wire into the release-gate aggregate**

Find where the full gate set is run together:
Run: `grep -rn "validate-ayanamsa\|release-gate\|validate_ayanamsa_corpus" crates/pleiades-validate/src crates/pleiades-cli/src`
Add a `validate_lilith_corpus()` call alongside the apparent/ayanamsa/topocentric/corpus calls in that aggregate (the `release-gate` subcommand or equivalent), so a Lilith regression fails the release gate. Mirror the existing entries exactly.

- [ ] **Step 9: Commit**

```bash
git add crates/pleiades-validate Cargo.toml crates/pleiades-cli/src/cli.rs
git commit -m "feat(validate): fail-closed validate-lilith gate vs SE_OSCU_APOG + release-gate wiring"
```

---

### Task 8: Promote claims in docs; mark the follow-up done

**Files:**
- Modify: `README.md`, `PLAN.md`
- Modify: `docs/lunar-theory-policy.md`
- Modify: `docs/superpowers/specs/notes/asteroid-calculated-points-readiness.md`
- Modify: `docs/follow-ups.md`
- Modify: `crates/pleiades-elp/src/lib.rs` and/or `crates/pleiades-elp/src/backend.rs` rustdoc (note the true apsides are now served release-grade by the packaged backend)

- [ ] **Step 1: README + PLAN**

In `README.md` and `PLAN.md`, find every "True Apogee/Perigee remain unsupported" phrasing and replace with a statement that the **osculating true apogee/perigee (True Lilith) are release-grade via the packaged-Moon-derived backend, gated against Swiss Ephemeris `SE_OSCU_APOG` by `validate-lilith`**. Update the `PLAN.md` status line's "still-deferred" lunar-point note accordingly.

- [ ] **Step 2: lunar-theory-policy.md**

In `docs/lunar-theory-policy.md`, move `true apogee` / `true perigee` from the unsupported list to a supported note, citing they are osculating (`SE_OSCU_APOG`) computed from the packaged Moon state in `pleiades-data`/`pleiades-apsides` (not the compact ELP theory), of-date via precession+nutation only.

- [ ] **Step 3: readiness note**

In `docs/superpowers/specs/notes/asteroid-calculated-points-readiness.md`, flip the two `GAP (unsupported)` rows (`TrueApogee`, `TruePerigee`) to `EXISTS`, citing `crates/pleiades-data/src/backend.rs` (osculating path) + `crates/pleiades-apsides` + the `validate-lilith` gate. Update the "Conclusion" paragraph that currently says these are the only genuine gap.

- [ ] **Step 4: follow-ups.md**

In `docs/follow-ups.md`, add a resolved entry recording the True (osculating) lunar apogee sub-project as done (date 2026-06-30, this branch), noting the equatorial/declination follow-up remains the next queued sub-project.

- [ ] **Step 5: ELP rustdoc**

In `crates/pleiades-elp/src/lib.rs` (the doc-comment at lines 12-13) and `crates/pleiades-elp/src/backend.rs` (the `elp_body_claims` doc at lines 26-28), keep ELP's own `unsupported` claim but add a sentence that the true apsides are now served release-grade by `PackagedDataBackend` ahead of ELP in the routing chain, so this is no longer a global gap. Do **not** change ELP's `unsupported` claim or its tests (`crates/pleiades-elp/src/tests.rs` still asserts ELP-local unsupported — that remains correct per-backend).

- [ ] **Step 6: Verify the whole workspace builds and tests green**

Run: `cargo fmt --all && cargo test --workspace`
Expected: PASS across the workspace.

- [ ] **Step 7: Run the publish/claims audit if present**

Run: `grep -rn "claims-audit\|claims_audit\|overclaim" crates/pleiades-cli/src crates/pleiades-validate/src | head`
If an audit subcommand exists, run it and confirm the new ReleaseGrade apsis claims pass (they are backed by the `validate-lilith` corpus). Fix any per-backend claim-count assertions that now need to include the two apsis bodies.

- [ ] **Step 8: Commit**

```bash
git add README.md PLAN.md docs crates/pleiades-elp
git commit -m "docs: promote true (osculating) lunar apsides to release-grade; mark follow-up done"
```

---

## Self-Review

**Spec coverage:**
- Release-grade `TrueApogee`/`TruePerigee` from packaged Moon → Tasks 1, 2, 4. ✓
- Isolated, unit-tested osculating math → Task 1. ✓
- Mean-J2000 geometric direction at backend; of-date via precession+nutation only → Tasks 3, 4, 5. ✓
- Both apsides; perigee = opposite apse → Task 1 (`apsides`), verified Tasks 4/5. ✓
- SE reference tool + committed corpus + fail-closed gate + release-gate wiring → Tasks 6, 7. ✓
- Empirical tolerance + SE-flag decision settled → Task 6 (MOSEPH), Task 7 Steps 4-5. ✓
- No `ARTIFACT_VERSION` bump → respected (derived at lookup); stated in Global Constraints. ✓
- Claims/docs/follow-ups/ELP note → Task 8. ✓

**Type consistency:** `apsides(pos_au, vel_au_per_day, mu) -> Result<Apsides, ApsidesError>` is produced in Task 1 and consumed identically in Task 4. `apparent_apsis_position(instant, EclipticCoordinates) -> Result<ApparentPosition, ApparentPlaceError>` produced in Task 3, consumed in Tasks 5 and 7. `validate_lilith_corpus() -> Result<LilithCorpusReport, LilithCorpusError>` produced in Task 7, wired in Task 7 Steps 6/8. `spherical_state_to_cartesian`/`SphericalState` field names match the recon. `BodyClaim::release_grade`, `ClaimEvidence::CorpusValidated { source }`, `BodyClaimTier::ReleaseGrade`, `AccuracyClass::High`, `QualityAnnotation::Interpolated`, `Motion::new`, `EclipticCoordinates::new` all match exact signatures.

**Known verification points flagged inline (not placeholders — real "confirm against source" steps):**
- Task 5 Step 1: the `BodyPlacement` accessor names (`placement`/`longitude`/`latitude`/`apparentness`) and the apparent chart-request helper must be matched to the real ones in `crates/pleiades-core/src/chart/{placement,snapshot,tests}.rs`.
- Task 6 Step 3: confirm `swe_calc` symbol/signature in `libswisseph-sys 0.1.2`.
- Task 7 Step 8: locate and extend the release-gate aggregate.
