# FU-5 Angles & Sidereal-Time Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the three open FU-5 items — single-source the GMST + equation-of-equinoxes math, add a southern-hemisphere `validate-angles` gate row, and assert the Porphyry high-latitude fallback `asc_mc` site — with no intended behavior change.

**Architecture:** `pleiades-time` becomes the single source of the GMST polynomial (exposing an unnormalized `gmst_degrees_raw` plus the existing normalized `gmst_degrees`); `pleiades-apparent` delegates to it and gains a shared `equation_of_equinoxes(delta_psi_deg, true_obliquity_deg)` helper that `pleiades-core`'s chart path also calls. Two test-only additions cover the southern angles branch (via a regenerated Swiss-Ephemeris corpus fixture) and the fallback `asc_mc` construction site.

**Tech Stack:** Rust (workspace crates `pleiades-time`, `pleiades-apparent`, `pleiades-core`, `pleiades-houses`, `pleiades-validate`), Swiss Ephemeris reference tool `tools/se-house-reference` (FFI via `libswisseph-sys`), FNV-1a-64 corpus checksums.

## Global Constraints

- **No behavior change.** `greenwich_mean_sidereal_time_degrees` MUST keep returning the **unnormalized** value it returns today (it is a documented stable public convenience). All gates stay green within their existing ceilings (`validate-angles`: ARMC/GAST 1.0″, geometry points 0.5″).
- **Build-env for the reference tool only:** `tools/se-house-reference` needs `LIBCLANG_PATH=/lib/x86_64-linux-gnu` to build (`libclang-19` is present there). This is NOT required to run any gate or build the workspace — gates read committed CSVs via `include_str!`.
- **Stable public API:** `pleiades_apparent::sidereal::{greenwich_mean_sidereal_time_degrees, equation_of_equinoxes_degrees, sidereal_time}` are stable. Additions are allowed; signature/semantic changes to existing stable fns are not.
- **Out of scope (stays for SP-2):** migrating `pleiades-core/chart/mod.rs`'s topocentric-LAST path onto the public `sidereal_time` API (that path converts TT→UT1 first; the public API consumes JD as-supplied — a time-scale contract change).
- **Corpus discipline:** if regenerating the Swiss-Ephemeris corpus changes any *existing* row (not just adds the new fixture's rows), STOP — that signals SE-version drift; surface it as a finding, do not commit.
- **Tooling:** `cargo fmt --all` and `cargo clippy --workspace` clean before each commit.

---

## File Structure

- `crates/pleiades-time/src/sidereal.rs` — MODIFY: add unnormalized `gmst_degrees_raw`; `gmst_degrees` reduces it. Single source of the GMST polynomial.
- `crates/pleiades-apparent/Cargo.toml` — MODIFY: add `pleiades-time` dependency.
- `crates/pleiades-apparent/src/sidereal.rs` — MODIFY: delegate GMST to `pleiades-time`; add `equation_of_equinoxes` helper; rewrite `equation_of_equinoxes_degrees` to call it; add tests.
- `crates/pleiades-apparent/src/lib.rs` — MODIFY: export `equation_of_equinoxes`.
- `crates/pleiades-core/src/chart/mod.rs` — MODIFY (~line 411): call the shared `equation_of_equinoxes` helper instead of hand-inlining `cos(ε)`.
- `crates/pleiades-houses/src/systems/tests.rs` — MODIFY: add Porphyry-fallback `asc_mc` consistency test.
- `tools/se-house-reference/src/main.rs` — MODIFY: add one southern fixture; update the `5 fixtures × 23 systems` comment.
- `crates/pleiades-validate/data/houses-corpus/{cusps.csv,sectors.csv,angles.csv}` — REGENERATE: append the southern fixture's rows.
- `crates/pleiades-validate/data/houses-corpus/manifest.txt` — MODIFY: bump `rows=` and `checksum=` for all three slices.

---

## Task 1: Single-source the GMST polynomial (delegate apparent → pleiades-time)

**Files:**
- Modify: `crates/pleiades-time/src/sidereal.rs`
- Modify: `crates/pleiades-apparent/Cargo.toml`
- Modify: `crates/pleiades-apparent/src/sidereal.rs`
- Test: `crates/pleiades-apparent/src/sidereal.rs` (inline `#[cfg(test)] mod tests`)

**Interfaces:**
- Produces: `pleiades_time::gmst_degrees_raw(jd_ut1: f64) -> f64` (unnormalized GMST polynomial); `pleiades_time::gmst_degrees(jd_ut1: f64) -> f64` (unchanged signature, now `= gmst_degrees_raw(jd).rem_euclid(360.0)`).
- Consumes: nothing new.

- [ ] **Step 1: Write the failing cross-crate agreement test**

Add to the `mod tests` block in `crates/pleiades-apparent/src/sidereal.rs`:

```rust
#[test]
fn apparent_gmst_matches_pleiades_time_source() {
    for jd in [2_415_020.5_f64, 2_433_283.0, 2_451_545.0, 2_469_807.0, 2_488_069.5] {
        let apparent = greenwich_mean_sidereal_time_degrees(jd);
        // Un-normalized apparent value must equal the pleiades-time source exactly.
        assert_eq!(apparent, pleiades_time::gmst_degrees_raw(jd), "raw jd {jd}");
        // Reducing to [0,360) must match the normalized public fn.
        assert!(
            (apparent.rem_euclid(360.0) - pleiades_time::gmst_degrees(jd)).abs() < 1e-9,
            "normalized jd {jd}"
        );
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-apparent apparent_gmst_matches_pleiades_time_source`
Expected: FAIL to compile — `pleiades_time` is not a dependency and `gmst_degrees_raw` does not exist.

- [ ] **Step 3: Add the unnormalized source in pleiades-time**

In `crates/pleiades-time/src/sidereal.rs`, replace the body of `gmst_degrees` and add the raw fn:

```rust
/// Greenwich mean sidereal time in degrees, **unnormalized** — the raw IAU-1982
/// (Meeus eq. 12.4) polynomial, which may lie outside `[0, 360)`. This is the
/// single source of the GMST coefficients; `gmst_degrees` reduces it to `[0,360)`.
pub fn gmst_degrees_raw(jd_ut1: f64) -> f64 {
    let t = (jd_ut1 - 2_451_545.0) / 36_525.0;
    280.460_618_37 + 360.985_647_366_29 * (jd_ut1 - 2_451_545.0) + 0.000_387_933 * t * t
        - (t * t * t) / 38_710_000.0
}

/// Greenwich mean sidereal time in degrees, normalized to `[0, 360)`.
///
/// `jd_ut1` is the Julian day in the UT1 time scale. Formula: Meeus,
/// *Astronomical Algorithms*, eq. 12.4.
pub fn gmst_degrees(jd_ut1: f64) -> f64 {
    gmst_degrees_raw(jd_ut1).rem_euclid(360.0)
}
```

- [ ] **Step 4: Add the pleiades-time dependency to apparent**

In `crates/pleiades-apparent/Cargo.toml`, under `[dependencies]`, add below the `pleiades-types` line:

```toml
pleiades-time = { workspace = true }
```

- [ ] **Step 5: Delegate apparent's GMST to the single source**

In `crates/pleiades-apparent/src/sidereal.rs`, replace the `greenwich_mean_sidereal_time_degrees` body:

```rust
/// Greenwich Mean Sidereal Time in degrees (unnormalized).
///
/// Delegates to `pleiades_time::gmst_degrees_raw` — the single source of the
/// GMST polynomial — so the coefficients live in exactly one crate. Still
/// returns the unnormalized value; `sidereal_time` normalizes downstream.
pub fn greenwich_mean_sidereal_time_degrees(jd: f64) -> f64 {
    pleiades_time::gmst_degrees_raw(jd)
}
```

- [ ] **Step 6: Run the agreement test and the existing sidereal tests**

Run: `cargo test -p pleiades-apparent sidereal`
Expected: PASS — `apparent_gmst_matches_pleiades_time_source`, `gmst_at_j2000_is_about_280_46_degrees`, and the other sidereal tests all pass.

- [ ] **Step 7: Run the pleiades-time tests**

Run: `cargo test -p pleiades-time sidereal`
Expected: PASS — `gmst_matches_meeus_example_12a` and `gmst_is_normalized` still pass (behavior of `gmst_degrees` unchanged).

- [ ] **Step 8: Commit**

```bash
cargo fmt --all && cargo clippy -p pleiades-time -p pleiades-apparent --all-targets
git add crates/pleiades-time/src/sidereal.rs crates/pleiades-apparent/Cargo.toml crates/pleiades-apparent/src/sidereal.rs
git commit -m "refactor(sidereal): single-source GMST polynomial in pleiades-time

apparent's greenwich_mean_sidereal_time_degrees now delegates to a new
pleiades_time::gmst_degrees_raw (unnormalized), removing the byte-identical
duplicated polynomial. Cross-crate agreement test guards against re-divergence.
Closes FU-5 item 1 (GMST half)."
```

---

## Task 2: Single-source the equation-of-equinoxes formula

**Files:**
- Modify: `crates/pleiades-apparent/src/sidereal.rs`
- Modify: `crates/pleiades-apparent/src/lib.rs`
- Modify: `crates/pleiades-core/src/chart/mod.rs` (~line 411)
- Test: `crates/pleiades-apparent/src/sidereal.rs` (inline tests)

**Interfaces:**
- Produces: `pleiades_apparent::sidereal::equation_of_equinoxes(delta_psi_deg: f64, true_obliquity_deg: f64) -> f64` (also re-exported as `pleiades_apparent::equation_of_equinoxes`).
- Consumes: nothing new.

- [ ] **Step 1: Write the failing helper test**

Add to the `mod tests` block in `crates/pleiades-apparent/src/sidereal.rs`:

```rust
#[test]
fn equation_of_equinoxes_helper_matches_formula() {
    let delta_psi_deg = 0.001_234;
    let true_obliquity_deg = 23.44;
    let expected = delta_psi_deg * true_obliquity_deg.to_radians().cos();
    assert!((equation_of_equinoxes(delta_psi_deg, true_obliquity_deg) - expected).abs() < 1e-15);
}

#[test]
fn equation_of_equinoxes_degrees_uses_shared_helper() {
    // The jd-driven wrapper must equal the helper fed the same nutation inputs.
    let jd = 2_451_545.0;
    let n = crate::nutation::nutation(jd).expect("nutation table available in tests");
    let delta_psi_deg = n.delta_psi_arcsec / 3600.0;
    let true_obl_deg = crate::nutation::mean_obliquity_degrees(jd) + n.delta_eps_arcsec / 3600.0;
    assert!(
        (equation_of_equinoxes_degrees(jd) - equation_of_equinoxes(delta_psi_deg, true_obl_deg))
            .abs()
            < 1e-15
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-apparent equation_of_equinoxes`
Expected: FAIL to compile — `equation_of_equinoxes` (two-argument helper) does not exist.

- [ ] **Step 3: Add the shared helper and route the wrapper through it**

In `crates/pleiades-apparent/src/sidereal.rs`, add the helper above `equation_of_equinoxes_degrees` and rewrite the wrapper to call it:

```rust
/// Equation of the equinoxes in degrees from its two inputs: `Δψ · cos(ε_true)`.
///
/// `delta_psi_deg` is nutation-in-longitude and `true_obliquity_deg` the true
/// obliquity, both in degrees. Single source of the equation-of-equinoxes
/// formula, shared by `equation_of_equinoxes_degrees` and the chart layer's
/// topocentric-LAST path (`pleiades-core`).
pub fn equation_of_equinoxes(delta_psi_deg: f64, true_obliquity_deg: f64) -> f64 {
    delta_psi_deg * true_obliquity_deg.to_radians().cos()
}

/// Equation of the equinoxes in degrees: `Δψ · cos(ε_true)`.
///
/// Falls back to `0.0` if the nutation table is unavailable (a development-time
/// artifact — a stale checksum — not a runtime condition), matching the prior
/// behavior in `pleiades-houses`.
pub fn equation_of_equinoxes_degrees(jd: f64) -> f64 {
    nutation(jd)
        .map(|n| {
            let delta_psi_deg = n.delta_psi_arcsec / 3600.0;
            let true_obl_deg = mean_obliquity_degrees(jd) + n.delta_eps_arcsec / 3600.0;
            equation_of_equinoxes(delta_psi_deg, true_obl_deg)
        })
        .unwrap_or(0.0)
}
```

- [ ] **Step 4: Export the helper**

In `crates/pleiades-apparent/src/lib.rs`, update the `pub use sidereal::{...}` line to include the helper:

```rust
pub use sidereal::{
    equation_of_equinoxes, equation_of_equinoxes_degrees, greenwich_mean_sidereal_time_degrees,
    sidereal_time,
};
```

(Keep any remaining items on that `pub use` list — e.g. `SiderealTime` — exactly as they are.)

- [ ] **Step 5: Run the apparent tests**

Run: `cargo test -p pleiades-apparent equation_of_equinoxes`
Expected: PASS — both new tests and the existing `equation_of_equinoxes_is_small`.

- [ ] **Step 6: Route the chart topocentric-LAST path through the helper**

In `crates/pleiades-core/src/chart/mod.rs`, replace the hand-inlined equation-of-equinoxes line (the `let eq_equinoxes = (nut.delta_psi_arcsec / 3600.0) * true_obliquity.to_radians().cos();` line, ~411):

```rust
                    // Apparent sidereal time = GMST + equation of the equinoxes + east longitude.
                    let eq_equinoxes = pleiades_apparent::sidereal::equation_of_equinoxes(
                        nut.delta_psi_arcsec / 3600.0,
                        true_obliquity,
                    );
```

Leave the surrounding `gmst`, `true_obliquity`, and `last` computations unchanged.

- [ ] **Step 7: Run the chart + topocentric tests to confirm no numeric change**

Run: `cargo test -p pleiades-core chart && cargo test -p pleiades-validate validate_topocentric`
Expected: PASS — topocentric LAST is bit-for-bit unchanged (same formula, same inputs).

- [ ] **Step 8: Commit**

```bash
cargo fmt --all && cargo clippy -p pleiades-apparent -p pleiades-core --all-targets
git add crates/pleiades-apparent/src/sidereal.rs crates/pleiades-apparent/src/lib.rs crates/pleiades-core/src/chart/mod.rs
git commit -m "refactor(sidereal): single-source equation-of-equinoxes helper

Extract equation_of_equinoxes(delta_psi_deg, true_obliquity_deg); both apparent's
jd-driven wrapper and pleiades-core's chart topocentric-LAST path now call it,
removing the hand-inlined cos(ε) at chart/mod.rs. Closes FU-5 item 1 (EE half)."
```

---

## Task 3: Assert the Porphyry high-latitude fallback `asc_mc` site

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests.rs`

**Interfaces:**
- Consumes: `calculate_houses`, `HouseRequest`, `HighLatitudePolicy::SwissEphemerisFallback`, `HouseSystem::Placidus`, and (via `super::*`) the private `asc_mc_from` and `local_sidereal_time` helpers. `AscMc` derives `PartialEq`.

- [ ] **Step 1: Write the failing fallback test**

Add to `crates/pleiades-houses/src/systems/tests.rs` (the module uses `super::*`, so private helpers are in scope):

```rust
#[test]
fn porphyry_fallback_snapshot_carries_consistent_asc_mc() {
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    // Latitude 75° is beyond Placidus's polar bound, so with the SE fallback
    // policy calculate_houses takes the early-return Porphyry-fallback branch.
    let observer = ObserverLocation::new(
        Latitude::from_degrees(75.0),
        Longitude::from_degrees(10.0),
        None,
    );
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let req = HouseRequest::new(instant, observer.clone(), HouseSystem::Placidus)
        .with_high_latitude_policy(HighLatitudePolicy::SwissEphemerisFallback);

    let snap = calculate_houses(&req).expect("Porphyry fallback should produce a snapshot");

    // It really took the fallback: Porphyry yields 12 quadrant cusps.
    assert_eq!(snap.cusps.len(), 12);

    // The fallback site's asc_mc must equal an independent recomputation.
    let expected = asc_mc_from(
        local_sidereal_time(instant, observer.longitude).degrees(),
        observer.latitude.degrees(),
        snap.obliquity.degrees(),
    )
    .expect("asc_mc_from");
    assert_eq!(snap.asc_mc, expected);
}
```

- [ ] **Step 2: Run test to verify it fails (or confirm it compiles against real APIs)**

Run: `cargo test -p pleiades-houses porphyry_fallback_snapshot_carries_consistent_asc_mc`
Expected: PASS if the fallback is already correct (this is a characterization test locking in the untested site). If it FAILS to compile, fix the referenced names against the real API — e.g. confirm `snap.cusps.len()` (adjust to the actual cusp accessor if `cusps` is not a slice) and that `local_sidereal_time`/`asc_mc_from` are reachable via `super::*`.

Note: `snap.cusps` is the `HouseSnapshot` cusps field; if it is a wrapper rather than a `Vec`/slice, replace `snap.cusps.len()` with the equivalent count accessor (grep `impl` blocks near the cusps type). The `asc_mc` equality is the assertion that matters.

- [ ] **Step 3: Run the full houses test suite**

Run: `cargo test -p pleiades-houses`
Expected: PASS — new test plus all existing `asc_mc`/snapshot tests.

- [ ] **Step 4: Commit**

```bash
cargo fmt --all && cargo clippy -p pleiades-houses --all-targets
git add crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): assert Porphyry high-latitude fallback asc_mc consistency

Covers the early-return SwissEphemerisFallback HouseSnapshot construction site
(previously verified by inspection only). Closes FU-5 item 3."
```

---

## Task 4: Add a southern-hemisphere `validate-angles` corpus row

**Files:**
- Modify: `tools/se-house-reference/src/main.rs`
- Regenerate: `crates/pleiades-validate/data/houses-corpus/{cusps.csv,sectors.csv,angles.csv}`
- Modify: `crates/pleiades-validate/data/houses-corpus/manifest.txt`

**Interfaces:**
- Consumes: the `validate-angles` and `validate-houses` gates (read the regenerated CSVs and the updated manifest checksums).

- [ ] **Step 1: Confirm the gate is currently green (baseline)**

Run: `cargo test -p pleiades-validate validate_angles_passes_over_committed_corpus validate_houses`
Expected: PASS — establishes a clean baseline before touching the corpus.

- [ ] **Step 2: Add the southern fixture to the reference tool**

In `tools/se-house-reference/src/main.rs`, update the fixtures list and its comment:

```rust
    // 6 fixtures × 23 systems = 138 data rows.
    // In-band latitudes only (northern 0, 40, 55, 66 and southern -33).
    // Strict-rejection latitudes (70, 80) are NOT included here — they are
    // asserted by the gate (Tasks 8/9).
    let fixtures: &[(&str, f64, f64, f64, f64)] = &[
        ("c0_lat00", 2_451_545.0, 0.0, 0.0, 0.0),
        ("c1_lat40", 2_451_545.0, 40.0, 0.0, 0.0),
        ("c2_lat55", 2_451_545.0, 55.0, 0.0, 0.0),
        ("c3_lat66", 2_451_545.0, 66.0, 0.0, 0.0),
        ("c4_lat40_e2", 2_433_283.0, 40.0, 30.0, 0.0),
        ("c5_lat33s", 2_451_545.0, -33.0, 20.0, 0.0),
    ];
```

(Longitude 20° gives the southern row a non-zero LST distinct from `c0`, so it exercises the `f_pole = -90 - lat` branch at a representative southern latitude.)

- [ ] **Step 3: Regenerate all three corpus slices**

Run: `LIBCLANG_PATH=/lib/x86_64-linux-gnu cargo run -p se-house-reference -- crates/pleiades-validate/data/houses-corpus`
Expected: overwrites `cusps.csv`, `sectors.csv`, `angles.csv` in that directory. (Houses/sidereal time need no ephemeris data files — the tool runs standalone.)

- [ ] **Step 4: Verify the diff only ADDS the new fixture's rows**

Run: `git diff --stat crates/pleiades-validate/data/houses-corpus/ && git diff crates/pleiades-validate/data/houses-corpus/angles.csv`
Expected: `angles.csv` gains exactly one row (`c5_lat33s,...`), `cusps.csv` gains 23 rows, `sectors.csv` gains 1 row; **no existing rows change**. If any existing row changed, STOP — this is SE-version drift (Global Constraint), surface as a finding and do not proceed.

- [ ] **Step 5: Run gates to read the new checksums from the mismatch errors**

Run: `cargo test -p pleiades-validate validate_angles_passes_over_committed_corpus validate_houses 2>&1 | grep -i 'checksum mismatch'`
Expected: FAIL — each gate prints `... checksum mismatch: expected <old>, got <new>`. Record the three `got` values (cusps checksum, sectors checksum from the houses gate; angles checksum from the angles gate) and the new row counts (`cusps` 115→138, `sectors` 5→6, `angles` 5→6).

- [ ] **Step 6: Update the manifest**

In `crates/pleiades-validate/data/houses-corpus/manifest.txt`, update the three `slice` lines' `rows=` and `checksum=` to the values from Step 5:

```
slice cusps file=cusps.csv role=cusps rows=138 checksum=<new-cusps-checksum>
slice sectors file=sectors.csv role=sectors rows=6 checksum=<new-sectors-checksum>
slice angles file=angles.csv role=angles rows=6 checksum=<new-angles-checksum>
```

(Substitute the actual `got` u64 values; leave the header comment lines unchanged.)

- [ ] **Step 7: Run the gates to verify green on the southern row**

Run: `cargo test -p pleiades-validate`
Expected: PASS — `validate_angles_passes_over_committed_corpus` and the houses cusp/sectors gates pass. The southern row's residuals must sit within existing ceilings (ARMC/GAST 1.0″, geometry points 0.5″). If a southern *cusp* row exceeds a ceiling, STOP and surface it as a finding (Global Constraint) — do not loosen the ceiling.

- [ ] **Step 8: Commit**

```bash
cargo fmt --all
git add tools/se-house-reference/src/main.rs crates/pleiades-validate/data/houses-corpus/
git commit -m "test(angles): add southern-hemisphere corpus row (lat -33)

Adds one southern fixture to se-house-reference and regenerates the houses
corpus (cusps/sectors/angles) so the asc_mc_from f_pole = -90 - lat branch is
exercised by validate-angles. Manifest rows/checksums bumped. Closes FU-5 item 2."
```

---

## Task 5: Close out FU-5 in the follow-ups ledger

**Files:**
- Modify: `docs/follow-ups.md`

- [ ] **Step 1: Run the full workspace verification**

Run: `cargo test --workspace && cargo fmt --all -- --check && cargo clippy --workspace --all-targets`
Expected: PASS / clean across the board.

- [ ] **Step 2: Mark FU-5's three items resolved**

In `docs/follow-ups.md` § FU-5, update the status line and the three bullet items: change each "**Remains open.**" to a resolution note referencing this plan and the commits (GMST/EE single-source; southern-hemisphere row; Porphyry-fallback `asc_mc` test). Set the section status to `resolved (2026-07-01)` with a one-line summary, mirroring the format of the already-resolved FU entries above it.

- [ ] **Step 3: Commit**

```bash
git add docs/follow-ups.md
git commit -m "docs(follow-ups): resolve FU-5 angles & sidereal-time items

GMST + equation-of-equinoxes single-sourced, southern-hemisphere validate-angles
row added, Porphyry high-latitude fallback asc_mc asserted."
```

---

## Self-Review

**Spec coverage:**
- Item 1 (GMST/EE single-source) → Tasks 1 (GMST) + 2 (EE). ✓ Delegation direction `apparent → time` acyclic; EE helper shared with chart. ✓ Out-of-scope SP-2 migration excluded (Global Constraints). ✓ Cross-crate agreement test present (Task 1 Step 1). ✓
- Item 2 (southern gate row) → Task 4, approach A (regenerate all slices). ✓ `libclang` path pinned. ✓ Manifest rows+checksums. ✓ Finding-not-patch discipline for a southern-cusp surprise. ✓
- Item 3 (Porphyry-fallback asc_mc) → Task 3. ✓
- Ordering Item 1 → 3 → 2 → Tasks 1,2,3,4. ✓ Ledger close-out → Task 5. ✓

**Placeholder scan:** The only `<...>` placeholders are the three checksum u64 values in Task 4 Step 6, which are runtime-derived outputs read from the gate mismatch errors in Step 5 — legitimately not knowable at plan-writing time. Every code/test block is complete.

**Type consistency:** `gmst_degrees_raw`/`gmst_degrees` (Task 1) consumed identically in the Task 1 agreement test; `equation_of_equinoxes(delta_psi_deg, true_obliquity_deg)` defined in Task 2 Step 3, exported in Step 4, consumed in Step 6 with matching argument order (Δψ in degrees, then true obliquity in degrees). `AscMc: PartialEq` (verified) supports the `assert_eq!` in Task 3. `with_high_latitude_policy` / `HighLatitudePolicy::SwissEphemerisFallback` verified against the real API.
