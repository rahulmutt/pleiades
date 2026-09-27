# Osculating true lunar node (`TrueNode`, issue #58) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Serve `CelestialBody::TrueNode` release-grade from `PackagedDataBackend` as the osculating ascending node of the lunar orbit, at measured Swiss Ephemeris parity, replacing the ±0.14° Meeus approximation in every routed chart.

**Architecture:** `PackagedDataBackend` gains an osculating-node path beside its existing osculating-apsis path: packaged Moon J2000 state → rotate to the mean ecliptic of date → `pleiades_apsides::elements_from_state` / `points_from_elements` → ascending node → precess back to J2000 for the boundary. The chart layer treats the node like the apsides (precession + nutation in longitude only). A new Swiss Ephemeris `SE_TRUE_NODE` corpus and `validate-true-node` gate mirror the True Lilith gate; a blocking-tier regression test pins the channel to the 8 Moon osculating rows already committed in the nod-aps corpus.

**Tech Stack:** Rust workspace (edition 2021), `cargo nextest`, `pleiades-apsides` Kepler helpers, `pleiades-apparent` precession/nutation, vendored Swiss Ephemeris via `libswisseph-sys 0.1.2` (tool only, built inside `devenv shell`), FNV-1a corpus checksums via `pleiades_apparent::fnv1a64`.

**Spec:** `docs/superpowers/specs/2026-09-26-true-node-osculating-design.md`

## Global Constraints

- Backends are **mean-only, J2000 ecliptic** at the boundary; the node is formed in the **mean ecliptic of date** and precessed back to J2000 with `precess_ecliptic_date_to_j2000` (spec §1).
- `pleiades-data` may depend on `pleiades-apparent` (it depends only on `pleiades-types`/`pleiades-time`; `pleiades-elp` already does). `pleiades-apsides` is **not modified**. `pleiades-data` must **not** depend on `pleiades-validate` or `pleiades-elp`.
- ELP numerics and claims are **untouched**; ELP output must stay bit-identical (spec §5).
- The reference tool lives in `tools/se-true-node-reference`, `publish = false`, `version = "0.0.0"`, outside the workspace; it is built only via `devenv shell` (which supplies `clang`, `libclang`, `LIBCLANG_PATH`).
- Corpus grid is identical to Lilith: `JD_START_TT = 2_415_020.5`, `JD_END_TT = 2_488_070.0`, `STEP_DAYS = 23.0` (3177 rows). SE flags: `SEFLG_MOSEPH` only (nutation on).
- Gate ceilings are `ceil(measured_max × 1.5)` with the measured maxima written in a source comment. Placeholder ceilings must not be committed on the final commit of Task 4.
- Compatibility profile id bumps `0.7.13 → 0.7.14` in all three pinned files; API stability profile id (`pleiades-api-stability/0.3.0`) unchanged.
- Run `cargo fmt --all` before every commit (the CI fmt gate is blocking). Commit messages are conventional commits (release-plz reads them).
- Gate tests in `pleiades-validate` are nightly-tier; everything else added here is blocking-tier and must pass `cargo nextest run --workspace -E 'not package(pleiades-validate)'`.

## Review Focus

1. A `TimeScale::Tdb` request must be served identically to `TimeScale::Tt` (the packaged backend accepts both; the precession epoch uses the raw JD). Pinned in Task 2 test `osculating_true_node_accepts_tdb_like_tt`.
2. The boundary longitude must be normalized to `[0, 360)` and the equatorial channel must be the mean-obliquity transform of the ecliptic channel, like every other packaged body. Pinned in Task 2 test `packaged_backend_serves_osculating_true_node`.
3. An instant before 1900 must fail with `OutOfRangeInstant`, not panic or return a garbage node. Pinned in Task 2 test `osculating_true_node_fails_closed_outside_window`.
4. A **mean** (non-apparent) chart must still place `TrueNode` (marked `Mean`) from the packaged backend rather than fall back to ELP or error. Pinned in Task 3 test `mean_chart_places_true_node_from_packaged_backend`.
5. The gate must fail closed on a truncated corpus (row-count drift) rather than validate a handful of rows and pass. Pinned in Task 4 test `true_node_gate_passes_within_ceilings` (row floor assertion, as the Lilith gate does).

---

### Task 1: Swiss Ephemeris reference tool and committed corpus

**Files:**
- Create: `tools/se-true-node-reference/Cargo.toml`
- Create: `tools/se-true-node-reference/src/main.rs`
- Create: `crates/pleiades-validate/data/true-node-corpus/true-node.csv` (generated)
- Create: `crates/pleiades-validate/data/true-node-corpus/manifest.txt` (checksum filled in Task 4)

**Interfaces:**
- Produces: `true-node.csv` with comment header lines starting `#`, one header row starting `jd_tt`, then rows `jd_tt,lon_deg,lat_deg,dist_au` (of-date true ecliptic, nutation on). Task 4's parser skips `#` lines and the `jd_tt` header.

- [ ] **Step 1: Create the tool manifest**

`tools/se-true-node-reference/Cargo.toml`:

```toml
[package]
name = "se-true-node-reference"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
swisseph = "0.1.1"
libswisseph-sys = "0.1.2"
```

- [ ] **Step 2: Write the generator**

`tools/se-true-node-reference/src/main.rs`:

```rust
//! Emits a Swiss Ephemeris reference corpus for the osculating true lunar
//! ascending node (`SE_TRUE_NODE`) to STDOUT as CSV.
//!
//! Frame: true ecliptic of date, nutation on (SE default — no SEFLG_NONUT,
//! no SEFLG_J2000). Ephemeris: Moshier (SEFLG_MOSEPH) — no data files needed.
//! Same deterministic grid as `tools/se-lilith-reference`, so the two corpora
//! sample the same instants.
//!
//! Build inside `devenv shell` (provides clang/libclang/LIBCLANG_PATH):
//! `devenv shell -- cargo run --release --manifest-path tools/se-true-node-reference/Cargo.toml \
//!    > crates/pleiades-validate/data/true-node-corpus/true-node.csv`

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::swe_calc;

const SE_TRUE_NODE: c_int = 11;
const SEFLG_MOSEPH: c_int = 4;

// Deterministic sampling grid across the 1900–2100 packaged window. 23 days is
// coprime-ish with the ~13.6-day node oscillation and the ~27.2-day draconic
// month, so successive samples land on different orbit phases.
const JD_START_TT: f64 = 2_415_020.5; // 1900-01-01
const JD_END_TT: f64 = 2_488_070.0; //   ~2100-01-01
const STEP_DAYS: f64 = 23.0;

fn se_true_node(jd_tt: f64) -> (f64, f64, f64) {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(
            jd_tt,
            SE_TRUE_NODE,
            SEFLG_MOSEPH,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        let msg = unsafe { CStr::from_ptr(serr.as_ptr() as *const c_char) }
            .to_string_lossy()
            .into_owned();
        panic!("swe_calc(SE_TRUE_NODE) failed at jd_tt={jd_tt}: {msg}");
    }
    let (lon, lat, dist) = (xx[0], xx[1], xx[2]);
    assert!(
        lon.is_finite() && lat.is_finite() && dist.is_finite(),
        "non-finite SE result at jd_tt={jd_tt}"
    );
    (lon.rem_euclid(360.0), lat, dist)
}

fn main() {
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), swe_calc SE_TRUE_NODE=11,");
    println!("# iflag=SEFLG_MOSEPH (Moshier, no data files). Frame: true ecliptic of date, nutation on.");
    println!("# Columns: of-date true ecliptic longitude/latitude (deg) and geocentric distance (AU).");
    println!("# Accuracy note: Moshier Moon vs the DE440-sourced packaged Moon is part of the gate budget.");
    println!("jd_tt,se_true_node_lon_deg,se_true_node_lat_deg,se_true_node_dist_au");
    let mut jd = JD_START_TT;
    while jd <= JD_END_TT {
        let (lon, lat, dist) = se_true_node(jd);
        println!("{jd:.1},{lon:.9},{lat:.9},{dist:.12}");
        jd += STEP_DAYS;
    }
}
```

- [ ] **Step 3: Build and generate the corpus inside the devenv shell**

Run (from the repo root; the first build downloads and compiles the vendored Swiss Ephemeris, allow several minutes):

```bash
mkdir -p crates/pleiades-validate/data/true-node-corpus
devenv shell -- cargo run --release \
  --manifest-path tools/se-true-node-reference/Cargo.toml \
  > crates/pleiades-validate/data/true-node-corpus/true-node.csv
```

Expected: exit 0; no `panic` text on stderr.

- [ ] **Step 4: Sanity-check the corpus against the committed nod-aps rows**

Run:

```bash
grep -vc '^#\|^jd_tt' crates/pleiades-validate/data/true-node-corpus/true-node.csv
grep '^2451545' crates/pleiades-validate/data/true-node-corpus/true-node.csv
```

Expected: `3177` rows. The J2000 row is on the grid (2415020.5 + 23·1588 = 2451544.5 — so instead check the nearest grid row `2451544.5`) and its longitude must be within ~0.1° of the nod-aps corpus Moon osculating J2000 value 123.953312512° (the node moves ≈ −0.05°/day plus a ±0.1°-class oscillation over the half-day gap; a gross frame or body mistake shows as tens of degrees). Also confirm latitude column values are all `0.000000000` or `-0.000000000` (the node lies in the of-date ecliptic by definition):

```bash
grep -v '^#\|^jd_tt' crates/pleiades-validate/data/true-node-corpus/true-node.csv | cut -d, -f3 | sort -u
```

Expected: only `0.000000000` and/or `-0.000000000`.

- [ ] **Step 5: Write a provisional manifest**

`crates/pleiades-validate/data/true-node-corpus/manifest.txt` (checksum is replaced in Task 4 once the gate reports the real value):

```
slice true-node file=true-node.csv role=true-node rows=3177 checksum=0
```

- [ ] **Step 6: Commit**

```bash
git add tools/se-true-node-reference crates/pleiades-validate/data/true-node-corpus
git commit -m "test(validate): add Swiss Ephemeris SE_TRUE_NODE reference tool and corpus (#58)"
```

Do **not** commit `tools/se-true-node-reference/target/` or a `Cargo.lock` unless `tools/se-lilith-reference/Cargo.lock` is committed (it is — mirror that: commit the lock file).

---

### Task 2: Osculating node path in `PackagedDataBackend`

**Files:**
- Modify: `crates/pleiades-data/Cargo.toml`
- Modify: `crates/pleiades-data/src/backend.rs` (imports at top; `osculating_apsis_*` fns around lines 76–200; `supports_body` ≈ line 265; `position` dispatch ≈ line 295; metadata claims ≈ line 247)
- Modify: `crates/pleiades-data/src/lib.rs` (after `apsis_body_claims`, ≈ line 230)
- Test: `crates/pleiades-data/src/tests/lookup.rs` (append)

**Interfaces:**
- Consumes: `pleiades_apsides::{elements_from_state, points_from_elements, MU_EARTH_MOON_AU3_PER_DAY2}`; `pleiades_apparent::{precess_ecliptic_j2000_to_date, precess_ecliptic_date_to_j2000, ApparentPlaceError}` (both precession fns are `(lambda_deg, beta_deg, jd_tt) -> Result<PrecessedEcliptic { longitude_deg, latitude_deg }, ApparentPlaceError>`).
- Produces: `PackagedDataBackend::position` serves `CelestialBody::TrueNode`; `pleiades_data::true_node_body_claims() -> Vec<BodyClaim>`; `supports_body(TrueNode) == true`. Tasks 3 and 4 rely on the `position` behaviour only.

- [ ] **Step 1: Add the dependency**

In `crates/pleiades-data/Cargo.toml` `[dependencies]`, add (alphabetical):

```toml
pleiades-apparent = { workspace = true }
```

- [ ] **Step 2: Write the failing tests**

Append to `crates/pleiades-data/src/tests/lookup.rs`:

```rust
/// Swiss Ephemeris 2.10.03 Moshier `swe_nod_aps(SE_MOON, SE_NODBIT_OSCU)` ascending-node
/// rows (nutation on, iflag 772), copied verbatim from
/// `crates/pleiades-validate/data/nod-aps-corpus/nod-aps.csv` (`Moon,1,2,0,...` rows,
/// columns jd_tt/asc_lon/asc_lat/asc_dist). For the Moon, SE's osculating nod_aps node
/// is `SE_TRUE_NODE`. `pleiades-data` cannot depend on `pleiades-validate`, hence the
/// literal copy. (jd_tt, lon_deg, lat_deg, dist_au)
const SE_MOON_OSCULATING_NODE_ROWS: [(f64, f64, f64, f64); 8] = [
    (2_415_100.5, 253.688_930_115, 0.0, 0.002_591_180),
    (2_433_282.5, 12.557_322_345, 0.0, 0.002_703_927),
    (2_441_683.5, 286.802_808_001, 0.0, 0.002_662_306),
    (2_451_545.0, 123.953_312_512, 0.0, 0.002_445_371),
    (2_459_000.5, 89.238_590_091, 0.0, 0.002_613_909),
    (2_466_154.5, 72.822_192_397, 0.0, 0.002_701_149),
    (2_477_476.5, 192.234_537_069, 0.0, 0.002_612_164),
    (2_488_021.5, 354.954_487_171, 0.0, 0.002_733_866),
];

fn wrap_deg(a: f64, b: f64) -> f64 {
    let mut d = a - b;
    while d > 180.0 {
        d -= 360.0;
    }
    while d < -180.0 {
        d += 360.0;
    }
    d
}

/// Meeus Ch. 47 mean longitude of the ascending node, mean equinox of date (the same
/// polynomial `pleiades-elp` uses for `MeanNode`).
fn meeus_mean_node_of_date_deg(jd_tt: f64) -> f64 {
    let t = (jd_tt - 2_451_545.0) / 36_525.0;
    (125.044_547_9
        + (-1_934.136_289_1 + (0.002_075_4 + (1.0 / 476_441.0 - t / 60_616_000.0) * t) * t) * t)
        .rem_euclid(360.0)
}

#[test]
fn packaged_backend_serves_osculating_true_node() {
    use pleiades_backend::{Apparentness, EphemerisBackend, EphemerisRequest};

    let backend = PackagedDataBackend::new();
    assert!(backend.supports_body(CelestialBody::TrueNode));

    // 2026-01-01 TT: inside the window, away from J2000 so the boundary precession is
    // non-trivial.
    let instant = Instant::new(JulianDay::from_days(2_461_041.5), TimeScale::Tt);
    let node = backend
        .position(&EphemerisRequest::new(CelestialBody::TrueNode, instant))
        .expect("TrueNode position");
    let ecl = node.ecliptic.expect("ecliptic present");

    assert_eq!(node.apparent, Apparentness::Mean);
    let lon = ecl.longitude.degrees();
    assert!((0.0..360.0).contains(&lon), "longitude {lon} not normalized");
    // J2000 boundary frame: the of-date node carries only the small tilt between the
    // two ecliptics (≈0.003° in 2026), never a real orbital latitude.
    assert!(ecl.latitude.degrees().abs() < 0.05, "β {}", ecl.latitude.degrees());
    let d = ecl.distance_au.expect("node distance");
    assert!((0.0023..0.0030).contains(&d), "node distance {d} AU");
    assert!(node.motion.expect("motion").longitude_deg_per_day.is_some());

    // Equatorial is the mean-obliquity transform of the ecliptic channel, like every
    // other packaged body.
    let eq = node.equatorial.expect("equatorial present");
    let expected_eq = ecl.to_equatorial(instant.mean_obliquity());
    assert!(
        (eq.right_ascension.degrees() - expected_eq.right_ascension.degrees()).abs() < 1e-9
    );
    assert!((eq.declination.degrees() - expected_eq.declination.degrees()).abs() < 1e-9);

    let claim = backend
        .metadata()
        .body_claims
        .into_iter()
        .find(|c| c.body == CelestialBody::TrueNode)
        .expect("TrueNode claim present");
    assert_eq!(claim.tier, pleiades_backend::BodyClaimTier::ReleaseGrade);
    match claim.evidence {
        pleiades_backend::ClaimEvidence::CorpusValidated { source } => {
            assert!(source.contains("validate-true-node"), "{source}");
        }
        other => panic!("expected CorpusValidated evidence, got {other:?}"),
    }
}

#[test]
fn osculating_true_node_lies_in_mean_ecliptic_of_date() {
    use pleiades_backend::{EphemerisBackend, EphemerisRequest};

    let backend = PackagedDataBackend::new();
    for jd in [2_433_282.5, 2_461_041.5, 2_477_476.5] {
        let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
        let ecl = backend
            .position(&EphemerisRequest::new(CelestialBody::TrueNode, instant))
            .unwrap()
            .ecliptic
            .unwrap();
        let of_date = pleiades_apparent::precess_ecliptic_j2000_to_date(
            ecl.longitude.degrees(),
            ecl.latitude.degrees(),
            jd,
        )
        .unwrap();
        // Formed in the mean ecliptic of date → forward precession restores β = 0.
        assert!(
            of_date.latitude_deg.abs() < 1e-6,
            "jd {jd}: of-date latitude {}° should be ~0",
            of_date.latitude_deg
        );
    }
}

#[test]
fn osculating_true_node_matches_swiss_ephemeris_nod_aps_rows() {
    use pleiades_backend::{EphemerisBackend, EphemerisRequest};

    // Regression for issue #58: the Meeus periodic-term node sits 0.027° (≈97″) from the
    // J2000 row below; the osculating node must sit within the cross-theory floor the
    // SP-4 engine already measured on these rows (≤18″).
    const LON_CEILING_ARCSEC: f64 = 40.0;
    const LAT_CEILING_ARCSEC: f64 = 10.0;
    const DIST_CEILING_REL: f64 = 1e-3;

    let backend = PackagedDataBackend::new();
    for (jd, se_lon, se_lat, se_dist) in SE_MOON_OSCULATING_NODE_ROWS {
        let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
        let mean = backend
            .position(&EphemerisRequest::new(CelestialBody::TrueNode, instant))
            .unwrap()
            .ecliptic
            .unwrap();
        // Reproduce the chart path: precession + nutation in longitude only.
        let apparent = pleiades_apparent::apparent_apsis_position(instant, mean).unwrap();
        let lon = apparent.ecliptic.longitude.degrees();
        let lat = apparent.ecliptic.latitude.degrees();
        let dist = apparent.ecliptic.distance_au.unwrap();

        let resid_lon = (wrap_deg(lon, se_lon) * 3600.0).abs();
        let resid_lat = ((lat - se_lat) * 3600.0).abs();
        let resid_dist = ((dist - se_dist) / se_dist).abs();
        assert!(resid_lon <= LON_CEILING_ARCSEC, "jd {jd}: lon residual {resid_lon:.3}\"");
        assert!(resid_lat <= LAT_CEILING_ARCSEC, "jd {jd}: lat residual {resid_lat:.3}\"");
        assert!(resid_dist <= DIST_CEILING_REL, "jd {jd}: dist residual {resid_dist:.2e}");
    }
}

#[test]
fn osculating_true_node_stays_within_meeus_envelope_of_mean_node() {
    use pleiades_backend::{EphemerisBackend, EphemerisRequest};

    // The osculating node oscillates around the mean node by up to ≈1.6°; the Meeus
    // periodic terms sum to ≈2°. A frame or sign slip would show up as tens of degrees.
    let backend = PackagedDataBackend::new();
    for jd in [2_415_100.5, 2_441_683.5, 2_461_041.5, 2_488_021.5] {
        let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
        let ecl = backend
            .position(&EphemerisRequest::new(CelestialBody::TrueNode, instant))
            .unwrap()
            .ecliptic
            .unwrap();
        let of_date = pleiades_apparent::precess_ecliptic_j2000_to_date(
            ecl.longitude.degrees(),
            ecl.latitude.degrees(),
            jd,
        )
        .unwrap();
        let delta = wrap_deg(of_date.longitude_deg, meeus_mean_node_of_date_deg(jd));
        assert!(delta.abs() < 2.5, "jd {jd}: osculating − mean node = {delta}°");
    }
}

#[test]
fn osculating_true_node_accepts_tdb_like_tt() {
    use pleiades_backend::{EphemerisBackend, EphemerisRequest};

    let backend = PackagedDataBackend::new();
    let jd = 2_461_041.5;
    let tt = backend
        .position(&EphemerisRequest::new(
            CelestialBody::TrueNode,
            Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
        ))
        .unwrap()
        .ecliptic
        .unwrap();
    let tdb = backend
        .position(&EphemerisRequest::new(
            CelestialBody::TrueNode,
            Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
        ))
        .expect("TDB requests are accepted")
        .ecliptic
        .unwrap();
    assert!((tt.longitude.degrees() - tdb.longitude.degrees()).abs() < 1e-6);
}

#[test]
fn osculating_true_node_fails_closed_outside_window() {
    use pleiades_backend::{EphemerisBackend, EphemerisErrorKind, EphemerisRequest};

    let backend = PackagedDataBackend::new();
    let err = backend
        .position(&EphemerisRequest::new(
            CelestialBody::TrueNode,
            Instant::new(JulianDay::from_days(2_400_000.5), TimeScale::Tt),
        ))
        .expect_err("1858 is outside the packaged window");
    assert_eq!(err.kind, EphemerisErrorKind::OutOfRangeInstant);
}

#[test]
fn osculating_node_motion_degrades_gracefully_at_coverage_boundary() {
    use pleiades_backend::{EphemerisBackend, EphemerisRequest};

    // At the coverage START boundary the −0.5 day motion probe is out of range; the
    // position must still be served with all motion channels None (same contract as
    // the osculating apsides).
    let backend = PackagedDataBackend::new();
    let boundary = Instant::new(JulianDay::from_days(2_415_020.5), TimeScale::Tt);
    let result = backend
        .position(&EphemerisRequest::new(CelestialBody::TrueNode, boundary))
        .expect("position at coverage boundary must succeed");
    assert!(result.ecliptic.unwrap().longitude.degrees().is_finite());
    let motion = result.motion.expect("motion field present");
    assert!(motion.longitude_deg_per_day.is_none());
    assert!(motion.latitude_deg_per_day.is_none());
    assert!(motion.distance_au_per_day.is_none());
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo nextest run -p pleiades-data true_node`
Expected: compile error or failures — `supports_body(TrueNode)` is false and `position` returns an artifact error for `TrueNode`.

- [ ] **Step 4: Refactor the apsis path into shared derived-point helpers and add the node path**

In `crates/pleiades-data/src/backend.rs`, replace the imports:

```rust
use pleiades_apsides::{apsides, MU_EARTH_MOON_AU3_PER_DAY2};
```

with

```rust
use pleiades_apparent::{
    precess_ecliptic_date_to_j2000, precess_ecliptic_j2000_to_date, ApparentPlaceError,
};
use pleiades_apsides::{
    apsides, elements_from_state, points_from_elements, MU_EARTH_MOON_AU3_PER_DAY2,
};
```

and extend the `pleiades_compression` import to `{spherical_state_to_cartesian, CartesianState, CompressedArtifact, SphericalState}`.

Replace the three functions `osculating_apsis_position`, `osculating_apsis_ecliptic`, `osculating_apsis_motion` with this block (same behaviour for the apsides, plus the node):

```rust
    /// Assembles a derived lunar point (osculating apsis or node) into a
    /// backend result: J2000 ecliptic from `eval`, mean-obliquity equatorial,
    /// central-difference motion, `Interpolated` quality.
    fn derived_point_position(
        &self,
        req: &EphemerisRequest,
        eval: &dyn Fn(Instant) -> Result<EclipticCoordinates, EphemerisError>,
    ) -> Result<EphemerisResult, EphemerisError> {
        let ecliptic = eval(req.instant)?;
        let equatorial = ecliptic.to_equatorial(req.instant.mean_obliquity());
        let motion = self.derived_point_motion(req.instant, eval)?;

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

    /// The packaged Moon's geocentric J2000 Cartesian state at `instant`: the
    /// shared input of every derived lunar point.
    fn moon_state_j2000(&self, instant: Instant) -> Result<CartesianState, EphemerisError> {
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
                "packaged Moon lacks distance for a derived lunar point",
            )
        })?;
        Ok(spherical_state_to_cartesian(SphericalState {
            lon_rad: ecl.longitude.degrees().to_radians(),
            lat_rad: ecl.latitude.degrees().to_radians(),
            dist_au: dist,
            lon_rate_rad_per_day: mot.longitude_deg_per_day.unwrap_or(0.0).to_radians(),
            lat_rate_rad_per_day: mot.latitude_deg_per_day.unwrap_or(0.0).to_radians(),
            dist_rate_au_per_day: mot.distance_au_per_day.unwrap_or(0.0),
        }))
    }

    fn osculating_apsis_ecliptic(
        &self,
        body: &CelestialBody,
        instant: Instant,
    ) -> Result<EclipticCoordinates, EphemerisError> {
        let cart = self.moon_state_j2000(instant)?;
        let aps = apsides(cart.pos_au, cart.vel_au_per_day, MU_EARTH_MOON_AU3_PER_DAY2).map_err(
            |_| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "osculating apsis undefined for the lunar state at this instant",
                )
            },
        )?;
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

    /// Osculating ascending node of the geocentric lunar orbit, J2000 boundary
    /// frame.
    ///
    /// The node is the intersection of the orbit plane with the *reference*
    /// plane, so it must be formed in the plane consumers will read it in: the
    /// mean ecliptic of date (spec §1; forming it in J2000 and rotating the
    /// point would misplace it by ≈ tilt/sin(i) ≈ 0.04°). Position and velocity
    /// are rotated J2000 → mean-of-date, the ellipse is formed there, and the
    /// node point is precessed back to J2000 like the ELP point channels
    /// (issue #57). The chart layer's forward precession + Δψ then reproduces
    /// Swiss Ephemeris `SE_TRUE_NODE`.
    fn osculating_node_ecliptic(&self, instant: Instant) -> Result<EclipticCoordinates, EphemerisError> {
        let jd_tt = instant.julian_day.days();
        let cart = self.moon_state_j2000(instant)?;
        let pos = rotate_j2000_to_mean_of_date(cart.pos_au, jd_tt)?;
        let vel = rotate_j2000_to_mean_of_date(cart.vel_au_per_day, jd_tt)?;
        let undefined = |_| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "osculating node undefined for the lunar state at this instant",
            )
        };
        let elements =
            elements_from_state(pos, vel, MU_EARTH_MOON_AU3_PER_DAY2).map_err(undefined)?;
        let node = points_from_elements(&elements, false)
            .map_err(undefined)?
            .ascending;
        let j2000 = precess_ecliptic_date_to_j2000(node.longitude_deg, node.latitude_deg, jd_tt)
            .map_err(map_precession_error)?;
        Ok(EclipticCoordinates::new(
            Longitude::from_degrees(j2000.longitude_deg),
            Latitude::from_degrees(j2000.latitude_deg),
            Some(node.distance_au),
        ))
    }

    /// Central-difference motion of a derived lunar point over ±0.5 day. A
    /// probe that falls outside the packaged window degrades to `None`
    /// channels rather than failing the position.
    fn derived_point_motion(
        &self,
        instant: Instant,
        eval: &dyn Fn(Instant) -> Result<EclipticCoordinates, EphemerisError>,
    ) -> Result<Motion, EphemerisError> {
        const HALF_SPAN_DAYS: f64 = 0.5;
        let shift = |days: f64| {
            Instant::new(
                JulianDay::from_days(instant.julian_day.days() + days),
                instant.scale,
            )
        };
        let before = match eval(shift(-HALF_SPAN_DAYS)) {
            Ok(e) => e,
            Err(ref e) if e.kind == EphemerisErrorKind::OutOfRangeInstant => {
                return Ok(Motion::new(None, None, None));
            }
            Err(e) => return Err(e),
        };
        let after = match eval(shift(HALF_SPAN_DAYS)) {
            Ok(e) => e,
            Err(ref e) if e.kind == EphemerisErrorKind::OutOfRangeInstant => {
                return Ok(Motion::new(None, None, None));
            }
            Err(e) => return Err(e),
        };
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
        Ok(Motion::new(
            Some(dlon_per_day),
            Some(dlat_per_day),
            ddist_per_day,
        ))
    }
```

Add these free functions at module level (below the `impl PackagedDataBackend` block, above `impl EphemerisBackend`):

```rust
/// Rotates a J2000 mean-ecliptic vector into the mean ecliptic of date at
/// `jd_tt`, preserving its magnitude. Precession is a rotation, so the same
/// map applies to position and velocity vectors alike (the events engine's
/// osculating path does the same).
fn rotate_j2000_to_mean_of_date(v: [f64; 3], jd_tt: f64) -> Result<[f64; 3], EphemerisError> {
    let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if r == 0.0 {
        return Ok(v);
    }
    let lon_deg = v[1].atan2(v[0]).to_degrees().rem_euclid(360.0);
    let lat_deg = (v[2] / r).asin().to_degrees();
    let p = precess_ecliptic_j2000_to_date(lon_deg, lat_deg, jd_tt).map_err(map_precession_error)?;
    let (sl, cl) = p.longitude_deg.to_radians().sin_cos();
    let (sb, cb) = p.latitude_deg.to_radians().sin_cos();
    Ok([r * cb * cl, r * cb * sl, r * sb])
}

fn map_precession_error(e: ApparentPlaceError) -> EphemerisError {
    EphemerisError::new(
        EphemerisErrorKind::InvalidRequest,
        format!("precession failed for derived lunar point: {e}"),
    )
}
```

Update `supports_body`:

```rust
    fn supports_body(&self, body: CelestialBody) -> bool {
        matches!(
            body,
            CelestialBody::TrueApogee | CelestialBody::TruePerigee | CelestialBody::TrueNode
        ) || self
            .artifact
            .bodies
            .iter()
            .any(|series| series.body == body)
    }
```

Update the dispatch in `position` (replace the existing `if matches!(req.body, TrueApogee | TruePerigee) { return self.osculating_apsis_position(req); }`):

```rust
        if matches!(
            req.body,
            CelestialBody::TrueApogee | CelestialBody::TruePerigee
        ) {
            let body = req.body.clone();
            return self.derived_point_position(req, &|i| self.osculating_apsis_ecliptic(&body, i));
        }
        if req.body == CelestialBody::TrueNode {
            return self.derived_point_position(req, &|i| self.osculating_node_ecliptic(i));
        }
```

In `metadata()`, directly after `claims.extend(crate::apsis_body_claims());` add:

```rust
                claims.extend(crate::true_node_body_claims());
```

In `crates/pleiades-data/src/lib.rs`, after `apsis_body_claims`:

```rust
/// Release claim for the derived osculating lunar ascending node
/// (`TrueNode`). Computed from the packaged Moon state at lookup (formed in
/// the mean ecliptic of date, emitted in J2000) and validated against the
/// Swiss Ephemeris `SE_TRUE_NODE` corpus by the `validate-true-node` gate, so
/// its evidence is `CorpusValidated`. Supersedes, in the routed chart chain,
/// the `pleiades-elp` Meeus periodic-term approximation (issue #58).
pub fn true_node_body_claims() -> Vec<pleiades_backend::BodyClaim> {
    use pleiades_backend::{AccuracyClass, BodyClaim, ClaimEvidence};
    vec![BodyClaim::release_grade(
        CelestialBody::TrueNode,
        AccuracyClass::High,
        ClaimEvidence::CorpusValidated {
            source: "Swiss Ephemeris 2.10.03 SE_TRUE_NODE (validate-true-node)".to_string(),
        },
    )]
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo nextest run -p pleiades-data`
Expected: all pass, including the seven new `true_node`/`osculating_node` tests and the pre-existing apsis tests (`packaged_backend_serves_osculating_true_apsides`, `osculating_apsis_motion_degrades_gracefully_at_coverage_boundary`). If `osculating_true_node_matches_swiss_ephemeris_nod_aps_rows` fails with residuals of tens of degrees, the rotation or the ascending/descending choice is wrong; if it fails by ~0.03–0.05°, the node was formed in the wrong plane (check that both `pos` and `vel` are rotated before `elements_from_state`).

Also confirm no test elsewhere pinned the old apsis distance message:

Run: `grep -rn "lacks distance for osculating apsis" crates/`
Expected: no matches.

- [ ] **Step 6: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-data --all-targets --all-features -- -D warnings
git add crates/pleiades-data Cargo.lock
git commit -m "feat(data): serve TrueNode as the osculating lunar node from the packaged Moon state (#58)"
```

---

### Task 3: Chart layer treats `TrueNode` as a geometric direction

**Files:**
- Modify: `crates/pleiades-core/src/chart/mod.rs:401-405` (the `TrueApogee | TruePerigee` match arm)
- Test: `crates/pleiades-core/src/chart/tests.rs` (append after `chart_serves_apparent_true_apsides_precession_nutation_only`)

**Interfaces:**
- Consumes: `PackagedDataBackend::position(TrueNode)` from Task 2; `pleiades_apparent::apparent_apsis_position`.
- Produces: apparent charts place `TrueNode` with `Apparentness::Apparent`, no light-time, no annual aberration.

- [ ] **Step 1: Write the failing tests**

Append to `crates/pleiades-core/src/chart/tests.rs`:

```rust
#[test]
fn chart_serves_apparent_true_node_precession_nutation_only() {
    use pleiades_backend::{Apparentness, EphemerisBackend, EphemerisRequest};
    use pleiades_data::PackagedDataBackend;
    use pleiades_types::CelestialBody;

    let instant = Instant::new(pleiades_types::JulianDay::from_days(2_461_041.5), TimeScale::Tt);
    let request = ChartRequest::new(instant)
        .with_bodies(vec![CelestialBody::TrueNode])
        .with_apparentness(Apparentness::Apparent);

    let backend = PackagedDataBackend::new();
    let snapshot = ChartEngine::new(backend.clone())
        .chart(&request)
        .expect("apparent true-node chart should succeed");
    let node = snapshot
        .placement_for(&CelestialBody::TrueNode)
        .expect("TrueNode placement must be present");

    assert_eq!(
        node.position.apparent,
        pleiades_types::Apparentness::Apparent,
        "TrueNode should be Apparent"
    );
    let prov = node
        .apparent
        .as_ref()
        .expect("TrueNode must carry apparent provenance");
    assert!(!prov.corrections.light_time, "no light-time (geometric direction)");
    assert!(!prov.corrections.annual_aberration, "no annual aberration (geometric direction)");
    assert_eq!(prov.aberration_longitude_arcsec, 0.0);

    // The chart value is exactly the backend's mean-J2000 node through
    // apparent_apsis_position (precession + Δψ): nothing else may creep in.
    let mean = backend
        .position(&EphemerisRequest::new(CelestialBody::TrueNode, instant))
        .unwrap()
        .ecliptic
        .unwrap();
    let expected = pleiades_apparent::apparent_apsis_position(instant, mean).unwrap();
    let chart_lon = node.position.ecliptic.unwrap().longitude.degrees();
    assert!(
        (chart_lon - expected.ecliptic.longitude.degrees()).abs() < 1e-9,
        "chart {chart_lon} vs apsis-path {}",
        expected.ecliptic.longitude.degrees()
    );
    // Of-date node lies in the ecliptic (|β| is only the tiny Δψ-induced term).
    assert!(node.position.ecliptic.unwrap().latitude.degrees().abs() < 0.01);
}

#[test]
fn mean_chart_places_true_node_from_packaged_backend() {
    use pleiades_backend::Apparentness;
    use pleiades_data::PackagedDataBackend;
    use pleiades_types::CelestialBody;

    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_461_041.5),
        TimeScale::Tt,
    ))
    .with_bodies(vec![CelestialBody::TrueNode])
    .with_apparentness(Apparentness::Mean);

    let snapshot = ChartEngine::new(PackagedDataBackend::new())
        .chart(&request)
        .expect("mean true-node chart should succeed");
    let node = snapshot
        .placement_for(&CelestialBody::TrueNode)
        .expect("TrueNode placement must be present");
    assert_eq!(node.position.apparent, pleiades_types::Apparentness::Mean);
    assert!(node.position.ecliptic.unwrap().longitude.degrees().is_finite());
}
```

If `ChartRequest::with_apparentness` takes a different `Apparentness` path than the neighbouring apsides test uses, copy that test's exact imports.

- [ ] **Step 2: Run the tests to verify the apparent one fails**

Run: `cargo nextest run -p pleiades-core true_node`
Expected: `chart_serves_apparent_true_node_precession_nutation_only` FAILS — either `light_time`/`annual_aberration` is true (the node went through the generic light-time branch) or the longitude differs from the apsis path by ~20″ (aberration). `mean_chart_places_true_node_from_packaged_backend` may already pass.

- [ ] **Step 3: Extend the match arm**

In `crates/pleiades-core/src/chart/mod.rs`, change

```rust
                        } else if matches!(
                            body,
                            pleiades_types::CelestialBody::TrueApogee
                                | pleiades_types::CelestialBody::TruePerigee
                        ) {
                            // Osculating apsis: a geometric direction. Apply precession +
                            // nutation only (no light-time re-query, no annual aberration).
                            // observer = None keeps it geocentric.
```

to

```rust
                        } else if matches!(
                            body,
                            pleiades_types::CelestialBody::TrueApogee
                                | pleiades_types::CelestialBody::TruePerigee
                                | pleiades_types::CelestialBody::TrueNode
                        ) {
                            // Osculating apsis or node: a geometric direction. Apply
                            // precession + nutation only (no light-time re-query, no
                            // annual aberration). observer = None keeps it geocentric.
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo nextest run -p pleiades-core`
Expected: all pass.

- [ ] **Step 5: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-core --all-targets --all-features -- -D warnings
git add crates/pleiades-core
git commit -m "feat(core): apply precession+nutation only to the osculating TrueNode in apparent charts (#58)"
```

---

### Task 4: `validate-true-node` gate

**Files:**
- Create: `crates/pleiades-validate/src/true_node_validation.rs`
- Modify: `crates/pleiades-validate/src/lib.rs:33` (`mod`) and `:230` (`pub use`)
- Modify: `crates/pleiades-validate/src/render/cli.rs` (`run_all_numeric_gates` ≈ line 98; dispatch ≈ line 305; help text ≈ line 2258)
- Modify: `crates/pleiades-cli/src/cli.rs:802` (dispatch)
- Modify: `crates/pleiades-validate/data/true-node-corpus/manifest.txt` (real checksum)

**Interfaces:**
- Consumes: Task 1 corpus; Task 2 `PackagedDataBackend::position(TrueNode)`.
- Produces: `pleiades_validate::{validate_true_node_corpus, TrueNodeCorpusError, TrueNodeCorpusReport}`; CLI subcommands `validate-true-node` / `true-node-gate`; the gate runs inside `release-gate`.

- [ ] **Step 1: Write the gate module with provisional ceilings**

`crates/pleiades-validate/src/true_node_validation.rs` (ceilings are provisional here and replaced in Step 5):

```rust
//! Fail-closed gate: our of-date osculating True Node vs the committed Swiss
//! Ephemeris `SE_TRUE_NODE` reference corpus. Reproduces the exact chart path —
//! packaged-backend mean-J2000 node → `apparent_apsis_position` (precession +
//! nutation in longitude only) — and compares against SE within published
//! ceilings. Sibling of `lilith_validation` (issue #58).

use pleiades_apparent::{apparent_apsis_position, fnv1a64};
use pleiades_backend::{EphemerisBackend, EphemerisErrorKind, EphemerisRequest};
use pleiades_data::PackagedDataBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/true-node-corpus/true-node.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/true-node-corpus/manifest.txt"
));

// PROVISIONAL — replaced by ceil(measured_max * 1.5) in the same task.
const LON_CEILING_ARCSEC: f64 = 1.0e9;
const LAT_CEILING_ARCSEC: f64 = 1.0e9;
const DIST_CEILING_REL: f64 = 1.0e9;

#[derive(Clone, Copy, Debug)]
struct TrueNodeRow {
    jd_tt: f64,
    lon_deg: f64,
    lat_deg: f64,
    dist_au: f64,
}

#[derive(Debug)]
pub enum TrueNodeCorpusError {
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
    CalculationFailed {
        jd_tt: f64,
        reason: String,
    },
    CeilingExceeded {
        jd_tt: f64,
        kind: &'static str,
        got: f64,
        want: f64,
        residual: f64,
        ceiling: f64,
    },
}

impl std::fmt::Display for TrueNodeCorpusError {
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
            Self::CalculationFailed { jd_tt, reason } => {
                write!(f, "calculation failed at jd_tt={jd_tt}: {reason}")
            }
            Self::CeilingExceeded { jd_tt, kind, got, want, residual, ceiling } => write!(
                f,
                "true-node {kind} ceiling exceeded at jd_tt={jd_tt}: got {got:.6} want {want:.6} residual {residual:.4} > ceiling {ceiling:.4}"
            ),
        }
    }
}

impl std::error::Error for TrueNodeCorpusError {}

#[derive(Debug)]
pub struct TrueNodeCorpusReport {
    pub rows_validated: usize,
    /// Rows skipped because the packaged backend's coverage boundary (±0.5 day
    /// motion probe) falls outside the supported range; a boundary handful at most.
    pub rows_skipped_oor: usize,
    pub max_residual_lon_arcsec: f64,
    pub max_residual_lat_arcsec: f64,
    pub max_residual_dist_rel: f64,
    summary_line: String,
}

impl TrueNodeCorpusReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

fn parse_corpus() -> Result<Vec<TrueNodeRow>, TrueNodeCorpusError> {
    let mut rows = Vec::new();
    for line in CORPUS_CSV.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("jd_tt") {
            continue;
        }
        let mut it = line.split(',');
        let mut next = |name: &str| -> Result<f64, TrueNodeCorpusError> {
            it.next()
                .ok_or_else(|| {
                    TrueNodeCorpusError::MalformedRow(format!("{name} missing in {line}"))
                })?
                .parse::<f64>()
                .map_err(|e| TrueNodeCorpusError::MalformedRow(format!("{name}: {e} in {line}")))
        };
        rows.push(TrueNodeRow {
            jd_tt: next("jd_tt")?,
            lon_deg: next("lon")?,
            lat_deg: next("lat")?,
            dist_au: next("dist")?,
        });
    }
    Ok(rows)
}

fn parse_manifest_rows() -> Result<(usize, u64), TrueNodeCorpusError> {
    let line = MANIFEST
        .lines()
        .find(|l| l.trim_start().starts_with("slice"))
        .ok_or_else(|| TrueNodeCorpusError::MalformedManifest("no slice line".into()))?;
    let mut rows = None;
    let mut checksum = None;
    for tok in line.split_whitespace() {
        if let Some(v) = tok.strip_prefix("rows=") {
            rows = Some(
                v.parse::<usize>()
                    .map_err(|e| TrueNodeCorpusError::MalformedManifest(format!("rows: {e}")))?,
            );
        } else if let Some(v) = tok.strip_prefix("checksum=") {
            checksum = Some(v.parse::<u64>().map_err(|e| {
                TrueNodeCorpusError::MalformedManifest(format!("checksum: {e}"))
            })?);
        }
    }
    Ok((
        rows.ok_or_else(|| TrueNodeCorpusError::MalformedManifest("rows= missing".into()))?,
        checksum
            .ok_or_else(|| TrueNodeCorpusError::MalformedManifest("checksum= missing".into()))?,
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

pub fn validate_true_node_corpus() -> Result<TrueNodeCorpusReport, TrueNodeCorpusError> {
    let (manifest_rows, manifest_checksum) = parse_manifest_rows()?;
    let got_checksum = fnv1a64(CORPUS_CSV);
    if got_checksum != manifest_checksum {
        return Err(TrueNodeCorpusError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let rows = parse_corpus()?;
    if rows.len() != manifest_rows {
        return Err(TrueNodeCorpusError::ManifestDrift {
            rows_csv: rows.len(),
            rows_manifest: manifest_rows,
        });
    }

    let backend = PackagedDataBackend::new();
    let mut max_lon = 0.0_f64;
    let mut max_lat = 0.0_f64;
    let mut max_dist = 0.0_f64;
    let mut validated = 0usize;
    let mut skipped_oor = 0usize;

    for row in &rows {
        let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);
        let mean = match backend.position(&EphemerisRequest::new(CelestialBody::TrueNode, instant)) {
            Ok(r) => r
                .ecliptic
                .ok_or_else(|| TrueNodeCorpusError::CalculationFailed {
                    jd_tt: row.jd_tt,
                    reason: "no ecliptic".into(),
                })?,
            Err(ref e) if e.kind == EphemerisErrorKind::OutOfRangeInstant => {
                // ±0.5-day motion probe outside the 1900–2100 window at the
                // first/last rows only.
                skipped_oor += 1;
                continue;
            }
            Err(e) => {
                return Err(TrueNodeCorpusError::CalculationFailed {
                    jd_tt: row.jd_tt,
                    reason: e.to_string(),
                })
            }
        };
        let apparent = apparent_apsis_position(instant, mean).map_err(|e| {
            TrueNodeCorpusError::CalculationFailed {
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
            return Err(TrueNodeCorpusError::CeilingExceeded {
                jd_tt: row.jd_tt,
                kind: "longitude_arcsec",
                got: our_lon,
                want: row.lon_deg,
                residual: resid_lon,
                ceiling: LON_CEILING_ARCSEC,
            });
        }
        if resid_lat > LAT_CEILING_ARCSEC {
            return Err(TrueNodeCorpusError::CeilingExceeded {
                jd_tt: row.jd_tt,
                kind: "latitude_arcsec",
                got: our_lat,
                want: row.lat_deg,
                residual: resid_lat,
                ceiling: LAT_CEILING_ARCSEC,
            });
        }
        if resid_dist > DIST_CEILING_REL {
            return Err(TrueNodeCorpusError::CeilingExceeded {
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
        validated += 1;
    }

    let summary_line = format!(
        "True-node gate: {validated} rows validated ({skipped_oor} oor-skipped) vs Swiss Ephemeris SE_TRUE_NODE, max lon {max_lon:.3}\" lat {max_lat:.3}\" dist {max_dist:.2e} rel"
    );
    Ok(TrueNodeCorpusReport {
        rows_validated: validated,
        rows_skipped_oor: skipped_oor,
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
    fn true_node_gate_passes_within_ceilings() {
        let report = validate_true_node_corpus().expect("true-node gate passes");
        assert!(report.rows_validated > 0);
        // Fail-closed floor: the OOR skip is a boundary-row accommodation, not a
        // license to validate almost nothing.
        assert!(
            report.rows_skipped_oor <= 5,
            "too many oor-skipped rows: {}",
            report.rows_skipped_oor
        );
        assert!(
            report.rows_validated >= 3170,
            "too few rows validated: {} (corpus is 3177 rows)",
            report.rows_validated
        );
        // Print measured maxima so the ceilings can be set/tightened.
        eprintln!("{}", report.summary_line());
    }
}
```

- [ ] **Step 2: Register the module and exports**

In `crates/pleiades-validate/src/lib.rs`, after `mod lilith_validation;` add `mod true_node_validation;` (keep the list alphabetical: it goes after `mod topocentric...`/wherever `t` sorts; if the list is alphabetical place it accordingly, otherwise directly after `lilith_validation`). After the `pub use lilith_validation::{...};` line add:

```rust
pub use true_node_validation::{
    validate_true_node_corpus, TrueNodeCorpusError, TrueNodeCorpusReport,
};
```

- [ ] **Step 3: Run the gate test once to obtain the real checksum**

Run: `cargo nextest run -p pleiades-validate true_node_gate --no-fail-fast 2>&1 | grep -o "checksum mismatch: got [0-9]* want 0"`
Expected: one line `corpus checksum mismatch: got <N> want 0`. Put `<N>` into `manifest.txt`:

```
slice true-node file=true-node.csv role=true-node rows=3177 checksum=<N>
```

- [ ] **Step 4: Run the gate test to measure residuals**

Run: `cargo nextest run -p pleiades-validate true_node_gate --no-capture 2>&1 | grep "True-node gate:"`
Expected: `True-node gate: 3176 rows validated (1 oor-skipped) ... max lon <L>" lat <B>" dist <D> rel` (the OOR count may be 0–2). If `<L>` exceeds ~60″ or `<B>` exceeds ~5″, stop: the node path or the frame is wrong (Task 2's 8-row test bounds should have caught it; re-check the generator's body constant `SE_TRUE_NODE = 11` and flags).

- [ ] **Step 5: Set the ceilings from the measurement**

Replace the three provisional constants with `ceil(measured × 1.5)` and the measured values in the comment, e.g. (numbers illustrative — use the measured ones):

```rust
// Ceilings — set to ceil(measured_max * 1.5), measured 2026-09-26 over 3176 rows.
// The dominant residual is the Moshier(SE corpus) vs DE440(our packaged Moon) difference
// amplified by 1/sin(i) in the node direction (i ≈ 5.1°).
// max measured: lon <L>", lat <B>", dist_rel <D>
const LON_CEILING_ARCSEC: f64 = <ceil(L*1.5)>; // measured max <L>"
const LAT_CEILING_ARCSEC: f64 = <ceil(B*1.5)>; // measured max <B>"
const DIST_CEILING_REL: f64 = <ceil-ish(D*1.5)>; // measured max <D>
```

Run: `cargo nextest run -p pleiades-validate true_node_gate`
Expected: PASS.

- [ ] **Step 6: Wire the CLI and the release gate set**

In `crates/pleiades-validate/src/render/cli.rs`:

1. In `run_all_numeric_gates`, directly after the `validate_lilith_corpus()` line add:

```rust
    crate::validate_true_node_corpus()
        .map_err(|e| format!("true-node gate failed: {e}"))?;
```

2. In the subcommand dispatch, directly after the `Some("validate-lilith") | Some("lilith-gate")` arm add:

```rust
        Some("validate-true-node") | Some("true-node-gate") => {
            ensure_no_extra_args(&args[1..], "validate-true-node")?;
            crate::validate_true_node_corpus()
                .map(|report| report.summary_line().to_string())
                .map_err(|e| e.to_string())
        }
```

3. In the help text string (≈ line 2258) there are **two** occurrences of `validate-lilith`: a described command line and a bare-name list. Run `grep -n -o 'validate-lilith[^\\]*' crates/pleiades-validate/src/render/cli.rs` to find both; after the described line insert

```
validate-true-node        Run the fail-closed osculating true node gate (Swiss Ephemeris SE_TRUE_NODE, arcsecond ceilings) over the committed true-node corpus\n
```

and after the bare `validate-lilith\n` insert `validate-true-node\n`.

In `crates/pleiades-cli/src/cli.rs`, directly after line 802 (`Some("validate-lilith") | Some("lilith-gate") => validate_render_cli(args),`) add:

```rust
        Some("validate-true-node") | Some("true-node-gate") => validate_render_cli(args),
```

- [ ] **Step 7: Verify the CLI end to end**

Run:

```bash
cargo run -q -p pleiades-validate -- validate-true-node
cargo run -q -p pleiades-validate -- true-node-gate
cargo run -q -p pleiades-cli -- validate-true-node
cargo run -q -p pleiades-validate -- --help 2>&1 | grep -c "validate-true-node"
```

Expected: the first three print the `True-node gate: ... rows validated ...` line and exit 0; the last prints `2` (or however many times `validate-lilith` appears in the same help output — match it).

Then run the CLI test suites that pin help/alias output:

Run: `cargo nextest run -p pleiades-cli -p pleiades-validate -E 'test(/cli|help|summary_commands/)'`
Expected: PASS. If a test pins the full help text verbatim, update the expectation in the same commit.

- [ ] **Step 8: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-validate -p pleiades-cli --all-targets --all-features -- -D warnings
git add crates/pleiades-validate crates/pleiades-cli
git commit -m "test(validate): add fail-closed validate-true-node gate over the SE_TRUE_NODE corpus (#58)"
```

---

### Task 5: Documentation, ELP fallback notes, compatibility profile bump

**Files:**
- Modify: `crates/pleiades-elp/src/backend.rs:24-30` (claims doc) and `:149` (`true_node_longitude` doc)
- Modify: `docs/lunar-theory-policy.md:21` (add a true-node note after the Lilith note)
- Modify: `docs/follow-ups.md` (FU-2 build-env note wording; append FU-12)
- Modify: `README.md:41` (validation table row)
- Modify: `crates/pleiades-apsides/README.md:10` (mention the node consumer)
- Modify: `crates/pleiades-core/src/compatibility/mod.rs:26` (profile id), `:45` (summary tail), `:109` (release_notes entry)
- Modify: `crates/pleiades-validate/src/tests/render_request.rs:333`, `crates/pleiades-cli/src/cli/tests/summary_commands.rs:440` (profile id pins)

**Interfaces:**
- Consumes: the measured maxima and ceilings from Task 4 Step 5 (`<L>`, `<B>`, `<D>`).
- Produces: nothing code-facing; `compat-claims-audit` and `claims-audit` must stay green.

- [ ] **Step 1: ELP doc comments**

In `crates/pleiades-elp/src/backend.rs`, replace the `elp_body_claims` doc comment paragraph beginning `/// Lunar bodies (Moon, Mean Node, True Node, ...` with:

```rust
/// Lunar bodies (Moon, Mean Node, True Node, Mean Apogee, Mean Perigee) are claimed as Constrained
/// with Moderate accuracy and AlgorithmicModel evidence. True Apogee and True Perigee are explicitly
/// listed as Unsupported by this backend. Note: the osculating true apogee/perigee (True Lilith)
/// and the osculating true node are served release-grade by `PackagedDataBackend` ahead of this
/// backend in the composite routing chain, so the ELP-local `Unsupported` apsis claims are not a
/// global gap, and this backend's `TrueNode` (Meeus's periodic-term-corrected mean node, ±0.14°
/// from Swiss Ephemeris' osculating node — issue #58) is reached only by direct ELP consumers.
```

Above `fn true_node_longitude(days: f64) -> f64 {` add:

```rust
    /// Meeus Ch. 47 "true" node: the mean node plus five periodic terms in D,
    /// M, M′ and F. This is an analytic approximation of the osculating node,
    /// **not** the osculating node itself; against Swiss Ephemeris
    /// `SE_TRUE_NODE` (same frame) it wanders by −0.137°…+0.141° across 2026
    /// (issue #58). The routed chart chain serves the osculating node from
    /// `PackagedDataBackend` instead; this channel remains for direct ELP
    /// consumers as a documented approximation.
```

- [ ] **Step 2: Lunar theory policy**

In `docs/lunar-theory-policy.md`, directly after the existing `**Note:** the osculating true apogee and true perigee ...` paragraph, add:

```markdown
**Note (true node):** the osculating true ascending node is likewise served release-grade by `PackagedDataBackend` (`crates/pleiades-data` osculating path + `crates/pleiades-apsides` Kepler helpers), formed in the mean ecliptic of date from the packaged Moon state and emitted in J2000. It is gated against the Swiss Ephemeris `SE_TRUE_NODE` corpus by `validate-true-node` (measured max longitude residual <L>″ vs ceiling <ceil>″). This ELP backend's own `true node` channel is Meeus's periodic-term-corrected mean node, an analytic approximation that differs from the osculating node by up to ±0.14° (issue #58); it is reached only by direct ELP consumers and is documented, not gated.
```

- [ ] **Step 3: Follow-ups**

In `docs/follow-ups.md` FU-2 entry, replace `requires \`libclang-dev\` + \`LIBCLANG_PATH\` to build Rust bindings to the vendored Swiss Ephemeris.` with `requires \`libclang\` + \`LIBCLANG_PATH\` to build Rust bindings to the vendored Swiss Ephemeris — provided by \`devenv.nix\` (\`devenv shell -- cargo run ...\`), the sanctioned way to get native libraries per \`AGENTS.md\`.`

Append at the end of the file:

```markdown
## FU-12: Osculating true lunar node for `TrueNode` (issue #58)

**Status:** resolved (2026-09-26) · Implemented on `feat/true-node-osculating`
(spec `docs/superpowers/specs/2026-09-26-true-node-osculating-design.md`).
`TrueNode` is now served release-grade by `PackagedDataBackend` as the
osculating ascending node of the geocentric lunar orbit (formed in the mean
ecliptic of date from the packaged Moon state, emitted in J2000; chart layer
applies precession + Δψ only, like the true apsides). Gated against the Swiss
Ephemeris 2.10.03 Moshier `SE_TRUE_NODE` corpus (3177 rows, 1900–2100, same
23-day grid as the Lilith corpus) by `validate-true-node`; gate parity as of
2026-09-26: max longitude residual <L>″, latitude <B>″, distance <D> relative,
vs ceilings <ceilL>″/<ceilB>″/<ceilD>. A blocking-tier regression test pins the
channel to the 8 Moon `SE_NODBIT_OSCU` rows of the nod-aps corpus at ≤40″.
· **Residual, documented not gated:** `ElpBackend`'s own `TrueNode` stays
Meeus's periodic-term-corrected mean node (±0.14° vs the osculating node) for
direct ELP consumers; its evidence rows cannot detect the gap because the 1913
sample is Meeus's own worked value. · **Build-env note:** the reference tool
`tools/se-true-node-reference` builds inside `devenv shell` (clang/libclang
from `devenv.nix`); the gate reads the committed CSV and never rebuilds the
tool. · **Severity:** accuracy (now closed) · **Opened:** 2026-09-26
```

- [ ] **Step 4: README and apsides README**

In `README.md`, after the `True (osculating) Lilith` row add:

```markdown
| True (osculating) Node | [`pleiades-data`](https://docs.rs/pleiades-data) | `validate-true-node` | arcsecond-class (cross-theory) |
```

(Use the accuracy class the measured `<L>` justifies: ≤60″ → `arcsecond-class (cross-theory)`, otherwise `arcminute-class`.)

In `crates/pleiades-apsides/README.md` line 10, after the sentence ending `by \`validate-lilith\` (max longitude residual ~306″).` add: `The same Kepler helpers serve the osculating true node (\`TrueNode\`) through \`PackagedDataBackend\`, gated by \`validate-true-node\`.`

- [ ] **Step 5: Compatibility profile**

In `crates/pleiades-core/src/compatibility/mod.rs`:

1. Line 26: `"pleiades-compatibility-profile/0.7.13"` → `"pleiades-compatibility-profile/0.7.14"`.
2. In `release_notes: &[ ... ]`, after the `"SP-6 (lunar occultations) additions: ..."` entry add:

```rust
            "SP-4-FU (osculating true lunar node, issue #58) additions: CelestialBody::TrueNode is now served release-grade by PackagedDataBackend as the osculating ascending node of the geocentric lunar orbit, formed in the mean ecliptic of date from the packaged Moon state via the shared pleiades-apsides elements_from_state/points_from_elements helpers and emitted in the J2000 boundary frame; the chart layer applies precession + nutation-in-longitude only (geometric direction, like the true apsides). Gated by the fail-closed validate-true-node gate (CLI aliases validate-true-node, true-node-gate), wired into run_all_numeric_gates, over a committed 3177-row Swiss-Ephemeris Moshier (SEFLG_MOSEPH, nutation on) SE_TRUE_NODE corpus, checksum-guarded (fnv1a64) and pinned by row count; measured accuracy: max longitude residual <L>\" (ceiling <ceilL>\"), latitude <B>\" (ceiling <ceilB>\"), distance <D> relative (ceiling <ceilD>) — a cross-theory floor (Moshier vs the DE440-sourced packaged Moon). Honesty caveat: ElpBackend's own TrueNode channel remains Meeus's periodic-term-corrected mean node, an analytic approximation up to ±0.14° from the osculating node, reachable only by direct ELP consumers and documented rather than gated. Compatibility profile bumped to 0.7.14; API stability profile unchanged (purely additive: one new public fn pleiades_data::true_node_body_claims).",
```

3. Append the same sentence (without the surrounding quotes/comma, escaped as the existing string is) to the end of `CURRENT_COMPATIBILITY_PROFILE_SUMMARY`, replacing its trailing `Compatibility profile bumped to 0.7.13; API stability profile unchanged at 0.2.2."` with `Compatibility profile bumped to 0.7.13; API stability profile unchanged at 0.2.2. SP-4-FU (osculating true lunar node, issue #58) additions: ... Compatibility profile bumped to 0.7.14; API stability profile unchanged."`.

Update the two pins:

```bash
sed -i 's#pleiades-compatibility-profile/0\.7\.13#pleiades-compatibility-profile/0.7.14#' \
  crates/pleiades-validate/src/tests/render_request.rs \
  crates/pleiades-cli/src/cli/tests/summary_commands.rs
grep -rn "0\.7\.13" crates/ --include=*.rs
```

Expected: the final grep prints only the historical `0.7.13` mention inside the summary string, nothing in a profile id.

- [ ] **Step 6: Run the claim audits and the full blocking suite**

```bash
cargo run -q -p pleiades-validate -- compat-claims-audit
cargo run -q -p pleiades-validate -- claims-audit
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace -E 'not package(pleiades-validate)'
cargo nextest run -p pleiades-validate -E 'test(/true_node|lilith|nod_aps|render_request|claims/)'
```

Expected: both audits report OK; fmt/clippy clean; all tests pass.

- [ ] **Step 7: Commit**

```bash
git add README.md docs crates/pleiades-elp crates/pleiades-apsides/README.md crates/pleiades-core crates/pleiades-validate crates/pleiades-cli
git commit -m "docs: document osculating TrueNode, ELP fallback ceiling, and bump compatibility profile to 0.7.14 (#58)"
```

---

### Task 6: Whole-branch verification

**Files:** none new.

- [ ] **Step 1: Run the blocking CI task exactly as CI does**

Run: `mise run ci`
Expected: exit 0.

- [ ] **Step 2: Run the full release gate (nightly tier, slow)**

Run: `cargo run -q -p pleiades-validate -- release-gate`
Expected: exit 0 and the output includes the `True-node gate:` summary line.

- [ ] **Step 3: Confirm ELP is bit-identical**

Run: `git diff main -- crates/pleiades-elp/src | grep '^[+-]' | grep -v '^[+-]\s*///' | grep -v '^+++\|^---'`
Expected: no output (only doc-comment lines changed in `pleiades-elp`).

- [ ] **Step 4: Confirm nothing from the reference tool build was committed by accident**

Run: `git status --porcelain; git ls-files tools/se-true-node-reference`
Expected: clean tree; only `Cargo.toml`, `Cargo.lock`, `src/main.rs` listed under the tool.
