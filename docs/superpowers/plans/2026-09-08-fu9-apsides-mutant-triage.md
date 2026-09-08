# FU-9 `pleiades-apsides` Mutant Triage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive `pleiades-apsides` from 33 surviving mutants to 4 documented
equivalents, using intent-expressing white-box tests referenced to independent
authorities.

> **Correction (2026-09-08, post-review) — read this before the plan body.**
> The Goal above and the Verification Summary's `Task 8` row below both say the
> residual is `4`; the measured outcome is **`3`**
> (`223 mutants tested in 3m: 3 missed, 216 caught, 4 unviable`). Review found
> that one of the four claimed equivalents — `81:35`, `||` → `&&` on
> `to_ecliptic`'s output guard — is killable after all. Its argument assumed
> `p[2] / r` always lies in `[-1, 1]`; that fails once `fl(z*z)` underflows to
> a subnormal, because then `sqrt(fl(z²)) < |z|` and the ratio exceeds `1`, so
> `asin` returns `NaN` while `atan2` stays finite. Exactly one output angle is
> non-finite, which is the region where `||` and `&&` differ.
> `to_ecliptic_rejects_underflowing_norm` kills it. The plan's task text is
> left as written — it is a design record — but no number in it may be read as
> the measured result. **This supersedes every such mention throughout:**
> wherever the plan says "four equivalents", "4 documented equivalents", or a
> campaign tally of "45 → 49", read **three** and **45 → 48**. Those sites are deliberately not edited individually — the plan
> records what was designed, and this note records what was measured. See the
> FU-9 "apsides" entry in `docs/follow-ups.md`.

**Architecture:** One crate, one source file (`crates/pleiades-apsides/src/lib.rs`,
456 lines). The inline test module is relocated to `src/tests.rs` first, then
tests are added in six batches grouped by survivor class. Expected values come
from a forward Gaussian construction (elements → state) that is a genuinely
different computation from the inverse under test, plus conic invariants the
crate never evaluates.

**Tech Stack:** Rust (stable, per `mise.toml`), `cargo-nextest`,
`cargo-mutants` 27.1.0, `mise` task runner.

## Global Constraints

- **No production behavior change.** Only the Task 1 test relocation touches
  non-test code, and it changes no runtime result.
- **No parity-gate change.** `validate-lilith` corpus, tolerances, and gate code
  are untouched. This slice adds *unit* coverage, not gate coverage.
- **The mutants tier stays report-only.** No mutation-score gate is introduced.
  `cargo mutants` exiting 2 is the expected, passing outcome.
- **Never `#[mutants::skip]`.** A function-level skip would blanket-suppress
  that function's numeric mutants. Documented equivalents stay visible with a
  written reachability argument.
- **Never assert against the code's own output.** Every expected value derives
  from the forward construction, a conic invariant, or a published constant.
- **Run `cargo fmt` before every commit.** Array/vector literals in this plan
  are not rustfmt-clean as written and the CI fmt gate is blocking.
- **Authoritative measurement command** (output written outside the repo):
  ```bash
  mise exec -- cargo mutants -p pleiades-apsides --test-tool nextest \
    --test-workspace=false --baseline run -o /tmp/apsides-mutants
  ```
- **Baseline to reproduce:** at `7aefa946c`, `223 mutants tested: 33 missed,
  186 caught, 4 unviable`. The 4 unviable are
  `replace <fn> -> Result<…> with Ok(Default::default())` on the four public
  functions; none of the return types implements `Default`, so they do not
  compile. They are neither survivors nor kills — no work is required for them.

**Line numbers** in this plan refer to `crates/pleiades-apsides/src/lib.rs` as
it stands at `7aefa946c`. Task 1 removes only the trailing test module, so all
production line numbers (76–287) are unchanged by it.

---

## File Structure

| File | Responsibility | Task |
|------|----------------|------|
| `crates/pleiades-apsides/src/lib.rs` | Production only after Task 1; gains `#[cfg(test)] mod tests;` | 1 |
| `crates/pleiades-apsides/src/tests.rs` | The crate's entire unit-test suite | 1–7 |
| `mise.toml` | `[tasks.mutants]` gains `-p pleiades-apsides` | 8 |
| `.github/workflows/mutants.yml` | Calibration comment cites this slice | 8 |
| `docs/follow-ups.md` | FU-9 progress note | 8 |
| `docs/superpowers/specs/notes/2026-07-25-mutants-roadmap-baseline.md` | Mark apsides triaged | 8 |

---

### Task 1: Relocate the inline test module

Per AGENTS.md ("keep large inline test suites out of the file under test") and
the `thresholds.rs` precedent from the houses campaign's PR 6. White-box unit
tests stay unit tests — `to_ecliptic`, `cross`, `norm`, and `MIN_ECCENTRICITY`
are private, so `tests.rs` reaches them via `use super::*`.

**Files:**
- Modify: `crates/pleiades-apsides/src/lib.rs` (removes the trailing `#[cfg(test)] mod tests { … }`)
- Create: `crates/pleiades-apsides/src/tests.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: `crates/pleiades-apsides/src/tests.rs`, a `#[cfg(test)]` submodule
  opening with `use super::*;`. Every later task appends to this file.

- [ ] **Step 1: Move the test module body into a new file**

Cut everything from `#[cfg(test)]\nmod tests {` to the final closing `}` of that
module out of `lib.rs`. Create `crates/pleiades-apsides/src/tests.rs` containing
the module's *body* only (not the `mod tests { … }` wrapper), starting with:

```rust
//! Unit tests for the osculating-apsides geometry.

use super::*;
```

followed by the six existing test functions and the existing
`perigee_on_x_state` helper, unchanged.

- [ ] **Step 2: Declare the submodule in `lib.rs`**

Replace the removed block at the end of `lib.rs` with exactly:

```rust
#[cfg(test)]
mod tests;
```

- [ ] **Step 3: Verify the suite still runs, unchanged**

Run: `mise exec -- cargo nextest run -p pleiades-apsides`
Expected: PASS, `6 tests run: 6 passed, 0 skipped`. The six names must be
identical to before the move.

- [ ] **Step 4: Confirm no production line drifted**

Run: `git diff --stat crates/pleiades-apsides/src/lib.rs`
Expected: only deletions, all after the last production line. Confirm with
`mise exec -- cargo mutants -p pleiades-apsides --list | wc -l` → `223`.

- [ ] **Step 5: Commit**

```bash
mise exec -- cargo fmt -p pleiades-apsides
git add crates/pleiades-apsides/src/lib.rs crates/pleiades-apsides/src/tests.rs
git commit -m "refactor(apsides): relocate inline tests to src/tests.rs

No behavior change. Follows AGENTS.md's rule on keeping large inline test
suites out of the file under test, and the thresholds.rs precedent from the
pleiades-houses campaign."
```

---

### Task 2: Independent forward reference + non-degenerate geometry

**Kills 10 mutants:** `112:17`×2 (`/`→`%`, `/`→`*`), `114:19`, `115:19`,
`115:24`, `116:19`, `139:39`, `139:49`, `139:58`, `139:68`.

**Why they survive today:** every existing test builds its state with
`perigee_on_x_state`, which is simultaneously **apsidal** (`r·v = 0`, so
`c2 = rv/mu` is exactly `0`) and **axis-aligned** (`e_hat = [1,0,0]`, so
`e_hat[1]` and `e_hat[2]` are exactly `0`). With `c2 == 0`,
`c1*r[i] - c2*v[i]` is bit-identical to `c1*r[i] + c2*v[i]`, and `rv/mu` is
bit-identical to `rv*mu` and `rv%mu`. With the cross components zero, negation
and `*`↔`/` on them are invisible. One state that is neither apsidal nor
axis-aligned kills all ten.

**Rejected geometries — do not re-propose:**
- Any state at true anomaly `0` or `180°` (apsidal ⇒ `c2 = 0`).
- Any state whose eccentricity vector lies along a coordinate axis
  (⇒ two `e_hat` components are exactly `0`).
- `i = 0` (⇒ `elements_from_state` returns `DegenerateNode`).

**Files:**
- Modify: `crates/pleiades-apsides/src/tests.rs`

**Interfaces:**
- Consumes: `tests.rs` from Task 1.
- Produces: `fn state_from_elements(a: f64, e: f64, i_deg: f64, node_deg: f64,
  argp_deg: f64, nu_deg: f64, mu: f64) -> ([f64; 3], [f64; 3])` — the shared
  independent forward reference, returning `(position_au, velocity_au_per_day)`.
  Tasks 3 and 6 call it.

- [ ] **Step 1: Add the forward reference helper**

Append to `tests.rs`. This is the **published Gaussian construction**: an
in-plane conic point in the perifocal basis, rotated by `R3(-Ω)·R1(-i)·R3(-ω)`.
It is not the crate's inverse (eccentricity-vector / vis-viva) computation.

```rust
/// Independent forward reference: Keplerian elements -> state vector, via the
/// published perifocal basis and the R3(-node)R1(-incl)R3(-argp) rotation.
/// Deliberately NOT the crate's inverse formulation, so a sign or operator
/// error in `apsides` cannot be masked by a shared expression.
fn state_from_elements(
    a: f64,
    e: f64,
    i_deg: f64,
    node_deg: f64,
    argp_deg: f64,
    nu_deg: f64,
    mu: f64,
) -> ([f64; 3], [f64; 3]) {
    let (i, o, w, nu) = (
        i_deg.to_radians(),
        node_deg.to_radians(),
        argp_deg.to_radians(),
        nu_deg.to_radians(),
    );
    let p = a * (1.0 - e * e);
    let r = p / (1.0 + e * nu.cos());
    let rp = [r * nu.cos(), r * nu.sin(), 0.0];
    let k = (mu / p).sqrt();
    let vp = [-k * nu.sin(), k * (e + nu.cos()), 0.0];
    let (co, so, ci, si, cw, sw) = (o.cos(), o.sin(), i.cos(), i.sin(), w.cos(), w.sin());
    let m = [
        [co * cw - so * sw * ci, -co * sw - so * cw * ci, so * si],
        [so * cw + co * sw * ci, -so * sw + co * cw * ci, -co * si],
        [sw * si, cw * si, ci],
    ];
    let rot = |q: [f64; 3]| -> [f64; 3] {
        [
            m[0][0] * q[0] + m[0][1] * q[1] + m[0][2] * q[2],
            m[1][0] * q[0] + m[1][1] * q[1] + m[1][2] * q[2],
            m[2][0] * q[0] + m[2][1] * q[1] + m[2][2] * q[2],
        ]
    };
    (rot(rp), rot(vp))
}
```

- [ ] **Step 2: Write the failing test**

Append to `tests.rs`. Geometry: `a = 2.0`, `e = 0.2`, `i = 10°`, `Ω = 40°`,
`ω = 30°`, `ν = 50°` (non-apsidal), `mu = 2.959e-4`. This yields
`r·v = 3.24e-3` (far from zero) and a perigee at longitude `69.62°`,
latitude `4.98°` (all three `e_hat` components non-zero).

```rust
/// Pins both apsides at a geometry that is neither apsidal (r.v != 0, so the
/// c2 term is live) nor axis-aligned (all three e_hat components non-zero),
/// against the independent forward construction. Also checks the bifocal sum
/// r_apo + r_peri = 2a -- a defining property of the ellipse that the crate
/// never evaluates.
#[test]
fn apsides_match_forward_construction_at_non_degenerate_geometry() {
    let (a, e, incl, node, argp, mu) = (2.0, 0.2, 10.0, 40.0, 30.0, 2.959e-4);
    let (pos, vel) = state_from_elements(a, e, incl, node, argp, 50.0, mu);

    // Precondition: this state must break BOTH halves of the blind spot.
    let rv = pos[0] * vel[0] + pos[1] * vel[1] + pos[2] * vel[2];
    assert!(rv.abs() > 1e-6, "state must be non-apsidal, r.v = {rv}");

    let aps = apsides(pos, vel, mu).unwrap();
    assert!((aps.eccentricity - e).abs() < 1e-12, "ecc {}", aps.eccentricity);
    assert!((aps.semi_major_au - a).abs() < 1e-12, "a {}", aps.semi_major_au);

    // Independent expected apsis directions: forward-rotate the in-plane points
    // at true anomaly 0 (perihelion) and 180 (aphelion).
    let (pp, _) = state_from_elements(a, e, incl, node, argp, 0.0, mu);
    let (pa, _) = state_from_elements(a, e, incl, node, argp, 180.0, mu);
    for (got, want) in [(aps.perigee, pp), (aps.apogee, pa)] {
        let r = (want[0] * want[0] + want[1] * want[1] + want[2] * want[2]).sqrt();
        let lon = want[1].atan2(want[0]).to_degrees().rem_euclid(360.0);
        let lat = (want[2] / r).asin().to_degrees();
        assert!((got.longitude_deg - lon).abs() < 1e-10, "lon {}", got.longitude_deg);
        assert!((got.latitude_deg - lat).abs() < 1e-10, "lat {}", got.latitude_deg);
        assert!((got.distance_au - r).abs() < 1e-12, "dist {}", got.distance_au);
    }

    // Conic invariant the crate never computes.
    assert!(
        (aps.apogee.distance_au + aps.perigee.distance_au - 2.0 * a).abs() < 1e-12,
        "bifocal sum"
    );
}
```

- [ ] **Step 3: Run the test**

Run: `mise exec -- cargo nextest run -p pleiades-apsides`
Expected: PASS. (This test passes against correct code — it is a *mutant*-
failing test, not a red-green test. The measured residuals are `2.84e-14°` in
longitude and `2.67e-15°` in latitude against a `1e-10` tolerance.)

- [ ] **Step 4: Verify the ten kills**

Run the authoritative command. Expected: `33 missed → 23 missed`, and
`missed.txt` no longer contains any of `112:17`, `114:19`, `115:19`, `115:24`,
`116:19`, `139:39`, `139:49`, `139:58`, `139:68`.

- [ ] **Step 5: Commit**

```bash
mise exec -- cargo fmt -p pleiades-apsides
git add crates/pleiades-apsides/src/tests.rs
git commit -m "test(apsides): pin apsides at a non-degenerate geometry (33 -> 23)

Every existing test used a state that was simultaneously apsidal (r.v = 0,
so c2 = 0) and axis-aligned (e_hat = [1,0,0]), blinding the whole c2 term and
both cross-component scalings. One geometry that is neither kills ten mutants.

Expected values come from an independent forward Gaussian construction, not
from the inverse under test."
```

---

### Task 3: Southern perihelion branch

**Kills 5 mutants:** `226:20` (`<`→`==`), `227:21`×2 (`*`→`+`, `*`→`/`),
`227:45`×2 (`-`→`+`, `-`→`/`).

**Why they survive today:** no existing test reaches the `peri_vec[2] < 0.0`
branch at all. Every current geometry has the perihelion north of the reference
plane, so `omega = 2.0 * PI - omega` never executes and its four operator swaps
are unexercised. With `ω = 210°`, `sin ω < 0` puts the perihelion south, and
`cos_omega = cos(210°) = -0.866` gives `acos = 150°`; the correct branch turns
that into `360 - 150 = 210°`, so `peri_lon_deg = 40 + 210 = 250°`. Each mutant
lands elsewhere: `==` gives `190°`, `-`→`+` gives `190°`, `-`→`/` gives
`177.5°`, `*`→`+` gives `184.6°`, `*`→`/` gives `286.5°`.

**Files:**
- Modify: `crates/pleiades-apsides/src/tests.rs`

**Interfaces:**
- Consumes: `state_from_elements` from Task 2.
- Produces: nothing consumed later.

- [ ] **Step 1: Write the test**

```rust
/// Exercises the southern-perihelion branch (peri_vec[2] < 0), where
/// `omega = 2*PI - omega` runs. No other test reaches it. Elements are
/// recovered from a state built by the independent forward construction.
#[test]
fn elements_recover_southern_perihelion_argument() {
    let (a, e, incl, node, argp, mu) = (2.0, 0.2, 10.0, 40.0, 210.0, 2.959e-4);
    let (pos, vel) = state_from_elements(a, e, incl, node, argp, 50.0, mu);
    let el = elements_from_state(pos, vel, mu).unwrap();
    assert!((el.node_deg - node).abs() < 1e-9, "node {}", el.node_deg);
    assert!((el.incl_deg - incl).abs() < 1e-9, "incl {}", el.incl_deg);
    assert!(
        (el.peri_lon_deg - (node + argp)).abs() < 1e-9,
        "peri_lon {}",
        el.peri_lon_deg
    );
}
```

- [ ] **Step 2: Run the test**

Run: `mise exec -- cargo nextest run -p pleiades-apsides`
Expected: PASS.

- [ ] **Step 3: Verify the five kills**

Run the authoritative command. Expected: `23 missed → 18 missed`; `226:20 <
with ==`, `227:21`×2 and `227:45`×2 all absent from `missed.txt`. **`226:20 <
with <=` must still be present** — it is a documented equivalent handled in
Task 8.

- [ ] **Step 4: Commit**

```bash
mise exec -- cargo fmt -p pleiades-apsides
git add crates/pleiades-apsides/src/tests.rs
git commit -m "test(apsides): exercise the southern-perihelion branch (23 -> 18)

No existing test reached peri_vec[2] < 0, so the omega = 2*PI - omega
correction and its four operator swaps were entirely unexercised."
```

---

### Task 4: `points_from_elements` free-parameter guards

**Kills 7 mutants:** `248:9`, `249:9`, `250:9`, `251:9` (`&&`→`||`),
`255:10`×2 (`<`→`<=`, `<`→`==`), `258:17` (`||`→`&&`).

**Why one input kills all four `&&` mutants:** the guard is
`!(e.is_finite() && a.is_finite() && node.is_finite() && peri_lon.is_finite()
&& incl.is_finite())`. Setting `a = f64::NEG_INFINITY` with every other field
finite makes the original conjunction `false`, so the original returns
`Err(NonFinite)`. Each mutant splits the chain into
`(left group) || (right group)`; because `a` appears in the left group of
`249`/`250`/`251` and `e` is finite for `248`, every mutant evaluates `true`,
skips the guard, and falls through to `a <= 0.0` — returning
`Err(UnboundOrbit)`. A different error variant is an observable difference.

**Caution — do not use `NaN` here.** Every `NaN` field poisons the downstream
arithmetic and `to_ecliptic` returns `Err(NonFinite)` anyway, so the mutant and
the original agree and nothing is killed. `-inf` works precisely because it is
caught by the *later* `a <= 0.0` check with a *different* variant.

**Files:**
- Modify: `crates/pleiades-apsides/src/tests.rs`

**Interfaces:**
- Consumes: `tests.rs` from Task 1.
- Produces: nothing consumed later.

- [ ] **Step 1: Write the three tests**

```rust
/// A non-finite semi-major axis must be rejected as NonFinite by the input
/// guard, not fall through to the later `a <= 0.0` unbound check. Setting
/// a = -inf with every other field finite distinguishes all four of the
/// guard's `&&` operators at once: each mutant reaches `a <= 0.0` and returns
/// UnboundOrbit instead.
#[test]
fn points_from_elements_rejects_non_finite_semi_major() {
    let el = KeplerianElements {
        node_deg: 40.0,
        peri_lon_deg: 70.0,
        incl_deg: 10.0,
        eccentricity: 0.5,
        semi_major_au: f64::NEG_INFINITY,
    };
    assert_eq!(
        points_from_elements(&el, false).unwrap_err(),
        ApsidesError::NonFinite
    );
}

/// The eccentricity floor is exclusive: e exactly at MIN_ECCENTRICITY is
/// accepted, e below it is DegenerateOrbit. Here `e` is a caller-supplied
/// field, so the boundary is directly addressable.
#[test]
fn points_from_elements_eccentricity_floor_is_exclusive() {
    let mk = |e: f64| KeplerianElements {
        node_deg: 40.0,
        peri_lon_deg: 70.0,
        incl_deg: 10.0,
        eccentricity: e,
        semi_major_au: 2.0,
    };
    assert!(points_from_elements(&mk(MIN_ECCENTRICITY), false).is_ok());
    assert_eq!(
        points_from_elements(&mk(1e-7), false).unwrap_err(),
        ApsidesError::DegenerateOrbit
    );
}

/// Both halves of `e >= 1.0 || a <= 0.0` independently mean "not an ellipse".
#[test]
fn points_from_elements_rejects_unbound_conics() {
    let mk = |e: f64, a: f64| KeplerianElements {
        node_deg: 40.0,
        peri_lon_deg: 70.0,
        incl_deg: 10.0,
        eccentricity: e,
        semi_major_au: a,
    };
    assert_eq!(
        points_from_elements(&mk(1.5, 2.0), false).unwrap_err(),
        ApsidesError::UnboundOrbit
    );
    assert_eq!(
        points_from_elements(&mk(0.5, -2.0), false).unwrap_err(),
        ApsidesError::UnboundOrbit
    );
}
```

- [ ] **Step 2: Run the tests**

Run: `mise exec -- cargo nextest run -p pleiades-apsides`
Expected: PASS.

- [ ] **Step 3: Verify the seven kills**

Run the authoritative command. Expected: `18 missed → 11 missed`; `248:9`,
`249:9`, `250:9`, `251:9`, `255:10`×2 and `258:17` absent from `missed.txt`.

- [ ] **Step 4: Commit**

```bash
mise exec -- cargo fmt -p pleiades-apsides
git add crates/pleiades-apsides/src/tests.rs
git commit -m "test(apsides): pin points_from_elements input guards (18 -> 11)

a = -inf with all other fields finite distinguishes all four of the finite
guard's && operators at once: each mutant falls through to `a <= 0.0` and
returns UnboundOrbit where the original returns NonFinite. NaN does not work
here -- it poisons the downstream arithmetic and both branches agree."
```

---

### Task 5: Overflow-lens guards and negative μ

**Kills 3 mutants:** `76:23` (`||`→`&&`), `104:27` (`||`→`&&`), `104:62`
(`||`→`&&`).

**The overflow lens** (`[[fu9-guard-equivalence-overflow-lens]]`): components
that are individually finite can still overflow a squared-norm sum to `+inf`.
`to_ecliptic([1e200; 3])` gives `r = inf`; the original guard rejects it, but
the mutant proceeds and produces a *finite* result — `atan2(1e200, 1e200) = 45°`
and `asin(1e200 / inf) = asin(0) = 0` both pass the line-81 check — so it
returns `Ok` where the original returns `Err`. The same input shape drives
`apsides`' `104:27`, though by a different exit: there the mutant proceeds past
the guard and reaches `inv_a = 2.0/inf - 1e-120`, which is `<= 0`, so it returns
`Err(UnboundOrbit)` — still a kill, because `UnboundOrbit != NonFinite`.

`104:62` is the `mu <= 0.0` arm, reached by a **negative μ**: the original
returns `Err(NonFinite)`, the mutant returns `Ok` with eccentricity `2.0`.

**Files:**
- Modify: `crates/pleiades-apsides/src/tests.rs`

**Interfaces:**
- Consumes: `tests.rs` from Task 1. `to_ecliptic` is private — reachable
  because `tests.rs` is an in-crate submodule with `use super::*`.
- Produces: nothing consumed later.

- [ ] **Step 1: Write the three tests**

```rust
/// Overflow lens: each component is finite, but the squared norm overflows to
/// +inf. The guard must reject this. Under the mutant the function proceeds
/// and returns Ok, because p[i]/inf = 0 makes both output angles finite.
#[test]
fn to_ecliptic_rejects_overflowing_norm() {
    assert_eq!(
        to_ecliptic([1e200, 1e200, 1e200]).unwrap_err(),
        ApsidesError::NonFinite
    );
}

/// The same overflow lens at the `apsides` input boundary.
#[test]
fn apsides_rejects_overflowing_position_norm() {
    assert_eq!(
        apsides([1e200, 1e200, 1e200], [1e-60, 0.0, 0.0], 1.0).unwrap_err(),
        ApsidesError::NonFinite
    );
}

/// A non-positive gravitational parameter is not a physical system.
#[test]
fn apsides_rejects_non_positive_mu() {
    assert_eq!(
        apsides([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], -1.0).unwrap_err(),
        ApsidesError::NonFinite
    );
}
```

- [ ] **Step 2: Run the tests**

Run: `mise exec -- cargo nextest run -p pleiades-apsides`
Expected: PASS.

- [ ] **Step 3: Verify the three kills**

Run the authoritative command. Expected: `11 missed → 8 missed`; `76:23`,
`104:27` and `104:62` absent. **`104:43` must still be present** — it is a
documented equivalent handled in Task 8.

- [ ] **Step 4: Commit**

```bash
mise exec -- cargo fmt -p pleiades-apsides
git add crates/pleiades-apsides/src/tests.rs
git commit -m "test(apsides): pin finiteness guards via the overflow lens (11 -> 8)

Finite components whose squared norm overflows to +inf reach the guards with
r = inf; the mutated guards proceed and return Ok because p[i]/inf = 0 leaves
both output angles finite. Negative mu covers the mu <= 0.0 arm."
```

---

### Task 6: Radial motion and the node threshold

**Kills 3 mutants:** `204:27` (`||`→`&&`), `210:14` (`<`→`<=`), `210:22`
(`*`→`/`).

**`204` needs a crafted radial state.** Radial motion gives `h = r × v = 0`
exactly, which is the distinguishing region. But a *plain* radial state has
`e == 1.0` exactly (algebraically, `e_vec = -1` for radial motion), so
`r_peri = a(1 - e) = 0` and `to_ecliptic` returns `Err(NonFinite)` **inside
`apsides()`** — line 204 is never reached and nothing is killed. The velocity
below is chosen so rounding leaves `e = 0.9999999999999999`, giving
`r_peri = 1.11e-16 ≠ 0` so `apsides()` succeeds. Then the original returns
`Err(NonFinite)` at line 204 while the mutant returns
`Ok(node = 180, incl = NaN, peri_lon = NaN)`.

**`210:14`** differs only at `n_mag == 1e-12 * h_mag` exactly. Craft it:
`cross([1,0,0], [0, 1, -1e-12]) = [0, 1e-12, 1]` exactly (products with `0` and
`1` are exact). Then `h_mag = sqrt(1e-24 + 1) = 1.0` exactly (`1e-24` is far
below `ulp(1)`), and `n_mag = sqrt(fl(1e-12²)) = 1e-12` exactly, because
`sqrt(fl(x·x)) == |x|` for all normal `x`. So `1e-12 * h_mag == n_mag` exactly.
`mu = 0.8` keeps `inv_a = 0.75 > 0` and `e = 0.25 ≥ MIN_ECCENTRICITY`.

**`210:22`** (`1e-12 * h_mag` vs `1e-12 / h_mag`) needs `n_mag` between the
two. For the Task 2 geometry `h_mag = 0.0238`, so the window is
`(2.38e-14, 4.19e-11)`, i.e. `sin i ∈ (1e-12, 1.76e-9)`. `i = 1e-10°`
(`sin i ≈ 1.75e-12`) sits inside it: the original proceeds, the mutant returns
`DegenerateNode`.

**Files:**
- Modify: `crates/pleiades-apsides/src/tests.rs`

**Interfaces:**
- Consumes: `state_from_elements` from Task 2; private `cross` and `norm`.
- Produces: nothing consumed later.

- [ ] **Step 1: Write the three tests**

```rust
/// Radial motion has zero angular momentum, so no orbital plane and no node.
///
/// The velocity is crafted, not arbitrary: a plain radial state has e == 1.0
/// exactly, which makes r_peri = a(1-e) = 0 and fails inside apsides() before
/// the h_mag guard is reached. This vx leaves e one ulp below 1.
#[test]
fn radial_motion_has_no_orbital_plane() {
    let pos = [2.0, 0.0, 0.0];
    let vel = [f64::from_bits(0x3f50624dd2f1aa15), 0.0, 0.0];
    let mu = 2.959e-4;

    // Preconditions: the state is radial AND apsides() succeeds, or line 204
    // is unreachable and this test proves nothing.
    assert_eq!(cross(pos, vel), [0.0, 0.0, 0.0], "state must be radial");
    let aps = apsides(pos, vel, mu).expect("crafted state must form an ellipse");
    assert_ne!(aps.perigee.distance_au, 0.0, "r_peri must be non-zero");

    assert_eq!(
        elements_from_state(pos, vel, mu).unwrap_err(),
        ApsidesError::NonFinite
    );
}

/// The node-degeneracy floor is exclusive: a state sitting exactly on
/// n_mag == 1e-12 * h_mag is accepted.
#[test]
fn node_threshold_is_exclusive_at_the_exact_boundary() {
    let pos = [1.0, 0.0, 0.0];
    let vel = [0.0, 1.0, -1e-12];
    let mu = 0.8;

    // Precondition: the crafted state must sit exactly ON the threshold.
    let h = cross(pos, vel);
    let h_mag = norm(h);
    let n_mag = norm([-h[1], h[0], 0.0]);
    assert_eq!(n_mag, 1e-12 * h_mag, "crafted state must sit ON the boundary");

    assert!(elements_from_state(pos, vel, mu).is_ok());
}

/// The node floor scales with |h| multiplicatively, not inversely. At this
/// inclination n_mag sits above 1e-12*h_mag but below 1e-12/h_mag, so the two
/// formulations disagree.
#[test]
fn node_threshold_scales_multiplicatively_with_angular_momentum() {
    let (a, e, node, argp, mu) = (2.0, 0.2, 40.0, 30.0, 2.959e-4);
    let (pos, vel) = state_from_elements(a, e, 1e-10, node, argp, 50.0, mu);

    // Precondition: n_mag must lie strictly between the two formulations.
    let h = cross(pos, vel);
    let h_mag = norm(h);
    let n_mag = norm([-h[1], h[0], 0.0]);
    assert!(n_mag > 1e-12 * h_mag, "must be above the correct floor");
    assert!(n_mag < 1e-12 / h_mag, "must be below the mutated floor");

    let el = elements_from_state(pos, vel, mu).unwrap();
    assert!(el.incl_deg < 1e-8, "incl {}", el.incl_deg);
}
```

- [ ] **Step 2: Run the tests**

Run: `mise exec -- cargo nextest run -p pleiades-apsides`
Expected: PASS, including all four precondition asserts.

- [ ] **Step 3: Verify the three kills**

Run the authoritative command. Expected: `8 missed → 5 missed`; `204:27`,
`210:14` and `210:22` absent.

- [ ] **Step 4: Commit**

```bash
mise exec -- cargo fmt -p pleiades-apsides
git add crates/pleiades-apsides/src/tests.rs
git commit -m "test(apsides): pin the h=0 guard and node threshold (8 -> 5)

The radial state is crafted: plain radial motion has e == 1.0 exactly, so
r_peri = 0 and apsides() fails before the h_mag guard is reached. The node
threshold is pinned by a state sitting exactly on n_mag == 1e-12*h_mag and by
a tiny inclination that separates 1e-12*h_mag from 1e-12/h_mag."
```

---

### Task 7: The exact eccentricity boundary

**Kills 1 mutant:** `122:10` (`<`→`<=`).

**Why this is the hard one.** Unlike `points_from_elements`' identical
comparison at line 255, `e` here is *derived*: `norm(c1·r − c2·v)` with
`c1 = (v² − μ/r)/μ`. Near `e ≈ 1e-6` that is a catastrophic cancellation, so
reachable `e` values sit on a grid of ~`2.2e-10` relative spacing — about `1e6`
times coarser than the `~2.1e-22` precision needed to hit `MIN_ECCENTRICITY`
exactly.

**Dead end — do not re-run it.** Sweeping `r_mag` over **powers of two** makes
the final `fl(c1 · rx)` product exact and therefore coarse. Two searches in that
family (~160k and ~11.1M trials) found nothing.

**What works:** sweeping `r_mag` over **consecutive doubles** gives that
multiply an independent rounding. The exact product moves by `≈ 2.2e-22` per
`r_mag` ulp, comparable to `ulp(1e-6)`, so hits occur at roughly `1e-6` density.
The state below was found that way and verified against the crate: `apsides`
returns `Ok` with eccentricity **bit-identical** to `MIN_ECCENTRICITY`. The
original accepts it (`1e-6 < 1e-6` is false); the mutant returns
`Err(DegenerateOrbit)`.

**Files:**
- Modify: `crates/pleiades-apsides/src/tests.rs`

**Interfaces:**
- Consumes: `tests.rs` from Task 1; private `MIN_ECCENTRICITY`.
- Produces: nothing consumed later.

- [ ] **Step 1: Write the test**

```rust
/// The eccentricity floor is exclusive: a state whose osculating eccentricity
/// is bit-identical to MIN_ECCENTRICITY is accepted, not rejected as
/// degenerate.
///
/// Unlike `points_from_elements`, `e` is derived here, through a cancellation
/// that makes the reachable grid ~1e6x coarser than the target's precision.
/// This state was found by sweeping r_mag over consecutive doubles so the
/// final c1*r_mag product gets an independent rounding; sweeping powers of two
/// makes that product exact and never hits the boundary.
#[test]
fn apsides_eccentricity_floor_is_exclusive() {
    let rx = f64::from_bits(0x400000000005a740); // 2.0000000001645333
    let vy = f64::from_bits(0x3fe6a09f244b3b60); // 0.707107134710764
    let aps = apsides([rx, 0.0, 0.0], [0.0, vy, 0.0], 1.0).unwrap();

    // Precondition: the crafted state must sit exactly ON the boundary.
    assert_eq!(
        aps.eccentricity,
        MIN_ECCENTRICITY,
        "crafted state must sit ON the boundary"
    );
}
```

- [ ] **Step 2: Run the test**

Run: `mise exec -- cargo nextest run -p pleiades-apsides`
Expected: PASS, including the bit-exact precondition.

- [ ] **Step 3: Verify the kill**

Run the authoritative command. Expected: `5 missed → 4 missed`; `122:10`
absent. The remaining 4 are the documented equivalents.

- [ ] **Step 4: Commit**

```bash
mise exec -- cargo fmt -p pleiades-apsides
git add crates/pleiades-apsides/src/tests.rs
git commit -m "test(apsides): pin the exact eccentricity floor (5 -> 4)

e is derived through a cancellation whose reachable grid is ~1e6x coarser
than ulp(1e-6), so the boundary state had to be searched for. Sweeping r_mag
over consecutive doubles (not powers of two, which make the final product
exact) finds a state whose eccentricity is bit-identical to MIN_ECCENTRICITY."
```

---

### Task 8: Document equivalents, expand the weekly tier, record the slice

**Files:**
- Modify: `crates/pleiades-apsides/src/lib.rs` (comments only, at the four equivalent sites)
- Modify: `crates/pleiades-apsides/src/tests.rs` (μ provenance test)
- Modify: `mise.toml`
- Modify: `.github/workflows/mutants.yml`
- Modify: `docs/follow-ups.md`
- Modify: `docs/superpowers/specs/notes/2026-07-25-mutants-roadmap-baseline.md`

**Interfaces:**
- Consumes: the `4 missed` state from Task 7.
- Produces: the slice's closing record.

- [ ] **Step 1: Add the μ provenance test**

This kills no mutant — `cargo-mutants` does not mutate `const` items, so
`MU_EARTH_MOON_AU3_PER_DAY2` has no mutant attached. It is included because the
design's third reference lever promised it and it documents the constant's
provenance. State that honestly in the doc comment; do not claim a kill.

The derivation gives `8.997011530622141e-10` against the constant's
`8.99714e-10` — a `1.43e-5` relative difference, consistent with the rustdoc's
"Starting value; tuned against the `validate-lilith` gate". The tolerance
below encodes that, and the comment says why it is not tighter.

```rust
/// Documents the provenance of MU_EARTH_MOON_AU3_PER_DAY2 by recomputing it
/// from the published constants it cites, outside the code.
///
/// No mutant is attached to this constant (cargo-mutants does not mutate
/// consts); this test exists so the tuned value cannot drift from its
/// documented derivation unnoticed. The 2e-5 tolerance is the *measured*
/// 1.43e-5 gap between the pure derivation and the shipped value, which the
/// rustdoc attributes to tuning against the validate-lilith gate. It is not a
/// tolerance chosen to make the assertion pass.
#[test]
fn mu_matches_its_published_derivation() {
    const GM_EARTH_KM3_S2: f64 = 398_600.4418;
    const GM_MOON_KM3_S2: f64 = 4_902.800;
    const AU_KM: f64 = 149_597_870.7;
    const DAY_S: f64 = 86_400.0;
    let derived =
        (GM_EARTH_KM3_S2 + GM_MOON_KM3_S2) * DAY_S * DAY_S / (AU_KM * AU_KM * AU_KM);
    let rel = (MU_EARTH_MOON_AU3_PER_DAY2 / derived - 1.0).abs();
    assert!(rel < 2e-5, "mu drifted from its derivation: rel {rel:e}");
}
```

- [ ] **Step 2: Document the four equivalents in place**

Add a comment at each site in `lib.rs`. Do **not** add `#[mutants::skip]`.

At line 81 (`to_ecliptic`'s output guard):
```rust
    // Documented equivalent mutant (FU-9): `||` -> `&&` here is
    // indistinguishable. Line 76 has already established that `r` is finite and
    // non-zero, so `p` is componentwise finite, `atan2` is total, and
    // `p[2] / r` lies in [-1, 1] making `asin` total. No input makes exactly
    // one of the two non-finite.
```

At line 104 (the second `||`, i.e. between `r_mag == 0.0` and `!mu.is_finite()`):
```rust
    // Documented equivalent mutant (FU-9): `&&` binds tighter than `||`, so
    // mutating THIS operator yields `a || (b && c) || d`. Its only
    // distinguishing region is {r_mag == 0.0, mu finite, mu > 0}, where
    // mu / r_mag = +inf forces c1 = -inf and poisons `e` to inf or NaN in every
    // sub-case (all-zero pos -> NaN; all-subnormal pos whose squared norm
    // underflows -> inf; mixed -> NaN). All fail `!e.is_finite()` below, so
    // both branches return Err(NonFinite). This arm is redundant with that
    // downstream check.
```

At line 226 (`peri_vec[2] < 0.0`):
```rust
    // `<` -> `<=` is a documented equivalent mutant (FU-9): the two differ only
    // at peri_vec[2] == 0.0 exactly, i.e. perihelion in the reference plane,
    // which with a non-degenerate inclination means omega in {0, PI}. There
    // acos returns exactly 0 or PI, and 2*PI - 0 = 2*PI and 2*PI - PI = PI both
    // give the same longitude after the rem_euclid(360) below.
```

At line 287 (the aphelion argument of latitude):
```rust
        // `+` -> `-` is a documented equivalent mutant (FU-9): omega + PI and
        // omega - PI differ by exactly 2*PI, so cos/sin agree to within
        // rounding. Measured displacement at (node 40, incl 10, a 2, e 0.2):
        // <= 2.84e-14 deg in longitude and <= 2.44e-15 deg in latitude, i.e.
        // below 1e-10 arcsec. Any assertion tight enough to kill it would be
        // pinning the code's own output.
```

- [ ] **Step 3: Confirm the comments changed no behavior**

Run: `mise exec -- cargo nextest run -p pleiades-apsides`
Expected: PASS. Then re-run the authoritative command; still exactly `4 missed`,
the same four.

- [ ] **Step 4: Expand the weekly tier**

In `mise.toml`, add `-p pleiades-apsides` to `[tasks.mutants]`, alongside the
existing `-p pleiades-types -p pleiades-time -p pleiades-apparent -p
pleiades-houses`.

In `.github/workflows/mutants.yml`, update the calibration comment: the weekly
projection becomes `2,620 + 223 = 2,843` mutants (~1.09x the previous
projection), roughly **+2 minutes** on the measured ~24-25m job wall-clock.
Mark this explicitly as a **projection, not a measurement** — the next
scheduled run supersedes it. `timeout-minutes: 90` is retained unchanged.

- [ ] **Step 5: Verify the whole-crate result and build the margin table**

Run the authoritative command one final time and record the verbatim summary
line. Expected: `223 mutants tested: 4 missed, 215 caught, 4 unviable`.

Build the **per-mutant margin table** (`[[fu9-margin-table-per-mutant-rows]]`):
one row per mutant, never aggregated. Most kills in this slice are **exact
equality or error-variant pins**, not scalar displacements — disclose that
per row rather than fabricating a margin. Only the Task 2 and Task 3 rows carry
a genuine displacement (assertion tolerance vs. the mutant's actual
displacement); measure each by applying the mutant and recording the value.

- [ ] **Step 6: Record the slice in `docs/follow-ups.md`**

Append an FU-9 progress note in the established format, framed as a **new
post-baseline expansion slice** (not part of the closed three-crate baseline or
the closed houses campaign). It must state:

- Measured baseline `223 / 33 missed` at `7aefa946c`; final `223 / 4 missed`.
- The four documented equivalents with their reachability arguments, taking the
  campaign-wide running tally `45 -> 49`.
- **Three design-phase predictions that measurement overturned**, so none is
  silent: (1) `122` was predicted an unreachable boundary and is killable — the
  failed searches swept `r_mag` over powers of two, which makes the final
  product exact; (2) `104:43` was predicted killable and is an equivalent —
  Rust's `&&`/`||` precedence makes the mutant `a || (b && c) || d`, not
  `((a || b) && c) || d`; (3) `204` was predicted killable by any radial state,
  but plain radial motion has `e == 1.0` exactly and fails inside `apsides()`
  before the guard is reached.
- The weekly-tier expansion and its projected (not measured) cost.
- That no parity gate was touched and the tier stays report-only.

- [ ] **Step 7: Update the roadmap note**

In `docs/superpowers/specs/notes/2026-07-25-mutants-roadmap-baseline.md`, mark
`pleiades-apsides` as triaged by this slice so the remaining three measured rows
(`pleiades-backend`, `pleiades-ayanamsa`, `pleiades-fict`) stay an accurate
queue. Do not restate the survivor numbers as if still open.

- [ ] **Step 8: Full CI**

Run: `mise run ci`
Expected: green (fmt + clippy `-D warnings` + workspace test).

- [ ] **Step 9: Commit**

```bash
mise exec -- cargo fmt --all
git add -A
git commit -m "test(apsides): FU-9 apsides mutant triage (33 -> 4 documented equivalents)

Closes the pleiades-apsides slice: 223 mutants, 33 missed -> 4 missed, each
with a written reachability argument (81 totality, 104:43 precedence +
poisoned e, 226 <= modular, 287 periodicity). Campaign tally 45 -> 49.

Adds -p pleiades-apsides to the weekly report-only mutants tier. No parity
gate touched; no mutation-score gate introduced."
```

---

## Spec coverage note — which reference levers were used

The design named three independent reference levers and four conic invariants.
What this plan actually uses, and why:

- **Lever 1, forward construction (elements → state):** used, as the backbone of
  Tasks 2, 3 and 6. It carries the whole numeric class.
- **Lever 2, conic invariants:** only the **bifocal sum** (`r_apo + r_peri = 2a`)
  is asserted, in Task 2. The other three named in the design — vis-viva, the
  conic radius law, and `h ⊥ r, v` — are **deliberately not added**: with the
  forward construction already pinning every apsis coordinate, they kill no
  additional mutant, and the residual reaches 4 documented equivalents without
  them. Adding assertions that constrain nothing new would be noise. They remain
  available if a future change to this crate reopens survivors here.
- **Lever 3, published-constant recomputation:** used in Task 8, with the
  explicit caveat that no mutant is attached to a `const` and the test exists
  for provenance, not for a kill.

This is a narrowing of the design's stated reference surface, recorded here
rather than left silent.

## Verification Summary

| After task | Expected `missed` | Kills |
|------------|-------------------|-------|
| Baseline | 33 | — |
| Task 2 | 23 | 10 |
| Task 3 | 18 | 5 |
| Task 4 | 11 | 7 |
| Task 5 | 8 | 3 |
| Task 6 | 5 | 3 |
| Task 7 | 4 | 1 |
| Task 8 | 4 | 0 (documentation) |

`10 + 5 + 7 + 3 + 3 + 1 = 29` kills, `+ 4` documented equivalents `= 33`.

Every figure in this table was verified end-to-end before the plan was written:
the full test suite was applied to a scratch copy of the crate and the
authoritative command measured `223 mutants tested in 2m: 4 missed, 215 caught,
4 unviable`.

> **This row is superseded — see the Correction note at the top of this
> plan.** Measured outcome is `3`, not `4`.
