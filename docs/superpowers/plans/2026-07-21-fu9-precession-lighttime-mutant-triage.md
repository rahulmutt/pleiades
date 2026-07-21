# FU-9 Precession + Lighttime Mutant Triage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive `crates/pleiades-apparent/src/precession.rs` from 17 surviving mutants to 2 documented equivalents and `crates/pleiades-apparent/src/lighttime.rs` from 5 to 0, finishing the `pleiades-apparent` crate's FU-9 backlog.

**Architecture:** Tests-only slice (seventh in the FU-9 series; spec: `docs/superpowers/specs/2026-07-21-fu9-precession-lighttime-mutant-triage-design.md`). Each file's inline `#[cfg(test)] mod tests` relocates to a co-located `tests.rs` per AGENTS.md, then gains intent-expressing tests whose expected values come from independent references (external Python-evaluated literals, crafted-exact f64 inputs) — never from the code's own output.

**Tech Stack:** Rust (stable, via mise), cargo-nextest, cargo-mutants 27.1.0.

## Global Constraints

- **Tests-only:** no production-code change in either file; the only source edits are the two inline test-module relocations.
- **No parity-gate change:** no `validate-*` file is touched; the mutants tier stays report-only.
- **No `#[mutants::skip]`:** the 2 residual precession guard mutants are documented equivalents, left visible.
- **Independence discipline:** every expected value comes from an independent reference (external literals or crafted-exact inputs), never from running the code under test and pinning its output.
- **Branch:** `fu9-precession-lighttime-mutant-triage` (already created; the design spec is committed on it).
- All commands run through mise: prefix cargo invocations with `mise exec --`.

---

### Task 1: Precession — relocate tests, add the three killing tests, verify 17 → 2

**Files:**
- Modify: `crates/pleiades-apparent/src/precession.rs` (delete inline tests module, lines 136–211; add `mod tests;` declaration)
- Create: `crates/pleiades-apparent/src/precession/tests.rs`

**Interfaces:**
- Consumes: `precess_ecliptic_date_to_j2000`, `precess_ecliptic_j2000_to_date`, `PrecessedEcliptic`, `crate::error::ApparentPlaceError` (all reachable via `use super::*` — the parent's private `use` bindings are importable by its child test module).
- Produces: nothing consumed by later tasks (Task 3 only cites results).

- [ ] **Step 1: Relocate the inline test module**

In `crates/pleiades-apparent/src/precession.rs`, replace the entire inline module — from the line `#[cfg(test)]` (line 136) through the final closing brace of `mod tests` (line 211, end of file) — with:

```rust
#[cfg(test)]
mod tests;
```

Create `crates/pleiades-apparent/src/precession/tests.rs` containing the four existing tests verbatim (white-box unit tests, kept per AGENTS.md — do not convert to integration tests):

```rust
use super::*;

#[test]
fn date_to_j2000_is_the_inverse_of_j2000_to_date() {
    // Round-trip a non-trivial point at a far epoch: J2000 -> date -> J2000.
    let jd = 2_415_025.5; // 1900
    let to_date = precess_ecliptic_j2000_to_date(123.456, 4.5, jd).unwrap();
    let back = precess_ecliptic_date_to_j2000(to_date.longitude_deg, to_date.latitude_deg, jd)
        .unwrap();
    assert!(
        (back.longitude_deg - 123.456).abs() < 1e-6,
        "λ round-trip {}",
        back.longitude_deg
    );
    assert!(
        (back.latitude_deg - 4.5).abs() < 1e-6,
        "β round-trip {}",
        back.latitude_deg
    );
}

#[test]
fn identity_at_j2000() {
    // At J2000 the precession angles are zero and the inbound/outbound
    // obliquities are equal, so the transform is the identity.
    let out = precess_ecliptic_j2000_to_date(123.456, 4.5, 2_451_545.0).unwrap();
    assert!(
        (out.longitude_deg - 123.456).abs() < 1e-6,
        "λ = {}",
        out.longitude_deg
    );
    assert!(
        (out.latitude_deg - 4.5).abs() < 1e-6,
        "β = {}",
        out.latitude_deg
    );
}

#[test]
fn general_precession_one_century() {
    // The J2000 vernal-equinox direction (λ=0, β=0) viewed in the
    // equinox-of-date frame one Julian century on has longitude ≈ the general
    // precession in longitude (5029.0966″/cy = 1.39697°). β stays small but
    // NOT exactly zero: the ecliptic plane itself precesses (~47″/cy), so a
    // point in the J2000 ecliptic acquires ≈ +4.4″ (0.00122°) of ecliptic-of-
    // date latitude. This is physically real and matches the rigorous Meeus
    // ch.21 ecliptic-precession result (4.39″) to sub-mas; the bound below is
    // widened from the naive 1e-3° to admit that residual while still catching
    // gross errors (a transcription bug would produce degrees, not arcsec).
    let jd = 2_451_545.0 + 36_525.0;
    let out = precess_ecliptic_j2000_to_date(0.0, 0.0, jd).unwrap();
    assert!(
        (out.longitude_deg - 1.39697).abs() < 5e-3,
        "λ' = {}",
        out.longitude_deg
    );
    assert!(out.latitude_deg.abs() < 2e-3, "β' = {}", out.latitude_deg);
}

#[test]
fn longitude_shifts_by_precession_off_the_ecliptic() {
    // For an off-ecliptic point, longitude still shifts by ≈ the general
    // precession over a century; latitude moves only slightly (ecliptic motion).
    let jd = 2_451_545.0 + 36_525.0;
    let out = precess_ecliptic_j2000_to_date(80.0, 30.0, jd).unwrap();
    let dlon = out.longitude_deg - 80.0;
    assert!((dlon - 1.397).abs() < 0.05, "Δλ = {dlon}");
    assert!(
        (out.latitude_deg - 30.0).abs() < 0.05,
        "β' = {}",
        out.latitude_deg
    );
}
```

- [ ] **Step 2: Verify the relocation is behavior-neutral**

Run: `mise exec -- cargo nextest run -p pleiades-apparent -E 'test(/precession::tests/)'`
Expected: 4 tests PASS (same four names as before the move).

- [ ] **Step 3: Add the three new tests**

Append to `crates/pleiades-apparent/src/precession/tests.rs`:

```rust
#[test]
fn date_to_j2000_matches_independent_literals_at_pm4_centuries() {
    // Expected values computed OUTSIDE this crate by an independent Python
    // implementation of the published pipeline (Meeus 20.3 precession
    // angles, 21.4 equatorial rotation, 13.x ecliptic<->equatorial bridges,
    // 22.2 mean obliquity), cross-validated against the genuinely different
    // Meeus 21.5 elements + 21.7 direct-ecliptic rotation (agreement
    // ~3e-3″ at t = ±4). See the 2026-07-21 FU-9 slice design doc §6.
    // Epochs sit at t = ±4 Julian centuries (≈ years 2400/1600, inside the
    // 1600–2600 coverage target), far from the |t| ≈ 1 degeneracy where
    // `t*t ≈ t/t` hid the quadratic/cubic `*` -> `/` mutants from the 1900
    // round-trip. Smallest mutant displacement at this geometry: 0.961″ =
    // 2.67e-4°, ~2.7e5× the tolerance below.
    const TOL_DEG: f64 = 1e-9;
    let cases = [
        // (jd_tt, expected λ_J2000, expected β_J2000); inputs are mean-of-date.
        (2_597_645.0, 117.860_897_668_741, 4.456_799_466_404), // t = +4
        (2_305_445.0, 129.041_779_511_373, 4.538_180_014_018), // t = -4
    ];
    for (jd, exp_lon, exp_lat) in cases {
        let out = precess_ecliptic_date_to_j2000(123.456, 4.5, jd).unwrap();
        assert!(
            (out.longitude_deg - exp_lon).abs() < TOL_DEG,
            "λ at jd {jd}: {} vs {exp_lon}",
            out.longitude_deg
        );
        assert!(
            (out.latitude_deg - exp_lat).abs() < TOL_DEG,
            "β at jd {jd}: {} vs {exp_lat}",
            out.latitude_deg
        );
    }
}

#[test]
fn date_to_j2000_identity_at_j2000() {
    // Inverse-direction mirror of `identity_at_j2000` — a genuine intent
    // gap: the inverse's identity property was never asserted. At t = 0 the
    // precession angles vanish and the of-date obliquity equals ε₀, so the
    // inverse transform is also the identity. (Redundant mutant kill: at
    // t = 0 every quadratic/cubic `*` -> `/` mutant divides by zero,
    // producing NaN and a NonFiniteCorrection error instead of Ok.)
    let out = precess_ecliptic_date_to_j2000(123.456, 4.5, 2_451_545.0).unwrap();
    assert!(
        (out.longitude_deg - 123.456).abs() < 1e-9,
        "λ = {}",
        out.longitude_deg
    );
    assert!(
        (out.latitude_deg - 4.5).abs() < 1e-9,
        "β = {}",
        out.latitude_deg
    );
}

#[test]
fn overflow_epoch_fails_closed_in_both_directions() {
    // jd_tt = 7.0e107 puts t ≈ 1.92e103 in the window where θ's cubic term
    // overflows to -inf while ζ, z, and the mean obliquity stay finite (θ's
    // 0.041833 coefficient is the largest of the cubics, so it overflows
    // first as |t| grows). sin/cos(±inf) = NaN then poisons BOTH the b
    // (→ longitude) and c (→ latitude) rotation terms, so both outputs go
    // non-finite together and the guard fails closed. This shared poisoning
    // is also why the `||` -> `&&` guard mutants are documented equivalents:
    // no reachable input makes exactly one output non-finite (design doc §5).
    let err = precess_ecliptic_date_to_j2000(123.456, 4.5, 7.0e107).unwrap_err();
    assert!(
        matches!(
            err,
            ApparentPlaceError::NonFiniteCorrection {
                stage: "precession"
            }
        ),
        "date->J2000: expected NonFiniteCorrection, got {err:?}"
    );
    let err = precess_ecliptic_j2000_to_date(123.456, 4.5, 7.0e107).unwrap_err();
    assert!(
        matches!(
            err,
            ApparentPlaceError::NonFiniteCorrection {
                stage: "precession"
            }
        ),
        "J2000->date: expected NonFiniteCorrection, got {err:?}"
    );
}
```

(`ApparentPlaceError` arrives via `use super::*` — the parent module's `use crate::error::ApparentPlaceError;` binding is visible to its child test module. If the compiler disagrees, add `use crate::error::ApparentPlaceError;` below `use super::*;`.)

- [ ] **Step 4: Run the new tests**

Run: `mise exec -- cargo nextest run -p pleiades-apparent -E 'test(/precession::tests/)'`
Expected: 7 tests PASS.

**If `date_to_j2000_matches_independent_literals_at_pm4_centuries` fails:** do NOT widen the tolerance. The literals were cross-validated Python↔published-formulation at design stage; Python↔Rust agreement should be ~1e-13° (identical IEEE operations). A mismatch above 1e-12° means a planning error (wrong literal transcription or a formula divergence) — stop and investigate with superpowers:systematic-debugging.

- [ ] **Step 5: Run the authoritative per-file mutants command**

Run: `mise exec -- cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file crates/pleiades-apparent/src/precession.rs`
Expected final line: `238 mutants tested: 2 missed, 234 caught, 2 unviable` — and the 2 missed must be exactly:

```
precession.rs:71:35: replace || with && in precess_ecliptic_date_to_j2000
precession.rs:125:35: replace || with && in precess_ecliptic_j2000_to_date
```

(these are the documented equivalents; any other survivor means a test is not killing what the design claims — stop and investigate).

- [ ] **Step 6: Format, lint**

Run: `mise exec -- cargo fmt --all && mise exec -- cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings`
Expected: no diff beyond the files above; clippy clean.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-apparent/src/precession.rs crates/pleiades-apparent/src/precession/tests.rs
git commit -m "test(apparent): FU-9 precession.rs mutant triage (17 -> 2 documented equivalents)"
```

---

### Task 2: Lighttime — relocate tests, add the three killing tests, verify 5 → 0

**Files:**
- Modify: `crates/pleiades-apparent/src/lighttime.rs` (delete inline tests module, lines 82–151; add `mod tests;` declaration)
- Create: `crates/pleiades-apparent/src/lighttime/tests.rs`

**Interfaces:**
- Consumes: `apparent_via_light_time`, `LightTimePosition`, `LIGHT_TIME_DAYS_PER_AU`, and the private consts `CONVERGENCE_DAYS`, `MAX_PLAUSIBLE_LIGHT_TIME_DAYS` (child test modules may reference the parent's private items), plus `ApparentLightTimeError`/`ApparentPlaceError`/`EclipticCoordinates`/`Instant`/`JulianDay` via `use super::*`.
- Produces: nothing consumed by later tasks (Task 3 only cites results).

- [ ] **Step 1: Relocate the inline test module**

In `crates/pleiades-apparent/src/lighttime.rs`, replace the entire inline module — from the line `#[cfg(test)]` (line 82) through the final closing brace of `mod tests` (line 151, end of file) — with:

```rust
#[cfg(test)]
mod tests;
```

Create `crates/pleiades-apparent/src/lighttime/tests.rs` containing the existing helper and four tests verbatim:

```rust
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
fn absurd_distance_is_rejected_as_non_convergent() {
    // A body returning 50,000 AU (light-time ≈ 289 days) must be rejected
    // by the sanity cap (MAX_PLAUSIBLE_LIGHT_TIME_DAYS = 10 days), not
    // silently returned as a huge retardation.
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let err = apparent_via_light_time::<_, ApparentPlaceError>(instant, 8, |_| {
        Ok(at(0.0, 90.0, 50_000.0))
    })
    .unwrap_err();
    assert!(
        matches!(
            err,
            ApparentLightTimeError::Apparent(ApparentPlaceError::NonConvergentLightTime { .. })
        ),
        "expected NonConvergentLightTime for absurd distance, got: {err:?}"
    );
}

#[test]
fn query_error_is_propagated() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let err =
        apparent_via_light_time::<_, &str>(instant, 8, |_| Err("backend down")).unwrap_err();
    assert!(matches!(err, ApparentLightTimeError::Query("backend down")));
}
```

- [ ] **Step 2: Verify the relocation is behavior-neutral**

Run: `mise exec -- cargo nextest run -p pleiades-apparent -E 'test(/lighttime::tests/)'`
Expected: 4 tests PASS (same four names as before the move).

- [ ] **Step 3: Add the three new tests**

Append to `crates/pleiades-apparent/src/lighttime/tests.rs`:

```rust
#[test]
fn converged_position_is_queried_at_retarded_epoch() {
    // The point of the light-time iteration is that the returned position
    // is the one evaluated at the RETARDED epoch t − τ. Every other test's
    // query ignores the instant it is given, which is exactly why the
    // retarded-epoch mutants survived the baseline. Here the query's
    // longitude depends on the instant (1000 °/day, constant 5 AU), so the
    // retarded epoch is observable: expected longitude = 100 − 1000τ
    // ≈ 71.1224085°. Mutant margins at this geometry (design doc §4.2):
    // `-` -> `+` queries base + τ → 128.878° (57.8° off); `-` -> `/`
    // queries jd = base/τ ≈ 8.49e7 → 183.967° (112.8° off); convergence
    // `<` -> `>` converges on iteration 1 with the UNRETARDED position
    // → 100.0° (28.9° off) and iterations == 1.
    const BASE: f64 = 2_451_545.0;
    let tau = 5.0 * LIGHT_TIME_DAYS_PER_AU;
    let instant = Instant::new(JulianDay::from_days(BASE), TimeScale::Tt);
    let out = apparent_via_light_time::<_, ApparentPlaceError>(instant, 8, |i| {
        let lon = 100.0 + 1000.0 * (i.julian_day.days() - BASE);
        Ok(at(i.julian_day.days(), lon, 5.0))
    })
    .unwrap();
    let expected_lon = 100.0 - 1000.0 * tau;
    assert!(
        (out.ecliptic.longitude.degrees() - expected_lon).abs() < 1e-9,
        "longitude {} should be {expected_lon} (queried at the retarded epoch)",
        out.ecliptic.longitude.degrees()
    );
    assert_eq!(out.iterations, 2);
    // Exact: light_time_days is the same f64 product the test computes.
    assert_eq!(out.light_time_days, tau);
}

#[test]
fn light_time_exactly_at_cap_is_accepted() {
    // The cap's contract is "EXCEEDING this cap" is non-convergent — a
    // light-time exactly AT the cap converges normally, pinning the strict
    // `>`. 1731.4463361669202 AU (0x1.b0dc90c591fc7p+10) is crafted so the
    // f64 product distance × LIGHT_TIME_DAYS_PER_AU is EXACTLY 10.0
    // (design-stage representability check; asserted below as a
    // precondition so a future constant change cannot silently degrade
    // this test into the non-boundary case).
    const D_CAP: f64 = 1731.446_336_166_920_2;
    assert_eq!(D_CAP * LIGHT_TIME_DAYS_PER_AU, MAX_PLAUSIBLE_LIGHT_TIME_DAYS);
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let out = apparent_via_light_time::<_, ApparentPlaceError>(instant, 8, |_| {
        Ok(at(0.0, 90.0, D_CAP))
    })
    .unwrap();
    assert_eq!(out.light_time_days, MAX_PLAUSIBLE_LIGHT_TIME_DAYS);
    assert_eq!(out.iterations, 2);
}

#[test]
fn convergence_requires_strict_retardation_decrease() {
    // 8.6572316808346e-05 AU is crafted so the first-iteration retardation
    // change |new_tau − 0| is EXACTLY CONVERGENCE_DAYS (5e-7; asserted as a
    // precondition). The strict `<` must NOT declare convergence on
    // iteration 1 — a change equal to the threshold is not yet converged —
    // so convergence lands on iteration 2. (Also re-kills `<` -> `>`,
    // which would never converge here and exhaust max_iterations.)
    const D_CONV: f64 = 8.657_231_680_834_6e-5;
    assert_eq!(D_CONV * LIGHT_TIME_DAYS_PER_AU, CONVERGENCE_DAYS);
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let out = apparent_via_light_time::<_, ApparentPlaceError>(instant, 8, |_| {
        Ok(at(0.0, 90.0, D_CONV))
    })
    .unwrap();
    assert_eq!(out.iterations, 2);
    assert_eq!(out.light_time_days, CONVERGENCE_DAYS);
}
```

- [ ] **Step 4: Run the new tests**

Run: `mise exec -- cargo nextest run -p pleiades-apparent -E 'test(/lighttime::tests/)'`
Expected: 7 tests PASS.

**If a precondition `assert_eq!` on the crafted products fails:** the boundary distances no longer land exactly on the thresholds (this can only happen if `LIGHT_TIME_DAYS_PER_AU` or the consts changed). Stop and re-derive the boundary distance (find the f64 `d` near `threshold / constant` whose product rounds exactly to the threshold, stepping by ulps) rather than loosening to a tolerance.

- [ ] **Step 5: Run the authoritative per-file mutants command**

Run: `mise exec -- cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file crates/pleiades-apparent/src/lighttime.rs`
Expected final line: `14 mutants tested: 0 missed, 13 caught, 1 unviable`.

- [ ] **Step 6: Format, lint**

Run: `mise exec -- cargo fmt --all && mise exec -- cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings`
Expected: no diff beyond the files above; clippy clean.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-apparent/src/lighttime.rs crates/pleiades-apparent/src/lighttime/tests.rs
git commit -m "test(apparent): FU-9 lighttime.rs mutant triage (5 -> 0)"
```

---

### Task 3: Record progress in follow-ups, run full CI, commit

**Files:**
- Modify: `docs/follow-ups.md` (append a progress note inside the FU-9 entry, after the sidereal progress note that ends "…then the remaining `pleiades-time` (non-sidereal) and `pleiades-types` survivors.")

**Interfaces:**
- Consumes: the verified outcomes of Tasks 1 and 2 (survivor counts, margins). If either task's Step 5 result differed from expectations, update the note's numbers to the measured truth — never record the plan's predictions over the actual output.
- Produces: the FU-9 progress record.

- [ ] **Step 1: Append the progress note**

In `docs/follow-ups.md`, inside the FU-9 section, insert after the sidereal progress paragraph (before the `---` that closes the FU-9 entry, keeping the existing "Residual audit gap" note in place):

```markdown
**Progress (2026-07-21) — precession + lighttime
(`pleiades-apparent/src/precession.rs` + `lighttime.rs`):** triaged from
`17` → `2` documented equivalent mutants and `5` → `0` (spec/plan:
`docs/superpowers/specs/2026-07-21-fu9-precession-lighttime-mutant-triage-design.md`).
The second two-file slice, closing out `pleiades-apparent` entirely. Both
baselines confirmed by the authoritative per-file commands (precession:
`238 tested, 17 missed, 219 caught, 2 unviable`; lighttime: `14 tested,
5 missed, 8 caught, 1 unviable`), both matching the whole-workspace figures
exactly. **Tests-only** like refraction/topocentric/sidereal: the only
source edits were relocating both inline test modules to
`src/precession/tests.rs` and `src/lighttime/tests.rs`. Root causes: all 15
precession polynomial survivors (`*`→`/` on the quadratic/cubic ζ/z/θ
terms) sit in the **inverse** function, whose only test was the 1900
round-trip at t ≈ −1 — where `t*t ≈ t/t` displaces the output by ~1e-8°,
under the 1e-6° tolerance — while the forward twin's identical mutants die
at t = 0 (`/t` → NaN) in the forward-only identity test; lighttime's 5
survived because every query closure ignored the instant it was given, so
no test could observe the retarded epoch, plus two never-hit exact
comparison boundaries. Kills: pinned literals for the inverse at t = ±4
(independent Python implementation of the published Meeus 20.3/21.4/13.x
pipeline, cross-validated against the genuinely different Meeus 21.5/21.7
direct-ecliptic formulation to ~3e-3″; smallest mutant displacement 0.961″
vs a 1e-9° tolerance, ~2.7e5× margin), an inverse identity test (closing a
real intent gap), an instant-dependent 1000 °/day query pinning the
retarded epoch (mutant margins 28.9°/57.8°/112.8°), and crafted-exact f64
boundary distances landing the light-time exactly on the 10-day cap and
the 5e-7-day convergence threshold (both representability-checked, with
in-test precondition asserts). A fail-closed overflow test at
jd_tt = 7.0e107 (the window where θ's cubic term alone overflows) records
why the residual exists. **Documented residual — 2 equivalent mutants**,
left visible rather than `#[mutants::skip]`-suppressed: `||`→`&&` in both
output non-finite guards — the `nutation.rs`/`topocentric.rs` shape,
checked against the overflow lens rather than by analogy: every non-finite
route (NaN inputs, or finite-huge `jd_tt` overflowing θ first) flows
through shared variables (t, ζ/z/θ, α/δ, ε) that poison both outputs
together, the outputs themselves cannot overflow (bounded `atan2`, clamped
`asin`), so no reachable input makes exactly one output non-finite. No
parity gate was touched; the tier stays report-only; `mise run ci` is
green. **Remaining slices** (priority order): `pleiades-time` non-sidereal
(`convert.rs` 16, `deltat.rs` 10, `tdb.rs` 9), then `pleiades-types`
(`zodiac.rs` 12, `time.rs` 10, and the small tail).
```

- [ ] **Step 2: Run full CI**

Run: `mise run ci`
Expected: green (fmt, clippy, tests, and the rest of the blocking tier all pass).

- [ ] **Step 3: Commit**

```bash
git add docs/follow-ups.md
git commit -m "docs(follow-ups): record FU-9 precession+lighttime triage (17 -> 2, 5 -> 0)"
```

---

## Completion

After Task 3, the branch holds four commits (spec + two test commits + docs). Hand off to superpowers:finishing-a-development-branch for PR/merge — per the repo's standing rule: merge the PR and delete the branch without re-asking, but NEVER via `gh pr merge --auto` (it merges immediately on this repo); watch checks go green first.
