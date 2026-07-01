# Design: Close actionable open follow-ups from `feat/equatorial-declination-output`

**Date:** 2026-07-01
**Status:** approved (design)
**Source:** the "Open: Deferred minor findings from feat/equatorial-declination-output tasks" section of `docs/follow-ups.md`

## Purpose

Resolve the actionable subset of the deferred minor findings left over from the
frame-correction / equatorial-output work. These are cosmetic and defensive-
hardening items — no behavioral feature change. The goal is to remove drift-prone
duplicated constants, tighten one test assertion, de-tautologize one test, and
correct stale documentation, then mark the follow-ups resolved.

Two items in the original list are already satisfied by existing code and require
no change (documented below so the resolution note is accurate). Two other items
(the ELP-of-date behavior note and the SE global-ceiling rationale) are
documentation-only observations with no code action and remain as-is.

## Scope decisions (locked)

- **ε₀ unification aggressiveness:** unify the exact houses literal and the
  eclipse J2000 radians cache; **leave the of-date polynomial at
  `geometry.rs:399` untouched** (it is a self-contained matched-pair
  approximation, not a reference to the J2000 obliquity constant).
- **B3 positive-latitude assertion:** already satisfied by an existing in-loop
  sentinel; only the row-count assertion is tightened.
- **Task 2 smoke test:** already satisfied by existing coverage; no change.

## Part A — ε₀ literal unification

Single-source the J2000 obliquity constant so it cannot drift.

- **`crates/pleiades-houses/src/systems/mod.rs:584`** — replace the bare literal
  `23.439_291_111_111_11` (lead term of the IAU mean-obliquity polynomial in
  `fn mean_obliquity`) with `pleiades_types::OBLIQUITY_J2000_DEG`. The lead term
  *is* ε₀ at J2000, and the literal is an exact match, so this is a **zero
  numeric change**. Add the import if not already present.
- **`crates/pleiades-eclipse/src/geometry.rs:323`** — replace
  `const OBLIQUITY_RAD: f64 = 0.409_092_804_222_329;` with a const expression
  derived from the shared constant:
  `const OBLIQUITY_RAD: f64 = OBLIQUITY_J2000_DEG * core::f64::consts::PI / 180.0;`
  (`f64::to_radians` is not guaranteed usable in `const` context, so the
  multiply form is used to keep it a `const`.) This shifts the value by
  ~1e-9 rad — a precision *improvement* over the previously truncated literal —
  and is guarded by the eclipse validation gate.
- **`crates/pleiades-eclipse/src/geometry.rs:399`** — **untouched.** The of-date
  polynomial `23.439_291 - 0.013_004_2 * t` pairs a truncated constant with a
  truncated slope; swapping only its lead term for the full-precision J2000 value
  would be internally inconsistent and would not improve accuracy.

**Left alone deliberately:** the test/assert sentinels that hardcode the full
literal (`crates/pleiades-apparent/src/nutation.rs:148`,
`crates/pleiades-types/src/tests.rs:112`). These pin the constant's value against
regression; rewriting them to reference the constant would make them tautological.

## Part B — B3 frame-consistency gate hardening

- **`crates/pleiades-validate/src/frame_consistency_validation.rs:293`** — tighten
  the test assertion `report.rows_validated >= 17` to `== 17`. The exact expected
  count (8 VSOP87 bodies × 2 epochs + Moon × 1 = 17) is stable, so an exact
  assertion catches accidental row additions/removals.
- **Positive-latitude assertion:** **not added.** The "prove the latitude
  component is non-trivial" goal is already met by the existing in-loop sentinel
  at lines 236–240 (Sun@1900 ecliptic latitude must be ≈ −45″). A one-line
  comment near line 293 will cross-reference that sentinel so the intent is
  explicit and the follow-up is verifiably closed.

## Part C — Test quality

- **Task 1 — `composes_rotation_with_true_obliquity`
  (`crates/pleiades-apparent/src/equatorial.rs:53–64`):** the test currently
  compares `apparent_equatorial_of_date(e, jd)` against
  `e.to_equatorial(Angle::from_degrees(true_obliquity_degrees(jd)))` — the same
  math on both sides, so it is tautological. De-tautologize by asserting the
  helper's output against an **independent, precomputed RA/Dec constant** for the
  test's fixed input (ecliptic 123.456°/1.234°/0.987 AU at JD 2_433_283.0). The
  expected RA/Dec values are computed once during implementation (from the
  closed-form rotation using the independently known true obliquity of that JD)
  and pinned as literals with a tolerance of ~1e-6°. `distance_au` preservation
  continues to be asserted exactly.
- **Task 2 — `true_obliquity_degrees`:** **no change.** Dedicated smoke coverage
  already exists (`true_obliquity_is_mean_plus_delta_eps`, lines 42–51) plus two
  indirect tests (`solstice_point_maps_to_ra90_dec_obliquity`,
  `roundtrips_through_to_ecliptic`). Marked resolved by existing coverage.

## Part D — Cosmetic docs

- **B5 — `crates/pleiades-apparent/src/precession.rs` (`PrecessedEcliptic`,
  module doc lines 1–7, struct doc line 17, field docs lines 20 and 22):** the
  struct is now used for both of-date and J2000 output, but the rustdoc still
  says "referred to the mean equinox/ecliptic of date". Reword to state the
  struct carries ecliptic longitude/latitude in the frame chosen by the caller
  (mean equinox of date **or** J2000), removing the unconditional "of date"
  claim.
- **Task 4 typo — `tools/se-equatorial-reference/src/main.rs:26`:** the comment
  for `JD_END_TT = 2_488_065.5` reads `2099-12-26`; the correct calendar date for
  that JD is `2099-12-28` (2100-01-01 00:00 TT = JD 2_488_069.5, minus 4 days).
  Fix the comment.

## Verification

1. `cargo build` for the workspace.
2. `cargo test` for the four touched crates: `pleiades-apparent`,
   `pleiades-houses`, `pleiades-eclipse`, `pleiades-validate`.
3. Run the **eclipse validation gate** explicitly to confirm the `geometry.rs:323`
   precision change introduces no regression.
4. Run the **frame-consistency gate** (`validate-frame-consistency`) to confirm
   the `== 17` assertion holds.
5. On green, update `docs/follow-ups.md`: move the resolved items out of the
   "Open" section (or annotate each with a resolution note dated 2026-07-01),
   retaining the two documentation-only observations (ELP raw of-date equatorial;
   SE global-ceiling rationale) as still-open notes.

## Non-goals

- No changes to `geometry.rs:399`, to the SE ceiling constants, or to the ELP
  raw-backend equatorial behavior.
- No new validation gates; no numeric tolerance changes beyond what the
  `geometry.rs:323` precision bump implies (which the existing eclipse gate
  already bounds).

## Summary of edits

| # | File | Change | Numeric impact |
|---|------|--------|----------------|
| A1 | `pleiades-houses/src/systems/mod.rs:584` | use `OBLIQUITY_J2000_DEG` | none |
| A2 | `pleiades-eclipse/src/geometry.rs:323` | derive `OBLIQUITY_RAD` from constant | ~1e-9 rad (improvement) |
| B1 | `pleiades-validate/src/frame_consistency_validation.rs:293` | `>= 17` → `== 17` + comment | none |
| C1 | `pleiades-apparent/src/equatorial.rs:53–64` | independent precomputed RA/Dec | none (test only) |
| D1 | `pleiades-apparent/src/precession.rs` (docs) | reframe "of date" wording | none |
| D2 | `tools/se-equatorial-reference/src/main.rs:26` | fix comment date | none |
