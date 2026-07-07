# SP-5 Phase, Phase Angle & Magnitude Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Swiss-Ephemeris `swe_pheno()` analogue — `EventEngine::pheno` returning phase angle, illuminated fraction (phase), elongation, apparent disc diameter, and apparent magnitude for the ten major bodies (Sun, Moon, Mercury–Pluto) — gated by a fail-closed `validate-pheno` SE-parity gate.

**Architecture:** A `pheno.rs` module in `pleiades-events` computes the illumination geometry from the apparent Sun–body–Earth triangle (reusing the crate's `geocentric_apparent_ecliptic` helper), and a sibling `magnitude.rs` submodule holds SE 2.10.03's per-body photometric models (Mallama-2018 planets, Vreijs Moon, `mag_elem`-table Pluto, disc-ratio Sun, Saturn ring term) transcribed verbatim from the vendored C source. A committed SE corpus (`tools/se-pheno-reference`) and a fail-closed `validate-pheno` gate enforce parity. Purely additive: new struct + method, no breaking changes.

**Tech Stack:** Rust workspace crates only (no new runtime deps); `libswisseph-sys 0.1.2` for the out-of-workspace reference tool; fnv1a64 checksum-guarded committed corpus.

## Global Constraints

- **No new runtime dependency:** `pleiades-events` gains no new deps for this slice (it already depends on `pleiades-apparent`, which supplies `geocentric_apparent_ecliptic`'s pipeline). Spec §Architecture.
- **API surface:** an engine function on `EventEngine<B>`, NOT new `CelestialBody` variants and NOT chart-layer routing. Spec §Non-goals.
- **Body coverage:** all five outputs for the ten majors (Sun, Moon, Mercury–Pluto). Any other backend-served body gets the four geometric outputs and `apparent_magnitude = None` (documented coverage bound). Spec §Coverage.
- **Fail closed:** out-of-window instants → typed `EventError`; degenerate/non-finite geometry → clamped, never NaN. Spec §Error handling.
- **Window:** 1900–2100 TDB (`WINDOW_START_JD = 2_415_020.5` / `WINDOW_END_JD = 2_488_069.5`); corpus epochs are interior. Spec §Gate.
- **Compatibility profile bump 0.7.10 → 0.7.11; API-stability profile unchanged at 0.2.2** (`pleiades-events` is unpublished; the surface is purely additive). Spec §Docs.
- **fnv1a64** is the repo checksum scheme (`pleiades_apparent::fnv1a64` in-workspace; byte-identical local copy in the tool).
- **Ceilings are measured, not promised:** provisional generous ceilings first, then pinned at ~1.4× measured maxima per metric (SP-4 / house-gate style). Spec §Gate.

## Design notes / errata vs. spec

These are corrections and refinements from pre-plan research against the vendored SE C source (`~/.cargo/registry/src/*/libswisseph-sys-0.1.2/libswisseph/swecl.c`, `swe_pheno` at **swecl.c:3791-4112**, tables at **sweph.h:314-334** and **swecl.c:3748-3790**). They are **binding**. The implementer MUST open that exact source and diff every transcribed constant before pinning ceilings — the committed corpus is the final arbiter.

**E1 — `attr[3]` is DEGREES, not arcsec.** SE computes `attr[3] = asin(dd/2/AUNIT/Δ) * 2 * RADTODEG` (swecl.c:3887) — the disc's full angular diameter in **degrees** (~0.53° for the Sun), despite SE's prose docs saying "arcsec". The struct field is therefore `apparent_diameter_deg`, and the corpus stores SE's raw degree value. (This renames the spec's `apparent_diameter_arcsec`.)

**E2 — The Sun returns phase_angle = 0, phase = 0, elongation = 0.** SE skips the phase block for `ipl == SE_SUN` (swecl.c:3848) and the elongation block for Sun/Earth (swecl.c:3858), leaving those `attr` slots at their initialized `0` (swecl.c:3806-3807). So SE emits **phase = 0** for the Sun, not the geometric 1. The Sun's magnitude IS computed (disc-ratio branch, swecl.c:3892-3896). Our engine special-cases the Sun to return `phase_angle_deg = 0.0, phase_fraction = 0.0, elongation_deg = 0.0` plus the computed diameter and magnitude.

**E3 — Both magnitude toggles are `1`** (swecl.c:3749-3750: `MAG_MALLAMA_2018 1`, `MAG_MOON_VREIJS 1`). The compiled model is therefore: **Mallama-2018** analytic forms for Mercury–Neptune (swecl.c:3923-4005), the **Vreijs** Moon law (swecl.c:3900-3914), **Pluto** via the classic `mag_elem[9] = {-1.00, 0, 0, 0}` branch (`ipl < SE_CHIRON`, swecl.c:4031-4036), and the **Sun** disc-ratio form from `mag_elem[0][0] = -26.86`. Task 2 transcribes exactly these branches.

**E4 — Phase angle and elongation via law of cosines.** SE uses `acos(unit·unit)` on Cartesian vectors (swecl.c:3869, 4066); the law-of-cosines forms `cos α = (r²+Δ²−R²)/(2rΔ)` and `cos ε = (Δ²+R²−r²)/(2ΔR)` are mathematically identical (same triangle angle) and avoid re-deriving Cartesian directions. Clamp both arguments to `[-1, 1]` before `acos` (never-NaN convention). Here `Δ` = geocentric body distance, `R` = Earth–Sun distance, `r` = heliocentric body distance.

**E5 — Saturn's ring term needs BOTH geocentric and heliocentric ecliptic lon/lat** (swecl.c:3963-3983), and `T = (tjd − dt − J2000)/36525` uses the light-time-retarded epoch. Use the literal base `2.7182818` (as SE does at swecl.c:3982), not `std::f64::consts::E`, so the exponential matches bit-for-bit intent.

**E6 — Heliocentric position from the apparent triangle.** `r` and the heliocentric ecliptic direction come from `helio = body_geo_cartesian − sun_geo_cartesian` (the same reconstruction `ephemeris.rs::heliocentric_longitude_deg` uses), NOT a separate retarded heliocentric backend call. SE samples the heliocentric body at `t − dt` with `SEFLG_HELCTR`; the small difference (light-time on the heliocentric leg, gravitational deflection) is a measured cross-pipeline residual the ceilings absorb.

**E7 — API takes `Instant`, not `jd_tdb: f64`** (the spec's signature showed `jd_tdb`), matching every other `EventEngine` method. `Instant` is `pleiades_types::Instant`; extract `jd = instant.julian_day.days()`.

**E8 — Corpus iflag = `SEFLG_MOSEPH | SEFLG_NOGDEFL` = `4 | 512` = `516`.** Moshier (kernel-free) + drop gravitational light deflection (omitted project-wide, matching `geocentric_apparent_ecliptic`, which applies light-time + annual aberration + nutation + precession-to-date but NOT deflection). Aberration and nutation are KEPT (no `SEFLG_NOABERR`/`SEFLG_NONUT`) so SE's geocentric leg is true-apparent-of-date, matching our helper.

**E9 — Diameter table is SE's `pla_diam`, not the crate's `radius_au`.** `semidiameter.rs::radius_au` differs slightly from SE's `pla_diam` (e.g. Moon 1737.4 vs 1737.5 km, Mercury 2439.7 vs 2439.4 km). For tight diameter parity, transcribe SE's `pla_diam` (sweph.h:315-333) into `pheno.rs` rather than reuse `radius_au`. The two tables stay independent.

**E10 — `CelestialBody` has no `Earth` variant** (per SP-4 §R1), so "Earth not addressable" needs no explicit arm — Earth is simply not constructible. Non-major bodies fall through to geometric-only output with `magnitude = None`.

---

## File structure

**`crates/pleiades-events/`:**
- Create `src/magnitude.rs` — SE 2.10.03 per-body photometric models + `pla_diam` table + `MagInputs`.
- Create `src/pheno.rs` — `PhenoData`, `EventEngine::pheno`, the illumination-triangle geometry.
- Modify `src/lib.rs` — `mod magnitude; mod pheno;` + `pub use pheno::PhenoData;`.
- Modify `src/error.rs` — nothing new required (reuses `OutOfWindow`, `Backend`, `MissingCoordinates`); a test is added.
- Create `tests/pheno.rs` — integration tests over the full routing chain.

**`crates/pleiades-validate/`:**
- Create `src/pheno_thresholds.rs`, `src/pheno_validation.rs`.
- Create `data/pheno-corpus/pheno.csv` + `manifest.txt` (committed, generated by the tool).
- Modify `src/lib.rs` (module decls + re-export), `src/render/cli.rs` (gate runner + match arm + help), `src/tests/validate_gates.rs` (alias/help tests), `Cargo.toml` (deps already present via the nod-aps/crossings gates: `pleiades-events`, `pleiades-data`, `pleiades-vsop87`, `pleiades-elp`, `pleiades-jpl`, `pleiades-fict` — verify, add any missing).

**Create `tools/se-pheno-reference/`** — `Cargo.toml`, `src/main.rs`.

**Docs closeout:** `crates/pleiades-core/src/compatibility/mod.rs` (0.7.11 + checksum + summary), version-string tests, `README.md`, `PLAN.md`, `plan/status/01-current-execution-frontier.md`, `plan/status/02-next-slice-candidates.md`.

---

### Task 1: `pleiades-events` — `PhenoData` type + module scaffold

**Files:**
- Create: `crates/pleiades-events/src/pheno.rs` (type + scaffold only)
- Create: `crates/pleiades-events/src/magnitude.rs` (empty scaffold)
- Modify: `crates/pleiades-events/src/lib.rs`

**Interfaces:**
- Produces (used by Tasks 2–4): `pub struct PhenoData { phase_angle_deg, phase_fraction, elongation_deg, apparent_diameter_deg, apparent_magnitude: Option<f64>, body }`.

- [ ] **Step 1: Write the failing test** (inline in `pheno.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::CelestialBody;

    #[test]
    fn pheno_data_is_constructible_and_holds_optional_magnitude() {
        let d = PhenoData {
            phase_angle_deg: 12.0,
            phase_fraction: 0.98,
            elongation_deg: 40.0,
            apparent_diameter_deg: 0.53,
            apparent_magnitude: Some(-4.4),
            body: CelestialBody::Venus,
        };
        assert_eq!(d.body, CelestialBody::Venus);
        assert_eq!(d.apparent_magnitude, Some(-4.4));
        let none = PhenoData { apparent_magnitude: None, ..d };
        assert!(none.apparent_magnitude.is_none());
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p pleiades-events pheno_data_is_constructible`
Expected: FAIL — `PhenoData` not found.

- [ ] **Step 3: Implement.** Create `src/magnitude.rs` with only a doc line so the crate compiles:

```rust
//! Swiss Ephemeris `swe_pheno` apparent-magnitude models (filled by the
//! magnitude task).
```

Create `src/pheno.rs` with the public type:

```rust
//! Illumination phenomena — Swiss Ephemeris `swe_pheno` analogue: phase angle,
//! illuminated fraction, elongation, apparent disc diameter, and apparent
//! magnitude. See [`EventEngine::pheno`](crate::EventEngine::pheno).

use pleiades_types::CelestialBody;

/// Phase, phase-angle, elongation, disc-diameter and magnitude of a body, as
/// computed by [`EventEngine::pheno`](crate::EventEngine::pheno). Geocentric
/// apparent-of-date.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhenoData {
    /// Sun–body–Earth phase angle, degrees in `[0, 180]`. Zero for the Sun.
    pub phase_angle_deg: f64,
    /// Illuminated fraction of the disc, `(1 + cos phase_angle) / 2` in
    /// `[0, 1]`. Zero for the Sun (Swiss Ephemeris leaves it unset — see the
    /// SP-5 plan §E2).
    pub phase_fraction: f64,
    /// Sun–Earth–body elongation, degrees in `[0, 180]`. Zero for the Sun.
    pub elongation_deg: f64,
    /// Apparent angular diameter of the disc, **degrees** (Swiss Ephemeris
    /// `attr[3]`; see the SP-5 plan §E1). Zero for bodies with no disc datum.
    pub apparent_diameter_deg: f64,
    /// Apparent visual magnitude. `Some` for the ten majors (Sun, Moon,
    /// Mercury–Pluto); `None` for any other body (no photometric model).
    pub apparent_magnitude: Option<f64>,
    /// The body served.
    pub body: CelestialBody,
}
```

`lib.rs` — add `mod magnitude;` and `mod pheno;` to the module list, and:

```rust
pub use pheno::PhenoData;
```

- [ ] **Step 4: Run tests** — `cargo test -p pleiades-events pheno`; Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/pheno.rs crates/pleiades-events/src/magnitude.rs crates/pleiades-events/src/lib.rs
git commit -m "feat(events): SP-5 pheno public type + module scaffold"
```

---

### Task 2: `magnitude` — SE 2.10.03 photometric models

**Files:**
- Modify: `crates/pleiades-events/src/magnitude.rs`

**Interfaces:**
- Produces (used by Task 3):
  - `pub(crate) struct MagInputs { phase_angle_deg, r_helio_au, delta_geo_au, diameter_deg, geo_lon_deg, geo_lat_deg, helio_lon_deg, helio_lat_deg, jd_tdb, light_time_days: f64 }`
  - `pub(crate) fn apparent_magnitude(body: &CelestialBody, m: &MagInputs) -> Option<f64>`
  - `pub(crate) fn diameter_deg(body: &CelestialBody, delta_au: f64) -> f64`
  - `pub(crate) const AUNIT_M: f64`, `pub(crate) const CLIGHT_M_S: f64`.

> **Before writing:** open the vendored `swecl.c:3888-4056` and `sweph.h:315-333` and confirm every constant below matches character-for-character. Correct any drift; the corpus will catch the rest.

- [ ] **Step 1: Write failing tests** (inline module)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::CelestialBody;

    fn base_inputs() -> MagInputs {
        MagInputs {
            phase_angle_deg: 0.0,
            r_helio_au: 1.0,
            delta_geo_au: 1.0,
            diameter_deg: 0.0,
            geo_lon_deg: 0.0,
            geo_lat_deg: 0.0,
            helio_lon_deg: 0.0,
            helio_lat_deg: 0.0,
            jd_tdb: 2_451_545.0,
            light_time_days: 0.0,
        }
    }

    #[test]
    fn pluto_is_absolute_minus_one_at_unit_distances() {
        // 5*log10(r*Δ) + (-1.00); r=Δ=1 ⇒ log term 0 ⇒ -1.00.
        let m = apparent_magnitude(&CelestialBody::Pluto, &base_inputs()).unwrap();
        assert!((m + 1.00).abs() < 1e-9, "pluto {m}");
    }

    #[test]
    fn jupiter_at_zero_phase_unit_distance_is_minus_9_395() {
        let m = apparent_magnitude(&CelestialBody::Jupiter, &base_inputs()).unwrap();
        assert!((m + 9.395).abs() < 1e-9, "jupiter {m}");
    }

    #[test]
    fn distance_term_adds_five_log10_r_delta() {
        let mut i = base_inputs();
        i.r_helio_au = 5.0;
        i.delta_geo_au = 4.0;
        let m = apparent_magnitude(&CelestialBody::Pluto, &i).unwrap();
        let expected = -1.00 + 5.0 * (20.0_f64).log10();
        assert!((m - expected).abs() < 1e-9, "pluto far {m}");
    }

    #[test]
    fn sun_magnitude_is_about_minus_26_86_at_one_au() {
        let mut i = base_inputs();
        i.diameter_deg = diameter_deg(&CelestialBody::Sun, 1.0);
        let m = apparent_magnitude(&CelestialBody::Sun, &i).unwrap();
        // At 1 AU the disc equals its mean disc ⇒ fac=1 ⇒ mag = -26.86.
        assert!((m + 26.86).abs() < 1e-6, "sun {m}");
    }

    #[test]
    fn non_major_bodies_have_no_model() {
        let id = pleiades_types::CustomBodyId::new("asteroid", "2060-Chiron");
        let m = apparent_magnitude(&CelestialBody::Custom(id), &base_inputs());
        assert!(m.is_none());
    }

    #[test]
    fn sun_disc_diameter_is_about_half_a_degree() {
        let d = diameter_deg(&CelestialBody::Sun, 1.0);
        assert!((d - 0.533).abs() < 0.01, "sun diam {d}");
    }
}
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p pleiades-events magnitude`; Expected: FAIL (items not found).

- [ ] **Step 3: Implement** (replace file contents)

```rust
//! Swiss Ephemeris 2.10.03 `swe_pheno` apparent-magnitude and disc-diameter
//! models, transcribed verbatim from the vendored C source
//! (`libswisseph-sys-0.1.2/libswisseph/swecl.c:3876-4056`, tables at
//! `sweph.h:315-333` and `swecl.c:3762-3790`). Compile-time toggles in that
//! build are `MAG_MALLAMA_2018 = 1` and `MAG_MOON_VREIJS = 1`, so: Mercury–
//! Neptune use the Mallama-2018 analytic forms, the Moon uses the Vreijs law,
//! Pluto uses the classic `mag_elem` table (−1.00), and the Sun uses the
//! disc-ratio form from `mag_elem[0][0] = −26.86`. See the SP-5 plan §E1–§E9.

use pleiades_types::CelestialBody;

/// AU in metres (SE `AUNIT`, DE431 value; sweph.h:273).
pub(crate) const AUNIT_M: f64 = 1.495_978_707_00e11;
/// Speed of light, m/s (SE `CLIGHT`; sweph.h:274).
pub(crate) const CLIGHT_M_S: f64 = 2.997_924_58e8;
/// Earth equatorial radius, m (SE `EARTH_RADIUS`; sweph.h:282).
const EARTH_RADIUS_M: f64 = 6_378_136.6;
/// J2000.0 as a Julian Day.
const J2000_JD: f64 = 2_451_545.0;

/// SE `pla_diam` — full physical diameters in metres, Sun..Pluto
/// (sweph.h:315-324). Bodies without a disc datum return 0.
fn pla_diam_m(body: &CelestialBody) -> f64 {
    match body {
        CelestialBody::Sun => 1_392_000_000.0,
        CelestialBody::Moon => 3_475_000.0,
        CelestialBody::Mercury => 2_439_400.0 * 2.0,
        CelestialBody::Venus => 6_051_800.0 * 2.0,
        CelestialBody::Mars => 3_389_500.0 * 2.0,
        CelestialBody::Jupiter => 69_911_000.0 * 2.0,
        CelestialBody::Saturn => 58_232_000.0 * 2.0,
        CelestialBody::Uranus => 25_362_000.0 * 2.0,
        CelestialBody::Neptune => 24_622_000.0 * 2.0,
        CelestialBody::Pluto => 1_188_300.0 * 2.0,
        _ => 0.0,
    }
}

/// Apparent disc diameter (degrees), SE `attr[3]` (swecl.c:3878-3887). Zero
/// when the body has no disc datum.
pub(crate) fn diameter_deg(body: &CelestialBody, delta_au: f64) -> f64 {
    let dd = pla_diam_m(body);
    if dd == 0.0 {
        return 0.0;
    }
    let sin_semi = dd / 2.0 / AUNIT_M / delta_au.max(1e-12);
    // Guard the asin domain (never-NaN convention).
    (sin_semi.clamp(-1.0, 1.0)).asin() * 2.0 * (180.0 / core::f64::consts::PI)
}

/// Inputs to the per-body magnitude model.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MagInputs {
    /// Phase angle, degrees.
    pub phase_angle_deg: f64,
    /// Heliocentric distance r, AU.
    pub r_helio_au: f64,
    /// Geocentric distance Δ, AU.
    pub delta_geo_au: f64,
    /// Apparent disc diameter, degrees (Sun only).
    pub diameter_deg: f64,
    /// Geocentric apparent ecliptic longitude/latitude, degrees (Saturn ring).
    pub geo_lon_deg: f64,
    pub geo_lat_deg: f64,
    /// Heliocentric ecliptic longitude/latitude, degrees (Saturn ring).
    pub helio_lon_deg: f64,
    pub helio_lat_deg: f64,
    /// TDB Julian Day (Neptune time term, Saturn ring epoch).
    pub jd_tdb: f64,
    /// Light-time, days (Saturn ring epoch `tjd − dt`).
    pub light_time_days: f64,
}

/// `5·log10(r·Δ)`, the common distance term.
fn dist_term(m: &MagInputs) -> f64 {
    5.0 * (m.r_helio_au * m.delta_geo_au).log10()
}

/// Apparent visual magnitude for the ten majors; `None` otherwise.
/// Verbatim SE 2.10.03 branch structure (swecl.c:3891-4056).
pub(crate) fn apparent_magnitude(body: &CelestialBody, m: &MagInputs) -> Option<f64> {
    let a = m.phase_angle_deg;
    let a2 = a * a;
    Some(match body {
        CelestialBody::Sun => {
            // Disc-ratio form (swecl.c:3892-3896): mag = −26.86 − 2.5·log10(fac),
            // fac = (Δ-diameter / mean-diameter-at-1AU)².
            let mean = diameter_deg(&CelestialBody::Sun, 1.0);
            let fac = (m.diameter_deg / mean).powi(2);
            -26.86 - 2.5 * fac.log10()
        }
        CelestialBody::Moon => {
            // Vreijs (swecl.c:3900-3914).
            let base = if a <= 147.138_546_5 {
                -21.62 + 0.026 * a.abs() + 0.000_000_004 * a.powi(4)
            } else {
                -4.5444 - 2.5 * (180.0 - a).powi(3).log10()
            };
            base + 5.0
                * (m.delta_geo_au * m.r_helio_au * AUNIT_M / EARTH_RADIUS_M).log10()
        }
        CelestialBody::Mercury => {
            // Mallama 2018 (swecl.c:3923-3927).
            let a3 = a2 * a;
            let a4 = a3 * a;
            let a5 = a4 * a;
            let a6 = a5 * a;
            (-0.613 + a * 6.3280e-2 - a2 * 1.6336e-3 + a3 * 3.3644e-5
                - a4 * 3.4265e-7 + a5 * 1.6893e-9 - a6 * 3.0334e-12)
                + dist_term(m)
        }
        CelestialBody::Venus => {
            // Mallama 2018 (swecl.c:3928-3935).
            let base = if a <= 163.7 {
                -4.384 - a * 1.044e-3 + a2 * 3.687e-4 - a2 * a * 2.814e-6
                    + a2 * a2 * 8.938e-9
            } else {
                236.058_28 - a * 2.819_14 + a2 * 8.390_34e-3
            };
            base + dist_term(m)
        }
        CelestialBody::Mars => {
            // Mallama 2018 (swecl.c:3938-3956).
            let base = if a <= 50.0 {
                -1.601 + a * 0.02267 - a2 * 0.000_130_2
            } else {
                -0.367 - a * 0.02573 + a2 * 0.000_344_5
            };
            base + dist_term(m)
        }
        CelestialBody::Jupiter => {
            // Mallama 2018 (swecl.c:3957-3962).
            (-9.395 - a * 3.7e-4 + a2 * 6.16e-4) + dist_term(m)
        }
        CelestialBody::Saturn => {
            // Mallama 2018 + Meeus ring (swecl.c:3963-3983).
            let t = (m.jd_tdb - m.light_time_days - J2000_JD) / 36_525.0;
            let inc = (28.075216 - 0.012998 * t + 0.000004 * t * t).to_radians();
            let om = (169.508470 + 1.394681 * t + 0.000412 * t * t).to_radians();
            let sin_b = inc.sin() * m.geo_lat_deg.to_radians().cos()
                * (m.geo_lon_deg.to_radians() - om).sin()
                - inc.cos() * m.geo_lat_deg.to_radians().sin();
            let sin_b2 = inc.sin() * m.helio_lat_deg.to_radians().cos()
                * (m.helio_lon_deg.to_radians() - om).sin()
                - inc.cos() * m.helio_lat_deg.to_radians().sin();
            let sin_b = ((sin_b.clamp(-1.0, 1.0).asin()
                + sin_b2.clamp(-1.0, 1.0).asin())
                / 2.0)
                .sin()
                .abs();
            (-8.914 - 1.825 * sin_b + 0.026 * a
                - 0.378 * sin_b * 2.7182818_f64.powf(-2.25 * a))
                + dist_term(m)
        }
        CelestialBody::Uranus => {
            // Mallama 2018, sub-Earth-latitude term dropped, −0.05 (swecl.c:3984-3994).
            (-7.110 + a * 6.587e-3 + a2 * 1.045e-4) + dist_term(m) - 0.05
        }
        CelestialBody::Neptune => {
            // Mallama 2018, time-dependent brightening (swecl.c:3995-4005).
            let base = if m.jd_tdb < 2_444_239.5 {
                -6.89
            } else if m.jd_tdb <= 2_451_544.5 {
                -6.89 - 0.0055 * (m.jd_tdb - 2_444_239.5) / 365.25
            } else {
                -7.00
            };
            base + dist_term(m)
        }
        CelestialBody::Pluto => {
            // Classic mag_elem[9] = {−1.00, 0, 0, 0} (swecl.c:4031-4036).
            -1.00 + dist_term(m)
        }
        _ => return None,
    })
}
```

- [ ] **Step 4: Run** — `cargo test -p pleiades-events magnitude`; Expected: PASS.
- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/magnitude.rs
git commit -m "feat(events): SP-5 SE 2.10.03 apparent-magnitude + disc-diameter models"
```

---

### Task 3: `pheno` — illumination geometry + `EventEngine::pheno`

**Files:**
- Modify: `crates/pleiades-events/src/pheno.rs`

**Interfaces:**
- Consumes: Task 2 (`magnitude::{apparent_magnitude, diameter_deg, MagInputs, AUNIT_M, CLIGHT_M_S}`), `crate::crossings::EventEngine`, `crate::ephemeris::{geocentric_apparent_ecliptic, spherical_to_cartesian, body_label}`, `crate::error::EventError`.
- Produces: `pub fn pheno(&self, body: CelestialBody, instant: Instant) -> Result<PhenoData, EventError>`.

> `body_label` is the same `&'static str` helper `crossings.rs` passes to `geocentric_apparent_longitude_deg`. If it is not already reachable from `pheno.rs`, import it from its defining module (`crate::ephemeris` or `crate::crossings` — grep `fn body_label`).

- [ ] **Step 1: Write the failing tests** (inline in `pheno.rs`, extend the existing test module; these use the in-crate `LinearSunMoon` test backend)

```rust
    use crate::crossings::EventEngine;
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{Instant, JulianDay, TimeScale};

    fn tdb(jd: f64) -> Instant {
        Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
    }

    #[test]
    fn sun_reports_zero_phase_and_a_magnitude() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_545.0));
        let d = engine.pheno(CelestialBody::Sun, tdb(2_451_545.0)).unwrap();
        assert_eq!(d.phase_angle_deg, 0.0);
        assert_eq!(d.phase_fraction, 0.0); // SE leaves it 0 for the Sun (§E2)
        assert_eq!(d.elongation_deg, 0.0);
        assert!(d.apparent_diameter_deg > 0.4 && d.apparent_diameter_deg < 0.6);
        assert!(d.apparent_magnitude.unwrap() < -20.0);
    }

    #[test]
    fn moon_phase_fraction_is_in_unit_interval() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_545.0));
        let d = engine.pheno(CelestialBody::Moon, tdb(2_451_545.0)).unwrap();
        assert!((0.0..=1.0).contains(&d.phase_fraction), "phase {}", d.phase_fraction);
        assert!((0.0..=180.0).contains(&d.phase_angle_deg));
        assert!(d.apparent_magnitude.is_some());
    }

    #[test]
    fn out_of_window_fails_closed() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_545.0));
        let err = engine.pheno(CelestialBody::Sun, tdb(2_400_000.5)).unwrap_err();
        assert!(matches!(err, EventError::OutOfWindow { .. }));
    }
```

> If `LinearSunMoon` cannot serve the geocentric-apparent Sun position required here (it serves Sun+Moon; confirm by reading `pleiades-backend/src/test_backend.rs`), keep only `out_of_window_fails_closed` as the unit test and move `sun_*`/`moon_*` coverage to the Task-4 integration file (which uses the production backend chain). Do not invent backend behavior.

- [ ] **Step 2: Run to verify failure** — `cargo test -p pleiades-events pheno`; Expected: FAIL.

- [ ] **Step 3: Implement.** Append to `pheno.rs` (above the test module):

```rust
use crate::crossings::EventEngine;
use crate::ephemeris::{body_label, geocentric_apparent_ecliptic, spherical_to_cartesian};
use crate::error::EventError;
use crate::magnitude::{apparent_magnitude, diameter_deg, MagInputs, AUNIT_M, CLIGHT_M_S};
use pleiades_backend::EphemerisBackend;
use pleiades_types::Instant;

/// `(longitude_deg, latitude_deg)` of a Cartesian ecliptic vector.
fn lon_lat_deg(v: [f64; 3]) -> (f64, f64) {
    let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let lon = v[1].atan2(v[0]).to_degrees().rem_euclid(360.0);
    let lat = if r == 0.0 {
        0.0
    } else {
        (v[2] / r).clamp(-1.0, 1.0).asin().to_degrees()
    };
    (lon, lat)
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Phase, phase angle, elongation, apparent disc diameter, and apparent
    /// magnitude of a body — Swiss Ephemeris `swe_pheno` analogue. Geocentric
    /// apparent-of-date. Full five-output parity for the ten majors (Sun, Moon,
    /// Mercury–Pluto); other backend-served bodies get the four geometric
    /// outputs with `apparent_magnitude = None`.
    ///
    /// Fail-closed: out-of-window instants return [`EventError::OutOfWindow`];
    /// missing backend coordinates return [`EventError::Backend`] /
    /// [`EventError::MissingCoordinates`]. Never returns NaN.
    pub fn pheno(
        &self,
        body: CelestialBody,
        instant: Instant,
    ) -> Result<PhenoData, EventError> {
        let jd = instant.julian_day.days();
        self.check_window(jd)?;

        // Geocentric apparent-of-date body and Sun.
        let (blon, blat, delta) =
            geocentric_apparent_ecliptic(&self.backend, body.clone(), body_label(&body), jd)?;
        let (slon, slat, r_sun) =
            geocentric_apparent_ecliptic(&self.backend, CelestialBody::Sun, "Sun", jd)?;

        // Heliocentric body = geocentric body − geocentric Sun (§E6).
        let bvec = spherical_to_cartesian(blon, blat, delta);
        let svec = spherical_to_cartesian(slon, slat, r_sun);
        let hvec = [bvec[0] - svec[0], bvec[1] - svec[1], bvec[2] - svec[2]];
        let r = (hvec[0] * hvec[0] + hvec[1] * hvec[1] + hvec[2] * hvec[2]).sqrt();
        let (hlon, hlat) = lon_lat_deg(hvec);

        let diameter = diameter_deg(&body, delta);
        let dt = delta * AUNIT_M / CLIGHT_M_S / 86_400.0;
        let mag_inputs = MagInputs {
            phase_angle_deg: 0.0, // set per-branch below
            r_helio_au: r,
            delta_geo_au: delta,
            diameter_deg: diameter,
            geo_lon_deg: blon,
            geo_lat_deg: blat,
            helio_lon_deg: hlon,
            helio_lat_deg: hlat,
            jd_tdb: jd,
            light_time_days: dt,
        };

        // The Sun is special: SE leaves phase/phase-angle/elongation at 0 (§E2).
        if body == CelestialBody::Sun {
            return Ok(PhenoData {
                phase_angle_deg: 0.0,
                phase_fraction: 0.0,
                elongation_deg: 0.0,
                apparent_diameter_deg: diameter,
                apparent_magnitude: apparent_magnitude(&body, &mag_inputs),
                body,
            });
        }

        // Phase angle and elongation via law of cosines (§E4).
        let cos_alpha =
            ((r * r + delta * delta - r_sun * r_sun) / (2.0 * r * delta)).clamp(-1.0, 1.0);
        let phase_angle_deg = cos_alpha.acos().to_degrees();
        let phase_fraction = (1.0 + cos_alpha) / 2.0;
        let cos_elong =
            ((delta * delta + r_sun * r_sun - r * r) / (2.0 * delta * r_sun)).clamp(-1.0, 1.0);
        let elongation_deg = cos_elong.acos().to_degrees();

        let magnitude_inputs = MagInputs { phase_angle_deg, ..mag_inputs };
        Ok(PhenoData {
            phase_angle_deg,
            phase_fraction,
            elongation_deg,
            apparent_diameter_deg: diameter,
            apparent_magnitude: apparent_magnitude(&body, &magnitude_inputs),
            body,
        })
    }
}
```

- [ ] **Step 4: Run tests** — `cargo test -p pleiades-events pheno`; Expected: PASS.
- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/pheno.rs
git commit -m "feat(events): SP-5 pheno illumination geometry + EventEngine::pheno"
```

---

### Task 4: Integration tests over the real backend chain

**Files:**
- Create: `crates/pleiades-events/tests/pheno.rs`
- Possibly modify: `crates/pleiades-events/Cargo.toml` (dev-deps for the production backend chain — mirror `tests/nod_aps.rs`'s dev-dep block; add `pleiades-data`, `pleiades-vsop87`, `pleiades-elp`, `pleiades-jpl` if absent).

**Interfaces:**
- Consumes: `pleiades_events::{EventEngine, PhenoData}`, the production backends.

- [ ] **Step 1: Write the integration test.** Build the same routing chain `tests/nod_aps.rs` uses (copy its backend-construction block verbatim so the chain matches production), then assert cross-body invariants at J2000:

```rust
use pleiades_backend::{CompositeBackend, RoutingBackend};
use pleiades_data::PackagedDataBackend;
use pleiades_elp::ElpBackend;
use pleiades_events::EventEngine;
use pleiades_fict::FictitiousBackend;
use pleiades_jpl::JplSnapshotBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
use pleiades_vsop87::Vsop87Backend;

// Same production routing chain as tests/nod_aps.rs.
fn engine() -> EventEngine<RoutingBackend> {
    EventEngine::new(RoutingBackend::new(vec![
        Box::new(PackagedDataBackend::new()),
        Box::new(CompositeBackend::new(Vsop87Backend::new(), ElpBackend::new())),
        Box::new(JplSnapshotBackend::new()),
        Box::new(FictitiousBackend::new(PackagedDataBackend::new())),
    ]))
}

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

#[test]
fn majors_report_finite_outputs_with_magnitude() {
    let e = engine();
    for body in [
        CelestialBody::Sun, CelestialBody::Moon, CelestialBody::Mercury,
        CelestialBody::Venus, CelestialBody::Mars, CelestialBody::Jupiter,
        CelestialBody::Saturn, CelestialBody::Uranus, CelestialBody::Neptune,
        CelestialBody::Pluto,
    ] {
        let d = e.pheno(body.clone(), tdb(2_451_545.0)).unwrap();
        assert!(d.phase_angle_deg.is_finite() && (0.0..=180.0).contains(&d.phase_angle_deg));
        assert!((0.0..=1.0).contains(&d.phase_fraction));
        assert!((0.0..=180.0).contains(&d.elongation_deg));
        assert!(d.apparent_diameter_deg.is_finite() && d.apparent_diameter_deg >= 0.0);
        let mag = d.apparent_magnitude.expect("major bodies carry magnitude");
        assert!(mag.is_finite(), "{body:?} mag {mag}");
    }
}

#[test]
fn inner_planet_phase_tracks_phase_angle() {
    // Illuminated fraction must fall as the phase angle grows.
    let e = engine();
    let d = e.pheno(CelestialBody::Venus, tdb(2_451_545.0)).unwrap();
    let expected = (1.0 + d.phase_angle_deg.to_radians().cos()) / 2.0;
    assert!((d.phase_fraction - expected).abs() < 1e-9);
}
```

- [ ] **Step 2: Run**

Run: `cargo test -p pleiades-events --test pheno`
Expected: PASS. (If the dev-deps for the backend chain are missing, `cargo` fails to resolve the imports — add them to `[dev-dependencies]` as noted in Files, mirroring `tests/nod_aps.rs`, then re-run.)

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-events/tests/pheno.rs crates/pleiades-events/Cargo.toml
git commit -m "test(events): SP-5 pheno integration invariants over the routing chain"
```

---

### Task 5: `tools/se-pheno-reference` + committed corpus

**Files:**
- Create: `tools/se-pheno-reference/Cargo.toml`, `tools/se-pheno-reference/src/main.rs`
- Create (generated): `crates/pleiades-validate/data/pheno-corpus/pheno.csv`, `manifest.txt`
- Modify: root `Cargo.toml` — add `tools/se-pheno-reference` to the workspace `exclude` list.

**Interfaces:**
- Produces: committed corpus CSV with header
  `label,se_body,jd_tt,phase_angle,phase,elongation,diameter_deg,magnitude`
  and `manifest.txt` line `file: pheno.csv rows=<n> checksum=<fnv1a64>`.

- [ ] **Step 1: Write `Cargo.toml`** (clone the fictitious/nodaps tool shape)

```toml
[package]
name = "se-pheno-reference"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]

[dependencies]
libswisseph-sys = "0.1.2"
```

- [ ] **Step 2: Write `src/main.rs`.** Follow `tools/se-fictitious-reference/src/main.rs` exactly for: provenance doc header, the local `fnv1a64` copy, `swe_version` print, fail-closed panics on SE errors/non-finite values, `--out`/`--dry-run` handling, and manifest emission. Add a `LICENSE-NOTES.md` copied from a sibling tool. The tool-specific parts:

```rust
use libswisseph_sys::raw::{swe_pheno, swe_set_ephe_path, swe_version};
use std::os::raw::{c_char, c_double, c_int};

// SE default apparent minus gravitational deflection (plan §E8):
// SEFLG_MOSEPH=4, SEFLG_NOGDEFL=512.
const IFLAG: c_int = 4 | 512;

// Interior 1900–2100 epochs (same set as the SP-4 nod-aps corpus): span crosses
// Neptune's 2451544.5 magnitude break and a range of Saturn ring openings and
// inner-planet phase angles.
const EPOCHS: [f64; 8] = [
    2_415_100.5, 2_433_282.5, 2_441_683.5, 2_451_545.0,
    2_459_000.5, 2_466_154.5, 2_477_476.5, 2_488_021.5,
];

/// (label, SE body number): the ten majors.
const BODIES: [(&str, c_int); 10] = [
    ("Sun", 0), ("Moon", 1), ("Mercury", 2), ("Venus", 3), ("Mars", 4),
    ("Jupiter", 5), ("Saturn", 6), ("Uranus", 7), ("Neptune", 8), ("Pluto", 9),
];

fn emit_row(csv: &mut String, label: &str, se_body: c_int, jd_tt: f64) {
    let mut attr = [0.0_c_double; 20];
    let mut serr = [0 as c_char; 256];
    let ret = unsafe {
        swe_pheno(jd_tt, se_body, IFLAG, attr.as_mut_ptr(), serr.as_mut_ptr())
    };
    assert!(ret >= 0, "swe_pheno failed for {label} at {jd_tt}");
    // attr[0]=phase angle, [1]=phase, [2]=elongation, [3]=diameter(deg), [4]=magnitude.
    for i in 0..5 {
        assert!(attr[i].is_finite(), "non-finite attr[{i}] for {label} at {jd_tt}");
    }
    csv.push_str(&format!(
        "{label},{se_body},{jd_tt:.9},{:.9},{:.9},{:.9},{:.9},{:.9}\n",
        attr[0], attr[1], attr[2], attr[3], attr[4]
    ));
}
```

`build_csv` emits `BODIES × EPOCHS` in order = **80 rows**. Header first, then one `emit_row` per (body, epoch).

- [ ] **Step 3: Generate.** (Needs `libclang-dev` + `LIBCLANG_PATH` for the FFI build — same as every `se-*-reference` tool. Moshier is kernel-free; no `.se1` files needed.)

```bash
cd tools/se-pheno-reference
cargo run --release -- --out ../../crates/pleiades-validate/data/pheno-corpus
```

Expected: writes `pheno.csv` (80 data rows) + `manifest.txt`.

- [ ] **Step 4: Sanity-check and commit**

```bash
head -4 crates/pleiades-validate/data/pheno-corpus/pheno.csv
grep -c . crates/pleiades-validate/data/pheno-corpus/pheno.csv   # 81 (header + 80)
cat crates/pleiades-validate/data/pheno-corpus/manifest.txt
git add tools/se-pheno-reference crates/pleiades-validate/data/pheno-corpus Cargo.toml
git commit -m "tools: SP-5 Swiss-Ephemeris pheno reference generator + committed 80-row corpus"
```

Inspect the Sun rows now (relevant to §E2): confirm `phase` and `elongation` columns are `0` for the Sun. Note the diameter column magnitude (~0.5 for the Sun) to confirm §E1 (degrees).

---

### Task 6: `validate-pheno` gate (provisional ceilings)

**Files:**
- Create: `crates/pleiades-validate/src/pheno_thresholds.rs`, `crates/pleiades-validate/src/pheno_validation.rs`
- Modify: `crates/pleiades-validate/src/lib.rs`, `src/render/cli.rs`, `src/tests/validate_gates.rs`, `Cargo.toml`

**Interfaces:**
- Consumes: the corpus (Task 5), `pleiades_events::{EventEngine, PhenoData}`, `pleiades_apparent::fnv1a64`.
- Produces: `pub fn validate_pheno_corpus() -> Result<PhenoReport, PhenoError>`, `PhenoReport` with `passed()` / `summary_line()`, `pub const EXPECTED_ROWS: usize = 80;`, CLI `validate-pheno` / `pheno-gate`.

- [ ] **Step 1: Write `pheno_thresholds.rs`** — provisional GENEROUS ceilings (tightened in Task 7), one `pub const` per metric, mirroring `nod_aps_thresholds.rs`'s doc style. Metrics and provisional values:

```rust
//! Provisional SP-5 pheno gate ceilings. Pinned from measured residuals in
//! Task 7 (SP-4 method: ~1.4× measured maxima). See the SP-5 plan §Gate.

/// Phase-angle residual vs Swiss Ephemeris, arcsec.
pub const PHASE_ANGLE_ARCSEC: f64 = 3600.0;
/// Illuminated-fraction residual (absolute).
pub const PHASE_FRACTION_ABS: f64 = 1e-2;
/// Elongation residual, arcsec.
pub const ELONGATION_ARCSEC: f64 = 3600.0;
/// Apparent-diameter residual, arcsec.
pub const DIAMETER_ARCSEC: f64 = 60.0;
/// Apparent-magnitude residual, all bodies except Saturn.
pub const MAGNITUDE_ABS: f64 = 0.5;
/// Apparent-magnitude residual for Saturn (ring term is the widest).
pub const SATURN_MAGNITUDE_ABS: f64 = 1.0;
```

- [ ] **Step 2: Write `pheno_validation.rs`** by direct imitation of `nod_aps_validation.rs` (same `include_str!` + manifest + `check_checksum` via `pleiades_apparent::fnv1a64` + `EXPECTED_ROWS` + measure/validate split + `PhenoError`/`PhenoReport` shape). The gate-specific parts:

```rust
const CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"), "/data/pheno-corpus/pheno.csv"));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"), "/data/pheno-corpus/manifest.txt"));
const CSV_FILE: &str = "pheno.csv";
pub const EXPECTED_ROWS: usize = 80;
```

Row parse: 8 fields per the Task-5 schema. Body mapping `body_from_se(se_body: i64) -> Option<CelestialBody>`: 0 Sun, 1 Moon, 2 Mercury, 3 Venus, 4 Mars, 5 Jupiter, 6 Saturn, 7 Uranus, 8 Neptune, 9 Pluto (copy the small-int arm pattern from `nod_aps_validation.rs::body_from_se`, majors only).

Backend + engine — the production-style chain (copy the exact `RoutingBackend::new(vec![...])` from `nod_aps_validation.rs`):

```rust
let engine = EventEngine::new(backend); // same backend vec as the nod-aps gate
```

Per row: parse `(label, se_body, jd_tt, phase_angle, phase, elongation, diameter_deg, magnitude)`; map `se_body → CelestialBody`; call `engine.pheno(body, Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tdb))`. **Tier 1** invariants — every output finite, `phase_angle ∈ [0,180]`, `phase ∈ [0,1]`, `elongation ∈ [0,180]`, `diameter ≥ 0`, magnitude `Some` for all ten majors. **Tier 2** residuals — `|Δphase_angle|·3600` arcsec, `|Δphase|` absolute, `|Δelongation|·3600` arcsec, `|Δdiameter|·3600` arcsec, `|Δmagnitude|` — accumulated into maxima. The Sun's `phase`/`phase_angle`/`elongation` are compared exactly against the corpus's zeros (do NOT feed them into the `[0,180]`/`[0,1]` band checks as residuals — they will be 0 on both sides). Magnitude residual is bucketed: Saturn → `SATURN_MAGNITUDE_ABS`, all others → `MAGNITUDE_ABS`. `validate_pheno_corpus()` applies the ceilings fail-closed (`ToleranceExceeded { metric, label, residual, ceiling }`). `summary_line()` states the coverage bound explicitly: `"…; magnitude covers the ten majors only (asteroids/fictitious geometric-only, gate-unreferenced)"`.

- [ ] **Step 3: Register.** `lib.rs`: `mod pheno_thresholds;` + `pub mod pheno_validation;` next to the nod-aps pair; re-export `pub use pheno_validation::{validate_pheno_corpus, PhenoError, PhenoReport};`. `render/cli.rs`: append to `run_all_numeric_gates()` after the nod-aps line:

```rust
    crate::validate_pheno_corpus().map_err(|e| format!("pheno gate failed: {e}"))?;
```

Match arm (next to the nod-aps arm):

```rust
    Some("validate-pheno") | Some("pheno-gate") => {
        ensure_no_extra_args(&args[1..], "validate-pheno")?;
        crate::validate_pheno_corpus()
            .map(|r| r.summary_line())
            .map_err(|e| e.to_string())
    }
```

Help banner (two lines, next to the nod-aps entries):

```
validate-pheno            Phase/phase-angle/magnitude SE-parity gate (swe_pheno)
pheno-gate                Alias for validate-pheno
```

- [ ] **Step 4: Tests.** In `src/tests/validate_gates.rs`, mirror the nod-aps alias/help tests (`pheno-gate` runs and succeeds; help mentions both spellings). Near `cli.rs`'s `run_all_numeric_gates_includes_nod_aps_and_passes`, add `run_all_numeric_gates_includes_pheno_and_passes`.

- [ ] **Step 5: Run**

Run: `cargo test -p pleiades-validate validate_gates && cargo run -p pleiades-cli -- validate-pheno` (copy the exact gate invocation from the nod-aps gate's test).
Expected: gate PASSES under provisional ceilings and prints a summary line with per-metric maxima. **Record the printed maxima — Task 7 needs them.** If any metric blows past even the provisional ceilings, STOP and debug (likely suspects, in order: §E6 heliocentric reconstruction sign, §E1 degrees-vs-arcsec on diameter, §E2 Sun zeros mis-handled, a transcribed magnitude coefficient typo vs swecl.c, §E8 iflag mismatch).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-validate
git commit -m "feat(validate): SP-5 validate-pheno SE-parity gate (provisional ceilings)"
```

---

### Task 7: Measure and pin ceilings

**Files:**
- Modify: `crates/pleiades-validate/src/pheno_thresholds.rs`

- [ ] **Step 1: Harvest per-metric maxima** from the gate summary line (`cargo run -p pleiades-cli -- validate-pheno`).

- [ ] **Step 2: Pin ceilings at ~1.4× measured maxima** (round up to clean values), replacing every provisional constant; document the measured max in each constant's doc comment (`nod_aps_thresholds` style: `/// Measured max X on YYYY-MM-DD corpus; ceiling ~1.4×.`). Expected magnitudes (sanity prior, not requirements): phase-angle/elongation arcsecond-to-few-arcsecond (§E6 cross-pipeline heliocentric); phase-fraction ~1e-5; diameter arcsecond-class; magnitude millimag-to-few-hundredths, Saturn the widest (ring term). If any metric is unexpectedly large (e.g. magnitude > ~0.2 for a non-Saturn body), diff the transcribed coefficients against `swecl.c` before pinning — a wide residual usually means a coefficient typo, not a real model gap.

- [ ] **Step 3: Full verification**

Run: `cargo test --workspace`
Expected: PASS — including `run_all_numeric_gates_includes_pheno_and_passes` under the pinned ceilings.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-validate/src/pheno_thresholds.rs
git commit -m "feat(validate): SP-5 pin pheno gate ceilings from measured residuals"
```

---

### Task 8: Docs + compatibility profile closeout

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/mod.rs`, `crates/pleiades-validate/src/tests/render_request.rs:333`, `crates/pleiades-cli/src/cli/tests/summary_commands.rs:433`, `README.md`, `PLAN.md`, `plan/status/01-current-execution-frontier.md`, `plan/status/02-next-slice-candidates.md`

- [ ] **Step 1: Profile bump.** In `compatibility/mod.rs`: set `CURRENT_COMPATIBILITY_PROFILE_ID` to `"pleiades-compatibility-profile/0.7.11"`; append an `SP-5 (phase, phase angle and magnitude) additions:` paragraph to `CURRENT_COMPATIBILITY_PROFILE_SUMMARY` (SP-4's trailing paragraph is the template — name the API `EventEngine::pheno`, the five outputs, the ten-major body coverage + the magnitude-only-for-majors bound, the §E1 degrees unit, the §E2 Sun-zeros behavior, the gate name/aliases, and measured accuracy per metric incl. the Saturn-magnitude carve-out). Run the profile checksum test, take the new value from the failure message, update `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` in the same commit (the doc comment at mod.rs:31-38 describes this loop).

- [ ] **Step 2: Version-string tests.** Update the two `0.7.10` assertions (`render_request.rs:333`, `summary_commands.rs:433`) to `0.7.11`. API-stability stays `0.2.2`.

- [ ] **Step 3: Docs.** README "Current state": add a bullet for SP-5 (mirror the SP-4 bullet's structure: what shipped, the gate, measured accuracy per metric, the magnitude coverage bound, and the §E1/§E2 notes). `PLAN.md` status line: append the SP-5 completion sentence with date + measured numbers, and remove `swe_pheno` from the remaining-follow-ups list. `plan/status/01-current-execution-frontier.md` and `plan/status/02-next-slice-candidates.md`: mark SP-5 done in the event-engine track; remaining candidates become custom fictitious-body elements, occultations, and central-path cartography for solar eclipses. (`CHANGELOG.md` is NOT hand-edited — it is regenerated by git-cliff from the conventional-commit subjects, per `cliff.toml`; the `feat(...)`/`docs(...)` commit messages in this plan supply its content.)

- [ ] **Step 4: Full workspace verification**

Run: `cargo test --workspace && cargo clippy --workspace -- -D warnings`
Expected: PASS, no warnings.

- [ ] **Step 5: Commit**

```bash
git add README.md PLAN.md plan/status crates/pleiades-core crates/pleiades-validate crates/pleiades-cli
git commit -m "docs(events): SP-5 declare phase/phase-angle/magnitude; profile 0.7.11; mark SP-5 done"
```

---

## Self-review notes (already applied)

- **Spec coverage:** `PhenoData` type + `Option<f64>` magnitude → Task 1; five outputs incl. per-body magnitude models → Tasks 2–3; geometry (phase angle, phase, elongation, diameter) → Task 3; Sun/Moon special cases → Tasks 2/3 (§E2); full-major coverage over the routing chain → Task 4; gate + corpus + ceilings → Tasks 5–7; docs/profile → Task 8. The spec's "measured-then-pinned" ceilings are Task 7; every spec §Non-goal (asteroid/fictitious magnitude, topocentric, new body variants, speeds) is preserved.
- **Type consistency:** `PhenoData`/field names identical across Tasks 1/3/4/6; `MagInputs`/`apparent_magnitude`/`diameter_deg` identical across Tasks 2/3; corpus schema (8 cols) matches Tasks 5/6; `EXPECTED_ROWS = 80` matches Task-5 row math (10 bodies × 8 epochs).
- **Known deliberate deviations from SE (all measured by the gate, none silent):** diameter in degrees not arcsec (§E1); Sun phase/elongation zeroed (§E2); heliocentric position reconstructed from the apparent triangle rather than a retarded HELCTR call (§E6); gravitational deflection omitted with the corpus generated to match (§E8).
- **Errata that rename the spec:** `apparent_diameter_arcsec` → `apparent_diameter_deg` (§E1); API takes `Instant` not `jd_tdb` (§E7). Both are binding.
