# FU-9 third slice — `refraction.rs` mutant triage

**Date:** 2026-07-20
**Status:** design approved, pre-implementation
**Origin:** [FU-9](../../follow-ups.md) — cargo-mutants surviving-mutant triage backlog
**Baseline:** [`notes/2026-07-18-mutants-baseline.md`](notes/2026-07-18-mutants-baseline.md)
**Predecessor slices:**
[`2026-07-19-fu9-nutation-mutant-triage-design.md`](2026-07-19-fu9-nutation-mutant-triage-design.md),
[`2026-07-19-fu9-apparent-mutant-triage-design.md`](2026-07-19-fu9-apparent-mutant-triage-design.md)
**Target file:** `crates/pleiades-apparent/src/refraction.rs` (37 survivors)

---

## 1. Context

FU-9 is the report-only mutation-testing backlog. Two slices are complete:
`nutation.rs` (45 → 1 documented equivalent mutant) and `apparent.rs` (10 → 0).
They established a reusable method — **regenerate → classify → address →
verify** — and an independence discipline: every expected value must come from
an independent reference, never from the code's own output.

This is the third slice, against `refraction.rs`, the top remaining file.

`refraction.rs` differs from both predecessors. `nutation.rs` was a
**polynomial evaluator** (survivors killed against published IAU-1980
coefficients). `apparent.rs` was an **orchestrator** (survivors killed by
independent recomposition, after a testability refactor). `refraction.rs` is a
**pair of small pure formula evaluators plus a branch-heavy blend model**:
Bennett (1982) and Saemundsson (1986), each pressure/temperature scaled, wrapped
in a hold-then-fade below-horizon blend with three regions per direction.

Its survivors are therefore dominated not by subtle arithmetic masking but by a
plain **coverage hole**: one whole function is untested, and the tested
directions are pinned only in regions where the mutated and unmutated values
happen to agree within a loose tolerance.

No testability refactor is needed this slice — the file is already decomposed
into small pure functions with exactly the right seams.

## 2. Goal & scope

**Goal:** drive `refraction.rs` from 37 surviving mutants to **0 killable
survivors**, with a small documented residual of provably equivalent mutants
(expected ~3, §5).

**In scope:**

- White-box unit tests covering groups A–D (§4).
- Relocating the module's tests to a co-located test file (§6).
- Updating the FU-9 entry in `docs/follow-ups.md`.

**Non-goals:**

- **Any change to `refraction.rs` runtime source.** This slice is tests-only
  (Approach A, §3): no refactor, no formula edit, no constant change. The single
  permitted source edit is the one-line `#[cfg(test)] mod tests;` declaration
  required by the §6 relocation.
- Any change to other files or crates. `lighttime.rs` (5 survivors) was
  explicitly considered and left queued as its own slice, preserving the
  one-file-per-slice cadence.
- Loosening or tightening any parity/release gate, including the existing SE
  corpus tolerances in `refraction_matches_se_below_horizon`.
- Wiring mutation score into any blocking gate — the tier stays **report-only**.
- Suppressing survivors with `#[mutants::skip]`. Equivalent mutants are left
  visible and documented, per the `nutation.rs` precedent.

## 3. Approach

Three approaches were weighed:

- **A. Tests-only, document equivalent residuals.** *(chosen)*
- **B. Tests plus a micro-refactor** removing the equivalent-mutant sites
  (Saemundsson's `* 1.0`, the duplicated boundary dispatch). Rejected: it edits
  release-grade numeric source purely to satisfy the mutation tool, and the
  `* 1.0` mirrors the published coefficient in the formula the rustdoc cites
  (`R = 1.0 / tan(...)`), so removing it costs spec-to-code readability.
- **C. Property/round-trip tests** (monotonicity, seam continuity,
  Bennett↔Saemundsson round-trip). Rejected as the primary strategy: round-trips
  are insensitive to symmetric errors and several operator mutants preserve
  monotonicity, so it would not reliably reach 0. Nothing here precludes adding
  such tests later on their own merits.

A is consistent with both predecessor slices and with AGENTS.md's
minimal-change guidance.

## 4. Survivor classification & treatment

Authoritative per-file run at design time
(`cargo mutants -p pleiades-apparent --test-tool nextest
--test-workspace=false --file refraction.rs`):
**101 mutants tested in 2m — 37 missed, 64 caught.** This matches the baseline's
recorded 37 for the file.

### 4.1 Group A — `true_from_apparent_below_horizon` is entirely untested (20 survivors)

Every mutant in this function survives, including the three whole-function
replacements (`-> 0.0`, `-> 1.0`, `-> -1.0`). The existing corpus test exercises
only the `apparent_from_true` direction; nothing calls the apparent→true
below-horizon path at all.

**Treatment:** mirror the coverage the other direction gets, at three pinned
altitudes chosen to isolate each region:

- `h = -1.0` exactly — the `>=` boundary, full Saemundsson applied.
- `h = -10.0` and one deeper (`h = -20.0`) — the `<=` identity region, asserting
  `true_from_apparent(h) == h` exactly.
- `h = -5.5` — mid-fade, where `fade` is exactly `0.5` (§5), pinning the
  anchor-scaling and the subtraction sign.

Together these kill the whole-function stubs, both comparison mutants, the
Saemundsson application operators, the fade-slope arithmetic, and the
`h - anchor * fade` operators.

### 4.2 Group B — blend-region gaps in `apparent_from_true_below_horizon` (6 survivors)

The committed SE corpus rows reach only `h <= -9.96`, where `fade ≈ 0.004` and
the blend contributes ≈ 9 arcsec — below that row's 15 arcsec tolerance. So the
blend's slope and its `h ∈ [-1, 0)` sub-branch are unconstrained.

**Treatment:** add points the corpus misses — `h = -0.5` (the `>= -1` branch,
full Bennett), `h = -1.0` (the boundary), and `h = -5.5` (mid-fade, `fade =
0.5`) — each asserted against an independently computed literal at tight
tolerance (~1e-9 deg). The existing corpus rows are left untouched.

### 4.3 Group C — loose-tolerance formula survivors (8 survivors)

Four distinct causes, all masked by wide assertion bands in the current tests:

- `scale -> 1.0`: the default atmosphere's scale is `0.9858`, within every
  current tolerance of `1.0`.
- `replace * with / in scale`: shifts the result ~3.5%.
- `replace * with / in bennett_refraction_arcmin`: turns `scale * 1.02` into
  `scale / 1.02`, ~4% ≈ 68 arcsec — inside the existing ~180 arcsec band.
- `delete -` on `BELOW_HORIZON_BLEND_START_DEG` / `_END_DEG` (i.e. `-1.0 → 1.0`,
  `-10.0 → 10.0`): unconstrained because no test evaluates the blend where those
  constants determine the answer.

**Treatment:** direct white-box tests on the private functions using
crafted-exact atmospheres (§5) with tight literal assertions. The two constant
mutants are killed by the exact-boundary and midpoint tests from groups A and B,
which fail immediately if either endpoint moves.

### 4.4 Group D — likely equivalent mutants (3 survivors)

- `replace * with / in saemundsson_refraction_arcmin`: the operand is the
  literal `1.0`, so `scale * 1.0` and `scale / 1.0` are bit-identical for every
  input, including non-finite ones.
- `replace < with <=` in `apparent_from_true` (line 134) and the same in
  `true_from_apparent` (line 147): these differ only at exactly `h = 0.0`, and
  at that value both paths evaluate the identical expression — the
  below-horizon helper's first branch (`h >= -1.0`) applies the same formula the
  `h >= 0` path applies.

Note the sibling mutants at line 147 (`< with ==`, `< with >`) are **not**
equivalent — they redirect ordinary above-horizon inputs into the wrong branch —
and are killed by the group A/B/C tests.

**Treatment:** documented residuals, with the justification recorded both as a
comment in the test file and in the FU-9 follow-up entry. No `#[mutants::skip]`:
a function-level skip would blanket-suppress that function's genuine numeric
mutants, exactly as rejected in the nutation slice.

**These equivalence claims are verified against the post-implementation re-run,
not assumed.** Any that turns out killable (e.g. a `-0.0` subtlety at the
dispatch boundary) is killed instead of documented.

## 5. Reference strategy (independence discipline)

Expected values come from three independent sources, never from
`refraction.rs`'s own output:

1. **Crafted-exact inputs.** Atmospheres constructed so `scale` is exact by
   arithmetic rather than by measurement — `(1010 mbar, 10 °C)` gives
   `(1010/1010) * (283/283) = 1.0` exactly, `(2020 mbar, 10 °C)` gives `2.0`.
   At least one case has both factors ≠ 1 and mutually distinct, so no operator
   swap in `scale` aliases another. Fade midpoints are chosen so `fade` is an
   exact binary fraction: at `h = -5.5`, `fade = (-5.5 + 10)/(-1 + 10) = 0.5`.
2. **Independently evaluated literals.** Bennett and Saemundsson values at the
   chosen altitudes are computed outside the code from the published formulas
   (already cited in the module rustdoc, matching SE `swe_refrac` conventions)
   and pinned at ~1e-9 deg. This mirrors the nutation slice's "published
   coefficients evaluated outside the code."
3. **The existing SE corpus rows**, unchanged, continuing to pin the deep
   identity region.

**On the blend model's authority:** the hold-then-fade blend is repo-invented
(SE's own below-horizon model is discontinuous and was deliberately not
reproduced — see the function's rustdoc). Its reference is therefore its own
documented specification: anchor = Bennett at −1.0, linear fade to zero at −10.0.
The tests pin that specification — anchor value, linearity via the midpoint, and
both endpoints — which is what makes a slope or sign mutant fail.

**Reviewer gate:** every expected literal must trace to a crafted input or an
outside-the-code evaluation. None may be captured from a run of the function
under test.

## 6. Test relocation

Per AGENTS.md ("keep large inline test suites out of the file under test") and
both predecessor slices, relocate `#[cfg(test)] mod tests` into a co-located
`crates/pleiades-apparent/src/refraction/tests.rs`, declared from
`refraction.rs` as `#[cfg(test)] mod tests;`. This preserves white-box access to
the private helpers (`scale`, `bennett_refraction_arcmin`,
`saemundsson_refraction_arcmin`, both `*_below_horizon` functions) and to the
blend constants. Existing tests move unchanged; they stay white-box unit tests
and are **not** converted to black-box integration tests.

## 7. Verification & acceptance criteria

- `cargo mutants -p pleiades-apparent --test-tool nextest
  --test-workspace=false --file refraction.rs` reports **0 missed**, or only the
  documented equivalent mutants from §4.4, each carrying a written
  justification.
- `mise run ci` (blocking tier) passes.
- `cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets
  --all-features -- -D warnings` clean.
- **`refraction.rs` source is unmodified** apart from the one-line
  `#[cfg(test)] mod tests;` declaration required by §6.
- **No parity/release gate tolerance is changed**, including the existing SE
  corpus row tolerances.
- `docs/follow-ups.md` FU-9 entry updated: `refraction.rs` marked done with its
  post-triage survivor count and any documented residual, and the remaining
  files re-listed as tracked follow-on slices.

## 8. Risks & mitigations

- **A documented "equivalent" is actually killable.** Mitigated by re-verifying
  each equivalence claim against the post-implementation re-run rather than
  assuming it; anything killable gets killed.
- **Circular references.** Mitigated by the §5 independence discipline and an
  explicit reviewer check on every literal.
- **Pinning the blend model makes future changes rigid.** Accepted and
  intentional: the blend is a deliberate design decision documented at length in
  the source, and pinning it is the point — a future change to it should be a
  conscious edit that updates these tests, not a silent drift. Tests reference
  the named constants where practical rather than re-hardcoding −1.0/−10.0.
- **Gate-tightness finding.** If a survivor reflects a genuinely under-tight
  *parity* gate rather than a unit-coverage gap, record it as a finding in the
  follow-up entry; do **not** silently tighten a release gate inside a coverage
  slice.

## 9. Deliverables

1. Relocated, expanded white-box test suite for `refraction.rs` covering
   groups A–C, with documented equivalence justifications for group D.
2. Updated FU-9 follow-up entry (per-file status + remaining slices).
3. Verification evidence: pre/post `--file` mutant counts, blocking-tier green.

## 10. Follow-on (out of scope here, tracked in FU-9)

Remaining `pleiades-apparent` survivor files in priority order —
`aberration.rs` (28), `topocentric.rs` (27), `sidereal.rs` (17),
`precession.rs` (17), `lighttime.rs` (5) — plus the `pleiades-time` and
`pleiades-types` survivors, each a subsequent slice applying this same method.
