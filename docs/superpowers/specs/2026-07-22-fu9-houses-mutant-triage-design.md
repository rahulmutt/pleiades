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
