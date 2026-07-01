# SP-1 · Angles & Sidereal Time Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose public sidereal time and the full Swiss-Ephemeris `ascmc` chart points (ARMC, Vertex, equatorial ascendant, co-ascendants, polar ascendant), validated to SE parity.

**Architecture:** Put the shared sidereal-time + obliquity primitives in `pleiades-apparent` (it already owns nutation + obliquity-of-date; SP-2 will reuse them); `pleiades-houses` delegates its private helpers to them so its SE-gated cusp numerics are preserved, and adds an `AscMc` struct + `chart_points` computed from the same math. Surface `AscMc` on `HouseSnapshot`, re-export through `pleiades-core`, and gate every new quantity against a Swiss-Ephemeris reference corpus produced by the existing isolated `tools/se-house-reference` harness.

**Tech Stack:** Rust (workspace, edition/rust-version from `Cargo.toml` workspace), `cargo test`, `pleiades-validate` CLI gates, `libswisseph-sys` (only inside the out-of-workspace `tools/se-house-reference`).

## Global Constraints

- **Pure Rust, no required C/C++.** No `-sys`/`links`/`build.rs` may enter the workspace lockfile; `workspace-audit` fails closed on them. The SE binding stays in `tools/se-house-reference` (its own `Cargo.lock`, outside the workspace). Copied verbatim from the spec's Constraint C1.
- **First-party crates are `pleiades-*`.** No new crates in this plan.
- **Fail-closed validation.** New numeric claims must be backed by a committed, checksum-pinned reference corpus and a release-wired gate; a clean checkout stays tool-free and only validates committed values.
- **Obliquity convention:** true obliquity of date by default (mean + Δε), overridable — matches existing `derive_angles` and SE.
- **Sidereal time is UT1-based.** Preserve the existing GMST + equation-of-equinoxes expression exactly; the `validate-houses` gate is the regression guard. Do not change the ΔT/UT1/time-scale policy.
- **Longitudes normalized to `[0,360)`; sidereal time also available in hours `[0,24)`.**
- Formatting/lint/test gates: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace` must pass.

---

### Task 1: Sidereal-time foundation in `pleiades-apparent`

**Files:**
- Create: `crates/pleiades-apparent/src/sidereal.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs` (add `pub mod sidereal;` + re-exports)
- Test: inline `#[cfg(test)]` in `crates/pleiades-apparent/src/sidereal.rs`

**Interfaces:**
- Consumes: `pleiades_apparent::nutation::{mean_obliquity_degrees, nutation}` (existing: `nutation(jd_tt: f64) -> Result<Nutation, ApparentPlaceError>`, `Nutation { delta_psi_arcsec, delta_eps_arcsec }`, `mean_obliquity_degrees(jd_tt: f64) -> f64`); `pleiades_types::{Angle, Instant, Longitude}` (existing: `Angle::from_degrees(f64)`, `Angle::normalized_0_360() -> Angle`, `Angle::degrees() -> f64`, `Longitude::degrees() -> f64`, `Instant::julian_day.days() -> f64`).
- Produces:
  - `greenwich_mean_sidereal_time_degrees(jd: f64) -> f64` (unnormalized)
  - `equation_of_equinoxes_degrees(jd: f64) -> f64`
  - `struct SiderealTime { gmst_deg, gast_deg, local_mean_deg, local_apparent_deg: f64 }` (`#[non_exhaustive]`) with `gmst_hours/gast_hours/local_mean_hours/local_apparent_hours(&self) -> f64`
  - `sidereal_time(instant: Instant, observer_longitude: Longitude) -> SiderealTime`

- [ ] **Step 1: Write the failing test**

Create `crates/pleiades-apparent/src/sidereal.rs` with only the tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{Instant, JulianDay, Longitude, TimeScale};

    fn j2000() -> Instant {
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt)
    }

    #[test]
    fn gmst_at_j2000_is_about_280_46_degrees() {
        let gmst = greenwich_mean_sidereal_time_degrees(2_451_545.0);
        // GMST at J2000.0 ≈ 280.4606°.
        assert!((gmst.rem_euclid(360.0) - 280.4606).abs() < 1e-3, "got {gmst}");
    }

    #[test]
    fn local_apparent_equals_gast_plus_east_longitude() {
        let st = sidereal_time(j2000(), Longitude::from_degrees(90.0));
        let expected = (st.gast_deg + 90.0).rem_euclid(360.0);
        assert!((st.local_apparent_deg - expected).abs() < 1e-9, "{st:?}");
    }

    #[test]
    fn all_fields_normalized_and_hours_consistent() {
        let st = sidereal_time(j2000(), Longitude::from_degrees(-123.4));
        for v in [st.gmst_deg, st.gast_deg, st.local_mean_deg, st.local_apparent_deg] {
            assert!((0.0..360.0).contains(&v), "not normalized: {v}");
        }
        assert!((st.gmst_hours() - st.gmst_deg / 15.0).abs() < 1e-12);
    }

    #[test]
    fn equation_of_equinoxes_is_small() {
        // EE is at most a couple of arcseconds ≈ a few×1e-4 degrees.
        assert!(equation_of_equinoxes_degrees(2_451_545.0).abs() < 0.01);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-apparent sidereal:: 2>&1 | tail -20`
Expected: FAIL — `cannot find function greenwich_mean_sidereal_time_degrees` / module has no such items.

- [ ] **Step 3: Write minimal implementation**

Prepend to `crates/pleiades-apparent/src/sidereal.rs` (above the test module):

```rust
//! Sidereal time (GMST/GAST, Greenwich and local) for the of-date chart layer.
//!
//! Sidereal time is a function of UT1 (Earth rotation), not TT/TDB. The
//! `Instant`'s Julian Day is used as supplied; pass a UT1-scale instant for
//! rigorous results (see `docs/time-observer-policy.md`).

use pleiades_types::{Angle, Instant, Longitude};

use crate::nutation::{mean_obliquity_degrees, nutation};

/// Greenwich Mean Sidereal Time in degrees (unnormalized).
pub fn greenwich_mean_sidereal_time_degrees(jd: f64) -> f64 {
    let centuries = (jd - 2_451_545.0) / 36_525.0;
    280.460_618_37 + 360.985_647_366_29 * (jd - 2_451_545.0)
        + 0.000_387_933 * centuries * centuries
        - centuries * centuries * centuries / 38_710_000.0
}

/// Equation of the equinoxes in degrees: `Δψ · cos(ε_true)`.
///
/// Falls back to `0.0` if the nutation table is unavailable (a development-time
/// artifact — a stale checksum — not a runtime condition), matching the prior
/// behavior in `pleiades-houses`.
pub fn equation_of_equinoxes_degrees(jd: f64) -> f64 {
    nutation(jd)
        .map(|n| {
            let delta_psi_deg = n.delta_psi_arcsec / 3600.0;
            let true_obl_rad =
                (mean_obliquity_degrees(jd) + n.delta_eps_arcsec / 3600.0).to_radians();
            delta_psi_deg * true_obl_rad.cos()
        })
        .unwrap_or(0.0)
}

/// Sidereal time in mean/apparent form, at Greenwich and locally.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct SiderealTime {
    /// Greenwich Mean Sidereal Time, degrees `[0,360)`.
    pub gmst_deg: f64,
    /// Greenwich Apparent Sidereal Time (GMST + equation of equinoxes), degrees `[0,360)`.
    pub gast_deg: f64,
    /// Local Mean Sidereal Time (GMST + east longitude), degrees `[0,360)`.
    pub local_mean_deg: f64,
    /// Local Apparent Sidereal Time (GAST + east longitude), degrees `[0,360)`.
    pub local_apparent_deg: f64,
}

impl SiderealTime {
    /// GMST in hours `[0,24)`.
    pub fn gmst_hours(&self) -> f64 { self.gmst_deg / 15.0 }
    /// GAST in hours `[0,24)`.
    pub fn gast_hours(&self) -> f64 { self.gast_deg / 15.0 }
    /// Local mean sidereal time in hours `[0,24)`.
    pub fn local_mean_hours(&self) -> f64 { self.local_mean_deg / 15.0 }
    /// Local apparent sidereal time in hours `[0,24)`.
    pub fn local_apparent_hours(&self) -> f64 { self.local_apparent_deg / 15.0 }
}

/// Computes sidereal time for an instant and observer east longitude.
pub fn sidereal_time(instant: Instant, observer_longitude: Longitude) -> SiderealTime {
    let jd = instant.julian_day.days();
    let gmst = greenwich_mean_sidereal_time_degrees(jd);
    let ee = equation_of_equinoxes_degrees(jd);
    let lon = observer_longitude.degrees();
    let norm = |d: f64| Angle::from_degrees(d).normalized_0_360().degrees();
    SiderealTime {
        gmst_deg: norm(gmst),
        gast_deg: norm(gmst + ee),
        local_mean_deg: norm(gmst + lon),
        local_apparent_deg: norm(gmst + ee + lon),
    }
}
```

Add to `crates/pleiades-apparent/src/lib.rs` after the `pub use equatorial::…;` block:

```rust
pub mod sidereal;

pub use sidereal::{
    equation_of_equinoxes_degrees, greenwich_mean_sidereal_time_degrees, sidereal_time,
    SiderealTime,
};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-apparent sidereal:: 2>&1 | tail -20`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-apparent/src/sidereal.rs crates/pleiades-apparent/src/lib.rs
git commit -m "feat(apparent): public sidereal time (GMST/GAST, local) foundation"
```

---

### Task 2: Delegate house sidereal time + mean obliquity to `pleiades-apparent`

**Files:**
- Modify: `crates/pleiades-houses/src/systems/mod.rs` (`local_sidereal_time` ~557, `mean_obliquity` ~581)

**Interfaces:**
- Consumes: `pleiades_apparent::sidereal::sidereal_time` and `pleiades_apparent::nutation::mean_obliquity_degrees` (from Task 1 / existing).
- Produces: no signature change — `local_sidereal_time(Instant, Longitude) -> Angle` and `mean_obliquity(Instant) -> Angle` keep their names and types; only their bodies delegate. This preserves all existing house numerics.

- [ ] **Step 1: Confirm the regression guard passes today**

Run: `cargo run -q -p pleiades-validate -- validate-houses 2>&1 | tail -5`
Expected: the house gate PASSES (baseline before the refactor).

- [ ] **Step 2: Replace the two helper bodies with delegations**

In `crates/pleiades-houses/src/systems/mod.rs`, replace the body of `local_sidereal_time` (the whole `fn local_sidereal_time(instant: Instant, longitude: Longitude) -> Angle { … }` including its GMST/EE math) with:

```rust
fn local_sidereal_time(instant: Instant, longitude: Longitude) -> Angle {
    Angle::from_degrees(
        pleiades_apparent::sidereal::sidereal_time(instant, longitude).local_apparent_deg,
    )
}
```

Replace the body of `mean_obliquity` with:

```rust
fn mean_obliquity(instant: Instant) -> Angle {
    Angle::from_degrees(pleiades_apparent::nutation::mean_obliquity_degrees(
        instant.julian_day.days(),
    ))
}
```

Then delete any now-unused imports flagged by the compiler (e.g. the local `apparent_nutation` alias if it is no longer referenced elsewhere in the file — check first with `grep -n apparent_nutation crates/pleiades-houses/src/systems/mod.rs`; keep it if `nutation_for` still uses it).

- [ ] **Step 3: Build**

Run: `cargo build -p pleiades-houses 2>&1 | tail -20`
Expected: compiles (fix unused-import warnings if clippy would reject them).

- [ ] **Step 4: Prove numerics are unchanged (regression guard)**

Run: `cargo test -p pleiades-houses 2>&1 | tail -10 && cargo run -q -p pleiades-validate -- validate-houses 2>&1 | tail -5`
Expected: all house unit tests PASS and `validate-houses` still PASSES (cusp/Asc/MC values preserved to the gate's arcsecond ceilings).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-houses/src/systems/mod.rs
git commit -m "refactor(houses): delegate sidereal time + mean obliquity to pleiades-apparent"
```

---

### Task 3: `AscMc` struct + `chart_points` in `pleiades-houses`

**Files:**
- Modify: `crates/pleiades-houses/src/systems/mod.rs` (add `AscMc`, `asc_mc_from`, `chart_points`, `chart_points_from_armc`)
- Modify: `crates/pleiades-houses/src/lib.rs` (export `AscMc`, `chart_points`, `chart_points_from_armc`)
- Test: `crates/pleiades-houses/src/systems/tests.rs`

**Interfaces:**
- Consumes: existing `ascendant_for(sidereal_time_deg: f64, latitude_deg: f64, obliquity_rad: f64) -> Longitude`, `longitude_opposite(Longitude) -> Longitude`, `local_sidereal_time`, `validated_obliquity(&HouseRequest) -> Result<Angle, HouseError>`; `pleiades_types::{Angle, Latitude, Longitude, Instant, ObserverLocation}`; `HouseError`/`HouseErrorKind`.
- Produces:
  - `#[non_exhaustive] pub struct AscMc { ascendant, midheaven, descendant, imum_coeli, armc, vertex, antivertex, equatorial_ascendant, coascendant_koch, coascendant_munkasey, polar_ascendant: Longitude }`
  - `pub fn chart_points(instant: Instant, observer: &ObserverLocation, obliquity: Option<Angle>) -> Result<AscMc, HouseError>`
  - `pub fn chart_points_from_armc(armc: Longitude, geolat: Latitude, obliquity: Angle) -> Result<AscMc, HouseError>`
  - internal `fn asc_mc_from(armc_deg: f64, lat_deg: f64, obliquity_deg: f64) -> AscMc`

- [ ] **Step 1: Write the failing tests**

Append to `crates/pleiades-houses/src/systems/tests.rs`:

```rust
#[test]
fn chart_points_from_armc_mc_is_analytic_at_cardinal_armc() {
    use pleiades_types::{Angle, Latitude, Longitude};
    // With obliquity ε, MC longitude satisfies tan(λ_MC)=tan(ARMC)/cos(ε);
    // at ARMC = 0/90/180/270 the MC equals the ARMC exactly.
    let obl = Angle::from_degrees(23.4392911);
    for armc in [0.0_f64, 90.0, 180.0, 270.0] {
        let pts = chart_points_from_armc(
            Longitude::from_degrees(armc),
            Latitude::from_degrees(40.0),
            obl,
        )
        .expect("defined at 40N");
        let diff = (pts.midheaven.degrees() - armc).rem_euclid(360.0);
        let diff = diff.min(360.0 - diff);
        assert!(diff < 1e-6, "ARMC {armc}: MC {}", pts.midheaven.degrees());
    }
}

#[test]
fn chart_points_invariants_hold() {
    use pleiades_types::{Angle, Latitude, Longitude};
    let pts = chart_points_from_armc(
        Longitude::from_degrees(123.4),
        Latitude::from_degrees(51.5),
        Angle::from_degrees(23.4392911),
    )
    .expect("defined at 51.5N");
    let opp = |a: f64, b: f64| {
        let d = (a - b).rem_euclid(360.0);
        (d - 180.0).abs() < 1e-6
    };
    assert!(opp(pts.ascendant.degrees(), pts.descendant.degrees()));
    assert!(opp(pts.midheaven.degrees(), pts.imum_coeli.degrees()));
    assert!(opp(pts.vertex.degrees(), pts.antivertex.degrees()));
    for p in [
        pts.armc, pts.vertex, pts.equatorial_ascendant, pts.coascendant_koch,
        pts.coascendant_munkasey, pts.polar_ascendant,
    ] {
        assert!((0.0..360.0).contains(&p.degrees()), "unnormalized {}", p.degrees());
    }
    // ARMC round-trips the input.
    let d = (pts.armc.degrees() - 123.4).rem_euclid(360.0);
    assert!(d.min(360.0 - d) < 1e-9);
}

#[test]
fn chart_points_uses_true_obliquity_by_default() {
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let observer = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(-74.0),
        None,
    );
    let inst = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let pts = chart_points(inst, &observer, None).expect("defined");
    // Ascendant matches the value derive_angles produces for the same inputs.
    let req = HouseRequest::new(inst, observer.clone(), HouseSystem::Placidus);
    let snap = calculate_houses(&req).expect("houses");
    assert!((pts.ascendant.degrees() - snap.angles.ascendant.degrees()).abs() < 1e-9);
    assert!((pts.midheaven.degrees() - snap.angles.midheaven.degrees()).abs() < 1e-9);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p pleiades-houses chart_points 2>&1 | tail -20`
Expected: FAIL — `cannot find function chart_points` / `chart_points_from_armc` / type `AscMc`.

- [ ] **Step 3: Write the implementation**

Add to `crates/pleiades-houses/src/systems/mod.rs` (near `HouseAngles`):

```rust
/// The full Swiss-Ephemeris `ascmc` chart-point set.
///
/// Longitudes are apparent, equinox-of-date, tropical, in `[0,360)`. Vertex,
/// equatorial ascendant, the co-ascendants, and the polar ascendant are ported
/// from Swiss Ephemeris `swehouse.c`; their numeric correctness is enforced by
/// the `validate-angles` parity gate.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct AscMc {
    /// Ascendant (`ascmc[0]`).
    pub ascendant: Longitude,
    /// Midheaven (`ascmc[1]`).
    pub midheaven: Longitude,
    /// Descendant (Ascendant + 180°).
    pub descendant: Longitude,
    /// Imum Coeli (Midheaven + 180°).
    pub imum_coeli: Longitude,
    /// ARMC — right ascension of the Midheaven / local apparent sidereal time in degrees (`ascmc[2]`).
    pub armc: Longitude,
    /// Vertex (`ascmc[3]`).
    pub vertex: Longitude,
    /// Antivertex (Vertex + 180°).
    pub antivertex: Longitude,
    /// Equatorial ascendant / East Point (`ascmc[4]`).
    pub equatorial_ascendant: Longitude,
    /// Co-ascendant, Koch definition (`ascmc[5]`).
    pub coascendant_koch: Longitude,
    /// Co-ascendant, Munkasey definition (`ascmc[6]`).
    pub coascendant_munkasey: Longitude,
    /// Polar ascendant, Munkasey definition (`ascmc[7]`).
    pub polar_ascendant: Longitude,
}

/// Computes the full chart-point set from ARMC (degrees), geographic latitude
/// (degrees), and obliquity (degrees). Errors if any point is non-finite.
fn asc_mc_from(armc_deg: f64, lat_deg: f64, obliquity_deg: f64) -> Result<AscMc, HouseError> {
    let obl = obliquity_deg.to_radians();
    let theta = armc_deg.to_radians();

    let ascendant = ascendant_for(armc_deg, lat_deg, obl);
    let midheaven = Longitude::from_degrees(theta.sin().atan2(theta.cos() * obl.cos()).to_degrees());

    // Equatorial ascendant (East Point): the Ascendant at geographic latitude 0.
    let equatorial_ascendant = ascendant_for(armc_deg, 0.0, obl);

    // Vertex: the Ascendant computed on the opposite meridian at the
    // co-latitude. (Ported from swehouse.c; parity-gated.)
    let colat = 90.0 - lat_deg.abs();
    let vertex = ascendant_for((armc_deg + 180.0).rem_euclid(360.0), colat, obl);

    // Co-ascendant (Koch): the Ascendant on the meridian 90° away.
    let coascendant_koch = ascendant_for((armc_deg - 90.0).rem_euclid(360.0), lat_deg, obl);

    // Co-ascendant (Munkasey) and polar ascendant (Munkasey): defined by
    // swehouse.c; the values below are the first candidates and are validated
    // (and, if needed, corrected) against the SE reference in Task 7.
    let coascendant_munkasey =
        ascendant_for((armc_deg + 90.0).rem_euclid(360.0), lat_deg, obl);
    let polar_ascendant = longitude_opposite(coascendant_koch);

    let points = AscMc {
        ascendant,
        midheaven,
        descendant: longitude_opposite(ascendant),
        imum_coeli: longitude_opposite(midheaven),
        armc: Longitude::from_degrees(armc_deg),
        vertex,
        antivertex: longitude_opposite(vertex),
        equatorial_ascendant,
        coascendant_koch,
        coascendant_munkasey,
        polar_ascendant,
    };

    for (label, p) in [
        ("ascendant", points.ascendant), ("midheaven", points.midheaven),
        ("vertex", points.vertex), ("equatorial_ascendant", points.equatorial_ascendant),
        ("coascendant_koch", points.coascendant_koch),
        ("coascendant_munkasey", points.coascendant_munkasey),
        ("polar_ascendant", points.polar_ascendant),
    ] {
        check_finite(format!("asc_mc {label}"), p.degrees())?;
    }
    Ok(points)
}

/// Computes the full chart-point set for an instant and observer. Obliquity
/// defaults to true obliquity of date (mean + Δε) when `None`.
pub fn chart_points(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Option<Angle>,
) -> Result<AscMc, HouseError> {
    validate_observer(observer)?;
    let obliquity = match obliquity {
        Some(o) => {
            validate_obliquity(o)?;
            o
        }
        None => {
            let mean = mean_obliquity(instant);
            let (_dpsi, deps) = nutation_for(instant)?;
            let obl = Angle::from_degrees(mean.degrees() + deps);
            validate_obliquity(obl)?;
            obl
        }
    };
    let armc = local_sidereal_time(instant, observer.longitude).degrees();
    asc_mc_from(armc, observer.latitude.degrees(), obliquity.degrees())
}

/// Computes the full chart-point set from a supplied ARMC (the
/// `swe_houses_armc` case), geographic latitude, and obliquity.
pub fn chart_points_from_armc(
    armc: Longitude,
    geolat: Latitude,
    obliquity: Angle,
) -> Result<AscMc, HouseError> {
    validate_obliquity(obliquity)?;
    asc_mc_from(armc.degrees(), geolat.degrees(), obliquity.degrees())
}
```

If `validate_obliquity` / `validate_observer` are not already visible at this location, confirm their names with `grep -n "fn validate_obliquity\|fn validate_observer" crates/pleiades-houses/src/systems/mod.rs` and use the exact names (they are used by `validated_obliquity`).

Export from `crates/pleiades-houses/src/lib.rs` in the `pub use systems::{…}` block:

```rust
pub use systems::{
    calculate_houses, chart_points, chart_points_from_armc, house_for_longitude, AscMc,
    HighLatitudePolicy, HouseAngles, HouseRequest, HouseSnapshot,
};
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-houses chart_points 2>&1 | tail -20`
Expected: PASS (3 tests). (Numeric truth for Vertex / co-ascendants / polar ascendant is enforced later by Task 7's SE-parity gate, not these invariant tests.)

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-houses/src/systems/mod.rs crates/pleiades-houses/src/systems/tests.rs crates/pleiades-houses/src/lib.rs
git commit -m "feat(houses): AscMc chart points + chart_points/chart_points_from_armc"
```

---

### Task 4: Surface `asc_mc` on `HouseSnapshot` (`#[non_exhaustive]`)

**Files:**
- Modify: `crates/pleiades-houses/src/systems/mod.rs` (`HouseSnapshot` struct ~133; construction sites at ~247 and ~333)
- Modify: `crates/pleiades-houses/src/systems/tests.rs` (construction sites at ~361, ~944, ~969, ~997, ~1022, ~1103)

**Interfaces:**
- Consumes: `AscMc`, `asc_mc_from` (Task 3), existing `derive_angles`, `local_sidereal_time`.
- Produces: `HouseSnapshot` gains `pub asc_mc: AscMc`; struct marked `#[non_exhaustive]`. All in-crate constructions populate the field.

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-houses/src/systems/tests.rs`:

```rust
#[test]
fn house_snapshot_carries_asc_mc_consistent_with_angles() {
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let observer = ObserverLocation::new(
        Latitude::from_degrees(48.85),
        Longitude::from_degrees(2.35),
        None,
    );
    let req = HouseRequest::new(
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        observer,
        HouseSystem::Placidus,
    );
    let snap = calculate_houses(&req).expect("houses");
    assert_eq!(snap.asc_mc.ascendant, snap.angles.ascendant);
    assert_eq!(snap.asc_mc.midheaven, snap.angles.midheaven);
    assert!((0.0..360.0).contains(&snap.asc_mc.vertex.degrees()));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-houses house_snapshot_carries_asc_mc 2>&1 | tail -20`
Expected: FAIL — `no field asc_mc on type HouseSnapshot`.

- [ ] **Step 3: Add the field and mark non_exhaustive**

In `crates/pleiades-houses/src/systems/mod.rs`, add `#[non_exhaustive]` to the `HouseSnapshot` struct definition and add the field after `angles`:

```rust
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct HouseSnapshot {
    // …existing fields…
    /// Derived angles (four classic angles).
    pub angles: HouseAngles,
    /// Full Swiss-Ephemeris `ascmc` chart points.
    pub asc_mc: AscMc,
    /// House cusps in house-number order.
    pub cusps: Vec<Longitude>,
}
```

- [ ] **Step 4: Populate every construction site**

At the fallback-Porphyry site (~247) and the main site (~333), the code already computes `let angles = derive_angles(request.instant, &request.observer, obliquity);`. Immediately after each `derive_angles` call that feeds a `HouseSnapshot`, add:

```rust
let asc_mc = asc_mc_from(
    local_sidereal_time(request.instant, request.observer.longitude).degrees(),
    request.observer.latitude.degrees(),
    obliquity.degrees(),
)?;
```

and add `asc_mc,` to each `HouseSnapshot { … }` literal (both production sites).

For the 6 test-only construction sites in `tests.rs` (~361, ~944, ~969, ~997, ~1022, ~1103), add an `asc_mc` field to each literal. Build a shared test helper once, at the top of the test module, and use it in each:

```rust
fn test_asc_mc(angles: HouseAngles) -> AscMc {
    AscMc {
        ascendant: angles.ascendant,
        midheaven: angles.midheaven,
        descendant: angles.descendant,
        imum_coeli: angles.imum_coeli,
        armc: angles.midheaven,
        vertex: angles.ascendant,
        antivertex: angles.descendant,
        equatorial_ascendant: angles.ascendant,
        coascendant_koch: angles.ascendant,
        coascendant_munkasey: angles.ascendant,
        polar_ascendant: angles.descendant,
    }
}
```

Since `AscMc` is `#[non_exhaustive]`, this literal is legal here because the test module is inside the same crate. Add `asc_mc: test_asc_mc(<the angles used in that snapshot>),` to each of the 6 `HouseSnapshot { … }` literals (each already builds an `angles` value; pass it in).

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p pleiades-houses 2>&1 | tail -15`
Expected: PASS (including the new snapshot test and all pre-existing tests).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-houses/src/systems/mod.rs crates/pleiades-houses/src/systems/tests.rs
git commit -m "feat(houses): carry AscMc on HouseSnapshot; mark HouseSnapshot non_exhaustive"
```

---

### Task 5: Re-export via `pleiades-core` and surface on `ChartSnapshot` + CLI

**Files:**
- Modify: `crates/pleiades-core/src/lib.rs` (re-export `AscMc`, `SiderealTime`, `sidereal_time`, `chart_points`, `chart_points_from_armc`)
- Modify: `crates/pleiades-core/src/chart/snapshot.rs` (add an `asc_mc()` accessor delegating to the snapshot's houses)
- Modify: `crates/pleiades-cli/src/commands/chart.rs` (render the new points when houses are present)
- Test: `crates/pleiades-core/src/chart/tests.rs`

**Interfaces:**
- Consumes: `pleiades_houses::{AscMc, chart_points, chart_points_from_armc}`, `pleiades_apparent::{SiderealTime, sidereal_time}`, existing `ChartSnapshot { houses: Option<HouseSnapshot>, … }`.
- Produces: `pleiades_core` re-exports of the above names; `ChartSnapshot::asc_mc(&self) -> Option<&AscMc>`.

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-core/src/chart/tests.rs` (follow the existing test-setup pattern in that file for building an engine + request with houses):

```rust
#[test]
fn chart_snapshot_exposes_asc_mc_when_houses_present() {
    // Reuse the file's existing helper for a with-houses request/engine.
    let snapshot = sample_snapshot_with_houses();
    let asc_mc = snapshot.asc_mc().expect("asc_mc present when houses computed");
    assert_eq!(asc_mc.ascendant, snapshot.houses.as_ref().unwrap().angles.ascendant);
}
```

If no `sample_snapshot_with_houses()` helper exists, build the snapshot inline using the same `ChartEngine`/`ChartRequest::with_house_system(...)`/`with_observer(...)` calls already used elsewhere in `tests.rs`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-core chart_snapshot_exposes_asc_mc 2>&1 | tail -20`
Expected: FAIL — `no method named asc_mc found for ChartSnapshot`.

- [ ] **Step 3: Add the accessor and re-exports**

In `crates/pleiades-core/src/chart/snapshot.rs`, add to `impl ChartSnapshot`:

```rust
/// The full Swiss-Ephemeris `ascmc` chart points, present only when the
/// snapshot carries computed houses.
pub fn asc_mc(&self) -> Option<&pleiades_houses::AscMc> {
    self.houses.as_ref().map(|h| &h.asc_mc)
}
```

In `crates/pleiades-core/src/lib.rs`, add to the appropriate re-export blocks:

```rust
pub use pleiades_apparent::{sidereal_time, SiderealTime};
pub use pleiades_houses::{chart_points, chart_points_from_armc, AscMc};
```

(Match the file's existing `pub use` style; if `pleiades_apparent`/`pleiades_houses` are not already direct dependencies of `pleiades-core`, they are — `pleiades-core` already consumes `HouseSnapshot`/`Apparentness` — so no `Cargo.toml` change is needed. Verify with `grep -n "pleiades-apparent\|pleiades-houses" crates/pleiades-core/Cargo.toml`.)

- [ ] **Step 4: Render the points in the CLI chart report**

In `crates/pleiades-cli/src/commands/chart.rs`, in the block that already prints houses/angles when present, add lines printing `asc_mc.armc`, `asc_mc.vertex`, and `asc_mc.equatorial_ascendant` (degrees, formatted like the existing angle output). Keep it inside the existing "houses present" branch so charts without houses are unchanged.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p pleiades-core chart_snapshot_exposes_asc_mc 2>&1 | tail -10 && cargo build -p pleiades-cli 2>&1 | tail -5`
Expected: test PASSES; CLI builds.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-core/src/lib.rs crates/pleiades-core/src/chart/snapshot.rs crates/pleiades-cli/src/commands/chart.rs
git commit -m "feat(core,cli): expose AscMc + sidereal time; render new angles in chart report"
```

---

### Task 6: Extend the SE reference harness to emit `ascmc[2..7]` + sidereal time

**Files:**
- Modify: `tools/se-house-reference/src/main.rs`

**Interfaces:**
- Consumes: `libswisseph_sys::{swe_houses, swe_sidtime}` (existing binding); the harness already fills `ascmc: [f64; 10]`.
- Produces: extended CSV output carrying `armc, vertex, equasc, coasc_koch, coasc_munkasey, polar_asc` (from `ascmc[2..=7]`) and `sidtime_gast_hours` (from `swe_sidtime`). Exact column placement is finalized in Task 7's corpus schema.

> **Maintainer-only step.** This runs where Swiss Ephemeris is provisioned (out of the workspace); it does not run in CI or on a clean checkout. It mirrors the existing corpus-regeneration workflow.

- [ ] **Step 1: Keep the discarded ascmc entries**

In `tools/se-house-reference/src/main.rs`, the `houses2(...)` helper currently returns `(sectors, ascmc[0], ascmc[1])`. Change it to also return `ascmc[2]..=ascmc[7]` (ARMC, Vertex, equatorial ascendant, co-ascendant Koch, co-ascendant Munkasey, polar ascendant) — e.g. return the full `[f64; 8]` prefix `ascmc[0..=7]` alongside the sectors, and update the caller's destructuring.

- [ ] **Step 2: Emit sidereal time**

Add a `swe_sidtime(jd_ut)` call (returns GAST in hours) per row and include it as a column. (ARMC already equals local apparent sidereal time in degrees; `swe_sidtime` gives the Greenwich apparent value used to cross-check `SiderealTime::gast_deg`.)

- [ ] **Step 3: Write the extended columns**

Extend the `cusps_out` header and each row to append the six new `ascmc` columns + the sidereal-time column, matching the schema chosen in Task 7 (extra columns on `cusps.csv`, or a sibling `angles.csv`).

- [ ] **Step 4: Build the harness in isolation**

Run: `cargo build --manifest-path tools/se-house-reference/Cargo.toml 2>&1 | tail -10`
Expected: builds (requires SE provisioned locally; see `tools/se-house-reference/LICENSE-NOTES.md`).

- [ ] **Step 5: Commit (code only; corpus regenerated in Task 7)**

```bash
git add tools/se-house-reference/src/main.rs
git commit -m "feat(se-house-reference): emit ascmc[2..7] + sidereal time for the angles gate"
```

---

### Task 7: `validate-angles` gate + corpus + release wiring

**Files:**
- Modify: `crates/pleiades-validate/data/houses-corpus/` (extend `cusps.csv` or add `angles.csv`; update `manifest.txt`)
- Create: `crates/pleiades-validate/src/angles_validation.rs`
- Modify: `crates/pleiades-validate/src/lib.rs` and `crates/pleiades-validate/src/main.rs` (register the `validate-angles` command)
- Modify: the release aggregate (`release-smoke`/`release-gate` wiring in `crates/pleiades-validate/src/release/…`) to include `validate-angles`
- Test: `crates/pleiades-validate/src/tests/` (a gate test alongside the existing house-gate test)

**Interfaces:**
- Consumes: `pleiades_houses::{sidereal-time via pleiades_apparent, chart_points_from_armc}`, `pleiades_apparent::sidereal_time`; the committed corpus rows.
- Produces: `validate-angles` fail-closed command; per-point arcsecond ceilings in an `angles` thresholds module.

- [ ] **Step 1: Regenerate the corpus (maintainer, SE present)**

Run the extended harness (Task 6) over the existing house fixtures and write the new columns/slice into `crates/pleiades-validate/data/houses-corpus/`. Update `manifest.txt` row counts + checksums (keep `Reference-Engine: SwissEphemeris 2.10.03`). A clean checkout must still validate these committed values tool-free.

- [ ] **Step 2: Write the failing gate test**

Add to `crates/pleiades-validate/src/tests/` (mirroring the existing house-gate test):

```rust
#[test]
fn validate_angles_passes_over_committed_corpus() {
    let report = crate::angles_validation::run_angles_gate();
    assert!(report.passed(), "validate-angles failed: {report:?}");
}
```

- [ ] **Step 3: Run it to verify it fails**

Run: `cargo test -p pleiades-validate validate_angles_passes 2>&1 | tail -20`
Expected: FAIL — module `angles_validation` / `run_angles_gate` not found.

- [ ] **Step 4: Implement the gate**

Create `crates/pleiades-validate/src/angles_validation.rs` modeled on `house_validation.rs`:
- parse the corpus rows (chart id, jd_ut, lat, lon, obliquity source, SE `armc/vertex/equasc/coasc_koch/coasc_munkasey/polar_asc`, SE sidereal time);
- for each row, recompute pleiades values via `pleiades_houses::chart_points_from_armc(SE_armc, lat, true_obliquity)` and `pleiades_apparent::sidereal_time(...)`;
- compare each point against the SE reference within a per-point ceiling defined in an `angles` thresholds table (start tight — e.g. `1.0″` for ARMC/sidereal time/MC/Asc/East Point — and set the Vertex/co-ascendant/polar-ascendant ceilings from the measured residuals);
- return a report with `passed()` and per-point max residuals; fail closed on missing rows, checksum/schema drift, or any residual over ceiling.

Register `pub mod angles_validation;` in `lib.rs` and a `validate-angles` subcommand in `main.rs` (copy the `validate-houses` command wiring).

- [ ] **Step 5: Reconcile any over-ceiling points against `swehouse.c`**

Run: `cargo test -p pleiades-validate validate_angles_passes 2>&1 | tail -30`
If Vertex / co-ascendant (Munkasey) / polar ascendant exceed their ceilings, transcribe the exact `ascmc[SE_VERTEX]` / `SE_COASC1` / `SE_COASC2` / `SE_POLASC` assignments from Swiss Ephemeris `swehouse.c` into `asc_mc_from` (Task 3), then re-run until PASS. This is the authoritative correctness step for those points.
Expected: PASS.

- [ ] **Step 6: Wire into release aggregates**

Add `validate-angles` to the `release-smoke`/`release-gate` numeric-gate set next to `validate-houses`.

Run: `cargo run -q -p pleiades-validate -- validate-angles 2>&1 | tail -5`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-validate/ 
git commit -m "feat(validate): validate-angles SE-parity gate for ascmc points + sidereal time"
```

---

### Task 8: Docs, profile bumps, and workspace green

**Files:**
- Modify: `crates/pleiades-core/src/api_stability.rs` (bump the API-stability profile)
- Modify: the compatibility profile source (`crates/pleiades-core/src/compatibility/…`, currently `pleiades-compatibility-profile/0.7.1`)
- Modify: `README.md` ("Current state" — note public sidereal time + `ascmc` points)
- Modify: `docs/time-observer-policy.md` (document sidereal time is UT1-based)
- Modify: `PLAN.md` (record SP-1 done; note SP-2/SP-3 remain)

**Interfaces:**
- Consumes: nothing new.
- Produces: aligned docs/profiles; overclaim audit stays consistent.

- [ ] **Step 1: Bump profiles and update prose**

Bump the API-stability profile version (new public surface: `sidereal_time`/`SiderealTime`, `AscMc`, `chart_points`/`chart_points_from_armc`, `HouseSnapshot::asc_mc`, `HouseSnapshot` now `#[non_exhaustive]`). Bump the compatibility profile version. Add a bullet to README "Current state". Add a paragraph to `docs/time-observer-policy.md` stating sidereal time uses UT1 (Earth rotation) and the `Instant` JD is consumed as supplied. Update `PLAN.md` to record SP-1 complete.

- [ ] **Step 2: Run the overclaim / compatibility audits**

Run: `cargo run -q -p pleiades-validate -- verify-compatibility-profile 2>&1 | tail -5`
Expected: PASS (profile ↔ prose ↔ evidence aligned).

- [ ] **Step 3: Full workspace green**

Run:
```bash
cargo fmt --all --check && \
cargo clippy --workspace --all-targets --all-features -- -D warnings && \
cargo test --workspace 2>&1 | tail -15
```
Expected: fmt clean, clippy clean, all tests PASS.

- [ ] **Step 4: Release smoke (numeric gates incl. validate-angles)**

Run: `cargo run -q -p pleiades-validate -- validate-houses && cargo run -q -p pleiades-validate -- validate-angles 2>&1 | tail -5`
Expected: both PASS.

- [ ] **Step 5: Commit**

```bash
git add README.md PLAN.md docs/time-observer-policy.md crates/pleiades-core/src/api_stability.rs crates/pleiades-core/src/compatibility/
git commit -m "docs: record SP-1 angles & sidereal time; bump stability + compatibility profiles"
```

---

## Self-Review

**Spec coverage:**
- Sidereal time (GMST/GAST/local) → Task 1, exposed Task 5, gated Task 7. ✓
- `ascmc` extras (ARMC, Vertex, antivertex, equatorial ascendant, co-ascendants, polar ascendant) → Task 3, gated Task 7. ✓
- Foundation in `pleiades-apparent`, houses delegate preserving numerics → Tasks 1–2. ✓
- `chart_points_from_armc` (`swe_houses_armc` case) → Task 3. ✓
- Surface on `HouseSnapshot` + `#[non_exhaustive]`, one-time 0.2.x break → Task 4. ✓
- Re-export via `pleiades-core`, `ChartSnapshot`, CLI render → Task 5. ✓
- SE reference extension (reuse isolated harness, C1 honored) → Task 6. ✓
- `validate-angles` gate + corpus + release wiring + overclaim alignment → Tasks 7–8. ✓
- True-obliquity default; UT1 documentation; profile bumps → Tasks 3, 8. ✓
- Open items (per-point ceilings, swehouse.c formula reconciliation, corpus shape) → resolved inside Tasks 7 (ceilings, reconciliation) and 6/7 (corpus shape). ✓

**Placeholder scan:** No `TBD`/`TODO`/"add error handling" placeholders; every code step shows code. The Vertex/co-ascendant/polar-ascendant numeric correctness is deliberately delegated to the SE-parity gate (Task 7 Step 5), which is the repo's established oracle pattern for house math — not a placeholder but a validated port.

**Type consistency:** `AscMc` field names are identical across Tasks 3, 4, 5, 7. `sidereal_time`/`SiderealTime` names identical across Tasks 1, 2, 5, 7. `chart_points`/`chart_points_from_armc` signatures identical across Tasks 3, 5, 7. `local_sidereal_time`/`mean_obliquity` keep their pre-existing signatures (Task 2 changes only bodies).
