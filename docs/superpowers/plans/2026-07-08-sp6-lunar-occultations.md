# SP-6 Lunar Occultations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Swiss-Ephemeris `swe_lun_occult_*` analogue to `pleiades-events` — local circumstances (`how`), per-observer search (`when_loc`), and global search (`when_glob`) for the Moon occulting a planet or a catalogued fixed star — validated to SE parity by a new fail-closed `validate-occultations` gate.

**Architecture:** Extend `EventEngine` in place (Approach A) with a new `occult` module, reusing the crate's `fixstar` (star apparent place), `semidiameter` (Moon + planet discs), `horizontal`/`root` machinery and `pleiades-apparent`'s `topocentric_position`/`refraction` — no new crate dependency. Contact geometry is a two-circle tangency root-find in equatorial RA/Dec (stars arrive as RA/Dec; the topocentric Moon rotates to RA/Dec), mirroring the SP-2c eclipse `local.rs` template. The global search is a geocentric Moon–target conjunction walk reporting the sub-lunar greatest-occultation point.

**Tech Stack:** Rust 2021 workspace; `pleiades-events`, `pleiades-apparent`, `pleiades-types`, `pleiades-backend`, `pleiades-data` (packaged backend for tests); `pleiades-validate` gate crate; a standalone Swiss-Ephemeris C harness under `tools/`.

## Global Constraints

- **Time base:** all instants are **TDB**; the supported window is **1900–2100 CE** (`WINDOW_START_JD = 2_415_020.5`, `WINDOW_END_JD = 2_488_069.5`). Out-of-window instants return `EventError::OutOfWindow`.
- **Fail-closed, never-NaN:** every `asin`/`acos` domain is `.clamp(-1.0, 1.0)`; missing backend reads return a structured `EventError`, never a panic or NaN.
- **Additive only:** new public surface on `EventEngine`, no rename. Compatibility profile bumps `0.7.11 → 0.7.12`; API-stability profile stays `0.2.2`.
- **Azimuth convention:** from SOUTH increasing WESTWARD, `[0,360)` degrees (matches `swe_azalt` / the crate's `Horizontal`).
- **Corpus discipline:** any committed corpus is fnv1a64-checksum-guarded and pinned by row count; SE `_ut` reference rows are converted to TDB once at generation.
- **`#![deny(missing_docs)]`** is active in `pleiades-events`: every new public item needs a doc comment.
- **Reference-value pins:** `R_MOON_KM = 1_737.4`, `AU_KM = 149_597_870.7`, Earth equatorial radius for parallax `R_EARTH_KM = 6_378.137`.

---

## Task 1: Occult public types + module scaffold

**Files:**
- Create: `crates/pleiades-events/src/occult.rs`
- Modify: `crates/pleiades-events/src/lib.rs:50-70` (add `mod occult;` and re-exports)

**Interfaces:**
- Consumes: `pleiades_types::{CelestialBody, Instant, Latitude, Longitude}`.
- Produces: `OccultTarget`, `OccultationType`, `OccultationContact`, `LocalOccultation`, `GlobalOccultation` — the value types every later task returns.

- [ ] **Step 1: Write the failing test**

Create `crates/pleiades-events/src/occult.rs` with only the types and this test module:

```rust
//! Lunar occultations — Swiss Ephemeris `swe_lun_occult_*` analogue: the Moon
//! occulting a planet (small disc) or a catalogued fixed star (point). Local
//! circumstances (`how`), per-observer search (`when_loc`), and global search
//! (`when_glob`). Geocentric/topocentric apparent-of-date, TDB, 1900–2100 CE.

use pleiades_types::{CelestialBody, Instant, Latitude, Longitude};

/// What the Moon is occulting.
#[derive(Clone, Debug, PartialEq)]
pub enum OccultTarget {
    /// A planet, Mercury..=Pluto. Sun and Moon are rejected (Sun ⇒ solar
    /// eclipse; the Moon is the occulter).
    Body(CelestialBody),
    /// A curated fixed-star catalog name (see [`crate::fixed_star_entry`]).
    Star(String),
}

/// What an observer (or the globe, for `when_glob`) actually sees.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OccultationType {
    /// Target fully covered at maximum (point star hidden, or planet disc fully
    /// behind the Moon's limb).
    Total,
    /// The Moon's limb crosses the target but never fully covers it.
    Grazing,
    /// No contact (topocentric/geocentric separation never small enough).
    Miss,
}

/// One observer-local contact event: its instant plus the target's horizontal
/// position and visibility there. A contact below the horizon is still timed
/// but flagged `visible == false`, matching SE.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OccultationContact {
    /// Instant of the contact (TDB).
    pub instant: Instant,
    /// Apparent (refracted) altitude of the target, degrees.
    pub altitude_degrees: f64,
    /// Azimuth from south increasing westward, `[0,360)` degrees.
    pub azimuth_degrees: f64,
    /// Whether the target is above the horizon at this instant.
    pub visible: bool,
}

/// Local circumstances of a lunar occultation of one target for one observer
/// (`how` / `when_loc`). Contact fields mirror the SP-2c eclipse C1–C4 layout.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalOccultation {
    /// The occulted target.
    pub target: OccultTarget,
    /// What THIS observer sees.
    pub occultation_type: OccultationType,
    /// Instant of least topocentric separation.
    pub maximum: OccultationContact,
    /// Covered fraction of the target's diameter at maximum (SE `attr[0]`).
    pub magnitude: f64,
    /// Covered fraction of the target's disc area at maximum (SE `attr[2]`);
    /// `0.0`/`1.0` for a point star.
    pub obscuration: f64,
    /// C1 — disappearance (exterior ingress).
    pub first_contact: OccultationContact,
    /// C2 — target fully hidden (planet disc only; `None` for a star / graze).
    pub second_contact: Option<OccultationContact>,
    /// C3 — target begins to reappear (planet disc only).
    pub third_contact: Option<OccultationContact>,
    /// C4 — reappearance (exterior egress).
    pub fourth_contact: OccultationContact,
    /// Whether the target is above the horizon during any part of the event.
    pub any_phase_visible: bool,
}

/// Global circumstances (`when_glob`): the greatest-occultation instant and the
/// sub-lunar point where it is central/greatest. NOT the full path polygon.
#[derive(Clone, Debug, PartialEq)]
pub struct GlobalOccultation {
    /// The occulted target.
    pub target: OccultTarget,
    /// Instant of greatest global occultation (TDB).
    pub maximum: Instant,
    /// Sub-lunar point of maximum occultation: geographic latitude, positive north.
    pub sublunar_latitude: Latitude,
    /// Sub-lunar point of maximum occultation: geographic longitude, positive east.
    pub sublunar_longitude: Longitude,
    /// Whether a central occultation exists somewhere on Earth.
    pub central: bool,
    /// Best-case type over the globe.
    pub occultation_type: OccultationType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{JulianDay, TimeScale};

    fn contact(jd: f64) -> OccultationContact {
        OccultationContact {
            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
            altitude_degrees: 42.0,
            azimuth_degrees: 100.0,
            visible: true,
        }
    }

    #[test]
    fn local_occultation_holds_optional_interior_contacts() {
        let star = LocalOccultation {
            target: OccultTarget::Star("Aldebaran".into()),
            occultation_type: OccultationType::Total,
            maximum: contact(2_451_545.0),
            magnitude: 1.0,
            obscuration: 1.0,
            first_contact: contact(2_451_544.98),
            second_contact: None,
            third_contact: None,
            fourth_contact: contact(2_451_545.02),
            any_phase_visible: true,
        };
        assert!(star.second_contact.is_none());
        assert_eq!(star.occultation_type, OccultationType::Total);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-events occult::tests 2>&1 | tail -20`
Expected: FAIL — `occult.rs` is not yet a module (`error[E0583]`/unresolved), or the test binary does not see it.

- [ ] **Step 3: Wire the module into `lib.rs`**

In `crates/pleiades-events/src/lib.rs`, add `mod occult;` in the module block (after `mod nod_aps;`) and the re-export line after the `pheno` export:

```rust
mod occult;
```
```rust
pub use occult::{
    GlobalOccultation, LocalOccultation, OccultTarget, OccultationContact, OccultationType,
};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-events occult::tests::local_occultation_holds_optional_interior_contacts 2>&1 | tail -20`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/occult.rs crates/pleiades-events/src/lib.rs
git commit -m "feat(events): SP-6 occultation public types + module scaffold"
```

---

## Task 2: Two-circle occultation geometry

**Files:**
- Modify: `crates/pleiades-events/src/occult.rs`

**Interfaces:**
- Produces: `OccGeom { sep_deg, s_moon_deg, s_tgt_deg }`, `covered_diameter_fraction(&OccGeom) -> f64`, `obscuration_fraction(&OccGeom) -> f64`, `classify(&OccGeom) -> OccultationType`, `angular_separation_deg(ra1,dec1,ra2,dec2) -> f64`. Pure functions consumed by Tasks 5–7.

The covered body is the **target** (radius `s_tgt`), occulted by the **Moon** (radius `s_moon`) — the mirror of the eclipse case where the Sun is covered by the Moon. For a point star `s_tgt == 0`: magnitude is binary (`1.0` hidden, `0.0` clear) and there is no interior contact.

- [ ] **Step 1: Write the failing test**

Add to `occult.rs`:

```rust
/// Instantaneous two-circle geometry: Moon vs target, all degrees.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OccGeom {
    /// Center-to-center Moon–target separation.
    pub sep_deg: f64,
    /// Moon's (topocentric) angular semidiameter.
    pub s_moon_deg: f64,
    /// Target's angular semidiameter (0 for a point star).
    pub s_tgt_deg: f64,
}

/// Great-circle separation (degrees) between two equatorial points (RA, Dec).
pub(crate) fn angular_separation_deg(ra1: f64, dec1: f64, ra2: f64, dec2: f64) -> f64 {
    let (a1, d1) = (ra1.to_radians(), dec1.to_radians());
    let (a2, d2) = (ra2.to_radians(), dec2.to_radians());
    let cos_sep = (d1.sin() * d2.sin() + d1.cos() * d2.cos() * (a1 - a2).cos()).clamp(-1.0, 1.0);
    cos_sep.acos().to_degrees()
}

/// Covered fraction of the TARGET's diameter (SE `attr[0]`). For a point star
/// (`s_tgt == 0`) this is binary: 1.0 when the target is inside the Moon's disc,
/// else 0.0. Clamped ≥ 0.
pub(crate) fn covered_diameter_fraction(g: &OccGeom) -> f64 {
    if g.s_tgt_deg <= 0.0 {
        return if g.sep_deg < g.s_moon_deg { 1.0 } else { 0.0 };
    }
    ((g.s_moon_deg + g.s_tgt_deg - g.sep_deg) / (2.0 * g.s_tgt_deg)).max(0.0)
}

/// Covered fraction of the TARGET's disc AREA (SE `attr[2]`). Standard
/// two-circle lens area; the covered disc is the target (radius `s_tgt`), the
/// covering disc is the Moon (radius `s_moon`). Binary for a point star.
pub(crate) fn obscuration_fraction(g: &OccGeom) -> f64 {
    let (r_t, r_m, d) = (g.s_tgt_deg, g.s_moon_deg, g.sep_deg);
    if r_t <= 0.0 {
        return if d < r_m { 1.0 } else { 0.0 };
    }
    if d >= r_t + r_m {
        return 0.0; // disjoint
    }
    if d <= r_m - r_t {
        return 1.0; // target fully behind the Moon
    }
    if d <= (r_t - r_m).max(0.0) {
        // Moon fully inside the target disc (target larger — impossible for real
        // planets vs Moon, but kept for closed-form completeness).
        return ((r_m / r_t).powi(2)).clamp(0.0, 1.0);
    }
    let r_t2 = r_t * r_t;
    let r_m2 = r_m * r_m;
    let a_t = ((d * d + r_t2 - r_m2) / (2.0 * d * r_t)).clamp(-1.0, 1.0).acos();
    let a_m = ((d * d + r_m2 - r_t2) / (2.0 * d * r_m)).clamp(-1.0, 1.0).acos();
    let lens = r_t2 * a_t + r_m2 * a_m
        - 0.5
            * ((r_t + r_m + d) * (-r_t + r_m + d) * (r_t - r_m + d) * (r_t + r_m - d))
                .max(0.0)
                .sqrt();
    (lens / (core::f64::consts::PI * r_t2)).clamp(0.0, 1.0)
}

/// Classify the occultation from the geometry at maximum: `Total` when the
/// target is fully covered (`sep < s_moon − s_tgt`), `Grazing` when the limb
/// crosses but never fully covers (`s_moon − s_tgt ≤ sep < s_moon + s_tgt`, and
/// for a point star the knife-edge `sep == s_moon`), else `Miss`.
pub(crate) fn classify(g: &OccGeom) -> OccultationType {
    let internal = (g.s_moon_deg - g.s_tgt_deg).max(0.0);
    let external = g.s_moon_deg + g.s_tgt_deg;
    if g.sep_deg < internal {
        OccultationType::Total
    } else if g.sep_deg < external {
        // A point star (s_tgt == 0) has internal == external == s_moon, so it is
        // never Grazing at this branch; a sep exactly at s_moon is the Miss edge.
        if g.s_tgt_deg <= 0.0 {
            OccultationType::Miss
        } else {
            OccultationType::Grazing
        }
    } else {
        OccultationType::Miss
    }
}

#[cfg(test)]
mod geom_tests {
    use super::*;

    fn g(sep: f64, s_moon: f64, s_tgt: f64) -> OccGeom {
        OccGeom { sep_deg: sep, s_moon_deg: s_moon, s_tgt_deg: s_tgt }
    }

    #[test]
    fn star_hidden_is_total_and_full_magnitude() {
        let geo = g(0.1, 0.25, 0.0); // sep < s_moon, point star
        assert_eq!(classify(&geo), OccultationType::Total);
        assert_eq!(covered_diameter_fraction(&geo), 1.0);
        assert_eq!(obscuration_fraction(&geo), 1.0);
    }

    #[test]
    fn star_clear_is_miss_zero_magnitude() {
        let geo = g(0.30, 0.25, 0.0); // sep > s_moon
        assert_eq!(classify(&geo), OccultationType::Miss);
        assert_eq!(covered_diameter_fraction(&geo), 0.0);
        assert_eq!(obscuration_fraction(&geo), 0.0);
    }

    #[test]
    fn planet_fully_behind_is_total() {
        let geo = g(0.20, 0.25, 0.003); // sep < s_moon - s_tgt
        assert_eq!(classify(&geo), OccultationType::Total);
        assert!(covered_diameter_fraction(&geo) >= 1.0);
        assert!((obscuration_fraction(&geo) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn planet_partly_covered_is_grazing_partial() {
        let geo = g(0.2495, 0.25, 0.003); // between internal and external
        assert_eq!(classify(&geo), OccultationType::Grazing);
        let o = obscuration_fraction(&geo);
        assert!(o > 0.0 && o < 1.0, "partial obscuration {o}");
    }

    #[test]
    fn separation_is_symmetric_and_zero_on_identity() {
        assert!(angular_separation_deg(10.0, 20.0, 10.0, 20.0) < 1e-9);
        let a = angular_separation_deg(10.0, 20.0, 12.0, 19.0);
        let b = angular_separation_deg(12.0, 19.0, 10.0, 20.0);
        assert!((a - b).abs() < 1e-12);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-events occult::geom_tests 2>&1 | tail -20`
Expected: FAIL — functions not yet defined (compile error) before you paste them; after pasting, all pass. (If it compiles and passes immediately, that is acceptable — the code and its tests are added together in this task.)

- [ ] **Step 3: (code already added in Step 1)**

No further implementation — Step 1 contains both the functions and their tests.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-events occult::geom_tests 2>&1 | tail -20`
Expected: PASS (5 tests)

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/occult.rs
git commit -m "feat(events): SP-6 two-circle occultation geometry + classification"
```

---

## Task 3: Moon + target sampler (topocentric & geocentric RA/Dec + semidiameters)

**Files:**
- Modify: `crates/pleiades-events/src/occult.rs`
- Reference (read, do not modify): `crates/pleiades-events/src/rise_trans.rs:167-221` (`target_equatorial` pattern), `crates/pleiades-events/src/semidiameter.rs`, `crates/pleiades-eclipse/src/local.rs:109-195` (UT1 parallax rotation).

**Interfaces:**
- Consumes: `OccGeom` (Task 2); `EventEngine<B>` (`crossings.rs`), `geocentric_apparent_ecliptic`/`read_mean_ecliptic` (`ephemeris.rs`), `fixed_star_apparent` (`fixstar.rs`), `semidiameter_deg` (`semidiameter.rs`), `pleiades_apparent::{topocentric_position, sidereal_time, true_obliquity_degrees}`, `RiseSetTarget` (`rise_trans.rs`).
- Produces: `EventEngine::occ_geom(&OccultTarget, Option<&ObserverLocation>, jd) -> Result<OccGeom, EventError>` (topocentric when `Some(observer)`, geocentric when `None`); `EventEngine::moon_target_radec(&OccultTarget, Option<&ObserverLocation>, jd) -> Result<((f64,f64),(f64,f64)), EventError>` returning `((moon_ra, moon_dec),(tgt_ra, tgt_dec))` degrees. Consumed by Tasks 5–7.

- [ ] **Step 1: Write the failing test**

Add to `occult.rs`:

```rust
use crate::crossings::EventEngine;
use crate::ephemeris::{geocentric_apparent_ecliptic, read_mean_ecliptic};
use crate::error::EventError;
use crate::fixstar::fixed_star_apparent;
use crate::rise_trans::RiseSetTarget;
use crate::semidiameter::semidiameter_deg;
use pleiades_apparent::{sidereal_time, topocentric_position, true_obliquity_degrees};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{
    Angle, CelestialBody, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude,
    ObserverLocation, TimeScale,
};

/// Moon physical radius (km) and AU, matching the eclipse crate's `solar_consts`.
pub(crate) const R_MOON_KM: f64 = 1_737.4;
pub(crate) const AU_KM: f64 = 149_597_870.7;

impl<B: EphemerisBackend> EventEngine<B> {
    /// Apparent equatorial-of-date RA/Dec (degrees) of the Moon and the target at
    /// `jd`. When `observer` is `Some`, the Moon (and a planet target) carry
    /// diurnal parallax via `topocentric_position`; a `Star` target has no
    /// parallax. When `observer` is `None`, both are geocentric.
    pub(crate) fn moon_target_radec(
        &self,
        target: &OccultTarget,
        observer: Option<&ObserverLocation>,
        jd: f64,
    ) -> Result<((f64, f64), (f64, f64)), EventError> {
        let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        let eps = true_obliquity_degrees(jd)
            .map_err(|e| EventError::Backend(format!("obliquity failed: {e}")))?;
        // UT1-rotated sidereal time for parallax (matches eclipse local.rs).
        let sid_jd = pleiades_time::ut1_jd_from_tt(jd).unwrap_or(jd);
        let sid_at = Instant::new(JulianDay::from_days(sid_jd), TimeScale::Tdb);
        let lst = sidereal_time(sid_at, Longitude::from_degrees(0.0)).local_apparent_deg;

        let body_radec = |b: &CelestialBody| -> Result<(f64, f64), EventError> {
            let (lon, lat, dist) =
                geocentric_apparent_ecliptic(&self.backend, b.clone(), "body", jd)?;
            let ecl = EclipticCoordinates::new(
                Longitude::from_degrees(lon),
                Latitude::from_degrees(lat),
                Some(dist),
            );
            let ecl = if let Some(obs) = observer {
                // Parallax needs the observer's local sidereal time.
                let l = sidereal_time(sid_at, obs.longitude).local_apparent_deg;
                topocentric_position(ecl, obs, l, eps)
                    .map_err(|e| EventError::Backend(format!("topocentric failed: {e}")))?
                    .ecliptic
            } else {
                ecl
            };
            let equ = ecl.to_equatorial(Angle::from_degrees(eps));
            Ok((equ.right_ascension.degrees(), equ.declination.degrees()))
        };
        let _ = lst; // observer-specific lst computed inside the closure

        let moon = body_radec(&CelestialBody::Moon)?;
        let tgt = match target {
            OccultTarget::Body(b) => body_radec(b)?,
            OccultTarget::Star(name) => {
                let equ = fixed_star_apparent(name, at)?;
                (equ.right_ascension.degrees(), equ.declination.degrees())
            }
        };
        Ok((moon, tgt))
    }

    /// Two-circle geometry (separation + semidiameters) of the Moon vs target at
    /// `jd`, topocentric when `observer` is `Some`.
    pub(crate) fn occ_geom(
        &self,
        target: &OccultTarget,
        observer: Option<&ObserverLocation>,
        jd: f64,
    ) -> Result<OccGeom, EventError> {
        let (moon, tgt) = self.moon_target_radec(target, observer, jd)?;
        let sep_deg = angular_separation_deg(moon.0, moon.1, tgt.0, tgt.1);
        // Moon semidiameter from its topocentric/geocentric distance.
        let moon_dist = self.body_distance_au(&CelestialBody::Moon, observer, jd)?;
        let s_moon_deg = (R_MOON_KM / (moon_dist * AU_KM)).clamp(-1.0, 1.0).asin().to_degrees();
        let s_tgt_deg = match target {
            OccultTarget::Body(b) => {
                let dist = self.body_distance_au(b, observer, jd)?;
                semidiameter_deg(&RiseSetTarget::Body(b.clone()), dist, false)
            }
            OccultTarget::Star(_) => 0.0,
        };
        Ok(OccGeom { sep_deg, s_moon_deg, s_tgt_deg })
    }

    /// Topocentric (if `observer`) or geocentric distance (AU) of a body.
    fn body_distance_au(
        &self,
        b: &CelestialBody,
        observer: Option<&ObserverLocation>,
        jd: f64,
    ) -> Result<f64, EventError> {
        let (lon, lat, dist) = geocentric_apparent_ecliptic(&self.backend, b.clone(), "body", jd)?;
        if let Some(obs) = observer {
            let eps = true_obliquity_degrees(jd)
                .map_err(|e| EventError::Backend(format!("obliquity failed: {e}")))?;
            let sid_jd = pleiades_time::ut1_jd_from_tt(jd).unwrap_or(jd);
            let sid_at = Instant::new(JulianDay::from_days(sid_jd), TimeScale::Tdb);
            let l = sidereal_time(sid_at, obs.longitude).local_apparent_deg;
            let ecl = EclipticCoordinates::new(
                Longitude::from_degrees(lon),
                Latitude::from_degrees(lat),
                Some(dist),
            );
            let topo = topocentric_position(ecl, obs, l, eps)
                .map_err(|e| EventError::Backend(format!("topocentric failed: {e}")))?;
            Ok(topo.ecliptic.distance_au.unwrap_or(dist))
        } else {
            Ok(dist)
        }
    }
}

#[cfg(test)]
mod sampler_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    fn obs() -> ObserverLocation {
        ObserverLocation::new(Latitude::from_degrees(0.0), Longitude::from_degrees(0.0), Some(0.0))
    }

    #[test]
    fn moon_semidiameter_is_about_a_quarter_degree() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let g = engine
            .occ_geom(&OccultTarget::Body(CelestialBody::Mercury), None, 2_451_550.0)
            .unwrap();
        assert!(g.s_moon_deg > 0.2 && g.s_moon_deg < 0.3, "moon SD {}", g.s_moon_deg);
        assert!(g.sep_deg.is_finite());
    }

    #[test]
    fn topocentric_separation_differs_from_geocentric() {
        // Diurnal parallax shifts the Moon, so the Moon–planet separation seen
        // topocentrically differs from the geocentric one.
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let geo = engine
            .occ_geom(&OccultTarget::Body(CelestialBody::Sun), None, 2_451_550.0)
            .unwrap();
        let topo = engine
            .occ_geom(&OccultTarget::Body(CelestialBody::Sun), Some(&obs()), 2_451_550.0)
            .unwrap();
        assert!((geo.sep_deg - topo.sep_deg).abs() > 0.0);
    }
}
```

- [ ] **Step 2: Run test to verify it fails, then passes**

Run: `cargo test -p pleiades-events occult::sampler_tests 2>&1 | tail -20`
Expected: after pasting Step 1, PASS (2 tests). If a compile error mentions `to_equatorial`/`topocentric_position` arg types, cross-check against `rise_trans.rs:206-218`.

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-events/src/occult.rs
git commit -m "feat(events): SP-6 Moon/target RA-Dec + semidiameter sampler"
```

---

## Task 4: Target resolution + un-occultable-star fast reject

**Files:**
- Modify: `crates/pleiades-events/src/error.rs:47-58` (add variant), `crates/pleiades-events/src/error.rs:60-90` (Display arm)
- Modify: `crates/pleiades-events/src/occult.rs`

**Interfaces:**
- Consumes: `EventError`, `fixed_star_entry` (`fixstar.rs`), `EquatorialCoordinates::to_ecliptic` (`pleiades-types`), `fixed_star_apparent`.
- Produces: `EventError::UnsupportedOccultTarget { detail }`; `EventEngine::validate_occult_target(&OccultTarget) -> Result<(), EventError>`; `EventEngine::target_never_occultable(&OccultTarget, jd) -> Result<bool, EventError>` (the fast-reject predicate consumed by Tasks 6–7). Constant `MOON_MAX_REACH_DEG = 6.6`.

- [ ] **Step 1: Add the error variant + Display arm**

In `crates/pleiades-events/src/error.rs`, add to the `EventError` enum (after `DegenerateNodAps`):

```rust
    /// A body that cannot be occulted this way (Sun or Moon as the target).
    UnsupportedOccultTarget {
        /// Human-readable explanation.
        detail: String,
    },
```

and to the `Display` match:

```rust
            EventError::UnsupportedOccultTarget { detail } => {
                write!(f, "unsupported occultation target: {detail}")
            }
```

- [ ] **Step 2: Write the failing test**

Add to `occult.rs`:

```rust
use pleiades_types::OBLIQUITY_J2000_DEG;

/// The Moon's maximum reach in ecliptic latitude: ~5.3° orbital inclination +
/// ~0.27° semidiameter + ~0.95° horizontal parallax ≈ 6.6°. A star beyond this
/// |ecliptic latitude| can never be occulted from anywhere on Earth.
pub(crate) const MOON_MAX_REACH_DEG: f64 = 6.6;

impl<B: EphemerisBackend> EventEngine<B> {
    /// Rejects Sun/Moon as an occultation target and unknown star names.
    pub(crate) fn validate_occult_target(&self, target: &OccultTarget) -> Result<(), EventError> {
        match target {
            OccultTarget::Body(CelestialBody::Sun) => Err(EventError::UnsupportedOccultTarget {
                detail: "the Sun occulted by the Moon is a solar eclipse; use the eclipse engine"
                    .into(),
            }),
            OccultTarget::Body(CelestialBody::Moon) => Err(EventError::UnsupportedOccultTarget {
                detail: "the Moon is the occulter, not a target".into(),
            }),
            OccultTarget::Body(_) => Ok(()),
            OccultTarget::Star(name) => crate::fixstar::fixed_star_entry(name).map(|_| ()),
        }
    }

    /// Whether the target's ecliptic latitude puts it permanently outside the
    /// Moon's reach (only meaningful — and only applied — for stars, whose
    /// latitude is effectively constant; planets always return `false`).
    pub(crate) fn target_never_occultable(
        &self,
        target: &OccultTarget,
        jd: f64,
    ) -> Result<bool, EventError> {
        match target {
            OccultTarget::Star(name) => {
                let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
                let equ = fixed_star_apparent(name, at)?;
                let eps = true_obliquity_degrees(jd).unwrap_or(OBLIQUITY_J2000_DEG);
                let ecl = equ.to_ecliptic(Angle::from_degrees(eps));
                Ok(ecl.latitude.degrees().abs() > MOON_MAX_REACH_DEG)
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod target_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn sun_and_moon_targets_are_rejected() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        assert!(matches!(
            engine.validate_occult_target(&OccultTarget::Body(CelestialBody::Sun)),
            Err(EventError::UnsupportedOccultTarget { .. })
        ));
        assert!(matches!(
            engine.validate_occult_target(&OccultTarget::Body(CelestialBody::Moon)),
            Err(EventError::UnsupportedOccultTarget { .. })
        ));
        assert!(engine
            .validate_occult_target(&OccultTarget::Body(CelestialBody::Venus))
            .is_ok());
    }

    #[test]
    fn unknown_star_is_rejected() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        assert!(matches!(
            engine.validate_occult_target(&OccultTarget::Star("Nope".into())),
            Err(EventError::UnknownFixedStar { .. })
        ));
    }

    #[test]
    fn far_off_ecliptic_star_is_never_occultable() {
        // Sirius sits ~39° below the ecliptic; the Moon can never reach it.
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        assert!(engine
            .target_never_occultable(&OccultTarget::Star("Sirius".into()), 2_451_545.0)
            .unwrap());
        // Aldebaran (~5.5°S) is within reach.
        assert!(!engine
            .target_never_occultable(&OccultTarget::Star("Aldebaran".into()), 2_451_545.0)
            .unwrap());
    }
}
```

Note: confirm `Sirius` and `Aldebaran` are both present in `crates/pleiades-events/data/fixstars-catalog.csv` before running (they are used by existing rise-trans tests, so they are). If `Sirius` is absent, substitute any catalog star with |ecliptic latitude| > 6.6°.

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test -p pleiades-events occult::target_tests 2>&1 | tail -20`
Expected: PASS (3 tests)

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-events/src/error.rs crates/pleiades-events/src/occult.rs
git commit -m "feat(events): SP-6 occult target resolution + un-occultable-star reject"
```

---

## Task 5: `occultation` (how) — local circumstances

**Files:**
- Modify: `crates/pleiades-events/src/occult.rs`
- Reference: `crates/pleiades-eclipse/src/local.rs:388-529` (contacts), `:566-651` (horizontal + classify).

**Interfaces:**
- Consumes: `occ_geom`, `moon_target_radec`, `validate_occult_target` (Tasks 3–4); `root::REFINE_TOLERANCE_DAYS`; `pleiades_apparent::{apparent_from_true, Atmosphere}`; `check_atmosphere` (`rise_trans.rs`).
- Produces: `EventEngine::occultation(target, observer, atmosphere, at) -> Result<LocalOccultation, EventError>`. Consumed by Task 6 and the gate.

Constants: `OCC_CONTACT_HALF_WINDOW_DAYS = 0.15` (a lunar occultation's ingress-to-egress never exceeds ~1.5 h even for a grazing chord; 0.15 day = 3.6 h is a safe superset).

- [ ] **Step 1: Write the failing test + implementation together**

Add to `occult.rs` (helpers + public method):

```rust
use crate::rise_trans::check_atmosphere;
use crate::root::REFINE_TOLERANCE_DAYS;
use pleiades_apparent::{apparent_from_true, Atmosphere};

/// Half-window (days) to bracket occultation contacts around the maximum.
const OCC_CONTACT_HALF_WINDOW_DAYS: f64 = 0.15;

impl<B: EphemerisBackend> EventEngine<B> {
    /// Topocentric az/alt + visibility of the target at `jd` for `observer`.
    fn target_horizontal(
        &self,
        target: &OccultTarget,
        observer: &ObserverLocation,
        atmos: Atmosphere,
        jd: f64,
    ) -> Result<(f64, f64, bool), EventError> {
        let (_, (ra_deg, dec_deg)) = self.moon_target_radec(target, Some(observer), jd)?;
        let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        let lst = sidereal_time(at, observer.longitude).local_apparent_deg;
        let ha = (lst - ra_deg).to_radians();
        let dec = dec_deg.to_radians();
        let phi = observer.latitude.degrees().to_radians();
        let sin_alt = (phi.sin() * dec.sin() + phi.cos() * dec.cos() * ha.cos()).clamp(-1.0, 1.0);
        let true_alt = sin_alt.asin().to_degrees();
        let az = ha.sin().atan2(ha.cos() * phi.sin() - dec.tan() * phi.cos());
        let app_alt = apparent_from_true(true_alt, atmos);
        Ok((az.to_degrees().rem_euclid(360.0), app_alt, app_alt > 0.0))
    }

    fn occ_contact_at(
        &self,
        target: &OccultTarget,
        observer: &ObserverLocation,
        atmos: Atmosphere,
        jd: f64,
    ) -> Result<OccultationContact, EventError> {
        let (az, alt, visible) = self.target_horizontal(target, observer, atmos, jd)?;
        Ok(OccultationContact {
            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
            altitude_degrees: alt,
            azimuth_degrees: az,
            visible,
        })
    }

    /// Golden-section minimize the topocentric separation in `[a,b]`.
    fn minimize_occ_sep(
        &self,
        target: &OccultTarget,
        observer: &ObserverLocation,
        mut a: f64,
        mut b: f64,
    ) -> Result<f64, EventError> {
        let phi = 0.618_033_988_75_f64;
        let sep = |jd: f64| Ok::<f64, EventError>(self.occ_geom(target, Some(observer), jd)?.sep_deg);
        let mut c = b - (b - a) * phi;
        let mut d = a + (b - a) * phi;
        let (mut fc, mut fd) = (sep(c)?, sep(d)?);
        while (b - a) > REFINE_TOLERANCE_DAYS {
            if fc < fd {
                b = d; d = c; fd = fc; c = b - (b - a) * phi; fc = sep(c)?;
            } else {
                a = c; c = d; fc = fd; d = a + (b - a) * phi; fd = sep(d)?;
            }
        }
        Ok(0.5 * (a + b))
    }

    /// Bisect `sep(t) − threshold` between `lo` and `hi`; `None` if no sign change.
    fn bisect_occ_contact(
        &self,
        target: &OccultTarget,
        observer: &ObserverLocation,
        threshold: f64,
        mut lo: f64,
        mut hi: f64,
    ) -> Result<Option<f64>, EventError> {
        let f = |jd: f64| Ok::<f64, EventError>(self.occ_geom(target, Some(observer), jd)?.sep_deg - threshold);
        let mut flo = f(lo)?;
        let fhi = f(hi)?;
        if flo.signum() == fhi.signum() {
            return Ok(None);
        }
        while (hi - lo) > REFINE_TOLERANCE_DAYS {
            let mid = 0.5 * (lo + hi);
            let fmid = f(mid)?;
            if fmid.signum() == flo.signum() { lo = mid; flo = fmid; } else { hi = mid; }
        }
        Ok(Some(0.5 * (lo + hi)))
    }

    /// Local circumstances of a lunar occultation of `target` for `observer` at
    /// (or around) `at` — Swiss Ephemeris `swe_lun_occult_how` analogue. Returns
    /// full circumstances even when the target is below the horizon; the
    /// `occultation_type` is `Miss` when no contact occurs for this observer.
    pub fn occultation(
        &self,
        target: OccultTarget,
        observer: ObserverLocation,
        atmosphere: Atmosphere,
        at: Instant,
    ) -> Result<LocalOccultation, EventError> {
        self.validate_occult_target(&target)?;
        observer.validate().map_err(|e| EventError::InvalidObserver { detail: e.to_string() })?;
        check_atmosphere(atmosphere)?;
        let jd0 = at.julian_day.days();
        self.check_window(jd0)?;

        let max_jd = self.minimize_occ_sep(
            &target,
            &observer,
            (jd0 - OCC_CONTACT_HALF_WINDOW_DAYS).max(WINDOW_START_JD),
            (jd0 + OCC_CONTACT_HALF_WINDOW_DAYS).min(WINDOW_END_JD),
        )?;
        let g = self.occ_geom(&target, Some(&observer), max_jd)?;
        let occ_type = classify(&g);
        let maximum = self.occ_contact_at(&target, &observer, atmosphere, max_jd)?;

        if matches!(occ_type, OccultationType::Miss) {
            // A timed "no occultation here" record: all contacts at the closest
            // approach, magnitude 0, not visible as an event.
            return Ok(LocalOccultation {
                target,
                occultation_type: OccultationType::Miss,
                maximum,
                magnitude: 0.0,
                obscuration: 0.0,
                first_contact: maximum,
                second_contact: None,
                third_contact: None,
                fourth_contact: maximum,
                any_phase_visible: false,
            });
        }

        let external = g.s_moon_deg + g.s_tgt_deg;
        let internal = (g.s_moon_deg - g.s_tgt_deg).max(0.0);
        let lo = (max_jd - OCC_CONTACT_HALF_WINDOW_DAYS).max(WINDOW_START_JD);
        let hi = (max_jd + OCC_CONTACT_HALF_WINDOW_DAYS).min(WINDOW_END_JD);
        let c1 = self.bisect_occ_contact(&target, &observer, external, lo, max_jd)?.unwrap_or(max_jd);
        let c4 = self.bisect_occ_contact(&target, &observer, external, max_jd, hi)?.unwrap_or(max_jd);

        let disc_total = g.s_tgt_deg > 0.0 && g.sep_deg < internal;
        let (c2, c3) = if disc_total {
            (
                self.bisect_occ_contact(&target, &observer, internal, c1, max_jd)?,
                self.bisect_occ_contact(&target, &observer, internal, max_jd, c4)?,
            )
        } else {
            (None, None)
        };

        let first_contact = self.occ_contact_at(&target, &observer, atmosphere, c1)?;
        let fourth_contact = self.occ_contact_at(&target, &observer, atmosphere, c4)?;
        let second_contact = match c2 { Some(jd) => Some(self.occ_contact_at(&target, &observer, atmosphere, jd)?), None => None };
        let third_contact = match c3 { Some(jd) => Some(self.occ_contact_at(&target, &observer, atmosphere, jd)?), None => None };

        // Any phase visible: coarse 30-second scan of the target's altitude over [c1,c4].
        let any_phase_visible = {
            let step = 30.0 / 86_400.0;
            let mut jd = c1;
            let mut vis = false;
            while jd <= c4 + 1e-12 {
                if self.target_horizontal(&target, &observer, atmosphere, jd)?.2 { vis = true; break; }
                jd += step;
            }
            vis
        };

        Ok(LocalOccultation {
            target,
            occultation_type: occ_type,
            maximum,
            magnitude: covered_diameter_fraction(&g),
            obscuration: obscuration_fraction(&g),
            first_contact,
            second_contact,
            third_contact,
            fourth_contact,
            any_phase_visible,
        })
    }
}
```

Add the window imports at the top of `occult.rs` if not already present: `use crate::error::{WINDOW_END_JD, WINDOW_START_JD};` (extend the existing `use crate::error::...` line).

Test:

```rust
#[cfg(test)]
mod how_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    fn equatorial_obs() -> ObserverLocation {
        ObserverLocation::new(Latitude::from_degrees(0.0), Longitude::from_degrees(0.0), Some(0.0))
    }

    #[test]
    fn contacts_bracket_the_maximum_when_occulted() {
        // On-node new moon → the analytic Moon passes over a target near the
        // ecliptic for an equatorial observer.
        let backend = LinearSunMoon::new_moon_at(2_451_550.0).with_moon_latitude(0.0);
        let engine = EventEngine::new(backend);
        let at = Instant::new(JulianDay::from_days(2_451_550.0), TimeScale::Tdb);
        // The Sun sits at the new-moon longitude; use it as a stand-in disc target
        // only to exercise ordering (validate_occult_target forbids Sun, so use a
        // planet the mock serves). The LinearSunMoon mock serves Sun/Moon only, so
        // assert ordering via a Miss-free path is covered in integration Task 8.
        let out = engine.occultation(
            OccultTarget::Body(CelestialBody::Mercury),
            equatorial_obs(),
            Atmosphere::default(),
            at,
        );
        // The mock has no Mercury → fail-closed MissingCoordinates/Backend, proving
        // the resolve path reaches the backend. Real-backend ordering is Task 8.
        assert!(out.is_err());
    }

    #[test]
    fn out_of_window_and_sun_target_fail_closed() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let bad = Instant::new(JulianDay::from_days(2_000_000.0), TimeScale::Tdb);
        assert!(matches!(
            engine.occultation(OccultTarget::Body(CelestialBody::Mars), equatorial_obs(), Atmosphere::default(), bad),
            Err(EventError::OutOfWindow { .. })
        ));
        let ok_at = Instant::new(JulianDay::from_days(2_451_550.0), TimeScale::Tdb);
        assert!(matches!(
            engine.occultation(OccultTarget::Body(CelestialBody::Sun), equatorial_obs(), Atmosphere::default(), ok_at),
            Err(EventError::UnsupportedOccultTarget { .. })
        ));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p pleiades-events occult::how_tests 2>&1 | tail -20`
Expected: PASS (2 tests). The `LinearSunMoon` mock only serves Sun/Moon, so real-geometry ordering is asserted against the packaged backend in Task 8; these tests pin the fail-closed paths.

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-events/src/occult.rs
git commit -m "feat(events): SP-6 occultation how (local circumstances)"
```

---

## Task 6: `next_occultation` / `previous_occultation` (when_loc)

**Files:**
- Modify: `crates/pleiades-events/src/occult.rs`

**Interfaces:**
- Consumes: `occultation` (Task 5), `target_never_occultable`/`validate_occult_target` (Task 4), `root::{first_crossing_after, last_crossing_before, wrap180}`, `geocentric_apparent_longitude_deg` (`ephemeris.rs`).
- Produces: `EventEngine::next_occultation(...) -> Result<Option<LocalOccultation>, EventError>` and `previous_occultation(...)`. Consumed by the gate.

Strategy: root-find the Moon–target apparent-longitude conjunction (`wrap180(moon_lon − tgt_lon) == 0`) with `first_crossing_after` (step 0.25 day). At each conjunction run `occultation`; return the first with `any_phase_visible`. Fast-reject un-occultable stars up front; bound the scan to the window end.

- [ ] **Step 1: Write the failing test + implementation**

Add to `occult.rs`:

```rust
use crate::ephemeris::geocentric_apparent_longitude_deg;
use crate::root::{first_crossing_after, last_crossing_before, wrap180};

/// Conjunction bracketing step (Moon moves ~0.5°/h; 0.25 day is the crate's
/// Moon step and cannot skip a monthly conjunction).
const OCC_CONJUNCTION_STEP_DAYS: f64 = 0.25;

impl<B: EphemerisBackend> EventEngine<B> {
    /// Signed Moon−target apparent ecliptic longitude difference, wrapped.
    fn moon_target_lon_diff(&self, target: &OccultTarget, jd: f64) -> Result<f64, EventError> {
        let moon = geocentric_apparent_longitude_deg(&self.backend, CelestialBody::Moon, "Moon", jd)?;
        let tgt = match target {
            OccultTarget::Body(b) => geocentric_apparent_longitude_deg(&self.backend, b.clone(), "body", jd)?,
            OccultTarget::Star(name) => {
                let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
                let equ = fixed_star_apparent(name, at)?;
                let eps = true_obliquity_degrees(jd)
                    .map_err(|e| EventError::Backend(format!("obliquity failed: {e}")))?;
                equ.to_ecliptic(Angle::from_degrees(eps)).longitude.degrees()
            }
        };
        Ok(wrap180(moon - tgt))
    }

    /// Next occultation of `target` locally visible at `observer`, strictly after
    /// `after` — `swe_lun_occult_when_loc` analogue. `None` if none occurs before
    /// the window end (or ever, for an un-occultable star).
    pub fn next_occultation(
        &self,
        target: OccultTarget,
        observer: ObserverLocation,
        atmosphere: Atmosphere,
        after: Instant,
    ) -> Result<Option<LocalOccultation>, EventError> {
        self.validate_occult_target(&target)?;
        observer.validate().map_err(|e| EventError::InvalidObserver { detail: e.to_string() })?;
        check_atmosphere(atmosphere)?;
        let after_jd = after.julian_day.days();
        self.check_window(after_jd)?;
        if self.target_never_occultable(&target, after_jd)? {
            return Ok(None);
        }
        let mut scan_start = after_jd.max(WINDOW_START_JD + OCC_CONJUNCTION_STEP_DAYS);
        let scan_end = WINDOW_END_JD - OCC_CONJUNCTION_STEP_DAYS;
        loop {
            let conj = first_crossing_after(
                |jd| self.moon_target_lon_diff(&target, jd),
                scan_start,
                scan_end,
                OCC_CONJUNCTION_STEP_DAYS,
            )?;
            let Some(conj_jd) = conj else { return Ok(None) };
            let at = Instant::new(JulianDay::from_days(conj_jd), TimeScale::Tdb);
            let local = self.occultation(target.clone(), observer.clone(), atmosphere, at)?;
            if !matches!(local.occultation_type, OccultationType::Miss)
                && local.any_phase_visible
                && local.maximum.instant.julian_day.days() > after_jd
            {
                return Ok(Some(local));
            }
            // Advance just past this conjunction to find the next one.
            scan_start = conj_jd + OCC_CONJUNCTION_STEP_DAYS;
            if scan_start >= scan_end {
                return Ok(None);
            }
        }
    }

    /// Previous occultation of `target` locally visible at `observer`, strictly
    /// before `before`.
    pub fn previous_occultation(
        &self,
        target: OccultTarget,
        observer: ObserverLocation,
        atmosphere: Atmosphere,
        before: Instant,
    ) -> Result<Option<LocalOccultation>, EventError> {
        self.validate_occult_target(&target)?;
        observer.validate().map_err(|e| EventError::InvalidObserver { detail: e.to_string() })?;
        check_atmosphere(atmosphere)?;
        let before_jd = before.julian_day.days();
        self.check_window(before_jd)?;
        if self.target_never_occultable(&target, before_jd)? {
            return Ok(None);
        }
        let mut scan_end = before_jd.min(WINDOW_END_JD - OCC_CONJUNCTION_STEP_DAYS);
        let scan_start = WINDOW_START_JD + OCC_CONJUNCTION_STEP_DAYS;
        loop {
            let conj = last_crossing_before(
                |jd| self.moon_target_lon_diff(&target, jd),
                scan_start,
                scan_end,
                OCC_CONJUNCTION_STEP_DAYS,
            )?;
            let Some(conj_jd) = conj else { return Ok(None) };
            let at = Instant::new(JulianDay::from_days(conj_jd), TimeScale::Tdb);
            let local = self.occultation(target.clone(), observer.clone(), atmosphere, at)?;
            if !matches!(local.occultation_type, OccultationType::Miss)
                && local.any_phase_visible
                && local.maximum.instant.julian_day.days() < before_jd
            {
                return Ok(Some(local));
            }
            scan_end = conj_jd - OCC_CONJUNCTION_STEP_DAYS;
            if scan_end <= scan_start {
                return Ok(None);
            }
        }
    }
}

#[cfg(test)]
mod when_loc_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn never_occultable_star_returns_none_fast() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let obs = ObserverLocation::new(Latitude::from_degrees(0.0), Longitude::from_degrees(0.0), Some(0.0));
        let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
        let out = engine
            .next_occultation(OccultTarget::Star("Sirius".into()), obs, Atmosphere::default(), after)
            .unwrap();
        assert!(out.is_none(), "Sirius can never be occulted");
    }

    #[test]
    fn out_of_window_fails_closed() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let obs = ObserverLocation::new(Latitude::from_degrees(0.0), Longitude::from_degrees(0.0), Some(0.0));
        let bad = Instant::new(JulianDay::from_days(2_000_000.0), TimeScale::Tdb);
        assert!(matches!(
            engine.next_occultation(OccultTarget::Body(CelestialBody::Venus), obs, Atmosphere::default(), bad),
            Err(EventError::OutOfWindow { .. })
        ));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p pleiades-events occult::when_loc_tests 2>&1 | tail -20`
Expected: PASS (2 tests). Real-conjunction location against the packaged backend is exercised in Task 8.

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-events/src/occult.rs
git commit -m "feat(events): SP-6 next/previous_occultation (when_loc)"
```

---

## Task 7: `next_global_occultation` (when_glob) + sub-lunar point

**Files:**
- Modify: `crates/pleiades-events/src/occult.rs`
- Reference: `crates/pleiades-eclipse/src/geometry.rs:420-435` (geographic longitude = RA − GMST).

**Interfaces:**
- Consumes: `occ_geom`/`moon_target_radec` (geocentric branch), `moon_target_lon_diff` (Task 6), `body_distance_au`.
- Produces: `EventEngine::next_global_occultation(target, after) -> Result<Option<GlobalOccultation>, EventError>`.

An occultation is visible somewhere on Earth iff the geocentric minimum separation `< s_moon + s_tgt + π_moon`, with `π_moon = asin(R_EARTH_KM / (moon_dist_au·AU_KM))`. The sub-lunar point is the Moon's geocentric Dec (latitude) and `RA − GAST` (longitude).

- [ ] **Step 1: Write the failing test + implementation**

Add to `occult.rs`:

```rust
/// Earth equatorial radius (km) for the Moon's horizontal parallax.
const R_EARTH_KM: f64 = 6_378.137;

impl<B: EphemerisBackend> EventEngine<B> {
    /// Next occultation of `target` anywhere on Earth, strictly after `after` —
    /// `swe_lun_occult_when_glob` analogue. Reports the greatest-occultation
    /// instant and the sub-lunar point where it is central/greatest (not the
    /// full path). `None` if none occurs before the window end.
    pub fn next_global_occultation(
        &self,
        target: OccultTarget,
        after: Instant,
    ) -> Result<Option<GlobalOccultation>, EventError> {
        self.validate_occult_target(&target)?;
        let after_jd = after.julian_day.days();
        self.check_window(after_jd)?;
        if self.target_never_occultable(&target, after_jd)? {
            return Ok(None);
        }
        let mut scan_start = after_jd.max(WINDOW_START_JD + OCC_CONJUNCTION_STEP_DAYS);
        let scan_end = WINDOW_END_JD - OCC_CONJUNCTION_STEP_DAYS;
        loop {
            let conj = first_crossing_after(
                |jd| self.moon_target_lon_diff(&target, jd),
                scan_start,
                scan_end,
                OCC_CONJUNCTION_STEP_DAYS,
            )?;
            let Some(conj_jd) = conj else { return Ok(None) };
            // Minimize geocentric separation around the conjunction.
            let max_jd = self.minimize_geo_sep(
                &target,
                (conj_jd - OCC_CONTACT_HALF_WINDOW_DAYS).max(WINDOW_START_JD),
                (conj_jd + OCC_CONTACT_HALF_WINDOW_DAYS).min(WINDOW_END_JD),
            )?;
            let g = self.occ_geom(&target, None, max_jd)?;
            let moon_dist = self.body_distance_au(&CelestialBody::Moon, None, max_jd)?;
            let pi_moon = (R_EARTH_KM / (moon_dist * AU_KM)).clamp(-1.0, 1.0).asin().to_degrees();
            if g.sep_deg < g.s_moon_deg + g.s_tgt_deg + pi_moon
                && max_jd > after_jd
            {
                // Sub-lunar point: Moon geocentric Dec + (RA − GAST).
                let ((moon_ra, moon_dec), _) = self.moon_target_radec(&target, None, max_jd)?;
                let at = Instant::new(JulianDay::from_days(max_jd), TimeScale::Tdb);
                let gast = sidereal_time(at, Longitude::from_degrees(0.0)).local_apparent_deg;
                let lon = wrap180(moon_ra - gast);
                // Central if, when the Moon is pulled fully toward the target by
                // parallax at the sub-lunar point, the target is fully behind it.
                let central = g.sep_deg < (g.s_moon_deg - g.s_tgt_deg).max(0.0) + pi_moon;
                let occ_type = if central { OccultationType::Total } else { OccultationType::Grazing };
                return Ok(Some(GlobalOccultation {
                    target,
                    maximum: at,
                    sublunar_latitude: Latitude::from_degrees(moon_dec),
                    sublunar_longitude: Longitude::from_degrees(lon),
                    central,
                    occultation_type: occ_type,
                }));
            }
            scan_start = conj_jd + OCC_CONJUNCTION_STEP_DAYS;
            if scan_start >= scan_end {
                return Ok(None);
            }
        }
    }

    /// Golden-section minimize the GEOCENTRIC separation in `[a,b]`.
    fn minimize_geo_sep(&self, target: &OccultTarget, mut a: f64, mut b: f64) -> Result<f64, EventError> {
        let phi = 0.618_033_988_75_f64;
        let sep = |jd: f64| Ok::<f64, EventError>(self.occ_geom(target, None, jd)?.sep_deg);
        let mut c = b - (b - a) * phi;
        let mut d = a + (b - a) * phi;
        let (mut fc, mut fd) = (sep(c)?, sep(d)?);
        while (b - a) > REFINE_TOLERANCE_DAYS {
            if fc < fd { b = d; d = c; fd = fc; c = b - (b - a) * phi; fc = sep(c)?; }
            else { a = c; c = d; fc = fd; d = a + (b - a) * phi; fd = sep(d)?; }
        }
        Ok(0.5 * (a + b))
    }
}

#[cfg(test)]
mod when_glob_tests {
    use super::*;
    use pleiades_backend::test_backend::LinearSunMoon;

    #[test]
    fn never_occultable_star_returns_none() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
        assert!(engine
            .next_global_occultation(OccultTarget::Star("Sirius".into()), after)
            .unwrap()
            .is_none());
    }

    #[test]
    fn sun_target_rejected() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
        assert!(matches!(
            engine.next_global_occultation(OccultTarget::Body(CelestialBody::Sun), after),
            Err(EventError::UnsupportedOccultTarget { .. })
        ));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p pleiades-events occult::when_glob_tests 2>&1 | tail -20`
Expected: PASS (2 tests)

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-events/src/occult.rs
git commit -m "feat(events): SP-6 next_global_occultation (when_glob) + sub-lunar point"
```

---

## Task 8: Integration invariants + property tests (packaged backend)

**Files:**
- Create: `crates/pleiades-events/tests/occult_integration.rs`

**Interfaces:**
- Consumes: the full public occultation surface + `pleiades_data::packaged_backend`.

- [ ] **Step 1: Write the test**

Create `crates/pleiades-events/tests/occult_integration.rs`:

```rust
//! SP-6 occultation integration invariants over the real routing chain.

use pleiades_apparent::Atmosphere;
use pleiades_data::packaged_backend;
use pleiades_events::{EventEngine, OccultTarget, OccultationType};
use pleiades_types::{
    CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
};

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}
fn observer() -> ObserverLocation {
    ObserverLocation::new(Latitude::from_degrees(40.0), Longitude::from_degrees(-3.7), Some(650.0))
}

#[test]
fn next_planet_occultation_has_ordered_contacts() {
    let engine = EventEngine::new(packaged_backend());
    // Aldebaran is occulted by the Moon repeatedly during 1900–2100; find one.
    let out = engine
        .next_occultation(
            OccultTarget::Star("Aldebaran".into()),
            observer(),
            Atmosphere::default(),
            tdb(2_451_545.0),
        )
        .unwrap();
    if let Some(o) = out {
        let c1 = o.first_contact.instant.julian_day.days();
        let mx = o.maximum.instant.julian_day.days();
        let c4 = o.fourth_contact.instant.julian_day.days();
        assert!(c1 <= mx + 1e-9 && mx <= c4 + 1e-9, "C1<=max<=C4: {c1} {mx} {c4}");
        assert!(!matches!(o.occultation_type, OccultationType::Miss));
        assert!(o.magnitude >= 0.0 && o.obscuration >= 0.0 && o.obscuration <= 1.0);
        // A point star that is occulted is Total with magnitude 1.
        if matches!(o.occultation_type, OccultationType::Total) {
            assert!((o.magnitude - 1.0).abs() < 1e-6);
            assert!(o.second_contact.is_none(), "a point star has no interior contact");
        }
    }
    // If None (no locally-visible Aldebaran occultation in span), the search still
    // terminated cleanly — that is the invariant under test for un-found cases.
}

#[test]
fn global_occultation_reports_finite_sublunar_point() {
    let engine = EventEngine::new(packaged_backend());
    let out = engine
        .next_global_occultation(OccultTarget::Star("Aldebaran".into()), tdb(2_451_545.0))
        .unwrap();
    if let Some(g) = out {
        assert!(g.sublunar_latitude.degrees().abs() <= 90.0);
        assert!((-180.0..=180.0).contains(&g.sublunar_longitude.degrees()));
        assert!(g.maximum.julian_day.days() > 2_451_545.0);
    }
}

#[test]
fn ingress_and_egress_are_symmetric_about_maximum() {
    let engine = EventEngine::new(packaged_backend());
    if let Some(o) = engine
        .next_occultation(
            OccultTarget::Star("Aldebaran".into()),
            observer(),
            Atmosphere::default(),
            tdb(2_451_545.0),
        )
        .unwrap()
    {
        if !matches!(o.occultation_type, OccultationType::Miss) {
            let pre = o.maximum.instant.julian_day.days() - o.first_contact.instant.julian_day.days();
            let post = o.fourth_contact.instant.julian_day.days() - o.maximum.instant.julian_day.days();
            // Chord halves need not be exactly equal, but both are positive and
            // of the same order (within 3x) for a genuine occultation.
            assert!(pre > 0.0 && post > 0.0, "positive half-chords: {pre} {post}");
            assert!(pre < 3.0 * post + 1e-9 && post < 3.0 * pre + 1e-9);
        }
    }
}

#[test]
fn sirius_never_occulted_terminates_with_none() {
    let engine = EventEngine::new(packaged_backend());
    let out = engine
        .next_occultation(
            OccultTarget::Star("Sirius".into()),
            observer(),
            Atmosphere::default(),
            tdb(2_451_545.0),
        )
        .unwrap();
    assert!(out.is_none(), "Sirius (~39°S ecliptic latitude) is never occultable");
}
```

- [ ] **Step 2: Run the integration tests**

Run: `cargo test -p pleiades-events --test occult_integration 2>&1 | tail -30`
Expected: PASS (4 tests). If `next_occultation` for Aldebaran is unexpectedly slow (> ~30 s), reduce the search by starting near a known Aldebaran occultation epoch (e.g. `tdb(2_458_000.0)`); note the change in the test comment.

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-events/tests/occult_integration.rs
git commit -m "test(events): SP-6 occultation integration invariants + property tests"
```

---

## Task 9: Swiss-Ephemeris reference generator + committed corpus

**Files:**
- Create: `tools/se-occultations-reference/` (Cargo.toml, Cargo.lock, LICENSE-NOTES.md, src/main.rs) — clone `tools/se-pheno-reference/` and adapt
- Create: `crates/pleiades-validate/data/occultations-corpus/occultations.csv` and `manifest.txt` (generated by the tool)
- Modify: `Cargo.toml:22` (add `"tools/se-occultations-reference"` to the `exclude` list)

**Interfaces:**
- Produces: the committed CSV corpus + `manifest.txt` with a pinned SE version and `fnv1a64` checksum, consumed by Task 11's gate.

Clone `tools/se-pheno-reference/` verbatim, then change only: the crate `name` to `se-occultations-reference`; the SE calls; the epoch/target table; and the CSV schema. Keep the `[workspace]` empty table, the `fnv1a64` copy, the `--dry-run/--out/--ephe` CLI, and the `build_csv`/`build_manifest` structure unchanged.

**CSV schema** (one header/comment block + data rows; `mode` distinguishes the two row kinds; unused numeric fields are `-1`):

```
label,mode,se_body,star,jd_tt,lat,lon,elev,max_jd,c1_jd,c2_jd,c3_jd,c4_jd,magnitude,obscuration,occ_type,sublunar_lat,sublunar_lon,central
```

- `mode`: `loc` (from `swe_lun_occult_when_loc` + `swe_lun_occult_how`) or `glob` (from `swe_lun_occult_when_glob` + `swe_lun_occult_where`).
- `se_body`: SE planet number (0–9) or `-1` when `star` is used; `star`: SE star name or empty.
- `occ_type`: `0`=Miss, `1`=Grazing, `2`=Total.
- For `loc` rows the sub-lunar columns are `-1`; for `glob` rows the observer/contact columns except `max_jd`/`occ_type`/sub-lunar are `-1`.

**SE calls** (add `use` for the raw fns from `libswisseph_sys::raw`):

```rust
// swe_lun_occult_when_loc(tjd_start, ipl, starname, ifl, ifltype, geopos, tret, attr, backward, serr)
//   geopos = [lon_deg, lat_deg, elev_m]; tret[0]=max, tret[1]=C1, tret[2]=C2, tret[3]=C3, tret[4]=C4
//   attr[0]=frac of target diameter covered (magnitude), attr[2]=frac of target disc covered (obscuration)
// swe_lun_occult_how(tjd_ut, ipl, starname, ifl, geopos, attr, serr) — attr at a given instant
// swe_lun_occult_when_glob(tjd_start, ipl, starname, ifl, ifltype, tret, backward, serr) — tret[0]=max
// swe_lun_occult_where(tjd_ut, ipl, starname, ifl, geopos_out, attr, serr) — geopos_out[0]=lon, [1]=lat
```

Use `ifl = SEFLG_MOSEPH | SEFLG_NOGDEFL` (= `4 | 512` = `516`), matching `se-pheno-reference` (kernel-free, no network). Convert each SE `_ut` instant to TDB once at emission via the same ΔT the pheno tool uses (or the committed `swe_deltat`), and write the `jd`/`max_jd`/`c*_jd` columns in **TDB**. `central` = `1` when SE flags a central occultation (`retflag & SE_ECL_CENTRAL`), else `0`.

**Corpus contents** (~50–70 rows) — bright-star occultations from multiple observers (Aldebaran, Regulus, Spica, Antares; observer up vs below horizon), planetary occultations (Venus/Jupiter/Saturn, one `Total` disc + one `Grazing`), a non-occultable star (Sirius → the tool emits an explicit `occ_type=0` "no event" / `None` row so the gate can pin the fast-reject), and several `glob` rows exercising `central` both ways.

- [ ] **Step 1: Scaffold the tool**

```bash
cp -r tools/se-pheno-reference tools/se-occultations-reference
```
Edit `tools/se-occultations-reference/Cargo.toml`: set `name = "se-occultations-reference"`. In `Cargo.toml` (repo root), add `"tools/se-occultations-reference",` to the `exclude` array at line 22.

- [ ] **Step 2: Adapt `src/main.rs`** — swap the `swe_pheno` call for the four occultation calls above, replace `BODIES`/`EPOCHS` with the curated target×observer×mode row table, and widen the CSV columns to the schema above. Keep `build_manifest`'s `fnv1a64(csv)` line.

- [ ] **Step 3: Build and generate the corpus**

Run:
```bash
cargo run --manifest-path tools/se-occultations-reference/Cargo.toml -- \
  --out crates/pleiades-validate/data/occultations-corpus
```
Expected: writes `occultations.csv` (N rows) + `manifest.txt` with `file: occultations.csv rows=N checksum=<fnv1a64>`. If `libswisseph-sys` fails to build (missing C toolchain), install per `devkit:developer-environment` (mise) first.

- [ ] **Step 4: Verify the corpus is non-empty and well-formed**

Run: `head -5 crates/pleiades-validate/data/occultations-corpus/occultations.csv && wc -l crates/pleiades-validate/data/occultations-corpus/occultations.csv`
Expected: header + N data lines, N in ~50–70.

- [ ] **Step 5: Commit**

```bash
git add tools/se-occultations-reference Cargo.toml crates/pleiades-validate/data/occultations-corpus
git commit -m "tools: SP-6 Swiss-Ephemeris occultation reference generator + committed corpus"
```

---

## Task 10: Occultation thresholds module

**Files:**
- Create: `crates/pleiades-validate/src/occult_thresholds.rs`
- Modify: `crates/pleiades-validate/src/lib.rs:36-37` (register the module)

**Interfaces:**
- Produces: `pub const` ceilings consumed by Task 11. Values below are the design envelope; **re-pin each to ~1.4× the measured maximum** during Task 11 (the `pheno_thresholds.rs` convention).

- [ ] **Step 1: Create the thresholds file**

`crates/pleiades-validate/src/occult_thresholds.rs`:

```rust
//! SE-parity ceilings for the `validate-occultations` gate (SP-6). Each is set
//! from the measured per-metric residual maximum × ~1.4, matching the
//! `pheno_thresholds`/`rise_trans_thresholds` convention. Provisional until
//! Task 11 measures the real residuals against the committed corpus.

/// Contact/maximum instant residual vs SE, seconds (well-conditioned).
pub const CONTACT_SECONDS: f64 = 6.0;
/// Contact/maximum instant residual near grazing/limb chords, seconds.
pub const CONTACT_SECONDS_GRAZING: f64 = 30.0;
/// Covered-diameter fraction (magnitude) residual.
pub const MAGNITUDE_ABS: f64 = 0.01;
/// Covered-area fraction (obscuration) residual.
pub const OBSCURATION_ABS: f64 = 0.01;
/// Sub-lunar point residual (great-circle), arcminutes.
pub const SUBLUNAR_ARCMIN: f64 = 6.0;
```

- [ ] **Step 2: Register the module** in `crates/pleiades-validate/src/lib.rs` beside the pheno lines (36-37):

```rust
mod occult_thresholds;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p pleiades-validate 2>&1 | tail -5`
Expected: builds (unused-const warnings are fine until Task 11 consumes them).

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-validate/src/occult_thresholds.rs crates/pleiades-validate/src/lib.rs
git commit -m "feat(validate): SP-6 occultation gate ceilings (provisional)"
```

---

## Task 11: `validate-occultations` gate

**Files:**
- Create: `crates/pleiades-validate/src/occult_validation.rs`
- Modify: `crates/pleiades-validate/src/lib.rs` (add `pub mod occult_validation;` + `pub use`)
- Reference (mirror): `crates/pleiades-validate/src/pheno_validation.rs` (backend chain lines 373-382, `check_checksum` 272, `parse_manifest` 226, `EXPECTED_ROWS` 68, report struct 108).

**Interfaces:**
- Consumes: `pleiades_events::{EventEngine, OccultTarget, OccultationType, LocalOccultation, GlobalOccultation}`; the corpus (Task 9); `occult_thresholds` (Task 10); `pleiades_apparent::fnv1a64`.
- Produces: `pub fn validate_occultations_corpus() -> Result<OccultReport, OccultError>`; `OccultReport::{passed, summary_line}`; `pub const EXPECTED_ROWS: usize` (set to the Task-9 row count).

The gate recomputes each row's geometry **at SE's reported instants** (Tier-2), not by re-running the search, so it tests the geometry directly:
- **Tier 1 (self-consistency):** `c1 ≤ max ≤ c4`; `c2/c3` present iff `Total` disc and bracketed; `c2/c3 == None` for stars; `magnitude, obscuration ∈ [0,1]`; `obscuration > 0` iff `magnitude > 0`.
- **Tier 2 (SE parity):** for `loc` rows call `engine.occultation(target, observer, Atmosphere::default(), at = SE_max_instant)` and compare recomputed `max/c1/c4` (seconds), `magnitude`, `obscuration`, and `occultation_type` (exact) to the row; for `glob` rows call `engine.next_global_occultation(target, after = SE_max − 1 day)` and compare returned `maximum` (seconds), the sub-lunar point (great-circle arcmin vs the row's `sublunar_lat/lon`), and `central` (exact). Un-occultable-star rows (`occ_type=0`, Sirius) assert `next_occultation` / `next_global_occultation` return `None`.

- [ ] **Step 1: Write the gate module** mirroring `pheno_validation.rs` structure — `include_str!` the CSV + manifest, `OccultError`/`OccultReport`, `check_checksum`, per-row parse (19 fields), Tier-1/Tier-2 as above, `measure()`, `validate_occultations_corpus()`, and a `mod tests` with `manifest_row_count_is_pinned`, `checksum_drift_fails_closed`, `gate_passes_on_committed_corpus`. Use the exact backend chain from `pheno_validation.rs:373-382`.

- [ ] **Step 2: Register in `lib.rs`** beside the pheno exports (line ~37 and ~217):

```rust
pub mod occult_validation;
```
```rust
pub use occult_validation::{validate_occultations_corpus, OccultError, OccultReport};
```

- [ ] **Step 3: Run the gate, then re-pin ceilings**

Run: `cargo test -p pleiades-validate occult_validation 2>&1 | tail -30`
If Tier-2 fails, print the measured per-metric maxima, then set each `occult_thresholds.rs` const to `~1.4×` that maximum and re-run until `gate_passes_on_committed_corpus` passes. Record the measured maxima in the thresholds doc header (SP-2b convention).

- [ ] **Step 4: Verify green**

Run: `cargo test -p pleiades-validate occult_validation 2>&1 | tail -10`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/occult_validation.rs crates/pleiades-validate/src/occult_thresholds.rs crates/pleiades-validate/src/lib.rs
git commit -m "feat(validate): SP-6 validate-occultations SE-parity gate + pinned ceilings"
```

---

## Task 12: Wire the gate into the CLI + release battery

**Files:**
- Modify: `crates/pleiades-validate/src/render/cli.rs` — `run_all_numeric_gates` (after line 119), CLI match arm (after lines 358-363), help string (line ~2242), `mod tests` (after line ~2987)
- Modify: `crates/pleiades-validate/src/tests/validate_gates.rs` (after lines 489-507)

**Interfaces:**
- Consumes: `validate_occultations_corpus` (Task 11).

- [ ] **Step 1: Add to `run_all_numeric_gates`** as the new last line (after the pheno line 119):

```rust
    crate::validate_occultations_corpus().map_err(|e| format!("occultations gate failed: {e}"))?;
```

- [ ] **Step 2: Add the CLI dispatch arm** (mirroring lines 358-363):

```rust
        Some("validate-occultations") | Some("occultations") | Some("occult-gate") => {
            ensure_no_extra_args(&args[1..], "validate-occultations")?;
            crate::validate_occultations_corpus()
                .map(|r| r.summary_line())
                .map_err(|e| e.to_string())
        }
```

- [ ] **Step 3: Add help-text entries** at the help string (line ~2242):

```
    validate-occultations  Lunar occultation SE-parity gate (swe_lun_occult_*)
    occultations           Alias for validate-occultations
    occult-gate            Alias for validate-occultations
```

- [ ] **Step 4: Add the release-battery test** in cli.rs `mod tests` (after `..._includes_pheno_and_passes`, line ~2987):

```rust
    #[test]
    fn run_all_numeric_gates_includes_occultations_and_passes() {
        crate::validate_occultations_corpus()
            .expect("occultations gate passes standalone on the committed corpus");
        super::run_all_numeric_gates()
            .expect("full numeric-gate battery (incl. occultations) passes");
    }
```

- [ ] **Step 5: Add the CLI dispatch test** in `crates/pleiades-validate/src/tests/validate_gates.rs` (mirroring lines 489-507):

```rust
    #[test]
    fn validate_occultations_and_aliases_agree_and_reject_extra_args() {
        let primary = run(&["validate-occultations"]);
        let alias1 = run(&["occultations"]);
        let alias2 = run(&["occult-gate"]);
        assert_eq!(primary, alias1);
        assert_eq!(primary, alias2);
        assert!(run(&["validate-occultations", "extra"]).is_err());
        let help = run(&["help"]).unwrap();
        assert!(help.contains("validate-occultations"));
        assert!(help.contains("occult-gate"));
    }
```
(Match the exact `run` helper signature already used in that test module.)

- [ ] **Step 6: Run the wiring tests**

Run: `cargo test -p pleiades-validate -- occultations 2>&1 | tail -20`
Expected: PASS (release-battery test + CLI dispatch test).

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-validate/src/render/cli.rs crates/pleiades-validate/src/tests/validate_gates.rs
git commit -m "feat(validate): SP-6 wire validate-occultations into CLI + release battery"
```

---

## Task 13: Bump the compatibility profile 0.7.11 → 0.7.12

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/mod.rs:26` (ID), `:41` (summary), `:88` (release_notes), `:38` (content checksum)
- Modify: `crates/pleiades-validate/src/tests/render_request.rs:333`, `crates/pleiades-cli/src/cli/tests/summary_commands.rs:433` (version-assertion strings)
- Modify: `README.md:18`

**Interfaces:** none (release metadata).

- [ ] **Step 1: Bump the ID + prose** in `compatibility/mod.rs`:
  - Line 26: `"pleiades-compatibility-profile/0.7.12"`.
  - Line 41 summary: append an SP-6 sentence (e.g. "SP-6 lunar occultations done — `EventEngine::occultation`/`next_occultation`/`previous_occultation`/`next_global_occultation` (`swe_lun_occult_*` analogues) for the Moon occulting planets Mercury–Pluto and curated fixed stars; gated by the fail-closed `validate-occultations` gate; compatibility profile bumped to 0.7.12.").
  - Line 88 `release_notes` array: add a new entry mirroring the SP-5 element's shape.

- [ ] **Step 2: Update the two version-assertion test strings** (`render_request.rs:333`, `summary_commands.rs:433`): change `compatibility=pleiades-compatibility-profile/0.7.11` → `0.7.12`.

- [ ] **Step 3: Update README.md:18** → `pleiades-compatibility-profile/0.7.12`.

- [ ] **Step 4: Recompute the content checksum.** Run the profile test to get the new expected checksum, then set `mod.rs:38` `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` to it:

Run: `cargo test -p pleiades-core rendered_profile_matches_pinned_content_checksum 2>&1 | tail -20`
Expected: FAIL first, printing `got 0x....`; paste that value into line 38, re-run → PASS.

- [ ] **Step 5: Verify version assertions pass**

Run: `cargo test -p pleiades-validate render_request 2>&1 | tail -10 && cargo test -p pleiades-cli summary_commands 2>&1 | tail -10`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-core/src/compatibility/mod.rs crates/pleiades-validate/src/tests/render_request.rs crates/pleiades-cli/src/cli/tests/summary_commands.rs README.md
git commit -m "chore(release): SP-6 bump compatibility profile 0.7.11 -> 0.7.12"
```

---

## Task 14: Docs — scope block, README bullet, plan status

**Files:**
- Modify: `crates/pleiades-events/src/lib.rs` (crate scope doc block), `README.md` (event-engine feature bullets), `PLAN.md` (status line), `plan/status/01-current-execution-frontier.md`, `plan/status/02-next-slice-candidates.md`

Note: per the wiring map, event-engine surfaces are **not** registered in the `claims/compat.rs` overclaim audit (that is catalog-scoped to houses/ayanamsas). "Registering the SP-6 surface" is exactly: the numeric gate (Task 12), the compatibility-profile prose (Task 13), and the README bullet (this task). No `compat.rs`/`audit.rs` edit.

- [ ] **Step 1: Extend the `pleiades-events` crate scope doc** in `lib.rs` (top `//!` block) with a one-line occultation coverage sentence.

- [ ] **Step 2: Add a README occultation bullet** in the event-engine feature list (after the SP-5 pheno bullet), describing `EventEngine::occultation`/`next_occultation`/`previous_occultation`/`next_global_occultation`, targets (planets + curated stars), the sub-lunar-point-not-path boundary, the `validate-occultations` gate, and the measured accuracy (fill in from Task 11's pinned ceilings).

- [ ] **Step 3: Update `PLAN.md`** — extend the status line to mark SP-6 done (append to the SP-5 sentence), and change "Next candidate slices" to the remaining two (central-path cartography; custom fictitious-body elements).

- [ ] **Step 4: Update `plan/status/01` and `plan/status/02`** — move SP-6 from "Next candidate slices" into the done list; leave central-path cartography and custom fictitious-body elements as the remaining candidates.

- [ ] **Step 5: Full verification**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: PASS (entire workspace). Then the release battery:
Run: `cargo run -p pleiades-validate -- validate-occultations 2>&1 | tail -5`
Expected: the gate's summary line, exit 0.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-events/src/lib.rs README.md PLAN.md plan/status/01-current-execution-frontier.md plan/status/02-next-slice-candidates.md
git commit -m "docs: SP-6 declare lunar occultations; mark SP-6 done; profile 0.7.12"
```

---

## Self-Review

**Spec coverage** — every spec section maps to a task:
- Data model (5 `Occult*` types) → Task 1. Two-circle geometry + magnitude/obscuration/type → Task 2. Reused topocentric/semidiameter/fixstar sampling → Task 3. Target rejection + un-occultable-star reject → Task 4. `how` → Task 5. `when_loc` → Task 6. `when_glob` + sub-lunar point → Task 7. Integration/property tests → Task 8. Reference tool + corpus → Task 9. Thresholds → Task 10. Two-tier gate → Task 11. CLI + release battery → Task 12. Profile bump `0.7.11→0.7.12` → Task 13. Scope-block/README/plan docs → Task 14.
- Spec boundary "central-path cartography out" / "grazing limb profiles out" → honored (Grazing is classified, not resolved to limb contacts; `when_glob` reports a single sub-lunar point, not a path).
- **Spec correction:** the spec's "add to the overclaim-audit claim tier ↔ evidence mapping" does not apply to event-engine surfaces (audit is catalog-scoped). Task 14 records the real registration surfaces (gate + profile prose + README) instead.

**Type consistency** — `OccultTarget`/`OccultationType`/`OccultationContact`/`LocalOccultation`/`GlobalOccultation` are defined in Task 1 and used verbatim in Tasks 2–12. `occ_geom`/`moon_target_radec`/`occ_contact_at`/`minimize_occ_sep`/`bisect_occ_contact`/`moon_target_lon_diff`/`minimize_geo_sep` are each defined once and referenced consistently. Gate fns `validate_occultations_corpus`/`OccultReport`/`OccultError` match between Tasks 11 and 12. Aliases `validate-occultations`/`occultations`/`occult-gate` match between Tasks 12 and the spec.

**Placeholder scan** — no TBD/TODO; the one deferred value (measured ceilings) has an explicit measure-and-pin step (Task 11 Step 3) with a starting envelope, matching how every prior SP gate landed.

**Right-sizing** — each task ends with an independently testable, committable deliverable; engine tasks (1–8) are pure Rust with unit/integration tests; validation tasks (9–14) each gate on a concrete `cargo test`/`cargo run` command.
