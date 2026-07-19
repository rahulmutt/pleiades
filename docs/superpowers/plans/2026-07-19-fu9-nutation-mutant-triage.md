# FU-9 nutation.rs Mutant Triage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive `crates/pleiades-apparent/src/nutation.rs` from 45 surviving cargo-mutants to 0 (or a documented, justified residual) by adding white-box unit tests that express the file's numeric intent.

**Architecture:** Add direct white-box unit tests for the private numeric functions (`fundamental_arguments`, `julian_centuries`, `mean_obliquity_degrees`, `nutation`) asserting against independently-derived reference values (published Meeus/IAU-1980 coefficients evaluated outside the code) at epochs spanning the full support range (`t ≈ −4 … +6`), where higher-order polynomial-term mutations are large. Make the CSV parse/validation branches reachable via a small behavior-preserving `parse_table(csv, expected_checksum)` extraction. Relocate the module's tests to a co-located `nutation/tests.rs`.

**Tech Stack:** Rust (edition per workspace), `cargo nextest`, `cargo-mutants` 27.1.0 (pinned via mise), `mise` task runner.

## Global Constraints

- Reference values MUST be derived from the published Meeus/IAU-1980 model, NOT copied from the code's output. (Spec §4.0 independence discipline.)
- No change to `nutation.rs` runtime behavior; the `parse_table` extraction is behavior-preserving.
- No parity/release-gate tolerance is changed by this work.
- Mutation testing stays report-only; nothing here wires it into a blocking gate.
- No blanket `#[mutants::skip]` on numeric survivors; a skip is allowed only for a genuinely unreachable mutant, with a one-line justification. (Expected: none needed.)
- White-box unit tests stay unit tests (co-located, `use super::*`); do not convert to black-box integration tests.
- All first-party crates keep the `pleiades-*` prefix; pure-Rust, no new runtime dependencies.
- Commands are run from the repo root `/workspace`.

**Reference literals (independent, published-coefficient evaluation; cross-checked against Meeus Example 22.a):**

| Epoch | `t` | `jd_tt` |
| --- | ---: | ---: |
| ~1600 CE | −4.0 | 2305445.0 |
| J2000 | 0.0 | 2451545.0 |
| ~2600 CE | +6.0 | 2670695.0 |
| Meeus 22.a (1987-04-10) | −0.1272963723 | 2446895.5 |

`fundamental_arguments(t)` → `[D, M, M1, F, Om]` degrees (raw, unreduced):

- `t = -4.0`: `[-1780770.6265249772, -143638.6759914666, -1908660.3685945775, -1932714.8573575574, 7861.6225545778]`
- `t = 6.0`: `[2671900.4514687983, 216351.8232692000, 2863328.4843071997, 2899305.2452280060, -11479.6980172000]`

`mean_obliquity_degrees(jd)` degrees:

- `jd = 2305445.0` → `23.491272924444`
- `jd = 2451545.0` → `23.439291111111` (existing anchor)
- `jd = 2670695.0` → `23.361368991111`

`nutation(jd)` → `(delta_psi_arcsec, delta_eps_arcsec)`:

- `jd = 2305445.0` → `(14.6519806449, 4.1982981873)`
- `jd = 2670695.0` → `(-10.4999306884, 6.4064419652)`
- `jd = 2446895.5` → `(-3.7893272394, 9.4412217751)` (existing Meeus example, book: −3.788 / +9.443)

`julian_centuries(jd)`:

- `julian_centuries(2305445.0) == -4.0` (exact)
- `julian_centuries(2451545.0) == 0.0` (exact)
- `julian_centuries(2670695.0) == 6.0` (exact)

---

### Task 1: Relocate the module's tests to a co-located file and capture the pre-state

**Files:**
- Modify: `crates/pleiades-apparent/src/nutation.rs` (replace inline `#[cfg(test)] mod tests { … }` with `#[cfg(test)] mod tests;`)
- Create: `crates/pleiades-apparent/src/nutation/tests.rs` (moved content)

**Interfaces:**
- Consumes: nothing.
- Produces: co-located test module `crates/pleiades-apparent/src/nutation/tests.rs` with `use super::*;` white-box access to all `nutation.rs` items. Later tasks add tests to this file.

- [ ] **Step 1: Capture the authoritative pre-triage survivor list**

Run:
```bash
cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file src/nutation.rs 2>&1 | tail -5
```
Expected: a summary line reporting surviving (MISSED) mutants for `nutation.rs` (baseline recorded ~45). Save the run's `mutants.out/missed.txt` aside for reference (it is git-ignored / not committed):
```bash
cp mutants.out/missed.txt /tmp/claude-1000/-workspace/nutation-missed-before.txt
wc -l /tmp/claude-1000/-workspace/nutation-missed-before.txt
```
Expected: a non-zero line count (~45).

- [ ] **Step 2: Create the co-located test file by moving the existing `mod tests` body**

Create `crates/pleiades-apparent/src/nutation/tests.rs` with exactly the current inline test body:
```rust
use super::*;

#[test]
fn pinned_checksum() {
    assert_eq!(
        fnv1a64(NUTATION_CSV),
        NUTATION_CSV_CHECKSUM,
        "checksum = {}",
        fnv1a64(NUTATION_CSV)
    );
}

#[test]
fn meeus_example_22a() {
    // Meeus Example 22.a: 1987 April 10, 0h TD -> JDE 2446895.5.
    // Δψ = -3.788", Δε = +9.443", ε0 = 23°26'27.407" = 23.4409463°.
    let n = nutation(2_446_895.5).unwrap();
    assert!(
        (n.delta_psi_arcsec - (-3.788)).abs() < 0.03,
        "Δψ = {}",
        n.delta_psi_arcsec
    );
    assert!(
        (n.delta_eps_arcsec - 9.443).abs() < 0.03,
        "Δε = {}",
        n.delta_eps_arcsec
    );
    let eps0 = mean_obliquity_degrees(2_446_895.5);
    assert!((eps0 - 23.440946).abs() < 1e-5, "ε0 = {eps0}");
}

#[test]
fn j2000_mean_obliquity_matches_anchor() {
    // At J2000 (t=0) the mean obliquity is the anchor constant used elsewhere.
    assert!((mean_obliquity_degrees(2_451_545.0) - 23.439_291_111_111_11).abs() < 1e-9);
}
```

- [ ] **Step 3: Replace the inline test module in `nutation.rs` with a module declaration**

In `crates/pleiades-apparent/src/nutation.rs`, delete the entire inline `#[cfg(test)] mod tests { … }` block (currently lines ~112–150) and replace it with:
```rust
#[cfg(test)]
mod tests;
```

- [ ] **Step 4: Run the tests to verify the relocation preserved them**

Run:
```bash
cargo nextest run -p pleiades-apparent nutation
```
Expected: PASS — `pinned_checksum`, `meeus_example_22a`, `j2000_mean_obliquity_matches_anchor` all run and pass.

- [ ] **Step 5: Verify formatting and lints are clean**

Run:
```bash
cargo fmt --all --check && cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings
```
Expected: no output / exit 0.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-apparent/src/nutation.rs crates/pleiades-apparent/src/nutation/tests.rs
git commit -m "refactor(apparent): relocate nutation tests to co-located tests.rs"
```

---

### Task 2: White-box tests for `fundamental_arguments` (archetype A)

**Files:**
- Modify: `crates/pleiades-apparent/src/nutation/tests.rs`

**Interfaces:**
- Consumes: private `fn fundamental_arguments(t: f64) -> [f64; 5]` (order `[D, M, M1, F, Om]`, raw degrees).
- Produces: tests `fundamental_arguments_matches_published_polynomials_large_t`.

- [ ] **Step 1: Add the direct multi-epoch test**

Append to `crates/pleiades-apparent/src/nutation/tests.rs`:
```rust
/// `fundamental_arguments` must reproduce the published Meeus 22.x polynomials
/// exactly. Reference values are an independent evaluation of those published
/// polynomials (NOT captured from this code) at large |t|, where every term —
/// including the cubic — is individually resolvable, so any per-term operator
/// or sign swap moves the result far above the 1e-6° tolerance.
#[test]
fn fundamental_arguments_matches_published_polynomials_large_t() {
    // t = -4.0 (~1600 CE)
    let a = fundamental_arguments(-4.0);
    let expected_m4 = [
        -1_780_770.626_524_977_2,
        -143_638.675_991_466_6,
        -1_908_660.368_594_577_5,
        -1_932_714.857_357_557_4,
        7_861.622_554_577_8,
    ];
    for (i, (got, want)) in a.iter().zip(expected_m4.iter()).enumerate() {
        assert!((got - want).abs() < 1e-6, "t=-4 arg[{i}] = {got}, want {want}");
    }

    // t = +6.0 (~2600 CE)
    let b = fundamental_arguments(6.0);
    let expected_p6 = [
        2_671_900.451_468_798_3,
        216_351.823_269_200_0,
        2_863_328.484_307_199_7,
        2_899_305.245_228_006_0,
        -11_479.698_017_200_0,
    ];
    for (i, (got, want)) in b.iter().zip(expected_p6.iter()).enumerate() {
        assert!((got - want).abs() < 1e-6, "t=+6 arg[{i}] = {got}, want {want}");
    }
}
```

- [ ] **Step 2: Run the test to verify it passes on the correct code**

Run:
```bash
cargo nextest run -p pleiades-apparent fundamental_arguments_matches_published_polynomials_large_t
```
Expected: PASS. (If it fails, the literals or the code disagree — stop and reconcile before proceeding; do NOT loosen the tolerance to force a pass.)

- [ ] **Step 3: Confirm the test kills the `fundamental_arguments` survivors**

Run:
```bash
cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file src/nutation.rs 2>&1 | tail -3
```
Expected: the MISSED count is lower than Task 1 Step 1, with the `fundamental_arguments` line-69–73 arithmetic survivors no longer listed in `mutants.out/missed.txt`.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-apparent/src/nutation/tests.rs
git commit -m "test(apparent): pin nutation fundamental_arguments against published polynomials"
```

---

### Task 3: White-box tests for `julian_centuries` and `mean_obliquity_degrees` (archetype B)

**Files:**
- Modify: `crates/pleiades-apparent/src/nutation/tests.rs`

**Interfaces:**
- Consumes: private `fn julian_centuries(jd_tt: f64) -> f64`; public `fn mean_obliquity_degrees(jd_tt: f64) -> f64`.
- Produces: tests `julian_centuries_maps_anchor_epochs_exactly`, `mean_obliquity_matches_published_polynomial_across_range`.

- [ ] **Step 1: Add the `julian_centuries` exact-anchor test**

Append to `crates/pleiades-apparent/src/nutation/tests.rs`:
```rust
/// `julian_centuries` is the exact TT-centuries-since-J2000 map. The chosen
/// epochs are exact integer multiples of the Julian century, so the expected
/// values are exact — any operator swap in `(jd - 2451545.0) / 36525.0` diverges.
#[test]
fn julian_centuries_maps_anchor_epochs_exactly() {
    assert_eq!(julian_centuries(2_305_445.0), -4.0);
    assert_eq!(julian_centuries(2_451_545.0), 0.0);
    assert_eq!(julian_centuries(2_670_695.0), 6.0);
}
```

- [ ] **Step 2: Add the `mean_obliquity_degrees` multi-epoch test**

Append to `crates/pleiades-apparent/src/nutation/tests.rs`:
```rust
/// `mean_obliquity_degrees` must reproduce the published Meeus 22.2 polynomial.
/// Reference values are an independent evaluation of that polynomial at large
/// |t|; the J2000 anchor is retained by `j2000_mean_obliquity_matches_anchor`.
#[test]
fn mean_obliquity_matches_published_polynomial_across_range() {
    // jd = 2305445.0 (t = -4)
    assert!(
        (mean_obliquity_degrees(2_305_445.0) - 23.491_272_924_444).abs() < 1e-8,
        "eps(t=-4) = {}",
        mean_obliquity_degrees(2_305_445.0)
    );
    // jd = 2670695.0 (t = +6)
    assert!(
        (mean_obliquity_degrees(2_670_695.0) - 23.361_368_991_111).abs() < 1e-8,
        "eps(t=+6) = {}",
        mean_obliquity_degrees(2_670_695.0)
    );
}
```

- [ ] **Step 3: Run the tests to verify they pass**

Run:
```bash
cargo nextest run -p pleiades-apparent 'julian_centuries_maps_anchor_epochs_exactly|mean_obliquity_matches_published_polynomial_across_range'
```
Expected: PASS for both. (Do not loosen tolerances to force a pass.)

- [ ] **Step 4: Confirm the survivors dropped**

Run:
```bash
cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file src/nutation.rs 2>&1 | tail -3
```
Expected: MISSED count lower again; `julian_centuries` and `mean_obliquity_degrees` polynomial survivors gone from `mutants.out/missed.txt`.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-apparent/src/nutation/tests.rs
git commit -m "test(apparent): pin julian_centuries and mean_obliquity across the support range"
```

---

### Task 4: White-box tests for the `nutation` series accumulation (archetype C)

**Files:**
- Modify: `crates/pleiades-apparent/src/nutation/tests.rs`

**Interfaces:**
- Consumes: public `fn nutation(jd_tt: f64) -> Result<Nutation, ApparentPlaceError>`, `Nutation { delta_psi_arcsec, delta_eps_arcsec }`.
- Produces: test `nutation_series_matches_independent_term_sum_across_range`.

- [ ] **Step 1: Add the multi-epoch series test**

Append to `crates/pleiades-apparent/src/nutation/tests.rs`:
```rust
/// `nutation` sums the 19 published IAU-1980 terms. Reference values are an
/// independent evaluation of those same 19 published rows (NOT captured from
/// this code) at large |t|, where the `psi_b * t` / `eps_d * t` rate terms are
/// amplified — so a swapped operator in the accumulation, argument reduction,
/// or the 0.0001 scaling diverges above the 1e-6" tolerance. The Meeus 22.a
/// example (near t=0) is retained by `meeus_example_22a`.
#[test]
fn nutation_series_matches_independent_term_sum_across_range() {
    // jd = 2305445.0 (t = -4)
    let a = nutation(2_305_445.0).unwrap();
    assert!(
        (a.delta_psi_arcsec - 14.651_980_644_9).abs() < 1e-6,
        "Δψ(t=-4) = {}",
        a.delta_psi_arcsec
    );
    assert!(
        (a.delta_eps_arcsec - 4.198_298_187_3).abs() < 1e-6,
        "Δε(t=-4) = {}",
        a.delta_eps_arcsec
    );

    // jd = 2670695.0 (t = +6)
    let b = nutation(2_670_695.0).unwrap();
    assert!(
        (b.delta_psi_arcsec - (-10.499_930_688_4)).abs() < 1e-6,
        "Δψ(t=+6) = {}",
        b.delta_psi_arcsec
    );
    assert!(
        (b.delta_eps_arcsec - 6.406_441_965_2).abs() < 1e-6,
        "Δε(t=+6) = {}",
        b.delta_eps_arcsec
    );
}
```

- [ ] **Step 2: Run the test to verify it passes**

Run:
```bash
cargo nextest run -p pleiades-apparent nutation_series_matches_independent_term_sum_across_range
```
Expected: PASS. (If it fails by a small margin only at these tight tolerances, first confirm the literals match a fresh independent evaluation; loosen ONLY toward 1e-5" with a comment if a genuine cross-libm `sin`/`cos` bit difference is demonstrated — never past a tolerance that would let a real term swap survive.)

- [ ] **Step 3: Confirm the accumulation survivors dropped**

Run:
```bash
cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file src/nutation.rs 2>&1 | tail -3
```
Expected: MISSED count lower; the `nutation` loop/accumulation survivors (lines ~92–102) gone or reduced to a documented tail.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-apparent/src/nutation/tests.rs
git commit -m "test(apparent): pin nutation series accumulation across the support range"
```

---

### Task 5: Extract `parse_table` and test the parse/validation/checksum branches (archetype D)

**Files:**
- Modify: `crates/pleiades-apparent/src/nutation.rs` (extract `parse_table`)
- Modify: `crates/pleiades-apparent/src/nutation/tests.rs` (add branch tests)

**Interfaces:**
- Consumes: `fnv1a64`, `NUTATION_CSV`, `NUTATION_CSV_CHECKSUM`, `Term`, `ApparentPlaceError::StaleModelData { kind }`.
- Produces: private `fn parse_table(csv: &str, expected_checksum: u64) -> Result<Vec<Term>, ApparentPlaceError>`; `table()` becomes a thin wrapper. Tests `parse_table_rejects_checksum_mismatch`, `parse_table_rejects_wrong_column_count`, `parse_table_rejects_non_numeric_cell`, `parse_table_accepts_embedded_table`.

- [ ] **Step 1: Refactor `table()` to delegate to a pure, checksum-parameterized `parse_table`**

In `crates/pleiades-apparent/src/nutation.rs`, replace the current `fn table() -> Result<Vec<Term>, ApparentPlaceError> { … }` (lines ~34–61) with:
```rust
fn table() -> Result<Vec<Term>, ApparentPlaceError> {
    parse_table(NUTATION_CSV, NUTATION_CSV_CHECKSUM)
}

/// Parse the IAU-1980 nutation term table from `csv`, rejecting input whose
/// FNV-1a checksum does not match `expected_checksum`. Taking the checksum as a
/// parameter (rather than reading the module constant) keeps the function pure
/// and lets tests exercise the malformed-input branches with a matching
/// checksum for the crafted input.
fn parse_table(csv: &str, expected_checksum: u64) -> Result<Vec<Term>, ApparentPlaceError> {
    if fnv1a64(csv) != expected_checksum {
        return Err(ApparentPlaceError::StaleModelData { kind: "nutation" });
    }
    let mut terms = Vec::new();
    for line in csv.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<f64> = line
            .split(',')
            .map(|s| s.trim().parse::<f64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ApparentPlaceError::StaleModelData { kind: "nutation" })?;
        if cols.len() != 9 {
            return Err(ApparentPlaceError::StaleModelData { kind: "nutation" });
        }
        terms.push(Term {
            multipliers: [cols[0], cols[1], cols[2], cols[3], cols[4]],
            psi_a: cols[5],
            psi_b: cols[6],
            eps_c: cols[7],
            eps_d: cols[8],
        });
    }
    Ok(terms)
}
```

- [ ] **Step 2: Verify the refactor is behavior-preserving**

Run:
```bash
cargo nextest run -p pleiades-apparent nutation
```
Expected: PASS — all existing nutation tests still pass (the embedded table parses identically through the wrapper).

- [ ] **Step 3: Add the branch tests**

Append to `crates/pleiades-apparent/src/nutation/tests.rs`:
```rust
#[test]
fn parse_table_accepts_embedded_table() {
    // The wrapper's own input must parse to the full 19-term table.
    let terms = parse_table(NUTATION_CSV, NUTATION_CSV_CHECKSUM).unwrap();
    assert_eq!(terms.len(), 19);
}

#[test]
fn parse_table_rejects_checksum_mismatch() {
    // Any body that does not hash to the expected checksum is refused before parsing.
    let err = parse_table("not the real table", NUTATION_CSV_CHECKSUM).unwrap_err();
    assert!(matches!(
        err,
        ApparentPlaceError::StaleModelData { kind: "nutation" }
    ));
}

#[test]
fn parse_table_rejects_wrong_column_count() {
    // Passes the checksum (computed for this crafted body) but has 3 columns, not 9.
    let body = "h1,h2,h3\n1,2,3";
    let err = parse_table(body, fnv1a64(body)).unwrap_err();
    assert!(matches!(
        err,
        ApparentPlaceError::StaleModelData { kind: "nutation" }
    ));
}

#[test]
fn parse_table_rejects_non_numeric_cell() {
    // Passes the checksum but a data cell is non-numeric.
    let body = "D,M,M1,F,Om,psi_a,psi_b,eps_c,eps_d\n0,0,0,0,1,-171996,-174.2,92025,NOPE";
    let err = parse_table(body, fnv1a64(body)).unwrap_err();
    assert!(matches!(
        err,
        ApparentPlaceError::StaleModelData { kind: "nutation" }
    ));
}
```

- [ ] **Step 4: Run the branch tests**

Run:
```bash
cargo nextest run -p pleiades-apparent parse_table
```
Expected: PASS for all four.

- [ ] **Step 5: Verify formatting, lints, and confirm the D survivors dropped**

Run:
```bash
cargo fmt --all --check && cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings
cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file src/nutation.rs 2>&1 | tail -3
```
Expected: lints clean; MISSED count lower; the `!= 9` column-count and parse/checksum branch survivors gone from `mutants.out/missed.txt`.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-apparent/src/nutation.rs crates/pleiades-apparent/src/nutation/tests.rs
git commit -m "test(apparent): extract parse_table and cover nutation table validation branches"
```

---

### Task 6: Drive the non-finite guard (archetype E)

**Files:**
- Modify: `crates/pleiades-apparent/src/nutation/tests.rs`

**Interfaces:**
- Consumes: `fn nutation(jd_tt: f64) -> Result<Nutation, ApparentPlaceError>`, `ApparentPlaceError::NonFiniteCorrection { stage }`.
- Produces: test `nutation_returns_typed_error_on_non_finite_input`.

- [ ] **Step 1: Add the guard test**

Append to `crates/pleiades-apparent/src/nutation/tests.rs`:
```rust
/// A non-finite input drives the accumulation non-finite, and the engine must
/// return a typed error rather than propagating NaN. This kills the survivors on
/// the `is_finite` guard by making the guarded branch reachable.
#[test]
fn nutation_returns_typed_error_on_non_finite_input() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let err = nutation(bad).unwrap_err();
        assert!(
            matches!(err, ApparentPlaceError::NonFiniteCorrection { stage: "nutation" }),
            "input {bad} produced {err:?}"
        );
    }
}
```

- [ ] **Step 2: Run the test**

Run:
```bash
cargo nextest run -p pleiades-apparent nutation_returns_typed_error_on_non_finite_input
```
Expected: PASS. (If any input returns `Ok`, stop — the guard is not doing what the test asserts; investigate before adjusting.)

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-apparent/src/nutation/tests.rs
git commit -m "test(apparent): assert nutation fails closed on non-finite input"
```

---

### Task 7: Verify zero survivors, update FU-9, and confirm the blocking tier

**Files:**
- Modify: `docs/follow-ups.md` (FU-9 entry)

**Interfaces:**
- Consumes: the completed test suite from Tasks 1–6.
- Produces: updated FU-9 status; final verification evidence.

- [ ] **Step 1: Regenerate the survivor list and record the post-state**

Run:
```bash
cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file src/nutation.rs 2>&1 | tail -5
cp mutants.out/missed.txt /tmp/claude-1000/-workspace/nutation-missed-after.txt
wc -l /tmp/claude-1000/-workspace/nutation-missed-after.txt
```
Expected: 0 surviving mutants (empty `missed.txt`). If any survive, inspect each: add a targeted test if drivable, or — only if genuinely unreachable — add a narrowly-scoped `#[mutants::skip]` in `nutation.rs` with a one-line justification comment, then re-run this step. Record the final residual (should be 0) and its justifications.

- [ ] **Step 2: Run the full blocking tier**

Run:
```bash
mise run ci
```
Expected: PASS (fmt, clippy `-D warnings`, and the workspace test tier all green).

- [ ] **Step 3: Update the FU-9 follow-up entry**

In `docs/follow-ups.md`, under FU-9, append a progress note recording:
- `nutation.rs` triaged: `<before>` → `0` surviving mutants (cite the Task 1 vs Task 7 counts).
- The reusable method: regenerate per-file list with `cargo mutants … --file <path>` → classify into archetypes (polynomial, accumulation, parse/validation, guard) → add intent-expressing white-box tests referencing published/independent values (never the code's own output) → verify 0 survivors.
- Remaining files as tracked follow-on slices, in priority order: `apparent.rs` (49), `refraction.rs` (37), `aberration.rs` (28), `topocentric.rs` (27), `sidereal.rs` (17), `precession.rs` (17), `lighttime.rs` (5), then the `pleiades-time` and `pleiades-types` survivors.

Use this exact block (fill in the two counts from the runs):
```markdown

**Progress (2026-07-19) — `pleiades-apparent/src/nutation.rs`:** triaged from
`<BEFORE>` → `0` surviving mutants by adding intent-expressing white-box unit
tests (spec/plan:
`docs/superpowers/specs/2026-07-19-fu9-nutation-mutant-triage-design.md`).
**Reusable method** for the remaining files: regenerate the per-file survivor
list with `cargo mutants -p <crate> --test-tool nextest --test-workspace=false
--file <path>`; classify each survivor as polynomial, series-accumulation,
parse/validation, or guard; add a white-box test asserting against an
*independent* reference (published coefficients evaluated outside the code, or a
crafted-input branch), never the code's own output; re-run `--file` to confirm
0. No parity gate was touched; the tier stays report-only. **Remaining slices**
(priority order): `apparent.rs` (49), `refraction.rs` (37), `aberration.rs`
(28), `topocentric.rs` (27), `sidereal.rs` (17), `precession.rs` (17),
`lighttime.rs` (5), then the `pleiades-time` and `pleiades-types` survivors.
```

- [ ] **Step 4: Commit**

```bash
git add docs/follow-ups.md
git commit -m "docs(follow-ups): record FU-9 nutation.rs triage complete and the reusable method"
```

---

## Self-Review

**Spec coverage:**
- Spec §2 scope (nutation.rs only, no behavior/gate change) → enforced by Global Constraints and Tasks 1–7. ✓
- Spec §3 method (regenerate → classify → address → verify) → Task 1 Step 1 (regenerate), Tasks 2–6 (classify+address), Task 7 (verify). ✓
- Spec §4.0 reference strategy (independent, published) → Global Constraints + literal table + per-test doc comments. ✓
- Spec §4.1 archetype A → Task 2. ✓
- Spec §4.2 archetype B → Task 3. ✓
- Spec §4.3 archetype C → Task 4. ✓
- Spec §4.4 archetype D → Task 5. ✓
- Spec §4.5 archetype E → Task 6. ✓
- Spec §5 parse_table extraction → Task 5 Step 1 (refined to take `expected_checksum` param, documented). ✓
- Spec §6 test relocation → Task 1. ✓
- Spec §7 acceptance (0 survivors, ci green, fmt/clippy, no gate change, follow-ups updated) → Task 7 + fmt/clippy steps in Tasks 1 and 5. ✓
- Spec §10 follow-on list → Task 7 Step 3. ✓

**Placeholder scan:** No TBD/TODO/"handle edge cases"; every code step shows complete code; every command has expected output. ✓

**Type consistency:** `parse_table(csv, expected_checksum)` used consistently in Task 5 impl and tests; `fundamental_arguments` order `[D, M, M1, F, Om]` consistent; `ApparentPlaceError::StaleModelData { kind }` and `NonFiniteCorrection { stage }` match the source; `Nutation { delta_psi_arcsec, delta_eps_arcsec }` field names match. ✓
