# FU-9 second slice — `apparent.rs` mutant triage

**Date:** 2026-07-19
**Status:** design approved, pre-implementation
**Origin:** [FU-9](../../follow-ups.md) — cargo-mutants surviving-mutant triage backlog
**Baseline:** [`notes/2026-07-18-mutants-baseline.md`](notes/2026-07-18-mutants-baseline.md)
**Predecessor slice:** [`2026-07-19-fu9-nutation-mutant-triage-design.md`](2026-07-19-fu9-nutation-mutant-triage-design.md)
**Target file:** `crates/pleiades-apparent/src/apparent.rs` (49 survivors — the largest single-file backlog)

---

## 1. Context

FU-9 is the report-only mutation-testing backlog. Its first slice cleared
`nutation.rs` (45 → 1 documented equivalent mutant) and established a reusable
method: **regenerate → classify → address → verify**. This is the second slice,
against `apparent.rs`, the top file at **49** survivors.

`apparent.rs` differs in kind from `nutation.rs`. Nutation is a **polynomial
evaluator**: its survivors were arithmetic-operator swaps inside series
evaluation, killed by testing the private polynomials directly against
independently-evaluated published IAU-1980 coefficients. `apparent.rs` is an
**orchestrator**: it composes already-tested sub-corrections — light-time,
precession, nutation Δψ, annual aberration — into an apparent ecliptic-of-date
position with provenance. Its survivors therefore live in the **combine logic**:
the operators wiring the sub-corrections together, the `/3600` arcsec scaling,
the `rem_euclid(360)` normalization, the precession-shift wrap block, the
per-function provenance assignments, and the non-finite guards.

The method transfers unchanged; the **reference strategy** adapts (§4.0). The
existing `sun_applies_aberration_once_no_light_time_requery` test already
demonstrates the target pattern — it hand-recomposes the expected longitude from
precession + aberration + nutation and asserts equality at `< 1e-6″`. This slice
generalizes that discipline across all combine paths.

The baseline's tolerance-masking hypothesis holds here too: the three existing
end-to-end tests assert loose bounds (`< 40″`, `< 1″`, `abs() < 0.02°`) evaluated
at or near J2000, where precession ≈ identity and the higher-order terms are
negligible — so operator swaps in the combine survive under the slack. The fix is
white-box unit tests on the combine primitives, not a gate change.

## 2. Goal & scope

**Goal:** drive `apparent.rs` from 49 surviving mutants to **0** (or a
documented, justified residual), using tests that express the file's numeric and
provenance intent.

**In scope:**

- A minimal, behavior-preserving testability refactor extracting two combine
  primitives (§5), kept as a separate labeled commit.
- White-box unit tests covering archetypes A–F (§4).
- Relocating the module's tests to a co-located test file (§6).
- Updating the FU-9 entry in `docs/follow-ups.md`.

**Non-goals:**

- Any change to other files or crates.
- Any change to `apparent.rs` **runtime behavior** — the refactor is a pure
  extraction; the three public functions compute byte-identical results.
- Loosening or tightening any parity/release gate.
- Wiring mutation score into any blocking gate — the tier stays **report-only**.
- Suppressing numeric survivors with blanket `#[mutants::skip]`.
- Deduplicating anything beyond the two combine primitives (broader structural
  cleanup of the three near-identical functions is not chased here).

## 3. The reusable method (applied)

1. **Regenerate** the authoritative per-file survivor list — the baseline
   recorded only a total, not the `apparent.rs` line list:

   ```
   cargo mutants -p pleiades-apparent \
     --test-tool nextest --test-workspace=false \
     --file src/apparent.rs
   ```

   Run this **after** the §5 refactor lands, so the survivor list reflects the
   extracted primitives (the surface tests will target) rather than the
   pre-refactor triplicated copies.
2. **Classify** each survivor into an archetype (§4).
3. **Address** each with an intent-expressing test (§4) or, only where a mutant
   is genuinely unreachable, a documented residual with a one-line justification.
4. **Verify** by re-running the same `--file` command → **0 survivors**, or a
   residual list in which every entry has a written justification. Confirm the
   blocking tier (`mise run ci`) stays green.

## 4. Survivor treatment

### 4.0 Reference strategy (orchestrator: independent recomposition)

The independent reference is **hand-recomposition from the already-tested
sub-corrections**, asserted at tight tolerance (`~1e-9`), never comparing the
orchestrator to its own output. This is non-circular by construction: the
expected side independently specifies *how* `apparent.rs` must combine the
sub-corrections — the operators, the `/3600` scaling, the `rem_euclid`, the wrap
— so a mutant that swaps `+`↔`−`, mis-scales, drops a term, or swaps which term
feeds which slot diverges from the recomposition and the test fails.

For the two extracted combine primitives (§5), the reference is even more direct:
they are pure numeric functions of their inputs, so expected values are computed
by hand from crafted inputs, independent of any sub-correction model at all.

**Independence discipline (reviewer gate):** expected literals and test-side
recompositions must be traceable to crafted inputs or to independently-invoked
sub-correction functions, *not* copied from `apparent.rs`'s output.

### 4.1 Archetype A — `combine_apparent` operators & scaling (the bulk)

Direct unit tests feeding crafted `(lambda, beta, d_lambda_arcsec,
d_beta_arcsec, delta_psi_arcsec)` and asserting the exact returned `(lon, lat)`.
Because the primitive is pure, a swapped operator (`+`↔`−`), a swapped scale
(`/`↔`*`, `3600` mutated), a dropped term, or a wrong-slot term shifts the output
far above a `1e-9` tolerance and is caught with wide margin. Include at least one
case with all of `Δλ`, `Δβ`, `Δψ` non-zero and mutually distinct so no
term-swap aliases another.

### 4.2 Archetype B — `rem_euclid(360)` normalization

Cases where `lambda + corrections` exceeds `360` and where it falls below `0`,
asserting the result lands in `[0, 360)`. Kills removal of the normalization and
constant swaps of the `360.0` modulus.

### 4.3 Archetype C — `precession_shift_arcsec` wrap branches

Direct tests with inputs whose raw `lambda − lambda_j2000` exceeds `+180`
(triggering `−360`) and falls below `−180` (triggering `+360`) — a body near the
0°/360° boundary whose precessed longitude crosses it — plus a mid-range case
that takes neither branch. Asserts each branch's arcsec output independently,
killing the `> 180` / `< −180` comparison swaps, the `±360` constant/sign
mutants, and the `× 3600` scaling.

### 4.4 Archetype D — non-finite guards

Drive a non-finite combine (e.g. the `query` closure returns a NaN-bearing
coordinate for `apparent_position`, or a NaN-bearing input coordinate for the
sun/apsis functions) and assert the returned
`ApparentPlaceError::NonFiniteCorrection { stage }` carries the correct
per-call-site `stage` string (`"apparent-combine"`, `"apparent-sun-combine"`,
`"apparent-apsis-combine"`). With the guard folded into `combine_apparent`
(§5), the `!lon.is_finite() || !lat.is_finite()` operator survivors are killed by
a single primitive-level test plus the per-function stage-string assertions.

### 4.5 Archetype E — provenance & `CorrectionSet`

Assert the **full** `ApparentProvenance` for each of the three functions:
`light_time_days`, `iterations`, `precession_longitude_arcsec`,
`nutation_longitude_arcsec`, `aberration_longitude_arcsec`, the passed-through
`distance_au`, and every `corrections` boolean. The three functions differ
deliberately — `apparent_position` has `light_time = true`; `apparent_sun_position`
has `light_time = false` with `annual_aberration = true`; `apparent_apsis_position`
has `annual_aberration = false` — so per-function full-provenance assertions kill
field-swap and boolean-flip mutants that a shared assertion would miss.

### 4.6 Archetype F — the `⊙ = λ` aberration argument

`apparent_sun_position` and (implicitly) the apsis path call
`annual_aberration(lambda, beta, lambda, jd)` — the Sun/apse is its own
aberration argument. A mutant passing a different argument is caught by the
exact-recomposition equality test (§4.0), which invokes `annual_aberration` with
the same `⊙ = λ` convention independently.

## 5. Minimal testability refactor (separate commit, behavior-preserving)

Extract two pure helpers from the three public functions:

```rust
/// Combine mean-of-date (λ, β) with arcsec corrections into an apparent
/// (lon, lat) in degrees, failing closed on non-finite output.
fn combine_apparent(
    lambda_deg: f64,
    beta_deg: f64,
    d_lambda_arcsec: f64,
    d_beta_arcsec: f64,
    delta_psi_arcsec: f64,
    stage: &'static str,
) -> Result<(f64, f64), ApparentPlaceError>;

/// Longitude precession shift for provenance, wrapped to (−180, 180], in arcsec.
fn precession_shift_arcsec(lambda_deg: f64, lambda_j2000_deg: f64) -> f64;
```

`combine_apparent` performs `(λ + (Δλ + Δψ)/3600).rem_euclid(360)`,
`β + Δβ/3600`, and the `!lon.is_finite() || !lat.is_finite()` guard emitting
`NonFiniteCorrection { stage }`. The apsis path calls it with `d_lambda_arcsec =
d_beta_arcsec = 0.0` (nutation-only), reproducing its current `apparent_lat =
beta` and `apparent_lon = (lambda + Δψ/3600).rem_euclid(360)` exactly.

`apparent_position` maps the helper's `ApparentPlaceError` through
`ApparentLightTimeError::Apparent`; the sun/apsis functions return it directly.

This is behavior-preserving: each public function computes the identical result.
It concentrates the archetype A–D survivor surface (combine operators, `/3600`
scaling, `rem_euclid`, finite guard, wrap branches) into two small pure functions
tested once, instead of three copies tested three times. The refactor is a
**separate, clearly-labeled commit** landed before the test additions, per the
change-management guidance in AGENTS.md, and the §3 survivor list is regenerated
against the post-refactor file.

## 6. Test relocation

Following the pattern the nutation slice established, relocate the module's
`#[cfg(test)] mod tests` into a co-located test file
(`crates/pleiades-apparent/src/apparent/tests.rs`), declared from `apparent.rs`
with `#[cfg(test)] mod tests;` (module-path form), preserving white-box access to
the private helpers (`combine_apparent`, `precession_shift_arcsec`) and to the
public functions under test. The existing white-box tests stay white-box unit
tests; they are **not** converted to black-box integration tests.

## 7. Verification & acceptance criteria

- `cargo mutants -p pleiades-apparent --test-tool nextest
  --test-workspace=false --file src/apparent.rs` reports **0 surviving mutants**,
  or a residual list where every entry carries a written justification
  (`#[mutants::skip]` comment or a note in the follow-up entry).
- `mise run ci` (blocking tier) passes.
- `cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets
  --all-features -- -D warnings` clean.
- **No parity/release gate tolerance is changed** by this slice.
- The refactor commit and the test commit are separate and individually
  reviewable; the refactor changes no runtime result.
- `docs/follow-ups.md` FU-9 entry updated: `apparent.rs` marked done with its
  post-triage survivor count, and the remaining files re-listed as tracked
  follow-on slices.

## 8. Risks & mitigations

- **Refactor changes behavior.** Mitigated by keeping the extraction pure and
  behavior-preserving, landing it as a separate commit, and re-running the full
  blocking tier plus the release parity gates that already exercise these
  functions (`validate-apparent`, and the equatorial/eclipse/lilith gates
  downstream) before adding tests — any accidental behavior change surfaces
  there.
- **Gate-tightness finding.** If a specific survivor reflects a genuinely
  under-tight *parity* gate rather than a unit-coverage gap, record it as a
  finding in the follow-up entry; do **not** silently tighten a release gate
  inside a coverage slice.
- **Circular references.** Mitigated by the §4.0 independence discipline and an
  explicit reviewer check.
- **Residual unreachable survivors.** Any survivor that cannot be honestly driven
  (e.g. the `DEFAULT_MAX_ITERATIONS` default-const value, if no reachable
  convergence test distinguishes it) is documented with justification rather than
  left silent, consistent with the "no silent caps" posture and the nutation
  slice's equivalent-mutant handling.

## 9. Deliverables

1. `combine_apparent` + `precession_shift_arcsec` testability refactor
   (separate, behavior-preserving commit).
2. Expanded, relocated white-box test suite for `apparent.rs` covering
   archetypes A–F.
3. Updated FU-9 follow-up entry (per-file status + remaining slices).
4. Verification evidence: pre/post `--file` mutant counts, blocking-tier green.

## 10. Follow-on (out of scope here, tracked in FU-9)

Remaining `pleiades-apparent` survivor files in priority order —
`refraction.rs` (37), `aberration.rs` (28), `topocentric.rs` (27),
`sidereal.rs` (17), `precession.rs` (17), `lighttime.rs` (5) — plus the
`pleiades-time` and `pleiades-types` survivors, each a subsequent slice applying
this same method.
