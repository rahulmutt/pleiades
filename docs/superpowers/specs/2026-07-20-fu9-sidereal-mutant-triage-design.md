# FU-9 sixth slice — sidereal mutant triage (`pleiades-apparent` + `pleiades-time`)

**Date:** 2026-07-20
**Status:** design approved, pre-implementation
**Origin:** [FU-9](../../follow-ups.md) — cargo-mutants surviving-mutant triage backlog
**Baseline:** [`notes/2026-07-18-mutants-baseline.md`](notes/2026-07-18-mutants-baseline.md)
**Predecessor slices:**
[`2026-07-19-fu9-nutation-mutant-triage-design.md`](2026-07-19-fu9-nutation-mutant-triage-design.md),
[`2026-07-19-fu9-apparent-mutant-triage-design.md`](2026-07-19-fu9-apparent-mutant-triage-design.md),
[`2026-07-20-fu9-refraction-mutant-triage-design.md`](2026-07-20-fu9-refraction-mutant-triage-design.md),
[`2026-07-20-fu9-aberration-mutant-triage-design.md`](2026-07-20-fu9-aberration-mutant-triage-design.md),
[`2026-07-20-fu9-topocentric-mutant-triage-design.md`](2026-07-20-fu9-topocentric-mutant-triage-design.md)
**Target files:** `crates/pleiades-apparent/src/sidereal.rs` (17 survivors)
**and** `crates/pleiades-time/src/sidereal.rs` (5 survivors)

---

## 1. Context

FU-9 is the report-only mutation-testing backlog. Five slices are complete:
`nutation.rs` (45 → 1 documented equivalent), `apparent.rs` (10 → 0),
`refraction.rs` (37 → 3 documented equivalents), `aberration.rs` (28 → 0),
and `topocentric.rs` (27 → 3 documented equivalents). They established the
reusable method — **regenerate → classify → address → verify** — and an
independence discipline: every expected value must come from an independent
reference, never from the code's own output.

This sixth slice covers **two files in one pass**, a deliberate scope decision:
after FU-5's single-sourcing, `pleiades-apparent/src/sidereal.rs` is a thin
composition layer that delegates the GMST polynomial to
`pleiades-time/src/sidereal.rs` (`gmst_degrees_raw`). The two files' survivor
sets are killed by the same reference material (independent Meeus 12.4
literals plus recomposition), so folding `pleiades-time`'s 5 survivors —
otherwise queued in the backlog tail — into this slice avoids re-deriving the
same references twice later.

## 2. Goal & scope

**Goal:** drive both files from 17 + 5 = 22 surviving mutants to 0, or to a
small set of *documented equivalent* mutants, using tests that express intent
against independent references.

**In scope:**

- relocated, expanded white-box test suites at
  `crates/pleiades-apparent/src/sidereal/tests.rs` and
  `crates/pleiades-time/src/sidereal/tests.rs`;
- FU-9 progress notes in `docs/follow-ups.md`.

**Out of scope / non-goals:**

- any production-code change in either file (tests-only slice; the only source
  edits are the two inline `#[cfg(test)] mod tests` relocations per AGENTS.md);
- any parity-gate change (`validate-*`); the mutants tier stays **report-only**;
- `#[mutants::skip]` suppression of numeric mutants;
- the truncated GMST copy in `pleiades-eclipse/src/geometry.rs`
  (`sub_shadow_point`) — intentionally independent per FU-5's scope note, and
  not in either baseline.

## 3. Baseline

Regenerated with the method's authoritative per-file commands:

```
cargo mutants -p pleiades-apparent --test-tool nextest \
  --test-workspace=false --file crates/pleiades-apparent/src/sidereal.rs
cargo mutants -p pleiades-time --test-tool nextest \
  --test-workspace=false --file crates/pleiades-time/src/sidereal.rs
```

Results: **apparent — 46 mutants tested, 17 missed, 28 caught, 1 unviable**;
**time — 30 mutants tested, 5 missed, 25 caught.** Both exactly confirm the
whole-workspace baseline figures (17 and 5) — like `topocentric.rs`, no
reconciliation note is needed.

## 4. Survivor classification & treatment

### 4.1 `pleiades-time/src/sidereal.rs` (5)

All five live in the small terms of the Meeus eq. 12.4 GMST polynomial in
`gmst_degrees_raw`, invisible to the existing single-epoch test whose 1e-4°
tolerance exceeds every mutant's displacement at t ≈ −0.127:

| Location | Mutant | Effect at the tested epoch |
| --- | --- | --- |
| L7:21 | `-` → `/` in `(jd − 2451545)/36525` | t collapses to ~2.7e-5; quadratic/cubic terms vanish (~6e-6°) |
| L8:66 | `+` → `-` on the quadratic term | ~1.3e-5° |
| L9:9 | `-` → `+` on the cubic term | ~1e-10° |
| L9:14, L9:18 | `*` → `+` inside `t·t·t` | ~1e-9° |

**Treatment — pinned external literals at t = ±4.** Assert
`gmst_degrees_raw` at JD 2597645.0 (t = +4, ≈ year 2400) and JD 2305445.0
(t = −4, ≈ year 1600) — both inside the project's 1600–2600 coverage target —
against literals evaluated **outside the code** (scratchpad script, double
precision, from the published Meeus 12.4 coefficients), tolerance **2e-7°**.
The ± pair separates the even quadratic term from the odd cubic term — the
`aberration.rs` slice's ±1 trick, at larger |t| so the cubic term clears f64
noise. Verified margins (design-stage numeric check):

| Mutant | min displacement at t = ±4 | in ulp of the raw value (~5.27e7°) |
| --- | --- | --- |
| quadratic `+` → `-` | 1.24e-2° | ~1.7e6 ulp |
| t-scaling `-` → `/` | 6.2e-3° | ~8.3e5 ulp |
| cubic `-` → `+` | 3.3e-6° | 444 ulp |
| `t·t·t` `*` → `+` (both) | 1.14e-6° | 153 ulp |

The 2e-7° tolerance is ~27 ulp — ≥5× below the smallest mutant displacement
and ≥25× above last-ulp evaluation noise. A companion assertion ties
`gmst_degrees` to `gmst_degrees_raw(jd).rem_euclid(360.0)` at the same epochs
(its mutants are already caught; this pins the normalized path at the new
epochs).

### 4.2 `pleiades-apparent/src/sidereal.rs` (17)

Two root causes:

| Group | Location | Count | Root cause |
| --- | --- | --- | --- |
| A — hours accessors | `gast_hours` (L66 ×5), `local_mean_hours` (L70 ×5), `local_apparent_hours` (L74 ×5): `→ 0.0/1.0/−1.0`, `/` → `%`, `/` → `*` | 15 | only `gmst_hours` has a test; the other three accessors are never called |
| B — `local_mean_deg` composition | L88:35 `+` → `-`, `+` → `*` in `norm(gmst + lon)` | 2 | `local_mean_deg` is the one struct field no test constrains by value (the normalization test checks range only) |

**Treatment — one recomposition-pinning test kills all 17.** At a pinned
`(jd = 2446895.5, lon = +52.5°)`, build `SiderealTime` via `sidereal_time` and
assert **all four `_deg` fields** against expectations recomposed in-test from
independently-invoked pieces (`pleiades_time::gmst_degrees_raw`, `nutation`,
`mean_obliquity_degrees`, the `equation_of_equinoxes` helper — the
`apparent.rs` slice's independent-recomposition strategy, and the same shape
as the existing `equation_of_equinoxes_degrees_uses_shared_helper` test), then
assert **all four hours accessors** equal `expected_deg / 15.0` where
`expected_deg` comes from the recomposition, not the struct — non-circular by
construction. Tolerance ≤1e-12° (identical operations in identical order).

Geometry non-degeneracy checklist, each property load-bearing:

- **`lon = +52.5° ≠ 0`** — separates local from Greenwich fields, and
  `norm(gmst + lon) ≠ norm(gmst · lon)` / `norm(gmst − lon)` (kills group B);
- **all four deg values ∉ {0, 15, ±15·k degeneracies}** — `deg/15 ≠ deg%15`
  and `deg/15 ≠ deg·15`, and no hours value lands on 0.0/1.0/−1.0 (kills
  group A's replacement and operator mutants alike);
- **jd = 2446895.5 (Meeus ch. 12 epoch)** — nutation table available, EE
  non-zero (~−1e-3°), so mean and apparent fields genuinely differ.

**External-authority anchor (kills no additional mutant, by design).** Meeus
example 12.b: GAST at 1987-04-10.0 UT (JD 2446895.5) = 13h10m46.1351s =
**197.692230°**, asserted on `sidereal_time(...).gast_deg` at ~1e-4°
tolerance (covers the difference between Meeus's 1980-nutation worked example
and the crate's nutation model, ~1e-3″). This ties the composed GAST to a
published value rather than only to the crate's own sub-functions —
scope-approved as a deliberate redundancy.

## 5. Equivalent-mutant analysis

**No equivalent-mutant candidates in either file.** Every survivor's mutated
expression diverges from the original by an amount far above assertion
tolerance under the chosen epochs/geometry (§4 tables), and none is behind an
unreachable guard or an identical-expression collapse. **Expected residual:
0 + 0.** If a survivor unexpectedly remains on re-run, it gets the established
classify-then-kill-or-document treatment rather than suppression.

## 6. Reference strategy (independence discipline)

- **Time side:** literals from the published Meeus 12.4 coefficients evaluated
  outside the code in a scratchpad script; derivation documented in test
  comments. The proposed pinned values (script re-confirms before pinning):
  `gmst_degrees_raw(2597645.0) ≈ 52740283.547038615` and
  `gmst_degrees_raw(2305445.0) ≈ −52739722.61338802`.
- **Apparent side:** independent recomposition from separately-invoked,
  already-triaged sub-functions (`gmst_degrees_raw` is pinned to external
  literals by this same slice; `nutation` was triaged 45 → 1 in slice one),
  plus the Meeus 12.b published GAST value as an external anchor.
- **Constants note:** no crate constant is inherited by the reference material
  in this slice (the topocentric slice's residual-audit concern does not
  apply — Meeus 12.4's coefficients are themselves the published authority).

## 7. Test relocation & inventory

Per AGENTS.md, both inline `#[cfg(test)] mod tests` move to co-located test
files — `crates/pleiades-apparent/src/sidereal/tests.rs` and
`crates/pleiades-time/src/sidereal/tests.rs` — matching the five prior
slices. White-box unit tests with `use super::*`; not converted to
integration tests.

| Test | File | Carries |
| --- | --- | --- |
| existing suites (both files) | both | kept — real intent (Meeus 12.a value, normalization, cross-crate agreement, helper equivalence) the new tests do not replace |
| `gmst_raw_matches_meeus_12_4_at_large_t` | time | all 5 time-side kills: ±4-century pinned literals at 2e-7° |
| `gmst_normalized_matches_raw_at_large_t` | time | pins `gmst_degrees` ↔ `rem_euclid` at the new epochs (redundant kill) |
| `sidereal_time_fields_match_independent_recomposition` | apparent | groups A + B (17 kills): all four `_deg` fields and all four hours accessors vs recomposed expectations |
| `gast_matches_meeus_example_12b` | apparent | external-authority anchor (redundant kill, by design) |

## 8. Verification & acceptance criteria

1. Both §3 per-file commands report **0 missed**.
2. `mise run ci` is green; `cargo fmt` / clippy clean.
3. No source change in either file other than the test-module relocations.
4. No `validate-*` gate file is touched.
5. No `#[mutants::skip]` is added.

## 9. Deliverables

1. Commit 1 — `test(sidereal): FU-9 mutant triage — Meeus 12.4 large-t
   literals, recomposition pinning, and 12.b anchor` (includes both test
   relocations).
2. Commit 2 — `docs(follow-ups): record FU-9 sidereal triage (17+5 → 0)`.

Branch `fu9-sidereal-mutant-triage`, PR flow as in prior slices.

## 10. Follow-on (out of scope here, tracked in FU-9)

Remaining slices in priority order: `precession.rs` (17), `lighttime.rs` (5),
then the remaining `pleiades-time` (non-sidereal) and `pleiades-types`
survivors.
