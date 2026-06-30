# Apparent equatorial (RA/Dec) of date — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Populate the chart-layer `equatorial` field with the apparent equatorial of date (RA/Dec) for release-grade bodies, derived from the final apparent ecliptic using true obliquity, and lock it in behind two fail-closed gates (JPL Horizons accuracy + Swiss Ephemeris convention parity).

**Architecture:** A pure helper `apparent_equatorial_of_date` in `pleiades-apparent` rotates the tropical apparent ecliptic of date into equatorial of date using `ε_true = mean_obliquity + Δε`. The chart assembly closure (`pleiades-core`) calls it after the apparent/topocentric ecliptic is finalized and before the sidereal longitude shift, writing the result into `position.equatorial`. Backends are untouched (they keep their mean/J2000 equatorial for mean-fallback rows). Two new gates in `pleiades-validate` cross-check the chart output against Horizons and SE.

**Tech Stack:** Rust (workspace), `pleiades-types` coordinate rotations, `pleiades-apparent` nutation/obliquity, `pleiades-validate` corpus gates, `tools/se-equatorial-reference` (libswisseph-sys), JPL Horizons API (offline corpus regen).

**Spec:** `docs/superpowers/specs/2026-06-30-equatorial-declination-output-design.md`

## Global Constraints

- **Branch:** `feat/equatorial-declination-output` (already created; the spec is committed there).
- **Backend invariant — do NOT violate:** first-party backends remain mean-only and J2000 at the backend boundary. No backend file changes. The backend-boundary metadata strings that say "equatorial coordinates are derived with a mean-obliquity transform" (`pleiades-vsop87/src/profiles.rs:284`, `pleiades-elp/src/specification.rs:98`, `pleiades-vsop87/src/source_docs/spec.rs:273`, `pleiades-data/src/lookup.rs:405`) stay TRUE and unchanged — they describe the backend boundary, not the chart layer.
- **Coverage window:** 1900–2100 CE (packaged backend). Reference epochs must stay a few days inside the boundary (see existing `regen-apparent-goldens.sh` notes: start `2415025.5`, end `2488065.5`).
- **Fail-closed gates:** every corpus gate checksum-pins its CSV, cross-checks a manifest row count where a manifest exists, and floors the validated-row count so a truncated/empty corpus can never pass vacuously (mirror `validate_lilith_corpus`).
- **Pole conditioning:** RA residuals are always compared as `|ΔRA|·cos(Dec)` (wrap-aware); Dec is compared signed.
- **TDD + frequent commits:** one test-first cycle per step group; commit at the end of each task.
- **Build/test commands:** `cargo test -p <crate>` per crate; `cargo build -p pleiades-validate` for the CLI.

---

### Task 1: Pure `apparent_equatorial_of_date` helper in `pleiades-apparent`

**Files:**
- Create: `crates/pleiades-apparent/src/equatorial.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs` (add `mod equatorial;` + re-export)
- Test: inline `#[cfg(test)]` module in `crates/pleiades-apparent/src/equatorial.rs`

**Interfaces:**
- Consumes: `pleiades_apparent::nutation::{mean_obliquity_degrees, nutation}` (existing); `pleiades_types::{Angle, EclipticCoordinates, EquatorialCoordinates}`; `crate::error::ApparentPlaceError`.
- Produces: `pub fn apparent_equatorial_of_date(tropical_apparent_ecliptic: EclipticCoordinates, jd_tt: f64) -> Result<EquatorialCoordinates, ApparentPlaceError>` — re-exported at crate root as `pleiades_apparent::apparent_equatorial_of_date`. Also `pub fn true_obliquity_degrees(jd_tt: f64) -> Result<f64, ApparentPlaceError>`.

- [ ] **Step 1: Write the failing tests**

Create `crates/pleiades-apparent/src/equatorial.rs` with this content (implementation stubbed to `unimplemented!()` so the module compiles but tests fail):

```rust
//! Apparent equatorial of date: rotate the tropical apparent ecliptic of date
//! into RA/Dec using the true obliquity of date (mean obliquity + Δε). Pure;
//! the only fallible step is the nutation table lookup.

use pleiades_types::{Angle, EclipticCoordinates, EquatorialCoordinates};

use crate::error::ApparentPlaceError;
use crate::nutation::{mean_obliquity_degrees, nutation};

/// True obliquity of date in degrees: mean obliquity (Meeus 22.2) plus
/// nutation-in-obliquity Δε.
pub fn true_obliquity_degrees(jd_tt: f64) -> Result<f64, ApparentPlaceError> {
    unimplemented!()
}

/// Apparent equatorial of date (RA/Dec) from the tropical apparent ecliptic of
/// date. The ecliptic must already carry of-date corrections (precession,
/// nutation-in-longitude, aberration, and any topocentric shift); this rotates
/// it into the equatorial frame of date. Distance is preserved.
pub fn apparent_equatorial_of_date(
    tropical_apparent_ecliptic: EclipticCoordinates,
    jd_tt: f64,
) -> Result<EquatorialCoordinates, ApparentPlaceError> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{Latitude, Longitude};

    fn ecl(lon: f64, lat: f64, dist: Option<f64>) -> EclipticCoordinates {
        EclipticCoordinates::new(
            Longitude::from_degrees(lon),
            Latitude::from_degrees(lat),
            dist,
        )
    }

    #[test]
    fn true_obliquity_is_mean_plus_delta_eps() {
        let jd = 2_451_545.0; // J2000
        let mean = mean_obliquity_degrees(jd);
        let delta_eps_deg = nutation(jd).unwrap().delta_eps_arcsec / 3600.0;
        let got = true_obliquity_degrees(jd).unwrap();
        assert!((got - (mean + delta_eps_deg)).abs() < 1e-12, "got {got}");
        // Near 23.44° at J2000.
        assert!((got - 23.4392).abs() < 0.01, "true obliquity {got}");
    }

    #[test]
    fn composes_rotation_with_true_obliquity() {
        // The helper must equal: rotate by true obliquity of date, nothing else.
        let jd = 2_433_283.0;
        let e = ecl(123.456, 1.234, Some(0.987));
        let eps = true_obliquity_degrees(jd).unwrap();
        let expected = e.to_equatorial(Angle::from_degrees(eps));
        let got = apparent_equatorial_of_date(e, jd).unwrap();
        assert!((got.right_ascension.degrees() - expected.right_ascension.degrees()).abs() < 1e-9);
        assert!((got.declination.degrees() - expected.declination.degrees()).abs() < 1e-9);
        assert_eq!(got.distance_au, expected.distance_au);
    }

    #[test]
    fn solstice_point_maps_to_ra90_dec_obliquity() {
        // Ecliptic longitude 90°, latitude 0° → RA 90°, Dec = +obliquity.
        let jd = 2_451_545.0;
        let eps = true_obliquity_degrees(jd).unwrap();
        let got = apparent_equatorial_of_date(ecl(90.0, 0.0, None), jd).unwrap();
        assert!((got.right_ascension.degrees() - 90.0).abs() < 1e-6, "ra {}", got.right_ascension.degrees());
        assert!((got.declination.degrees() - eps).abs() < 1e-6, "dec {}", got.declination.degrees());
    }

    #[test]
    fn roundtrips_through_to_ecliptic() {
        let jd = 2_469_807.5;
        let e = ecl(305.0, -4.5, Some(2.0));
        let eps = true_obliquity_degrees(jd).unwrap();
        let eq = apparent_equatorial_of_date(e, jd).unwrap();
        let back = eq.to_ecliptic(Angle::from_degrees(eps));
        assert!((back.longitude.degrees() - e.longitude.degrees()).abs() < 1e-7);
        assert!((back.latitude.degrees() - e.latitude.degrees()).abs() < 1e-7);
    }
}
```

- [ ] **Step 2: Wire the module into the crate so it compiles**

In `crates/pleiades-apparent/src/lib.rs`, after the `pub mod nutation;` / `pub use nutation::Nutation;` block (around line 8–10), add:

```rust
pub mod equatorial;

pub use equatorial::{apparent_equatorial_of_date, true_obliquity_degrees};
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p pleiades-apparent equatorial:: -- --nocapture`
Expected: compiles, tests FAIL with `not implemented` panics from `unimplemented!()`.

- [ ] **Step 4: Implement the helper**

Replace the two `unimplemented!()` bodies in `crates/pleiades-apparent/src/equatorial.rs`:

```rust
pub fn true_obliquity_degrees(jd_tt: f64) -> Result<f64, ApparentPlaceError> {
    let delta_eps_arcsec = nutation(jd_tt)?.delta_eps_arcsec;
    Ok(mean_obliquity_degrees(jd_tt) + delta_eps_arcsec / 3600.0)
}

pub fn apparent_equatorial_of_date(
    tropical_apparent_ecliptic: EclipticCoordinates,
    jd_tt: f64,
) -> Result<EquatorialCoordinates, ApparentPlaceError> {
    let eps = true_obliquity_degrees(jd_tt)?;
    Ok(tropical_apparent_ecliptic.to_equatorial(Angle::from_degrees(eps)))
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p pleiades-apparent equatorial::`
Expected: PASS (4 tests).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-apparent/src/equatorial.rs crates/pleiades-apparent/src/lib.rs
git commit -m "feat(apparent): apparent_equatorial_of_date helper (true obliquity of date)"
```

---

### Task 2: Chart-layer wiring in `pleiades-core`

**Files:**
- Modify: `crates/pleiades-core/src/chart/mod.rs` (import the helper; insert the equatorial derivation after the topocentric block and before the sidereal re-apply)
- Test: `crates/pleiades-core/src/chart/tests.rs` (add four tests)

**Interfaces:**
- Consumes: `pleiades_apparent::apparent_equatorial_of_date` (Task 1); the existing per-body closure locals `position` (mutable `EphemerisResult`), `apparent: Option<ApparentProvenance>`, `request.instant`.
- Produces: `position.equatorial = Some(apparent equatorial of date)` for apparent rows; unchanged (backend mean equatorial) for mean-fallback rows.

- [ ] **Step 1: Write the failing tests**

Add to `crates/pleiades-core/src/chart/tests.rs` (use the existing test imports/`test_support` helpers in that file; if a `packaged_engine()`/`ChartEngine` builder already exists there, reuse it — otherwise build `ChartEngine::new(PackagedDataBackend::new())` as `apparent_validation.rs` does). These tests use the Sun (release-grade) at JD 2451545.0:

```rust
#[test]
fn apparent_chart_populates_equatorial_of_date() {
    use pleiades_apparent::{apparent_equatorial_of_date, true_obliquity_degrees};
    use pleiades_types::Angle;
    let engine = ChartEngine::new(PackagedDataBackend::new());
    let jd = 2_451_545.0;
    let req = ChartRequest::new(Instant::new(JulianDay::from_days(jd), TimeScale::Tt))
        .with_bodies(vec![CelestialBody::Sun])
        .with_apparentness(Apparentness::Apparent);
    let snap = engine.chart(&req).unwrap();
    let p = snap.placement_for(&CelestialBody::Sun).unwrap();
    assert!(p.apparent.is_some(), "Sun should be apparent");
    let ecl = p.position.ecliptic.as_ref().unwrap();
    let eq = p.position.equatorial.expect("equatorial populated");
    // Locks the wiring to the true-obliquity-of-date helper on the FINAL ecliptic.
    let expected = apparent_equatorial_of_date(*ecl, jd).unwrap();
    assert!((eq.right_ascension.degrees() - expected.right_ascension.degrees()).abs() < 1e-9);
    assert!((eq.declination.degrees() - expected.declination.degrees()).abs() < 1e-9);
    // Locks "of date": differs from the mean-obliquity transform by the nutation Δε.
    let mean_eps = pleiades_apparent::nutation::mean_obliquity_degrees(jd);
    let mean_eq = ecl.to_equatorial(Angle::from_degrees(mean_eps));
    let d = (eq.declination.degrees() - mean_eq.declination.degrees()).abs() * 3600.0;
    assert!(d > 0.0 && d < 60.0, "of-date vs mean Dec delta {d}\" (expected small, nonzero)");
    let _ = true_obliquity_degrees(jd).unwrap();
}

#[test]
fn equatorial_is_identical_tropical_vs_sidereal() {
    let engine = ChartEngine::new(PackagedDataBackend::new());
    let inst = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let tropical = engine
        .chart(&ChartRequest::new(inst)
            .with_bodies(vec![CelestialBody::Sun])
            .with_apparentness(Apparentness::Apparent))
        .unwrap();
    let sidereal = engine
        .chart(&ChartRequest::new(inst)
            .with_bodies(vec![CelestialBody::Sun])
            .with_apparentness(Apparentness::Apparent)
            .with_zodiac_mode(ZodiacMode::Sidereal { ayanamsa: AyanamsaMode::Lahiri }))
        .unwrap();
    let a = tropical.placement_for(&CelestialBody::Sun).unwrap().position.equatorial.unwrap();
    let b = sidereal.placement_for(&CelestialBody::Sun).unwrap().position.equatorial.unwrap();
    assert!((a.right_ascension.degrees() - b.right_ascension.degrees()).abs() < 1e-9, "RA must be ayanamsa-independent");
    assert!((a.declination.degrees() - b.declination.degrees()).abs() < 1e-9, "Dec must be ayanamsa-independent");
}

#[test]
fn mean_fallback_keeps_backend_equatorial() {
    // A mean-mode chart does not run the apparent path; equatorial stays the
    // backend's mean-obliquity transform (Some, but NOT of-date-recomputed).
    let backend = PackagedDataBackend::new();
    let inst = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let direct = backend.position(&EphemerisRequest::new(CelestialBody::Sun, inst)).unwrap();
    let engine = ChartEngine::new(PackagedDataBackend::new());
    let snap = engine
        .chart(&ChartRequest::new(inst)
            .with_bodies(vec![CelestialBody::Sun])
            .with_apparentness(Apparentness::Mean))
        .unwrap();
    let p = snap.placement_for(&CelestialBody::Sun).unwrap();
    assert!(p.apparent.is_none(), "mean mode → no apparent provenance");
    let eq = p.position.equatorial.unwrap();
    let backend_eq = direct.equatorial.unwrap();
    assert!((eq.right_ascension.degrees() - backend_eq.right_ascension.degrees()).abs() < 1e-9);
    assert!((eq.declination.degrees() - backend_eq.declination.degrees()).abs() < 1e-9);
}

#[test]
fn topocentric_equatorial_reflects_topocentric_ecliptic() {
    // The Moon's topocentric equatorial differs from its geocentric equatorial.
    let engine = ChartEngine::new(PackagedDataBackend::new());
    let inst = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let observer = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(-74.0),
        Some(0.0),
    );
    let geo = engine
        .chart(&ChartRequest::new(inst)
            .with_bodies(vec![CelestialBody::Moon])
            .with_apparentness(Apparentness::Apparent))
        .unwrap();
    let topo = engine
        .chart(&ChartRequest::new(inst)
            .with_bodies(vec![CelestialBody::Moon])
            .with_apparentness(Apparentness::Apparent)
            .with_observer(observer)
            .with_topocentric(true))
        .unwrap();
    let g = geo.placement_for(&CelestialBody::Moon).unwrap().position.equatorial.unwrap();
    let t = topo.placement_for(&CelestialBody::Moon).unwrap().position.equatorial.unwrap();
    let dra = (g.right_ascension.degrees() - t.right_ascension.degrees()).abs() * 3600.0;
    let ddec = (g.declination.degrees() - t.declination.degrees()).abs() * 3600.0;
    assert!(dra + ddec > 1.0, "topocentric Moon equatorial should differ measurably from geocentric");
}
```

Note on imports: the exact request-builder names (`with_zodiac_mode`, `with_observer`, `with_topocentric`, `AyanamsaMode::Lahiri`, `ObserverLocation`) must match the `ChartRequest` API in `crates/pleiades-core/src/chart/request.rs` and the re-exports in `crates/pleiades-core/src/lib.rs`. Grep `request.rs` for the builder method names and adjust the calls/imports to the real signatures before running. If `placement_for` / `Apparentness::Mean` / `ZodiacMode` are not already imported in `tests.rs`, add them from `crate::` / `pleiades_types`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p pleiades-core chart::tests::apparent_chart_populates_equatorial_of_date chart::tests::equatorial_is_identical_tropical_vs_sidereal chart::tests::mean_fallback_keeps_backend_equatorial chart::tests::topocentric_equatorial_reflects_topocentric_ecliptic`
Expected: `apparent_chart_populates_equatorial_of_date` FAILS at `apparent_equatorial_of_date(*ecl, jd)` mismatch (chart still holds backend mean equatorial); `topocentric_…` FAILS (topo equatorial not recomputed). `mean_fallback_…` and the parity test may already pass — that is fine.

- [ ] **Step 3: Add the import**

In `crates/pleiades-core/src/chart/mod.rs`, extend the `use pleiades_apparent::{…}` block (lines 39–43) to include `apparent_equatorial_of_date`:

```rust
use pleiades_apparent::{
    apparent_apsis_position, apparent_equatorial_of_date, apparent_position, apparent_sun_position,
    precess_ecliptic_j2000_to_date, ApparentLightTimeError, ApparentPlaceError,
    DEFAULT_MAX_ITERATIONS,
};
```

- [ ] **Step 4: Insert the equatorial derivation**

In `crates/pleiades-core/src/chart/mod.rs`, locate the per-body closure point **after** the topocentric block (the `let topocentric_prov = if request.topocentric { … } else { None };` block, ending ~line 433) and **before** the sidereal re-apply comment (`// For non-native sidereal charts, re-apply the ayanamsa …`, ~line 437). Insert:

```rust
                // Derive the apparent equatorial of date from the final tropical
                // apparent ecliptic (geocentric or topocentric), BEFORE the
                // sidereal longitude shift so RA/Dec stay ayanamsa-independent.
                // Mean-fallback rows (apparent.is_none()) keep the backend's
                // mean-obliquity equatorial. Degrade gracefully if nutation is
                // unavailable for this instant.
                if apparent.is_some() {
                    if let Some(ecliptic) = position.ecliptic.as_ref() {
                        let jd_tt = request.instant.julian_day.days();
                        if let Ok(eq) = apparent_equatorial_of_date(*ecliptic, jd_tt) {
                            position.equatorial = Some(eq);
                        }
                    }
                }
```

(Use `position.ecliptic.as_ref()` — read-only — to avoid borrow conflicts with the later sidereal `as_mut()`. The `jd_tt` expression matches the one already used in the topocentric block, `request.instant.julian_day.days()`.)

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p pleiades-core chart::tests::apparent_chart_populates_equatorial_of_date chart::tests::equatorial_is_identical_tropical_vs_sidereal chart::tests::mean_fallback_keeps_backend_equatorial chart::tests::topocentric_equatorial_reflects_topocentric_ecliptic`
Expected: PASS (4 tests).

- [ ] **Step 6: Run the full crate test suite (no regressions)**

Run: `cargo test -p pleiades-core`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-core/src/chart/mod.rs crates/pleiades-core/src/chart/tests.rs
git commit -m "feat(chart): apparent equatorial of date for release-grade bodies"
```

---

### Task 3: Horizons accuracy gate (`validate-equatorial`)

> **Data-build note (one-off, requires network):** Step 1 regenerates a committed CSV from the JPL Horizons API. This mirrors how `apparent-goldens.csv` was built. If the execution environment has no network, treat Step 1 as a data-build handoff: the deliverable of this task is the gate code (Steps 3–6); the committed `equatorial-goldens.csv` is produced once by running the script and is then checksum-pinned. Do not fabricate rows.

**Files:**
- Create: `crates/pleiades-validate/scripts/regen-equatorial-goldens.sh`
- Create: `crates/pleiades-validate/data/equatorial-goldens.csv` (generated by the script)
- Create: `crates/pleiades-validate/src/equatorial_validation.rs`
- Modify: `crates/pleiades-validate/src/lib.rs` (declare + export)
- Test: inline `#[cfg(test)]` in `equatorial_validation.rs`

**Interfaces:**
- Consumes: `pleiades_core::{Apparentness, CelestialBody, ChartEngine, ChartRequest, Instant, JulianDay, TimeScale}`; `pleiades_data::PackagedDataBackend`; `pleiades_apparent::fnv1a64`.
- Produces: `pub fn validate_equatorial_goldens() -> Result<EquatorialValidationReport, EquatorialValidationError>`; `pub struct EquatorialValidationReport { pub rows_validated: usize, pub max_residual_ra_arcsec: f64, pub max_residual_dec_arcsec: f64, summary_line: String }` with `pub fn summary_line(&self) -> &str`.

- [ ] **Step 1: Create and run the regen script (data build)**

Create `crates/pleiades-validate/scripts/regen-equatorial-goldens.sh` (mirrors `regen-apparent-goldens.sh`; same epochs/bodies; Horizons quantity `2` = Apparent RA & DEC, true equator/equinox of date):

```bash
#!/usr/bin/env bash
# Regenerate apparent equatorial (RA/Dec) of-date goldens from JPL Horizons
# (QUANTITIES='2': apparent RA & DEC, referred to the true equator and equinox
# of date, with light-time, deflection, and stellar aberration), geocentric
# observer (500@399). Same epochs/bodies as regen-apparent-goldens.sh. Run
# manually when refreshing the corpus.
#
# Tolerances mirror the apparent-longitude gate's per-body model limits: planets
# 26", Moon 45", Sun 5" — applied to BOTH the cos(Dec)-weighted RA residual and
# the signed Dec residual. 433-Eros excluded (apparent iteration diverges; see
# apparent gate header).
set -euo pipefail
OUT="$(dirname "$0")/../data/equatorial-goldens.csv"
API="https://ssd.jpl.nasa.gov/api/horizons.api"
EPOCHS="2415025.5 2433282.5 2451545.0 2469807.5 2488065.5"
BODIES="Sun:10 Moon:301 Mercury:199 Venus:299 Mars:499 Jupiter:599 Saturn:699 Uranus:799 Neptune:899 Pluto:999"
{
  echo "# Source: JPL Horizons API ($API), EPHEM_TYPE=OBSERVER, CENTER='500@399',"
  echo "# QUANTITIES='2' (apparent RA & DEC, true equator/equinox of date, with light-time,"
  echo "# deflection, and stellar aberration), ANG_FORMAT=DEG, extra_prec=YES."
  echo "# Regenerated by crates/pleiades-validate/scripts/regen-equatorial-goldens.sh"
  echo "# Tolerance rationale matches apparent-goldens per-body model limits (Sun 5\", planets 26\", Moon 45\"),"
  echo "# applied to cos(Dec)-weighted RA residual and signed Dec residual. 433-Eros excluded."
  echo "body,jd_tt,apparent_ra_deg,apparent_dec_deg,ra_tolerance_arcsec,dec_tolerance_arcsec"
  for entry in $BODIES; do
    label="${entry%%:*}"; code="${entry##*:}"
    if [ "$label" = "Moon" ]; then tol=45.0; elif [ "$label" = "Sun" ]; then tol=5.0; else tol=26.0; fi
    for jd in $EPOCHS; do
      radec=$(curl -sS -m 30 "$API?format=text&COMMAND='$code'&EPHEM_TYPE=OBSERVER&CENTER='500@399'&TLIST='$jd'&QUANTITIES='2'&ANG_FORMAT=DEG&CAL_FORMAT=JD&extra_prec=YES" \
        | sed -n '/\$\$SOE/,/\$\$EOE/p' | sed '1d;$d' | awk '{print $2","$3}')
      echo "$label,$jd,$radec,$tol,$tol"
    done
  done
} > "$OUT"
echo "wrote $OUT"
```

Run it:

```bash
chmod +x crates/pleiades-validate/scripts/regen-equatorial-goldens.sh
crates/pleiades-validate/scripts/regen-equatorial-goldens.sh
```

Inspect `crates/pleiades-validate/data/equatorial-goldens.csv`: 50 data rows (10 bodies × 5 epochs), each `body,jd_tt,ra_deg,dec_deg,ra_tol,dec_tol` with finite numeric RA/Dec. If the Horizons `$2`/`$3` column split is off (the SOE row format can shift), fix the `awk` field indices so RA and Dec parse — verify against one body by hand.

- [ ] **Step 2: Write the gate (failing) and the tests**

Create `crates/pleiades-validate/src/equatorial_validation.rs`. Model it on `apparent_validation.rs`. Leave `GOLDENS_CHECKSUM` at `0` for now (the checksum gate is skipped while `0`, matching the `apparent_validation` pattern `if GOLDENS_CHECKSUM != 0 && …`).

```rust
//! Fail-closed cross-check of the engine's apparent equatorial-of-date RA/Dec
//! against JPL Horizons apparent RA/Dec goldens (quantity 2). Reads the
//! committed CSV offline. RA residual is cos(Dec)-weighted; Dec residual signed.
#![forbid(unsafe_code)]

use core::fmt;

use pleiades_core::{
    Apparentness, CelestialBody, ChartEngine, ChartRequest, Instant, JulianDay, TimeScale,
};
use pleiades_data::PackagedDataBackend;

const GOLDENS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/equatorial-goldens.csv"
));
const GOLDENS_CHECKSUM: u64 = 0; // pinned in Step 5

#[derive(Clone, Debug, PartialEq)]
pub struct EquatorialValidationReport {
    pub rows_validated: usize,
    pub max_residual_ra_arcsec: f64,
    pub max_residual_dec_arcsec: f64,
    summary_line: String,
}

impl EquatorialValidationReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

impl fmt::Display for EquatorialValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EquatorialValidationError {
    ChecksumMismatch { expected: u64, actual: u64 },
    MalformedRow { row: usize, line: String, reason: String },
    UnknownBody { row: usize, label: String },
    ChartError { row: usize, body: String, jd: f64, message: String },
    UnexpectedMeanFallback { row: usize, body: String, jd_tt: f64 },
    MissingEquatorial { row: usize, body: String, jd_tt: f64 },
    ToleranceExceeded {
        row: usize, body: String, jd: f64, axis: &'static str,
        got: f64, want: f64, residual_arcsec: f64, tolerance_arcsec: f64,
    },
    EmptyCorpus,
}

impl fmt::Display for EquatorialValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChecksumMismatch { expected, actual } =>
                write!(f, "equatorial goldens checksum mismatch: expected {expected:#018x}, got {actual:#018x}"),
            Self::MalformedRow { row, line, reason } =>
                write!(f, "equatorial goldens row {row} malformed ({reason}): {line:?}"),
            Self::UnknownBody { row, label } =>
                write!(f, "equatorial goldens row {row}: unknown body {label:?}"),
            Self::ChartError { row, body, jd, message } =>
                write!(f, "equatorial goldens row {row} ({body} @ JD {jd}): chart error: {message}"),
            Self::UnexpectedMeanFallback { row, body, jd_tt } =>
                write!(f, "equatorial goldens row {row} ({body} @ JD {jd_tt}): mean fallback — apparent provenance absent"),
            Self::MissingEquatorial { row, body, jd_tt } =>
                write!(f, "equatorial goldens row {row} ({body} @ JD {jd_tt}): equatorial channel absent"),
            Self::ToleranceExceeded { row, body, jd, axis, got, want, residual_arcsec, tolerance_arcsec } =>
                write!(f, "equatorial goldens row {row} ({body} @ JD {jd}) {axis}: got {got:.7} want {want:.7} residual {residual_arcsec:.2}\u{2033} > tol {tolerance_arcsec:.1}\u{2033}"),
            Self::EmptyCorpus => write!(f, "equatorial goldens corpus is empty (fail-closed)"),
        }
    }
}

impl std::error::Error for EquatorialValidationError {}

fn resolve_body(label: &str) -> Option<CelestialBody> {
    match label {
        "Sun" => Some(CelestialBody::Sun),
        "Moon" => Some(CelestialBody::Moon),
        "Mercury" => Some(CelestialBody::Mercury),
        "Venus" => Some(CelestialBody::Venus),
        "Mars" => Some(CelestialBody::Mars),
        "Jupiter" => Some(CelestialBody::Jupiter),
        "Saturn" => Some(CelestialBody::Saturn),
        "Uranus" => Some(CelestialBody::Uranus),
        "Neptune" => Some(CelestialBody::Neptune),
        "Pluto" => Some(CelestialBody::Pluto),
        _ => None,
    }
}

struct Row {
    body_label: String,
    body: CelestialBody,
    jd_tt: f64,
    ra_deg: f64,
    dec_deg: f64,
    ra_tol_arcsec: f64,
    dec_tol_arcsec: f64,
}

fn parse() -> Result<Vec<Row>, EquatorialValidationError> {
    let mut rows = Vec::new();
    let mut n = 0usize;
    for line in GOLDENS_CSV.lines() {
        let t = line.trim();
        if t.starts_with('#') || t.is_empty() { continue; }
        if t.starts_with("body,jd_tt,apparent_ra_deg") { continue; }
        n += 1;
        let p: Vec<&str> = t.splitn(6, ',').collect();
        if p.len() != 6 {
            return Err(EquatorialValidationError::MalformedRow {
                row: n, line: line.to_string(),
                reason: format!("expected 6 fields, got {}", p.len()),
            });
        }
        let f = |i: usize, name: &str| -> Result<f64, EquatorialValidationError> {
            p[i].trim().parse::<f64>().map_err(|_| EquatorialValidationError::MalformedRow {
                row: n, line: line.to_string(), reason: format!("{name} {:?} not a float", p[i]),
            })
        };
        let body_label = p[0].trim().to_string();
        let body = resolve_body(&body_label).ok_or_else(|| EquatorialValidationError::UnknownBody {
            row: n, label: body_label.clone(),
        })?;
        rows.push(Row {
            body_label, body,
            jd_tt: f(1, "jd_tt")?, ra_deg: f(2, "ra")?, dec_deg: f(3, "dec")?,
            ra_tol_arcsec: f(4, "ra_tol")?, dec_tol_arcsec: f(5, "dec_tol")?,
        });
    }
    Ok(rows)
}

fn wrap_deg(mut d: f64) -> f64 {
    while d > 180.0 { d -= 360.0; }
    while d < -180.0 { d += 360.0; }
    d
}

pub fn validate_equatorial_goldens() -> Result<EquatorialValidationReport, EquatorialValidationError> {
    let actual = pleiades_apparent::fnv1a64(GOLDENS_CSV);
    if GOLDENS_CHECKSUM != 0 && actual != GOLDENS_CHECKSUM {
        return Err(EquatorialValidationError::ChecksumMismatch { expected: GOLDENS_CHECKSUM, actual });
    }
    let rows = parse()?;
    if rows.is_empty() {
        return Err(EquatorialValidationError::EmptyCorpus);
    }
    let engine = ChartEngine::new(PackagedDataBackend::new());
    let (mut max_ra, mut max_dec) = (0.0_f64, 0.0_f64);
    for (idx, row) in rows.iter().enumerate() {
        let r = idx + 1;
        let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);
        let req = ChartRequest::new(instant)
            .with_bodies(vec![row.body.clone()])
            .with_apparentness(Apparentness::Apparent);
        let snap = engine.chart(&req).map_err(|e| EquatorialValidationError::ChartError {
            row: r, body: row.body_label.clone(), jd: row.jd_tt, message: e.to_string(),
        })?;
        let p = snap.placement_for(&row.body).ok_or_else(|| EquatorialValidationError::ChartError {
            row: r, body: row.body_label.clone(), jd: row.jd_tt, message: "body not in snapshot".into(),
        })?;
        if p.apparent.is_none() {
            return Err(EquatorialValidationError::UnexpectedMeanFallback {
                row: r, body: row.body_label.clone(), jd_tt: row.jd_tt,
            });
        }
        let eq = p.position.equatorial.ok_or_else(|| EquatorialValidationError::MissingEquatorial {
            row: r, body: row.body_label.clone(), jd_tt: row.jd_tt,
        })?;
        let got_ra = eq.right_ascension.degrees();
        let got_dec = eq.declination.degrees();
        // cos(Dec)-weighted RA residual (pole-safe), arcsec.
        let cos_dec = row.dec_deg.to_radians().cos();
        let ra_resid = (wrap_deg(got_ra - row.ra_deg).abs() * cos_dec) * 3600.0;
        let dec_resid = (got_dec - row.dec_deg).abs() * 3600.0;
        if ra_resid > row.ra_tol_arcsec {
            return Err(EquatorialValidationError::ToleranceExceeded {
                row: r, body: row.body_label.clone(), jd: row.jd_tt, axis: "ra",
                got: got_ra, want: row.ra_deg, residual_arcsec: ra_resid, tolerance_arcsec: row.ra_tol_arcsec,
            });
        }
        if dec_resid > row.dec_tol_arcsec {
            return Err(EquatorialValidationError::ToleranceExceeded {
                row: r, body: row.body_label.clone(), jd: row.jd_tt, axis: "dec",
                got: got_dec, want: row.dec_deg, residual_arcsec: dec_resid, tolerance_arcsec: row.dec_tol_arcsec,
            });
        }
        max_ra = max_ra.max(ra_resid);
        max_dec = max_dec.max(dec_resid);
    }
    let summary_line = format!(
        "Equatorial goldens: {} rows validated vs JPL Horizons, max RA {:.2}\u{2033} (cos\u{03b4}-wt), max Dec {:.2}\u{2033}",
        rows.len(), max_ra, max_dec
    );
    Ok(EquatorialValidationReport {
        rows_validated: rows.len(),
        max_residual_ra_arcsec: max_ra,
        max_residual_dec_arcsec: max_dec,
        summary_line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equatorial_goldens_pass() {
        let report = validate_equatorial_goldens().expect("equatorial goldens within tolerance");
        // Fail-closed floor: 10 bodies × 5 epochs = 50 rows.
        assert!(report.rows_validated >= 45, "too few rows validated: {}", report.rows_validated);
        eprintln!("{}", report.summary_line());
    }

    #[test]
    fn pinned_checksum() {
        let actual = pleiades_apparent::fnv1a64(GOLDENS_CSV);
        assert_eq!(GOLDENS_CHECKSUM, actual, "update GOLDENS_CHECKSUM to {actual}");
    }
}
```

- [ ] **Step 3: Declare and export from `lib.rs`**

In `crates/pleiades-validate/src/lib.rs`, add the module declaration next to `mod lilith_validation;` (line 25) and a re-export next to the lilith one (line 185):

```rust
mod equatorial_validation;
```
```rust
pub use equatorial_validation::{
    validate_equatorial_goldens, EquatorialValidationError, EquatorialValidationReport,
};
```

- [ ] **Step 4: Run the gate test to verify it passes (data present)**

Run: `cargo test -p pleiades-validate equatorial_validation::tests::equatorial_goldens_pass -- --nocapture`
Expected: PASS, prints the summary with measured max RA/Dec residuals. If a residual exceeds its tolerance, first confirm the Horizons column parse (Step 1) is correct; only widen a per-body tolerance with a header-documented rationale (match the apparent-gate convention).

- [ ] **Step 5: Pin the checksum**

Run: `cargo test -p pleiades-validate equatorial_validation::tests::pinned_checksum -- --nocapture`
It fails and prints the actual value. Set `const GOLDENS_CHECKSUM: u64 = <printed value>;` in `equatorial_validation.rs`. Re-run — expected PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-validate/scripts/regen-equatorial-goldens.sh \
        crates/pleiades-validate/data/equatorial-goldens.csv \
        crates/pleiades-validate/src/equatorial_validation.rs \
        crates/pleiades-validate/src/lib.rs
git commit -m "feat(validate): validate-equatorial gate vs JPL Horizons apparent RA/Dec"
```

---

### Task 4: SE convention-parity gate (`validate-equatorial-se`)

> **Data-build note (one-off, requires Swiss Ephemeris + libclang):** Step 2 builds and runs `tools/se-equatorial-reference`, which links the vendored Swiss Ephemeris via `libswisseph-sys` (needs `libclang-dev` + `LIBCLANG_PATH`, exactly like `tools/se-lilith-reference`). The gate itself never rebuilds the tool — it reads the committed CSV. If SE/libclang is unavailable, this is a data-build handoff: the deliverable is the tool source + the gate code; the corpus CSV + manifest are produced once by running the tool.

**Files:**
- Create: `tools/se-equatorial-reference/Cargo.toml`
- Create: `tools/se-equatorial-reference/src/main.rs`
- Create: `crates/pleiades-validate/data/equatorial-se-corpus/equatorial-se.csv` (generated)
- Create: `crates/pleiades-validate/data/equatorial-se-corpus/manifest.txt` (generated)
- Modify: `crates/pleiades-validate/src/equatorial_validation.rs` (add the SE arm)
- Modify: `crates/pleiades-validate/src/lib.rs` (export the SE function/types)

**Interfaces:**
- Consumes: same chart-engine path as Task 3; `pleiades_apparent::fnv1a64`.
- Produces: `pub fn validate_equatorial_se_corpus() -> Result<EquatorialSeReport, EquatorialSeError>`; `pub struct EquatorialSeReport { pub rows_validated: usize, pub max_residual_ra_arcsec: f64, pub max_residual_dec_arcsec: f64, summary_line: String }` with `pub fn summary_line(&self) -> &str`.

- [ ] **Step 1: Create the SE reference tool**

Create `tools/se-equatorial-reference/Cargo.toml` (mirror `tools/se-lilith-reference/Cargo.toml`):

```toml
[package]
name = "se-equatorial-reference"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
swisseph = "0.1.1"
libswisseph-sys = "0.1.2"
```

Create `tools/se-equatorial-reference/src/main.rs` (emits apparent RA/Dec of date for the 10 major bodies on the same kind of grid as the lilith tool):

```rust
//! Emits a Swiss Ephemeris apparent equatorial-of-date (RA/Dec) reference corpus
//! for the major bodies to STDOUT as CSV.
//!
//! Frame: true equator/equinox of date, nutation + aberration on (SE default —
//! no SEFLG_NONUT/NOABERR/J2000), equatorial output (SEFLG_EQUATORIAL).
//! Ephemeris: Moshier (SEFLG_MOSEPH) — no data files needed.
//!
//! Usage: `cargo run --release > .../equatorial-se-corpus/equatorial-se.csv`

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::swe_calc;

const SEFLG_MOSEPH: c_int = 4;
const SEFLG_EQUATORIAL: c_int = 2048;
const IFLAG: c_int = SEFLG_MOSEPH | SEFLG_EQUATORIAL;

// (label, SE body number): Sun=0, Moon=1, Mercury=2 … Pluto=9.
const BODIES: &[(&str, c_int)] = &[
    ("Sun", 0), ("Moon", 1), ("Mercury", 2), ("Venus", 3), ("Mars", 4),
    ("Jupiter", 5), ("Saturn", 6), ("Uranus", 7), ("Neptune", 8), ("Pluto", 9),
];

const JD_START_TT: f64 = 2_415_025.5; // 1900-01-06 (inside coverage; see apparent gate)
const JD_END_TT: f64 = 2_488_065.5;   // 2099-12-26
const STEP_DAYS: f64 = 365.25 * 5.0;  // ~5-year cadence → ~40 epochs × 10 bodies

fn se_radec(jd_tt: f64, body: c_int) -> (f64, f64) {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(jd_tt, body, IFLAG, xx.as_mut_ptr(), serr.as_mut_ptr() as *mut c_char)
    };
    if ret < 0 {
        let msg = unsafe { CStr::from_ptr(serr.as_ptr() as *const c_char) }
            .to_string_lossy().into_owned();
        panic!("swe_calc(body={body}) failed at jd_tt={jd_tt}: {msg}");
    }
    let (ra, dec) = (xx[0], xx[1]);
    assert!(ra.is_finite() && dec.is_finite(), "non-finite SE RA/Dec at jd_tt={jd_tt}");
    (ra.rem_euclid(360.0), dec)
}

fn main() {
    println!("# Source: Swiss Ephemeris (libswisseph-sys 0.1.2), swe_calc, iflag=SEFLG_MOSEPH|SEFLG_EQUATORIAL.");
    println!("# Frame: apparent equatorial of date (nutation + aberration on). Columns: jd_tt,body,ra_deg,dec_deg.");
    println!("# Convention-parity reference (Moshier); ceilings are deliberately loose vs the Horizons accuracy gate.");
    println!("jd_tt,body,se_ra_deg,se_dec_deg");
    let mut jd = JD_START_TT;
    while jd <= JD_END_TT {
        for (label, num) in BODIES {
            let (ra, dec) = se_radec(jd, *num);
            println!("{jd:.1},{label},{ra:.9},{dec:.9}");
        }
        jd += STEP_DAYS;
    }
}
```

- [ ] **Step 2: Generate the corpus + manifest (data build)**

```bash
mkdir -p crates/pleiades-validate/data/equatorial-se-corpus
( cd tools/se-equatorial-reference && cargo run --release ) \
  > crates/pleiades-validate/data/equatorial-se-corpus/equatorial-se.csv
```

Then compute the row count and checksum and write `manifest.txt` (mirror `lilith-corpus/manifest.txt` format `slice <name> file=<f> role=<r> rows=<N> checksum=<fnv1a64>`). Get the values from the gate test in Step 4 (it prints both on first failure), or with a one-off: the row count is data lines (exclude `#` and header); the checksum is `pleiades_apparent::fnv1a64` of the whole file. Write:

```
slice equatorial-se file=equatorial-se.csv role=equatorial-se rows=<N> checksum=<CHECKSUM>
```

- [ ] **Step 3: Write the SE gate arm (failing) + test**

Append to `crates/pleiades-validate/src/equatorial_validation.rs`. The structure mirrors `validate_lilith_corpus` (checksum + manifest-row cross-check + fail-closed floor) but compares chart-engine apparent RA/Dec to SE with **loose** ceilings whose job is convention/sign/units parity, not accuracy:

```rust
const SE_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/equatorial-se-corpus/equatorial-se.csv"
));
const SE_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/equatorial-se-corpus/manifest.txt"
));

// Loose parity ceilings: Moshier(SE) vs DE440(packaged) differ at the arcmin
// level for the Moon and tens of arcsec for planets; this arm catches gross
// convention/sign/units errors, not sub-arcsec accuracy. RA is cos(Dec)-weighted.
const SE_RA_CEILING_ARCSEC: f64 = 600.0;
const SE_DEC_CEILING_ARCSEC: f64 = 600.0;

#[derive(Clone, Debug, PartialEq)]
pub struct EquatorialSeReport {
    pub rows_validated: usize,
    pub max_residual_ra_arcsec: f64,
    pub max_residual_dec_arcsec: f64,
    summary_line: String,
}

impl EquatorialSeReport {
    pub fn summary_line(&self) -> &str { &self.summary_line }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EquatorialSeError {
    ChecksumMismatch { got: u64, want: u64 },
    ManifestDrift { rows_csv: usize, rows_manifest: usize },
    MalformedRow(String),
    MalformedManifest(String),
    UnknownBody(String),
    ChartError { jd_tt: f64, body: String, message: String },
    MeanFallback { jd_tt: f64, body: String },
    MissingEquatorial { jd_tt: f64, body: String },
    CeilingExceeded { jd_tt: f64, body: String, axis: &'static str, residual: f64, ceiling: f64 },
}

impl fmt::Display for EquatorialSeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChecksumMismatch { got, want } =>
                write!(f, "equatorial-se checksum mismatch: got {got:016x} want {want:016x}"),
            Self::ManifestDrift { rows_csv, rows_manifest } =>
                write!(f, "equatorial-se manifest drift: csv {rows_csv} vs manifest {rows_manifest}"),
            Self::MalformedRow(s) => write!(f, "malformed equatorial-se row: {s}"),
            Self::MalformedManifest(s) => write!(f, "malformed equatorial-se manifest: {s}"),
            Self::UnknownBody(s) => write!(f, "unknown body in equatorial-se corpus: {s}"),
            Self::ChartError { jd_tt, body, message } =>
                write!(f, "equatorial-se chart error ({body} @ {jd_tt}): {message}"),
            Self::MeanFallback { jd_tt, body } =>
                write!(f, "equatorial-se ({body} @ {jd_tt}): unexpected mean fallback"),
            Self::MissingEquatorial { jd_tt, body } =>
                write!(f, "equatorial-se ({body} @ {jd_tt}): equatorial channel absent"),
            Self::CeilingExceeded { jd_tt, body, axis, residual, ceiling } =>
                write!(f, "equatorial-se ({body} @ {jd_tt}) {axis}: residual {residual:.2}\u{2033} > ceiling {ceiling:.1}\u{2033}"),
        }
    }
}

impl std::error::Error for EquatorialSeError {}

fn se_manifest_rows() -> Result<(usize, u64), EquatorialSeError> {
    let line = SE_MANIFEST.lines()
        .find(|l| l.trim_start().starts_with("slice"))
        .ok_or_else(|| EquatorialSeError::MalformedManifest("no slice line".into()))?;
    let (mut rows, mut checksum) = (None, None);
    for tok in line.split_whitespace() {
        if let Some(v) = tok.strip_prefix("rows=") {
            rows = Some(v.parse::<usize>().map_err(|e| EquatorialSeError::MalformedManifest(format!("rows: {e}")))?);
        } else if let Some(v) = tok.strip_prefix("checksum=") {
            checksum = Some(v.parse::<u64>().map_err(|e| EquatorialSeError::MalformedManifest(format!("checksum: {e}")))?);
        }
    }
    Ok((
        rows.ok_or_else(|| EquatorialSeError::MalformedManifest("rows= missing".into()))?,
        checksum.ok_or_else(|| EquatorialSeError::MalformedManifest("checksum= missing".into()))?,
    ))
}

pub fn validate_equatorial_se_corpus() -> Result<EquatorialSeReport, EquatorialSeError> {
    let (manifest_rows, manifest_checksum) = se_manifest_rows()?;
    let got = pleiades_apparent::fnv1a64(SE_CSV);
    if got != manifest_checksum {
        return Err(EquatorialSeError::ChecksumMismatch { got, want: manifest_checksum });
    }

    // Parse rows: jd_tt,body,ra_deg,dec_deg.
    let mut parsed: Vec<(f64, String, CelestialBody, f64, f64)> = Vec::new();
    for line in SE_CSV.lines() {
        let t = line.trim();
        if t.starts_with('#') || t.is_empty() || t.starts_with("jd_tt") { continue; }
        let c: Vec<&str> = t.split(',').collect();
        if c.len() != 4 { return Err(EquatorialSeError::MalformedRow(t.to_string())); }
        let jd = c[0].trim().parse::<f64>().map_err(|_| EquatorialSeError::MalformedRow(t.to_string()))?;
        let label = c[1].trim().to_string();
        let body = resolve_body(&label).ok_or_else(|| EquatorialSeError::UnknownBody(label.clone()))?;
        let ra = c[2].trim().parse::<f64>().map_err(|_| EquatorialSeError::MalformedRow(t.to_string()))?;
        let dec = c[3].trim().parse::<f64>().map_err(|_| EquatorialSeError::MalformedRow(t.to_string()))?;
        parsed.push((jd, label, body, ra, dec));
    }
    if parsed.len() != manifest_rows {
        return Err(EquatorialSeError::ManifestDrift { rows_csv: parsed.len(), rows_manifest: manifest_rows });
    }

    let engine = ChartEngine::new(PackagedDataBackend::new());
    let (mut max_ra, mut max_dec, mut validated) = (0.0_f64, 0.0_f64, 0usize);
    for (jd, label, body, se_ra, se_dec) in &parsed {
        let instant = Instant::new(JulianDay::from_days(*jd), TimeScale::Tt);
        let req = ChartRequest::new(instant)
            .with_bodies(vec![body.clone()])
            .with_apparentness(Apparentness::Apparent);
        let snap = engine.chart(&req).map_err(|e| EquatorialSeError::ChartError {
            jd_tt: *jd, body: label.clone(), message: e.to_string(),
        })?;
        let p = snap.placement_for(body).ok_or_else(|| EquatorialSeError::ChartError {
            jd_tt: *jd, body: label.clone(), message: "body not in snapshot".into(),
        })?;
        if p.apparent.is_none() {
            return Err(EquatorialSeError::MeanFallback { jd_tt: *jd, body: label.clone() });
        }
        let eq = p.position.equatorial.ok_or_else(|| EquatorialSeError::MissingEquatorial {
            jd_tt: *jd, body: label.clone(),
        })?;
        let cos_dec = se_dec.to_radians().cos();
        let ra_resid = wrap_deg(eq.right_ascension.degrees() - se_ra).abs() * cos_dec * 3600.0;
        let dec_resid = (eq.declination.degrees() - se_dec).abs() * 3600.0;
        if ra_resid > SE_RA_CEILING_ARCSEC {
            return Err(EquatorialSeError::CeilingExceeded { jd_tt: *jd, body: label.clone(), axis: "ra", residual: ra_resid, ceiling: SE_RA_CEILING_ARCSEC });
        }
        if dec_resid > SE_DEC_CEILING_ARCSEC {
            return Err(EquatorialSeError::CeilingExceeded { jd_tt: *jd, body: label.clone(), axis: "dec", residual: dec_resid, ceiling: SE_DEC_CEILING_ARCSEC });
        }
        max_ra = max_ra.max(ra_resid);
        max_dec = max_dec.max(dec_resid);
        validated += 1;
    }
    let summary_line = format!(
        "Equatorial-SE parity: {validated} rows vs Swiss Ephemeris SEFLG_EQUATORIAL, max RA {max_ra:.2}\u{2033} (cos\u{03b4}-wt) Dec {max_dec:.2}\u{2033}"
    );
    Ok(EquatorialSeReport {
        rows_validated: validated,
        max_residual_ra_arcsec: max_ra,
        max_residual_dec_arcsec: max_dec,
        summary_line,
    })
}

#[cfg(test)]
mod se_tests {
    use super::*;

    #[test]
    fn equatorial_se_parity_passes() {
        let report = validate_equatorial_se_corpus().expect("equatorial-se parity within loose ceilings");
        assert!(report.rows_validated >= 1, "fail-closed floor");
        eprintln!("{}", report.summary_line());
    }
}
```

- [ ] **Step 4: Export and run**

Add to the `pub use equatorial_validation::{…}` export in `lib.rs` (extend the Task 3 export list):

```rust
pub use equatorial_validation::{
    validate_equatorial_goldens, validate_equatorial_se_corpus, EquatorialSeError,
    EquatorialSeReport, EquatorialValidationError, EquatorialValidationReport,
};
```

Run: `cargo test -p pleiades-validate equatorial_validation::se_tests -- --nocapture`
Expected: PASS. If it fails on `ChecksumMismatch`/`ManifestDrift`, the printed `got`/`rows_csv` are the values to write into `manifest.txt` (Step 2); fix the manifest and re-run. If a residual blows the 600″ ceiling for a planet, that signals a real convention error (RA units, sign) — debug rather than widen.

- [ ] **Step 5: Commit**

```bash
git add tools/se-equatorial-reference \
        crates/pleiades-validate/data/equatorial-se-corpus \
        crates/pleiades-validate/src/equatorial_validation.rs \
        crates/pleiades-validate/src/lib.rs
git commit -m "feat(validate): validate-equatorial-se convention-parity gate vs Swiss Ephemeris"
```

---

### Task 5: CLI dispatch, release-gate wiring, help text, dispatch tests

**Files:**
- Modify: `crates/pleiades-validate/src/render/cli.rs` (add the two `validate-equatorial*` dispatch arms; add both gates to `run_all_numeric_gates`; add help lines)
- Test: `crates/pleiades-validate/src/tests/validate_gates.rs` (add dispatch tests)

**Interfaces:**
- Consumes: `crate::validate_equatorial_goldens`, `crate::validate_equatorial_se_corpus` (Tasks 3–4); the `render_cli` test helper used by existing gate tests.
- Produces: CLI commands `validate-equatorial` / `equatorial-gate` and `validate-equatorial-se` / `equatorial-se-gate`; both gates run inside the release gate.

- [ ] **Step 1: Write the failing dispatch tests**

Add to `crates/pleiades-validate/src/tests/validate_gates.rs`:

```rust
#[test]
fn validate_equatorial_command_dispatches_and_reports_pass() {
    let result = render_cli(&["validate-equatorial"])
        .expect("validate-equatorial should succeed on committed goldens");
    assert!(result.contains("Equatorial goldens"),
        "validate-equatorial output should contain 'Equatorial goldens': {result}");
}

#[test]
fn equatorial_gate_alias_matches_validate_equatorial() {
    let primary = render_cli(&["validate-equatorial"]).expect("validate-equatorial should succeed");
    let alias = render_cli(&["equatorial-gate"]).expect("equatorial-gate should succeed");
    assert_eq!(primary, alias);
}

#[test]
fn validate_equatorial_rejects_extra_args() {
    let error = render_cli(&["validate-equatorial", "extra"])
        .expect_err("validate-equatorial should reject extra arguments");
    assert!(error.contains("validate-equatorial does not accept extra arguments"), "{error}");
}

#[test]
fn validate_equatorial_se_command_dispatches_and_reports_pass() {
    let result = render_cli(&["validate-equatorial-se"])
        .expect("validate-equatorial-se should succeed on committed corpus");
    assert!(result.contains("Equatorial-SE parity"),
        "validate-equatorial-se output should contain 'Equatorial-SE parity': {result}");
}

#[test]
fn help_text_mentions_validate_equatorial() {
    let help = render_cli(&["help"]).expect("help should render");
    assert!(help.contains("validate-equatorial"), "help should mention validate-equatorial");
    assert!(help.contains("validate-equatorial-se"), "help should mention validate-equatorial-se");
}
```

(If `render_cli` / the `help` argument differ in this test module, copy the exact pattern from the existing `help_text_mentions_validate_topocentric_and_validate_apparent` test in the same file.)

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p pleiades-validate tests::validate_gates::validate_equatorial`
Expected: FAIL — unknown command (dispatch arms not added yet).

- [ ] **Step 3: Add the dispatch arms**

In `crates/pleiades-validate/src/render/cli.rs`, immediately after the `validate-lilith` arm (lines 197–202), add:

```rust
        Some("validate-equatorial") | Some("equatorial-gate") => {
            ensure_no_extra_args(&args[1..], "validate-equatorial")?;
            crate::validate_equatorial_goldens()
                .map(|report| report.summary_line().to_string())
                .map_err(|e| e.to_string())
        }
        Some("validate-equatorial-se") | Some("equatorial-se-gate") => {
            ensure_no_extra_args(&args[1..], "validate-equatorial-se")?;
            crate::validate_equatorial_se_corpus()
                .map(|report| report.summary_line().to_string())
                .map_err(|e| e.to_string())
        }
```

- [ ] **Step 4: Add both gates to the release numeric-gate set**

In `run_all_numeric_gates()` (`cli.rs` lines 11–20), after the lilith line, add:

```rust
    crate::validate_equatorial_goldens().map_err(|e| format!("equatorial gate failed: {e}"))?;
    crate::validate_equatorial_se_corpus().map_err(|e| format!("equatorial-se gate failed: {e}"))?;
```

- [ ] **Step 5: Add help-text lines**

In the help string (`cli.rs`, the block containing the `validate-lilith` / `lilith-gate` lines, ~line 2051), add after the lilith/eclipses lines:

```
  validate-equatorial       Run the fail-closed apparent equatorial-of-date gate (JPL Horizons apparent RA/Dec, cos\u{03b4}-weighted residuals) over the committed equatorial goldens\n  equatorial-gate           Alias for validate-equatorial\n  validate-equatorial-se    Run the fail-closed equatorial convention-parity gate (Swiss Ephemeris SEFLG_EQUATORIAL, loose ceilings) over the committed equatorial-se corpus\n  equatorial-se-gate        Alias for validate-equatorial-se\n
```

(Match the exact escaping/spacing style of the surrounding lines in that string literal; if the help text is asserted verbatim by a snapshot test, update that fixture too — grep for `validate-lilith` in `crates/pleiades-validate/src/tests/` and in any `*.snap`/expected fixture.)

- [ ] **Step 6: Run the dispatch tests + full validate suite**

Run: `cargo test -p pleiades-validate`
Expected: PASS (new dispatch tests green; no regressions). The release-gate test path now also exercises both new gates.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-validate/src/render/cli.rs crates/pleiades-validate/src/tests/validate_gates.rs
git commit -m "feat(validate): wire validate-equatorial + validate-equatorial-se into CLI and release gate"
```

---

### Task 6: Claims, README, PLAN, follow-ups

> **Scope guard:** Update only the **chart-layer** equatorial claim. The backend-boundary strings "equatorial coordinates are derived with a mean-obliquity transform" (`pleiades-vsop87/src/profiles.rs:284`, `pleiades-elp/src/specification.rs:98`, `pleiades-vsop87/src/source_docs/spec.rs:273`, `pleiades-data/src/lookup.rs:405`) remain TRUE and must NOT change — backends still emit mean-obliquity equatorial for mean rows.

**Files:**
- Modify: `crates/pleiades-backend/src/policy/mod.rs` (the equatorial-output policy summary, line 35) + any test asserting it
- Modify: `README.md` ("Important current limits")
- Modify: `PLAN.md` (status line)
- Modify: `docs/follow-ups.md` (new resolved entry; FU-2 "Next queued" pointer)
- Possibly modify: `crates/pleiades-core/src/compatibility/mod.rs` (profile id + checksum) — only if a compat test demands it (see Step 4)

**Interfaces:** none (docs/metadata only).

- [ ] **Step 1: Update the backend equatorial-output policy summary**

In `crates/pleiades-backend/src/policy/mod.rs` (line 35), the current text is:

```
"ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
```

Replace with a version that records the new chart-layer behavior while keeping the backend-boundary truth:

```
"ecliptic body positions are the default request shape; at the backend boundary equatorial output is derived via mean-obliquity transforms when supported, while the chart layer reports apparent equatorial of date (true obliquity = mean obliquity + nutation-in-obliquity) for release-grade bodies; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
```

Then grep for the old string in tests and update every verbatim copy:

```bash
grep -rn "equatorial output is backend-specific" crates/ --include=*.rs
```

Update each asserting test (e.g. `crates/pleiades-backend/src/request_tests.rs:802`, `:436`, and any policy round-trip test) to the new string.

- [ ] **Step 2: Run the backend + policy tests**

Run: `cargo test -p pleiades-backend`
Expected: PASS (string updated everywhere it is asserted).

- [ ] **Step 3: Update README, PLAN, follow-ups**

- `README.md` "Important current limits": replace the apparent-place bullet's clause that first-party equatorial is mean-only with a statement that **chart-layer body positions now carry apparent equatorial of date (RA/Dec, true obliquity) for release-grade bodies, gated by `validate-equatorial` (JPL Horizons) and `validate-equatorial-se` (Swiss Ephemeris parity); first-party backends remain mean/J2000 at the backend boundary.** (Grep README for "equatorial" and "mean-only" to find the exact sentence.)
- `PLAN.md` status line: remove "equatorial/declination follow-up is the next queued sub-project"; replace with a done note: **"chart-layer apparent equatorial of date (RA/Dec) for release-grade bodies done — gated by `validate-equatorial` + `validate-equatorial-se`."**
- `docs/follow-ups.md`: add a resolved entry (FU-3) recording this sub-project as done (date 2026-06-30, branch `feat/equatorial-declination-output`), and update FU-2's "**Next queued:** equatorial/declination output …" line to mark it done. Carry the same SE build-env note (libclang) for `tools/se-equatorial-reference`.

- [ ] **Step 4: Verify compatibility profile + run the full workspace gate**

Run:
```bash
cargo test --workspace
cargo run -q -p pleiades-validate -- verify-compatibility-profile
cargo run -q -p pleiades-validate -- release-gate-summary
```
Expected: all PASS; the release-gate summary now includes the two equatorial gates.

If `verify-compatibility-profile` or the compat content-checksum test (`crates/pleiades-core/src/compatibility/mod.rs:38`, validated at `validation.rs:264`) reports drift because the policy/summary text changed, bump it the way slice 4 did: edit the profile summary clause, bump `CURRENT_COMPATIBILITY_PROFILE_ID` `0.7.1` → `0.7.2`, run the failing checksum test to get the new value, set `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM`, and update the `0.7.1` mention in `README.md`. (The spec anticipated a profile bump; do it only if a test actually requires it — the equatorial claim lives in the backend policy summary, not the house/ayanamsa catalog the profile enumerates.)

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "docs: promote chart-layer apparent equatorial of date; mark follow-up done"
```

---

## Self-Review

**1. Spec coverage**
- Goal 1 (populate equatorial of date for release-grade bodies) → Task 2. ✓
- Goal 2 (single source of truth from final tropical ecliptic) → Task 2 Step 4 (derive from `position.ecliptic` before sidereal shift). ✓
- Goal 3 (backend invariant preserved) → Global Constraints + Task 6 scope guard (no backend changes; mean strings unchanged). ✓
- Goal 4 (two-authority fail-closed gate) → Task 3 (Horizons) + Task 4 (SE). ✓
- Goal 5 (isolated reusable helper) → Task 1. ✓
- Frame definition (true obliquity, tropical before sidereal) → Task 1 + Task 2 Step 4. ✓
- Edge cases: tropical vs sidereal parity → Task 2 test 2; mean fallback → Task 2 test 3; apsides (uniform, no special-case) → covered by the body-agnostic Task 2 wiring (no branch on body); pole conditioning → cos(Dec) weighting in Tasks 3–4; distance preserved → Task 1 `composes_…` test asserts `distance_au` equality. ✓
- Error handling (graceful degrade) → Task 2 Step 4 (`if let Ok(eq) = …`). ✓
- Validation (both corpora parse, residuals under ceilings, fail-closed) → Tasks 3–4 tests + floors. ✓
- Claims/compat/docs impact → Task 6. ✓

**2. Placeholder scan** — No "TBD/TODO/handle edge cases". The two one-off data-build steps (Task 3 Step 1 Horizons regen; Task 4 Step 2 SE corpus) are explicitly flagged as network/SE-dependent handoffs with exact scripts and output formats, mirroring the lilith/eclipse corpus builds — not vague work. The few "grep and adjust to the real API name" notes (Task 2 request-builder names, Task 5 help-snapshot) are bounded mechanical reconciliations against named files, not open scope.

**3. Type consistency** — `apparent_equatorial_of_date(EclipticCoordinates, f64) -> Result<EquatorialCoordinates, ApparentPlaceError>` defined in Task 1, consumed verbatim in Task 2 and (via the chart engine) Tasks 3–4. `EquatorialValidationReport::summary_line(&self) -> &str` and `EquatorialSeReport::summary_line(&self) -> &str` match their `.map(|report| report.summary_line().to_string())` use in Task 5. CLI command strings `validate-equatorial` / `validate-equatorial-se` match between Task 5 dispatch, help text, and tests. `resolve_body` is defined once in Task 3 and reused by the Task 4 SE arm (same file). ✓
