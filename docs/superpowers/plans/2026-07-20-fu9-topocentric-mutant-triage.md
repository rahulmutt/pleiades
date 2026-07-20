# FU-9 `topocentric.rs` Mutant Triage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive `crates/pleiades-apparent/src/topocentric.rs` from 27 surviving mutants to exactly 4 documented equivalents, using tests pinned to an independent reference — tests-only, no production-code change.

**Architecture:** One exact-literal test at a crafted discriminating geometry (Palomar observer, Moon-scale distance) kills 17 survivors; two wrap-crossing tests kill the 6 killable Δlon-wrap mutants; a fail-closed test pins guard intent. Expected literals were computed by an independent Python reimplementation of the published Meeus ch. 11/40 pipeline (Appendix A) and cross-validated against the crate to ~1e-11″ during planning — they are final, not placeholders.

**Tech Stack:** Rust (stable, pinned via `mise.toml`), cargo-nextest, cargo-mutants 27.1.0.

**Spec:** `docs/superpowers/specs/2026-07-20-fu9-topocentric-mutant-triage-design.md`

## Global Constraints

- Tests-only: the ONLY edit to `topocentric.rs` is replacing the inline test module with `#[cfg(test)] mod tests;` (relocation). No numeric or structural production change.
- No `validate-*` gate file is touched; the mutants tier stays **report-only**.
- No `#[mutants::skip]` anywhere; the 4 equivalent mutants stay visible and documented.
- Independence discipline: every expected literal comes from the Appendix A script, never from the crate's own output.
- Assertion tolerances: `1e-9` (degrees), `1e-6` (arcsec fields), `1e-12` (AU distance).
- Branch: `fu9-topocentric-mutant-triage` (already exists, spec committed on it).
- Commit split note: the spec's Deliverables section lists one combined test commit; this plan splits relocation from new tests (Tasks 1 vs 2–3) per AGENTS.md's "separate refactors from behavioral changes". Intentional, not drift.

---

### Task 1: Relocate the inline test module

**Files:**
- Modify: `crates/pleiades-apparent/src/topocentric.rs` (delete lines 112–177, the inline `#[cfg(test)] mod tests { ... }`; append the two-line module declaration)
- Create: `crates/pleiades-apparent/src/topocentric/tests.rs`

**Interfaces:**
- Produces: `src/topocentric/tests.rs` with helpers `ecl(lon, lat, dist) -> EclipticCoordinates` and `observer(lat) -> ObserverLocation`, which Tasks 2–3 extend.

- [ ] **Step 1: Create the relocated test file**

Create `crates/pleiades-apparent/src/topocentric/tests.rs` containing the four existing tests, moved verbatim (only the module docs are new):

```rust
//! White-box unit tests for the topocentric module.
//!
//! Relocated out of `topocentric.rs` per AGENTS.md ("keep large inline test
//! suites out of the file under test"). These remain white-box unit tests with
//! access to the module's internals — they are deliberately not converted to
//! black-box integration tests.

use super::*;

fn ecl(lon: f64, lat: f64, dist: f64) -> EclipticCoordinates {
    EclipticCoordinates::new(
        Longitude::from_degrees(lon),
        Latitude::from_degrees(lat),
        Some(dist),
    )
}

fn observer(lat: f64) -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(lat),
        Longitude::from_degrees(0.0),
        Some(0.0),
    )
}

#[test]
fn moon_parallax_is_about_one_degree() {
    // Moon at ~0.00257 AU (60.3 Earth radii). For an observer with the Moon
    // near the horizon the parallax approaches ~0.95°. Assert it is large.
    let out =
        topocentric_position(ecl(100.0, 0.0, 0.002_57), &observer(0.0), 100.0, 23.4).unwrap();
    let shift = out
        .provenance
        .parallax_longitude_arcsec
        .hypot(out.provenance.parallax_latitude_arcsec)
        / 3600.0;
    assert!(shift > 0.3, "moon parallax {shift}° too small");
}

#[test]
fn distant_body_parallax_is_negligible() {
    // A body at 30 AU: parallax < 1".
    let out = topocentric_position(ecl(100.0, 0.0, 30.0), &observer(0.0), 100.0, 23.4).unwrap();
    let shift = out
        .provenance
        .parallax_longitude_arcsec
        .hypot(out.provenance.parallax_latitude_arcsec);
    assert!(shift < 1.0, "distant parallax {shift}\" too large");
}

#[test]
fn missing_distance_errors() {
    let no_dist = EclipticCoordinates::new(
        Longitude::from_degrees(100.0),
        Latitude::from_degrees(0.0),
        None,
    );
    let err = topocentric_position(no_dist, &observer(0.0), 100.0, 23.4).unwrap_err();
    assert_eq!(err, ApparentPlaceError::MissingDistance);
}

#[test]
fn diurnal_aberration_is_sub_arcsec() {
    let out = topocentric_position(ecl(100.0, 0.0, 1.0), &observer(0.0), 100.0, 23.4).unwrap();
    assert!(
        out.provenance.diurnal_aberration_arcsec < 0.36,
        "diurnal aberration {}\"",
        out.provenance.diurnal_aberration_arcsec
    );
}
```

- [ ] **Step 2: Replace the inline module in `topocentric.rs`**

Delete the entire `#[cfg(test)] mod tests { ... }` block (lines 112–177) and append in its place:

```rust
#[cfg(test)]
mod tests;
```

(Same pattern as `aberration.rs:66-67`, `refraction.rs:153-154`.)

- [ ] **Step 3: Verify the move is behavior-neutral**

Run: `cargo nextest run -p pleiades-apparent topocentric`
Expected: 4 tests run, 4 pass (same four test names as before the move).

Run: `cargo fmt --all --check && cargo clippy -p pleiades-apparent --all-targets -- -D warnings`
Expected: no output / no warnings.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-apparent/src/topocentric.rs crates/pleiades-apparent/src/topocentric/tests.rs
git commit -m "test(topocentric): relocate inline tests to topocentric/tests.rs"
```

---

### Task 2: Primary discriminating-geometry tests (17 kills)

**Files:**
- Modify: `crates/pleiades-apparent/src/topocentric/tests.rs` (append)

**Interfaces:**
- Consumes: `ecl(...)` helper from Task 1.
- Produces: `palomar() -> ObserverLocation` helper, used again by Task 3.

**Background for the implementer:** the geometry `λ=100°, β=5°, Δ=0.00257 AU, ε=23.44°, LAST=70°` at Palomar was chosen so no mutant survives by degeneracy: `ρcosφ′ = 0.8363 ≠ 1` (an equator/sea-level observer makes `* rho_cos_phi_prime → /` bit-identical — that degeneracy is why these mutants survived until now), `dec_topo ≈ 27.9° ≠ 0` (separates `/cos δ` from `%` and `*`, keeps the `sin δ` term alive), `H ≈ 328.2°` (both `cos H` and `sin H` non-zero). Do not "simplify" the geometry — each property is load-bearing (spec §6).

- [ ] **Step 1: Append the Palomar helper and the two tests**

Append to `crates/pleiades-apparent/src/topocentric/tests.rs`:

```rust
/// Meeus ch. 11 worked-example observer (Palomar): φ = +33.356111°, 1706 m.
/// ρcosφ′ ≈ 0.836339 ≠ 1, which is what makes the diurnal-aberration factor
/// mutants (`* rho_cos_phi_prime` → `/`) distinguishable at all.
fn palomar() -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(33.356_111),
        Longitude::from_degrees(0.0),
        Some(1706.0),
    )
}

// ---------------------------------------------------------------------------
// FU-9 exact-literal tests. Every expected value below was computed OUTSIDE
// this crate by an independent Python reimplementation of the published
// pipeline — Meeus ch. 11 observer terms (WGS84), ch. 40 rectangular
// diurnal-parallax subtraction, the classical diurnal-aberration terms
// (Δα = 0.3192″ ρcosφ′ cos H / cos δ, Δδ = 0.3192″ ρcosφ′ sin H sin δ), and
// the standard ecliptic↔equatorial rotation. The script is reproduced in
// docs/superpowers/plans/2026-07-20-fu9-topocentric-mutant-triage.md
// (Appendix A). Reference-vs-crate agreement is ~1e-11″, far inside the
// tolerances asserted here; the literals are the reference's output, never
// this crate's own.
// ---------------------------------------------------------------------------

#[test]
fn palomar_moon_matches_independent_meeus_pipeline() {
    // λ=100°, β=+5°, Δ=0.00257 AU, ε=23.44°, LAST=70°, Palomar.
    // Discriminating geometry (spec §6): ρcosφ′≈0.836, dec_topo≈27.9°,
    // H≈328.2° — no factor is 0, 1, or otherwise mutation-degenerate.
    let out = topocentric_position(ecl(100.0, 5.0, 0.002_57), &palomar(), 70.0, 23.44).unwrap();

    let lon = out.ecliptic.longitude.degrees();
    let lat = out.ecliptic.latitude.degrees();
    let dist = out.ecliptic.distance_au.unwrap();
    assert!((lon - 100.430_618_719_114_62).abs() < 1e-9, "lon {lon}");
    assert!((lat - 4.891_647_280_609_852).abs() < 1e-9, "lat {lat}");
    assert!((dist - 0.002_532_223_707_150_349_7).abs() < 1e-12, "dist {dist}");

    let p = &out.provenance;
    assert!(
        (p.parallax_longitude_arcsec - 1_550.227_388_812_629_7).abs() < 1e-6,
        "parallax lon {}",
        p.parallax_longitude_arcsec
    );
    assert!(
        (p.parallax_latitude_arcsec - -390.069_789_804_534).abs() < 1e-6,
        "parallax lat {}",
        p.parallax_latitude_arcsec
    );
    assert!(
        (p.diurnal_aberration_arcsec - 0.236_287_334_372_904_58).abs() < 1e-6,
        "diurnal {}",
        p.diurnal_aberration_arcsec
    );
    assert!(
        (p.distance_au_used - 0.002_57).abs() < 1e-15,
        "distance used {}",
        p.distance_au_used
    );
}

#[test]
fn parallax_displaces_toward_horizon() {
    // Direction, not just magnitude: for a body above the observer's horizon
    // the observer is closer to it than the geocenter (topocentric distance
    // shrinks) and the ecliptic shift has the reference-predicted signs. The
    // pre-existing magnitude tests are sign-free (`hypot`), which is exactly
    // why the L53–55 subtraction-sign mutants survived until this slice.
    let out = topocentric_position(ecl(100.0, 5.0, 0.002_57), &palomar(), 70.0, 23.44).unwrap();
    assert!(out.ecliptic.distance_au.unwrap() < 0.002_57);
    assert!(out.provenance.parallax_longitude_arcsec > 0.0);
    assert!(out.provenance.parallax_latitude_arcsec < 0.0);
}
```

- [ ] **Step 2: Run the new tests — they must pass first time**

Run: `cargo nextest run -p pleiades-apparent topocentric`
Expected: 6 tests run, 6 pass.

A failure here means the pinned literal disagrees with the crate — that is a
finding, not a tolerance to widen. Stop and reconcile against Appendix A
before proceeding (the cross-check at plan time already validated agreement,
so a failure indicates a transcription error).

- [ ] **Step 3: Verify the mutant kills**

Run:
```bash
cargo mutants -p pleiades-apparent --test-tool nextest \
  --test-workspace=false --file crates/pleiades-apparent/src/topocentric.rs
```
Expected: **10 missed** (down from 27), and `mutants.out/missed.txt` no longer
contains any of lines 53, 54, 55, 67, 69, 71, 72, 73, 78, 88, or 104. The 10
remaining are lines 57, 83 (×2), 84 (×2), 85 (×2), 86 (×2), 95.
If any of the 17 still survives, the geometry transcription is wrong — fix the
test, do not proceed.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-apparent/src/topocentric/tests.rs
git commit -m "test(topocentric): exact-literal Meeus recomposition + parallax direction"
```

---

### Task 3: Wrap-crossing and fail-closed tests (6 kills)

**Files:**
- Modify: `crates/pleiades-apparent/src/topocentric/tests.rs` (append)

**Interfaces:**
- Consumes: `ecl(...)` (Task 1), `palomar()` (Task 2).

**Background:** the provenance Δlon wrap branches (`> 180 → −360`, `< −180 → +360`) never fire in any existing test. Body longitudes just across the 0°/360° seam with a Moon-scale parallax (~0.93°) push the topocentric longitude over the boundary. LAST values were scanned in the reference script: LAST=80° maximizes the westward crossing for λ=0.02°, LAST=280° the eastward for λ=359.98°.

- [ ] **Step 1: Append the three tests**

```rust
#[test]
fn wrap_westward_across_zero() {
    // λ=0.02°: the ~0.93° westward parallax carries the topocentric longitude
    // across 0° to ~359.09°, so raw Δlon = +359.07° and the `> 180 → −360`
    // wrap branch fires. Unwrapped, the provenance would read ~+1.29e6″; the
    // exact literal pins the wrap and its direction.
    let out = topocentric_position(ecl(0.02, 0.0, 0.002_57), &palomar(), 80.0, 23.44).unwrap();
    let lon = out.ecliptic.longitude.degrees();
    assert!((lon - 359.092_861_428_952_8).abs() < 1e-9, "lon {lon}");
    assert!(
        (out.provenance.parallax_longitude_arcsec - -3_337.698_855_769_849_6).abs() < 1e-6,
        "parallax lon {}",
        out.provenance.parallax_longitude_arcsec
    );
    assert!(
        (out.provenance.parallax_latitude_arcsec - -597.126_617_152_460_1).abs() < 1e-6,
        "parallax lat {}",
        out.provenance.parallax_latitude_arcsec
    );
}

#[test]
fn wrap_eastward_across_zero() {
    // λ=359.98°: the ~0.51° eastward parallax carries the topocentric
    // longitude across 360° to ~0.49°, so raw Δlon = −359.49° and the
    // `< −180 → +360` wrap branch fires.
    let out = topocentric_position(ecl(359.98, 0.0, 0.002_57), &palomar(), 280.0, 23.44).unwrap();
    let lon = out.ecliptic.longitude.degrees();
    assert!((lon - 0.492_685_616_714_902_74).abs() < 1e-9, "lon {lon}");
    assert!(
        (out.provenance.parallax_longitude_arcsec - 1_845.668_220_173_502).abs() < 1e-6,
        "parallax lon {}",
        out.provenance.parallax_longitude_arcsec
    );
    assert!(
        (out.provenance.parallax_latitude_arcsec - -2_844.541_308_965_105).abs() < 1e-6,
        "parallax lat {}",
        out.provenance.parallax_latitude_arcsec
    );
}

#[test]
fn non_finite_inputs_fail_closed() {
    // Guard intent: a non-finite LAST or obliquity yields the typed error,
    // never a NaN coordinate. This cannot distinguish the two `||`→`&&` guard
    // mutants (documented equivalents, spec §5.1) — it pins the fail-closed
    // contract itself.
    let err =
        topocentric_position(ecl(100.0, 5.0, 1.0), &palomar(), f64::NAN, 23.44).unwrap_err();
    assert_eq!(
        err,
        ApparentPlaceError::NonFiniteCorrection {
            stage: "topocentric"
        }
    );
    let err =
        topocentric_position(ecl(100.0, 5.0, 1.0), &palomar(), 70.0, f64::NAN).unwrap_err();
    assert_eq!(
        err,
        ApparentPlaceError::NonFiniteCorrection {
            stage: "topocentric"
        }
    );
}
```

- [ ] **Step 2: Run the tests**

Run: `cargo nextest run -p pleiades-apparent topocentric`
Expected: 9 tests run, 9 pass. Same rule as Task 2: a failing literal is a
finding to reconcile, not a tolerance to widen.

- [ ] **Step 3: Verify the mutant kills**

Run:
```bash
cargo mutants -p pleiades-apparent --test-tool nextest \
  --test-workspace=false --file crates/pleiades-apparent/src/topocentric.rs
```
Expected: **4 missed**, exactly:

```
crates/pleiades-apparent/src/topocentric.rs:57:35: replace || with && in topocentric_position
crates/pleiades-apparent/src/topocentric.rs:83:14: replace > with >= in topocentric_position
crates/pleiades-apparent/src/topocentric.rs:85:21: replace < with <= in topocentric_position
crates/pleiades-apparent/src/topocentric.rs:95:50: replace || with && in topocentric_position
```

These are the spec §5 documented equivalents. If any OTHER mutant survives,
fix the tests. If one of these four is unexpectedly CAUGHT, that is good news —
note it for the Task 4 docs update (the follow-ups entry must then claim fewer
equivalents).

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-apparent/src/topocentric/tests.rs
git commit -m "test(topocentric): wrap-crossing provenance + fail-closed guard tests"
```

---

### Task 4: Full validation and follow-ups entry

**Files:**
- Modify: `docs/follow-ups.md` (append a progress paragraph inside the FU-9 section, after the aberration.rs entry, before the `---` that closes FU-9)

- [ ] **Step 1: Run the full blocking CI tier**

Run: `mise run ci`
Expected: green (fmt, clippy, tests). This is the merge gate; do not proceed on failure.

- [ ] **Step 2: Append the FU-9 progress entry**

Append to `docs/follow-ups.md` immediately after the `aberration.rs` progress paragraph (adjust the equivalent-mutant count only if Task 3 Step 3 caught one of the four):

```markdown
**Progress (2026-07-20) — `pleiades-apparent/src/topocentric.rs`:** triaged from
`27` → `4` documented equivalent mutants (spec/plan:
`docs/superpowers/specs/2026-07-20-fu9-topocentric-mutant-triage-design.md`).
Baseline confirmed by the authoritative per-file command (`82 mutants tested,
27 missed, 54 caught, 1 unviable`) — the first slice where the per-file and
whole-workspace figures agree exactly. **Tests-only** like `refraction.rs`: the
only source edit was relocating the inline test module to
`src/topocentric/tests.rs` per AGENTS.md. The dominant root cause was
**sign-free and degenerate assertions**: every parallax assertion used `hypot`
(no sign), the diurnal-aberration bound (`< 0.36″`) constrained no term, and —
decisively — the existing test observer (equator, sea level) makes
`ρcosφ′ = 1.0` exactly, so the `* rho_cos_phi_prime → /` mutants were
**bit-identical** and unkillable from those tests. Reference strategy:
**independent recomposition** — a Python reimplementation of the published
Meeus ch. 11/40 pipeline (script reproduced in the plan doc), cross-validated
against the crate at ~1e-11″, pins exact literals at one discriminating
geometry (Palomar `ρcosφ′ = 0.836`, `dec_topo ≈ 27.9°`, `H ≈ 328.2°` — 17
kills including all four provenance fields) plus two wrap-crossing geometries
(body at λ = 0.02°/359.98°, Moon-scale parallax carrying the topocentric
longitude across the 0°/360° seam — 6 kills). Rejected geometries recorded in
the spec so they are not re-proposed: equator/sea-level observer
(`ρcosφ′ = 1`), and `β ≈ 0` for the primary geometry (`cos δ = 1`,
`sin δ = 0` degeneracies). **Documented residual — 4 equivalent mutants**,
left visible rather than `#[mutants::skip]`-suppressed: `||`→`&&` in both
non-finite guards (both guards return the byte-identical error, and any
non-finite poisons every downstream value, so no reachable input distinguishes
the operators — the L95 case is the `nutation.rs` shape), and `>`→`>=` /
`<`→`<=` in the Δlon wrap comparisons (they differ only at a raw Δlon of
exactly ±180.0°, unreachable since the topocentric shift is bounded ≪ 2°). No
parity gate was touched; the tier stays report-only; `mise run ci` is green.
**Remaining slices** (priority order): `sidereal.rs` (17), `precession.rs`
(17), `lighttime.rs` (5), then the `pleiades-time` and `pleiades-types`
survivors.
```

Also update the trailing "**Remaining slices** (priority order): …" sentence of
the *aberration.rs* progress paragraph — no edit needed there (history is
preserved as written); only the new paragraph carries the updated remaining
list.

- [ ] **Step 3: Commit**

```bash
git add docs/follow-ups.md
git commit -m "docs(follow-ups): record FU-9 topocentric.rs triage (27 -> 4 documented equivalents)"
```

---

### Task 5: Finish the branch

- [ ] **Step 1: Final verification**

Run: `git log --oneline main..HEAD`
Expected: 5 commits (spec, relocation, primary tests, wrap/guard tests, docs).

Run: `git status --short`
Expected: clean (note: a local `mutants.out/` directory may exist from the
verification runs — it is gitignored; do not commit it).

- [ ] **Step 2: Push and open the PR**

Use the **superpowers:finishing-a-development-branch** skill. PR flow as in
prior slices (#35, #36): title
`test(topocentric): FU-9 topocentric.rs mutant triage (27 -> 4 documented equivalents)`,
body summarizing the kill strategy and the 4 documented equivalents, then
watch checks green before merging. **Never use `gh pr merge --auto` on this
repo** (it merges immediately — see memory note); wait for checks, then merge
and delete the branch.

---

## Appendix A: Independent reference script

Reproduced verbatim so the literal derivation is reviewable and re-runnable
(`python3 topocentric_reference.py`). It reimplements the published pipeline
without calling the crate; run output at plan time is pinned in Tasks 2–3.

```python
#!/usr/bin/env python3
"""Independent reference for pleiades-apparent topocentric tests (FU-9 slice 5).

Reimplements the published pipeline WITHOUT calling the crate:
  - Meeus ch. 11 observer terms (rho sin/cos phi' on WGS84)
  - Meeus ch. 40 rectangular diurnal-parallax subtraction
  - classical diurnal aberration: da = 0.3192" rho_cos cosH / cos d,
                                  dd = 0.3192" rho_cos sinH sin d
  - standard ecliptic<->equatorial rotation (Meeus ch. 13, vector form)

Prints exact f64 literals (repr) to pin in src/topocentric/tests.rs.
"""
import math

B_OVER_A = 0.996647189
EARTH_EQ_RADIUS_M = 6378137.0
AU_IN_EARTH_RADII = 23454.779
K_DIURNAL = 0.3192  # arcsec


def observer_terms(lat_deg, h_m):
    phi = math.radians(lat_deg)
    u = math.atan(B_OVER_A * math.tan(phi))
    h = h_m / EARTH_EQ_RADIUS_M
    return (B_OVER_A * math.sin(u) + h * math.sin(phi),
            math.cos(u) + h * math.cos(phi))


def ecl_to_eq(lon_deg, lat_deg, eps_deg):
    lo, la, e = (math.radians(v) for v in (lon_deg, lat_deg, eps_deg))
    x = math.cos(lo) * math.cos(la)
    y = math.sin(lo) * math.cos(la) * math.cos(e) - math.sin(la) * math.sin(e)
    z = math.sin(lo) * math.cos(la) * math.sin(e) + math.sin(la) * math.cos(e)
    ra = math.degrees(math.atan2(y, x)) % 360.0
    dec = math.degrees(math.atan2(z, math.hypot(x, y)))
    return ra, dec


def eq_to_ecl(ra_deg, dec_deg, eps_deg):
    r, d, e = (math.radians(v) for v in (ra_deg, dec_deg, eps_deg))
    x = math.cos(r) * math.cos(d)
    y = math.sin(r) * math.cos(d) * math.cos(e) + math.sin(d) * math.sin(e)
    z = -math.sin(r) * math.cos(d) * math.sin(e) + math.sin(d) * math.cos(e)
    lon = math.degrees(math.atan2(y, x)) % 360.0
    lat = math.degrees(math.atan2(z, math.hypot(x, y)))
    return lon, lat


def topocentric(lon, lat, dist_au, obs_lat, obs_h_m, last_deg, eps_deg):
    ra, dec = ecl_to_eq(lon, lat, eps_deg)
    d = dist_au * AU_IN_EARTH_RADII
    ra_r, dec_r = math.radians(ra), math.radians(dec)
    b = [d * math.cos(dec_r) * math.cos(ra_r),
         d * math.cos(dec_r) * math.sin(ra_r),
         d * math.sin(dec_r)]
    rho_s, rho_c = observer_terms(obs_lat, obs_h_m)
    lst = math.radians(last_deg)
    o = [rho_c * math.cos(lst), rho_c * math.sin(lst), rho_s]
    t = [b[i] - o[i] for i in range(3)]
    topo_dist = math.sqrt(t[0] * t[0] + t[1] * t[1] + t[2] * t[2])
    ra0 = math.atan2(t[1], t[0])
    dec0 = math.asin(t[2] / topo_dist)
    hour_angle = (lst - ra0) % (2.0 * math.pi)
    da_as = K_DIURNAL * rho_c * math.cos(hour_angle) / math.cos(dec0)
    dd_as = K_DIURNAL * rho_c * math.sin(hour_angle) * math.sin(dec0)
    ra1 = ra0 + math.radians(da_as / 3600.0)
    dec1 = dec0 + math.radians(dd_as / 3600.0)
    lon_t, lat_t = eq_to_ecl(math.degrees(ra1) % 360.0, math.degrees(dec1), eps_deg)
    d_lon = lon_t - lon
    branch = "none"
    if d_lon > 180.0:
        d_lon -= 360.0
        branch = ">180 (-=360)"
    elif d_lon < -180.0:
        d_lon += 360.0
        branch = "<-180 (+=360)"
    d_lat = lat_t - lat
    return {
        "out_lon": lon_t % 360.0,
        "out_lat": lat_t,
        "out_dist": topo_dist / AU_IN_EARTH_RADII,
        "prov_plon": d_lon * 3600.0,
        "prov_plat": d_lat * 3600.0,
        "prov_diurnal": math.hypot(da_as * math.cos(dec1), dd_as),
        "dec0_deg": math.degrees(dec0),
        "H_deg": math.degrees(hour_angle),
        "cosH": math.cos(hour_angle),
        "sinH": math.sin(hour_angle),
        "rho_c": rho_c,
        "branch": branch,
    }


PALOMAR = (33.356111, 1706.0)

print("=== primary: lam=100 beta=5 dist=0.00257 eps=23.44 LAST=70, Palomar ===")
p = topocentric(100.0, 5.0, 0.00257, *PALOMAR, 70.0, 23.44)
for k, v in p.items():
    print(f"  {k} = {v!r}")

print("\n=== wrap westward: lam=0.02 LAST=80 ===")
w = topocentric(0.02, 0.0, 0.00257, *PALOMAR, 80.0, 23.44)
for k, v in w.items():
    print(f"  {k} = {v!r}")

print("\n=== wrap eastward: lam=359.98 LAST=280 ===")
e = topocentric(359.98, 0.0, 0.00257, *PALOMAR, 280.0, 23.44)
for k, v in e.items():
    print(f"  {k} = {v!r}")
```

Plan-time output (the source of every literal in Tasks 2–3):

```
primary:  out_lon = 100.43061871911462   out_lat = 4.891647280609852
          out_dist = 0.0025322237071503497
          prov_plon = 1550.2273888126297  prov_plat = -390.069789804534
          prov_diurnal = 0.23628733437290458
          dec0_deg = 27.906182992235266   H_deg = 328.22199155042864
          rho_c = 0.8363392329705123      branch = none
westward: out_lon = 359.0928614289528
          prov_plon = -3337.6988557698496 prov_plat = -597.1266171524601
          branch = >180 (-=360)
eastward: out_lon = 0.49268561671490274
          prov_plon = 1845.668220173502   prov_plat = -2844.541308965105
          branch = <-180 (+=360)
```

Crate cross-check at plan time (temporary example, since removed): every field
agrees with the reference to ≤ 2.3e-11″ / 7e-15°.
