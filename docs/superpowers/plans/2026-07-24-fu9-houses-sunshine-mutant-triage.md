# FU-9 Houses Sunshine/Solar-Arc Mutant Triage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive the **Sunshine/solar-arc** family of `crates/pleiades-houses/src/systems/mod.rs` (`sunshine_houses`, `sunshine_offsets`, `apparent_solar_declination`, `apparent_midheaven_declination`, and the shared `nutation_for`) from a **measured 126 surviving mutants to 2 documented equivalents**, reusing the independent house-math reference established by the Foundation PR.

**Architecture:** Fourth PR of the ~6-PR `pleiades-houses` FU-9 campaign (spec: `docs/superpowers/specs/2026-07-22-fu9-houses-mutant-triage-design.md`; Foundation PR: `docs/superpowers/plans/2026-07-22-fu9-houses-foundation-mutant-triage.md`; Great-circle PR: `docs/superpowers/plans/2026-07-23-fu9-houses-greatcircle-mutant-triage.md`; Sector PR: `docs/superpowers/plans/2026-07-24-fu9-houses-sector-mutant-triage.md`). **Tests-only** — no production-code change; every added test lives in `crates/pleiades-houses/src/systems/tests.rs` (`use super::*;` reaches the private functions). The reference note `docs/superpowers/specs/notes/2026-07-22-houses-reference.py` is **extended** with independent `apparent_solar_declination` and `sunshine_offsets` ports.

**Reference strategy, keyed to survivor structure (the key design decision):**
- `apparent_solar_declination`, `apparent_midheaven_declination`, `sunshine_offsets` are **pure numeric functions** of their arguments (no `Instant`-internal helper except `apparent_solar_declination`'s JD arithmetic). Reference = **independent recomputation** from the PUBLISHED low-precision solar formula (NOAA/USNO almanac) and the published Albategnian semi-arc trisection, evaluated outside the crate in `houses-reference.py`, pinning exact `f64` literals at discriminating (non-degenerate) inputs. Every literal in this plan was emitted by that reference during authoring (`docs/superpowers/specs/notes/2026-07-22-houses-reference.py`).
- `sunshine_houses` calls `local_sidereal_time` (GMST+nutation, *not* reproduced), plus `apparent_solar_declination` and `sunshine_offsets` (both pinned independently in this same PR) and the Foundation-pinned `asc1`/`longitude_opposite`/`signed_longitude_difference`. Its 68 survivors are ~55 loop-body arithmetic-operator swaps plus ~13 structural guards (the `acmc < 0` axis flip, the `mc_under_horizon` under-horizon test, and the final `mc_under_horizon` cusp flip). Reference = **independent recomposition** (the `apparent.rs` / Great-circle precedent): a `recompose_sunshine` helper threads `st` from the un-mutated `local_sidereal_time` and `sundec`/`offsets` from the un-mutated (independently-pinned) helpers, recomposes all 12 cusps from the published Sunshine algorithm plus already-pinned shared primitives, then asserts equality against the crate over a **geometry table** chosen so every loop term is non-degenerate and every guard branch is reached. Non-circular: `local_sidereal_time`, `asc1`, and `longitude_opposite` are not mutated in this slice, and `apparent_solar_declination`/`sunshine_offsets` are pinned to their own independent references in Tasks 1–2.
- `nutation_for` returns `(delta_psi_deg, delta_eps_deg)`. **Both** call sites (`asc_mc` at `mod.rs:268`, `validated_obliquity` at `mod.rs:610`) discard the first element (`_dpsi` / `_delta_psi_deg`); only `delta_eps_deg` is observable (it feeds obliquity). Its 2 survivors are both the `delta_psi_arcsec / 3600.0` term (`mod.rs:600:30`), whose value is never read through any public path — these are **documented equivalents** (unobservable output), left visible with a written reachability argument, never `#[mutants::skip]`-suppressed.

**Mutation-triage TDD cycle** (differs from feature TDD — read once): a triage test *passes* on correct `HEAD` code (it pins intent against the independent reference); it *kills a mutant* by failing on the mutated tree. Each task: write the test → run `cargo nextest` to confirm it **passes** on `HEAD` (proves the literal/geometry is right) → the final task re-runs `cargo mutants -F` to confirm the survivors are **caught** down to the 2 documented equivalents.

**Tech Stack:** Rust (stable, via mise), cargo-nextest, cargo-mutants 27.1.0, Python 3 (reference only).

## Global Constraints

- **Tests-only:** no production-code change anywhere in this PR; the only source file modified is `crates/pleiades-houses/src/systems/tests.rs` (plus the reference-note extension and the follow-up entry). No behavior-preserving refactor is needed (unlike Foundation's `apparent.rs`) — every survivor is reachable through the existing private-function surface.
- **No parity-gate change:** no `validate-*` file is touched; the mutants tier stays report-only.
- **No `#[mutants::skip]`:** the 2 residual documented-equivalent mutants (`nutation_for` `delta_psi`) are left visible with a written reachability argument, enumerated in a characterization test (`sunshine_family_equivalent_mutants_are_documented`).
- **Independence discipline:** every expected value comes from the independent Python port (cross-validated against the crate to ~1e-9 at `HEAD` before its literals are trusted), from independent recomposition threaded through un-mutated helpers, or from hand arithmetic — never from running the mutated function and pinning its output.
- **Branch:** create `fu9-houses-sunshine-mutant-triage` off `main` (do not work on `main`).
- All commands run through mise: prefix cargo invocations with `mise exec --`.
- **cargo-mutants + mise trust gotcha (operational):** cargo-mutants copies the workspace to `/tmp/cargo-mutants-workspace-*.tmp`, and this environment's mise refuses the untrusted copied `mise.toml`, failing the baseline build with `Config files ... are not trusted`. Prefix every `cargo mutants` invocation with `MISE_TRUSTED_CONFIG_PATHS=/tmp`. Without it the run aborts at "build failed in an unmutated tree".
- **`[tasks.mutants]` weekly-tier expansion is NOT in this PR.** Adding `-p pleiades-houses` to the default `[tasks.mutants]` set is the **crate-completing** PR 6 step (per the design spec), not this Sunshine PR. Do not touch `mise.toml`.

## Measured baseline (2026-07-24, at `188c5f1c4`)

Authoritative per-file command, scoped to the five Sunshine/solar-arc functions (cargo-mutants 27.1.0):

```bash
MISE_TRUSTED_CONFIG_PATHS=/tmp mise exec -- cargo mutants \
  --test-tool nextest --test-workspace=false --baseline run \
  -p pleiades-houses \
  --file crates/pleiades-houses/src/systems/mod.rs \
  -F 'in (sunshine_houses|sunshine_offsets|apparent_solar_declination|apparent_midheaven_declination|nutation_for)$'
```

**137 mutants tested — 126 missed / 11 caught / 0 unviable** (5 min wall-clock). Survivors by function: `sunshine_houses` **68**, `sunshine_offsets` **34**, `apparent_solar_declination` **20**, `nutation_for` **2**, `apparent_midheaven_declination` **2**. This matches the design spec's whole-crate prediction (68/34/20/2/2) exactly.

`sunshine_houses`'s 68 survivors, by source line (from `mutants.out/missed.txt`):

| Region | Lines | Count | Class |
|--------|-------|-------|-------|
| under-horizon guard | 1545–1546 | 7 | `!=`/`&&`/`>` comparison + `-` swaps on `mc_under_horizon` |
| axis-flip guard | 1552, 1554 | 4 | `<` swaps on `acmc < 0.0`, `delete !` on `!KEEP_MC_SOUTH` |
| per-house loop body | 1571–1602 | 51 | arith-op swaps in `xhs`/`cosa`/`alph`/`alpha2`/`b`/`cosc`/`sinzd`/`rax`/`pole`/`a` |
| final under-horizon flip | 1607, 1609 | 6 | `&&`/`delete !` guard + `house - 1` index swaps |

**Target after this PR: `126 -> 2 documented equivalents`** — `nutation_for` 2→2 (both `delta_psi` unobservable), and `sunshine_houses` 68→0, `sunshine_offsets` 34→0, `apparent_solar_declination` 20→0, `apparent_midheaven_declination` 2→0. The `sunshine_houses` `1546:92` `>`→`>=` measure-zero boundary is **attempted as a crafted exact-`abs()==90` kill** in Task 5 and documented-equivalent only if not `f64`-representable (per `[[fu9-jd-grid-representability]]`); the true killed-vs-equivalent split is **confirmed by the final scoped re-run in Task 6**, not predicted. Scoped re-run target: `2 missed / 135 caught / 0 unviable`.

---

### Task 1: `apparent_midheaven_declination` — 2 survivors → 0

`apparent_midheaven_declination(sidereal_time_deg, obliquity_deg)` (mod.rs:1650–1654) is `atan(sin(st°)·tan(ε°))`. Both survivors are at `1651:43` — the `*` between `sin(st)` and `tan(ε)` (`*`→`+` and `*`→`/`). Killed by a **single crafted geometry** where `sin(st)·tan(ε)`, `sin(st)+tan(ε)`, and `sin(st)/tan(ε)` are three distinct values (avoid `st∈{0,90,180,270}` and `ε` where `tan(ε)∈{0,1}`).

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `apparent_midheaven_declination(sidereal_time_deg: f64, obliquity_deg: f64) -> f64` (private, via `use super::*;`); `assert_close_degrees(actual: f64, expected: f64)` (existing helper, tolerance `1e-12`).
- Produces: nothing consumed by later tasks.

- [ ] **Step 1: Write the failing-on-mutant test**

Append to `crates/pleiades-houses/src/systems/tests.rs`:

```rust
#[test]
fn apparent_midheaven_declination_pins_the_product_form() {
    // Independent reference (houses-reference.py `mid_dec`): atan(sin(st)*tan(eps)).
    // st=57, eps=23.4392811 chosen so sin*tan != sin+tan != sin/tan, killing the
    // 1651:43 `* -> +` and `* -> /` survivors. Literals emitted by the reference.
    assert_close_degrees(
        apparent_midheaven_declination(57.0, 23.4392811),
        19.98167206592639,
    );
    assert_close_degrees(
        apparent_midheaven_declination(205.0, 23.4392811),
        -10.382982940241511,
    );
}
```

- [ ] **Step 2: Run the test — confirm it PASSES on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses apparent_midheaven_declination_pins_the_product_form`
Expected: PASS (proves the literals match un-mutated code to `1e-12`).

- [ ] **Step 3: Confirm the 2 survivors are now caught**

Run:
```bash
MISE_TRUSTED_CONFIG_PATHS=/tmp mise exec -- cargo mutants \
  --test-tool nextest --test-workspace=false \
  -p pleiades-houses --file crates/pleiades-houses/src/systems/mod.rs \
  -F 'in apparent_midheaven_declination$'
```
Expected: `2 mutants tested: 0 missed, 2 caught`.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): FU-9 pin apparent_midheaven_declination product form (2->0)"
```

---

### Task 2: `apparent_solar_declination` — 20 survivors → 0

`apparent_solar_declination(instant, obliquity)` (mod.rs:1633–1648) evaluates the published low-precision Sun: `L = 280.460 + 0.9856474·d`, `g = 357.528 + 0.9856003·d` (both `normalized_0_360`), `λ = L + 1.915·sin g + 0.020·sin 2g` (`normalized_0_360`), `δ = asin(sin ε · sin λ)`, where `d = JD − 2451545.0`. The 20 survivors span the `−` (1634), the two `+ … · d` series terms (1635–1636), the `λ` assembly `+`/`·` (1639–1640), and the final `sin·sin` (1644). Killed by **independent recomputation** at **two non-degenerate instants** (`d ≠ 0`, so every `·d` term is observable; two distinct `g` so the `sin g`/`sin 2g` terms differ).

**Files:**
- Modify: `docs/superpowers/specs/notes/2026-07-22-houses-reference.py` (append `solar_declination` + print)
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `apparent_solar_declination(instant: Instant, obliquity: Angle) -> Angle`; `Instant::new(JulianDay, TimeScale)`; `JulianDay::from_days(f64)`; `TimeScale::Tt`; `Angle::from_degrees(f64)`; `Angle::degrees(&self) -> f64`; `assert_close_degrees`.
- Produces: `docs/.../2026-07-22-houses-reference.py::solar_declination(jd, obl_deg)` (independent reference reused nowhere else, documented for review).

- [ ] **Step 1: Extend the independent reference**

Append to `docs/superpowers/specs/notes/2026-07-22-houses-reference.py`, **before** the `if __name__ == "__main__":` block (uses `norm360`/`D2R` already defined):

```python
def solar_declination(jd, obl_deg):
    """crate `apparent_solar_declination`: the published low-precision Sun
    (NOAA/USNO Astronomical Almanac "Low precision formulae for the Sun").
    Re-derived from the published series, NOT copied operator-by-operator from
    the Rust. d = JD(TT) - J2000; L, g, lambda in degrees; delta in degrees."""
    d = jd - 2451545.0
    L = norm360(280.460 + 0.9856474 * d)
    g = norm360(357.528 + 0.9856003 * d)
    lam = norm360(L + 1.915 * math.sin(g * D2R) + 0.020 * math.sin(2.0 * g * D2R))
    return math.degrees(math.asin(math.sin(obl_deg * D2R) * math.sin(lam * D2R)))
```

Add to the `__main__` block a print so the literals are reproducible:

```python
    print("# solar_declination")
    for jd in (2451600.0, 2455000.0):
        print(f"  jd={jd!r} -> {solar_declination(jd, 23.4392811)!r}")
```

- [ ] **Step 2: Run the reference — confirm the literals**

Run: `python3 docs/superpowers/specs/notes/2026-07-22-houses-reference.py`
Expected output includes:
```
# solar_declination
  jd=2451600.0 -> -9.23948196138383
  jd=2455000.0 -> 23.39101729022372
```

- [ ] **Step 3: Write the failing-on-mutant test**

Append to `crates/pleiades-houses/src/systems/tests.rs`:

```rust
#[test]
fn apparent_solar_declination_pins_the_published_sun_series() {
    // Independent reference (houses-reference.py `solar_declination`), the
    // published NOAA/USNO low-precision Sun. Two non-degenerate instants
    // (d=55 and d=3455 days from J2000) make every `*d` series term and both
    // sin(g)/sin(2g) terms observable, killing all 20 arith survivors.
    let obl = Angle::from_degrees(23.4392811);
    let dec = |jd: f64| {
        apparent_solar_declination(
            Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
            obl,
        )
        .degrees()
    };
    assert_close_degrees(dec(2_451_600.0), -9.23948196138383);
    assert_close_degrees(dec(2_455_000.0), 23.39101729022372);
}
```

- [ ] **Step 4: Run the test — confirm it PASSES on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses apparent_solar_declination_pins_the_published_sun_series`
Expected: PASS.

- [ ] **Step 5: Confirm the 20 survivors are now caught**

Run:
```bash
MISE_TRUSTED_CONFIG_PATHS=/tmp mise exec -- cargo mutants \
  --test-tool nextest --test-workspace=false \
  -p pleiades-houses --file crates/pleiades-houses/src/systems/mod.rs \
  -F 'in apparent_solar_declination$'
```
Expected: `20 mutants tested: 0 missed, 20 caught`.

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/specs/notes/2026-07-22-houses-reference.py crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): FU-9 pin apparent_solar_declination published series (20->0)"
```

---

### Task 3: `sunshine_offsets` — 34 survivors → 0

`sunshine_offsets(latitude_deg, sun_declination_deg)` (mod.rs:1616–1631) computes the ascensional difference `ad = asin(clamp(tan δ · tan φ))`, the nocturnal/diurnal semi-arcs `nsa = 90 − ad`, `dsa = 90 + ad`, and the eight non-zero house offsets (`±nsa/3`, `±2·nsa/3`, `±dsa/3`, `±2·dsa/3`). The 34 survivors are the `tan·tan` product (1618), the `90 ± ad` (1620–1621), the unary `delete -` on the negative offsets (1619, 1622, 1623, 1626, 1627), the `2·` numerators (1622, 1625, 1626, 1629), and the `/3` divisors. Killed by **independent recomputation** at **two geometries** (northern/southern, `δ ≠ 0` and `φ ≠ 0` so `ad ≠ 0`, making `nsa ≠ dsa` and every `±`/`2·`/`/3` term observable).

**Files:**
- Modify: `docs/superpowers/specs/notes/2026-07-22-houses-reference.py` (append `sunshine_offsets` + print)
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `sunshine_offsets(latitude_deg: f64, sun_declination_deg: f64) -> [f64; 13]`; `assert_close_degrees`.
- Produces: `docs/.../2026-07-22-houses-reference.py::sunshine_offsets(lat, sundec)`.

- [ ] **Step 1: Extend the independent reference**

Append to `docs/superpowers/specs/notes/2026-07-22-houses-reference.py`, before `__main__`:

```python
def sunshine_offsets(lat, sundec):
    """crate `sunshine_offsets`: house offsets from the nocturnal/diurnal
    semi-arcs (published Sunshine/solar-arc trisection). Re-derived, not copied.
    Returns the 8 non-zero offsets keyed by house index (2,3,5,6,8,9,11,12)."""
    ad = math.degrees(math.asin(max(-1.0, min(1.0,
        math.tan(sundec * D2R) * math.tan(lat * D2R)))))
    nsa = 90.0 - ad
    dsa = 90.0 + ad
    return {
        2: -2.0 * nsa / 3.0, 3: -nsa / 3.0, 5: nsa / 3.0, 6: 2.0 * nsa / 3.0,
        8: -2.0 * dsa / 3.0, 9: -dsa / 3.0, 11: dsa / 3.0, 12: 2.0 * dsa / 3.0,
    }
```

Add to `__main__`:

```python
    print("# sunshine_offsets")
    for lat, sd in ((52.0, -10.0), (-33.0, 15.0)):
        print(f"  lat={lat} sundec={sd}: {sunshine_offsets(lat, sd)}")
```

- [ ] **Step 2: Run the reference — confirm the literals**

Run: `python3 docs/superpowers/specs/notes/2026-07-22-houses-reference.py`
Expected output includes:
```
# sunshine_offsets
  lat=52.0 sundec=-10.0: {2: -68.69556843044369, 3: -34.34778421522184, 5: 34.34778421522184, 6: 68.69556843044369, 8: -51.30443156955631, 9: -25.652215784778154, 11: 25.652215784778154, 12: 51.30443156955631}
  lat=-33.0 sundec=15.0: {2: -66.68063265912221, 3: -33.340316329561105, 5: 33.340316329561105, 6: 66.68063265912221, 8: -53.31936734087779, 9: -26.659683670438895, 11: 26.659683670438895, 12: 53.31936734087779}
```

- [ ] **Step 3: Write the failing-on-mutant test**

Append to `crates/pleiades-houses/src/systems/tests.rs`:

```rust
#[test]
fn sunshine_offsets_pins_the_semi_arc_trisection() {
    // Independent reference (houses-reference.py `sunshine_offsets`). Two
    // geometries with ad != 0 (so nsa != dsa) make every +/-, 2*, and /3 term
    // observable, killing all 34 arith survivors. Only the 8 non-zero house
    // indices carry signal; the other 5 are hard 0.0 and untested here.
    let check = |lat: f64, sundec: f64, want: [(usize, f64); 8]| {
        let got = sunshine_offsets(lat, sundec);
        for (idx, expected) in want {
            assert_close_degrees(got[idx], expected);
        }
    };
    check(
        52.0,
        -10.0,
        [
            (2, -68.69556843044369),
            (3, -34.34778421522184),
            (5, 34.34778421522184),
            (6, 68.69556843044369),
            (8, -51.30443156955631),
            (9, -25.652215784778154),
            (11, 25.652215784778154),
            (12, 51.30443156955631),
        ],
    );
    check(
        -33.0,
        15.0,
        [
            (2, -66.68063265912221),
            (3, -33.340316329561105),
            (5, 33.340316329561105),
            (6, 66.68063265912221),
            (8, -53.31936734087779),
            (9, -26.659683670438895),
            (11, 26.659683670438895),
            (12, 53.31936734087779),
        ],
    );
}
```

- [ ] **Step 4: Run the test — confirm it PASSES on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses sunshine_offsets_pins_the_semi_arc_trisection`
Expected: PASS.

- [ ] **Step 5: Confirm the 34 survivors are now caught**

Run:
```bash
MISE_TRUSTED_CONFIG_PATHS=/tmp mise exec -- cargo mutants \
  --test-tool nextest --test-workspace=false \
  -p pleiades-houses --file crates/pleiades-houses/src/systems/mod.rs \
  -F 'in sunshine_offsets$'
```
Expected: `34 mutants tested: 0 missed, 34 caught`.

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/specs/notes/2026-07-22-houses-reference.py crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): FU-9 pin sunshine_offsets semi-arc trisection (34->0)"
```

---

### Task 4: `sunshine_houses` loop body + axis flip — recomposition (≈57 survivors → residual guards only)

`sunshine_houses(instant, observer, obliquity, angles)` (mod.rs:1533–1614) is a composed function. The 51 loop-body survivors (1571–1602) plus the axis-flip guard survivors (1552 `acmc < 0`, 1554 `delete !`) are killed by a **`recompose_sunshine` helper** that re-derives all 12 cusps from the published Sunshine algorithm, threading `st` from the un-mutated `local_sidereal_time`, `sundec` from the (Task-2-pinned) `apparent_solar_declination`, and `offsets` from the (Task-3-pinned) `sunshine_offsets`, and calling the Foundation-pinned `asc1`/`longitude_opposite`/`signed_longitude_difference`. Asserting `crate == recompose` at a **geometry table** kills every loop-body arith swap (any non-degenerate geometry) and the axis-flip guards (a table row with `acmc < 0`). The under-horizon guards (1545–1546, 1607, 1609) are handled in Task 5.

**Non-circularity:** `recompose_sunshine` is a *separate* transcription of the published algorithm; a mutation inside `sunshine_houses` does not touch it, so `crate == recompose` fails on the mutated tree. `local_sidereal_time`, `asc1`, `longitude_opposite` are not mutated in this slice; `apparent_solar_declination`/`sunshine_offsets` are independently pinned in Tasks 2–3. This is the accepted campaign recomposition pattern (`[[fu9-houses-reference-independence]]`) — confirmed to reproduce the crate at `HEAD` to `1e-9` before it is trusted.

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `sunshine_houses(instant: Instant, observer: &ObserverLocation, obliquity: Angle, angles: HouseAngles) -> [Longitude; 12]`; `local_sidereal_time(instant: Instant, longitude: Longitude) -> Angle`; `apparent_solar_declination(Instant, Angle) -> Angle`; `sunshine_offsets(f64, f64) -> [f64; 13]`; `asc1(x1: f64, pole_height: f64, sine: f64, cose: f64) -> Longitude`; `longitude_opposite(Longitude) -> Longitude`; `signed_longitude_difference(a: f64, b: f64) -> f64`; `ObserverLocation::new(Latitude, Longitude, Option<f64>)`; `gc_angles(f64, f64) -> HouseAngles` (existing); `Angle`, `Instant`, `JulianDay`, `TimeScale`, `Latitude`, `Longitude`.
- Produces: `recompose_sunshine(instant, observer, obliquity, angles) -> [f64; 12]` and `sun_geom(jd, lat, lon, obl, asc, mc)` builder, consumed by Task 5.

- [ ] **Step 1: Write the `recompose_sunshine` helper + geometry-table test**

Append to `crates/pleiades-houses/src/systems/tests.rs`:

```rust
/// Independent recomposition of `sunshine_houses` (published Sunshine/solar-arc
/// algorithm), threading `st`/`sundec`/`offsets` from the un-mutated (and
/// independently-pinned) helpers. Separate transcription => a mutant inside
/// `sunshine_houses` is not mirrored here, so equality kills it. Returns the 12
/// cusp longitudes in degrees.
fn recompose_sunshine(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Angle,
    angles: HouseAngles,
) -> [f64; 12] {
    let sidereal_time = local_sidereal_time(instant, observer.longitude).degrees();
    let latitude = observer.latitude.degrees();
    let obliquity_deg = obliquity.degrees();
    let sundec = apparent_solar_declination(instant, obliquity).degrees();
    let mc_under_horizon = latitude.signum() != 0.0
        && (latitude - apparent_midheaven_declination(sidereal_time, obliquity_deg)).abs() > 90.0;

    let mut cusps = [0.0_f64; 12];
    let mut ascendant = angles.ascendant;
    let mut midheaven = angles.midheaven;
    let acmc = signed_longitude_difference(ascendant.degrees(), midheaven.degrees());
    if acmc < 0.0 {
        ascendant = longitude_opposite(ascendant);
        midheaven = longitude_opposite(midheaven); // KEEP_MC_SOUTH is const false
    }
    cusps[0] = ascendant.degrees();
    cusps[3] = longitude_opposite(midheaven).degrees();
    cusps[6] = longitude_opposite(ascendant).degrees();
    cusps[9] = midheaven.degrees();

    let offsets = sunshine_offsets(latitude, sundec);
    let sin_ecl = obliquity_deg.to_radians().sin();
    let cos_ecl = obliquity_deg.to_radians().cos();

    for house in [2usize, 3, 5, 6, 8, 9, 11, 12] {
        let offset = offsets[house];
        let xhs = 2.0
            * (sundec.to_radians().cos() * (offset.to_radians() / 2.0).sin())
                .asin()
                .to_degrees();
        let cosa = (sundec.to_radians().tan() * (xhs.to_radians() / 2.0).tan()).clamp(-1.0, 1.0);
        let alph = cosa.acos().to_degrees();
        let (alpha2, b) = if house > 7 {
            (180.0 - alph, 90.0 - latitude + sundec)
        } else {
            (alph, 90.0 - latitude - sundec)
        };
        let cosc = xhs.to_radians().cos() * b.to_radians().cos()
            + xhs.to_radians().sin() * b.to_radians().sin() * alpha2.to_radians().cos();
        let c = cosc.clamp(-1.0, 1.0).acos().to_degrees();
        let sinzd = if c.abs() < f64::EPSILON {
            0.0
        } else {
            xhs.to_radians().sin() * alpha2.to_radians().sin() / c.to_radians().sin()
        };
        let zd = sinzd.clamp(-1.0, 1.0).asin().to_degrees();
        let rax = (latitude.to_radians().cos() * zd.to_radians().tan())
            .atan()
            .to_degrees();
        let pole = (sinzd * latitude.to_radians().sin())
            .clamp(-1.0, 1.0)
            .asin()
            .to_degrees();
        let pole = if house <= 6 { -pole } else { pole };
        let a = if house <= 6 {
            sidereal_time + 180.0 + rax
        } else {
            sidereal_time + rax
        };
        cusps[house - 1] = asc1(a, pole, sin_ecl, cos_ecl).degrees();
    }

    if mc_under_horizon {
        for house in [2usize, 3, 5, 6, 8, 9, 11, 12] {
            cusps[house - 1] = longitude_opposite(Longitude::from_degrees(cusps[house - 1])).degrees();
        }
    }
    cusps
}

/// Builds the `(Instant, ObserverLocation, Angle, HouseAngles)` argument tuple
/// for a Sunshine geometry, so Tasks 4-5 share one constructor.
fn sun_geom(
    jd: f64,
    lat: f64,
    lon: f64,
    obl: f64,
    asc: f64,
    mc: f64,
) -> (Instant, ObserverLocation, Angle, HouseAngles) {
    (
        Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
        ObserverLocation::new(Latitude::from_degrees(lat), Longitude::from_degrees(lon), None),
        Angle::from_degrees(obl),
        gc_angles(asc, mc),
    )
}

#[test]
fn sunshine_houses_matches_independent_recomposition() {
    // Geometry table: row A mid-lat acmc>0 (no axis flip, mc above horizon);
    // row B mid-lat acmc<0 (exercises the 1552 axis flip + 1554 mc flip);
    // row C high-lat with mc below horizon (exercises the per-house loop under a
    // different hemisphere sign). All rows have non-degenerate loop terms, so the
    // 51 loop-body swaps and the 2 axis-flip guards are all killed.
    let obl = 23.4392811;
    let rows = [
        sun_geom(2_451_600.0, 52.0, 10.0, obl, 100.0, 15.0),   // A: acmc = 85 > 0
        sun_geom(2_455_000.0, 40.0, -75.0, obl, 15.0, 100.0),  // B: acmc = -85 < 0
        sun_geom(2_451_600.0, 66.0, 200.0, obl, 300.0, 210.0), // C: high lat
    ];
    for (i, (instant, observer, obliquity, angles)) in rows.iter().enumerate() {
        let got = sunshine_houses(*instant, observer, *obliquity, *angles);
        let want = recompose_sunshine(*instant, observer, *obliquity, *angles);
        for h in 0..12 {
            assert!(
                (got[h].degrees() - want[h]).abs() < 1.0e-9,
                "row {i} cusp[{h}]: crate {} != recompose {}",
                got[h].degrees(),
                want[h]
            );
        }
    }
}
```

- [ ] **Step 2: Run the test — confirm it PASSES on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses sunshine_houses_matches_independent_recomposition`
Expected: PASS (proves `recompose_sunshine` reproduces the crate to `1e-9` at every cusp on the un-mutated tree). If it fails, the recomposition is mis-transcribed — fix the helper, not the crate.

- [ ] **Step 3: Confirm the loop-body + axis-flip survivors are now caught**

Run:
```bash
MISE_TRUSTED_CONFIG_PATHS=/tmp mise exec -- cargo mutants \
  --test-tool nextest --test-workspace=false \
  -p pleiades-houses --file crates/pleiades-houses/src/systems/mod.rs \
  -F 'in sunshine_houses$'
```
Expected: **the 1571–1602 loop-body and 1552/1554 survivors are caught**; only the under-horizon guard survivors (1545–1546, 1607, 1609 — about 15) may remain MISSED. Record the exact residual line list from `mutants.out/missed.txt`; Task 5 targets it.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): FU-9 sunshine_houses recomposition — loop body + axis flip"
```

---

### Task 5: `sunshine_houses` under-horizon guards — crafted geometries → 0 (or documented boundary)

The residual `sunshine_houses` survivors are the `mc_under_horizon` computation (1545 `!= 0.0`, 1546 `&&`, `-`, and `> 90.0`) and its consumer, the final cusp flip (1607 `&&` + `delete !`, 1609 `house - 1` index swaps). These are killed by **crafted geometries** that toggle `mc_under_horizon` and reach the flip loop, asserting against `recompose_sunshine` (which carries the correct guard). The `1546:92` `>`→`>=` boundary differs only at exactly `|latitude − mc_declination| == 90.0`; it is **attempted as an exact-boundary kill** and documented-equivalent only if that value is not `f64`-representable through the trig chain.

**Guard-kill geometry design** (`mc_dec = apparent_midheaven_declination(st, ε) ∈ [−ε, +ε] ≈ ±23.44°`):
- **`mc_under_horizon == true`** needs `|lat − mc_dec| > 90`: high `|lat|` (≥ ~67°) with `mc_dec` of the opposite sign — i.e. `sin(st) < 0` (`st ∈ (180, 360)`) at `lat > 0`. Kills 1607 `delete !` (mutant never flips) and 1609 index swaps (flip loop runs, wrong index observable).
- **`mc_under_horizon == false`** at high `|lat|` with `mc_dec` same sign (`sin(st) > 0`) so `|lat − mc_dec| < 90`. Together with the true row, distinguishes 1545 `!= 0.0` (vs `== 0.0`), 1546 `&&` (vs `||`), 1546 `-` (vs `+`/`/`), 1546 `>` (vs `==`/`<`), and 1607 `&&` (vs `||`).
- **`lat == 0`** row: `latitude.signum() == 0.0`, so `mc_under_horizon` is always `false`; pairs with a `lat ≠ 0`, `|lat−mc_dec|<90` row where the mutated `== 0.0` would flip the result — pins 1545.

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `sunshine_houses`, `recompose_sunshine`, `sun_geom` (Task 4); `apparent_midheaven_declination` (Task 1).
- Produces: nothing consumed later (except the Task-6 documented-equivalent census references this test's coverage).

- [ ] **Step 1: Confirm the guard-toggling geometries hit both branches**

Add a temporary assertion (kept in the final test) that the chosen rows actually straddle the guard, so the kill is not vacuous:

```rust
#[test]
fn sunshine_houses_under_horizon_guard_is_exercised_both_ways() {
    // Precondition: the crafted rows must land on opposite sides of the guard,
    // else the comparison-swap mutants would survive vacuously.
    let obl = 23.4392811;
    let st_true = local_sidereal_time(
        Instant::new(JulianDay::from_days(2_451_600.0), TimeScale::Tt),
        Longitude::from_degrees(200.0),
    )
    .degrees();
    let mc_dec_true = apparent_midheaven_declination(st_true, obl);
    assert!(
        (80.0_f64 - mc_dec_true).abs() > 90.0,
        "expected mc_under_horizon TRUE row: |80 - {mc_dec_true}| must exceed 90"
    );
    let st_false = local_sidereal_time(
        Instant::new(JulianDay::from_days(2_451_600.0), TimeScale::Tt),
        Longitude::from_degrees(20.0),
    )
    .degrees();
    let mc_dec_false = apparent_midheaven_declination(st_false, obl);
    assert!(
        (80.0_f64 - mc_dec_false).abs() < 90.0,
        "expected mc_under_horizon FALSE row: |80 - {mc_dec_false}| must be under 90"
    );
}
```

Run: `mise exec -- cargo nextest run -p pleiades-houses sunshine_houses_under_horizon_guard_is_exercised_both_ways`
Expected: PASS. If either assert fails, adjust the `lon` values (which set `st`, hence `mc_dec`'s sign) until one row is under-horizon and the other is not, then update the guard-kill test below to match.

- [ ] **Step 2: Write the guard-kill test (recomposition equality over the guard-straddling table)**

Append:

```rust
#[test]
fn sunshine_houses_under_horizon_guards_match_recomposition() {
    // Rows chosen (Step 1) to straddle mc_under_horizon and to include a lat==0
    // row, so every 1545/1546/1607/1609 guard mutant changes at least one cusp
    // vs the correct recomposition. lon sets sidereal time (mc_dec sign).
    let obl = 23.4392811;
    let rows = [
        sun_geom(2_451_600.0, 80.0, 200.0, obl, 300.0, 210.0), // mc_under_horizon = true
        sun_geom(2_451_600.0, 80.0, 20.0, obl, 100.0, 15.0),   // mc_under_horizon = false
        sun_geom(2_451_600.0, 0.0, 20.0, obl, 100.0, 15.0),    // lat == 0 -> false
    ];
    for (i, (instant, observer, obliquity, angles)) in rows.iter().enumerate() {
        let got = sunshine_houses(*instant, observer, *obliquity, *angles);
        let want = recompose_sunshine(*instant, observer, *obliquity, *angles);
        for h in 0..12 {
            assert!(
                (got[h].degrees() - want[h]).abs() < 1.0e-9,
                "row {i} cusp[{h}]: crate {} != recompose {}",
                got[h].degrees(),
                want[h]
            );
        }
    }
}
```

- [ ] **Step 3: Run both tests — confirm they PASS on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses sunshine_houses_under_horizon`
Expected: PASS (both).

- [ ] **Step 4: Re-run mutants scoped to `sunshine_houses`; classify the residual**

Run:
```bash
MISE_TRUSTED_CONFIG_PATHS=/tmp mise exec -- cargo mutants \
  --test-tool nextest --test-workspace=false \
  -p pleiades-houses --file crates/pleiades-houses/src/systems/mod.rs \
  -F 'in sunshine_houses$'
```
Expected: `68 mutants tested: 0 missed` (all caught). **If** `1546:92 replace > with >=` remains MISSED, attempt a crafted exact-boundary kill (Step 5); if that is not `f64`-representable, carry it as the sole `sunshine_houses` documented equivalent into Task 6's census and update this plan's target line accordingly.

- [ ] **Step 5 (conditional): exact-`abs()==90` boundary kill attempt**

Only if `1546:92 >`→`>=` survived Step 4. Search for an `(lat, st)` where `latitude − apparent_midheaven_declination(st, ε)` equals exactly `90.0` in `f64` (per `[[fu9-jd-grid-representability]]`, verify with an in-test precondition `assert_eq!(latitude - mc_dec, 90.0)` before pinning). At that input the un-mutated `> 90.0` is `false` (mc above horizon, no flip) while `>= 90.0` is `true` (flip) — the cusps differ, killing the mutant. If no representable exact-90 input exists, do **not** add a vacuous test; document it equivalent in Task 6 with the per-mutant margin argument (the minimum non-zero `|(lat − mc_dec) − 90|` over the reachable grid).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): FU-9 sunshine_houses under-horizon guards (68->0 or documented boundary)"
```

---

### Task 6: `nutation_for` documented equivalents + PR wrap-up

`nutation_for(instant) -> (delta_psi_deg, delta_eps_deg)` (mod.rs:592–601). Both survivors are at `600:30` — the `delta_psi_arcsec / 3600.0` term (`/`→`*`, `/`→`%`). Both call sites discard `delta_psi`: `asc_mc` at `mod.rs:268` binds `let (_dpsi, deps) = ...`, `validated_obliquity` at `mod.rs:610` binds `let (_delta_psi_deg, delta_eps_deg) = ...`. No public path observes the first tuple element, so any mutation of its arithmetic is a **documented equivalent** (unobservable output) — not killable without changing behavior (which this tests-only slice forbids). Document with a written reachability argument; do **not** `#[mutants::skip]`.

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append the census test)
- Modify: `docs/follow-ups.md` (append the FU-9 Progress entry)

**Interfaces:**
- Consumes: `nutation_for` reachability facts (call sites `mod.rs:268`, `mod.rs:610`); the observable half via `validated_obliquity`/`asc_mc` already covered by `validate-houses`/`validate-angles`.
- Produces: `sunshine_family_equivalent_mutants_are_documented` census test.

- [ ] **Step 1: Write the documented-equivalent census test**

Append to `crates/pleiades-houses/src/systems/tests.rs`:

```rust
/// Census of the Sunshine/solar-arc-family mutants left as *documented
/// equivalents* after this FU-9 slice. Each is unobservable through the public
/// API; killing it would require a production behavior change (out of scope for
/// this tests-only slice). Left visible (never `#[mutants::skip]`) so a future
/// reader sees the reachability argument.
///
/// - `nutation_for` `mod.rs:600:30` `/`->`*` and `/`->`%` (2 mutants): mutate
///   `delta_psi_arcsec / 3600.0`, the FIRST tuple element. BOTH call sites
///   discard it — `asc_mc` (`mod.rs:268`, `let (_dpsi, deps) = ...`) and
///   `validated_obliquity` (`mod.rs:610`, `let (_delta_psi_deg, delta_eps_deg)
///   = ...`). Only `delta_eps` feeds obliquity (observed, and caught by
///   `validate-houses`/`validate-angles`). `delta_psi` reaches no public
///   output, so its arithmetic is unobservable => equivalent.
#[test]
fn sunshine_family_equivalent_mutants_are_documented() {
    // Assert the observable half is genuinely exercised (so this census does not
    // silently rot if a future edit starts reading delta_psi): a None-obliquity
    // request drives validated_obliquity -> nutation_for -> delta_eps -> cusps.
    let request = HouseRequest::new(
        Instant::new(JulianDay::from_days(2_451_600.0), TimeScale::Tt),
        ObserverLocation::new(
            Latitude::from_degrees(48.0),
            Longitude::from_degrees(12.0),
            None,
        ),
        HouseSystem::Sunshine,
    );
    // obliquity defaulted (None) => mean_obliquity + delta_eps from nutation_for.
    assert!(request.obliquity.is_none());
    let snapshot = calculate_houses(&request).expect("sunshine houses should work");
    assert_eq!(snapshot.cusps.len(), 12);
}
```

*(If Task 5 Step 5 documented the `sunshine_houses` `1546:92 >=` boundary as equivalent, add a third bullet to this doc-comment with its per-mutant margin argument, and adjust the assertions/comment accordingly.)*

- [ ] **Step 2: Run the census test — confirm it PASSES on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses sunshine_family_equivalent_mutants_are_documented`
Expected: PASS.

- [ ] **Step 3: Final scoped mutants re-run over the whole family**

Run:
```bash
MISE_TRUSTED_CONFIG_PATHS=/tmp mise exec -- cargo mutants \
  --test-tool nextest --test-workspace=false --baseline run \
  -p pleiades-houses --file crates/pleiades-houses/src/systems/mod.rs \
  -F 'in (sunshine_houses|sunshine_offsets|apparent_solar_declination|apparent_midheaven_declination|nutation_for)$'
```
Expected: `137 mutants tested: 2 missed, 135 caught` (the 2 `nutation_for` `delta_psi` equivalents; **3 missed** if the `sunshine_houses` boundary was documented). Confirm the 2 (or 3) MISSED lines are exactly the documented equivalents.

- [ ] **Step 4: Run the blocking CI gate**

Run: `mise run ci`
Expected: green (fmt + clippy `-D warnings` + workspace test). Fix any `cargo fmt` drift in the new test literals before committing (`[[fu9-houses-plan-literals-not-rustfmt-clean]]`).

- [ ] **Step 5: Append the FU-9 Progress entry to `docs/follow-ups.md`**

Add, in the established format (per-mutant reachability for each documented equivalent, `[[fu9-margin-table-per-mutant-rows]]` — enumerate mutant × argument, never aggregate):

```markdown
**Progress (2026-07-24) — houses Sunshine/solar-arc
(`pleiades-houses/src/systems/mod.rs`):** triaged the Sunshine/solar-arc family
from `126` surviving mutants (`sunshine_houses` 68, `sunshine_offsets` 34,
`apparent_solar_declination` 20, `apparent_midheaven_declination` 2,
`nutation_for` 2) to **2 documented equivalents**. `apparent_solar_declination`
(20→0) and `sunshine_offsets` (34→0) pinned by independent recomputation from
the published NOAA/USNO low-precision Sun and the Albategnian semi-arc
trisection (`houses-reference.py` extended with `solar_declination` and
`sunshine_offsets`, cross-validated to ~1e-9). `apparent_midheaven_declination`
(2→0) pinned by a crafted product-form geometry. `sunshine_houses` (68→0) pinned
by `recompose_sunshine` — an independent recomposition threading un-mutated
`local_sidereal_time`/`asc1`/`longitude_opposite` and the just-pinned
`apparent_solar_declination`/`sunshine_offsets` — over a geometry table
straddling the `acmc < 0` axis flip and the `mc_under_horizon` under-horizon
flip (both hemispheres + a `lat == 0` row). Documented equivalents (both
`nutation_for` `mod.rs:600:30`, `/`→`*` and `/`→`%`): the `delta_psi_arcsec /
3600.0` term is the first tuple element, discarded at both call sites (`asc_mc`
`mod.rs:268`, `validated_obliquity` `mod.rs:610`) — unobservable output, so
mutating its arithmetic cannot change any public result. Left visible (no
`#[mutants::skip]`), enumerated in
`sunshine_family_equivalent_mutants_are_documented`. No production code changed;
no parity gate touched; mutants tier stays report-only. Scoped re-run:
`137 tested / 2 missed / 135 caught`.
```

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs docs/follow-ups.md
git commit -m "test(houses): FU-9 houses Sunshine mutant triage (126 -> 2 documented equivalents)"
```

---

## Self-Review

**1. Spec coverage** (design `2026-07-22-fu9-houses-mutant-triage-design.md`, PR 4 row):
- `sunshine_houses` (68) → Tasks 4–5. `sunshine_offsets` (34) → Task 3. `apparent_solar_declination` (20) → Task 2. `apparent_midheaven_declination` (2) → Task 1. `nutation_for` (2) → Task 6. ✅ All five PR-4 functions covered.
- Method (authoritative per-file baseline → classify → independent-reference tests → re-run): each task follows it. ✅
- Reference strategy: pure helpers = independent recomputation (Python port); `sunshine_houses` = independent recomposition threaded through un-mutated helpers; guards = reachability/crafted-boundary. ✅ Matches the design's per-class mapping.
- No parity-gate change; no `#[mutants::skip]`; mutants stays report-only; `[tasks.mutants]` untouched (PR 6). ✅
- FU-9 Progress note appended per the running format with per-mutant reachability. ✅

**2. Placeholder scan:** every step carries real test code, real commands, and real expected output. The only conditional is Task 5 Step 5 (exact-boundary kill), explicitly gated on the Step-4 measurement — not a placeholder. ✅

**3. Type consistency:** `recompose_sunshine(Instant, &ObserverLocation, Angle, HouseAngles) -> [f64; 12]` and `sun_geom(f64,f64,f64,f64,f64,f64) -> (Instant, ObserverLocation, Angle, HouseAngles)` (Task 4) are consumed with matching signatures in Task 5. `apparent_solar_declination(Instant, Angle) -> Angle`, `apparent_midheaven_declination(f64, f64) -> f64`, `sunshine_offsets(f64, f64) -> [f64; 13]`, `sunshine_houses(Instant, &ObserverLocation, Angle, HouseAngles) -> [Longitude; 12]`, `asc1(f64, f64, f64, f64) -> Longitude`, `local_sidereal_time(Instant, Longitude) -> Angle`, `signed_longitude_difference(f64, f64) -> f64`, `longitude_opposite(Longitude) -> Longitude`, `gc_angles(f64, f64) -> HouseAngles` match the crate as read at `188c5f1c4`. `assert_close_degrees` (tolerance `1e-12`) is the existing helper; recomposition rows use an inline `1e-9` tolerance (libm drift across two transcriptions). ✅

## References

- `docs/superpowers/specs/2026-07-22-fu9-houses-mutant-triage-design.md` — campaign design (PR 4 row).
- `docs/superpowers/plans/2026-07-23-fu9-houses-greatcircle-mutant-triage.md` — recomposition-threading precedent (`recompose_horizon`/`recompose_krusinski`, `gc_angles`/`gc_instant`).
- `docs/superpowers/plans/2026-07-24-fu9-houses-sector-mutant-triage.md` — reference-extension + documented-equivalent census precedent.
- `docs/superpowers/specs/notes/2026-07-22-houses-reference.py` — independent house-math reference (extended here with `solar_declination`, `sunshine_offsets`).
- `docs/follow-ups.md` — FU-9 running documented-equivalent tally + reusable method.
- Memory: `[[fu9-margin-table-per-mutant-rows]]`, `[[fu9-jd-grid-representability]]`, `[[fu9-guard-equivalence-overflow-lens]]`, `[[fu9-houses-reference-independence]]`, `[[fu9-houses-plan-literals-not-rustfmt-clean]]`.
