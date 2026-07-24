# FU-9 slice — `pleiades-houses` mutant triage (design)

**Status:** design approved · **Opened:** 2026-07-22 · **Follow-up:** FU-9 (new
post-baseline slice) · **Crate:** `pleiades-houses`

## Context

FU-9's original three-crate measured baseline (`pleiades-types`,
`pleiades-time`, `pleiades-apparent`) is **CLOSED** — every file reaches `0`
surviving mutants or a documented equivalent (see `docs/follow-ups.md`, FU-9).
The closing note frames any `mise run mutants` expansion to `pleiades-*`
domain/backend crates outside the original three as **new work, opening a new
slice under FU-9** — not part of the closed baseline. This is the first such
expansion slice.

`pleiades-houses` is the release-grade house-system layer directly above the
just-completed `pleiades-apparent`. It is pure-logic (no large generated data
tables to inflate the mutant surface) and is guarded by two dedicated parity
gates — `validate-houses` (cusp corpus) and `validate-angles` (Asc/MC/armc/gast
geometry) — which are the intended safety net for its numeric paths. Its dense
trig/interpolation logic is exactly the "arithmetic-operator swap in numeric
evaluation" survivor class FU-9 targets.

## Goal & scope

Drive surviving mutants in `pleiades-houses` to **0-or-documented-equivalent**,
measured by the authoritative per-file cargo-mutants command, using
intent-expressing white-box tests referenced to independent authorities.

### Measured baseline (2026-07-22)

Whole-crate `cargo mutants -p pleiades-houses --test-tool nextest
--test-workspace=false --baseline run` at `7572b234c`, cargo-mutants 27.1.0:
**1,231 mutants, 569 missed / 638 caught / 24 unviable** (27 min wall-clock,
test suite itself 0.06 s — the run is compile-bound). Survivors by file:

| File | Survivors | Dominant classes |
|------|-----------|------------------|
| `systems/mod.rs` | **554** | 372 arith-op swaps, 72 comparison swaps, 36 `/`→`%`, 28 match-arm deletes, spread across ~20 house-formula functions + shared primitives + 2 solvers |
| `catalog/mod.rs` | 15 | 4 `||`→`&&` guards, 1 match-arm delete, 3 return-value replacements, 4 `+=`→`*=`, 3 `vec![]`/return-value |
| `thresholds.rs` | 0 | its 1 mutant is **caught**; only the AGENTS.md test relocation remains |

`554` in one file is ~15× the previous largest slice (refraction's 37), so
`systems/mod.rs` is a **multi-PR campaign, not a single PR**. Crucially the
effort is **per-system, not per-survivor**: one independent-recomputation test
pinning all 12 cusps at a discriminating geometry kills a whole system's
40–70 arith-op survivors at once, so the real work is ~20 units grouped by
formula-family.

### Delivery — a ~6-PR family-grouped campaign (one design doc)

All PRs share **one** independent house-math reference (a Python
reimplementation of the published SE `swehouse.c` / Meeus ch. 20 pipeline,
reproduced in the plan and cross-validated against the crate at ~machine
precision) established by the Foundation PR and reused thereafter.

| PR | Focus | Functions (survivors) |
|----|-------|-----------------------|
| **1 — Foundation** | Shared primitives + angles + trivial/Porphyry family | `spherical_cotrans` (34), `asc2` (16), `asc1` (12), `asc_mc_from` (22), `interpolate_longitude` (6), `signed_longitude_difference` (3), RA/ecliptic transforms (1), `longitude_opposite`/`longitude_in_arc` (2), `porphyry_houses` (16), `whole_sign_houses` (1) — pinning the shared primitives once kills survivors across every composing system |
| **2 — Great-circle** | GreatCircle family | `apc_sector` (58), `krusinski_pisa_goelzer_houses` (19), `horizon_houses` (12), `apc_houses` (1) |
| **3 — Sector** | Sector family | `pullen_sr_houses` (73), `pullen_sd_houses` (42), `albategnius_houses` (42), `solve_gauquelin_sector` (4) + `gauquelin_houses` |
| **4 — Sunshine/solar-arc** | SolarArc family | `sunshine_houses` (68), `sunshine_offsets` (34), `apparent_solar_declination` (20), `apparent_midheaven_declination` (2), `nutation_for` (2) |
| **5 — Quadrant/projection** | Quadrant + EquatorialProjection families | `solve_placidian_cusp` (6), `topocentric_latitude` (9), `regiomontanus_houses` (5), `koch_houses` (1), campanus/alcabitius/morinus/carter residuals |
| **6 — Catalog + thresholds** | Non-numeric tail; **crate-completing PR** | `catalog/mod.rs` (15) + `catalog_name` match arms in `systems/mod.rs` (26) + `thresholds.rs` test relocation |

PR ordering is Foundation-first (it establishes the shared reference every later
PR builds on); PRs 2–5 are independent of one another and may be sequenced in
any order; PR 6 lands last and enables the weekly-tier expansion. Exact survivor
membership per PR is confirmed against the measured `mutants.out/missed.txt` at
the start of each PR's plan.

### In scope

- `crates/pleiades-houses/src/systems/mod.rs`
- `crates/pleiades-houses/src/catalog/mod.rs`
- `crates/pleiades-houses/src/thresholds.rs`

### Out of scope (negligible mutable surface)

- `lib.rs` — re-exports and doctests only; no branch/arithmetic surface. (Its
  doctests still run under `mise run ci` and are not weakened.)
- `error.rs` — a small error enum + `Display`; if the per-crate baseline
  surfaces a survivor here it is folded into the crate-completing PR (PR 6), but
  it is not a planned focus.

### Non-goals

- **No production behavior change**, except behavior-preserving testability
  refactors (each its own separate commit, no runtime-result change, per the
  `apparent.rs` / `aberration.rs` precedent). None is anticipated for the
  already-decomposed `catalog`/`thresholds` files; a shared-seam extraction in
  `systems/mod.rs` is possible only if a survivor is otherwise unreachable
  through the public API.
- **No parity-gate change.** `validate-houses` and `validate-angles` corpora,
  tolerances, and code are untouched. This slice adds *unit* coverage, not gate
  coverage.
- The mutants tier **stays report-only**. No mutation-score gate is introduced.

## Method (established reusable method)

Unchanged from every prior FU-9 slice:

1. **Authoritative per-file baseline:**
   `cargo mutants -p pleiades-houses --test-tool nextest --test-workspace=false --file <crate-relative path>`.
   Record `N tested, M missed, K caught` for each file. (A whole-crate
   `cargo mutants -p pleiades-houses --test-tool nextest --test-workspace=false`
   run confirms the aggregate before and after.)
2. **Classify** each survivor: trig/polynomial-eval, iterative-solver
   boundary, validation guard, match-arm, string-render, or
   documented-equivalent guard.
3. **Add white-box tests asserting against an *independent* reference** —
   published coefficients/formulas evaluated outside the code, an independent
   reimplementation cross-validated against the crate, or a crafted-input
   branch. **Never** assert against the code's own output.
4. **Re-run `--file`** to confirm the residual is `0` or a documented
   equivalent mutant.

## Reference strategy (per survivor class)

`pleiades-houses` spans two distinct classes and needs a **mix** keyed to each.

### Numeric — `systems/mod.rs`

- **Shared trig primitives** — `asc1`, `asc2`, `spherical_cotrans`,
  `right_ascension_from_ecliptic_longitude`, `ecliptic_longitude_from_ra`,
  `interpolate_longitude`, `midpoint_longitude`, `signed_longitude_difference`,
  `normalize_degrees`: **crafted-exact-geometry pins** (aberration/topocentric
  precedent). Geometries are chosen to avoid degeneracies that let a mutant
  survive bit-identically — no `cos = 1`, `sin = 0`, or equal-angle collapse on
  the term under test. These primitives are composed by most quadrant/
  projection systems, so pinning them exactly kills survivors across many
  callers at once.
- **Per-system top-level formulas** — Placidus, Koch, Campanus, Regiomontanus,
  Alcabitius, Topocentric, APC, Krusinski-Pisa-Goelzer, Horizon, Sunshine,
  Pullen SD/SR, Gauquelin, Carter, Morinus, Meridian/Axial: **independent
  recomputation.** A Python reimplementation of the published SE
  (`swehouse.c`) / Meeus ch. 20 house pipeline, reproduced in the
  implementation plan, cross-validated against the crate at ~machine precision,
  pins exact literals at **one discriminating non-degenerate geometry per
  system**. Rejected (degenerate) geometries are recorded in the plan so they
  are not re-proposed, per the topocentric-slice discipline.
- **Trivial family** — Equal, EqualMidheaven, EqualAries, Vehlow, Whole-Sign,
  Porphyry, Sripati: pinned by **hand arithmetic** on Asc/MC (30° increments,
  quadrant trisection); no script needed.
- **Iterative solvers** — `solve_placidian_cusp`, `solve_gauquelin_sector`:
  **convergence-threshold and iteration-count boundaries** via crafted-exact
  `f64` inputs landing on the cap/threshold (lighttime precedent), each
  representability-checked with an in-test precondition assert. The solver's
  *result* is additionally pinned by the independent recomputation above.

### Guards — `systems/mod.rs`

- `validate_observer`, `validate_topocentric_observer`, `validate_obliquity`,
  `validate_house_snapshot`, `check_finite`, and the finite/`is_finite`
  guards on derived angles: **reachability analysis.** Kill the mutant if any
  reachable input distinguishes the operators (a NaN/∞ or out-of-range value
  reaching the guard through the public API). Document as an **equivalent
  mutant** only under the **overflow lens** (a finite-huge input overflowing to
  `inf`, per `[[fu9-guard-equivalence-overflow-lens]]`) when no reachable input
  makes exactly one operand non-finite — the recurring
  `nutation`/`topocentric`/`precession` shape. Follow the memo: test
  finite-overflow-to-inf *before* claiming any non-finite guard equivalent.

### String / match / constant — `catalog/mod.rs`, `thresholds.rs`

- **Catalog string-render** — the `Display` impls for `HouseFormulaFamily`,
  `HouseSystemDescriptor`, `HouseSystemCodeAlias`, and the two validation-error
  types, plus `summary_line` / `failure_mode_summary_line` /
  `validated_summary_line` / `matches_label`: **exact string assertions** on
  every rendering (release-facing diagnostics that a mutant could silently
  empty or drift — the `pleiades-types` finding).
- **Match-arm coverage** — `formula_family`, `catalog_name`,
  `resolve_house_system` / `resolve_house_system_code`,
  `house_formula_family_sort_key`, `expected_cusp_count`: exercise **one input
  per arm** so no arm can be swapped or defaulted unnoticed; alias resolution
  covered per SE letter code.
- **Catalog validation guards** — `validate_house_catalog_entries`,
  `validate_house_system_code_alias_entries`, `has_surrounding_whitespace`,
  `contains_line_break`: reachable-input tests that flip each guard (the
  `pleiades-types` `validate_against_reserved_labels` enum-vs-struct-dispatch
  lesson — assert through the *public* entry point, not only an internal
  helper).
- **`thresholds` ceilings** — `house_family_ceiling`: assert the **exact
  documented ceiling per family** (the rustdoc table, evaluated independently
  of the `match`), the 1.0″ floor, and the "space-division ≤ quadrant"
  ordering invariant. Every family arm asserted so an arm swap is caught.

## Expected documented-equivalent candidates

Enumerated so they are not re-litigated during triage — but the **true count is
measured, not predicted** (per `[[fu9-margin-table-per-mutant-rows]]`, never
aggregate; enumerate mutant × geometry and state the true minimum displacement):

- `||`↔`&&` non-finite **output** guards on shared-poisoned variables
  (`nutation`/`topocentric`/`precession` shape).
- Unreachable exact comparison boundaries: longitude wrap at exactly `±180°`,
  the southern-hemisphere `f_pole = -90 - lat` flip at exactly the pole,
  `<`↔`<=` at a physically unreachable equality.

Each documented equivalent is left **visible with a written reachability
argument**, never `#[mutants::skip]`-suppressed (established posture — a
function-level skip would blanket-suppress that function's numeric mutants).
Where the overflow lens shows a guard *is* killable (the `topocentric.rs`
input-guard precedent: a finite `1e301` overflowing a squared-norm sum to
`+inf`), it is **killed, not documented**.

## Structure changes

- `systems/mod.rs`, `catalog/mod.rs`: tests already relocated to
  `systems/tests.rs` (55 tests) and `catalog/tests.rs` (18 tests) — **no
  structural change** unless a survivor requires a behavior-preserving
  testability seam, which would be its own no-op commit.
- `thresholds.rs`: **relocate** the inline `#[cfg(test)] mod tests { … }` to
  `thresholds/tests.rs`, keeping `#[cfg(test)] mod tests;` in `thresholds.rs`
  (Rust resolves the submodule to `thresholds/tests.rs`). Matches AGENTS.md and
  every prior slice.

## Weekly-tier expansion (`[tasks.mutants]`)

The default `[tasks.mutants]` task in `mise.toml` currently enumerates only
`-p pleiades-types -p pleiades-time -p pleiades-apparent`. **In the
crate-completing PR (PR 6), add `-p pleiades-houses`** so the weekly report-only
tier regression-checks the crate going forward — the "make it stick" step,
mirroring how the baseline three are enumerated. Accepted trade-off: extra
weekly wall-clock. `mutants-crate` already covers houses ad hoc via its
argument; this makes the default set include it.

## Acceptance criteria

- Each in-scope file reaches **0 surviving mutants or documented equivalents**,
  confirmed by the authoritative per-file command; whole-crate re-check reports
  `0 missed` (or only the documented equivalents).
- Every expected value derives from an **independent reference**, not the code's
  own output; any independent reimplementation is reproduced in the plan and
  cross-validated against the crate.
- **No parity gate touched** — `validate-houses` / `validate-angles` corpora,
  tolerances, and code unchanged.
- Mutants tier **stays report-only**; no score gate introduced.
- `mise run ci` green (fmt + clippy `-D warnings` + workspace test).
- `[tasks.mutants]` includes `-p pleiades-houses` after the crate-completing PR
  (PR 6).
- An **FU-9 Progress note** appended to `docs/follow-ups.md` per PR, in the
  established format, framed as a new post-baseline expansion slice (not part of
  the closed baseline). Any documented equivalents added to the running tally
  with per-mutant reachability arguments.

## Risks & mitigations

- **Large numeric surface (systems/mod.rs).** ~25 formulas × operator-swap
  surface may yield the largest single-file survivor set of any slice so far.
  *Mitigation:* the shared-primitive-first strategy — pinning `asc1`/`asc2`/
  `spherical_cotrans`/RA transforms exactly kills survivors across every
  composing system, shrinking the per-system residual before the independent
  recomputation is applied.
- **Reference-independence for a `swehouse.c` port.** The crate *is* a port of
  SE; a reference that re-uses the crate's own constants would be circular.
  *Mitigation:* recompute from **published** Meeus/SE formulas and constants
  evaluated outside the code; where a constant is SE-specific, cross-validate a
  second, genuinely different formulation (the precession-slice discipline of
  cross-checking Meeus 20.3/21.4 against 21.5/21.7).
- **JD-grid / f64 representability for solver-boundary pins.** Convergence-
  threshold and iteration-cap inputs must be exactly representable.
  *Mitigation:* per `[[fu9-jd-grid-representability]]`, compute expected values
  through the same full-magnitude arithmetic the code uses and assert an
  in-test precondition that the crafted input hits the boundary exactly.
- **Predicting equivalents instead of measuring.** *Mitigation:* candidates
  above are hypotheses only; the plan measures the real per-file survivor list
  first and classifies from the measurement.

## PR 5 addendum — Quadrant/projection + non-catalog tail (2026-07-24)

Design decisions specific to PR 5, recorded here rather than in a competing
spec so the campaign keeps **one** design doc. Supersedes, for PR 5 only, the
"Structure changes" note above (which predates the test file's growth) and the
`Quadrant/projection` row of the delivery table (which predicted `21`
survivors; the measurement found `23`).

### Re-measured baseline (2026-07-24, at `a8917919f`)

Whole-file, authoritative command, output written outside the repo:

```bash
MISE_TRUSTED_CONFIG_PATHS=/tmp mise exec -- cargo mutants \
  --test-tool nextest --test-workspace=false --baseline run \
  -p pleiades-houses --file crates/pleiades-houses/src/systems/mod.rs
```

**1,128 mutants tested in 31 min — 83 missed / 1,038 caught / 7 unviable.**
The 83 decompose exactly, with no unaccounted remainder:

| Bucket | Count |
|--------|-------|
| Prior slices' documented equivalents, reproduced identically (Foundation 13, Great-circle 8, Sector 6, Sunshine 5) | 32 |
| `catalog_name` — 26 match-arm deletes + 2 return-value replacements — **deferred to PR 6** | 28 |
| **PR 5 work** | **23** |

That the 32 prior residuals reproduce line-for-line is the measurement's own
consistency check: the four landed PRs neither regressed nor silently absorbed
each other's survivors.

### PR 5 scope — the measured 23

Per the scope decision below, PR 5 is **every remaining survivor in
`systems/mod.rs` except `catalog_name`**, not only the two formula families the
delivery table names. This makes PR 6 purely the non-numeric tail
(`catalog/mod.rs`, `catalog_name`, `thresholds.rs` relocation, `[tasks.mutants]`
expansion) and avoids a seventh slice.

| Function | Survivors | Lines | Root cause |
|----------|-----------|-------|------------|
| `topocentric_latitude` | 9 | 1680–1681 | the WGS-84 prime-vertical and eccentricity terms are unconstrained by the single existing test |
| `solve_placidian_cusp` | 6 | 1739–1756 | Newton **internals**: the `gp` derivative terms, the `gp.abs() < 1e-12` zero-derivative guard, the `delta.abs() < 1e-9` convergence test, the `!converged \|\| !q.is_finite()` fail-closed guard — invisible while the iteration still reaches the same root |
| `regiomontanus_houses` | 5 | 947–949 | the only Regiomontanus unit test runs at **lat 0**, where `sin(lat) = 0` and `cos(lat) = 1` make three of the four factors degenerate |
| `koch_houses` | 1 | 843 | the `90.0 - obliquity_deg` polar-circle threshold — nothing asserts Koch *fails* inside the polar circle |
| `validate_topocentric_observer` | 1 | 618 | `-> Ok(())`; **equivalent-mutant candidate** — see below |
| `midpoint_longitude` | 1 | 1790 | `-> Default::default()`; the Sripati test asserts cusps `==` `midpoint_longitude(...)` — the same function on both sides, so `0 == 0` still passes (circular) |

**Target: `23 → 0-or-documented-equivalent`**, with the killed-vs-equivalent
split **confirmed by a scoped re-run, not predicted** (established discipline).
Two survivors are equivalent-*candidates* at design time —
`validate_topocentric_observer` 618 and, conditionally, the two
`solve_placidian_cusp` 1739 derivative terms — each with its own section below;
every other survivor is expected killable.

### Reference strategy — hybrid (decision)

Unlike PRs 1–4, PR 5's systems are already covered by Swiss-Ephemeris corpus
rows *and* the crate already carries in-crate SE anchors at 1 arcsec
(`systems/tests.rs`, the `*_match_swiss_ephemeris_corpus_*` tests for Placidus,
Topocentric, Koch, Campanus, Alcabitius, Morinus). Their mutants survive only
because `cargo mutants` runs `--test-workspace=false`, so the `pleiades-validate`
gate is outside the mutation test set. PR 5 therefore mixes two authorities:

- **Swiss-Ephemeris corpus anchors** (new tests, 1 arcsec, extending the
  existing in-crate pattern): Regiomontanus at `c1_lat40` and `c2_lat55`, and
  Sripati at `c1_lat40`. `crates/pleiades-validate/data/houses-corpus/cusps.csv`
  carries all 23 systems × 6 charts (lat 0/40/55/66, a second epoch, lat −33).
  The Sripati row is what breaks `midpoint_longitude`'s circularity — SE never
  calls our function. Values are copied as literals with a provenance comment
  naming the row; the crate does **not** grow a path dependency on the
  tooling crate's data directory (that would invert the layering).
  This also answers the reviewer flag recorded in
  `[[fu9-houses-reference-independence]]`: for these systems the authority is
  genuinely foreign, not a mirror of our own formula.
- **`houses-reference.py` extension** (1e-12 pins, the campaign's established
  reference): `topocentric_latitude` recomputed from the **published** WGS-84
  constants (`a = 6_378_137.0`, `1/f = 298.257_223_563`) outside the crate, at
  several latitude/elevation pairs **including elevation ≠ 0** (the `+ elevation`
  terms at 1681 are otherwise degenerate); `solve_placidian_cusp` rooted by an
  **independent bisection** on the published residual
  `g(q) = cos(q/f) + tan φ · tan δ(α)`, `α = RAMC + q` — a genuinely different
  root-finder from the crate's Newton iteration, pinning all four solved cusps
  (11, 12, 2, 3).
- **Crafted guard tests, at the private seam.** Two of these guards are
  provably unreachable through `calculate_houses`, so they are exercised by
  calling the private function directly (`use super::*;`), per the `apparent.rs`
  private-primitive precedent:
  - **Koch polar circle (843).** The catalog gives Koch
    `max_abs_latitude_deg = Some(66.0)` (`catalog/mod.rs:712–720`), while the
    internal guard fires at `|lat| >= 90 - ε ≈ 66.56°`. Under `Strict` the
    public path rejects `|lat| > 66.0` first; under `SwissEphemerisFallback` it
    substitutes Porphyry. Neither reaches `koch_houses`, so the kill is a direct
    `koch_houses(...)` call at `lat = 70°` asserting `Err(NumericalFailure)` —
    the mutant moves the threshold to `90 + ε ≈ 113.4°`, unreachable for any
    `|lat| ≤ 90`, so it returns `Ok` and dies.
  - **Placidus zero-derivative / non-convergence (1741, 1756).** Driven by
    calling `solve_placidian_cusp` directly with a crafted
    `(st, lat, obliquity, house)`.

  A third, `midpoint_longitude`, needs no guard test — the Sripati corpus
  anchor kills it.

Corpus rows are 6-decimal, so corpus-anchored assertions pin at 1 arcsec while
reference-anchored ones pin at 1e-12 — recorded per mutant in the plan's margin
table, never aggregated (`[[fu9-margin-table-per-mutant-rows]]`).

### `validate_topocentric_observer` (618) — equivalent-mutant candidate

`validated_obliquity` calls `validate_observer` **before**
`validate_topocentric_observer` (`mod.rs:604–605`), and `validate_observer`
already maps `ObserverLocationValidationError::NonFiniteElevation` to
`HouseError` for *every* system — the behavior the existing
`house_request_validate_rejects_non_finite_elevation_even_without_topocentric_houses`
test pins. A non-finite elevation is `topocentric_latitude`'s **only** error
path, so no input can reach `validate_topocentric_observer` in a state where it
would return `Err`: the function is redundant defensive validation and its
`-> Ok(())` mutant is very likely equivalent.

The plan **attempts the kill first** (enumerating `topocentric_latitude`'s error
paths to confirm non-finite elevation is the only one) and documents it with a
written reachability argument only if that fails. Deleting the redundant
validator is the alternative fix but is a **production change**, out of scope
for a tests-only slice; if the equivalence is confirmed, the plan records it as
a candidate cleanup for a later PR rather than doing it here.

### Newton-internals mutants (`solve_placidian_cusp` 1739) — decision

The two derivative-term mutants change the Newton **step**, not the root: a
mutated derivative that still converges inside the 64-iteration cap yields a
bit-identical cusp. The plan **hunts a geometry where the mutated derivative
fails to converge (or lands on a different root)**, which makes them
observable; only if that search comes up empty are they left visible with a
written reachability argument. No `#[mutants::skip]`, and **no testability
refactor** — PR 5 stays tests-only, so the extract-a-seam option is explicitly
rejected. The `1741` (`gp.abs() < 1e-12`) and `1750` (`delta.abs() < 1e-9`)
comparison swaps are assessed with the free-parameter lens
(`[[fu9-equivalence-free-param-and-modular-lenses]]`): `<` → `==` is *not*
measure-zero (it disables the guard for every genuinely near-zero derivative)
and is expected killable at the private seam; `<` → `<=` at an exact threshold
equality is the measure-zero case and may be a documented equivalent.

### Test-layout split (decision, first commit)

`crates/pleiades-houses/src/systems/tests.rs` has reached **3,209 lines / 90
tests** across four triage PRs, with PR 5 and PR 6 still to add. Per AGENTS.md
("split it before adding more, not after") and the `pleiades-types` slice
precedent (a 1,464-line `tests.rs` relocated into a per-module `src/tests/`),
PR 5 opens with a **pure-move commit**: `systems/tests.rs` →
`systems/tests/` containing `mod.rs`, `support.rs` (the existing
`assert_close_degrees` / `test_asc_mc` helpers), and per-family files
(`primitives`, `greatcircle`, `sector`, `sunshine`, `quadrant`, `dispatch`).
No test body changes; verified a no-op by identical test count and names before
and after. PR 5's new tests land in `systems/tests/quadrant.rs` and
`systems/tests/dispatch.rs` in later commits.

### PR 5 acceptance criteria (in addition to the campaign's)

- Scoped re-run
  `-F 'in (topocentric_latitude|solve_placidian_cusp|regiomontanus_houses|koch_houses|validate_topocentric_observer|midpoint_longitude)$'`
  reports `0 missed`, or only documented equivalents with per-mutant
  reachability arguments.
- **Tests-only:** no production file is modified. The whole diff is
  `crates/pleiades-houses/src/systems/tests.rs` → `systems/tests/**` (the move),
  the new tests, one `#[cfg(test)] mod tests;` declaration line in
  `systems/mod.rs` if the move requires it, the reference-note extension, and
  the `docs/follow-ups.md` Progress entry.
- `mise.toml` untouched (the `-p pleiades-houses` weekly-tier expansion is PR 6).

## References

- `docs/follow-ups.md` — FU-9 (baseline CLOSED note; running documented-
  equivalent tally; reusable method).
- Prior slice specs: `2026-07-20-fu9-topocentric-mutant-triage-design.md`
  (independent-reimplementation reference discipline),
  `2026-07-21-fu9-precession-lighttime-mutant-triage-design.md` (solver-
  boundary pins; cross-formulation independence),
  `2026-07-21-fu9-types-mutant-triage-design.md` (string-render + match-arm +
  guard class).
- `crates/pleiades-houses/src/thresholds.rs` — documented per-family ceiling
  table (independent-evaluation source for the threshold pins).
- Memory: overflow-lens guard rule, per-mutant margin tables, JD-grid
  representability.
