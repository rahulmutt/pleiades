# Close Equatorial-Branch Follow-ups Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Resolve the actionable open follow-ups from `feat/equatorial-declination-output` — unify the J2000 obliquity constant, tighten one gate test, de-tautologize one test, and fix stale docs.

**Architecture:** A batch of small, independent edits across four crates plus one tool. Each task is self-contained and ends with a passing test run. No behavioral feature change; the only numeric movement is a ~1e-9 rad precision improvement in the eclipse obliquity cache, bounded by the existing eclipse gate.

**Tech Stack:** Rust (cargo workspace), the `pleiades-*` crates.

## Global Constraints

- Shared J2000 obliquity constant is `pleiades_types::OBLIQUITY_J2000_DEG = 23.439_291_111_111_11` (re-exported from `pleiades-types`).
- Do **not** modify `crates/pleiades-eclipse/src/geometry.rs:399` (of-date polynomial), the SE ceiling constants, or the ELP raw-backend equatorial behavior.
- Do **not** rewrite the value-pinning sentinels at `crates/pleiades-apparent/src/nutation.rs:148` and `crates/pleiades-types/src/tests.rs:112` to reference the constant (keeps them as regression guards).
- Spec: `docs/superpowers/specs/2026-07-01-equatorial-branch-followups-design.md`.

---

### Task 1: Unify the J2000 obliquity literals

**Files:**
- Modify: `crates/pleiades-houses/src/systems/mod.rs:584`
- Modify: `crates/pleiades-eclipse/src/geometry.rs` (around lines 321–323)
- Test: existing suites `cargo test -p pleiades-houses`, `cargo test -p pleiades-eclipse`

**Interfaces:**
- Consumes: `pleiades_types::OBLIQUITY_J2000_DEG` (existing `pub const f64`).
- Produces: no new symbols; `OBLIQUITY_RAD` in `geometry.rs` keeps its name and type (`const f64`).

- [ ] **Step 1: Confirm both crates depend on `pleiades-types`**

Run: `grep -n "pleiades-types\|pleiades_types" crates/pleiades-houses/Cargo.toml crates/pleiades-eclipse/Cargo.toml crates/pleiades-houses/src/systems/mod.rs crates/pleiades-eclipse/src/geometry.rs`
Expected: `pleiades-types` appears as a dependency in both `Cargo.toml` files. If `pleiades_types::Instant`/`Angle` are already used in these files, the dependency is present.

- [ ] **Step 2: Replace the exact houses literal**

In `crates/pleiades-houses/src/systems/mod.rs`, inside `fn mean_obliquity`, change the polynomial's lead term from the bare literal to the shared constant:

```rust
    Angle::from_degrees(
        pleiades_types::OBLIQUITY_J2000_DEG
            - 0.013_004_166_666_666_667 * centuries
            // ...remaining polynomial terms unchanged...
```

(Use the fully-qualified path `pleiades_types::OBLIQUITY_J2000_DEG` to avoid touching imports. This is an exact-value swap — zero numeric change.)

- [ ] **Step 3: Replace the eclipse radians cache**

In `crates/pleiades-eclipse/src/geometry.rs`, add an import near the top of the file (with the other `use` statements):

```rust
use pleiades_types::OBLIQUITY_J2000_DEG;
```

Then change the constant (around line 323) from:

```rust
const OBLIQUITY_RAD: f64 = 0.409_092_804_222_329; // 23.439291°
```

to:

```rust
// J2000 mean obliquity in radians, single-sourced from pleiades_types::OBLIQUITY_J2000_DEG.
const OBLIQUITY_RAD: f64 = OBLIQUITY_J2000_DEG * core::f64::consts::PI / 180.0;
```

(`f64::to_radians` is not usable in `const` context; the multiply form keeps it a `const`. Value shifts ~1e-9 rad — a precision improvement.)

- [ ] **Step 4: Build and run both crates' tests**

Run: `cargo test -p pleiades-houses -p pleiades-eclipse`
Expected: PASS. In particular the eclipse geometry/validation tests must remain green, confirming the ~1e-9 rad change introduces no regression.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-houses/src/systems/mod.rs crates/pleiades-eclipse/src/geometry.rs
git commit -m "refactor: single-source J2000 obliquity constant in houses and eclipse"
```

---

### Task 2: Tighten the frame-consistency row-count assertion

**Files:**
- Modify: `crates/pleiades-validate/src/frame_consistency_validation.rs:293` (and add a comment near it)
- Test: `cargo test -p pleiades-validate`

**Interfaces:**
- Consumes: existing `FrameConsistencyReport { rows_validated, max_residual_lat_arcsec, summary_line }` and the existing in-loop Sun@1900 latitude sentinel (lines 236–240).
- Produces: no new symbols.

- [ ] **Step 1: Tighten the assertion and document the latitude sentinel**

In `crates/pleiades-validate/src/frame_consistency_validation.rs`, change the test assertion (line ~293):

```rust
    // Exact expected count: 8 VSOP87 bodies x 2 epochs + Moon x 1 epoch = 17.
    // A positive, non-trivial latitude is already proven inside the gate loop by
    // the Sun@1900 sentinel (this file, ~lines 236-240: |ecliptic latitude| ~= 45"),
    // so it is not re-asserted here.
    assert_eq!(
        report.rows_validated, 17,
        "unexpected row count: {}",
        report.rows_validated
    );
```

(Replaces the previous `assert!(report.rows_validated >= 17, ...)`.)

- [ ] **Step 2: Run the gate test**

Run: `cargo test -p pleiades-validate frame_consistency`
Expected: PASS with `rows_validated == 17`.

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-validate/src/frame_consistency_validation.rs
git commit -m "test: assert exact frame-consistency row count (== 17)"
```

---

### Task 3: De-tautologize the equatorial composition test

**Files:**
- Modify: `crates/pleiades-apparent/src/equatorial.rs:53–64` (`composes_rotation_with_true_obliquity`)
- Test: `cargo test -p pleiades-apparent`

**Interfaces:**
- Consumes: `apparent_equatorial_of_date(ecliptic: EclipticCoordinates, jd_tt: f64) -> Result<EquatorialCoordinates, ApparentPlaceError>` and its output fields `right_ascension_deg`, `declination_deg`, `distance_au` (names as used in the existing test).
- Produces: no new symbols. Correctness of the rotation direction remains covered by the sibling test `solstice_point_maps_to_ra90_dec_obliquity`; this test becomes a regression lock against an independent pinned value.

- [ ] **Step 1: Capture the current output as the independent pinned expectation**

Temporarily add a `dbg!` (or `eprintln!`) of the RA/Dec for the test's fixed input and run once to read the values:

```rust
// TEMPORARY — capture expected constants, then delete this block.
let e = EclipticCoordinates { longitude_deg: 123.456, latitude_deg: 1.234, distance_au: 0.987 };
let got = apparent_equatorial_of_date(e, 2_433_283.0).unwrap();
eprintln!("RA={:.9} DEC={:.9}", got.right_ascension_deg, got.declination_deg);
```

Run: `cargo test -p pleiades-apparent composes_rotation_with_true_obliquity -- --nocapture`
Expected: prints `RA=<value> DEC=<value>`. Record both numbers.

- [ ] **Step 2: Rewrite the test to assert against the pinned literals**

Replace the body of `composes_rotation_with_true_obliquity` so it no longer compares the helper to `to_equatorial(true_obliquity_degrees(jd))` (same math on both sides). Assert against the captured constants instead, and keep the exact `distance_au` preservation:

```rust
#[test]
fn composes_rotation_with_true_obliquity() {
    // Regression lock against independently pinned RA/Dec (captured once, not
    // recomputed from the function under test). Rotation-direction correctness
    // is covered separately by `solstice_point_maps_to_ra90_dec_obliquity`.
    let e = EclipticCoordinates { longitude_deg: 123.456, latitude_deg: 1.234, distance_au: 0.987 };
    let got = apparent_equatorial_of_date(e, 2_433_283.0).unwrap();

    const EXPECTED_RA_DEG: f64 = /* value from Step 1 */;
    const EXPECTED_DEC_DEG: f64 = /* value from Step 1 */;

    assert!((got.right_ascension_deg - EXPECTED_RA_DEG).abs() < 1e-6,
        "RA drifted: {} vs {}", got.right_ascension_deg, EXPECTED_RA_DEG);
    assert!((got.declination_deg - EXPECTED_DEC_DEG).abs() < 1e-6,
        "Dec drifted: {} vs {}", got.declination_deg, EXPECTED_DEC_DEG);
    assert_eq!(got.distance_au, 0.987, "distance must be preserved exactly");
}
```

Delete the temporary `eprintln!` block from Step 1. Adjust struct/field names if the existing test uses different constructors (match what is already in the file).

- [ ] **Step 3: Run the test**

Run: `cargo test -p pleiades-apparent composes_rotation_with_true_obliquity`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-apparent/src/equatorial.rs
git commit -m "test: pin independent RA/Dec in equatorial composition test"
```

---

### Task 4: Fix stale documentation

**Files:**
- Modify: `crates/pleiades-apparent/src/precession.rs` (module doc lines 1–7, struct doc line 17, field docs lines 20 and 22)
- Modify: `tools/se-equatorial-reference/src/main.rs:26`
- Test: `cargo build -p pleiades-apparent` and `cargo build -p se-equatorial-reference` (doc-only; build must still succeed)

**Interfaces:**
- Consumes: nothing new.
- Produces: nothing new (documentation only).

- [ ] **Step 1: Reframe the `PrecessedEcliptic` rustdoc**

In `crates/pleiades-apparent/src/precession.rs`, reword the field docs so they no longer unconditionally say "of date" (the struct now also carries J2000 output):

```rust
pub struct PrecessedEcliptic {
    /// Ecliptic longitude in the caller-selected frame (mean equinox of date or J2000), degrees [0, 360).
    pub longitude_deg: f64,
    /// Ecliptic latitude in the caller-selected frame (mean ecliptic of date or J2000), degrees.
    pub latitude_deg: f64,
}
```

Also soften the struct-level doc (line ~17) and module-level doc (lines 1–7) similarly: state the coordinates are referred to the frame chosen by the caller rather than unconditionally "of date". Keep wording minimal; no semantic/behavioral change.

- [ ] **Step 2: Fix the corpus-generator date comment**

In `tools/se-equatorial-reference/src/main.rs`, correct the end-epoch comment (line ~26):

```rust
const JD_END_TT: f64 = 2_488_065.5;   // 2099-12-28
```

(JD 2_488_065.5 is 2099-12-28, not 2099-12-26: 2100-01-01 00:00 TT = JD 2_488_069.5, minus 4 days.)

- [ ] **Step 3: Verify both still build**

Run: `cargo build -p pleiades-apparent && cargo build -p se-equatorial-reference`
Expected: build succeeds (doc-only changes).

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-apparent/src/precession.rs tools/se-equatorial-reference/src/main.rs
git commit -m "docs: reframe PrecessedEcliptic frame wording; fix SE corpus end-date comment"
```

---

### Task 5: Full verification and follow-up bookkeeping

**Files:**
- Modify: `docs/follow-ups.md`
- Test: `cargo build --workspace`, `cargo test --workspace`

**Interfaces:**
- Consumes: the completed Tasks 1–4.
- Produces: an updated follow-ups ledger.

- [ ] **Step 1: Workspace build and full test run**

Run: `cargo build --workspace && cargo test --workspace`
Expected: build and all tests PASS, including the eclipse validation gate and `validate-frame-consistency`.

- [ ] **Step 2: Update the follow-ups ledger**

In `docs/follow-ups.md`, in the "Open: Deferred minor findings" section, mark the resolved items with a 2026-07-01 resolution note: B3 row-count (`== 17`), B5 rustdoc, Task 1 test tautology, Task 2 (resolved-by-existing-coverage), Task 4 typo, and the ε₀ unification for the houses `584` + eclipse `323` sites. Explicitly retain as still-open: the SE global-ceiling rationale note, the ELP raw of-date equatorial note, and the deliberately-untouched `geometry.rs:399` literal (note it was left by design).

- [ ] **Step 3: Commit**

```bash
git add docs/follow-ups.md
git commit -m "docs: mark equatorial-branch follow-ups resolved (2026-07-01)"
```

---

## Self-Review

**Spec coverage:** Part A → Task 1 (houses 584 + eclipse 323; 399 excluded per Global Constraints). Part B → Task 2 (`== 17` + sentinel comment; no redundant assertion). Part C → Task 3 (de-tautology) + Task 2-note (Task 2 smoke-test resolved by existing coverage, recorded in Task 5 bookkeeping). Part D → Task 4 (B5 rustdoc + SE typo). Verification → Task 5. All spec sections mapped.

**Placeholder scan:** The only intentional fill-ins are `EXPECTED_RA_DEG`/`EXPECTED_DEC_DEG` in Task 3, which Step 1 captures via a concrete printed procedure before Step 2 pins them — this is a compute-then-pin step, not an unresolved TODO.

**Type consistency:** `OBLIQUITY_J2000_DEG` (const f64), `OBLIQUITY_RAD` (const f64), `FrameConsistencyReport.rows_validated`, and `apparent_equatorial_of_date(...) -> Result<EquatorialCoordinates, _>` with `right_ascension_deg`/`declination_deg`/`distance_au` are referenced consistently across tasks. Field/constructor names in Task 3 are flagged to match the existing file if they differ.
