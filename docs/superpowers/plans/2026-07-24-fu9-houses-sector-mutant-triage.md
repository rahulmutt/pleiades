# FU-9 Houses Sector Mutant Triage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive the **Sector** family of `crates/pleiades-houses/src/systems/mod.rs` (`pullen_sr_houses`, `pullen_sd_houses`, `albategnius_houses`, `solve_gauquelin_sector`, and its caller `gauquelin_houses`) from a **measured 161 surviving mutants to 6 documented equivalents**, reusing the independent house-math reference established by the Foundation PR.

**Architecture:** Third PR of the ~6-PR `pleiades-houses` FU-9 campaign (spec: `docs/superpowers/specs/2026-07-22-fu9-houses-mutant-triage-design.md`; Foundation PR: `docs/superpowers/plans/2026-07-22-fu9-houses-foundation-mutant-triage.md`; Great-circle PR: `docs/superpowers/plans/2026-07-23-fu9-houses-greatcircle-mutant-triage.md`). **Tests-only** — no production-code change; every added test lives in `crates/pleiades-houses/src/systems/tests.rs` (`use super::*;` reaches the private functions). The reference note `docs/superpowers/specs/notes/2026-07-22-houses-reference.py` is **extended** with independent `pullen_sd` and `pullen_sr` ports.

**Reference strategy, keyed to survivor structure (the key design decision):**
- `pullen_sr_houses`, `pullen_sd_houses`, `albategnius_houses` are **pure functions of `HouseAngles`** — every cusp depends only on `(ascendant, midheaven)` via `acmc = signed_longitude_difference(asc, mc)`. Unlike the Great-circle family they do **not** call `local_sidereal_time`, so no threading is needed. Reference = **independent Python port** (extends `houses-reference.py`), pinning all 12 cusps at a small set of discriminating `(asc, mc)` geometries chosen to make every arithmetic term observable and every branch reachable. `pullen_sd_houses` and `albategnius_houses` are **byte-identical** in the crate (same Albategnian equal-quadrant split), so one reference and one geometry set pins both. For `pullen_sr_houses` the ratio `r` is re-derived independently: `r` is the positive real root of `r^4 + 2 r^3 - 2 c r - c = 0` (with `c = (180-q)/q`), from the published Pullen SR symmetric-arc property `x r^3 (2 + r) = 180 - q`; the port solves it by bisection + Newton polish — a genuinely different method than the crate's Ferrari closed form, so their agreement (cross-validated to ~1e-12 during plan authoring) validates both non-circularly.
- `solve_gauquelin_sector` is a **Newton iterative solver**. Its 4 survivors are all guard / convergence-boundary mutants (no result-affecting survivor — the result arithmetic is already caught by the `validate-houses` / `validate-angles` parity gates through `gauquelin_houses`). One (`|| -> &&` on the fail-closed guard) is **killed** by a crafted non-convergence-but-finite geometry (the lighttime solver-boundary precedent); the other three are **documented equivalents** (measure-zero exact-boundary or same-`Result`-kind).
- `gauquelin_houses` (the 36-sector caller) has **0 surviving mutants** in the measured baseline — its structural arithmetic (fractions, signs, indices, antipode fold, anchors) is fully caught by the existing gate/unit tests. It needs **no new test**; this is confirmed, not assumed (see the measured baseline).

**Mutation-triage TDD cycle** (differs from feature TDD — read once): a triage test *passes* on correct `HEAD` code (it pins intent against the independent reference); it *kills a mutant* by failing on the mutated tree. Each task: write the test → run `cargo nextest` to confirm it **passes** on `HEAD` (proves the literal/geometry is right) → the final task re-runs `cargo mutants -F` to confirm the survivors are **caught** down to the documented equivalents.

**Tech Stack:** Rust (stable, via mise), cargo-nextest, cargo-mutants 27.1.0, Python 3 (reference only).

## Global Constraints

- **Tests-only:** no production-code change anywhere in this PR; the only source file modified is `crates/pleiades-houses/src/systems/tests.rs` (plus the reference-note extension and the follow-up entry). No behavior-preserving refactor is needed (unlike Foundation's `apparent.rs`) — every survivor is reachable through the existing private-function surface.
- **No parity-gate change:** no `validate-*` file is touched; the mutants tier stays report-only.
- **No `#[mutants::skip]`:** the 6 residual documented-equivalent mutants are left visible with a written reachability argument each, enumerated in a characterization test (`sector_equivalent_mutants_are_documented`).
- **Independence discipline:** every expected value comes from the independent Python port (cross-validated against the crate to ~1e-12 at `HEAD` before its literals are trusted), from a crafted-boundary input, or from hand arithmetic — never from running the mutated function and pinning its output.
- **Branch:** create `fu9-houses-sector-mutant-triage` off `main` (do not work on `main`).
- All commands run through mise: prefix cargo invocations with `mise exec --`.
- **cargo-mutants + mise trust gotcha (operational):** cargo-mutants copies the workspace to `/tmp/cargo-mutants-workspace-*.tmp`, and this environment's mise refuses the untrusted copied `mise.toml`, failing the baseline build with `Config files ... are not trusted`. Prefix every `cargo mutants` invocation with `MISE_TRUSTED_CONFIG_PATHS=/tmp`. Without it the run aborts at "build failed in an unmutated tree".

## Measured baseline (2026-07-24, at `261c1235b`)

Authoritative per-file command, scoped to the five Sector functions (cargo-mutants 27.1.0):

```bash
MISE_TRUSTED_CONFIG_PATHS=/tmp mise exec -- cargo mutants \
  --test-tool nextest --test-workspace=false --baseline run \
  -p pleiades-houses \
  --file crates/pleiades-houses/src/systems/mod.rs \
  -F 'in (pullen_sr_houses|pullen_sd_houses|albategnius_houses|solve_gauquelin_sector|gauquelin_houses)$'
```

**233 mutants tested — 161 missed / 72 caught / 0 unviable.** Survivors by function: `pullen_sr_houses` **73**, `pullen_sd_houses` **42**, `albategnius_houses` **42**, `solve_gauquelin_sector` **4**, `gauquelin_houses` **0**. This matches the design spec's whole-crate prediction (73/42/42/4) exactly, and confirms `gauquelin_houses` is already fully caught by the parity gates.

**Target after this PR (measured during plan authoring): `161 -> 6 documented equivalents`,** split `pullen_sr_houses` 73→3 and `solve_gauquelin_sector` 4→3 (one killed), with `pullen_sd_houses` 42→0, `albategnius_houses` 42→0, `gauquelin_houses` 0→0. Scoped re-run reports `6 missed / 227 caught / 0 unviable`.

---

### Task 1: reference extension + `pullen_sd_houses` / `albategnius_houses` — 84 survivors → 0

`pullen_sd_houses` (lines 1387–1421) and `albategnius_houses` (lines 1351–1385) are byte-identical: an Albategnian equal-quadrant split. Each cusp is a linear function of `acmc = signed_longitude_difference(asc, mc)` (after an optional ascendant flip). The 84 survivors (42 each) are arithmetic-operator swaps in `q1 = 180 - acmc`, `d = (acmc-90)/4`, the `30 + d` / `60 + 3d` else-branch terms, the `acmc/2` / `q1/2` bisect terms, and the two `<= 30` comparison guards plus the `< 0` flip guard. **Five `(asc, mc)` geometries pinning all 12 cusps** kill all 84 (measured 84/84 caught): each geometry makes the relevant terms non-degenerate, and the set reaches every branch and every killable comparison boundary.

**Files:**
- Modify: `docs/superpowers/specs/notes/2026-07-22-houses-reference.py` (append `signed_diff`, `_complete_opposite`, `pullen_sd`, `_sr_ratio`, `pullen_sr` + prints)
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `pullen_sd_houses(angles: HouseAngles) -> [Longitude; 12]`, `albategnius_houses(angles: HouseAngles) -> [Longitude; 12]`; `gc_angles(asc: f64, mc: f64) -> HouseAngles` (already in `tests.rs` from the Great-circle PR); `HouseAngles { ascendant, descendant, midheaven, imum_coeli }` (all `Longitude`). All reached via `use super::*;`.
- Produces: `assert_sector_cusps(&[Longitude;12], &[f64;12], &str)` helper and the `pullen_sd`/`pullen_sr` Python reference (consumed by Task 2 and Task 4).

- [ ] **Step 1: Extend the independent reference with the Sector ports**

Append to `docs/superpowers/specs/notes/2026-07-22-houses-reference.py`, **before** the `if __name__ == "__main__":` block, the following (re-derived from the published Albategnian and Pullen-SR rules, NOT copied operator-by-operator from the Rust; uses `norm360`/`opposite`/`fmt` already defined in the file):

```python
def signed_diff(a, b):
    """crate `signed_longitude_difference`: (a-b) normalized to [-180, 180)."""
    delta = norm360(a - b)
    return delta - 360.0 if delta >= 180.0 else delta


def _complete_opposite(c):
    """crate `complete_opposite_houses`: cusps 4-9 as ecliptic antipodes."""
    c[3] = opposite(c[9])
    c[4] = opposite(c[10])
    c[5] = opposite(c[11])
    c[6] = opposite(c[0])
    c[7] = opposite(c[1])
    c[8] = opposite(c[2])


def pullen_sd(asc_deg, mc_deg):
    """crate `pullen_sd_houses` == `albategnius_houses` (byte-identical): the
    published equal-quadrant (Albategnian) split of each ASC/MC quadrant.
    Elementary published arithmetic; re-derived here, not copied from the Rust."""
    asc = asc_deg
    acmc = signed_diff(asc, mc_deg)
    if acmc < 0.0:
        asc = opposite(asc)
        acmc = signed_diff(asc, mc_deg)
    c = [0.0] * 12
    c[0] = norm360(asc)
    c[9] = norm360(mc_deg)
    q1 = 180.0 - acmc
    d = (acmc - 90.0) / 4.0
    if acmc <= 30.0:
        c[10] = norm360(mc_deg + acmc / 2.0)
        c[11] = c[10]
    else:
        c[10] = norm360(mc_deg + 30.0 + d)
        c[11] = norm360(mc_deg + 60.0 + 3.0 * d)
    d = (q1 - 90.0) / 4.0
    if q1 <= 30.0:
        c[1] = norm360(asc + q1 / 2.0)
        c[2] = c[1]
    else:
        c[1] = norm360(asc + 30.0 + d)
        c[2] = norm360(asc + 60.0 + 3.0 * d)
    _complete_opposite(c)
    return c


def _sr_ratio(q):
    """Pullen SR ratio r: the positive real root of r^4 + 2 r^3 - 2 c r - c = 0,
    with c=(180-q)/q, derived from the published SR symmetric-arc property
    x*r^3*(2 + r) = 180 - q (small quadrant split xr:x:xr summing to q). Solved by
    bracketed bisection + Newton polish -- a genuinely different method than the
    crate's Ferrari closed form, so agreement cross-validates both. Sanity:
    q=90 -> c=1 -> r=1 (equal division)."""
    c = (180.0 - q) / q

    def f(r):
        return r ** 4 + 2.0 * r ** 3 - 2.0 * c * r - c

    def fp(r):
        return 4.0 * r ** 3 + 6.0 * r ** 2 - 2.0 * c

    lo, hi = 1.0, c + 3.0            # f(1)=3(1-c)<=0, f(hi)>0 for c>=1
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        if f(mid) <= 0.0:
            lo = mid
        else:
            hi = mid
    r = 0.5 * (lo + hi)
    for _ in range(60):              # Newton polish to ~machine precision
        r -= f(r) / fp(r)
    return r


def pullen_sr(asc_deg, mc_deg):
    """crate `pullen_sr_houses`: Pullen Sinusoidal Ratio house division."""
    asc = asc_deg
    acmc = signed_diff(asc, mc_deg)
    if acmc < 0.0:
        asc = opposite(asc)
        acmc = signed_diff(asc, mc_deg)
    c = [0.0] * 12
    c[0] = norm360(asc)
    c[9] = norm360(mc_deg)
    q = acmc
    if q > 90.0:
        q = 180.0 - q
    if q < 1.0e-30:
        x, xr, xr3, xr4 = 0.0, 0.0, 0.0, 180.0
    else:
        r = _sr_ratio(q)
        x = q / (2.0 * r + 1.0)
        xr = r * x
        xr3 = xr * r * r
        xr4 = xr3 * r
    if acmc > 90.0:
        c[10] = norm360(mc_deg + xr3)
        c[11] = norm360(c[10] + xr4)
        c[1] = norm360(asc + xr)
        c[2] = norm360(c[1] + x)
    else:
        c[10] = norm360(mc_deg + xr)
        c[11] = norm360(c[10] + x)
        c[1] = norm360(asc + xr3)
        c[2] = norm360(c[1] + xr4)
    _complete_opposite(c)
    return c
```

And append inside the `__main__` block:

```python
    print("# pullen_sd / albategnius (byte-identical) -- all 12 cusps:")
    for name, a, m in [("200/100", 200.0, 100.0), ("120/100", 120.0, 100.0),
                       ("260/100", 260.0, 100.0), ("10/100(flip)", 10.0, 100.0),
                       ("100/100(deg)", 100.0, 100.0)]:
        print(f"  [{name}]", [fmt(v) for v in pullen_sd(a, m)])
    print("# pullen_sr -- all 12 cusps:")
    for name, a, m in [("200/100", 200.0, 100.0), ("140/100", 140.0, 100.0),
                       ("10/100(flip)", 10.0, 100.0), ("100/100(guard)", 100.0, 100.0)]:
        print(f"  [{name}]", [fmt(v) for v in pullen_sr(a, m)])
    print("# _sr_ratio(90) sanity (must be 1.0):", _sr_ratio(90.0))
```

Run `python3 docs/superpowers/specs/notes/2026-07-22-houses-reference.py` and confirm it prints `_sr_ratio(90) = 1.0` and the literals used in Steps 2 and in Task 2 (they were generated by exactly this port and cross-validated to ~1e-12 against the crate during plan authoring).

- [ ] **Step 2: Write the `assert_sector_cusps` helper + the pullen_sd/albategnius pin**

Append to `crates/pleiades-houses/src/systems/tests.rs`:

```rust
// ===== FU-9 Sector PR: pullen_sr / pullen_sd / albategnius / gauquelin =====
// Independent reference: docs/superpowers/specs/notes/2026-07-22-houses-reference.py
// (`pullen_sd`, `pullen_sr`), cross-validated against the crate to ~1e-12 during
// plan authoring. gauquelin_houses already reaches 0 surviving mutants via the
// validate-houses / validate-angles parity gates, so it needs no new unit test;
// only solve_gauquelin_sector's guard survivors are addressed here.

fn assert_sector_cusps(got: &[Longitude; 12], want: &[f64; 12], label: &str) {
    for i in 0..12 {
        let mut d = (got[i].degrees() - want[i]).rem_euclid(360.0);
        if d > 180.0 {
            d -= 360.0;
        }
        assert!(
            d.abs() < 1e-9,
            "{label} cusp[{i}] = {}, want {}",
            got[i].degrees(),
            want[i]
        );
    }
}

#[test]
fn pullen_sd_and_albategnius_pin_all_cusps_against_independent_reference() {
    // pullen_sd_houses and albategnius_houses are byte-identical in the crate
    // (same equal-quadrant split); the same reference pins both. Geometries:
    //  200/100 acmc=100 -> both `else` quadrant-split branches (d != 0);
    //  120/100 acmc=20  -> MC-side bisect branch (acmc <= 30);
    //  260/100 acmc=160 -> ASC-side bisect branch (q1 = 180-acmc <= 30);
    //  10/100  acmc<0   -> ascendant flip branch (kills `< 0 -> == 0`);
    //  100/100 acmc=0   -> flip-guard equality (kills `< 0 -> <= 0`).
    let cases: [(f64, f64, [f64; 12]); 5] = [
        (200.0, 100.0, [200.0, 227.5, 252.5, 280.0, 312.5, 347.5, 20.0, 47.5, 72.5, 100.0, 132.5, 167.5]),
        (120.0, 100.0, [120.0, 167.5, 232.5, 280.0, 290.0, 290.0, 300.0, 347.5, 52.5, 100.0, 110.0, 110.0]),
        (260.0, 100.0, [260.0, 270.0, 270.0, 280.0, 327.5, 32.5, 80.0, 90.0, 90.0, 100.0, 147.5, 212.5]),
        (10.0, 100.0, [190.0, 220.0, 250.0, 280.0, 310.0, 340.0, 10.0, 40.0, 70.0, 100.0, 130.0, 160.0]),
        (100.0, 100.0, [100.0, 152.5, 227.5, 280.0, 280.0, 280.0, 280.0, 332.5, 47.5, 100.0, 100.0, 100.0]),
    ];
    for (asc, mc, want) in cases {
        let angles = gc_angles(asc, mc);
        assert_sector_cusps(&pullen_sd_houses(angles), &want, &format!("pullen_sd asc={asc}"));
        assert_sector_cusps(&albategnius_houses(angles), &want, &format!("albategnius asc={asc}"));
    }
}
```

- [ ] **Step 3: Run the test — confirm it passes on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses pullen_sd_and_albategnius`
Expected: `1 passed`. (A failure means a literal or the `gc_angles` import is wrong — fix before proceeding.)

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs docs/superpowers/specs/notes/2026-07-22-houses-reference.py
git commit -m "test(houses): FU-9 pin pullen_sd + albategnius vs independent reference (84 mutants)"
```

---

### Task 2: `pullen_sr_houses` — 73 survivors → 3 documented equivalents

`pullen_sr_houses` (lines 1423–1472) adds a Ferrari-style quartic root for the Pullen SR ratio `r`, then places cusps from `x`, `xr`, `xr3`, `xr4`. The 73 survivors are arithmetic swaps in `q1`/`d`-style terms, the `q > 90` reduction, the `q < 1e-30` guard, the ratio expressions, and the `acmc > 90` placement split. **Four `(asc, mc)` geometries pinning all 12 cusps** kill 70 and leave 3 documented equivalents (measured 70 caught, 3 missed). The reference literals come from the independent quartic-root port added in Task 1.

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `pullen_sr_houses(angles: HouseAngles) -> [Longitude; 12]`; `gc_angles`; `assert_sector_cusps` (Task 1).
- Produces: nothing consumed by later tasks (the 3 equivalents are documented in Task 4).

- [ ] **Step 1: Write the pullen_sr pin**

Append:

```rust
#[test]
fn pullen_sr_pins_all_cusps_against_independent_reference() {
    // Independent reference (houses-reference.py `pullen_sr`): ratio r solved as
    // the positive root of r^4 + 2r^3 - 2c*r - c = 0 (c=(180-q)/q) by bisection+
    // Newton, a different method than the crate's Ferrari closed form; matched to
    // ~1e-12 during plan authoring. Geometries:
    //  200/100 acmc=100 -> q>90 reduction (q=80) AND acmc>90 placement branch;
    //  140/100 acmc=40  -> no reduction AND acmc<=90 placement branch;
    //  10/100  acmc<0   -> flip -> acmc=90 (r=1 exactly);
    //  100/100 acmc=0   -> q<1e-30 guard branch (x=xr=xr3=0, xr4=180).
    let cases: [(f64, f64, [f64; 12]); 4] = [
        (200.0, 100.0, [200.0, 227.399778974511, 252.600221025489, 280.0, 312.391037626774, 347.608962373226, 20.0, 47.399778974511, 72.600221025489, 100.0, 132.391037626774, 167.608962373226]),
        (140.0, 100.0, [140.0, 178.908843802504, 241.091156197496, 280.0, 295.233904915732, 304.766095084268, 320.0, 358.908843802504, 61.091156197496, 100.0, 115.233904915732, 124.766095084268]),
        (10.0, 100.0, [190.0, 220.0, 250.0, 280.0, 310.0, 340.0, 10.0, 40.0, 70.0, 100.0, 130.0, 160.0]),
        (100.0, 100.0, [100.0, 100.0, 280.0, 280.0, 280.0, 280.0, 280.0, 280.0, 100.0, 100.0, 100.0, 100.0]),
    ];
    for (asc, mc, want) in cases {
        assert_sector_cusps(&pullen_sr_houses(gc_angles(asc, mc)), &want, &format!("pullen_sr asc={asc}"));
    }
}
```

- [ ] **Step 2: Run the test — confirm it passes on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses pullen_sr_pins_all_cusps`
Expected: `1 passed`.

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): FU-9 pin pullen_sr vs independent quartic-root port (73 -> 3 equiv)"
```

---

### Task 3: `solve_gauquelin_sector` — 4 survivors → 1 killed, 3 documented equivalents

`solve_gauquelin_sector` (lines 1304–1349) is a Newton solve for one Gauquelin sector boundary. Its 4 survivors are all guard/convergence-boundary comparisons: the zero-derivative guard `if gp.abs() < 1e-12` (1327; `< -> ==` and `< -> <=`), the convergence test `if delta.abs() < 1e-9` (1335; `< -> <=`), and the fail-closed guard `if !converged || !q.is_finite()` (1341; `|| -> &&`). The `|| -> &&` mutant is **killed** by a crafted geometry that runs all 64 iterations without converging while `q` stays finite; the other three are **documented equivalents** (Task 4). `gauquelin_houses` (the 36-sector caller) already has 0 survivors and needs no test.

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `solve_gauquelin_sector(ramc_deg: f64, latitude_deg: f64, obliquity_deg: f64, fraction: f64, sign: f64) -> Result<Longitude, HouseError>` (private, via `use super::*;`).
- Produces: nothing consumed by later tasks.

- [ ] **Step 1: Write the non-convergence fail-closed test**

Append:

```rust
#[test]
fn solve_gauquelin_sector_fails_closed_on_nonconvergence() {
    // Kills 1341 `!converged || !q.is_finite()` -> `&&`: at lat=80, obl=23.4366,
    // fraction=1/9, sign=+1, ramc=30 the Newton iteration does not converge in 64
    // steps but q stays finite (~52.4). HEAD: `!converged(true) || ...` -> Err;
    // the `&&` mutant: `true && !finite(false)` -> false -> Ok(unconverged). So
    // HEAD MUST return Err here for the `&&` mutant to be observable. (Geometry
    // found by a lat/obl/fraction/ramc sweep of the crate's Newton during plan
    // authoring: 22 non-converged-but-finite candidates, this is the first.)
    let r = solve_gauquelin_sector(30.0, 80.0, 23.4366, 1.0 / 9.0, 1.0);
    assert!(r.is_err(), "expected non-convergence Err, got {r:?}");
    // A physical Gauquelin geometry converges to a finite Ok (the live path).
    let ok = solve_gauquelin_sector(280.4570696, 52.0, 23.4366, 8.0 / 9.0, 1.0);
    assert!(ok.is_ok(), "expected convergence Ok, got {ok:?}");
}
```

- [ ] **Step 2: Run the test — confirm it passes on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses solve_gauquelin_sector_fails_closed`
Expected: `1 passed`. (If the `Err` assertion fails, the crate's Newton converged here — re-pick from the other 21 candidates: `lat=80, obl=23.4366, fraction=1/9, sign=+1` at `ramc` in `{60,90,210,270}`, or `sign=-1` at `ramc` in `{90,120,150}`.)

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): FU-9 solve_gauquelin_sector non-convergence fail-closed (kills || -> &&)"
```

---

### Task 4: documented-equivalent characterization test — the 6 residual equivalents

The 6 residual survivors (3 in `pullen_sr_houses`, 3 in `solve_gauquelin_sector`) are each left visible (no `#[mutants::skip]`) with a reachability argument, enumerated in one characterization test that also asserts the two facts the `pullen_sr` arguments rest on.

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `pullen_sr_houses`, `solve_gauquelin_sector`, `gc_angles`, `assert_sector_cusps` (Tasks 1–3).

- [ ] **Step 1: Write the characterization test**

Append:

```rust
#[test]
fn sector_equivalent_mutants_are_documented() {
    // FU-9 Sector residual: 6 surviving mutants, each an EQUIVALENT MUTANT left
    // visible (no #[mutants::skip]), enumerated with a reachability argument.
    // Measured by the authoritative scoped run: 233 tested, 6 missed, 227 caught.
    //
    // --- pullen_sr_houses (3) ---
    // (SR-1) 1437:10 `q > 90.0 -> q >= 90.0` (quadrant reduction): differs only at
    //   q == 90.0, where HEAD keeps q=90 and the mutant sets q=180-90=90 -- same q,
    //   same output. Reachable (acmc=90 via the asc=10/mc=100 flip) but coincident.
    // (SR-2) 1458:13 `acmc > 90.0 -> acmc >= 90.0` (placement branch): differs only
    //   at acmc == 90.0, where q=90 -> c=1 -> r=1 exactly, so xr == xr3 and x == xr4
    //   and the `if`/`else` placements produce bit-identical cusps.
    // (SR-3) 1441:34 `q < 1e-30 -> q <= 1e-30` (degenerate-quadrant guard): differs
    //   only at q == 1e-30 exactly -- a measure-zero boundary q (from
    //   signed_longitude_difference of f64 degrees) cannot reach.
    //
    // At the acmc=90 flip geometry the SR division is the r=1 equal 30-degree split,
    // so both the reduction and placement branches coincide (kills SR-1/SR-2 intent):
    let acmc90 = pullen_sr_houses(gc_angles(10.0, 100.0));
    let equal = [190.0, 220.0, 250.0, 280.0, 310.0, 340.0, 10.0, 40.0, 70.0, 100.0, 130.0, 160.0];
    assert_sector_cusps(&acmc90, &equal, "SR acmc=90 is the r=1 equal split");
    // The q<1e-30 guard is reached only at acmc=0 (asc==mc); signature x=xr=xr3=0
    // gives cusp[1]==asc and cusp[2]==desc (SR-3 boundary is this degenerate point):
    let guard = pullen_sr_houses(gc_angles(100.0, 100.0));
    assert!((guard[1].degrees() - 100.0).abs() < 1e-9, "SR guard cusp[1]==asc");
    assert!((guard[2].degrees() - 280.0).abs() < 1e-9, "SR guard cusp[2]==desc");
    //
    // --- solve_gauquelin_sector (3) ---
    // (GQ-1) 1327:21 `gp.abs() < 1e-12 -> ==` (zero-derivative guard): the divergence
    //   interval gp.abs() in (0, 1e-12) IS reachable (min |gp| over a physical
    //   lat/obl/fraction/ramc sweep reaches ~1.9e-13), but both HEAD and the mutant
    //   return Err(NumericalFailure) there -- HEAD via the zero-derivative guard, the
    //   mutant via the subsequent non-convergence/non-finite exit -- so no test
    //   observing the public Result (kind) distinguishes them; only the diagnostic
    //   message differs, and the campaign does not pin error-message text.
    // (GQ-2) 1327:21 `gp.abs() < 1e-12 -> <=`: differs only at gp.abs()==1e-12
    //   exactly -- measure-zero, unreachable.
    // (GQ-3) 1335:24 `delta.abs() < 1e-9 -> <=` (convergence): differs only at
    //   delta.abs()==1e-9 exactly -- a Newton iterate shrinking quadratically past
    //   1e-9 does not land on it; measure-zero, unreachable.
    //
    // The reachable non-convergence exit is a real Err (pinned by
    // solve_gauquelin_sector_fails_closed_on_nonconvergence) and a physical geometry
    // converges to Ok -- the live path both operators share:
    assert!(solve_gauquelin_sector(280.4570696, 52.0, 23.4366, 8.0 / 9.0, 1.0).is_ok());
}
```

- [ ] **Step 2: Run the test — confirm it passes on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses sector_equivalent_mutants_are_documented`
Expected: `1 passed`.

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): FU-9 document 6 Sector equivalent mutants with reachability args"
```

---

### Task 5: Verify Sector reaches 6 documented equivalents + follow-up note

Re-run cargo-mutants scoped to the five Sector functions and confirm exactly **6 missed** (3 `pullen_sr` + 3 `solve_gauquelin_sector`). Then record the slice in `docs/follow-ups.md`.

**Files:**
- Modify: `docs/follow-ups.md` (append an FU-9 Progress note)

- [ ] **Step 1: Confirm the whole crate test suite is green**

Run: `mise exec -- cargo nextest run -p pleiades-houses`
Expected: all pass (96 existing from the Great-circle PR + 4 new = 100).

- [ ] **Step 2: Run scoped mutation verification (233 mutants, ~6 min)**

Run:
```bash
MISE_TRUSTED_CONFIG_PATHS=/tmp mise exec -- cargo mutants \
  --test-tool nextest --test-workspace=false --baseline run \
  -p pleiades-houses \
  --file crates/pleiades-houses/src/systems/mod.rs \
  -F 'in (pullen_sr_houses|pullen_sd_houses|albategnius_houses|solve_gauquelin_sector|gauquelin_houses)$'
```
Expected (measured during plan authoring): **`6 missed / 227 caught / 0 unviable`** out of 233 mutants. Confirm `cat mutants.out/missed.txt` is exactly these 6:
```
1327:21  replace < with ==   in solve_gauquelin_sector  (GQ-1, same-Result-kind)
1327:21  replace < with <=   in solve_gauquelin_sector  (GQ-2, measure-zero 1e-12)
1335:24  replace < with <=   in solve_gauquelin_sector  (GQ-3, measure-zero 1e-9)
1437:10  replace > with >=   in pullen_sr_houses        (SR-1, q=90 coincident)
1441:34  replace < with <=   in pullen_sr_houses        (SR-3, measure-zero 1e-30)
1458:13  replace > with >=   in pullen_sr_houses        (SR-2, acmc=90 r=1 coincident)
```
If any *other* mutant is still missed (not in this set), classify it (an arithmetic/comparison swap in one of the five functions), add a discriminating geometry to the matching pin test using the independent reference, and re-run — do not proceed until the residual is exactly these 6 documented equivalents.

- [ ] **Step 3: Append the FU-9 Progress note to `docs/follow-ups.md`**

Under the FU-9 section, after the houses Great-circle progress note, add:

```markdown
**Progress (2026-07-24) — houses Sector
(`pleiades-houses/src/systems/mod.rs`, `pullen_sr_houses`/`pullen_sd_houses`/
`albategnius_houses`/`solve_gauquelin_sector`/`gauquelin_houses`):** third PR of
the post-baseline `pleiades-houses` expansion campaign (spec:
`docs/superpowers/specs/2026-07-22-fu9-houses-mutant-triage-design.md`; plan:
`docs/superpowers/plans/2026-07-24-fu9-houses-sector-mutant-triage.md`). Triaged
the Sector family from `161` surviving mutants (`pullen_sr` 73, `pullen_sd` 42,
`albategnius` 42, `solve_gauquelin_sector` 4, `gauquelin_houses` 0 — matching the
design's whole-crate prediction 73/42/42/4 exactly, and confirming
`gauquelin_houses` is already fully caught by the parity gates) to **6 documented
equivalents**. **Tests-only.** The three pure systems (`pullen_sd`/`albategnius`
byte-identical, `pullen_sr`) are pure functions of `(asc, mc)` — no
`local_sidereal_time`, unlike the Great-circle family — so their 157 arith/
comparison survivors fell to **independent-port** pins of all 12 cusps at a small
set of discriminating geometries (extending the shared `houses-reference.py` with
`pullen_sd` + `pullen_sr`, cross-validated to ~1e-12). `pullen_sr`'s ratio `r` was
re-derived non-circularly as the positive root of `r^4 + 2r^3 - 2c*r - c = 0`
(`c=(180-q)/q`, from the published SR symmetric-arc property) solved by bisection+
Newton — a different method than the crate's Ferrari closed form. The 4
`solve_gauquelin_sector` survivors are all guard/convergence boundaries: the
`|| -> &&` fail-closed guard was **killed** by a crafted non-convergence-but-finite
geometry (lat=80, fraction=1/9 — HEAD returns `Err`, the `&&` mutant `Ok`), the
lighttime solver-boundary precedent. **Documented residual — 6 equivalent mutants**,
left visible (no `#[mutants::skip]`) and enumerated with per-mutant reachability
arguments in `sector_equivalent_mutants_are_documented`: `pullen_sr` `1437 > -> >=`
(q=90 maps to itself under both operators), `1458 > -> >=` (at acmc=90, r=1 exactly
so the two placement branches are bit-identical), `1441 < -> <=` (q=1e-30
unreachable); `solve_gauquelin_sector` `1327 < -> ==` (the gp<1e-12 interval is
reachable — min |gp| ~1.9e-13 — but both HEAD and mutant return
`Err(NumericalFailure)` there, differing only in message, which the campaign does
not pin), `1327 < -> <=` (gp==1e-12 measure-zero), `1335 < -> <=` (delta==1e-9
measure-zero). This brings the **running documented-equivalent tally to
`30 + 6 = 36`**. The `6` is **measured, not predicted**: the authoritative scoped
run (`-F 'in (pullen_sr_houses|pullen_sd_houses|albategnius_houses|
solve_gauquelin_sector|gauquelin_houses)$'`, 233 mutants) reports
`6 missed / 227 caught / 0 unviable`. No parity gate was touched; the tier stays
report-only; `mise run ci` is green. **Remaining houses PRs:** sunshine/solar-arc
(`sunshine_houses`/`sunshine_offsets`/`apparent_solar_declination`/…),
quadrant/projection (`solve_placidian_cusp`/`topocentric_latitude`/
`regiomontanus`/`koch`/campanus/alcabitius/morinus/carter), then catalog +
thresholds (which adds `-p pleiades-houses` to `[tasks.mutants]`).
```

- [ ] **Step 4: Run the blocking CI gate**

Run: `mise run ci`
Expected: green (fmt + clippy `-D warnings` + workspace test).

- [ ] **Step 5: Commit**

```bash
git add docs/follow-ups.md
git commit -m "docs(follow-ups): record FU-9 houses Sector triage (161 -> 6 documented equivalents)"
```

---

## Self-Review

- **Spec coverage:** This plan implements the spec's **PR 3 — Sector** row (`pullen_sr_houses` 73, `pullen_sd_houses` 42, `albategnius_houses` 42, `solve_gauquelin_sector` 4, `gauquelin_houses`) and the method/reference-strategy/acceptance sections for those functions. It confirms exact survivor membership against a fresh measured baseline (`161 missed`, matching the spec's 73/42/42/4 prediction) per the spec's "Exact survivor membership per PR is confirmed against the measured `mutants.out/missed.txt` at the start of each PR's plan," and additionally establishes the measured fact — not in the design's prediction — that `gauquelin_houses` already has 0 survivors (gate-covered), so no test is written for it (YAGNI). The remaining two campaign PRs are separate plans.
- **Placeholder scan:** none — every test body is complete and was validated to pass on HEAD during plan authoring; every literal was generated by the independent port (`houses-reference.py` `pullen_sd`/`pullen_sr`) and cross-validated to ~1e-12 against the crate; every kill count and the 6-equivalent residual were measured by scoped `cargo mutants` (161→6, exit code 2, report-only).
- **Type consistency:** signatures used match the crate as read at `261c1235b`: `pullen_sr_houses`/`pullen_sd_houses`/`albategnius_houses(HouseAngles) -> [Longitude; 12]`, `solve_gauquelin_sector(f64, f64, f64, f64, f64) -> Result<Longitude, HouseError>`, `gc_angles(f64, f64) -> HouseAngles` (already in `tests.rs`), `HouseAngles { ascendant, descendant, midheaven, imum_coeli }`. The `assert_sector_cusps` helper name is consistent across Tasks 1–4.
- **Independence:** every expected value derives from the independent Python port (a bisection+Newton quartic solve for `pullen_sr`'s ratio, genuinely different from the crate's Ferrari closed form; elementary re-derived arithmetic for `pullen_sd`) cross-validated at HEAD before use, or from a crafted-boundary input (`solve_gauquelin_sector` non-convergence) — non-circular. The two facts the `pullen_sr` equivalence arguments rest on (r=1 at acmc=90; the q<1e-30 guard signature at acmc=0) are asserted in the characterization test, so they cannot silently rot.
- **Measure-don't-predict:** the 6-equivalent residual, the single `|| -> &&` kill, and the `gauquelin_houses`-already-0 fact were all measured, not asserted; the Foundation over-documentation lesson is applied (the gp<1e-12 guard reachability was probed — min |gp| ~1.9e-13 — before GQ-1 was classified equivalent, and the classification rests on same-`Result`-kind rather than a false "unreachable" claim).
