# FU-9 slice 2 — `apparent.rs` mutant triage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive `crates/pleiades-apparent/src/apparent.rs` from 49 surviving cargo-mutants to 0 (or a documented, justified residual) by extracting two behavior-preserving combine primitives and adding white-box unit tests that express the file's numeric and provenance intent.

**Architecture:** `apparent.rs` is an *orchestrator* — it composes already-tested sub-corrections (light-time, precession, nutation Δψ, annual aberration) into an apparent ecliptic-of-date position with provenance. Two pure helpers (`combine_apparent`, `precession_shift_arcsec`) are extracted from the three near-identical public functions so the combine/wrap/guard mutant surface is defined and tested once. Tests use an *independent-recomposition* reference (crafted inputs, or the sub-correction functions invoked independently), never the orchestrator's own output. The refactor lands as its own commit before the test additions.

**Tech Stack:** Rust (stable, workspace toolchain via `mise`), `cargo-nextest`, `cargo-mutants` 27.1.0, all managed through `mise.toml`.

## Global Constraints

- All first-party crates use the `pleiades-*` prefix; pure Rust, no C/C++ deps.
- **No change to `apparent.rs` runtime behavior.** The refactor is a pure extraction; each public function must compute byte-identical results. (Spec §2, §5.)
- **No parity/release gate tolerance is changed** by this slice. The mutants tier stays **report-only** — mutation score gates nothing. (Spec §2, §7.)
- **No blanket `#[mutants::skip]`** on numeric survivors. Any genuinely-undrivable survivor is a *documented* residual with a one-line justification, not a suppression. (Spec §2, §8.)
- **Independence discipline:** every expected value is traceable to crafted inputs or to independently-invoked sub-correction functions, never copied from `apparent.rs`'s output. (Spec §4.0.)
- White-box unit tests stay white-box unit tests; do **not** convert them to black-box integration tests. (Spec §6.)
- Run validation through `mise` tasks: `mise run ci` (blocking tier), `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- The authoritative mutant command for this file is:
  `cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file src/apparent.rs`

---

## Reference: existing signatures this plan consumes

These already exist and are stable — the new helpers and tests call them:

```rust
// crates/pleiades-apparent/src/precession.rs
pub struct PrecessedEcliptic { pub longitude_deg: f64, pub latitude_deg: f64 }
pub fn precess_ecliptic_j2000_to_date(lambda_deg: f64, beta_deg: f64, jd_tt: f64)
    -> Result<PrecessedEcliptic, ApparentPlaceError>;

// crates/pleiades-apparent/src/aberration.rs
pub struct AberrationOffset { pub d_lambda_arcsec: f64, pub d_beta_arcsec: f64 }
pub fn annual_aberration(lambda_deg: f64, beta_deg: f64, sun_true_longitude_deg: f64, jd_tt: f64)
    -> AberrationOffset;

// crates/pleiades-apparent/src/nutation.rs
pub struct Nutation { pub delta_psi_arcsec: f64, pub delta_eps_arcsec: f64 }
pub fn nutation(jd_tt: f64) -> Result<Nutation, ApparentPlaceError>;

// crates/pleiades-apparent/src/error.rs
pub enum ApparentPlaceError { NonConvergentLightTime{..}, MissingDistance,
    NonFiniteCorrection { stage: &'static str }, StaleModelData{..} }
pub enum ApparentLightTimeError<E> { Query(E), Apparent(ApparentPlaceError) }
```

---

## Task 1: Extract the two combine primitives (behavior-preserving refactor)

Pure extraction only — no test additions in this task's *test suite* beyond confirming existing tests still pass. This is the "separate, clearly-labeled commit" the spec requires (§5).

**Files:**
- Modify: `crates/pleiades-apparent/src/apparent.rs` (add two private `fn`s; rewrite the three public functions' combine/wrap blocks to call them)

**Interfaces:**
- Consumes: `ApparentPlaceError::NonFiniteCorrection` (existing).
- Produces (private to the module, used by Task 2+ tests):
  - `fn combine_apparent(lambda_deg: f64, beta_deg: f64, d_lambda_arcsec: f64, d_beta_arcsec: f64, delta_psi_arcsec: f64, stage: &'static str) -> Result<(f64, f64), ApparentPlaceError>`
  - `fn precession_shift_arcsec(lambda_deg: f64, lambda_j2000_deg: f64) -> f64`

- [ ] **Step 1: Add the two private helpers**

Insert into `crates/pleiades-apparent/src/apparent.rs` (after the `use` block, before `apparent_position`):

```rust
/// Combines mean-of-date ecliptic (λ, β) in degrees with arcsecond corrections
/// into an apparent (longitude, latitude) pair in degrees, normalizing longitude
/// to `[0, 360)` and failing closed on non-finite output.
///
/// The apsis path (nutation only, no aberration) calls this with
/// `d_lambda_arcsec = d_beta_arcsec = 0.0`.
fn combine_apparent(
    lambda_deg: f64,
    beta_deg: f64,
    d_lambda_arcsec: f64,
    d_beta_arcsec: f64,
    delta_psi_arcsec: f64,
    stage: &'static str,
) -> Result<(f64, f64), ApparentPlaceError> {
    let apparent_lon =
        (lambda_deg + (d_lambda_arcsec + delta_psi_arcsec) / 3600.0).rem_euclid(360.0);
    let apparent_lat = beta_deg + d_beta_arcsec / 3600.0;
    if !apparent_lon.is_finite() || !apparent_lat.is_finite() {
        return Err(ApparentPlaceError::NonFiniteCorrection { stage });
    }
    Ok((apparent_lon, apparent_lat))
}

/// Longitude precession shift (λ − λ_J2000) for provenance, wrapped to
/// `(−180, 180]` degrees and returned in arcseconds.
fn precession_shift_arcsec(lambda_deg: f64, lambda_j2000_deg: f64) -> f64 {
    let mut shift = lambda_deg - lambda_j2000_deg;
    if shift > 180.0 {
        shift -= 360.0;
    } else if shift < -180.0 {
        shift += 360.0;
    }
    shift * 3600.0
}
```

- [ ] **Step 2: Rewrite `apparent_position` to use the helpers**

In `apparent_position`, replace the combine block (currently the `apparent_lon`/`apparent_lat`/`if !…is_finite()` block, ~lines 56–65) with:

```rust
    let (apparent_lon, apparent_lat) = combine_apparent(
        lambda,
        beta,
        aberration.d_lambda_arcsec,
        aberration.d_beta_arcsec,
        nut.delta_psi_arcsec,
        "apparent-combine",
    )
    .map_err(ApparentLightTimeError::Apparent)?;
```

Replace the precession-shift block (the `let mut precession_shift = …; if … else if …` block, ~lines 67–73) and its later use `precession_longitude_arcsec: precession_shift * 3600.0` with a single value:

```rust
    let precession_longitude_arcsec = precession_shift_arcsec(lambda, lambda_j2000);
```

and in the `ApparentProvenance { … }` literal set `precession_longitude_arcsec,` (shorthand).

- [ ] **Step 3: Rewrite `apparent_sun_position` to use the helpers**

Same two replacements, using `stage = "apparent-sun-combine"` and returning the error directly (this function's return type is `Result<_, ApparentPlaceError>`, so **no** `.map_err`):

```rust
    let (apparent_lon, apparent_lat) = combine_apparent(
        lambda,
        beta,
        aberration.d_lambda_arcsec,
        aberration.d_beta_arcsec,
        nut.delta_psi_arcsec,
        "apparent-sun-combine",
    )?;
```

```rust
    let precession_longitude_arcsec = precession_shift_arcsec(lambda, lambda_j2000);
```

and `precession_longitude_arcsec,` (shorthand) in the provenance literal.

- [ ] **Step 4: Rewrite `apparent_apsis_position` to use the helpers**

The apsis path applies nutation only (no aberration), so pass `0.0` for both aberration terms and `stage = "apparent-apsis-combine"`:

```rust
    let (apparent_lon, apparent_lat) = combine_apparent(
        lambda,
        beta,
        0.0,
        0.0,
        nut.delta_psi_arcsec,
        "apparent-apsis-combine",
    )?;
```

```rust
    let precession_longitude_arcsec = precession_shift_arcsec(lambda, lambda_j2000);
```

and `precession_longitude_arcsec,` (shorthand) in the provenance literal. The apsis provenance keeps `aberration_longitude_arcsec: 0.0`.

- [ ] **Step 5: Run the existing tests to prove behavior is unchanged**

Run: `cargo nextest run -p pleiades-apparent`
Expected: PASS — all existing `apparent.rs` tests (`at_j2000_only_aberration_and_nutation_shift_longitude`, `precession_dominates_far_from_j2000`, `latitude_moves_by_precession_and_aberration_only`, `sun_applies_aberration_once_no_light_time_requery`, `apsis_position_is_precession_and_nutation_only_no_aberration`) still pass with no edits — they are the behavior-preservation guard.

- [ ] **Step 6: Formatting and lint**

Run: `cargo fmt --all && cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings`
Expected: no diff from fmt beyond the edited region; clippy clean.

- [ ] **Step 7: Commit (labeled as a pure refactor)**

```bash
git add crates/pleiades-apparent/src/apparent.rs
git commit -m "refactor(apparent): extract combine_apparent + precession_shift_arcsec

Behavior-preserving extraction of the combine formula, longitude
normalization, non-finite guard, and precession-shift wrap from the three
near-identical apparent-position functions into two pure private helpers.
No runtime result changes; existing tests unchanged. Prepares the FU-9
apparent.rs mutant-triage surface (spec 2026-07-19-fu9-apparent-mutant-triage)."
```

---

## Task 2: Regenerate the authoritative post-refactor survivor list

No code change — this produces the classification input for Tasks 3–5.

**Files:** none modified.

- [ ] **Step 1: Run the file-scoped mutant sweep**

Run:
```bash
cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file src/apparent.rs 2>&1 | tee /tmp/claude-1000/-workspace/0737dec2-9587-429f-a08e-60368f39f524/scratchpad/apparent-missed-pre.txt
```
Expected: exit code 2 (survivors found). The run prints a `MISSED` line per surviving mutant. cargo-mutants also writes `mutants.out/missed.txt`.

- [ ] **Step 2: Capture the survivor list for classification**

Run: `cp mutants.out/missed.txt /tmp/claude-1000/-workspace/0737dec2-9587-429f-a08e-60368f39f524/scratchpad/apparent-missed-pre.list`
Expected: a file with one `src/apparent.rs:LINE:COL: replace … in …` entry per survivor. Read it and map each entry to an archetype (A–F, spec §4). This list is the checklist Tasks 3–5 must drive to zero; do not delete it until Task 6 confirms 0 survivors.

Note: `mutants.out/` is a tool artifact — do **not** commit it. It is covered by the workspace's ignore rules; if `git status` shows it, add `mutants.out/` to `.gitignore` in this task's follow-up rather than committing the directory.

---

## Task 3: White-box tests for the two primitives (archetypes A, B, C, D)

Create the co-located test file and cover the pure helpers directly — this is where the bulk of the 49 survivors die, at `1e-9`-scale tolerances the pure functions permit.

**Files:**
- Create: `crates/pleiades-apparent/src/apparent/tests.rs`
- Modify: `crates/pleiades-apparent/src/apparent.rs` (remove inline `#[cfg(test)] mod tests { … }`; add module-path declaration)

**Interfaces:**
- Consumes: `combine_apparent`, `precession_shift_arcsec` (Task 1); `ApparentPlaceError` (existing).

- [ ] **Step 1: Relocate the existing test module to a co-located file**

In `crates/pleiades-apparent/src/apparent.rs`, delete the entire inline `#[cfg(test)] mod tests { … }` block (currently ~lines 243–386) and replace it with the module-path form (mirroring `nutation.rs:122-123`):

```rust

#[cfg(test)]
mod tests;
```

Create `crates/pleiades-apparent/src/apparent/tests.rs` and move the deleted block's *body* into it verbatim, starting with the imports the inline block used:

```rust
use super::*;
use pleiades_types::{JulianDay, TimeScale};

fn fixed(lon: f64, lat: f64, dist: f64) -> EclipticCoordinates {
    EclipticCoordinates::new(
        Longitude::from_degrees(lon),
        Latitude::from_degrees(lat),
        Some(dist),
    )
}

// ... the five existing #[test] fns, moved verbatim ...
```

- [ ] **Step 2: Run to confirm the relocation is behavior-neutral**

Run: `cargo nextest run -p pleiades-apparent`
Expected: PASS — the same five tests run from their new location.

- [ ] **Step 3: Write failing tests for `combine_apparent` (archetypes A, B, D)**

Append to `crates/pleiades-apparent/src/apparent/tests.rs`:

```rust
#[test]
fn combine_apparent_applies_all_terms_with_correct_scaling() {
    // Crafted, mutually-distinct arcsec terms so no term-swap aliases another,
    // and no wrap is triggered (result stays in-range). Expected computed by
    // hand: lon = 100 + (12 + 5)/3600 ; lat = 20 + (-9)/3600.
    let (lon, lat) = combine_apparent(100.0, 20.0, 12.0, -9.0, 5.0, "t").unwrap();
    assert!((lon - (100.0 + 17.0 / 3600.0)).abs() < 1e-9, "lon {lon}");
    assert!((lat - (20.0 - 9.0 / 3600.0)).abs() < 1e-9, "lat {lat}");
}

#[test]
fn combine_apparent_normalizes_longitude_above_360() {
    // λ + corrections crosses 360 -> must wrap into [0, 360).
    let (lon, _) = combine_apparent(359.999, 0.0, 7200.0, 0.0, 0.0, "t").unwrap();
    // 359.999 + 7200/3600 = 361.999 -> 1.999
    assert!((lon - 1.999).abs() < 1e-9, "lon {lon}");
    assert!((0.0..360.0).contains(&lon), "lon out of range: {lon}");
}

#[test]
fn combine_apparent_normalizes_longitude_below_zero() {
    // λ + corrections goes negative -> rem_euclid must return a positive angle.
    let (lon, _) = combine_apparent(0.001, 0.0, -7200.0, 0.0, 0.0, "t").unwrap();
    // 0.001 - 2.0 = -1.999 -> 358.001
    assert!((lon - 358.001).abs() < 1e-9, "lon {lon}");
    assert!((0.0..360.0).contains(&lon), "lon out of range: {lon}");
}

#[test]
fn combine_apparent_fails_closed_on_non_finite_with_stage() {
    let err = combine_apparent(f64::NAN, 0.0, 0.0, 0.0, 0.0, "stage-x").unwrap_err();
    assert!(
        matches!(
            err,
            ApparentPlaceError::NonFiniteCorrection { stage: "stage-x" }
        ),
        "unexpected error: {err:?}"
    );
}
```

- [ ] **Step 4: Run to verify they pass against the real implementation**

Run: `cargo nextest run -p pleiades-apparent combine_apparent`
Expected: PASS (Task 1's helper is already correct; these tests exist to *kill mutants*, so they pass now and fail under mutation).

- [ ] **Step 5: Write tests for `precession_shift_arcsec` (archetype C — both wrap branches)**

Append:

```rust
#[test]
fn precession_shift_no_wrap_midrange() {
    // Small positive shift, neither branch taken: (100.5 - 100.0) * 3600.
    let s = precession_shift_arcsec(100.5, 100.0);
    assert!((s - 0.5 * 3600.0).abs() < 1e-6, "shift {s}");
}

#[test]
fn precession_shift_wraps_large_positive_raw() {
    // Raw shift 359.9 - 0.1 = 359.8 (> 180) -> -0.2 deg -> -720".
    let s = precession_shift_arcsec(359.9, 0.1);
    assert!((s - (-0.2 * 3600.0)).abs() < 1e-6, "shift {s}");
}

#[test]
fn precession_shift_wraps_large_negative_raw() {
    // Raw shift 0.1 - 359.9 = -359.8 (< -180) -> +0.2 deg -> +720".
    let s = precession_shift_arcsec(0.1, 359.9);
    assert!((s - (0.2 * 3600.0)).abs() < 1e-6, "shift {s}");
}
```

- [ ] **Step 6: Run to verify**

Run: `cargo nextest run -p pleiades-apparent precession_shift`
Expected: PASS.

- [ ] **Step 7: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings
git add crates/pleiades-apparent/src/apparent.rs crates/pleiades-apparent/src/apparent/tests.rs
git commit -m "test(apparent): white-box tests for combine_apparent + precession_shift_arcsec

Relocate the apparent tests to a co-located tests.rs and add direct unit
tests for the two extracted primitives: term application/scaling, longitude
normalization above 360 and below 0, fail-closed non-finite guard with stage,
and both precession-shift wrap branches (FU-9 archetypes A-D)."
```

---

## Task 4: Full-provenance assertions per public function (archetype E)

Each of the three functions assembles a distinct `ApparentProvenance`; asserting the *whole* struct per function kills field-swap and boolean-flip mutants.

**Files:**
- Modify: `crates/pleiades-apparent/src/apparent/tests.rs`

**Interfaces:**
- Consumes: `apparent_position`, `apparent_sun_position`, `apparent_apsis_position`, `combine_apparent`, `precession_shift_arcsec`, and the sub-correction functions (`precess_ecliptic_j2000_to_date`, `annual_aberration`, `nutation`) — all invoked independently to build the reference.

- [ ] **Step 1: Write a full-provenance test for `apparent_position`**

Append to `tests.rs`. The reference recomputes every provenance field independently from the sub-correction functions (spec §4.0):

```rust
#[test]
fn apparent_position_provenance_is_fully_specified() {
    let jd = 2_451_545.0 + 36_525.0; // one century from J2000, precession resolvable
    let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
    let (l0, b0) = (100.0, 5.0);
    let out = apparent_position::<_, ApparentPlaceError>(instant, 280.0, 8, |_| {
        Ok(fixed(l0, b0, 1.0))
    })
    .unwrap();

    // Independent recomposition of the corrections.
    let p = crate::precession::precess_ecliptic_j2000_to_date(l0, b0, jd).unwrap();
    let ab = crate::aberration::annual_aberration(p.longitude_deg, p.latitude_deg, 280.0, jd);
    let nut = crate::nutation::nutation(jd).unwrap();

    assert!((out.provenance.nutation_longitude_arcsec - nut.delta_psi_arcsec).abs() < 1e-12);
    assert!((out.provenance.aberration_longitude_arcsec - ab.d_lambda_arcsec).abs() < 1e-12);
    assert!(
        (out.provenance.precession_longitude_arcsec
            - precession_shift_arcsec(p.longitude_deg, l0))
        .abs()
            < 1e-9
    );
    assert!(out.provenance.light_time_days > 0.0, "light-time must be applied");
    assert!(out.provenance.iterations >= 1);
    assert_eq!(out.ecliptic.distance_au, Some(1.0), "distance passes through");
    // CorrectionSet — every flag pinned.
    assert!(out.provenance.corrections.light_time);
    assert!(out.provenance.corrections.precession);
    assert!(out.provenance.corrections.annual_aberration);
    assert!(out.provenance.corrections.nutation_longitude);
    assert!(!out.provenance.corrections.diurnal_parallax);
    assert!(!out.provenance.corrections.diurnal_aberration);
}
```

- [ ] **Step 2: Write a full-provenance test for `apparent_sun_position`**

```rust
#[test]
fn apparent_sun_position_provenance_is_fully_specified() {
    let jd = 2_451_545.0;
    let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
    let sun_j2000 = fixed(280.0, 0.0, 0.983);
    let out = apparent_sun_position(instant, sun_j2000).unwrap();

    let p = crate::precession::precess_ecliptic_j2000_to_date(280.0, 0.0, jd).unwrap();
    // Sun is its own aberration argument: ⊙ = λ.
    let ab = crate::aberration::annual_aberration(p.longitude_deg, p.latitude_deg, p.longitude_deg, jd);
    let nut = crate::nutation::nutation(jd).unwrap();

    assert!((out.provenance.nutation_longitude_arcsec - nut.delta_psi_arcsec).abs() < 1e-12);
    assert!((out.provenance.aberration_longitude_arcsec - ab.d_lambda_arcsec).abs() < 1e-12);
    assert_eq!(out.provenance.light_time_days, 0.0, "Sun applies no light-time");
    assert_eq!(out.provenance.iterations, 0);
    assert_eq!(out.ecliptic.distance_au, Some(0.983));
    // CorrectionSet — light_time false, the Sun-specific difference.
    assert!(!out.provenance.corrections.light_time);
    assert!(out.provenance.corrections.precession);
    assert!(out.provenance.corrections.annual_aberration);
    assert!(out.provenance.corrections.nutation_longitude);
    assert!(!out.provenance.corrections.diurnal_parallax);
    assert!(!out.provenance.corrections.diurnal_aberration);
}
```

- [ ] **Step 3: Write a full-provenance test for `apparent_apsis_position`**

```rust
#[test]
fn apparent_apsis_position_provenance_is_fully_specified() {
    let jd = 2_451_545.0;
    let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
    let j2000 = fixed(100.0, 5.0, 0.0025);
    let out = apparent_apsis_position(instant, j2000).unwrap();

    let nut = crate::nutation::nutation(jd).unwrap();
    assert!((out.provenance.nutation_longitude_arcsec - nut.delta_psi_arcsec).abs() < 1e-12);
    // Apse line carries NO aberration term.
    assert_eq!(out.provenance.aberration_longitude_arcsec, 0.0);
    assert_eq!(out.provenance.light_time_days, 0.0);
    assert_eq!(out.provenance.iterations, 0);
    assert_eq!(out.ecliptic.distance_au, Some(0.0025));
    // CorrectionSet — annual_aberration false, the apsis-specific difference.
    assert!(!out.provenance.corrections.light_time);
    assert!(out.provenance.corrections.precession);
    assert!(!out.provenance.corrections.annual_aberration);
    assert!(out.provenance.corrections.nutation_longitude);
    assert!(!out.provenance.corrections.diurnal_parallax);
    assert!(!out.provenance.corrections.diurnal_aberration);
}
```

- [ ] **Step 4: Run the provenance tests**

Run: `cargo nextest run -p pleiades-apparent provenance_is_fully_specified`
Expected: PASS (3 tests).

- [ ] **Step 5: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings
git add crates/pleiades-apparent/src/apparent/tests.rs
git commit -m "test(apparent): full-provenance assertions per apparent function

Assert every ApparentProvenance field and CorrectionSet flag for
apparent_position / apparent_sun_position / apparent_apsis_position against an
independent recomposition, pinning the per-function differences (light_time,
annual_aberration) so field-swap and boolean-flip mutants are caught (FU-9
archetype E)."
```

---

## Task 5: Exact-recomposition equality + non-finite propagation (archetypes A, D, F) and residual sweep

Lock the end-to-end combine wiring per function and the `⊙ = λ` argument, then drive the survivor list to zero.

**Files:**
- Modify: `crates/pleiades-apparent/src/apparent/tests.rs`
- Modify (only if a residual is documented): `crates/pleiades-apparent/src/apparent.rs`

**Interfaces:**
- Consumes: all three public functions and the sub-correction functions (independently invoked).

- [ ] **Step 1: Exact-recomposition equality for `apparent_position` (archetype A/F)**

The existing `sun_applies_aberration_once_no_light_time_requery` test already does this for the Sun. Add the planet-path equivalent so `apparent_position`'s combine wiring is pinned at `< 1e-6″`:

```rust
#[test]
fn apparent_position_equals_independent_recomposition() {
    let jd = 2_451_545.0 + 36_525.0;
    let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
    let (l0, b0) = (100.0, 5.0);
    let sun = 280.0;
    let out = apparent_position::<_, ApparentPlaceError>(instant, sun, 8, |_| {
        Ok(fixed(l0, b0, 1.0))
    })
    .unwrap();

    let p = crate::precession::precess_ecliptic_j2000_to_date(l0, b0, jd).unwrap();
    let ab = crate::aberration::annual_aberration(p.longitude_deg, p.latitude_deg, sun, jd);
    let nut = crate::nutation::nutation(jd).unwrap();
    let (exp_lon, exp_lat) = combine_apparent(
        p.longitude_deg,
        p.latitude_deg,
        ab.d_lambda_arcsec,
        ab.d_beta_arcsec,
        nut.delta_psi_arcsec,
        "apparent-combine",
    )
    .unwrap();

    let dlon = (out.ecliptic.longitude.degrees() - exp_lon) * 3600.0;
    let dlat = (out.ecliptic.latitude.degrees() - exp_lat) * 3600.0;
    assert!(dlon.abs() < 1e-6, "lon off by {dlon}\"");
    assert!(dlat.abs() < 1e-6, "lat off by {dlat}\"");
}
```

- [ ] **Step 2: Non-finite propagation through each public function (archetype D — stage strings)**

`apparent_position` reaches its guard via a non-finite `query` result; the sun/apsis functions via a non-finite input coordinate. Assert the correct `stage` per function:

```rust
#[test]
fn apparent_position_propagates_non_finite_query() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let err = apparent_position::<_, ApparentPlaceError>(instant, 280.0, 8, |_| {
        Ok(fixed(f64::NAN, 0.0, 1.0))
    })
    .unwrap_err();
    assert!(
        matches!(
            err,
            ApparentLightTimeError::Apparent(ApparentPlaceError::NonFiniteCorrection {
                stage: "apparent-combine"
            })
        ),
        "unexpected error: {err:?}"
    );
}

#[test]
fn apparent_sun_position_propagates_non_finite_input() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let err = apparent_sun_position(instant, fixed(f64::NAN, 0.0, 0.983)).unwrap_err();
    assert!(
        matches!(
            err,
            ApparentPlaceError::NonFiniteCorrection { stage: "apparent-sun-combine" }
        ),
        "unexpected error: {err:?}"
    );
}

#[test]
fn apparent_apsis_position_propagates_non_finite_input() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let err = apparent_apsis_position(instant, fixed(f64::NAN, 0.0, 0.0025)).unwrap_err();
    assert!(
        matches!(
            err,
            ApparentPlaceError::NonFiniteCorrection { stage: "apparent-apsis-combine" }
        ),
        "unexpected error: {err:?}"
    );
}
```

Note: `precess_ecliptic_j2000_to_date` is called before `combine_apparent`. Confirm during Step 4 that a NaN input reaches the `combine_apparent` guard (stage `apparent-*-combine`) rather than being rejected earlier by precession. If precession rejects NaN first with a *different* error, adjust these three tests to assert whichever `NonFiniteCorrection` stage is actually produced (the goal is to pin the fail-closed behavior, not a specific stage that the code doesn't produce), and note it in the commit message.

- [ ] **Step 3: Run all new tests**

Run: `cargo nextest run -p pleiades-apparent`
Expected: PASS (all existing + all new tests).

- [ ] **Step 4: Regenerate the survivor list and drive to zero**

Run:
```bash
cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file src/apparent.rs 2>&1 | tee /tmp/claude-1000/-workspace/0737dec2-9587-429f-a08e-60368f39f524/scratchpad/apparent-missed-post.txt
```
Expected: **0 surviving mutants** (exit 0), OR a short residual list.

For each *remaining* survivor: classify it (§4 archetypes) and add the missing assertion to `tests.rs`, then re-run this step. The known candidate documented-residual is the `DEFAULT_MAX_ITERATIONS` default-const value (`pub const DEFAULT_MAX_ITERATIONS: u8 = 8;`): the three public functions take `max_iterations` as a parameter, so the const is only a caller default and no test here reaches it. If — and only if — it survives and no honest reachable test distinguishes it, leave it as a **documented residual** (recorded in Task 6's follow-up update), NOT a `#[mutants::skip]` and NOT a `assert_eq!(DEFAULT_MAX_ITERATIONS, 8)` change-detector.

- [ ] **Step 5: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings
git add crates/pleiades-apparent/src/apparent/tests.rs
git commit -m "test(apparent): exact-recomposition equality + non-finite propagation

Pin apparent_position's end-to-end combine wiring against an independent
recomposition, and assert each public function fails closed with its own
NonFiniteCorrection stage. Drives the apparent.rs file-scoped mutant sweep to
0 survivors (or documented residual) (FU-9 archetypes A/D/F)."
```

---

## Task 6: Blocking-tier gate + FU-9 follow-up update

**Files:**
- Modify: `docs/follow-ups.md` (FU-9 entry)

- [ ] **Step 1: Run the full blocking tier**

Run: `mise run ci`
Expected: PASS. If it fails, fix the cause before proceeding — do not weaken any gate.

- [ ] **Step 2: Confirm fmt + clippy across the workspace**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: both clean.

- [ ] **Step 3: Update the FU-9 entry in `docs/follow-ups.md`**

Append a progress paragraph to FU-9 (mirroring the existing `Progress (2026-07-19) — nutation.rs` note), recording: `apparent.rs` triaged from 49 → the post-Task-5 survivor count; the behavior-preserving `combine_apparent`/`precession_shift_arcsec` extraction; the orchestrator reference strategy (independent recomposition); any documented residual (e.g. `DEFAULT_MAX_ITERATIONS` default-const, with its justification) or "0 survivors"; and the remaining-slices list updated to drop `apparent.rs` (next: `refraction.rs` (37)). Use the actual measured post count from Task 5 Step 4 — do not guess.

- [ ] **Step 4: Commit**

```bash
git add docs/follow-ups.md
git commit -m "docs(follow-ups): record FU-9 apparent.rs triage (49 -> N) and method reuse"
```

- [ ] **Step 5: Final verification snapshot**

Run: `git log --oneline -5`
Expected: the refactor commit, the three test commits, and the docs commit, in order. Confirm `mutants.out/` is not staged in any of them.

---

## Self-review notes (author)

- **Spec coverage:** §2 refactor → Task 1; §3 regenerate → Tasks 2 & 5.4; §4.1–4.6 archetypes A–F → Tasks 3 (A/B/C/D primitives), 4 (E), 5 (A/D/F end-to-end); §5 primitives → Task 1; §6 relocation → Task 3.1; §7 acceptance (`--file` 0 survivors, `mise run ci`, fmt/clippy, no gate change, separate commits, FU-9 update) → Tasks 5.4 & 6; §8 residual policy → Task 5.4; §10 remaining slices → Task 6.3.
- **No parity gate touched** anywhere — only `apparent/tests.rs`, the two private helpers, and `docs/follow-ups.md` change.
- **Type consistency:** helper names `combine_apparent` / `precession_shift_arcsec` and stage strings `apparent-combine` / `apparent-sun-combine` / `apparent-apsis-combine` are used identically across Tasks 1, 3, 4, 5.
