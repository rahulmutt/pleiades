# SP-3 Fictitious (Hypothetical) Bodies Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the full default `seorbel.txt` fictitious-body set (Uranian/Hamburg planets, Transpluto, Vulcan, White Moon, Waldemath, and the historical pre-discovery predictions — SE numbers 40–58) as first-class `CelestialBody` variants served by a new `pleiades-fict::FictitiousBackend`, at Swiss-Ephemeris parity, gated by a two-tier `validate-fictitious` gate.

**Architecture:** A new pure-Rust crate `pleiades-fict` holds a Kepler solver, a committed orbital-element table (native CSV transcribed from SE's `seorbel.txt`), and a `FictitiousBackend<S: EphemerisBackend>` that implements the existing backend trait. It computes each body's unperturbed Kepler orbit, rotates it to the J2000 mean ecliptic, assembles the geocentric place by reusing a Sun-source backend (heliocentric bodies) or directly (geocentric-orbit bodies), and emits mean/geometric/geocentric J2000 — so the existing chart pipeline (apparent, topocentric, sidereal, houses) applies unchanged. A committed SE-parity corpus and a fail-closed `validate-fictitious` gate enforce definitional correctness.

**Tech Stack:** Rust (workspace crates), `std` only for `pleiades-fict` (no new dependencies), `libswisseph-sys` for the out-of-workspace reference generator tool, fnv1a64 checksum-guarded committed corpora.

## Global Constraints

- **No new runtime dependency** in shipped crates: `pleiades-fict` depends only on `pleiades-types`, `pleiades-backend`, `pleiades-apparent` (all workspace crates) + `std`. Copied verbatim from spec §Design.1.
- **Backend boundary is mean, geometric, geocentric, J2000 mean ecliptic** for every first-party backend — `pleiades-fict` must match this and pass `validate-frame-consistency`. Spec §Design.3.
- **Claim tier:** fictitious bodies are `BodyClaimTier::ReleaseGrade` with `AccuracyClass::Exact` and a *definitional* evidence string; do NOT invent a new tier. Spec §Design.4.
- **Fail closed, no placeholders:** unknown body → structured error; corpus drift → checksum failure; missing elements → error, never a silent default. Spec §Design.6.
- **Compatibility profile bump 0.7.8 → 0.7.9; API-stability profile unchanged at 0.2.2** (additive to `#[non_exhaustive]` enums). Spec §Design.7.
- **fnv1a64** is the repo checksum scheme (`pleiades_apparent::fnv1a64` inside the workspace; a byte-identical local copy inside the out-of-workspace tool). Spec §Design.6.
- **Window:** SE-parity corpus sampled over 1900–2100 TDB, consistent with the other numeric gates.
- **Bodies:** SE numbers 40–58 (all 19; `seorbel.txt` element sets 1–19); White Moon/Selena (56) and Waldemath (58) are **geocentric-orbit**; all others are **heliocentric**. Spec §"SE functions targeted".
- **Model fidelity (binding):** the Kepler/frame model must match SE `swi_osc_el_plan` per "Reconciliation with Swiss Ephemeris `swemplan.c`" above — Kepler-third-law mean motion (except T-term bodies), `Equinox { Fixed(jd), OfDate }`, and geometric corpus flags. Any regression to the spec's original "mean motion from polynomial" / 3-value `ElementFrame` model is an Important defect.

## Design notes / errata vs. spec

- **Motion (decision A refinement):** the spec said "analytic Kepler velocity." For consistency with `ElpBackend::motion` and the planetary backends (which all produce `Motion = Derived` via a symmetric finite difference of the same position model), this plan computes motion as a **symmetric finite difference of the assembled geocentric ecliptic position** (±0.5 day), not analytic velocity. Same public outcome (`Motion = Derived`); no separate velocity-frame bookkeeping. The Kepler core still exposes position only.
- **Elements provenance:** the committed `fictitious-elements.csv` values are transcribed from SE's `seorbel.txt` (element sets 1–19 = SE bodies 40–58). A verbatim copy of `seorbel.txt` is committed alongside the reference generator (`tools/se-fictitious-reference/data/seorbel.txt`) as the transcription source and so the SE tool computes all 19 bodies. The `validate-fictitious` SE-parity gate is the authoritative acceptance test for the transcription — do not hand-verify numbers, let the gate catch drift.

### Reconciliation with Swiss Ephemeris `swemplan.c` (corrections applied 2026-07-06)

A pre-execution review against the vendored SE C source (`libswisseph-sys-0.1.2/libswisseph/swemplan.c`, `swi_osc_el_plan` + `read_elements_file`) found the original spec-derived model did not match SE's actual algorithm. These corrections are **binding** and are folded into Tasks 3, 4, 7, 8 below. They are the crux of SE parity — a reviewer must treat any regression to the original model as an Important defect.

1. **Mean motion comes from Kepler's third law, NOT from a mean-anomaly polynomial.** SE reads mean anomaly *at epoch* and advances it with `dmot = 0.9856076686 / a^1.5` deg/day (`dmot /= sqrt(SUN_EARTH_MRAT)` for geocentric orbits), i.e. `M(t) = M_epoch + dmot·(jd − epoch)`. The **only** bodies whose motion lives in the element polynomial are those with an explicit `+ c1·T` term in `seorbel.txt` mean anomaly (Vulcan, Selena/White Moon, Waldemath); for those SE freezes `dmot` (sets `tjd0 = tjd`) and the T-term *is* the motion. So `state_at` must: evaluate the mean-anomaly polynomial in `T = (jd − epoch)/36525`, and **add the Kepler `dmot` term only when the polynomial has no T-term** (`c1 == 0 && c2 == 0`). Without this, non-T-term bodies are stationary and the gate fails catastrophically. Constants: `0.9856076686` deg/day, `SUN_EARTH_MRAT = 332946.050895`. (The geocentric `/sqrt(SUN_EARTH_MRAT)` scaling is SE-faithful but unexercised by the default set — both geocentric bodies, Selena and Waldemath, carry mean-anomaly T-terms and so bypass `dmot` entirely; keep it as defensive parity, not dead code.)

2. **The reference equinox is an arbitrary Julian Day (or "of date"), not a 3-value frame enum.** `seorbel.txt` equinoxes are `J1900` (2415020.0), `B1950` (2433282.42345905), `J2000` (2451545.0), `JDATE` (= evaluation instant, "of date"), **or a bare JD** (Transpluto 2431456.5, Nibiru 1856113.380954, Leverrier/Adams 2395662.5, Lowell/Pickering 2425977.5, Waldemath 2414290.95827875). The `ElementFrame {J2000,B1950,OfDate}` enum in the original plan cannot represent these. Replace it with `Equinox { Fixed(f64), OfDate }` (`Fixed(2451545.0)` = identity; `OfDate` precesses from the evaluation JD). Rotation to J2000 reuses `pleiades_apparent::precess_ecliptic_date_to_j2000(lon, lat, equinox_jd)` (ecliptic-frame precession, the same helper `ElpBackend` uses). This helper is IAU-1976 (accurate for ~a few centuries); all default bodies except **Nibiru** have equinoxes within ~150 yr of J2000. Nibiru's equinox is ~370 AD (~1630 yr of extrapolation) → it will carry a larger residual than the rest; the gate measures it and Task 8 sets its ceiling from the measurement (do not hand-tighten Nibiru to the others' level).

3. **Body coverage is the full 19 (SE 40–58), enabled by the committed `seorbel.txt`.** SE's *built-in* fallback table covers only 40–54 (`SE_NFICT_ELEM = 15`); bodies 55–58 (Vulcan, White Moon/Selena, Proserpina, Waldemath — including **both** geocentric bodies) exist only in `seorbel.txt`. The generator (Task 7) points `swe_set_ephe_path` at the committed copy so all 19 resolve.

4. **Corpus flags must be geometric, not apparent.** The backend boundary is *geometric* geocentric J2000; bare `SEFLG_J2000` yields *apparent* positions (light-time + aberration). Task 7 uses `SEFLG_MOSEPH | SEFLG_J2000 | SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL` so the SE reference matches the backend's geometric assembly. (Values: TRUEPOS=16, J2000=32, NOGDEFL=512, NOABERR=1024, MOSEPH=4.)

5. **Geocentric assembly (unchanged, verified correct):** for heliocentric bodies, `body_geo = body_helio − earth_helio` with `earth_helio = −sun_geocentric` (packaged Sun source), which equals SE's `body_helio + sun_geocentric`. Geocentric-orbit bodies (Selena, Waldemath) return their orbital position directly. Note the residual absorbs `(packaged Sun − SE Moshier Sun)`, a small term the gate measures.

---

## File structure

**New crate `crates/pleiades-fict/`:**
- `Cargo.toml` — package + workspace deps (types, backend, apparent).
- `src/lib.rs` — crate root; module decls + public re-exports; `PACKAGE_NAME`.
- `src/kepler.rs` — `solve_kepler`, `orbital_plane_position`.
- `src/elements.rs` — `KeplerElements`, `Equinox`, `Center`, the polynomial-in-time element model with Kepler-third-law mean motion, CSV parse (`LazyLock` table), `elements_for(body)`.
- `src/frame.rs` — rotate an equinox-frame ecliptic vector → J2000 mean ecliptic (`rotate_ecliptic_to_j2000`: J2000 identity, else ecliptic precession from the equinox JD).
- `src/backend.rs` — `FictitiousBackend<S>`, `EphemerisBackend` impl, `fictitious_body_claims()`.
- `data/fictitious-elements.csv` — committed element table (19 rows), provenance header citing `seorbel.txt`.
- `README.md`, `LICENSE-APACHE`, `LICENSE-MIT` (copy from a sibling crate).

**Modified:**
- `crates/pleiades-types/src/bodies.rs` — new `CelestialBodyClass::Fictitious`; 19 new `CelestialBody` variants; update `class()`, `built_in_name()`, `Display`.
- `Cargo.toml` (root) — add `crates/pleiades-fict` to `members`; add `pleiades-fict` to `[workspace.dependencies]`; add `tools/se-fictitious-reference` to `exclude`.
- `crates/pleiades-cli/src/commands/chart.rs:559-566` — add `FictitiousBackend` to the routing chain.
- `crates/pleiades-validate/src/lib.rs` — module decls + re-exports for the gate.
- `crates/pleiades-validate/src/render/cli.rs` — `run_all_numeric_gates`, dispatch arms, help banner, battery test.
- `crates/pleiades-validate/src/tests/validate_gates.rs` — per-gate test block.
- `crates/pleiades-core/src/compatibility/mod.rs` — profile id 0.7.9, content checksum, summary prose, SP array.
- `README.md`, `PLAN.md`, `plan/status/01-*.md`, `plan/status/02-*.md`, `plan/stages/*` — mark SP-3 done.

**New tool:**
- `tools/se-fictitious-reference/` — own workspace, `libswisseph-sys`, emits parity corpus + manifest (+ optionally the elements CSV).
- `crates/pleiades-validate/src/fictitious_validation.rs`, `src/fictitious_thresholds.rs` — the gate.
- `crates/pleiades-validate/data/fictitious-corpus/` — committed corpus CSV(s) + `manifest.txt` + `MANIFEST.md`.

---

## Task 1: Fictitious body variants + `Fictitious` class

**Files:**
- Modify: `crates/pleiades-types/src/bodies.rs` (enum `CelestialBodyClass` ~line 11; enum `CelestialBody` ~line 47; `class()` ~line 97; `built_in_name()` ~line 120; `Display` ~line 160)
- Test: `crates/pleiades-types/src/bodies.rs` (inline `#[cfg(test)]` or existing test module)

**Interfaces:**
- Produces: `CelestialBodyClass::Fictitious`; 19 unit variants `CelestialBody::{Cupido, Hades, Zeus, Kronos, Apollon, Admetos, Vulkanus, Poseidon, Transpluto, Nibiru, Harrington, NeptuneLeverrier, NeptuneAdams, PlutoLowell, PlutoPickering, Vulcan, WhiteMoon, Proserpina, Waldemath}`; each maps to `CelestialBodyClass::Fictitious` via `class()` and has a `built_in_name()` + `Display` string.

- [ ] **Step 1: Write the failing test**

Add to the test module in `crates/pleiades-types/src/bodies.rs`:

```rust
#[test]
fn fictitious_bodies_map_to_fictitious_class() {
    for body in [
        CelestialBody::Cupido,
        CelestialBody::Transpluto,
        CelestialBody::Vulcan,
        CelestialBody::WhiteMoon,
        CelestialBody::Waldemath,
    ] {
        assert_eq!(body.class(), CelestialBodyClass::Fictitious);
        assert!(body.built_in_name().is_some());
        assert!(!body.to_string().is_empty());
    }
    assert_eq!(CelestialBody::Transpluto.built_in_name(), Some("Transpluto"));
    assert_eq!(CelestialBody::WhiteMoon.to_string(), "White Moon");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-types fictitious_bodies_map_to_fictitious_class`
Expected: FAIL — `no variant named Cupido` (compile error).

- [ ] **Step 3: Add the class variant**

In `CelestialBodyClass` (the `#[non_exhaustive]` enum ~line 11), add after `Custom`:

```rust
    /// A hypothetical/fictitious body defined by osculating orbital elements
    /// (Uranian planets, Transpluto, Vulcan, historical pre-discovery predictions).
    Fictitious,
```

And in its `label()` match, add:

```rust
            Self::Fictitious => "fictitious body",
```

- [ ] **Step 4: Add the body variants**

In `CelestialBody` (the `#[non_exhaustive]` enum ~line 47), add before `Custom(CustomBodyId)`:

```rust
    /// Cupido (Uranian/Hamburg, Witte) — SE fictitious body 40.
    Cupido,
    /// Hades (Uranian/Hamburg, Witte) — SE 41.
    Hades,
    /// Zeus (Uranian/Hamburg, Sieggrün) — SE 42.
    Zeus,
    /// Kronos (Uranian/Hamburg, Sieggrün) — SE 43.
    Kronos,
    /// Apollon (Uranian/Hamburg, Sieggrün) — SE 44.
    Apollon,
    /// Admetos (Uranian/Hamburg, Sieggrün) — SE 45.
    Admetos,
    /// Vulkanus (Uranian/Hamburg, Sieggrün) — SE 46.
    Vulkanus,
    /// Poseidon (Uranian/Hamburg, Sieggrün) — SE 47.
    Poseidon,
    /// Isis / Transpluto — SE 48.
    Transpluto,
    /// Nibiru — SE 49.
    Nibiru,
    /// Harrington — SE 50.
    Harrington,
    /// Neptune (Leverrier historical prediction) — SE 51.
    NeptuneLeverrier,
    /// Neptune (Adams historical prediction) — SE 52.
    NeptuneAdams,
    /// Pluto (Lowell historical prediction) — SE 53.
    PlutoLowell,
    /// Pluto (Pickering historical prediction) — SE 54.
    PlutoPickering,
    /// Vulcan (intramercurial) — SE 55.
    Vulcan,
    /// White Moon / Selena (geocentric orbit) — SE 56.
    WhiteMoon,
    /// Proserpina — SE 57.
    Proserpina,
    /// Waldemath (hypothetical second Earth moon, geocentric orbit) — SE 58.
    Waldemath,
```

- [ ] **Step 5: Update `class()`, `built_in_name()`, and `Display`**

The compiler will flag each non-exhaustive match. In `class()` add an arm listing all 19 variants → `CelestialBodyClass::Fictitious`:

```rust
            Self::Cupido
            | Self::Hades
            | Self::Zeus
            | Self::Kronos
            | Self::Apollon
            | Self::Admetos
            | Self::Vulkanus
            | Self::Poseidon
            | Self::Transpluto
            | Self::Nibiru
            | Self::Harrington
            | Self::NeptuneLeverrier
            | Self::NeptuneAdams
            | Self::PlutoLowell
            | Self::PlutoPickering
            | Self::Vulcan
            | Self::WhiteMoon
            | Self::Proserpina
            | Self::Waldemath => CelestialBodyClass::Fictitious,
```

In `built_in_name()` and `Display`, add one arm per variant with the display name (identical strings in both). Use: Cupido→"Cupido", Hades→"Hades", Zeus→"Zeus", Kronos→"Kronos", Apollon→"Apollon", Admetos→"Admetos", Vulkanus→"Vulkanus", Poseidon→"Poseidon", Transpluto→"Transpluto", Nibiru→"Nibiru", Harrington→"Harrington", NeptuneLeverrier→"Neptune (Leverrier)", NeptuneAdams→"Neptune (Adams)", PlutoLowell→"Pluto (Lowell)", PlutoPickering→"Pluto (Pickering)", Vulcan→"Vulcan", WhiteMoon→"White Moon", Proserpina→"Proserpina", Waldemath→"Waldemath". For `built_in_name()` return `Some("…")`; for `Display` `f.write_str("…")`.

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test -p pleiades-types fictitious_bodies_map_to_fictitious_class`
Expected: PASS.

- [ ] **Step 7: Verify the workspace still builds**

Run: `cargo build --workspace`
Expected: builds clean. (Other crates already use `_ =>` arms on the non-exhaustive enum, so no other match sites break.)

- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-types/src/bodies.rs
git commit -m "feat(types): SP-3 fictitious body variants + Fictitious body class"
```

---

## Task 2: `pleiades-fict` crate scaffold + Kepler solver core

**Files:**
- Create: `crates/pleiades-fict/Cargo.toml`, `crates/pleiades-fict/src/lib.rs`, `crates/pleiades-fict/src/kepler.rs`, `crates/pleiades-fict/README.md`
- Modify: `Cargo.toml` (root) `members` + `[workspace.dependencies]`
- Copy: `crates/pleiades-fict/LICENSE-APACHE`, `LICENSE-MIT` (from `crates/pleiades-elp/`)
- Test: inline `#[cfg(test)]` in `src/kepler.rs`

**Interfaces:**
- Produces: `pleiades_fict::kepler::solve_kepler(mean_anomaly_rad: f64, eccentricity: f64) -> f64` (returns eccentric anomaly, radians); `pleiades_fict::kepler::orbital_plane_position(a_au: f64, e: f64, eccentric_anomaly_rad: f64) -> (f64, f64)` (returns (x, y) in the orbital plane, AU, focus at origin, x toward perihelion).

- [ ] **Step 1: Create the crate manifest**

`crates/pleiades-fict/Cargo.toml`:

```toml
[package]
name = "pleiades-fict"
description = "Fictitious/hypothetical body backend (Uranian planets, Transpluto, Vulcan, …) from osculating orbital elements for the pleiades astrology workspace."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[dependencies]
pleiades-apparent = { workspace = true }
pleiades-backend = { workspace = true }
pleiades-types = { workspace = true }
```

- [ ] **Step 2: Register the crate in the workspace**

In root `Cargo.toml`, add `"crates/pleiades-fict"` to `members` (keep alphabetical), and under `[workspace.dependencies]` add:

```toml
pleiades-fict = { path = "crates/pleiades-fict", version = "..." }
```

Match the exact `version = "..."` string the sibling `pleiades-elp` entry uses in `[workspace.dependencies]`.

- [ ] **Step 3: Copy license files + write README stub**

```bash
cp crates/pleiades-elp/LICENSE-APACHE crates/pleiades-elp/LICENSE-MIT crates/pleiades-fict/
```

`crates/pleiades-fict/README.md`:

```markdown
# pleiades-fict

Fictitious/hypothetical body backend for the pleiades astrology workspace.

Computes the Swiss-Ephemeris default `seorbel.txt` fictitious bodies (SE numbers
40–58: the Uranian/Hamburg planets, Transpluto, Vulcan, White Moon, Waldemath,
and the historical pre-discovery Neptune/Pluto predictions) from committed
osculating orbital elements via an unperturbed Kepler orbit. These bodies are
*definitional*: correctness means parity with SE's `seorbel.txt`-driven output,
enforced by the `validate-fictitious` gate.
```

- [ ] **Step 4: Write the crate root**

`crates/pleiades-fict/src/lib.rs`:

```rust
//! Fictitious/hypothetical bodies from osculating orbital elements.
//!
//! Computes the Swiss-Ephemeris default `seorbel.txt` fictitious body set as an
//! unperturbed Kepler orbit, rotated to the J2000 mean ecliptic and assembled to
//! a geocentric place. Definitional: parity with SE, gated by `validate-fictitious`.

pub mod kepler;

/// Crate/backend identifier used in backend metadata and results.
pub const PACKAGE_NAME: &str = "pleiades-fict";

/// Julian Day (TT) of the J2000.0 epoch.
pub const J2000_JD: f64 = 2_451_545.0;
```

- [ ] **Step 5: Write the failing Kepler test**

`crates/pleiades-fict/src/kepler.rs`:

```rust
//! Kepler's-equation solver and orbital-plane position.

/// Solve Kepler's equation `M = E − e·sin E` for the eccentric anomaly `E`
/// (radians), by Newton–Raphson. Converges to machine precision in a few
/// iterations for the bounded eccentricities (`e < 1`) of `seorbel.txt`.
pub fn solve_kepler(mean_anomaly_rad: f64, eccentricity: f64) -> f64 {
    let m = mean_anomaly_rad.rem_euclid(std::f64::consts::TAU);
    let mut e = if eccentricity < 0.8 { m } else { std::f64::consts::PI };
    for _ in 0..64 {
        let delta = (e - eccentricity * e.sin() - m) / (1.0 - eccentricity * e.cos());
        e -= delta;
        if delta.abs() < 1.0e-14 {
            break;
        }
    }
    e
}

/// Orbital-plane Cartesian position (AU), focus at the origin, x-axis toward
/// perihelion, from the semi-major axis, eccentricity, and eccentric anomaly.
pub fn orbital_plane_position(a_au: f64, e: f64, eccentric_anomaly_rad: f64) -> (f64, f64) {
    let x = a_au * (eccentric_anomaly_rad.cos() - e);
    let y = a_au * (1.0 - e * e).sqrt() * eccentric_anomaly_rad.sin();
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_kepler_inverts_the_equation() {
        for &e in &[0.0, 0.05, 0.2, 0.5, 0.9] {
            for step in 0..12 {
                let m = step as f64 * 0.5;
                let ea = solve_kepler(m, e);
                let recovered = ea - e * ea.sin();
                let diff = (recovered - m.rem_euclid(std::f64::consts::TAU)).abs();
                assert!(diff < 1.0e-12, "e={e} m={m} diff={diff}");
            }
        }
    }

    #[test]
    fn circular_orbit_has_radius_a() {
        let (x, y) = orbital_plane_position(2.5, 0.0, 1.0);
        assert!((x.hypot(y) - 2.5).abs() < 1.0e-12);
    }
}
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p pleiades-fict`
Expected: PASS (2 tests).

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-fict Cargo.toml
git commit -m "feat(fict): SP-3 pleiades-fict crate scaffold + Kepler solver core"
```

---

## Task 3: Orbital-element model + frame rotation to J2000 mean ecliptic

**Files:**
- Create: `crates/pleiades-fict/src/elements.rs`, `crates/pleiades-fict/src/frame.rs`
- Modify: `crates/pleiades-fict/src/lib.rs` (add `pub mod elements; pub mod frame;`)
- Test: inline `#[cfg(test)]` in both modules

**Interfaces:**
- Consumes: `kepler::solve_kepler`, `kepler::orbital_plane_position`; `pleiades_apparent::precess_ecliptic_date_to_j2000`.
- Produces:
  - `elements::Equinox` (enum `Fixed(f64)`, `OfDate`); `elements::Center` (enum `Heliocentric`, `Geocentric`).
  - `elements::KeplerElements` with fields: epoch `epoch_jd_tt: f64`; per-element polynomial coefficients `a_au: [f64;3]`, `e: [f64;3]`, `incl_deg: [f64;3]`, `node_deg: [f64;3]`, `arg_peri_deg: [f64;3]`, `mean_anom_deg: [f64;3]` (each `[c0, c1, c2]`, `T` in Julian centuries since `epoch_jd_tt`); plus `equinox: Equinox`, `center: Center`.
  - `elements::KeplerElements::state_at(&self, jd_tt: f64) -> (f64, f64, f64)` — J2000-mean-ecliptic Cartesian position `(x, y, z)` in AU, in the elements' native centering (helio or geo). Adds the Kepler-third-law mean-motion term unless the mean-anomaly polynomial carries a T-term (see Reconciliation §1).
  - `frame::rotate_ecliptic_to_j2000(x: f64, y: f64, z: f64, equinox_jd: f64) -> (f64, f64, f64)`.

- [ ] **Step 1: Write the element model + failing test**

`crates/pleiades-fict/src/elements.rs`:

```rust
//! Osculating orbital-element model (polynomial-in-time), transcribed from SE's
//! `seorbel.txt`. Each element is `c0 + c1·T + c2·T²`, `T` in Julian centuries
//! (36525 d) since the element epoch.

use crate::frame::rotate_ecliptic_to_j2000;
use crate::kepler::{orbital_plane_position, solve_kepler};

/// Gaussian mean-motion constant (SE `swi_osc_el_plan`): daily motion in
/// degrees is `MEAN_MOTION_DEG_PER_DAY / a^1.5`.
const MEAN_MOTION_DEG_PER_DAY: f64 = 0.9856076686;
/// Sun / Earth mass ratio (Earth only), SE `SUN_EARTH_MRAT`. Scales the mean
/// motion of geocentric-orbit bodies (central mass is Earth, not the Sun).
const SUN_EARTH_MRAT: f64 = 332946.050895;

/// Reference equinox the angular elements are expressed in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Equinox {
    /// Fixed equinox at a Julian Day (TT): `J1900` (2415020.0), `B1950`, `J2000`
    /// (2451545.0), or an arbitrary JD from `seorbel.txt`. `Fixed(2451545.0)` is
    /// identity (no precession).
    Fixed(f64),
    /// Equinox "of date" (`seorbel.txt` `JDATE`): the equinox is the evaluation
    /// instant; precess from the evaluation JD to J2000.
    OfDate,
}

/// Whether the osculating orbit is centered on the Sun or the Earth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Center {
    /// Heliocentric orbit — caller adds the Earth→Sun vector to geocentricize.
    Heliocentric,
    /// Geocentric orbit (White Moon, Waldemath) — already Earth-centered.
    Geocentric,
}

/// Classical osculating elements with linear/quadratic time terms.
#[derive(Clone, Copy, Debug)]
pub struct KeplerElements {
    /// Element epoch, Julian Day (TT).
    pub epoch_jd_tt: f64,
    /// Semi-major axis polynomial [c0, c1, c2], AU.
    pub a_au: [f64; 3],
    /// Eccentricity polynomial (dimensionless).
    pub e: [f64; 3],
    /// Inclination polynomial, degrees.
    pub incl_deg: [f64; 3],
    /// Longitude of ascending node polynomial, degrees.
    pub node_deg: [f64; 3],
    /// Argument of perihelion polynomial, degrees.
    pub arg_peri_deg: [f64; 3],
    /// Mean anomaly polynomial, degrees. A nonzero c1/c2 (T-term) supplies the
    /// body's mean motion directly; otherwise Kepler's third law is used.
    pub mean_anom_deg: [f64; 3],
    /// Reference equinox of the angular elements.
    pub equinox: Equinox,
    /// Orbit centering.
    pub center: Center,
}

fn poly(c: [f64; 3], t: f64) -> f64 {
    c[0] + c[1] * t + c[2] * t * t
}

impl KeplerElements {
    /// J2000-mean-ecliptic Cartesian position (AU) at `jd_tt`, in the elements'
    /// native centering (helio or geo). Mirrors SE `swi_osc_el_plan`: the
    /// mean-anomaly polynomial supplies any explicit T-term (Vulcan, Selena,
    /// Waldemath); bodies with no T-term advance by the Kepler-third-law mean
    /// motion. See Reconciliation §1.
    pub fn state_at(&self, jd_tt: f64) -> (f64, f64, f64) {
        let t = (jd_tt - self.epoch_jd_tt) / 36_525.0;
        let a = poly(self.a_au, t);
        let e = poly(self.e, t);
        let incl = poly(self.incl_deg, t).to_radians();
        let node = poly(self.node_deg, t).to_radians();
        let argp = poly(self.arg_peri_deg, t).to_radians();

        let mut mean_anom_deg = poly(self.mean_anom_deg, t);
        if self.mean_anom_deg[1] == 0.0 && self.mean_anom_deg[2] == 0.0 {
            // No T-term: advance mean anomaly by the Kepler-third-law daily motion.
            let mut dmot = MEAN_MOTION_DEG_PER_DAY / (a * a.sqrt());
            if self.center == Center::Geocentric {
                dmot /= SUN_EARTH_MRAT.sqrt();
            }
            mean_anom_deg += dmot * (jd_tt - self.epoch_jd_tt);
        }
        let mean_anom = mean_anom_deg.to_radians();

        let ea = solve_kepler(mean_anom, e);
        let (xo, yo) = orbital_plane_position(a, e, ea);

        // Rotate orbital plane → equinox-frame ecliptic by argp, incl, node
        // (classic 3-1-3). The matrix is linear, applied to the position vector.
        let (sa, ca) = argp.sin_cos();
        let (si, ci) = incl.sin_cos();
        let (sn, cn) = node.sin_cos();
        let xp = ca * xo - sa * yo;
        let yp = sa * xo + ca * yo;
        let x = cn * xp - sn * ci * yp;
        let y = sn * xp + cn * ci * yp;
        let z = si * yp;

        let equinox_jd = match self.equinox {
            Equinox::Fixed(jd) => jd,
            Equinox::OfDate => jd_tt,
        };
        rotate_ecliptic_to_j2000(x, y, z, equinox_jd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn circular_j2000(a: f64) -> KeplerElements {
        KeplerElements {
            epoch_jd_tt: crate::J2000_JD,
            a_au: [a, 0.0, 0.0],
            e: [0.0, 0.0, 0.0],
            incl_deg: [0.0, 0.0, 0.0],
            node_deg: [0.0, 0.0, 0.0],
            arg_peri_deg: [0.0, 0.0, 0.0],
            mean_anom_deg: [0.0, 0.0, 0.0],
            equinox: Equinox::Fixed(crate::J2000_JD),
            center: Center::Heliocentric,
        }
    }

    #[test]
    fn zero_inclination_circular_orbit_lies_in_ecliptic_at_radius_a() {
        // At the epoch (t=0) the Kepler dmot term is zero, so the body sits at
        // mean anomaly 0 on a circle of radius a in the J2000 ecliptic plane.
        let (x, y, z) = circular_j2000(3.0).state_at(crate::J2000_JD);
        assert!(z.abs() < 1.0e-12, "z={z}");
        assert!((x.hypot(y) - 3.0).abs() < 1.0e-12);
    }

    #[test]
    fn kepler_mean_motion_advances_a_non_t_term_body() {
        // Guard against the stationary-body bug: a body with no mean-anomaly
        // T-term must advance via Kepler's third law (dmot ∝ a^-1.5).
        let el = circular_j2000(3.0);
        let (x0, y0, _) = el.state_at(crate::J2000_JD);
        let (x1, y1, _) = el.state_at(crate::J2000_JD + 365.25);
        let sep = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        assert!(sep > 1.0e-3, "non-T-term body should advance; sep={sep}");
    }
}
```

- [ ] **Step 2: Write the frame module**

`crates/pleiades-fict/src/frame.rs`:

```rust
//! Rotate an element-equinox ecliptic Cartesian vector to the J2000 mean ecliptic.

/// Rotate `(x, y, z)` (AU) from the mean ecliptic of the element equinox
/// `equinox_jd` to the J2000 mean ecliptic. J2000 is identity; any other equinox
/// is precessed via `pleiades_apparent::precess_ecliptic_date_to_j2000`
/// (ecliptic-frame IAU-1976 precession; distance is preserved). For
/// `Equinox::OfDate` the caller passes the evaluation JD as `equinox_jd`.
pub fn rotate_ecliptic_to_j2000(x: f64, y: f64, z: f64, equinox_jd: f64) -> (f64, f64, f64) {
    if (equinox_jd - crate::J2000_JD).abs() < 1.0e-6 {
        return (x, y, z);
    }
    let r = (x * x + y * y + z * z).sqrt();
    if r == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let lon = y.atan2(x).to_degrees().rem_euclid(360.0);
    let lat = (z / r).clamp(-1.0, 1.0).asin().to_degrees();
    let p = pleiades_apparent::precess_ecliptic_date_to_j2000(lon, lat, equinox_jd)
        .expect("fictitious body lon/lat precess cleanly to J2000");
    let lon_r = p.longitude_deg.to_radians();
    let lat_r = p.latitude_deg.to_radians();
    (
        r * lat_r.cos() * lon_r.cos(),
        r * lat_r.cos() * lon_r.sin(),
        r * lat_r.sin(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2000_equinox_is_identity() {
        let (x, y, z) = rotate_ecliptic_to_j2000(1.0, 2.0, 0.5, crate::J2000_JD);
        assert!((x - 1.0).abs() < 1.0e-12);
        assert!((y - 2.0).abs() < 1.0e-12);
        assert!((z - 0.5).abs() < 1.0e-12);
    }

    #[test]
    fn precession_preserves_distance() {
        // Ecliptic precession is a pure rotation: |v| is invariant. J1900 = 2415020.0.
        let (x, y, z) = rotate_ecliptic_to_j2000(3.0, 0.0, 0.0, 2_415_020.0);
        let r = (x * x + y * y + z * z).sqrt();
        assert!((r - 3.0).abs() < 1.0e-9, "distance changed: r={r}");
    }
}
```

Note: `precess_ecliptic_date_to_j2000` is already used by `ElpBackend` (`crates/pleiades-elp/src/backend.rs`) and returns a struct with `longitude_deg` / `latitude_deg`. It is IAU-1976 (accurate to a few centuries around J2000); every default body's equinox is within ~150 yr of J2000 **except Nibiru** (~370 AD, ~1630 yr of extrapolation), whose residual will be larger — expected, and its Task 8 ceiling is set from the measurement, not hand-tightened to the others' level. None of the 19 bodies uses a `B1950` equinox (B1950 appears only in out-of-scope comet sets), so no fixed B1950 matrix is needed; `Equinox::Fixed(2433282.42345905)` would precess generically if one did. The `.expect()` never fires for finite inputs (the helper returns `Err` only on a non-finite result), matching `ElpBackend`'s precedent.

- [ ] **Step 3: Wire the modules into lib.rs**

In `crates/pleiades-fict/src/lib.rs`, add under `pub mod kepler;`:

```rust
pub mod elements;
pub mod frame;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-fict`
Expected: PASS (Kepler tests + `zero_inclination_circular_orbit_lies_in_ecliptic_at_radius_a`, `kepler_mean_motion_advances_a_non_t_term_body`, `j2000_equinox_is_identity`, `precession_preserves_distance`).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-fict/src
git commit -m "feat(fict): SP-3 orbital-element model + J2000 frame rotation"
```

---

## Task 4: Committed element table + CSV parser

**Files:**
- Create: `crates/pleiades-fict/data/fictitious-elements.csv`
- Modify: `crates/pleiades-fict/src/elements.rs` (add CSV parse + `elements_for`)
- Test: inline in `src/elements.rs`

**Interfaces:**
- Produces: `elements::elements_for(body: pleiades_types::CelestialBody) -> Option<&'static KeplerElements>` (returns `None` for non-fictitious bodies); `elements::TABLE: LazyLock<Vec<(CelestialBody, KeplerElements)>>`.

- [ ] **Step 1: Create the committed CSV**

`crates/pleiades-fict/data/fictitious-elements.csv`. **Transcribe every value from the committed `tools/se-fictitious-reference/data/seorbel.txt`** (element sets 1–19 = SE bodies 40–58; committed in Task 7 Step 0). Do NOT invent coefficients — the `validate-fictitious` gate (Task 9) is the acceptance test. A `seorbel.txt` element field may carry a linear time term written `v0 + v1 * T` (T in Julian centuries since epoch); **explode it into `c0,c1,c2`** (`c0=v0, c1=v1, c2=0`; almost every field is constant → `c1=c2=0`). The `epoch` and `equinox` columns accept the tokens `J1900` (2415020.0), `B1950` (2433282.42345905), `J2000` (2451545.0), or a bare JD; `equinox` additionally accepts `JDATE` (= of-date). Column order (positional, comma-separated), header + `#` comments skipped:

```
# fictitious-elements.csv — osculating elements transcribed from Swiss Ephemeris seorbel.txt (sets 1–19 = SE 40–58).
# Angles in degrees; a in AU; each element is a polynomial c0,c1,c2 with T in Julian
# centuries (36525 d) since epoch. epoch ∈ {J1900,B1950,J2000,<jd>}; equinox ∈
# {J1900,B1950,J2000,JDATE,<jd>}; center ∈ {helio,geo}.
# body,epoch,a0,a1,a2,e0,e1,e2,i0,i1,i2,node0,node1,node2,argp0,argp1,argp2,M0,M1,M2,equinox,center
# Cupido (seorbel set 1): "J1900, J1900, 163.7409, 40.99837, 0.00460, 171.4333, 129.8325, 1.0833, Cupido"
Cupido,J1900,40.99837,0,0,0.00460,0,0,1.0833,0,0,129.8325,0,0,171.4333,0,0,163.7409,0,0,J1900,helio
# Nibiru (set 10): arbitrary-JD epoch & equinox, high eccentricity
Nibiru,1856113.380954,234.8921,0,0,0.981092,0,0,158.708,0,0,-44.567,0,0,103.966,0,0,0.0,0,0,1856113.380954,helio
# Vulcan (set 16): equinox JDATE; mean-anomaly, argp, node carry T-terms
Vulcan,J1900,0.13744,0,0,0.019,0,0,7.5,0,0,47.787931,-1670.056,0,322.212069,1670.056,0,252.8987988,707550.7341,0,JDATE,helio
...one row per body through Waldemath (set 19)...
```

**seorbel column order is (epoch, equinox, mean-anomaly, semi-axis, eccentricity, arg-perihelion, ascending-node, inclination, name[, "geo"])** — the CSV reorders these into the `a,e,i,node,argp,M` layout above, so transcribe **by field name, not position** (note mean-anomaly is field 3 in seorbel but the `M` columns are last-but-two in the CSV). Use these body tokens (order matches seorbel sets 1–19): `Cupido, Hades, Zeus, Kronos, Apollon, Admetos, Vulkanus, Poseidon, Transpluto, Nibiru, Harrington, NeptuneLeverrier, NeptuneAdams, PlutoLowell, PlutoPickering, Vulcan, WhiteMoon, Proserpina, Waldemath`. Set `center=geo` for `WhiteMoon` (Selena, set 17) and `Waldemath` (set 19) — both tagged `geo` in seorbel — and `helio` for the rest.

- [ ] **Step 2: Write the failing parser test**

Add to `crates/pleiades-fict/src/elements.rs`:

```rust
use pleiades_types::CelestialBody;
use std::sync::LazyLock;

const RAW: &str = include_str!("../data/fictitious-elements.csv");

/// All 19 fictitious bodies with their osculating elements, parsed once.
pub static TABLE: LazyLock<Vec<(CelestialBody, KeplerElements)>> = LazyLock::new(parse_table);

fn body_from_token(token: &str) -> CelestialBody {
    match token {
        "Cupido" => CelestialBody::Cupido,
        "Hades" => CelestialBody::Hades,
        "Zeus" => CelestialBody::Zeus,
        "Kronos" => CelestialBody::Kronos,
        "Apollon" => CelestialBody::Apollon,
        "Admetos" => CelestialBody::Admetos,
        "Vulkanus" => CelestialBody::Vulkanus,
        "Poseidon" => CelestialBody::Poseidon,
        "Transpluto" => CelestialBody::Transpluto,
        "Nibiru" => CelestialBody::Nibiru,
        "Harrington" => CelestialBody::Harrington,
        "NeptuneLeverrier" => CelestialBody::NeptuneLeverrier,
        "NeptuneAdams" => CelestialBody::NeptuneAdams,
        "PlutoLowell" => CelestialBody::PlutoLowell,
        "PlutoPickering" => CelestialBody::PlutoPickering,
        "Vulcan" => CelestialBody::Vulcan,
        "WhiteMoon" => CelestialBody::WhiteMoon,
        "Proserpina" => CelestialBody::Proserpina,
        "Waldemath" => CelestialBody::Waldemath,
        other => panic!("unknown fictitious body token in elements CSV: {other}"),
    }
}

/// Resolve an epoch/equinox JD token: the `J1900`/`B1950`/`J2000` keywords or a
/// bare Julian Day. (`JDATE` is handled separately by `equinox_from_token`.)
fn jd_from_token(tok: &str) -> f64 {
    match tok.trim() {
        "J1900" => 2_415_020.0,
        "B1950" => 2_433_282.423_459_05,
        "J2000" => 2_451_545.0,
        other => other
            .parse::<f64>()
            .unwrap_or_else(|_| panic!("bad epoch/JD token in elements CSV: {other}")),
    }
}

fn equinox_from_token(tok: &str) -> Equinox {
    match tok.trim() {
        "JDATE" => Equinox::OfDate,
        other => Equinox::Fixed(jd_from_token(other)),
    }
}

fn parse_table() -> Vec<(CelestialBody, KeplerElements)> {
    RAW.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            let g = |i: usize| f[i].trim().parse::<f64>().unwrap();
            let body = body_from_token(f[0].trim());
            let elements = KeplerElements {
                epoch_jd_tt: jd_from_token(f[1]),
                a_au: [g(2), g(3), g(4)],
                e: [g(5), g(6), g(7)],
                incl_deg: [g(8), g(9), g(10)],
                node_deg: [g(11), g(12), g(13)],
                arg_peri_deg: [g(14), g(15), g(16)],
                mean_anom_deg: [g(17), g(18), g(19)],
                equinox: equinox_from_token(f[20]),
                center: match f[21].trim() {
                    "helio" => Center::Heliocentric,
                    "geo" => Center::Geocentric,
                    other => panic!("unknown center token: {other}"),
                },
            };
            (body, elements)
        })
        .collect()
}

/// Osculating elements for a fictitious body, or `None` if the body is not one.
pub fn elements_for(body: CelestialBody) -> Option<&'static KeplerElements> {
    TABLE.iter().find(|(b, _)| *b == body).map(|(_, el)| el)
}

#[cfg(test)]
mod table_tests {
    use super::*;

    #[test]
    fn table_has_all_nineteen_bodies() {
        assert_eq!(TABLE.len(), 19);
    }

    #[test]
    fn geocentric_bodies_are_earth_centered() {
        assert_eq!(elements_for(CelestialBody::WhiteMoon).unwrap().center, Center::Geocentric);
        assert_eq!(elements_for(CelestialBody::Waldemath).unwrap().center, Center::Geocentric);
        assert_eq!(elements_for(CelestialBody::Cupido).unwrap().center, Center::Heliocentric);
    }

    #[test]
    fn non_fictitious_body_has_no_elements() {
        assert!(elements_for(CelestialBody::Mars).is_none());
    }
}
```

`CelestialBody` must be `PartialEq` for `*b == body` — it already derives it (used throughout the workspace). If not in scope, derive is already present.

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test -p pleiades-fict table_tests`
Expected: PASS (3 tests). If `table_has_all_nineteen_bodies` fails, the CSV is missing rows — complete all 19 from `seorbel.txt`.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-fict/data crates/pleiades-fict/src/elements.rs
git commit -m "feat(fict): SP-3 committed seorbel.txt element table + parser"
```

---

## Task 5: `FictitiousBackend` — `EphemerisBackend` implementation

**Files:**
- Create: `crates/pleiades-fict/src/backend.rs`
- Modify: `crates/pleiades-fict/src/lib.rs` (add `pub mod backend; pub use backend::FictitiousBackend;`)
- Test: inline in `src/backend.rs`

**Interfaces:**
- Consumes: `elements::{elements_for, Center, KeplerElements}`; `pleiades_backend::{EphemerisBackend, EphemerisRequest, EphemerisResult, EphemerisError, BackendMetadata, BodyClaim, ...}`; a Sun-source backend `S: EphemerisBackend`.
- Produces: `FictitiousBackend<S>` with `new(sun_source: S) -> Self`; `EphemerisBackend` impl; `fictitious_body_claims() -> Vec<BodyClaim>`.

- [ ] **Step 1: Write the backend + failing test**

`crates/pleiades-fict/src/backend.rs`:

```rust
//! `FictitiousBackend` — serves the SE fictitious bodies through the standard
//! backend trait. Heliocentric bodies are geocentricized by reusing a Sun-source
//! backend (Earth heliocentric = − Sun geocentric); geocentric-orbit bodies are
//! returned directly. Output is mean, geometric, geocentric, J2000 mean ecliptic.

use pleiades_backend::{
    validate_observer_policy, validate_request_policy, validate_zodiac_policy, AccuracyClass,
    BackendCapabilities, BackendFamily, BackendId, BackendMetadata, BackendProvenance, BodyClaim,
    ClaimEvidence, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
    EphemerisResult, QualityAnnotation,
};
use pleiades_types::{
    CelestialBody, CoordinateFrame, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude,
    Motion, TimeRange, TimeScale, ZodiacMode,
};

use crate::elements::{elements_for, Center, KeplerElements};
use crate::PACKAGE_NAME;

/// The 19 SE fictitious bodies this backend serves.
pub fn fictitious_bodies() -> Vec<CelestialBody> {
    crate::elements::TABLE.iter().map(|(b, _)| *b).collect()
}

/// Body claims: every fictitious body is release-grade *by definition* — parity
/// with SE's `seorbel.txt`-driven Kepler orbit, gated by `validate-fictitious`.
pub fn fictitious_body_claims() -> Vec<BodyClaim> {
    fictitious_bodies()
        .into_iter()
        .map(|body| {
            BodyClaim::release_grade(
                body,
                AccuracyClass::Exact,
                ClaimEvidence::AlgorithmicModel,
            )
        })
        .collect()
}

/// A fictitious-body backend parameterized over a Sun-source backend `S`.
#[derive(Debug, Clone)]
pub struct FictitiousBackend<S> {
    sun_source: S,
}

impl<S: EphemerisBackend> FictitiousBackend<S> {
    /// Create a backend that uses `sun_source` for the Sun's geocentric position.
    pub const fn new(sun_source: S) -> Self {
        Self { sun_source }
    }

    /// Earth's heliocentric J2000-ecliptic Cartesian position (AU) at `instant`,
    /// obtained as the negation of the Sun's geocentric position from the source.
    fn earth_heliocentric(&self, instant: Instant) -> Result<[f64; 3], EphemerisError> {
        let req = EphemerisRequest::new(CelestialBody::Sun, instant);
        let sun = self.sun_source.position(&req)?;
        let ecl = sun.ecliptic.ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::MissingDataset,
                "Sun source returned no ecliptic position for the fictitious-body geocentric assembly",
            )
        })?;
        let (sx, sy, sz) = spherical_to_cartesian(&ecl);
        Ok([-sx, -sy, -sz])
    }

    /// Geocentric J2000-mean-ecliptic coordinates for a fictitious body at `instant`.
    fn geocentric_ecliptic(
        &self,
        el: &KeplerElements,
        instant: Instant,
    ) -> Result<EclipticCoordinates, EphemerisError> {
        let jd = instant.julian_day.days();
        let (bx, by, bz) = el.state_at(jd);
        let (gx, gy, gz) = match el.center {
            Center::Geocentric => (bx, by, bz),
            Center::Heliocentric => {
                let earth = self.earth_heliocentric(instant)?;
                (bx - earth[0], by - earth[1], bz - earth[2])
            }
        };
        Ok(cartesian_to_ecliptic(gx, gy, gz))
    }

    /// Symmetric finite-difference motion (±0.5 d), matching `ElpBackend::motion`.
    fn motion(&self, el: &KeplerElements, instant: Instant) -> Result<Motion, EphemerisError> {
        const HALF: f64 = 0.5;
        let at = |offset: f64| -> Result<EclipticCoordinates, EphemerisError> {
            let shifted = Instant::new(
                JulianDay::from_days(instant.julian_day.days() + offset),
                instant.scale,
            );
            self.geocentric_ecliptic(el, shifted)
        };
        let before = at(-HALF)?;
        let after = at(HALF)?;
        let full = HALF * 2.0;
        let lon_speed = signed_longitude_delta(before.longitude.degrees(), after.longitude.degrees()) / full;
        let lat_speed = (after.latitude.degrees() - before.latitude.degrees()) / full;
        let dist_speed = match (before.distance_au, after.distance_au) {
            (Some(b), Some(a)) => Some((a - b) / full),
            _ => None,
        };
        Ok(Motion::new(Some(lon_speed), Some(lat_speed), dist_speed))
    }
}

fn spherical_to_cartesian(ecl: &EclipticCoordinates) -> (f64, f64, f64) {
    let r = ecl.distance_au.unwrap_or(1.0);
    let lon = ecl.longitude.degrees().to_radians();
    let lat = ecl.latitude.degrees().to_radians();
    (r * lat.cos() * lon.cos(), r * lat.cos() * lon.sin(), r * lat.sin())
}

fn cartesian_to_ecliptic(x: f64, y: f64, z: f64) -> EclipticCoordinates {
    let r = (x * x + y * y + z * z).sqrt();
    let lon = y.atan2(x).to_degrees().rem_euclid(360.0);
    let lat = if r == 0.0 { 0.0 } else { (z / r).asin().to_degrees() };
    EclipticCoordinates::new(
        Longitude::from_degrees(lon),
        Latitude::from_degrees(lat),
        Some(r),
    )
}

fn signed_longitude_delta(before: f64, after: f64) -> f64 {
    let mut d = after - before;
    while d > 180.0 {
        d -= 360.0;
    }
    while d < -180.0 {
        d += 360.0;
    }
    d
}

impl<S: EphemerisBackend> EphemerisBackend for FictitiousBackend<S> {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new(PACKAGE_NAME),
            version: env!("CARGO_PKG_VERSION").to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance {
                summary: "Fictitious/hypothetical bodies (SE seorbel.txt 40–58) as unperturbed Kepler orbits; definitional parity with Swiss Ephemeris via validate-fictitious.".to_string(),
                data_sources: vec![
                    "Osculating elements transcribed from Swiss Ephemeris seorbel.txt; unperturbed Kepler propagation in pure Rust.".to_string(),
                ],
            },
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
            body_claims: fictitious_body_claims(),
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities {
                geocentric: true,
                topocentric: false,
                apparent: false,
                mean: true,
                batch: true,
                native_sidereal: false,
            },
            accuracy: AccuracyClass::Exact,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        elements_for(body).is_some()
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let el = elements_for(req.body.clone()).ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "the fictitious backend serves only SE seorbel.txt bodies 40–58",
            )
        })?;

        validate_zodiac_policy(req, "the fictitious backend", &[ZodiacMode::Tropical])?;
        validate_request_policy(
            req,
            "the fictitious backend",
            &[TimeScale::Tt, TimeScale::Tdb],
            &[CoordinateFrame::Ecliptic],
            true,
            false,
        )?;
        validate_observer_policy(req, "the fictitious backend", false)?;

        let mut result = EphemerisResult::new(
            BackendId::new(PACKAGE_NAME),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.quality = QualityAnnotation::Exact;
        result.ecliptic = Some(self.geocentric_ecliptic(el, req.instant)?);
        result.motion = Some(self.motion(el, req.instant)?);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A trivial Sun source placing the Sun at 1 AU along +x (geocentric), i.e.
    // Earth at -1 AU heliocentric — enough to exercise the assembly path.
    #[derive(Debug, Clone)]
    struct StubSun;
    impl EphemerisBackend for StubSun {
        fn metadata(&self) -> BackendMetadata {
            FictitiousBackend::new(StubSun).metadata() // reuse shape; unused fields
        }
        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }
        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            let mut r = EphemerisResult::new(
                BackendId::new("stub-sun"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            );
            r.ecliptic = Some(EclipticCoordinates::new(
                Longitude::from_degrees(0.0),
                Latitude::from_degrees(0.0),
                Some(1.0),
            ));
            Ok(r)
        }
    }

    #[test]
    fn supports_only_fictitious_bodies() {
        let b = FictitiousBackend::new(StubSun);
        assert!(b.supports_body(CelestialBody::Cupido));
        assert!(!b.supports_body(CelestialBody::Mars));
    }

    #[test]
    fn position_returns_ecliptic_and_motion_for_a_fictitious_body() {
        let b = FictitiousBackend::new(StubSun);
        let req = EphemerisRequest::new(
            CelestialBody::Cupido,
            Instant::new(JulianDay::from_days(crate::J2000_JD), TimeScale::Tt),
        );
        let r = b.position(&req).unwrap();
        assert!(r.ecliptic.is_some());
        assert!(r.motion.is_some());
    }

    #[test]
    fn unsupported_body_fails_closed() {
        let b = FictitiousBackend::new(StubSun);
        let req = EphemerisRequest::new(
            CelestialBody::Mars,
            Instant::new(JulianDay::from_days(crate::J2000_JD), TimeScale::Tt),
        );
        assert!(b.position(&req).is_err());
    }
}
```

Notes for the implementer: verify exact names against `pleiades-backend` — `BodyClaim::release_grade(body, accuracy, evidence)` exists (`crates/pleiades-backend/src/claims.rs:126`); `ClaimEvidence` variants live in the same module (use the closest analogue to "algorithmic model"; if a more fitting variant exists for definitional/parity evidence, use it). `EphemerisResult` fields `ecliptic`, `motion`, `quality`, `equatorial` are public (see `ElpBackend`). If `validate_request_policy`'s arity/signature differs, mirror the exact call in `crates/pleiades-elp/src/backend.rs:position`. The stub's `metadata()` is only shape-filler; if it recurses awkwardly, return a minimal hand-built `BackendMetadata` instead.

- [ ] **Step 2: Wire into lib.rs**

Add to `crates/pleiades-fict/src/lib.rs`:

```rust
pub mod backend;
pub use backend::{fictitious_body_claims, fictitious_bodies, FictitiousBackend};
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p pleiades-fict`
Expected: PASS.

- [ ] **Step 4: Add the definitional evidence string (claim honesty)**

If `ClaimEvidence` supports a free-text/definitional note, set the fictitious claim evidence to state *"Definitional: unperturbed Kepler orbit from committed seorbel.txt elements; SE swe_calc parity via validate-fictitious (bodies ≥ 40)."* If `ClaimEvidence` is a closed enum with no free-text variant, keep `AlgorithmicModel` and record the definitional wording in the crate README + the overclaim map (Task 10) instead. Re-run `cargo test -p pleiades-fict`.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-fict/src
git commit -m "feat(fict): SP-3 FictitiousBackend EphemerisBackend impl + claims"
```

---

## Task 6: Wire `FictitiousBackend` into the chart routing chain

**Files:**
- Modify: `crates/pleiades-cli/src/commands/chart.rs:559-566` (routing chain)
- Modify: `crates/pleiades-cli/Cargo.toml` (add `pleiades-fict` dep)
- Test: `crates/pleiades-cli/` chart test (mirror an existing chart CLI test)

**Interfaces:**
- Consumes: `pleiades_fict::FictitiousBackend`, `pleiades_data::PackagedDataBackend` (the Sun source, already imported in `chart.rs`).

- [ ] **Step 1: Add the dependency**

In `crates/pleiades-cli/Cargo.toml` `[dependencies]`, add `pleiades-fict = { workspace = true }`.

- [ ] **Step 2: Write the failing test**

Add a test that a fictitious body appears in a rendered chart (mirror the closest existing chart CLI test — find one that calls the chart command with `--bodies` and asserts on output). Assert that requesting `Transpluto` (via whatever body-name flag the CLI uses) yields a longitude line, not an "unsupported body" error.

```rust
#[test]
fn chart_includes_a_fictitious_body() {
    // mirror the existing chart-command test harness in this file/module
    let out = run_chart(&["--date", "2000-01-01", "--bodies", "Transpluto"]);
    assert!(out.contains("Transpluto"), "chart output: {out}");
    assert!(!out.to_lowercase().contains("unsupported"), "chart output: {out}");
}
```

Adjust flag names/harness (`run_chart`, date format, body-name parsing) to match the existing chart tests in the CLI crate. If body-name → `CelestialBody` parsing is a separate map, extend it to accept the fictitious names in this step.

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p pleiades-cli chart_includes_a_fictitious_body`
Expected: FAIL — body unsupported / not parsed.

- [ ] **Step 4: Add the backend to the routing chain**

In `crates/pleiades-cli/src/commands/chart.rs`, change the chain (currently lines 559-566) to append the fictitious backend, using the packaged backend as its Sun source:

```rust
    let backend = RoutingBackend::new(vec![
        Box::new(PackagedDataBackend::new()),
        Box::new(CompositeBackend::new(
            Vsop87Backend::new(),
            ElpBackend::new(),
        )),
        Box::new(JplSnapshotBackend::new()),
        Box::new(pleiades_fict::FictitiousBackend::new(PackagedDataBackend::new())),
    ]);
```

Add `use pleiades_fict::FictitiousBackend;` if preferred over the fully-qualified path. If the CLI has a body-name parser, ensure the 19 fictitious names resolve to their `CelestialBody` variants.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p pleiades-cli chart_includes_a_fictitious_body`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-cli
git commit -m "feat(cli): SP-3 route fictitious bodies into the chart backend chain"
```

---

## Task 7: SE reference-corpus generator tool

**Files:**
- Create: `tools/se-fictitious-reference/Cargo.toml`, `tools/se-fictitious-reference/src/main.rs`
- Modify: `Cargo.toml` (root) `exclude` array — add `"tools/se-fictitious-reference"`
- Test: manual (`--dry-run`); requires Swiss Ephemeris via `libswisseph-sys`.

**Interfaces:**
- Produces: `tools/se-fictitious-reference` binary emitting `fictitious.csv` + `manifest.txt` (row schema agreed with Task 8's validator). fnv1a64 re-implemented locally, byte-identical to `pleiades_apparent::fnv1a64`.

- [ ] **Step 0: Commit the SE element source (`seorbel.txt`)**

The generator needs Swiss Ephemeris's `seorbel.txt`; SE's built-in fallback covers only bodies 40–54, so bodies 55–58 (Vulcan, White Moon, Proserpina, Waldemath) require the file. Fetch the verbatim distribution file and commit it as provenance (it is also the transcription source for Task 4's CSV):

```bash
mkdir -p tools/se-fictitious-reference/data
curl -fsSL https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/seorbel.txt \
  -o tools/se-fictitious-reference/data/seorbel.txt
# sanity: 19 in-scope element sets (sets 1–19 = SE 40–58); the file also holds
# out-of-scope sets 20+ (comets, Planet 9) which SE never reaches for ipl 40–58.
```

Keep the file **verbatim** — do not edit or trim it. (A copy fetched during planning is also staged at `<session-scratchpad>/seorbel.txt`.) Commit it together with the tool in Step 5.

- [ ] **Step 1: Scaffold the tool manifest**

Model on `tools/se-eclipse-local-reference/Cargo.toml` (own empty `[workspace]` table, `libswisseph-sys = "0.1.2"`):

```toml
[package]
name = "se-fictitious-reference"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]

[dependencies]
libswisseph-sys = "0.1.2"
```

- [ ] **Step 2: Exclude it from the root workspace**

In root `Cargo.toml`, add `"tools/se-fictitious-reference"` to the `exclude` array (alongside `"tools/se-eclipse-local-reference"`).

- [ ] **Step 3: Write the generator**

`tools/se-fictitious-reference/src/main.rs`. Mirror `tools/se-eclipse-local-reference/src/main.rs` structure. Core:
- Point SE at the committed elements file: `swe_set_ephe_path(dir)` where `dir` = `tools/se-fictitious-reference/data` (contains `seorbel.txt`, committed in Step 0). **Required** — without `seorbel.txt`, SE's built-in fallback covers only bodies 40–54 and errors on 55–58 (`SE_NFICT_ELEM = 15`).
- Flags: `SEFLG_MOSEPH | SEFLG_J2000 | SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL` (values `4 | 32 | 16 | 1024 | 512`). This matches the backend's **geometric** geocentric J2000 boundary: `SEFLG_TRUEPOS` drops light-time, `SEFLG_NOABERR` drops annual aberration, `SEFLG_NOGDEFL` drops gravitational deflection. Bare `SEFLG_J2000` alone emits *apparent* positions (light-time + aberration) and would break parity vs `FictitiousBackend::position`.
- For each SE body number 40..=58, sample a per-body date grid over JD for 1900–2100 TDB (e.g., N dates each), call `swe_calc(jd_tt, ipl, iflag, xx)` to get J2000 ecliptic lon/lat/dist. **Check the return code / `serr` and abort on any SE error** (fail-closed) so a missing or unreadable `seorbel.txt` cannot silently yield a short corpus.
- Emit CSV rows: `label,se_body,jd_tt,lon_deg,lat_deg,dist_au`. Include a header line prefixed so the validator's filter skips it, and `#` comments citing the SE version and the exact `iflag`.
- Copy the local `fnv1a64` from the eclipse tool verbatim (with the same "matches repo scheme" comment).
- `build_manifest()` emits `file: fictitious.csv rows={n} checksum={fnv1a64(csv)}`.
- `--dry-run` prints; `--out <dir>` writes `fictitious.csv` + `manifest.txt`.

- [ ] **Step 4: Build + dry-run**

Run: `cargo run --manifest-path tools/se-fictitious-reference/Cargo.toml -- --dry-run`
Expected: prints rows for all 19 bodies with finite lon/lat/dist. If SE cannot find `seorbel.txt`, set `swe_set_ephe_path` to the directory containing it and re-run.

- [ ] **Step 5: Commit the tool (not yet the corpus)**

```bash
git add tools/se-fictitious-reference Cargo.toml
git commit -m "tools: SP-3 Swiss-Ephemeris fictitious-body reference generator"
```

---

## Task 8: Generate corpus + `validate-fictitious` two-tier gate

**Files:**
- Create: `crates/pleiades-validate/data/fictitious-corpus/fictitious.csv`, `manifest.txt`, `MANIFEST.md`
- Create: `crates/pleiades-validate/src/fictitious_validation.rs`, `crates/pleiades-validate/src/fictitious_thresholds.rs`
- Modify: `crates/pleiades-validate/src/lib.rs` (module decls + re-exports)
- Modify: `crates/pleiades-validate/Cargo.toml` (add `pleiades-fict`, `pleiades-data` deps if absent)
- Test: inline `#[cfg(test)]` in `fictitious_validation.rs`

**Interfaces:**
- Consumes: `pleiades_fict::FictitiousBackend`, `pleiades_data::packaged_backend`, `pleiades_apparent::fnv1a64`.
- Produces: `validate_fictitious_corpus() -> Result<FictitiousReport, FictitiousError>`; `FictitiousReport::summary_line()`; `FictitiousReport::passed()`; `run_fictitious_tier1_only()`.

- [ ] **Step 1: Generate + commit the corpus**

```bash
cargo run --manifest-path tools/se-fictitious-reference/Cargo.toml -- --out crates/pleiades-validate/data/fictitious-corpus
```

Then hand-write `crates/pleiades-validate/data/fictitious-corpus/MANIFEST.md` (human companion: source SE version, flags `SEFLG_MOSEPH|SEFLG_J2000`, window, per-body sample count, row schema) mirroring `data/eclipses-local-corpus/MANIFEST.md`.

- [ ] **Step 2: Write the thresholds module (initial generous ceilings)**

`crates/pleiades-validate/src/fictitious_thresholds.rs`. Start generous; tighten in Step 6:

```rust
//! Measured-basis ceilings for the fictitious-body SE-parity gate. Set from
//! observed residual maxima (~1.4×) once the corpus is measured (see Step 6).
//! Nibiru (equinox ~370 AD) is expected to exceed the others and may get a
//! documented per-body carve-out rather than inflating these globals.

/// Max ecliptic-longitude residual vs SE, arcseconds.
pub const LONGITUDE_ARCSEC: f64 = 60.0;
/// Max ecliptic-latitude residual vs SE, arcseconds.
pub const LATITUDE_ARCSEC: f64 = 60.0;
/// Max radial-distance residual vs SE, AU (relative-scale check).
pub const DISTANCE_AU: f64 = 1.0e-3;
```

- [ ] **Step 3: Write the validation module (two-tier), failing test first**

`crates/pleiades-validate/src/fictitious_validation.rs`. Mirror `eclipse_local_validation.rs`. Structure:
- `include_str!` the committed `fictitious.csv` + `manifest.txt`.
- `const EXPECTED_ROWS: usize = <N>;` (the total row count the generator emitted — read it from the manifest).
- `use pleiades_apparent::fnv1a64;` `use crate::fictitious_thresholds::*;`
- `parse_manifest()` + `check_checksum()` copied structurally from the eclipse module (same `file: NAME rows=N checksum=U64` format).
- `FictitiousError` enum (`ChecksumMismatch`, `RowCountMismatch`, `ToleranceExceeded`, `Parse`, `Backend`) with `Display`.
- `FictitiousReport { rows, max_lon_arcsec, max_lat_arcsec, max_dist_au }` with `passed()` + `summary_line()`.
- `measure()`: checksum guard → parse rows → build `FictitiousBackend::new(pleiades_data::packaged_backend())` once → for each row, `position()` at `jd_tt` (TT/TDB Instant) → **Tier 1 self-consistency**: assert finite lon/lat/dist, lon∈[0,360), lat∈[−90,90]; accumulate **Tier 2** residuals |lon−se|, |lat−se| (with 360° wrap for lon), |dist−se|, tracking the max **per body** (record which body produced each maximum — for diagnosis and a possible per-body ceiling, see Step 6); enforce row count == `EXPECTED_ROWS`.
- `run_fictitious_tier1_only()` → tier-1 checks + row count, no ceilings.
- `validate_fictitious_corpus()` → `measure()` then gate the three maxima against `LONGITUDE_ARCSEC` / `LATITUDE_ARCSEC` / `DISTANCE_AU`, returning `ToleranceExceeded` on breach.

Failing test to add in the module's `#[cfg(test)] mod tests`:

```rust
#[test]
fn manifest_row_count_is_pinned() {
    let report = super::run_fictitious_tier1_only().expect("tier1 passes");
    assert_eq!(report.rows, super::EXPECTED_ROWS);
}

#[test]
fn checksum_drift_fails_closed() {
    // Corrupt-copy path: mirror the eclipse module's drift test — feed a mutated
    // CSV to check_checksum and assert ChecksumMismatch.
    assert!(super::check_checksum("fictitious.csv", "mutated,body\n").is_err());
}

#[test]
fn gate_passes_on_committed_corpus() {
    super::validate_fictitious_corpus().expect("fictitious gate passes");
}
```

Match `check_checksum`'s real signature to the eclipse module's (it takes the file name + the committed `&str` and compares against the manifest entry — adapt the drift test to that exact shape).

- [ ] **Step 4: Wire modules into lib.rs**

In `crates/pleiades-validate/src/lib.rs`, near the eclipse-local decls (lines ~25-26):

```rust
mod fictitious_thresholds;
pub mod fictitious_validation;
```

And in the public re-export block (near lines ~189-191):

```rust
pub use fictitious_validation::{
    validate_fictitious_corpus, FictitiousError, FictitiousReport,
};
```

Ensure `crates/pleiades-validate/Cargo.toml` depends on `pleiades-fict` and `pleiades-data` (add `= { workspace = true }` if missing).

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p pleiades-validate fictitious`
Expected: `manifest_row_count_is_pinned`, `checksum_drift_fails_closed`, `gate_passes_on_committed_corpus` PASS. If the gate fails on residuals, that's expected until Step 6.

- [ ] **Step 6: Set ceilings from measured residuals**

Temporarily print the measured maxima **and the body that produced each** (add an `eprintln!` in `measure()` or a throwaway test), run the gate, then set each constant in `fictitious_thresholds.rs` to ~1.4× the observed maximum (matching the eclipse-thresholds convention). Remove the debug print.

Two diagnostic rules:
- If longitude/latitude residuals are large for one **specific body**, its transcribed element row is likely wrong — a mis-exploded T-term, a swapped seorbel field (mean-anomaly is seorbel field 3 but a late CSV column), or the wrong `equinox`/`center` token. Fix Task 4, not the ceiling.
- **Nibiru is the expected outlier.** Its equinox is ~370 AD (~1630 yr), beyond the IAU-1976 precession helper's accurate range (Reconciliation §2), so its residual will exceed the others' by a wide margin. If Nibiru would force the global longitude ceiling above ~arcminute level, do **not** inflate the global gate: set the global ceilings from the max over the *other 18* bodies and give Nibiru a documented per-body carve-out (e.g. a `NIBIRU_LONGITUDE_ARCSEC` constant applied only to Nibiru rows in `validate_fictitious_corpus`). Disclose the Nibiru limitation in `MANIFEST.md` and the crate README — never hide it behind a loose global ceiling.

Re-run:

Run: `cargo test -p pleiades-validate fictitious`
Expected: PASS with tightened ceilings.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-validate/data/fictitious-corpus crates/pleiades-validate/src/fictitious_validation.rs crates/pleiades-validate/src/fictitious_thresholds.rs crates/pleiades-validate/src/lib.rs crates/pleiades-validate/Cargo.toml
git commit -m "feat(validate): SP-3 validate-fictitious two-tier SE-parity gate"
```

---

## Task 9: Wire the gate into the CLI + numeric battery

**Files:**
- Modify: `crates/pleiades-validate/src/render/cli.rs` (`run_all_numeric_gates` ~line 99-118; dispatch arms ~line 337-346; help banner ~line 2221; battery test ~line 2914-2938)
- Modify: `crates/pleiades-validate/src/tests/validate_gates.rs` (per-gate test block)

**Interfaces:**
- Consumes: `crate::validate_fictitious_corpus`. New CLI names: `validate-fictitious` (gate), `fictitious-gate` (alias), `fictitious` (listing — optional; if no listing renderer, omit).

- [ ] **Step 1: Add to the numeric battery**

In `run_all_numeric_gates()` (cli.rs ~line 99-118), after the eclipses-local entry, add:

```rust
    crate::validate_fictitious_corpus()
        .map_err(|e| format!("fictitious gate failed: {e}"))?;
```

This automatically covers `release-smoke` and `release-gate` (both route through this function).

- [ ] **Step 2: Add the dispatch arm**

In `render_cli()` near the eclipses-local arms (~line 337-346):

```rust
        Some("validate-fictitious") | Some("fictitious-gate") => {
            ensure_no_extra_args(&args[1..], "validate-fictitious")?;
            crate::validate_fictitious_corpus()
                .map(|r| r.summary_line())
                .map_err(|e| e.to_string())
        }
```

(Skip a `fictitious` listing arm unless you add a `render_fictitious_listing` in Task 8; the chart CLI already renders positions, so a listing alias is optional.)

- [ ] **Step 3: Add to the help/command banner**

In the help string literal (~line 2221), add lines after the `eclipses-local` block, matching the existing format:

```
  validate-fictitious        Fictitious-body SE-parity gate (SE seorbel.txt 40–58)
  fictitious-gate            Alias for validate-fictitious
```

- [ ] **Step 4: Add the battery test**

In the inline test at cli.rs (~line 2914-2938), mirror `run_all_numeric_gates_includes_eclipses_local_and_passes`:

```rust
#[test]
fn run_all_numeric_gates_includes_fictitious_and_passes() {
    crate::validate_fictitious_corpus().expect("fictitious gate passes standalone");
    assert!(run_all_numeric_gates().is_ok());
}
```

- [ ] **Step 5: Add the per-gate CLI test**

In `crates/pleiades-validate/src/tests/validate_gates.rs`, mirror the eclipses-local block (~line 233-256): assert `validate-fictitious` returns Ok, primary output == `fictitious-gate` alias output, extra args are rejected ("does not accept extra arguments"), and help text mentions both `validate-fictitious` and `fictitious-gate`.

```rust
#[test]
fn validate_fictitious_and_alias_agree_and_reject_extra_args() {
    let primary = render_cli(&["validate-fictitious"]).expect("gate ok");
    let alias = render_cli(&["fictitious-gate"]).expect("alias ok");
    assert_eq!(primary, alias);
    assert!(render_cli(&["validate-fictitious", "extra"]).is_err());
    let help = render_cli(&["help"]).expect("help ok");
    assert!(help.contains("validate-fictitious"));
    assert!(help.contains("fictitious-gate"));
}
```

Match the exact test-harness call convention used by the neighboring gate tests (`render_cli` argument shape, help subcommand name).

- [ ] **Step 6: Run the tests**

Run: `cargo test -p pleiades-validate fictitious`
Expected: all fictitious tests + battery test PASS.

Run: `cargo run -p pleiades-cli -- validate-fictitious`
Expected: prints the gate summary line, exit 0.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-validate/src/render/cli.rs crates/pleiades-validate/src/tests/validate_gates.rs
git commit -m "feat(cli): SP-3 wire validate-fictitious into gate battery + fictitious-gate alias"
```

---

## Task 10: Version bump, docs, and SP-3 closure

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/mod.rs` (profile id line 26; content checksum line 38; summary prose line 41; SP array lines 83-85)
- Modify: `crates/pleiades-fict/README.md`, `README.md` (root), `PLAN.md`, `plan/status/01-current-execution-frontier.md`, `plan/status/02-next-slice-candidates.md`
- Modify: overclaim-audit claim↔evidence surface if it enumerates gates/bodies (`crates/pleiades-validate/src/claims/compat.rs` — verify)

**Interfaces:** none (docs + version).

- [ ] **Step 1: Bump the compatibility profile id**

In `crates/pleiades-core/src/compatibility/mod.rs` line 26:

```rust
pub const CURRENT_COMPATIBILITY_PROFILE_ID: &str = "pleiades-compatibility-profile/0.7.9";
```

- [ ] **Step 2: Extend the summary prose + SP array**

Add a sentence to `CURRENT_COMPATIBILITY_PROFILE_SUMMARY` (line 41) describing SP-3 (fictitious bodies via `pleiades-fict::FictitiousBackend`, SE seorbel.txt 40–58, gated by `validate-fictitious`, profile 0.7.9). Add a new element to the per-SP `&[&str]` array (lines 83-85) mirroring the SP-2c entry.

- [ ] **Step 3: Fix the content checksum**

The content checksum (line 38) is `fnv1a64` of the rendered profile and will now be stale. Run the compat-profile verification to get the expected value:

Run: `cargo test -p pleiades-core compatibility 2>&1 | grep -iE "checksum|expected|0x"`
Expected: a test fails printing the new checksum (or asserts expected vs actual). Set `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` (line 38) to the new value.

Run: `cargo test -p pleiades-core compatibility`
Expected: PASS.

- [ ] **Step 4: Verify the overclaim audit still passes**

Run: `cargo run -p pleiades-cli -- compat-claims-audit`
Expected: OK. If it enumerates per-body/per-gate claim tiers and flags the new fictitious bodies, extend the claim↔evidence mapping in `crates/pleiades-validate/src/claims/compat.rs` so the 19 fictitious bodies' `ReleaseGrade`/definitional tier matches the profile + README prose. Re-run until OK.

- [ ] **Step 5: Update the narrative docs**

- `README.md` (root): add a fictitious-bodies line to the event-engine/feature list and bump any profile-version mention to 0.7.9.
- `PLAN.md`: append an SP-3 done clause to the status line (mirroring the SP-2c clause), and note remaining event-engine follow-ups (`swe_pheno`, `swe_nod_aps`, custom elements, occultations, central-path cartography).
- `plan/status/01-current-execution-frontier.md` and `plan/status/02-next-slice-candidates.md`: move SP-3 from "next candidate / not yet scoped" to **done**; record `swe_pheno` / `swe_nod_aps` as the next candidate slices.
- `crates/pleiades-fict/README.md`: ensure it states the definitional-parity claim and the gate name, cites the `seorbel.txt` provenance, and discloses the Nibiru precession limitation (IAU-1976 ecliptic precession vs SE's long-term model at Nibiru's ~370 AD equinox).

- [ ] **Step 6: Full-workspace verification**

Run: `cargo test --workspace`
Expected: all green.

Run: `cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all --check`
Expected: clean.

Run: `cargo run -p pleiades-cli -- release-gate`
Expected: the full battery (including `validate-fictitious`) passes.

Run: `cargo run -p pleiades-cli -- compatibility-profile 2>/dev/null | grep 0.7.9` (adapt to the actual profile-print subcommand)
Expected: prints `0.7.9`.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-core/src/compatibility/mod.rs crates/pleiades-fict/README.md README.md PLAN.md plan/status crates/pleiades-validate/src/claims/compat.rs
git commit -m "docs(events): SP-3 declare fictitious bodies; profile 0.7.9; mark SP-3 done"
```

---

## Self-Review

**1. Spec coverage** (against `docs/superpowers/specs/2026-07-06-sp3-fictitious-bodies-design.md`):

- §Design.1 crate & structure → Tasks 2–5. ✓
- §Design.2 body model (`Fictitious` class + 19 variants, non-breaking) → Task 1. ✓
- §Design.3 computation & frame (time-poly elements, two centering paths, J2000 output, motion) → Tasks 3–5 (motion via finite-difference per the errata note). ✓
- §Design.4 claim tier (`ReleaseGrade` + definitional evidence) → Task 5 Steps 1/4. ✓
- §Design.5 native CSV element table → Task 4 (values from seorbel.txt; generator can cross-emit in Task 7). ✓
- §Design.6 `validate-fictitious` two-tier gate + CLI aliases + generator → Tasks 7–9. ✓
- §Design.7 versioning + docs + SP-3 closure → Task 10. ✓
- §Non-goals → recorded in Task 10 Step 5. ✓
- §"SE functions targeted" (bodies 40–58, geo vs helio) → Task 1 variants + Task 4 `center` column + Task 7 SE numbers. ✓

**2. Placeholder scan:** The only deferred values are the `seorbel.txt` orbital-element numbers (Task 4) and the measured-residual ceilings (Task 8 Step 6) — both are legitimately data-sourced/measurement-derived, not code placeholders, and each has an explicit acceptance step (the parity gate; the ~1.4×-max rule). All code steps contain complete code.

**3. Type consistency:** `FictitiousBackend::new(sun_source)`, `elements_for(body) -> Option<&'static KeplerElements>`, `KeplerElements::state_at(jd) -> (f64,f64,f64)`, `rotate_ecliptic_to_j2000(x,y,z,equinox_jd)`, `Equinox { Fixed(f64), OfDate }`, `solve_kepler`/`orbital_plane_position`, `validate_fictitious_corpus() -> Result<FictitiousReport, FictitiousError>` are used consistently across tasks. CSV column order (Task 4 Step 1: `…,equinox,center`) matches the parser field indices (Task 4 Step 2: `f[20]` equinox, `f[21]` center). The Kepler/frame model matches SE `swi_osc_el_plan` per the Reconciliation section. The gate names `validate-fictitious`/`fictitious-gate` are identical in Tasks 8–10.

**Known verification points for the implementer** (call out at review, not blockers): exact `ClaimEvidence` variant + whether it carries free text (Task 5 Step 4); exact `validate_request_policy` signature (mirror `ElpBackend`); `EphemerisResult` field names (`ecliptic`/`motion`/`quality`); the CLI chart body-name parser location (Task 6); `check_checksum` signature in the eclipse module (Task 8 Step 3); the compat-profile print subcommand name (Task 10 Step 6).
