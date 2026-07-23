# FU-9 Houses Great-circle Mutant Triage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive the **Great-circle** family of `crates/pleiades-houses/src/systems/mod.rs` (`apc_sector`, `apc_houses`, `horizon_houses`, `krusinski_pisa_goelzer_houses`) from a **measured 90 surviving mutants to 8 documented equivalents**, reusing the independent house-math reference established by the Foundation PR.

**Architecture:** Second PR of the ~6-PR `pleiades-houses` FU-9 campaign (spec: `docs/superpowers/specs/2026-07-22-fu9-houses-mutant-triage-design.md`; Foundation PR: `docs/superpowers/plans/2026-07-22-fu9-houses-foundation-mutant-triage.md`). **Tests-only** — no production-code change; every added test lives in `crates/pleiades-houses/src/systems/tests.rs` (`use super::*;` reaches the private functions). The reference note `docs/superpowers/specs/notes/2026-07-22-houses-reference.py` is **extended** with an independent `apc_sector` port.

**Two reference strategies, keyed to survivor structure (this is the key design decision):**
- `apc_sector` is a **pure function** (`fn apc_sector(n, latitude_rad, obliquity_rad, sidereal_rad) -> Longitude`). Its 58 survivors are arithmetic-operator swaps in one trig expression. Reference = **independent Python port** (extends `houses-reference.py`), pinning all 12 sector outputs at one non-degenerate geometry.
- `apc_houses`, `horizon_houses`, `krusinski_pisa_goelzer_houses` take `Instant`/`ObserverLocation` and call `local_sidereal_time` internally (which resolves to `pleiades_apparent::sidereal::sidereal_time(...).local_apparent_deg` — GMST + nutation, *not* reproduced here). Their survivors are **structural** (which sector index, hemisphere-sign, rotation-angle sign, offset arithmetic) — the inner trig is already killed by the existing SE-anchored gate/unit tests. Reference = **independent recomposition** (the apparent.rs precedent): the test threads `st` from the un-mutated `local_sidereal_time` and recomposes the expected cusps from the published SE formula plus already-pinned shared primitives (`ascendant_for`, `longitude_opposite`, `spherical_cotrans`, `ecliptic_longitude_from_ra`, `signed_longitude_difference`, `normalize_degrees`), then asserts equality against the crate. Non-circular: `local_sidereal_time` and the shared primitives are not mutated in this slice, and `apc_sector`'s own arithmetic is pinned independently in Task 1.

**Mutation-triage TDD cycle** (differs from feature TDD — read once): a triage test *passes* on correct `HEAD` code (it pins intent against the independent reference/recomposition); it *kills a mutant* by failing on the mutated tree. Each task: write the test → run `cargo nextest` to confirm it **passes** on `HEAD` (proves the literal/recomposition is right) → the final task re-runs `cargo mutants -F` to confirm the survivors are **caught** down to the documented equivalents.

**Tech Stack:** Rust (stable, via mise), cargo-nextest, cargo-mutants 27.1.0, Python 3 (reference only).

## Global Constraints

- **Tests-only:** no production-code change anywhere in this PR; the only source file modified is `crates/pleiades-houses/src/systems/tests.rs` (plus the reference-note extension and the follow-up entry). No behavior-preserving refactor is needed (unlike Foundation's `apparent.rs`/`aberration.rs`) — every survivor is reachable through the existing private-function surface.
- **No parity-gate change:** no `validate-*` file is touched; the mutants tier stays report-only.
- **No `#[mutants::skip]`:** the 8 residual documented-equivalent mutants are left visible with a written reachability argument each, enumerated in a characterization test (`horizon_pole_singularity_equivalent_mutants_are_documented`).
- **Independence discipline:** every expected value comes from the independent Python port (`apc_sector`), from independent recomposition threaded through un-mutated helpers, or from hand arithmetic — never from running the mutated function and pinning its output. Each recomposition is confirmed to reproduce the crate at `HEAD` to 1e-9 before it is trusted.
- **Branch:** create `fu9-houses-greatcircle-mutant-triage` off `main` (do not work on `main`).
- All commands run through mise: prefix cargo invocations with `mise exec --`.
- **Obliquity constant used by every geometry:** `EPS = 23.4366` degrees (a representative true obliquity of date), matching the Foundation reference. `apc_sector` takes radians directly.
- **cargo-mutants + mise trust gotcha (operational):** cargo-mutants copies the workspace to `/tmp/cargo-mutants-workspace-*.tmp`, and this environment's mise (2026.7.12) refuses the untrusted copied `mise.toml`, failing the baseline build with `Config files ... are not trusted`. Prefix every `cargo mutants` invocation with `MISE_TRUSTED_CONFIG_PATHS=/tmp` (covers the temp workspace). Without it the run aborts at "build failed in an unmutated tree".

## Measured baseline (2026-07-23, post-Foundation, at `af8a6fd2c`)

Authoritative per-file command, scoped to the four Great-circle functions (cargo-mutants 27.1.0):

```bash
MISE_TRUSTED_CONFIG_PATHS=/tmp mise exec -- cargo mutants \
  --test-tool nextest --test-workspace=false --baseline run \
  -p pleiades-houses \
  --file crates/pleiades-houses/src/systems/mod.rs \
  -F 'in (apc_sector|apc_houses|krusinski_pisa_goelzer_houses|horizon_houses)$'
```

**131 mutants tested — 90 missed / 41 caught / 0 unviable.** Survivors by function: `apc_sector` **58**, `krusinski_pisa_goelzer_houses` **19**, `horizon_houses` **12**, `apc_houses` **1**. This matches the design spec's whole-crate prediction (58/19/12/1) exactly.

**Target after this PR (measured during plan authoring, verified twice): `90 → 8 documented equivalents`,** all in `horizon_houses`'s pole-singularity handling. `apc_sector` 58→0, `apc_houses` 1→0, `krusinski` 19→0, `horizon_houses` 12→4-killed-8-documented. Scoped re-run reports `8 missed / 123 caught`.

---

### Task 1: `apc_sector` — 58 survivors → 0

`apc_sector(n, latitude_rad, obliquity_rad, sidereal_rad) -> Longitude` computes one APC in-between-sector longitude from a single trig expression (`kv = atan2(...)`, `a = kv + sidereal + π/2 + k·(π/2 ∓ kv)/3`, `atan2(y, x)`). All 58 survivors are `*`/`+`/`-`/`/` swaps in that expression, plus the `n < 8` below/above-horizon split. **One 12-value pin** at a single non-degenerate geometry (`lat=52°, obl=23.4366°, sidereal=45°`) kills **all 58** — verified during plan authoring: scoped `cargo mutants -F 'in (apc_sector)$'` reported `59 mutants, 59 caught, 0 missed`. The `n < 8` split is covered because pinning all 12 outputs makes `n=8` (the boundary sector) observable.

**Files:**
- Modify: `docs/superpowers/specs/notes/2026-07-22-houses-reference.py` (append `apc_sector` port + prints)
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `apc_sector(n: usize, latitude_rad: f64, obliquity_rad: f64, sidereal_rad: f64) -> Longitude` (private, reached via `use super::*;`).
- Produces: the `apc_sector` reference (used by Task 2's recomposition sanity).

- [ ] **Step 1: Extend the independent reference with an `apc_sector` port**

Append to `docs/superpowers/specs/notes/2026-07-22-houses-reference.py`, before the `if __name__ == "__main__":` block, an independent port derived from the published APC in-between-sector algorithm (NOT copied operator-by-operator from the Rust):

```python
def apc_sector(n, lat_rad, obl_rad, sid_rad):
    """crate `apc_sector`: APC in-between-sector longitude (SE 'B'/APC family).
    Independent port from the published APC algorithm; NOT copied from the Rust."""
    tan_lat = math.tan(lat_rad)
    tan_obl = math.tan(obl_rad)
    kv = math.atan2(tan_lat * tan_obl * math.cos(sid_rad),
                    1.0 + tan_lat * tan_obl * math.sin(sid_rad))
    sin_kv = math.sin(kv)
    below = n < 8
    k = float(n - 1) if below else float(n - 13)
    if below:
        a = kv + sid_rad + math.pi / 2 + k * (math.pi / 2 - kv) / 3.0
    else:
        a = kv + sid_rad + math.pi / 2 + k * (math.pi / 2 + kv) / 3.0
    y = sin_kv * math.sin(sid_rad) + math.sin(a)
    x = math.cos(obl_rad) * (sin_kv * math.cos(sid_rad) + math.cos(a)) \
        + math.sin(obl_rad) * tan_lat * math.sin(sid_rad - a)
    return norm360(math.degrees(math.atan2(y, x)))
```

And append to the `__main__` block:

```python
    print("# apc_sector — lat=52, obl=EPS, sidereal=45 (non-degenerate):")
    _lat, _obl, _sid = math.radians(52.0), math.radians(EPS), math.radians(45.0)
    for _n in range(1, 13):
        print(f"    apc_sector({_n:2d}) = {fmt(apc_sector(_n, _lat, _obl, _sid))}")
```

Run `python3 docs/superpowers/specs/notes/2026-07-22-houses-reference.py` and confirm it prints the 12 literals used in Step 2 (they were generated by exactly this port and cross-validated to 1e-9 against the crate during plan authoring). Note: the reference uses `EPS = 23.4366` and `fmt`/`norm360` already defined in the file.

- [ ] **Step 2: Write the 12-value pin**

Append to `crates/pleiades-houses/src/systems/tests.rs`:

```rust
// ===== FU-9 Great-circle PR: apc_sector / apc_houses / horizon / krusinski =====

#[test]
fn apc_sector_pins_all_twelve_against_independent_reference() {
    // Independent reference (houses-reference.py `apc_sector`, published APC
    // algorithm) at lat=52°, obl=23.4366°, sidereal=45° — non-degenerate, so
    // every `*`/`+`/`-`/`/` swap and the `n < 8` split is observable. Pinning
    // all 12 sectors kills all 58 arith survivors (measured 59/59 caught).
    let lat = 52.0_f64.to_radians();
    let obl = 23.4366_f64.to_radians();
    let sid = 45.0_f64.to_radians();
    let expected = [
        148.587_249_395_771, 166.495_240_772_036, 189.747_228_099_578,
        227.463_595_280_938, 275.481_343_990_138, 308.273_382_675_614,
        328.587_249_395_771, 350.866_340_446_180, 14.729_289_455_955,
        47.463_595_280_938, 88.169_411_590_291, 122.556_859_248_324,
    ];
    for (i, e) in expected.iter().enumerate() {
        let got = apc_sector(i + 1, lat, obl, sid).degrees();
        assert!((got - e).abs() < 1e-9, "apc_sector({}) = {got}, want {e}", i + 1);
    }
}
```

- [ ] **Step 3: Run the test — confirm it passes on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses apc_sector_pins_all_twelve`
Expected: `1 passed`. (A failure means the literal or import is wrong — fix before proceeding.)

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs docs/superpowers/specs/notes/2026-07-22-houses-reference.py
git commit -m "test(houses): FU-9 pin apc_sector against independent APC port (58 mutants)"
```

---

### Task 2: shared recomposition helpers + `apc_houses` — 1 survivor → 0

`apc_houses` fills `cusps[i] = apc_sector(index + 1, ...)` then overwrites `cusps[0] = asc` and `cusps[9] = mc`. The single survivor is `index + 1 → index * 1` (`+`→`*`) at line 1182: under it, `cusp[i] = apc_sector(i)` instead of `apc_sector(i+1)`. Killed by a **recomposition-equality** test that asserts each non-overwritten cusp equals `apc_sector(i+1, ...)` computed independently with `st` threaded from the un-mutated `local_sidereal_time`. This task also introduces the shared `gc_instant`/`gc_angles` helpers used by Tasks 3–4.

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `apc_houses(instant: Instant, observer: &ObserverLocation, obliquity: Angle, angles: HouseAngles) -> [Longitude; 12]`; `local_sidereal_time(instant: Instant, longitude: Longitude) -> Angle`; `HouseAngles { ascendant, descendant, midheaven, imum_coeli }` (all `Longitude`); `apc_sector` (Task 1). All private, reached via `use super::*;`.
- Produces: `gc_instant() -> Instant`, `gc_angles(asc: f64, mc: f64) -> HouseAngles` (consumed by Tasks 3–4).

- [ ] **Step 1: Write the shared helpers + the apc_houses recomposition**

Append:

```rust
fn gc_instant() -> Instant {
    Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt)
}

fn gc_angles(asc: f64, mc: f64) -> HouseAngles {
    HouseAngles {
        ascendant: Longitude::from_degrees(asc),
        descendant: Longitude::from_degrees(asc + 180.0),
        midheaven: Longitude::from_degrees(mc),
        imum_coeli: Longitude::from_degrees(mc + 180.0),
    }
}

#[test]
fn apc_houses_calls_apc_sector_with_one_indexed_sectors() {
    // Recomposition (independent of apc_houses's own indexing arithmetic): each
    // non-angle cusp i must equal the pure apc_sector for sector n = i+1. The
    // `index + 1 -> index * 1` mutant makes cusp[i] = apc_sector(i), differing
    // from apc_sector(i+1) at every reachable geometry. `st` is threaded from
    // the un-mutated local_sidereal_time (not mutated in this slice), so the
    // check is non-circular; apc_sector's own arithmetic is pinned in Task 1.
    let instant = gc_instant();
    let obs = ObserverLocation::new(
        Latitude::from_degrees(52.0), Longitude::from_degrees(0.0), None);
    let obl = Angle::from_degrees(23.4366);
    let angles = gc_angles(10.0, 280.0);
    let cusps = apc_houses(instant, &obs, obl, angles);
    let st_rad = local_sidereal_time(instant, obs.longitude).degrees().to_radians();
    let (lat_rad, obl_rad) = (52.0_f64.to_radians(), 23.4366_f64.to_radians());
    for i in [1usize, 2, 3, 4, 5, 6, 7, 8, 10, 11] {
        let want = apc_sector(i + 1, lat_rad, obl_rad, st_rad).degrees();
        assert!((cusps[i].degrees() - want).abs() < 1e-9,
            "apc cusp[{i}] = {}, want apc_sector({}) = {want}", cusps[i].degrees(), i + 1);
    }
    assert_eq!(cusps[0], angles.ascendant);
    assert_eq!(cusps[9], angles.midheaven);
}
```

- [ ] **Step 2: Run the test — confirm it passes on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses apc_houses_calls_apc_sector`
Expected: `1 passed`.

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): FU-9 recomposition pin apc_houses sector indexing (1 mutant)"
```

---

### Task 3: `horizon_houses` — 12 survivors → 4 killed, 8 documented equivalents

`horizon_houses` (SE 'H' azimuth convention) survivors split into **killable structural/pole mutants** and a **documented-equivalent tail** in the pole-singularity clamp. Killable (4): the hemisphere-sign `-90 - lat` (1089, killed by a southern observer), the `/90` clamp-guard variant (1094:36 `/`, killed at the pole `lat=90`), and the two N-side clamp-target mutants (1098:18 `+`/`/`, killed by a near-equator northern observer `lat=1e-11` where `cosfi` flips sign). All four fall to **independent recomposition** across five geometries. The remaining **8 are documented equivalents** (Step 3). This split was measured twice during plan authoring: recomposition at `[-33, 0, 40]` left 12 survivors; adding `[1e-11, 90]` killed 4 → **8 residual**.

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `horizon_houses(instant: Instant, observer: &ObserverLocation, obliquity: Angle, angles: HouseAngles) -> [Longitude; 12]`; shared primitives `ascendant_for(sidereal_time_deg: f64, latitude_deg: f64, obliquity_rad: f64) -> Longitude`, `longitude_opposite(Longitude) -> Longitude`; `local_sidereal_time`; `gc_instant`/`gc_angles` (Task 2).
- Produces: nothing consumed by later tasks.

- [ ] **Step 1: Write the independent recomposition + five-geometry pin**

Append:

```rust
// Independent recomposition of the SE 'H' horizon body from the published
// formula (azimuth origin Asc1(th+90) via ascendant_for(th,·), +180 post-fold
// via longitude_opposite, strict `>0` hemisphere branch, VERY_SMALL pole clamp).
// Calls only shared primitives already pinned in Foundation; `st` is threaded
// from the un-mutated local_sidereal_time. Confirmed to equal the crate at HEAD
// to 1e-9 during plan authoring, so any structural mutant that diverges dies.
fn recompose_horizon(
    instant: Instant, obs: &ObserverLocation, obl_deg: f64, mc: Longitude,
) -> [Longitude; 12] {
    let st = (local_sidereal_time(instant, obs.longitude).degrees() + 180.0).rem_euclid(360.0);
    let obl_rad = obl_deg.to_radians();
    let lat = obs.latitude.degrees();
    let mut tl = if lat > 0.0 { 90.0 - lat } else { -90.0 - lat };
    const VS: f64 = 1e-10;
    if (tl.abs() - 90.0).abs() < VS {
        tl = if tl < 0.0 { -90.0 + VS } else { 90.0 - VS };
    }
    let tlr = tl.to_radians();
    let fh1 = (tlr.sin() / 2.0).asin().to_degrees();
    let fh2 = ((3.0_f64).sqrt() / 2.0 * tlr.sin()).asin().to_degrees();
    let cosfi = tlr.cos();
    let (xh1, xh2) = if cosfi == 0.0 {
        if tl > 0.0 { (90.0, 90.0) } else { (270.0, 270.0) }
    } else {
        (
            (3.0_f64.sqrt() / cosfi).atan().to_degrees(),
            (1.0 / 3.0_f64.sqrt() / cosfi).atan().to_degrees(),
        )
    };
    let mut c = [Longitude::from_degrees(0.0); 12];
    c[0] = longitude_opposite(ascendant_for(st, tl, obl_rad));
    c[9] = mc;
    c[10] = longitude_opposite(ascendant_for(st - xh1, fh1, obl_rad));
    c[11] = longitude_opposite(ascendant_for(st - xh2, fh2, obl_rad));
    c[1] = longitude_opposite(ascendant_for(st + xh2, fh2, obl_rad));
    c[2] = longitude_opposite(ascendant_for(st + xh1, fh1, obl_rad));
    c[3] = longitude_opposite(c[9]);
    c[4] = longitude_opposite(c[10]);
    c[5] = longitude_opposite(c[11]);
    c[6] = longitude_opposite(c[0]);
    c[7] = longitude_opposite(c[1]);
    c[8] = longitude_opposite(c[2]);
    c
}

#[test]
fn horizon_houses_match_independent_recomposition_across_geometries() {
    let instant = gc_instant();
    let obl = Angle::from_degrees(23.4366);
    // lat=-33 kills the `-90 - lat` southern-hemisphere sign mutant (1089);
    // lat=1e-11 (N, near equator) reaches the +90-VS clamp target and flips
    //   cosfi's sign under the 1098 mutants (killing both);
    // lat=90 (pole) makes the `/90` clamp-guard mutant (1094:36 `/`) clamp
    //   where HEAD does not (cosfi 1 -> 1.7e-12), killing it.
    for lat in [-33.0_f64, 0.0, 40.0, 1e-11, 90.0] {
        let obs = ObserverLocation::new(
            Latitude::from_degrees(lat), Longitude::from_degrees(0.0), None);
        let angles = gc_angles(10.0, 280.0);
        let got = horizon_houses(instant, &obs, obl, angles);
        let want = recompose_horizon(instant, &obs, 23.4366, angles.midheaven);
        for i in 0..12 {
            let mut d = (got[i].degrees() - want[i].degrees()).rem_euclid(360.0);
            if d > 180.0 { d -= 360.0; }
            assert!(d.abs() < 1e-9, "horizon lat={lat} cusp[{i}] diff {d}");
        }
    }
}
```

- [ ] **Step 2: Run the test — confirm it passes on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses horizon_houses_match_independent_recomposition`
Expected: `1 passed`.

- [ ] **Step 3: Write the documented-equivalent characterization test**

The 8 residual survivors are all in the `horizon_houses` pole-singularity clamp (lines 1082, 1094, 1095, 1108). Each is left visible (no `#[mutants::skip]`) with a reachability argument, grouped by structural reason. Append:

```rust
#[test]
fn horizon_pole_singularity_equivalent_mutants_are_documented() {
    // FU-9 Great-circle residual: 8 surviving mutants in horizon_houses, all in
    // the pole-singularity handling, each an EQUIVALENT MUTANT left visible (no
    // #[mutants::skip]). Magnitudes below are MEASURED, not claimed exact.
    //
    // (A) Sub-tolerance mod-360 antipode — 1082:69 `+ 180.0 -> - 180.0`:
    //   sidereal_time = (LST ± 180).rem_euclid(360). (x+180)%360 and (x-180)%360
    //   are equal in real arithmetic (differ by 360) and differ in f64 by at most
    //   ~5.68e-14° over a fine LST sweep — far below the 1e-9° pin tolerance, so
    //   no white-box pin can distinguish them. (Same shape as the Foundation
    //   `armc ± 180` finding.)
    //
    // (B) Clamp-guard mutants only reachable where the clamp is sub-tolerance:
    //   1094:36 `(|tl|-90) -> (|tl|+90)`  — `|tl|+90 >= 90` so the guard never
    //     fires (never clamps); differs from HEAD only at lat≈0 (tl≈±90), where
    //     the clamp alters tl by ≤1e-10° and atan(√3/cosφ) has already saturated
    //     to ±90°, so the cusp displacement is <1e-9°. (Its `/90` sibling was
    //     killable at lat=90 and IS caught — see the recomposition test.)
    //   1094:50 `< -> ==` and `< -> <=` — the guard fires only at exactly
    //     value == 1e-10 (measure-zero); the reachable difference (lat≈0) is the
    //     same sub-tolerance clamp effect as above.
    //   1095:56 `tl < 0.0 -> tl <= 0.0` — this branch is entered only when the
    //     clamp fires, i.e. |tl|≈90, so tl is ±90 and never 0; the operators
    //     differ only at tl == 0.0, unreachable here.
    //
    // (C) Structurally dead branch — 1108:33 `> -> ==`, `-> <`, `-> >=`:
    //   these steer the `if cosfi == 0.0 { ... }` arm. cos(tl_rad) is never
    //   exactly 0.0 for any reachable tl (min |cos| over the clamp-guaranteed
    //   range incl. raw ±90 is 6.123e-17), so the whole arm is unreachable and
    //   all three mutants are equivalent regardless of geometry.
    //
    // This test asserts the two facts the arguments rest on, so the reasoning is
    // guarded against a future code change that would make a survivor killable.
    // Fact (A): the antipode difference stays sub-tolerance.
    let mut worst = 0.0_f64;
    let mut lst = 0.0_f64;
    while lst < 360.0 {
        let a = (lst + 180.0).rem_euclid(360.0);
        let b = (lst - 180.0).rem_euclid(360.0);
        let mut d = (a - b).rem_euclid(360.0);
        if d > 180.0 { d -= 360.0; }
        worst = worst.max(d.abs());
        lst += 0.0007;
    }
    assert!(worst < 1e-9, "antipode (x±180)%360 diff {worst} must stay sub-tolerance");
    // Fact (C): cos(tl_rad) never hits exactly 0 across the reachable tl range.
    for tl in [90.0 - 1e-10, -90.0 + 1e-10, 90.0, -90.0, 0.0, 45.0, -57.0] {
        assert!(tl.to_radians().cos() != 0.0, "cosfi==0.0 arm is dead (tl={tl})");
    }
}
```

- [ ] **Step 4: Run the characterization test — confirm it passes on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses horizon_pole_singularity_equivalent`
Expected: `1 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): FU-9 horizon recomposition (12 -> 4 killed, 8 documented equivalents)"
```

---

### Task 4: `krusinski_pisa_goelzer_houses` — 19 survivors → 0

`krusinski_pisa_goelzer_houses` projects house-circle points through two `spherical_cotrans` rotations. Survivors are the flip guard (1200 `signed_longitude_difference(asc, mc) < 0`), the two `delete -` rotation-angle signs (1205, 1207), and the offset arithmetic (1206, 1212, 1214, 1215, 1216). All 19 fall to **independent recomposition** across four geometries: two with the flip inactive/active (asc=200/mc=100 vs asc=10/mc=100), a southern observer, and — decisively — **`asc == mc`** (signed_diff exactly 0), which kills the `< -> <=` flip-boundary mutant that no non-degenerate geometry can reach. Measured 19→0 during plan authoring.

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `krusinski_pisa_goelzer_houses(instant: Instant, observer: &ObserverLocation, obliquity: Angle, angles: HouseAngles) -> [Longitude; 12]`; shared primitives `signed_longitude_difference(a: f64, b: f64) -> f64`, `longitude_opposite`, `spherical_cotrans(coord: &mut [f64; 3], angle_deg: f64)`, `normalize_degrees(f64) -> f64`, `ecliptic_longitude_from_ra(ra_deg: f64, obliquity: f64) -> Longitude`; `local_sidereal_time`; `gc_instant`/`gc_angles`.
- Produces: nothing consumed by later tasks.

- [ ] **Step 1: Write the independent recomposition + four-geometry pin**

Append:

```rust
// Independent recomposition of the Krusinski-Pisa-Goelzer body from the
// published great-circle projection: flip the ascendant when it trails the MC,
// rotate the house-circle anchor into the horizon frame, then step 30° arcs and
// project back to ecliptic longitude. Calls only Foundation-pinned primitives;
// `st` threaded from the un-mutated local_sidereal_time. Confirmed == crate at
// HEAD to 1e-9 during plan authoring.
fn recompose_krusinski(
    instant: Instant, obs: &ObserverLocation, obl_deg: f64, angles: HouseAngles,
) -> [Longitude; 12] {
    let st = local_sidereal_time(instant, obs.longitude).degrees();
    let lat = obs.latitude.degrees();
    let mut ascendant = angles.ascendant;
    if signed_longitude_difference(ascendant.degrees(), angles.midheaven.degrees()) < 0.0 {
        ascendant = longitude_opposite(ascendant);
    }
    let mut hcp = [ascendant.degrees(), 0.0, 1.0];
    spherical_cotrans(&mut hcp, -obl_deg);
    hcp[0] = normalize_degrees(hcp[0] - (st - 90.0));
    spherical_cotrans(&mut hcp, -(90.0 - lat));
    let horizon_offset = hcp[0];
    let mut c = [Longitude::from_degrees(0.0); 12];
    for index in 0..6 {
        let mut p = [30.0 * index as f64, 0.0, 1.0];
        spherical_cotrans(&mut p, 90.0);
        p[0] = normalize_degrees(p[0] + horizon_offset);
        spherical_cotrans(&mut p, 90.0 - lat);
        p[0] = normalize_degrees(p[0] + (st - 90.0));
        c[index] = ecliptic_longitude_from_ra(p[0], obl_deg.to_radians());
        c[index + 6] = longitude_opposite(c[index]);
    }
    c[0] = ascendant;
    c[6] = longitude_opposite(ascendant);
    c
}

#[test]
fn krusinski_houses_match_independent_recomposition_across_geometries() {
    let instant = gc_instant();
    let obl = Angle::from_degrees(23.4366);
    // (asc=10, mc=100): signed_diff = -90 < 0 -> flip fires.
    // (asc=200, mc=100): signed_diff = +100 > 0 -> flip does not fire.
    // (-33, ...): southern observer.
    // (asc=100, mc=100): signed_diff == 0 -> kills the `< -> <=` boundary mutant.
    for (lat, asc, mc) in [
        (52.0_f64, 10.0, 100.0),
        (52.0, 200.0, 100.0),
        (-33.0, 10.0, 100.0),
        (52.0, 100.0, 100.0),
    ] {
        let obs = ObserverLocation::new(
            Latitude::from_degrees(lat), Longitude::from_degrees(0.0), None);
        let angles = gc_angles(asc, mc);
        let got = krusinski_pisa_goelzer_houses(instant, &obs, obl, angles);
        let want = recompose_krusinski(instant, &obs, 23.4366, angles);
        for i in 0..12 {
            let mut d = (got[i].degrees() - want[i].degrees()).rem_euclid(360.0);
            if d > 180.0 { d -= 360.0; }
            assert!(d.abs() < 1e-9, "krusinski lat={lat} asc={asc} cusp[{i}] diff {d}");
        }
    }
}
```

- [ ] **Step 2: Run the test — confirm it passes on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses krusinski_houses_match_independent_recomposition`
Expected: `1 passed`.

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): FU-9 krusinski recomposition incl asc==mc flip boundary (19 mutants)"
```

---

### Task 5: Verify Great-circle reaches 8 documented equivalents + follow-up note

Re-run cargo-mutants scoped to the four Great-circle functions and confirm exactly **8 missed** (the horizon pole-singularity equivalents). Then record the slice in `docs/follow-ups.md`.

**Files:**
- Modify: `docs/follow-ups.md` (append an FU-9 Progress note)

- [ ] **Step 1: Confirm the whole crate test suite is green**

Run: `mise exec -- cargo nextest run -p pleiades-houses`
Expected: all pass (91 existing + 5 new = 96).

- [ ] **Step 2: Run scoped mutation verification (~131 mutants, ~5 min)**

Run:
```bash
MISE_TRUSTED_CONFIG_PATHS=/tmp mise exec -- cargo mutants \
  --test-tool nextest --test-workspace=false --baseline run \
  -p pleiades-houses \
  --file crates/pleiades-houses/src/systems/mod.rs \
  -F 'in (apc_sector|apc_houses|krusinski_pisa_goelzer_houses|horizon_houses)$'
```
Expected (measured during plan authoring): **`8 missed / 123 caught / 0 unviable`** out of 131 mutants. Confirm `cat mutants.out/missed.txt` is exactly these 8, all in `horizon_houses`:
```
1082:69  replace + with -    (mod-360 antipode, sub-tolerance ~5.68e-14°)
1094:36  replace - with +     (never-clamp guard, sub-tolerance at lat≈0)
1094:50  replace < with ==     (measure-zero clamp-fire boundary)
1094:50  replace < with <=     (measure-zero clamp-fire boundary)
1095:56  replace < with <=     (unreachable tl==0 inside clamp branch)
1108:33  replace > with ==     (dead cosfi==0.0 arm)
1108:33  replace > with <      (dead cosfi==0.0 arm)
1108:33  replace > with >=     (dead cosfi==0.0 arm)
```
If any *other* mutant is still missed (not in this set), classify it (an arithmetic/comparison swap in one of the four functions), add a discriminating geometry or assertion to the matching recomposition test using the independent reference, and re-run — do not proceed until the residual is exactly these 8 documented equivalents.

- [ ] **Step 3: Append the FU-9 Progress note to `docs/follow-ups.md`**

Under the FU-9 section, after the houses Foundation progress note, add:

```markdown
**Progress (2026-07-23) — houses Great-circle
(`pleiades-houses/src/systems/mod.rs`, `apc_sector`/`apc_houses`/`horizon_houses`/
`krusinski_pisa_goelzer_houses`):** second PR of the post-baseline
`pleiades-houses` expansion campaign (spec:
`docs/superpowers/specs/2026-07-22-fu9-houses-mutant-triage-design.md`; plan:
`docs/superpowers/plans/2026-07-23-fu9-houses-greatcircle-mutant-triage.md`).
Triaged the Great-circle family from `90` surviving mutants (`apc_sector` 58,
`krusinski` 19, `horizon_houses` 12, `apc_houses` 1 — matching the design's
whole-crate prediction exactly) to **8 documented equivalents**, all in
`horizon_houses`'s pole-singularity clamp. **Tests-only.** Two reference
strategies keyed to survivor structure: `apc_sector` is a pure function, so its
58 arith survivors fell to a single **independent-port** pin of all 12 sector
outputs at one non-degenerate geometry (lat=52°, obl=23.4366°, sidereal=45° —
measured 59/59 caught); the other three take `Instant`/`ObserverLocation` and
call `local_sidereal_time` (GMST+nutation, not reproduced), so their
**structural** survivors (sector index, hemisphere sign, rotation-angle signs,
offset arithmetic — the inner trig was already gate-killed) fell to
**independent recomposition** (the `apparent.rs` precedent): the test threads
`st` from the un-mutated `local_sidereal_time` and recomposes expected cusps
from the published SE formula plus Foundation-pinned primitives (`ascendant_for`,
`longitude_opposite`, `spherical_cotrans`, `ecliptic_longitude_from_ra`,
`signed_longitude_difference`), then asserts equality with the crate. The
`apc_sector` port extends the shared `houses-reference.py`. Per the Foundation
lesson (probe extremes before documenting equivalence), four horizon survivors
first classified as pole-clamp equivalents were **killed** by extreme
geometries: the `-90 - lat` hemisphere sign by a southern observer, the `/90`
clamp-guard variant at the pole (`lat=90`, where it clamps and HEAD does not),
and both N-side clamp-target mutants by a near-equator northern observer
(`lat=1e-11`, where `cosfi` flips sign); krusinski's `< -> <=` flip-boundary
mutant was killed by an `asc == mc` (signed_diff == 0) geometry. **Documented
residual — 8 equivalent mutants**, all `horizon_houses`, left visible (no
`#[mutants::skip]`) and enumerated with per-mutant reachability arguments in
`horizon_pole_singularity_equivalent_mutants_are_documented`, grouped:
(A) `1082:69` `+180 -> -180` — `(LST±180).rem_euclid(360)` differ by at most
`~5.68e-14°` (measured over a fine sweep; the Foundation `armc±180` shape),
far below the 1e-9° tolerance; (B) `1094:36` `-` -> `+` (never-clamp),
`1094:50` `<` -> `==`/`<=` (measure-zero clamp-fire boundary), and `1095:56`
`<` -> `<=` (unreachable `tl==0` inside the clamp branch) — all four reachable
only where the clamp effect is itself sub-tolerance or at a measure-zero
equality; (C) `1108:33` `>` -> `==`/`<`/`>=` (×3) — the `if cosfi == 0.0` arm is
structurally dead (`cos(tl_rad)` is never exactly `0.0`; min `|cos|` over the
reachable range is `6.123e-17`). This brings the **running documented-equivalent
tally to `22 + 8 = 30`**. The `8` is **measured, not predicted**: the
authoritative scoped run (`-F 'in (apc_sector|apc_houses|
krusinski_pisa_goelzer_houses|horizon_houses)$'`, 131 mutants) reports
`8 missed / 123 caught / 0 unviable`. No parity gate was touched; the tier stays
report-only; `mise run ci` is green. **Remaining houses PRs:** sector
(`pullen_sr`/`pullen_sd`/`albategnius`/`gauquelin`), sunshine/solar-arc,
quadrant/projection, then catalog + thresholds (which adds `-p pleiades-houses`
to `[tasks.mutants]`).
```

- [ ] **Step 4: Run the blocking CI gate**

Run: `mise run ci`
Expected: green (fmt + clippy `-D warnings` + workspace test).

- [ ] **Step 5: Commit**

```bash
git add docs/follow-ups.md
git commit -m "docs(follow-ups): record FU-9 houses Great-circle triage (90 -> 8 documented equivalents)"
```

---

## Self-Review

- **Spec coverage:** This plan implements the spec's **PR 2 — Great-circle** row (`apc_sector` 58, `krusinski` 19, `horizon_houses` 12, `apc_houses` 1) and the method/reference-strategy/acceptance sections for those functions. It confirms exact survivor membership against a fresh measured baseline (`90 missed`, matching the spec) rather than reusing the whole-crate prediction, per the spec's "Exact survivor membership per PR is confirmed against the measured `mutants.out/missed.txt` at the start of each PR's plan." The remaining four campaign PRs are separate plans.
- **Placeholder scan:** none — every test body is complete and was validated to pass on HEAD during plan authoring; every literal was generated by the independent port and cross-validated to 1e-9; every kill count and the 8-equivalent residual were measured twice by scoped `cargo mutants`.
- **Type consistency:** signatures (`apc_sector(usize, f64, f64, f64) -> Longitude`, `apc_houses`/`horizon_houses`/`krusinski_pisa_goelzer_houses(Instant, &ObserverLocation, Angle, HouseAngles) -> [Longitude; 12]`, `local_sidereal_time(Instant, Longitude) -> Angle`, `ascendant_for(f64, f64, f64) -> Longitude`, `spherical_cotrans(&mut [f64;3], f64)`, `ecliptic_longitude_from_ra(f64, f64) -> Longitude`, `signed_longitude_difference(f64, f64) -> f64`, `normalize_degrees(f64) -> f64`, `HouseAngles { ascendant, descendant, midheaven, imum_coeli }`) match the crate as read at `af8a6fd2c`. The `gc_instant`/`gc_angles`/`recompose_horizon`/`recompose_krusinski` helper names are consistent across Tasks 2–4.
- **Independence:** every expected value derives from the independent Python port or from recomposition threaded through un-mutated helpers (`local_sidereal_time` and the Foundation-pinned primitives are not mutated in this slice) — non-circular. The equivalent-mutant arguments rest on two facts the characterization test asserts (sub-tolerance antipode, dead `cosfi==0.0` arm), so they cannot silently rot.
- **Measure-don't-predict:** the 8-equivalent residual and the four extreme-geometry kills were measured, not asserted; the Foundation over-documentation lesson is applied (extremes probed before any equivalence claim).
