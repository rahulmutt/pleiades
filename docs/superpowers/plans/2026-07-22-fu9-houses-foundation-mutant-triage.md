# FU-9 Houses Foundation Mutant Triage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive the **Foundation** functions of `crates/pleiades-houses/src/systems/mod.rs` (shared geometry primitives, chart-point set, and the trivial/Porphyry family) from **113 surviving mutants to a measured set of documented equivalents**, establishing the independent house-math reference that PRs 2–6 reuse.

> **Execution correction (2026-07-22):** this plan predicted a `113 → 1` residual. Mutation verification during execution measured **26** survivors: **7** were real coverage gaps the crafted normal-path geometries never reached (`asc2`'s `sinx ≈ 0` guard branch; an `asc_mc_from` geometry where the vertex flip actually fires) and **19** were *initially* documented as genuine equivalents. Tasks 5–6 were extended to kill the 7 (two degenerate `asc2` pins + a flip-firing `asc_mc_from` geometry) and document the 19 (see `asc_geometry_equivalent_mutants_are_documented` and the `docs/follow-ups.md` note).
>
> **Further correction (final whole-branch review, 2026-07-23):** 6 of those 19 "equivalents" were themselves misclassified — the equivalence sweep never sampled `lat = 0` (pole height exactly `±90°`, where `tan` is not 180-periodic in f64), and `asc2`'s `1e-12` guard assignment made a comparison reachable at equality that was assumed unreachable. All 6 are now killed by two additional tests. **The true residual is `113 → 13` documented equivalents**, confirmed by the authoritative scoped `cargo mutants` run (164 mutants: `13 missed / 151 caught / 0 unviable`) — see the `docs/follow-ups.md` note for the per-bucket breakdown.

**Architecture:** First PR of the ~6-PR `pleiades-houses` FU-9 campaign (spec: `docs/superpowers/specs/2026-07-22-fu9-houses-mutant-triage-design.md`). **Tests-only** — no production-code change; every added test lives in the existing `crates/pleiades-houses/src/systems/tests.rs` (`use super::*;` reaches the private functions). Expected values come from an **independent reference** (`docs/superpowers/specs/notes/2026-07-22-houses-reference.py`, a from-scratch port of the published swehouse.c Asc1/Asc2 + `swe_houses_armc` point set and Meeus ch. 26 angle formulas), cross-validated against the crate to **all 12 decimals** — never from the code's own output.

**Mutation-triage TDD cycle** (differs from feature TDD — read once): a triage test *passes* on correct `HEAD` code (it pins intent against the independent reference); it *kills a mutant* by failing on the mutated tree. So each task is: write the test → run `cargo nextest` to confirm it **passes** on `HEAD` (proves the literal is right) → the final task re-runs `cargo mutants -F` to confirm the survivors are **caught**.

**Tech Stack:** Rust (stable, via mise), cargo-nextest, cargo-mutants 27.1.0, Python 3 (reference only).

## Global Constraints

- **Tests-only:** no production-code change anywhere in this PR; the only file modified is `crates/pleiades-houses/src/systems/tests.rs` (plus the reference note and the follow-up entry).
- **No parity-gate change:** no `validate-*` file is touched; the mutants tier stays report-only.
- **No `#[mutants::skip]`:** the residual documented-equivalent mutants (13, per the 2026-07-23 correction — see the note above) are left visible with a reachability argument each, e.g. `longitude_opposite`'s `+ -> -`.
- **Independence discipline:** every expected value comes from the independent reference or hand arithmetic, never from running the code under test and pinning its output. The reference is cross-validated against the crate to 1e-12 before its literals are trusted.
- **Branch:** `fu9-houses-mutant-triage` (already created; the design spec + re-scope commits are on it). This branch is the Foundation PR.
- All commands run through mise: prefix cargo invocations with `mise exec --`.
- **Obliquity constant used by every geometry:** `EPS = 23.4366` degrees (a representative true obliquity of date). `SINE = EPS.to_radians().sin()`, `COSE = EPS.to_radians().cos()`.

## Reference literals (cross-validated to 1e-12 against the crate)

All produced by `houses-reference.py` and confirmed equal to the crate's private-function output at all 12 decimals (see the plan's provenance step in Task 6). Reproduced inline in each task.

---

### Task 1: `spherical_cotrans` — 34 survivors → 0

`spherical_cotrans(coord, angle)` is a pure rotation of a spherical coordinate about the x-axis. All 34 survivors are `* → +`/`/` and `+ → *`/`-` swaps in the Cartesian conversion + rotation + back-conversion. One crafted non-degenerate input (`lon=40°, lat=25°, r=2`, `angle=15°` — no zero/unit term to hide a swap) pins the full `[lon', lat', r']` output and kills every survivor.

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `spherical_cotrans(coord: &mut [f64; 3], angle_deg: f64)` (private, reached via `use super::*;`).
- Produces: nothing consumed by later tasks.

- [ ] **Step 1: Write the pinning test**

Append to `crates/pleiades-houses/src/systems/tests.rs`:

```rust
// --- FU-9 Foundation: shared geometry primitives ---

#[test]
fn spherical_cotrans_matches_independent_x_axis_rotation() {
    // Independent reference (houses-reference.py `spherical_cotrans`): a pure
    // x-axis rotation of (lon,lat,r) -> Cartesian -> rotate by `angle` -> back.
    // Geometry avoids every degeneracy (no 0°/90° angle, non-unit radius) so
    // each `*`/`+` term is observable. Cross-validated to 1e-12 vs the crate.
    let mut coord = [40.0_f64, 25.0, 2.0];
    spherical_cotrans(&mut coord, 15.0);
    assert!((coord[0] - 44.070_120_506_012).abs() < 1e-9, "lon' = {}", coord[0]);
    assert!((coord[1] - 14.918_178_485_226).abs() < 1e-9, "lat' = {}", coord[1]);
    assert!((coord[2] - 2.000_000_000_000).abs() < 1e-9, "r' = {}", coord[2]);
}
```

- [ ] **Step 2: Run the test — confirm it passes on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses spherical_cotrans_matches_independent_x_axis_rotation`
Expected: `1 passed`. (A failure means the literal or import is wrong — fix before proceeding.)

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): FU-9 pin spherical_cotrans against independent rotation (34 mutants)"
```

---

### Task 2: `asc1` + `asc2` — 28 survivors → 0

`asc2` is the swehouse.c Asc2 kernel (`atan(sinx/value)`, `value = -tan(pole)·sine + cose·cos(x)`), with three internal guard branches (`|value|<1e-12`, `|sinx|<1e-12`, `value==0`, `longitude<0 += 180`). `asc1` dispatches by quadrant into `asc2` with sign/argument folding (`180-x`, `x-180`, `360-x`, `±pole`). Survivors are the comparison swaps in the guards, the `delete match arm 2/3`, and the quadrant arithmetic. Four quadrant inputs (`30/120/210/300`, `pole=52`, `EPS`) pin `asc1` across all four arms; the same four pin `asc2` directly (exercising the normal `atan` path and the `longitude += 180` fold).

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `asc2(x, pole_height, sine, cose) -> f64`, `asc1(x1, pole_height, sine, cose) -> Longitude` (private).
- Produces: nothing consumed by later tasks.

- [ ] **Step 1: Write the pinning tests**

Append:

```rust
#[test]
fn asc2_matches_independent_swehouse_kernel() {
    // Independent reference (houses-reference.py `asc2`, swehouse.c Asc2) at
    // pole height 52°, obliquity sine/cosine. Four x values, one per asc1
    // quadrant; each takes the normal atan branch. Cross-validated to 1e-12.
    let eps = 23.4366_f64;
    let (sine, cose) = (eps.to_radians().sin(), eps.to_radians().cos());
    let cases = [
        (30.0_f64, 60.273_411_210_075),
        (120.0, 138.177_359_444_927),
        (210.0, 20.983_664_735_370),
        (300.0, 86.674_198_092_798),
    ];
    for (x, expected) in cases {
        assert!(
            (asc2(x, 52.0, sine, cose) - expected).abs() < 1e-9,
            "asc2({x}) = {}",
            asc2(x, 52.0, sine, cose)
        );
    }
}

#[test]
fn asc1_dispatches_each_quadrant_to_independent_reference() {
    // houses-reference.py `asc1` (swehouse.c Asc1): quadrant fold into Asc2.
    // 30° -> Q1, 120° -> Q2, 210° -> Q3, 300° -> Q4 exercise all four match
    // arms and the ±pole / (180-x)/(x-180)/(360-x) argument folding.
    let eps = 23.4366_f64;
    let (sine, cose) = (eps.to_radians().sin(), eps.to_radians().cos());
    let cases = [
        (30.0_f64, 60.273_411_210_075),
        (120.0, 138.177_359_444_927),
        (210.0, 200.983_664_735_370),
        (300.0, 266.674_198_092_798),
    ];
    for (x, expected) in cases {
        let got = asc1(x, 52.0, sine, cose).degrees();
        assert!((got - expected).abs() < 1e-9, "asc1({x}) = {got}");
    }
}
```

- [ ] **Step 2: Run the tests — confirm they pass on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses asc1_dispatches asc2_matches`
Expected: `2 passed`.

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): FU-9 pin asc1/asc2 swehouse kernel per quadrant (28 mutants)"
```

---

### Task 3: `asc_mc_from` — 22 survivors → 0

`asc_mc_from` composes the full `AscMc` chart-point set (ascendant, MC, vertex, equatorial ascendant, both co-ascendants, polar ascendant) from armc/lat/obliquity. Survivors are in the pole-height branch (`f_pole = 90-lat` vs `-90-lat`, the `lat >= 0` test), the vertex flip (`|lat| <= obl`, `vemc > 180`, `vemc > 0`, `vertex + 180`), and the MC/opposite composition. **Three geometries** pin every branch: G1 `lat > obl` (non-flip, `f_pole = 90-lat`), G2 `0 < lat <= obl` (flip branch active), G3 `lat < 0` (`-90-lat` branch). Each pins all seven independent points.

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `asc_mc_from(armc_deg, lat_deg, obliquity_deg) -> Result<AscMc, HouseError>`; the `AscMc` fields `ascendant`, `midheaven`, `vertex`, `equatorial_ascendant`, `coascendant_koch`, `coascendant_munkasey`, `polar_ascendant` (all `Longitude`).
- Produces: nothing consumed by later tasks.

- [ ] **Step 1: Write the three-geometry pin**

Append:

```rust
#[test]
fn asc_mc_from_pins_all_points_across_pole_and_flip_branches() {
    // Independent reference (houses-reference.py `asc_mc_from`, swehouse.c
    // swe_houses_armc). obl = 23.4366°. Three geometries cover:
    //   G1 lat>obl  -> f_pole = 90-lat, vertex flip inactive
    //   G2 0<lat<=obl -> flip branch active (vemc>0 path)
    //   G3 lat<0    -> f_pole = -90-lat branch
    // Every literal cross-validated to 1e-12 against the crate.
    let eps = 23.4366_f64;
    let check = |armc: f64, lat: f64, exp: [f64; 7]| {
        let p = asc_mc_from(armc, lat, eps).expect("finite");
        let got = [
            p.ascendant.degrees(),
            p.midheaven.degrees(),
            p.vertex.degrees(),
            p.equatorial_ascendant.degrees(),
            p.coascendant_koch.degrees(),
            p.coascendant_munkasey.degrees(),
            p.polar_ascendant.degrees(),
        ];
        for (i, (g, e)) in got.iter().zip(exp.iter()).enumerate() {
            assert!((g - e).abs() < 1e-8, "armc={armc} lat={lat} point[{i}] = {g}, want {e}");
        }
    };
    // G1: armc=45, lat=52 (> obl) — non-flip, f_pole = 38.
    check(45.0, 52.0, [
        148.587_249_395_771, 47.463_595_280_938, 295.549_781_631_009,
        132.536_404_719_062, 101.175_335_496_703, 143.611_940_830_436,
        281.175_335_496_703,
    ]);
    // G2: armc=200, lat=10 (0<lat<=obl) — vertex flip active.
    check(200.0, 10.0, [
        284.537_224_659_332, 201.638_102_932_963, 159.911_766_495_904,
        288.466_379_243_755, 292.223_701_037_155, 205.822_995_524_640,
        112.223_701_037_155,
    ]);
    // G3: armc=100, lat=-33 (< 0) — f_pole = -90-lat = -57.
    check(100.0, -33.0, [
        195.061_993_029_707, 99.189_697_612_154, 6.534_310_124_205,
        190.878_573_217_375, 188.500_387_274_068, 210.816_655_151_624,
        8.500_387_274_068,
    ]);
}
```

- [ ] **Step 2: Run the test — confirm it passes on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses asc_mc_from_pins_all_points`
Expected: `1 passed`.

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): FU-9 pin asc_mc_from chart points across 3 branch geometries (22 mutants)"
```

---

### Task 4: small primitives + trivial/Porphyry family — 28 survivors → 0

Groups the remaining low-count Foundation functions: `interpolate_longitude` (6), `porphyry_houses` (16), `signed_longitude_difference` (3), `right_ascension_from_ecliptic_longitude` (1), `whole_sign_houses` (1), `longitude_in_arc` (1). Each pinned by hand-arithmetic or the independent reference at a discriminating geometry.

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append)

**Interfaces:**
- Consumes: `interpolate_longitude(start, end, fraction) -> Longitude`, `porphyry_houses(angles: HouseAngles) -> [Longitude; 12]`, `HouseAngles::new(asc, mc)`, `signed_longitude_difference(a, b) -> f64`, `right_ascension_from_ecliptic_longitude(lon, obl_rad) -> f64`, `whole_sign_houses(asc) -> [Longitude; 12]`, `longitude_in_arc(lon, start, end) -> bool` (all private).
- Produces: nothing consumed by later tasks.

- [ ] **Step 1: Write the grouped pins**

Append:

```rust
#[test]
fn interpolate_longitude_wraps_and_scales() {
    // span = (end-start).rem_euclid(360) = 30; start + span*frac = 357.5.
    // start=350,end=20,frac=0.25 keeps every mutant (- -> +//, + -> *//-,
    // * -> +//) observably wrong. Hand-computed.
    let got = interpolate_longitude(
        Longitude::from_degrees(350.0),
        Longitude::from_degrees(20.0),
        0.25,
    )
    .degrees();
    assert!((got - 357.5).abs() < 1e-9, "interp = {got}");
}

#[test]
fn porphyry_houses_trisect_each_quadrant() {
    // asc=100, mc=10 -> desc=280, ic=190. Each quadrant spans 90°, trisected
    // at 30°/60°. Independent hand arithmetic (houses-reference.py `porphyry`)
    // makes the 1/3 and 2/3 fractions observable (mutating / -> % or *).
    let cusps = porphyry_houses(HouseAngles::new(
        Longitude::from_degrees(100.0),
        Longitude::from_degrees(10.0),
    ));
    let expected = [
        100.0, 130.0, 160.0, 190.0, 220.0, 250.0, 280.0, 310.0, 340.0, 10.0,
        40.0, 70.0,
    ];
    for (i, e) in expected.iter().enumerate() {
        assert!(
            (cusps[i].degrees() - e).abs() < 1e-9,
            "cusp[{i}] = {}, want {e}",
            cusps[i].degrees()
        );
    }
}

#[test]
fn signed_longitude_difference_both_branches() {
    // delta<180 branch: (10-350).rem_euclid(360)=20 -> 20.
    assert!((signed_longitude_difference(10.0, 350.0) - 20.0).abs() < 1e-9);
    // delta>=180 branch: (200-10).rem_euclid(360)=190 -> 190-360 = -170.
    assert!((signed_longitude_difference(200.0, 10.0) + 170.0).abs() < 1e-9);
}

#[test]
fn right_ascension_from_ecliptic_longitude_matches_reference() {
    // atan2(sinλ·cosε, cosλ) at λ=60°, ε=23.4366°. Independent reference.
    let eps = 23.4366_f64;
    let got = right_ascension_from_ecliptic_longitude(
        Longitude::from_degrees(60.0),
        eps.to_radians(),
    );
    assert!((got - 57.819_266_732_173).abs() < 1e-9, "ra = {got}");
}

#[test]
fn whole_sign_first_cusp_floors_to_sign_boundary() {
    // asc=95° -> first cusp floor(95/30)*30 = 90°. The `* 30` mutant (-> /30)
    // collapses the cusp to 0.1; pin the first two cusps.
    let cusps = whole_sign_houses(Longitude::from_degrees(95.0));
    assert!((cusps[0].degrees() - 90.0).abs() < 1e-9, "c0 = {}", cusps[0].degrees());
    assert!((cusps[1].degrees() - 120.0).abs() < 1e-9, "c1 = {}", cusps[1].degrees());
}

#[test]
fn longitude_in_arc_handles_wraparound() {
    // Wraparound arc [350,10): membership is `lon>=350 || lon<10`. A point at
    // 355 is in via the first disjunct only, so || -> && flips it to false.
    assert!(longitude_in_arc(355.0, 350.0, 10.0), "355 in [350,10)");
    assert!(longitude_in_arc(5.0, 350.0, 10.0), "5 in [350,10)");
    // Non-wrap arc [10,20): 15 in, 25 out.
    assert!(longitude_in_arc(15.0, 10.0, 20.0));
    assert!(!longitude_in_arc(25.0, 10.0, 20.0));
}
```

- [ ] **Step 2: Run the tests — confirm they pass on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses interpolate_longitude_wraps porphyry_houses_trisect signed_longitude_difference right_ascension_from_ecliptic whole_sign_first_cusp longitude_in_arc_handles`
Expected: `6 passed`.

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs
git commit -m "test(houses): FU-9 pin interpolate/porphyry/signed-diff/RA/whole-sign/arc primitives (28 mutants)"
```

---

### Task 5: `longitude_opposite` documented equivalent + reference note

`longitude_opposite(x) = Longitude::from_degrees(x + 180.0)`. The one survivor is `+ → -`. Because `Longitude::from_degrees` normalizes mod 360 and `x + 180 ≡ x - 180 (mod 360)` for **all** `x`, the mutated function is bit-identical to the original on every reachable input — a genuine **equivalent mutant**. It is left visible (no `#[mutants::skip]`) and documented, per FU-9 posture. This task also lands the independent reference note that the earlier tasks cite.

**Files:**
- Create: `docs/superpowers/specs/notes/2026-07-22-houses-reference.py`
- Test: `crates/pleiades-houses/src/systems/tests.rs` (append a characterization test that documents the equivalence)

**Interfaces:**
- Consumes: `longitude_opposite(longitude: Longitude) -> Longitude` (private).
- Produces: the committed reference script (provenance for Tasks 1–4).

- [ ] **Step 1: Commit the independent reference script**

Copy the cross-validated reference into the repo notes directory so the literals in Tasks 1–4 have committed provenance:

```bash
cp /tmp/claude-1000/-workspace/eae209fc-a264-4012-87be-3fef12ebe44d/scratchpad/houses_ref.py \
   docs/superpowers/specs/notes/2026-07-22-houses-reference.py
```

(The script content is the verified port in the plan's provenance step; it prints every literal used above.)

- [ ] **Step 2: Write the equivalence-documenting characterization test**

Append:

```rust
#[test]
fn longitude_opposite_is_the_antipode() {
    // `longitude_opposite(x) = from_degrees(x + 180)`. NOTE: the cargo-mutants
    // survivor `+ -> -` here is a DOCUMENTED EQUIVALENT MUTANT, not a coverage
    // hole: from_degrees normalizes mod 360 and x+180 ≡ x-180 (mod 360) for all
    // x, so no reachable input distinguishes `+` from `-`. It is left visible
    // (no #[mutants::skip]) per FU-9 posture. This test pins the antipode
    // intent; it cannot and does not claim to kill the equivalent mutant.
    assert!((longitude_opposite(Longitude::from_degrees(50.0)).degrees() - 230.0).abs() < 1e-9);
    assert!((longitude_opposite(Longitude::from_degrees(300.0)).degrees() - 120.0).abs() < 1e-9);
}
```

- [ ] **Step 3: Run the test — confirm it passes on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses longitude_opposite_is_the_antipode`
Expected: `1 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-houses/src/systems/tests.rs docs/superpowers/specs/notes/2026-07-22-houses-reference.py
git commit -m "test(houses): FU-9 document longitude_opposite equivalent mutant + commit reference"
```

---

### Task 6: Verify Foundation reaches 1 documented equivalent + follow-up note

Re-run cargo-mutants scoped to the Foundation functions and confirm exactly **1 missed** (the `longitude_opposite` equivalent). Then record the slice in `docs/follow-ups.md`.

**Files:**
- Modify: `docs/follow-ups.md` (append an FU-9 Progress note)

- [ ] **Step 1: Confirm the whole test suite is green**

Run: `mise exec -- cargo nextest run -p pleiades-houses`
Expected: all pass (75 existing + 11 new = 86).

- [ ] **Step 2: Run scoped mutation verification (~164 mutants, ~4 min)**

Run:
```bash
mise exec -- cargo mutants -p pleiades-houses \
  --test-tool nextest --test-workspace=false \
  --file crates/pleiades-houses/src/systems/mod.rs \
  -F 'in (spherical_cotrans|asc1|asc2|asc_mc_from|interpolate_longitude|signed_longitude_difference|right_ascension_from_ecliptic_longitude|longitude_opposite|longitude_in_arc|whole_sign_houses|porphyry_houses)$'
```
Expected (as measured during execution): **`19 missed`**, all documented equivalents (see the execution-correction note at the top of this plan and `asc_geometry_equivalent_mutants_are_documented`); all others **caught**. Confirm with `grep -c '' mutants.out/missed.txt` → `19` and `cat mutants.out/missed.txt` matches the enumerated equivalent set.

> **Update (final whole-branch review, 2026-07-23):** 6 of the 19 were later found killable (see the correction note at the top of this plan). After the fix, the expected residual is **`13 missed`** (predicted, pending a fresh scoped `cargo mutants` run to confirm) — the 6 newly-added tests should catch `lat_deg >= 0.0 -> < 0.0` (192), `delete -` at 195, `vemc > 180.0 -> >= 180.0` (204), `vemc > 0.0 -> >= 0.0` (207), `value.abs() < 1e-12 -> == 1e-12` (1807), and `value < 0.0 -> <= 0.0` (1812).

If any *other* mutant is still missed (not in the documented-equivalent set), classify it (it will be an arithmetic/comparison swap in a function above), add a discriminating assertion to the matching test using the independent reference, and re-run — do not proceed until the residual is exactly the documented equivalents.

- [ ] **Step 3: Append the FU-9 Progress note to `docs/follow-ups.md`**

Under the FU-9 section, after the "FU-9 measured baseline CLOSED" paragraph, add a new post-baseline-expansion progress entry. **Note (execution correction):** the block below was the plan's draft assuming a `113 → 1` residual; the note actually committed to `docs/follow-ups.md` reflects the measured `113 → 19 documented equivalents` (7 killable gaps also killed). Use the committed note, not this draft, as the source of truth:

```markdown
**Progress (2026-07-22) — houses Foundation
(`pleiades-houses/src/systems/mod.rs`, shared primitives):** first PR of the
post-baseline `pleiades-houses` expansion campaign (spec:
`docs/superpowers/specs/2026-07-22-fu9-houses-mutant-triage-design.md`). The
whole-crate baseline measured `1,231 mutants, 569 missed` — `systems/mod.rs`
alone has `554`, ~15× the previous largest slice, so the crate is worked as a
~6-PR family-grouped campaign. This Foundation PR triaged the shared geometry
primitives + chart-point set + trivial/Porphyry family from `113` → `1`
documented equivalent: `spherical_cotrans` (34), `asc1`/`asc2` (28),
`asc_mc_from` (22), `porphyry_houses` (16), `interpolate_longitude` (6),
`signed_longitude_difference` (3), and one each in
`right_ascension_from_ecliptic_longitude`, `whole_sign_houses`,
`longitude_in_arc`. **Tests-only** — every expected value comes from an
independent from-scratch port of the published swehouse.c Asc1/Asc2 +
`swe_houses_armc` point set (`docs/superpowers/specs/notes/2026-07-22-houses-reference.py`),
cross-validated against the crate to 1e-12 before its literals were trusted.
Killing the shared primitives once removes their survivors from every
composing system, which the later family PRs build on. **Documented residual —
1 equivalent mutant**, left visible: `longitude_opposite`'s `+ -> -`, because
`from_degrees` normalizes mod 360 and `x + 180 ≡ x - 180 (mod 360)` for all
`x`, so no reachable input distinguishes the operators. No parity gate was
touched; the tier stays report-only; `mise run ci` is green. **Remaining
houses PRs:** great-circle (`apc_sector`/`krusinski`/`horizon`), sector
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
git commit -m "docs(follow-ups): record FU-9 houses Foundation triage (113 -> 1 documented equivalent)"
```

---

## Self-Review

- **Spec coverage:** This plan implements the spec's **Foundation** PR row (shared primitives + angles + trivial/Porphyry) and the method/reference-strategy/acceptance sections for those functions. The other five campaign PRs are separate plans (written after this merges, each re-confirming its survivor membership against `mutants.out/missed.txt`).
- **Placeholder scan:** none — every test body is complete with cross-validated literals; the reference script is committed in Task 5.
- **Type consistency:** function signatures (`spherical_cotrans`, `asc1`/`asc2`, `asc_mc_from` returning `Result<AscMc, _>`, `HouseAngles::new`, `Longitude::degrees()`) match the crate as read at `7572b234c`.
- **Independence:** literals verified equal to the crate to 1e-12, but derived from an independent published-formula port — non-circular.
