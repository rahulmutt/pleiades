# Ayanamsa Fitted-Family Promotion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote the smooth fitted ayanamsa family (4 True-star + the galactic modes with a distinct SE code) from `DescriptorOnly` to `ReleaseGradeNumeric` via committed cubic polynomials fit to Swiss Ephemeris mean ayanamsa, gated by the existing fail-closed numeric gate.

**Architecture:** Extend the proven TrueChitra/TrueCitra cubic-fit path. The SE reference tool emits per-mode cubic coefficients and a holdout corpus from SE 2.10.03 mean ayanamsa; the coefficients are committed into the crate (`truestar.rs` for star modes, a new `galactic.rs` for galactic modes); `sidereal_offset` routes each gated mode class to its evaluator; the gate validates every committed corpus row against a per-family arcsecond ceiling. Promotion is measured, not asserted: a mode passes only if every holdout residual is within its family ceiling, otherwise it is recorded as deferred and stays `DescriptorOnly`.

**Tech Stack:** Rust (workspace crates `pleiades-ayanamsa`, `pleiades-validate`, `pleiades-apparent`, `pleiades-types`); standalone `tools/se-ayanamsa-reference` binary linking Swiss Ephemeris (`swisseph`/`libswisseph-sys`); FNV-1a-64 corpus checksums.

## Global Constraints

- **Reference engine:** Swiss Ephemeris **2.10.03** mean ayanamsa, `iflag = SEFLG_NONUT | SEFLG_NOABERR` (= 1088). This exact version is pinned in the corpus manifest; do not change it.
- **FNV prime is non-canonical:** corpus checksums use `pleiades_apparent::fnv1a64` (a typo'd FNV prime). Recompute checksums only with that in-repo function — stock FNV will not match.
- **Fail-closed:** every gate failure (checksum mismatch, manifest drift on `rows=`/completeness, malformed row, unknown mode code, `None` calculation, ceiling exceeded) aborts the gate. No silent skips.
- **Fit/eval window:** 1900–2100, JD TT `[2_415_020.0, 2_488_070.0]` (the existing `TRUE_STAR_FIT_JD_MIN/MAX`). Outside the window the evaluator returns `None` (fail-closed), never an extrapolated value.
- **Ceiling policy:** per mode class, `ceil(measured_max_arcsec × 2)` with a **1.0″ floor**, recomputed over the full promoted set for that class.
- **Holdout grid:** the existing 10 `HOLDOUT_JD_TT` instants. Do not change the grid.
- **Membership is empirical:** any in-scope mode for which SE has no distinct `SE_SIDM` code, or whose cubic cannot meet its ceiling, is deferred with a recorded reason and stays `DescriptorOnly`. No overclaim.
- **SE tool is excluded from the workspace** (`Cargo.toml` `exclude`); build/run it with `--manifest-path tools/se-ayanamsa-reference/Cargo.toml`.

### Mode inventory (target membership)

True-star → `TrueStar` class:

| Mode | SE_SIDM |
| --- | --- |
| `TrueRevati` | 28 |
| `TruePushya` | 29 |
| `TrueMula` | 35 |
| `TrueSheoran` | 39 |

Galactic → new `Galactic` class:

| Mode | SE_SIDM |
| --- | --- |
| `GalacticCenter` | 17 |
| `GalacticCenterRgilbrand` | 30 |
| `GalacticEquatorIau1958` | 31 |
| `GalacticEquatorTrue` | 32 |
| `GalacticEquatorMula` | 33 |
| `GalacticCenterMardyks` (GALALIGN_MARDYKS) | 34 |
| `GalacticCenterMulaWilhelm` | 36 |
| `GalacticCenterCochrane` | 40 |
| `GalacticEquatorFiorenza` | 41 |

Expected-deferred (no distinct SE_SIDM code; confirm in Task 1 and keep `DescriptorOnly`): `DhruvaGalacticCenterMula` (a Wilhelm-projection synonym of `GalacticCenterMulaWilhelm`/36) and the legacy `GalacticEquator` alias mode (duplicate of a `GalacticEquator*` variant). Record the reason; do not force-gate.

> **Empirical values produced at runtime:** the cubic coefficients (Task 3), the measured worst residual per mode and final PASS/DEFER membership (Task 1), the recomputed ceilings (Task 2), the regenerated corpus rows + checksum (Task 4), and the final gated mode count (Tasks 4–6) are all **outputs of running the SE tool and the gate**. Each step below gives the exact command that produces the value and where to paste it. These are not author placeholders — they are tool outputs the engineer captures.

---

## File Structure

- `tools/se-ayanamsa-reference/src/main.rs` — extend `MODES`; generalize `emit_fit`/holdout-measure to iterate the fitted family (not just code 27). Responsibility: produce SE reference rows, cubic coefficients, and per-mode residuals.
- `crates/pleiades-ayanamsa/src/thresholds.rs` — add `Galactic` mode class, its ceiling, and class mapping for the promoted modes.
- `crates/pleiades-ayanamsa/src/truestar.rs` — add the 4 new True-star cubic coefficient constants and their match arms.
- `crates/pleiades-ayanamsa/src/galactic.rs` — **new**. Galactic cubic coefficients + `galactic_offset_degrees(&Ayanamsa, jd_tt) -> Option<f64>` with the shared window guard.
- `crates/pleiades-ayanamsa/src/lib.rs` — declare `mod galactic;`.
- `crates/pleiades-ayanamsa/src/lookup.rs` — route the `Galactic` class in `sidereal_offset`.
- `crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv` + `manifest.txt` — regenerated corpus rows; bumped `rows=` and `checksum=`.
- `crates/pleiades-validate/src/ayanamsa_validation.rs` — extend `mode_for_code`, the completeness list, and the `modes_checked` test count.
- `crates/pleiades-ayanamsa/src/catalog.rs` — flip promoted modes from `AyanamsaDescriptor::new(` to `new_release_grade(` in **both** the `BASELINE_AYANAMSAS`/`RELEASE_AYANAMSAS` source arrays and the `BUILT_IN_AYANAMSAS` flat array.
- Test/prose surfaces updated for the new count: `crates/pleiades-ayanamsa/src/catalog/tests.rs`, `crates/pleiades-validate/src/tests/compatibility.rs`, `crates/pleiades-validate/src/tests/release_checklist.rs`, `crates/pleiades-validate/src/tests/render_catalog.rs`, `crates/pleiades-cli/src/cli/tests/summary_commands.rs`, `README.md`, `PLAN.md`.

---

## Task 1: Extend the SE tool to emit, fit, and measure the fitted family

**Files:**
- Modify: `tools/se-ayanamsa-reference/src/main.rs` (`MODES` ~16-45; `emit_fit` ~106-130; add a fitted-family holdout-measure)

**Interfaces:**
- Consumes: existing `ayanamsa(code, jd_tt)`, `solve`, `eval_cubic`, `HOLDOUT_JD_TT`, `MEAN_IFLAG`.
- Produces: `corpus` subcommand emits reference rows for the fitted modes; `fit` prints a `<NAME>_COEFFS` const block per fitted mode; a `measure-fitted` subcommand prints `<name> worst=<arcsec> verdict=<PASS|DEFER>` per mode using the cubic fit vs SE.

- [ ] **Step 1: Add the fitted family to `MODES`**

In `tools/se-ayanamsa-reference/src/main.rs`, replace the trailing deferred-comment block of `MODES` (the lines from `// Deferred: DeLuce …` through `// PvrPushyaPaksha, Udayagiri, Sheoran have no upstream SE_SIDM …`) with the fitted entries, keeping the existing 22 entries above untouched:

```rust
    // fitted family — slice 2: true-star + galactic cubic fits.
    ("TrueRevati",                28),
    ("TruePushya",                29),
    ("TrueMula",                  35),
    ("TrueSheoran",               39),
    ("GalacticCenter",            17),
    ("GalacticCenterRgilbrand",   30),
    ("GalacticEquatorIau1958",    31),
    ("GalacticEquatorTrue",       32),
    ("GalacticEquatorMula",       33),
    ("GalacticCenterMardyks",     34),
    ("GalacticCenterMulaWilhelm", 36),
    ("GalacticCenterCochrane",    40),
    ("GalacticEquatorFiorenza",   41),
    // Still deferred (offset family that exceeded the 3.0" linear ceiling,
    // anchorless modes, observational Babylonians). No SE_SIDM emitted here for
    // DhruvaGalacticCenterMula / legacy GalacticEquator: no distinct SE code.
```

- [ ] **Step 2: Generalize `emit_fit` to iterate a fitted-mode list**

Replace the hardcoded `for &(name, code) in &[("TrueChitra", 27), ("TrueCitra", 27)] {` line in `emit_fit` with a named slice covering the originals plus the fitted family, so one `fit` run prints every committed cubic:

```rust
    const FIT_MODES: &[(&str, i32)] = &[
        ("TrueChitra", 27), ("TrueCitra", 27),
        ("TrueRevati", 28), ("TruePushya", 29), ("TrueMula", 35), ("TrueSheoran", 39),
        ("GalacticCenter", 17), ("GalacticCenterRgilbrand", 30),
        ("GalacticEquatorIau1958", 31), ("GalacticEquatorTrue", 32),
        ("GalacticEquatorMula", 33), ("GalacticCenterMardyks", 34),
        ("GalacticCenterMulaWilhelm", 36), ("GalacticCenterCochrane", 40),
        ("GalacticEquatorFiorenza", 41),
    ];
    for &(name, code) in FIT_MODES {
```

(Leave the body of the loop — normal-equation assembly, `solve`, and the `println!` const block — unchanged.)

- [ ] **Step 3: Add a `measure-fitted` subcommand**

Add this function near `emit_holdout_check`, and wire it into `main`'s match (`Some("measure-fitted") => emit_measure_fitted(),`):

```rust
/// Fit each fitted-family mode over the dense even-year grid, then report the
/// worst holdout residual (arcsec) and a PASS/DEFER verdict at the 1.0" floor
/// (the per-family ceiling is recomputed in the crate from the measured maxima).
fn emit_measure_fitted() {
    let dense: Vec<f64> = (1900..=2100)
        .step_by(2)
        .map(|y| 2_451_545.0 + (y as f64 - 2000.0) * 365.25)
        .collect();
    const FIT_MODES: &[(&str, i32)] = &[
        ("TrueRevati", 28), ("TruePushya", 29), ("TrueMula", 35), ("TrueSheoran", 39),
        ("GalacticCenter", 17), ("GalacticCenterRgilbrand", 30),
        ("GalacticEquatorIau1958", 31), ("GalacticEquatorTrue", 32),
        ("GalacticEquatorMula", 33), ("GalacticCenterMardyks", 34),
        ("GalacticCenterMulaWilhelm", 36), ("GalacticCenterCochrane", 40),
        ("GalacticEquatorFiorenza", 41),
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

- [ ] **Step 4: Build the tool**

Run: `cargo build --manifest-path tools/se-ayanamsa-reference/Cargo.toml`
Expected: compiles cleanly (warnings about unused `OFFSET_DEFINED_*` constants are pre-existing and fine).

- [ ] **Step 5: Run `measure-fitted` and record membership**

Run: `cargo run --quiet --manifest-path tools/se-ayanamsa-reference/Cargo.toml -- measure-fitted | tee docs/superpowers/specs/notes/2026-06-25-fitted-family-residuals.txt`
Expected: one `worst=…/verdict=…` line per mode. Record the worst-residual table and the PASS/DEFER set in that notes file. Any `DEFER` mode is dropped from the promoted set in all later tasks (kept `DescriptorOnly`) with its measured residual recorded. Confirm `DhruvaGalacticCenterMula` and legacy `GalacticEquator` are absent (no SE code) and note them as deferred-by-lack-of-code.

- [ ] **Step 6: Commit**

```bash
git add tools/se-ayanamsa-reference/src/main.rs docs/superpowers/specs/notes/2026-06-25-fitted-family-residuals.txt
git commit -m "feat(tool): emit/fit/measure fitted ayanamsa family (true-star + galactic)"
```

---

## Task 2: Add the `Galactic` mode class and recompute per-family ceilings

**Files:**
- Modify: `crates/pleiades-ayanamsa/src/thresholds.rs`

**Interfaces:**
- Consumes: Task 1's recorded worst-residual table.
- Produces: `AyanamsaModeClass::Galactic`; `ayanamsa_mode_ceiling(Galactic)`; `ayanamsa_mode_class` returns `TrueStar` for the 4 new star modes and `Galactic` for the promoted galactic modes.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `thresholds.rs`:

```rust
    #[test]
    fn promoted_fitted_modes_map_to_their_class() {
        for m in [
            Ayanamsa::TrueRevati, Ayanamsa::TruePushya,
            Ayanamsa::TrueMula, Ayanamsa::TrueSheoran,
        ] {
            assert_eq!(ayanamsa_mode_class(&m), Some(AyanamsaModeClass::TrueStar), "{m:?}");
        }
        for m in [
            Ayanamsa::GalacticCenter, Ayanamsa::GalacticCenterRgilbrand,
            Ayanamsa::GalacticEquatorIau1958, Ayanamsa::GalacticEquatorTrue,
            Ayanamsa::GalacticEquatorMula, Ayanamsa::GalacticCenterMardyks,
            Ayanamsa::GalacticCenterMulaWilhelm, Ayanamsa::GalacticCenterCochrane,
            Ayanamsa::GalacticEquatorFiorenza,
        ] {
            assert_eq!(ayanamsa_mode_class(&m), Some(AyanamsaModeClass::Galactic), "{m:?}");
        }
        // Expected-deferred: no distinct SE code -> stay ungated.
        assert_eq!(ayanamsa_mode_class(&Ayanamsa::DhruvaGalacticCenterMula), None);
        assert_eq!(ayanamsa_mode_class(&Ayanamsa::GalacticEquator), None);
    }
```

(If Task 1 deferred any galactic mode, move it from the `Galactic` list here to the `None` assertions and leave a `// deferred: worst=…"` comment.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-ayanamsa thresholds::tests::promoted_fitted_modes_map_to_their_class`
Expected: FAIL — `Galactic` variant does not exist / arms return `None`.

- [ ] **Step 3: Add the `Galactic` class, ceiling, and class mapping**

In `thresholds.rs`, add the enum variant (after `TrueStar`):

```rust
    /// Sidereal point pinned to a galactic reference (center / equator), fit to
    /// Swiss Ephemeris mean ayanamsa.
    Galactic,
```

Add the ceiling arm in `ayanamsa_mode_ceiling` (use Task 1's measured galactic max in the `ceil(max×2)`/1.0″-floor formula; replace `<GAL_CEIL>` with that computed value, e.g. `1.0`):

```rust
        AyanamsaModeClass::Galactic => AyanamsaCeiling { offset_arcsec: <GAL_CEIL> },
```

Update the `TrueStar` arm only if Task 1's recomputed True-star max raises it above the current 1.0″ floor; otherwise leave it `1.0`.

Add the promoted modes to `ayanamsa_mode_class`. Extend the existing `TrueChitra | TrueCitra` arm to include the 4 new star modes:

```rust
        Ayanamsa::TrueChitra | Ayanamsa::TrueCitra
        | Ayanamsa::TrueRevati | Ayanamsa::TruePushya
        | Ayanamsa::TrueMula | Ayanamsa::TrueSheoran => Some(AyanamsaModeClass::TrueStar),
        Ayanamsa::GalacticCenter
        | Ayanamsa::GalacticCenterRgilbrand
        | Ayanamsa::GalacticEquatorIau1958
        | Ayanamsa::GalacticEquatorTrue
        | Ayanamsa::GalacticEquatorMula
        | Ayanamsa::GalacticCenterMardyks
        | Ayanamsa::GalacticCenterMulaWilhelm
        | Ayanamsa::GalacticCenterCochrane
        | Ayanamsa::GalacticEquatorFiorenza => Some(AyanamsaModeClass::Galactic),
```

Update the doc comment above `ayanamsa_mode_ceiling` to record both families' measured maxima (from Task 1) and the resulting ceilings. Update `every_class_has_finite_positive_ceiling` to include `AyanamsaModeClass::Galactic` in its array, and `ayanamsa_thresholds_summary_for_report` to mention the galactic ceiling.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-ayanamsa thresholds`
Expected: PASS (all `thresholds` tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-ayanamsa/src/thresholds.rs
git commit -m "feat(ayanamsa): add Galactic mode class + per-family fitted ceilings"
```

---

## Task 3: Commit the cubic coefficients and route the offset path

**Files:**
- Modify: `crates/pleiades-ayanamsa/src/truestar.rs`
- Create: `crates/pleiades-ayanamsa/src/galactic.rs`
- Modify: `crates/pleiades-ayanamsa/src/lib.rs` (add `mod galactic;`)
- Modify: `crates/pleiades-ayanamsa/src/lookup.rs` (`sidereal_offset` match)

**Interfaces:**
- Consumes: `Ayanamsa` variants; the shared window constants `TRUE_STAR_FIT_JD_MIN/MAX`.
- Produces: `truestar::true_star_offset_degrees` returns `Some` for the 4 new star modes in-window; `galactic::galactic_offset_degrees(&Ayanamsa, f64) -> Option<f64>` returns `Some` for the promoted galactic modes in-window, `None` otherwise; `sidereal_offset` returns `Some(Angle)` for every promoted mode at a holdout JD.

- [ ] **Step 1: Generate the cubic coefficients**

Run: `cargo run --quiet --manifest-path tools/se-ayanamsa-reference/Cargo.toml -- fit`
Expected: a `// Generated by …` line followed by one `pub(crate) const <NAME>_COEFFS: [f64; 4] = […];` line per fitted mode. Keep this output for Steps 3–4.

- [ ] **Step 2: Write the failing tests**

In `truestar.rs` tests, add:

```rust
    #[test]
    fn new_true_star_modes_return_some_in_window() {
        let jd = 2_451_545.0;
        for m in [Ayanamsa::TrueRevati, Ayanamsa::TruePushya, Ayanamsa::TrueMula, Ayanamsa::TrueSheoran] {
            assert!(true_star_offset_degrees(&m, jd).is_some(), "{m:?} in-window should be Some");
            assert!(true_star_offset_degrees(&m, 2_300_000.0).is_none(), "{m:?} pre-window should be None");
        }
    }
```

Create `crates/pleiades-ayanamsa/src/galactic.rs` with only its test module + an empty signature stub so the test compiles and fails:

```rust
//! Committed cubic polynomials for the galactic-reference ayanamsa modes
//! (galactic center / galactic equator families), fit by
//! `tools/se-ayanamsa-reference fit` to Swiss Ephemeris **mean** ayanamsa over
//! 1900–2100. Same smooth-cubic rationale as `truestar.rs`.
#![forbid(unsafe_code)]

use pleiades_types::Ayanamsa;

/// Galactic sidereal offset in degrees, or `None` if `ayanamsa` is not a
/// committed galactic mode or `jd_tt` is outside the fit window (fail-closed).
pub(crate) fn galactic_offset_degrees(_ayanamsa: &Ayanamsa, _jd_tt: f64) -> Option<f64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promoted_galactic_modes_return_some_in_window() {
        let jd = 2_451_545.0;
        for m in [
            Ayanamsa::GalacticCenter, Ayanamsa::GalacticCenterRgilbrand,
            Ayanamsa::GalacticEquatorIau1958, Ayanamsa::GalacticEquatorTrue,
            Ayanamsa::GalacticEquatorMula, Ayanamsa::GalacticCenterMardyks,
            Ayanamsa::GalacticCenterMulaWilhelm, Ayanamsa::GalacticCenterCochrane,
            Ayanamsa::GalacticEquatorFiorenza,
        ] {
            assert!(galactic_offset_degrees(&m, jd).is_some(), "{m:?} in-window should be Some");
            assert!(galactic_offset_degrees(&m, 2_300_000.0).is_none(), "{m:?} pre-window should be None");
        }
        assert!(galactic_offset_degrees(&Ayanamsa::Lahiri, jd).is_none());
    }
}
```

Add `mod galactic;` to `crates/pleiades-ayanamsa/src/lib.rs` (next to `mod truestar;`), and change `truestar.rs`'s window constants from `pub(crate) const` so `galactic.rs` can import them (they are already `pub(crate)` — confirm). They are; no change needed beyond the `use`.

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p pleiades-ayanamsa galactic:: truestar::tests::new_true_star_modes_return_some_in_window`
Expected: FAIL — galactic stub returns `None`; new star modes hit the `_ => None` arm.

- [ ] **Step 4: Paste coefficients and implement evaluators**

In `truestar.rs`, paste the 4 new `*_COEFFS` consts (from Step 1) beside the existing ones, and extend the `match ayanamsa` in `true_star_offset_degrees` so the 4 new modes are window-checked and evaluated:

```rust
        Ayanamsa::TrueChitra | Ayanamsa::TrueCitra
        | Ayanamsa::TrueRevati | Ayanamsa::TruePushya
        | Ayanamsa::TrueMula | Ayanamsa::TrueSheoran => {
            if !(TRUE_STAR_FIT_JD_MIN..=TRUE_STAR_FIT_JD_MAX).contains(&jd_tt) {
                return None;
            }
            let coeffs = match ayanamsa {
                Ayanamsa::TrueChitra => &TRUECHITRA_COEFFS,
                Ayanamsa::TrueCitra => &TRUECITRA_COEFFS,
                Ayanamsa::TrueRevati => &TRUEREVATI_COEFFS,
                Ayanamsa::TruePushya => &TRUEPUSHYA_COEFFS,
                Ayanamsa::TrueMula => &TRUEMULA_COEFFS,
                _ => &TRUESHEORAN_COEFFS,
            };
            Some(eval(coeffs, jd_tt))
        }
```

In `galactic.rs`, replace the stub: add `use crate::truestar::{TRUE_STAR_FIT_JD_MAX, TRUE_STAR_FIT_JD_MIN};` (both are `pub(crate)` in `truestar.rs`), paste the 9 galactic `*_COEFFS` consts, add the shared `eval` (copy the 3-line `eval` from `truestar.rs`), and implement:

```rust
pub(crate) fn galactic_offset_degrees(ayanamsa: &Ayanamsa, jd_tt: f64) -> Option<f64> {
    let coeffs = match ayanamsa {
        Ayanamsa::GalacticCenter => &GALACTICCENTER_COEFFS,
        Ayanamsa::GalacticCenterRgilbrand => &GALACTICCENTERRGILBRAND_COEFFS,
        Ayanamsa::GalacticEquatorIau1958 => &GALACTICEQUATORIAU1958_COEFFS,
        Ayanamsa::GalacticEquatorTrue => &GALACTICEQUATORTRUE_COEFFS,
        Ayanamsa::GalacticEquatorMula => &GALACTICEQUATORMULA_COEFFS,
        Ayanamsa::GalacticCenterMardyks => &GALACTICCENTERMARDYKS_COEFFS,
        Ayanamsa::GalacticCenterMulaWilhelm => &GALACTICCENTERMULAWILHELM_COEFFS,
        Ayanamsa::GalacticCenterCochrane => &GALACTICCENTERCOCHRANE_COEFFS,
        Ayanamsa::GalacticEquatorFiorenza => &GALACTICEQUATORFIORENZA_COEFFS,
        _ => return None,
    };
    if !(TRUE_STAR_FIT_JD_MIN..=TRUE_STAR_FIT_JD_MAX).contains(&jd_tt) {
        return None;
    }
    Some(eval(coeffs, jd_tt))
}
```

(Drop any Task-1-deferred galactic mode from both the `match` arm here and the test list in Step 2.)

- [ ] **Step 5: Route the `Galactic` class in `sidereal_offset`**

In `lookup.rs`, add a match arm beside the `TrueStar` arm in `sidereal_offset`:

```rust
        // Galactic: committed cubic fit to Swiss Ephemeris.
        Some(AyanamsaModeClass::Galactic) => {
            crate::galactic::galactic_offset_degrees(ayanamsa, jd_tt).map(Angle::from_degrees)
        }
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p pleiades-ayanamsa galactic:: truestar:: lookup`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-ayanamsa/src/truestar.rs crates/pleiades-ayanamsa/src/galactic.rs crates/pleiades-ayanamsa/src/lib.rs crates/pleiades-ayanamsa/src/lookup.rs
git commit -m "feat(ayanamsa): commit fitted-family cubics + route galactic offset path"
```

---

## Task 4: Regenerate the corpus and extend the gate

**Files:**
- Modify: `crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv`
- Modify: `crates/pleiades-validate/data/ayanamsa-corpus/manifest.txt`
- Modify: `crates/pleiades-validate/src/ayanamsa_validation.rs`

**Interfaces:**
- Consumes: the tool's `corpus` output; the gate's `sidereal_offset` (now routing all promoted modes).
- Produces: a committed corpus whose every row passes the gate; `mode_for_code` and the completeness list cover the promoted set; the `gate_passes_over_committed_corpus` test asserts the new count.

- [ ] **Step 1: Regenerate the corpus CSV**

Run: `cargo run --quiet --manifest-path tools/se-ayanamsa-reference/Cargo.toml -- corpus > crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv`
Expected: header + `(22 + N_promoted) × 10` data rows, where `N_promoted` is the PASS set from Task 1. If a mode was deferred, remove its `(name, code)` from `MODES` (Task 1 Step 1) before regenerating so no deferred rows are emitted. Bump `rows=` in `manifest.txt` to the new data-row count (`wc -l` minus the header).

- [ ] **Step 2: Extend `mode_for_code` and the completeness list**

In `ayanamsa_validation.rs`, add a match arm to `mode_for_code` for each promoted mode, e.g.:

```rust
        "TrueRevati" => Some(Ayanamsa::TrueRevati),
        "TruePushya" => Some(Ayanamsa::TruePushya),
        "TrueMula" => Some(Ayanamsa::TrueMula),
        "TrueSheoran" => Some(Ayanamsa::TrueSheoran),
        "GalacticCenter" => Some(Ayanamsa::GalacticCenter),
        "GalacticCenterRgilbrand" => Some(Ayanamsa::GalacticCenterRgilbrand),
        "GalacticEquatorIau1958" => Some(Ayanamsa::GalacticEquatorIau1958),
        "GalacticEquatorTrue" => Some(Ayanamsa::GalacticEquatorTrue),
        "GalacticEquatorMula" => Some(Ayanamsa::GalacticEquatorMula),
        "GalacticCenterMardyks" => Some(Ayanamsa::GalacticCenterMardyks),
        "GalacticCenterMulaWilhelm" => Some(Ayanamsa::GalacticCenterMulaWilhelm),
        "GalacticCenterCochrane" => Some(Ayanamsa::GalacticCenterCochrane),
        "GalacticEquatorFiorenza" => Some(Ayanamsa::GalacticEquatorFiorenza),
```

Append the same code strings to the completeness `for code in [ … ]` array, and update its `// Completeness: all 23 gated modes present.` comment to the new count.

- [ ] **Step 3: Bump the manifest checksum (use the gate to read it)**

Run: `cargo test -p pleiades-validate checksum_drift_fails_closed -- --nocapture`
Expected: the assertion fails with the recomputed `fnv1a64(CORPUS_CSV)` value visible (or run `gate_passes_over_committed_corpus`, which returns `ChecksumMismatch { expected, actual }` — `actual` is the value to use). Paste `actual` into `manifest.txt`'s `checksum=` field. Do **not** compute it with any external FNV tool (non-canonical prime).

- [ ] **Step 4: Update the gate's count test**

In `ayanamsa_validation.rs` tests, change `assert_eq!(report.modes_checked, 23);` to the new total (`23 + N_promoted`).

- [ ] **Step 5: Run the gate to verify it passes**

Run: `cargo test -p pleiades-validate ayanamsa`
Expected: PASS — `gate_passes_over_committed_corpus`, `checksum_drift_fails_closed`, and the ceiling/parse tests all green over the expanded corpus.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv crates/pleiades-validate/data/ayanamsa-corpus/manifest.txt crates/pleiades-validate/src/ayanamsa_validation.rs
git commit -m "feat(validate): commit fitted-family corpus rows + gate promoted modes"
```

---

## Task 5: Promote the catalog claim tiers

**Files:**
- Modify: `crates/pleiades-ayanamsa/src/catalog.rs`
- Modify: `crates/pleiades-ayanamsa/src/catalog/tests.rs`

**Interfaces:**
- Consumes: the PASS set from Task 1.
- Produces: each promoted mode's descriptor reports `claim_tier == ReleaseGradeNumeric` (in both the source arrays and `BUILT_IN_AYANAMSAS`).

- [ ] **Step 1: Write the failing test**

In `catalog/tests.rs`, add (the `descriptor` accessor is re-exported at the crate root as `crate::descriptor`; `pleiades_types` provides the tier enum):

```rust
    #[test]
    fn promoted_fitted_modes_are_release_grade() {
        use crate::descriptor;
        use pleiades_types::{Ayanamsa, CompatibilityClaimTier};
        use pleiades_types::CompatibilityClaimTier::ReleaseGradeNumeric;
        for m in [
            Ayanamsa::TrueRevati, Ayanamsa::TruePushya, Ayanamsa::TrueMula, Ayanamsa::TrueSheoran,
            Ayanamsa::GalacticCenter, Ayanamsa::GalacticCenterRgilbrand,
            Ayanamsa::GalacticEquatorIau1958, Ayanamsa::GalacticEquatorTrue,
            Ayanamsa::GalacticEquatorMula, Ayanamsa::GalacticCenterMardyks,
            Ayanamsa::GalacticCenterMulaWilhelm, Ayanamsa::GalacticCenterCochrane,
            Ayanamsa::GalacticEquatorFiorenza,
        ] {
            let d = descriptor(&m).expect("descriptor exists");
            assert_eq!(d.claim_tier, ReleaseGradeNumeric, "{m:?}");
        }
        // Deferred: stay descriptor-only.
        assert_eq!(
            descriptor(&Ayanamsa::DhruvaGalacticCenterMula).unwrap().claim_tier,
            CompatibilityClaimTier::DescriptorOnly
        );
    }
```

(Match this list to Task 1's PASS set.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-ayanamsa promoted_fitted_modes_are_release_grade`
Expected: FAIL — descriptors are still `DescriptorOnly`.

- [ ] **Step 3: Flip the constructor for each promoted mode (both arrays)**

For each promoted `Ayanamsa` variant, change its descriptor constructor from `AyanamsaDescriptor::new(` to `AyanamsaDescriptor::new_release_grade(`. Each variant appears **twice** — once in `BASELINE_AYANAMSAS`/`RELEASE_AYANAMSAS` (lines ~7–529) and once in `BUILT_IN_AYANAMSAS` (lines ~530+). Flip both. Locate them with:

```bash
grep -nE "AyanamsaDescriptor::new\(\s*$" -A1 crates/pleiades-ayanamsa/src/catalog.rs | grep -E "TrueRevati|TruePushya|TrueMula|TrueSheoran|GalacticCenter|GalacticEquator"
```

Leave `DhruvaGalacticCenterMula` and the legacy `GalacticEquator` mode on `new(` (they stay `DescriptorOnly`).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-ayanamsa`
Expected: PASS, including the new test and any existing release-grade-count test in `catalog/tests.rs`. If a count assertion fails, update it to the new total release-grade count.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-ayanamsa/src/catalog.rs crates/pleiades-ayanamsa/src/catalog/tests.rs
git commit -m "feat(ayanamsa): promote fitted family to release-grade in catalog"
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

**Interfaces:**
- Consumes: the new gated count (`23 + N_promoted`) and the shrunken deferred set.
- Produces: all four `compat-claims-audit` surfaces agree; README/PLAN prose matches; full workspace test suite + release gate green.

- [ ] **Step 1: Run the suite to surface every count/golden that moved**

Run: `cargo test --workspace 2>&1 | tee /tmp/claude-1000/-workspace/5c5ddaa2-798b-43fe-8084-e4c243bd4e38/scratchpad/slice2-fails.txt`
Expected: failures only in count/golden assertions in the four test files above (release-grade ayanamsa count, rendered catalog snapshots, CLI summary goldens). Read each failure's expected-vs-actual.

- [ ] **Step 2: Update each failing assertion to the new measured value**

For each failure, update the asserted count/golden to the value the gate/render now produces (these are the audit-derived truths, not free choices). Do not weaken any assertion — only update the number/string to the new correct value.

- [ ] **Step 3: Update README and PLAN prose**

In `README.md` line 20, change `23 release-claimed ayanamsa modes pass theirs` to the new count and adjust the trailing "rest are catalogued with metadata only" phrasing.

In `PLAN.md`, update the Phase 6 ayanamsa note (~line 81) and the Status line (~line 172): new gated count = `6 original + 17 offset-defined + N_promoted fitted`; record slice 2 as done; list the still-deferred set (failed-offset, anchorless, observational Babylonians, plus `DhruvaGalacticCenterMula`/legacy `GalacticEquator` for lack of a distinct SE code).

- [ ] **Step 4: Run the full workspace suite**

Run: `cargo test --workspace`
Expected: PASS (entire workspace).

- [ ] **Step 5: Run the release gate and claims audit**

Run: `cargo run -p pleiades-cli -- release-gate` and `cargo run -p pleiades-cli -- compat-claims-audit`
(Use the actual subcommand names from `cargo run -p pleiades-cli -- --help` if they differ.)
Expected: release gate green; claims audit reports bidirectional agreement across catalog / evidence / profile / prose with no overclaim and no missing evidence.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-validate/src/tests/compatibility.rs crates/pleiades-validate/src/tests/release_checklist.rs crates/pleiades-validate/src/tests/render_catalog.rs crates/pleiades-cli/src/cli/tests/summary_commands.rs README.md PLAN.md
git commit -m "docs+test: align claims surfaces to fitted-family promotion (slice 2)"
```

---

## Notes for the implementer

- **Run the SE tool first (Task 1) and let its output drive everything.** The PASS/DEFER membership, the cubic coefficients, the ceilings, the corpus rows, and every "new count" are tool/gate outputs. Wherever a step says "paste the printed …" or "the value the gate reports," use that captured output — never invent a number.
- **If a mode defers,** remove it consistently from: the corpus `MODES` (Task 1), the class mapping and tests (Task 2), the evaluator `match` and tests (Task 3), `mode_for_code`/completeness (Task 4), and the catalog flip + tests (Task 5). Record it with its measured residual (or "no distinct SE_SIDM") in the Task 1 notes file. It must remain `DescriptorOnly` everywhere.
- **The compatibility profile** needs no hand-edited mode list — the `compat-claims-audit` derives the claimed set from catalog `claim_tier`. Task 6 Step 5 verifies the audit agrees; if it reports a profile mismatch, follow its message to the offending surface.
- **Compatibility-profile version bump is deliberately deferred** to the end of the full ayanamsa family (see the design doc §7). Do not bump it in this slice.
