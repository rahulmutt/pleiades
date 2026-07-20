# FU-9 Slice 4 — `aberration.rs` Mutant Triage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive `crates/pleiades-apparent/src/aberration.rs` from 28 surviving cargo-mutants mutants to 0, via one behavior-preserving extraction plus white-box unit tests whose expected values come from independently evaluated published coefficients.

**Architecture:** Extract the Earth-orbit element polynomials into a private `earth_orbit_elements(t)` seam (11 of the 28 survivors perturb the public output by only 0.001–0.006″ and are unreachable through the public API). Relocate the inline test module to a co-located `aberration/tests.rs`. Then add tests in two groups: polynomial/epoch tests against Meeus 25.4 coefficients hand-evaluated at three epochs, and formula-line tests at a single crafted geometry proven to discriminate all 12 remaining mutants.

**Tech Stack:** Rust 2021, `cargo-mutants` 27.1.0, `cargo-nextest`, `mise` task runner.

**Spec:** [`../specs/2026-07-20-fu9-aberration-mutant-triage-design.md`](../specs/2026-07-20-fu9-aberration-mutant-triage-design.md)

**Branch:** `fu9-aberration-mutant-triage` (already exists; spec committed as `db5097bf8`)

## Global Constraints

- **No parity gate may be touched.** Do not modify any `crates/pleiades-validate/**` file or any `validate-*` corpus. The mutants tier stays **report-only**.
- **No `#[mutants::skip]`.** A function-level skip would blanket-hide that function's entire numeric mutant surface. A genuinely equivalent mutant is left visible and documented in `docs/follow-ups.md` instead.
- **The refactor changes no runtime result.** It is a pure extraction: identical expressions, identical evaluation order.
- **Independence discipline.** Every expected value is an independently computed literal — published Meeus coefficients evaluated outside the code, or crafted-input branch reasoning. Never capture an expected value by running the code under test.
- **Tests stay white-box unit tests** with `use super::*` access to private helpers. Do not convert them to black-box integration tests.
- **Pre-existing tests are kept**, not replaced. They encode real physical intent (conjunction/opposition sign and magnitude).

---

## TDD inversion for mutation triage

Standard TDD writes a failing test, then makes it pass. Mutation triage inverts this: the production code is already correct, so a new test **passes immediately**. The red/green evidence is not the test failing — it is the **mutant dying**.

So each test task's "verify it fails" step is replaced by: run the per-file `cargo mutants` command and confirm the targeted survivors moved from MISSED to caught. Do not skip these verification steps; a test that passes but kills nothing is the exact failure mode this backlog exists to surface.

**The authoritative per-file command** (referred to below as `MUTANTS_CMD`):

```bash
cargo mutants -p pleiades-apparent --test-tool nextest \
  --test-workspace=false --file crates/pleiades-apparent/src/aberration.rs
```

Baseline at the start of this plan: **56 mutants tested, 28 missed, 27 caught, 1 unviable.**

---

## Reference Values (independently computed)

All values below were evaluated **outside** the code under test, from the published formulas. They are the only expected values this plan uses.

**Meeus 25.4 — Earth orbital elements:**

```
e(t)  = 0.016708634 - 0.000042037 t - 0.0000001267 t^2
ϖ(t)  = 102.93735   + 1.71946 t     + 0.00046 t^2
```

| `t` | `e` | `ϖ` (deg) |
| --- | --- | --- |
| `0.0` | `0.016708634` | `102.93735` |
| `+1.0` | `0.0166664703` | `104.65727` |
| `-1.0` | `0.0167505443` | `101.21835` |

**Julian centuries:** `julian_centuries(2469807.5) = 0.5` exactly (J2000 + 18262.5 d).

**Crafted geometry G2** — `λ = 30°, β = 60°, ⊙ = 120°, jd_tt = 2451545.0 (t = 0)`:

| Quantity | Value |
| --- | --- |
| `cos β` | `0.5` (analytically; `0.5000000000000001` in binary f64) |
| `cos(⊙ − λ) = cos 90°` | `0` (analytically; `6.123233995736766e-17` in binary f64) |
| `ϖ − λ` | `72.93735°` |
| `cos(ϖ − λ)` | `0.29341719999540683` |
| `sin(ϖ − λ)` | `0.9559844908505867` |
| `e·κ` | `0.34245214231967996` |
| **`Δλ`** | **`0.20096269746373557`** |
| **`Δβ`** | **`-17.46612250773869`** |

Note on the f64 caveats: `cos 60°` and `cos 90°` are not bitwise exact, so the assertions use a `1e-9` absolute tolerance rather than equality. That tolerance is ~6 orders looser than the worst-case f64 noise and ~6 orders tighter than the smallest mutant margin (`0.0267″`), so it is unambiguous in both directions.

---

## File Structure

| File | Responsibility | Change |
| --- | --- | --- |
| `crates/pleiades-apparent/src/aberration.rs` | Annual-aberration formula (Meeus 23.2) + element polynomials | Modify: extract `earth_orbit_elements`; replace inline `mod tests { … }` with `mod tests;` |
| `crates/pleiades-apparent/src/aberration/tests.rs` | White-box unit tests for the module | Create |
| `docs/follow-ups.md` | FU-9 backlog record | Modify: append slice-4 progress entry |

---

## Survivor → Task Map

| Group | Location | Count | Killed by |
| --- | --- | --- | --- |
| A | `julian_centuries` (L19) | 5 | Task 3 |
| B | `e` polynomial (L36) | 5 | Task 3 |
| C | `ϖ` polynomial (L37) | 6 | Task 3 |
| D | Meeus formula lines (L46–47) | 12 | Task 4 |
| | **Total** | **28** | |

**Commit granularity vs. spec §9.** The spec sketches three commits (refactor / tests / docs); this plan produces five, splitting the relocation out of the refactor and the tests into their two survivor groups. This is a deliberate refinement, not a contradiction — each of the five is independently reviewable and independently revertable, which is the task-boundary rule. In particular, keeping the verbatim test relocation (Task 2) apart from new assertions (Tasks 3–4) means a reviewer can confirm at a glance that the move changed nothing.

---

### Task 1: Extract `earth_orbit_elements` (behavior-preserving)

**Files:**
- Modify: `crates/pleiades-apparent/src/aberration.rs` (the `annual_aberration` body, currently ~lines 34–37)

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces: `fn earth_orbit_elements(t: f64) -> (f64, f64)` — private to the `aberration` module, returns `(e, pi_deg)` where `e` is Earth's orbital eccentricity (dimensionless) and `pi_deg` is the longitude of perihelion ϖ in **degrees**. Tasks 3 and 4 depend on this exact name, arity, and tuple order.

- [ ] **Step 1: Confirm the pre-existing tests pass before any change**

Run:
```bash
cargo nextest run -p pleiades-apparent aberration
```
Expected: PASS (2 tests — `magnitude_is_bounded_by_kappa_over_cos_beta`, `sign_and_magnitude_match_known_geometry`).

- [ ] **Step 2: Add the extracted function**

Insert immediately after the existing `julian_centuries` function in `crates/pleiades-apparent/src/aberration.rs`:

```rust
/// Earth's orbital eccentricity and longitude of perihelion ϖ (degrees),
/// both of date. Meeus 25.4.
///
/// Extracted from `annual_aberration` so the polynomial coefficients have a
/// direct test seam: they reach the public output only through the ~0.34″
/// `e κ cos(ϖ - λ)` term, where a coefficient error moves the result by
/// ~0.001-0.006″ — far below any tolerance the model's own accuracy justifies.
fn earth_orbit_elements(t: f64) -> (f64, f64) {
    let e = 0.016_708_634 - 0.000_042_037 * t - 0.000_000_126_7 * t * t;
    let pi_deg = 102.937_35 + 1.719_46 * t + 0.000_46 * t * t;
    (e, pi_deg)
}
```

- [ ] **Step 3: Replace the inlined polynomials in `annual_aberration`**

In `annual_aberration`, replace these three lines:

```rust
    let t = julian_centuries(jd_tt);
    let e = 0.016_708_634 - 0.000_042_037 * t - 0.000_000_126_7 * t * t;
    let pi_deg = 102.937_35 + 1.719_46 * t + 0.000_46 * t * t;
```

with this single line:

```rust
    let (e, pi_deg) = earth_orbit_elements(julian_centuries(jd_tt));
```

The `t` binding is removed because it has no other use in the function. Everything below (the `to_radians` conversions and the two Meeus formula lines) is untouched.

- [ ] **Step 4: Verify no behavior change**

Run:
```bash
cargo nextest run -p pleiades-apparent aberration
cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings
cargo fmt --all --check
```
Expected: tests PASS (same 2 tests, unmodified), clippy clean, fmt clean. The pre-existing tests passing against the refactor **is** the no-behavior-change evidence required by spec §8 criterion 3.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-apparent/src/aberration.rs
git commit -m "refactor(aberration): extract earth_orbit_elements

Pure extraction of the Meeus 25.4 element polynomials out of
annual_aberration. Identical expressions and evaluation order; no
runtime result change. Gives the polynomial coefficients a direct
test seam for FU-9 slice 4."
```

---

### Task 2: Relocate the test module to a co-located file

**Files:**
- Modify: `crates/pleiades-apparent/src/aberration.rs` (replace inline `mod tests { … }`)
- Create: `crates/pleiades-apparent/src/aberration/tests.rs`

**Interfaces:**
- Consumes: `earth_orbit_elements` from Task 1 (in scope via `use super::*`, though not yet referenced by any test).
- Produces: the file `crates/pleiades-apparent/src/aberration/tests.rs`, into which Tasks 3 and 4 append tests.

This matches the layout already used by `apparent/tests.rs`, `nutation/tests.rs`, and `refraction/tests.rs`, per AGENTS.md ("keep large inline test suites out of the file under test").

- [ ] **Step 1: Create the co-located test file**

Create `crates/pleiades-apparent/src/aberration/tests.rs` containing the module docstring plus the two **unmodified** pre-existing tests moved verbatim out of `aberration.rs`:

```rust
//! White-box unit tests for the annual-aberration module.
//!
//! Relocated out of `aberration.rs` per AGENTS.md ("keep large inline test
//! suites out of the file under test"). These remain white-box unit tests with
//! access to the module's private helpers (`julian_centuries`,
//! `earth_orbit_elements`) — they are deliberately not converted into
//! black-box integration tests.

use super::*;

#[test]
fn magnitude_is_bounded_by_kappa_over_cos_beta() {
    // For modest latitudes Δλ stays within a few × κ; never explosive.
    let off = annual_aberration(100.0, 2.0, 280.0, 2_451_545.0);
    assert!(
        off.d_lambda_arcsec.abs() < 25.0,
        "Δλ = {}",
        off.d_lambda_arcsec
    );
    assert!(off.d_beta_arcsec.abs() < 1.0, "Δβ = {}", off.d_beta_arcsec);
}

#[test]
fn sign_and_magnitude_match_known_geometry() {
    // Physically valid sign/magnitude checks at J2000 (t=0), β=0 so cosβ=1.
    // The dominant term is -κ cos(⊙-λ); the e·κ term adds ≈ +0.34″.
    // (The earlier "Venus" example used an impossible geometry — Venus
    // 180° from the Sun — and an inconsistent expected value; replaced with
    // valid geometries. Precision is gated end-to-end against Horizons in
    // Task 14.)

    // Conjunction (body at the Sun's longitude, ⊙-λ = 0): Δλ ≈ -κ + 0.34 ≈ -20.15″.
    let conj = annual_aberration(100.0, 0.0, 100.0, 2_451_545.0);
    assert!(
        conj.d_lambda_arcsec < 0.0,
        "conjunction Δλ should be negative: {}",
        conj.d_lambda_arcsec
    );
    assert!(
        (conj.d_lambda_arcsec - (-20.15)).abs() < 0.2,
        "conjunction Δλ = {}",
        conj.d_lambda_arcsec
    );

    // Opposition (body opposite the Sun, ⊙-λ = 180): Δλ ≈ +κ + 0.34 ≈ +20.84″.
    let opp = annual_aberration(100.0, 0.0, 280.0, 2_451_545.0);
    assert!(
        opp.d_lambda_arcsec > 0.0,
        "opposition Δλ should be positive: {}",
        opp.d_lambda_arcsec
    );
    assert!(
        (opp.d_lambda_arcsec - 20.84).abs() < 0.2,
        "opposition Δλ = {}",
        opp.d_lambda_arcsec
    );

    // Quadrature (⊙-λ = 90): the main term vanishes; only the small e·κ term
    // remains (< 1″), and Δβ stays bounded by κ off the ecliptic.
    let quad = annual_aberration(100.0, 10.0, 190.0, 2_451_545.0);
    assert!(
        quad.d_lambda_arcsec.abs() < 1.0,
        "quadrature Δλ = {}",
        quad.d_lambda_arcsec
    );
    assert!(
        quad.d_beta_arcsec.abs() < KAPPA_ARCSEC,
        "quadrature Δβ = {}",
        quad.d_beta_arcsec
    );
}
```

- [ ] **Step 2: Replace the inline module in `aberration.rs`**

Delete the entire `#[cfg(test)] mod tests { … }` block at the end of `crates/pleiades-apparent/src/aberration.rs` and replace it with the two-line declaration:

```rust
#[cfg(test)]
mod tests;
```

- [ ] **Step 3: Verify the relocation is inert**

Run:
```bash
cargo nextest run -p pleiades-apparent aberration
cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings
cargo fmt --all --check
```
Expected: the same 2 tests PASS, clippy clean, fmt clean. Test count must be unchanged — if a test disappeared, the module wiring is wrong.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-apparent/src/aberration.rs crates/pleiades-apparent/src/aberration/tests.rs
git commit -m "test(aberration): relocate tests to co-located tests.rs

Matches the apparent/nutation/refraction layout per AGENTS.md. Tests
moved verbatim; no assertion changed."
```

---

### Task 3: Pin `julian_centuries` and `earth_orbit_elements` (Groups A, B, C — 16 mutants)

**Files:**
- Modify: `crates/pleiades-apparent/src/aberration/tests.rs` (append two tests)

**Interfaces:**
- Consumes: `julian_centuries(jd_tt: f64) -> f64` (pre-existing, private) and `earth_orbit_elements(t: f64) -> (f64, f64)` from Task 1.
- Produces: nothing consumed by later tasks.

- [ ] **Step 1: Append the `julian_centuries` test**

Append to `crates/pleiades-apparent/src/aberration/tests.rs`:

```rust
#[test]
fn julian_centuries_counts_from_j2000_in_units_of_36525_days() {
    // 2451545.0 is J2000 itself -> t = 0.
    assert!(
        (julian_centuries(2_451_545.0) - 0.0).abs() < 1e-15,
        "t(J2000) = {}",
        julian_centuries(2_451_545.0)
    );

    // 2469807.5 = J2000 + 18262.5 d = J2000 + half a Julian century.
    // A *half* century is deliberate: t = 1.0 would be indistinguishable
    // from the `replace julian_centuries -> 1.0` whole-function mutant,
    // and t = 0 alone is indistinguishable from `-> 0.0`.
    assert!(
        (julian_centuries(2_469_807.5) - 0.5).abs() < 1e-15,
        "t(J2000 + 18262.5 d) = {}",
        julian_centuries(2_469_807.5)
    );
}
```

- [ ] **Step 2: Append the element-polynomial test**

Append to the same file:

```rust
#[test]
fn earth_orbit_elements_match_meeus_25_4() {
    // Meeus 25.4, evaluated OUTSIDE this code:
    //   e(t) = 0.016708634 - 0.000042037 t - 0.0000001267 t^2
    //   ϖ(t) = 102.93735   + 1.71946 t     + 0.00046 t^2
    //
    // Three epochs are required, not one. At t = 0 only the lead constants
    // are exercised. Evaluating at both +1 and -1 separates the linear term
    // (which flips sign) from the quadratic (which does not) — that is what
    // distinguishes a mutated quadratic coefficient from a mutated linear one.
    for (t, e_expected, pi_expected) in [
        (0.0, 0.016_708_634, 102.937_35),
        (1.0, 0.016_666_470_3, 104.657_27),
        (-1.0, 0.016_750_544_3, 101.218_35),
    ] {
        let (e, pi_deg) = earth_orbit_elements(t);
        assert!(
            (e - e_expected).abs() < 1e-15,
            "e({t}) = {e}, expected {e_expected}"
        );
        assert!(
            (pi_deg - pi_expected).abs() < 1e-12,
            "ϖ({t}) = {pi_deg}, expected {pi_expected}"
        );
    }
}
```

Tolerance rationale: the tightest mutant margin in this group is `2.5e-7` for `e` (the quadratic sign swap) and `9.2e-4` for `ϖ`. The `1e-15` / `1e-12` tolerances sit ~8 orders below those margins while staying above f64 rounding noise.

- [ ] **Step 3: Run the tests**

Run:
```bash
cargo nextest run -p pleiades-apparent aberration
```
Expected: PASS, 4 tests. (Per the TDD-inversion note, these pass immediately — the real evidence is Step 4.)

- [ ] **Step 4: Verify the mutants died**

Run `MUTANTS_CMD`:
```bash
cargo mutants -p pleiades-apparent --test-tool nextest \
  --test-workspace=false --file crates/pleiades-apparent/src/aberration.rs
```
Expected: **12 missed** (down from 28). All five `aberration.rs:19` survivors and all eleven `:36`/`:37` survivors must be gone; the twelve remaining MISSED lines must all be at `:46` or `:47`.

If any `:19`, `:36`, or `:37` mutant is still MISSED, stop and diagnose — do not proceed to Task 4.

Note: the extracted `earth_orbit_elements` may add newly-generated mutants of its own at its new line numbers (the polynomials moved). These are the same expressions and are covered by the same test; they must show as caught, not missed.

- [ ] **Step 5: Run formatting and lints**

Run:
```bash
cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings
cargo fmt --all --check
```
Expected: clean. Watch specifically for `clippy::excessive_precision` on the float literals — every literal above is a shortest-round-trip f64 representation, so it should not fire; if it does, trim the literal to the reported precision rather than allowing the lint.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-apparent/src/aberration/tests.rs
git commit -m "test(aberration): pin julian_centuries and Meeus 25.4 elements

Kills the 16 FU-9 survivors in the epoch conversion and the element
polynomials. Expected values are the published Meeus 25.4 coefficients
evaluated outside the code at t = 0, +1, -1; the +1/-1 pair separates
the linear term from the quadratic."
```

---

### Task 4: Pin the Meeus 23.2 formula lines (Group D — 12 mutants)

**Files:**
- Modify: `crates/pleiades-apparent/src/aberration/tests.rs` (append two tests)

**Interfaces:**
- Consumes: `annual_aberration(lambda_deg: f64, beta_deg: f64, sun_true_longitude_deg: f64, jd_tt: f64) -> AberrationOffset`, with fields `d_lambda_arcsec: f64` and `d_beta_arcsec: f64` (all pre-existing public API).
- Produces: nothing consumed by later tasks.

- [ ] **Step 1: Append the crafted-geometry test**

Append to `crates/pleiades-apparent/src/aberration/tests.rs`:

```rust
#[test]
fn meeus_23_2_matches_independent_evaluation_at_crafted_geometry() {
    // Crafted discriminating geometry (design §6):
    //   λ = 30°, β = 60°, ⊙ = 120°, jd = J2000 (t = 0)
    //
    // Chosen so that:
    //   cos β = 0.5   -> `/ cos_beta` and `* cos_beta` differ by a factor of 4
    //   ⊙ - λ = 90°   -> cos = 0, sin = 1, isolating the e·κ term in Δλ
    //   λ ≠ 0         -> otherwise ϖ + λ ≡ ϖ - λ and the `-` → `+` mutant lives
    //   λ ≠ ϖ         -> otherwise sin(ϖ - λ) = 0, annihilating the whole
    //                    `e sin(ϖ - λ)` subtraction and letting its mutants live
    //   β ≠ 0         -> Δβ is non-zero, so its sign is assertable
    //
    // Expected values evaluated OUTSIDE this code from Meeus 23.2, using the
    // t = 0 elements e = 0.016708634 and ϖ = 102.93735, so ϖ - λ = 72.93735°:
    //   Δλ = (0 + 0.34245214231967996 * 0.29341719999540683) / 0.5
    //      = 0.20096269746373557
    //   Δβ = -20.49552 * sin(60°) * (1 - 0.016708634 * 0.9559844908505867)
    //      = -17.46612250773869
    let off = annual_aberration(30.0, 60.0, 120.0, 2_451_545.0);

    assert!(
        (off.d_lambda_arcsec - 0.200_962_697_463_735_57).abs() < 1e-9,
        "Δλ = {}",
        off.d_lambda_arcsec
    );
    assert!(
        (off.d_beta_arcsec - (-17.466_122_507_738_69)).abs() < 1e-9,
        "Δβ = {}",
        off.d_beta_arcsec
    );
}
```

Tolerance rationale: `1e-9` is ~6 orders above worst-case f64 noise (`cos 60°` is `0.5000000000000001`, `cos 90°` is `6.12e-17`, not bitwise exact) and ~6 orders below the smallest mutant margin at this geometry (`0.0267″`, from `e * sin(…)` → `e / sin(…)`).

- [ ] **Step 2: Append the Δβ sign test**

Append to the same file:

```rust
#[test]
fn aberration_in_latitude_is_signed_not_merely_bounded() {
    // The pre-existing tests assert only |Δβ| against a bound, which leaves
    // the leading `-` of the Δβ formula completely unconstrained. Pin the
    // sign directly: at β = +60° with ⊙ - λ = +90°, sin β > 0 and the
    // bracket (1 - e sin(ϖ-λ)) > 0, so Δβ must be strictly negative.
    let off = annual_aberration(30.0, 60.0, 120.0, 2_451_545.0);
    assert!(
        off.d_beta_arcsec < -17.0,
        "Δβ should be strictly negative here: {}",
        off.d_beta_arcsec
    );

    // sin β is odd, so mirroring the ecliptic latitude must negate Δβ exactly
    // while leaving Δλ (which depends on β only through the even cos β) alone.
    let mirrored = annual_aberration(30.0, -60.0, 120.0, 2_451_545.0);
    assert!(
        (mirrored.d_beta_arcsec + off.d_beta_arcsec).abs() < 1e-12,
        "Δβ(-β) should negate Δβ(+β): {} vs {}",
        mirrored.d_beta_arcsec,
        off.d_beta_arcsec
    );
    assert!(
        (mirrored.d_lambda_arcsec - off.d_lambda_arcsec).abs() < 1e-12,
        "Δλ should be even in β: {} vs {}",
        mirrored.d_lambda_arcsec,
        off.d_lambda_arcsec
    );
}
```

- [ ] **Step 3: Run the tests**

Run:
```bash
cargo nextest run -p pleiades-apparent aberration
```
Expected: PASS, 6 tests.

- [ ] **Step 4: Verify the mutants died**

Run `MUTANTS_CMD`:
```bash
cargo mutants -p pleiades-apparent --test-tool nextest \
  --test-workspace=false --file crates/pleiades-apparent/src/aberration.rs
```
Expected: **0 missed.**

If any survivor remains, do not add an assertion that merely pins the function's own output to force it green. Instead determine whether it is a genuine equivalent mutant (no reachable input distinguishes it) and carry it into Task 5 as a documented residual.

- [ ] **Step 5: Run formatting and lints**

Run:
```bash
cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings
cargo fmt --all --check
```
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-apparent/src/aberration/tests.rs
git commit -m "test(aberration): pin Meeus 23.2 at a discriminating geometry

Kills the 12 FU-9 survivors on the Delta-lambda and Delta-beta formula
lines. The geometry (lam=30, beta=60, sun=120) avoids the two
degeneracies (lam=0 and lam=perihelion) that would let the Delta-beta
operator mutants survive, and pins the Delta-beta sign that the previous
abs()-only assertions left free."
```

---

### Task 5: Verify the whole slice and record the result

**Files:**
- Modify: `docs/follow-ups.md` (append a slice-4 progress paragraph to the FU-9 entry)

**Interfaces:**
- Consumes: the final `MUTANTS_CMD` result from Task 4.
- Produces: nothing.

- [ ] **Step 1: Confirm no gate file was touched**

Run:
```bash
git diff --name-only main...HEAD
```
Expected: exactly these four paths, and nothing under `crates/pleiades-validate/`:
```
crates/pleiades-apparent/src/aberration.rs
crates/pleiades-apparent/src/aberration/tests.rs
docs/superpowers/plans/2026-07-20-fu9-aberration-mutant-triage.md
docs/superpowers/specs/2026-07-20-fu9-aberration-mutant-triage-design.md
```
(`docs/follow-ups.md` joins this list after Step 3.)

- [ ] **Step 2: Run the full blocking CI tier**

Run:
```bash
mise run ci
```
Expected: green.

- [ ] **Step 3: Record the slice in `docs/follow-ups.md`**

Append a new paragraph to the FU-9 entry, immediately after the `Progress (2026-07-20) — pleiades-apparent/src/refraction.rs` paragraph and before the closing `---`. Use this text, substituting the actual final survivor count if it was not 0:

```markdown
**Progress (2026-07-20) — `pleiades-apparent/src/aberration.rs`:** triaged from
`28` → `0` surviving mutants (spec/plan:
`docs/superpowers/specs/2026-07-20-fu9-aberration-mutant-triage-design.md`).
Baseline confirmed by the authoritative per-file command (`56 mutants tested,
28 missed, 27 caught, 1 unviable`). The distinguishing finding of this slice is
that **11 of the 28 survivors were arithmetically unreachable through the
public API**: the Earth-orbit elements `e` and `ϖ` enter the output only via the
~0.34″ `e κ cos(ϖ - λ)` term, so mutating their polynomial coefficients moves
Δλ by only ~0.001″ (`e`) to ~0.006″ (`ϖ`) — below any tolerance the model's own
accuracy justifies. Killing them without pinning the function's own output
therefore required a testability seam, so a minimal **behavior-preserving
refactor** (its own commit, no runtime-result change) extracted
`earth_orbit_elements(t) -> (e, pi_deg)`; the polynomials are now asserted
directly against Meeus 25.4 coefficients evaluated outside the code at
`t = 0, +1, -1` (the `±1` pair is what separates the linear term, which flips
sign, from the quadratic, which does not). `julian_centuries` needed no refactor
— it was already a seam with no test, and every prior test passed the J2000
epoch, the one input where `t = 0` is indistinguishable from the `-> 0.0`
whole-function mutant; a single half-century epoch (`2469807.5` → `t = 0.5`)
kills all five. The remaining 12 formula-line survivors were genuine coverage
holes — notably every prior Δβ assertion used `.abs()` against a bound, leaving
the sign free — and fall to one **crafted discriminating geometry**
(`λ = 30°, β = 60°, ⊙ = 120°`, `t = 0`) which makes `cos β = 0.5` (so
`/cos_beta` and `*cos_beta` differ 4×) while avoiding two degeneracies that a
more obvious choice would hit: `λ = ϖ` zeroes `sin(ϖ - λ)` and lets the
bracket-minus mutant survive bit-identically, and `λ = 0` makes `ϖ + λ ≡ ϖ - λ`.
Both rejected geometries are recorded in the design so they are not
re-proposed. **No documented residual this slice** — like `apparent.rs`, and
unlike `nutation.rs` (1) and `refraction.rs` (3), `aberration.rs` reached a
genuine `0`, so nothing was suppressed or excused. No parity gate was touched;
the tier stays report-only; `mise run ci` is green. **Remaining slices**
(priority order): `topocentric.rs` (27), `sidereal.rs` (17), `precession.rs`
(17), `lighttime.rs` (5), then the `pleiades-time` and `pleiades-types`
survivors.
```

Also update the **"Remaining slices"** sentence in each of the three earlier progress paragraphs? **No** — leave them untouched. They are historical records of what was remaining at the time each slice landed; rewriting them would falsify the log.

- [ ] **Step 4: Commit**

```bash
git add docs/follow-ups.md
git commit -m "docs(follow-ups): record FU-9 aberration.rs triage (28 -> 0)"
```

- [ ] **Step 5: Push and open the PR**

```bash
git push -u origin fu9-aberration-mutant-triage
gh pr create --title "test(aberration): FU-9 aberration.rs mutant triage (28 -> 0)" --body "$(cat <<'BODY'
FU-9 slice 4. Drives `crates/pleiades-apparent/src/aberration.rs` from 28
surviving cargo-mutants mutants to 0.

- **Task 1** extracts `earth_orbit_elements` (behavior-preserving, own commit).
  11 of the 28 survivors sit in the `e`/`ϖ` polynomials, which reach the public
  output only through a ~0.34″ term — a coefficient mutation moves Δλ by
  0.001-0.006″, so no honest end-to-end tolerance can catch them. The seam is
  what makes them testable without pinning the function's own output.
- **Tasks 3-4** add white-box tests: Meeus 25.4 coefficients evaluated outside
  the code at t = 0, ±1, and one crafted geometry (λ=30°, β=60°, ⊙=120°) proven
  to discriminate all 12 formula-line mutants.

No parity gate touched; the mutants tier stays report-only. No
`#[mutants::skip]` added.

Spec: `docs/superpowers/specs/2026-07-20-fu9-aberration-mutant-triage-design.md`
BODY
)"
```

---

## Acceptance Criteria (whole slice)

Mirrors spec §8:

1. `MUTANTS_CMD` reports **0 missed**, or only mutants documented as equivalent in `docs/follow-ups.md`.
2. `mise run ci` is green.
3. The refactor commit (Task 1) changes no numeric result — evidenced by the pre-existing tests passing against it unmodified, before any new test was added.
4. No `validate-*` gate file is touched (verified in Task 5 Step 1).
5. No `#[mutants::skip]` is added anywhere.
