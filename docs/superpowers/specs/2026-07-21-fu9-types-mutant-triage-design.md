# FU-9 ninth (final) slice — `pleiades-types` + `provenance.rs` mutant triage

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
[`2026-07-20-fu9-sidereal-mutant-triage-design.md`](2026-07-20-fu9-sidereal-mutant-triage-design.md),
[`2026-07-21-fu9-precession-lighttime-mutant-triage-design.md`](2026-07-21-fu9-precession-lighttime-mutant-triage-design.md),
[`2026-07-21-fu9-time-mutant-triage-design.md`](2026-07-21-fu9-time-mutant-triage-design.md)
**Target files (11):** `crates/pleiades-types/src/{zodiac,time,time_range,coordinates,ayanamsa,angles,house_systems,motion,observer,frames}.rs`
and `crates/pleiades-apparent/src/provenance.rs`

---

## 1. Context

FU-9 is the report-only mutation-testing backlog opened by the devkit
Phase 3 cargo-mutants slice. Eight slices are complete: `pleiades-apparent`
and `pleiades-time` are fully triaged. What remains is the third and last
crate of the original three-crate baseline, **`pleiades-types`** (41
survivors across 10 files), plus the one file the backlog's own tail never
listed — **`pleiades-apparent/src/provenance.rs`** (3 survivors), which the
baseline notes recorded but every prior slice's remaining-slices line
omitted (the `apparent.rs` slice closed the *numeric* `apparent`-crate
files but not this non-numeric tail).

Scope decision, confirmed with the maintainer: cover **all 11 files in one
final slice**, folding `provenance.rs` in the way the sidereal slice folded
in `pleiades-time/src/sidereal.rs` — so FU-9's entire measured baseline
reaches a documented terminal state in one pass rather than leaving a
3-mutant orphan. This slice **closes the FU-9 baseline**.

`pleiades-types` is foundational (primitive/shared types only, no backend
dependencies). Unlike the numeric-heavy `apparent`/`time` crates, its
survivors are dominated by **enum plumbing** — `Display` impls, match-arm
dispatch, validation guards, and simple conversions — with exactly one
polynomial (`Instant::mean_obliquity`). The survivors cluster into a small
number of shapes already seen in prior slices, so a broad-but-shallow slice
stays tractable.

## 2. Goal & scope

**Goal:** drive all 11 files from 41 + 3 = **44 surviving mutants to 0**,
using tests that express intent against independent references, never
assertions that pin the code's own output.

**Expected residual: a genuine 0.** No equivalent-mutant candidate surfaced
during design (§5) — like the `apparent.rs`, `aberration.rs`, `sidereal.rs`,
and `time` slices, and unlike `nutation.rs` (1), `refraction.rs` (3),
`topocentric.rs` (3), and `precession.rs` (2).

**In scope:**

- a mechanical, pure-move relocation of `pleiades-types`'s single monolithic
  `src/tests.rs` (1,464 lines, 69 tests) into a per-source-module
  `src/tests/` directory (§7), per AGENTS.md's "split a large file before
  adding to it";
- relocation of `provenance.rs`'s inline `#[cfg(test)] mod tests` to
  `crates/pleiades-apparent/src/provenance/tests.rs`;
- new white-box tests, added into the relevant relocated submodule;
- FU-9 progress notes in `docs/follow-ups.md` marking the backlog's
  measured baseline complete.

**Out of scope / non-goals:**

- any production-code change in any of the 11 files — this is a **tests-only**
  slice; the only source edits are the test-module relocations (like the
  refraction/topocentric/sidereal/time slices, and unlike apparent/aberration
  which needed a testability refactor). `pleiades-types` survivors are already
  at the right seams — small pure functions and enum dispatch — so no seam is
  missing;
- any parity-gate change (`validate-*`); the mutants tier stays
  **report-only**;
- `#[mutants::skip]` suppression;
- re-deriving the mean-obliquity coefficients: they are the shared IAU-1976
  cubic already single-sourced through `OBLIQUITY_J2000_DEG` (FU-4/B-series);
  this slice pins the *polynomial evaluation*, not the constants' provenance.

## 3. Baseline

Whole-crate figures from `cargo mutants -p pleiades-types --test-tool
nextest --test-workspace=false` (confirmed this slice: **311 tested, 41
missed, 232 caught, 38 unviable**, identical to the 2026-07-18 baseline and
its documented zero-jitter repeat run) and the per-file command for
`provenance.rs` (`--file crates/pleiades-apparent/src/provenance.rs`: **6
tested, 3 missed, 3 caught**).

Per-file survivor counts (all confirmed against `missed.txt`):

| File | Survivors |
| --- | ---: |
| `zodiac.rs` | 12 |
| `time.rs` | 10 |
| `time_range.rs` | 4 |
| `coordinates.rs` | 3 |
| `ayanamsa.rs` | 3 |
| `angles.rs` | 3 |
| `house_systems.rs` | 2 |
| `motion.rs` | 2 |
| `observer.rs` | 1 |
| `frames.rs` | 1 |
| **`pleiades-types` subtotal** | **41** |
| `provenance.rs` (`pleiades-apparent`) | 3 |
| **Total** | **44** |

The authoritative per-file re-check for the `pleiades-types` files uses
`cargo mutants -p pleiades-types --test-tool nextest --test-workspace=false
--file crates/pleiades-types/src/<file>.rs`.

## 4. Survivor classification & treatment

The 44 survivors fall into six recurring shapes. Each shape's treatment is
described once, then mapped to every file it touches.

### Shape 1 — `Display` / `summary_line` never string-asserted (11 mutants)

`Ok(Default::default())` (an empty write) and `String::new()` / `"xyzzy"`
body replacements survive wherever no test asserts the *rendered string*.

| File:line | Mutant | Why it survives |
| --- | --- | --- |
| `zodiac.rs:103` | `Display for ZodiacSign` → `Ok(default)` | `ZodiacSign` display never string-asserted |
| `zodiac.rs:84` ×2 | `ZodiacSign::name` → `""` / `"xyzzy"` | `name()` never asserted |
| `motion.rs:19` | `Display for MotionDirection` → `Ok(default)` | direction *logic* tested (`motion_direction_tracks_the_sign`), display string not |
| `motion.rs:54` | `Display for MotionValidationError` → `Ok(default)` | error rendering not string-asserted |
| `observer.rs:130` | `Display for ObserverLocationValidationError` → `Ok(default)` | rendering not asserted |
| `frames.rs:39` | `Display for Apparentness` → `Ok(default)` | `CoordinateFrame` display *is* tested (line 535) but `Apparentness` is not |
| `coordinates.rs:205` | `Display for CoordinateValidationError` → `Ok(default)` | `summary_line` asserted directly (line 1295) but never via the `Display` wrapper |
| `provenance.rs:96` ×2 | `TopocentricProvenance::summary_line` → `String::new()` / `"xyzzy"` | only `ApparentProvenance::summary_line` is tested; the `Topocentric` variant is not |
| `provenance.rs:108` | `Display for TopocentricProvenance` → `Ok(default)` | same gap |

**Treatment — one string-vocabulary test per file** (5 in `pleiades-types`,
1 in `provenance.rs`), each asserting the exact rendered string of every
variant against a literal stated in the test:

- `zodiac_sign_names_and_display_are_stable` — all 12 signs: `.name()` and
  `.to_string()` both equal the expected name (e.g. `"Aries"`, `"Pisces"`).
  Also kills Shape 2's arm deletions where the display path is exercised, but
  Shape 2 is killed independently by the band test.
- `motion_direction_and_error_render_stable_strings` — `Direct`/
  `Stationary`/`Retrograde` → `"Direct"`/… ; `MotionValidationError`
  rendered via `to_string()` equals `"motion field `longitude_deg_per_day`
  must be finite, got NaN"` (constructed value).
- `observer_location_validation_errors_render_stable_strings` — each
  `ObserverLocationValidationError` variant's `to_string()` against its
  literal (the existing test asserts `summary_line`; this asserts the
  `Display` wrapper too).
- `apparentness_displays_stable_labels` — `Apparent`→`"Apparent"`,
  `Mean`→`"Mean"`.
- `coordinate_validation_error_display_matches_summary_line` — for one
  representative variant, assert `err.to_string() == err.summary_line()` and
  both equal the expected literal (ties the `Display` wrapper to the
  already-pinned `summary_line`).
- (`provenance.rs`) `topocentric_provenance_summary_line_and_display_are_stable`
  — build a `TopocentricProvenance` with distinct field values and assert
  `summary_line()` equals the exact `format!`-rendered literal (all four
  fields present, correct precision) and `to_string() == summary_line()`.

Intent expressed: these strings are release-facing diagnostics and must not
silently become empty or drift. This is a **coverage hole**, not tolerance
masking (the refraction-slice distinction).

### Shape 2 — match-arm dispatch deletion (9 mutants, `zodiac.rs`)

`ZodiacSign::from_longitude` (lines 66–79) maps `(deg/30).floor() % 12` to a
sign. Deleting any arm `2..=10` makes that band fall through to
`_ => Pisces`. The only existing test (`zodiac_signs_follow_longitude_bands`)
checks longitudes 0°, 29.999°, 30.0° — arms 0 and 1 only.

**Treatment — `zodiac_signs_cover_every_thirty_degree_band`:** assert one
representative longitude inside each of the 12 sign bands (e.g. band mid-point
`15 + 30·k`) maps to the correct sign, plus the two existing boundary
assertions (0°→Aries, 30.0°→Taurus) are retained. Every deleted arm now
returns Pisces where the correct sign is expected → categorical mismatch, no
tolerance. Also add a `≥360°`/wraparound case (e.g. 360.0°→Aries, 780.0°→
Gemini via the `% 12`) to pin the `floor … % 12` reduction that the arm
dispatch depends on. (Implementation note: an earlier draft used `750.0°`,
which normalizes to `30.0°`→Taurus, not the intended distinct third band;
`780.0°`→`60.0°`→Gemini is the corrected value that landed in the test.)

### Shape 3 — `mean_obliquity` polynomial (10 mutants, `time.rs`)

All 10 survivors are arithmetic-operator swaps in the IAU-1976 mean-obliquity
cubic (`time.rs:349–354`). The sole existing test pins the value at **t = 0**
(J2000, JD 2451545.0), where every `t`, `t²`, `t³` term vanishes — so no
polynomial mutant is observable.

| Line:col | Mutant | Term affected |
| --- | --- | --- |
| 349:56 | `/`→`%`, `/`→`*` | `(jd−2451545)/36525` (the `t` definition) |
| 352:17 | `−`→`+` | linear `−0.0130041…·t` sign |
| 353:17 | `−`→`+`, `−`→`/` | quadratic term join |
| 353:56 | `*`→`+` | `t·t` |
| 354:17 | `+`→`*`, `+`→`−` | cubic term join |
| 354:55 | `*`→`+` | first `*` of `t·t·t` |
| 354:59 | `*`→`+` | second `*` of `t·t·t` |

**Treatment — `mean_obliquity_matches_published_cubic_off_epoch`:** pin the
exact value at **two epochs, t = +1 and t = −1** (JD 2488070.0 = year 2100,
JD 2415020.0 = year 1900) — both `jd − 2451545` = ±36525.0 exactly
representable, so `t` is exactly ±1.0 (the JD-grid representability
discipline). Expected values come from the published IAU-1976 coefficients
evaluated **outside the code** (§6 mirror), pinned as full-precision f64
literals at `1e-12°`.

The ±1 pair follows the aberration/sidereal idiom: it separates the odd
terms (linear 352, cubic 354 — sign-flip under mutation moves +1 and −1
oppositely) from the even quadratic (353). Design-stage per-mutant
displacement (enumerated, not assumed) at t = ±1:

| Mutant | Δ at t=+1 (deg) | Δ at t=−1 (deg) |
| --- | ---: | ---: |
| 349:56 `/`→`%` (t→0) | 0.0130 | 0.0130 |
| 349:56 `/`→`*` | huge | huge |
| 352:17 `−`→`+` | 0.0260 | 0.0260 |
| 353:17 `−`→`+` | 3.28e-7 | 3.28e-7 |
| 353:17 `−`→`/` | huge | huge |
| 353:56 `*`→`+` | 1.64e-7 | 3.3e-7 |
| 354:17 `+`→`*` | huge | huge |
| 354:17 `+`→`−` | 1.01e-6 | 1.01e-6 |
| 354:55 `*`→`+` | 1.64e-7 | 5.0e-7 |
| 354:59 `*`→`+` | 1.64e-7 | 5.0e-7 |

True minimum ≈ **1.64e-7°** vs a `1e-12°` tolerance — a ~1.6e5× margin.
Every mutant is displaced at both epochs; the ± pair guards against a
coincidental cancellation at a single t (e.g. `t·t = t+t` at t = 2, avoided).
The existing t = 0 test is kept — it owns the J2000-constant intent.

### Shape 4 — validation guard reachable-boundary / inverted-guard (9 mutants)

Boolean-operator and comparison-boundary swaps in validators that no test
exercises at the discriminating input.

| File:line | Mutant | Discriminating input (untested) |
| --- | --- | --- |
| `time_range.rs:32` | `&&`→`\|\|` in `contains` (start side) | instant *before* start, same scale |
| `time_range.rs:35` | `&&`→`\|\|` in `contains` (end side) | instant *after* end, same scale |
| `time_range.rs:37` | `&&`→`\|\|` (`after_start && before_end`) | instant failing exactly one side |
| `time_range.rs:61` | `>`→`>=` in `validate` (out-of-order) | `start == end` (must validate `Ok`) |
| `coordinates.rs:266` | `<`→`<=` in `validate_distance` | `distance_au == 0.0` (must validate `Ok`) |
| `coordinates.rs:216` | `validate_finite_coordinate_value`→`Ok(())` | a *non-finite longitude/latitude* (routes here before the range check) |
| `ayanamsa.rs:222` | delete `!` in `CustomAyanamsa::validate` | a *finite* offset must be accepted / a *non-finite* one rejected, with epoch present |
| `ayanamsa.rs:167` | `Ayanamsa::validate_against_reserved_labels`→`Ok(())` | enum-wrapped `Custom` with a colliding label |
| `ayanamsa.rs:168` | delete match arm `Self::Custom` | same |
| `house_systems.rs:121` | `HouseSystem::validate_against_reserved_labels`→`Ok(())` | enum-wrapped `Custom` with a colliding label |
| `house_systems.rs:122` | delete match arm `Self::Custom` | same |

(11 mutants total: 7 boolean/comparison-guard survivors —
`time_range` 4, `coordinates` 2, `ayanamsa:222` 1 — plus the 4 reserved-label
dispatch survivors — `ayanamsa` 167/168, `house_systems` 121/122. The five
shapes sum to 11 + 9 + 10 + 11 + 3 = 44; see §7.3's per-test inventory.)

**Treatment:**

1. **`time_range.rs` — `time_range_contains_and_validate_respect_boundaries`:**
   with a bounded `[start, end]` range (same scale), assert an instant *before*
   `start` and one *after* `end` are **not** contained (kills 32, 35, 37 — the
   `||` mutant would wrongly include a value failing one clause), and assert a
   `start == end` degenerate range validates `Ok` (kills 61 — `>=` would flag it
   out-of-order). The existing `time_range_checks_scale_and_julian_day` and
   `time_range_validation_rejects_non_finite_bounds_and_invalid_order` are kept.
2. **`coordinates.rs` — extend the existing validation test (or add
   `coordinate_validation_accepts_zero_distance_and_rejects_non_finite_angle`):**
   - distance exactly `0.0` validates `Ok` (kills 266 `<`→`<=`);
   - an `EclipticCoordinates` with a **non-finite longitude** (`f64::NAN`,
     which survives `Longitude::from_degrees` normalization) validates to
     `Err(NonFiniteValue{ field: "longitude", … })` (kills 216 — the
     `Ok(())` mutant would skip the finite check and fall through). This is
     reachable via the public constructor, so it is killed, not documented.
3. **`ayanamsa.rs` — `custom_ayanamsa_accepts_finite_offset_pair_and_rejects_non_finite`:**
   a `CustomAyanamsa` with both `epoch` and a **finite** `offset_degrees`
   present validates `Ok`, and one with a **non-finite** `offset_degrees`
   (both present) returns `Err(non_finite("offset_degrees"))` (kills 222:
   the deleted `!` inverts both directions).
4. **Reserved-label enum dispatch — `enum_validate_against_reserved_labels_checks_wrapped_custom`
   (one test per enum):** call
   `Ayanamsa::Custom(CustomAyanamsa::new("Lahiri")).validate_against_reserved_labels(is_reserved)`
   and `HouseSystem::Custom(CustomHouseSystem::new("Equal")).validate_against_reserved_labels(is_reserved)`
   with a resolver that flags the label, and assert `Err`. The existing tests
   (lines 473, 632) call the method on the **struct** (`CustomAyanamsa` /
   `CustomHouseSystem`) directly — they never traverse the **enum** method or
   its `Self::Custom` arm, which is exactly why 167/168 and 121/122 survive.
   A GREEN assertion that a **built-in** variant (`Ayanamsa::Lahiri`,
   `HouseSystem::Placidus`) returns `Ok` documents the `_ => Ok(())` arm's
   intent.

### Shape 5 — simple accessor / conversion (3 mutants, `angles.rs`)

| File:line | Mutant | Why it survives |
| --- | --- | --- |
| `angles.rs:52` ×2 | `Angle::is_finite` → `true` / `false` | `is_finite` never asserted on both a finite and a non-finite `Angle` |
| `angles.rs:125` | `From<Angle> for Latitude` → `Default::default()` | the `Angle`→`Latitude` conversion never asserts the value round-trips |

**Treatment — `angle_is_finite_and_latitude_from_angle_preserve_values`:**
`Angle::from_degrees(42.0).is_finite()` is `true` and
`Angle::from_degrees(f64::NAN).is_finite()` is `false` (kills both 52
mutants — each fixes the output to one constant); `Latitude::from(Angle::
from_degrees(-33.5)).degrees() == -33.5` (kills 125 — `Default::default()`
would yield `0.0`). The `Longitude` `From<Angle>` twin is already covered by
`longitude_constructor_normalizes` / existing display tests; only the
`Latitude` conversion and `is_finite` are gaps.

### Shape 6 — n/a

(No sixth shape; the survivor set is fully covered by Shapes 1–5.)

## 5. Equivalent-mutant analysis

**Expected residual: 0 across all 11 files.** The two shapes that produced
documented equivalents in prior slices were checked and neither is present:

- **Guard `||`↔`&&`:** the only boolean-operator swaps here are
  `time_range.rs`'s three `&&`→`||` mutants, which are **reachable and
  behaviorally distinct** — an instant on the wrong side of a single bound is
  a physical input that the `||` form wrongly admits (unlike the
  `nutation.rs`/`topocentric.rs`/`precession.rs` non-finite guards, where a
  shared poisoned variable drove both operands together). They are killed.
- **Exact comparison boundaries:** `time_range.rs:61` (`>`→`>=`, boundary
  `start == end`) and `coordinates.rs:266` (`<`→`<=`, boundary
  `distance == 0.0`) are both **reachable** — equal bounds and zero distance
  are ordinary valid inputs — unlike the topocentric slice's unreachable
  ±180° wrap boundaries. They are killed by GREEN boundary assertions.
- **`validate_finite_coordinate_value`→`Ok(())`** (`coordinates.rs:216`) is
  reachable through the public constructor with a NaN angle (normalization
  preserves NaN), so it is killed, not documented — the overflow-lens
  conclusion of prior slices does not apply (no in-window arithmetic is
  involved; the input itself is non-finite).

Should implementation surface an unexpected equivalence, the
killed-instead-of-documented rule from the topocentric slice applies:
document it in follow-ups with a reachability argument, never
`#[mutants::skip]`.

## 6. Reference strategy (independence discipline)

- **`mean_obliquity` polynomial:** the two off-epoch expected values come
  from a scratchpad Python mirror (retained in the plan doc) of the
  published IAU-1976 mean-obliquity cubic, written from the published
  coefficients (independently, not copied from the source's inline literals),
  evaluated in f64 with matching
  operation order, and cross-validated against the crate before the
  literals are pinned. The per-mutant displacement table in §4.3 comes from
  the same mirror with each mutation applied in turn (per-mutant rows, never
  aggregated).
- **Strings:** every `Display`/`name`/`summary_line` expectation is the
  spec'd rendering of each variant, stated as a literal in the test, not
  read from the code's output.
- **Bands / boundaries:** the zodiac band mid-points, the `start == end`
  range, the `distance == 0.0` value, the NaN-angle input, and the
  finite/non-finite offset pair are all crafted discriminating inputs; the
  expected sign/error variant is derived from the type's documented contract,
  not from running the function.
- **Representability:** the off-epoch obliquity epochs are chosen so
  `jd − 2451545` is exactly ±36525.0 and `t` is exactly ±1.0.

## 7. Test relocation & inventory

### 7.1 `pleiades-types` test split (mechanical, one commit)

Per AGENTS.md ("split a large file before adding to it" and "co-locate a
module's tests"), the monolithic `src/tests.rs` (1,464 lines) is converted
into a `src/tests/` directory: `src/tests/mod.rs` declares one submodule per
source module, and each existing test moves **unchanged** into the matching
submodule (`src/tests/angles.rs`, `time.rs`, `time_range.rs`, `coordinates.rs`,
`zodiac.rs`, `motion.rs`, `ayanamsa.rs`, `house_systems.rs`, `observer.rs`,
`frames.rs`, `bodies.rs`, `custom_bodies.rs`). Submodules use `use crate::*;`
(the crate root re-exports the full public vocabulary — §lib.rs) plus the
per-test local imports already present (`core::time::Duration`,
`proptest::prelude::*` for the angles proptests). `lib.rs`'s `mod tests;`
line is unchanged (it now resolves to `tests/mod.rs`). This commit is a pure
move — **no test body changes, zero behavioral delta** — reviewable by
confirming the test count (69) and `cargo test -p pleiades-types` are
identical before and after.

### 7.2 `provenance.rs` relocation

`provenance.rs`'s inline `#[cfg(test)] mod tests` moves to
`crates/pleiades-apparent/src/provenance/tests.rs` (matching every prior
`pleiades-apparent` slice and this crate's own layout). `use super::*`.

### 7.3 New-test inventory

| New test | Submodule / file | Kills | Shape |
| --- | --- | ---: | --- |
| `zodiac_sign_names_and_display_are_stable` | `tests/zodiac.rs` | 3 (84 ×2, 103) | 1 |
| `zodiac_signs_cover_every_thirty_degree_band` | `tests/zodiac.rs` | 9 (arms 2–10) | 2 |
| `mean_obliquity_matches_published_cubic_off_epoch` | `tests/time.rs` | 10 | 3 |
| `time_range_contains_and_validate_respect_boundaries` | `tests/time_range.rs` | 4 | 4 |
| `coordinate_validation_accepts_zero_distance_and_rejects_non_finite_angle` | `tests/coordinates.rs` | 2 (216, 266) | 4 |
| `coordinate_validation_error_display_matches_summary_line` | `tests/coordinates.rs` | 1 (205) | 1 |
| `custom_ayanamsa_accepts_finite_offset_pair_and_rejects_non_finite` | `tests/ayanamsa.rs` | 1 (222) | 4 |
| `ayanamsa_enum_validate_against_reserved_labels_checks_wrapped_custom` | `tests/ayanamsa.rs` | 2 (167, 168) | 4 |
| `house_system_enum_validate_against_reserved_labels_checks_wrapped_custom` | `tests/house_systems.rs` | 2 (121, 122) | 4 |
| `angle_is_finite_and_latitude_from_angle_preserve_values` | `tests/angles.rs` | 3 (52 ×2, 125) | 5 |
| `motion_direction_and_error_render_stable_strings` | `tests/motion.rs` | 2 (19, 54) | 1 |
| `observer_location_validation_errors_render_stable_strings` | `tests/observer.rs` | 1 (130) | 1 |
| `apparentness_displays_stable_labels` | `tests/frames.rs` | 1 (39) | 1 |
| `topocentric_provenance_summary_line_and_display_are_stable` | `provenance/tests.rs` | 3 (96 ×2, 108) | 1 |
| **Total** | | **44** | |

All existing tests are kept — they own real intent (band boundaries,
J2000 constant, scale/order validation, `summary_line` literals, direction
logic) that the new tests extend rather than replace.

## 8. Verification & acceptance criteria

1. `cargo mutants -p pleiades-types --test-tool nextest
   --test-workspace=false` reports **0 missed** (was 41).
2. `cargo mutants -p pleiades-apparent --test-tool nextest
   --test-workspace=false --file crates/pleiades-apparent/src/provenance.rs`
   reports **0 missed** (was 3).
3. `mise run ci` is green; `cargo fmt --all --check` and clippy
   (`-D warnings`) clean.
4. No source change in any of the 11 files other than the test-module
   relocations (§7).
5. No `validate-*` gate file is touched; no `#[mutants::skip]` added.
6. The mean-obliquity off-epoch literals are re-confirmed against the crate
   (< 1e-12 relative) by the §6 mirror before pinning, and the mirror script
   is reproduced in the plan doc.
7. The `pleiades-types` test-split commit is verified pure-move: identical
   test count (69) and green `cargo test -p pleiades-types` before and after,
   no assertion changes in that commit.

## 9. Deliverables

Ordered so the mechanical relocation lands first and each triage commit is
independently greppable:

1. Commit 1 — `test(types): relocate tests.rs into per-module src/tests/`
   (pure move, §7.1; no new tests).
2. Commit 2 — `test(types): FU-9 zodiac.rs mutant triage (12 -> 0)`.
3. Commit 3 — `test(types): FU-9 time.rs mean_obliquity mutant triage (10 -> 0)`.
4. Commit 4 — `test(types): FU-9 guard + conversion mutant triage
   (time_range/coordinates/ayanamsa/house_systems/angles, 15 -> 0)` — the
   `coordinates` file's three tests (both guard boundaries and the
   `Display`/`summary_line` tie) all live in `tests/coordinates.rs` and land
   here.
5. Commit 5 — `test(types): FU-9 Display mutant triage
   (motion/observer/frames, 4 -> 0)`.
6. Commit 6 — `test(apparent): FU-9 provenance.rs mutant triage (3 -> 0)`
   (includes the `provenance/tests.rs` relocation).
7. Commit 7 — `docs(follow-ups): FU-9 pleiades-types + provenance triage —
   baseline complete`, marking the FU-9 measured backlog closed and recording
   the final posture.

Branch `fu9-types-mutant-triage`, PR flow as in prior slices.

## 10. Follow-on

This slice **closes the FU-9 baseline** — every file in the 2026-07-18
three-crate measurement reaches 0 surviving mutants or a documented
equivalent. FU-9 stays open as a standing posture entry only if a future
`mise run mutants` baseline expands coverage to crates outside the original
three (`pleiades-*` domain/backend crates not yet measured); those are new
work, not part of this backlog. The follow-ups entry records the closure and
the total documented-equivalent tally across all nine slices.
