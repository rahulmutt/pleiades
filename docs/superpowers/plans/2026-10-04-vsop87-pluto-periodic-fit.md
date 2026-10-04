# Pluto Periodic-Term Fit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Serve Pluto from `Vsop87Backend` via the Meeus Table 37.A periodic-term fit over 1885–2099 (mean elements outside), gated at arcsecond class (issue #129).

**Architecture:** The 43-term table and its evaluation live in a new data module `pleiades-vsop87/src/tables/pluto_meeus.rs`. A small `src/pluto.rs` owns the validity window and the per-instant path choice. `backend.rs` routes Pluto through the fit inside the window and through the existing mean-element orbit outside it, subtracting the same VSOP87B Earth either way. The catalog gains a `PeriodicTermFit` source kind; Pluto's claim becomes `Constrained` like every other body. The validate comparison tolerance and release prose follow.

**Tech Stack:** Rust (workspace toolchain from `mise.toml`), `cargo nextest`, `cargo test`.

**Spec:** `docs/superpowers/specs/2026-10-04-vsop87-pluto-periodic-fit-design.md` (approved 2026-10-04; read §2 "Buckets (amended)").

## Global Constraints

- Window: JD_TT `2_409_542.5` (1885-01-01 0h) inclusive to `2_488_069.5` (2100-01-01 0h) exclusive.
- Outside the window Pluto keeps the mean-element fallback; it never fails closed.
- `Vsop87BodySourceKind` gains `PeriodicTermFit` (label `"published periodic-term fit"`) and becomes `#[non_exhaustive]`; the commit that does this uses a `feat(vsop87)!:` subject.
- "Source-backed" keeps meaning VSOP87B-file-backed; `fallback_body_profiles()` keeps returning exactly `[Pluto]`.
- Pluto claim: `BodyClaim::constrained(Pluto, AccuracyClass::Moderate, ClaimEvidence::AlgorithmicModel)`.
- Pluto quality: `QualityAnnotation::Exact` inside the window, `Approximate` outside.
- Every non-Pluto body's output stays bit-identical.
- Table data module is kept whole (`AGENTS.md`: generated/embedded data in dedicated modules, never reformatted).
- No code copied from pymeeus (LGPL) or soniakeys/meeus (MIT); the coefficients are Meeus Table 37.A, transcription cross-checked against both (all 43 rows agree).
- Run `cargo fmt --all` before every commit (memory: CI fmt gate).
- Worktree sessions refuse heredocs, loops and variable `sed`; use the Edit/Write tools and plain commands.

## Review Focus

1. A request exactly at a window bound (1885-01-01 0h is inside; 2100-01-01 0h is outside) → the path and quality flip exactly there. Pinned in Task 3.
2. A speed sample whose ±0.5-day neighbour crosses a window edge → the speed stays smooth (the centre instant picks the path for all three samples), no 0.6°/day spike. Pinned in Task 3.
3. A TDB-scale request → same path and bit-identical output as TT at the same Julian day. Pinned in Task 3.
4. A batch mixing in-window and out-of-window Pluto instants → each result carries its own quality and equals the single query. Pinned in Task 3.
5. An instant far outside the window (1600-01-01) → finite place and speed, `Approximate`, no error. Pinned in Task 3.

---

### Task 1: Meeus Table 37.A data module

**Files:**
- Create: `crates/pleiades-vsop87/src/tables/pluto_meeus.rs`
- Modify: `crates/pleiades-vsop87/src/tables/mod.rs`
- Test: `crates/pleiades-vsop87/src/tests/pluto.rs` (append)

**Interfaces:**
- Consumes: `crate::tables::vsop87b_earth::SphericalLbr` (`pub(crate) struct` with `pub longitude_rad`, `pub latitude_rad`, `pub radius_au`).
- Produces: `pub(crate) const PLUTO_TERMS: [[f64; 9]; 43]` (columns `i, j, k, lon_a, lon_b, lat_a, lat_b, r_a, r_b`; angles in degrees, radius in AU) and `pub(crate) fn pluto_lbr(jd_tt: f64) -> SphericalLbr` (heliocentric ecliptic, dynamical equinox and ecliptic J2000).

- [ ] **Step 1: Bring the worktree up to date**

Run: `git -c credential.helper= -c credential.helper='!gh auth git-credential' fetch -q origin main && git rebase origin/main`
Expected: rebase succeeds (the branch only adds docs).

- [ ] **Step 2: Write the failing tests**

Append to `crates/pleiades-vsop87/src/tests/pluto.rs`:

```rust
/// Meeus, Astronomical Algorithms (2nd ed.), Example 37.a: 1992-10-13 0h TD.
#[test]
fn meeus_table_37a_reproduces_example_37a() {
    let lbr = crate::tables::pluto_meeus::pluto_lbr(2_448_908.5);
    assert_degrees_close(lbr.longitude_rad.to_degrees().rem_euclid(360.0), 232.740_71, 1e-5);
    assert_close(lbr.latitude_rad.to_degrees(), 14.587_82, 1e-5);
    assert_close(lbr.radius_au, 29.711_111, 1e-6);
}

#[test]
fn meeus_table_37a_coefficients_are_pinned() {
    // FNV-1a/64 over the little-endian IEEE-754 bits of all 387 coefficients,
    // row-major. Pinned from the transcription that agrees row for row with
    // two independent implementations of Table 37.A.
    let terms = &crate::tables::pluto_meeus::PLUTO_TERMS;
    assert_eq!(terms.len(), 43);
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for row in terms {
        for value in row {
            for byte in value.to_bits().to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
    }
    assert_eq!(hash, 0x800f_2aac_998a_fb08);
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p pleiades-vsop87 --lib meeus_table_37a`
Expected: FAIL to compile, "could not find `pluto_meeus` in `tables`".

- [ ] **Step 4: Write the data module**

Create `crates/pleiades-vsop87/src/tables/pluto_meeus.rs`:

```rust
//! Pluto heliocentric position from Meeus, *Astronomical Algorithms*
//! (2nd ed.), chapter 37, Table 37.A: 43 periodic terms in the mean longitudes
//! of Jupiter (J), Saturn (S) and Pluto (P), a fit by J. Chapront to the
//! JPL DE200 ephemeris. Results are heliocentric ecliptic coordinates referred
//! to the dynamical equinox and ecliptic J2000. Meeus quotes 0.6″ in longitude
//! and 0.2″ in latitude over 1885–2099, falling off quickly outside it; the
//! backend uses this path only inside that window (see `crate::pluto`).
//!
//! The coefficients are transcribed from Table 37.A and were checked row for
//! row against two independent implementations of the same table; no code is
//! taken from either. This is embedded data: keep the table whole.

use super::vsop87b_earth::SphericalLbr;

/// Table 37.A, one row per term: argument multipliers `i, j, k` of J, S, P,
/// then the longitude `A, B` (degrees), latitude `A, B` (degrees) and radius
/// vector `A, B` (AU) coefficients of `A sin α + B cos α`.
pub(crate) const PLUTO_TERMS: [[f64; 9]; 43] = [
    [0.0, 0.0, 1.0, -19.799805, 19.850055, -5.452852, -14.974862, 6.6865439, 6.8951812],
    [0.0, 0.0, 2.0, 0.897144, -4.954829, 3.527812, 1.67279, -1.1827535, -0.0332538],
    [0.0, 0.0, 3.0, 0.611149, 1.211027, -1.050748, 0.327647, 0.1593179, -0.143889],
    [0.0, 0.0, 4.0, -0.341243, -0.189585, 0.17869, -0.292153, -0.0018444, 0.048322],
    [0.0, 0.0, 5.0, 0.129287, -0.034992, 0.01865, 0.10034, -0.0065977, -0.0085431],
    [0.0, 0.0, 6.0, -0.038164, 0.030893, -0.030697, -0.025823, 0.0031174, -0.0006032],
    [0.0, 1.0, -1.0, 0.020442, -0.009987, 0.004878, 0.011248, -0.0005794, 0.0022161],
    [0.0, 1.0, 0.0, -0.004063, -0.005071, 0.000226, -0.000064, 0.0004601, 0.0004032],
    [0.0, 1.0, 1.0, -0.006016, -0.003336, 0.00203, -0.000836, -0.0001729, 0.0000234],
    [0.0, 1.0, 2.0, -0.003956, 0.003039, 0.000069, -0.000604, -0.0000415, 0.0000702],
    [0.0, 1.0, 3.0, -0.000667, 0.003572, -0.000247, -0.000567, 0.0000239, 0.0000723],
    [0.0, 2.0, -2.0, 0.001276, 0.000501, -0.000057, 0.000001, 0.0000067, -0.0000067],
    [0.0, 2.0, -1.0, 0.001152, -0.000917, -0.000122, 0.000175, 0.0001034, -0.0000451],
    [0.0, 2.0, 0.0, 0.00063, -0.001277, -0.000049, -0.000164, -0.0000129, 0.0000504],
    [1.0, -1.0, 0.0, 0.002571, -0.000459, -0.000197, 0.000199, 0.000048, -0.0000231],
    [1.0, -1.0, 1.0, 0.000899, -0.001449, -0.000025, 0.000217, 0.0000002, -0.0000441],
    [1.0, 0.0, -3.0, -0.001016, 0.001043, 0.000589, -0.000248, -0.0003359, 0.0000265],
    [1.0, 0.0, -2.0, -0.002343, -0.001012, -0.000269, 0.000711, 0.0007856, -0.0007832],
    [1.0, 0.0, -1.0, 0.007042, 0.000788, 0.000185, 0.000193, 0.0000036, 0.0045763],
    [1.0, 0.0, 0.0, 0.001199, -0.000338, 0.000315, 0.000807, 0.0008663, 0.0008547],
    [1.0, 0.0, 1.0, 0.000418, -0.000067, -0.00013, -0.000043, -0.0000809, -0.0000769],
    [1.0, 0.0, 2.0, 0.00012, -0.000274, 0.000005, 0.000003, 0.0000263, -0.0000144],
    [1.0, 0.0, 3.0, -0.00006, -0.000159, 0.000002, 0.000017, -0.0000126, 0.0000032],
    [1.0, 0.0, 4.0, -0.000082, -0.000029, 0.000002, 0.000005, -0.0000035, -0.0000016],
    [1.0, 1.0, -3.0, -0.000036, -0.000029, 0.000002, 0.000003, -0.0000019, -0.0000004],
    [1.0, 1.0, -2.0, -0.00004, 0.000007, 0.000003, 0.000001, -0.0000015, 0.0000008],
    [1.0, 1.0, -1.0, -0.000014, 0.000022, 0.000002, -0.000001, -0.0000004, 0.0000012],
    [1.0, 1.0, 0.0, 0.000004, 0.000013, 0.000001, -0.000001, 0.0000005, 0.0000006],
    [1.0, 1.0, 1.0, 0.000005, 0.000002, 0.0, -0.000001, 0.0000003, 0.0000001],
    [1.0, 1.0, 3.0, -0.000001, 0.0, 0.0, 0.0, 0.0000006, -0.0000002],
    [2.0, 0.0, -6.0, 0.000002, 0.0, 0.0, -0.000002, 0.0000002, 0.0000002],
    [2.0, 0.0, -5.0, -0.000004, 0.000005, 0.000002, 0.000002, -0.0000002, -0.0000002],
    [2.0, 0.0, -4.0, 0.000004, -0.000007, -0.000007, 0.0, 0.0000014, 0.0000013],
    [2.0, 0.0, -3.0, 0.000014, 0.000024, 0.00001, -0.000008, -0.0000063, 0.0000013],
    [2.0, 0.0, -2.0, -0.000049, -0.000034, -0.000003, 0.00002, 0.0000136, -0.0000236],
    [2.0, 0.0, -1.0, 0.000163, -0.000048, 0.000006, 0.000005, 0.0000273, 0.0001065],
    [2.0, 0.0, 0.0, 0.000009, -0.000024, 0.000014, 0.000017, 0.0000251, 0.0000149],
    [2.0, 0.0, 1.0, -0.000004, 0.000001, -0.000002, 0.0, -0.0000025, -0.0000009],
    [2.0, 0.0, 2.0, -0.000003, 0.000001, 0.0, 0.0, 0.0000009, -0.0000002],
    [2.0, 0.0, 3.0, 0.000001, 0.000003, 0.0, 0.0, -0.0000008, 0.0000007],
    [3.0, 0.0, -2.0, -0.000003, -0.000001, 0.0, 0.000001, 0.0000002, -0.000001],
    [3.0, 0.0, -1.0, 0.000005, -0.000003, 0.0, 0.0, 0.0000019, 0.0000035],
    [3.0, 0.0, 0.0, 0.0, 0.0, 0.000001, 0.0, 0.000001, 0.0000003],
];

/// Heliocentric ecliptic longitude, latitude (radians) and radius vector (AU)
/// of Pluto at `jd_tt`, from Table 37.A (Meeus eq. 37 and the text before it).
pub(crate) fn pluto_lbr(jd_tt: f64) -> SphericalLbr {
    let t = (jd_tt - 2_451_545.0) / 36_525.0;
    let jupiter = 34.35 + 3_034.905_7 * t;
    let saturn = 50.08 + 1_222.113_8 * t;
    let pluto = 238.96 + 144.96 * t;
    let (mut longitude, mut latitude, mut radius) = (0.0_f64, 0.0_f64, 0.0_f64);
    for [i, j, k, lon_a, lon_b, lat_a, lat_b, r_a, r_b] in PLUTO_TERMS {
        let (sin_a, cos_a) = (i * jupiter + j * saturn + k * pluto).to_radians().sin_cos();
        longitude += lon_a * sin_a + lon_b * cos_a;
        latitude += lat_a * sin_a + lat_b * cos_a;
        radius += r_a * sin_a + r_b * cos_a;
    }
    SphericalLbr {
        longitude_rad: (238.958_116 + 144.96 * t + longitude).to_radians(),
        latitude_rad: (-3.908_239 + latitude).to_radians(),
        radius_au: 40.724_134_6 + radius,
    }
}
```

In `crates/pleiades-vsop87/src/tables/mod.rs`, update the module doc's first line to `//! Embedded planetary coefficient table modules.`, add a sentence after the existing paragraph: `//! \`pluto_meeus\` holds Meeus Table 37.A for Pluto, which the VSOP87 files exclude.`, and add `pub(crate) mod pluto_meeus;` above `pub(crate) mod vsop87b_earth;`.

If rustfmt reflows the table rows, that is acceptable: rustfmt output is the canonical form for a hand-written `const` (there is no regenerator for this table); the checksum test pins the values, not the layout.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p pleiades-vsop87 --lib meeus_table_37a`
Expected: PASS (2 tests). A `dead_code` warning for `pluto_lbr` outside tests is expected until Task 3; if `-D warnings` is in effect for your build, add `#[cfg_attr(not(test), allow(dead_code))]` to `pluto_lbr` and `PLUTO_TERMS` now and remove it in Task 3 Step 4.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add crates/pleiades-vsop87/src/tables/pluto_meeus.rs crates/pleiades-vsop87/src/tables/mod.rs crates/pleiades-vsop87/src/tests/pluto.rs
git commit -m "feat(vsop87): embed Meeus Table 37.A for Pluto (#129)"
```

---

### Task 2: `PeriodicTermFit` source kind and bucket helper

Behaviour-neutral: Pluto still uses `MeanOrbitalElements` after this task.

**Files:**
- Modify: `crates/pleiades-vsop87/src/profiles.rs:9-37` (enum, label), `:185-218` (bucket filters and docs)
- Modify: `crates/pleiades-vsop87/src/backend.rs` (quality `match` in `position`)
- Test: `crates/pleiades-vsop87/src/tests/documentation.rs` (label cases near line 1100), `crates/pleiades-vsop87/src/tests/profiles.rs` (append)

**Interfaces:**
- Produces: `Vsop87BodySourceKind::PeriodicTermFit`; `pub const fn is_vsop87b(self) -> bool` on `Vsop87BodySourceKind` (true for `TruncatedVsop87b | VendoredVsop87b | GeneratedBinaryVsop87b`).

- [ ] **Step 1: Write the failing tests**

In `crates/pleiades-vsop87/src/tests/documentation.rs`, add to the `cases` array that holds the `(kind, label)` pairs (after the `MeanOrbitalElements` pair):

```rust
        (
            Vsop87BodySourceKind::PeriodicTermFit,
            "published periodic-term fit",
        ),
```

Append to `crates/pleiades-vsop87/src/tests/profiles.rs`:

```rust
#[test]
fn only_vsop87b_kinds_count_as_source_backed() {
    assert!(Vsop87BodySourceKind::TruncatedVsop87b.is_vsop87b());
    assert!(Vsop87BodySourceKind::VendoredVsop87b.is_vsop87b());
    assert!(Vsop87BodySourceKind::GeneratedBinaryVsop87b.is_vsop87b());
    assert!(!Vsop87BodySourceKind::MeanOrbitalElements.is_vsop87b());
    assert!(!Vsop87BodySourceKind::PeriodicTermFit.is_vsop87b());
    assert!(source_backed_body_profiles()
        .iter()
        .all(|profile| profile.kind.is_vsop87b()));
    assert!(fallback_body_profiles()
        .iter()
        .all(|profile| !profile.kind.is_vsop87b()));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p pleiades-vsop87 --lib -- only_vsop87b_kinds source_kind`
Expected: FAIL to compile, "no variant named `PeriodicTermFit`".

- [ ] **Step 3: Implement**

In `profiles.rs`, put `#[non_exhaustive]` on the enum (below the `#[derive(...)]` line), add after `MeanOrbitalElements`:

```rust
    /// Heliocentric spherical coordinates are evaluated from a published
    /// periodic-term fit inside its validity window (Pluto: Meeus Table 37.A,
    /// 1885–2099), with the mean-element orbit outside it.
    PeriodicTermFit,
```

Add the label arm `Self::PeriodicTermFit => "published periodic-term fit",` and, inside the same `impl Vsop87BodySourceKind`:

```rust
    /// Whether this kind is evaluated from a vendored VSOP87B source file, the
    /// split between [`source_backed_body_profiles`] and
    /// [`fallback_body_profiles`].
    pub const fn is_vsop87b(self) -> bool {
        matches!(
            self,
            Self::TruncatedVsop87b | Self::VendoredVsop87b | Self::GeneratedBinaryVsop87b
        )
    }
```

Rewrite the two filters and their docs:

```rust
/// Returns the source-backed VSOP87 body profiles used by [`crate::Vsop87Backend`]:
/// the bodies evaluated from a vendored VSOP87B source file (Sun through
/// Neptune), the reproducibility subset the table regenerator covers.
pub fn source_backed_body_profiles() -> Vec<Vsop87BodySource> {
    body_catalog_entries()
        .iter()
        .filter(|entry| entry.source_profile.kind.is_vsop87b())
        .map(|entry| entry.source_profile.clone())
        .collect()
}
```

```rust
/// Returns the VSOP87 backend's body profiles outside the VSOP87B files.
///
/// The VSOP87 theory excludes Pluto, so Pluto is the one body here; it is
/// served from a published periodic-term fit (or mean elements outside the
/// fit's window), not from a VSOP87B source file.
pub fn fallback_body_profiles() -> Vec<Vsop87BodySource> {
    body_catalog_entries()
        .iter()
        .filter(|entry| !entry.source_profile.kind.is_vsop87b())
        .map(|entry| entry.source_profile.clone())
        .collect()
}
```

Update `source_backed_body_order`'s doc comment "excluding the mean-element Pluto fallback" to "excluding Pluto, which the VSOP87B files do not cover".

In `backend.rs` `position`, extend the quality `match` with `| Some(Vsop87BodySourceKind::PeriodicTermFit)` in the `Approximate` arm (Task 3 makes it window-aware).

- [ ] **Step 4: Run the crate tests**

Run: `cargo test -p pleiades-vsop87`
Expected: PASS (all; outputs unchanged).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add crates/pleiades-vsop87/src/profiles.rs crates/pleiades-vsop87/src/backend.rs crates/pleiades-vsop87/src/tests/documentation.rs crates/pleiades-vsop87/src/tests/profiles.rs
git commit -m "feat(vsop87)!: add the PeriodicTermFit source kind; make Vsop87BodySourceKind non_exhaustive (#129)"
```

---

### Task 3: Route Pluto through the fit inside its window

**Files:**
- Create: `crates/pleiades-vsop87/src/pluto.rs`
- Modify: `crates/pleiades-vsop87/src/lib.rs` (add `mod pluto;`, crate docs lines 28-47)
- Modify: `crates/pleiades-vsop87/src/backend.rs` (`vsop87_body_claims`, `geocentric_coordinates`, `motion`, `position` quality)
- Modify: `crates/pleiades-vsop87/src/profiles.rs` (Pluto catalog entry, ~line 541)
- Modify: `crates/pleiades-vsop87/Cargo.toml` (dev-dependency)
- Test: `crates/pleiades-vsop87/src/tests/pluto.rs` (rewrite the module doc and the SE test; append), `src/tests/backend.rs`, `src/tests/profiles.rs`, `src/tests/evidence.rs`, `src/tests/documentation.rs`

**Interfaces:**
- Consumes: `crate::tables::pluto_meeus::pluto_lbr` (Task 1); `Vsop87BodySourceKind::PeriodicTermFit` (Task 2).
- Produces: `crate::pluto::{PLUTO_FIT_START_JD, PLUTO_FIT_END_JD, PlutoPath}` with `PlutoPath::for_julian_day(jd_tt: f64) -> PlutoPath` (`PeriodicTermFit` or `MeanElements`); `Vsop87Backend::geocentric_coordinates_on(body: CelestialBody, days: f64, pluto_path: PlutoPath) -> Option<HeliocentricCoordinates>`.

- [ ] **Step 1: Write the failing tests**

Replace the module doc and the constants/test that follow it in `crates/pleiades-vsop87/src/tests/pluto.rs` (keep `mean_element_orbit_longitude_includes_the_argument_of_perihelion` and the two Task 1 tests):

```rust
//! Pluto: the Meeus Table 37.A periodic-term fit inside 1885–2099 (issue
//! #129) and the mean-element orbit outside it (issue #119).

use super::*;
use crate::elements::OrbitalElements;
use crate::pluto::{PlutoPath, PLUTO_FIT_END_JD, PLUTO_FIT_START_JD};

/// Swiss Ephemeris 2.10 (Moshier) geometric J2000 ecliptic place of Pluto,
/// quoted in issue #119: `(JD TT, longitude deg, latitude deg, distance AU)`.
/// The longitudes are rounded to 0.01°; the latitude and distance are the
/// issue's apparent-of-date values, which differ from J2000 geometric by
/// seconds of arc and the fourth decimal of an AU.
const SWISS_EPHEMERIS_PLUTO_J2000: [(f64, f64, f64, f64); 2] = [
    (2_451_545.0, 251.46, 10.8552, 31.064),
    (2_460_763.5, 303.16, -3.4414, 35.639),
];

#[test]
fn pluto_fit_lands_on_the_rounded_swiss_ephemeris_places() {
    let backend = Vsop87Backend::new();
    for (jd_tt, lon, lat, dist) in SWISS_EPHEMERIS_PLUTO_J2000 {
        let instant = Instant::new(pleiades_types::JulianDay::from_days(jd_tt), TimeScale::Tt);
        let result = backend
            .position(&mean_request_at(CelestialBody::Pluto, instant))
            .expect("Pluto query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        assert_eq!(result.quality, QualityAnnotation::Exact);
        assert_degrees_close(ecliptic.longitude.degrees(), lon, 0.01);
        assert_close(ecliptic.latitude.degrees(), lat, 0.01);
        assert_close(ecliptic.distance_au.expect("distance should exist"), dist, 0.01);
    }
}

/// Ceilings for the packaged-reference sweep: ceil(1.5 × measured max), measured
/// 2026-10-04 on a 10-day grid over 1900–2099 (7303 samples): 3.29″ longitude,
/// 0.34″ latitude, 3.27e-4 AU. The packaged (DE440-fitted) Pluto is itself about
/// 1″ from Swiss Ephemeris (`validate-helio-position`).
const SWEEP_LON_CEILING_ARCSEC: f64 = 5.0;
const SWEEP_LAT_CEILING_ARCSEC: f64 = 0.6;
const SWEEP_DIST_CEILING_AU: f64 = 5.0e-4;

#[test]
fn pluto_fit_tracks_the_packaged_pluto_across_1900_2099() {
    // Regression for issue #129: the mean-element Pluto sat 0.4-0.6° from Swiss
    // Ephemeris at every epoch (35.7′ max over 1972-2099).
    let vsop87 = Vsop87Backend::new();
    let packaged = pleiades_data::packaged_backend();
    let (mut max_lon, mut max_lat, mut max_dist) = (0.0_f64, 0.0_f64, 0.0_f64);
    let mut samples = 0_usize;
    let mut jd = 2_415_030.5; // 1900-01-10, clear of the packaged window edge
    while jd < 2_488_060.0 {
        let instant = Instant::new(pleiades_types::JulianDay::from_days(jd), TimeScale::Tt);
        let ours = vsop87
            .position(&mean_request_at(CelestialBody::Pluto, instant))
            .expect("VSOP87 Pluto")
            .ecliptic
            .expect("ecliptic");
        let reference = packaged
            .position(&mean_request_at(CelestialBody::Pluto, instant))
            .expect("packaged Pluto")
            .ecliptic
            .expect("ecliptic");
        let lon = signed_longitude_delta_degrees(
            reference.longitude.degrees(),
            ours.longitude.degrees(),
        )
        .abs()
            * 3600.0;
        let lat = (ours.latitude.degrees() - reference.latitude.degrees()).abs() * 3600.0;
        let dist = (ours.distance_au.expect("distance") - reference.distance_au.expect("distance")).abs();
        max_lon = max_lon.max(lon);
        max_lat = max_lat.max(lat);
        max_dist = max_dist.max(dist);
        samples += 1;
        jd += 30.0;
    }
    let summary = format!(
        "Pluto vs packaged over {samples} samples: max lon {max_lon:.3}″, lat {max_lat:.3}″, dist {max_dist:.2e} AU"
    );
    // Printed so the ceilings can be re-derived, as the corpus gates do.
    eprintln!("{summary}");
    assert!(samples >= 2_430, "only {samples} samples");
    assert!(
        max_lon <= SWEEP_LON_CEILING_ARCSEC
            && max_lat <= SWEEP_LAT_CEILING_ARCSEC
            && max_dist <= SWEEP_DIST_CEILING_AU,
        "{summary}"
    );
}

fn pluto_at(jd_tt: f64, scale: TimeScale) -> pleiades_backend::EphemerisResult {
    let instant = Instant::new(pleiades_types::JulianDay::from_days(jd_tt), scale);
    Vsop87Backend::new()
        .position(&mean_request_at(CelestialBody::Pluto, instant))
        .expect("Pluto query should work")
}

#[test]
fn the_window_starts_at_1885_inclusive_and_ends_at_2100_exclusive() {
    assert_eq!(PLUTO_FIT_START_JD, 2_409_542.5);
    assert_eq!(PLUTO_FIT_END_JD, 2_488_069.5);
    let cases = [
        (PLUTO_FIT_START_JD - 1e-4, PlutoPath::MeanElements, QualityAnnotation::Approximate),
        (PLUTO_FIT_START_JD, PlutoPath::PeriodicTermFit, QualityAnnotation::Exact),
        (PLUTO_FIT_END_JD - 1e-4, PlutoPath::PeriodicTermFit, QualityAnnotation::Exact),
        (PLUTO_FIT_END_JD, PlutoPath::MeanElements, QualityAnnotation::Approximate),
    ];
    for (jd, path, quality) in cases {
        assert_eq!(PlutoPath::for_julian_day(jd), path, "{jd}");
        assert_eq!(pluto_at(jd, TimeScale::Tt).quality, quality, "{jd}");
    }
}

#[test]
fn speed_near_a_window_edge_does_not_difference_across_the_jump() {
    // At 2099-12-31 12h the +0.5-day neighbour is outside the window; the centre
    // instant picks the fit for all three samples, so the speed matches a day
    // earlier to well under the ~0.6°/day a jump between paths would add.
    for (edge_side, interior) in [
        (PLUTO_FIT_END_JD - 0.5, PLUTO_FIT_END_JD - 1.5),
        (PLUTO_FIT_START_JD + 0.25, PLUTO_FIT_START_JD + 1.25),
    ] {
        let near = pluto_at(edge_side, TimeScale::Tt).motion.expect("motion");
        let inside = pluto_at(interior, TimeScale::Tt).motion.expect("motion");
        let delta = (near.longitude_deg_per_day.expect("speed")
            - inside.longitude_deg_per_day.expect("speed"))
        .abs();
        assert!(delta < 1e-3, "speed jumped by {delta}°/day near {edge_side}");
    }
}

#[test]
fn tdb_and_tt_requests_take_the_same_path_bit_for_bit() {
    for jd in [PLUTO_FIT_START_JD, 2_451_545.0, PLUTO_FIT_END_JD] {
        let tt = pluto_at(jd, TimeScale::Tt);
        let tdb = pluto_at(jd, TimeScale::Tdb);
        assert_eq!(tt.quality, tdb.quality, "{jd}");
        assert_eq!(tt.ecliptic, tdb.ecliptic, "{jd}");
        assert_eq!(tt.motion, tdb.motion, "{jd}");
    }
}

#[test]
fn a_batch_mixing_window_sides_keeps_per_instant_quality() {
    let backend = Vsop87Backend::new();
    let requests = [PLUTO_FIT_START_JD - 10.0, 2_451_545.0, PLUTO_FIT_END_JD + 10.0]
        .map(|jd| {
            mean_request_at(
                CelestialBody::Pluto,
                Instant::new(pleiades_types::JulianDay::from_days(jd), TimeScale::Tt),
            )
        });
    let batch = backend.positions(&requests).expect("batch");
    let qualities = batch.iter().map(|r| r.quality).collect::<Vec<_>>();
    assert_eq!(
        qualities,
        vec![
            QualityAnnotation::Approximate,
            QualityAnnotation::Exact,
            QualityAnnotation::Approximate
        ]
    );
    for (request, result) in requests.iter().zip(&batch) {
        let single = backend.position(request).expect("single");
        assert_eq!(single.ecliptic, result.ecliptic);
        assert_eq!(single.quality, result.quality);
    }
}

#[test]
fn far_outside_the_window_pluto_is_still_served_approximately() {
    let result = pluto_at(2_305_447.5, TimeScale::Tt); // 1600-01-01
    assert_eq!(result.quality, QualityAnnotation::Approximate);
    let ecliptic = result.ecliptic.expect("ecliptic");
    assert!(ecliptic.longitude.degrees().is_finite());
    assert!(ecliptic.distance_au.expect("distance").is_finite());
    assert!(result
        .motion
        .expect("motion")
        .longitude_deg_per_day
        .expect("speed")
        .is_finite());
}
```

In `crates/pleiades-vsop87/src/tests/backend.rs`:
- In each `match result.body { CelestialBody::Pluto => { assert_eq!(result.quality, QualityAnnotation::Approximate); } _ => { assert_eq!(result.quality, QualityAnnotation::Exact); } }` block (around lines 266, 348, 575, 636, 697, 758, 859; all query at J2000 or the corpus epochs, inside the window), replace the whole `match` with `assert_eq!(result.quality, QualityAnnotation::Exact);`. If one of those blocks queries an instant outside 1885–2099, keep the Pluto arm for it and say so in a comment.
- Rename `vsop87_claims_majors_constrained_pluto_approximate` to `vsop87_claims_every_body_constrained` and change its Pluto expectation to `Some(BodyClaimTier::Constrained)`.

In `crates/pleiades-vsop87/src/tests/profiles.rs` (around line 144) change the Pluto expectations to:

```rust
    assert_eq!(pluto.kind, Vsop87BodySourceKind::PeriodicTermFit);
    assert!(pluto.provenance.contains("Table 37.A"));
    assert_eq!(pluto.summary_line(), pluto.to_string());
    assert!(pluto
        .summary_line()
        .starts_with("Pluto: kind=published periodic-term fit, accuracy=Exact"));
```

and in `body_source_profiles_validate_the_current_catalog_pairings` change the drift match so it still produces a different kind:

```rust
    kind_drift.kind = match kind_drift.kind {
        Vsop87BodySourceKind::PeriodicTermFit => Vsop87BodySourceKind::MeanOrbitalElements,
        _ => Vsop87BodySourceKind::PeriodicTermFit,
    };
```

In `crates/pleiades-vsop87/src/tests/evidence.rs`:
- `source_backed_and_fallback_body_profiles_are_exposed_for_reproducibility_tooling`: replace the two `MeanOrbitalElements` assertions with `assert!(fallback_profiles.iter().all(|profile| !profile.kind.is_vsop87b()));` and `assert!(source_backed_profiles.iter().all(|profile| profile.kind.is_vsop87b()));`.
- `unified_body_catalog_keeps_profiles_specs_and_samples_aligned`: the `fallback` filter becomes `.filter(|entry| !entry.source_profile.kind.is_vsop87b())`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p pleiades-vsop87 --lib pluto`
Expected: FAIL to compile ("unresolved import `crate::pluto`"; `pleiades_data` not found).

- [ ] **Step 3: Implement**

Add to `crates/pleiades-vsop87/Cargo.toml` under `[dev-dependencies]` (create the table if absent): `pleiades-data = { path = "../pleiades-data" }`. (`pleiades-data` does not depend on `pleiades-vsop87`; verify with `grep -n pleiades-vsop87 crates/pleiades-data/Cargo.toml crates/pleiades-jpl/Cargo.toml` → no output.)

Create `crates/pleiades-vsop87/src/pluto.rs`:

```rust
//! Which source serves Pluto at a given instant (issue #129).
//!
//! The VSOP87 theory excludes Pluto. Inside the validity window of Meeus
//! Table 37.A (1885–2099) the backend evaluates that periodic-term fit;
//! outside it, the mean-element orbit (issue #119). The fit degrades quickly
//! outside its window, and rejecting out-of-window instants would break an
//! algorithmic backend whose nominal range is unbounded, so the fallback
//! stays. At each edge Pluto jumps by up to about 0.6°: a root finder that
//! spans an edge sees the jump. A speed sample takes the path of its centre
//! instant for all of its samples, so speeds stay smooth up to the edge.

/// First instant served by the fit: 1885-01-01 0h TT (inclusive).
pub(crate) const PLUTO_FIT_START_JD: f64 = 2_409_542.5;
/// End of the fit's window: 2100-01-01 0h TT (exclusive).
pub(crate) const PLUTO_FIT_END_JD: f64 = 2_488_069.5;

/// The source that serves Pluto at an instant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlutoPath {
    /// Meeus Table 37.A.
    PeriodicTermFit,
    /// The mean-element Keplerian orbit.
    MeanElements,
}

impl PlutoPath {
    /// The path for a TT/TDB Julian day.
    pub(crate) fn for_julian_day(jd_tt: f64) -> Self {
        if (PLUTO_FIT_START_JD..PLUTO_FIT_END_JD).contains(&jd_tt) {
            Self::PeriodicTermFit
        } else {
            Self::MeanElements
        }
    }
}
```

In `crates/pleiades-vsop87/src/lib.rs` add `mod pluto;` next to the other private `mod` lines, and replace the crate-doc paragraph "Pluto still uses compact Keplerian orbital elements, ... added incrementally." with:

```rust
//! Pluto, which the VSOP87 theory excludes, comes from the Meeus Table 37.A
//! periodic-term fit over 1885–2099 (arcsecond class) and from mean Keplerian
//! orbital elements outside that window, minus the same VSOP87B Earth.
```

In `backend.rs`:

1. Imports: add `use crate::pluto::PlutoPath;`.
2. `vsop87_body_claims`: drop the Pluto arm, so every body maps to `BodyClaim::constrained(body, AccuracyClass::Moderate, ClaimEvidence::AlgorithmicModel)`.
3. Rename the existing `fn geocentric_coordinates(body, days)` to `fn geocentric_coordinates_on(body: CelestialBody, days: f64, pluto_path: PlutoPath) -> Option<HeliocentricCoordinates>`, and replace its trailing mean-element fallback block with:

```rust
        // Pluto, which VSOP87 excludes: Meeus Table 37.A inside its window,
        // the mean-element orbit outside it; either way minus the same
        // VSOP87B heliocentric Earth the table-backed planets use.
        let earth = Self::heliocentric_earth_from_vsop87b(days);
        let target = match (&body, pluto_path) {
            (CelestialBody::Pluto, PlutoPath::PeriodicTermFit) => {
                let pluto = tables::pluto_meeus::pluto_lbr(J2000 + days);
                spherical_lbr_to_cartesian(pluto.longitude_rad, pluto.latitude_rad, pluto.radius_au)
            }
            _ => Self::heliocentric_coordinates(Self::orbital_elements(body, days)?),
        };
        Some(HeliocentricCoordinates {
            xh: target.xh - earth.xh,
            yh: target.yh - earth.yh,
            zh: target.zh - earth.zh,
        })
```

   Then add the wrapper that keeps every existing caller unchanged:

```rust
    fn geocentric_coordinates(body: CelestialBody, days: f64) -> Option<HeliocentricCoordinates> {
        Self::geocentric_coordinates_on(body, days, PlutoPath::for_julian_day(J2000 + days))
    }
```

4. `motion`: compute `let pluto_path = PlutoPath::for_julian_day(J2000 + days);` before the samples and use `Self::geocentric_coordinates_on(body.clone(), days - HALF_SPAN_DAYS, pluto_path)` and `Self::geocentric_coordinates_on(body, days + HALF_SPAN_DAYS, pluto_path)`.
5. `position` quality: replace the `PeriodicTermFit` part of the `Approximate` arm with its own arm:

```rust
            Some(Vsop87BodySourceKind::PeriodicTermFit) => {
                match PlutoPath::for_julian_day(J2000 + days) {
                    PlutoPath::PeriodicTermFit => QualityAnnotation::Exact,
                    PlutoPath::MeanElements => QualityAnnotation::Approximate,
                }
            }
```

In `profiles.rs`, replace the Pluto catalog entry's `source_profile(...)` arguments with:

```rust
                source_profile(
                    CelestialBody::Pluto,
                    Vsop87BodySourceKind::PeriodicTermFit,
                    "Meeus Astronomical Algorithms Table 37.A periodic-term fit (Chapront, DE200), valid 1885-2099; mean-element fallback outside",
                    AccuracyClass::Exact,
                ),
```

keeping `source_specification: None` and `canonical_sample: None`. Update the `Vsop87BodySource::accuracy` field doc ("while `Approximate` marks the mean-element Pluto fallback") to "`Exact` also marks Pluto's full published Table 37.A".

If Task 1 added `allow(dead_code)` attributes, remove them now.

- [ ] **Step 4: Run the crate tests**

Run: `cargo test -p pleiades-vsop87`
Expected: PASS. If `pluto_fit_tracks_the_packaged_pluto_across_1900_2099` fails, the message prints the maxima: do not raise a ceiling; investigate (frame, Earth subtraction, table transcription) first.

- [ ] **Step 5: Confirm the other bodies are bit-identical**

Run: `cargo test -p pleiades-vsop87 --lib evidence`
Expected: PASS. These tests pin every other body's canonical J2000 sample and the `source_documentation_summary` counts (8 source-backed, 1 fallback), none of which this task may move. (Do not use `git stash` to A/B: the stash stack is shared with other sessions.)

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add crates/pleiades-vsop87
git commit -m "feat(vsop87): serve Pluto from Meeus Table 37.A over 1885-2099 (#129)"
```

---

### Task 4: Validation tolerance and release prose

**Files:**
- Modify: `crates/pleiades-validate/src/lib.rs:556-558`
- Modify: `crates/pleiades-validate/src/comparison/tolerance.rs:99-111`
- Modify: `crates/pleiades-validate/src/comparison/report.rs:754-760`
- Modify: `crates/pleiades-validate/src/posture/backend_policy.rs:47-57`
- Modify: `crates/pleiades-validate/src/crossings_validation.rs:48-52`
- Test (pinned strings): `crates/pleiades-validate/src/tests/{comparison.rs,report.rs,release_checklist.rs,render_request.rs,snapshot_render.rs,release_bundle_verify_a.rs,release_bundle_verify_b.rs}`, `crates/pleiades-cli/src/cli/tests/{validation.rs,summary_commands.rs}`

**Interfaces:**
- Consumes: Task 3's Pluto output (arcsecond class against the JPL snapshot rows, all inside 1885–2099).

- [ ] **Step 1: Change the sources**

`lib.rs`:

```rust
const PLUTO_LONGITUDE_THRESHOLD_DEG: f64 = 0.01;
const PLUTO_LATITUDE_THRESHOLD_DEG: f64 = 0.01;
const PLUTO_DISTANCE_THRESHOLD_AU: f64 = 0.001;
```

`tolerance.rs`: variant doc `/// Pluto scope: the Meeus Table 37.A periodic-term fit on the algorithmic path.` and label `Self::Pluto => "Pluto (periodic-term fit)",`.

`report.rs`: `profile: "phase-1 Pluto periodic-term fit evidence",`.

`backend_policy.rs`: replace the doc paragraph's "treats Pluto as an explicitly approximate fallback" with "serves Pluto from the Meeus Table 37.A periodic-term fit over 1885–2099 (mean elements outside)", and the constant with:

```rust
pub const CURRENT_PLUTO_FALLBACK_POLICY_SUMMARY_TEXT: &str =
    "Pluto comes from the Meeus Table 37.A periodic-term fit on the algorithmic (VSOP87) path over 1885-2099, with mean elements outside; the packaged-data artifact ships Pluto as release-grade";
```

`crossings_validation.rs:48-52`: replace the comment with:

```rust
// Pluto meets a normal measured per-body ceiling like every other body (not a coverage
// boundary or an exclusion). This gate runs on the packaged backend, whose Pluto is
// fitted from JPL like every other body; the ceiling is simply wider than the inner
// planets' — measured max 0.697" (geo) / 3.530" (helio) — at Pluto's own 1.4x value.
```

- [ ] **Step 2: Update every pinned string**

Run each search and replace the old text with the new text in every hit:

```bash
grep -rn "Pluto fallback (approximate)" crates
grep -rn "Δlon≤45.000°, Δlat≤1.000°, Δdist=0.250 AU" crates
grep -rn "phase-1 Pluto approximate fallback evidence" crates
grep -rn "explicitly approximate fallback on the algorithmic" crates
grep -rn "kind=mean orbital elements fallback, accuracy=Approximate" crates
```

| old | new |
|---|---|
| `Pluto fallback (approximate)` | `Pluto (periodic-term fit)` |
| `Δlon≤45.000°, Δlat≤1.000°, Δdist=0.250 AU` | `Δlon≤0.010°, Δlat≤0.010°, Δdist=0.001 AU` |
| `phase-1 Pluto approximate fallback evidence` | `phase-1 Pluto periodic-term fit evidence` |
| `Pluto remains an explicitly approximate fallback on the algorithmic (VSOP87) path; the packaged-data artifact ships Pluto as release-grade` | the new `CURRENT_PLUTO_FALLBACK_POLICY_SUMMARY_TEXT` above |
| `kind=mean orbital elements fallback, accuracy=Approximate, current approximate mean-element fallback special case until a Pluto-specific source path is selected` | `kind=published periodic-term fit, accuracy=Exact, Meeus Astronomical Algorithms Table 37.A periodic-term fit (Chapront, DE200), valid 1885-2099; mean-element fallback outside` |

Also search the pinned "limit Δlon≤45.000000°" form: `grep -rn "limit Δlon≤45.000000°" crates` → replace with `limit Δlon≤0.010000°`, and the matching `limit Δlat≤1.000000°` → `limit Δlat≤0.010000°`, `limit Δdist=0.250000 AU` → `limit Δdist=0.001000 AU` on Pluto lines only.

- [ ] **Step 3: Run the affected suites, including the slow tier**

Run:
```bash
cargo test -p pleiades-validate --lib -- --include-ignored comparison report release_checklist render_request snapshot_render release_bundle_verify crossings
cargo test -p pleiades-cli
```
Expected: PASS. A failure that prints a rendered report is a pinned string this task missed: fix the expectation to the new text, never the source text back. If a Pluto comparison row now exceeds 0.01° against the JPL snapshot, stop and report the row: it contradicts the Task 3 sweep.

- [ ] **Step 4: Commit**

```bash
cargo fmt --all
git add crates/pleiades-validate crates/pleiades-cli
git commit -m "test(validate): hold the algorithmic Pluto to the planet comparison tolerance (#129)"
```

---

### Task 5: Documentation and compatibility profile

**Files:**
- Modify: `crates/pleiades-vsop87/README.md` (lines 7 and 13), `crates/pleiades-vsop87/Cargo.toml` (`description`)
- Modify: `README.md:54-55` and `:114`
- Modify: `docs/follow-ups.md` (append FU-26)
- Modify: `plan/status/02-next-slice-candidates.md:42`
- Modify: `crates/pleiades-core/src/compatibility/mod.rs` (profile id, checksum, additions entry)
- Test (pinned id): `crates/pleiades-cli/src/cli/tests/summary_commands.rs`, `crates/pleiades-validate/src/tests/render_request.rs`

- [ ] **Step 1: Prose**

- Crate README line 7: "with generated binary coefficient tables and an approximate Pluto fallback" → "with generated binary coefficient tables and a Meeus Table 37.A Pluto path".
- Crate README Status paragraph: replace the sentence starting "Pluto is approximate and excluded from release-grade claims" through its end with "Pluto, which VSOP87 excludes, comes from the Meeus Table 37.A periodic-term fit over 1885–2099, within about 3″ of the packaged DE440-fitted Pluto; outside that window it falls back to mean Keplerian elements (about 0.6° from Swiss Ephemeris), with a jump of that size at each window edge."
- `Cargo.toml` description: "...and an approximate Pluto fallback, for the pleiades astrology workspace." → "...and a Meeus Table 37.A Pluto path, for the pleiades astrology workspace."
- Workspace `README.md:54-55`: "VSOP87 Pluto and the compact ELP Moon stay constrained." stays true (Pluto is now `Constrained`); leave it. Line 114: "and a Pluto approximate path" → "and a Meeus Table 37.A Pluto path (1885–2099)".
- `plan/status/02-next-slice-candidates.md:42`: append " — done for the algorithmic path (issue #129: Meeus Table 37.A, constrained)."
- `docs/follow-ups.md`: append after the last entry:

```markdown

---

## FU-26: Pluto source path for the algorithmic backends (issue #129)

**Status:** resolved (2026-10-04) · Spec
`docs/superpowers/specs/2026-10-04-vsop87-pluto-periodic-fit-design.md`.
`Vsop87Backend` serves Pluto from the Meeus Table 37.A periodic-term fit
(Chapront, DE200) over 1885–2099 and from the mean-element orbit outside.
Against the packaged DE440-fitted Pluto over 1900–2099 the fit measures
max 3.29″ longitude, 0.34″ latitude, 3.27e-4 AU (the mean elements measured
35.7′ against Swiss Ephemeris); a blocking test holds it at 5″ / 0.6″ /
5e-4 AU. Pluto's claim is now `Constrained` like every other VSOP87 body; its
result quality is `Exact` inside the window and `Approximate` outside.
**Known limitation:** Pluto jumps by up to about 0.6° at each window edge.
A longer-window fit would remove it at the cost of a much larger table.
**Severity:** accuracy (closed) · **Opened:** 2026-10-04
```

- [ ] **Step 2: Compatibility profile**

In `crates/pleiades-core/src/compatibility/mod.rs`:
- Set `CURRENT_COMPATIBILITY_PROFILE_ID` to `"pleiades-compatibility-profile/0.7.23"` (main is at `0.7.22` after #133; if `main` has moved past that since, use the next unused patch number and write that number in the entry below instead of `0.7.23`).
- Append to the additions array, after the last entry:

```rust
            "Pluto periodic-term fit on the algorithmic path (issue #129): Vsop87Backend serves Pluto from the Meeus Astronomical Algorithms Table 37.A periodic-term fit (43 terms, Chapront's fit to DE200; heliocentric ecliptic J2000 minus the VSOP87B Earth) over 1885-2099 and from the mean-element orbit outside it, replacing the mean-element orbit everywhere. Over 1900-2099 it measures max 3.29\" longitude, 0.34\" latitude and 3.27e-4 AU against the packaged DE440-fitted Pluto (the mean elements were 35.7' from Swiss Ephemeris), held at 5\" / 0.6\" / 5e-4 AU by a blocking test. Pluto's claim becomes Constrained/Moderate/AlgorithmicModel like every other VSOP87 body; result quality is Exact inside the window and Approximate outside, where Pluto jumps by up to about 0.6 degrees at each edge. Vsop87BodySourceKind gains PeriodicTermFit and becomes non_exhaustive (breaking for exhaustive external matches). The comparison tolerance for Pluto drops from 45/1 degrees and 0.25 AU to the major-planet 0.01/0.01 degrees and 0.001 AU. Routed charts are unchanged (the packaged artifact serves Pluto first). Compatibility profile bumped to 0.7.23; API stability profile unchanged.",
```
- Run `cargo test -p pleiades-core --lib rendered_profile_matches_pinned_content_checksum`; it fails and prints the new checksum. Put that value in `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` and re-run → PASS.
- `grep -rn "pleiades-compatibility-profile/0.7" crates --include=*.rs | grep -v compatibility/mod.rs` and update each pinned id to the new one (as of planning: `crates/pleiades-cli/src/cli/tests/summary_commands.rs`, `crates/pleiades-validate/src/tests/render_request.rs`).

- [ ] **Step 3: Verify**

Run:
```bash
cargo test -p pleiades-core --lib compatibility
cargo test -p pleiades-cli summary
cargo test -p pleiades-validate --lib render_request
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
```
Expected: PASS, no rustdoc warnings.

- [ ] **Step 4: Commit**

```bash
cargo fmt --all
git add README.md crates/pleiades-vsop87/README.md crates/pleiades-vsop87/Cargo.toml docs/follow-ups.md plan/status/02-next-slice-candidates.md crates/pleiades-core crates/pleiades-cli crates/pleiades-validate
git commit -m "docs: record the Pluto periodic-term fit (#129)"
```

---

### Task 6: Full verification, PR, nightly

- [ ] **Step 1: Blocking tier**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
mise exec -- cargo nextest run --workspace --exclude pleiades-validate
cargo test --doc -p pleiades-vsop87
```
Expected: all PASS.

- [ ] **Step 2: Push and open the PR**

```bash
git -c credential.helper= -c credential.helper='!gh auth git-credential' fetch -q origin main
git rebase origin/main
git -c credential.helper= -c credential.helper='!gh auth git-credential' push -u origin HEAD
gh pr create --base main --title "feat(vsop87)!: serve Pluto from the Meeus Table 37.A periodic-term fit (#129)" --body-file pr-body.md
```

where `pr-body.md` (written with the Write tool in the scratchpad, not committed) contains, filled from the actual run:

```markdown
Closes #129. Spec: docs/superpowers/specs/2026-10-04-vsop87-pluto-periodic-fit-design.md

`Vsop87Backend` serves Pluto from the Meeus Table 37.A periodic-term fit over
1885–2099 and from the mean-element orbit outside.

| vs packaged Pluto, 1900–2099 | mean elements (before) | Table 37.A (after) |
|---|---|---|
| max longitude | 35.7′ (vs Swiss Ephemeris) | <max lon from the sweep test> |
| max latitude | 10.8′ | <max lat> |
| max distance | 0.116 AU | <max dist> |

- Claim: Pluto is now Constrained/Moderate/AlgorithmicModel.
- Quality: Exact inside the window, Approximate outside; Pluto jumps up to
  ~0.6° at each window edge.
- Breaking: `Vsop87BodySourceKind` gains `PeriodicTermFit` and is now
  `#[non_exhaustive]`.
- Validate: the Pluto comparison tolerance drops to the major-planet values.

Validation: fmt, workspace clippy -D warnings, rustdoc -D warnings, blocking
nextest, vsop87 doctests, the edited validate suites with --include-ignored,
and a nightly on the branch.
```

The three `<max ...>` cells come from `cargo test -p pleiades-vsop87 --lib pluto_fit_tracks -- --nocapture`, which prints the measured maxima.

- [ ] **Step 3: Nightly on the branch**

`test-full` is the only tier that runs the slow validate tests; a change that moves a body inside its comparison tolerance has broken it before (#122 → #132).

```bash
gh workflow run nightly.yml --ref worktree-issue-129-pluto-periodic-fit
```

Watch it with `gh run watch <id> --exit-status`. Merge only when the PR checks and this nightly are green (memory: finish-branch auto-merge; never `gh pr merge --auto`).
