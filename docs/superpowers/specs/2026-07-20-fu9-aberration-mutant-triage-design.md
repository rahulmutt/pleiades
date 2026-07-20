# FU-9 fourth slice — `aberration.rs` mutant triage

**Date:** 2026-07-20
**Status:** design approved, pre-implementation
**Origin:** [FU-9](../../follow-ups.md) — cargo-mutants surviving-mutant triage backlog
**Baseline:** [`notes/2026-07-18-mutants-baseline.md`](notes/2026-07-18-mutants-baseline.md)
**Predecessor slices:**
[`2026-07-19-fu9-nutation-mutant-triage-design.md`](2026-07-19-fu9-nutation-mutant-triage-design.md),
[`2026-07-19-fu9-apparent-mutant-triage-design.md`](2026-07-19-fu9-apparent-mutant-triage-design.md),
[`2026-07-20-fu9-refraction-mutant-triage-design.md`](2026-07-20-fu9-refraction-mutant-triage-design.md)
**Target file:** `crates/pleiades-apparent/src/aberration.rs` (28 survivors)

---

## 1. Context

FU-9 is the report-only mutation-testing backlog. Three slices are complete:
`nutation.rs` (45 → 1 documented equivalent mutant), `apparent.rs` (10 → 0), and
`refraction.rs` (37 → 3 documented equivalent mutants). They established a
reusable method — **regenerate → classify → address → verify** — and an
independence discipline: every expected value must come from an independent
reference, never from the code's own output.

This is the fourth slice, against `aberration.rs`, the top remaining file.

`aberration.rs` is a **single public formula evaluator with inlined element
polynomials**. It implements Meeus ch. 23 eq. 23.2 — annual aberration in
ecliptic coordinates — as one public function, `annual_aberration`, which
computes Earth's orbital eccentricity `e` and longitude of perihelion `ϖ` as
polynomials in Julian centuries `t` and then applies two formula lines:

```
Δλ = (-κ cos(⊙ - λ) + e κ cos(ϖ - λ)) / cos β
Δβ = -κ sin β (sin(⊙ - λ) - e sin(ϖ - λ))
```

Its survivor profile is unlike all three predecessors, and that difference
drives the whole design: the survivors split into a group that is **testable
through the public API** and a group that is **arithmetically unreachable**
through it.

## 2. Goal & scope

**Goal:** drive `aberration.rs` from 28 surviving mutants to 0, or to a small
set of *documented equivalent* mutants, using tests that express intent against
independent references.

**In scope:**

- a minimal behavior-preserving refactor extracting the element polynomials;
- a relocated, expanded white-box test suite at `src/aberration/tests.rs`;
- FU-9 progress notes in `docs/follow-ups.md`.

**Out of scope / non-goals:**

- any change to numeric results at runtime — the refactor is a pure extraction;
- any parity-gate change (`validate-*`); the mutants tier stays **report-only**;
- `#[mutants::skip]` suppression of numeric mutants (see §8).

## 3. Baseline

Regenerated with the method's authoritative per-file command:

```
cargo mutants -p pleiades-apparent --test-tool nextest \
  --test-workspace=false --file crates/pleiades-apparent/src/aberration.rs
```

Result: **56 mutants tested, 28 missed, 27 caught, 1 unviable.** This confirms
the whole-workspace baseline figure of 28 for this file.

## 4. Survivor classification & treatment

| Group | Location | Count | Root cause |
| --- | --- | --- | --- |
| A | `julian_centuries` (L19) | 5 | never exercised off `t = 0` |
| B | `e` polynomial (L36) | 5 | effect on output ~0.001″ — unreachable |
| C | `ϖ` polynomial (L37) | 6 | effect on output ~0.006″ — unreachable |
| D | Meeus formula lines (L46–47) | 12 | genuine coverage holes |

### 4.1 Group A — `julian_centuries` is untested (5 survivors)

`julian_centuries` is already a separate function with the right seam; it simply
has no test. Every existing test passes `jd_tt = 2_451_545.0`, the J2000 epoch,
for which `t = 0` — the one input that makes the whole-function replacements
`-> 0.0` indistinguishable from the real result.

Survivors: `-> 0.0`, `-> 1.0`, `-> -1.0`, `/ → %`, `- → /`.

**Treatment:** one direct test at a non-degenerate epoch. `jd_tt =
2_469_807.5` gives exactly `t = 0.5` — a value distinct from all three constant
replacements, and one where `%` (yielding `18262.5`) and the `-`/`/` swap
(yielding `≈2.76e-5`) both diverge grossly. Choosing a *half*-century rather
than a whole one is deliberate: `t = 1.0` would be indistinguishable from the
`-> 1.0` replacement.

### 4.2 Groups B & C — element polynomials are arithmetically unreachable (11 survivors)

This is the finding that shapes the slice. `e` and `ϖ` enter the output only
through the second term of each formula, `e κ cos(ϖ - λ)`, whose magnitude is
about **0.34″** — already small. Perturbing the polynomials perturbs that small
term only slightly:

- `e`'s linear coefficient is `4.2e-5` per century; a mutation of its sign or
  operator moves Δλ by roughly **0.001″**.
- `ϖ`'s linear coefficient is `1.72°` per century; a mutation there moves Δλ by
  roughly **0.006″**.

No honest end-to-end assertion can constrain values at that scale: the physical
model's own accuracy is far coarser, so a tolerance tight enough to catch these
mutants would be asserting precision the formula does not claim. The only way to
pin them without pinning the orchestrator's own output — the exact failure mode
FU-9 exists to avoid — is to give the polynomials a **direct test seam**.

**Treatment:** extract the two polynomials into one private function (§5), then
assert its output against coefficients evaluated independently from Meeus 25.4.

### 4.3 Group D — formula-line coverage holes (12 survivors)

These are reachable through the public API; they survive because the existing
assertions are too loose or test the wrong thing.

- **`/ cos_beta → * cos_beta` (46:89).** The existing quadrature test uses
  `β = 10°`, where `cos β = 0.985`; multiplying instead of dividing changes the
  result by only ~3%, well inside the test's `< 1.0″` bound.
- **`e * KAPPA → e / KAPPA` (46:66)** and **`ϖ - λ → ϖ / λ` (46:72).** Both
  perturb only the ~0.34″ second term, inside existing tolerances.
- **The `Δβ` line (47:18, 47:53, 47:69, 47:73, 47:79 — 9 mutants).** Every
  existing `Δβ` assertion uses `.abs()` against a bound, so the **sign is never
  constrained** (killing `delete -` outright), and the bounds are loose enough
  to hide the operator swaps.

**Treatment:** a discriminating crafted geometry, §6.

## 5. Refactor (behavior-preserving, separate commit)

Extract the element polynomials:

```rust
/// Earth's orbital eccentricity and longitude of perihelion ϖ (degrees),
/// of date. Meeus 25.4.
fn earth_orbit_elements(t: f64) -> (f64, f64) {
    let e = 0.016_708_634 - 0.000_042_037 * t - 0.000_000_126_7 * t * t;
    let pi_deg = 102.937_35 + 1.719_46 * t + 0.000_46 * t * t;
    (e, pi_deg)
}
```

`annual_aberration` then opens with:

```rust
let (e, pi_deg) = earth_orbit_elements(julian_centuries(jd_tt));
```

The two Meeus formula lines are untouched, as is `julian_centuries`. This is a
pure extraction: identical expressions, identical evaluation order, no runtime
result change. It lands as **its own commit**, separate from the tests, per the
AGENTS.md rule that refactors be separable from behavioral change.

## 6. Reference strategy (independence discipline)

Two independent sources; in neither case is an expected value taken from the
code's own output.

**Polynomials — published coefficients, hand-evaluated at three epochs.**
Expected `(e, ϖ)` literals are computed outside the code from Meeus 25.4 at
`t = 0, +1, -1`:

| `t` | `e` | `ϖ` (deg) |
| --- | --- | --- |
| `0` | `0.016_708_634` | `102.937_35` |
| `+1` | `0.016_666_470_3` | `104.657_27` |
| `-1` | `0.016_750_544_3` | `101.218_35` |

Three epochs are required, not one. At `t = 0` only the lead constants are
exercised. Evaluating at both `+1` and `-1` separates the linear term (which
flips sign) from the quadratic (which does not), which is what distinguishes the
`- → +` and `* → +` mutations on the `t²` terms from the linear ones.

**Formula lines — a crafted discriminating geometry.** The chosen geometry is

```
λ = 30°,  β = 60°,  ⊙ = 120°,  jd_tt = J2000 (t = 0)
```

with these properties:

- `cos β = 0.5` exactly, so `/ cos_beta` and `* cos_beta` differ by a factor of
  **4** — no tolerance can hide it;
- `⊙ - λ = 90°`, so `sin = 1` and `cos = 0`, isolating the `e κ` term in Δλ;
- `λ ≠ 0` and `λ ≠ ϖ`, which matters (below);
- `β ≠ 0`, so Δβ is non-zero and its **sign** is assertable.

Each of the 12 formula-line mutants was checked against this geometry before the
design was fixed. All 12 diverge; the **smallest** margin is `0.0267″`
(`e * sin(...) → e / sin(...)`), which a `1e-9` assertion tolerance separates by
about seven orders of magnitude.

**Rejected geometry (recorded so it is not re-proposed).** The natural-looking
choice `λ = ϖ` — which makes `cos(ϖ - λ) = 1` and cleanly isolates the κ term —
is **degenerate for Δβ**: it also makes `sin(ϖ - λ) = 0`, which annihilates the
entire `e sin(ϖ - λ)` subtraction, so the bracket-minus mutant (47:69) computes
a bit-identical result and survives. Likewise `λ = 0` must be avoided: it makes
`ϖ + λ ≡ ϖ - λ`, letting the 47:79 `- → +` mutant survive. The geometry must
avoid both degeneracies simultaneously; `λ = 30°` does.

## 7. Test relocation

Per AGENTS.md ("keep large inline test suites out of the file under test"), the
inline `#[cfg(test)] mod tests` moves to `crates/pleiades-apparent/src/aberration/tests.rs`,
matching the layout already used by `apparent/tests.rs`, `nutation/tests.rs`, and
`refraction/tests.rs`. These stay white-box unit tests with `use super::*`
access to `earth_orbit_elements` and `julian_centuries`; they are **not**
converted to integration tests.

The two existing tests are kept — they encode real physical intent
(conjunction/opposition sign and magnitude) that the new tests do not replace.

## 8. Verification & acceptance criteria

1. `cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false
   --file crates/pleiades-apparent/src/aberration.rs` reports **0 missed**, or
   only mutants documented as equivalent in `docs/follow-ups.md`.
2. `mise run ci` is green.
3. The refactor commit changes no numeric result: the pre-existing tests pass
   unmodified against it, before any new test is added.
4. No `validate-*` gate file is touched.
5. No `#[mutants::skip]` is added. Consistent with the `nutation.rs` precedent,
   a genuinely equivalent mutant is left **visible and documented** rather than
   suppressed, because a function-level skip would blanket-hide that function's
   entire numeric mutant surface.

**Expected residual: 0.** Unlike `nutation.rs` and `refraction.rs`, no
equivalent-mutant candidate has been identified here — all 28 survivors were
individually shown to be killable. Any survivor that does appear will be
documented rather than suppressed.

## 9. Deliverables

1. Commit 1 — `refactor(aberration): extract earth_orbit_elements` (no result change).
2. Commit 2 — `test(aberration): white-box tests for elements, julian_centuries, and Meeus 23.2`,
   including the relocation to `src/aberration/tests.rs`.
3. Commit 3 — `docs(follow-ups): record FU-9 aberration.rs triage`.

## 10. Follow-on (out of scope here, tracked in FU-9)

Remaining slices in priority order: `topocentric.rs` (27), `sidereal.rs` (17),
`precession.rs` (17), `lighttime.rs` (5), then the `pleiades-time` and
`pleiades-types` survivors.
