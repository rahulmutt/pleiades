# FU-9 Slice 6 — Sidereal Mutant Triage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Kill all 22 surviving cargo-mutants mutants in `crates/pleiades-time/src/sidereal.rs` (5) and `crates/pleiades-apparent/src/sidereal.rs` (17) with intent-expressing tests against independent references — tests-only, expected residual 0 + 0.

**Architecture:** Two per-crate test tasks plus a docs task. Time side: pin `gmst_degrees_raw` to Meeus eq. 12.4 literals evaluated outside the code at t = ±4 Julian centuries (the ± pair separates the even quadratic from the odd cubic term). Apparent side: one recomposition-pinning test asserts all four `SiderealTime` degree fields and all four hours accessors against expectations rebuilt in-test from independently-invoked sub-functions, plus a Meeus example 12.b external-authority anchor. Each crate's inline `#[cfg(test)] mod tests` relocates to `src/sidereal/tests.rs` per AGENTS.md.

**Tech Stack:** Rust (stable, via `mise`), cargo-nextest, cargo-mutants 27.1.0.

**Spec:** `docs/superpowers/specs/2026-07-20-fu9-sidereal-mutant-triage-design.md` (approved). Work happens on the existing branch `fu9-sidereal-mutant-triage`.

## Global Constraints

- **Tests-only slice:** no production-code change in either file; the only source edits are the two inline test-module relocations (`#[cfg(test)] mod tests;` + new `src/sidereal/tests.rs`).
- **No `#[mutants::skip]`** anywhere.
- **No `validate-*` parity-gate file** is touched; the mutants tier stays report-only.
- **Independence discipline:** every expected value comes from an independent reference (external Meeus literals, or recomposition from independently-invoked sub-functions) — never from the code-under-test's own output.
- Existing tests are **kept**, relocated verbatim — they carry real intent the new tests do not replace.
- Mutation-triage TDD note: the usual red→green cycle does not apply — the "red" evidence is the already-captured per-file baseline (17 + 5 missed). New tests must PASS immediately on unmutated code; the kill is verified by the per-file cargo-mutants re-run in each task.
- If a mutants re-run unexpectedly still reports a missed mutant, do NOT suppress it — stop and apply the established classify-then-kill-or-document treatment (see spec §5), updating spec + follow-ups text to match reality.

---

### Task 1: `pleiades-time` — relocate tests + Meeus 12.4 large-t literal pinning

**Files:**
- Create: `crates/pleiades-time/src/sidereal/tests.rs`
- Modify: `crates/pleiades-time/src/sidereal.rs` (replace inline `mod tests` block, lines 20–42, with `#[cfg(test)] mod tests;`)

**Interfaces:**
- Consumes: `gmst_degrees_raw(jd_ut1: f64) -> f64`, `gmst_degrees(jd_ut1: f64) -> f64` (existing public functions; unchanged).
- Produces: nothing new — tests only. Task 2 relies on these same public functions but does not depend on this task's tests.

- [ ] **Step 1: Create the relocated + expanded test file**

Create `crates/pleiades-time/src/sidereal/tests.rs` with the two existing tests moved verbatim and two new tests:

```rust
use super::*;

#[test]
fn gmst_matches_meeus_example_12a() {
    // Meeus Example 12.a: 1987 April 10, 0h UT -> JD 2446895.5,
    // GMST = 13h10m46.3668s = 197.693195 deg.
    let gmst = gmst_degrees(2_446_895.5);
    assert!((gmst - 197.693_195).abs() < 1e-4, "gmst {gmst}");
}

#[test]
fn gmst_is_normalized() {
    for jd in [2_415_020.5_f64, 2_451_545.0, 2_488_069.5] {
        let g = gmst_degrees(jd);
        assert!(
            (0.0..360.0).contains(&g),
            "gmst {g} out of range at jd {jd}"
        );
    }
}

#[test]
fn gmst_raw_matches_meeus_12_4_at_large_t() {
    // Meeus eq. 12.4 evaluated OUTSIDE the code (double precision, same
    // published coefficients) at t = ±4 Julian centuries from J2000
    // (JD 2451545 ± 4·36525). Large |t| makes the quadratic (~6.2e-3°) and
    // cubic (~1.65e-6°) terms visible; the ± pair separates the even
    // quadratic term (same sign at ±t) from the odd cubic term (flips sign).
    // Tolerance 2e-7° is ~27 ulp of the ~5.27e7° raw value — ≥4× below the
    // smallest single-epoch mutant displacement (~8.3e-7°) and ≥25× above
    // last-ulp evaluation noise (margins verified in the slice design doc §4.1).
    assert!(
        (gmst_degrees_raw(2_597_645.0) - 52_740_283.547_038_615).abs() < 2e-7,
        "t=+4: {}",
        gmst_degrees_raw(2_597_645.0)
    );
    assert!(
        (gmst_degrees_raw(2_305_445.0) - (-52_739_722.613_388_02)).abs() < 2e-7,
        "t=-4: {}",
        gmst_degrees_raw(2_305_445.0)
    );
}

#[test]
fn gmst_normalized_matches_raw_at_large_t() {
    // Pins the normalized path to the raw polynomial at the new ±4-century
    // epochs (redundant kill — `gmst_degrees`'s own mutants are already
    // caught; this keeps raw and normalized coupled at the epochs the
    // literal test above relies on).
    for jd in [2_597_645.0_f64, 2_305_445.0] {
        assert!(
            (gmst_degrees(jd) - gmst_degrees_raw(jd).rem_euclid(360.0)).abs() < 1e-9,
            "jd {jd}"
        );
    }
}
```

- [ ] **Step 2: Replace the inline module in `sidereal.rs`**

In `crates/pleiades-time/src/sidereal.rs`, delete the entire inline block (lines 20–42):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    // ... both existing tests ...
}
```

and replace it with:

```rust
#[cfg(test)]
mod tests;
```

No other change to the file — `gmst_degrees_raw` and `gmst_degrees` are untouched.

- [ ] **Step 3: Confirm the pinned literals independently**

Before trusting the literals, re-derive them outside the crate:

```bash
python3 - <<'EOF'
J2000 = 2_451_545.0
def gmst_raw(jd):
    d = jd - J2000
    t = d / 36_525.0
    return 280.460_618_37 + 360.985_647_366_29*d + 0.000_387_933*t*t - (t*t*t)/38_710_000.0
print(repr(gmst_raw(2_597_645.0)), repr(gmst_raw(2_305_445.0)))
EOF
```

Expected output: `52740283.547038615 -52739722.61338802`. If it differs, the test literals are wrong — fix the literals, not the code.

- [ ] **Step 4: Run the crate tests**

```bash
cargo nextest run -p pleiades-time sidereal
```

Expected: PASS, 4 tests (2 relocated + 2 new). New tests must pass on unmutated code — a failure here means a wrong literal or tolerance, not a code bug.

- [ ] **Step 5: Format and lint**

```bash
cargo fmt --all
cargo clippy -p pleiades-time --all-targets --all-features -- -D warnings
```

Expected: no diff / no warnings.

- [ ] **Step 6: Re-run the authoritative per-file mutants command**

```bash
cargo mutants -p pleiades-time --test-tool nextest \
  --test-workspace=false --file crates/pleiades-time/src/sidereal.rs
```

Expected: **30 mutants tested: 0 missed** (baseline was 5 missed, 25 caught). If any mutant is still missed, STOP — apply the Global Constraints' classify-then-kill-or-document rule before committing.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-time/src/sidereal.rs crates/pleiades-time/src/sidereal/tests.rs
git commit -m "test(time): FU-9 sidereal — pin gmst_degrees_raw to Meeus 12.4 literals at t = ±4 (5 -> 0)"
```

---

### Task 2: `pleiades-apparent` — relocate tests + recomposition pinning + Meeus 12.b anchor

**Files:**
- Create: `crates/pleiades-apparent/src/sidereal/tests.rs`
- Modify: `crates/pleiades-apparent/src/sidereal.rs` (replace inline `mod tests` block, lines 93–184, with `#[cfg(test)] mod tests;`)

**Interfaces:**
- Consumes: `sidereal_time(instant: Instant, observer_longitude: Longitude) -> SiderealTime`, `equation_of_equinoxes(delta_psi_deg: f64, true_obliquity_deg: f64) -> f64`, `greenwich_mean_sidereal_time_degrees(jd: f64) -> f64` (existing, unchanged); `pleiades_time::gmst_degrees_raw` / `gmst_degrees`; `crate::nutation::{nutation, mean_obliquity_degrees}`.
- Produces: nothing new — tests only.

- [ ] **Step 1: Create the relocated + expanded test file**

Create `crates/pleiades-apparent/src/sidereal/tests.rs`. The seven existing tests move verbatim (they carry intent the new tests do not replace: J2000 GMST value, GAST+lon relationship, normalization/hours consistency, EE magnitude, helper equivalence, EE wrapper/helper agreement, cross-crate GMST agreement); two new tests are appended:

```rust
use super::*;
use pleiades_types::{Instant, JulianDay, Longitude, TimeScale};

fn j2000() -> Instant {
    Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt)
}

#[test]
fn gmst_at_j2000_is_about_280_46_degrees() {
    let gmst = greenwich_mean_sidereal_time_degrees(2_451_545.0);
    // GMST at J2000.0 ≈ 280.4606°.
    assert!(
        (gmst.rem_euclid(360.0) - 280.4606).abs() < 1e-3,
        "got {gmst}"
    );
}

#[test]
fn local_apparent_equals_gast_plus_east_longitude() {
    let st = sidereal_time(j2000(), Longitude::from_degrees(90.0));
    let expected = (st.gast_deg + 90.0).rem_euclid(360.0);
    assert!((st.local_apparent_deg - expected).abs() < 1e-9, "{st:?}");
}

#[test]
fn all_fields_normalized_and_hours_consistent() {
    let st = sidereal_time(j2000(), Longitude::from_degrees(-123.4));
    for v in [
        st.gmst_deg,
        st.gast_deg,
        st.local_mean_deg,
        st.local_apparent_deg,
    ] {
        assert!((0.0..360.0).contains(&v), "not normalized: {v}");
    }
    assert!((st.gmst_hours() - st.gmst_deg / 15.0).abs() < 1e-12);
}

#[test]
fn equation_of_equinoxes_is_small() {
    // EE is at most a couple of arcseconds ≈ a few×1e-4 degrees.
    assert!(equation_of_equinoxes_degrees(2_451_545.0).abs() < 0.01);
}

#[test]
fn equation_of_equinoxes_helper_matches_formula() {
    let delta_psi_deg: f64 = 0.001_234;
    let true_obliquity_deg: f64 = 23.44;
    let expected = delta_psi_deg * true_obliquity_deg.to_radians().cos();
    assert!(
        (equation_of_equinoxes(delta_psi_deg, true_obliquity_deg) - expected).abs() < 1e-15
    );
}

#[test]
fn equation_of_equinoxes_degrees_uses_shared_helper() {
    // The jd-driven wrapper must equal the helper fed the same nutation inputs.
    let jd = 2_451_545.0;
    let n = crate::nutation::nutation(jd).expect("nutation table available in tests");
    let delta_psi_deg = n.delta_psi_arcsec / 3600.0;
    let true_obl_deg =
        crate::nutation::mean_obliquity_degrees(jd) + n.delta_eps_arcsec / 3600.0;
    assert!(
        (equation_of_equinoxes_degrees(jd)
            - equation_of_equinoxes(delta_psi_deg, true_obl_deg))
        .abs()
            < 1e-15
    );
}

#[test]
fn apparent_gmst_matches_pleiades_time_source() {
    for jd in [
        2_415_020.5_f64,
        2_433_283.0,
        2_451_545.0,
        2_469_807.0,
        2_488_069.5,
    ] {
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

#[test]
fn sidereal_time_fields_match_independent_recomposition() {
    // Pinned epoch/longitude: the Meeus ch. 12 epoch (JD 2446895.5) and
    // lon = +52.5° east. Non-degeneracy, each property load-bearing
    // (design doc §4.2): lon ≠ 0 separates local from Greenwich fields and
    // `norm(gmst + lon)` from the `-`/`*` mutants; EE ≠ 0 separates mean
    // from apparent fields; no field's value collides with the accessor
    // mutants (deg/15 ∉ {deg%15, deg·15, 0.0, 1.0, −1.0}).
    let jd = 2_446_895.5;
    let lon_deg = 52.5;
    let st = sidereal_time(
        Instant::new(JulianDay::from_days(jd), TimeScale::Ut1),
        Longitude::from_degrees(lon_deg),
    );

    // Expected values recomposed from independently-invoked pieces — never
    // read back from `st`. `Angle::normalized_0_360` is rem_euclid(360.0),
    // and the recomposition mirrors production's operation order, so the
    // comparison is bit-level up to the 1e-12 tolerance.
    let gmst = pleiades_time::gmst_degrees_raw(jd);
    let n = crate::nutation::nutation(jd).expect("nutation table available in tests");
    let ee = equation_of_equinoxes(
        n.delta_psi_arcsec / 3600.0,
        crate::nutation::mean_obliquity_degrees(jd) + n.delta_eps_arcsec / 3600.0,
    );
    let expected_deg = [
        gmst.rem_euclid(360.0),
        (gmst + ee).rem_euclid(360.0),
        (gmst + lon_deg).rem_euclid(360.0),
        (gmst + ee + lon_deg).rem_euclid(360.0),
    ];
    let actual_deg = [st.gmst_deg, st.gast_deg, st.local_mean_deg, st.local_apparent_deg];
    let actual_hours = [
        st.gmst_hours(),
        st.gast_hours(),
        st.local_mean_hours(),
        st.local_apparent_hours(),
    ];
    let field = ["gmst", "gast", "local_mean", "local_apparent"];
    for i in 0..4 {
        assert!(
            (actual_deg[i] - expected_deg[i]).abs() < 1e-12,
            "{}_deg: {} vs {}",
            field[i],
            actual_deg[i],
            expected_deg[i]
        );
        assert!(
            (actual_hours[i] - expected_deg[i] / 15.0).abs() < 1e-12,
            "{}_hours: {} vs {}",
            field[i],
            actual_hours[i],
            expected_deg[i] / 15.0
        );
    }
}

#[test]
fn gast_matches_meeus_example_12b() {
    // External-authority anchor (kills no additional mutant, by design —
    // spec §4.2): Meeus Example 12.b, 1987 April 10, 0h UT (JD 2446895.5),
    // apparent sidereal time = 13h10m46.1351s = 197.692230°. The 1e-4°
    // tolerance covers Meeus's 1980-nutation worked example vs the crate's
    // nutation model (~1e-3″ difference).
    let st = sidereal_time(
        Instant::new(JulianDay::from_days(2_446_895.5), TimeScale::Ut1),
        Longitude::from_degrees(0.0),
    );
    assert!((st.gast_deg - 197.692_230).abs() < 1e-4, "gast {}", st.gast_deg);
}
```

- [ ] **Step 2: Replace the inline module in `sidereal.rs`**

In `crates/pleiades-apparent/src/sidereal.rs`, delete the entire inline block (lines 93–184, from `#[cfg(test)]` / `mod tests {` through its closing brace) and replace it with:

```rust
#[cfg(test)]
mod tests;
```

No other change to the file.

- [ ] **Step 3: Run the crate tests**

```bash
cargo nextest run -p pleiades-apparent sidereal
```

Expected: PASS, 9 tests (7 relocated + 2 new). The two new tests must pass on unmutated code. If `sidereal_time_fields_match_independent_recomposition` fails at 1e-12, check the recomposition's operation order against production (`(gmst + ee) + lon` left-to-right) before touching tolerances.

- [ ] **Step 4: Format and lint**

```bash
cargo fmt --all
cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings
```

Expected: no diff / no warnings.

- [ ] **Step 5: Re-run the authoritative per-file mutants command**

```bash
cargo mutants -p pleiades-apparent --test-tool nextest \
  --test-workspace=false --file crates/pleiades-apparent/src/sidereal.rs
```

Expected: **46 mutants tested: 0 missed, 1 unviable** (baseline was 17 missed, 28 caught, 1 unviable). If any mutant is still missed, STOP — apply the Global Constraints' classify-then-kill-or-document rule before committing.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-apparent/src/sidereal.rs crates/pleiades-apparent/src/sidereal/tests.rs
git commit -m "test(apparent): FU-9 sidereal — recomposition pinning + Meeus 12.b anchor (17 -> 0)"
```

---

### Task 3: Docs — follow-ups progress entry, spec deliverables sync, full CI

**Files:**
- Modify: `docs/follow-ups.md` (append a Progress block inside FU-9, after the topocentric entry's "Residual audit gap" note)
- Modify: `docs/superpowers/specs/2026-07-20-fu9-sidereal-mutant-triage-design.md` (§9 Deliverables — sync commit structure)

**Interfaces:**
- Consumes: the verified results of Tasks 1–2 (both per-file re-runs at 0 missed). If either task ended with a documented residual instead of 0, adjust the numbers and add the residual description before committing.
- Produces: the FU-9 record future slices build on.

- [ ] **Step 1: Append the FU-9 progress entry**

In `docs/follow-ups.md`, insert the following block after the topocentric "Residual audit gap (future slice)" paragraph (keeping it inside the FU-9 section, before the `---` that precedes FU-10):

```markdown
**Progress (2026-07-20) — sidereal (`pleiades-apparent/src/sidereal.rs` +
`pleiades-time/src/sidereal.rs`):** triaged from `17 + 5` → `0` surviving
mutants (spec/plan:
`docs/superpowers/specs/2026-07-20-fu9-sidereal-mutant-triage-design.md`).
The first **two-file slice**: after FU-5's single-sourcing, the apparent-side
file is a thin composition layer delegating the GMST polynomial to
`pleiades-time`, so the `pleiades-time` sidereal survivors (queued in the
backlog tail) were folded in rather than re-deriving the same references
later. Both baselines confirmed by the authoritative per-file commands
(apparent: `46 tested, 17 missed, 28 caught, 1 unviable`; time: `30 tested,
5 missed, 25 caught`). **Tests-only** like `refraction.rs`/`topocentric.rs`:
the only source edits were relocating both inline test modules to
`src/sidereal/tests.rs`. Root causes: on the time side, all 5 survivors were
the small quadratic/cubic Meeus 12.4 terms invisible to a single-epoch 1e-4°
test — killed by literals evaluated outside the code at **t = ±4** Julian
centuries (JD 2597645.0 / 2305445.0, ≈ years 2400/1600, inside the project's
coverage target) at 2e-7° tolerance, with design-stage verified margins
(smallest mutant displacement 153 ulp of the raw value vs a ~27 ulp
tolerance); the ± pair separates the even quadratic term from the odd cubic
term (the aberration slice's ±1 trick at larger |t|). On the apparent side,
15 survivors were three entirely untested hours accessors and 2 were the
`gmst + lon` composition in `local_mean_deg` (the one field no test
constrained by value) — all 17 killed by a single recomposition-pinning test
at `(jd = 2446895.5, lon = +52.5°)` asserting every `_deg` field and every
hours accessor against expectations rebuilt from independently-invoked
sub-functions, plus a Meeus example 12.b GAST anchor (197.692230°, 1e-4°)
tying the composed output to a published value. **No documented residual
this slice** — like `apparent.rs` and `aberration.rs`, a genuine `0 + 0`;
no equivalent-mutant candidates surfaced. No parity gate was touched; the
tier stays report-only; `mise run ci` is green. **Remaining slices**
(priority order): `precession.rs` (17), `lighttime.rs` (5), then the
remaining `pleiades-time` (non-sidereal) and `pleiades-types` survivors.
```

- [ ] **Step 2: Sync the spec's Deliverables section**

In `docs/superpowers/specs/2026-07-20-fu9-sidereal-mutant-triage-design.md` §9, replace:

```markdown
1. Commit 1 — `test(sidereal): FU-9 mutant triage — Meeus 12.4 large-t
   literals, recomposition pinning, and 12.b anchor` (includes both test
   relocations).
2. Commit 2 — `docs(follow-ups): record FU-9 sidereal triage (17+5 → 0)`.
```

with:

```markdown
1. Commit 1 — `test(time): FU-9 sidereal — pin gmst_degrees_raw to Meeus
   12.4 literals at t = ±4 (5 -> 0)` (includes the pleiades-time test
   relocation).
2. Commit 2 — `test(apparent): FU-9 sidereal — recomposition pinning +
   Meeus 12.b anchor (17 -> 0)` (includes the pleiades-apparent test
   relocation).
3. Commit 3 — `docs(follow-ups): record FU-9 sidereal triage (17+5 -> 0)`.

(Refined during planning: one test commit per crate — each independently
testable and reviewable — instead of a single cross-crate test commit.)
```

- [ ] **Step 3: Run full blocking CI**

```bash
mise run ci
```

Expected: green (fmt, clippy, nextest, and the rest of the blocking tier). This is the whole-workspace check that the relocations broke nothing.

- [ ] **Step 4: Commit**

```bash
git add docs/follow-ups.md docs/superpowers/specs/2026-07-20-fu9-sidereal-mutant-triage-design.md
git commit -m "docs(follow-ups): record FU-9 sidereal triage (17+5 -> 0)"
```

---

## Completion

After Task 3, use superpowers:finishing-a-development-branch. Per the repo's
established FU-9 flow: push `fu9-sidereal-mutant-triage`, open a PR titled
`test(sidereal): FU-9 sidereal mutant triage (17+5 -> 0)`, watch checks go
green, then merge and delete the branch. **Never use `gh pr merge --auto` on
this repo** (it merges immediately — see memory note); wait for green checks
first.
