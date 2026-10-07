# Apparent-Star Ayanamsa Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** An opt-in `SiderealStarPlace::Apparent` that makes pleiades' sidereal positions, cusps and events match Swiss Ephemeris's default `SEFLG_SIDEREAL` output for the nine star-anchored ayanamsas, by adding the anchor star's light deflection and annual aberration to the mean ayanamsa.

**Architecture:** `pleiades-types` gains the `SiderealStarPlace` enum. `pleiades-ayanamsa` (types-only) gains the five anchor stars, each mode's anchor and each star's mean place of date. `pleiades-apparent` gains the Sun's radius vector, Swiss Ephemeris's light deflection (with its `meff` taper) and a star's apparent place. `pleiades-core` composes them into `apparent_star_ayanamsa_correction` and applies it at its one true-equinox sidereal step (placements, cusps, speeds); `pleiades-events` applies a twin in `in_zodiac` for the apparent frame. A new fail-closed gate `validate-ayanamsa-apparent` and new apparent rows in `validate-sidereal-position` hold it to Swiss Ephemeris.

**Tech Stack:** Rust (workspace toolchain from `mise.toml`), cargo-nextest, Swiss Ephemeris 2.10.03 via `libswisseph-sys` 0.1.2 in the `tools/se-*-reference` crates (built inside `devenv`).

**Spec:** `docs/superpowers/specs/2026-10-07-apparent-star-ayanamsa-design.md` — read the **Amendments** section; it overrides the body where they differ.

## Global Constraints

- Default behaviour is bit-identical: every existing test, golden and gate passes unchanged under `SiderealStarPlace::Mean`.
- `pleiades-ayanamsa` depends on `pleiades-types` only (`spec/architecture.md` rule 5). Do not add `pleiades-apparent` to it.
- No new external dependencies anywhere.
- The correction applies only when: zodiac is sidereal **and** star place is `Apparent` **and** the placement/frame is apparent (core: an apparent placement, or houses in an `Apparentness::Apparent` chart; events: `CrossingFrame::GeocentricApparentOfDate`) **and** the ayanamsa has a star anchor. Everything else is unchanged.
- Anchored modes (ten catalog entries, nine SE modes): `TrueCitra`, `TrueChitra`, `TrueRevati`, `TruePushya`, `TrueMula`, `TrueSheoran`, `GalacticCenter`, `GalacticCenterRgilbrand`, `GalacticCenterMulaWilhelm`, `GalacticCenterCochrane`. Every other ayanamsa, custom ones included, has no anchor and a zero correction.
- Ceilings follow the repo rule: `ceil(1.4 × measured max)` at the stated precision; the target for rows away from solar conjunction is 0.1″.
- Commit `pleiades-core`'s field addition as `feat(core)!:` with a `BREAKING CHANGE:` footer (spec amendment 9).
- `cargo fmt --all` before every commit; `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean; rustdoc clean under `mise run docs`.
- For `pleiades-validate`, run `cargo test -p pleiades-validate --lib <filter>` (its nextest default skips the slow tier and takes 20+ minutes). Run `mise run test-full` once before the PR.
- Do not commit or edit sources while a background test run is in progress (release-bundle tests compare git provenance).
- Swiss Ephemeris reference tools build with `devenv shell -- env CFLAGS=-std=gnu17 cargo build --release --offline --manifest-path tools/<tool>/Cargo.toml` and run **outside** devenv (`tools/<tool>/target/release/<tool> > file`) so the devenv banner does not land in the CSV.

## Review Focus

1. **Houses in a mean chart.** `Apparentness::Mean` + `SiderealStarPlace::Apparent` + houses: cusps must keep the mean ayanamsa (SE geometric flags). Pinned in Task 5 (`mean_chart_cusps_ignore_the_star_place`).
2. **A mean-fallback placement inside an apparent chart** must keep the mean ayanamsa while its neighbours take the correction. Pinned in Task 5 (`a_mean_fallback_placement_keeps_the_mean_ayanamsa`), using a body/instant the existing fallback tests already use.
3. **Deserializing a `CrossingReference` serialized before this change** (no `star_place` key) must read as `Mean`, not fail. Pinned in Task 6 (`a_reference_without_a_star_place_deserializes_as_mean`).
4. **`--star-place apparent` without `--ayanamsa`** is a user error in the CLI with a clear message, not a silent no-op. Pinned in Task 9.
5. **The seam at 0°/360°.** True Revati's anchor star sits at λ ≈ 359.8° + ayanamsa, i.e. near the 0/360 seam of the anchor sum; the correction must wrap (−180, 180], never jump by 360°. Pinned in Task 4 (`the_correction_wraps_at_the_seam`).

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/pleiades-types/src/zodiac.rs` (modify) | `SiderealStarPlace` enum beside `ZodiacMode` |
| `crates/pleiades-ayanamsa/src/anchor_star.rs` (create) | `AnchorStar`, `AnchorProjection`, `StarAnchor`, `star_anchor`, `AnchorStarPlace`, `anchor_star_mean_place` |
| `crates/pleiades-ayanamsa/src/anchor_star/tests.rs` (create) | its tests |
| `crates/pleiades-apparent/src/aberration.rs` (modify) | `sun_radius_vector_au_of_date` (Meeus 25.5) |
| `crates/pleiades-apparent/src/deflection.rs` (create) | `DeflectionOffset`, `gravitational_deflection` |
| `crates/pleiades-apparent/src/deflection/meff.rs` (create) | Swiss Ephemeris `meff` table (data module, kept whole) |
| `crates/pleiades-apparent/src/deflection/tests.rs` (create) | its tests |
| `crates/pleiades-apparent/src/star.rs` (create) | `apparent_star_place`, `polar_projection_deg` |
| `crates/pleiades-apparent/src/star/tests.rs` (create) | its tests |
| `crates/pleiades-core/src/chart/star_place.rs` (create) | `apparent_star_ayanamsa_correction` (the one composition) |
| `crates/pleiades-core/src/chart/star_place_tests.rs` (create) | chart-level behaviour tests |
| `crates/pleiades-core/src/chart/{request,snapshot,sidereal,mod}.rs` (modify) | field, builder, Display, threading |
| `crates/pleiades-events/src/reference.rs` (modify) | `star_place` field, builder, `in_zodiac` correction |
| `crates/pleiades-events/tests/star_place.rs` (create) | events behaviour + cross-crate equality |
| `tools/se-ayanamsa-reference/src/main.rs` (modify) | `apparent` subcommand emitting the new corpus |
| `crates/pleiades-validate/data/ayanamsa-apparent-corpus/{ayanamsa-apparent.csv,manifest.txt}` (create) | the corpus |
| `crates/pleiades-validate/src/ayanamsa_apparent_{validation,thresholds}.rs` + `ayanamsa_apparent_validation/tests.rs` (create) | the gate |
| `tools/se-sidereal-position-reference/src/main.rs`, `crates/pleiades-validate/src/sidereal_position_*` (modify) | apparent end-to-end rows |
| `crates/pleiades-cli/src/{parse.rs,commands/chart.rs,commands/events.rs,help.rs}` (modify) | `--star-place` |
| READMEs, `docs/cli.md`, `docs/status.md`, `spec/api-and-ergonomics.md`, `crates/pleiades-core/src/compatibility/mod.rs` (modify) | docs and profile |

---

### Task 1: `SiderealStarPlace` in `pleiades-types`

**Files:**
- Modify: `crates/pleiades-types/src/zodiac.rs`, `crates/pleiades-types/src/lib.rs:55`
- Modify: `crates/pleiades-core/src/lib.rs:137-143` (re-export)
- Test: inline `#[cfg(test)]` in `zodiac.rs` if it has a test module, else `crates/pleiades-types/src/zodiac/tests.rs`

**Interfaces:**
- Produces: `pleiades_types::SiderealStarPlace { Mean, Apparent }` — `Clone, Copy, Debug, Default (= Mean), PartialEq, Eq, Hash`, `#[non_exhaustive]`, serde behind the crate's `serde` feature, `Display` as `"mean"` / `"apparent"`. Re-exported as `pleiades_core::SiderealStarPlace`.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn sidereal_star_place_defaults_to_mean_and_displays_lowercase() {
    assert_eq!(SiderealStarPlace::default(), SiderealStarPlace::Mean);
    assert_eq!(SiderealStarPlace::Mean.to_string(), "mean");
    assert_eq!(SiderealStarPlace::Apparent.to_string(), "apparent");
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p pleiades-types sidereal_star_place`
Expected: FAIL to compile, "cannot find type `SiderealStarPlace`".

- [ ] **Step 3: Implement**

In `zodiac.rs`, after `ZodiacMode`'s `Display` impl:

```rust
/// Which place of a sidereal ayanamsa's anchor star the ayanamsa is read
/// from (issue #164 (c)).
///
/// `Mean`, the default, is the anchor star's mean (geometric) place: the
/// ayanamsa pleiades has always used, in every frame. `Apparent` adds the
/// anchor star's light deflection and annual aberration, as Swiss Ephemeris
/// does under its default `SEFLG_SIDEREAL` with apparent flags. It changes
/// only the star-anchored ayanamsas (True Citra and Chitra, Revati, Pushya,
/// Mula, Sheoran, and the Galactic Center modes other than Mardyks), by up to
/// about 22″, and only for apparent places; a mean place and every other
/// ayanamsa keep the mean ayanamsa.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SiderealStarPlace {
    /// The anchor star's mean place: the mean ayanamsa.
    #[default]
    Mean,
    /// The anchor star's apparent place: Swiss Ephemeris default parity.
    Apparent,
}

impl fmt::Display for SiderealStarPlace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Mean => "mean",
            Self::Apparent => "apparent",
        })
    }
}
```

`lib.rs:55`: `pub use zodiac::{SiderealStarPlace, ZodiacMode, ZodiacSign};`
`pleiades-core/src/lib.rs`: add `SiderealStarPlace` to the `pub use pleiades_types::{...}` list (alphabetical).

- [ ] **Step 4: Run tests**

Run: `cargo test -p pleiades-types sidereal_star_place && cargo build -p pleiades-core`
Expected: PASS; core builds.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add crates/pleiades-types crates/pleiades-core/src/lib.rs
git commit -m "feat(types): add SiderealStarPlace (#164)"
```

---

### Task 2: Anchor stars in `pleiades-ayanamsa`

**Files:**
- Create: `crates/pleiades-ayanamsa/src/anchor_star.rs`, `crates/pleiades-ayanamsa/src/anchor_star/tests.rs`
- Modify: `crates/pleiades-ayanamsa/src/lib.rs` (`mod anchor_star;` + `pub use`)

**Interfaces:**
- Consumes: `sidereal_offset(&Ayanamsa, Instant) -> Option<Angle>` (`lookup.rs:306`).
- Produces:
  ```rust
  pub enum AnchorStar { Spica, ZetaPiscium, DeltaCancri, LambdaScorpii, GalacticCenter } // #[non_exhaustive], Copy, Eq, Hash, Debug
  pub enum AnchorProjection { EclipticLongitude, PolarRightAscension }                  // #[non_exhaustive], Copy, Eq, Debug
  pub struct StarAnchor { pub star: AnchorStar, pub projection: AnchorProjection }       // Copy, PartialEq, Debug
  pub fn star_anchor(ayanamsa: &Ayanamsa) -> Option<StarAnchor>;
  pub struct AnchorStarPlace { pub longitude_deg: f64, pub latitude_deg: f64 }          // Copy, PartialEq, Debug
  pub fn anchor_star_mean_place(star: AnchorStar, instant: Instant) -> Option<AnchorStarPlace>;
  ```

- [ ] **Step 1: Write the failing tests** (`anchor_star/tests.rs`)

```rust
use super::*;
use pleiades_types::{Ayanamsa, Instant, JulianDay, TimeScale};

fn tt(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

#[test]
fn exactly_the_swiss_ephemeris_star_modes_have_an_anchor() {
    use AnchorProjection::*;
    use AnchorStar::*;
    let expected = [
        (Ayanamsa::TrueCitra, Spica, EclipticLongitude),
        (Ayanamsa::TrueChitra, Spica, EclipticLongitude),
        (Ayanamsa::TrueRevati, ZetaPiscium, EclipticLongitude),
        (Ayanamsa::TruePushya, DeltaCancri, EclipticLongitude),
        (Ayanamsa::TrueSheoran, DeltaCancri, EclipticLongitude),
        (Ayanamsa::TrueMula, LambdaScorpii, EclipticLongitude),
        (Ayanamsa::GalacticCenter, GalacticCenter, EclipticLongitude),
        (Ayanamsa::GalacticCenterRgilbrand, GalacticCenter, EclipticLongitude),
        (Ayanamsa::GalacticCenterCochrane, GalacticCenter, EclipticLongitude),
        (Ayanamsa::GalacticCenterMulaWilhelm, GalacticCenter, PolarRightAscension),
    ];
    for (ayanamsa, star, projection) in &expected {
        assert_eq!(
            star_anchor(ayanamsa),
            Some(StarAnchor { star: *star, projection: *projection }),
            "{ayanamsa:?}"
        );
    }
    // Swiss Ephemeris applies no aberration to these (measured 0 over 1900–2100).
    for ayanamsa in [
        Ayanamsa::GalacticEquatorIau1958,
        Ayanamsa::GalacticEquatorTrue,
        Ayanamsa::GalacticEquatorMula,
        Ayanamsa::GalacticEquatorFiorenza,
        Ayanamsa::GalacticCenterMardyks,
        Ayanamsa::Lahiri,
        Ayanamsa::FaganBradley,
        Ayanamsa::DeLuce,
    ] {
        assert_eq!(star_anchor(&ayanamsa), None, "{ayanamsa:?}");
    }
    let anchored = crate::built_in_ayanamsas()
        .iter()
        .filter(|descriptor| star_anchor(&descriptor.ayanamsa).is_some())
        .count();
    assert_eq!(anchored, expected.len());
}

// Swiss Ephemeris `swe_fixstar`, geometric mean ecliptic and equinox of date
// (MOSEPH|NONUT|TRUEPOS|NOABERR|NOGDEFL), measured 2026-10-07: the latitude
// at J2000 and the identity longitude = primary mode's ayanamsa + anchor.
#[test]
fn mean_places_match_swiss_ephemeris_at_j2000() {
    let cases = [
        (AnchorStar::Spica, Ayanamsa::TrueCitra, 180.0, -2.054_487_222),
        (AnchorStar::ZetaPiscium, Ayanamsa::TrueRevati, 359.833_333_333_3, -0.213_433_452),
        (AnchorStar::DeltaCancri, Ayanamsa::TruePushya, 106.0, 0.077_172_355),
        (AnchorStar::LambdaScorpii, Ayanamsa::TrueMula, 240.0, -13.788_463_334),
        (AnchorStar::GalacticCenter, Ayanamsa::GalacticCenter, 240.0, -5.607_686_222),
    ];
    let j2000 = tt(2_451_545.0);
    for (star, primary, anchor, beta) in cases {
        let place = anchor_star_mean_place(star, j2000).expect("place");
        let ayanamsa = crate::sidereal_offset(&primary, j2000).unwrap().degrees();
        let lon = (ayanamsa + anchor).rem_euclid(360.0);
        assert!((place.longitude_deg - lon).abs() < 1e-12, "{star:?}");
        // The linear fit is within 0.11″ of Swiss Ephemeris everywhere.
        assert!(((place.latitude_deg - beta) * 3600.0).abs() < 0.11, "{star:?}");
    }
}

#[test]
fn longitudes_are_normalized() {
    // ζ Psc sits at 359.83° + ayanamsa, past 360° for the whole window.
    for jd in [2_415_020.5, 2_451_545.0, 2_488_069.5] {
        for star in [
            AnchorStar::Spica,
            AnchorStar::ZetaPiscium,
            AnchorStar::DeltaCancri,
            AnchorStar::LambdaScorpii,
            AnchorStar::GalacticCenter,
        ] {
            let lon = anchor_star_mean_place(star, tt(jd)).unwrap().longitude_deg;
            assert!((0.0..360.0).contains(&lon), "{star:?} {jd} {lon}");
        }
    }
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p pleiades-ayanamsa anchor_star`
Expected: FAIL to compile (module missing).

- [ ] **Step 3: Implement** (`anchor_star.rs`)

```rust
//! The stars the star-anchored ayanamsas are fixed to, and each star's mean
//! place of date (issue #164 (c)).
//!
//! Swiss Ephemeris defines each star-anchored ayanamsa as a star's longitude
//! less a constant (`sweph.c`, `swi_get_ayanamsa_ex`). A star's mean place is
//! therefore its primary mode's mean ayanamsa plus that constant: Swiss
//! Ephemeris's own geometric `swe_fixstar` longitude equals that sum to
//! 0.000000″ over 1900–2100 for all five stars (measured 2026-10-07). The
//! latitude is a straight-line fit to `swe_fixstar` over 1900–2100, within
//! 0.11″ of it everywhere.

use crate::sidereal_offset;
use pleiades_types::{Ayanamsa, Instant};

/// A star a sidereal ayanamsa is anchored to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AnchorStar {
    /// α Virginis (Citra): True Citra and True Chitra.
    Spica,
    /// ζ Piscium (Revati): True Revati.
    ZetaPiscium,
    /// δ Cancri (Asellus Australis, Pushya): True Pushya and True Sheoran.
    DeltaCancri,
    /// λ Scorpii (Mula): True Mula.
    LambdaScorpii,
    /// Sgr A*: the Galactic Center modes other than Mardyks.
    GalacticCenter,
}

/// How an anchored ayanamsa reads its star.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnchorProjection {
    /// The star's ecliptic longitude.
    EclipticLongitude,
    /// The ecliptic longitude whose right ascension is the star's: Swiss
    /// Ephemeris's polar projection (`swi_armc_to_mc`), used by Galactic
    /// Center (Mula/Wilhelm).
    PolarRightAscension,
}

/// The star an ayanamsa is anchored to, and how it reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StarAnchor {
    /// The anchor star.
    pub star: AnchorStar,
    /// How the ayanamsa reads the star's place.
    pub projection: AnchorProjection,
}

/// The star anchor of `ayanamsa`, or `None` for an ayanamsa that is not read
/// from a star's place (every other built-in mode, the galactic-equator modes
/// and Mardyks included, and every custom ayanamsa).
pub fn star_anchor(ayanamsa: &Ayanamsa) -> Option<StarAnchor> {
    use AnchorProjection::{EclipticLongitude, PolarRightAscension};
    let (star, projection) = match ayanamsa {
        Ayanamsa::TrueCitra | Ayanamsa::TrueChitra => (AnchorStar::Spica, EclipticLongitude),
        Ayanamsa::TrueRevati => (AnchorStar::ZetaPiscium, EclipticLongitude),
        Ayanamsa::TruePushya | Ayanamsa::TrueSheoran => (AnchorStar::DeltaCancri, EclipticLongitude),
        Ayanamsa::TrueMula => (AnchorStar::LambdaScorpii, EclipticLongitude),
        Ayanamsa::GalacticCenter
        | Ayanamsa::GalacticCenterRgilbrand
        | Ayanamsa::GalacticCenterCochrane => (AnchorStar::GalacticCenter, EclipticLongitude),
        Ayanamsa::GalacticCenterMulaWilhelm => (AnchorStar::GalacticCenter, PolarRightAscension),
        _ => return None,
    };
    Some(StarAnchor { star, projection })
}

/// A star's mean place on the mean ecliptic and equinox of date, degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnchorStarPlace {
    /// Ecliptic longitude, in `[0, 360)`.
    pub longitude_deg: f64,
    /// Ecliptic latitude.
    pub latitude_deg: f64,
}

/// `(primary mode, anchor longitude in that mode, latitude at J2000, latitude
/// rate per Julian century)`. The anchors are Swiss Ephemeris's
/// (`sweph.c` 3048–3090); the latitude fit is to its geometric `swe_fixstar`.
fn star_data(star: AnchorStar) -> (Ayanamsa, f64, f64, f64) {
    match star {
        AnchorStar::Spica => (Ayanamsa::TrueCitra, 180.0, -2.054_501_717, -0.007_550_067),
        AnchorStar::ZetaPiscium => {
            (Ayanamsa::TrueRevati, 359.833_333_333_3, -0.213_417_908, 0.002_565_390)
        }
        AnchorStar::DeltaCancri => (Ayanamsa::TruePushya, 106.0, 0.077_157_475, 0.003_140_560),
        AnchorStar::LambdaScorpii => (Ayanamsa::TrueMula, 240.0, -13.788_460_720, -0.013_920_718),
        AnchorStar::GalacticCenter => {
            (Ayanamsa::GalacticCenter, 240.0, -5.607_682_523, -0.013_203_301)
        }
    }
}

/// `star`'s mean place of date at `instant` (read as TT), or `None` when its
/// primary mode has no offset there.
pub fn anchor_star_mean_place(star: AnchorStar, instant: Instant) -> Option<AnchorStarPlace> {
    let (primary, anchor, beta_j2000, beta_rate) = star_data(star);
    let ayanamsa = sidereal_offset(&primary, instant)?.degrees();
    let centuries = (instant.julian_day.days() - 2_451_545.0) / 36_525.0;
    Some(AnchorStarPlace {
        longitude_deg: (ayanamsa + anchor).rem_euclid(360.0),
        latitude_deg: beta_j2000 + beta_rate * centuries,
    })
}

#[cfg(test)]
mod tests;
```

Note: if `built_in_ayanamsas()` returns descriptors without an `ayanamsa` field, adapt the count in the first test to iterate the catalog's actual element type (check `lookup.rs`); the assertion stays "exactly ten".

`lib.rs`: add `mod anchor_star;` (alphabetical, before `mod catalog;`) and
`pub use anchor_star::{anchor_star_mean_place, star_anchor, AnchorProjection, AnchorStar, AnchorStarPlace, StarAnchor};`

- [ ] **Step 4: Run tests**

Run: `cargo test -p pleiades-ayanamsa anchor_star`
Expected: 3 PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add crates/pleiades-ayanamsa
git commit -m "feat(ayanamsa): name the anchor star of each star-anchored ayanamsa (#164)"
```

---

### Task 3: Light deflection and a star's apparent place in `pleiades-apparent`

**Files:**
- Modify: `crates/pleiades-apparent/src/aberration.rs` (add `sun_radius_vector_au_of_date`), `crates/pleiades-apparent/src/aberration/tests.rs`
- Create: `crates/pleiades-apparent/src/deflection.rs`, `deflection/meff.rs`, `deflection/tests.rs`, `crates/pleiades-apparent/src/star.rs`, `star/tests.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs` (modules + re-exports)

**Interfaces:**
- Consumes: `annual_aberration(lambda_deg, beta_deg, sun_true_longitude_deg, jd_tt) -> AberrationOffset`, `sun_true_longitude_of_date_deg(jd) -> f64`.
- Produces:
  ```rust
  pub fn sun_radius_vector_au_of_date(jd: f64) -> f64;                                     // aberration.rs
  pub struct DeflectionOffset { pub d_lambda_arcsec: f64, pub d_beta_arcsec: f64 }        // deflection.rs
  pub fn gravitational_deflection(lambda_deg: f64, beta_deg: f64, jd_tt: f64) -> DeflectionOffset;
  pub fn apparent_star_place(lambda_deg: f64, beta_deg: f64, jd_tt: f64) -> (f64, f64);   // star.rs
  pub fn polar_projection_deg(lambda_deg: f64, beta_deg: f64, obliquity_deg: f64) -> f64; // star.rs
  ```

- [ ] **Step 1: Write the failing tests**

`aberration/tests.rs` (append):

```rust
// Meeus, Astronomical Algorithms, example 25.a: 1992 October 13.0 TD,
// R = 0.99766 AU.
#[test]
fn sun_radius_vector_matches_meeus_example_25a() {
    let r = super::sun_radius_vector_au_of_date(2_448_908.5);
    assert!((r - 0.99766).abs() < 2e-5, "{r}");
}
```

`deflection/tests.rs`:

```rust
use super::*;
use crate::aberration::{sun_radius_vector_au_of_date, sun_true_longitude_of_date_deg};

const JD: f64 = 2_451_545.0;
/// 2GM☉ / (c² · 1 AU), in arcseconds: the deflection scale at 1 AU.
const SCALE_ARCSEC: f64 = 0.004_071_9;

fn sun() -> (f64, f64) {
    (sun_true_longitude_of_date_deg(JD), sun_radius_vector_au_of_date(JD))
}

// At 90° elongation the displacement is the full scale (cot 45° = 1), along
// the ecliptic away from the Sun.
#[test]
fn a_star_at_quadrature_is_pushed_away_from_the_sun_by_the_full_scale() {
    let (sun_lon, r) = sun();
    let east = gravitational_deflection(sun_lon + 90.0, 0.0, JD);
    assert!((east.d_lambda_arcsec - SCALE_ARCSEC / r).abs() < 2e-6, "{east:?}");
    assert!(east.d_beta_arcsec.abs() < 1e-9);
    let west = gravitational_deflection(sun_lon - 90.0, 0.0, JD);
    assert!((west.d_lambda_arcsec + SCALE_ARCSEC / r).abs() < 2e-6, "{west:?}");
}

// Outside the disc |Δ| = scale · cot(ψ/2) / r.
#[test]
fn the_deflection_grows_as_cot_half_elongation() {
    let (sun_lon, r) = sun();
    for psi in [10.0_f64, 2.0, 0.5] {
        let d = gravitational_deflection(sun_lon + psi, 0.0, JD);
        let expected = SCALE_ARCSEC / r / (psi.to_radians() / 2.0).tan();
        assert!((d.d_lambda_arcsec - expected).abs() < 1e-4 * expected, "{psi}: {d:?}");
    }
}

// Inside the disc Swiss Ephemeris scales the mass by `meff` (Stix's solar
// model): at a tenth of the solar radius m_eff = 0.186794.
#[test]
fn inside_the_disc_the_mass_is_tapered_by_meff() {
    let (sun_lon, r) = sun();
    let radius_deg = 959.63 / 3600.0 / r;
    let psi = 0.1 * radius_deg;
    let d = gravitational_deflection(sun_lon + psi, 0.0, JD);
    let expected = 0.186_794 * SCALE_ARCSEC / r / (psi.to_radians() / 2.0).tan();
    assert!((d.d_lambda_arcsec - expected).abs() < 1e-3 * expected, "{d:?}");
    // At the centre there is no deflection at all.
    let centre = gravitational_deflection(sun_lon, 0.0, JD);
    assert_eq!(centre.d_lambda_arcsec, 0.0);
}

#[test]
fn meff_matches_the_swiss_ephemeris_table_ends_and_interpolates() {
    assert_eq!(meff::meff(0.0), 0.0);
    assert_eq!(meff::meff(1.0), 1.0);
    assert_eq!(meff::meff(1.5), 1.0);
    assert_eq!(meff::meff(0.5), 0.937_790);
    // Halfway between 0.10 (0.186794) and 0.11 (0.218327).
    assert!((meff::meff(0.105) - 0.202_560_5).abs() < 1e-9);
}
```

`star/tests.rs`:

```rust
use super::*;

// Swiss Ephemeris, measured 2026-10-07 (libswisseph-sys 0.1.2, Moshier):
// the anchor stars' geometric places at J2000 (swe_fixstar,
// MOSEPH|NONUT|TRUEPOS|NOABERR|NOGDEFL) and their apparent − geometric
// ayanamsa there (swe_get_ayanamsa_ex, MOSEPH|NONUT) — an independent
// reference for deflection + aberration.
#[test]
fn anchor_stars_match_swiss_ephemeris_at_j2000() {
    let jd = 2_451_545.0;
    let cases = [
        // (λ, β, SE correction ″)
        (203.841_362_759, -2.054_487_222, -4.8521),  // Spica (True Citra)
        (19.877_543_860, -0.213_433_452, 3.4348),    // ζ Psc (True Revati)
        (128.721_995_524, 0.077_172_355, 18.3457),   // δ Cnc (True Pushya)
        (264.585_713_184, -13.788_463_334, -20.6651), // λ Sco (True Mula)
        (266.851_709_361, -5.607_686_222, -20.3888), // Sgr A* (Galactic Center)
    ];
    for (lambda, beta, expected) in cases {
        let (app, _) = apparent_star_place(lambda, beta, jd);
        let got = ((app - lambda + 540.0).rem_euclid(360.0) - 180.0) * 3600.0;
        assert!((got - expected).abs() < 0.06, "{lambda}: {got} vs {expected}");
    }
}

#[test]
fn polar_projection_is_the_identity_on_the_ecliptic_at_the_equinoxes_and_solstices() {
    for lambda in [0.0_f64, 90.0, 180.0, 270.0] {
        let p = polar_projection_deg(lambda, 0.0, 23.44);
        assert!((p - lambda).abs() < 1e-9 || (p - lambda).abs() > 359.999_999, "{lambda} {p}");
    }
}

#[test]
fn polar_projection_differs_from_longitude_off_the_ecliptic() {
    // A point 5° south of the ecliptic at λ = 240° has a different right
    // ascension than the ecliptic point at 240°, so its projection moves.
    let p = polar_projection_deg(240.0, -5.6, 23.44);
    assert!((p - 240.0).abs() > 0.5, "{p}");
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p pleiades-apparent deflection star sun_radius`
Expected: FAIL to compile.

- [ ] **Step 3: Implement**

`aberration.rs`, after `sun_true_longitude_of_date_deg`:

```rust
/// The Sun's distance from the Earth, AU, via the same Meeus low-precision
/// theory (Astronomical Algorithms 25.5): `R = 1.000001018 (1 − e²) / (1 + e cos ν)`
/// with ν the true anomaly. Accurate to about 1e-5 AU, which is all the
/// light-deflection scale (∝ 1/R) needs. `jd` is the TT/TDB Julian Day.
pub fn sun_radius_vector_au_of_date(jd: f64) -> f64 {
    let t = (jd - 2_451_545.0) / 36_525.0;
    let m_deg = 357.529_11 + 35_999.050_29 * t - 0.000_153_7 * t * t;
    let m = m_deg.to_radians();
    let c = (1.914_602 - 0.004_817 * t - 0.000_014 * t * t) * m.sin()
        + (0.019_993 - 0.000_101 * t) * (2.0 * m).sin()
        + 0.000_289 * (3.0 * m).sin();
    let (e, _) = earth_orbit_elements(t);
    let nu = (m_deg + c).to_radians();
    1.000_001_018 * (1.0 - e * e) / (1.0 + e * nu.cos())
}
```

`deflection/meff.rs` — the Swiss Ephemeris table, transcribed whole from `sweph.c` `eff_arr` (libswisseph-sys 0.1.2, lines 5857–5965), with `meff` ported from lines 5966–5980:

```rust
//! Swiss Ephemeris's effective solar mass for a ray passing the Sun at a
//! fraction `r` of its radius (`sweph.c` `eff_arr` / `meff`, SE 2.10.03,
//! computed there from Stix, *The Sun*, p. 47). Data module: keep whole.

/// `(r, m_eff)`, `r` descending from 1 to 0.
const EFF_ARR: &[(f64, f64)] = &[
    (1.000, 1.000000), (0.990, 0.999979), (0.980, 0.999940), (0.970, 0.999881),
    (0.960, 0.999811), (0.950, 0.999724), (0.940, 0.999622), (0.930, 0.999497),
    (0.920, 0.999354), (0.910, 0.999192), (0.900, 0.999000), (0.890, 0.998786),
    (0.880, 0.998535), (0.870, 0.998242), (0.860, 0.997919), (0.850, 0.997571),
    (0.840, 0.997198), (0.830, 0.996792), (0.820, 0.996316), (0.810, 0.995791),
    (0.800, 0.995226), (0.790, 0.994625), (0.780, 0.993991), (0.770, 0.993326),
    (0.760, 0.992598), (0.750, 0.991770), (0.740, 0.990873), (0.730, 0.989919),
    (0.720, 0.988912), (0.710, 0.987856), (0.700, 0.986755), (0.690, 0.985610),
    (0.680, 0.984398), (0.670, 0.982986), (0.660, 0.981437), (0.650, 0.979779),
    (0.640, 0.978024), (0.630, 0.976182), (0.620, 0.974256), (0.610, 0.972253),
    (0.600, 0.970174), (0.590, 0.968024), (0.580, 0.965594), (0.570, 0.962797),
    (0.560, 0.959758), (0.550, 0.956515), (0.540, 0.953088), (0.530, 0.949495),
    (0.520, 0.945741), (0.510, 0.941838), (0.500, 0.937790), (0.490, 0.933563),
    (0.480, 0.928668), (0.470, 0.923288), (0.460, 0.917527), (0.450, 0.911432),
    (0.440, 0.905035), (0.430, 0.898353), (0.420, 0.891022), (0.410, 0.882940),
    (0.400, 0.874312), (0.390, 0.865206), (0.380, 0.855423), (0.370, 0.844619),
    (0.360, 0.833074), (0.350, 0.820876), (0.340, 0.808031), (0.330, 0.793962),
    (0.320, 0.778931), (0.310, 0.763021), (0.300, 0.745815), (0.290, 0.727557),
    (0.280, 0.708234), (0.270, 0.687583), (0.260, 0.665741), (0.250, 0.642597),
    (0.240, 0.618252), (0.230, 0.592586), (0.220, 0.565747), (0.210, 0.537697),
    (0.200, 0.508554), (0.190, 0.478420), (0.180, 0.447322), (0.170, 0.415454),
    (0.160, 0.382892), (0.150, 0.349955), (0.140, 0.316691), (0.130, 0.283565),
    (0.120, 0.250431), (0.110, 0.218327), (0.100, 0.186794), (0.090, 0.156287),
    (0.080, 0.128421), (0.070, 0.102237), (0.060, 0.077393), (0.050, 0.054833),
    (0.040, 0.036361), (0.030, 0.020953), (0.020, 0.009645), (0.010, 0.002767),
    (0.000, 0.000000),
];

/// Effective mass fraction for a ray passing at `r` solar radii; linear
/// between table rows, 0 at or below the centre and 1 at or beyond the limb.
pub(crate) fn meff(r: f64) -> f64 {
    if r <= 0.0 {
        return 0.0;
    }
    if r >= 1.0 {
        return 1.0;
    }
    // First row with r_i <= r; the table starts at 1.0 > r, so i >= 1.
    let i = EFF_ARR.iter().position(|&(ri, _)| ri <= r).unwrap_or(EFF_ARR.len() - 1);
    let (r0, m0) = EFF_ARR[i - 1];
    let (r1, m1) = EFF_ARR[i];
    m0 + (r - r0) / (r1 - r0) * (m1 - m0)
}
```

(Wrap the table in `#[rustfmt::skip]` so `cargo fmt` keeps it four to a line.)

`deflection.rs`:

```rust
//! Gravitational deflection of starlight by the Sun, as Swiss Ephemeris
//! computes it (`sweph.c` `swi_deflect_light`, Explanatory Supplement p. 136)
//! for a source at infinity, with the Sun's position from the Meeus theory.
//! Inside the solar disc the Sun's mass is tapered by [`meff`](meff::meff)
//! so the deflection stays finite, as Swiss Ephemeris does.

use crate::aberration::{sun_radius_vector_au_of_date, sun_true_longitude_of_date_deg};

mod meff;

/// 2·G·M☉ / (c² · 1 AU), radians: the deflection scale at 1 AU
/// (`HELGRAVCONST` 1.32712440017987e20 m³/s², `CLIGHT` 2.99792458e8 m/s,
/// `AUNIT` 1.4959787070e11 m, as in Swiss Ephemeris).
const SCALE_RAD_AT_1_AU: f64 =
    2.0 * 1.327_124_400_179_87e20 / (2.997_924_58e8 * 2.997_924_58e8) / 1.495_978_707_00e11;
/// Solar radius at 1 AU, radians (`SUN_RADIUS`, 959.63″).
const SUN_RADIUS_RAD_AT_1_AU: f64 = 959.63 / 3600.0 * std::f64::consts::PI / 180.0;

/// Deflection offset in ecliptic longitude and latitude, arcseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeflectionOffset {
    /// Δλ, arcseconds.
    pub d_lambda_arcsec: f64,
    /// Δβ, arcseconds.
    pub d_beta_arcsec: f64,
}

fn unit(lambda_deg: f64, beta_deg: f64) -> [f64; 3] {
    let (l, b) = (lambda_deg.to_radians(), beta_deg.to_radians());
    [b.cos() * l.cos(), b.cos() * l.sin(), b.sin()]
}

/// The Sun's deflection of light from a star at ecliptic `(λ, β)` (mean
/// ecliptic and equinox of date, degrees) at `jd_tt`:
/// `u' = u + g1/(1 + u·e) · (e − (u·e) u)`, `e` the Sun-to-Earth unit vector,
/// `g1 = SCALE · meff / R`. The star is pushed away from the Sun by
/// `g1 · cot(ψ/2)`, ψ the elongation: 0.0041″ at quadrature, 1.75″ at the
/// limb.
pub fn gravitational_deflection(lambda_deg: f64, beta_deg: f64, jd_tt: f64) -> DeflectionOffset {
    let r = sun_radius_vector_au_of_date(jd_tt);
    let u = unit(lambda_deg, beta_deg);
    let e = unit(sun_true_longitude_of_date_deg(jd_tt) + 180.0, 0.0);
    let ue = u[0] * e[0] + u[1] * e[1] + u[2] * e[2];
    let sin_elongation = (1.0 - ue * ue).max(0.0).sqrt();
    let sin_sun_radius = SUN_RADIUS_RAD_AT_1_AU / r;
    let meff = if sin_elongation < sin_sun_radius {
        meff::meff(sin_elongation / sin_sun_radius)
    } else {
        1.0
    };
    let g1 = SCALE_RAD_AT_1_AU * meff / r;
    let g2 = 1.0 + ue;
    if g1 == 0.0 || g2 <= 0.0 {
        // At the Sun's centre (meff = 0) nothing moves.
        return DeflectionOffset { d_lambda_arcsec: 0.0, d_beta_arcsec: 0.0 };
    }
    let d: [f64; 3] = std::array::from_fn(|i| u[i] + g1 / g2 * (e[i] - ue * u[i]));
    let lambda = d[1].atan2(d[0]).to_degrees();
    let beta = d[2].atan2(d[0].hypot(d[1])).to_degrees();
    DeflectionOffset {
        d_lambda_arcsec: ((lambda - lambda_deg + 540.0).rem_euclid(360.0) - 180.0) * 3600.0,
        d_beta_arcsec: (beta - beta_deg) * 3600.0,
    }
}

#[cfg(test)]
mod tests;
```

`star.rs`:

```rust
//! The apparent place of a star at infinity: light deflection by the Sun,
//! then annual aberration, the order Swiss Ephemeris applies them. Used for
//! the anchor star of a star-anchored ayanamsa (issue #164 (c)).

use crate::aberration::{annual_aberration, sun_true_longitude_of_date_deg};
use crate::deflection::gravitational_deflection;

/// Apparent `(λ, β)` (degrees, mean ecliptic and equinox of date, no
/// nutation) of a star whose mean place is `(λ, β)` at `jd_tt`.
pub fn apparent_star_place(lambda_deg: f64, beta_deg: f64, jd_tt: f64) -> (f64, f64) {
    let d = gravitational_deflection(lambda_deg, beta_deg, jd_tt);
    let (l1, b1) = (lambda_deg + d.d_lambda_arcsec / 3600.0, beta_deg + d.d_beta_arcsec / 3600.0);
    let a = annual_aberration(l1, b1, sun_true_longitude_of_date_deg(jd_tt), jd_tt);
    (
        (l1 + a.d_lambda_arcsec / 3600.0).rem_euclid(360.0),
        b1 + a.d_beta_arcsec / 3600.0,
    )
}

/// The ecliptic longitude whose right ascension equals that of `(λ, β)`,
/// degrees in `[0, 360)`: Swiss Ephemeris's `swi_armc_to_mc` applied to the
/// point's right ascension, `atan2(sin α, cos α · cos ε)`.
pub fn polar_projection_deg(lambda_deg: f64, beta_deg: f64, obliquity_deg: f64) -> f64 {
    let (l, b, eps) = (lambda_deg.to_radians(), beta_deg.to_radians(), obliquity_deg.to_radians());
    let ra = (l.sin() * eps.cos() - b.tan() * eps.sin()).atan2(l.cos());
    ra.sin().atan2(ra.cos() * eps.cos()).to_degrees().rem_euclid(360.0)
}

#[cfg(test)]
mod tests;
```

`lib.rs`: `pub mod deflection;` and `pub mod star;` (beside `pub mod aberration;`), and
`pub use aberration::{sun_radius_vector_au_of_date, sun_true_longitude_of_date_deg, AberrationOffset};`
`pub use deflection::{gravitational_deflection, DeflectionOffset};`
`pub use star::{apparent_star_place, polar_projection_deg};`
Update the crate description in `crates/pleiades-apparent/Cargo.toml` to mention gravitational light deflection.

- [ ] **Step 4: Run tests**

Run: `cargo test -p pleiades-apparent`
Expected: all PASS (existing + 8 new).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add crates/pleiades-apparent
git commit -m "feat(apparent): add light deflection and a star's apparent place (#164)"
```

---

### Task 4: `apparent_star_ayanamsa_correction` in `pleiades-core`

**Files:**
- Create: `crates/pleiades-core/src/chart/star_place.rs`, tests in the same file's `#[cfg(test)] mod tests` → put them in `crates/pleiades-core/src/chart/star_place/tests.rs`
- Modify: `crates/pleiades-core/src/chart/mod.rs` (`mod star_place; pub use star_place::apparent_star_ayanamsa_correction;`), `crates/pleiades-core/src/lib.rs` (re-export from `chart`)

**Interfaces:**
- Consumes: Task 2 (`star_anchor`, `anchor_star_mean_place`, `AnchorProjection`), Task 3 (`apparent_star_place`, `polar_projection_deg`), `pleiades_apparent::nutation::mean_obliquity_degrees(jd_tt) -> f64`.
- Produces: `pub fn apparent_star_ayanamsa_correction(ayanamsa: &Ayanamsa, instant: Instant) -> Option<Angle>` — apparent minus mean ayanamsa, in (−180°, 180°], `None` for an unanchored ayanamsa. Used by Task 5, Task 6's cross-crate test and Task 7's gate.

- [ ] **Step 1: Write the failing tests** (`star_place/tests.rs`)

```rust
use super::*;
use pleiades_types::{Ayanamsa, Instant, JulianDay, TimeScale};

fn tt(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

fn arcsec(ayanamsa: Ayanamsa, jd: f64) -> f64 {
    apparent_star_ayanamsa_correction(&ayanamsa, tt(jd)).expect("anchored").degrees() * 3600.0
}

// Swiss Ephemeris apparent − geometric ayanamsa (MOSEPH|NONUT), measured
// 2026-10-07 with libswisseph-sys 0.1.2. Away from solar conjunction the
// model holds these to 0.06″ (spec amendment 11).
#[test]
fn matches_swiss_ephemeris_reference_values() {
    let cases = [
        (Ayanamsa::TrueCitra, 2_451_545.0, -4.8521),
        (Ayanamsa::TrueCitra, 2_460_000.5, 13.6521),
        (Ayanamsa::TrueRevati, 2_451_545.0, 3.4348),
        (Ayanamsa::TrueRevati, 2_460_000.5, -14.6800),
        (Ayanamsa::TruePushya, 2_451_545.0, 18.3457),
        (Ayanamsa::TrueMula, 2_451_545.0, -20.6651),
        (Ayanamsa::TrueMula, 2_460_000.5, -7.1331),
        (Ayanamsa::GalacticCenter, 2_451_545.0, -20.3888),
        (Ayanamsa::GalacticCenter, 2_460_000.5, -7.7311),
        (Ayanamsa::GalacticCenterMulaWilhelm, 2_451_545.0, -21.2815),
        (Ayanamsa::GalacticCenterMulaWilhelm, 2_460_000.5, -8.0303),
    ];
    for (ayanamsa, jd, expected) in cases {
        let got = arcsec(ayanamsa.clone(), jd);
        assert!((got - expected).abs() < 0.06, "{ayanamsa:?} {jd}: {got} vs {expected}");
    }
}

#[test]
fn modes_sharing_a_star_and_projection_share_the_correction() {
    for jd in [2_420_000.5, 2_451_545.0, 2_480_000.5] {
        assert_eq!(arcsec(Ayanamsa::TrueChitra, jd), arcsec(Ayanamsa::TrueCitra, jd));
        assert_eq!(arcsec(Ayanamsa::TrueSheoran, jd), arcsec(Ayanamsa::TruePushya, jd));
        assert_eq!(arcsec(Ayanamsa::GalacticCenterCochrane, jd), arcsec(Ayanamsa::GalacticCenter, jd));
        assert_eq!(arcsec(Ayanamsa::GalacticCenterRgilbrand, jd), arcsec(Ayanamsa::GalacticCenter, jd));
    }
}

#[test]
fn unanchored_ayanamsas_have_no_correction() {
    for ayanamsa in [Ayanamsa::Lahiri, Ayanamsa::GalacticEquatorTrue, Ayanamsa::GalacticCenterMardyks] {
        assert_eq!(apparent_star_ayanamsa_correction(&ayanamsa, tt(2_451_545.0)), None);
    }
}

// ζ Psc's mean longitude crosses 0°/360° in the window; the correction is a
// difference and must stay within about ±25″ (Review Focus 5).
#[test]
fn the_correction_wraps_at_the_seam() {
    let mut jd = 2_415_020.5;
    while jd <= 2_488_069.5 {
        for ayanamsa in [Ayanamsa::TrueRevati, Ayanamsa::TrueCitra, Ayanamsa::GalacticCenterMulaWilhelm] {
            let got = arcsec(ayanamsa.clone(), jd);
            assert!(got.abs() < 25.0, "{ayanamsa:?} {jd}: {got}");
        }
        jd += 97.3;
    }
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p pleiades-core star_place`
Expected: FAIL to compile.

- [ ] **Step 3: Implement** (`star_place.rs`)

```rust
//! The apparent-star correction to a star-anchored ayanamsa (issue #164 (c)).

use pleiades_apparent::nutation::mean_obliquity_degrees;
use pleiades_apparent::{apparent_star_place, polar_projection_deg};
use pleiades_ayanamsa::{anchor_star_mean_place, star_anchor, AnchorProjection};
use pleiades_types::{Angle, Ayanamsa, Instant};

fn wrap180(degrees: f64) -> f64 {
    (degrees + 540.0).rem_euclid(360.0) - 180.0
}

/// What `ayanamsa` gains when read from its anchor star's apparent place
/// instead of its mean place, at `instant` (TT): the star's light deflection
/// and annual aberration as Swiss Ephemeris applies them under
/// `SEFLG_SIDEREAL` with apparent flags, within 0.06″ of it away from the
/// star's conjunction with the Sun and 0.5″ at δ Cnc's passage behind the
/// solar disc. Up to about 22″. `None` for an ayanamsa not anchored to a star.
///
/// [`SiderealStarPlace::Apparent`](pleiades_types::SiderealStarPlace) applies
/// it: the apparent sidereal longitude is the mean-equinox longitude less the
/// mean ayanamsa less this correction.
///
/// ```
/// use pleiades_core::apparent_star_ayanamsa_correction;
/// use pleiades_types::{Ayanamsa, Instant, JulianDay, TimeScale};
///
/// let j2000 = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
/// let citra = apparent_star_ayanamsa_correction(&Ayanamsa::TrueCitra, j2000).unwrap();
/// assert!((citra.degrees() * 3600.0 + 4.85).abs() < 0.06); // Swiss Ephemeris: −4.8521″
/// assert!(apparent_star_ayanamsa_correction(&Ayanamsa::Lahiri, j2000).is_none());
/// ```
pub fn apparent_star_ayanamsa_correction(ayanamsa: &Ayanamsa, instant: Instant) -> Option<Angle> {
    let anchor = star_anchor(ayanamsa)?;
    let mean = anchor_star_mean_place(anchor.star, instant)?;
    let jd = instant.julian_day.days();
    let (lambda, beta) = apparent_star_place(mean.longitude_deg, mean.latitude_deg, jd);
    let degrees = match anchor.projection {
        AnchorProjection::EclipticLongitude => wrap180(lambda - mean.longitude_deg),
        AnchorProjection::PolarRightAscension => {
            let eps = mean_obliquity_degrees(jd);
            wrap180(
                polar_projection_deg(lambda, beta, eps)
                    - polar_projection_deg(mean.longitude_deg, mean.latitude_deg, eps),
            )
        }
        _ => return None,
    };
    Some(Angle::from_degrees(degrees))
}

#[cfg(test)]
mod tests;
```

Wire it: `chart/mod.rs` — `mod star_place;` and `pub use star_place::apparent_star_ayanamsa_correction;` next to `pub use sidereal::sidereal_longitude;` (line 54); `lib.rs` — add it to the `pub use chart::{...}` list that exports `sidereal_longitude`.

- [ ] **Step 4: Run tests**

Run: `cargo test -p pleiades-core star_place && cargo test -p pleiades-core --doc apparent_star_ayanamsa_correction`
Expected: 4 + 1 PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add crates/pleiades-core
git commit -m "feat(core): compute the apparent-star correction to an ayanamsa (#164)"
```

---

### Task 5: Chart opt-in: placements, cusps, speeds (breaking)

**Files:**
- Modify: `crates/pleiades-core/src/chart/request.rs` (field, `new`, builder, `summary_line`), `snapshot.rs` (field, `Display`, six doc literals at lines 38, 215, 318, 367, 425, 527), `sidereal.rs` (`sidereal_longitude_of_true_equinox`), `mod.rs` (houses block 316–372, re-apply 653–664, `apparent_motion`/`correction_sample` 772–846, snapshot literal 693)
- Create: `crates/pleiades-core/src/chart/star_place_tests.rs` (+ `#[cfg(test)] mod star_place_tests;` in `chart/mod.rs`)
- Every other `ChartSnapshot { .. }` literal the compiler reports (`cargo build --workspace --all-targets` lists them).

**Interfaces:**
- Consumes: Task 1 `SiderealStarPlace`, Task 4 `apparent_star_ayanamsa_correction`.
- Produces: `ChartRequest::sidereal_star_place: SiderealStarPlace` (default `Mean`), `ChartRequest::with_sidereal_star_place(self, SiderealStarPlace) -> Self`, `ChartSnapshot::sidereal_star_place: SiderealStarPlace`. Used by Tasks 8 and 9.

- [ ] **Step 1: Write the failing tests** (`star_place_tests.rs`)

```rust
use pleiades_apparent::motion::HALF_SPAN_DAYS;
use pleiades_backend::Apparentness;
use pleiades_data::packaged_backend;
use pleiades_types::{
    Ayanamsa, CelestialBody, HouseSystem, Instant, JulianDay, Latitude, Longitude,
    ObserverLocation, SiderealStarPlace, TimeScale, ZodiacMode,
};

use super::test_support::AbsurdDistanceReleaseGradeBackend;
use crate::apparent_star_ayanamsa_correction;
use crate::chart::{ChartEngine, ChartRequest, ChartSnapshot};

const JD: f64 = 2_460_000.5;

fn instant() -> Instant {
    Instant::new(JulianDay::from_days(JD), TimeScale::Tt)
}

fn request(ayanamsa: Ayanamsa, star_place: SiderealStarPlace) -> ChartRequest {
    ChartRequest::new(instant())
        .with_bodies(vec![
            CelestialBody::Sun, CelestialBody::Moon, CelestialBody::Mars,
            CelestialBody::MeanNode, CelestialBody::TrueNode,
        ])
        .with_zodiac_mode(ZodiacMode::Sidereal { ayanamsa })
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(13.0827),
            Longitude::from_degrees(80.2707),
            None,
        ))
        .with_house_system(HouseSystem::Placidus)
        .with_sidereal_star_place(star_place)
}

fn chart(request: &ChartRequest) -> ChartSnapshot {
    ChartEngine::new(packaged_backend()).chart(request).expect("chart")
}

fn correction_deg(ayanamsa: &Ayanamsa, jd: f64) -> f64 {
    apparent_star_ayanamsa_correction(ayanamsa, Instant::new(JulianDay::from_days(jd), TimeScale::Tt))
        .map_or(0.0, |a| a.degrees())
}

fn wrap(d: f64) -> f64 { (d + 540.0).rem_euclid(360.0) - 180.0 }

#[test]
fn the_default_star_place_is_mean() {
    assert_eq!(ChartRequest::new(instant()).sidereal_star_place, SiderealStarPlace::Mean);
    let snapshot = chart(&request(Ayanamsa::TrueCitra, SiderealStarPlace::Mean));
    assert_eq!(snapshot.sidereal_star_place, SiderealStarPlace::Mean);
}

// Every placement of an apparent chart, the lunar points included (spec
// amendment 6), moves by exactly −correction; so do the cusps and angles
// (amendment 5).
#[test]
fn the_apparent_star_place_moves_placements_and_cusps_by_the_correction() {
    let ayanamsa = Ayanamsa::TrueCitra;
    let mean = chart(&request(ayanamsa.clone(), SiderealStarPlace::Mean));
    let apparent = chart(&request(ayanamsa.clone(), SiderealStarPlace::Apparent));
    let shift = -correction_deg(&ayanamsa, JD);
    assert!(shift.abs() > 1.0 / 3600.0, "the test epoch must have a sizeable correction");
    for (m, a) in mean.placements.iter().zip(&apparent.placements) {
        let dm = m.position.ecliptic.unwrap().longitude.degrees();
        let da = a.position.ecliptic.unwrap().longitude.degrees();
        assert!((wrap(da - dm) - shift).abs() < 1e-9, "{:?}", m.body);
        assert_eq!(m.position.ecliptic.unwrap().latitude, a.position.ecliptic.unwrap().latitude);
    }
    let (hm, ha) = (mean.houses.unwrap(), apparent.houses.unwrap());
    assert!((wrap(ha.angles.ascendant.degrees() - hm.angles.ascendant.degrees()) - shift).abs() < 1e-9);
    for (cm, ca) in hm.cusps.iter().zip(&ha.cusps) {
        assert!((wrap(ca.degrees() - cm.degrees()) - shift).abs() < 1e-9);
    }
}

// The speed takes the correction's rate, through the chart's own central
// difference over ±HALF_SPAN_DAYS (spec amendment 8).
#[test]
fn speeds_take_the_correction_rate() {
    let ayanamsa = Ayanamsa::TruePushya;
    let mean = chart(&request(ayanamsa.clone(), SiderealStarPlace::Mean));
    let apparent = chart(&request(ayanamsa.clone(), SiderealStarPlace::Apparent));
    let rate = (correction_deg(&ayanamsa, JD + HALF_SPAN_DAYS) - correction_deg(&ayanamsa, JD - HALF_SPAN_DAYS))
        / (2.0 * HALF_SPAN_DAYS);
    for (m, a) in mean.placements.iter().zip(&apparent.placements) {
        let sm = m.position.motion.unwrap().longitude_deg_per_day.unwrap();
        let sa = a.position.motion.unwrap().longitude_deg_per_day.unwrap();
        assert!((sa - sm + rate).abs() < 1e-9, "{:?}", m.body);
    }
}

#[test]
fn unanchored_ayanamsas_and_the_tropical_zodiac_are_unchanged() {
    for zodiac in [ZodiacMode::Tropical, ZodiacMode::Sidereal { ayanamsa: Ayanamsa::Lahiri }] {
        let base = request(Ayanamsa::Lahiri, SiderealStarPlace::Mean).with_zodiac_mode(zodiac.clone());
        let with = base.clone().with_sidereal_star_place(SiderealStarPlace::Apparent);
        assert_eq!(chart(&base).placements, chart(&with).placements, "{zodiac:?}");
        assert_eq!(chart(&base).houses, chart(&with).houses, "{zodiac:?}");
    }
}

// Review Focus 1: Swiss Ephemeris's geometric flags keep the mean ayanamsa.
#[test]
fn mean_chart_cusps_ignore_the_star_place() {
    let base = request(Ayanamsa::TrueCitra, SiderealStarPlace::Mean).with_apparentness(Apparentness::Mean);
    let with = base.clone().with_sidereal_star_place(SiderealStarPlace::Apparent);
    assert_eq!(chart(&base).placements, chart(&with).placements);
    assert_eq!(chart(&base).houses, chart(&with).houses);
}

// Review Focus 2: on the backend `apparentness_applied_tests.rs` uses, Mars's
// distance trips the light-time cap and Mars falls back to its mean place
// while the Sun is reduced. The fallback keeps the mean ayanamsa; the Sun
// moves by −correction.
#[test]
fn a_mean_fallback_placement_keeps_the_mean_ayanamsa() {
    let jd = 2_451_545.0;
    let fallback_chart = |place| {
        ChartEngine::new(AbsurdDistanceReleaseGradeBackend)
            .chart(
                &ChartRequest::new(Instant::new(JulianDay::from_days(jd), TimeScale::Tt))
                    .with_bodies(vec![CelestialBody::Sun, CelestialBody::Mars])
                    .with_zodiac_mode(ZodiacMode::Sidereal { ayanamsa: Ayanamsa::TrueCitra })
                    .with_sidereal_star_place(place),
            )
            .expect("chart succeeds")
    };
    let mean = fallback_chart(SiderealStarPlace::Mean);
    let apparent = fallback_chart(SiderealStarPlace::Apparent);
    let fallback: Vec<_> = apparent.mean_fallback_placements().map(|p| p.body.clone()).collect();
    assert_eq!(fallback, vec![CelestialBody::Mars]);
    let lon = |s: &ChartSnapshot, i: usize| s.placements[i].position.ecliptic.unwrap().longitude.degrees();
    // Mars (index 1): bit-identical.
    assert_eq!(lon(&mean, 1), lon(&apparent, 1));
    assert_eq!(mean.placements[1].position.motion, apparent.placements[1].position.motion);
    // Sun (index 0): moved by the correction.
    let shift = -correction_deg(&Ayanamsa::TrueCitra, jd);
    assert!((wrap(lon(&apparent, 0) - lon(&mean, 0)) - shift).abs() < 1e-9);
}

#[test]
fn display_names_the_apparent_star_place_only_when_it_applies() {
    let mean = chart(&request(Ayanamsa::TrueCitra, SiderealStarPlace::Mean)).to_string();
    let apparent = chart(&request(Ayanamsa::TrueCitra, SiderealStarPlace::Apparent)).to_string();
    assert!(!mean.contains("Sidereal star place"));
    assert!(apparent.contains("Sidereal star place: apparent"), "{apparent}");
}
```

(The observer is the Chennai one `sidereal_tests.rs::issue_157_request` uses. `pleiades-data` is a `pleiades-core` dev-dependency.)

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p pleiades-core star_place_tests`
Expected: FAIL to compile (`with_sidereal_star_place` missing).

- [ ] **Step 3: Implement**

`request.rs`:
- Field after `topocentric`:
  ```rust
  /// Which place of a star-anchored ayanamsa's anchor star the sidereal
  /// zodiac is read from; see [`ChartRequest::with_sidereal_star_place`].
  pub sidereal_star_place: SiderealStarPlace,
  ```
- `new`: `sidereal_star_place: SiderealStarPlace::Mean,`
- Builder after `with_zodiac_mode`:
  ```rust
  /// Sets which place of the ayanamsa's anchor star a sidereal chart uses.
  ///
  /// `Mean`, the default, keeps the mean ayanamsa in every placement.
  /// `Apparent` reads a star-anchored ayanamsa (True Citra and Chitra,
  /// Revati, Pushya, Mula, Sheoran, and the Galactic Center modes other than
  /// Mardyks) from its anchor star's apparent place, adding the star's light
  /// deflection and annual aberration (up to about 22″,
  /// [`apparent_star_ayanamsa_correction`](crate::apparent_star_ayanamsa_correction)),
  /// as Swiss Ephemeris does under its default `SEFLG_SIDEREAL`. It applies
  /// to every apparent placement, lunar points included, and to the house
  /// cusps and angles of an apparent chart; a mean chart, a mean-fallback
  /// placement, the tropical zodiac and every other ayanamsa are unchanged.
  ///
  /// ```
  /// use pleiades_core::{ChartEngine, ChartRequest, SiderealStarPlace};
  /// use pleiades_data::packaged_backend;
  /// use pleiades_types::{Ayanamsa, CelestialBody, Instant, JulianDay, TimeScale, ZodiacMode};
  ///
  /// let instant = Instant::new(JulianDay::from_days(2_460_000.5), TimeScale::Tt);
  /// let request = |place| {
  ///     ChartRequest::new(instant)
  ///         .with_bodies(vec![CelestialBody::Sun])
  ///         .with_zodiac_mode(ZodiacMode::Sidereal { ayanamsa: Ayanamsa::TrueCitra })
  ///         .with_sidereal_star_place(place)
  /// };
  /// let engine = ChartEngine::new(packaged_backend());
  /// let lon = |place| {
  ///     engine.chart(&request(place)).unwrap().placements[0].position.ecliptic.unwrap().longitude.degrees()
  /// };
  /// // Swiss Ephemeris's correction for True Citra at this instant is +13.65″.
  /// let shift = (lon(SiderealStarPlace::Apparent) - lon(SiderealStarPlace::Mean)) * 3600.0;
  /// assert!((shift + 13.65).abs() < 0.06, "{shift}");
  /// ```
  pub fn with_sidereal_star_place(mut self, sidereal_star_place: SiderealStarPlace) -> Self {
      self.sidereal_star_place = sidereal_star_place;
      self
  }
  ```
  (If `pleiades-data` is not a doc-test dependency of `pleiades-core`, it is a dev-dependency — check `Cargo.toml` line 24; doc tests can use dev-dependencies.)
- `summary_line`: append `; sidereal star place=apparent` only when the zodiac is sidereal and the star place is `Apparent`, so default summaries stay byte-identical.

`sidereal.rs` — give `sidereal_longitude_of_true_equinox` a `star_place: SiderealStarPlace` parameter and subtract the correction after the mean-ayanamsa step:

```rust
pub(super) fn sidereal_longitude_of_true_equinox(
    longitude: Longitude,
    nutation_longitude_arcsec: f64,
    instant: Instant,
    zodiac_mode: &ZodiacMode,
    star_place: SiderealStarPlace,
) -> Result<Longitude, EphemerisError> {
    if matches!(zodiac_mode, ZodiacMode::Tropical) {
        return Ok(longitude);
    }
    let mean_equinox =
        Longitude::from_degrees(longitude.degrees() - nutation_longitude_arcsec / 3600.0);
    let sidereal = sidereal_longitude(mean_equinox, instant, zodiac_mode)?;
    let correction = match (star_place, zodiac_mode) {
        (SiderealStarPlace::Apparent, ZodiacMode::Sidereal { ayanamsa }) => {
            super::star_place::apparent_star_ayanamsa_correction(ayanamsa, instant)
                .map_or(0.0, |angle| angle.degrees())
        }
        _ => 0.0,
    };
    Ok(Longitude::from_degrees(sidereal.degrees() - correction))
}
```
Extend its doc comment: "Under `SiderealStarPlace::Apparent` a star-anchored ayanamsa is read from its anchor star's apparent place (issue #164 (c))." Keep `if correction == 0.0 { return Ok(sidereal) }` before the subtraction so the `Mean` path returns the very same `Longitude` value (bit-identity).

`mod.rs`:
- Houses (line 330): before `to_sidereal`, add
  ```rust
  // Swiss Ephemeris passes the chart's flags to the ayanamsa: an apparent
  // chart's cusps read the anchor star's apparent place, a mean chart's its
  // mean place (spec amendment 5).
  let star_place = if matches!(request.apparentness, Apparentness::Apparent) {
      request.sidereal_star_place
  } else {
      SiderealStarPlace::Mean
  };
  ```
  and pass `star_place` as the new last argument.
- Re-apply (line 657): pass `request.sidereal_star_place`.
- `apparent_motion` and `correction_sample`: add a `star_place: SiderealStarPlace` parameter after `chart_sidereal_mode`; `correction_sample` passes it to `sidereal_longitude_of_true_equinox` (line 795); `apparent_motion` passes it to `correction_sample`; the call at line 525 passes `request.sidereal_star_place`.
- Snapshot literal (line 693): `sidereal_star_place: request.sidereal_star_place,`.

`snapshot.rs`:
- Field after `apparentness`:
  ```rust
  /// The anchor-star place the chart's sidereal zodiac was read from
  /// ([`ChartRequest::with_sidereal_star_place`](crate::ChartRequest::with_sidereal_star_place)).
  pub sidereal_star_place: SiderealStarPlace,
  ```
- The six doc literals: add `sidereal_star_place: SiderealStarPlace::Mean,` (import it in each doc block's `use`).
- `Display` (line 754): after the line that prints the zodiac mode, add
  ```rust
  if matches!(self.zodiac_mode, ZodiacMode::Sidereal { .. })
      && self.sidereal_star_place == SiderealStarPlace::Apparent
  {
      writeln!(f, "Sidereal star place: apparent (anchor star's light deflection and annual aberration)")?;
  }
  ```

Then `cargo build --workspace --all-targets` and add `sidereal_star_place: SiderealStarPlace::Mean` to every other `ChartSnapshot { .. }` literal the compiler names.

- [ ] **Step 4: Run tests**

Run: `cargo test -p pleiades-core star_place_tests && cargo nextest run -p pleiades-core && cargo test -p pleiades-core --doc`
Expected: new tests PASS; every existing core test and doc test PASS unchanged (bit-identity of the default path).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add crates/pleiades-core
git commit -F - <<'EOF'
feat(core)!: read a star-anchored ayanamsa from its anchor star's apparent place on request (#164)

ChartRequest::with_sidereal_star_place(SiderealStarPlace::Apparent) adds the
anchor star's light deflection and annual aberration to True Citra/Chitra,
Revati, Pushya, Mula, Sheoran and the Galactic Center modes other than
Mardyks, as Swiss Ephemeris does under SEFLG_SIDEREAL with apparent flags:
every apparent placement, lunar points included, and an apparent chart's
cusps and angles. The default (Mean) is unchanged.

BREAKING CHANGE: ChartRequest and ChartSnapshot gain the public field
`sidereal_star_place`; code that builds either with a struct literal must
add it (SiderealStarPlace::Mean keeps the previous behaviour).
EOF
```

---

### Task 6: Events opt-in

**Files:**
- Modify: `crates/pleiades-events/src/reference.rs` (struct, constructors, builder, `in_zodiac`, docs at lines 50–70), `crates/pleiades-events/src/lib.rs` (re-export `SiderealStarPlace` beside `CrossingReference`), `crates/pleiades-events/tests/serde_zodiac.rs` (serde default)
- Create: `crates/pleiades-events/tests/star_place.rs`

**Interfaces:**
- Consumes: Tasks 1–3; `pleiades_core::apparent_star_ayanamsa_correction` in tests only (dev-dependency).
- Produces: `CrossingReference::star_place: SiderealStarPlace`, `CrossingReference::with_star_place(self, SiderealStarPlace) -> Self`, `pleiades_events::SiderealStarPlace`. Used by Task 9.

- [ ] **Step 1: Write the failing tests** (`tests/star_place.rs`)

```rust
//! `CrossingReference::with_star_place` (issue #164 (c)).

use pleiades_core::apparent_star_ayanamsa_correction;
use pleiades_data::packaged_backend;
use pleiades_events::{CrossingFrame, CrossingReference, EventEngine, SiderealStarPlace};
use pleiades_types::{Ayanamsa, CelestialBody, Instant, JulianDay, Longitude, TimeScale};

const JD: f64 = 2_460_000.5;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn reference(frame: CrossingFrame, ayanamsa: Ayanamsa, place: SiderealStarPlace) -> CrossingReference {
    CrossingReference::sidereal(frame, ayanamsa).with_star_place(place)
}

fn lon(body: CelestialBody, r: CrossingReference, jd: f64) -> f64 {
    EventEngine::new(packaged_backend()).longitude_at(body, r, tdb(jd)).expect("longitude").degrees()
}

fn wrap(d: f64) -> f64 { (d + 540.0).rem_euclid(360.0) - 180.0 }

#[test]
fn a_reference_defaults_to_the_mean_star_place() {
    let r = CrossingReference::sidereal(CrossingFrame::GeocentricApparentOfDate, Ayanamsa::TrueCitra);
    assert_eq!(r.star_place, SiderealStarPlace::Mean);
    assert_eq!(CrossingReference::from(CrossingFrame::Heliocentric).star_place, SiderealStarPlace::Mean);
}

// The events twin of the correction equals core's (spec amendment 10).
#[test]
fn the_apparent_frame_moves_by_core_s_correction() {
    for ayanamsa in [Ayanamsa::TrueCitra, Ayanamsa::TruePushya, Ayanamsa::GalacticCenterMulaWilhelm] {
        let correction = apparent_star_ayanamsa_correction(&ayanamsa, Instant::new(JulianDay::from_days(JD), TimeScale::Tt))
            .unwrap()
            .degrees();
        for body in [CelestialBody::Sun, CelestialBody::Mars, CelestialBody::TrueNode] {
            let geo = CrossingFrame::GeocentricApparentOfDate;
            let mean = lon(body.clone(), reference(geo, ayanamsa.clone(), SiderealStarPlace::Mean), JD);
            let app = lon(body.clone(), reference(geo, ayanamsa.clone(), SiderealStarPlace::Apparent), JD);
            assert!((wrap(app - mean) + correction).abs() < 1e-12, "{ayanamsa:?} {body:?}");
        }
    }
}

#[test]
fn the_mean_and_heliocentric_frames_and_unanchored_ayanamsas_are_unchanged() {
    for (frame, ayanamsa, body) in [
        (CrossingFrame::GeocentricMeanOfDate, Ayanamsa::TrueCitra, CelestialBody::Mars),
        (CrossingFrame::Heliocentric, Ayanamsa::TrueCitra, CelestialBody::Mars),
        (CrossingFrame::GeocentricApparentOfDate, Ayanamsa::Lahiri, CelestialBody::Mars),
    ] {
        let mean = lon(body.clone(), reference(frame, ayanamsa.clone(), SiderealStarPlace::Mean), JD);
        let app = lon(body.clone(), reference(frame, ayanamsa.clone(), SiderealStarPlace::Apparent), JD);
        assert_eq!(mean, app, "{frame:?} {ayanamsa:?}");
    }
}

#[test]
fn a_crossing_under_the_apparent_star_place_is_at_the_target_longitude() {
    let engine = EventEngine::new(packaged_backend());
    let r = reference(CrossingFrame::GeocentricApparentOfDate, Ayanamsa::TrueCitra, SiderealStarPlace::Apparent);
    let crossing = engine
        .next_longitude_crossing(CelestialBody::Sun, Longitude::from_degrees(0.0), r.clone(), tdb(JD))
        .expect("search")
        .expect("the Sun crosses 0° every year");
    let at = lon(CelestialBody::Sun, r, crossing.instant.julian_day.days());
    assert!(wrap(at).abs() < 1e-6, "{at}");
}
```

`tests/serde_zodiac.rs` (append, `#[cfg(feature = "serde")]` like its neighbours) — Review Focus 3:

```rust
#[test]
fn a_reference_without_a_star_place_deserializes_as_mean() {
    // Serialize a reference, drop the new key, and read it back.
    let r = CrossingReference::sidereal(CrossingFrame::GeocentricApparentOfDate, Ayanamsa::TrueCitra);
    let mut value = serde_json::to_value(&r).unwrap();
    value.as_object_mut().unwrap().remove("star_place");
    let back: CrossingReference = serde_json::from_value(value).unwrap();
    assert_eq!(back.star_place, SiderealStarPlace::Mean);
}
```
(Use whatever serde format `serde_zodiac.rs` already uses; it has the pre-`zodiac` precedent.)

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p pleiades-events --test star_place`
Expected: FAIL to compile.

- [ ] **Step 3: Implement** (`reference.rs`)

- Struct field after `zodiac`:
  ```rust
  /// Which place of a star-anchored ayanamsa's anchor star the zodiac is read
  /// from; see [`CrossingReference::with_star_place`]. `Mean` by default.
  #[cfg_attr(feature = "serde", serde(default))]
  pub star_place: SiderealStarPlace,
  ```
- `tropical` and `sidereal`: `star_place: SiderealStarPlace::Mean`.
- Builder:
  ```rust
  /// This reference with its anchor-star place set. `Apparent` reads a
  /// star-anchored ayanamsa from its anchor star's apparent place in
  /// [`CrossingFrame::GeocentricApparentOfDate`], as Swiss Ephemeris's
  /// default `SEFLG_SIDEREAL` does, for every body including the lunar
  /// points; the mean-of-date and heliocentric frames, which use geometric
  /// flags in Swiss Ephemeris, keep the mean ayanamsa.
  #[must_use]
  pub fn with_star_place(mut self, star_place: SiderealStarPlace) -> Self {
      self.star_place = star_place;
      self
  }
  ```
- A private twin of core's composition (core cannot be a dependency):
  ```rust
  /// The apparent-star correction to `ayanamsa` at `julian_day`, degrees:
  /// `pleiades_core::apparent_star_ayanamsa_correction`, which
  /// `tests/star_place.rs` holds this equal to. Zero for an unanchored ayanamsa.
  fn apparent_star_correction_deg(ayanamsa: &Ayanamsa, julian_day: f64) -> f64 {
      let instant = Instant::new(JulianDay::from_days(julian_day), TimeScale::Tt);
      let Some(anchor) = star_anchor(ayanamsa) else { return 0.0 };
      let Some(mean) = anchor_star_mean_place(anchor.star, instant) else { return 0.0 };
      let (lambda, beta) = apparent_star_place(mean.longitude_deg, mean.latitude_deg, julian_day);
      let degrees = match anchor.projection {
          AnchorProjection::EclipticLongitude => lambda - mean.longitude_deg,
          AnchorProjection::PolarRightAscension => {
              let eps = mean_obliquity_degrees(julian_day);
              polar_projection_deg(lambda, beta, eps)
                  - polar_projection_deg(mean.longitude_deg, mean.latitude_deg, eps)
          }
          _ => return 0.0,
      };
      (degrees + 540.0).rem_euclid(360.0) - 180.0
  }
  ```
- `in_zodiac`: after computing `shift`, add the correction for the apparent frame:
  ```rust
  let star = match (reference.frame, reference.star_place) {
      (CrossingFrame::GeocentricApparentOfDate, SiderealStarPlace::Apparent) => {
          apparent_star_correction_deg(ayanamsa, julian_day)
      }
      _ => 0.0,
  };
  let shift = nutation_deg + ayanamsa_deg(ayanamsa, julian_day)? + star;
  ```
  (`+ 0.0` keeps the `Mean` path bit-identical.)
- Replace the doc paragraph at lines 58–66 ("The mean ayanamsa is used in every frame. …") with: the mean ayanamsa by default; `with_star_place(SiderealStarPlace::Apparent)` gives Swiss Ephemeris default parity for the nine star-anchored modes in the apparent frame (up to about 22″, measured), not for the galactic-equator modes or Mardyks, which Swiss Ephemeris does not aberrate.
- `lib.rs`: `pub use pleiades_types::SiderealStarPlace;` next to the `reference::CrossingReference` re-export.

- [ ] **Step 4: Run tests**

Run: `cargo test -p pleiades-events --test star_place && cargo test -p pleiades-events --all-features --test serde_zodiac && cargo nextest run -p pleiades-events`
Expected: new tests PASS; all existing events tests PASS unchanged.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add crates/pleiades-events
git commit -m "feat(events): read a star-anchored ayanamsa from its anchor star's apparent place on request (#164)"
```

---

### Task 7: Gate `validate-ayanamsa-apparent`

**Files:**
- Modify: `tools/se-ayanamsa-reference/src/main.rs` (`apparent` subcommand)
- Create: `crates/pleiades-validate/data/ayanamsa-apparent-corpus/ayanamsa-apparent.csv`, `manifest.txt`
- Create: `crates/pleiades-validate/src/ayanamsa_apparent_validation.rs`, `ayanamsa_apparent_thresholds.rs`, `ayanamsa_apparent_validation/tests.rs`
- Modify: `crates/pleiades-validate/src/lib.rs` (mods + re-export), `src/render/cli.rs` (`NUMERIC_GATES`, dispatch, help text), `src/tests/validate_gates.rs`, `crates/pleiades-cli/src/cli.rs` (passthrough ~line 809), `docs/status.md`

**Interfaces:**
- Consumes: Task 4 `pleiades_core::apparent_star_ayanamsa_correction`.
- Produces: `pub fn validate_ayanamsa_apparent_corpus() -> Result<AyanamsaApparentReport, AyanamsaApparentError>`; CLI `validate-ayanamsa-apparent` / alias `ayanamsa-apparent-gate`.

- [ ] **Step 1: Add the reference-tool subcommand**

In `tools/se-ayanamsa-reference/src/main.rs` add `Some("apparent") => emit_apparent(),` to `main`'s match, and:

```rust
use libswisseph_sys::raw::{swe_calc, swe_fixstar};
use std::ffi::CString;

const SEFLG_MOSEPH: i32 = 4;
const SEFLG_TRUEPOS: i32 = 16;
const SEFLG_NOGDEFL: i32 = 512;
/// Apparent star place, nutation-free (the place SEFLG_SIDEREAL uses).
const APPARENT_IFLAG: i32 = SEFLG_MOSEPH | SEFLG_NONUT;
/// Geometric star place: the mean ayanamsa.
const GEOMETRIC_IFLAG: i32 = APPARENT_IFLAG | SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL;

/// (mode, SE_SIDM, anchor star name for its solar conjunctions, or None for a
/// mode Swiss Ephemeris does not aberrate).
const APPARENT_MODES: &[(&str, i32, Option<&str>)] = &[
    ("TrueCitra", 27, Some("Spica")),
    ("TrueRevati", 28, Some(",zePsc")),
    ("TruePushya", 29, Some(",deCnc")),
    ("TrueMula", 35, Some(",laSco")),
    ("TrueSheoran", 39, Some(",deCnc")),
    ("GalacticCenter", 17, Some(",SgrA*")),
    ("GalacticCenterRgilbrand", 30, Some(",SgrA*")),
    ("GalacticCenterMulaWilhelm", 36, Some(",SgrA*")),
    ("GalacticCenterCochrane", 40, Some(",SgrA*")),
    ("GalacticEquatorIau1958", 31, None),
    ("GalacticEquatorTrue", 32, None),
    ("GalacticEquatorMula", 33, None),
    ("GalacticCenterMardyks", 34, None),
    ("GalacticEquatorFiorenza", 41, None),
];

fn ayanamsa_with(code: i32, jd_tt: f64, iflag: i32) -> f64 {
    unsafe { swe_set_sid_mode(code, 0.0, 0.0) };
    let mut out = 0.0;
    let mut err = [0 as c_char; 256];
    let rc = unsafe { swe_get_ayanamsa_ex(jd_tt, iflag, &mut out, err.as_mut_ptr()) };
    assert!(rc >= 0 && out.is_finite(), "swe_get_ayanamsa_ex({code}, {jd_tt})");
    out
}

fn star_longitude(name: &str, jd_tt: f64) -> f64 {
    let mut buf = [0 as c_char; 64];
    for (dst, src) in buf.iter_mut().zip(CString::new(name).unwrap().as_bytes_with_nul()) {
        *dst = *src as c_char;
    }
    let (mut xx, mut err) = ([0.0f64; 6], [0 as c_char; 256]);
    let rc = unsafe { swe_fixstar(buf.as_mut_ptr(), jd_tt, GEOMETRIC_IFLAG, xx.as_mut_ptr(), err.as_mut_ptr()) };
    assert!(rc >= 0, "swe_fixstar {name}");
    xx[0]
}

fn sun_longitude(jd_tt: f64) -> f64 {
    let (mut xx, mut err) = ([0.0f64; 6], [0 as c_char; 256]);
    let rc = unsafe { swe_calc(jd_tt, 0, GEOMETRIC_IFLAG, xx.as_mut_ptr(), err.as_mut_ptr()) };
    assert!(rc >= 0, "swe_calc Sun");
    xx[0]
}

fn wrap180(d: f64) -> f64 {
    (d + 540.0).rem_euclid(360.0) - 180.0
}

/// The instant in `year`'s window [jd0, jd0 + 366) at which the Sun's
/// geometric longitude equals `star`'s: daily scan, then bisection to 1e-6 d.
fn conjunction(star: &str, jd0: f64) -> f64 {
    let f = |jd: f64| wrap180(sun_longitude(jd) - star_longitude(star, jd));
    let mut lo = jd0;
    while !(f(lo) < 0.0 && f(lo + 1.0) >= 0.0) {
        lo += 1.0;
        assert!(lo < jd0 + 367.0, "no conjunction of {star} after {jd0}");
    }
    let mut hi = lo + 1.0;
    while hi - lo > 1e-6 {
        let mid = 0.5 * (lo + hi);
        if f(mid) < 0.0 { lo = mid } else { hi = mid }
    }
    0.5 * (lo + hi)
}

/// Apparent − geometric ayanamsa (Swiss Ephemeris, Moshier), arcsec: 60
/// uniform rows per anchored mode plus 50 rows around ten conjunctions of its
/// anchor star with the Sun (where light deflection peaks), and 10 uniform rows
/// per mode Swiss Ephemeris does not aberrate.
fn emit_apparent() {
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), swe_get_ayanamsa_ex after swe_set_sid_mode.");
    println!("# se_correction_arcsec = (ayanamsa with SEFLG_MOSEPH|SEFLG_NONUT - ayanamsa with SEFLG_MOSEPH|SEFLG_NONUT|SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL) * 3600.");
    println!("# class: uniform (spread over 1900-2100) or conjunction (within a day of the anchor star's conjunction with the Sun).");
    println!("# Generated by `tools/se-ayanamsa-reference apparent`. jd_tt is TT.");
    println!("mode,jd_tt,class,se_correction_arcsec");
    for &(name, code, star) in APPARENT_MODES {
        let uniform = if star.is_some() { 60 } else { 10 };
        let step = (2_488_069.5 - 2_415_020.5) / uniform as f64;
        for k in 0..uniform {
            let jd = 2_415_020.87 + k as f64 * step;
            let c = (ayanamsa_with(code, jd, APPARENT_IFLAG) - ayanamsa_with(code, jd, GEOMETRIC_IFLAG)) * 3600.0;
            println!("{name},{jd:.6},uniform,{c:.6}");
        }
        let Some(star) = star else { continue };
        for year in 0..10 {
            let jd0 = 2_416_480.5 + year as f64 * 7_305.0; // 1904-01-01 + 20 y steps
            let t0 = conjunction(star, jd0);
            for offset in [-1.0, -0.15, 0.0, 0.15, 1.0] {
                let jd = t0 + offset;
                let c = (ayanamsa_with(code, jd, APPARENT_IFLAG) - ayanamsa_with(code, jd, GEOMETRIC_IFLAG)) * 3600.0;
                println!("{name},{jd:.6},conjunction,{c:.6}");
            }
        }
    }
}
```

(`SEFLG_NONUT` and `SEFLG_NOABERR` already exist in the file; `swe_set_sid_mode`, `swe_get_ayanamsa_ex` and `c_char` are already imported. Add `swe_calc`, `swe_fixstar` to the `use`.)

- [ ] **Step 2: Generate the corpus**

```bash
devenv shell -- env CFLAGS=-std=gnu17 cargo build --release --offline --manifest-path tools/se-ayanamsa-reference/Cargo.toml
mkdir -p crates/pleiades-validate/data/ayanamsa-apparent-corpus
tools/se-ayanamsa-reference/target/release/se-ayanamsa-reference apparent > crates/pleiades-validate/data/ayanamsa-apparent-corpus/ayanamsa-apparent.csv
grep -vc '^#\|^mode' crates/pleiades-validate/data/ayanamsa-apparent-corpus/ayanamsa-apparent.csv
```
Expected: `1040` (9 × 110 + 5 × 10). The file must start with `# Source:`.

- [ ] **Step 3: Write the gate's failing tests** (`ayanamsa_apparent_validation/tests.rs`), copying the shape of `sidereal_position_validation/tests.rs`:

```rust
use super::*;
use pleiades_apparent::fnv1a64;

#[test]
fn ayanamsa_apparent_gate_passes_within_ceilings() {
    let report = validate_ayanamsa_apparent_corpus().expect("gate");
    assert_eq!(report.rows_validated, 1040);
}

#[test]
fn tampered_corpus_fails_the_checksum() {
    let csv = CORPUS_CSV.replacen("TrueCitra,", "TrueCitra, ", 1);
    assert!(matches!(validate(&csv, MANIFEST), Err(AyanamsaApparentError::ChecksumMismatch { .. })));
}

#[test]
fn manifest_row_count_drift_fails_closed() {
    let manifest = format!("slice ayanamsa-apparent file=ayanamsa-apparent.csv role=ayanamsa-apparent rows=1039 checksum={}", fnv1a64(CORPUS_CSV));
    assert!(matches!(validate(CORPUS_CSV, &manifest), Err(AyanamsaApparentError::ManifestDrift { .. })));
}

#[test]
fn a_truncated_corpus_with_a_matching_manifest_fails_the_floor() {
    let csv: String = CORPUS_CSV.lines().take(30).map(|l| format!("{l}\n")).collect();
    let manifest = format!("slice x rows={} checksum={}", csv.lines().filter(|l| !l.starts_with('#') && !l.starts_with("mode")).count(), fnv1a64(&csv));
    assert!(matches!(
        validate_with_floor(&csv, &manifest, MIN_ROWS_VALIDATED),
        Err(AyanamsaApparentError::TooFewRowsValidated { .. })
    ));
}

/// One row, its correction shifted by `shift_arcsec`.
fn one_row(mode: &str, class: &str, shift_arcsec: f64) -> (String, String) {
    let line = CORPUS_CSV
        .lines()
        .find(|l| l.starts_with(&format!("{mode},")) && l.contains(&format!(",{class},")))
        .expect("row");
    let mut fields: Vec<String> = line.split(',').map(str::to_string).collect();
    let value: f64 = fields[3].parse().unwrap();
    fields[3] = format!("{:.6}", value + shift_arcsec);
    let csv = format!("mode,jd_tt,class,se_correction_arcsec\n{}\n", fields.join(","));
    let manifest = format!("slice x rows=1 checksum={}", fnv1a64(&csv));
    (csv, manifest)
}

#[test]
fn an_unshifted_row_passes_alone() {
    let (csv, manifest) = one_row("TrueCitra", "uniform", 0.0);
    validate_with_floor(&csv, &manifest, 1).expect("passes");
}

// The gate would catch the composition dropping the aberration (≈20″) or
// the deflection near conjunction (≈2.8″ for δ Cnc).
#[test]
fn a_correction_off_by_a_ceiling_fails() {
    for (mode, class) in [("TrueCitra", "uniform"), ("TruePushya", "conjunction"), ("GalacticEquatorTrue", "uniform")] {
        let (csv, manifest) = one_row(mode, class, 1.0);
        assert!(matches!(
            validate_with_floor(&csv, &manifest, 1),
            Err(AyanamsaApparentError::CeilingExceeded { .. })
        ), "{mode} {class}");
    }
}

#[test]
fn malformed_rows_are_rejected() {
    for bad in ["TrueCitra,2451545.0,uniform", "TrueCitra,2451545.0,uniform,NaN", "Nonesuch,2451545.0,uniform,0.0", "TrueCitra,2451545.0,sideways,0.0"] {
        let csv = format!("{bad}\n");
        let manifest = format!("slice x rows=1 checksum={}", fnv1a64(&csv));
        assert!(matches!(validate_with_floor(&csv, &manifest, 1), Err(AyanamsaApparentError::MalformedRow(_))), "{bad}");
    }
}

/// Measurement helper, not a gate: prints the maxima the ceilings are sized
/// from. `cargo test -p pleiades-validate --lib measure_ayanamsa_apparent -- --ignored --nocapture`
#[test]
#[ignore]
fn measure_ayanamsa_apparent_maxima() {
    let rows = parse_rows(CORPUS_CSV).unwrap();
    let (mut uniform, mut conjunction) = (0.0f64, 0.0f64);
    for row in rows {
        let r = residual_arcsec(&row).unwrap();
        match row.class {
            RowClass::Uniform => uniform = uniform.max(r),
            RowClass::Conjunction => conjunction = conjunction.max(r),
        }
    }
    println!("uniform max {uniform:.4}\"  conjunction max {conjunction:.4}\"");
}
```

- [ ] **Step 4: Run them to verify they fail**

Run: `cargo test -p pleiades-validate --lib ayanamsa_apparent`
Expected: FAIL to compile.

- [ ] **Step 5: Implement the gate**

`ayanamsa_apparent_thresholds.rs`:

```rust
//! Ceilings for `validate-ayanamsa-apparent` (issue #164 (c)): pleiades'
//! apparent-star correction (`pleiades_core::apparent_star_ayanamsa_correction`)
//! against Swiss Ephemeris's apparent − geometric ayanamsa, 1040 rows.
//!
//! Sized `ceil(1.4 × measured max)` to 0.01″, measured <DATE> with
//! `measure_ayanamsa_apparent_maxima`: uniform rows <U>″, conjunction rows
//! <C>″ (δ Cnc behind the solar disc, where the Meeus Sun's ~0.01° error is
//! large against the star's 278″ closest approach).

/// Rows spread over 1900–2100 (aberration-dominated). Target 0.10″.
pub(crate) const UNIFORM_CEILING_ARCSEC: f64 = 0.10;
/// Rows within a day of the anchor star's solar conjunction.
pub(crate) const CONJUNCTION_CEILING_ARCSEC: f64 = 0.70;
```

Then run the measurement (Step 7) and replace `<DATE>`, `<U>`, `<C>` and the two constants with the measured values (expected from the prototype: uniform ≲ 0.06″ → 0.09–0.10; conjunction ≈ 0.5″ → ≈ 0.70). **If uniform exceeds 0.1″ or conjunction exceeds 1.0″, stop and report**: the model does not match the spec's accuracy and the cause must be found, not absorbed into a ceiling.

`ayanamsa_apparent_validation.rs` — mirror `sidereal_position_validation.rs`'s structure exactly (constants with `include_str!`, `parse_manifest`, `validate`, `validate_with_floor`, error enum with `Display`/`Error`, report with `summary_line()`), with:

```rust
const MIN_ROWS_VALIDATED: usize = 1040;

#[derive(Clone, Copy, Debug, PartialEq)]
enum RowClass { Uniform, Conjunction }

struct Row { mode: Ayanamsa, mode_name: String, jd_tt: f64, class: RowClass, se_arcsec: f64 }

fn ayanamsa_from_name(name: &str) -> Option<Ayanamsa> {
    Some(match name {
        "TrueCitra" => Ayanamsa::TrueCitra,
        "TrueRevati" => Ayanamsa::TrueRevati,
        "TruePushya" => Ayanamsa::TruePushya,
        "TrueMula" => Ayanamsa::TrueMula,
        "TrueSheoran" => Ayanamsa::TrueSheoran,
        "GalacticCenter" => Ayanamsa::GalacticCenter,
        "GalacticCenterRgilbrand" => Ayanamsa::GalacticCenterRgilbrand,
        "GalacticCenterMulaWilhelm" => Ayanamsa::GalacticCenterMulaWilhelm,
        "GalacticCenterCochrane" => Ayanamsa::GalacticCenterCochrane,
        "GalacticEquatorIau1958" => Ayanamsa::GalacticEquatorIau1958,
        "GalacticEquatorTrue" => Ayanamsa::GalacticEquatorTrue,
        "GalacticEquatorMula" => Ayanamsa::GalacticEquatorMula,
        "GalacticCenterMardyks" => Ayanamsa::GalacticCenterMardyks,
        "GalacticEquatorFiorenza" => Ayanamsa::GalacticEquatorFiorenza,
        _ => return None,
    })
}

/// |pleiades − Swiss Ephemeris|, arcsec. An unanchored mode's correction is 0.
fn residual_arcsec(row: &Row) -> Result<f64, AyanamsaApparentError> {
    let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);
    let ours = pleiades_core::apparent_star_ayanamsa_correction(&row.mode, instant)
        .map_or(0.0, |angle| angle.degrees() * 3600.0);
    Ok((ours - row.se_arcsec).abs())
}
```

- Parsing, in `fn parse_rows(csv: &str) -> Result<Vec<Row>, AyanamsaApparentError>` (used by `validate` and the measurement test): skip empty, `#` and `mode,` lines; exactly 4 fields; finite numbers; class `uniform`|`conjunction`; unknown mode or class → `MalformedRow`.
- Ceiling per class; NaN residual fails closed (`residual.is_nan() || residual > ceiling`); `CeilingExceeded { mode, jd_tt, class, residual, ceiling }`.
- Report `AyanamsaApparentReport { rows_validated, uniform_max_arcsec, conjunction_max_arcsec, summary_line }`; summary: `"Ayanamsa-apparent gate: {n} Swiss Ephemeris apparent-star ayanamsa corrections validated (9 star-anchored modes, 5 unaberrated); max residual {u:.3}\" uniform (ceiling {U}\"), {c:.3}\" near solar conjunction (ceiling {C}\")"`.
- `pub fn validate_ayanamsa_apparent_corpus()`.

`manifest.txt` (one line): `slice ayanamsa-apparent file=ayanamsa-apparent.csv role=ayanamsa-apparent rows=1040 checksum=<fnv1a64 of the CSV>` — get the checksum by running `ayanamsa_apparent_gate_passes_within_ceilings` once with a placeholder `checksum=0`; the `ChecksumMismatch { got, .. }` error prints it.

Registration (exact patterns in the explorer notes, all in `pleiades-validate` unless noted):
- `lib.rs`: `mod ayanamsa_apparent_thresholds; mod ayanamsa_apparent_validation;` (alphabetical) and `pub use ayanamsa_apparent_validation::{validate_ayanamsa_apparent_corpus, AyanamsaApparentError, AyanamsaApparentReport};`
- `render/cli.rs` `NUMERIC_GATES`: `("ayanamsa-apparent gate failed", || crate::validate_ayanamsa_apparent_corpus().map(drop).map_err(|e| e.to_string())),` after the ayanamsa entry.
- `render/cli.rs` `render_cli`: `Some("validate-ayanamsa-apparent") | Some("ayanamsa-apparent-gate") => { ensure_no_extra_args(&args[1..], "validate-ayanamsa-apparent")?; crate::validate_ayanamsa_apparent_corpus().map(|r| r.summary_line().to_string()).map_err(|e| e.to_string()) }`
- `render/cli.rs` `help_text()`: after the `ayanamsa-gate` line, `  validate-ayanamsa-apparent  Run the fail-closed apparent-star ayanamsa gate (Swiss Ephemeris SEFLG_SIDEREAL default parity, issue #164) over the committed ayanamsa-apparent corpus\n  ayanamsa-apparent-gate    Alias for validate-ayanamsa-apparent\n`
- `src/tests/validate_gates.rs`: copy `validate_sidereal_position_and_alias_report_the_summary` and `help_text_mentions_validate_sidereal_position` for the new names.
- `crates/pleiades-cli/src/cli.rs` (~809): `Some("validate-ayanamsa-apparent") | Some("ayanamsa-apparent-gate") => validate_render_cli(args),`
- `docs/status.md`: a row after the sidereal-position row: `| Apparent-star ayanamsa (issue #164) | [\`pleiades-core\`](...) | \`validate-ayanamsa-apparent\` | 0.1″-class against Swiss Ephemeris; δ Cnc near the Sun <C>″ |`

- [ ] **Step 6: Run tests**

Run: `cargo test -p pleiades-validate --lib ayanamsa_apparent && cargo test -p pleiades-validate --lib validate_gates && cargo test -p pleiades-cli validate`
Expected: PASS.

- [ ] **Step 7: Measure, size the ceilings, re-run**

Run: `cargo test -p pleiades-validate --lib measure_ayanamsa_apparent -- --ignored --nocapture`
Write the measured maxima and `ceil(1.4 × max)` into `ayanamsa_apparent_thresholds.rs` (see Step 5's stop rule), then re-run Step 6.

- [ ] **Step 8: Commit**

```bash
cargo fmt --all
git add tools/se-ayanamsa-reference crates/pleiades-validate crates/pleiades-cli/src/cli.rs docs/status.md
git commit -m "feat(validate): gate the apparent-star ayanamsa against Swiss Ephemeris (#164)"
```

---

### Task 8: Apparent end-to-end rows in `validate-sidereal-position`

**Files:**
- Modify: `tools/se-sidereal-position-reference/src/main.rs`, `crates/pleiades-validate/data/sidereal-position-corpus/{sidereal-position.csv,manifest.txt}`, `crates/pleiades-validate/src/sidereal_position_validation.rs`, `sidereal_position_thresholds.rs`, `sidereal_position_validation/tests.rs`

**Interfaces:**
- Consumes: Task 5 (`with_sidereal_star_place`).
- Produces: 402 apparent rows (Sun, Moon, Mars × TrueCitra, GalacticCenter × 67 epochs) checked against `ChartRequest` with `Apparentness::Apparent` + `SiderealStarPlace::Apparent`.

- [ ] **Step 1: Extend the reference tool**

After the existing loop in `main()` (which must stay byte-identical so the 2680 mean rows do not move), append:

```rust
    // Apparent rows (issue #164 (c)): Swiss Ephemeris's default SEFLG_SIDEREAL,
    // whose ayanamsa is read from the anchor star's apparent place. The 7th
    // column marks them.
    println!("# Apparent rows: iflag=SEFLG_MOSEPH|SEFLG_SIDEREAL|SEFLG_SPEED (apparent place, nutation-free; star-anchored ayanamsa from the anchor star's apparent place). 7th column = apparent.");
    let apparent = SEFLG_MOSEPH | SEFLG_SIDEREAL | SEFLG_SPEED;
    for (ayanamsa, sid_mode) in [("TrueCitra", 27), ("GalacticCenter", 17)] {
        unsafe { swe_set_sid_mode(sid_mode, 0.0, 0.0) };
        let mut jd = JD_FIRST_TT;
        while jd < JD_END_TT {
            for (body, name) in [(0, "Sun"), (1, "Moon"), (4, "Mars")] {
                let state = se_state(jd, body, name, apparent);
                println!("{jd:.1},{ayanamsa},{name},{:.9},{:.9},{:.12},apparent", state[0], state[1], state[3]);
            }
            jd += STEP_DAYS;
        }
    }
```

Regenerate (build in devenv, run outside it) to a scratch file and confirm with `diff <(head -n <mean-row-line-count> new) old` (or `cmp` on the first N lines) that every pre-existing line is unchanged; then install it.

- [ ] **Step 2: Write the failing tests** (append to `sidereal_position_validation/tests.rs`; update the 2680 literals to 3082)

```rust
#[test]
fn an_unshifted_apparent_row_passes_alone() {
    let (csv, manifest) = apparent_row("Sun", |_| {});
    validate_with_floor(&csv, &manifest, 1).expect("passes");
}

// Without the anchor star's aberration the chart is ~20″ off Swiss Ephemeris:
// the apparent rows are what hold the composition end to end.
#[test]
fn an_apparent_row_read_with_the_mean_ayanamsa_fails() {
    // Shift the SE longitude by the correction at that row, i.e. what a chart
    // using the mean ayanamsa would match.
    let (csv, manifest) = apparent_row("Sun", |fields| {
        let jd: f64 = fields[0].parse().unwrap();
        let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
        let c = pleiades_core::apparent_star_ayanamsa_correction(&Ayanamsa::TrueCitra, instant).unwrap().degrees();
        let lon: f64 = fields[3].parse().unwrap();
        fields[3] = format!("{:.9}", lon + c);
    });
    assert_eq!(exceeded_kind(validate_with_floor(&csv, &manifest, 1)), Some("longitude_arcsec"));
}
```

with a helper `apparent_row(body, edit)` that picks the first `,TrueCitra,{body},` line ending in `,apparent`, applies `edit` to its fields, and builds a one-row corpus + manifest exactly as `shifted_row` does.

- [ ] **Step 3: Run them to verify they fail**

Run: `cargo test -p pleiades-validate --lib sidereal_position`
Expected: FAIL (7-field rows rejected as malformed; row count 3082 vs 2680).

- [ ] **Step 4: Implement**

- `Row` gains `apparent: bool`; the parser accepts 6 fields (mean) or 7 with the last exactly `apparent` (anything else → `MalformedRow`).
- Evaluation: for an apparent row build
  ```rust
  ChartRequest::new(instant)
      .with_bodies(vec![row.body.clone()])
      .with_apparentness(Apparentness::Apparent)
      .with_zodiac_mode(ZodiacMode::Sidereal { ayanamsa: row.ayanamsa.clone() })
      .with_sidereal_star_place(SiderealStarPlace::Apparent)
  ```
  and select `APPARENT_SUN_CEILINGS` / `APPARENT_MOON_CEILINGS` / `APPARENT_PLANET_CEILINGS` (new in `sidereal_position_thresholds.rs`, same `Ceilings` struct; initial values lon 2.0/6.0/4.0″, lat 1.0/6.0/1.0″, speed 0.05/3.0/0.1″/day, to be replaced by measurement in Step 5).
- `MIN_ROWS_VALIDATED = 3082`; manifest `rows=3082` and the new checksum (printed by the checksum failure).
- Report: add `apparent_sun_maxima`, `apparent_moon_maxima`, `apparent_planet_maxima`; summary line names both row kinds ("… 2680 mean and 402 apparent sidereal chart placements …").
- Add an `#[ignore]` measurement test printing the three apparent maxima (pattern of Task 7 Step 3), run it, and set each apparent ceiling to `ceil(1.4 × max)` at the existing constants' precision; record measured maxima and date in the thresholds module doc.

**Stop rule:** an apparent Sun longitude maximum above 1″ or planet above 4″ means the composition is wrong (the mean rows already hold the theory floor) — stop and report.

- [ ] **Step 5: Run tests**

Run: `cargo test -p pleiades-validate --lib sidereal_position`
Expected: PASS, including `sidereal_position_gate_passes_within_ceilings` with 3082 rows.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add tools/se-sidereal-position-reference crates/pleiades-validate
git commit -m "test(validate): hold apparent sidereal charts to Swiss Ephemeris's default SEFLG_SIDEREAL (#164)"
```

---

### Task 9: CLI `--star-place`

**Files:**
- Modify: `crates/pleiades-cli/src/parse.rs` (parser), `commands/chart.rs` (flag, cross-check, request, help string 474–483), `commands/events.rs` (`SharedArgs`, `take`, `reference()`, `render()` header, usage constants 21–23), `help.rs` (chart flags list)
- Test: `crates/pleiades-cli/src/cli/tests/chart.rs`, `cli/tests/events.rs`

**Interfaces:**
- Consumes: `pleiades_core::SiderealStarPlace` (Task 1 re-export), Task 5 builder, Task 6 builder.

- [ ] **Step 1: Write the failing tests**

`cli/tests/chart.rs`:

```rust
#[test]
fn chart_command_reads_an_apparent_star_place() {
    let base = ["--jd", "2460000.5", "--ayanamsa", "TrueCitra", "--body", "Sun"];
    let mean = render_chart(&base).expect("mean");
    let mut args = base.to_vec();
    args.extend(["--star-place", "apparent"]);
    let apparent = render_chart(&args).expect("apparent");
    assert_ne!(mean, apparent);
    assert!(apparent.contains("Sidereal star place: apparent"), "{apparent}");
}

#[test]
fn chart_command_rejects_a_star_place_without_an_ayanamsa() {
    let err = render_chart(&["--jd", "2460000.5", "--star-place", "apparent"]).unwrap_err();
    assert!(err.contains("--star-place requires --ayanamsa"), "{err}");
}

#[test]
fn chart_command_rejects_an_unknown_star_place() {
    let err = render_chart(&["--jd", "2460000.5", "--ayanamsa", "TrueCitra", "--star-place", "true"]).unwrap_err();
    assert!(err.contains("--star-place must be mean|apparent"), "{err}");
}
```

`cli/tests/events.rs` (use its `run`/`error` helpers):

```rust
#[test]
fn stations_accept_an_apparent_star_place() {
    let out = run(&["stations", "--body", "Mercury", "--next", "--at", J2000, "--ayanamsa", "TrueCitra", "--star-place", "apparent"]);
    assert!(out.contains("apparent star place"), "{out}");
    assert!(error(&["stations", "--body", "Mercury", "--next", "--at", J2000, "--star-place", "apparent"])
        .contains("--star-place requires --ayanamsa"));
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p pleiades-cli star_place`
Expected: FAIL ("unknown argument: --star-place").

- [ ] **Step 3: Implement**

`parse.rs`:

```rust
pub(crate) fn parse_star_place(value: Option<&str>) -> Result<SiderealStarPlace, String> {
    match value {
        Some("mean") => Ok(SiderealStarPlace::Mean),
        Some("apparent") => Ok(SiderealStarPlace::Apparent),
        other => Err(format!("--star-place must be mean|apparent, got {other:?}")),
    }
}
```

`chart.rs`: `let mut star_place: Option<SiderealStarPlace> = None;`; arm `"--star-place" => star_place = Some(parse_star_place(iter.next())?),`; after the loop, next to the topocentric check:
```rust
if star_place.is_some() && !matches!(zodiac_mode, ZodiacMode::Sidereal { .. }) {
    return Err("--star-place requires --ayanamsa".to_string());
}
```
and `.with_sidereal_star_place(star_place.unwrap_or_default())` on the request. Help string: after `[--ayanamsa <name>]` add `[--star-place mean|apparent]` and a sentence: "--star-place apparent reads a star-anchored ayanamsa (True Citra, Revati, Pushya, Mula, Sheoran, Galactic Center) from its anchor star's apparent place, as Swiss Ephemeris's default SEFLG_SIDEREAL does."

`events.rs`: `SharedArgs.star_place: Option<SiderealStarPlace>`; `take` arm `"--star-place" => self.star_place = Some(parse_star_place(rest.next())?),`; validation where `SharedArgs` is finished (same message as chart); `reference()` appends `.with_star_place(self.star_place.unwrap_or_default())` in the sidereal branch; `render()` header appends `", apparent star place"` inside the parentheses when it is `Apparent`. Usage constants: add `[--star-place mean|apparent]` after `[--ayanamsa <name>]`; `SHARED_HELP`: `"  --star-place apparent reads a star-anchored ayanamsa from its anchor star's apparent place (Swiss Ephemeris default).\n"`.

`help.rs`: add `--ayanamsa <name>` and `--star-place mean|apparent` to the chart flag list.

- [ ] **Step 4: Run tests**

Run: `cargo test -p pleiades-cli`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add crates/pleiades-cli
git commit -m "feat(cli): add --star-place to chart, stations and aspects (#164)"
```

---

### Task 10: Docs, compatibility profile, full verification, PR

**Files:**
- Modify: `crates/pleiades-core/README.md`, `crates/pleiades-events/README.md` (the sidereal paragraph near line 49 that says True Citra and Galactic Center differ by ~20″), `crates/pleiades-ayanamsa/README.md`, `crates/pleiades-apparent/README.md`, `docs/cli.md` (lines 59–66 and 99), `spec/api-and-ergonomics.md` (`## Configuration`, line 45–47), `crates/pleiades-core/src/compatibility/mod.rs` (id line 26, checksum line 42, append to `release_notes`), `crates/pleiades-cli/src/cli/tests/summary_commands.rs:466`, `crates/pleiades-validate/src/tests/render_request.rs:333`, `crates/pleiades-events/src/position.rs` module table (line 14: "− mean ayanamsa" → mention the star-place option)

- [ ] **Step 1: Docs**

- Each README: one short paragraph on `SiderealStarPlace` (core/events: the opt-in and what it changes; ayanamsa: `star_anchor` / `anchor_star_mean_place`; apparent: `gravitational_deflection` / `apparent_star_place`). Replace the events README's "differ by about 20″" paragraph with the measured table from the spec (10 of 15, 5 unaffected) and the option that removes the difference.
- `docs/cli.md`: document `--star-place mean|apparent` in the chart notes and the stations/aspects notes; update the sample header if it changed.
- `spec/api-and-ergonomics.md` `## Configuration`: "A convention option that matches another library's output (for example `SiderealStarPlace::Apparent`, Swiss Ephemeris's default sidereal convention) defaults to the existing behaviour, so adding it never changes a result a caller did not ask to change."

- [ ] **Step 2: Compatibility profile**

- Append to `release_notes` (one string, the pattern of the last entry): `"Apparent-star ayanamsa (issue #164 (c)): SiderealStarPlace (pleiades-types), ChartRequest::with_sidereal_star_place and CrossingReference::with_star_place read True Citra/Chitra, Revati, Pushya, Mula, Sheoran and the Galactic Center modes other than Mardyks from the anchor star's apparent place (light deflection, with Swiss Ephemeris's solar-disc taper, and annual aberration), as Swiss Ephemeris's default SEFLG_SIDEREAL does: every apparent placement including lunar points, an apparent chart's cusps and angles, and GeocentricApparentOfDate events. Up to about 22 arcseconds; the galactic-equator modes and Mardyks, which Swiss Ephemeris does not aberrate, are unchanged. Default Mean is unchanged. Gated by the fail-closed validate-ayanamsa-apparent gate (alias ayanamsa-apparent-gate), wired into run_all_numeric_gates, over a committed 1040-row Swiss Ephemeris corpus (max residual <U> arcsec uniform, <C> arcsec near the anchor star's solar conjunction), and by 402 apparent rows in validate-sidereal-position. Compatibility profile bumped to 0.7.33; API stability profile unchanged (breaking for pleiades-core: ChartRequest and ChartSnapshot gain the public field sidereal_star_place)."` — fill `<U>`/`<C>` from Task 7.
- Id → `pleiades-compatibility-profile/0.7.33`; update the two test strings (`summary_commands.rs:466`, `render_request.rs:333`).
- `grep -rn "0.7.32" crates --include=*.rs` → update any other pin.
- Run `cargo nextest run -p pleiades-core rendered_profile_matches_pinned_content_checksum`; copy the printed `{actual:#018x}` into `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` (keep the `0x____` grouping); re-run → PASS.

- [ ] **Step 3: Full verification**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
mise run docs
mise run ci
mise run test-full
cargo run -q -p pleiades-validate -- validate-ayanamsa-apparent
cargo run -q -p pleiades-validate -- validate-sidereal-position
```
Expected: all green; the two gate commands print their summary lines. (`mise run test-full` runs in the foreground; make no edits or commits while it runs.)

- [ ] **Step 4: Commit and open the PR**

```bash
git add -A
git commit -m "docs: document the apparent-star ayanamsa option; bump the compatibility profile (#164)"
git -c credential.helper= -c credential.helper='!gh auth git-credential' push -u origin worktree-issue-164c-apparent-star-ayanamsa
gh pr create --base main --title "feat(core)!: Swiss Ephemeris default parity for star-anchored ayanamsas (#164)" --body-file <scratchpad body>
```
The PR body summarizes: the measurement table, the decisions and amendments, the breaking field, the two gates with measured maxima, and the validation commands run. Merge only after CI is green (squash; never `--auto`), then comment on #164 with the measured figures and close item (c).
