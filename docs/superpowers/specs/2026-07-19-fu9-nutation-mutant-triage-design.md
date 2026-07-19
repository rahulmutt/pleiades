# FU-9 first slice — `nutation.rs` mutant triage

**Date:** 2026-07-19
**Status:** design approved, pre-implementation
**Origin:** [FU-9](../../follow-ups.md) — cargo-mutants surviving-mutant triage backlog
**Baseline:** [`notes/2026-07-18-mutants-baseline.md`](notes/2026-07-18-mutants-baseline.md)
**Target file:** `crates/pleiades-apparent/src/nutation.rs` (45 survivors)

---

## 1. Context

The devkit Phase 3 cargo-mutants slice established a report-only mutation-testing
tier and recorded a first baseline: **318 surviving mutants out of 1451** across
`pleiades-types`, `pleiades-time`, and `pleiades-apparent`. Survivors concentrate
in `pleiades-apparent`'s release-grade numeric code, with `nutation.rs` holding
45. A surviving mutant is a **coverage** signal, not a bug: production logic that
can be altered without any test noticing.

The baseline flagged a hypothesis "to be confirmed, not assumed" during triage:
that these survivors reflect tolerance-based parity gates masking small
perturbations rather than missing unit coverage. Inspection of `nutation.rs`
confirms the coverage-gap reading for this file. The only numeric test epoch is
Meeus Example 22.a (1987 April 10, `t ≈ 0.13`), very close to `t = 0`. Near
`t = 0` the `t²` and `t³` terms of every fundamental-argument and obliquity
polynomial are negligible, so a mutant that swaps the sign or operator of a
higher-order term perturbs the end result far below the `0.03″` assertion
tolerance — the test passes and the mutant survives. The fix is not to tighten a
release gate; it is to add white-box unit tests that exercise the polynomials
**directly** and across the model's full support range.

This is the **first slice of FU-9**. Its dual purpose is to clear `nutation.rs`
and to establish a **reusable triage method** for the remaining files.

## 2. Goal & scope

**Goal:** drive `nutation.rs` from 45 surviving mutants to **0** (or a documented,
justified residual), using tests that express the file's numeric intent, and
establish the triage method that later FU-9 files reuse.

**In scope:**

- White-box unit tests for `fundamental_arguments`, `mean_obliquity_degrees`,
  `nutation`, and the `table()` parse/validation paths.
- A minimal, behavior-preserving testability refactor (§5).
- Relocating the module's tests to a co-located test file (§6).
- Updating the FU-9 entry in `docs/follow-ups.md`.

**Non-goals:**

- Any change to other files or crates.
- Any change to `nutation.rs` runtime behavior.
- Loosening or tightening any parity/release gate.
- Wiring mutation score into any blocking gate — the tier stays **report-only**.
- Suppressing numeric survivors with blanket `#[mutants::skip]`.

## 3. The reusable method (primary deliverable)

For the target file:

1. **Regenerate** the exact survivor list in isolation — the baseline recorded a
   *sample*, not the full per-file list:

   ```
   cargo mutants -p pleiades-apparent \
     --test-tool nextest --test-workspace=false \
     --file src/nutation.rs
   ```

   This yields the authoritative `missed.txt` for the file.
2. **Classify** each survivor into an archetype (§4).
3. **Address** each with an intent-expressing test (§4, using the §4.0 reference
   strategy) or, only where a mutant is genuinely unreachable, a
   narrowly-scoped `#[mutants::skip]` carrying a one-line justification.
4. **Verify** by re-running the same `--file` command → **0 survivors**, or a
   residual list in which every entry has a written justification. Confirm the
   blocking tier (`mise run ci`) stays green.

The method — regenerate, classify, address, verify — is documented in the FU-9
follow-up entry so subsequent slices apply it unchanged.

## 4. Survivor treatment

### 4.0 Reference strategy (self-contained, independent evaluation)

Every expected value is evaluated **independently of the code under test**, from
the published authority: the Meeus / IAU-1980 polynomial coefficients and the
same 19 published rows of `data/nutation-iau1980.csv`. Reference epochs span
`t ≈ −4 … +6` (≈ 1600–2600 CE, the packaged-data support range).

This is non-circular by construction: the expected side is derived from the
published model, never from running `nutation.rs`. A mutant's operator swap makes
the code diverge from the independent evaluation, and the test fails. The
decisive property is that testing the private functions **directly** makes a
swapped `t²`/`t³` operator shift the output by arcminutes-to-degrees — orders of
magnitude above any sane tolerance — instead of relying on a faint perturbation
surviving propagation through `nutation()` and the `0.03″` end-to-end tolerance.

**Independence discipline (reviewer gate):** expected literals and test-side
polynomial evaluations must be traceable to published coefficients, *not* copied
from the code's output. The spec calls this out so review can verify it.

### 4.1 Archetype A — `fundamental_arguments` polynomial swaps (the bulk)

Direct tests asserting each element of `[D, M, M′, F, Ω]` at several epochs
(including large `|t|`) against an independent evaluation of the published Meeus
22.x polynomials, tolerance ≈ `1e-6°`. Large-`|t|` epochs make each term,
including the cubic, individually resolvable, so every per-term operator/sign
swap is caught with wide margin. Anchor one epoch to Meeus's own printed
intermediate values for the 22.a example where the book lists them.

### 4.2 Archetype B — `mean_obliquity_degrees` swaps

Direct multi-epoch tests against an independent evaluation of the Meeus 22.2
polynomial. Retain the existing J2000-anchor test
(`j2000_mean_obliquity_matches_anchor`).

### 4.3 Archetype C — `nutation()` series accumulation

Survivors in the accumulation loop (`arg_deg += mult * args[i]`, `args[i]`
indexing, `psi_a + psi_b * t`, `.sin()`/`.cos()`, the `* 0.0001` scaling).
Multi-epoch `nutation()` tests against an independent evaluation of the **same 19
published terms**, plus retention of the Meeus 22.a example. Because the
reference re-evaluates the identical truncated term set, there is no
truncation-mismatch floor — the tolerance can be tight enough to catch
accumulation-operator swaps.

### 4.4 Archetype D — `table()` parse / validation / checksum branches

Survivors in the wrong-column-count check, the non-numeric-cell parse error, and
the checksum-mismatch branch. These fire only on malformed input, which the
`include_str!` embedding prevents today. Enabled by the §5 refactor; then tested
with crafted malformed CSVs (wrong column count, non-numeric cell, and a
checksum-mismatching body) asserting the corresponding `StaleModelData` error.

### 4.5 Archetype E — `NonFiniteCorrection` guard

The `!delta_psi_arcsec.is_finite() || !delta_eps_arcsec.is_finite()` guard is
directly drivable: `nutation(f64::NAN)` (or `f64::INFINITY`) makes the
accumulation non-finite, so the function returns
`ApparentPlaceError::NonFiniteCorrection { stage: "nutation" }`. A test asserting
this kills the guard survivors — no skip required.

## 5. Minimal testability refactor

Extract a pure function from `table()`:

```rust
fn parse_table(csv: &str) -> Result<Vec<Term>, ApparentPlaceError>;
```

`parse_table` performs the checksum verification and row parsing; `table()`
becomes a thin wrapper that calls `parse_table(NUTATION_CSV)`. This is
behavior-preserving and lets Archetype D tests feed malformed input directly.
The refactor is kept as a separate, clearly-labeled change from the
test-additions, per the change-management guidance in AGENTS.md.

## 6. Test relocation

The expanded suite exceeds what belongs inline in a ~150-line source file
(AGENTS.md: keep large inline test suites out of the file under test). Relocate
the module's `#[cfg(test)] mod tests` into a co-located test file
(`crates/pleiades-apparent/src/nutation/tests.rs`), declared from `nutation.rs`
with `#[cfg(test)] mod tests;` (module-path form), preserving white-box access to
private items (`fundamental_arguments`, `parse_table`, `Term`,
`julian_centuries`). White-box unit tests stay unit tests; they are **not**
converted to black-box integration tests. This establishes the co-located-test
pattern the later FU-9 slices reuse.

## 7. Verification & acceptance criteria

- `cargo mutants -p pleiades-apparent --test-tool nextest
  --test-workspace=false --file src/nutation.rs` reports **0 surviving mutants**,
  or a residual list where every entry carries a written justification
  (`#[mutants::skip]` comment or a note in the follow-up entry).
- `mise run ci` (blocking tier) passes.
- `cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets
  --all-features -- -D warnings` clean.
- **No parity/release gate tolerance is changed** by this slice.
- `docs/follow-ups.md` FU-9 entry updated: `nutation.rs` marked done with its
  post-triage survivor count, the reusable method recorded, and the remaining
  files listed as tracked follow-on slices.

## 8. Risks & mitigations

- **Gate-tightness finding.** If a specific survivor turns out to reflect a
  genuinely under-tight *parity* gate rather than a unit-coverage gap, record it
  as a finding in the follow-up entry; do **not** silently tighten a release gate
  inside a coverage slice. Expected to be rare given the direct white-box
  approach.
- **Circular references.** Mitigated by the §4.0 independence discipline and an
  explicit reviewer check.
- **Residual unreachable survivors.** Any survivor that cannot be honestly driven
  (should be none after §4.5) is documented with justification rather than left
  silent, consistent with the "no silent caps" posture.

## 9. Deliverables

1. Expanded, relocated white-box test suite for `nutation.rs` covering
   archetypes A–E.
2. `parse_table` testability refactor.
3. Updated FU-9 follow-up entry (method + per-file status + remaining slices).
4. Verification evidence: pre/post `--file` mutant counts, blocking-tier green.

## 10. Follow-on (out of scope here, tracked in FU-9)

Remaining `pleiades-apparent` survivor files in priority order —
`apparent.rs` (49), `refraction.rs` (37), `aberration.rs` (28),
`topocentric.rs` (27), `sidereal.rs` (17), `precession.rs` (17),
`lighttime.rs` (5) — plus the `pleiades-time` and `pleiades-types` survivors,
each a subsequent slice applying this same method.
