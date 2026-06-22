# Topocentric Body Positions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add opt-in topocentric body positions (diurnal parallax + diurnal aberration) as a chart-layer correction layered on the existing apparent-place-of-date output.

**Architecture:** Pure correction math lives in `pleiades-apparent` (depends only on `pleiades-types`), receiving time-derived scalars (local apparent sidereal time, obliquity) as parameters — exactly like the existing `apparent_position()` takes the Sun's longitude. `pleiades-time` exposes the UT1/GMST scalars. `pleiades-core` orchestrates: it computes local apparent sidereal time and calls the new `topocentric_position()` immediately after `apparent_position()`. Backends stay geocentric/mean; the existing backend-boundary "topocentric not implemented" rejection is untouched.

**Tech Stack:** Rust workspace (`cargo`), no new external dependencies. Existing crates: `pleiades-types`, `pleiades-time`, `pleiades-apparent`, `pleiades-core`, `pleiades-backend`, `pleiades-cli`.

## Global Constraints

- Default coverage window is 1900–2100 CE; topocentric must work across it.
- `pleiades-apparent` MUST remain `pleiades-types`-only pure (no `pleiades-time` dependency). All time-derived inputs are passed in by the caller.
- Backends remain mean/geocentric. The chart-layer feature MUST NOT set `EphemerisRequest.observer`/`ChartRequest.body_observer` — those drive the still-unsupported native-backend topocentric path.
- Geocentric default output (existing goldens, existing provenance lines) MUST remain byte-for-byte unchanged when `--topocentric` is not requested.
- Topocentric is **opt-in** and builds on apparent place: it requires an observer location and a body with a known geocentric distance; an explicit request that cannot be honored **errors** (no silent geocentric fallback).
- Reuse existing error variants `ApparentPlaceError::MissingDistance` and `ApparentPlaceError::NonFiniteCorrection { stage }`; do not add new ones.
- All gated canonical policy strings asserting "topocentric ... unsupported" must be updated in lockstep (their gates fail otherwise).
- Constants (WGS84): equatorial radius `EARTH_EQUATORIAL_RADIUS_M = 6_378_137.0`; polar/equatorial ratio `EARTH_B_OVER_A = 0.996_647_189`; AU in Earth equatorial radii `AU_IN_EARTH_RADII = 23_454.779`; diurnal aberration constant `DIURNAL_ABERRATION_ARCSEC = 0.319_2` (arcsec, = 0.0213 s · 15).

---

### Task 1: Expose UT1 + GMST scalars in `pleiades-time`

`pleiades-core` needs local apparent sidereal time, which starts from UT1 (= TT − ΔT) and GMST(UT1). The ΔT table already exists (`deltat::delta_t`). Add two pure public helpers.

**Files:**
- Modify: `crates/pleiades-time/src/convert.rs` (add `ut1_jd_from_tt`)
- Create: `crates/pleiades-time/src/sidereal.rs`
- Modify: `crates/pleiades-time/src/lib.rs` (declare `pub mod sidereal;` and re-export)

**Interfaces:**
- Consumes: `pleiades_time::deltat::delta_t(jd: f64) -> Result<(f64, DeltaTQuality), CivilTimeError>`.
- Produces:
  - `pleiades_time::ut1_jd_from_tt(jd_tt: f64) -> Result<f64, CivilTimeError>`
  - `pleiades_time::gmst_degrees(jd_ut1: f64) -> f64` (IAU-1982 GMST, Meeus 12.4, normalized to [0, 360)).

- [ ] **Step 1: Write the failing test for GMST**

Create `crates/pleiades-time/src/sidereal.rs`:

```rust
//! Greenwich mean sidereal time (IAU-1982, Meeus 12.4) from a UT1 Julian day.

/// Greenwich mean sidereal time in degrees, normalized to `[0, 360)`.
///
/// `jd_ut1` is the Julian day in the UT1 time scale. Formula: Meeus,
/// *Astronomical Algorithms*, eq. 12.4.
pub fn gmst_degrees(jd_ut1: f64) -> f64 {
    let t = (jd_ut1 - 2_451_545.0) / 36_525.0;
    let theta = 280.460_618_37
        + 360.985_647_366_29 * (jd_ut1 - 2_451_545.0)
        + 0.000_387_933 * t * t
        - (t * t * t) / 38_710_000.0;
    theta.rem_euclid(360.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gmst_matches_meeus_example_12a() {
        // Meeus Example 12.a: 1987 April 10, 0h UT -> JD 2446895.5,
        // GMST = 13h10m46.3668s = 197.693195 deg.
        let gmst = gmst_degrees(2_446_895.5);
        assert!((gmst - 197.693_195).abs() < 1e-4, "gmst {gmst}");
    }

    #[test]
    fn gmst_is_normalized() {
        for jd in [2_415_020.5_f64, 2_451_545.0, 2_488_069.5] {
            let g = gmst_degrees(jd);
            assert!((0.0..360.0).contains(&g), "gmst {g} out of range at jd {jd}");
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-time sidereal`
Expected: FAIL — `sidereal` module not declared (compile error: file not in module tree).

- [ ] **Step 3: Wire the module and add the UT1 helper**

In `crates/pleiades-time/src/lib.rs`, add the module declaration next to the other `pub mod` lines (after `pub mod policy;`):

```rust
pub mod sidereal;
```

And add to the re-export block (extend the existing `pub use` list):

```rust
pub use sidereal::gmst_degrees;
```

In `crates/pleiades-time/src/convert.rs`, add (near `to_terrestrial`):

```rust
/// Returns the UT1 Julian day for a Terrestrial Time Julian day, using the
/// Delta-T table (`UT1 = TT - ΔT`). The Julian-day argument is interpreted in
/// the TT scale; the ΔT lookup uses it directly (the sub-second feedback of ΔT
/// on its own lookup epoch is negligible for sidereal time).
pub fn ut1_jd_from_tt(jd_tt: f64) -> Result<f64, crate::error::CivilTimeError> {
    let (delta_t_seconds, _quality) = crate::deltat::delta_t(jd_tt)?;
    Ok(jd_tt - delta_t_seconds / 86_400.0)
}
```

In `crates/pleiades-time/src/lib.rs`, extend the `pub use convert::{...}` list to include `ut1_jd_from_tt`.

- [ ] **Step 4: Add the UT1 test**

Append to the `tests` module in `crates/pleiades-time/src/convert.rs`:

```rust
#[test]
fn ut1_is_earlier_than_tt_by_delta_t() {
    // J2000.0: ΔT ≈ 63.8 s, so UT1 JD < TT JD by ~63.8/86400 days.
    let jd_tt = 2_451_545.0;
    let jd_ut1 = ut1_jd_from_tt(jd_tt).unwrap();
    let diff_seconds = (jd_tt - jd_ut1) * 86_400.0;
    assert!((50.0..80.0).contains(&diff_seconds), "ΔT {diff_seconds}s");
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p pleiades-time sidereal && cargo test -p pleiades-time ut1_is_earlier`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-time/src/sidereal.rs crates/pleiades-time/src/convert.rs crates/pleiades-time/src/lib.rs
git commit -m "feat(time): expose UT1-from-TT and GMST scalars for sidereal time"
```

---

### Task 2: Observer geocentric vector + Cartesian helpers in `pleiades-apparent`

Pure geometry: geodetic latitude/elevation → geocentric `ρ·sinφ′`, `ρ·cosφ′` (Meeus Ch. 11), and the observer's equatorial rectangular vector given local apparent sidereal time.

**Files:**
- Create: `crates/pleiades-apparent/src/parallax.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs` (declare `pub mod parallax;` and re-export)

**Interfaces:**
- Consumes: `pleiades_types::ObserverLocation` (fields `latitude: Latitude`, `longitude: Longitude`, `elevation_m: Option<f64>`).
- Produces:
  - `pleiades_apparent::parallax::ObserverGeocentric { rho_sin_phi_prime: f64, rho_cos_phi_prime: f64 }`
  - `ObserverGeocentric::from_location(&ObserverLocation) -> ObserverGeocentric`
  - `ObserverGeocentric::equatorial_vector(self, local_sidereal_time_deg: f64) -> [f64; 3]` (units: Earth equatorial radii)
  - `pub const AU_IN_EARTH_RADII: f64`

- [ ] **Step 1: Write the failing test**

Create `crates/pleiades-apparent/src/parallax.rs`:

```rust
//! Observer geocentric position on the WGS84 ellipsoid and the equatorial
//! rectangular vector used for diurnal parallax. Pure geometry (Meeus Ch. 11).

use pleiades_types::ObserverLocation;

/// WGS84 equatorial radius, meters.
pub const EARTH_EQUATORIAL_RADIUS_M: f64 = 6_378_137.0;
/// WGS84 polar/equatorial axis ratio (b/a).
pub const EARTH_B_OVER_A: f64 = 0.996_647_189;
/// One astronomical unit expressed in Earth equatorial radii.
pub const AU_IN_EARTH_RADII: f64 = 23_454.779;

/// Geocentric quantities `ρ·sinφ′` and `ρ·cosφ′` for an observer (Meeus 11).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObserverGeocentric {
    /// `ρ·sinφ′`, dimensionless (fraction of Earth equatorial radius).
    pub rho_sin_phi_prime: f64,
    /// `ρ·cosφ′`, dimensionless (fraction of Earth equatorial radius).
    pub rho_cos_phi_prime: f64,
}

impl ObserverGeocentric {
    /// Computes `ρ·sinφ′` and `ρ·cosφ′` from geodetic latitude and elevation
    /// (Meeus Ch. 11). Elevation defaults to sea level when absent.
    pub fn from_location(observer: &ObserverLocation) -> Self {
        let phi = observer.latitude.degrees().to_radians();
        let height_m = observer.elevation_m.unwrap_or(0.0);
        let u = (EARTH_B_OVER_A * phi.tan()).atan();
        let h_over_a = height_m / EARTH_EQUATORIAL_RADIUS_M;
        let rho_sin_phi_prime = EARTH_B_OVER_A * u.sin() + h_over_a * phi.sin();
        let rho_cos_phi_prime = u.cos() + h_over_a * phi.cos();
        Self {
            rho_sin_phi_prime,
            rho_cos_phi_prime,
        }
    }

    /// Observer rectangular vector in the equatorial frame, in Earth equatorial
    /// radii, given local apparent sidereal time in degrees.
    pub fn equatorial_vector(self, local_sidereal_time_deg: f64) -> [f64; 3] {
        let lst = local_sidereal_time_deg.to_radians();
        [
            self.rho_cos_phi_prime * lst.cos(),
            self.rho_cos_phi_prime * lst.sin(),
            self.rho_sin_phi_prime,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{Latitude, Longitude};

    fn loc(lat: f64, lon: f64, elev: Option<f64>) -> ObserverLocation {
        ObserverLocation::new(
            Latitude::from_degrees(lat),
            Longitude::from_degrees(lon),
            elev,
        )
    }

    #[test]
    fn palomar_matches_meeus_chapter_11() {
        // Meeus Ch. 11 worked example: Palomar, φ=+33.356111°, H=1706 m →
        // ρ·sinφ′ = 0.546861, ρ·cosφ′ = 0.836339.
        let g = ObserverGeocentric::from_location(&loc(33.356_111, 0.0, Some(1706.0)));
        assert!((g.rho_sin_phi_prime - 0.546_861).abs() < 1e-5, "{g:?}");
        assert!((g.rho_cos_phi_prime - 0.836_339).abs() < 1e-5, "{g:?}");
    }

    #[test]
    fn equator_sea_level_is_unit_in_plane() {
        let g = ObserverGeocentric::from_location(&loc(0.0, 0.0, Some(0.0)));
        assert!((g.rho_cos_phi_prime - 1.0).abs() < 1e-6, "{g:?}");
        assert!(g.rho_sin_phi_prime.abs() < 1e-6, "{g:?}");
    }

    #[test]
    fn vector_at_lst_zero_points_along_x() {
        let g = ObserverGeocentric::from_location(&loc(0.0, 0.0, Some(0.0)));
        let v = g.equatorial_vector(0.0);
        assert!((v[0] - 1.0).abs() < 1e-6, "{v:?}");
        assert!(v[1].abs() < 1e-6 && v[2].abs() < 1e-6, "{v:?}");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-apparent parallax`
Expected: FAIL — `parallax` module not declared (compile error).

- [ ] **Step 3: Wire the module**

In `crates/pleiades-apparent/src/lib.rs`, after the `pub mod precession;` block, add:

```rust
pub mod parallax;

pub use parallax::{ObserverGeocentric, AU_IN_EARTH_RADII};
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-apparent parallax`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-apparent/src/parallax.rs crates/pleiades-apparent/src/lib.rs
git commit -m "feat(apparent): observer geocentric vector for diurnal parallax (Meeus 11)"
```

---

### Task 3: Topocentric orchestrator + provenance in `pleiades-apparent`

Convert the apparent ecliptic position to equatorial Cartesian, subtract the observer vector (diurnal parallax), apply diurnal aberration, convert back to ecliptic-of-date. Extend provenance.

**Files:**
- Modify: `crates/pleiades-apparent/src/provenance.rs` (extend `CorrectionSet`, add `TopocentricProvenance`, extend `MODEL_SOURCES`)
- Create: `crates/pleiades-apparent/src/topocentric.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs` (declare module, re-export)

**Interfaces:**
- Consumes: `pleiades_types::{EclipticCoordinates, ObserverLocation, Angle, Longitude, Latitude}`, `ObserverGeocentric`, `AU_IN_EARTH_RADII`, `ApparentPlaceError`.
- Produces:
  - `pleiades_apparent::topocentric::TopocentricProvenance { parallax_longitude_arcsec: f64, parallax_latitude_arcsec: f64, diurnal_aberration_arcsec: f64, distance_au_used: f64 }`
  - `pleiades_apparent::topocentric::TopocentricPosition { ecliptic: EclipticCoordinates, provenance: TopocentricProvenance }`
  - `pleiades_apparent::topocentric_position(apparent: EclipticCoordinates, observer: &ObserverLocation, local_sidereal_time_deg: f64, obliquity_deg: f64) -> Result<TopocentricPosition, ApparentPlaceError>`
  - New `CorrectionSet` fields `diurnal_parallax: bool`, `diurnal_aberration: bool`.

- [ ] **Step 1: Extend provenance (write the change)**

In `crates/pleiades-apparent/src/provenance.rs`, add the two fields to `CorrectionSet` (after `nutation_longitude`):

```rust
    /// Diurnal (geocentric) parallax was applied (topocentric place).
    pub diurnal_parallax: bool,
    /// Diurnal aberration was applied (topocentric place).
    pub diurnal_aberration: bool,
```

Update `MODEL_SOURCES` to append the topocentric sources (replace the existing constant value):

```rust
pub const MODEL_SOURCES: &str =
    "precession (IAU-1976, Meeus 20.3/21.4); nutation-iau1980.csv (IAU-1980 truncated, Meeus Table 22.A); annual aberration (Meeus 23.2); light-time iteration; light-deflection omitted; diurnal parallax (Meeus 11/40, WGS84 ellipsoid); diurnal aberration (0.319\"·ρcosφ′); atmospheric refraction omitted";
```

Add the new provenance struct after `ApparentProvenance`:

```rust
/// Provenance for the topocentric (observer-centric) correction stage.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TopocentricProvenance {
    /// Parallax shift applied to ecliptic longitude, arcseconds.
    pub parallax_longitude_arcsec: f64,
    /// Parallax shift applied to ecliptic latitude, arcseconds.
    pub parallax_latitude_arcsec: f64,
    /// Diurnal aberration magnitude applied, arcseconds.
    pub diurnal_aberration_arcsec: f64,
    /// Geocentric distance used for the parallax, AU.
    pub distance_au_used: f64,
}

impl TopocentricProvenance {
    /// Compact one-line rendering for diagnostics and release-facing summaries.
    pub fn summary_line(&self) -> String {
        format!(
            "topocentric parallax_lon={:.3}\" parallax_lat={:.3}\" diurnal_aberration={:.4}\" distance_au={:.6}",
            self.parallax_longitude_arcsec,
            self.parallax_latitude_arcsec,
            self.diurnal_aberration_arcsec,
            self.distance_au_used,
        )
    }
}

impl fmt::Display for TopocentricProvenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}
```

Update the existing `CorrectionSet` literal in the `provenance.rs` `tests` module (the `summary_line_is_nonempty_and_matches_display` test) to include the two new fields set to `false`:

```rust
            corrections: CorrectionSet {
                light_time: true,
                precession: true,
                annual_aberration: true,
                nutation_longitude: true,
                diurnal_parallax: false,
                diurnal_aberration: false,
            },
```

- [ ] **Step 2: Fix the other `CorrectionSet` literal**

In `crates/pleiades-apparent/src/apparent.rs`, the `apparent_position()` body constructs a `CorrectionSet`. Add the two new fields set to `false` there:

```rust
        corrections: CorrectionSet {
            light_time: true,
            precession: true,
            annual_aberration: true,
            nutation_longitude: true,
            diurnal_parallax: false,
            diurnal_aberration: false,
        },
```

- [ ] **Step 3: Run to verify the crate compiles with extended provenance**

Run: `cargo test -p pleiades-apparent provenance`
Expected: PASS (the existing provenance test now compiles and passes with the new fields).

- [ ] **Step 4: Write the failing topocentric test**

Create `crates/pleiades-apparent/src/topocentric.rs`:

```rust
//! Topocentric correction: diurnal parallax + diurnal aberration applied to a
//! geocentric apparent ecliptic-of-date position. Pure; the caller supplies the
//! local apparent sidereal time and obliquity of date.

use pleiades_types::{Angle, EclipticCoordinates, Latitude, Longitude, ObserverLocation};

use crate::error::ApparentPlaceError;
use crate::parallax::{ObserverGeocentric, AU_IN_EARTH_RADII};
use crate::provenance::TopocentricProvenance;

/// Diurnal aberration constant in arcseconds (0.0213 s of time × 15).
pub const DIURNAL_ABERRATION_ARCSEC: f64 = 0.319_2;

/// A topocentric ecliptic-of-date position and its provenance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TopocentricPosition {
    pub ecliptic: EclipticCoordinates,
    pub provenance: TopocentricProvenance,
}

/// Applies diurnal parallax and diurnal aberration to a geocentric apparent
/// ecliptic position.
///
/// `apparent` is the geocentric apparent ecliptic-of-date position (must carry
/// `distance_au`). `local_sidereal_time_deg` is the observer's local apparent
/// sidereal time (degrees). `obliquity_deg` is the true obliquity of date.
pub fn topocentric_position(
    apparent: EclipticCoordinates,
    observer: &ObserverLocation,
    local_sidereal_time_deg: f64,
    obliquity_deg: f64,
) -> Result<TopocentricPosition, ApparentPlaceError> {
    let distance_au = apparent.distance_au.ok_or(ApparentPlaceError::MissingDistance)?;

    let obliquity = Angle::from_degrees(obliquity_deg);
    let equatorial = apparent.to_equatorial(obliquity);
    let ra = equatorial.right_ascension.degrees().to_radians();
    let dec = equatorial.declination.degrees().to_radians();

    // Geocentric body vector in Earth equatorial radii.
    let d = distance_au * AU_IN_EARTH_RADII;
    let bx = d * dec.cos() * ra.cos();
    let by = d * dec.cos() * ra.sin();
    let bz = d * dec.sin();

    // Observer vector and topocentric (observer-relative) vector.
    let geo = ObserverGeocentric::from_location(observer);
    let [ox, oy, oz] = geo.equatorial_vector(local_sidereal_time_deg);
    let tx = bx - ox;
    let ty = by - oy;
    let tz = bz - oz;
    let topo_distance = (tx * tx + ty * ty + tz * tz).sqrt();
    if !topo_distance.is_finite() || topo_distance <= 0.0 {
        return Err(ApparentPlaceError::NonFiniteCorrection { stage: "topocentric" });
    }
    let mut ra_topo = ty.atan2(tx);
    let dec_topo = (tz / topo_distance).asin();

    // Diurnal aberration: observer moves east at ω·ρcosφ′. Hour angle H = LAST - RA.
    let hour_angle = (local_sidereal_time_deg.to_radians() - ra_topo).rem_euclid(std::f64::consts::TAU);
    let aberr_ra_arcsec =
        DIURNAL_ABERRATION_ARCSEC * geo.rho_cos_phi_prime * hour_angle.cos() / dec_topo.cos();
    let aberr_dec_arcsec =
        DIURNAL_ABERRATION_ARCSEC * geo.rho_cos_phi_prime * hour_angle.sin() * dec_topo.sin();
    ra_topo += (aberr_ra_arcsec / 3600.0).to_radians();
    let dec_topo = dec_topo + (aberr_dec_arcsec / 3600.0).to_radians();

    let equatorial_topo = pleiades_types::EquatorialCoordinates::new(
        Angle::from_degrees(ra_topo.to_degrees()).normalized_0_360(),
        Latitude::from_degrees(dec_topo.to_degrees()),
        Some(topo_distance / AU_IN_EARTH_RADII),
    );
    let ecliptic_topo = equatorial_topo.to_ecliptic(obliquity);

    let mut d_lon = ecliptic_topo.longitude.degrees() - apparent.longitude.degrees();
    if d_lon > 180.0 {
        d_lon -= 360.0;
    } else if d_lon < -180.0 {
        d_lon += 360.0;
    }
    let d_lat = ecliptic_topo.latitude.degrees() - apparent.latitude.degrees();

    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(ecliptic_topo.longitude.degrees().rem_euclid(360.0)),
        ecliptic_topo.latitude,
        ecliptic_topo.distance_au,
    );
    if !ecliptic.longitude.degrees().is_finite() || !ecliptic.latitude.degrees().is_finite() {
        return Err(ApparentPlaceError::NonFiniteCorrection { stage: "topocentric" });
    }

    Ok(TopocentricPosition {
        ecliptic,
        provenance: TopocentricProvenance {
            parallax_longitude_arcsec: d_lon * 3600.0,
            parallax_latitude_arcsec: d_lat * 3600.0,
            diurnal_aberration_arcsec: aberr_ra_arcsec.hypot(aberr_dec_arcsec),
            distance_au_used: distance_au,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ecl(lon: f64, lat: f64, dist: f64) -> EclipticCoordinates {
        EclipticCoordinates::new(
            Longitude::from_degrees(lon),
            Latitude::from_degrees(lat),
            Some(dist),
        )
    }

    fn observer(lat: f64) -> ObserverLocation {
        ObserverLocation::new(
            Latitude::from_degrees(lat),
            Longitude::from_degrees(0.0),
            Some(0.0),
        )
    }

    #[test]
    fn moon_parallax_is_about_one_degree() {
        // Moon at ~0.00257 AU (60.3 Earth radii). For an observer with the Moon
        // near the horizon the parallax approaches ~0.95°. Assert it is large.
        let out = topocentric_position(ecl(100.0, 0.0, 0.002_57), &observer(0.0), 100.0, 23.4)
            .unwrap();
        let shift = out.provenance.parallax_longitude_arcsec.hypot(
            out.provenance.parallax_latitude_arcsec,
        ) / 3600.0;
        assert!(shift > 0.3, "moon parallax {shift}° too small");
    }

    #[test]
    fn distant_body_parallax_is_negligible() {
        // A body at 30 AU: parallax < 1".
        let out = topocentric_position(ecl(100.0, 0.0, 30.0), &observer(0.0), 100.0, 23.4)
            .unwrap();
        let shift = out.provenance.parallax_longitude_arcsec.hypot(
            out.provenance.parallax_latitude_arcsec,
        );
        assert!(shift < 1.0, "distant parallax {shift}\" too large");
    }

    #[test]
    fn missing_distance_errors() {
        let no_dist = EclipticCoordinates::new(
            Longitude::from_degrees(100.0),
            Latitude::from_degrees(0.0),
            None,
        );
        let err = topocentric_position(no_dist, &observer(0.0), 100.0, 23.4).unwrap_err();
        assert_eq!(err, ApparentPlaceError::MissingDistance);
    }

    #[test]
    fn diurnal_aberration_is_sub_arcsec() {
        let out = topocentric_position(ecl(100.0, 0.0, 1.0), &observer(0.0), 100.0, 23.4)
            .unwrap();
        assert!(
            out.provenance.diurnal_aberration_arcsec < 0.33,
            "diurnal aberration {}\"",
            out.provenance.diurnal_aberration_arcsec
        );
    }
}
```

- [ ] **Step 5: Run test to verify it fails**

Run: `cargo test -p pleiades-apparent topocentric`
Expected: FAIL — `topocentric` module not declared (compile error).

- [ ] **Step 6: Wire the module and exports**

In `crates/pleiades-apparent/src/lib.rs`, after the `mod apparent;` / `pub use apparent::...` block, add:

```rust
mod topocentric;

pub use topocentric::{topocentric_position, TopocentricPosition, DIURNAL_ABERRATION_ARCSEC};
```

And extend the provenance re-export line to add `TopocentricProvenance`:

```rust
pub use provenance::{ApparentProvenance, CorrectionSet, TopocentricProvenance, MODEL_SOURCES};
```

- [ ] **Step 7: Run the full apparent crate tests**

Run: `cargo test -p pleiades-apparent`
Expected: PASS (all existing + 4 new topocentric tests + provenance test).

- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-apparent/src/topocentric.rs crates/pleiades-apparent/src/provenance.rs crates/pleiades-apparent/src/apparent.rs crates/pleiades-apparent/src/lib.rs
git commit -m "feat(apparent): topocentric place (diurnal parallax + diurnal aberration)"
```

---

### Task 4: Chart-layer orchestration in `pleiades-core`

Add an opt-in `topocentric` flag to `ChartRequest`, compute local apparent sidereal time, and call `topocentric_position()` after `apparent_position()`.

**Files:**
- Modify: `crates/pleiades-core/src/chart/request.rs` (add `topocentric: bool` field + `with_topocentric` builder + default)
- Modify: `crates/pleiades-core/src/chart/mod.rs` (compute LAST; call topocentric; error handling)
- Modify: `crates/pleiades-core/src/chart/placement.rs` (carry optional topocentric provenance, if placement stores apparent provenance)
- Test: `crates/pleiades-core/src/chart/mod.rs` (`#[cfg(test)]`)

**Interfaces:**
- Consumes: `pleiades_apparent::{topocentric_position, TopocentricProvenance}`, `pleiades_time::{ut1_jd_from_tt, gmst_degrees}`, `pleiades_apparent::nutation::{nutation, mean_obliquity_degrees}`.
- Produces: `ChartRequest.topocentric: bool`, `ChartRequest::with_topocentric(self, bool) -> Self`.

- [ ] **Step 1: Add the request field and builder (write the change)**

In `crates/pleiades-core/src/chart/request.rs`, add to the `ChartRequest` struct (after `apparentness`):

```rust
    /// When true, apply the opt-in chart-layer topocentric correction (diurnal
    /// parallax + diurnal aberration) using `observer`. Requires `observer` to be
    /// set, apparent mode, and bodies with a known geocentric distance.
    pub topocentric: bool,
```

Set `topocentric: false` in every `ChartRequest` constructor/`Default`/literal in `request.rs` (search the file for `apparentness:` and add `topocentric: false,` adjacent to each). Add the builder near `with_observer`:

```rust
    /// Enables the opt-in chart-layer topocentric correction.
    pub fn with_topocentric(mut self, topocentric: bool) -> Self {
        self.topocentric = topocentric;
        self
    }
```

- [ ] **Step 2: Write the failing integration test**

Add to the `#[cfg(test)]` module in `crates/pleiades-core/src/chart/mod.rs` (reuse the existing test backend/`test_support` helpers in that module — match the construction already used by neighboring tests):

```rust
#[test]
fn topocentric_moon_differs_from_geocentric() {
    // Build a geocentric apparent Moon chart and a topocentric one for the same
    // instant/observer; the longitudes must differ by the lunar parallax (>0.1°).
    let observer = pleiades_types::ObserverLocation::new(
        pleiades_types::Latitude::from_degrees(40.0),
        pleiades_types::Longitude::from_degrees(-3.7),
        Some(650.0),
    );
    let geocentric = sample_apparent_moon_chart(observer.clone(), false);
    let topocentric = sample_apparent_moon_chart(observer, true);
    let geo_lon = geocentric.placements[0]
        .position
        .ecliptic
        .as_ref()
        .unwrap()
        .longitude
        .degrees();
    let topo_lon = topocentric.placements[0]
        .position
        .ecliptic
        .as_ref()
        .unwrap()
        .longitude
        .degrees();
    let mut diff = (topo_lon - geo_lon).abs();
    if diff > 180.0 {
        diff = 360.0 - diff;
    }
    assert!(diff > 0.1, "lunar parallax {diff}° too small (geo {geo_lon}, topo {topo_lon})");
}
```

Add a `sample_apparent_moon_chart(observer, topocentric: bool) -> ChartSnapshot` helper in the same test module that builds a `ChartRequest` for the Moon at a fixed JD (e.g. `2_451_545.0`), `Apparentness::Apparent`, `.with_observer(observer)`, `.with_topocentric(topocentric)`, runs it through the existing test backend used by the surrounding tests, and returns the snapshot. (Model it on the nearest existing apparent-chart test in this module; reuse its backend constructor.)

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p pleiades-core topocentric_moon_differs_from_geocentric`
Expected: FAIL — topocentric correction not applied; longitudes equal (diff ≈ 0).

- [ ] **Step 4: Implement the orchestration**

In `crates/pleiades-core/src/chart/mod.rs`, inside the per-body placement closure, immediately after the apparent block sets `position.ecliptic` and `position.apparent = Apparentness::Apparent` (around the `Ok(outcome) => { ... }` arm), insert the topocentric stage. Place it **before** the sidereal re-application is finalized so the existing sidereal re-apply still wraps the topocentric longitude. Use:

```rust
// Opt-in chart-layer topocentric correction (diurnal parallax + diurnal aberration).
if request.topocentric {
    let observer = request.observer.as_ref().ok_or_else(|| {
        EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            "topocentric chart requires an observer location",
        )
    })?;
    let jd_tt = request.instant.julian_day.days();
    let jd_ut1 = pleiades_time::ut1_jd_from_tt(jd_tt).map_err(|e| {
        EphemerisError::new(EphemerisErrorKind::InvalidRequest, e.to_string())
    })?;
    let gmst = pleiades_time::gmst_degrees(jd_ut1);
    let nut = pleiades_apparent::nutation::nutation(jd_tt)
        .map_err(map_apparent_error)?;
    let mean_obliquity = pleiades_apparent::nutation::mean_obliquity_degrees(jd_tt);
    let true_obliquity = mean_obliquity + nut.delta_eps_arcsec / 3600.0;
    // Apparent sidereal time = GMST + equation of the equinoxes + east longitude.
    let eq_equinoxes = (nut.delta_psi_arcsec / 3600.0) * true_obliquity.to_radians().cos();
    let last = (gmst + eq_equinoxes + observer.longitude.degrees()).rem_euclid(360.0);
    if let Some(ecliptic) = position.ecliptic.as_mut() {
        let topo = pleiades_apparent::topocentric_position(
            *ecliptic,
            observer,
            last,
            true_obliquity,
        )
        .map_err(|e| map_apparent_error(pleiades_apparent::ApparentLightTimeError::Apparent(e)))?;
        *ecliptic = topo.ecliptic;
        if matches!(request.zodiac_mode, ZodiacMode::Sidereal { .. }) && !native_sidereal {
            ecliptic.longitude =
                sidereal_longitude(ecliptic.longitude, request.instant, &request.zodiac_mode)?;
        }
    }
}
```

(Field names: confirm `nut.delta_eps_arcsec`/`nut.delta_psi_arcsec` against `Nutation` in `pleiades-apparent/src/nutation.rs` and adjust if the obliquity field is named differently.)

- [ ] **Step 5: Reject incompatible requests**

In `crates/pleiades-core/src/chart/mod.rs`, before the body loop, add a guard so an explicit topocentric request with mean mode or no observer fails closed:

```rust
if request.topocentric {
    if matches!(request.apparentness, Apparentness::Mean) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            "topocentric positions require apparent place; remove --mean",
        ));
    }
    if request.observer.is_none() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            "topocentric positions require an observer location",
        ));
    }
}
```

(For a body that is not release-grade / lacks a distance, the topocentric stage's `MissingDistance` error already propagates as a structured error per Step 4.)

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test -p pleiades-core topocentric_moon_differs_from_geocentric`
Expected: PASS.

- [ ] **Step 7: Run the full core test suite for regressions**

Run: `cargo test -p pleiades-core`
Expected: PASS (geocentric/apparent charts unchanged).

- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-core/src/chart/request.rs crates/pleiades-core/src/chart/mod.rs crates/pleiades-core/src/chart/placement.rs
git commit -m "feat(core): opt-in chart-layer topocentric correction"
```

---

### Task 5: CLI `--topocentric` and `--elevation` flags

**Files:**
- Modify: `crates/pleiades-cli/src/commands/chart.rs` (flag parsing, validation, observer elevation, provenance output, usage text)
- Test: `crates/pleiades-cli/src/commands/chart.rs` (`#[cfg(test)]`)

**Interfaces:**
- Consumes: `ChartRequest::with_topocentric`, `ObserverLocation` with elevation, `TopocentricProvenance` (for the output line).
- Produces: CLI flags `--topocentric`, `--elevation <m>`.

- [ ] **Step 1: Write the failing CLI tests**

Add to the `#[cfg(test)]` module in `crates/pleiades-cli/src/commands/chart.rs`:

```rust
#[test]
fn topocentric_requires_observer() {
    let err = render_chart(&["--jd", "2451545.0", "--body", "Moon", "--topocentric"]).unwrap_err();
    assert!(err.contains("observer") || err.contains("--lat"), "got: {err}");
}

#[test]
fn topocentric_conflicts_with_mean() {
    let err = render_chart(&[
        "--jd", "2451545.0", "--body", "Moon", "--lat", "40", "--lon", "-3.7",
        "--topocentric", "--mean",
    ])
    .unwrap_err();
    assert!(err.contains("apparent") || err.contains("--mean"), "got: {err}");
}

#[test]
fn topocentric_moon_emits_provenance_line() {
    let out = render_chart(&[
        "--jd", "2451545.0", "--body", "Moon", "--lat", "40", "--lon", "-3.7",
        "--elevation", "650", "--topocentric",
    ])
    .unwrap();
    assert!(out.contains("topocentric"), "missing topocentric provenance: {out}");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p pleiades-cli topocentric`
Expected: FAIL — `--topocentric`/`--elevation` unknown flags or no provenance line.

- [ ] **Step 3: Parse the new flags**

In `crates/pleiades-cli/src/commands/chart.rs`, alongside the existing `let mut lat`/`let mut lon` declarations, add:

```rust
    let mut elevation: Option<f64> = None;
    let mut topocentric = false;
```

In the argument match loop (near the `"--lat"`/`"--lon"` arms), add:

```rust
            "--elevation" => elevation = Some(parse_f64(iter.next(), "--elevation")?),
            "--topocentric" => topocentric = true,
```

In the `--mean`/apparentness conflict handling, add the topocentric/mean conflict (after the existing mean parsing, or in the post-parse validation block):

```rust
    if topocentric && mean_requested {
        return Err("topocentric positions require apparent place; remove --mean".to_string());
    }
    if topocentric && (lat.is_none() || lon.is_none()) {
        return Err("topocentric positions require both --lat and --lon".to_string());
    }
```

(Use whatever local variable already records `--mean` — match the existing apparentness handling in this file; if it is tracked via the `apparentness` value, compare against `Apparentness::Mean` instead.)

- [ ] **Step 4: Thread elevation and the flag into the request**

In `crates/pleiades-cli/src/commands/chart.rs`, update the observer construction (the `match (lat, lon)` block) to pass `elevation`:

```rust
    let observer = match (lat, lon) {
        (Some(lat), Some(lon)) => Some(ObserverLocation::new(
            Latitude::from_degrees(lat),
            Longitude::from_degrees(lon),
            elevation,
        )),
        (None, None) => None,
        _ => return Err("both --lat and --lon must be provided together".to_string()),
    };
```

After the existing `if let Some(observer) = observer { request = request.with_observer(observer); }`, add:

```rust
    if topocentric {
        request = request.with_topocentric(true);
    }
```

- [ ] **Step 5: Emit the topocentric provenance line**

In the chart-rendering output path (where the per-body `ApparentProvenance` summary line is appended), also append the topocentric provenance line when present. Match the existing apparent-provenance rendering: if the placement carries a `TopocentricProvenance`, push `format!("  {}", topo_prov.summary_line())`. If placement does not yet carry it, render from the position (the `position.apparent == Apparentness::Apparent` and `request.topocentric` both hold) — append a line beginning with `topocentric` derived from `TopocentricProvenance::summary_line()`.

- [ ] **Step 6: Update usage/help text**

In `crates/pleiades-cli/src/commands/chart.rs`, extend the usage string to include `[--elevation <m>]` next to `--lat/--lon` and `[--topocentric]` next to the apparentness flags, with a sentence: "`--topocentric` applies diurnal parallax + diurnal aberration for the `--lat`/`--lon`/`--elevation` observer; requires apparent mode." Also update `crates/pleiades-cli/src/help.rs` if it carries a separate chart synopsis (grep for `--apparent` there).

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo test -p pleiades-cli topocentric`
Expected: PASS (3 tests).

- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-cli/src/commands/chart.rs crates/pleiades-cli/src/help.rs
git commit -m "feat(cli): --topocentric and --elevation flags with provenance output"
```

---

### Task 6: Horizons topocentric golden gate

Commit Horizons topocentric goldens and a fail-closed test asserting our output matches and is genuinely non-geocentric. Mirror the existing apparent goldens gate.

**Files:**
- Inspect first: the existing apparent goldens test/fixture (grep `apparent` goldens) to copy its structure and tolerance idiom.
- Create: `crates/pleiades-core/tests/topocentric_goldens.rs` (or alongside the existing apparent goldens test, matching its location)
- Create: golden fixture file next to the existing apparent goldens (same directory/format).

**Interfaces:**
- Consumes: the public chart API used by the existing apparent goldens test.

- [ ] **Step 1: Locate the existing apparent goldens gate**

Run: `grep -rl "apparent" crates/pleiades-core/tests crates/pleiades-jpl/tests 2>/dev/null; grep -rln "Horizons\|goldens\|apparent-of-date" crates/*/tests`
Read the apparent goldens test + fixture it finds. Copy its observer-less structure; the topocentric version adds a fixed observer.

- [ ] **Step 2: Generate Horizons topocentric goldens**

Obtain Horizons **topocentric** ecliptic-of-date longitude/latitude for a fixed observer (e.g. lat `+40.4168`, lon `-3.7038`, elev `650 m`, Madrid) at 3–4 epochs spanning 1900–2100, for: Moon, Sun, Mars, and one release-grade asteroid (Eros or Ceres). Record them in the same fixture format the apparent goldens use (JD, body, expected lon, expected lat). Commit the fixture. Document the Horizons query parameters in a header comment (center = the topocentric site, `CSV`, apparent, ecliptic of date).

- [ ] **Step 3: Write the failing golden test**

Create the test mirroring the apparent goldens test, but constructing each `ChartRequest` with `.with_observer(<fixed site>)` and `.with_topocentric(true)`. Assert per body:

```rust
// Tolerances: Moon a few arcsec; Sun/planets/asteroid sub-arcsec.
let tol_deg = if body_is_moon { 5.0 / 3600.0 } else { 1.0 / 3600.0 };
assert!((computed_lon - expected_lon).abs() < tol_deg, "{body} lon {computed_lon} vs {expected_lon}");
assert!((computed_lat - expected_lat).abs() < tol_deg, "{body} lat");
```

And add the no-silent-fallback assertion for the Moon: compute the same Moon chart geocentric (`.with_topocentric(false)`) and assert the topocentric longitude differs by > 0.1°:

```rust
assert!((moon_topo_lon - moon_geo_lon).abs() > 0.1, "Moon topocentric must differ from geocentric");
```

- [ ] **Step 4: Run test to verify it fails, then passes**

Run: `cargo test -p pleiades-core --test topocentric_goldens`
Expected first: FAIL (fixture present, values not yet matched if any wiring bug) → fix until PASS. The non-geocentric assertion guards against a silent fallback regression.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-core/tests/topocentric_goldens.rs crates/pleiades-core/tests/<fixture-file>
git commit -m "test(topocentric): Horizons topocentric goldens gate (non-geocentric assertion)"
```

---

### Task 7: Update gated policy strings, capability surfaces, and docs

Flip every canonical "topocentric ... unsupported" assertion to "chart-layer topocentric supported (opt-in); native backend topocentric unsupported", in lockstep so the policy gates pass.

**Files:**
- Modify: `crates/pleiades-backend/src/policy/mod.rs` (`CURRENT_OBSERVER_POLICY_SUMMARY_TEXT`)
- Modify: `crates/pleiades-core/src/compatibility/mod.rs` (`CURRENT_COMPATIBILITY_PROFILE_SUMMARY` — the "topocentric body positions remain unsupported" clause)
- Modify: `crates/pleiades-backend/src/capabilities.rs` if it carries unsupported-mode prose (grep)
- Modify: `crates/pleiades-apparent/src/policy.rs` if the apparent policy summary mentions topocentric (grep)
- Modify: `PLAN.md`, `README.md`, `plan/stages/04-advanced-request-modes.md`

**Interfaces:** none (strings + docs).

- [ ] **Step 1: Find every assertion to update**

Run:
```bash
grep -rn "topocentric body positions remain unsupported\|topocentric positions are not implemented\|topocentric.*unsupported" crates docs PLAN.md README.md
```
Note: `topocentric positions are not implemented` in `pleiades-backend` is the **backend-boundary native** rejection and MUST stay (native backend topocentric is still unsupported). Only the *chart-layer unsupported* prose changes.

- [ ] **Step 2: Update the observer policy summary**

In `crates/pleiades-backend/src/policy/mod.rs`, change `CURRENT_OBSERVER_POLICY_SUMMARY_TEXT` so it states chart-layer topocentric (diurnal parallax + diurnal aberration) is supported as opt-in, while backends remain geocentric and native-backend topocentric stays unsupported. Keep it a single line (the gate rejects line breaks/padding).

- [ ] **Step 3: Update the compatibility profile clause**

In `crates/pleiades-core/src/compatibility/mod.rs`, change the clause "topocentric body positions remain unsupported" within `CURRENT_COMPATIBILITY_PROFILE_SUMMARY` to "chart-layer topocentric body positions are supported as an opt-in correction (diurnal parallax + diurnal aberration); native sidereal backend output remains unsupported". Leave the rest of the (very long) string intact.

- [ ] **Step 4: Run the policy gates**

Run: `cargo test -p pleiades-backend policy && cargo test -p pleiades-core compatibility`
Expected: any test asserting the old "unsupported" substring now fails — update those test expectations to the new wording, then PASS.

- [ ] **Step 5: Update PLAN/README/stage docs**

- `PLAN.md`: in "Important current limits" and "Current priority", remove topocentric from remaining Phase 4 work; state Phase 4's only remaining item is native sidereal (the deliberate non-goal). Update the bottom `Status:` line.
- `README.md`: update the capability/claims section (grep `topocentric`) to note opt-in topocentric support.
- `plan/stages/04-advanced-request-modes.md`: move the "Implement topocentric body positions" bullet from "Remaining implementation work" into a new "Completed: topocentric body positions (chart layer)" section describing parallax + diurnal aberration, opt-in flag, and the Horizons golden gate.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-backend/src/policy/mod.rs crates/pleiades-core/src/compatibility/mod.rs PLAN.md README.md plan/stages/04-advanced-request-modes.md
git commit -m "docs(policy): chart-layer topocentric is supported (opt-in); update gates and PLAN"
```

---

### Task 8: Full workspace verification

**Files:** none (verification).

- [ ] **Step 1: Format, lint, test**

Run:
```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --workspace
```
Expected: clean format, no clippy warnings, all tests pass.

- [ ] **Step 2: Confirm geocentric output is unchanged**

Run the existing chart/apparent goldens explicitly and confirm no regressions:
```bash
cargo test -p pleiades-core apparent
cargo test -p pleiades-core --test '*golden*'
```
Expected: PASS, unchanged.

- [ ] **Step 3: Commit any fmt-only changes**

```bash
git add -A
git commit -m "style: cargo fmt for topocentric feature" || echo "nothing to format"
```

---

## Self-Review Notes

- **Spec coverage:** parallax + diurnal aberration (Tasks 2–3); pure crate / caller-supplied scalars (Tasks 1, 3, 4); opt-in flag, geocentric default preserved (Tasks 4–5); two-observer distinction / backend stays geocentric (Task 4 — uses `observer`, never `body_observer`); error-vs-fallback for distance-less bodies (Task 3 `MissingDistance` + Task 4 propagation); Horizons golden gate with non-geocentric assertion (Task 6); policy/PLAN/README/stage-doc updates (Task 7); CLI `--topocentric`/`--elevation` (Task 5); out-of-scope items untouched (native backend rejection preserved, Task 7 Step 1 note).
- **Reused existing errors** `MissingDistance` / `NonFiniteCorrection { stage }` per the global constraint (no new variants).
- **Type consistency:** `topocentric_position(EclipticCoordinates, &ObserverLocation, f64, f64) -> Result<TopocentricPosition, ApparentPlaceError>` is referenced identically in Tasks 3, 4, 6; `with_topocentric(bool)` in Tasks 4, 5; `gmst_degrees`/`ut1_jd_from_tt` in Tasks 1, 4.
- **Open verification point:** the `Nutation` obliquity field name (`delta_eps_arcsec`) and the apparent goldens fixture format must be confirmed against the code during Tasks 4 and 6 (flagged inline).
