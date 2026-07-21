# FU-9 seventh slice — precession + lighttime mutant triage (`pleiades-apparent`)

**Date:** 2026-07-21
**Status:** design approved, pre-implementation
**Origin:** [FU-9](../../follow-ups.md) — cargo-mutants surviving-mutant triage backlog
**Baseline:** [`notes/2026-07-18-mutants-baseline.md`](notes/2026-07-18-mutants-baseline.md)
**Predecessor slices:**
[`2026-07-19-fu9-nutation-mutant-triage-design.md`](2026-07-19-fu9-nutation-mutant-triage-design.md),
[`2026-07-19-fu9-apparent-mutant-triage-design.md`](2026-07-19-fu9-apparent-mutant-triage-design.md),
[`2026-07-20-fu9-refraction-mutant-triage-design.md`](2026-07-20-fu9-refraction-mutant-triage-design.md),
[`2026-07-20-fu9-aberration-mutant-triage-design.md`](2026-07-20-fu9-aberration-mutant-triage-design.md),
[`2026-07-20-fu9-topocentric-mutant-triage-design.md`](2026-07-20-fu9-topocentric-mutant-triage-design.md),
[`2026-07-20-fu9-sidereal-mutant-triage-design.md`](2026-07-20-fu9-sidereal-mutant-triage-design.md)
**Target files:** `crates/pleiades-apparent/src/precession.rs` (17 survivors)
**and** `crates/pleiades-apparent/src/lighttime.rs` (5 survivors)

---

## 1. Context

FU-9 is the report-only mutation-testing backlog. Six slices are complete:
`nutation.rs` (45 → 1 documented equivalent), `apparent.rs` (10 → 0),
`refraction.rs` (37 → 3 documented equivalents), `aberration.rs` (28 → 0),
`topocentric.rs` (27 → 3 documented equivalents), and sidereal
(17 + 5 → 0 across two crates). They established the reusable method —
**regenerate → classify → address → verify** — and an independence
discipline: every expected value must come from an independent reference,
never from the code's own output.

This seventh slice covers the last two `pleiades-apparent` files in one pass
(a two-file slice like sidereal, per scope decision): `precession.rs` and
`lighttime.rs`. Completing it finishes the `pleiades-apparent` crate's
backlog entirely.

## 2. Goal & scope

**Goal:** drive `precession.rs` from 17 surviving mutants to a documented
2-equivalent residual and `lighttime.rs` from 5 to 0, using tests that
express intent against independent references.

**In scope:**

- relocated, expanded white-box test suites at
  `crates/pleiades-apparent/src/precession/tests.rs` and
  `crates/pleiades-apparent/src/lighttime/tests.rs`;
- FU-9 progress notes in `docs/follow-ups.md`.

**Out of scope / non-goals:**

- any production-code change in either file (tests-only slice; the only
  source edits are the two inline `#[cfg(test)] mod tests` relocations per
  AGENTS.md);
- any parity-gate change (`validate-*`); the mutants tier stays
  **report-only**;
- `#[mutants::skip]` suppression of numeric mutants;
- forward-function (`precess_ecliptic_j2000_to_date`) pinned literals — its
  mutants are all caught today (219 of 238); redundant forward pinning was
  offered and declined during design (plain Approach A chosen).

## 3. Baseline

Regenerated with the method's authoritative per-file commands:

```
cargo mutants -p pleiades-apparent --test-tool nextest \
  --test-workspace=false --file crates/pleiades-apparent/src/precession.rs
cargo mutants -p pleiades-apparent --test-tool nextest \
  --test-workspace=false --file crates/pleiades-apparent/src/lighttime.rs
```

Results: **precession — 238 mutants tested, 17 missed, 219 caught, 2
unviable**; **lighttime — 14 mutants tested, 5 missed, 8 caught, 1
unviable.** Both exactly confirm the whole-workspace baseline figures (17
and 5) — like `topocentric.rs` and sidereal, no reconciliation note is
needed.

## 4. Survivor classification & treatment

### 4.1 `precession.rs` (17)

| Group | Location | Count | Root cause |
| --- | --- | --- | --- |
| A — polynomial `*` → `/` | L38–40 (ζ, z, θ in `precess_ecliptic_date_to_j2000`): both stars of each quadratic term, all three stars of each cubic term | 15 | the inverse function is only exercised by the 1900 round-trip, where t ≈ −1 and `t*t ≈ t/t` (~1e-8° displacement, under the 1e-6° tolerance); the forward function's identical mutants die at t = 0 in `identity_at_j2000`, where `/t` → NaN — but no test calls the inverse at t = 0 or at any \|t\| far from 1 |
| B — output guard `\|\|` → `&&` | L71:35 (`date_to_j2000`), L125:35 (`j2000_to_date`) | 2 | equivalent-mutant candidates — §5 |

**Treatment for group A — two tests, each killing all 15:**

1. **`date_to_j2000_matches_independent_literals_at_pm4_centuries`** — input
   (λ = 123.456°, β = 4.5°) as mean-of-date coordinates at **JD 2597645.0
   (t = +4, ≈ year 2400)** and **JD 2305445.0 (t = −4, ≈ year 1600)** — the
   sidereal slice's epochs, inside the project's 1600–2600 coverage target —
   asserted against literals evaluated **outside the code** (scratchpad
   Python reimplementation of the published Meeus 20.3/21.4/13.x pipeline;
   §6), tolerance **1e-9°** on both longitude and latitude. Proposed pinned
   values (script re-confirms against the crate to < 1e-12° before pinning):
   t = +4 → (117.860897668741°, 4.456799466404°);
   t = −4 → (129.041779511373°, 4.538180014018°).

   Per-mutant displacement at this exact geometry (design-stage numeric
   check; each cell = max of the lon/lat displacements at that epoch). All
   15 mutants × both epochs were enumerated; within each term the
   `/`-placement variants (`c/t·t`, `c·t/t`, …) evaluate to the same mutated
   value, so their rows coincide and are shown once per term:

   | Mutant (all star variants of the term) | t = +4 | t = −4 | min vs 1e-9° tolerance |
   | --- | --- | --- | --- |
   | ζ quadratic `*`→`/` (L38:41, :45) | 4.030″ | 4.044″ | ~4.0e6× |
   | ζ cubic `*`→`/` (L38:60, :64, :68) | 0.961″ | 0.964″ | ~2.7e5× |
   | z quadratic `*`→`/` (L39:38, :42) | 14.640″ | 14.633″ | ~1.5e7× |
   | z cubic `*`→`/` (L39:57, :61, :65) | 0.974″ | 0.973″ | ~2.7e5× |
   | θ quadratic `*`→`/` (L40:42, :46) | 2.994″ | 3.473″ | ~3.0e6× |
   | θ cubic `*`→`/` (L40:61, :65, :69) | 1.174″ | 1.362″ | ~3.3e5× |

   The true minimum across all 15 mutants × both epochs is **0.961″ =
   2.67e-4°** (ζ cubic at t = +4), a ~2.7e5× margin over the tolerance.

   Geometry non-degeneracy: λ = 123.456° keeps sin λ and cos λ non-zero,
   β = 4.5° ≠ 0 keeps the `tan β` term active, and no output lands near the
   0°/360° seam.

2. **`date_to_j2000_identity_at_j2000`** — the inverse-direction mirror of
   the existing forward `identity_at_j2000` test (a genuine intent gap: the
   inverse's identity property was never asserted). At t = 0 every group-A
   mutant evaluates `c/0` → ±inf → NaN, so the non-finite guard converts
   all 15 into `Err` — a redundant second kill via a different mechanism.
   Tolerance 1e-9° (numeric roundoff at t = 0 is ~1e-13°).

### 4.2 `lighttime.rs` (5)

| Group | Location | Count | Root cause |
| --- | --- | --- | --- |
| C — retarded-epoch blind spot | L72:66 `-`→`+`, `-`→`/` in `base_jd - tau`; L64:34 `<`→`>` on convergence | 3 | every existing test's `query` closure ignores the instant it is given, so no test can observe *where* the retarded epoch lands or whether iteration actually happened |
| D — exact comparison boundaries | L59:20 `>`→`>=` (cap), L64:34 `<`→`<=` (convergence) | 2 | boundaries never hit exactly; both are **reachable** in f64 (design-stage check, §6) — killable, not equivalent |

**Treatment — three tests:**

1. **`converged_position_is_queried_at_retarded_epoch`** (kills group C) — a
   query with instant-*dependent* longitude: lon(jd) = 100 + 1000 °/day ×
   (jd − base), constant distance 5 AU, at base JD 2451545.0. Converges on
   iteration 2 with τ = 5 × `LIGHT_TIME_DAYS_PER_AU` ≈ 0.0288776 days and
   ecliptic longitude 100 − 1000τ ≈ 71.122°. Asserts the longitude within
   1e-9° of `100.0 − 1000.0 * (5.0 * LIGHT_TIME_DAYS_PER_AU)` (recomputed
   in-test from the same crafted constants), `iterations == 2`, and
   `light_time_days == 5.0 * LIGHT_TIME_DAYS_PER_AU` exactly (identical
   f64 product). Kills: `-`→`+` (queries
   at base + τ → ≈ 128.88°, off 57.8°); `<`→`>` (declares convergence on
   iteration 1, returning the *unretarded* position, off 28.9°); `-`→`/`
   (queries at jd = base/τ ≈ 8.5e7 → an unrelated longitude; the actual
   mutated value and its margin are computed and recorded during
   implementation — pseudo-random mod 360, but verified, not assumed).
2. **`cap_boundary_light_time_is_accepted`** (kills `>`→`>=`) — distance
   **1731.4463361669202 AU** (0x1.b0dc90c591fc7p+10), for which
   `distance × LIGHT_TIME_DAYS_PER_AU == 10.0` **exactly** (design-stage f64
   check; the next-ulp neighbour also lands exactly). The cap's documented
   semantics are "**exceeding** this cap is treated as non-convergent", so a
   light-time exactly *at* the cap converges normally: original returns `Ok`
   with `light_time_days == 10.0`, `iterations == 2`; the `>=` mutant
   returns `NonConvergentLightTime` at step 1.
3. **`convergence_requires_strict_decrease_at_boundary`** (kills `<`→`<=`) —
   distance **8.6572316808346e-05 AU**, for which the first-iteration
   `|new_tau − tau| == 5e-7` (`CONVERGENCE_DAYS`) **exactly**. The strict
   `<` does not converge on iteration 1; convergence lands on iteration 2
   with `iterations == 2`. The `<=` mutant converges on iteration 1
   (`iterations == 1`). Redundantly re-kills `<`→`>` (which never converges
   here → `Err` instead of `Ok`).

## 5. Equivalent-mutant analysis

**Expected residual: the two precession `||`→`&&` output-guard mutants
(group B); lighttime residual 0.** Same shape as the `nutation.rs` and
`topocentric.rs` output guards, and analyzed with the topocentric slice's
**overflow lens** (finite-input overflow tested before claiming
equivalence) rather than by analogy:

- **Non-finite inputs** (`jd_tt`, λ, or β NaN/±inf) poison shared upstream
  variables — t, then ζ/z/θ, or the trig of λ/β feeding both α and δ, or
  the obliquity ε — and every one of those feeds *both* output expressions,
  so longitude and latitude go NaN together.
- **Finite-overflow route:** for huge finite `jd_tt`, the first expression
  to overflow among the angle polynomials is θ's cubic term (coefficient
  0.041833, ~2.3× ζ's 0.017998 and z's 0.018203), so every overflow window
  includes θ_r = ±inf — and `sin/cos(±inf)` = NaN lands in **both** the `b`
  (→ longitude) and `c` (→ latitude) rotation terms. The mean-obliquity
  cubic (0.001813) overflows only at still-larger \|t\|, and ε likewise
  feeds both outputs. There is no window in which exactly one output is
  poisoned.
- **The outputs themselves cannot overflow:** longitude comes from `atan2`
  (bounded) then `rem_euclid`, latitude from a clamped `asin` (bounded);
  non-finiteness can only arrive as NaN through the shared variables above.

  A latitude-only route also fails: lat is NaN iff `c` is NaN (±inf is
  clamped to ±1 → finite), and every route to a NaN `c` (θ_r, δ, or ε
  poison) makes the longitude expression NaN through the same variable.

No reachable input makes exactly one output non-finite, so the `&&` form is
behaviourally identical and both mutants are **documented equivalent — left
visible, not `#[mutants::skip]`-suppressed** (a function-level skip would
blanket-suppress the numeric mutants this tier exists to surface). The
overflow route receives a numeric confirmation during implementation
(scratch check driving `jd_tt` into the θ-overflow window and asserting both
outputs are non-finite together, i.e. the same `Err` under `||` and `&&`).

## 6. Reference strategy (independence discipline)

- **Precession:** exact literals from a scratchpad Python reimplementation
  of the *published* pipeline (Meeus 20.3 angles, 21.4 rotation, 13.x
  ecliptic↔equatorial bridges, 22.2 mean obliquity), written from the book,
  not from the code. Cross-validated two ways at design stage:
  1. against a **genuinely different published formulation** — Meeus 21.5
     elements (η, Π, p) + 21.7 direct ecliptic rotation — agreeing to
     ~1e-4″ at t = ±1 and ~3e-3″ at t = ±4 (expected truncation-order
     divergence of the IAU-1976 series), which validates the recalled
     coefficients of both paths;
  2. against the crate itself to < 1e-12° before pinning (the topocentric
     slice's cross-validation step; both implement the same published
     pipeline, so agreement is at roundoff level).
- **Lighttime:** no external ephemeris authority exists for a synthetic
  query closure; the references are **crafted-exact inputs** (the
  refraction slice's strategy): boundary distances chosen at design stage
  so the products land exactly on `10.0` and `5e-7` in f64, and a linear
  instant-dependent longitude whose expected value is computable by hand
  (100 − 1000τ). The constant `LIGHT_TIME_DAYS_PER_AU` is used as-is to
  *construct* discriminating inputs, not asserted as a value — its own
  correctness remains the parity gates' job (and the topocentric slice's
  constants-audit note already tracks constant re-derivation generally).
- **Constants note:** the precession literals inherit no crate constant —
  Meeus's published coefficients are themselves the authority; the script's
  `OBLIQUITY_J2000` is written from 23°26′21.448″ (Meeus 22.2), not read
  from `pleiades-types`.

## 7. Test relocation & inventory

Per AGENTS.md, both inline `#[cfg(test)] mod tests` move to co-located test
files — `crates/pleiades-apparent/src/precession/tests.rs` and
`crates/pleiades-apparent/src/lighttime/tests.rs` — matching the six prior
slices. White-box unit tests with `use super::*`; not converted to
integration tests.

| Test | File | Carries |
| --- | --- | --- |
| existing suites (4 + 4 tests) | both | kept — real intent (round-trip, forward identity, general-precession magnitude, off-ecliptic behaviour; convergence, missing-distance, absurd-distance cap, query-error propagation) the new tests do not replace |
| `date_to_j2000_matches_independent_literals_at_pm4_centuries` | precession | all 15 group-A kills: ±4-century pinned literals at 1e-9° |
| `date_to_j2000_identity_at_j2000` | precession | redundant group-A kill via the t = 0 NaN route; closes the inverse-identity intent gap |
| `converged_position_is_queried_at_retarded_epoch` | lighttime | group C (3 kills): instant-dependent query pins the retarded epoch, iteration count, and τ |
| `cap_boundary_light_time_is_accepted` | lighttime | `>`→`>=` kill at the exactly-representable cap boundary |
| `convergence_requires_strict_decrease_at_boundary` | lighttime | `<`→`<=` kill at the exactly-representable convergence boundary; redundant `<`→`>` re-kill |

## 8. Verification & acceptance criteria

1. The §3 precession command reports **2 missed** (exactly the two §5 guard
   mutants); the lighttime command reports **0 missed**.
2. `mise run ci` is green; `cargo fmt` / clippy clean.
3. No source change in either file other than the test-module relocations.
4. No `validate-*` gate file is touched.
5. No `#[mutants::skip]` is added.
6. The `-`→`/` mutated-longitude margin (§4.2 item 1) and the θ-overflow
   both-outputs-poisoned confirmation (§5) are numerically verified during
   implementation and recorded in the plan/commit.

## 9. Deliverables

1. Commit 1 — `test(apparent): FU-9 precession.rs mutant triage (17 -> 2
   documented equivalents)` (includes the precession test relocation).
2. Commit 2 — `test(apparent): FU-9 lighttime.rs mutant triage (5 -> 0)`
   (includes the lighttime test relocation).
3. Commit 3 — `docs(follow-ups): record FU-9 precession+lighttime triage`,
   updating the remaining-slices list to the `pleiades-time` (non-sidereal)
   and `pleiades-types` tails and noting that `pleiades-apparent` is
   complete.

Branch `fu9-precession-lighttime-mutant-triage`, PR flow as in prior slices.

## 10. Follow-on (out of scope here, tracked in FU-9)

Remaining slices after this one: the `pleiades-time` non-sidereal survivors
(`convert.rs` 16, `deltat.rs` 10, `tdb.rs` 9) and the `pleiades-types`
survivors (`zodiac.rs` 12, `time.rs` 10, and the small tail).
