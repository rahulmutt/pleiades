# FU-9 Slice 3 — `refraction.rs` Mutant Triage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive `crates/pleiades-apparent/src/refraction.rs` from 37 surviving cargo-mutants mutants to 0 killable survivors plus 3 documented equivalent mutants, using tests only.

**Architecture:** Relocate the module's inline test suite to a co-located `refraction/tests.rs`, then add white-box unit tests against the private formula helpers and both below-horizon blend functions. Every expected value is an independently computed literal (published Bennett/Saemundsson formulas evaluated outside the code), never captured from the code under test. No runtime source changes.

**Tech Stack:** Rust (stable, per `mise.toml`), `cargo-nextest`, `cargo-mutants` 27.1.0, `mise` task runner.

**Spec:** [`docs/superpowers/specs/2026-07-20-fu9-refraction-mutant-triage-design.md`](../specs/2026-07-20-fu9-refraction-mutant-triage-design.md)

**Branch:** `fu9-refraction-mutant-triage` (already created; design doc committed as `1329e0dd0`)

## Global Constraints

- **Tests-only slice.** The ONLY permitted edit to `crates/pleiades-apparent/src/refraction.rs` is replacing its inline `#[cfg(test)] mod tests { ... }` block with the one-line declaration `#[cfg(test)] mod tests;`. No formula, constant, signature, or control-flow change.
- **No parity/release gate tolerance may be changed**, including the existing SE corpus tolerances inside `refraction_matches_se_below_horizon`.
- **No `#[mutants::skip]` attributes.** Equivalent mutants are documented in comments and in `docs/follow-ups.md`, left visible to the tool.
- **Independence discipline.** Every expected literal in a new test must trace to a crafted-exact input or to the independent evaluation recorded in this plan. No literal may be captured by running the function under test.
- **Mutation tier stays report-only.** Nothing in this slice wires mutation score into a blocking gate.
- Code must be `rustfmt`-clean and pass `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Existing tests move unchanged and remain white-box unit tests; they are not converted to integration tests.

## Reference Values (independently computed)

These were produced by evaluating the **published** formulas outside the codebase —
`scale = (p/1010) * (283/(273+t))`, Bennett `R = scale * 1.02 / tan(h + 10.3/(h+5.11))`,
Saemundsson `R = scale * 1.0 / tan(h + 7.31/(h+4.4))` (arcmin, `h` in degrees) —
and the blend spec `anchor = R(-1)/60`, `fade = (h+10)/9`.

Two atmospheres are used because they make `scale` exact by construction:

| Name | Pressure | Temp | `scale` |
|---|---|---|---|
| `EXACT` | 1010.0 mbar | 10.0 °C | `1.0` exactly (both factors unity) |
| `DENSE` | 2020.0 mbar | 25.0 °C | `1.8993288590604027` (both factors ≠ 1, distinct) |

Independently computed expected values:

| Quantity | `EXACT` | `DENSE` |
|---|---|---|
| `bennett(-0.5)` arcmin | `33.687796094672827` | — |
| `bennett(-1.0)` arcmin | `38.794837252861491` | `73.68415397691142` |
| `saemundsson(-0.5)` arcmin | `41.681097299305712` | — |
| `saemundsson(-1.0)` arcmin | `49.815726359405957` | `94.61644670947574` |
| `apparent_from_true(-0.5)` | `0.061463268244547065` | — |
| `apparent_from_true(-1.0)` | `-0.35341937911897514` | — |
| `apparent_from_true(-5.5)` | `-5.1767096895594875` | `-4.885965383525738` |
| `true_from_apparent(-0.5)` | `-1.1946849549884284` | — |
| `true_from_apparent(-1.0)` | `-1.8302621059900992` | — |
| `true_from_apparent(-5.5)` | `-5.91513105299505` | `-6.2884703892456315` |

`fade(-5.5) = (-5.5 + 10)/(-1 + 10) = 4.5/9 = 0.5` exactly, so the −5.5 cases pin the
anchor scaling with no rounding slack. At `h <= -10.0` both directions return `h`
unchanged, asserted with exact equality.

**Note for the implementer:** these literals are the *reference*, and the current code
agrees with all of them bit-for-bit. Your new tests should therefore **pass on first
run** — that is the expected outcome, not a red flag (§ "TDD inversion" below).

**Literal formatting note:** the values above are shortest-round-trip `f64`
representations, so `clippy::excessive_precision` should not fire. A prior FU-9
slice did hit that lint on test literals (`7fab65f14`), so if clippy flags one,
trim the trailing digits it names — do not change the value's magnitude, and
re-run the test to confirm it still passes at the stated tolerance. `cargo fmt`
will also re-group the `_` digit separators; accept its formatting.

## TDD inversion for mutation triage

Normal TDD writes a failing test for absent behavior. This slice tests behavior that
already exists and is already correct, so the red/green cycle is inverted:

- **The "red" is the surviving mutant**, not a failing test.
- A new test that *fails* on unmutated code means the literal is wrong or was
  mistyped — stop and investigate; do not "fix" `refraction.rs` to match.
- **Verification is the `cargo mutants` re-run**, where the missed count must drop.

Each task therefore runs the test suite (expect PASS) *and* a scoped mutants re-run
(expect the missed count to fall to a stated number).

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `crates/pleiades-apparent/src/refraction.rs` | Modify (1 line) | Replace inline test module with `#[cfg(test)] mod tests;` |
| `crates/pleiades-apparent/src/refraction/tests.rs` | Create | All white-box unit tests for the module: relocated existing tests plus new coverage |
| `docs/follow-ups.md` | Modify | Record slice result in the FU-9 entry |

## Survivor → Task Map

All 37 survivors from the authoritative baseline run
(`101 mutants tested in 2m: 37 missed, 64 caught`) are assigned:

| Task | Survivors killed | Running missed count |
|---|---|---|
| Task 1 (relocation) | 0 (no behavior change) | 37 |
| Task 2 (`scale`, Bennett, Saemundsson) | 3 | 34 |
| Task 3 (`apparent_from_true_below_horizon` + blend constants) | 8 | 26 |
| Task 4 (`true_from_apparent_below_horizon` + dispatcher) | 23 | 3 |
| Task 5 (document equivalents, follow-ups) | 0 (3 documented equivalents remain) | 3 |

---

### Task 1: Relocate the test module to a co-located file

**Files:**
- Modify: `crates/pleiades-apparent/src/refraction.rs:153-235` (the whole `#[cfg(test)] mod tests { ... }` block)
- Create: `crates/pleiades-apparent/src/refraction/tests.rs`

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces: `crates/pleiades-apparent/src/refraction/tests.rs` containing `use super::*;`, which gives every later task white-box access to the private items `scale`, `bennett_refraction_arcmin`, `saemundsson_refraction_arcmin`, `apparent_from_true_below_horizon`, `true_from_apparent_below_horizon`, `BELOW_HORIZON_BLEND_START_DEG`, `BELOW_HORIZON_BLEND_END_DEG`, plus the public `Atmosphere`, `apparent_from_true`, `true_from_apparent`.

This task is a pure move: no test is added, removed, or edited. It exists as its own
task so a reviewer can confirm the relocation is behavior-neutral before new tests
land on top of it.

- [ ] **Step 1: Create the new test file with the existing tests moved verbatim**

Create `crates/pleiades-apparent/src/refraction/tests.rs` with exactly the body of the
current inline module (the five existing tests, unchanged, including their comments):

```rust
//! White-box unit tests for the refraction module.
//!
//! Relocated out of `refraction.rs` per AGENTS.md ("keep large inline test
//! suites out of the file under test"). These remain white-box unit tests with
//! access to the module's private helpers — they are deliberately not converted
//! into black-box integration tests.

use super::*;

#[test]
fn default_atmosphere_is_se_standard() {
    let a = Atmosphere::default();
    assert_eq!(a.pressure_mbar, 1013.25);
    assert_eq!(a.temperature_c, 15.0);
}

#[test]
fn refraction_at_horizon_is_about_34_arcmin() {
    // Bennett, evaluated ON the true altitude at h=0 with standard
    // atmosphere, gives a true→apparent lift of ~29' (0.4752° ≈ 28.5').
    // The ~34' figure (0.567°) belongs to the *other* direction — see
    // `true_from_apparent_at_horizon_is_about_negative_34_arcmin` below,
    // which evaluates Saemundsson on the apparent altitude at h=0. Assert
    // the apparent altitude sits ~29' above 0 within a loose band.
    let app = apparent_from_true(0.0, Atmosphere::default());
    assert!(
        (app - 0.4752).abs() < 0.05,
        "apparent horizon altitude {app}"
    );
}

#[test]
fn refraction_vanishes_at_zenith() {
    let app = apparent_from_true(90.0, Atmosphere::default());
    assert!((app - 90.0).abs() < 1e-4, "zenith {app}");
}

#[test]
fn saemundsson_inverts_bennett_within_a_few_arcsec() {
    // Round-trip: for altitudes above the horizon the two formulae are near-inverses.
    for h in [5.0, 15.0, 45.0, 80.0] {
        let app = apparent_from_true(h, Atmosphere::default());
        let back = true_from_apparent(app, Atmosphere::default());
        assert!((back - h).abs() < 0.01, "round-trip h={h} back={back}");
    }
}

#[test]
fn true_from_apparent_at_horizon_is_about_negative_34_arcmin() {
    // A body seen ON the apparent horizon (h_app=0) is geometrically ~34' below it.
    let t = true_from_apparent(0.0, Atmosphere::default());
    assert!(
        (t + 0.5667).abs() < 0.02,
        "true altitude at apparent horizon {t}"
    );
}

#[test]
fn refraction_matches_se_below_horizon() {
    // Pinned from `crates/pleiades-validate/data/rise-trans-corpus/azalt.csv`
    // (`se_true_alt_deg < 0` rows; standard atmosphere, `swe_azalt` ->
    // `swe_refrac_extended` ground truth). SE reports `se_apparent_alt_deg
    // == se_true_alt_deg` (refraction fully suppressed) for every one of
    // these — see `apparent_from_true_below_horizon`'s doc for why this
    // module approximates rather than exactly reproduces SE's own
    // (discontinuous) below-horizon model. The shallowest row (-9.96 deg)
    // sits right at the edge of the fade and is pinned within 15 arcsec;
    // every deeper row is pinned within a fraction of an arcsec. Both are
    // a large improvement over the pre-fix ~282 arcsec worst case.
    let atmos = Atmosphere::default();
    for (true_alt, se_apparent_alt, tolerance_arcsec) in [
        (-9.964249, -9.964249, 15.0),
        (-15.874977, -15.874977, 0.01),
        (-34.289902, -34.289902, 0.01),
        (-43.529313, -43.529313, 0.01),
        (-60.896360, -60.896360, 0.01),
        (-64.642565, -64.642565, 0.01),
        (-70.739219, -70.739219, 0.01),
    ] {
        let app = apparent_from_true(true_alt, atmos);
        let residual_arcsec = (app - se_apparent_alt).abs() * 3600.0;
        assert!(
            residual_arcsec < tolerance_arcsec,
            "true={true_alt} app={app} se={se_apparent_alt} residual={residual_arcsec}\""
        );
    }
}
```

- [ ] **Step 2: Replace the inline module in `refraction.rs` with a declaration**

Delete lines 153–235 of `crates/pleiades-apparent/src/refraction.rs` (the entire
`#[cfg(test)] mod tests { ... }` block, from `#[cfg(test)]` through the final closing
brace) and put this in their place, as the last item in the file:

```rust
#[cfg(test)]
mod tests;
```

Nothing else in the file changes.

- [ ] **Step 3: Run the module's tests to verify the move is behavior-neutral**

Run: `cargo nextest run -p pleiades-apparent refraction`

Expected: PASS — 6 tests run, 0 failed. The same six test names that existed before
the move (`default_atmosphere_is_se_standard`, `refraction_at_horizon_is_about_34_arcmin`,
`refraction_vanishes_at_zenith`, `saemundsson_inverts_bennett_within_a_few_arcsec`,
`true_from_apparent_at_horizon_is_about_negative_34_arcmin`,
`refraction_matches_se_below_horizon`) must all appear.

- [ ] **Step 4: Verify formatting and lints**

Run: `cargo fmt --all --check && cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings`

Expected: no output from `fmt`, and clippy finishes with no warnings.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-apparent/src/refraction.rs crates/pleiades-apparent/src/refraction/tests.rs
git commit -m "test(apparent): relocate refraction tests to co-located test file"
```

---

### Task 2: Pin `scale`, Bennett, and Saemundsson against independent literals

**Files:**
- Modify: `crates/pleiades-apparent/src/refraction/tests.rs`

**Interfaces:**
- Consumes: the test file and `use super::*;` from Task 1.
- Produces: the constants `EXACT` and `DENSE` (both `Atmosphere`) and the tolerance
  constants `TOL_DEG` and `TOL_ARCMIN`, all used by Tasks 3 and 4.

Kills 3 survivors: `26:5 replace scale -> f64 with 1.0`, `26:36 replace * with / in scale`,
`47:18 replace * with / in bennett_refraction_arcmin`.

Why the current tests miss them: every existing assertion uses the default atmosphere,
whose `scale` is `0.9858` — within all current tolerances of `1.0`, so the
`scale -> 1.0` stub hides. And the `* with /` swaps shift results by only ~3.5–4%
(≈68 arcsec on Bennett), which fits inside the existing ~180 arcsec assertion band.
The fix is a non-unit `scale` plus tight literal assertions.

- [ ] **Step 1: Add the shared fixtures and the formula tests**

Append to `crates/pleiades-apparent/src/refraction/tests.rs`:

```rust
/// Atmosphere crafted so `scale` is exactly `1.0`: `1010/1010 = 1` and
/// `283/(273+10) = 1`. Lets a refraction literal be compared without any
/// scaling factor folded in.
const EXACT: Atmosphere = Atmosphere {
    pressure_mbar: 1010.0,
    temperature_c: 10.0,
};

/// Atmosphere where BOTH scale factors differ from 1 and from each other
/// (`2020/1010 = 2`, `283/298 = 0.9497`), so no operator swap inside `scale`
/// can alias another and still produce the right answer.
const DENSE: Atmosphere = Atmosphere {
    pressure_mbar: 2020.0,
    temperature_c: 25.0,
};

/// Tolerance for degree-valued altitude assertions. Values here are O(1) and
/// f64 carries ~1e-16 relative precision; 1e-11 absorbs any last-ULP `tan()`
/// variation between platform libm implementations while staying far tighter
/// than the smallest mutant-induced shift (~1e-3 deg).
const TOL_DEG: f64 = 1e-11;

/// Same rationale, for arcminute-valued refraction assertions (values O(10)).
const TOL_ARCMIN: f64 = 1e-10;

#[test]
fn scale_matches_independent_pressure_temperature_ratio() {
    // scale = (p/1010) * (283/(273+t)), evaluated independently.
    // EXACT is constructed so both factors are exactly 1.
    assert_eq!(scale(EXACT), 1.0);
    // Pressure doubled, temperature factor still exactly 1.
    assert_eq!(
        scale(Atmosphere {
            pressure_mbar: 2020.0,
            temperature_c: 10.0,
        }),
        2.0
    );
    // Both factors non-unit: 2 * (283/298) = 1.8993288590604027. This case is
    // what distinguishes `*` from `/` between the two factors — with a unity
    // second factor the swap would be invisible.
    assert!(
        (scale(DENSE) - 1.899_328_859_060_402_7).abs() < 1e-15,
        "dense scale {}",
        scale(DENSE)
    );
}

#[test]
fn bennett_matches_independently_evaluated_formula() {
    // R = scale * 1.02 / tan(h + 10.3/(h + 5.11)) arcmin, h in degrees,
    // evaluated outside this crate from the published Bennett (1982) formula.
    for (h, atmos, expected_arcmin) in [
        (-0.5, EXACT, 33.687_796_094_672_827),
        (-1.0, EXACT, 38.794_837_252_861_491),
        (-1.0, DENSE, 73.684_153_976_911_42),
    ] {
        let got = bennett_refraction_arcmin(h, atmos);
        assert!(
            (got - expected_arcmin).abs() < TOL_ARCMIN,
            "bennett h={h} got={got} expected={expected_arcmin}"
        );
    }
}

#[test]
fn saemundsson_matches_independently_evaluated_formula() {
    // R = scale * 1.0 / tan(h + 7.31/(h + 4.4)) arcmin, h in degrees,
    // evaluated outside this crate from the published Saemundsson (1986)
    // formula.
    //
    // Documented equivalent mutant: `replace * with / in
    // saemundsson_refraction_arcmin` cannot be killed here or anywhere. The
    // operand is the literal `1.0`, so `scale * 1.0` and `scale / 1.0` are
    // bit-identical for every input, including non-finite ones. The `* 1.0` is
    // kept in the source because it mirrors the published coefficient in the
    // formula the rustdoc cites; it is not dead weight.
    for (h, atmos, expected_arcmin) in [
        (-0.5, EXACT, 41.681_097_299_305_712),
        (-1.0, EXACT, 49.815_726_359_405_957),
        (-1.0, DENSE, 94.616_446_709_475_74),
    ] {
        let got = saemundsson_refraction_arcmin(h, atmos);
        assert!(
            (got - expected_arcmin).abs() < TOL_ARCMIN,
            "saemundsson h={h} got={got} expected={expected_arcmin}"
        );
    }
}
```

- [ ] **Step 2: Run the new tests**

Run: `cargo nextest run -p pleiades-apparent refraction`

Expected: PASS — 9 tests, 0 failed. (If any of the three new tests FAILS, a literal was
mistyped; re-check it against the Reference Values table. Do not modify `refraction.rs`.)

- [ ] **Step 3: Verify the missed-mutant count dropped**

Run: `cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file refraction.rs`

Expected: the summary line reports **34 missed** (down from 37). Confirm via
`cat mutants.out/missed.txt` that these three no longer appear:
- `refraction.rs:26:5: replace scale -> f64 with 1.0`
- `refraction.rs:26:36: replace * with / in scale`
- `refraction.rs:47:18: replace * with / in bennett_refraction_arcmin`

- [ ] **Step 4: Verify formatting and lints**

Run: `cargo fmt --all --check && cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings`

Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-apparent/src/refraction/tests.rs
git commit -m "test(apparent): pin refraction scale + Bennett/Saemundsson to independent literals"
```

---

### Task 3: Cover the `apparent_from_true` below-horizon blend

**Files:**
- Modify: `crates/pleiades-apparent/src/refraction/tests.rs`

**Interfaces:**
- Consumes: `EXACT`, `DENSE`, `TOL_DEG` from Task 2.
- Produces: nothing new for later tasks.

Kills 8 survivors: `37:44 delete -`, `44:42 delete -`, `96:18 + with -`, `96:18 + with *`,
`96:56 / with %`, `96:56 / with *`, `103:42 - with +`, `104:7 + with -`.

Why the current tests miss them: the committed SE corpus rows only reach `h <= -9.96`,
where `fade ≈ 0.004` and the blend contributes ≈ 9 arcsec — under that row's 15 arcsec
tolerance. So the `h ∈ [-1, 0)` sub-branch and the fade slope are unconstrained, and the
two blend-boundary constants never determine an asserted answer. The new cases sample
each region: `-0.5` (full Bennett branch), `-1.0` (the `>=` boundary), `-5.5` (mid-fade,
`fade = 0.5` exactly), and `-10.0`/`-20.0` (the identity region).

- [ ] **Step 1: Add the blend tests**

Append to `crates/pleiades-apparent/src/refraction/tests.rs`:

```rust
#[test]
fn apparent_from_true_below_horizon_matches_blend_spec() {
    // The hold-then-fade blend is this crate's own model (SE's real
    // below-horizon behavior is a step discontinuity that was deliberately not
    // reproduced — see `apparent_from_true_below_horizon`'s rustdoc). Its
    // specification is therefore the reference: full Bennett down to -1 deg,
    // then the -1 deg refraction value faded linearly to zero at -10 deg.
    //
    // Expected values were evaluated independently from that spec plus the
    // published Bennett formula, with EXACT chosen so `scale` is 1.
    for (h, expected) in [
        // h in [-1, 0): full Bennett applied, no fade yet.
        (-0.5, 0.061_463_268_244_547_065),
        // h == -1: the >= boundary, still full Bennett.
        (-1.0, -0.353_419_379_118_975_14),
        // Mid-fade: fade = (-5.5 + 10)/(-1 + 10) = 0.5 exactly, so this pins
        // the anchor scaling with no rounding slack.
        (-5.5, -5.176_709_689_559_487_5),
    ] {
        let got = apparent_from_true(h, EXACT);
        assert!(
            (got - expected).abs() < TOL_DEG,
            "apparent_from_true h={h} got={got} expected={expected}"
        );
    }
}

#[test]
fn apparent_from_true_is_identity_at_and_below_fade_end() {
    // At/below BELOW_HORIZON_BLEND_END_DEG the blend has faded to exactly
    // zero, so the function returns its input unchanged — exact equality, no
    // tolerance needed.
    assert_eq!(
        apparent_from_true(BELOW_HORIZON_BLEND_END_DEG, EXACT),
        BELOW_HORIZON_BLEND_END_DEG
    );
    assert_eq!(apparent_from_true(-20.0, EXACT), -20.0);
    assert_eq!(apparent_from_true(-45.0, DENSE), -45.0);
}

#[test]
fn apparent_from_true_blend_scales_with_atmosphere() {
    // Same mid-fade altitude under a denser/warmer atmosphere: the anchor is
    // Bennett(-1) under DENSE, so the blend must carry the non-unit scale
    // through. Independently evaluated.
    let got = apparent_from_true(-5.5, DENSE);
    let expected = -4.885_965_383_525_738;
    assert!(
        (got - expected).abs() < TOL_DEG,
        "dense blend got={got} expected={expected}"
    );
}

#[test]
fn blend_boundary_constants_have_expected_signs_and_ordering() {
    // Guards the two constants directly: both are below the horizon and the
    // fade start sits above the fade end. A sign flip on either would invert
    // the fade denominator and silently reshape the whole below-horizon model.
    assert_eq!(BELOW_HORIZON_BLEND_START_DEG, -1.0);
    assert_eq!(BELOW_HORIZON_BLEND_END_DEG, -10.0);
    assert!(BELOW_HORIZON_BLEND_END_DEG < BELOW_HORIZON_BLEND_START_DEG);
    assert!(BELOW_HORIZON_BLEND_START_DEG < 0.0);
}
```

- [ ] **Step 2: Run the new tests**

Run: `cargo nextest run -p pleiades-apparent refraction`

Expected: PASS — 13 tests, 0 failed.

- [ ] **Step 3: Verify the missed-mutant count dropped**

Run: `cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file refraction.rs`

Expected: the summary reports **26 missed** (down from 34). Confirm via
`cat mutants.out/missed.txt` that no mutant on lines `37`, `44`, `96`, `103`, or `104`
remains.

- [ ] **Step 4: Verify formatting and lints**

Run: `cargo fmt --all --check && cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings`

Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-apparent/src/refraction/tests.rs
git commit -m "test(apparent): cover apparent_from_true below-horizon blend regions"
```

---

### Task 4: Cover `true_from_apparent_below_horizon` (the untested function)

**Files:**
- Modify: `crates/pleiades-apparent/src/refraction/tests.rs`

**Interfaces:**
- Consumes: `EXACT`, `DENSE`, `TOL_DEG` from Task 2.
- Produces: nothing new for later tasks.

Kills 23 survivors — the largest group. `true_from_apparent_below_horizon` currently has
**no test at all**: the existing SE corpus test exercises only the `apparent_from_true`
direction, so every mutant in this function survives, including all three whole-function
replacements (`-> 0.0`, `-> 1.0`, `-> -1.0`). This task mirrors Task 3's region sampling
for the opposite direction, and additionally kills the two killable dispatcher mutants at
line 147 (`< with ==`, `< with >`).

Region coverage and what each altitude is for:
- `-0.5` — the `h >= -1` branch (full Saemundsson); also kills `114:10 >= with <`, which
  would misroute this input into the blend.
- `-1.0` — the `>=` boundary, still full Saemundsson.
- `-5.5` — mid-fade (`fade = 0.5` exactly); the workhorse that kills the anchor, fade, and
  sign mutants on lines 120–123, plus both killable line-147 dispatcher mutants.
- `-10.0` / `-20.0` — the identity region; `-20.0` kills `117:10 <= with >`, which would
  otherwise apply an extrapolated negative fade there.

- [ ] **Step 1: Add the reverse-direction tests**

Append to `crates/pleiades-apparent/src/refraction/tests.rs`:

```rust
#[test]
fn true_from_apparent_below_horizon_matches_blend_spec() {
    // Mirror of `apparent_from_true_below_horizon_matches_blend_spec` for the
    // opposite direction: Saemundsson in place of Bennett, subtracted rather
    // than added. Same hold-then-fade spec, same independent evaluation.
    for (h, expected) in [
        // h in [-1, 0): full Saemundsson subtracted.
        (-0.5, -1.194_684_954_988_428_4),
        // h == -1: the >= boundary.
        (-1.0, -1.830_262_105_990_099_2),
        // Mid-fade: fade = 0.5 exactly.
        (-5.5, -5.915_131_052_995_05),
    ] {
        let got = true_from_apparent(h, EXACT);
        assert!(
            (got - expected).abs() < TOL_DEG,
            "true_from_apparent h={h} got={got} expected={expected}"
        );
    }
}

#[test]
fn true_from_apparent_is_identity_at_and_below_fade_end() {
    // As with the forward direction, the fade reaches exactly zero at
    // BELOW_HORIZON_BLEND_END_DEG and stays there.
    assert_eq!(
        true_from_apparent(BELOW_HORIZON_BLEND_END_DEG, EXACT),
        BELOW_HORIZON_BLEND_END_DEG
    );
    assert_eq!(true_from_apparent(-20.0, EXACT), -20.0);
    assert_eq!(true_from_apparent(-45.0, DENSE), -45.0);
}

#[test]
fn true_from_apparent_blend_scales_with_atmosphere() {
    // Mid-fade under DENSE: the anchor is Saemundsson(-1) scaled by 1.8993...,
    // independently evaluated.
    let got = true_from_apparent(-5.5, DENSE);
    let expected = -6.288_470_389_245_631_5;
    assert!(
        (got - expected).abs() < TOL_DEG,
        "dense reverse blend got={got} expected={expected}"
    );
}

#[test]
fn true_from_apparent_below_horizon_helper_matches_public_entry_point() {
    // The public `true_from_apparent` must delegate to the below-horizon
    // helper for every negative altitude — nothing may bypass it. Asserted
    // across all three regions (full-Saemundsson, mid-fade, identity).
    for h in [-0.5, -1.0, -5.5, -10.0, -20.0] {
        assert_eq!(
            true_from_apparent(h, EXACT),
            true_from_apparent_below_horizon(h, EXACT),
            "delegation mismatch at h={h}"
        );
    }
}

#[test]
fn refraction_is_suppressed_below_horizon_in_both_directions() {
    // Both directions hold the same identity floor. Pinning them together
    // documents that the two blends are deliberate mirrors of one another
    // rather than independently drifting approximations.
    for h in [-10.0, -12.5, -30.0, -70.739_219] {
        assert_eq!(apparent_from_true(h, EXACT), h, "forward at h={h}");
        assert_eq!(true_from_apparent(h, EXACT), h, "reverse at h={h}");
    }
}
```

- [ ] **Step 2: Run the new tests**

Run: `cargo nextest run -p pleiades-apparent refraction`

Expected: PASS — 18 tests, 0 failed.

- [ ] **Step 3: Verify the missed-mutant count dropped to the documented residual**

Run: `cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file refraction.rs`

Expected: the summary reports **3 missed**. Run `cat mutants.out/missed.txt` and confirm
the residual is exactly these three, and nothing else:

```
crates/pleiades-apparent/src/refraction.rs:51:18: replace * with / in saemundsson_refraction_arcmin
crates/pleiades-apparent/src/refraction.rs:134:10: replace < with <= in apparent_from_true
crates/pleiades-apparent/src/refraction.rs:147:10: replace < with <= in true_from_apparent
```

If any *other* mutant is still listed, it is killable and this task is not finished —
add a case that reaches it before moving on. In particular, `147:10 replace < with ==`
and `147:10 replace < with >` must both be gone (the `-5.5` cases kill them).

- [ ] **Step 4: Verify formatting and lints**

Run: `cargo fmt --all --check && cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings`

Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-apparent/src/refraction/tests.rs
git commit -m "test(apparent): cover true_from_apparent below-horizon branch (20 survivors)"
```

---

### Task 5: Document the equivalent mutants and record the slice result

**Files:**
- Modify: `crates/pleiades-apparent/src/refraction/tests.rs` (add the equivalence note)
- Modify: `docs/follow-ups.md` (FU-9 entry)

**Interfaces:**
- Consumes: the verified 3-mutant residual from Task 4.
- Produces: the closing record for this slice; the next slice (`aberration.rs`) starts from the updated FU-9 entry.

Per the Global Constraints, no `#[mutants::skip]` is added — a function-level skip would
blanket-suppress that function's genuine numeric mutants, exactly as rejected in the
`nutation.rs` slice. The residual stays visible to the tool and is justified in writing.

- [ ] **Step 1: Add the equivalence rationale to the test file**

Append to `crates/pleiades-apparent/src/refraction/tests.rs`:

```rust
// ---------------------------------------------------------------------------
// Documented equivalent mutants (FU-9 slice 3)
//
// After this suite, `cargo mutants -p pleiades-apparent --test-tool nextest
// --test-workspace=false --file refraction.rs` reports exactly three surviving
// mutants. All three are provably equivalent — no input distinguishes them from
// the original — so they are documented here rather than suppressed with
// `#[mutants::skip]`, which would also hide that function's genuine numeric
// mutants:
//
//   1. `51:18 replace * with / in saemundsson_refraction_arcmin`
//      The operand is the literal `1.0`, so `scale * 1.0` and `scale / 1.0`
//      are bit-identical for every f64 input, non-finite ones included.
//
//   2. `134:10 replace < with <= in apparent_from_true`
//   3. `147:10 replace < with <= in true_from_apparent`
//      Both differ from the original only at exactly `h == 0.0`. At that value
//      the two paths compute the same expression: the below-horizon helper's
//      first branch is `h >= BELOW_HORIZON_BLEND_START_DEG` (-1.0), which holds
//      at 0.0 and applies the identical full-refraction formula the `h >= 0`
//      path applies. So routing h == 0.0 either way yields the same result.
//
// Note the sibling mutants `147:10 < with ==` and `147:10 < with >` are NOT
// equivalent — they misroute ordinary inputs — and are killed above by the
// -5.5 deg cases.
// ---------------------------------------------------------------------------
```

- [ ] **Step 2: Run the full blocking tier**

Run: `mise run ci`

Expected: PASS. This confirms the slice changed no behavior and broke no gate. If a
release/parity gate fails, stop — a tests-only slice must not move any gate.

- [ ] **Step 3: Update the FU-9 follow-up entry**

In `docs/follow-ups.md`, immediately after the `**Progress (2026-07-19) —
pleiades-apparent/src/apparent.rs:**` paragraph (the one ending with the remaining-slices
list), insert:

```markdown
**Progress (2026-07-20) — `pleiades-apparent/src/refraction.rs`:** triaged from
`37` → `3` documented equivalent mutants (spec/plan:
`docs/superpowers/specs/2026-07-20-fu9-refraction-mutant-triage-design.md`).
Baseline confirmed by the authoritative per-file command (`101 mutants tested,
37 missed, 64 caught`). This slice was **tests-only** — no refactor was needed,
unlike `apparent.rs`: the file was already decomposed into small pure functions
at exactly the right seams, so the only source edit was relocating the inline
test module to `src/refraction/tests.rs` per AGENTS.md. The dominant finding was
a plain **coverage hole** rather than tolerance masking: `true_from_apparent_below_horizon`
had no test at all (the committed SE corpus exercises only the
`apparent_from_true` direction), accounting for 20 of the 37 survivors including
all three whole-function replacements. The remainder split into blend-region gaps
(the corpus reaches only `h <= -9.96`, where the fade contributes ~9″ under a 15″
tolerance, leaving the `h ∈ [-1, 0)` branch and the fade slope unconstrained) and
loose-tolerance formula survivors (`scale -> 1.0` hides because the default
atmosphere's scale is 0.9858 ≈ 1.0). Reference strategy: **crafted-exact
atmospheres** — `(1010 mbar, 10 °C)` makes `scale` exactly `1.0` and
`(2020 mbar, 25 °C)` makes both factors non-unit and distinct — combined with
Bennett/Saemundsson literals evaluated outside the code from the published
formulas, and fade midpoints chosen so `fade` is an exact binary fraction
(`h = -5.5` → `fade = 0.5`). The blend model is repo-invented (SE's own
below-horizon model is discontinuous and deliberately not reproduced), so its
authority is its own documented spec: anchor = R(-1), linear fade to zero at -10.
**Documented residual — 3 equivalent mutants**, left visible rather than
`#[mutants::skip]`-suppressed: `saemundsson`'s `scale * 1.0` → `/ 1.0`
(bit-identical), and `< → <=` in both public dispatchers, which differ only at
exactly `h == 0.0` where both branches evaluate the identical expression. No
parity gate was touched; the tier stays report-only; `mise run ci` is green.
**Remaining slices** (priority order): `aberration.rs` (28), `topocentric.rs`
(27), `sidereal.rs` (17), `precession.rs` (17), `lighttime.rs` (5), then the
`pleiades-time` and `pleiades-types` survivors.
```

Then update the two stale "Remaining slices" lists earlier in the FU-9 entry (the one
ending the nutation progress paragraph and the one ending the apparent progress
paragraph) by deleting `refraction.rs` (37), ` from each so `aberration.rs` (28) leads
both lists.

- [ ] **Step 4: Verify formatting and lints one final time**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings`

Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-apparent/src/refraction/tests.rs docs/follow-ups.md
git commit -m "docs(follow-ups): record FU-9 refraction.rs triage (37 -> 3 equivalent mutants)"
```

---

## Acceptance Criteria (whole slice)

- `cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file refraction.rs`
  reports **3 missed**, and those 3 are exactly the documented equivalents listed in Task 4 Step 3.
- `mise run ci` (blocking tier) passes.
- `cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean.
- `crates/pleiades-apparent/src/refraction.rs` differs from `main` by exactly one change:
  the inline test module replaced with `#[cfg(test)] mod tests;`. Verify with
  `git diff main -- crates/pleiades-apparent/src/refraction.rs`.
- No parity/release gate tolerance changed. Verify with
  `git diff main -- crates/pleiades-validate/` (expect no output).
- `docs/follow-ups.md` FU-9 entry records the slice result and the updated remaining-slice list.
