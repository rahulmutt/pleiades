# Mean Lunar Points on the Packaged Backend (#90) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `packaged_backend()` serves `MeanNode`, `MeanApogee` and `MeanPerigee` release-grade behind a new Swiss Ephemeris gate, and `nod_aps(Moon, Mean)` works on any backend.

**Architecture:** The Meeus mean-lunar-element polynomials and the Moon's mean orbit constants move into the dependency-free `pleiades-apsides` crate. `pleiades-events` builds the Moon's mean elements from it directly instead of reading backend channels; `pleiades-elp` delegates to it; `pleiades-data` forms the three points with `points_from_elements` in the mean ecliptic of date and emits them in the J2000 boundary frame through the existing derived-point path. `pleiades-core` treats all six lunar points as geometric directions. A new `validate-mean-lunar-points` gate pins the packaged points against a committed `SE_MEAN_NODE` / `SE_MEAN_APOG` corpus.

**Tech Stack:** Rust (workspace toolchain from `mise.toml`), `cargo nextest`, `proptest` (already a workspace dev-dependency where used), `libswisseph-sys` for the out-of-workspace reference tool (built in `devenv shell`).

**Spec:** `docs/superpowers/specs/2026-10-01-mean-lunar-points-packaged-design.md` — read it before starting any task.

## Global Constraints

- Polynomials and the three constants move **verbatim**: node `125.044_547_9, -1_934.136_289_1, 0.002_075_4, 1/476_441, -1/60_616_000`; perigee `83.353_246_5, 4_069.013_728_7, -0.010_32, -1/80_053, 1/18_999_000`; `5.145_396_4`°, `0.054_900_489`, `384_400_000.0 / 1.495_978_707_00e11` AU.
- `pleiades-apsides` keeps **zero dependencies**. `pleiades-data` gains no new dependency. `pleiades-elp` gains exactly one: `pleiades-apsides`.
- Backend boundary frame is J2000 mean ecliptic; of-date points are precessed with `pleiades_apparent::precess_ecliptic_date_to_j2000`.
- Mean points are served only inside the packaged window; outside it the error kind is `OutOfRangeInstant`.
- Corpus grid: JD 2415020.5 to 2488070.0 TT, 23-day step, **3177 rows**, `SEFLG_MOSEPH`, nutation on.
- **Stop conditions (do not work around; report to the user):** any channel's measured longitude maximum above 5″ against the corpus; the reference tool cannot be built in `devenv shell`.
- `validate-nod-aps` `MEAN_MOON_LONGITUDE_ARCSEC` stays 0.8.
- Compatibility profile id `0.7.15` → `0.7.16` in all three pinned files.
- Commit type `feat`/`refactor`/`test`/`docs` per task; never bump crate versions (release-plz owns them).
- Run `cargo fmt --all` before every commit. Do not edit sources or commit while a background test run is in progress.
- No `unwrap`/`expect` in new library paths.

## Review Focus

1. **Instant outside the packaged window** for a mean point: must be `OutOfRangeInstant`, not a silently extrapolated value (Task 5 test).
2. **Coverage boundary instant** (JD 2415020.5): position must be served, motion channels degrade to `None` (Task 5 test).
3. **TDB-tagged request** for a mean point: accepted and equal to the TT answer (Task 5 test).
4. **Topocentric chart containing a mean point**: unchanged by the topocentric flag, no parallax in provenance (Task 6 test).
5. **Non-finite or far-out-of-range JD into the polynomial functions**: non-finite in gives non-finite out without panic; finite input always lands in `[0, 360)` (Task 1 tests).

## File Structure

| File | Responsibility | Task |
|---|---|---|
| `crates/pleiades-apsides/src/mean.rs` (new) | Mean lunar element polynomials, constants, `mean_lunar_elements_of_date` | 1 |
| `crates/pleiades-apsides/src/mean/tests.rs` (new) | Unit + property tests for the above | 1 |
| `crates/pleiades-apsides/src/lib.rs`, `Cargo.toml`, `README.md` | Module wiring, re-exports, widened description | 1 |
| `crates/pleiades-elp/src/backend.rs`, `Cargo.toml` | Delegate polynomials | 2 |
| `crates/pleiades-events/src/nod_aps.rs`, `mean_elements.rs`, `ephemeris.rs`, `tests/nod_aps.rs` | Moon mean elements from `pleiades-apsides`; remove backend read | 3 |
| `tools/se-mean-lunar-reference/` (new) | SE corpus generator | 4 |
| `crates/pleiades-validate/data/mean-lunar-corpus/` (new) | Committed corpus + manifest | 4 |
| `crates/pleiades-data/src/backend.rs`, `src/tests/lookup.rs` | Serve the three points | 5 |
| `crates/pleiades-core/src/chart/mod.rs`, `chart/tests.rs` | One lunar-point predicate | 6 |
| `crates/pleiades-validate/src/mean_lunar_validation.rs` (new), `lib.rs`, `render/cli.rs`; `crates/pleiades-cli/src/cli.rs`; `crates/pleiades-data/src/lib.rs`, `backend.rs` | Gate, wiring, release-grade claims | 7 |
| Compatibility profile, READMEs, `docs/follow-ups.md`, `docs/lunar-theory-policy.md` | Docs and release posture | 8 |

---

### Task 1: Mean lunar orbit in `pleiades-apsides`

**Files:**
- Create: `crates/pleiades-apsides/src/mean.rs`
- Create: `crates/pleiades-apsides/src/mean/tests.rs`
- Modify: `crates/pleiades-apsides/src/lib.rs` (module doc at top; add `mod mean; pub use ...` after the `#![deny(missing_docs)]` line)
- Modify: `crates/pleiades-apsides/Cargo.toml:3` (description), `crates/pleiades-apsides/README.md`

**Interfaces:**
- Consumes: `KeplerianElements` (existing, `crates/pleiades-apsides/src/lib.rs`).
- Produces (all re-exported at the crate root):
  - `pub const MOON_MEAN_INCLINATION_DEG: f64`
  - `pub const MOON_MEAN_ECCENTRICITY: f64`
  - `pub const MOON_MEAN_SEMI_MAJOR_AU: f64`
  - `pub fn mean_lunar_node_longitude_of_date(jd_tt: f64) -> f64`
  - `pub fn mean_lunar_perigee_longitude_of_date(jd_tt: f64) -> f64`
  - `pub fn mean_lunar_elements_of_date(jd_tt: f64) -> KeplerianElements`

- [ ] **Step 1: Check whether `proptest` is available to this crate**

Run: `grep -n "proptest" Cargo.toml crates/pleiades-apsides/Cargo.toml`
If the workspace declares `proptest` under `[workspace.dependencies]`, add `proptest = { workspace = true }` under `[dev-dependencies]` in `crates/pleiades-apsides/Cargo.toml`. If the workspace does not declare it, copy the exact dev-dependency line another crate uses (`grep -rn "^proptest" crates/*/Cargo.toml | head -1`). Dev-dependencies do not break the zero-dependency constraint.

- [ ] **Step 2: Write the failing tests**

Create `crates/pleiades-apsides/src/mean/tests.rs`:

```rust
//! Unit tests for the mean lunar orbit elements.

use super::*;
use crate::points_from_elements;
use proptest::prelude::*;

const J2000: f64 = 2_451_545.0;

#[test]
fn polynomials_reproduce_their_j2000_constants() {
    assert!((mean_lunar_node_longitude_of_date(J2000) - 125.044_547_9).abs() < 1e-12);
    assert!((mean_lunar_perigee_longitude_of_date(J2000) - 83.353_246_5).abs() < 1e-12);
}

#[test]
fn polynomials_match_hand_evaluation_one_century_out() {
    // t = 1 Julian century: every coefficient contributes.
    let jd = J2000 + 36_525.0;
    assert!((mean_lunar_node_longitude_of_date(jd) - 350.910_336_282_4).abs() < 1e-7);
    assert!((mean_lunar_perigee_longitude_of_date(jd) - 192.356_642_760_9).abs() < 1e-7);
}

#[test]
fn polynomials_match_the_meeus_worked_dates() {
    // Published examples: the mean node passes 0° on 1913-05-27 and 180° on
    // 1959-12-07; the mean perigee is 224.89194° on 2021-03-05 (0h TT).
    assert!(mean_lunar_node_longitude_of_date(2_419_914.5) < 0.1);
    assert!((mean_lunar_node_longitude_of_date(2_436_909.5) - 180.0).abs() < 0.1);
    assert!((mean_lunar_perigee_longitude_of_date(2_459_278.5) - 224.891_94).abs() < 1e-4);
}

#[test]
fn elements_assemble_the_five_mean_values() {
    let e = mean_lunar_elements_of_date(J2000);
    assert_eq!(e.node_deg, mean_lunar_node_longitude_of_date(J2000));
    assert_eq!(e.peri_lon_deg, mean_lunar_perigee_longitude_of_date(J2000));
    assert_eq!(e.incl_deg, MOON_MEAN_INCLINATION_DEG);
    assert_eq!(e.eccentricity, MOON_MEAN_ECCENTRICITY);
    assert_eq!(e.semi_major_au, MOON_MEAN_SEMI_MAJOR_AU);
}

#[test]
fn projected_mean_apogee_at_j2000_is_the_swiss_ephemeris_point() {
    // SE mean apogee at J2000 (nod-aps corpus row): 263.464250479°, +3.419723161°
    // in the true equinox of date; Δψ(J2000) ≈ −0.003868°, so the mean-equinox
    // value is ≈ 263.46812°. The raw element (perigee + 180°) is 263.3532°:
    // this test fails if the point is not projected through the inclined orbit.
    let points = points_from_elements(&mean_lunar_elements_of_date(J2000), false).unwrap();
    assert!((points.aphelion.longitude_deg - 263.468_12).abs() < 1e-4);
    assert!((points.aphelion.latitude_deg - 3.419_722).abs() < 1e-5);
    assert!((points.aphelion.distance_au - 0.002_710_625).abs() < 1e-9);
    assert!((points.perihelion.latitude_deg + 3.419_722).abs() < 1e-5);
    assert!((points.ascending.latitude_deg).abs() < 1e-12);
}

#[test]
fn non_finite_input_does_not_panic() {
    assert!(mean_lunar_node_longitude_of_date(f64::NAN).is_nan());
    assert!(mean_lunar_perigee_longitude_of_date(f64::INFINITY).is_nan());
}

proptest! {
    #[test]
    fn longitudes_stay_normalized(jd in 0.0_f64..5_000_000.0) {
        let node = mean_lunar_node_longitude_of_date(jd);
        let peri = mean_lunar_perigee_longitude_of_date(jd);
        prop_assert!((0.0..360.0).contains(&node), "node {node}");
        prop_assert!((0.0..360.0).contains(&peri), "perigee {peri}");
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Add `mod mean;` to `lib.rs` temporarily with an empty `mean.rs` containing only `#[cfg(test)] mod tests;`.
Run: `cargo nextest run -p pleiades-apsides mean`
Expected: compile error, `cannot find function mean_lunar_node_longitude_of_date`.

- [ ] **Step 4: Write the implementation**

`crates/pleiades-apsides/src/mean.rs`:

```rust
//! Mean lunar orbit: the Moon's mean node and mean perigee as slowly varying
//! orbital elements, and the constant mean shape of the orbit.
//!
//! The two longitudes are the Meeus (Astronomical Algorithms, ch. 47)
//! polynomials, referred to the **mean equinox and ecliptic of date**. They are
//! *elements*, not points on the sky: the perigee longitude is the node
//! longitude plus the argument of perigee measured in the orbit plane. To get
//! the point Swiss Ephemeris reports as `SE_MEAN_APOG`, pass
//! [`mean_lunar_elements_of_date`] to [`crate::points_from_elements`], which
//! places the apsis on the inclined orbit (up to about 7′ in longitude and
//! 5.1° in latitude away from the raw element).

use crate::KeplerianElements;

const J2000_JD: f64 = 2_451_545.0;
const DAYS_PER_JULIAN_CENTURY: f64 = 36_525.0;
const METERS_PER_AU: f64 = 1.495_978_707_00e11;

/// Mean inclination of the lunar orbit to the ecliptic, degrees (Swiss
/// Ephemeris `MOON_MEAN_INCL`).
pub const MOON_MEAN_INCLINATION_DEG: f64 = 5.145_396_4;
/// Mean eccentricity of the lunar orbit (Swiss Ephemeris `MOON_MEAN_ECC`).
pub const MOON_MEAN_ECCENTRICITY: f64 = 0.054_900_489;
/// Mean semi-major axis of the lunar orbit, AU (384 400 km, Swiss Ephemeris
/// `MOON_MEAN_DIST`).
pub const MOON_MEAN_SEMI_MAJOR_AU: f64 = 384_400_000.0 / METERS_PER_AU;

fn julian_centuries(jd_tt: f64) -> f64 {
    (jd_tt - J2000_JD) / DAYS_PER_JULIAN_CENTURY
}

/// Mean longitude of the Moon's ascending node, degrees in `[0, 360)`, mean
/// equinox of date, at the TT Julian day `jd_tt`. Non-finite input yields NaN.
pub fn mean_lunar_node_longitude_of_date(jd_tt: f64) -> f64 {
    let t = julian_centuries(jd_tt);
    (125.044_547_9
        + (-1_934.136_289_1 + (0.002_075_4 + (1.0 / 476_441.0 - t / 60_616_000.0) * t) * t) * t)
        .rem_euclid(360.0)
}

/// Mean longitude of the Moon's perigee (node longitude plus in-plane argument
/// of perigee), degrees in `[0, 360)`, mean equinox of date, at the TT Julian
/// day `jd_tt`. Non-finite input yields NaN.
pub fn mean_lunar_perigee_longitude_of_date(jd_tt: f64) -> f64 {
    let t = julian_centuries(jd_tt);
    (83.353_246_5
        + (4_069.013_728_7 + (-0.010_32 + (-1.0 / 80_053.0 + t / 18_999_000.0) * t) * t) * t)
        .rem_euclid(360.0)
}

/// The Moon's mean Keplerian elements in the mean ecliptic of date at the TT
/// Julian day `jd_tt`, ready for [`crate::points_from_elements`].
pub fn mean_lunar_elements_of_date(jd_tt: f64) -> KeplerianElements {
    KeplerianElements {
        node_deg: mean_lunar_node_longitude_of_date(jd_tt),
        peri_lon_deg: mean_lunar_perigee_longitude_of_date(jd_tt),
        incl_deg: MOON_MEAN_INCLINATION_DEG,
        eccentricity: MOON_MEAN_ECCENTRICITY,
        semi_major_au: MOON_MEAN_SEMI_MAJOR_AU,
    }
}

#[cfg(test)]
mod tests;
```

In `lib.rs`, after `#![deny(missing_docs)]`:

```rust
mod mean;

pub use mean::{
    mean_lunar_elements_of_date, mean_lunar_node_longitude_of_date,
    mean_lunar_perigee_longitude_of_date, MOON_MEAN_ECCENTRICITY, MOON_MEAN_INCLINATION_DEG,
    MOON_MEAN_SEMI_MAJOR_AU,
};
```

Note on `rem_euclid`: for a tiny negative input it can return exactly `360.0`. If `longitudes_stay_normalized` finds such a case, wrap both returns as `let d = (...).rem_euclid(360.0); if d >= 360.0 { 0.0 } else { d }` and keep the regression seed proptest writes.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo nextest run -p pleiades-apsides`
Expected: all pass (existing tests untouched).

- [ ] **Step 6: Widen the crate description and README**

`Cargo.toml` line 3:

```toml
description = "Lunar orbit points (mean and osculating nodes and apsides) for the pleiades astrology workspace"
```

Prepend to the `lib.rs` module doc a first paragraph, keeping the existing text below it:

```rust
//! Lunar orbit points for the pleiades workspace: the osculating (true) apsides
//! and node from a state vector, and the mean node and apsides from the mean
//! lunar elements (see [`mean_lunar_elements_of_date`]).
//!
```

Append to `README.md`:

```markdown

The crate also owns the **mean** lunar orbit: the Meeus mean node and mean
perigee longitudes (mean equinox of date) and the Moon's mean inclination,
eccentricity and semi-major axis, assembled by `mean_lunar_elements_of_date`.
Passing those elements to `points_from_elements` gives the mean node and the
mean apogee/perigee as Swiss Ephemeris defines them (`SE_MEAN_NODE`,
`SE_MEAN_APOG`); `pleiades-data` serves them that way, gated by
`validate-mean-lunar-points`.
```

Change the README's first sentence from "Osculating lunar apsides (true apogee / true perigee)" to "Lunar orbit points (osculating and mean nodes and apsides)".

- [ ] **Step 7: Lint and commit**

Run: `cargo fmt --all && cargo clippy -p pleiades-apsides --all-targets -- -D warnings && RUSTDOCFLAGS="-D warnings" cargo doc -p pleiades-apsides --no-deps`
Expected: clean.

```bash
git add crates/pleiades-apsides Cargo.lock
git commit -m "feat(apsides): own the mean lunar orbit elements (#90)"
```

---

### Task 2: `pleiades-elp` delegates its mean polynomials

**Files:**
- Modify: `crates/pleiades-elp/Cargo.toml` (`[dependencies]`)
- Modify: `crates/pleiades-elp/src/backend.rs` (`mean_node_longitude`, `mean_perigee_longitude`, about lines 129–146)

**Interfaces:**
- Consumes: `pleiades_apsides::{mean_lunar_node_longitude_of_date, mean_lunar_perigee_longitude_of_date}` (Task 1).
- Produces: nothing new; ELP behaviour is unchanged.

- [ ] **Step 1: Add the dependency**

In `crates/pleiades-elp/Cargo.toml` `[dependencies]`, keeping alphabetical order:

```toml
pleiades-apparent = { workspace = true }
pleiades-apsides = { workspace = true }
pleiades-backend = { workspace = true }
pleiades-types = { workspace = true }
```

- [ ] **Step 2: Replace the two polynomial bodies**

```rust
    fn mean_node_longitude(days: f64) -> f64 {
        pleiades_apsides::mean_lunar_node_longitude_of_date(crate::J2000 + days)
    }

    fn mean_perigee_longitude(days: f64) -> f64 {
        pleiades_apsides::mean_lunar_perigee_longitude_of_date(crate::J2000 + days)
    }
```

Leave `mean_apogee_longitude` and `true_node_longitude` as they are (the latter still calls `Self::mean_node_longitude`).

- [ ] **Step 3: Add the honesty caveat to the mean apsis channel**

Above `fn mean_apogee_longitude`, add:

```rust
    /// The mean apogee *element*: mean perigee longitude + 180°, emitted with
    /// latitude 0. This is **not** the point Swiss Ephemeris reports as
    /// `SE_MEAN_APOG`, which lies on the inclined mean orbit (up to about 7′
    /// in longitude and 5.1° in latitude away). The routed chart chain serves
    /// the Swiss Ephemeris point from `PackagedDataBackend`; this channel
    /// remains for direct ELP consumers as a documented approximation (#90).
```

- [ ] **Step 4: Run the ELP tests**

Run: `cargo nextest run -p pleiades-elp`
Expected: all pass with no test edits. `J2000 + days` then `- J2000` inside the callee can differ from the old `days / 36525` at the 1e-10° level; if an evidence test pinned to tighter than that fails, stop and report the test name and delta rather than loosening it.

- [ ] **Step 5: Lint and commit**

Run: `cargo fmt --all && cargo clippy -p pleiades-elp --all-targets -- -D warnings`

```bash
git add crates/pleiades-elp Cargo.lock
git commit -m "refactor(elp): take the mean lunar polynomials from pleiades-apsides (#90)"
```

---

### Task 3: `nod_aps` Mean for the Moon needs no backend channel

**Files:**
- Modify: `crates/pleiades-events/src/nod_aps.rs` (imports at 188–191; `mean_points_at` Moon branch at about 393–417; `mod tests` near line 65)
- Modify: `crates/pleiades-events/src/mean_elements.rs:89-91` (remove three constants)
- Modify: `crates/pleiades-events/src/ephemeris.rs:56-82` (remove `read_mean_longitude_of_date`)
- Test: `crates/pleiades-events/tests/nod_aps.rs`

**Interfaces:**
- Consumes: `pleiades_apsides::mean_lunar_elements_of_date(jd_tt: f64) -> KeplerianElements` (Task 1).
- Produces: `EventEngine::nod_aps(CelestialBody::Moon, _, NodApsMethod::Mean, _)` succeeds on any backend.

- [ ] **Step 1: Write the failing tests**

Append to `crates/pleiades-events/tests/nod_aps.rs`:

```rust
/// Issue #90: the mean lunar node and apsides are analytic, so the Mean method
/// for the Moon must work on the packaged backend alone.
#[test]
fn moon_mean_points_work_on_the_packaged_backend_alone() {
    let engine = EventEngine::new(PackagedDataBackend::new());
    let r = engine
        .nod_aps(
            CelestialBody::Moon,
            tdb(2_460_000.5),
            NodApsMethod::Mean,
            ApsisConvention::Aphelion,
        )
        .expect("Moon mean nod_aps on the packaged backend");
    assert!(r.ascending.longitude_deg.is_finite());

    // Swiss Ephemeris swe_nod_aps, Moon, SE_NODBIT_MEAN at J2000 (nod-aps corpus).
    let j2000 = engine
        .nod_aps(
            CelestialBody::Moon,
            tdb(JD),
            NodApsMethod::Mean,
            ApsisConvention::Aphelion,
        )
        .unwrap();
    let arcsec = |got: f64, want: f64| ((got - want + 180.0).rem_euclid(360.0) - 180.0).abs() * 3600.0;
    assert!(arcsec(j2000.ascending.longitude_deg, 125.040_685_175) < 0.8);
    assert!(arcsec(j2000.aphelion.longitude_deg, 263.464_250_479) < 0.8);
    assert!(((j2000.aphelion.latitude_deg - 3.419_723_161) * 3600.0).abs() < 0.06);
}
```

In `crates/pleiades-events/src/nod_aps.rs` `mod tests`, add:

```rust
    #[test]
    fn moon_mean_method_reads_no_lunar_point_channel() {
        // LinearSunMoon serves only the Sun and Moon. Before #90 the Mean
        // method read MeanNode/MeanPerigee from the backend and failed here.
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_545.0));
        let r = engine
            .nod_aps(
                CelestialBody::Moon,
                tdb(2_451_545.0),
                NodApsMethod::Mean,
                ApsisConvention::Aphelion,
            )
            .expect("mean lunar elements are analytic");
        assert!((r.ascending.latitude_deg).abs() < 1e-9);
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo nextest run -p pleiades-events moon_mean`
Expected: both FAIL with `backend error: ... MeanNode` (wording differs per backend).

- [ ] **Step 3: Implement**

In `mean_points_at`, replace the Moon branch (the comment block, the two `read_mean_longitude_of_date` calls and the `KeplerianElements { .. }` literal) with:

```rust
        let elements = if *body == CelestialBody::Moon {
            // The Moon's mean elements are analytic (Meeus mean node and
            // perigee, SE's mean inclination/eccentricity/distance), already
            // in the mean ecliptic of date like SE's mean lunar elements.
            mean_lunar_elements_of_date(jd)
        } else {
```

Update the imports:

```rust
use crate::ephemeris::{read_mean_ecliptic, spherical_to_cartesian};
use crate::mean_elements::{
    elem_index, mean_elements_of_date, mu_au3_day2, EARTH_MOON_MASS_RATIO,
};
```

and add `mean_lunar_elements_of_date` to the existing `use pleiades_apsides::{...}` list in that file (`grep -n "use pleiades_apsides" crates/pleiades-events/src/nod_aps.rs`). If `KeplerianElements` becomes unused in the import list, remove it.

Delete `MOON_MEAN_INCL_DEG`, `MOON_MEAN_ECC`, `MOON_MEAN_SEMA_AU` from `mean_elements.rs`. Keep `AUNIT_M` (the GM constants use it).

Delete `read_mean_longitude_of_date` and its doc comment from `ephemeris.rs`; then remove `precess_ecliptic_j2000_to_date` from that file's imports only if the compiler reports it unused.

Update the `nod_aps` rustdoc wherever it says the Moon's mean points come from a backend's `MeanNode`/`MeanPerigee` channels (`grep -n "MeanNode\|MeanPerigee\|ELP" crates/pleiades-events/src/nod_aps.rs`): state that the Moon's mean elements are analytic and need no backend channel.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo nextest run -p pleiades-events`
Expected: all pass, including the two new tests.

- [ ] **Step 5: Re-measure the nod-aps gate**

Run: `cargo run -q -p pleiades-validate -- validate-nod-aps`
Expected: PASS. Read the `MEAN_MOON` longitude maximum from the summary. Update the two "0.561″" mentions in `crates/pleiades-validate/src/nod_aps_thresholds.rs` (lines 25 and ~39) to the new measured value only if it changed at the printed precision. Do not change `MEAN_MOON_LONGITUDE_ARCSEC`. If the gate fails, stop and report.

- [ ] **Step 6: Lint and commit**

Run: `cargo fmt --all && cargo clippy -p pleiades-events -p pleiades-validate --all-targets --all-features -- -D warnings`

```bash
git add crates/pleiades-events crates/pleiades-validate/src/nod_aps_thresholds.rs
git commit -m "fix(events): nod_aps Mean for the Moon no longer reads backend lunar-point channels (#90)"
```

---

### Task 4: Swiss Ephemeris reference tool and corpus

**Files:**
- Create: `tools/se-mean-lunar-reference/Cargo.toml`
- Create: `tools/se-mean-lunar-reference/src/main.rs`
- Create: `crates/pleiades-validate/data/mean-lunar-corpus/mean-lunar.csv` (generated)
- Create: `crates/pleiades-validate/data/mean-lunar-corpus/manifest.txt`

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces: CSV with header `jd_tt,se_mean_node_lon_deg,se_mean_node_lat_deg,se_mean_node_dist_au,se_mean_apog_lon_deg,se_mean_apog_lat_deg,se_mean_apog_dist_au`, 3177 data rows; manifest line `slice mean-lunar file=mean-lunar.csv role=mean-lunar rows=3177 checksum=<fnv1a64>`; and a recorded **node distance convention** (Step 4) that Task 5 reads.

- [ ] **Step 1: Write the tool**

`tools/se-mean-lunar-reference/Cargo.toml`:

```toml
[package]
name = "se-mean-lunar-reference"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
swisseph = "0.1.1"
libswisseph-sys = "0.1.2"
```

`tools/se-mean-lunar-reference/src/main.rs`:

```rust
//! Emits a Swiss Ephemeris reference corpus for the mean lunar node
//! (`SE_MEAN_NODE`) and mean lunar apogee (`SE_MEAN_APOG`, "mean Lilith") to
//! STDOUT as CSV.
//!
//! Frame: true ecliptic of date, nutation on (SE default — no SEFLG_NONUT,
//! no SEFLG_J2000). Ephemeris flag: Moshier (SEFLG_MOSEPH) — no data files
//! needed; both bodies are analytic mean elements in Swiss Ephemeris.
//! Same deterministic grid as `tools/se-true-node-reference`.
//!
//! Build inside `devenv shell` (provides clang/libclang/LIBCLANG_PATH):
//! `devenv shell -- cargo run --release --manifest-path tools/se-mean-lunar-reference/Cargo.toml \
//!    > crates/pleiades-validate/data/mean-lunar-corpus/mean-lunar.csv`

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::swe_calc;

const SE_MEAN_NODE: c_int = 10;
const SE_MEAN_APOG: c_int = 12;
const SEFLG_MOSEPH: c_int = 4;

const JD_START_TT: f64 = 2_415_020.5; // 1900-01-01
const JD_END_TT: f64 = 2_488_070.0; //   ~2100-01-01
const STEP_DAYS: f64 = 23.0;

fn se_point(jd_tt: f64, body: c_int, name: &str) -> (f64, f64, f64) {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(
            jd_tt,
            body,
            SEFLG_MOSEPH,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        let msg = unsafe { CStr::from_ptr(serr.as_ptr() as *const c_char) }
            .to_string_lossy()
            .into_owned();
        panic!("swe_calc({name}) failed at jd_tt={jd_tt}: {msg}");
    }
    let (lon, lat, dist) = (xx[0], xx[1], xx[2]);
    assert!(
        lon.is_finite() && lat.is_finite() && dist.is_finite(),
        "non-finite SE {name} result at jd_tt={jd_tt}"
    );
    (lon.rem_euclid(360.0), lat, dist)
}

fn main() {
    println!(
        "# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), swe_calc SE_MEAN_NODE=10 and SE_MEAN_APOG=12,"
    );
    println!(
        "# iflag=SEFLG_MOSEPH (Moshier, no data files). Frame: true ecliptic of date, nutation on."
    );
    println!(
        "# Columns: of-date true ecliptic longitude/latitude (deg) and geocentric distance (AU), node then apogee."
    );
    println!("jd_tt,se_mean_node_lon_deg,se_mean_node_lat_deg,se_mean_node_dist_au,se_mean_apog_lon_deg,se_mean_apog_lat_deg,se_mean_apog_dist_au");
    let mut jd = JD_START_TT;
    while jd <= JD_END_TT {
        let (nl, nb, nd) = se_point(jd, SE_MEAN_NODE, "SE_MEAN_NODE");
        let (al, ab, ad) = se_point(jd, SE_MEAN_APOG, "SE_MEAN_APOG");
        println!("{jd:.1},{nl:.9},{nb:.9},{nd:.12},{al:.9},{ab:.9},{ad:.12}");
        jd += STEP_DAYS;
    }
}
```

The `unsafe` here is the same FFI call the sibling tools make, in an unpublished out-of-workspace tool; it is not library code.

- [ ] **Step 2: Generate the corpus**

```bash
mkdir -p crates/pleiades-validate/data/mean-lunar-corpus
devenv shell -- cargo run --release --manifest-path tools/se-mean-lunar-reference/Cargo.toml > crates/pleiades-validate/data/mean-lunar-corpus/mean-lunar.csv
```

Expected: exit 0. If the build fails for lack of `libclang` even inside `devenv shell`, this is a **stop condition** — report it; do not hand-write corpus values.

- [ ] **Step 3: Verify the corpus**

```bash
grep -vc '^#\|^jd_tt' crates/pleiades-validate/data/mean-lunar-corpus/mean-lunar.csv
grep '^2451546.5\|^2415020.5' crates/pleiades-validate/data/mean-lunar-corpus/mean-lunar.csv
head -5 crates/pleiades-validate/data/mean-lunar-corpus/mean-lunar.csv
```

Expected: `3177`; the first data row is `2415020.5,...`; node latitude column is `0.000000000` on every row (`awk -F, '!/^#|^jd/ && $3+0!=0' ... | wc -l` prints `0`); apogee latitude magnitudes are at most about 5.15.

Sanity-check against the polynomials: for the first row, the node longitude should be within about 0.01° of `mean_lunar_node_longitude_of_date(2415020.5)` (the difference is Δψ, at most ±0.005°, plus sub-arcsecond model difference). Compute it with:

```bash
python3 -c "t=(2415020.5-2451545.0)/36525; print((125.0445479+(-1934.1362891+(0.0020754+(1/476441-t/60616000)*t)*t)*t)%360)"
```

If the difference exceeds 0.02°, stop and report.

- [ ] **Step 4: Record the node distance convention**

```bash
awk -F, '!/^#|^jd/ {print $4}' crates/pleiades-validate/data/mean-lunar-corpus/mean-lunar.csv | sort -u | head -5
awk -F, '!/^#|^jd/ {print $7}' crates/pleiades-validate/data/mean-lunar-corpus/mean-lunar.csv | sort -u | head -5
```

Expected (spec section 3): the node column is the single value `0.002569555290` (the mean distance `a`), and the apogee column is the single value `0.002710625132` (`a(1+e)`).

Write the outcome into the task report and into the commit message body as one of:
- `node-distance: constant-a` (the expected case), or
- `node-distance: varies` (the column has many values — then compare a few rows with `points_from_elements(..).ascending.distance_au`), or
- anything else: **stop**, the spec must be amended before Task 5.

If the apogee column is not constant `a(1+e)` to 1e-9, stop and report.

- [ ] **Step 5: Write the manifest**

Compute the checksum with the workspace's own function, in a scratch test so nothing is committed. Add temporarily to the bottom of `crates/pleiades-validate/src/true_node_validation.rs` `mod tests`:

```rust
    #[test]
    fn print_mean_lunar_checksum() {
        let csv = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/mean-lunar-corpus/mean-lunar.csv"
        ));
        eprintln!("CHECKSUM={}", fnv1a64(csv));
    }
```

Run: `cargo nextest run -p pleiades-validate print_mean_lunar_checksum --no-capture 2>&1 | grep CHECKSUM`
Then **remove the scratch test** (`git diff --stat crates/pleiades-validate/src/true_node_validation.rs` must print nothing afterwards).

`crates/pleiades-validate/data/mean-lunar-corpus/manifest.txt` (one line, with the printed number):

```
slice mean-lunar file=mean-lunar.csv role=mean-lunar rows=3177 checksum=<printed value>
```

- [ ] **Step 6: Commit**

`tools/se-mean-lunar-reference/target` is ignored by the root `.gitignore` (`target`). Commit the tool's `Cargo.lock` as the sibling tools do.

```bash
git add tools/se-mean-lunar-reference/Cargo.toml tools/se-mean-lunar-reference/Cargo.lock tools/se-mean-lunar-reference/src crates/pleiades-validate/data/mean-lunar-corpus
git status --short   # nothing else staged or modified
git commit -m "test(validate): Swiss Ephemeris SE_MEAN_NODE / SE_MEAN_APOG reference corpus (#90)" -m "node-distance: <outcome from Step 4>"
```

---

### Task 5: `PackagedDataBackend` serves the mean lunar points

**Files:**
- Modify: `crates/pleiades-data/src/backend.rs` (imports at 9–11; new method after `osculating_node_ecliptic`; `supports_body` at ~349; `position` at ~381)
- Test: `crates/pleiades-data/src/tests/lookup.rs` (append after the TrueNode tests, ~line 1465)

**Interfaces:**
- Consumes: `pleiades_apsides::{mean_lunar_elements_of_date, points_from_elements, MOON_MEAN_SEMI_MAJOR_AU}` (Task 1); the node distance convention recorded in Task 4's commit message (`git log -1 --format=%b --grep "node-distance"`).
- Produces: `PackagedDataBackend::position` answers `MeanNode`, `MeanApogee`, `MeanPerigee` with J2000 ecliptic (longitude, latitude, `Some(distance)`), mean-obliquity equatorial, and central-difference motion. No claims yet (Task 7 adds them with the gate).

- [ ] **Step 1: Write the failing tests**

Append to `crates/pleiades-data/src/tests/lookup.rs`:

```rust
const MEAN_LUNAR_POINTS: [CelestialBody; 3] = [
    CelestialBody::MeanNode,
    CelestialBody::MeanApogee,
    CelestialBody::MeanPerigee,
];

fn mean_point_of_date(body: CelestialBody, jd: f64) -> (f64, f64, f64) {
    use pleiades_backend::{EphemerisBackend, EphemerisRequest};
    let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
    let ecl = PackagedDataBackend::new()
        .position(&EphemerisRequest::new(body, instant))
        .expect("mean lunar point position")
        .ecliptic
        .expect("ecliptic present");
    let of_date = pleiades_apparent::precess_ecliptic_j2000_to_date(
        ecl.longitude.degrees(),
        ecl.latitude.degrees(),
        jd,
    )
    .unwrap();
    (
        of_date.longitude_deg,
        of_date.latitude_deg,
        ecl.distance_au.expect("distance present"),
    )
}

#[test]
fn packaged_backend_supports_the_mean_lunar_points() {
    use pleiades_backend::EphemerisBackend;
    let backend = PackagedDataBackend::new();
    for body in MEAN_LUNAR_POINTS {
        assert!(backend.supports_body(body.clone()), "{body:?}");
    }
}

#[test]
fn mean_node_is_the_meeus_polynomial_on_the_ecliptic_of_date() {
    let jd = 2_461_041.5; // 2026-01-01 TT
    let (lon, lat, _) = mean_point_of_date(CelestialBody::MeanNode, jd);
    let want = pleiades_apsides::mean_lunar_node_longitude_of_date(jd);
    assert!((lon - want).abs() < 1e-8, "lon {lon} want {want}");
    assert!(lat.abs() < 1e-8, "node latitude of date {lat}");
}

#[test]
fn mean_apogee_at_j2000_is_the_swiss_ephemeris_point() {
    // SE mean apogee at J2000 in the true equinox of date is 263.464250479°,
    // +3.419723161°; without Δψ (−0.003868°) the mean-equinox longitude is
    // ≈ 263.46812°. The raw Meeus element would be 263.3532°, latitude 0.
    let (lon, lat, dist) = mean_point_of_date(CelestialBody::MeanApogee, 2_451_545.0);
    assert!((lon - 263.468_12).abs() < 1e-4, "lon {lon}");
    assert!((lat - 3.419_722).abs() < 1e-5, "lat {lat}");
    assert!((dist - 0.002_710_625).abs() < 1e-9, "dist {dist}");
}

#[test]
fn mean_perigee_is_antipodal_to_mean_apogee() {
    let jd = 2_461_041.5;
    let (alon, alat, adist) = mean_point_of_date(CelestialBody::MeanApogee, jd);
    let (plon, plat, pdist) = mean_point_of_date(CelestialBody::MeanPerigee, jd);
    let dlon = (plon - alon).rem_euclid(360.0);
    assert!((dlon - 180.0).abs() < 1e-8, "Δλ {dlon}");
    assert!((plat + alat).abs() < 1e-8, "β {plat} vs {alat}");
    assert!((pdist - 0.002_428_485).abs() < 1e-9, "perigee dist {pdist}");
    assert!(adist > pdist);
}

#[test]
fn mean_lunar_point_motion_has_the_expected_rates() {
    use pleiades_backend::{EphemerisBackend, EphemerisRequest};
    let backend = PackagedDataBackend::new();
    let instant = Instant::new(JulianDay::from_days(2_461_041.5), TimeScale::Tt);
    let rate = |body: CelestialBody| {
        backend
            .position(&EphemerisRequest::new(body, instant))
            .unwrap()
            .motion
            .expect("motion")
            .longitude_deg_per_day
            .expect("longitude rate")
    };
    // J2000-frame rates: of-date rate minus general precession (3.8e-5°/day).
    assert!((rate(CelestialBody::MeanNode) + 0.052_99).abs() < 2e-4);
    // The projected apsis oscillates around the element's 0.1114°/day.
    let apogee = rate(CelestialBody::MeanApogee);
    assert!((0.110..0.113).contains(&apogee), "apogee rate {apogee}");
}

#[test]
fn mean_lunar_points_accept_tdb_like_tt() {
    use pleiades_backend::{EphemerisBackend, EphemerisRequest};
    let backend = PackagedDataBackend::new();
    let jd = 2_461_041.5;
    for body in MEAN_LUNAR_POINTS {
        let lon = |scale| {
            backend
                .position(&EphemerisRequest::new(
                    body.clone(),
                    Instant::new(JulianDay::from_days(jd), scale),
                ))
                .expect("TT and TDB requests are accepted")
                .ecliptic
                .unwrap()
                .longitude
                .degrees()
        };
        assert_eq!(lon(TimeScale::Tt), lon(TimeScale::Tdb), "{body:?}");
    }
}

#[test]
fn mean_lunar_points_fail_closed_outside_window() {
    use pleiades_backend::{EphemerisBackend, EphemerisErrorKind, EphemerisRequest};
    let backend = PackagedDataBackend::new();
    for body in MEAN_LUNAR_POINTS {
        let err = backend
            .position(&EphemerisRequest::new(
                body.clone(),
                Instant::new(JulianDay::from_days(2_400_000.5), TimeScale::Tt),
            ))
            .expect_err("1858 is outside the packaged window");
        assert_eq!(err.kind, EphemerisErrorKind::OutOfRangeInstant, "{body:?}");
    }
}

#[test]
fn mean_lunar_point_motion_degrades_gracefully_at_coverage_boundary() {
    use pleiades_backend::{EphemerisBackend, EphemerisRequest};
    let backend = PackagedDataBackend::new();
    let boundary = Instant::new(JulianDay::from_days(2_415_020.5), TimeScale::Tt);
    let result = backend
        .position(&EphemerisRequest::new(CelestialBody::MeanNode, boundary))
        .expect("position at coverage boundary must succeed");
    assert!(result.ecliptic.unwrap().longitude.degrees().is_finite());
    let motion = result.motion.expect("motion field present");
    assert!(motion.longitude_deg_per_day.is_none());
    assert!(motion.latitude_deg_per_day.is_none());
    assert!(motion.distance_au_per_day.is_none());
}
```

If Task 4 recorded `node-distance: constant-a`, also add:

```rust
#[test]
fn mean_node_reports_the_mean_lunar_distance() {
    // Swiss Ephemeris reports the mean distance for SE_MEAN_NODE.
    let (_, _, dist) = mean_point_of_date(CelestialBody::MeanNode, 2_461_041.5);
    assert_eq!(dist, pleiades_apsides::MOON_MEAN_SEMI_MAJOR_AU);
}
```

If it recorded `node-distance: varies`, add instead:

```rust
#[test]
fn mean_node_reports_the_orbit_radius_at_the_node() {
    let jd = 2_461_041.5;
    let (_, _, dist) = mean_point_of_date(CelestialBody::MeanNode, jd);
    let want = pleiades_apsides::points_from_elements(
        &pleiades_apsides::mean_lunar_elements_of_date(jd),
        false,
    )
    .unwrap()
    .ascending
    .distance_au;
    assert_eq!(dist, want);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo nextest run -p pleiades-data mean_`
Expected: FAIL — `supports_body` false, positions `UnsupportedBody`.

- [ ] **Step 3: Implement**

Imports in `backend.rs`:

```rust
use pleiades_apsides::{
    apsides, elements_from_state, mean_lunar_elements_of_date, points_from_elements,
    MOON_MEAN_SEMI_MAJOR_AU, MU_EARTH_MOON_AU3_PER_DAY2,
};
```

(Drop `MOON_MEAN_SEMI_MAJOR_AU` from the import in the `node-distance: varies` case.)

New method after `osculating_node_ecliptic`:

```rust
    /// A mean lunar point (mean node, mean apogee or mean perigee), J2000
    /// boundary frame.
    ///
    /// The point is placed on the Moon's mean orbit in the mean ecliptic of
    /// date — the apsides through the inclined orbit, as Swiss Ephemeris'
    /// `SE_MEAN_APOG` is, not as the raw longitude-of-perigee element — and
    /// precessed back to J2000 like the osculating node. The elements are
    /// analytic, but the point is served only inside the packaged window so
    /// the backend's advertised range holds for every body it answers.
    fn mean_lunar_point_ecliptic(
        &self,
        body: &CelestialBody,
        instant: Instant,
    ) -> Result<EclipticCoordinates, EphemerisError> {
        // Window probe: the Moon series spans the packaged window.
        self.artifact
            .lookup_ecliptic(&CelestialBody::Moon, normalize_lookup_instant(instant))
            .map_err(map_artifact_error)?;

        let jd_tt = instant.julian_day.days();
        let points =
            points_from_elements(&mean_lunar_elements_of_date(jd_tt), false).map_err(|_| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "mean lunar point undefined at this instant",
                )
            })?;
        let (point, distance_au) = match body {
            // Swiss Ephemeris reports the mean lunar distance for SE_MEAN_NODE,
            // not the orbit radius at the node.
            CelestialBody::MeanNode => (points.ascending, MOON_MEAN_SEMI_MAJOR_AU),
            CelestialBody::MeanApogee => (points.aphelion, points.aphelion.distance_au),
            CelestialBody::MeanPerigee => (points.perihelion, points.perihelion.distance_au),
            _ => {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "not a mean lunar point",
                ))
            }
        };
        let j2000 = precess_ecliptic_date_to_j2000(point.longitude_deg, point.latitude_deg, jd_tt)
            .map_err(map_precession_error)?;
        Ok(EclipticCoordinates::new(
            Longitude::from_degrees(j2000.longitude_deg),
            Latitude::from_degrees(j2000.latitude_deg),
            Some(distance_au),
        ))
    }
```

In the `node-distance: varies` case the `MeanNode` arm is `(points.ascending, points.ascending.distance_au)` and its comment reads "Swiss Ephemeris reports the orbit radius at the node."

`supports_body`:

```rust
        matches!(
            body,
            CelestialBody::TrueApogee
                | CelestialBody::TruePerigee
                | CelestialBody::TrueNode
                | CelestialBody::MeanNode
                | CelestialBody::MeanApogee
                | CelestialBody::MeanPerigee
        ) || self
```

`position`, after the `TrueNode` branch:

```rust
        if matches!(
            req.body,
            CelestialBody::MeanNode | CelestialBody::MeanApogee | CelestialBody::MeanPerigee
        ) {
            let body = req.body.clone();
            return self.derived_point_position(req, &|i| self.mean_lunar_point_ecliptic(&body, i));
        }
```

Update the doc comment on `derived_point_position` from "(osculating apsis or node)" to "(osculating or mean apsis or node)".

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo nextest run -p pleiades-data`
Expected: all pass. If `mean_lunar_point_motion_has_the_expected_rates` is off, print the two rates and compare with the comment's reasoning before touching the bounds; a wrong sign means the point selection is wrong, not the test.

- [ ] **Step 5: Run the dependent crates**

Run: `cargo nextest run -p pleiades-events -p pleiades-core -p pleiades-cli`
Expected: pass. A failure here is most likely a test that pinned which backend serves `MeanNode`/`MeanApogee` in a routed chain, or an ELP-valued Mean Apogee expectation. For each: if it asserts the *old ELP element value* for a routed chart, update the expectation to the packaged value and say so in the commit body; if it is anything else, stop and report.

- [ ] **Step 6: Lint and commit**

Run: `cargo fmt --all && cargo clippy -p pleiades-data --all-targets --all-features -- -D warnings`

```bash
git add crates/pleiades-data crates/pleiades-events crates/pleiades-core crates/pleiades-cli
git commit -m "feat(data): serve the mean lunar node, apogee and perigee from the packaged backend (#90)"
```

---

### Task 6: The chart treats every lunar point as a geometric direction

**Files:**
- Modify: `crates/pleiades-core/src/chart/mod.rs:476-482` (topocentric exemption) and `:615-625` (apparent-place arm)
- Test: `crates/pleiades-core/src/chart/tests.rs` (after `topocentric_chart_leaves_derived_lunar_points_geocentric`)

**Interfaces:**
- Consumes: packaged mean points (Task 5); `CelestialBody::class() -> CelestialBodyClass` and `CelestialBodyClass::LunarPoint` (existing, `pleiades-types`).
- Produces: `fn is_lunar_point(body: &CelestialBody) -> bool` (private to `chart/mod.rs`).

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn chart_serves_mean_lunar_points_precession_nutation_only() {
    use pleiades_backend::{Apparentness, EphemerisBackend, EphemerisRequest};
    use pleiades_data::PackagedDataBackend;
    use pleiades_types::CelestialBody;

    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2_461_041.5),
        TimeScale::Tt,
    );
    let bodies = vec![
        CelestialBody::MeanNode,
        CelestialBody::MeanApogee,
        CelestialBody::MeanPerigee,
    ];
    let request = ChartRequest::new(instant)
        .with_bodies(bodies.clone())
        .with_apparentness(Apparentness::Apparent);
    let backend = PackagedDataBackend::new();
    let snapshot = ChartEngine::new(backend.clone())
        .chart(&request)
        .expect("apparent mean-lunar-point chart should succeed");

    for body in bodies {
        let placement = snapshot
            .placement_for(&body)
            .unwrap_or_else(|| panic!("{body:?} placement missing"));
        let prov = placement
            .apparent
            .as_ref()
            .unwrap_or_else(|| panic!("{body:?} must carry apparent provenance"));
        assert!(!prov.corrections.light_time, "{body:?}: no light-time");
        assert!(
            !prov.corrections.annual_aberration,
            "{body:?}: no annual aberration"
        );

        let mean = backend
            .position(&EphemerisRequest::new(body.clone(), instant))
            .unwrap()
            .ecliptic
            .unwrap();
        let expected = pleiades_apparent::apparent_apsis_position(instant, mean).unwrap();
        let got = placement.position.ecliptic.unwrap();
        assert!(
            (got.longitude.degrees() - expected.ecliptic.longitude.degrees()).abs() < 1e-9,
            "{body:?} longitude"
        );
        assert!(
            (got.latitude.degrees() - expected.ecliptic.latitude.degrees()).abs() < 1e-9,
            "{body:?} latitude"
        );
    }

    // Mean Lilith carries the orbit's latitude (Swiss Ephemeris SE_MEAN_APOG),
    // not the latitude-0 element.
    let apogee = snapshot
        .placement_for(&CelestialBody::MeanApogee)
        .unwrap()
        .position
        .ecliptic
        .unwrap();
    assert!(apogee.latitude.degrees().abs() <= 5.146);
    assert!(apogee.latitude.degrees().abs() > 0.01);
}

#[test]
fn topocentric_chart_leaves_mean_lunar_points_geocentric() {
    use pleiades_backend::Apparentness;
    use pleiades_data::PackagedDataBackend;
    use pleiades_types::CelestialBody;

    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2_461_041.5),
        TimeScale::Tt,
    );
    let observer = pleiades_types::ObserverLocation::new(
        pleiades_types::Latitude::from_degrees(51.5),
        pleiades_types::Longitude::from_degrees(0.0),
        None,
    );
    let bodies = vec![
        CelestialBody::MeanNode,
        CelestialBody::MeanApogee,
        CelestialBody::MeanPerigee,
    ];
    let chart = |topocentric: bool| {
        ChartEngine::new(PackagedDataBackend::new())
            .chart(
                &ChartRequest::new(instant)
                    .with_bodies(bodies.clone())
                    .with_apparentness(Apparentness::Apparent)
                    .with_observer(observer.clone())
                    .with_topocentric(topocentric),
            )
            .expect("chart should succeed")
    };
    let (geo, topo) = (chart(false), chart(true));
    for body in &bodies {
        let g = geo.placement_for(body).unwrap().position.ecliptic.unwrap();
        let t = topo.placement_for(body).unwrap();
        let te = t.position.ecliptic.unwrap();
        assert!(
            (g.longitude.degrees() - te.longitude.degrees()).abs() < 1e-9,
            "{body:?} longitude moved under the topocentric flag"
        );
        assert!(
            (g.latitude.degrees() - te.latitude.degrees()).abs() < 1e-9,
            "{body:?} latitude moved under the topocentric flag"
        );
        assert!(
            !t.apparent.as_ref().unwrap().corrections.diurnal_parallax,
            "{body:?}: no diurnal parallax"
        );
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo nextest run -p pleiades-core mean_lunar`
Expected: FAIL. The topocentric test fails on the apogee (about 1° parallax shift at lunar distance); the first test fails on `light_time` provenance. If either unexpectedly passes, print the provenance and longitudes and confirm why before continuing — do not weaken the assertions.

- [ ] **Step 3: Implement**

Add near the other private helpers in `chart/mod.rs`:

```rust
/// Whether `body` is a lunar orbit point (mean or true node, apogee, perigee).
///
/// These are geometric directions of the lunar orbit, not bodies. The chart
/// rotates them to the true ecliptic of date with precession + nutation in
/// longitude only: no light-time re-query and no annual aberration, and — in a
/// topocentric chart — no diurnal parallax or diurnal aberration, so they stay
/// geocentric. Swiss Ephemeris does the same. Without this exemption a point
/// carrying a lunar-scale distance would be shifted by up to about 1° of
/// parallax (issues #58, #63, #90).
fn is_lunar_point(body: &CelestialBody) -> bool {
    body.class() == pleiades_types::CelestialBodyClass::LunarPoint
}
```

Topocentric exemption:

```rust
                let topocentric_prov = if request.topocentric && !is_lunar_point(body) {
```

Apparent-place arm (keep the Sun arm above it):

```rust
            // Lunar orbit point: a geometric direction (see `is_lunar_point`).
            // observer = None keeps it geocentric.
            body if is_lunar_point(body) => self
                .query_mean_ecliptic(body, instant, zodiac_mode, None)
                .and_then(|point_j2000| {
                    apparent_apsis_position(instant, point_j2000)
                        .map_err(map_apparent_place_error)
                }),
```

Adjust paths (`CelestialBody` vs `pleiades_types::CelestialBody`, whether `CelestialBodyClass` needs importing) to what the file already imports; if `body` is not a reference at the topocentric site, pass `&body`.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo nextest run -p pleiades-core`
Expected: all pass, including the existing TrueNode/TrueApogee tests.

- [ ] **Step 5: Measure the MeanNode change**

The spec expects the charted `MeanNode` to move by far less than 0.01″. Verify with a scratch test added and then removed (do not commit it): chart `MeanNode` apparent at JD 2461041.5 on `git stash`-free code and compare with `apparent_apsis_position` of ELP's `MeanNode` channel — or simpler, run:

Run: `cargo nextest run -p pleiades-cli -p pleiades-validate -E 'test(/chart|mean|node/)'`
Expected: pass. Any failure that pins a charted mean-point longitude tells you the size of the move: record the delta in the commit body. A `MeanNode` delta above 0.01″ is unexpected — stop and report.

- [ ] **Step 6: Lint and commit**

Run: `cargo fmt --all && cargo clippy -p pleiades-core --all-targets --all-features -- -D warnings`

```bash
git add crates/pleiades-core
git commit -m "fix(core): treat mean lunar points as geometric directions in charts (#90, #63)"
```

---

### Task 7: `validate-mean-lunar-points` gate and release-grade claims

**Files:**
- Create: `crates/pleiades-validate/src/mean_lunar_validation.rs`
- Modify: `crates/pleiades-validate/src/lib.rs` (`mod` at ~48, `pub use` at ~236)
- Modify: `crates/pleiades-validate/src/render/cli.rs` (`run_all_numeric_gates` ~107; command arm ~312; help text ~2265)
- Modify: `crates/pleiades-cli/src/cli.rs:803` (passthrough)
- Modify: `crates/pleiades-data/src/lib.rs` (after `true_node_body_claims`, ~245), `crates/pleiades-data/src/backend.rs` (`metadata().body_claims` ~331)
- Test: `crates/pleiades-data/src/tests/lookup.rs`

**Interfaces:**
- Consumes: corpus + manifest (Task 4); packaged mean points (Task 5); `pleiades_apparent::{apparent_apsis_position, fnv1a64}`.
- Produces:
  - `pub fn validate_mean_lunar_points_corpus() -> Result<MeanLunarCorpusReport, MeanLunarCorpusError>`
  - `MeanLunarCorpusReport { rows_validated, rows_skipped_oor, node: ChannelMaxima, apogee: ChannelMaxima, perigee: ChannelMaxima }` with `summary_line()`
  - `pub fn pleiades_data::mean_lunar_point_body_claims() -> Vec<pleiades_backend::BodyClaim>`
  - CLI: `validate-mean-lunar-points`, alias `mean-lunar-points-gate`

- [ ] **Step 1: Write the gate with provisional ceilings and its tests**

`crates/pleiades-validate/src/mean_lunar_validation.rs`:

```rust
//! Fail-closed gate: the packaged backend's mean lunar node, mean apogee and
//! mean perigee vs the committed Swiss Ephemeris `SE_MEAN_NODE` /
//! `SE_MEAN_APOG` reference corpus. Reproduces the exact chart path —
//! packaged-backend J2000 point → `apparent_apsis_position` (precession +
//! nutation in longitude only) — and compares against SE within published
//! ceilings. Sibling of `true_node_validation` (issue #90).
//!
//! Swiss Ephemeris has no mean-perigee body; the perigee is gated on the same
//! rows as the point antipodal to the mean apogee (longitude + 180°, latitude
//! negated) at distance `a(1−e)`.

use pleiades_apparent::{apparent_apsis_position, fnv1a64};
use pleiades_backend::{EphemerisBackend, EphemerisErrorKind, EphemerisRequest};
use pleiades_data::PackagedDataBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/mean-lunar-corpus/mean-lunar.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/mean-lunar-corpus/manifest.txt"
));

/// Swiss Ephemeris `MOON_MEAN_ECC`, used only to derive the perigee reference
/// distance from the corpus apogee distance.
const SE_MOON_MEAN_ECC: f64 = 0.054_900_489;

/// Fail-closed floor on validated rows (corpus is 3177 rows; the first row
/// sits on the coverage boundary and is served, so skips should be zero).
const MIN_ROWS_VALIDATED: usize = 3170;

// Ceilings — PROVISIONAL until Step 4 replaces them with measured values.
const LON_CEILING_ARCSEC: f64 = 5.0;
const LAT_CEILING_ARCSEC: f64 = 5.0;
const DIST_CEILING_REL: f64 = 1.0e-4;

#[derive(Clone, Copy, Debug)]
struct Reference {
    lon_deg: f64,
    lat_deg: f64,
    dist_au: f64,
}

#[derive(Clone, Copy, Debug)]
struct MeanLunarRow {
    jd_tt: f64,
    node: Reference,
    apogee: Reference,
}

impl MeanLunarRow {
    fn perigee(&self) -> Reference {
        Reference {
            lon_deg: (self.apogee.lon_deg + 180.0).rem_euclid(360.0),
            lat_deg: -self.apogee.lat_deg,
            dist_au: self.apogee.dist_au * (1.0 - SE_MOON_MEAN_ECC) / (1.0 + SE_MOON_MEAN_ECC),
        }
    }
}

#[derive(Debug)]
pub enum MeanLunarCorpusError {
    MalformedRow(String),
    MalformedManifest(String),
    ChecksumMismatch {
        got: u64,
        want: u64,
    },
    ManifestDrift {
        rows_csv: usize,
        rows_manifest: usize,
    },
    TooFewRowsValidated {
        validated: usize,
        floor: usize,
    },
    CalculationFailed {
        point: &'static str,
        jd_tt: f64,
        reason: String,
    },
    CeilingExceeded {
        point: &'static str,
        jd_tt: f64,
        kind: &'static str,
        got: f64,
        want: f64,
        residual: f64,
        ceiling: f64,
    },
}

impl std::fmt::Display for MeanLunarCorpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedRow(s) => write!(f, "malformed corpus row: {s}"),
            Self::MalformedManifest(s) => write!(f, "malformed manifest: {s}"),
            Self::ChecksumMismatch { got, want } => {
                write!(f, "corpus checksum mismatch: got {got} want {want}")
            }
            Self::ManifestDrift { rows_csv, rows_manifest } => {
                write!(f, "manifest drift: csv has {rows_csv} rows, manifest says {rows_manifest}")
            }
            Self::TooFewRowsValidated { validated, floor } => {
                write!(f, "only {validated} rows validated, floor is {floor}")
            }
            Self::CalculationFailed { point, jd_tt, reason } => {
                write!(f, "{point} calculation failed at jd_tt={jd_tt}: {reason}")
            }
            Self::CeilingExceeded { point, jd_tt, kind, got, want, residual, ceiling } => write!(
                f,
                "{point} {kind} ceiling exceeded at jd_tt={jd_tt}: got {got:.9} want {want:.9} residual {residual:.6} > ceiling {ceiling:.6}"
            ),
        }
    }
}

impl std::error::Error for MeanLunarCorpusError {}

/// Largest residuals seen for one point across the corpus.
#[derive(Clone, Copy, Debug, Default)]
pub struct ChannelMaxima {
    pub lon_arcsec: f64,
    pub lat_arcsec: f64,
    pub dist_rel: f64,
}

#[derive(Debug)]
pub struct MeanLunarCorpusReport {
    pub rows_validated: usize,
    /// Rows skipped because the instant is outside the packaged window.
    pub rows_skipped_oor: usize,
    pub node: ChannelMaxima,
    pub apogee: ChannelMaxima,
    pub perigee: ChannelMaxima,
    summary_line: String,
}

impl MeanLunarCorpusReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

fn parse_corpus(csv: &str) -> Result<Vec<MeanLunarRow>, MeanLunarCorpusError> {
    let mut rows = Vec::new();
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("jd_tt") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 7 {
            return Err(MeanLunarCorpusError::MalformedRow(format!(
                "expected 7 fields, got {} in {line}",
                fields.len()
            )));
        }
        let num = |i: usize| -> Result<f64, MeanLunarCorpusError> {
            let v = fields[i]
                .parse::<f64>()
                .map_err(|e| MeanLunarCorpusError::MalformedRow(format!("field {i}: {e} in {line}")))?;
            if v.is_finite() {
                Ok(v)
            } else {
                Err(MeanLunarCorpusError::MalformedRow(format!(
                    "field {i} is not finite in {line}"
                )))
            }
        };
        rows.push(MeanLunarRow {
            jd_tt: num(0)?,
            node: Reference { lon_deg: num(1)?, lat_deg: num(2)?, dist_au: num(3)? },
            apogee: Reference { lon_deg: num(4)?, lat_deg: num(5)?, dist_au: num(6)? },
        });
    }
    Ok(rows)
}

fn parse_manifest(manifest: &str) -> Result<(usize, u64), MeanLunarCorpusError> {
    let line = manifest
        .lines()
        .find(|l| l.trim_start().starts_with("slice"))
        .ok_or_else(|| MeanLunarCorpusError::MalformedManifest("no slice line".into()))?;
    let mut rows = None;
    let mut checksum = None;
    for tok in line.split_whitespace() {
        if let Some(v) = tok.strip_prefix("rows=") {
            rows = Some(
                v.parse::<usize>()
                    .map_err(|e| MeanLunarCorpusError::MalformedManifest(format!("rows: {e}")))?,
            );
        } else if let Some(v) = tok.strip_prefix("checksum=") {
            checksum = Some(
                v.parse::<u64>()
                    .map_err(|e| MeanLunarCorpusError::MalformedManifest(format!("checksum: {e}")))?,
            );
        }
    }
    Ok((
        rows.ok_or_else(|| MeanLunarCorpusError::MalformedManifest("rows= missing".into()))?,
        checksum
            .ok_or_else(|| MeanLunarCorpusError::MalformedManifest("checksum= missing".into()))?,
    ))
}

fn wrap_arcsec(got_deg: f64, want_deg: f64) -> f64 {
    ((got_deg - want_deg + 180.0).rem_euclid(360.0) - 180.0).abs() * 3600.0
}

/// Residuals of one packaged point against its reference, or `None` when the
/// instant is outside the packaged window.
fn check_point(
    backend: &PackagedDataBackend,
    body: CelestialBody,
    point: &'static str,
    jd_tt: f64,
    want: Reference,
    maxima: &mut ChannelMaxima,
) -> Result<Option<()>, MeanLunarCorpusError> {
    let failed = |reason: String| MeanLunarCorpusError::CalculationFailed { point, jd_tt, reason };
    let instant = Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tt);
    let mean = match backend.position(&EphemerisRequest::new(body, instant)) {
        Ok(r) => r.ecliptic.ok_or_else(|| failed("no ecliptic".into()))?,
        Err(ref e) if e.kind == EphemerisErrorKind::OutOfRangeInstant => return Ok(None),
        Err(e) => return Err(failed(e.to_string())),
    };
    let apparent = apparent_apsis_position(instant, mean).map_err(|e| failed(format!("{e:?}")))?;
    let got_lon = apparent.ecliptic.longitude.degrees();
    let got_lat = apparent.ecliptic.latitude.degrees();
    let got_dist = apparent
        .ecliptic
        .distance_au
        .ok_or_else(|| failed("no distance".into()))?;

    let checks = [
        ("longitude_arcsec", got_lon, want.lon_deg, wrap_arcsec(got_lon, want.lon_deg), LON_CEILING_ARCSEC),
        ("latitude_arcsec", got_lat, want.lat_deg, ((got_lat - want.lat_deg) * 3600.0).abs(), LAT_CEILING_ARCSEC),
        ("distance_rel", got_dist, want.dist_au, ((got_dist - want.dist_au) / want.dist_au).abs(), DIST_CEILING_REL),
    ];
    for (kind, got, want, residual, ceiling) in checks {
        // `!(residual <= ceiling)` also rejects a NaN residual.
        if !(residual <= ceiling) {
            return Err(MeanLunarCorpusError::CeilingExceeded {
                point, jd_tt, kind, got, want, residual, ceiling,
            });
        }
    }
    maxima.lon_arcsec = maxima.lon_arcsec.max(checks[0].3);
    maxima.lat_arcsec = maxima.lat_arcsec.max(checks[1].3);
    maxima.dist_rel = maxima.dist_rel.max(checks[2].3);
    Ok(Some(()))
}

fn validate(csv: &str, manifest: &str) -> Result<MeanLunarCorpusReport, MeanLunarCorpusError> {
    let (manifest_rows, manifest_checksum) = parse_manifest(manifest)?;
    let got_checksum = fnv1a64(csv);
    if got_checksum != manifest_checksum {
        return Err(MeanLunarCorpusError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let rows = parse_corpus(csv)?;
    if rows.len() != manifest_rows {
        return Err(MeanLunarCorpusError::ManifestDrift {
            rows_csv: rows.len(),
            rows_manifest: manifest_rows,
        });
    }

    let backend = PackagedDataBackend::new();
    let (mut node, mut apogee, mut perigee) =
        (ChannelMaxima::default(), ChannelMaxima::default(), ChannelMaxima::default());
    let mut validated = 0usize;
    let mut skipped_oor = 0usize;

    for row in &rows {
        let n = check_point(&backend, CelestialBody::MeanNode, "mean-node", row.jd_tt, row.node, &mut node)?;
        let a = check_point(&backend, CelestialBody::MeanApogee, "mean-apogee", row.jd_tt, row.apogee, &mut apogee)?;
        let p = check_point(&backend, CelestialBody::MeanPerigee, "mean-perigee", row.jd_tt, row.perigee(), &mut perigee)?;
        if n.is_some() && a.is_some() && p.is_some() {
            validated += 1;
        } else {
            skipped_oor += 1;
        }
    }
    if validated < MIN_ROWS_VALIDATED.min(manifest_rows) {
        return Err(MeanLunarCorpusError::TooFewRowsValidated {
            validated,
            floor: MIN_ROWS_VALIDATED.min(manifest_rows),
        });
    }

    let summary_line = format!(
        "Mean-lunar-points gate: {validated} rows validated ({skipped_oor} oor-skipped) vs Swiss Ephemeris SE_MEAN_NODE/SE_MEAN_APOG, \
         node max lon {:.4}\" lat {:.4}\" dist {:.2e} rel; apogee max lon {:.4}\" lat {:.4}\" dist {:.2e} rel; \
         perigee max lon {:.4}\" lat {:.4}\" dist {:.2e} rel",
        node.lon_arcsec, node.lat_arcsec, node.dist_rel,
        apogee.lon_arcsec, apogee.lat_arcsec, apogee.dist_rel,
        perigee.lon_arcsec, perigee.lat_arcsec, perigee.dist_rel,
    );
    Ok(MeanLunarCorpusReport {
        rows_validated: validated,
        rows_skipped_oor: skipped_oor,
        node,
        apogee,
        perigee,
        summary_line,
    })
}

pub fn validate_mean_lunar_points_corpus() -> Result<MeanLunarCorpusReport, MeanLunarCorpusError> {
    validate(CORPUS_CSV, MANIFEST)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_lunar_points_gate_passes_within_ceilings() {
        let report = validate_mean_lunar_points_corpus().expect("mean-lunar-points gate passes");
        assert!(report.rows_validated >= MIN_ROWS_VALIDATED);
        eprintln!("{}", report.summary_line());
    }

    #[test]
    fn tampered_corpus_fails_the_checksum() {
        let tampered = CORPUS_CSV.replacen("2415020.5,", "2415020.5, ", 1);
        assert!(matches!(
            validate(&tampered, MANIFEST),
            Err(MeanLunarCorpusError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn manifest_row_count_drift_fails_closed() {
        let checksum = fnv1a64(CORPUS_CSV);
        let manifest = format!("slice mean-lunar file=mean-lunar.csv role=mean-lunar rows=3176 checksum={checksum}");
        assert!(matches!(
            validate(CORPUS_CSV, &manifest),
            Err(MeanLunarCorpusError::ManifestDrift { rows_csv: 3177, rows_manifest: 3176 })
        ));
    }

    #[test]
    fn manifest_without_a_slice_line_is_rejected() {
        assert!(matches!(
            validate(CORPUS_CSV, "rows=3177"),
            Err(MeanLunarCorpusError::MalformedManifest(_))
        ));
    }

    #[test]
    fn short_and_non_finite_rows_are_rejected() {
        for bad in ["2451545.0,1.0,0.0", "2451545.0,NaN,0.0,0.0025,1.0,1.0,0.0027"] {
            let manifest = format!("slice x rows=1 checksum={}", fnv1a64(bad));
            assert!(
                matches!(validate(bad, &manifest), Err(MeanLunarCorpusError::MalformedRow(_))),
                "{bad}"
            );
        }
    }

    #[test]
    fn a_shifted_reference_exceeds_the_ceiling() {
        // One real row with the node longitude moved by 0.01° (36″).
        let line = CORPUS_CSV
            .lines()
            .find(|l| l.starts_with("2451546.5,") || l.starts_with("2451523.5,"))
            .or_else(|| CORPUS_CSV.lines().find(|l| l.starts_with("245")))
            .expect("a data row near J2000");
        let mut f: Vec<String> = line.split(',').map(str::to_string).collect();
        let lon: f64 = f[1].parse().unwrap();
        f[1] = format!("{:.9}", (lon + 0.01).rem_euclid(360.0));
        let csv = f.join(",");
        let manifest = format!("slice x rows=1 checksum={}", fnv1a64(&csv));
        assert!(matches!(
            validate(&csv, &manifest),
            Err(MeanLunarCorpusError::CeilingExceeded { point: "mean-node", kind: "longitude_arcsec", .. })
        ));
    }
}
```

`rustfmt` will reflow the long lines; run `cargo fmt --all` rather than hand-wrapping. Note `a_shifted_reference_exceeds_the_ceiling` relies on the row-floor being `min(3170, manifest_rows)`, and on the 36″ shift exceeding the provisional 5″ and every tighter final ceiling.

In `lib.rs`: add `mod mean_lunar_validation;` next to `mod true_node_validation;` (keep the list's ordering convention) and

```rust
pub use mean_lunar_validation::{
    validate_mean_lunar_points_corpus, ChannelMaxima, MeanLunarCorpusError, MeanLunarCorpusReport,
};
```

- [ ] **Step 2: Run the gate tests**

Run: `cargo nextest run -p pleiades-validate mean_lunar --no-capture`
Expected: all six pass; the summary line is printed. If `mean_lunar_points_gate_passes_within_ceilings` fails with `CeilingExceeded` at the provisional 5″, that is the spec's **stop condition** — report the printed error, do not raise the ceiling.

- [ ] **Step 3: Check the measured maxima against the spec's expectation**

From the printed summary: longitude under 1″, latitude under 0.1″, distance under 1e-5 relative, on all three points. If any exceeds its expectation but is under the stop threshold, continue, and state the measured value and the exceeded expectation prominently in the commit body and the final report.

- [ ] **Step 4: Set the final ceilings**

Replace the three provisional constants. For each, take the largest measured value across the three points and set the ceiling to `1.5 ×` it rounded **up** to two significant figures, with floors of `0.01` (arcsec) and `1.0e-9` (relative) so floating-point-noise channels do not get a noise-sized ceiling. Record the measurement in the comment, for example:

```rust
// Ceilings — 1.5 × the largest measured maximum across node/apogee/perigee,
// rounded up to two significant figures (floors 0.01″ / 1e-9), measured
// 2026-10-01 over 3177 rows. The residual is the difference between the Meeus
// mean-element polynomials and Swiss Ephemeris' mean lunar elements.
// max measured: lon <X>", lat <Y>", dist_rel <Z>
const LON_CEILING_ARCSEC: f64 = <1.5·X rounded up>; // measured max <X>"
const LAT_CEILING_ARCSEC: f64 = <1.5·Y rounded up>; // measured max <Y>"
const DIST_CEILING_REL: f64 = <1.5·Z rounded up>; // measured max <Z>
```

(`<X>`, `<Y>`, `<Z>` are the numbers printed in Step 2 — they do not exist until the gate has run; this is a measurement step, not a placeholder to leave in the code.)

Run: `cargo nextest run -p pleiades-validate mean_lunar`
Expected: all pass.

- [ ] **Step 5: Wire the gate into the CLIs**

`render/cli.rs`, in `run_all_numeric_gates` after the true-node line:

```rust
    crate::validate_mean_lunar_points_corpus()
        .map_err(|e| format!("mean-lunar-points gate failed: {e}"))?;
```

Command arm after the `validate-true-node` arm:

```rust
        Some("validate-mean-lunar-points") | Some("mean-lunar-points-gate") => {
            ensure_no_extra_args(&args[1..], "validate-mean-lunar-points")?;
            crate::validate_mean_lunar_points_corpus()
                .map(|report| report.summary_line().to_string())
                .map_err(|e| e.to_string())
        }
```

Help text: in the long help string, after the `  true-node-gate            Alias for validate-true-node\n` segment insert

```
  validate-mean-lunar-points  Run the fail-closed mean lunar node/apogee/perigee gate (Swiss Ephemeris SE_MEAN_NODE / SE_MEAN_APOG, arcsecond ceilings) over the committed mean-lunar corpus\n  mean-lunar-points-gate    Alias for validate-mean-lunar-points\n
```

`crates/pleiades-cli/src/cli.rs`, after the true-node passthrough line:

```rust
        Some("validate-mean-lunar-points") | Some("mean-lunar-points-gate") => {
            validate_render_cli(args)
        }
```

If `pleiades-cli` has its own help listing the gates (`grep -n "true-node-gate" crates/pleiades-cli/src -r`), add the two commands there in the same style.

Run: `cargo run -q -p pleiades-validate -- validate-mean-lunar-points && cargo run -q -p pleiades-cli -- mean-lunar-points-gate`
Expected: the summary line twice.
Run: `cargo run -q -p pleiades-validate -- validate-mean-lunar-points extra; echo "exit=$?"`
Expected: an "unexpected argument"-style error and a non-zero exit.

- [ ] **Step 6: Add the release-grade claims (failing test first)**

Append to `crates/pleiades-data/src/tests/lookup.rs`:

```rust
#[test]
fn mean_lunar_points_carry_release_grade_corpus_claims() {
    use pleiades_backend::EphemerisBackend;
    let claims = PackagedDataBackend::new().metadata().body_claims;
    for body in MEAN_LUNAR_POINTS {
        let claim = claims
            .iter()
            .find(|c| c.body == body)
            .unwrap_or_else(|| panic!("{body:?} claim present"));
        assert_eq!(claim.tier, pleiades_backend::BodyClaimTier::ReleaseGrade);
        match &claim.evidence {
            pleiades_backend::ClaimEvidence::CorpusValidated { source } => {
                assert!(source.contains("validate-mean-lunar-points"), "{source}");
            }
            other => panic!("expected CorpusValidated evidence, got {other:?}"),
        }
    }
    assert_eq!(
        claims.iter().filter(|c| MEAN_LUNAR_POINTS.contains(&c.body)).count(),
        3,
        "exactly one claim per mean lunar point"
    );
}
```

Run: `cargo nextest run -p pleiades-data mean_lunar_points_carry` — Expected: FAIL (`claim present`).

`crates/pleiades-data/src/lib.rs`, after `true_node_body_claims`:

```rust
/// Release claims for the mean lunar points (`MeanNode`, `MeanApogee`,
/// `MeanPerigee`). Formed at lookup from the mean lunar elements in the mean
/// ecliptic of date (the apsides on the inclined mean orbit, as Swiss
/// Ephemeris' `SE_MEAN_APOG` is), emitted in J2000, and validated against the
/// Swiss Ephemeris `SE_MEAN_NODE` / `SE_MEAN_APOG` corpus by the
/// `validate-mean-lunar-points` gate. Supersede, in the routed chart chain,
/// the `pleiades-elp` mean-element channels (issue #90).
pub fn mean_lunar_point_body_claims() -> Vec<pleiades_backend::BodyClaim> {
    use pleiades_backend::{AccuracyClass, BodyClaim, ClaimEvidence};
    let source =
        "Swiss Ephemeris 2.10.03 SE_MEAN_NODE / SE_MEAN_APOG (validate-mean-lunar-points)";
    [
        CelestialBody::MeanNode,
        CelestialBody::MeanApogee,
        CelestialBody::MeanPerigee,
    ]
    .into_iter()
    .map(|body| {
        BodyClaim::release_grade(
            body,
            AccuracyClass::High,
            ClaimEvidence::CorpusValidated {
                source: source.to_string(),
            },
        )
    })
    .collect()
}
```

`backend.rs` `metadata()`: after `claims.extend(crate::true_node_body_claims());` add

```rust
                claims.extend(crate::mean_lunar_point_body_claims());
```

Run: `cargo nextest run -p pleiades-data` — Expected: all pass.

- [ ] **Step 7: Reconcile claim-dependent fixtures across the workspace**

Run in the foreground (do not edit or commit while it runs): `mise run test-full 2>&1 | tail -60`

Expected failures are confined to text pins that render the packaged backend's body claims, the release-grade body list, the capability matrix, or the validate/CLI help text. For each failing test:
1. Read the assertion. If it pins rendered text or a count that now legitimately includes the three mean points or the new gate/commands, update the expectation to the new rendering (re-render with the same command the test exercises and paste the result; for checksum pins, use the value the failure message prints).
2. If a failing test asserts that `MeanNode`/`MeanApogee`/`MeanPerigee` are *not* release-grade or *not* packaged, it encodes the old posture: update it and name it in the commit body.
3. Anything else — a numeric gate, a chart value outside mean points, a release-bundle provenance mismatch — **stop and report**; do not edit it.

Also run: `cargo run -q -p pleiades-validate -- claims-audit && cargo run -q -p pleiades-validate -- compat-claims-audit`
Expected: pass. If the claim audit reports the three new claims as lacking a registered gate, register `validate-mean-lunar-points` wherever `validate-true-node` is registered for it (`grep -rn "validate-true-node" crates/pleiades-validate/src/claims`), mirroring the true-node entry.

Re-run `mise run test-full` until green.

- [ ] **Step 8: Lint and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets --all-features -- -D warnings`

```bash
git add -A crates
git status --short   # only crates/ changes
git commit -m "feat(validate): validate-mean-lunar-points gate and release-grade mean lunar point claims (#90)" -m "Measured maxima: <summary line from Step 2>"
```

---

### Task 8: Documentation and release posture

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/mod.rs` (id line 26; `CURRENT_COMPATIBILITY_PROFILE_SUMMARY` line 45; entry list after line 110)
- Modify: `crates/pleiades-validate/src/tests/render_request.rs:333`, `crates/pleiades-cli/src/cli/tests/summary_commands.rs:440` (profile id pins)
- Modify: `README.md` (capability table after the True Node row), `crates/pleiades-data/README.md`, `crates/pleiades-elp/README.md`, `docs/lunar-theory-policy.md`, `docs/follow-ups.md`

**Interfaces:**
- Consumes: the measured maxima from Task 7's commit body (`git log -1 --format=%b --grep "Measured maxima"`).
- Produces: documentation only.

- [ ] **Step 1: Bump the compatibility profile**

```bash
sed -i 's#pleiades-compatibility-profile/0\.7\.15#pleiades-compatibility-profile/0.7.16#' \
  crates/pleiades-core/src/compatibility/mod.rs \
  crates/pleiades-validate/src/tests/render_request.rs \
  crates/pleiades-cli/src/cli/tests/summary_commands.rs
grep -rn "0\.7\.15" crates   # expected: no output
```

Add a new string after the `"SP-4-FU (osculating true lunar node, issue #58) additions: ..."` entry in the same array, with the measured numbers substituted from Task 7:

```rust
            "Mean lunar points on the packaged backend (issue #90) additions: CelestialBody::MeanNode, MeanApogee and MeanPerigee are now served release-grade by PackagedDataBackend, formed in the mean ecliptic of date from the mean lunar elements (Meeus mean node and mean perigee polynomials plus the Swiss Ephemeris mean inclination, eccentricity and distance, single-sourced in pleiades-apsides) via points_from_elements and emitted in the J2000 boundary frame; the mean apogee and perigee are the points on the inclined mean orbit (Swiss Ephemeris SE_MEAN_APOG, with a latitude of up to 5.15 degrees), not the latitude-0 longitude-of-perigee element. Gated by the fail-closed validate-mean-lunar-points gate (CLI aliases validate-mean-lunar-points, mean-lunar-points-gate), wired into run_all_numeric_gates, over a committed 3177-row Swiss-Ephemeris (SEFLG_MOSEPH, nutation on) SE_MEAN_NODE / SE_MEAN_APOG corpus, checksum-guarded (fnv1a64) and pinned by row count; measured accuracy: max longitude residual <X>\" (ceiling <CX>\"), latitude <Y>\" (ceiling <CY>\"), distance <Z> relative (ceiling <CZ>); the mean perigee is gated as the point antipodal to the SE mean apogee. The chart layer treats all six lunar points (mean and true node, apogee, perigee) as geometric directions: precession + nutation-in-longitude only, no light-time, no aberration, and no topocentric parallax. EventEngine::nod_aps with NodApsMethod::Mean for the Moon now builds its elements from pleiades-apsides and no longer needs a backend that serves MeanNode/MeanPerigee. Behaviour change: in a routed chart chain with PackagedDataBackend first, MeanApogee/MeanPerigee move from the ElpBackend element to the Swiss Ephemeris point (up to about 7 arcminutes in longitude, non-zero latitude). Honesty caveat: ElpBackend's own MeanApogee/MeanPerigee channels remain the raw element with latitude 0, reachable only by direct ELP consumers and documented rather than gated. Compatibility profile bumped to 0.7.16; API stability profile unchanged (purely additive: pleiades_data::mean_lunar_point_body_claims and the pleiades_apsides mean-element functions and constants).",
```

(`<X>`…`<CZ>` are Task 7's measured maxima and final ceilings.)

Append the same sentence to `CURRENT_COMPATIBILITY_PROFILE_SUMMARY` exactly the way the SP-4-FU sentence was appended to it (find it with `grep -o "SP-4-FU (osculating true lunar node.\{0,80\}" crates/pleiades-core/src/compatibility/mod.rs` and follow the same separator and escaping).

- [ ] **Step 2: READMEs and policy doc**

`README.md`, after the True Node table row:

```markdown
| Mean lunar node & apsides (Mean Lilith) | [`pleiades-data`](crates/pleiades-data) | `validate-mean-lunar-points` | sub-arcsecond |
```

Use "arcsecond-class" instead of "sub-arcsecond" if any Task 7 longitude maximum is 1″ or more.

`crates/pleiades-data/README.md`: add a short paragraph after the existing description of derived points (or at the end if there is none):

```markdown
Besides the packaged bodies, the backend serves six derived lunar points: the
osculating `TrueNode`, `TrueApogee` and `TruePerigee` (from the packaged Moon
state) and the mean `MeanNode`, `MeanApogee` and `MeanPerigee` (from the mean
lunar elements in `pleiades-apsides`). The mean apogee and perigee are the
points on the inclined mean orbit, as Swiss Ephemeris reports `SE_MEAN_APOG`.
All six are served only inside the packaged window.
```

`crates/pleiades-elp/README.md`: add

```markdown
The `MeanApogee`/`MeanPerigee` channels are the raw mean longitude-of-perigee
element with latitude 0; they differ from Swiss Ephemeris' `SE_MEAN_APOG` point
by up to about 7′ in longitude and 5.1° in latitude. The routed chart chain
serves the Swiss Ephemeris point from `pleiades-data` instead.
```

`docs/lunar-theory-policy.md`: after the "**Note (true node):**" paragraph add

```markdown
**Note (mean lunar points):** the mean node, mean apogee and mean perigee are
served release-grade by `PackagedDataBackend` from the mean lunar elements
single-sourced in `crates/pleiades-apsides`, gated by
`validate-mean-lunar-points` against Swiss Ephemeris `SE_MEAN_NODE` /
`SE_MEAN_APOG` (issue #90). `ElpBackend`'s mean-element channels remain a
documented lower-tier fallback for direct ELP consumers.
```

- [ ] **Step 3: Follow-ups**

Append to `docs/follow-ups.md` (after FU-14, using the next free number; check with `grep -n "^## FU-" docs/follow-ups.md | tail -3`):

```markdown
---

## FU-15: Mean lunar points on the packaged backend (issue #90)

**Status:** resolved (2026-10-01) · Spec
`docs/superpowers/specs/2026-10-01-mean-lunar-points-packaged-design.md`, plan
`docs/superpowers/plans/2026-10-01-mean-lunar-points-packaged.md`.

**What:** `PackagedDataBackend` did not serve `MeanNode`/`MeanPerigee`, so
`EventEngine::nod_aps(Moon, NodApsMethod::Mean)` failed on
`packaged_backend()`. Resolved by single-sourcing the mean lunar elements in
`pleiades-apsides`, having `nod_aps` use them directly, and serving
`MeanNode`/`MeanApogee`/`MeanPerigee` release-grade from the packaged backend
behind `validate-mean-lunar-points` (3177-row `SE_MEAN_NODE` / `SE_MEAN_APOG`
corpus, 1900–2100; measured maxima: <summary line from Task 7>).

**Behaviour change:** routed charts' `MeanApogee`/`MeanPerigee` moved from the
ELP element (latitude 0) to the Swiss Ephemeris point (up to about 7′ in
longitude, latitude up to 5.15°). The chart now treats all six lunar points as
geometric directions, which also closes #63.

**Build-env note:** `tools/se-mean-lunar-reference` builds inside
`devenv shell` (libclang), like the other `se-*-reference` tools; it is not
needed to run the gate.

**Severity:** feature gap (closed) · **Opened:** 2026-09-30
```

- [ ] **Step 4: Full verification**

Run each in the foreground, in order, with no edits in between:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
mise run test-full
mise run doctest
mise run docs
cargo run -q -p pleiades-validate -- validate-mean-lunar-points
cargo run -q -p pleiades-validate -- validate-nod-aps
cargo run -q -p pleiades-validate -- validate-true-node
cargo run -q -p pleiades-validate -- release-gate
mise run package-check
```

Expected: all pass. `package-check` matters because `pleiades-elp` gained a dependency. A text pin that fails on the new profile entry is fixed as in Task 7 Step 7; anything else is reported, not edited.

- [ ] **Step 5: Commit**

```bash
git add -A README.md docs crates
git status --short
git commit -m "docs: document packaged mean lunar points and bump compatibility profile to 0.7.16 (#90)"
```

---

## Self-review notes

- Spec §1 → Task 1; §2 → Task 3; §3 → Tasks 5 and 7 (claims land with the gate so no commit carries an ungated release-grade claim); §4 → Task 2; §5 → Task 6; §6 → Tasks 4 and 7; §7 tests are distributed to the owning tasks (the ELP-vs-packaged `MeanNode` agreement is covered by `mean_node_is_the_meeus_polynomial_on_the_ecliptic_of_date`, which pins the packaged node to the same single-sourced function ELP now calls); §8 → Task 8.
- Spec risk "distance convention of `SE_MEAN_NODE`" is resolved at Task 4 Step 4 before Task 5 writes the code; both outcomes have explicit code.
- Task order keeps every commit green: the corpus (Task 4) precedes the backend change, and the chart-layer fix (Task 6) follows it by one commit on the same branch.
