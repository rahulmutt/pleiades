# Apparent Place of Date Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make apparent ecliptic position of date (light-time + precession-to-date + nutation-in-longitude + annual aberration, true equinox of date) the default chart output for release-grade bodies, validated end-to-end against a regenerated JPL Horizons apparent-of-date corpus to arcsecond tolerance.

**Architecture:** A new pure crate `pleiades-apparent` (depends only on `pleiades-types`) provides the correction math: a light-time iterator, an IAU-1976 precession (J2000 ecliptic → ecliptic of date), a checksum-pinned IAU-1980 truncated nutation series, annual aberration, and an orchestrator returning an apparent position plus `ApparentProvenance`. Backends stay mean-only and J2000; the `pleiades-core` chart facade queries backends in mean mode and, by default, drives the apparent pipeline for release-grade bodies. Non-release-grade bodies fall back to mean. Validation regenerates an apparent-of-date corpus from JPL Horizons (quantity 31) and gates the chart output to arcsec; the existing J2000 corpus is retained as a geometric-core gate.

**Tech Stack:** Rust 2021, workspace cargo, std + optional `serde`, FNV-1a checksums (byte-identical to `pleiades_time::fnv1a64`), JPL Horizons API (`https://ssd.jpl.nasa.gov/api/horizons.api`) for corpus regeneration.

**Supersedes:** `docs/superpowers/plans/2026-06-22-apparent-place-corrections.md` (parked at Task 1). Design: `docs/superpowers/specs/2026-06-22-apparent-place-of-date-design.md`.

## Global Constraints

- Edition 2021; `version.workspace = true` (0.2.0); `rust-version = 1.96.0`; `license = "MIT OR Apache-2.0"` — copy the `[package]` field-inheritance style from `crates/pleiades-time/Cargo.toml` verbatim.
- Only dependency is `pleiades-types` (workspace dep) plus optional `serde` behind a `serde` feature, matching `pleiades-time`.
- Published crate — add to `[workspace.dependencies]` and `members` (keep alphabetical; `pleiades-apparent` sorts first, before `pleiades-ayanamsa`).
- Checksums use FNV-1a 64-bit (`FNV_OFFSET_BASIS = 0xcbf2_9ce4_8422_2325`, `FNV_PRIME = 0x0000_0001_0000_01b3`), byte-identical to `pleiades_time::fnv1a64`.
- Every public error/summary type exposes `summary_line()` and `Display`, matching repo convention.
- Anchor constants (verified): `J2000 = 2451545.0`; Julian centuries `T = (jd_tt - 2451545.0) / 36525.0`; light-time per AU `= 0.005_775_518_3` days; aberration constant `κ = 20.495_52` arcsec; J2000 mean obliquity `= 23.439_291_111_111_11` degrees (matches `Instant::mean_obliquity`); general precession in longitude `≈ 5029.0966″/century`; `SECONDS_PER_DAY = 86_400.0`.
- Scope is the **astrology-standard apparent place**: light-time (planetary aberration), precession-to-date, annual aberration, nutation-in-longitude (Δψ), referred to true equinox of date. Gravitational light-deflection and the full untruncated nutation series are deliberately omitted (documented in the policy summary; absorbed by validation tolerance).
- In **ecliptic** coordinates the apparent longitude is: `λ_apparent = precess_λ(λ_J2000, β_J2000) + Δψ + Δλ(aberration)`; the apparent latitude is `β_apparent = precess_β(λ_J2000, β_J2000) + Δβ(aberration)` (Δψ is a rotation about the ecliptic pole, so it does not change latitude). Precession changes both λ and β.
- `pleiades_types::Instant` has public fields `julian_day: JulianDay` and `scale: TimeScale`; read JD as `instant.julian_day.days()`; construct via `Instant::new(JulianDay::from_days(x), scale)`.
- `pleiades_types::EclipticCoordinates { longitude: Longitude, latitude: Latitude, distance_au: Option<f64> }`, constructed via `EclipticCoordinates::new(Longitude, Latitude, Option<f64>)`. `Longitude::from_degrees(f64)`, `Longitude::degrees() -> f64`, same for `Latitude`. `Angle::from_degrees`, `.normalized_0_360()`.
- Release-grade bodies are exactly those returned by `metadata.release_grade_bodies()` (for the packaged-data backend: Sun, Moon, Mercury–Pluto, and Custom asteroid `433-Eros`). Apparent is the default for release-grade bodies; non-release-grade bodies fall back to mean (J2000) with provenance noting it — this is not an error.
- The backend boundary stays **mean-only and J2000**: apparent is never sent to a backend; the chart layer composes apparent from mean queries. The existing J2000 corpus remains valid as the geometric-core gate.

---

## Task 1: Scaffold the `pleiades-apparent` crate

**Files:**
- Create: `crates/pleiades-apparent/Cargo.toml`
- Create: `crates/pleiades-apparent/README.md`
- Create: `crates/pleiades-apparent/src/lib.rs`
- Create: `crates/pleiades-apparent/LICENSE-APACHE`, `crates/pleiades-apparent/LICENSE-MIT` (copy from `crates/pleiades-time/`)
- Modify: `Cargo.toml` (workspace `members` + `[workspace.dependencies]`)

**Interfaces:**
- Produces: `pleiades_apparent::fnv1a64(&str) -> u64`.

- [ ] **Step 1: Create the crate manifest**

`crates/pleiades-apparent/Cargo.toml`:
```toml
[package]
name = "pleiades-apparent"
description = "Apparent-place corrections for the pleiades astrology workspace: light-time, precession-to-date, annual aberration, and nutation-in-longitude with typed provenance."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[features]
serde = ["dep:serde", "pleiades-types/serde"]

[dependencies]
pleiades-types = { workspace = true }
serde = { workspace = true, optional = true }

[package.metadata.docs.rs]
all-features = true
```

- [ ] **Step 2: Create the README**

`crates/pleiades-apparent/README.md`:
```markdown
# pleiades-apparent

Apparent-place corrections for the `pleiades` workspace: light-time (planetary
aberration), precession-to-date, annual aberration, and nutation-in-longitude,
referred to the true equinox of date, with typed correction provenance. Pure
math; the chart layer supplies positions and the Sun's longitude.
```

- [ ] **Step 3: Copy the licenses**

Run: `cp crates/pleiades-time/LICENSE-APACHE crates/pleiades-time/LICENSE-MIT crates/pleiades-apparent/`
Expected: both license files present (publish audit gate requires them).

- [ ] **Step 4: Write the crate root with the shared checksum helper + a failing test**

`crates/pleiades-apparent/src/lib.rs`:
```rust
//! Apparent-place corrections: light-time, precession-to-date, annual
//! aberration, and nutation-in-longitude, with typed provenance.

/// Deterministic 64-bit content checksum (FNV-1a), byte-identical to
/// `pleiades_time::fnv1a64`. Detects drift between a checked-in data table and
/// its pinned checksum. Not cryptographic.
pub fn fnv1a64(text: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0001_0000_01b3;
    let mut hash = FNV_OFFSET_BASIS;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a64_is_deterministic_and_sensitive() {
        assert_eq!(fnv1a64("abc"), fnv1a64("abc"));
        assert_ne!(fnv1a64("abc"), fnv1a64("abd"));
    }
}
```

- [ ] **Step 5: Register the crate in the workspace**

In `Cargo.toml`, add to `members` as the first entry (alphabetical):
```toml
    "crates/pleiades-apparent",
```
And add to `[workspace.dependencies]` (before the `pleiades-ayanamsa` line):
```toml
pleiades-apparent = { path = "crates/pleiades-apparent", version = "0.2.0" }
```

- [ ] **Step 6: Build and test**

Run: `cargo test -p pleiades-apparent`
Expected: compiles; `fnv1a64_is_deterministic_and_sensitive` PASSES.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-apparent Cargo.toml
git commit -m "feat(apparent): scaffold pleiades-apparent crate with fnv1a64 helper"
```

---

## Task 2: `error` module — `ApparentPlaceError` and `ApparentLightTimeError`

**Files:**
- Create: `crates/pleiades-apparent/src/error.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs`

**Interfaces:**
- Produces:
  - `pleiades_apparent::ApparentPlaceError` with variants `NonConvergentLightTime { iterations: u8 }`, `MissingDistance`, `NonFiniteCorrection { stage: &'static str }`, `StaleModelData { kind: &'static str }`; methods `summary_line(&self) -> String`, `Display`, `std::error::Error`.
  - `pleiades_apparent::ApparentLightTimeError<E>` with variants `Query(E)`, `Apparent(ApparentPlaceError)`; `Display` where `E: Display`, `std::error::Error` where `E: Error + 'static`.

- [ ] **Step 1: Write the module with failing tests**

`crates/pleiades-apparent/src/error.rs`:
```rust
//! Structured, fail-closed errors for apparent-place computation.

use core::fmt;

/// Error returned when an apparent-place correction cannot be performed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApparentPlaceError {
    /// The light-time iteration did not converge within the iteration cap.
    NonConvergentLightTime { iterations: u8 },
    /// A position lacked the geocentric distance light-time needs.
    MissingDistance,
    /// A computed correction was not finite (defensive).
    NonFiniteCorrection { stage: &'static str },
    /// A pinned model table failed its checksum/freshness gate.
    StaleModelData { kind: &'static str },
}

impl ApparentPlaceError {
    /// Compact one-line rendering for diagnostics and release-facing summaries.
    pub fn summary_line(&self) -> String {
        match self {
            Self::NonConvergentLightTime { iterations } => {
                format!("light-time iteration did not converge after {iterations} step(s)")
            }
            Self::MissingDistance => {
                "apparent place requires a geocentric distance the position did not carry"
                    .to_string()
            }
            Self::NonFiniteCorrection { stage } => {
                format!("apparent-place correction stage `{stage}` produced a non-finite value")
            }
            Self::StaleModelData { kind } => {
                format!("{kind} apparent-model table failed its checksum/freshness gate")
            }
        }
    }
}

impl fmt::Display for ApparentPlaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for ApparentPlaceError {}

/// Error from the light-time iterator: either the caller's position query failed
/// (`Query`) or an apparent-place correction failed (`Apparent`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApparentLightTimeError<E> {
    /// The caller-supplied position query returned an error.
    Query(E),
    /// An apparent-place correction failed.
    Apparent(ApparentPlaceError),
}

impl<E: fmt::Display> fmt::Display for ApparentLightTimeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Query(error) => write!(f, "apparent-place position query failed: {error}"),
            Self::Apparent(error) => write!(f, "{error}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for ApparentLightTimeError<E> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_lines_are_distinct_and_nonempty() {
        let errors = [
            ApparentPlaceError::NonConvergentLightTime { iterations: 5 },
            ApparentPlaceError::MissingDistance,
            ApparentPlaceError::NonFiniteCorrection { stage: "aberration" },
            ApparentPlaceError::StaleModelData { kind: "nutation" },
        ];
        let mut seen = std::collections::HashSet::new();
        for e in errors {
            assert!(!e.summary_line().is_empty());
            assert_eq!(e.to_string(), e.summary_line());
            assert!(seen.insert(e.summary_line()), "duplicate summary: {}", e.summary_line());
        }
    }

    #[test]
    fn light_time_error_wraps_query_and_apparent() {
        let q: ApparentLightTimeError<&str> = ApparentLightTimeError::Query("boom");
        assert!(q.to_string().contains("boom"));
        let a: ApparentLightTimeError<&str> =
            ApparentLightTimeError::Apparent(ApparentPlaceError::MissingDistance);
        assert!(a.to_string().contains("distance"));
    }
}
```

NOTE: the `summary_lines_are_distinct_and_nonempty` test asserts genuine cross-variant distinctness (via the `HashSet`), matching its name — this corrects the name/assertion gap flagged in the parked plan's review.

- [ ] **Step 2: Wire into `lib.rs`**

Add after the doc comment in `crates/pleiades-apparent/src/lib.rs`:
```rust
mod error;

pub use error::{ApparentLightTimeError, ApparentPlaceError};
```

- [ ] **Step 3: Run the tests**

Run: `cargo test -p pleiades-apparent error::`
Expected: `summary_lines_are_distinct_and_nonempty` and `light_time_error_wraps_query_and_apparent` PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-apparent/src/error.rs crates/pleiades-apparent/src/lib.rs
git commit -m "feat(apparent): add ApparentPlaceError and ApparentLightTimeError"
```

---

## Task 3: `nutation` module — IAU-1980 truncated series + checksum gate

**Files:**
- Create: `crates/pleiades-apparent/data/nutation-iau1980.csv`
- Create: `crates/pleiades-apparent/src/nutation.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs`

**Interfaces:**
- Consumes: `ApparentPlaceError`, `fnv1a64`.
- Produces:
  - `pleiades_apparent::Nutation { delta_psi_arcsec: f64, delta_eps_arcsec: f64 }`
  - `nutation::nutation(jd_tt: f64) -> Result<Nutation, ApparentPlaceError>`
  - `nutation::mean_obliquity_degrees(jd_tt: f64) -> f64`

- [ ] **Step 1: Create the pinned data file**

`crates/pleiades-apparent/data/nutation-iau1980.csv` — principal terms of the IAU-1980 nutation theory (Meeus, *Astronomical Algorithms*, Table 22.A). Columns: multipliers of the fundamental arguments `D,M,M1,F,Om`, then `psi_a,psi_b` (Δψ sine coefficient and its T-rate, units 0.0001″) and `eps_c,eps_d` (Δε cosine coefficient and its T-rate, units 0.0001″):
```csv
D,M,M1,F,Om,psi_a,psi_b,eps_c,eps_d
0,0,0,0,1,-171996,-174.2,92025,8.9
-2,0,0,2,2,-13187,-1.6,5736,-3.1
0,0,0,2,2,-2274,-0.2,977,-0.5
0,0,0,0,2,2062,0.2,-895,0.5
0,1,0,0,0,1426,-3.4,54,-0.1
0,0,1,0,0,712,0.1,-7,0.0
-2,1,0,2,2,-517,1.2,224,-0.6
0,0,0,2,1,-386,-0.4,200,0.0
0,0,1,2,2,-301,0.0,129,-0.1
-2,-1,0,2,2,217,-0.5,-95,0.3
-2,0,1,0,0,-158,0.0,0,0.0
-2,0,0,2,1,129,0.1,-70,0.0
0,0,-1,2,2,123,0.0,-53,0.0
2,0,0,0,0,63,0.0,0,0.0
0,0,1,0,1,63,0.1,-33,0.0
2,0,-1,2,2,-59,0.0,26,0.0
0,0,-1,0,1,-58,-0.1,32,0.0
0,0,1,2,1,-51,0.0,27,0.0
-2,0,2,0,0,48,0.0,0,0.0
```

- [ ] **Step 2: Write the module with a wrong checksum + failing tests**

`crates/pleiades-apparent/src/nutation.rs`:
```rust
//! Nutation in longitude (Δψ) and obliquity (Δε) from the truncated IAU-1980
//! series, plus the mean obliquity of the ecliptic. Δψ is the term the chart
//! layer applies to ecliptic longitude; Δε is exposed for equatorial callers.

use crate::error::ApparentPlaceError;
use crate::fnv1a64;

const NUTATION_CSV: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/nutation-iau1980.csv"));

/// FNV-1a checksum of `data/nutation-iau1980.csv`; pinned in Step 4.
const NUTATION_CSV_CHECKSUM: u64 = 0; // replaced in Step 4

/// Nutation components in arcseconds.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nutation {
    /// Nutation in longitude (Δψ), arcseconds.
    pub delta_psi_arcsec: f64,
    /// Nutation in obliquity (Δε), arcseconds.
    pub delta_eps_arcsec: f64,
}

struct Term {
    multipliers: [f64; 5],
    psi_a: f64,
    psi_b: f64,
    eps_c: f64,
    eps_d: f64,
}

fn table() -> Result<Vec<Term>, ApparentPlaceError> {
    if fnv1a64(NUTATION_CSV) != NUTATION_CSV_CHECKSUM {
        return Err(ApparentPlaceError::StaleModelData { kind: "nutation" });
    }
    let mut terms = Vec::new();
    for line in NUTATION_CSV.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<f64> = line
            .split(',')
            .map(|s| s.trim().parse::<f64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ApparentPlaceError::StaleModelData { kind: "nutation" })?;
        if cols.len() != 9 {
            return Err(ApparentPlaceError::StaleModelData { kind: "nutation" });
        }
        terms.push(Term {
            multipliers: [cols[0], cols[1], cols[2], cols[3], cols[4]],
            psi_a: cols[5],
            psi_b: cols[6],
            eps_c: cols[7],
            eps_d: cols[8],
        });
    }
    Ok(terms)
}

fn julian_centuries(jd_tt: f64) -> f64 {
    (jd_tt - 2_451_545.0) / 36_525.0
}

/// Fundamental arguments (Meeus 22.x), returned in degrees.
fn fundamental_arguments(t: f64) -> [f64; 5] {
    let d = 297.85036 + 445_267.111_480 * t - 0.001_914_2 * t * t + t * t * t / 189_474.0;
    let m = 357.52772 + 35_999.050_340 * t - 0.000_160_3 * t * t - t * t * t / 300_000.0;
    let m1 = 134.96298 + 477_198.867_398 * t + 0.008_697_2 * t * t + t * t * t / 56_250.0;
    let f = 93.27191 + 483_202.017_538 * t - 0.003_682_5 * t * t + t * t * t / 327_270.0;
    let om = 125.04452 - 1_934.136_261 * t + 0.002_070_8 * t * t + t * t * t / 450_000.0;
    [d, m, m1, f, om]
}

/// Mean obliquity of the ecliptic (Meeus 22.2), degrees.
pub fn mean_obliquity_degrees(jd_tt: f64) -> f64 {
    let t = julian_centuries(jd_tt);
    // 23°26'21.448" expressed in arcseconds, minus the polynomial.
    let eps_arcsec = 84_381.448 - 46.8150 * t - 0.000_59 * t * t + 0.001_813 * t * t * t;
    eps_arcsec / 3600.0
}

/// Nutation in longitude and obliquity for a TT Julian Day.
pub fn nutation(jd_tt: f64) -> Result<Nutation, ApparentPlaceError> {
    let t = julian_centuries(jd_tt);
    let args = fundamental_arguments(t);
    let terms = table()?;
    let mut psi = 0.0_f64; // in 0.0001"
    let mut eps = 0.0_f64; // in 0.0001"
    for term in &terms {
        let mut arg_deg = 0.0;
        for (i, mult) in term.multipliers.iter().enumerate() {
            arg_deg += mult * args[i];
        }
        let arg = arg_deg.to_radians();
        psi += (term.psi_a + term.psi_b * t) * arg.sin();
        eps += (term.eps_c + term.eps_d * t) * arg.cos();
    }
    let delta_psi_arcsec = psi * 0.0001;
    let delta_eps_arcsec = eps * 0.0001;
    if !delta_psi_arcsec.is_finite() || !delta_eps_arcsec.is_finite() {
        return Err(ApparentPlaceError::NonFiniteCorrection { stage: "nutation" });
    }
    Ok(Nutation { delta_psi_arcsec, delta_eps_arcsec })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_checksum() {
        assert_eq!(
            fnv1a64(NUTATION_CSV),
            NUTATION_CSV_CHECKSUM,
            "checksum = {}",
            fnv1a64(NUTATION_CSV)
        );
    }

    #[test]
    fn meeus_example_22a() {
        // Meeus Example 22.a: 1987 April 10, 0h TD -> JDE 2446895.5.
        // Δψ = -3.788", Δε = +9.443", ε0 = 23°26'27.407" = 23.4409463°.
        let n = nutation(2_446_895.5).unwrap();
        assert!((n.delta_psi_arcsec - (-3.788)).abs() < 0.03, "Δψ = {}", n.delta_psi_arcsec);
        assert!((n.delta_eps_arcsec - 9.443).abs() < 0.03, "Δε = {}", n.delta_eps_arcsec);
        let eps0 = mean_obliquity_degrees(2_446_895.5);
        assert!((eps0 - 23.440946).abs() < 1e-5, "ε0 = {eps0}");
    }

    #[test]
    fn j2000_mean_obliquity_matches_anchor() {
        // At J2000 (t=0) the mean obliquity is the anchor constant used elsewhere.
        assert!((mean_obliquity_degrees(2_451_545.0) - 23.439_291_111_111_11).abs() < 1e-9);
    }
}
```

- [ ] **Step 3: Wire into `lib.rs`**

Add to `crates/pleiades-apparent/src/lib.rs`:
```rust
pub mod nutation;

pub use nutation::Nutation;
```

- [ ] **Step 4: Pin the checksum**

Run: `cargo test -p pleiades-apparent nutation::tests::pinned_checksum -- --nocapture`
Expected: FAIL printing `checksum = <N>`. Copy `<N>` into `NUTATION_CSV_CHECKSUM` (e.g. `const NUTATION_CSV_CHECKSUM: u64 = 0xXXXX;`).

- [ ] **Step 5: Run the nutation tests**

Run: `cargo test -p pleiades-apparent nutation::`
Expected: `pinned_checksum`, `meeus_example_22a`, `j2000_mean_obliquity_matches_anchor` PASS. If `meeus_example_22a` fails by more than the tolerance, the truncated series is too short or a coefficient was mistyped — add the next Table-22.A terms to the CSV (re-pin the checksum) until Δψ and Δε match to within 0.03″.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-apparent/data/nutation-iau1980.csv crates/pleiades-apparent/src/nutation.rs crates/pleiades-apparent/src/lib.rs
git commit -m "feat(apparent): add checksum-pinned IAU-1980 nutation series"
```

---

## Task 4: `aberration` module — annual aberration

**Files:**
- Create: `crates/pleiades-apparent/src/aberration.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs`

**Interfaces:**
- Produces:
  - `pleiades_apparent::AberrationOffset { d_lambda_arcsec: f64, d_beta_arcsec: f64 }`
  - `aberration::annual_aberration(lambda_deg: f64, beta_deg: f64, sun_true_longitude_deg: f64, jd_tt: f64) -> AberrationOffset`

- [ ] **Step 1: Write the module + failing tests**

`crates/pleiades-apparent/src/aberration.rs`:
```rust
//! Annual aberration in ecliptic coordinates (Meeus ch. 23, eq. 23.2).
//! Pure function: the caller supplies the body's ecliptic position and the
//! Sun's true longitude; this crate has no ephemeris of its own.

/// Aberration constant κ, arcseconds.
const KAPPA_ARCSEC: f64 = 20.495_52;

/// Annual-aberration offset in ecliptic longitude and latitude, arcseconds.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AberrationOffset {
    /// Aberration in ecliptic longitude (Δλ), arcseconds.
    pub d_lambda_arcsec: f64,
    /// Aberration in ecliptic latitude (Δβ), arcseconds.
    pub d_beta_arcsec: f64,
}

fn julian_centuries(jd_tt: f64) -> f64 {
    (jd_tt - 2_451_545.0) / 36_525.0
}

/// Annual aberration for an ecliptic position, given the Sun's true longitude ⊙.
///
/// Meeus 23.2:
///   Δλ = (-κ cos(⊙ - λ) + e κ cos(ϖ - λ)) / cos β
///   Δβ = -κ sin β (sin(⊙ - λ) - e sin(ϖ - λ))
/// with e the eccentricity and ϖ the longitude of perihelion of Earth's orbit
/// (Meeus 25.4 / 23.x), both of date.
pub fn annual_aberration(
    lambda_deg: f64,
    beta_deg: f64,
    sun_true_longitude_deg: f64,
    jd_tt: f64,
) -> AberrationOffset {
    let t = julian_centuries(jd_tt);
    let e = 0.016_708_634 - 0.000_042_037 * t - 0.000_000_126_7 * t * t;
    let pi_deg = 102.937_35 + 1.719_46 * t - 0.000_46 * t * t;

    let lambda = lambda_deg.to_radians();
    let beta = beta_deg.to_radians();
    let sun = sun_true_longitude_deg.to_radians();
    let pi = pi_deg.to_radians();

    let cos_beta = beta.cos();
    let d_lambda = (-KAPPA_ARCSEC * (sun - lambda).cos()
        + e * KAPPA_ARCSEC * (pi - lambda).cos())
        / cos_beta;
    let d_beta =
        -KAPPA_ARCSEC * beta.sin() * ((sun - lambda).sin() - e * (pi - lambda).sin());

    AberrationOffset { d_lambda_arcsec: d_lambda, d_beta_arcsec: d_beta }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magnitude_is_bounded_by_kappa_over_cos_beta() {
        // For modest latitudes Δλ stays within a few × κ; never explosive.
        let off = annual_aberration(100.0, 2.0, 280.0, 2_451_545.0);
        assert!(off.d_lambda_arcsec.abs() < 25.0, "Δλ = {}", off.d_lambda_arcsec);
        assert!(off.d_beta_arcsec.abs() < 1.0, "Δβ = {}", off.d_beta_arcsec);
    }

    #[test]
    fn meeus_example_23a_venus() {
        // Meeus Example 23.a: Venus 1992 Dec 20, 0h TD (JDE 2448976.5),
        // geometric λ = 88.5598°, β = -0.91549°, Sun true λ ⊙ = 268.6037°.
        // Aberration: Δλ ≈ -0.00264° = -9.5", Δβ ≈ +0.06" (small).
        let off = annual_aberration(88.5598, -0.91549, 268.6037, 2_448_976.5);
        assert!((off.d_lambda_arcsec - (-9.5)).abs() < 1.5, "Δλ = {}", off.d_lambda_arcsec);
        assert!(off.d_beta_arcsec.abs() < 1.0, "Δβ = {}", off.d_beta_arcsec);
    }
}
```

NOTE: the Meeus 23.a expected Δλ is approximate here (the worked example also includes nutation and light-time); the 1.5″ tolerance is intentional and only guards sign and order of magnitude. The end-to-end Horizons fixtures (Task 14) are the precise gate.

- [ ] **Step 2: Wire into `lib.rs`**

Add to `crates/pleiades-apparent/src/lib.rs`:
```rust
pub mod aberration;

pub use aberration::AberrationOffset;
```

- [ ] **Step 3: Run the tests**

Run: `cargo test -p pleiades-apparent aberration::`
Expected: `magnitude_is_bounded_by_kappa_over_cos_beta` and `meeus_example_23a_venus` PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-apparent/src/aberration.rs crates/pleiades-apparent/src/lib.rs
git commit -m "feat(apparent): add annual aberration in ecliptic coordinates"
```

---

## Task 5: `lighttime` module — light-time iterator

**Files:**
- Create: `crates/pleiades-apparent/src/lighttime.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs`

**Interfaces:**
- Consumes: `ApparentPlaceError`, `ApparentLightTimeError`, `pleiades_types::{EclipticCoordinates, Instant, JulianDay}`.
- Produces:
  - `pleiades_apparent::LightTimePosition { ecliptic: EclipticCoordinates, light_time_days: f64, iterations: u8 }`
  - `lighttime::LIGHT_TIME_DAYS_PER_AU: f64`
  - `lighttime::apparent_via_light_time<F, E>(instant: Instant, max_iterations: u8, query: F) -> Result<LightTimePosition, ApparentLightTimeError<E>>` where `F: FnMut(Instant) -> Result<EclipticCoordinates, E>`

- [ ] **Step 1: Write the module + failing tests**

`crates/pleiades-apparent/src/lighttime.rs`:
```rust
//! Light-time (planetary aberration) iteration: re-evaluate the geocentric
//! position at the retarded epoch t - τ until it converges.

use pleiades_types::{EclipticCoordinates, Instant, JulianDay};

use crate::error::{ApparentLightTimeError, ApparentPlaceError};

/// Light travel time across one AU, in days (≈ 499.0047 s).
pub const LIGHT_TIME_DAYS_PER_AU: f64 = 0.005_775_518_3;

/// Convergence threshold on the retardation, in days (≈ 0.04 s).
const CONVERGENCE_DAYS: f64 = 5e-7;

/// A light-time-corrected geocentric position and the retardation used.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightTimePosition {
    /// Geocentric ecliptic position at the retarded epoch.
    pub ecliptic: EclipticCoordinates,
    /// Light-time retardation applied, in days.
    pub light_time_days: f64,
    /// Iterations taken to converge.
    pub iterations: u8,
}

/// Iterates t - τ until the retardation converges. `query` returns the
/// geocentric ecliptic position (with `distance_au`) at a given instant.
pub fn apparent_via_light_time<F, E>(
    instant: Instant,
    max_iterations: u8,
    mut query: F,
) -> Result<LightTimePosition, ApparentLightTimeError<E>>
where
    F: FnMut(Instant) -> Result<EclipticCoordinates, E>,
{
    let base_jd = instant.julian_day.days();
    let mut tau = 0.0_f64;
    let mut last = query(instant).map_err(ApparentLightTimeError::Query)?;
    for step in 1..=max_iterations {
        let distance = last
            .distance_au
            .ok_or(ApparentLightTimeError::Apparent(ApparentPlaceError::MissingDistance))?;
        let new_tau = distance * LIGHT_TIME_DAYS_PER_AU;
        if !new_tau.is_finite() {
            return Err(ApparentLightTimeError::Apparent(
                ApparentPlaceError::NonFiniteCorrection { stage: "light-time" },
            ));
        }
        if (new_tau - tau).abs() < CONVERGENCE_DAYS {
            return Ok(LightTimePosition {
                ecliptic: last,
                light_time_days: new_tau,
                iterations: step,
            });
        }
        tau = new_tau;
        let retarded = Instant::new(JulianDay::from_days(base_jd - tau), instant.scale);
        last = query(retarded).map_err(ApparentLightTimeError::Query)?;
    }
    Err(ApparentLightTimeError::Apparent(
        ApparentPlaceError::NonConvergentLightTime { iterations: max_iterations },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{Latitude, Longitude, TimeScale};

    fn at(jd: f64, lon: f64, dist: f64) -> EclipticCoordinates {
        let _ = jd;
        EclipticCoordinates::new(
            Longitude::from_degrees(lon),
            Latitude::from_degrees(0.0),
            Some(dist),
        )
    }

    #[test]
    fn converges_for_a_fixed_distance_body() {
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        // A body at constant 5 AU: τ should converge to 5 × per-AU on iteration 2.
        let out = apparent_via_light_time::<_, ApparentPlaceError>(instant, 8, |i| {
            Ok(at(i.julian_day.days(), 100.0, 5.0))
        })
        .unwrap();
        assert!((out.light_time_days - 5.0 * LIGHT_TIME_DAYS_PER_AU).abs() < 1e-9);
        assert!(out.iterations <= 3);
    }

    #[test]
    fn missing_distance_is_rejected() {
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let err = apparent_via_light_time::<_, ApparentPlaceError>(instant, 8, |_| {
            Ok(EclipticCoordinates::new(
                Longitude::from_degrees(0.0),
                Latitude::from_degrees(0.0),
                None,
            ))
        })
        .unwrap_err();
        assert!(matches!(
            err,
            ApparentLightTimeError::Apparent(ApparentPlaceError::MissingDistance)
        ));
    }

    #[test]
    fn query_error_is_propagated() {
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let err = apparent_via_light_time::<_, &str>(instant, 8, |_| Err("backend down")).unwrap_err();
        assert!(matches!(err, ApparentLightTimeError::Query("backend down")));
    }
}
```

- [ ] **Step 2: Wire into `lib.rs`**

Add to `crates/pleiades-apparent/src/lib.rs`:
```rust
pub mod lighttime;

pub use lighttime::{LightTimePosition, LIGHT_TIME_DAYS_PER_AU};
```

- [ ] **Step 3: Run the tests**

Run: `cargo test -p pleiades-apparent lighttime::`
Expected: `converges_for_a_fixed_distance_body`, `missing_distance_is_rejected`, `query_error_is_propagated` PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-apparent/src/lighttime.rs crates/pleiades-apparent/src/lib.rs
git commit -m "feat(apparent): add light-time iterator"
```

---

## Task 6: `precession` module — IAU-1976 ecliptic precession (J2000 → of date)

This is the new module the parked plan lacked; it is what makes the output genuinely of-date.

**Files:**
- Create: `crates/pleiades-apparent/src/precession.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs`

**Interfaces:**
- Consumes: `ApparentPlaceError`, `nutation::mean_obliquity_degrees`.
- Produces:
  - `pleiades_apparent::PrecessedEcliptic { longitude_deg: f64, latitude_deg: f64 }`
  - `precession::precess_ecliptic_j2000_to_date(lambda_deg: f64, beta_deg: f64, jd_tt: f64) -> Result<PrecessedEcliptic, ApparentPlaceError>`

- [ ] **Step 1: Write the module + failing tests**

`crates/pleiades-apparent/src/precession.rs`:
```rust
//! Precession of ecliptic coordinates from the J2000 mean equinox/ecliptic to
//! the mean equinox/ecliptic of date. IAU-1976 equatorial precession angles
//! (Meeus 20.3 / 21.4) are bridged through the ecliptic↔equatorial rotation
//! (Meeus 13.x): convert ecliptic-J2000 -> equatorial-J2000 with the J2000
//! obliquity, precess the equatorial coordinates, then convert back to ecliptic
//! using the mean obliquity OF DATE. The result is referred to the mean
//! equinox and ecliptic of date.

use crate::error::ApparentPlaceError;
use crate::nutation::mean_obliquity_degrees;

/// J2000 mean obliquity of the ecliptic, degrees. Matches `Instant::mean_obliquity`
/// and the value the backend used to produce its J2000 ecliptic coordinates, so
/// the inbound conversion is exactly the inverse of the backend's rotation.
const OBLIQUITY_J2000_DEG: f64 = 23.439_291_111_111_11;

fn julian_centuries(jd_tt: f64) -> f64 {
    (jd_tt - 2_451_545.0) / 36_525.0
}

/// Ecliptic longitude and latitude of date, degrees (longitude normalized to 0–360).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrecessedEcliptic {
    /// Ecliptic longitude referred to the mean equinox of date, degrees [0, 360).
    pub longitude_deg: f64,
    /// Ecliptic latitude referred to the mean ecliptic of date, degrees.
    pub latitude_deg: f64,
}

/// Precesses geocentric ecliptic coordinates from the J2000 mean equinox/ecliptic
/// to the mean equinox/ecliptic of date `jd_tt`.
pub fn precess_ecliptic_j2000_to_date(
    lambda_deg: f64,
    beta_deg: f64,
    jd_tt: f64,
) -> Result<PrecessedEcliptic, ApparentPlaceError> {
    let t = julian_centuries(jd_tt);
    // IAU-1976 precession angles for a J2000 starting epoch (Meeus 20.3),
    // arcseconds -> degrees.
    let zeta = (2306.2181 * t + 0.30188 * t * t + 0.017998 * t * t * t) / 3600.0;
    let z = (2306.2181 * t + 1.09468 * t * t + 0.018203 * t * t * t) / 3600.0;
    let theta = (2004.3109 * t - 0.42665 * t * t - 0.041833 * t * t * t) / 3600.0;

    // ecliptic (J2000) -> equatorial (J2000), Meeus 13.3/13.4.
    let eps0 = OBLIQUITY_J2000_DEG.to_radians();
    let lambda = lambda_deg.to_radians();
    let beta = beta_deg.to_radians();
    let alpha0 = (lambda.sin() * eps0.cos() - beta.tan() * eps0.sin()).atan2(lambda.cos());
    let delta0 = (beta.sin() * eps0.cos() + beta.cos() * eps0.sin() * lambda.sin())
        .clamp(-1.0, 1.0)
        .asin();

    // precess equatorial (J2000) -> equatorial (of date), Meeus 21.4.
    let zeta_r = zeta.to_radians();
    let z_r = z.to_radians();
    let theta_r = theta.to_radians();
    let a = delta0.cos() * (alpha0 + zeta_r).sin();
    let b = theta_r.cos() * delta0.cos() * (alpha0 + zeta_r).cos() - theta_r.sin() * delta0.sin();
    let c = theta_r.sin() * delta0.cos() * (alpha0 + zeta_r).cos() + theta_r.cos() * delta0.sin();
    let alpha = a.atan2(b) + z_r;
    let delta = c.clamp(-1.0, 1.0).asin();

    // equatorial (of date) -> ecliptic (of date), Meeus 13.1/13.2, using the
    // mean obliquity OF DATE.
    let eps = mean_obliquity_degrees(jd_tt).to_radians();
    let lon = (alpha.sin() * eps.cos() + delta.tan() * eps.sin()).atan2(alpha.cos());
    let lat = (delta.sin() * eps.cos() - delta.cos() * eps.sin() * alpha.sin())
        .clamp(-1.0, 1.0)
        .asin();

    let longitude_deg = lon.to_degrees().rem_euclid(360.0);
    let latitude_deg = lat.to_degrees();
    if !longitude_deg.is_finite() || !latitude_deg.is_finite() {
        return Err(ApparentPlaceError::NonFiniteCorrection { stage: "precession" });
    }
    Ok(PrecessedEcliptic { longitude_deg, latitude_deg })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_at_j2000() {
        // At J2000 the precession angles are zero and the inbound/outbound
        // obliquities are equal, so the transform is the identity.
        let out = precess_ecliptic_j2000_to_date(123.456, 4.5, 2_451_545.0).unwrap();
        assert!((out.longitude_deg - 123.456).abs() < 1e-6, "λ = {}", out.longitude_deg);
        assert!((out.latitude_deg - 4.5).abs() < 1e-6, "β = {}", out.latitude_deg);
    }

    #[test]
    fn general_precession_one_century() {
        // The J2000 vernal-equinox direction (λ=0, β=0) viewed in the
        // equinox-of-date frame one Julian century on has longitude ≈ the general
        // precession in longitude (5029.0966″/cy = 1.39697°); β stays ≈ 0.
        let jd = 2_451_545.0 + 36_525.0;
        let out = precess_ecliptic_j2000_to_date(0.0, 0.0, jd).unwrap();
        assert!((out.longitude_deg - 1.39697).abs() < 5e-3, "λ' = {}", out.longitude_deg);
        assert!(out.latitude_deg.abs() < 1e-3, "β' = {}", out.latitude_deg);
    }

    #[test]
    fn longitude_shifts_by_precession_off_the_ecliptic() {
        // For an off-ecliptic point, longitude still shifts by ≈ the general
        // precession over a century; latitude moves only slightly (ecliptic motion).
        let jd = 2_451_545.0 + 36_525.0;
        let out = precess_ecliptic_j2000_to_date(80.0, 30.0, jd).unwrap();
        let dlon = out.longitude_deg - 80.0;
        assert!((dlon - 1.397).abs() < 0.05, "Δλ = {dlon}");
        assert!((out.latitude_deg - 30.0).abs() < 0.05, "β' = {}", out.latitude_deg);
    }
}
```

- [ ] **Step 2: Wire into `lib.rs`**

Add to `crates/pleiades-apparent/src/lib.rs`:
```rust
pub mod precession;

pub use precession::{precess_ecliptic_j2000_to_date, PrecessedEcliptic};
```

- [ ] **Step 3: Run the tests**

Run: `cargo test -p pleiades-apparent precession::`
Expected: `identity_at_j2000`, `general_precession_one_century`, `longitude_shifts_by_precession_off_the_ecliptic` PASS. The arcsec-level correctness is gated end-to-end against Horizons in Task 14; these unit tests guard sign, scale, and transcription.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-apparent/src/precession.rs crates/pleiades-apparent/src/lib.rs
git commit -m "feat(apparent): add IAU-1976 precession (J2000 ecliptic to of date)"
```

---

## Task 7: `provenance` module — `CorrectionSet` and `ApparentProvenance`

**Files:**
- Create: `crates/pleiades-apparent/src/provenance.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs`

**Interfaces:**
- Produces:
  - `pleiades_apparent::CorrectionSet { light_time: bool, precession: bool, annual_aberration: bool, nutation_longitude: bool }`
  - `pleiades_apparent::ApparentProvenance { light_time_days: f64, iterations: u8, precession_longitude_arcsec: f64, nutation_longitude_arcsec: f64, aberration_longitude_arcsec: f64, corrections: CorrectionSet, model_sources: &'static str }` with `summary_line(&self) -> String` + `Display`.
  - `provenance::MODEL_SOURCES: &'static str`

- [ ] **Step 1: Write the module + failing test**

`crates/pleiades-apparent/src/provenance.rs`:
```rust
//! Typed provenance describing which apparent-place corrections were applied
//! and how large they were.

use core::fmt;

/// Which corrections were applied to produce an apparent position.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CorrectionSet {
    /// Light-time (planetary aberration) was iterated.
    pub light_time: bool,
    /// Precession from J2000 to the equinox of date was applied.
    pub precession: bool,
    /// Annual aberration was applied.
    pub annual_aberration: bool,
    /// Nutation in longitude (Δψ) was applied.
    pub nutation_longitude: bool,
}

/// Data/model sources behind the apparent-place corrections.
pub const MODEL_SOURCES: &str =
    "precession (IAU-1976, Meeus 20.3/21.4); nutation-iau1980.csv (IAU-1980 truncated, Meeus Table 22.A); annual aberration (Meeus 23.2); light-time iteration; light-deflection omitted";

/// Provenance describing how an apparent position was produced.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ApparentProvenance {
    pub light_time_days: f64,
    pub iterations: u8,
    pub precession_longitude_arcsec: f64,
    pub nutation_longitude_arcsec: f64,
    pub aberration_longitude_arcsec: f64,
    pub corrections: CorrectionSet,
    pub model_sources: &'static str,
}

impl ApparentProvenance {
    /// Compact one-line rendering for diagnostics and release-facing summaries.
    pub fn summary_line(&self) -> String {
        format!(
            "apparent-place light_time={:.6}d iters={} precession_lon={:.3}\" nutation_lon={:.3}\" aberration_lon={:.3}\"",
            self.light_time_days,
            self.iterations,
            self.precession_longitude_arcsec,
            self.nutation_longitude_arcsec,
            self.aberration_longitude_arcsec,
        )
    }
}

impl fmt::Display for ApparentProvenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_line_is_nonempty_and_matches_display() {
        let p = ApparentProvenance {
            light_time_days: 0.028,
            iterations: 2,
            precession_longitude_arcsec: 1234.5,
            nutation_longitude_arcsec: -3.788,
            aberration_longitude_arcsec: -9.5,
            corrections: CorrectionSet {
                light_time: true,
                precession: true,
                annual_aberration: true,
                nutation_longitude: true,
            },
            model_sources: MODEL_SOURCES,
        };
        assert!(!p.summary_line().is_empty());
        assert_eq!(p.to_string(), p.summary_line());
        assert!(p.summary_line().contains("precession_lon"));
        assert!(p.summary_line().contains("nutation_lon"));
    }
}
```

- [ ] **Step 2: Wire into `lib.rs`**

Add to `crates/pleiades-apparent/src/lib.rs`:
```rust
mod provenance;

pub use provenance::{ApparentProvenance, CorrectionSet, MODEL_SOURCES};
```

- [ ] **Step 3: Run the test**

Run: `cargo test -p pleiades-apparent provenance::`
Expected: `summary_line_is_nonempty_and_matches_display` PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-apparent/src/provenance.rs crates/pleiades-apparent/src/lib.rs
git commit -m "feat(apparent): add ApparentProvenance and CorrectionSet"
```

---

## Task 8: `apparent` orchestrator — `apparent_position`

**Files:**
- Create: `crates/pleiades-apparent/src/apparent.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs`

**Interfaces:**
- Consumes: `aberration::annual_aberration`, `lighttime::apparent_via_light_time`, `nutation::nutation`, `precession::precess_ecliptic_j2000_to_date`, `ApparentLightTimeError`, `ApparentProvenance`, `CorrectionSet`, `MODEL_SOURCES`, `pleiades_types::{EclipticCoordinates, Instant, Latitude, Longitude}`.
- Produces:
  - `pleiades_apparent::ApparentPosition { ecliptic: EclipticCoordinates, provenance: ApparentProvenance }`
  - `apparent::apparent_position<F, E>(instant: Instant, sun_true_longitude_of_date_deg: f64, max_iterations: u8, query: F) -> Result<ApparentPosition, ApparentLightTimeError<E>>` where `F: FnMut(Instant) -> Result<EclipticCoordinates, E>`
  - `apparent::DEFAULT_MAX_ITERATIONS: u8`

- [ ] **Step 1: Write the orchestrator + failing tests**

`crates/pleiades-apparent/src/apparent.rs`:
```rust
//! Orchestrator: light-time-corrected J2000 position + Sun's longitude of date +
//! instant -> apparent ecliptic-of-date position with provenance. Applies, in
//! order: light-time, precession (J2000 -> mean equinox of date), nutation Δψ
//! (-> true equinox of date), then annual aberration.

use pleiades_types::{EclipticCoordinates, Instant, Latitude, Longitude};

use crate::aberration::annual_aberration;
use crate::error::{ApparentLightTimeError, ApparentPlaceError};
use crate::lighttime::apparent_via_light_time;
use crate::nutation::nutation;
use crate::precession::precess_ecliptic_j2000_to_date;
use crate::provenance::{ApparentProvenance, CorrectionSet, MODEL_SOURCES};

/// Default light-time iteration cap (planets converge in 2–3 steps).
pub const DEFAULT_MAX_ITERATIONS: u8 = 8;

/// An apparent ecliptic-of-date position and its provenance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ApparentPosition {
    pub ecliptic: EclipticCoordinates,
    pub provenance: ApparentProvenance,
}

/// Computes the apparent ecliptic-of-date position for a body.
///
/// `query` returns the body's geocentric ecliptic position (J2000, with
/// `distance_au`) at a given instant in mean mode. `sun_true_longitude_of_date_deg`
/// is the Sun's true geometric longitude OF DATE at `instant` (the caller is
/// responsible for precessing it), supplied for the aberration term.
pub fn apparent_position<F, E>(
    instant: Instant,
    sun_true_longitude_of_date_deg: f64,
    max_iterations: u8,
    query: F,
) -> Result<ApparentPosition, ApparentLightTimeError<E>>
where
    F: FnMut(Instant) -> Result<EclipticCoordinates, E>,
{
    let light_timed = apparent_via_light_time(instant, max_iterations, query)?;
    let jd_tt = instant.julian_day.days();
    let lambda_j2000 = light_timed.ecliptic.longitude.degrees();
    let beta_j2000 = light_timed.ecliptic.latitude.degrees();

    let precessed = precess_ecliptic_j2000_to_date(lambda_j2000, beta_j2000, jd_tt)
        .map_err(ApparentLightTimeError::Apparent)?;
    let lambda = precessed.longitude_deg;
    let beta = precessed.latitude_deg;

    let aberration = annual_aberration(lambda, beta, sun_true_longitude_of_date_deg, jd_tt);
    let nut = nutation(jd_tt).map_err(ApparentLightTimeError::Apparent)?;

    let apparent_lon =
        (lambda + (aberration.d_lambda_arcsec + nut.delta_psi_arcsec) / 3600.0).rem_euclid(360.0);
    let apparent_lat = beta + aberration.d_beta_arcsec / 3600.0;
    if !apparent_lon.is_finite() || !apparent_lat.is_finite() {
        return Err(ApparentLightTimeError::Apparent(
            ApparentPlaceError::NonFiniteCorrection { stage: "apparent-combine" },
        ));
    }

    // Precession shift in longitude for provenance, wrapped to (-180, 180].
    let mut precession_shift = lambda - lambda_j2000;
    if precession_shift > 180.0 {
        precession_shift -= 360.0;
    } else if precession_shift < -180.0 {
        precession_shift += 360.0;
    }

    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(apparent_lon),
        Latitude::from_degrees(apparent_lat),
        light_timed.ecliptic.distance_au,
    );
    let provenance = ApparentProvenance {
        light_time_days: light_timed.light_time_days,
        iterations: light_timed.iterations,
        precession_longitude_arcsec: precession_shift * 3600.0,
        nutation_longitude_arcsec: nut.delta_psi_arcsec,
        aberration_longitude_arcsec: aberration.d_lambda_arcsec,
        corrections: CorrectionSet {
            light_time: true,
            precession: true,
            annual_aberration: true,
            nutation_longitude: true,
        },
        model_sources: MODEL_SOURCES,
    };
    Ok(ApparentPosition { ecliptic, provenance })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{JulianDay, TimeScale};

    fn fixed(lon: f64, lat: f64, dist: f64) -> EclipticCoordinates {
        EclipticCoordinates::new(
            Longitude::from_degrees(lon),
            Latitude::from_degrees(lat),
            Some(dist),
        )
    }

    #[test]
    fn at_j2000_only_aberration_and_nutation_shift_longitude() {
        // At J2000 precession is the identity, so the shift from mean is only
        // (Δψ + Δλ)/3600, < ~0.01°, and the precession provenance is ~0.
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let out = apparent_position::<_, ApparentPlaceError>(instant, 280.0, 8, |_| {
            Ok(fixed(100.0, 0.0, 1.0))
        })
        .unwrap();
        let shift_arcsec = (out.ecliptic.longitude.degrees() - 100.0) * 3600.0;
        assert!(shift_arcsec.abs() < 40.0, "shift {shift_arcsec}\"");
        assert!(out.provenance.precession_longitude_arcsec.abs() < 1.0, "precession should be ~0 at J2000");
        assert!(out.provenance.corrections.precession);
        assert!(out.provenance.corrections.nutation_longitude);
        assert!(out.provenance.iterations >= 1);
    }

    #[test]
    fn precession_dominates_far_from_j2000() {
        // One century from J2000, precession shifts longitude by ~1.4° (≫ the
        // arcsec aberration/nutation), and the provenance records ~5029".
        let jd = 2_451_545.0 + 36_525.0;
        let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
        let out = apparent_position::<_, ApparentPlaceError>(instant, 280.0, 8, |_| {
            Ok(fixed(100.0, 0.0, 1.0))
        })
        .unwrap();
        let shift_deg = out.ecliptic.longitude.degrees() - 100.0;
        assert!((shift_deg - 1.397).abs() < 0.02, "shift {shift_deg}°");
        assert!((out.provenance.precession_longitude_arcsec - 5029.0).abs() < 80.0,
            "precession_lon {}\"", out.provenance.precession_longitude_arcsec);
    }

    #[test]
    fn latitude_moves_by_precession_and_aberration_only() {
        // At J2000, Δψ does not change latitude; only aberration's sub-arcsec Δβ does.
        let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
        let out = apparent_position::<_, ApparentPlaceError>(instant, 280.0, 8, |_| {
            Ok(fixed(100.0, 5.0, 1.0))
        })
        .unwrap();
        let dlat_arcsec = (out.ecliptic.latitude.degrees() - 5.0) * 3600.0;
        assert!(dlat_arcsec.abs() < 1.0, "Δβ {dlat_arcsec}\"");
    }
}
```

- [ ] **Step 2: Wire into `lib.rs`**

Add to `crates/pleiades-apparent/src/lib.rs`:
```rust
mod apparent;

pub use apparent::{apparent_position, ApparentPosition, DEFAULT_MAX_ITERATIONS};
```

- [ ] **Step 3: Run the tests**

Run: `cargo fmt -p pleiades-apparent && cargo test -p pleiades-apparent apparent::`
Expected: `at_j2000_only_aberration_and_nutation_shift_longitude`, `precession_dominates_far_from_j2000`, `latitude_moves_by_precession_and_aberration_only` PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-apparent/src/apparent.rs crates/pleiades-apparent/src/lib.rs
git commit -m "feat(apparent): add apparent_position orchestrator with precession"
```

---

## Task 9: `policy` module — `ApparentPlacePolicySummary`

**Files:**
- Create: `crates/pleiades-apparent/src/policy.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs`

**Interfaces:**
- Produces:
  - `pleiades_apparent::ApparentPlacePolicySummary` with `new(&'static str)`, `current() -> Self`, `summary_line(&self) -> &'static str`, `validate(&self) -> Result<(), ApparentPlacePolicySummaryValidationError>`, `validated_summary_line(&self)`, `Display`.
  - `pleiades_apparent::CURRENT_APPARENT_PLACE_POLICY_SUMMARY_TEXT: &'static str`.

- [ ] **Step 1: Write the module + failing tests**

`crates/pleiades-apparent/src/policy.rs`:
```rust
//! Compact, validated summary of the apparent-place posture this crate implements.

use core::fmt;

/// Canonical one-line apparent-place posture.
pub const CURRENT_APPARENT_PLACE_POLICY_SUMMARY_TEXT: &str =
    "apparent place (chart layer, default): light-time + precession-to-date + annual aberration + nutation-in-longitude, true equinox of date, release-grade bodies; gravitational light-deflection omitted";

/// Compact summary of the current apparent-place policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ApparentPlacePolicySummary {
    summary: &'static str,
}

/// Validation error for the apparent-place policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApparentPlacePolicySummaryValidationError {
    BlankSummary,
    WhitespacePaddedSummary,
    EmbeddedLineBreak,
    CurrentPolicyOutOfSync,
}

impl fmt::Display for ApparentPlacePolicySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSummary => f.write_str("apparent-place policy summary is blank"),
            Self::WhitespacePaddedSummary => {
                f.write_str("apparent-place policy summary has surrounding whitespace")
            }
            Self::EmbeddedLineBreak => {
                f.write_str("apparent-place policy summary contains a line break")
            }
            Self::CurrentPolicyOutOfSync => {
                f.write_str("apparent-place policy summary is out of sync with the current posture")
            }
        }
    }
}

impl std::error::Error for ApparentPlacePolicySummaryValidationError {}

impl ApparentPlacePolicySummary {
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }

    pub const fn current() -> Self {
        Self::new(CURRENT_APPARENT_PLACE_POLICY_SUMMARY_TEXT)
    }

    pub const fn summary_line(self) -> &'static str {
        self.summary
    }

    pub fn validate(&self) -> Result<(), ApparentPlacePolicySummaryValidationError> {
        if self.summary.trim().is_empty() {
            Err(ApparentPlacePolicySummaryValidationError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(ApparentPlacePolicySummaryValidationError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(ApparentPlacePolicySummaryValidationError::EmbeddedLineBreak)
        } else if self.summary != CURRENT_APPARENT_PLACE_POLICY_SUMMARY_TEXT {
            Err(ApparentPlacePolicySummaryValidationError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }

    pub fn validated_summary_line(
        &self,
    ) -> Result<&'static str, ApparentPlacePolicySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ApparentPlacePolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_summary_validates() {
        assert_eq!(
            ApparentPlacePolicySummary::current().validated_summary_line().unwrap(),
            CURRENT_APPARENT_PLACE_POLICY_SUMMARY_TEXT
        );
    }

    #[test]
    fn out_of_sync_summary_is_rejected() {
        assert_eq!(
            ApparentPlacePolicySummary::new("stale").validate(),
            Err(ApparentPlacePolicySummaryValidationError::CurrentPolicyOutOfSync)
        );
    }
}
```

- [ ] **Step 2: Wire into `lib.rs`**

Add to `crates/pleiades-apparent/src/lib.rs`:
```rust
pub mod policy;

pub use policy::{
    ApparentPlacePolicySummary, ApparentPlacePolicySummaryValidationError,
    CURRENT_APPARENT_PLACE_POLICY_SUMMARY_TEXT,
};
```

- [ ] **Step 3: Run the tests + full crate suite + clippy**

Run: `cargo test -p pleiades-apparent && cargo clippy -p pleiades-apparent --all-features -- -D warnings`
Expected: all tests pass; no clippy warnings.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-apparent/src/policy.rs crates/pleiades-apparent/src/lib.rs
git commit -m "feat(apparent): add ApparentPlacePolicySummary"
```

---

## Task 10: Chart-layer integration in `pleiades-core` (apparent by default)

**Files:**
- Modify: `crates/pleiades-core/Cargo.toml` (add `pleiades-apparent` dep)
- Modify: `crates/pleiades-core/src/chart/placement.rs` (add `apparent` field)
- Modify: `crates/pleiades-core/src/chart/request.rs` (validate as mean; default apparentness)
- Modify: `crates/pleiades-core/src/chart/mod.rs` (mean query + apparent transform + fallback)
- Modify: `crates/pleiades-core/src/lib.rs` (re-export apparent provenance types)
- Test: `crates/pleiades-core/src/chart/tests.rs` (apparent chart tests)

**Interfaces:**
- Consumes: `pleiades_apparent::{apparent_position, precess_ecliptic_j2000_to_date, ApparentProvenance, ApparentLightTimeError, DEFAULT_MAX_ITERATIONS}`, `metadata.release_grade_bodies()`, `EphemerisBackend::position`.
- Produces: `BodyPlacement.apparent: Option<ApparentProvenance>`; default chart placements with `position.apparent == Apparentness::Apparent` for release-grade bodies.

- [ ] **Step 1: Add the dependency**

In `crates/pleiades-core/Cargo.toml`, under `[dependencies]`, add:
```toml
pleiades-apparent = { workspace = true }
```
Run: `cargo build -p pleiades-core`
Expected: builds (dependency resolves).

- [ ] **Step 2: Add the `apparent` field to `BodyPlacement`**

In `crates/pleiades-core/src/chart/placement.rs`, add the import and field:
```rust
use pleiades_apparent::ApparentProvenance;
```
Add the field to the struct (after `house`):
```rust
    /// Apparent-place provenance, when this placement was computed in apparent mode.
    pub apparent: Option<ApparentProvenance>,
```
Run: `cargo build -p pleiades-core 2>&1 | head -30`
Expected: compile errors at every `BodyPlacement { ... }` literal that now lacks `apparent`. Fixed next.

- [ ] **Step 3: Default the new field at existing construction sites**

Run: `grep -rn "BodyPlacement {" crates/pleiades-core/src | grep -v "struct BodyPlacement"`
Add `apparent: None,` to each literal found (chart/mod.rs, chart/snapshot.rs doctests, chart/test_support.rs, chart/tests.rs).

Run: `cargo build -p pleiades-core 2>&1 | head -20`
Expected: builds.

- [ ] **Step 4: Make the default apparentness Apparent, and validate backend requests as mean**

First find how `ChartRequest` sets its apparentness default and how `validate_against_metadata` builds the per-body backend request:

Run: `grep -n "apparentness\|Apparentness\|fn default\|with_apparentness\|EphemerisRequest {" crates/pleiades-core/src/chart/request.rs`
Expected: locate the field default (or builder default) and the `body_request` construction.

Set the request's default apparentness to `Apparentness::Apparent` (whatever mechanism the struct uses — `Default` impl, `new`, or builder initial value). Then ensure the per-body validation request is built as mean (the chart never sends apparent to the backend):
```rust
            let body_request = EphemerisRequest {
                body: body.clone(),
                instant: self.instant,
                observer: self.body_observer.clone(),
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: backend_zodiac_mode.clone(),
                apparent: Apparentness::Mean,
            };
```
Ensure `Apparentness` is imported in `request.rs`.

Run: `cargo build -p pleiades-core`
Expected: builds.

- [ ] **Step 5: Write the failing apparent-chart tests**

Inspect the test harness backend:

Run: `grep -n "fn .*backend\|struct .*Backend\|release_grade\|BodyClaim\|fn chart\|ChartEngine\|positions\|metadata()\|with_apparentness\|with_bodies" crates/pleiades-core/src/chart/test_support.rs crates/pleiades-core/src/chart/tests.rs | head -50`
Expected: identify the test backend, how it advertises release-grade bodies, and the request builders.

Add to `crates/pleiades-core/src/chart/tests.rs` (the test backend must serve the Sun + the body release-grade with `distance_au = Some(..)`; reuse or extend the existing harness; for the fallback test add or flag a constrained non-release-grade body):
```rust
#[test]
fn default_chart_applies_apparent_for_release_grade_body() {
    let backend = /* existing test backend: Sun release-grade with distance */;
    let engine = ChartEngine::new(backend);
    // No explicit apparentness: default is apparent.
    let request = ChartRequest::new(Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt))
        .with_bodies(vec![CelestialBody::Sun]);
    let snapshot = engine.chart(&request).expect("default apparent chart should succeed");
    let placement = snapshot.placement_for(&CelestialBody::Sun).unwrap();
    assert_eq!(placement.position.apparent, Apparentness::Apparent);
    assert!(placement.apparent.is_some(), "apparent provenance should be attached");
}

#[test]
fn non_release_grade_body_falls_back_to_mean() {
    let backend = /* test backend serving a constrained (non-release-grade) body */;
    let engine = ChartEngine::new(backend);
    let request = ChartRequest::new(Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt))
        .with_bodies(vec![/* the constrained body */]);
    let snapshot = engine.chart(&request).expect("non-release-grade falls back, not errors");
    let placement = snapshot.placement_for(/* the constrained body */).unwrap();
    assert_eq!(placement.position.apparent, Apparentness::Mean);
    assert!(placement.apparent.is_none(), "no apparent provenance on fallback");
}

#[test]
fn explicit_mean_mode_returns_raw_j2000() {
    let backend = /* same release-grade backend */;
    let engine = ChartEngine::new(backend);
    let request = ChartRequest::new(Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt))
        .with_bodies(vec![CelestialBody::Sun])
        .with_apparentness(Apparentness::Mean);
    let snapshot = engine.chart(&request).unwrap();
    let placement = snapshot.placement_for(&CelestialBody::Sun).unwrap();
    assert_eq!(placement.position.apparent, Apparentness::Mean);
    assert!(placement.apparent.is_none());
}
```
If the existing `test_support.rs` backend marks all bodies release-grade, add a minimal constrained-body backend (or a constructor flag) in `test_support.rs` for the fallback test.

Run: `cargo test -p pleiades-core 'chart::tests::default_chart_applies_apparent_for_release_grade_body' 2>&1 | tail -20`
Expected: FAIL — the chart does not yet apply apparent corrections.

- [ ] **Step 6: Implement the apparent transform in `chart()`**

In `crates/pleiades-core/src/chart/mod.rs`:

(a) Add imports near the top of the file:
```rust
use pleiades_apparent::{apparent_position, precess_ecliptic_j2000_to_date, ApparentLightTimeError, DEFAULT_MAX_ITERATIONS};
use pleiades_backend::Apparentness;
```

(b) Ensure the per-body backend request always queries mean (find it with `grep -n "apparent:" crates/pleiades-core/src/chart/mod.rs`):
```rust
                apparent: Apparentness::Mean,
```

(c) After positions are fetched and length-checked, compute the apparent context. Add (adjust variable names to the surrounding code):
```rust
        let apparent_requested = matches!(request.apparentness, Apparentness::Apparent);
        let release_grade = metadata.release_grade_bodies();
        let sun_true_longitude_of_date = if apparent_requested {
            Some(self.query_sun_longitude_of_date(request, &backend_zodiac_mode)?)
        } else {
            None
        };
```

(d) In the placement-building closure, after sign/house logic, apply the transform (release-grade) or fall back (non-release-grade). Replace the final `Ok(BodyPlacement { body, position, sign, house })` with:
```rust
                let apparent = if let Some(sun_lon) = sun_true_longitude_of_date {
                    if release_grade.contains(&body) {
                        let body_for_query = body.clone();
                        let outcome = apparent_position::<_, EphemerisError>(
                            request.instant,
                            sun_lon,
                            DEFAULT_MAX_ITERATIONS,
                            |instant| self.query_mean_ecliptic(&body_for_query, instant, &backend_zodiac_mode),
                        )
                        .map_err(map_apparent_error)?;
                        if let Some(ecliptic) = position.ecliptic.as_mut() {
                            *ecliptic = outcome.ecliptic;
                        }
                        position.apparent = Apparentness::Apparent;
                        Some(outcome.provenance)
                    } else {
                        // Non-release-grade: graceful mean fallback (J2000), not an error.
                        position.apparent = Apparentness::Mean;
                        None
                    }
                } else {
                    position.apparent = Apparentness::Mean;
                    None
                };
                // Re-derive the sign from the final (possibly apparent) longitude.
                let sign = position
                    .ecliptic
                    .as_ref()
                    .map(|coords| pleiades_types::ZodiacSign::from_longitude(coords.longitude))
                    .or(sign);
                Ok(BodyPlacement { body, position, sign, house, apparent })
```
NOTE: keep existing sidereal sign handling; the re-derivation only refreshes the sign from the final longitude. If the borrow checker objects to `body` being borrowed and later moved, the `body_for_query` clone above resolves it.

(e) Add helper methods to `impl<B: EphemerisBackend> ChartEngine<B>` and a free error mapper:
```rust
    fn query_mean_ecliptic(
        &self,
        body: &CelestialBody,
        instant: Instant,
        zodiac_mode: &ZodiacMode,
    ) -> Result<pleiades_types::EclipticCoordinates, EphemerisError> {
        let req = EphemerisRequest {
            body: body.clone(),
            instant,
            observer: None,
            frame: CoordinateFrame::Ecliptic,
            zodiac_mode: zodiac_mode.clone(),
            apparent: pleiades_backend::Apparentness::Mean,
        };
        let result = self.backend.position(&req)?;
        result.ecliptic.ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("apparent place requires ecliptic coordinates for {body}"),
            )
        })
    }

    fn query_sun_longitude_of_date(
        &self,
        request: &ChartRequest,
        zodiac_mode: &ZodiacMode,
    ) -> Result<f64, EphemerisError> {
        let ecliptic = self.query_mean_ecliptic(&CelestialBody::Sun, request.instant, zodiac_mode)?;
        // Precess the Sun's J2000 longitude to of date so the aberration term is consistent.
        let precessed = precess_ecliptic_j2000_to_date(
            ecliptic.longitude.degrees(),
            ecliptic.latitude.degrees(),
            request.instant.julian_day.days(),
        )
        .map_err(|e| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("apparent-place Sun precession failed: {e}"),
            )
        })?;
        Ok(precessed.longitude_deg)
    }
```
And the free error mapper (near `map_house_error`):
```rust
fn map_apparent_error(error: ApparentLightTimeError<EphemerisError>) -> EphemerisError {
    match error {
        ApparentLightTimeError::Query(inner) => inner,
        ApparentLightTimeError::Apparent(inner) => EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("apparent-place computation failed: {inner}"),
        ),
    }
}
```

Run: `cargo build -p pleiades-core 2>&1 | head -30`
Expected: builds (resolve any borrow issues with the `body_for_query` clone).

- [ ] **Step 7: Run the new chart tests**

Run: `cargo test -p pleiades-core chart::tests::default_chart_applies_apparent_for_release_grade_body chart::tests::non_release_grade_body_falls_back_to_mean chart::tests::explicit_mean_mode_returns_raw_j2000`
Expected: all three PASS.

- [ ] **Step 8: Re-export the apparent provenance types from `pleiades-core`**

Add to `crates/pleiades-core/src/lib.rs` (next to other re-exports):
```rust
pub use pleiades_apparent::{ApparentProvenance, CorrectionSet};
```
Run: `cargo build -p pleiades-core`
Expected: builds.

- [ ] **Step 9: Commit (full-suite migration happens in Task 12)**

Run: `cargo test -p pleiades-core chart::tests::default_chart_applies_apparent_for_release_grade_body chart::tests::non_release_grade_body_falls_back_to_mean chart::tests::explicit_mean_mode_returns_raw_j2000`
Expected: pass. (The rest of the `pleiades-core` suite — snapshot/longitude goldens — will fail until Task 12 regenerates them; that is expected and handled there. Do NOT update those goldens in this task.)

```bash
git add crates/pleiades-core
git commit -m "feat(core): compute apparent-place of date as the default chart output"
```

---

## Task 11: CLI apparent-by-default output + `--mean` diagnostic

**Files:**
- Modify: `crates/pleiades-cli/src/commands/chart.rs`
- Test: `crates/pleiades-cli/src/commands/chart.rs` (inline test) or the CLI test module

**Interfaces:**
- Consumes: `ChartSnapshot.placements[*].apparent: Option<ApparentProvenance>`; the (now apparent-by-default) chart engine.
- Produces: apparent-provenance line per release-grade body by default; a `--mean` flag for diagnostic raw-J2000 output.

- [ ] **Step 1: Find the chart command's flag parsing and output assembly**

Run: `grep -n "fn render_chart\|Apparentness\|--apparent\|--mean\|with_apparentness\|snapshot\|push_str\|format!\|provenance\|Display for ChartSnapshot" crates/pleiades-cli/src/commands/chart.rs | tail -50`
Expected: locate where apparentness is set from flags and where the snapshot is rendered.

- [ ] **Step 2: Write failing tests**

Add to the CLI chart command's test module (find the pattern with `grep -n "#\[test\]\|render_chart(" crates/pleiades-cli/src/commands/chart.rs`). Use a JD inside 1900–2100 and the release-grade Sun:
```rust
#[test]
fn default_chart_emits_apparent_provenance_line() {
    let out = render_chart(&["--jd", "2451545.0", "--body", "Sun"]).unwrap();
    assert!(out.contains("apparent-place"), "missing provenance line in:\n{out}");
}

#[test]
fn mean_flag_suppresses_apparent_provenance() {
    let out = render_chart(&["--jd", "2451545.0", "--body", "Sun", "--mean"]).unwrap();
    assert!(!out.contains("apparent-place"), "mean output should have no provenance line:\n{out}");
}
```
NOTE: confirm the default CLI backend is the packaged-data backend (so the Sun is release-grade with distance); check with `grep -n "Backend\|engine\|ChartEngine::new" crates/pleiades-cli/src/commands/chart.rs`.

Run: `cargo test -p pleiades-cli default_chart_emits_apparent_provenance_line mean_flag_suppresses_apparent_provenance 2>&1 | tail -20`
Expected: FAIL — default still mean / no provenance line / no `--mean` flag yet.

- [ ] **Step 3: Add the `--mean` flag and default-apparent behavior**

In `crates/pleiades-cli/src/commands/chart.rs`: add a `--mean` flag that sets `Apparentness::Mean` on the request; with no flag the request keeps its default (apparent). If a legacy `--apparent` flag exists, keep it as an explicit no-op alias for the default. Ensure `--mean` and `--apparent` conflict is rejected with a message mentioning both flags.

- [ ] **Step 4: Render the provenance line**

In the output assembly, after the per-body placement lines, append the apparent provenance when present:
```rust
    for placement in &snapshot.placements {
        if let Some(provenance) = &placement.apparent {
            output.push_str(&format!("  {} apparent: {}\n", placement.body, provenance.summary_line()));
        }
    }
```
(Adapt `output`/`push_str` to how this command accumulates its string.)

Run: `cargo test -p pleiades-cli default_chart_emits_apparent_provenance_line mean_flag_suppresses_apparent_provenance`
Expected: PASS.

- [ ] **Step 5: Update the inline `--help`/usage text**

Ensure the usage string lists `--mean` (diagnostic, raw J2000) and states apparent of date is the default for release-grade bodies. Update both the inline synopsis and any separate help block.

- [ ] **Step 6: Commit (snapshot CLI tests migrate in Task 12)**

Run: `cargo test -p pleiades-cli default_chart_emits_apparent_provenance_line mean_flag_suppresses_apparent_provenance`
Expected: pass. (Other CLI chart snapshot tests shift with the default flip and are regenerated in Task 12.)

```bash
git add crates/pleiades-cli/src/commands/chart.rs
git commit -m "feat(cli): apparent place of date by default; --mean diagnostic flag"
```

---

## Task 12: Workspace golden migration (default-apparent behavior change)

The default-apparent flip shifts every chart longitude/sign/snapshot that previously
asserted J2000 mean output. This task regenerates those goldens and verifies each
change is legitimate (a precession-sized shift), not a masked regression.

**Files:**
- Modify: snapshot/golden test files across the workspace (enumerated in Step 1).

**Interfaces:** none (test-data migration only).

- [ ] **Step 1: Inventory what shifts and identify the snapshot mechanism**

Run: `cargo test --workspace 2>&1 | tee /tmp/apparent-migration-failures.txt | tail -40`
Run: `grep -rn "insta\|assert_snapshot\|expect_test\|expect!\|UPDATE_EXPECT\|\.snap" crates/*/Cargo.toml crates/*/src crates/*/tests 2>/dev/null | head -20`
Expected: a list of failing tests, and the snapshot framework in use (e.g. `insta`, `expect-test`, or hand-pinned `assert_eq!` on longitudes). Record which crates have failures (expected: `pleiades-core`, `pleiades-cli`, possibly `pleiades-validate` if it asserts chart longitudes).

- [ ] **Step 2: Triage every failure**

For each failing test, classify it:
- **(a) Expected shift** — a chart longitude/sign/snapshot that moved because output is now apparent of date. The new longitude must differ from the old by approximately the precession at that epoch (≈ 50.3″ × years-from-2000, up to ~1.4°) plus ≤ ~40″ of nutation+aberration, in the direction of increasing of-date longitude for past→future. Verify the delta is in this range before accepting.
- **(b) Unexpected** — any failure that is NOT a chart-output longitude shift, or whose delta is far outside the precession-sized band. STOP and investigate; this may be a real bug introduced by Tasks 10–11. Do not "fix" it by editing the golden.

List the (a) and (b) sets explicitly in your report.

- [ ] **Step 3: Regenerate snapshot-framework goldens**

For tests using a snapshot framework, regenerate via its accept mechanism (use the one discovered in Step 1), e.g. one of:
- `insta`: `cargo insta test --accept -p <crate>` (or `INSTA_UPDATE=always cargo test -p <crate>`)
- `expect-test`: `UPDATE_EXPECT=1 cargo test -p <crate>`

Run the relevant command per affected crate, then `git diff --stat` to confirm only snapshot files changed.

- [ ] **Step 4: Update hand-pinned longitude assertions**

For each hand-pinned `assert_eq!`/`assert!` on a longitude or sign (the (a) set), replace the old value with the engine's new apparent value. For each one, record in your report: file:line, old value, new value, epoch, and the precession-sized delta that justifies it.

- [ ] **Step 5: Re-run the full workspace suite**

Run: `cargo test --workspace 2>&1 | tail -30`
Expected: 0 failures. If any (b)-class failure remains, it is a real regression — fix the code (or escalate), not the golden.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "test: regenerate chart goldens for apparent-place-of-date default"
```

---

## Task 13: Policy, docs, and README alignment

**Files:**
- Modify: `crates/pleiades-backend/src/policy/mod.rs` (apparentness + unsupported-modes text)
- Modify: `crates/pleiades-backend/src/policy/apparentness.rs` tests if they assert exact text
- Modify: `crates/pleiades-validate/src` posture/summary strings that restate the old posture
- Modify: `docs/time-observer-policy.md`
- Modify: `README.md`
- Modify: `PLAN.md`
- Modify: `plan/stages/04-advanced-request-modes.md`

**Interfaces:** none (documentation/posture only).

- [ ] **Step 1: Find every place the old "apparent unsupported" posture is asserted**

Run: `grep -rn "apparent" crates/pleiades-backend/src/policy/mod.rs README.md docs/time-observer-policy.md PLAN.md plan/stages/04-advanced-request-modes.md`
Run: `grep -rn "apparent" crates/pleiades-validate/src | head -30`
Expected: the list of strings to update. Read `crates/pleiades-backend/src/policy/mod.rs` around `CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT` and `CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT`.

- [ ] **Step 2: Update the apparentness + unsupported-modes posture text**

In `crates/pleiades-backend/src/policy/mod.rs`: update `CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT` to state that **backends remain mean-only and J2000 at the backend boundary**, while **apparent place of date (light-time + precession-to-date + annual aberration + nutation-in-longitude, release-grade bodies) is the default chart-layer output**. Update `CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT` to remove apparent place from the unsupported list (topocentric and native sidereal remain). Keep each a single line, no surrounding whitespace, no embedded newlines (validators enforce this).

- [ ] **Step 3: Fix policy-summary tests that pin the exact old wording**

Run: `cargo test -p pleiades-backend 2>&1 | tail -30`
Expected: failures only where a test asserts the old apparent/unsupported wording. Update those assertions to the new wording. Re-run until green.

- [ ] **Step 4: Update validation summaries that echo the posture**

Update any `pleiades-validate` release/validation summary that restates "apparent unsupported" to the chart-layer-default posture. Run `cargo test -p pleiades-validate 2>&1 | tail -30` and fix any pinned-string failures.

- [ ] **Step 5: Update README, docs, PLAN, and the Phase-4 stage doc**

- `README.md`: change the apparent-place limit bullet to state apparent place of date is the default chart output for release-grade bodies (light-time + precession + aberration + nutation-in-longitude), backends remaining mean-only/J2000, light-deflection omitted. Add the `pleiades-apparent` crate to the crate list/count.
- `docs/time-observer-policy.md`: update the apparentness section to the implemented chart-layer-default posture; document the existing J2000 corpus as the geometric-core gate.
- `PLAN.md`: in "Important current limits" and "Current priority", mark apparent place of date complete; narrow remaining Phase 4 work to topocentric + native sidereal. Update the Status line date to 2026-06-22.
- `plan/stages/04-advanced-request-modes.md`: move apparent place to a "Completed" subsection mirroring the civil-time entry; leave topocentric + native sidereal as remaining.

- [ ] **Step 6: Build + test the touched crates**

Run: `cargo test -p pleiades-backend -p pleiades-validate -p pleiades-core -p pleiades-cli 2>&1 | tail -20`
Expected: all pass.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "docs(policy): apparent place of date is the default chart-layer output"
```

---

## Task 14: Validation fixtures — Horizons apparent-of-date goldens + offline gate

**Files:**
- Create: `crates/pleiades-validate/scripts/regen-apparent-goldens.sh` (Horizons fetch; run once)
- Create: `crates/pleiades-validate/data/apparent-goldens.csv` (committed goldens)
- Create: `crates/pleiades-validate/src/apparent_validation.rs` (fail-closed cross-check)
- Modify: `crates/pleiades-validate/src/lib.rs` (wire the module + any CLI subcommand)

**Interfaces:**
- Consumes: the packaged-data backend + chart engine (apparent default); the committed goldens.
- Produces: `validate_apparent_goldens() -> Result<ApparentValidationReport, ApparentValidationError>`; a fail-closed apparent-place validation gate.

- [ ] **Step 1: Write the Horizons regeneration script**

`crates/pleiades-validate/scripts/regen-apparent-goldens.sh` — fetches Horizons quantity 31 (apparent ecliptic longitude of date) for release-grade bodies at epochs spanning 1900–2100 and writes the goldens CSV. The endpoint is `https://ssd.jpl.nasa.gov/api/horizons.api` (confirmed reachable; the host the project's `pleiades-jpl` ingest uses).
```bash
#!/usr/bin/env bash
# Regenerate apparent-of-date goldens from JPL Horizons (quantity 31, ObsEcLon:
# apparent ecliptic longitude referred to the true equinox & ecliptic of date),
# geocentric observer (500@399). Run manually when refreshing the corpus.
set -euo pipefail
OUT="$(dirname "$0")/../data/apparent-goldens.csv"
API="https://ssd.jpl.nasa.gov/api/horizons.api"
EPOCHS="2415020.5 2433282.5 2451545.0 2469807.5 2488070.5" # 1900,1950,2000,2050,2100 (JD)
# body label -> Horizons COMMAND code
BODIES="Sun:10 Moon:301 Mercury:199 Venus:299 Mars:499 Jupiter:599 Saturn:699 Uranus:799 Neptune:899 Pluto:999 433-Eros:2000433"
{
  echo "# Source: JPL Horizons API ($API), EPHEM_TYPE=OBSERVER, CENTER='500@399',"
  echo "# QUANTITIES='31' (ObsEcLon, apparent ecliptic longitude of date), ANG_FORMAT=DEG, extra_prec=YES."
  echo "# Regenerated by crates/pleiades-validate/scripts/regen-apparent-goldens.sh"
  echo "body,jd_tt,apparent_longitude_deg,tolerance_arcsec"
  for entry in $BODIES; do
    label="${entry%%:*}"; code="${entry##*:}"
    # Moon is theory-limited under the generic pipeline (annual aberration is
    # mis-applied to a body sharing Earth's velocity); give it a looser tolerance.
    if [ "$label" = "Moon" ]; then tol=30.0; else tol=5.0; fi
    for jd in $EPOCHS; do
      lon=$(curl -sS -m 30 "$API?format=text&COMMAND='$code'&EPHEM_TYPE=OBSERVER&CENTER='500@399'&TLIST='$jd'&QUANTITIES='31'&ANG_FORMAT=DEG&CAL_FORMAT=JD&extra_prec=YES" \
        | sed -n '/\$\$SOE/,/\$\$EOE/p' | sed '1d;$d' | awk '{print $2}')
      echo "$label,$jd,$lon,$tol"
    done
  done
} > "$OUT"
echo "wrote $OUT"
```

- [ ] **Step 2: Generate the goldens and pin the checksum**

Run: `chmod +x crates/pleiades-validate/scripts/regen-apparent-goldens.sh && crates/pleiades-validate/scripts/regen-apparent-goldens.sh`
Expected: `data/apparent-goldens.csv` is written with real numeric longitudes (55 rows). Open it and confirm every `apparent_longitude_deg` is a number in [0,360) (no blanks/errors). If any row is blank, the body code or query failed — fix and re-run.

- [ ] **Step 3: Write the fail-closed cross-check (with a wrong checksum + failing test)**

First read the established validation pattern:

Run: `grep -rn "include_str!\|fn validate\|pub fn\|ChartEngine\|PackagedDataBackend\|fail\|fnv1a64\|CHECKSUM" crates/pleiades-validate/src/house_validation.rs | head -30`
Expected: the structure for a fail-closed validation function + how the crate builds a chart and pins data checksums.

`crates/pleiades-validate/src/apparent_validation.rs` — load the goldens (checksum-gated), build an apparent chart per row via the packaged backend + `ChartEngine`, assert the chart's apparent longitude is within the per-row tolerance. Mirror `house_validation.rs` for the report/error types and checksum gate. Structure:
```rust
//! Fail-closed cross-check of the engine's apparent-of-date longitudes against
//! JPL Horizons goldens (quantity 31). Reads the committed CSV offline.

const GOLDENS_CSV: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/apparent-goldens.csv"));
const GOLDENS_CHECKSUM: u64 = 0; // pinned in Step 4

// ApparentValidationReport / ApparentValidationError mirroring house_validation.rs
// (summary_line + Display on the error, per repo convention).

// fn parse rows (skip '#' comments and the header); fail closed on malformed rows.
// For each row: resolve the CelestialBody, build a default (apparent) ChartRequest
// at Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tt) for that one body via
// the packaged backend + ChartEngine, read the placement's apparent ecliptic
// longitude, and compare to apparent_longitude_deg with a wrap-aware difference:
//   let mut d = (got - want).abs(); if d > 180.0 { d = 360.0 - d; }
//   if d * 3600.0 > tolerance_arcsec { return Err(... row exceeds tolerance ...) }
// Fail closed on: checksum mismatch, malformed row, unknown/non-release-grade
// body, chart error, or any row exceeding tolerance.

pub fn validate_apparent_goldens() -> Result<ApparentValidationReport, ApparentValidationError> {
    // ... implementation ...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_checksum() {
        assert_eq!(
            pleiades_apparent::fnv1a64(GOLDENS_CSV),
            GOLDENS_CHECKSUM,
            "checksum = {}",
            pleiades_apparent::fnv1a64(GOLDENS_CSV)
        );
    }

    #[test]
    fn apparent_goldens_pass() {
        validate_apparent_goldens().expect("apparent goldens within tolerance");
    }
}
```
NOTE: `pleiades-validate` must depend on `pleiades-apparent` (for `fnv1a64`) and on the packaged backend + `pleiades-core` chart engine (it already uses them for other validations — confirm in its `Cargo.toml`). Add `pleiades-apparent = { workspace = true }` if missing.

- [ ] **Step 4: Wire the module + pin the checksum + tighten tolerances**

Add `pub mod apparent_validation;` to `crates/pleiades-validate/src/lib.rs`. If the crate exposes CLI subcommands (check `grep -rn "validate-corpus\|match.*command\|subcommand" crates/pleiades-validate/src crates/pleiades-cli/src`), register a `validate-apparent` command alongside `validate-corpus`.

Run: `cargo test -p pleiades-validate apparent_validation::tests::pinned_checksum -- --nocapture`
Expected: FAIL printing `checksum = <N>`. Copy `<N>` into `GOLDENS_CHECKSUM`.

Run: `cargo test -p pleiades-validate apparent`
Expected: `apparent_goldens_pass` PASS. If a non-Moon row exceeds 5″, first check it is not a precession/sign error in the engine; if the model genuinely needs more room (omitted light-deflection, truncated nutation), raise only that body's `tolerance_arcsec` to the smallest value it passes at and document why in the CSV header. Do not loosen tolerances to mask a precession-direction bug.

- [ ] **Step 5: Add the frame-guard regression test**

`crates/pleiades-apparent` cannot see the backend, so add this in `pleiades-validate` (or `pleiades-core` chart tests) using the packaged backend: a default (apparent) chart for the Sun at JD 2433283.0 (1950) yields an ecliptic longitude within 0.05° of ~280.4° (the of-date apparent value; the old J2000 output was 281.22°). This is the corrected, now-passing version of the original Task 1 guard.
```rust
#[test]
fn sun_apparent_longitude_is_of_date_1950() {
    // Of-date apparent Sun at 1950-01-01 12:00 TT ≈ 280.4°; J2000 was 281.22°.
    let backend = /* packaged-data backend constructor */;
    let engine = ChartEngine::new(backend);
    let request = ChartRequest::new(Instant::new(JulianDay::from_days(2_433_283.0), TimeScale::Tt))
        .with_bodies(vec![CelestialBody::Sun]);
    let snapshot = engine.chart(&request).unwrap();
    let lon = snapshot.placement_for(&CelestialBody::Sun).unwrap()
        .position.ecliptic.as_ref().unwrap().longitude.degrees();
    let mut d = (lon - 280.4_f64).abs(); if d > 180.0 { d = 360.0 - d; }
    assert!(d < 0.05, "apparent Sun longitude {lon}° not within 0.05° of of-date 280.4°");
}
```
Run the test: `cargo test -p pleiades-validate sun_apparent_longitude_is_of_date_1950` (or the crate you placed it in).
Expected: PASS.

- [ ] **Step 6: Full workspace gate**

Run: `cargo test --workspace 2>&1 | tail -30 && cargo clippy --workspace --all-features -- -D warnings 2>&1 | tail -20 && cargo fmt --all --check`
Expected: all tests pass, no clippy warnings, formatting clean.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-validate
git commit -m "test(apparent): Horizons apparent-of-date goldens gate + 1950 frame regression"
```

---

## Self-Review

**Spec coverage:**
- Scope (light-time + precession-to-date + aberration + nutation-in-longitude, true equinox of date, deflection omitted) → Tasks 3–9 + Global Constraints.
- New pure `pleiades-apparent` crate (checksum-pinned, typed provenance) → Tasks 1–9; the precession module that the parked plan lacked → Task 6.
- Apparent of date as the **default** chart output; mean (J2000) diagnostic mode; non-release-grade graceful mean fallback; `BodyPlacement.apparent`; Sun precessed for the aberration term → Task 10.
- CLI default-apparent output + `--mean` diagnostic + provenance line → Task 11.
- Behavior-change migration (workspace golden regeneration with a precession-sized verification rule) → Task 12.
- Policy/docs/README/PLAN/stage alignment; existing J2000 corpus retained as the geometric-core gate → Task 13.
- Validation: regenerated Horizons apparent-of-date (Q31) corpus, fail-closed arcsec gate, offline after regeneration; 1950 frame-guard regression (now passing) → Task 14.
- Error handling (fail-closed; precession `NonFiniteCorrection { stage: "precession" }`) → Tasks 6, 8.

**Intentional, non-forbidden placeholders:** (a) the two `… = 0;` checksum constants are the standard print-and-pin step (Task 3 Step 4, Task 14 Step 4). (b) A few `/* … */` markers in test code (Task 10 Step 5, Task 14 Step 5) name the exact test-backend wiring to substitute and are each preceded by a `grep` discovery step — this matches the established pattern of the superseded plan and the repo's other validation modules, where the harness/type names must be read from existing code rather than guessed. (c) Task 14 Step 3 specifies `validate_apparent_goldens()` by precise behavior + the exact wrap-aware comparison logic and points at `house_validation.rs` for the report/error type pattern, because that module must conform to an existing in-repo convention.

**Type consistency:** `apparent_position(instant, sun_true_longitude_of_date_deg, max_iterations, query) -> Result<ApparentPosition, ApparentLightTimeError<E>>` (Task 8) is called identically in Task 10. `precess_ecliptic_j2000_to_date(lambda_deg, beta_deg, jd_tt) -> Result<PrecessedEcliptic, ApparentPlaceError>` (Task 6) is called consistently in Tasks 8 and 10. `CorrectionSet` carries four flags (`light_time`, `precession`, `annual_aberration`, `nutation_longitude`) in Tasks 7 and 8. `ApparentProvenance` fields (incl. `precession_longitude_arcsec`) match between Tasks 7 and 8. `BodyPlacement.apparent: Option<ApparentProvenance>` (Task 10) is consumed in Task 11. `pleiades_apparent::fnv1a64` (Task 1) is reused in Task 14. `Nutation` / `AberrationOffset` / `LightTimePosition` / `PrecessedEcliptic` field names are consistent across producer and consumer tasks.

**Task-1 dependency note (for the executor):** Tasks 1–9 build the pure crate and are independently testable offline. Task 10 flips the default and will (intentionally) break workspace goldens until Task 12; Tasks 10/11 each commit only their own focused tests, deferring the golden regeneration to Task 12 — do not regenerate goldens early. Task 14 requires one online Horizons fetch (Step 2) to materialize the corpus; everything else runs offline.

