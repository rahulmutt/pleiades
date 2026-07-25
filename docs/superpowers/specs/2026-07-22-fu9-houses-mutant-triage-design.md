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

## PR 6 addendum — Catalog + thresholds, the crate-completing slice (2026-07-25)

Design decisions specific to PR 6, recorded here rather than in a competing
spec so the campaign keeps **one** design doc. This is the campaign's last PR:
it closes the non-numeric tail, adds `pleiades-houses` to the weekly tier, and
restates FU-9 for whatever comes after.

### Measured baseline (2026-07-25, at `348c00ac6`)

Two authoritative runs, both re-measured today rather than carried over from
the 2026-07-22 whole-crate prediction:

```bash
cargo mutants -p pleiades-houses --test-tool nextest --test-workspace=false \
  --baseline run \
  --file crates/pleiades-houses/src/catalog/mod.rs \
  --file crates/pleiades-houses/src/thresholds.rs

cargo mutants -p pleiades-houses --test-tool nextest --test-workspace=false \
  --baseline run \
  --file crates/pleiades-houses/src/systems/mod.rs -F 'catalog_name'
```

| Surface | Tested | Missed | Caught | Unviable |
|---------|--------|--------|--------|----------|
| `catalog/mod.rs` + `thresholds.rs` | 101 (2 min) | **15** | 69 | 17 |
| `catalog_name` (`systems/mod.rs`) | 28 (41 s) | **28** | 0 | 0 |

Both match the figures the campaign already carried — the 2026-07-22 delivery
table predicted `15` for `catalog/mod.rs`, and PR 5's whole-file decomposition
reserved `28` for `catalog_name`. `thresholds.rs` contributes **no** survivors
(its single mutant is caught); its only PR 6 work is the AGENTS.md test
relocation.

Note on the `catalog_name` filter: `-F 'catalog_name'` is deliberately **not**
anchored as `-F 'in (catalog_name)$'`. Per PR 5's closing guidance, the anchored
form structurally excludes whole-function replacement mutants, whose description
reads `replace catalog_name -> &'static str with …` and never ends in
`in <function>`. The unanchored filter captures both the 26 arm deletes and the
2 return-value replacements, and the `28 tested` total confirms it.

**Triage surface: 43 survivors.**

### `catalog_name` — single-source refactor (28 → ~3)

Two structural facts drive this, both established during design and neither
recorded anywhere in the crate today:

1. **`catalog_name` is dead through the public API.** Its sole call site is
   `systems/mod.rs:495`, inside the dispatch match's `_` arm. All 25 concrete
   `HouseSystem` variants have explicit arms above it; the `_` exists only
   because `HouseSystem` is `#[non_exhaustive]` and lives in another crate. No
   input reaches `catalog_name` through `calculate_houses`. That is *why* all
   28 survived: no integration path constrains them.
2. **It duplicates the catalog.** Its 25 name strings are byte-for-byte
   identical to the corresponding `HouseSystemDescriptor::canonical_name`
   values, and **nothing asserts the two tables agree** — they can drift
   silently. This is the FU-5 GMST-duplication shape.

The slice therefore closes the seam rather than pinning the duplicate, in three
ordered commits so the write-up can state unambiguously which mutants were
*killed by test* and which *ceased to exist*:

**(a) Cross-table characterization test** — assert
`catalog_name(&s) == descriptor(&s).canonical_name` for all 25 built-ins, plus
`Custom(_) -> "Custom"`. Against `HEAD` this is a genuine tests-only kill of all
28, and it doubles as the **no-op proof** for the refactor that follows.

**(b) Behavior-preserving refactor** (own commit, no runtime-result change):

```rust
fn catalog_name(system: &HouseSystem) -> &'static str {
    match system {
        HouseSystem::Custom(_) => "Custom",
        other => crate::catalog::descriptor(other)
            .map_or("Unspecified", |d| d.canonical_name),
    }
}
```

Behavior identity rests on two facts the plan **asserts rather than assumes**:
that the descriptor table covers exactly the 25 concrete variants
`catalog_name` enumerates, and that the `canonical_name` strings are
byte-identical. Test (a) is what proves both. `descriptor()` returns `None` for
`Custom(_)` (no catalog entry carries a `Custom` system), so the explicit
`Custom` arm is load-bearing and is kept.

**(c) Replace the now-tautological test** — after (b) the cross-table assertion
compares the descriptor table against itself, so it is replaced by a
three-behavior residual pin: known system → catalog `canonical_name`,
`Custom(_)` → `"Custom"`, unknown variant → `"Unspecified"`.

Residual after (b): the `Custom` arm delete, the `map_or` fallback, and the
whole-function replacement — ~3 mutants, killed by (c). **Measured, not
predicted.**

The write-up must not conflate the two mechanisms: 26 mutants **cease to exist**
(the arms are gone), they are not killed by a test.

### `catalog/mod.rs` — the 15

| Bucket | Mutants | Kill strategy |
|--------|---------|---------------|
| Rendering return-values — `failure_mode_summary_line -> String::new()` / `"xyzzy".into()` (245:9 ×2), `HouseSystemCodeAliasValidationError::fmt -> Ok(default)` (428:9) | 3 | Exact string assertions, the `pleiades-types` `Display` precedent — release-facing diagnostics that a mutant could silently empty |
| `latitude_sensitive_house_failure_modes -> vec![]` / `vec![String::new()]` / `vec!["xyzzy".into()]` (678:5 ×3) | 3 | Assert exact contents **and** length, enumerated independently from the descriptor table's `latitude_sensitive` entries |
| Counter `+=` → `*=` (462:24, 594:24, 610:28) | 3 | Assert exact `entry_count` / `label_count` / alias count against independently counted values (`spec/compatibility-catalog.md`'s 23-code SE set; 25 built-ins) — a counter starting at `0` is invariant under `*=`, so only an exact-count assertion distinguishes them |
| `formula_family` delete-arm `HouseSystem::Custom(_)` (219:13) | 1 | Construct a `Custom` descriptor and assert `HouseFormulaFamily::Custom`, distinguishing it from the `_ => Unknown` fallthrough it would otherwise reach |
| Validation guard `\|\|` → `&&` (118:13, 160:50, 645:49) | 3 | Reachability analysis first, then craft entries that flip exactly one operand. `160:50` is an equivalence *candidate*: the clause `!notes.is_empty() && notes.trim() != notes` is arguably implied by the preceding `notes.trim().is_empty()` test |
| Whole-function `-> Ok(())`: `validate_house_catalog` (637:5), `validate_house_system_code_aliases` (475:5) | 2 | **Equivalent-mutant candidates**, PR 5's VT-1 shape — the built-in catalog is valid by construction, so `HEAD` and mutant both return `Ok(())` on every reachable input. Probe for a testability seam before documenting |

**Target: `43 → 0-or-documented-equivalent`**, with the killed-vs-equivalent
split **measured, not predicted**. Predicted residual is `0–3` (the two `Ok(())`
wrappers plus possibly `160:50`), but per the Foundation and Sunshine
corrections that number is a hypothesis: every equivalence claim gets a
reachability probe *before* it is written down, with per-mutant rows and a
stated true minimum — never an aggregated table.

### Structure changes

Supersedes, for PR 6 only, the "Structure changes" section above, which
predates the test files' growth.

- **`thresholds.rs`** — relocate the inline `#[cfg(test)] mod tests { … }`
  (lines 103–133) to `thresholds/tests.rs`, keeping `#[cfg(test)] mod tests;`
  in `thresholds.rs`. The last inline test module in the crate; this is the
  file's entire contribution to PR 6.
- **`catalog/tests.rs`** — 1,140 lines, and PR 6 adds to it. Per AGENTS.md's
  "split it before adding more, not after" and PR 5's opening move, split into
  `catalog/tests/` (`support` / `descriptor` / `validation` / `aliases` /
  `families`) as a **verified no-op move** in its own commit: identical
  `cargo nextest list` inventory before and after, no test body edited.
- **`systems/tests/quadrant.rs`** — migrate the six open-coded corpus closures
  (Morinus, Placidus+Topocentric, Koch, Campanus, Alcabitius c1_lat40,
  Alcabitius c2_lat55 — ~270 lines of near-identical arrange blocks) onto
  `assert_corpus_cusps`, and rename the five tests named `*_within_120_arcsec`
  that actually assert a `1.0`-arcsec tolerance. A seventh,
  `equal_house_angles_match_swiss_ephemeris_corpus_within_120_arcsec` in
  `trivial.rs`, carries the same misleading name but asserts *angles*, not
  cusps; check its tolerance and rename it too if it is also 1″, rather than
  leave one behind. PR 5 deferred this migration here explicitly, as a
  maintainability change rather than a triage change.

### Sector GQ-1 withdrawal

PR 5 recorded a correction it could not apply in its own territory:
`solve_gauquelin_sector`'s `1327:21 <` → `==` was documented equivalent on the
premise that "the campaign does not pin error-message text", which is false —
`systems/tests.rs` already pinned message text before PR 3, and PR 5 killed the
structurally identical `solve_placidian_cusp` `1741 <` → `==` exactly that way.

PR 6 adds the message assertion to
`solve_gauquelin_sector_fails_closed_on_nonconvergence` and withdraws GQ-1.
Tally effect: Sector residual **6 → 5**, houses sub-total **35 → 34**,
campaign-wide **44 → 43**, before PR 6's own residual is added.

### Weekly-tier expansion

`[tasks.mutants]` in `mise.toml` gains `-p pleiades-houses`, alongside the
existing `-p pleiades-types -p pleiades-time -p pleiades-apparent`. This is the
"make it stick" step: the weekly report-only tier regression-checks the crate
from then on.

Budget evidence, which the design has not had until now — the workflow comment
in `.github/workflows/mutants.yml` named the first scheduled run as its
calibration point, and that run has since happened:

| Measure | Value |
|---------|-------|
| 2026-07-20 scheduled `mutants.yml` run | **16 min 5 s** |
| Workflow timeout | 90 min |
| Current tier mutant count | ~1,451 |
| After adding `pleiades-houses` | ~2,680 (~1.85×) |
| Projected wall-clock | **~30–35 min** |

Comfortable headroom. PR 6 records this in the workflow comment so the next
maintainer inherits a calibrated number rather than the original guess.

### FU-9 disposition and next-crate roadmap

FU-9 stays **open as a standing posture entry** — the same disposition it took
when the three-crate baseline closed — carrying a campaign-closing summary:
final crate-wide tally, the whole-crate confirmation run, and the fact that
`pleiades-houses` now sits in the weekly tier.

For the roadmap to be **real rather than guessed**, PR 6 measures survivor
baselines for the four smallest candidate crates:

| Crate | Mutants | Baseline measured by PR 6 |
|-------|---------|---------------------------|
| `pleiades-apsides` | 223 | yes |
| `pleiades-backend` | 263 | yes |
| `pleiades-ayanamsa` | 305 | yes |
| `pleiades-fict` | 308 | yes |

1,099 mutants, ≈20 min — bounded, and it makes the ordering defensible.

The remaining eight are recorded as **mutant counts only** (from
`cargo mutants -p <crate> --list`, 2026-07-25), labelled explicitly as *sizing,
not survivor counts*, so no ordering is fabricated from unmeasured data:

| Crate | Mutants | Crate | Mutants |
|-------|---------|-------|---------|
| `pleiades-compression` | 607 | `pleiades-elp` | 1,521 |
| `pleiades-eclipse` | 913 | `pleiades-data` | 1,752 |
| `pleiades-core` | 962 | `pleiades-events` | 1,901 |
| `pleiades-vsop87` | 1,493 | `pleiades-jpl` | 3,662 |

~13,900 unmeasured mutants across twelve crates. That is the honest headline
for the closing note: the houses campaign covered **one** crate of thirteen
remaining, and FU-9's standing posture is what carries the rest.

### Acceptance criteria (PR 6)

- `catalog/mod.rs`, `thresholds.rs`, and `catalog_name` reach
  **0-or-documented-equivalent**, each equivalent carrying a per-mutant
  reachability argument and left visible (no `#[mutants::skip]`).
- **Whole-crate confirmation run** — `cargo mutants -p pleiades-houses
  --test-tool nextest --test-workspace=false` (~1,205 mutants after the
  refactor removes 26 arms, down from 1,231; ~21+ min). This is required, not
  optional: PR 5's closing guidance is that a scoped `-F` run structurally
  excludes whole-function replacement mutants, and PR 6 has two of those as
  residual candidates.
- The confirmation run decomposes with **no unaccounted remainder**. Going in,
  `systems/mod.rs` carries `63` survivors (PR 5's measured figure: `28`
  `catalog_name` + `35` prior-slice documented equivalents — the `32` from
  PRs 1–4 plus PR 5's own `3`) and `catalog/mod.rs` carries `15`, for `78`
  crate-wide. Coming out: prior-slice documented equivalents `35 → 34` (GQ-1
  killed), plus PR 6's own measured residual, plus zero elsewhere.
- The `catalog/tests.rs` split and the `quadrant.rs` corpus migration are
  **inventory-verified** (`cargo nextest list` identical across the move), not
  eyeballed.
- `[tasks.mutants]` includes `-p pleiades-houses`; `mutants.yml` timeout comment
  updated with the calibrated figure.
- **No parity gate touched** — `validate-houses` / `validate-angles` corpora,
  tolerances, and code unchanged. Tier stays report-only.
- `mise run ci` green (fmt + clippy `-D warnings` + workspace test). Run
  `cargo fmt` before every commit — array literals in these docs and tests have
  broken the fmt gate in prior slices.
- `docs/follow-ups.md` gains the PR 6 Progress note **and** the campaign-closing
  restatement with the roadmap table above.

### Risks specific to PR 6

- **The refactor must be provably no-op.** Mitigated by ordering the cross-table
  characterization test first; the refactor commit is gated on it passing
  unchanged.
- **Mutant-surface reduction is not a test kill.** Deleting 26 match arms
  improves the score without improving the suite. Mitigated by landing (a)
  first, so the tests-only kill is a real, recorded event, and by stating both
  mechanisms separately in the follow-up note.
- **Equivalence over-claiming.** Three of this campaign's five landed PRs had an
  equivalence claim refuted on review. Mitigated by probing each candidate
  before writing it down, and by treating the two `Ok(())` wrappers as
  *candidates* until the confirmation run says otherwise.
- **Roadmap measurement scope creep.** Four crates, 1,099 mutants, ≈20 min is
  the whole budget; the other eight get counts only. Mitigated by making the
  sizing-vs-survivor distinction explicit in the recorded table.

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
