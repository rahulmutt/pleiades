# Ayanamsa Audit + Numeric Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a fail-closed `validate-ayanamsa` numeric gate that checks the six release-claimed ayanamsa modes against Swiss Ephemeris within measured per-mode-class arcsecond ceilings, and correct those modes' computation so they actually match SE.

**Architecture:** Mirror the just-merged house gate. A dev-only `tools/se-ayanamsa-reference` binary (links real Swiss Ephemeris, lives **outside** the published workspace) emits a committed SE reference corpus and the fitted true-star polynomial coefficients. The `pleiades-ayanamsa` computation is corrected — offset-defined modes get an IAU-2006 general-precession-in-longitude drift term anchored at each mode's reference epoch; true-star modes (True Chitra/Citra) get a committed cubic polynomial fit to SE. A new pure-Rust `validate_ayanamsa_corpus()` in `pleiades-validate` recomputes every corpus row and fails closed on checksum/manifest/parse drift or a residual over the mode-class ceiling. Ceilings are set from measured residuals with a 2× margin (house precedent).

**Tech Stack:** Rust (workspace crates `pleiades-ayanamsa`, `pleiades-validate`, `pleiades-apparent`, `pleiades-types`, `pleiades-core`); Swiss Ephemeris via the `swisseph` 0.1.1 / `libswisseph-sys` 0.1.2 crates (verification-only, never shipped).

## Global Constraints

- **C1 — Pure-Rust workspace audit (hard).** `workspace-audit` fails closed on `links` assignments, `-sys` dependencies, `build.rs`, and lockfile packages ending in `-sys`. The SE generator **must not** be a workspace member and **must not** enter the workspace `Cargo.lock`. It lives at `tools/se-ayanamsa-reference/` with its **own** `Cargo.lock`, exactly like `tools/se-house-reference/`.
- **Pure-Rust shipping crates.** No new dependency may be added to any `crates/*` crate for this work. SE is touched only by the `tools/` binary.
- **Checksum function.** Corpus checksums use `pleiades_apparent::fnv1a64(&str) -> u64` (the project's established FNV-1a-64, which uses a non-canonical prime). Do not substitute a standard FNV implementation.
- **No claim broadening.** Only the six release-claimed modes (Lahiri, Raman, Krishnamurti, Fagan/Bradley, True Chitra, True Citra) are numerically gated. The other built-ins keep their existing descriptor tests and are recorded as not-yet-gated.
- **Time scale.** All Julian Days in the corpus and in the gate are **TT** (`TimeScale::Tt`). The generator calls SE's ET/TT entry point `swisseph::swe::get_ayanamsa(tjd_et)` (NOT `get_ayanamsa_ut`), so no ΔT conversion is needed.
- **TDD, frequent commits, DRY, YAGNI.**

---

## File Structure

| File | Responsibility | Action |
| --- | --- | --- |
| `tools/se-ayanamsa-reference/Cargo.toml` | Dev-only SE generator manifest (own lockfile, outside workspace) | Create |
| `tools/se-ayanamsa-reference/src/main.rs` | Emit SE reference corpus CSV + fit true-star cubic coefficients | Create |
| `tools/se-ayanamsa-reference/LICENSE-NOTES.md` | SE license posture (verification-only) | Create |
| `crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv` | Committed SE reference values (source of truth) | Create (generated) |
| `crates/pleiades-validate/data/ayanamsa-corpus/manifest.txt` | Engine version + row count + fnv1a64 checksum | Create (generated) |
| `crates/pleiades-ayanamsa/src/precession.rs` | IAU-2006 general precession in longitude `pA(T)` (arcsec) | Create |
| `crates/pleiades-ayanamsa/src/truestar.rs` | Committed cubic polynomials for True Chitra/Citra fit to SE | Create |
| `crates/pleiades-ayanamsa/src/thresholds.rs` | Per-mode-class arcsecond ceilings + summary | Create |
| `crates/pleiades-ayanamsa/src/lookup.rs` | Route the six gated modes through corrected computation | Modify |
| `crates/pleiades-ayanamsa/src/lib.rs` | Declare new modules + re-export new public items | Modify |
| `crates/pleiades-ayanamsa/src/tests.rs` | Update the one epoch-value assertion that the correction changes | Modify |
| `crates/pleiades-validate/src/ayanamsa_validation.rs` | Parse corpus, checksum/manifest gate, numeric-residual gate | Create |
| `crates/pleiades-validate/src/lib.rs` | Declare module + re-export gate types | Modify |
| `crates/pleiades-validate/src/render/cli.rs` | `validate-ayanamsa` / `ayanamsa-gate` dispatch + help text | Modify |
| `crates/pleiades-validate/src/tests/validate_gates.rs` | CLI dispatch/alias/extra-arg/help tests for the new gate | Modify |
| `docs/superpowers/specs/2026-06-24-ayanamsa-numeric-gate-design.md` | (already committed) audit findings appended | Modify |
| `PLAN.md`, `plan/status/*.md` | Mark ayanamsa audit done; refresh status | Modify |

---

## Task 1: Swiss Ephemeris ayanamsa reference generator + committed corpus

**Files:**
- Create: `tools/se-ayanamsa-reference/Cargo.toml`
- Create: `tools/se-ayanamsa-reference/src/main.rs`
- Create: `tools/se-ayanamsa-reference/LICENSE-NOTES.md`
- Create: `crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv` (generated)
- Create: `crates/pleiades-validate/data/ayanamsa-corpus/manifest.txt` (generated)

**Interfaces:**
- Produces: a committed corpus where each data row is `mode_code,jd_tt,se_ayanamsa_deg`; `mode_code` ∈ {`Lahiri`,`Raman`,`Krishnamurti`,`FaganBradley`,`TrueChitra`,`TrueCitra`}. Manifest line: `slice ayanamsa file=ayanamsa.csv role=ayanamsa rows=<n> checksum=<u64>` with the `pleiades_apparent::fnv1a64` checksum of `ayanamsa.csv`.
- Produces (stdout, not committed): a Rust `const` block of cubic coefficients for the true-star modes, consumed verbatim by Task 4.

- [ ] **Step 1: Create the tool manifest (outside the workspace, own lockfile).**

`tools/se-ayanamsa-reference/Cargo.toml`:
```toml
[package]
name = "se-ayanamsa-reference"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
swisseph = "0.1.1"
libswisseph-sys = "0.1.2"
```

- [ ] **Step 2: Write the generator.**

`tools/se-ayanamsa-reference/src/main.rs`. SE sidereal-mode integer codes (from `sweph.h`): Fagan/Bradley=0, Lahiri=1, Raman=3, Krishnamurti=5, True Citra=27. "True Chitra" and "True Citra" are the same SE mode (TRUE_CITRA=27); both pleiades entries map to code 27 (the audit in Task 7 records this near-equivalence). `swe_set_sid_mode` is reached through the raw `-sys` crate; `swe_get_ayanamsa` (ET/TT) is wrapped by the high-level crate.

```rust
use libswisseph_sys::raw::swe_set_sid_mode;
use swisseph::swe::get_ayanamsa;

/// (pleiades mode_code, SE sidereal-mode integer)
const MODES: &[(&str, i32)] = &[
    ("FaganBradley",  0),
    ("Lahiri",        1),
    ("Raman",         3),
    ("Krishnamurti",  5),
    ("TrueChitra",   27),
    ("TrueCitra",    27),
];

/// Hold-out validation instants (jd_tt). Deliberately NOT on the dense fit grid
/// (Task: fit uses an even-year grid), so the gate is a genuine hold-out check.
const HOLDOUT_JD_TT: &[f64] = &[
    2_415_020.5,   // 1900-01-01
    2_420_000.5,
    2_424_152.0,   // ~1925
    2_429_000.5,
    2_433_282.5,   // ~1950
    2_438_000.5,
    2_451_545.0,   // J2000.0
    2_460_676.5,   // ~2025
    2_469_807.0,   // ~2050
    2_488_070.0,   // ~2100
];

fn ayanamsa(code: i32, jd_tt: f64) -> f64 {
    unsafe { swe_set_sid_mode(code, 0.0, 0.0); }
    let v = get_ayanamsa(jd_tt);
    assert!(v.is_finite(), "SE returned non-finite ayanamsa for code {code} at jd {jd_tt}");
    v
}

fn emit_corpus() {
    println!("mode_code,jd_tt,se_ayanamsa_deg");
    for &(name, code) in MODES {
        for &jd in HOLDOUT_JD_TT {
            println!("{name},{jd},{:.9}", ayanamsa(code, jd));
        }
    }
}

/// Solve a small symmetric normal-equation system by Gaussian elimination.
fn solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Vec<f64> {
    let n = b.len();
    for col in 0..n {
        let mut piv = col;
        for r in col + 1..n { if a[r][col].abs() > a[piv][col].abs() { piv = r; } }
        a.swap(col, piv); b.swap(col, piv);
        let d = a[col][col];
        for r in 0..n {
            if r == col { continue; }
            let f = a[r][col] / d;
            for c in col..n { a[r][c] -= f * a[col][c]; }
            b[r] -= f * b[col];
        }
    }
    (0..n).map(|i| b[i] / a[i][i]).collect()
}

/// Fit ayanamsa_deg(T) = c0 + c1 T + c2 T^2 + c3 T^3, T = (jd-2451545)/36525,
/// over a dense even-year grid, and print a Rust const block.
fn emit_fit() {
    let dense: Vec<f64> = (1900..=2100).step_by(2)
        .map(|y| 2_451_545.0 + (y as f64 - 2000.0) * 365.25)
        .collect();
    println!("// Generated by se-ayanamsa-reference fit. T = (jd_tt - 2451545.0)/36525.0");
    for &(name, code) in &[("TrueChitra", 27), ("TrueCitra", 27)] {
        // Normal equations for a degree-3 polynomial.
        let mut ata = vec![vec![0.0f64; 4]; 4];
        let mut atb = vec![0.0f64; 4];
        for &jd in &dense {
            let t = (jd - 2_451_545.0) / 36_525.0;
            let powers = [1.0, t, t * t, t * t * t];
            let y = ayanamsa(code, jd);
            for i in 0..4 {
                atb[i] += powers[i] * y;
                for j in 0..4 { ata[i][j] += powers[i] * powers[j]; }
            }
        }
        let c = solve(ata, atb);
        println!(
            "pub(crate) const {}_COEFFS: [f64; 4] = [{:.12e}, {:.12e}, {:.12e}, {:.12e}];",
            name.to_uppercase(), c[0], c[1], c[2], c[3]
        );
    }
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("fit") => emit_fit(),
        _ => emit_corpus(),
    }
}
```

- [ ] **Step 3: Build and run the generator; confirm SE returns finite values for the true-star mode.**

Run (in the devenv shell that provides `clang`/`libclang` per `devenv.nix`):
```bash
cd tools/se-ayanamsa-reference && cargo run --quiet | head -5
```
Expected: a CSV header line then `FaganBradley,2415020.5,<deg>` rows, all numeric.
If SE aborts or returns non-finite for code 27 (missing `sefstars.txt`), set the ephemeris path before the calls by adding, at the top of `main`, `unsafe { libswisseph_sys::raw::swe_set_ephe_path(std::ffi::CString::new("/path/to/ephe").unwrap().as_ptr()); }` and point it at an SE data dir containing `sefstars.txt`. Re-run until all six modes emit finite values.

- [ ] **Step 4: Generate the committed corpus CSV.**

```bash
cd /workspace
mkdir -p crates/pleiades-validate/data/ayanamsa-corpus
( cd tools/se-ayanamsa-reference && cargo run --quiet ) > crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv
wc -l crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv   # expect 61 (1 header + 60 data)
```

- [ ] **Step 5: Compute the checksum and write the manifest.**

Compute the fnv1a64 of the CSV using a throwaway test in the workspace (the tool cannot import workspace crates per C1). Add a temporary test to `crates/pleiades-validate/src/tests/mod.rs` (or run via `cargo test`):
```rust
#[test]
fn print_ayanamsa_corpus_checksum() {
    let csv = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/ayanamsa-corpus/ayanamsa.csv"));
    println!("AYANAMSA_CHECKSUM={} ROWS={}", pleiades_apparent::fnv1a64(csv), csv.lines().filter(|l| !l.starts_with("mode_code") && !l.trim().is_empty()).count());
    panic!("checksum print");
}
```
Run `cargo test -p pleiades-validate print_ayanamsa_corpus_checksum -- --nocapture`, read `AYANAMSA_CHECKSUM`/`ROWS`, then **delete the temporary test**.

Write `crates/pleiades-validate/data/ayanamsa-corpus/manifest.txt` (substitute the real version, checksum, rows):
```
#Pleiades Ayanamsa Reference Corpus Manifest
#Reference-Engine: SwissEphemeris <version>
#CrossCheck-Engine: not-run
slice ayanamsa file=ayanamsa.csv role=ayanamsa rows=60 checksum=<u64>
```
Get `<version>` from the `swisseph` crate's bundled SE (record the crate version if the C version is unavailable).

- [ ] **Step 6: Capture the true-star coefficients for Task 4.**

```bash
cd tools/se-ayanamsa-reference && cargo run --quiet fit
```
Save the printed `TRUECHITRA_COEFFS` / `TRUECITRA_COEFFS` lines — Task 4 pastes them verbatim.

- [ ] **Step 7: Write the license note.**

`tools/se-ayanamsa-reference/LICENSE-NOTES.md`: one paragraph stating this binary links Swiss Ephemeris (dual-licensed AGPL-3.0 / commercial) for verification only, is never distributed, and is excluded from the published workspace — mirroring `tools/se-house-reference/LICENSE-NOTES.md`.

- [ ] **Step 8: Verify the workspace stayed pure-Rust.**

Run: `cargo run -p pleiades-validate -- workspace-audit`
Expected: passes (no `-sys` package entered the workspace `Cargo.lock`; the tool has its own lockfile under `tools/`).

- [ ] **Step 9: Commit.**

```bash
git add tools/se-ayanamsa-reference crates/pleiades-validate/data/ayanamsa-corpus
git commit -m "feat(validate): SE ayanamsa reference corpus + generator (Phase 5)"
```

---

## Task 2: IAU-2006 general precession in longitude

**Files:**
- Create: `crates/pleiades-ayanamsa/src/precession.rs`
- Modify: `crates/pleiades-ayanamsa/src/lib.rs` (add `mod precession;`)
- Test: inline `#[cfg(test)]` in `precession.rs`

**Interfaces:**
- Produces: `pub(crate) fn general_precession_longitude_arcsec(t_centuries: f64) -> f64` — accumulated general precession in longitude `pA(T)` from J2000, arcseconds, `T` in Julian centuries TT.
- Produces: `pub(crate) fn precession_delta_degrees(jd_tt: f64, epoch_jd_tt: f64) -> f64` — `(pA(T_t) − pA(T_epoch)) / 3600`.

- [ ] **Step 1: Write the failing test.**

In `crates/pleiades-ayanamsa/src/precession.rs`:
```rust
//! IAU-2006 (P03) general precession in longitude, used as the drift term for
//! offset-defined ayanamsa modes.
#![forbid(unsafe_code)]

/// Accumulated general precession in longitude pA(T) from J2000.0, in arcseconds.
/// T is Julian centuries of TT from J2000.0. IAU 2006 (Capitaine et al. 2003).
pub(crate) fn general_precession_longitude_arcsec(t: f64) -> f64 {
    5028.796195 * t + 1.1054348 * t * t + 0.000_079_64 * t * t * t
}

/// Precession accumulated between two instants, expressed in degrees of longitude.
pub(crate) fn precession_delta_degrees(jd_tt: f64, epoch_jd_tt: f64) -> f64 {
    let t = (jd_tt - 2_451_545.0) / 36_525.0;
    let t0 = (epoch_jd_tt - 2_451_545.0) / 36_525.0;
    (general_precession_longitude_arcsec(t) - general_precession_longitude_arcsec(t0)) / 3600.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precession_rate_is_about_one_point_four_degrees_per_century() {
        // ~50.29"/yr ≈ 1.3969°/century near J2000.
        let one_century = precession_delta_degrees(2_451_545.0 + 36_525.0, 2_451_545.0);
        assert!((one_century - 1.396_9).abs() < 0.001, "got {one_century}");
    }

    #[test]
    fn precession_delta_is_zero_at_epoch() {
        assert_eq!(precession_delta_degrees(2_440_000.0, 2_440_000.0), 0.0);
    }

    #[test]
    fn precession_is_nonlinear_over_the_window() {
        // The quadratic term makes a +1 century delta differ from a -1 century delta.
        let fwd = precession_delta_degrees(2_451_545.0 + 36_525.0, 2_451_545.0);
        let bwd = precession_delta_degrees(2_451_545.0, 2_451_545.0 - 36_525.0);
        assert!((fwd - bwd).abs() > 1.0e-5, "expected nonlinearity, fwd={fwd} bwd={bwd}");
    }
}
```

- [ ] **Step 2: Add the module declaration.**

In `crates/pleiades-ayanamsa/src/lib.rs`, add alongside the other `mod` lines:
```rust
mod precession;
```

- [ ] **Step 3: Run the tests to verify they pass.**

Run: `cargo test -p pleiades-ayanamsa precession -- --nocapture`
Expected: 3 tests PASS.

- [ ] **Step 4: Commit.**

```bash
git add crates/pleiades-ayanamsa/src/precession.rs crates/pleiades-ayanamsa/src/lib.rs
git commit -m "feat(ayanamsa): IAU-2006 general precession in longitude"
```

---

## Task 3: Per-mode-class ceilings module

**Files:**
- Create: `crates/pleiades-ayanamsa/src/thresholds.rs`
- Modify: `crates/pleiades-ayanamsa/src/lib.rs` (`pub mod thresholds;`)
- Test: inline `#[cfg(test)]` in `thresholds.rs`

**Interfaces:**
- Produces: `pub enum AyanamsaModeClass { OffsetDefined, TrueStar }`
- Produces: `pub struct AyanamsaCeiling { pub offset_arcsec: f64 }`
- Produces: `pub fn ayanamsa_mode_ceiling(class: AyanamsaModeClass) -> AyanamsaCeiling`
- Produces: `pub fn ayanamsa_thresholds_summary_for_report() -> String`
- Produces: `pub fn ayanamsa_mode_class(ayanamsa: &pleiades_types::Ayanamsa) -> Option<AyanamsaModeClass>` — `Some` only for the six gated modes; `None` otherwise.

Ceilings use the house convention: set to `ceil(measured_max × 2)` with a 1.0″ floor. **Initial placeholder values below are replaced in Task 5 Step 6 with measured numbers; do not finalize this task's ceiling numbers until the gate has measured residuals.**

- [ ] **Step 1: Write the failing test.**

In `crates/pleiades-ayanamsa/src/thresholds.rs`:
```rust
//! Per-mode-class arcsecond ceilings for the ayanamsa numeric gate.
//! Mirrors `pleiades-houses/src/thresholds.rs`: ceilings are set to
//! `ceil(measured_max × 2)` over the committed SE corpus, with a 1.0″ floor.
#![forbid(unsafe_code)]

use pleiades_types::Ayanamsa;

/// Computation class of a gated ayanamsa mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AyanamsaModeClass {
    /// Fixed offset at a reference epoch plus general precession (Lahiri, Raman,
    /// Krishnamurti, Fagan/Bradley).
    OffsetDefined,
    /// Sidereal point pinned to a fixed star, fit to Swiss Ephemeris
    /// (True Chitra, True Citra).
    TrueStar,
}

/// Arcsecond ceiling for one mode class.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AyanamsaCeiling {
    /// Max allowed |residual| on the sidereal offset, arcseconds.
    pub offset_arcsec: f64,
}

/// Returns the measured-derived ceiling for a mode class.
pub fn ayanamsa_mode_ceiling(class: AyanamsaModeClass) -> AyanamsaCeiling {
    match class {
        // Replaced with ceil(measured_max × 2) in Task 5 Step 6.
        AyanamsaModeClass::OffsetDefined => AyanamsaCeiling { offset_arcsec: 5.0 },
        AyanamsaModeClass::TrueStar => AyanamsaCeiling { offset_arcsec: 5.0 },
    }
}

/// Maps a typed ayanamsa to its gated mode class, or `None` if it is not gated.
pub fn ayanamsa_mode_class(ayanamsa: &Ayanamsa) -> Option<AyanamsaModeClass> {
    match ayanamsa {
        Ayanamsa::Lahiri | Ayanamsa::Raman | Ayanamsa::Krishnamurti | Ayanamsa::FaganBradley => {
            Some(AyanamsaModeClass::OffsetDefined)
        }
        Ayanamsa::TrueChitra | Ayanamsa::TrueCitra => Some(AyanamsaModeClass::TrueStar),
        _ => None,
    }
}

/// Compact release-facing summary of the mode-class ceilings.
pub fn ayanamsa_thresholds_summary_for_report() -> String {
    let off = ayanamsa_mode_ceiling(AyanamsaModeClass::OffsetDefined);
    let star = ayanamsa_mode_ceiling(AyanamsaModeClass::TrueStar);
    format!(
        "Ayanamsa ceilings: offset-defined {:.1}\u{2033}, true-star {:.1}\u{2033}",
        off.offset_arcsec, star.offset_arcsec
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_class_has_finite_positive_ceiling() {
        for class in [AyanamsaModeClass::OffsetDefined, AyanamsaModeClass::TrueStar] {
            let c = ayanamsa_mode_ceiling(class);
            assert!(c.offset_arcsec.is_finite() && c.offset_arcsec > 0.0);
        }
    }

    #[test]
    fn only_the_six_release_modes_are_gated() {
        assert_eq!(ayanamsa_mode_class(&Ayanamsa::Lahiri), Some(AyanamsaModeClass::OffsetDefined));
        assert_eq!(ayanamsa_mode_class(&Ayanamsa::TrueChitra), Some(AyanamsaModeClass::TrueStar));
        assert_eq!(ayanamsa_mode_class(&Ayanamsa::J2000), None);
    }

    #[test]
    fn summary_line_mentions_both_classes() {
        let s = ayanamsa_thresholds_summary_for_report();
        assert!(s.contains("offset-defined") && s.contains("true-star"), "{s}");
    }
}
```

- [ ] **Step 2: Add the module declaration and re-exports.**

In `crates/pleiades-ayanamsa/src/lib.rs`:
```rust
pub mod thresholds;
```

- [ ] **Step 3: Run the tests.**

Run: `cargo test -p pleiades-ayanamsa thresholds`
Expected: 3 tests PASS.

- [ ] **Step 4: Commit.**

```bash
git add crates/pleiades-ayanamsa/src/thresholds.rs crates/pleiades-ayanamsa/src/lib.rs
git commit -m "feat(ayanamsa): per-mode-class arcsecond ceiling scaffold"
```

---

## Task 4: Corrected computation for the six gated modes

**Files:**
- Create: `crates/pleiades-ayanamsa/src/truestar.rs`
- Modify: `crates/pleiades-ayanamsa/src/lookup.rs` (route gated modes in `sidereal_offset`)
- Modify: `crates/pleiades-ayanamsa/src/lib.rs` (`mod truestar;`)
- Modify: `crates/pleiades-ayanamsa/src/tests.rs` (update one epoch-value assertion)

**Interfaces:**
- Consumes: `precession::precession_delta_degrees` (Task 2); `truestar::true_star_offset_degrees` (this task).
- Produces: corrected `pleiades_ayanamsa::sidereal_offset` for the six gated modes; all other modes and `Custom` keep the existing `offset_from_components` path unchanged.

- [ ] **Step 1: Write the true-star polynomial module (paste coefficients from Task 1 Step 6).**

`crates/pleiades-ayanamsa/src/truestar.rs`:
```rust
//! Committed cubic polynomials for the true-star ayanamsa modes (True Chitra /
//! True Citra), fit by `tools/se-ayanamsa-reference fit` to Swiss Ephemeris over
//! 1900–2100. ayanamsa_deg(T) = c0 + c1 T + c2 T^2 + c3 T^3, T = (jd-2451545)/36525.
#![forbid(unsafe_code)]

use pleiades_types::Ayanamsa;

// Generated by se-ayanamsa-reference fit — paste the two printed lines here.
pub(crate) const TRUECHITRA_COEFFS: [f64; 4] = [/* c0, c1, c2, c3 */];
pub(crate) const TRUECITRA_COEFFS: [f64; 4] = [/* c0, c1, c2, c3 */];

fn eval(coeffs: &[f64; 4], jd_tt: f64) -> f64 {
    let t = (jd_tt - 2_451_545.0) / 36_525.0;
    coeffs[0] + coeffs[1] * t + coeffs[2] * t * t + coeffs[3] * t * t * t
}

/// Returns the true-star sidereal offset in degrees, or `None` if the mode is
/// not a committed true-star mode.
pub(crate) fn true_star_offset_degrees(ayanamsa: &Ayanamsa, jd_tt: f64) -> Option<f64> {
    match ayanamsa {
        Ayanamsa::TrueChitra => Some(eval(&TRUECHITRA_COEFFS, jd_tt)),
        Ayanamsa::TrueCitra => Some(eval(&TRUECITRA_COEFFS, jd_tt)),
        _ => None,
    }
}
```
Replace the empty literals with the `TRUECHITRA_COEFFS` / `TRUECITRA_COEFFS` lines captured in Task 1 Step 6.

- [ ] **Step 2: Declare the module.**

In `crates/pleiades-ayanamsa/src/lib.rs`: `mod truestar;`

- [ ] **Step 3: Write the failing routing test.**

Add to `crates/pleiades-ayanamsa/src/tests.rs` (it already imports `sidereal_offset`, `Ayanamsa`, `Instant`, `JulianDay`, `TimeScale`, `Angle`):
```rust
#[test]
fn true_chitra_tracks_a_star_not_a_fixed_offset_from_lahiri() {
    // Before the correction, True Chitra == Lahiri at every instant (same epoch/offset).
    // After it, the two differ and True Chitra is non-linear vs a Lahiri-style linear offset.
    let early = Instant::new(JulianDay::from_days(2_415_020.5), TimeScale::Tt);
    let late = Instant::new(JulianDay::from_days(2_488_070.0), TimeScale::Tt);
    let tc_early = sidereal_offset(&Ayanamsa::TrueChitra, early).unwrap().degrees();
    let tc_late = sidereal_offset(&Ayanamsa::TrueChitra, late).unwrap().degrees();
    let lah_early = sidereal_offset(&Ayanamsa::Lahiri, early).unwrap().degrees();
    let lah_late = sidereal_offset(&Ayanamsa::Lahiri, late).unwrap().degrees();
    // Both increase with time (precession), staying in a sane sidereal range.
    assert!(tc_late > tc_early && tc_early > 22.0 && tc_late < 26.0, "tc {tc_early}..{tc_late}");
    // True Chitra and Lahiri are close but NOT identical (true-star vs offset model).
    assert!((tc_early - lah_early).abs() < 0.1 && (tc_late - lah_late).abs() < 0.1);
}

#[test]
fn lahiri_drift_is_nonlinear_after_correction() {
    // Equal time steps forward and backward from epoch give unequal deltas
    // (general precession is non-linear); the old constant-rate model gave equal.
    let epoch = 2_435_553.5;
    let step = 36_525.0;
    let at = |jd: f64| sidereal_offset(
        &Ayanamsa::Lahiri, Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
    ).unwrap().degrees();
    let fwd = at(epoch + step) - at(epoch);
    let bwd = at(epoch) - at(epoch - step);
    assert!((fwd - bwd).abs() > 1.0e-5, "fwd={fwd} bwd={bwd}");
}
```

- [ ] **Step 4: Run to verify failure.**

Run: `cargo test -p pleiades-ayanamsa true_chitra_tracks_a_star_not_a_fixed_offset_from_lahiri lahiri_drift_is_nonlinear_after_correction`
Expected: FAIL (current `sidereal_offset` is linear and True Chitra == Lahiri exactly).

- [ ] **Step 5: Route the six gated modes in `sidereal_offset`.**

In `crates/pleiades-ayanamsa/src/lookup.rs`, replace the body of `sidereal_offset` (currently the `match` at lines ~309–316) with:
```rust
pub fn sidereal_offset(ayanamsa: &Ayanamsa, instant: Instant) -> Option<Angle> {
    use crate::thresholds::{ayanamsa_mode_class, AyanamsaModeClass};

    if let Ayanamsa::Custom(custom) = ayanamsa {
        return offset_from_components(custom.epoch, custom.offset_degrees, instant);
    }

    let jd_tt = instant.julian_day.days();
    match ayanamsa_mode_class(ayanamsa) {
        // Offset-defined: documented epoch anchor + IAU-2006 precession drift.
        Some(AyanamsaModeClass::OffsetDefined) => {
            let entry = descriptor(ayanamsa)?;
            let epoch = entry.epoch?;
            let offset = entry.offset_degrees?;
            let drift = crate::precession::precession_delta_degrees(jd_tt, epoch.days());
            Some(Angle::from_degrees(offset.degrees() + drift))
        }
        // True-star: committed cubic fit to Swiss Ephemeris.
        Some(AyanamsaModeClass::TrueStar) => {
            crate::truestar::true_star_offset_degrees(ayanamsa, jd_tt).map(Angle::from_degrees)
        }
        // Not gated: unchanged legacy linear-rate path.
        None => descriptor(ayanamsa).and_then(|entry| entry.offset_at(instant)),
    }
}
```
Note: `descriptor`, `offset_from_components`, `Angle`, `Ayanamsa` are already in scope in `lookup.rs`.

- [ ] **Step 6: Update the one existing assertion the correction changes.**

In `crates/pleiades-ayanamsa/src/tests.rs`, the test that asserts `sidereal_offset(&Ayanamsa::Lahiri, <epoch>) == Angle::from_degrees(23.245_524_743)` (around line 795–797) now sees the SE-anchored value at epoch (≈ the documented offset, but not bit-exact). Replace that exact-equality assertion with a tolerance band; the gate provides the tight numeric check:
```rust
    let instant = Instant::new(JulianDay::from_days(2_435_553.5), TimeScale::Tt);
    let offset = sidereal_offset(&Ayanamsa::Lahiri, instant).expect("offset should exist");
    // At its reference epoch the corrected (precession-drift) value equals the
    // documented anchor to within the gate ceiling; tight residual is checked by
    // validate-ayanamsa against the SE corpus.
    assert!(
        (offset.degrees() - 23.245_524_743).abs() < 0.01,
        "Lahiri at epoch should be ~23.2455°, got {}",
        offset.degrees()
    );
```
The `descriptor.offset_at`-based test `reference_epoch_offsets_match_the_documented_baseline_values` is unaffected (it exercises the metadata path, not `sidereal_offset`) and must stay green.

- [ ] **Step 7: Run the full ayanamsa test suite.**

Run: `cargo test -p pleiades-ayanamsa`
Expected: all PASS (new routing tests, updated epoch test, and all existing catalog/metadata/custom tests).

- [ ] **Step 8: Commit.**

```bash
git add crates/pleiades-ayanamsa/src/truestar.rs crates/pleiades-ayanamsa/src/lookup.rs crates/pleiades-ayanamsa/src/lib.rs crates/pleiades-ayanamsa/src/tests.rs
git commit -m "fix(ayanamsa): SE-correct gated modes (precession drift + true-star fit)"
```

---

## Task 5: The `validate_ayanamsa_corpus` gate

**Files:**
- Create: `crates/pleiades-validate/src/ayanamsa_validation.rs`
- Modify: `crates/pleiades-validate/src/lib.rs` (`mod ayanamsa_validation;` + re-exports)
- Modify: `crates/pleiades-ayanamsa/src/thresholds.rs` (finalize ceilings, Step 6)

**Interfaces:**
- Consumes: the committed corpus (Task 1); `pleiades_ayanamsa::sidereal_offset` (Task 4); `pleiades_ayanamsa::thresholds::{ayanamsa_mode_class, ayanamsa_mode_ceiling}` (Task 3); `pleiades_apparent::fnv1a64`.
- Produces: `pub fn validate_ayanamsa_corpus() -> Result<AyanamsaCorpusReport, AyanamsaCorpusError>`; `pub struct AyanamsaCorpusReport { rows_validated, modes_checked, max_residual_arcsec, summary_line }`; `pub enum AyanamsaCorpusError { MalformedRow, MalformedManifest, ChecksumMismatch, ManifestDrift, UnknownModeCode, CalculationFailed, CeilingExceeded }`. `summary_line()` returns a string beginning with `"Ayanamsa gate"`.

- [ ] **Step 1: Write the module with parser, manifest, and gate (mirrors `house_validation.rs`).**

`crates/pleiades-validate/src/ayanamsa_validation.rs`:
```rust
//! Fail-closed numeric-residual gate for the six release-claimed ayanamsa modes
//! against the committed Swiss Ephemeris reference corpus. Mirrors
//! `house_validation.rs`: parse → checksum → manifest drift → per-row residual
//! vs the per-mode-class ceiling. Pure-Rust; no SE or network dependency.
use core::fmt;

use pleiades_ayanamsa::sidereal_offset;
use pleiades_ayanamsa::thresholds::{ayanamsa_mode_class, ayanamsa_mode_ceiling};
use pleiades_types::{Ayanamsa, Instant, JulianDay, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/ayanamsa-corpus/ayanamsa.csv"
));
const CORPUS_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/ayanamsa-corpus/manifest.txt"
));

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AyanamsaCorpusRow {
    pub(crate) mode_code: String,
    pub(crate) jd_tt: f64,
    pub(crate) se_ayanamsa_deg: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AyanamsaCorpusError {
    MalformedRow { row: usize, line: String, reason: String },
    MalformedManifest { reason: String },
    ChecksumMismatch { expected: u64, actual: u64 },
    ManifestDrift { field: String, expected: String, actual: String },
    UnknownModeCode { row: usize, code: String },
    CalculationFailed { row: usize, mode: String },
    CeilingExceeded {
        row: usize, mode: String, got: f64, want: f64,
        residual_arcsec: f64, ceiling_arcsec: f64,
    },
}

impl fmt::Display for AyanamsaCorpusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedRow { row, line, reason } =>
                write!(f, "ayanamsa corpus row {row} is malformed ({reason}): {line:?}"),
            Self::MalformedManifest { reason } =>
                write!(f, "ayanamsa corpus manifest is malformed: {reason}"),
            Self::ChecksumMismatch { expected, actual } =>
                write!(f, "ayanamsa corpus checksum mismatch: expected {expected}, got {actual}"),
            Self::ManifestDrift { field, expected, actual } =>
                write!(f, "ayanamsa corpus manifest drift on field '{field}': expected {expected:?}, got {actual:?}"),
            Self::UnknownModeCode { row, code } =>
                write!(f, "ayanamsa corpus row {row}: unknown mode code {code:?}"),
            Self::CalculationFailed { row, mode } =>
                write!(f, "ayanamsa corpus row {row} ({mode}): sidereal_offset returned None"),
            Self::CeilingExceeded { row, mode, got, want, residual_arcsec, ceiling_arcsec } =>
                write!(f, "ayanamsa corpus row {row} ({mode}): residual {residual_arcsec:.3}\u{2033} > ceiling {ceiling_arcsec:.1}\u{2033} (got={got:.9}\u{00b0}, want={want:.9}\u{00b0})"),
        }
    }
}
impl std::error::Error for AyanamsaCorpusError {}

#[derive(Clone, Debug, PartialEq)]
pub struct AyanamsaCorpusReport {
    pub rows_validated: usize,
    pub modes_checked: usize,
    pub max_residual_arcsec: f64,
    pub summary_line: String,
}
impl AyanamsaCorpusReport {
    pub fn summary_line(&self) -> &str { &self.summary_line }
}
impl fmt::Display for AyanamsaCorpusReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.summary_line) }
}

pub(crate) fn parse_corpus(csv: &str) -> Result<Vec<AyanamsaCorpusRow>, AyanamsaCorpusError> {
    let mut rows = Vec::new();
    let mut data_row = 0usize;
    for line in csv.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || t.starts_with("mode_code,") { continue; }
        data_row += 1;
        let parts: Vec<&str> = t.split(',').collect();
        if parts.len() != 3 {
            return Err(AyanamsaCorpusError::MalformedRow {
                row: data_row, line: line.to_string(),
                reason: format!("expected 3 fields, got {}", parts.len()),
            });
        }
        let jd_tt = parts[1].trim().parse().map_err(|_| AyanamsaCorpusError::MalformedRow {
            row: data_row, line: line.to_string(),
            reason: format!("jd_tt {:?} is not a valid float", parts[1]),
        })?;
        let se_ayanamsa_deg = parts[2].trim().parse().map_err(|_| AyanamsaCorpusError::MalformedRow {
            row: data_row, line: line.to_string(),
            reason: format!("se_ayanamsa_deg {:?} is not a valid float", parts[2]),
        })?;
        rows.push(AyanamsaCorpusRow { mode_code: parts[0].trim().to_string(), jd_tt, se_ayanamsa_deg });
    }
    Ok(rows)
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AyanamsaManifest { pub(crate) rows: usize, pub(crate) checksum: u64 }

pub(crate) fn parse_manifest(text: &str) -> Result<AyanamsaManifest, AyanamsaCorpusError> {
    let mut rows = None;
    let mut checksum = None;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("slice ") {
            for tok in t.split_whitespace() {
                if let Some(v) = tok.strip_prefix("rows=") {
                    rows = Some(v.parse().map_err(|_| AyanamsaCorpusError::MalformedManifest {
                        reason: format!("rows value {v:?} is not a valid usize"),
                    })?);
                } else if let Some(v) = tok.strip_prefix("checksum=") {
                    checksum = Some(v.parse().map_err(|_| AyanamsaCorpusError::MalformedManifest {
                        reason: format!("checksum value {v:?} is not a valid u64"),
                    })?);
                }
            }
        }
    }
    Ok(AyanamsaManifest {
        rows: rows.ok_or(AyanamsaCorpusError::MalformedManifest { reason: "rows= not found".into() })?,
        checksum: checksum.ok_or(AyanamsaCorpusError::MalformedManifest { reason: "checksum= not found".into() })?,
    })
}

fn mode_for_code(code: &str) -> Option<Ayanamsa> {
    match code {
        "Lahiri" => Some(Ayanamsa::Lahiri),
        "Raman" => Some(Ayanamsa::Raman),
        "Krishnamurti" => Some(Ayanamsa::Krishnamurti),
        "FaganBradley" => Some(Ayanamsa::FaganBradley),
        "TrueChitra" => Some(Ayanamsa::TrueChitra),
        "TrueCitra" => Some(Ayanamsa::TrueCitra),
        _ => None,
    }
}

fn wrap_arcsec(got: f64, want: f64) -> f64 {
    let mut d = (got - want).abs();
    if d > 180.0 { d = 360.0 - d; }
    d * 3600.0
}

/// Runs the fail-closed ayanamsa numeric-residual gate over the committed corpus.
pub fn validate_ayanamsa_corpus() -> Result<AyanamsaCorpusReport, AyanamsaCorpusError> {
    let actual = pleiades_apparent::fnv1a64(CORPUS_CSV);
    let manifest = parse_manifest(CORPUS_MANIFEST)?;
    if actual != manifest.checksum {
        return Err(AyanamsaCorpusError::ChecksumMismatch { expected: manifest.checksum, actual });
    }
    let rows = parse_corpus(CORPUS_CSV)?;
    if rows.len() != manifest.rows {
        return Err(AyanamsaCorpusError::ManifestDrift {
            field: "rows".into(), expected: manifest.rows.to_string(), actual: rows.len().to_string(),
        });
    }
    // Completeness: all six gated modes present.
    for code in ["Lahiri", "Raman", "Krishnamurti", "FaganBradley", "TrueChitra", "TrueCitra"] {
        if !rows.iter().any(|r| r.mode_code == code) {
            return Err(AyanamsaCorpusError::ManifestDrift {
                field: "completeness".into(), expected: format!("rows for {code}"), actual: "missing".into(),
            });
        }
    }
    let mut max_residual_arcsec = 0.0_f64;
    let mut modes = std::collections::BTreeSet::new();
    for (idx, row) in rows.iter().enumerate() {
        let data_row = idx + 1;
        let mode = mode_for_code(&row.mode_code).ok_or_else(|| AyanamsaCorpusError::UnknownModeCode {
            row: data_row, code: row.mode_code.clone(),
        })?;
        modes.insert(row.mode_code.clone());
        let class = ayanamsa_mode_class(&mode).ok_or_else(|| AyanamsaCorpusError::UnknownModeCode {
            row: data_row, code: row.mode_code.clone(),
        })?;
        let ceiling = ayanamsa_mode_ceiling(class);
        let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);
        let got = sidereal_offset(&mode, instant)
            .ok_or_else(|| AyanamsaCorpusError::CalculationFailed { row: data_row, mode: row.mode_code.clone() })?
            .degrees();
        let resid = wrap_arcsec(got, row.se_ayanamsa_deg);
        if resid > max_residual_arcsec { max_residual_arcsec = resid; }
        if resid > ceiling.offset_arcsec {
            return Err(AyanamsaCorpusError::CeilingExceeded {
                row: data_row, mode: row.mode_code.clone(), got, want: row.se_ayanamsa_deg,
                residual_arcsec: resid, ceiling_arcsec: ceiling.offset_arcsec,
            });
        }
    }
    let summary_line = format!(
        "Ayanamsa gate: {} rows, {} modes validated vs Swiss Ephemeris, max residual {:.3}\u{2033}",
        rows.len(), modes.len(), max_residual_arcsec
    );
    Ok(AyanamsaCorpusReport {
        rows_validated: rows.len(), modes_checked: modes.len(), max_residual_arcsec, summary_line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_passes_over_committed_corpus() {
        let report = validate_ayanamsa_corpus().expect("ayanamsa gate should pass");
        assert_eq!(report.modes_checked, 6);
        assert!(report.summary_line().starts_with("Ayanamsa gate"));
    }

    #[test]
    fn checksum_drift_fails_closed() {
        // The committed checksum must equal the recomputed CSV checksum.
        let manifest = parse_manifest(CORPUS_MANIFEST).unwrap();
        assert_eq!(manifest.checksum, pleiades_apparent::fnv1a64(CORPUS_CSV));
    }

    #[test]
    fn malformed_row_fails_closed() {
        let err = parse_corpus("mode_code,jd_tt,se_ayanamsa_deg\nLahiri,not_a_float,23.0\n").unwrap_err();
        assert!(matches!(err, AyanamsaCorpusError::MalformedRow { .. }));
    }

    #[test]
    fn unknown_mode_code_fails_closed() {
        assert!(mode_for_code("Bogus").is_none());
    }
}
```

- [ ] **Step 2: Declare the module and re-export the public types.**

In `crates/pleiades-validate/src/lib.rs`, add `mod ayanamsa_validation;` alongside the other module decls, and a re-export mirroring the house line (`pub use house_validation::{...}`):
```rust
pub use ayanamsa_validation::{validate_ayanamsa_corpus, AyanamsaCorpusError, AyanamsaCorpusReport};
```

- [ ] **Step 3: Run the gate tests (expect a possible ceiling failure first).**

Run: `cargo test -p pleiades-validate ayanamsa`
Expected: parse/checksum/unknown-mode tests PASS. `gate_passes_over_committed_corpus` PASSES if the placeholder 5.0″ ceiling already covers the residuals; if it FAILS with `CeilingExceeded`, read the printed `residual` — that is the measured max for that mode class. Proceed to Step 4.

- [ ] **Step 4: Measure per-mode-class residuals.**

Temporarily relax both ceilings in `thresholds.rs` to a large value (e.g. `3600.0`) and add a measurement test in `ayanamsa_validation.rs`:
```rust
    #[test]
    fn measure_residuals() {
        let rows = parse_corpus(CORPUS_CSV).unwrap();
        let (mut max_off, mut max_star) = (0.0_f64, 0.0_f64);
        for r in &rows {
            let m = mode_for_code(&r.mode_code).unwrap();
            let got = sidereal_offset(&m, Instant::new(JulianDay::from_days(r.jd_tt), TimeScale::Tt)).unwrap().degrees();
            let resid = wrap_arcsec(got, r.se_ayanamsa_deg);
            match ayanamsa_mode_class(&m).unwrap() {
                pleiades_ayanamsa::thresholds::AyanamsaModeClass::OffsetDefined => max_off = max_off.max(resid),
                pleiades_ayanamsa::thresholds::AyanamsaModeClass::TrueStar => max_star = max_star.max(resid),
            }
        }
        println!("MAX_OFFSET_ARCSEC={max_off:.4} MAX_TRUESTAR_ARCSEC={max_star:.4}");
        panic!("measurement");
    }
```
Run: `cargo test -p pleiades-validate measure_residuals -- --nocapture`. Record `MAX_OFFSET_ARCSEC` and `MAX_TRUESTAR_ARCSEC`, then **delete this test** and restore the ceilings.

- [ ] **Step 5: Investigate if a class residual is large (> ~30″).**

If `MAX_OFFSET_ARCSEC` is large, SE's offset modes use a precession model incompatible with IAU-2006 over the window — either accept the measured value as the ceiling, or switch offset modes to the same committed-cubic-fit approach as true-star modes (extend `truestar.rs`/the generator `fit` subcommand to all six modes and route offset modes through the fit). If `MAX_TRUESTAR_ARCSEC` is large, the cubic fit degree is too low — bump the fit to degree 4 in the generator and re-run Task 1 Step 6 + Task 4 Step 1. Re-measure until both maxima are small (target: offset-defined ≤ a few arcsec; true-star ≤ ~1″).

- [ ] **Step 6: Finalize the ceilings from measurement.**

In `crates/pleiades-ayanamsa/src/thresholds.rs`, set each ceiling to `ceil(measured_max × 2)` with a 1.0″ floor, and document the measured maxima in the doc comment (house-thresholds style). Example (substitute real numbers):
```rust
        // OffsetDefined: measured max <X>″ over the corpus → ceil(<2X>) = <n>″.
        AyanamsaModeClass::OffsetDefined => AyanamsaCeiling { offset_arcsec: <n> },
        // TrueStar: measured max <Y>″ → ceil(<2Y>) = <m>″ (1.0″ floor).
        AyanamsaModeClass::TrueStar => AyanamsaCeiling { offset_arcsec: <m> },
```

- [ ] **Step 7: Run the gate; confirm it passes with the finalized ceilings.**

Run: `cargo test -p pleiades-validate ayanamsa && cargo test -p pleiades-ayanamsa thresholds`
Expected: all PASS, including `gate_passes_over_committed_corpus`.

- [ ] **Step 8: Commit.**

```bash
git add crates/pleiades-validate/src/ayanamsa_validation.rs crates/pleiades-validate/src/lib.rs crates/pleiades-ayanamsa/src/thresholds.rs
git commit -m "feat(validate): validate_ayanamsa_corpus numeric-residual gate + measured ceilings"
```

---

## Task 6: CLI wiring (`validate-ayanamsa` / `ayanamsa-gate`)

**Files:**
- Modify: `crates/pleiades-validate/src/render/cli.rs` (dispatch arm + help text)
- Modify: `crates/pleiades-validate/src/tests/validate_gates.rs` (dispatch/alias/extra-arg/help tests)

**Interfaces:**
- Consumes: `crate::validate_ayanamsa_corpus` (Task 5).
- Produces: CLI subcommand `validate-ayanamsa` with alias `ayanamsa-gate`.

- [ ] **Step 1: Write the failing CLI tests.**

Append to `crates/pleiades-validate/src/tests/validate_gates.rs`:
```rust
#[test]
fn validate_ayanamsa_command_dispatches_and_reports_pass() {
    let result = render_cli(&["validate-ayanamsa"])
        .expect("validate-ayanamsa should succeed on committed ayanamsa corpus");
    assert!(result.contains("Ayanamsa gate"),
        "validate-ayanamsa output should contain 'Ayanamsa gate': {result}");
}

#[test]
fn ayanamsa_gate_alias_matches_validate_ayanamsa() {
    let via_primary = render_cli(&["validate-ayanamsa"]).expect("validate-ayanamsa should succeed");
    let via_alias = render_cli(&["ayanamsa-gate"]).expect("ayanamsa-gate alias should succeed");
    assert_eq!(via_primary, via_alias);
}

#[test]
fn validate_ayanamsa_rejects_extra_args() {
    let error = render_cli(&["validate-ayanamsa", "extra"])
        .expect_err("validate-ayanamsa should reject extra arguments");
    assert!(error.contains("validate-ayanamsa does not accept extra arguments"),
        "unexpected error: {error}");
}

#[test]
fn help_text_mentions_validate_ayanamsa() {
    let help = render_cli(&["help"]).expect("help command should render");
    assert!(help.contains("validate-ayanamsa"), "help should mention validate-ayanamsa");
    assert!(help.contains("ayanamsa-gate"), "help should mention ayanamsa-gate alias");
}
```

- [ ] **Step 2: Run to verify failure.**

Run: `cargo test -p pleiades-validate validate_ayanamsa ayanamsa_gate help_text_mentions_validate_ayanamsa`
Expected: FAIL (`validate-ayanamsa` is an unknown command).

- [ ] **Step 3: Add the dispatch arm.**

In `crates/pleiades-validate/src/render/cli.rs`, immediately after the `validate-houses` / `houses-gate` arm (around line 158–163), add:
```rust
        Some("validate-ayanamsa") | Some("ayanamsa-gate") => {
            ensure_no_extra_args(&args[1..], "validate-ayanamsa")?;
            crate::validate_ayanamsa_corpus()
                .map(|report| report.summary_line().to_string())
                .map_err(|e| e.to_string())
        }
```

- [ ] **Step 4: Add the help text.**

In the help banner string in `cli.rs` (the long `Commands:` literal, around line 2001), add two lines after the `houses-gate` entry:
```
  validate-ayanamsa         Run the fail-closed ayanamsa gate (Swiss Ephemeris reference offsets, per-mode-class arcsecond ceilings) over the committed ayanamsa corpus\n  ayanamsa-gate             Alias for validate-ayanamsa\n
```
(Match the exact `\n`-joined inline format of the surrounding entries.)

- [ ] **Step 5: Run the tests.**

Run: `cargo test -p pleiades-validate validate_ayanamsa ayanamsa_gate help_text_mentions_validate_ayanamsa`
Expected: all PASS.

- [ ] **Step 6: Run any CLI help snapshot tests that enumerate commands.**

Run: `cargo test -p pleiades-cli help && cargo test -p pleiades-validate`
Expected: PASS. If a `pleiades-cli` help snapshot lists validate subcommands and now mismatches, update the snapshot to include `validate-ayanamsa` / `ayanamsa-gate`.

- [ ] **Step 7: Commit.**

```bash
git add crates/pleiades-validate/src/render/cli.rs crates/pleiades-validate/src/tests/validate_gates.rs
git commit -m "feat(validate): validate-ayanamsa CLI gate (wired like validate-houses)"
```

---

## Task 7: Audit record + plan/status alignment

**Files:**
- Modify: `docs/superpowers/specs/2026-06-24-ayanamsa-numeric-gate-design.md` (append audit findings)
- Modify: `PLAN.md` (Phase 5 status)
- Modify: `plan/status/01-current-execution-frontier.md`, `plan/status/02-next-slice-candidates.md`

**Interfaces:** none (documentation).

- [ ] **Step 1: Append the audit record to the design doc.**

Add an "## Audit findings (<date>)" section recording, per gated mode: numerically-gated (yes), measured max residual (arcsec), the mode-class ceiling, the computation used (offset-defined = epoch anchor + IAU-2006 precession; true-star = committed cubic fit to SE), and the SE `(t0, ayan_t0)` reconciliation. State explicitly that True Chitra and True Citra both map to SE TRUE_CITRA (code 27) and are near-equivalents. List that the remaining ~48 metadata-carrying built-ins stay descriptor-tested and are **not yet numerically gated** (no claim broadening).

- [ ] **Step 2: Update `PLAN.md`.**

In the Phase 5 row / "Current priority" / status line, record that the ayanamsa audit + `validate-ayanamsa` numeric gate is **done** (house gate + ayanamsa gate now both land the Phase 5 compatibility-audit pair), keeping the wording style of the existing house entry. Do not broaden any public claim beyond the six gated modes.

- [ ] **Step 3: Update the status files.**

In `plan/status/02-next-slice-candidates.md` Phase 5 section, mark the ayanamsa epoch/offset/formula/alias/provenance audit slice as completed via the numeric gate; note remaining Phase 5 work (release-gate hardening / compatibility-profile overclaim checks) as the next candidates.

- [ ] **Step 4: Full workspace verification.**

Run:
```bash
cargo test --workspace
cargo run -p pleiades-validate -- validate-ayanamsa
cargo run -p pleiades-validate -- workspace-audit
cargo fmt --all -- --check && cargo clippy --workspace --all-targets
```
Expected: all green; `validate-ayanamsa` prints the `Ayanamsa gate: …` summary; `workspace-audit` confirms no `-sys` leak.

- [ ] **Step 5: Commit.**

```bash
git add docs/superpowers/specs/2026-06-24-ayanamsa-numeric-gate-design.md PLAN.md plan/status
git commit -m "docs: record ayanamsa audit findings; mark Phase 5 ayanamsa gate done"
```

---

## Self-Review

**Spec coverage:**
- SE reference generator outside the workspace → Task 1 (C1 honored; `workspace-audit` checked in Steps 8 & Task 7).
- Committed corpus CSV + manifest + fnv1a64 checksum → Task 1.
- Per-mode-class ceilings module → Task 3 (finalized from measurement in Task 5).
- Evidence-first correction (measure → correct) → Task 4 (offset-defined precession drift; true-star fit) + Task 5 Steps 4–6 (measure, investigate, finalize).
- True-star modes implemented for real (not constrained) → Task 4 (`truestar.rs`) + generator `fit` (Task 1).
- `validate_ayanamsa_corpus` fail-closed gate → Task 5.
- CLI `validate-ayanamsa` → Task 6.
- Audit record + not-yet-gated disclosure + no claim broadening → Task 7.
- Open items from the spec: SE_SIDM code for True Chitra/Citra (Task 1 Step 2 + Task 7 Step 1), `(t0, ayan_t0)` reconciliation (Task 7 Step 1), time scale UT vs TT (resolved: TT via `get_ayanamsa`, Global Constraints + Task 1 Step 2), concrete ceilings (Task 5 Step 6).

**Placeholder scan:** The only intentionally-deferred values are (a) the corpus CSV/manifest numbers and (b) the polynomial coefficients and ceilings — all **generated data**, produced by concrete, complete code in Tasks 1/5 and pasted at named steps (mirrors how the house corpus CSV was produced). No prose-only "add error handling" steps; every code step shows code.

**Type consistency:** `sidereal_offset(&Ayanamsa, Instant) -> Option<Angle>` used consistently (Task 4 defines, Task 5 consumes). `ayanamsa_mode_class`/`ayanamsa_mode_ceiling`/`AyanamsaModeClass`/`AyanamsaCeiling` consistent across Tasks 3/5. Gate types `validate_ayanamsa_corpus`/`AyanamsaCorpusReport`/`AyanamsaCorpusError` consistent across Tasks 5/6. Summary string prefix `"Ayanamsa gate"` consistent across Task 5 (definition) and Task 6 (test assertion). Corpus schema `mode_code,jd_tt,se_ayanamsa_deg` consistent across Tasks 1 and 5.
