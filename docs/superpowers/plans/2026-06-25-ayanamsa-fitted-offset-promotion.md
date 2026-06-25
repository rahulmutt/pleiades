# Ayanamsa Fitted-Offset Promotion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote the 12 "failed-offset" ayanamsa modes — smooth SE-coded modes that passed the descriptor stage but missed the *linear* anchor+precession 3.0″ ceiling in slice 1 — from `DescriptorOnly` to `ReleaseGradeNumeric` via committed cubic polynomials fit to Swiss Ephemeris **mean** ayanamsa, gated by the existing fail-closed numeric gate.

**Architecture:** Reuse the proven slice-2 cubic-fit path verbatim. The SE reference tool emits per-mode cubic coefficients and a holdout corpus from SE 2.10.03 mean ayanamsa; coefficients are committed into a new `fitted_offset.rs` module (sibling of `truestar.rs`/`galactic.rs`); `sidereal_offset` routes a new `FittedOffset` mode class to its evaluator; the gate validates every committed corpus row against a per-family arcsecond ceiling. Promotion is measured, not asserted: a mode passes only if every holdout residual is within its family ceiling, otherwise it stays `DescriptorOnly` with its measured residual recorded.

**Tech Stack:** Rust (workspace crates `pleiades-ayanamsa`, `pleiades-validate`, `pleiades-apparent`, `pleiades-types`); standalone `tools/se-ayanamsa-reference` binary linking Swiss Ephemeris (`libswisseph-sys` 0.1.2, vendored SE 2.10.03); FNV-1a-64 corpus checksums.

## Global Constraints

- **Reference engine:** Swiss Ephemeris **2.10.03** mean ayanamsa, `iflag = SEFLG_NONUT | SEFLG_NOABERR` (= 1088, the existing `MEAN_IFLAG`). This exact version is pinned in the corpus manifest; do not change it.
- **FNV prime is non-canonical:** corpus checksums use `pleiades_apparent::fnv1a64` (a typo'd FNV prime). Recompute checksums only with that in-repo function — stock FNV will not match.
- **Fail-closed:** every gate failure (checksum mismatch, manifest drift on `rows=`/completeness, malformed row, unknown mode code, `None` calculation, ceiling exceeded) aborts the gate. No silent skips.
- **Fit/eval window:** 1900–2100, JD TT `[2_415_020.0, 2_488_070.0]` (the existing `TRUE_STAR_FIT_JD_MIN/MAX`). Outside the window the evaluator returns `None` (fail-closed), never an extrapolated value.
- **Ceiling policy:** per mode class, `ceil(measured_max_arcsec × 2)` with a **1.0″ floor**, computed over the modes that actually pass into the new class.
- **Holdout grid:** the existing 10 `HOLDOUT_JD_TT` instants. Do not change the grid.
- **Membership is empirical:** any candidate whose cubic cannot meet its ceiling is deferred with its measured residual recorded and stays `DescriptorOnly`. No overclaim.
- **SE tool is excluded from the workspace** (`Cargo.toml` `exclude`); build/run it with `--manifest-path tools/se-ayanamsa-reference/Cargo.toml`.
- **Classification = Approach A:** the promoted modes join a single new `FittedOffset` mode class. They do **not** join `OffsetDefined` (whose 3.0″ ceiling is the *linear* model's budget) and they do **not** disturb the existing `OffsetDefined`/`TrueStar`/`Galactic` modes.
- **Sub-family contingency:** if a subset (most likely the Babylonians) clusters at a distinctly higher-but-bounded residual, split it into its own class+ceiling rather than inflating `FittedOffset`. Default is one class. A group that cannot be bounded at all defers.

### Mode inventory (target membership — verified against libswisseph-sys 0.1.2 SE_SIDM table)

**In scope → new `FittedOffset` class (12 candidates, all have a distinct SE_SIDM code):**

| Mode (enum variant / corpus name) | SE_SIDM |
| --- | --- |
| `DeLuce` | 2 |
| `BabylonianKugler1` | 9 |
| `BabylonianKugler2` | 10 |
| `BabylonianKugler3` | 11 |
| `BabylonianHuber` | 12 |
| `BabylonianEtaPiscium` | 13 |
| `BabylonianAldebaran` | 14 |
| `Hipparchus` | 15 |
| `BabylonianBritton` | 38 |
| `ValensMoon` | 42 |
| `LahiriVP285` | 44 |
| `KrishnamurtiVP291` | 45 |

**Expected-deferred — no distinct SE_SIDM code (confirmed: the SE 2.10.03 table has no entry for these; keep `DescriptorOnly`):** `Udayagiri`, `PvrPushyaPaksha`, `Sheoran`. Task 1 Step 5 re-confirms by their absence from SE; record them as deferred-by-lack-of-code.

**Out of scope — stay deferred (genuine known gaps):** the 6 observational/topocentric/house Babylonians (`BabylonianTrueGeoc`, `BabylonianTrueTopc`, `BabylonianTrueObs`, `BabylonianHouse`, `BabylonianHouseObs`, `BabylonianSissy` — not smooth functions of time) and the 2 no-SE-code galactic modes (`DhruvaGalacticCenterMula`, `GalacticEquator`).

> **Empirical values produced at runtime:** the cubic coefficients (Task 3), the measured worst residual per mode and final PASS/DEFER membership (Task 1), the computed `FittedOffset` ceiling (Task 2), the regenerated corpus rows + checksum (Task 4), and the final gated mode count (Tasks 4–6) are all **outputs of running the SE tool and the gate**. Each step gives the exact command that produces the value and where to paste it. These are tool outputs the engineer captures, not author placeholders.

---

## File Structure

- `tools/se-ayanamsa-reference/src/main.rs` — extend `MODES` (corpus emit) and `emit_fit`'s `FIT_MODES` with the 12 candidates; add a `measure-fitted-offset` subcommand. Responsibility: produce SE reference rows, cubic coefficients, and per-mode residuals.
- `crates/pleiades-ayanamsa/src/thresholds.rs` — add `FittedOffset` mode class, its ceiling, and class mapping for the promoted modes.
- `crates/pleiades-ayanamsa/src/fitted_offset.rs` — **new**. The 12 cubic coefficient constants + `fitted_offset_degrees(&Ayanamsa, jd_tt) -> Option<f64>` with the shared window guard.
- `crates/pleiades-ayanamsa/src/lib.rs` — declare `mod fitted_offset;`.
- `crates/pleiades-ayanamsa/src/lookup.rs` — route the `FittedOffset` class in `sidereal_offset`.
- `crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv` + `manifest.txt` — regenerated corpus rows; bumped `rows=` and `checksum=`.
- `crates/pleiades-validate/src/ayanamsa_validation.rs` — extend `mode_for_code`, the completeness list, and the `modes_checked` test count.
- `crates/pleiades-ayanamsa/src/catalog.rs` — flip promoted modes from `AyanamsaDescriptor::new(` to `new_release_grade(` in **both** the `BASELINE_AYANAMSAS`/`RELEASE_AYANAMSAS` source arrays and the `BUILT_IN_AYANAMSAS` flat array.
- Test/prose surfaces updated for the new count: `crates/pleiades-ayanamsa/src/catalog/tests.rs` (the `deferred_modes_stay_descriptor_only` and `release_grade_numeric_ayanamsa_set_is_exactly_the_gated_modes` tests), `crates/pleiades-validate/src/tests/*`, `crates/pleiades-cli/src/cli/tests/summary_commands.rs`, `README.md`, `PLAN.md`.

---

## Task 1: Extend the SE tool to emit, fit, and measure the failed-offset family

**Files:**
- Modify: `tools/se-ayanamsa-reference/src/main.rs` (`MODES` ~16-58; `emit_fit`'s `FIT_MODES` ~124-132; add `emit_measure_fitted_offset` near `emit_measure_fitted` ~223; wire into `main` ~289-294)

**Interfaces:**
- Consumes: existing `ayanamsa(code, jd_tt)`, `solve`, `eval_cubic`, `HOLDOUT_JD_TT`, `MEAN_IFLAG`.
- Produces: `corpus` emits reference rows for the 12 candidates; `fit` prints a `<NAME>_COEFFS` const block per candidate; `measure-fitted-offset` prints `<name> worst=<arcsec> verdict=<PASS|DEFER>` per candidate.

- [ ] **Step 1: Add the 12 candidates to `MODES`**

In `tools/se-ayanamsa-reference/src/main.rs`, replace the trailing deferred-comment block of `MODES` (the three `// Still deferred …` comment lines after `("GalacticEquatorFiorenza", 41),`) with the candidate entries plus an updated deferred comment, keeping all existing entries above untouched:

```rust
    // fitted-offset family — slice 3: failed-offset modes re-fit with cubics.
    ("DeLuce",                2),
    ("BabylonianKugler1",     9),
    ("BabylonianKugler2",    10),
    ("BabylonianKugler3",    11),
    ("BabylonianHuber",      12),
    ("BabylonianEtaPiscium", 13),
    ("BabylonianAldebaran",  14),
    ("Hipparchus",           15),
    ("BabylonianBritton",    38),
    ("ValensMoon",           42),
    ("LahiriVP285",          44),
    ("KrishnamurtiVP291",    45),
    // Still deferred: anchorless modes (Udayagiri, PvrPushyaPaksha, Sheoran —
    // no distinct SE_SIDM code), observational/topocentric/house Babylonians
    // (TrueGeoc/TrueTopc/TrueObs/House/HouseObs/Sissy — not smooth in time), and
    // DhruvaGalacticCenterMula / legacy GalacticEquator (no distinct SE code).
];
```

- [ ] **Step 2: Add the 12 candidates to `emit_fit`'s `FIT_MODES`**

In `emit_fit`, append the 12 candidate `(name, code)` pairs to the existing `FIT_MODES` slice (after the galactic entries), so one `fit` run prints every committed cubic, including the new ones. Leave the loop body unchanged:

```rust
        ("GalacticEquatorFiorenza", 41),
        // slice 3 — fitted-offset family:
        ("DeLuce", 2), ("BabylonianKugler1", 9), ("BabylonianKugler2", 10),
        ("BabylonianKugler3", 11), ("BabylonianHuber", 12), ("BabylonianEtaPiscium", 13),
        ("BabylonianAldebaran", 14), ("Hipparchus", 15), ("BabylonianBritton", 38),
        ("ValensMoon", 42), ("LahiriVP285", 44), ("KrishnamurtiVP291", 45),
    ];
```

- [ ] **Step 3: Add a `measure-fitted-offset` subcommand**

Add this function next to `emit_measure_fitted`, and wire it into `main`'s match (`Some("measure-fitted-offset") => emit_measure_fitted_offset(),`):

```rust
/// Fit each slice-3 failed-offset candidate over the dense even-year grid, then
/// report the worst holdout residual (arcsec) and a PASS/DEFER verdict at the
/// 1.0" floor (the FittedOffset ceiling is recomputed in the crate from the
/// measured maxima).
fn emit_measure_fitted_offset() {
    let dense: Vec<f64> = (1900..=2100)
        .step_by(2)
        .map(|y| 2_451_545.0 + (y as f64 - 2000.0) * 365.25)
        .collect();
    const FIT_MODES: &[(&str, i32)] = &[
        ("DeLuce", 2), ("BabylonianKugler1", 9), ("BabylonianKugler2", 10),
        ("BabylonianKugler3", 11), ("BabylonianHuber", 12), ("BabylonianEtaPiscium", 13),
        ("BabylonianAldebaran", 14), ("Hipparchus", 15), ("BabylonianBritton", 38),
        ("ValensMoon", 42), ("LahiriVP285", 44), ("KrishnamurtiVP291", 45),
    ];
    for &(name, code) in FIT_MODES {
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
        let c4 = solve(ata, atb);
        let c = [c4[0], c4[1], c4[2], c4[3]];
        let mut worst = 0.0f64;
        for &jd in HOLDOUT_JD_TT {
            let resid = (ayanamsa(code, jd) - eval_cubic(&c, jd)).abs() * 3600.0;
            if resid > worst { worst = resid; }
        }
        let verdict = if worst <= 1.0 { "PASS" } else { "DEFER" };
        println!("{name} worst={worst:.6} verdict={verdict}");
    }
}
```

(If `emit_measure_fitted` and this share a `solve`/`eval_cubic` signature, reuse them as-is — do not duplicate those helpers.)

- [ ] **Step 4: Build the tool**

Run: `cargo build --manifest-path tools/se-ayanamsa-reference/Cargo.toml`
Expected: compiles cleanly (pre-existing warnings about unused offset constants are fine).

- [ ] **Step 5: Run `measure-fitted-offset` and record membership**

Run: `cargo run --quiet --manifest-path tools/se-ayanamsa-reference/Cargo.toml -- measure-fitted-offset | tee docs/superpowers/specs/notes/2026-06-25-fitted-offset-residuals.txt`
Expected: one `worst=…/verdict=…` line per candidate. Record the worst-residual table and the PASS/DEFER set in that notes file. Any `DEFER` mode is dropped from the promoted set in all later tasks (kept `DescriptorOnly`) with its measured residual recorded. Note that `Udayagiri`, `PvrPushyaPaksha`, and `Sheoran` are absent here (no SE_SIDM code) and stay deferred-by-lack-of-code.

- [ ] **Step 6: Commit**

```bash
git add tools/se-ayanamsa-reference/src/main.rs docs/superpowers/specs/notes/2026-06-25-fitted-offset-residuals.txt
git commit -m "feat(tool): emit/fit/measure fitted-offset ayanamsa family (slice 3)"
```

---

## Task 2: Add the `FittedOffset` mode class and its ceiling

**Files:**
- Modify: `crates/pleiades-ayanamsa/src/thresholds.rs`

**Interfaces:**
- Consumes: Task 1's recorded worst-residual table (the PASS set + measured maxima).
- Produces: `AyanamsaModeClass::FittedOffset`; `ayanamsa_mode_ceiling(FittedOffset)`; `ayanamsa_mode_class` returns `FittedOffset` for each promoted mode.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `thresholds.rs` (drop any Task-1-deferred mode from the `FittedOffset` list and assert it `None` instead, with a `// deferred: worst=…` comment):

```rust
    #[test]
    fn promoted_fitted_offset_modes_map_to_their_class() {
        for m in [
            Ayanamsa::DeLuce,
            Ayanamsa::BabylonianKugler1,
            Ayanamsa::BabylonianKugler2,
            Ayanamsa::BabylonianKugler3,
            Ayanamsa::BabylonianHuber,
            Ayanamsa::BabylonianEtaPiscium,
            Ayanamsa::BabylonianAldebaran,
            Ayanamsa::Hipparchus,
            Ayanamsa::BabylonianBritton,
            Ayanamsa::ValensMoon,
            Ayanamsa::LahiriVP285,
            Ayanamsa::KrishnamurtiVP291,
        ] {
            assert_eq!(
                ayanamsa_mode_class(&m),
                Some(AyanamsaModeClass::FittedOffset),
                "{m:?}"
            );
        }
        // No-SE_SIDM anchorless modes stay ungated.
        assert_eq!(ayanamsa_mode_class(&Ayanamsa::Udayagiri), None);
        assert_eq!(ayanamsa_mode_class(&Ayanamsa::PvrPushyaPaksha), None);
        assert_eq!(ayanamsa_mode_class(&Ayanamsa::Sheoran), None);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-ayanamsa thresholds::tests::promoted_fitted_offset_modes_map_to_their_class`
Expected: FAIL — `FittedOffset` variant does not exist / arms return `None`.

- [ ] **Step 3: Add the `FittedOffset` class, ceiling, and class mapping**

In `thresholds.rs`, add the enum variant (after `Galactic`):

```rust
    /// Offset/historical sidereal mode whose linear anchor+precession model
    /// missed the OffsetDefined ceiling, promoted instead by a committed cubic
    /// fit to Swiss Ephemeris mean ayanamsa (same mechanism as TrueStar/Galactic).
    FittedOffset,
```

Add the ceiling arm in `ayanamsa_mode_ceiling` (replace `<FO_CEIL>` with `ceil(measured_max × 2)` from Task 1's maxima, 1.0″ floor — almost certainly `1.0`):

```rust
        AyanamsaModeClass::FittedOffset => AyanamsaCeiling { offset_arcsec: <FO_CEIL> },
```

Add the promoted modes to `ayanamsa_mode_class` (drop any Task-1-deferred mode):

```rust
        Ayanamsa::DeLuce
        | Ayanamsa::BabylonianKugler1
        | Ayanamsa::BabylonianKugler2
        | Ayanamsa::BabylonianKugler3
        | Ayanamsa::BabylonianHuber
        | Ayanamsa::BabylonianEtaPiscium
        | Ayanamsa::BabylonianAldebaran
        | Ayanamsa::Hipparchus
        | Ayanamsa::BabylonianBritton
        | Ayanamsa::ValensMoon
        | Ayanamsa::LahiriVP285
        | Ayanamsa::KrishnamurtiVP291 => Some(AyanamsaModeClass::FittedOffset),
```

Update the doc comment above `ayanamsa_mode_ceiling` to record the `FittedOffset` measured max and ceiling (from Task 1). Add `AyanamsaModeClass::FittedOffset` to the `every_class_has_finite_positive_ceiling` test array, and extend `ayanamsa_thresholds_summary_for_report` to mention the fitted-offset ceiling.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-ayanamsa thresholds`
Expected: PASS (all `thresholds` tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-ayanamsa/src/thresholds.rs
git commit -m "feat(ayanamsa): add FittedOffset mode class + measured ceiling (slice 3)"
```

---

## Task 3: Commit the cubic coefficients and route the offset path

**Files:**
- Create: `crates/pleiades-ayanamsa/src/fitted_offset.rs`
- Modify: `crates/pleiades-ayanamsa/src/lib.rs` (add `mod fitted_offset;`)
- Modify: `crates/pleiades-ayanamsa/src/lookup.rs` (`sidereal_offset` match)

**Interfaces:**
- Consumes: `Ayanamsa` variants; the shared `eval` and window constants `TRUE_STAR_FIT_JD_MIN/MAX` from `truestar.rs` (all `pub(crate)`).
- Produces: `fitted_offset::fitted_offset_degrees(&Ayanamsa, f64) -> Option<f64>` returns `Some` for each promoted mode in-window, `None` otherwise; `sidereal_offset` returns `Some(Angle)` for every promoted mode at a holdout JD.

- [ ] **Step 1: Generate the cubic coefficients**

Run: `cargo run --quiet --manifest-path tools/se-ayanamsa-reference/Cargo.toml -- fit`
Expected: a `// Generated by …` line followed by one `pub(crate) const <NAME>_COEFFS: [f64; 4] = […];` line per fitted mode (the existing 15 plus the 12 new ones). Keep the 12 new `*_COEFFS` lines (e.g. `DELUCE_COEFFS`, `BABYLONIANKUGLER1_COEFFS`, …, `KRISHNAMURTIVP291_COEFFS`) for Step 3.

- [ ] **Step 2: Create `fitted_offset.rs` with a failing-test stub**

Create `crates/pleiades-ayanamsa/src/fitted_offset.rs` (mirrors `galactic.rs`), with the evaluator stubbed to `None` so the test compiles and fails:

```rust
//! Committed cubic polynomials for the "fitted-offset" ayanamsa modes —
//! offset/historical modes whose linear anchor+precession model missed the
//! OffsetDefined ceiling, re-fit by `tools/se-ayanamsa-reference fit` to Swiss
//! Ephemeris **mean** ayanamsa over 1900–2100. Same smooth-cubic rationale as
//! `truestar.rs`/`galactic.rs`.
//!
//! Provenance: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), MEAN_IFLAG
//! (`SEFLG_NONUT | SEFLG_NOABERR`). Per-mode `SE_SIDM` code:
//!   DeLuce=2, BabylonianKugler1=9, BabylonianKugler2=10, BabylonianKugler3=11,
//!   BabylonianHuber=12, BabylonianEtaPiscium=13, BabylonianAldebaran=14,
//!   Hipparchus=15, BabylonianBritton=38, ValensMoon=42, LahiriVP285=44,
//!   KrishnamurtiVP291=45.
#![forbid(unsafe_code)]

use pleiades_types::Ayanamsa;

/// Fitted-offset sidereal offset in degrees, or `None` if `ayanamsa` is not a
/// committed fitted-offset mode or `jd_tt` is outside the fit window (fail-closed).
pub(crate) fn fitted_offset_degrees(_ayanamsa: &Ayanamsa, _jd_tt: f64) -> Option<f64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promoted_fitted_offset_modes_return_some_in_window() {
        let jd = 2_451_545.0;
        for m in [
            Ayanamsa::DeLuce,
            Ayanamsa::BabylonianKugler1,
            Ayanamsa::BabylonianKugler2,
            Ayanamsa::BabylonianKugler3,
            Ayanamsa::BabylonianHuber,
            Ayanamsa::BabylonianEtaPiscium,
            Ayanamsa::BabylonianAldebaran,
            Ayanamsa::Hipparchus,
            Ayanamsa::BabylonianBritton,
            Ayanamsa::ValensMoon,
            Ayanamsa::LahiriVP285,
            Ayanamsa::KrishnamurtiVP291,
        ] {
            assert!(
                fitted_offset_degrees(&m, jd).is_some(),
                "{m:?} in-window should be Some"
            );
            assert!(
                fitted_offset_degrees(&m, 2_300_000.0).is_none(),
                "{m:?} pre-window should be None"
            );
        }
        assert!(fitted_offset_degrees(&Ayanamsa::Lahiri, jd).is_none());
    }
}
```

Add `mod fitted_offset;` to `crates/pleiades-ayanamsa/src/lib.rs` next to `mod galactic;`.

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p pleiades-ayanamsa fitted_offset::`
Expected: FAIL — stub returns `None`, so the in-window assertions fail.

- [ ] **Step 4: Paste coefficients and implement the evaluator**

In `fitted_offset.rs`, add `use crate::truestar::{eval, TRUE_STAR_FIT_JD_MAX, TRUE_STAR_FIT_JD_MIN};` beside the existing `use`, paste the 12 `*_COEFFS` consts (from Step 1) above the function, and replace the stub body (drop any Task-1-deferred mode from this `match`):

```rust
pub(crate) fn fitted_offset_degrees(ayanamsa: &Ayanamsa, jd_tt: f64) -> Option<f64> {
    let coeffs = match ayanamsa {
        Ayanamsa::DeLuce => &DELUCE_COEFFS,
        Ayanamsa::BabylonianKugler1 => &BABYLONIANKUGLER1_COEFFS,
        Ayanamsa::BabylonianKugler2 => &BABYLONIANKUGLER2_COEFFS,
        Ayanamsa::BabylonianKugler3 => &BABYLONIANKUGLER3_COEFFS,
        Ayanamsa::BabylonianHuber => &BABYLONIANHUBER_COEFFS,
        Ayanamsa::BabylonianEtaPiscium => &BABYLONIANETAPISCIUM_COEFFS,
        Ayanamsa::BabylonianAldebaran => &BABYLONIANALDEBARAN_COEFFS,
        Ayanamsa::Hipparchus => &HIPPARCHUS_COEFFS,
        Ayanamsa::BabylonianBritton => &BABYLONIANBRITTON_COEFFS,
        Ayanamsa::ValensMoon => &VALENSMOON_COEFFS,
        Ayanamsa::LahiriVP285 => &LAHIRIVP285_COEFFS,
        Ayanamsa::KrishnamurtiVP291 => &KRISHNAMURTIVP291_COEFFS,
        _ => return None,
    };
    if !(TRUE_STAR_FIT_JD_MIN..=TRUE_STAR_FIT_JD_MAX).contains(&jd_tt) {
        return None;
    }
    Some(eval(coeffs, jd_tt))
}
```

(The pasted const names must match the `fit` output, which uppercases the mode name — verify against Step 1's printed lines.)

- [ ] **Step 5: Route the `FittedOffset` class in `sidereal_offset`**

In `lookup.rs`, add a match arm beside the `Galactic` arm in `sidereal_offset`:

```rust
        // Fitted-offset: committed cubic fit to Swiss Ephemeris.
        Some(AyanamsaModeClass::FittedOffset) => {
            crate::fitted_offset::fitted_offset_degrees(ayanamsa, jd_tt).map(Angle::from_degrees)
        }
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p pleiades-ayanamsa fitted_offset:: lookup`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-ayanamsa/src/fitted_offset.rs crates/pleiades-ayanamsa/src/lib.rs crates/pleiades-ayanamsa/src/lookup.rs
git commit -m "feat(ayanamsa): commit fitted-offset cubics + route offset path (slice 3)"
```

---

## Task 4: Regenerate the corpus and extend the gate

**Files:**
- Modify: `crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv`
- Modify: `crates/pleiades-validate/data/ayanamsa-corpus/manifest.txt`
- Modify: `crates/pleiades-validate/src/ayanamsa_validation.rs`

**Interfaces:**
- Consumes: the tool's `corpus` output; the gate's `sidereal_offset` (now routing all promoted modes).
- Produces: a committed corpus whose every row passes the gate; `mode_for_code` and the completeness list cover the promoted set; `gate_passes_over_committed_corpus` asserts the new count.

- [ ] **Step 1: Regenerate the corpus CSV and bump `rows=`**

Run: `cargo run --quiet --manifest-path tools/se-ayanamsa-reference/Cargo.toml -- corpus > crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv`
Expected: header + `(36 + N_promoted) × 10` data rows, where `N_promoted` is the PASS set from Task 1 (≤ 12). If a candidate deferred, remove its `(name, code)` from `MODES` (Task 1 Step 1) before regenerating so no deferred rows are emitted. Set `rows=` in `manifest.txt` to the new data-row count (total lines minus the 1 header line; confirm with `wc -l`).

- [ ] **Step 2: Extend `mode_for_code` and the completeness list**

In `ayanamsa_validation.rs`, add a `mode_for_code` arm for each promoted mode (drop any deferred):

```rust
        "DeLuce" => Some(Ayanamsa::DeLuce),
        "BabylonianKugler1" => Some(Ayanamsa::BabylonianKugler1),
        "BabylonianKugler2" => Some(Ayanamsa::BabylonianKugler2),
        "BabylonianKugler3" => Some(Ayanamsa::BabylonianKugler3),
        "BabylonianHuber" => Some(Ayanamsa::BabylonianHuber),
        "BabylonianEtaPiscium" => Some(Ayanamsa::BabylonianEtaPiscium),
        "BabylonianAldebaran" => Some(Ayanamsa::BabylonianAldebaran),
        "Hipparchus" => Some(Ayanamsa::Hipparchus),
        "BabylonianBritton" => Some(Ayanamsa::BabylonianBritton),
        "ValensMoon" => Some(Ayanamsa::ValensMoon),
        "LahiriVP285" => Some(Ayanamsa::LahiriVP285),
        "KrishnamurtiVP291" => Some(Ayanamsa::KrishnamurtiVP291),
```

Append the same code strings to the completeness `for code in [ … ]` array (after `"GalacticEquatorFiorenza",`), and update its `// Completeness: all 36 gated modes present.` comment to `36 + N_promoted`.

- [ ] **Step 3: Bump the manifest checksum (use the gate to read it)**

Run: `cargo test -p pleiades-validate ayanamsa -- --nocapture`
Expected: `gate_passes_over_committed_corpus` fails with `ChecksumMismatch { expected, actual }` — `actual` is the recomputed `fnv1a64(CORPUS_CSV)`. Paste `actual` into `manifest.txt`'s `checksum=` field. Do **not** compute it with any external FNV tool (non-canonical prime).

- [ ] **Step 4: Update the gate's count test**

In `ayanamsa_validation.rs` tests, change `assert_eq!(report.modes_checked, 36);` (~line 384) to the new total (`36 + N_promoted`).

- [ ] **Step 5: Run the gate to verify it passes**

Run: `cargo test -p pleiades-validate ayanamsa`
Expected: PASS — `gate_passes_over_committed_corpus`, `checksum_drift_fails_closed`, and the ceiling/parse tests all green over the expanded corpus.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv crates/pleiades-validate/data/ayanamsa-corpus/manifest.txt crates/pleiades-validate/src/ayanamsa_validation.rs
git commit -m "feat(validate): commit fitted-offset corpus rows + gate promoted modes (slice 3)"
```

---

## Task 5: Promote the catalog claim tiers and tighten the deferred guard

**Files:**
- Modify: `crates/pleiades-ayanamsa/src/catalog.rs`
- Modify: `crates/pleiades-ayanamsa/src/catalog/tests.rs`

**Interfaces:**
- Consumes: the PASS set from Task 1.
- Produces: each promoted mode's descriptor reports `claim_tier == ReleaseGradeNumeric` (in both source arrays and `BUILT_IN_AYANAMSAS`); the deferred-guard test covers only the still-deferred set.

- [ ] **Step 1: Write the failing test**

In `catalog/tests.rs`, add (drop any Task-1-deferred mode from the promoted list):

```rust
    #[test]
    fn promoted_fitted_offset_modes_are_release_grade() {
        use crate::descriptor;
        use pleiades_types::{Ayanamsa, CompatibilityClaimTier};
        for m in [
            Ayanamsa::DeLuce,
            Ayanamsa::BabylonianKugler1,
            Ayanamsa::BabylonianKugler2,
            Ayanamsa::BabylonianKugler3,
            Ayanamsa::BabylonianHuber,
            Ayanamsa::BabylonianEtaPiscium,
            Ayanamsa::BabylonianAldebaran,
            Ayanamsa::Hipparchus,
            Ayanamsa::BabylonianBritton,
            Ayanamsa::ValensMoon,
            Ayanamsa::LahiriVP285,
            Ayanamsa::KrishnamurtiVP291,
        ] {
            let d = descriptor(&m).expect("descriptor exists");
            assert_eq!(
                d.claim_tier,
                CompatibilityClaimTier::ReleaseGradeNumeric,
                "{m:?}"
            );
        }
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-ayanamsa promoted_fitted_offset_modes_are_release_grade`
Expected: FAIL — descriptors are still `DescriptorOnly`.

- [ ] **Step 3: Flip the constructor for each promoted mode (both arrays)**

For each promoted `Ayanamsa` variant, change its descriptor constructor from `AyanamsaDescriptor::new(` to `AyanamsaDescriptor::new_release_grade(`. Each variant appears **twice** — once in `BASELINE_AYANAMSAS`/`RELEASE_AYANAMSAS` and once in `BUILT_IN_AYANAMSAS`. Flip both. Locate them with:

```bash
grep -nE "Ayanamsa::(DeLuce|BabylonianKugler1|BabylonianKugler2|BabylonianKugler3|BabylonianHuber|BabylonianEtaPiscium|BabylonianAldebaran|Hipparchus|BabylonianBritton|ValensMoon|LahiriVP285|KrishnamurtiVP291)\b" crates/pleiades-ayanamsa/src/catalog.rs
```

Leave the anchorless and observational modes on `new(` (they stay `DescriptorOnly`).

- [ ] **Step 4: Update the deferred-guard and exact-gated-set tests**

In `catalog/tests.rs`, edit `deferred_modes_stay_descriptor_only`: **remove** the 12 now-promoted modes from the `deferred` array, leaving the still-deferred set, and update the leading comment. The array should become:

```rust
    let deferred = [
        // Anchorless modes — no distinct SE_SIDM code, must stay DescriptorOnly.
        Ayanamsa::Udayagiri,
        Ayanamsa::PvrPushyaPaksha,
        Ayanamsa::Sheoran,
        // Observational/topocentric/house Babylonians — not smooth in time.
        Ayanamsa::BabylonianTrueGeoc,
        Ayanamsa::BabylonianTrueTopc,
        Ayanamsa::BabylonianTrueObs,
        Ayanamsa::BabylonianHouse,
        Ayanamsa::BabylonianHouseObs,
        Ayanamsa::BabylonianSissy,
        // No distinct SE code.
        Ayanamsa::DhruvaGalacticCenterMula,
        Ayanamsa::GalacticEquator,
    ];
```

(If Task 1 deferred any of the 12 candidates, add it back here with a `// deferred: worst=…` comment.)

In `release_grade_numeric_ayanamsa_set_is_exactly_the_gated_modes`, add the promoted modes to the `expected` array (a `// Fitted-offset family promoted in Phase 6 slice 3` block after the slice-2 entries).

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p pleiades-ayanamsa`
Expected: PASS, including the new test, the updated deferred guard, and the exact-gated-set test. If any other release-grade-count assertion in `catalog/tests.rs` fails, update it to the new total.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-ayanamsa/src/catalog.rs crates/pleiades-ayanamsa/src/catalog/tests.rs
git commit -m "feat(ayanamsa): promote fitted-offset family to release-grade in catalog (slice 3)"
```

---

## Task 6: Align claims surfaces and run the full gates

**Files:**
- Modify: `crates/pleiades-validate/src/tests/compatibility.rs`
- Modify: `crates/pleiades-validate/src/tests/release_checklist.rs`
- Modify: `crates/pleiades-validate/src/tests/render_catalog.rs`
- Modify: `crates/pleiades-cli/src/cli/tests/summary_commands.rs`
- Modify: `README.md`
- Modify: `PLAN.md`

(The exact set of failing test files is whatever Step 1 surfaces; the four above are the ones slice 2 touched. Update only those that actually fail.)

**Interfaces:**
- Consumes: the new gated count (`36 + N_promoted`) and the shrunken deferred set.
- Produces: all four `compat-claims-audit` surfaces agree; README/PLAN prose matches; full workspace test suite + release gate green.

- [ ] **Step 1: Run the suite to surface every count/golden that moved**

Run: `cargo test --workspace 2>&1 | tee /tmp/claude-1000/-workspace/75205938-934e-4392-805d-53471b2e580a/scratchpad/slice3-fails.txt`
Expected: failures only in count/golden assertions (release-grade ayanamsa count, rendered catalog snapshots, CLI summary goldens). Read each failure's expected-vs-actual.

- [ ] **Step 2: Update each failing assertion to the new measured value**

For each failure, update the asserted count/golden to the value the gate/render now produces (audit-derived truths, not free choices). Do not weaken any assertion — only update the number/string to the new correct value.

- [ ] **Step 3: Update README and PLAN prose**

In `README.md` line 20, change `36 release-claimed ayanamsa modes pass theirs (the remaining 23 are catalogued with metadata only)` to `(36 + N_promoted)` release-claimed and `(23 − N_promoted)` remaining.

In `PLAN.md`, update the Phase 6 ayanamsa note (~lines 80–90), the Phase 5 note (~lines 125–142), and the Status line (~line 179): new gated count = `6 original + 17 offset-defined + 13 fitted (slice 2) + N_promoted fitted-offset (slice 3)`; record slice 3 as done (2026-06-25); list the still-deferred set (3 anchorless no-code modes, 6 observational/topocentric/house Babylonians, `DhruvaGalacticCenterMula`, legacy `GalacticEquator`).

- [ ] **Step 4: Run the full workspace suite**

Run: `cargo test --workspace`
Expected: PASS (entire workspace).

- [ ] **Step 5: Run the release gate and claims audit**

Run: `mise run release-gate` (or `cargo run -p pleiades-cli -- release-gate`) and `cargo run -p pleiades-cli -- compat-claims-audit`.
(Confirm subcommand names with `cargo run -p pleiades-cli -- --help` if they differ.)
Expected: release gate green; claims audit reports bidirectional agreement across catalog / evidence / profile / prose with no overclaim and no missing evidence.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-validate/src/tests/ crates/pleiades-cli/src/cli/tests/summary_commands.rs README.md PLAN.md
git commit -m "docs+test: align claims surfaces to fitted-offset promotion (slice 3)"
```

---

## Notes for the implementer

- **Run the SE tool first (Task 1) and let its output drive everything.** The PASS/DEFER membership, the cubic coefficients, the ceiling, the corpus rows, and every "new count" are tool/gate outputs. Wherever a step says "paste the printed …" or "the value the gate reports," use that captured output — never invent a number.
- **If a candidate defers,** remove it consistently from: the corpus `MODES` (Task 1), the class mapping and tests (Task 2), the evaluator `match` and tests (Task 3), `mode_for_code`/completeness (Task 4), and the catalog flip + the exact-gated-set test (Task 5) — and add it back to `deferred_modes_stay_descriptor_only` (Task 5 Step 4) with its measured residual. It must remain `DescriptorOnly` everywhere.
- **The 3 anchorless modes (Udayagiri, PvrPushyaPaksha, Sheoran) have no SE_SIDM code** in libswisseph-sys 0.1.2 / SE 2.10.03 — they cannot be fit and stay `DescriptorOnly`. This is verified, not assumed; Task 1 Step 5 simply re-confirms their absence.
- **The compatibility profile** needs no hand-edited mode list — `compat-claims-audit` derives the claimed set from catalog `claim_tier`. Task 6 Step 5 verifies the audit agrees; if it reports a profile mismatch, follow its message to the offending surface.
- **Compatibility-profile version bump is deliberately deferred** to the end of the full ayanamsa family (design doc §8). Do not bump it in this slice.
- **Approach A is fixed:** promoted modes join `FittedOffset` only; never fold them into `OffsetDefined`, and do not re-touch already-gated `OffsetDefined`/`TrueStar`/`Galactic` modes.
