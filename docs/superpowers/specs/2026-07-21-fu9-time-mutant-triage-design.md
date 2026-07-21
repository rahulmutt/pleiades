# FU-9 eighth slice — `pleiades-time` non-sidereal mutant triage (convert + deltat + tdb + calendar)

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
[`2026-07-21-fu9-precession-lighttime-mutant-triage-design.md`](2026-07-21-fu9-precession-lighttime-mutant-triage-design.md)
**Target files:** `crates/pleiades-time/src/convert.rs` (16 survivors),
`deltat.rs` (10), `tdb.rs` (9), `calendar.rs` (9)

---

## 1. Context

FU-9 is the report-only mutation-testing backlog. Seven slices are complete
and `pleiades-apparent` is fully triaged. The backlog's remaining-slices
line reads "`pleiades-time` non-sidereal (`convert.rs` 16, `deltat.rs` 10,
`tdb.rs` 9)" — but the baseline notes also record **`calendar.rs` at 9
survivors**, omitted from that line by transcription oversight (the
sidereal slice already covered `sidereal.rs`'s 5). Scope decision for this
slice: cover **all four files**, closing out the `pleiades-time` crate
entirely, the way the seventh slice closed `pleiades-apparent`. The
follow-ups progress entry corrects the omission.

This is the first **four-file** slice; it stays tractable because the
files are small and the survivors cluster into a handful of shapes.

## 2. Goal & scope

**Goal:** drive all four files from 16 + 10 + 9 + 9 = 44 surviving mutants
to **0**, using tests that express intent against independent references.
Expected residual is a genuine zero: no equivalent-mutant candidate
surfaced during design (§5), like the sidereal slice.

**In scope:**

- relocated, expanded white-box test suites at
  `crates/pleiades-time/src/convert/tests.rs`, `src/deltat/tests.rs`,
  `src/tdb/tests.rs`, and `src/calendar/tests.rs`;
- FU-9 progress notes in `docs/follow-ups.md`, including the
  `calendar.rs` omission correction.

**Out of scope / non-goals:**

- any production-code change in any of the four files (tests-only slice;
  the only source edits are the four inline `#[cfg(test)] mod tests`
  relocations per AGENTS.md);
- any parity-gate change (`validate-*`); the mutants tier stays
  **report-only**;
- `#[mutants::skip]` suppression;
- re-derivation of the leap-second/ΔT table *contents* — the committed
  CSVs are the authority for interpolation references, and their own
  correctness remains the checksum pins' and upstream sources' job.

## 3. Baseline

Regenerated with the method's authoritative per-file commands
(`cargo mutants -p pleiades-time --test-tool nextest
--test-workspace=false --file crates/pleiades-time/src/<file>.rs`):

| File | Tested | Missed | Caught | Unviable |
| --- | ---: | ---: | ---: | ---: |
| `convert.rs` | 47 | 16 | 23 | 8 |
| `deltat.rs` | 65 | 10 | 52 | 3 |
| `tdb.rs` | 17 | 9 | 8 | 0 |
| `calendar.rs` | 130 | 9 | 120 | 1 |

All four exactly confirm the whole-workspace baseline figures — no
reconciliation note is needed.

## 4. Survivor classification & treatment

### 4.1 `convert.rs` (16)

| Group | Location | Count | Root cause |
| --- | --- | --- | --- |
| A — diagnostics never string-asserted | 40:9, 62:9 (`Display` for `ConversionPath`/`ConversionQuality`), 91:9 ×2 (`summary_line` → `String::new()` / `"xyzzy"`), 107:9 (`Display` for `ConversionProvenance`) | 5 | no test renders any of them |
| B — `finite` guard | 121:5 (`finite` → `Ok(())`) | 1 | unreachable through the public API: the 1900–2101 support window bounds `jd_civil`, and ΔT within the window is bounded ~153 s, so `jd + dt/86400` can never be non-finite (overflow lens checked — no finite-input overflow route exists) |
| C — leap-epoch boundary | 139:25 (`<` → `<=` on `jd_civil < LEAP_EPOCH_JD`) | 1 | no test converts UTC at exactly JD 2441317.5 (1972-01-01 00:00) |
| D — converted-JD arithmetic | 159:34 (`+`→`-`, `+`→`*`), 159:39 (`/`→`%`, `/`→`*`) on the future-UTC path; 174:34, 174:39 (same four) on the UT1 path | 8 | existing tests assert path/quality/provenance but never the converted JD value on either ΔT path |
| E — TDB step sign | 230:32 (`+`→`-` on `jd_tt + tdb_minus_tt/86400`) | 1 | the only TDB-vs-TT test uses `.abs()` — the sign-free-assertion failure mode the topocentric slice diagnosed |

**Treatment — five tests:**

1. **`display_vocabulary_is_stable`** (kills group A, 5 mutants) — exact
   string assertions on every `ConversionPath` and `ConversionQuality`
   variant (`"utc-leap-second"`, `"ut1-delta-t"`, `"future-extrapolated"`,
   `"exact"`, `"observed"`, `"predicted"`), plus `summary_line` in both
   shapes (Some/Some with `delta_t` formatting and None/None → `"n/a"`)
   and `ConversionProvenance`'s `Display` against the same expected
   literal. Intent: the diagnostic vocabulary is release-facing and must
   not drift silently.
2. **`finite_guard_fails_closed_on_non_finite`** (kills group B) — direct
   white-box tests of the private `finite`: `Err` on `f64::NAN` and
   `f64::INFINITY`, `Ok` on a finite value. Justification for testing the
   private fn directly: the guard's fail-closed contract is real intent,
   but no public input can reach it (group B root cause), mirroring the
   `apparent.rs` slice's direct tests of private combine primitives.
3. **`utc_at_exact_leap_epoch_is_exact`** (kills group C) — UTC
   1972-01-01 00:00:00 (JD exactly `LEAP_EPOCH_JD` = 2441317.5) must
   convert `Ok` with `quality == Exact`, `tai_minus_utc == Some(10)`
   (first leap-table row), and `jd_tt` pinned to
   `2441317.5 + 42.184/86400` = **2441317.5004882407** (1e-9 days). The
   `<=` mutant rejects this instant with `UtcBeforeLeapEpoch`.
4. **`future_utc_and_ut1_jd_values_match_hand_computation`** (kills group
   D, 8 mutants) — two epochs, each pinning the converted JD at 1e-9 days
   against literals computed outside the code (§6):
   - **future-UTC:** civil 2090-06-01 00:00 UTC → `jd_civil` 2484568.5
     (past the leap table's `VALID_THROUGH_JD` 2461040.5, inside the
     support window), ΔT = extrapolate(decimal_year) =
     137.73624952070443 s, `jd_tt` = **2484568.5015941695**; also asserts
     `path == FutureExtrapolated` and `delta_t_seconds` pinned.
   - **UT1:** civil 1955-06-15 00:00 UT1 → `jd_civil` 2435273.5, ΔT
     hand-interpolated between the committed 1950 (29.1 s) and 1960
     (33.2 s) decade nodes = 31.334934976043815 s, `jd_tt` =
     **2435273.500362673**; also asserts `quality == Observed` and
     `delta_t_seconds` pinned.

   Per-mutant displacement at these epochs (design-stage numeric check;
   tolerance 1e-9 days):

   | Mutant | future-UTC (159) | UT1 (174) | min margin |
   | --- | --- | --- | --- |
   | `+`→`-` | 3.19e-3 days | 7.25e-4 days | ~7.3e5× |
   | `+`→`*` | 2.48e6 days | 2.43e6 days | huge |
   | `/`→`%` | 137.7 days (dt % 86400 = dt, in *days*) | 31.3 days | huge |
   | `/`→`*` | 1.19e7 days | 2.71e6 days | huge |

   True minimum across group D: **7.25e-4 days** (UT1 `+`→`-`), a
   ~7.3e5× margin.
5. **`tdb_target_applies_positive_periodic_term`** (kills group E) — UTC
   civil 2000-04-01 00:00 converted to both TT and TDB targets; the epoch
   sits near the annual peak of TDB−TT (g ≈ 87°), so the signed
   difference `(jd_tdb − jd_tt) × 86400` ≈ **+1.657e-3 s**. The two JDs
   are within a factor of two, so the subtraction is exact (Sterbenz);
   the JD-grid quantization of the stored `jd_tdb` bounds the observable
   diff's error by half-ulp(JD) ≈ 2.33e-10 days ≈ 2.0e-5 s. Assert the
   signed diff within **5e-5 s** of the externally computed term: the
   `+`→`-` mutant lands at −1.657e-3 s, a displacement of 3.3e-3 s —
   ~66× the tolerance. (Exact expected literal fixed in the plan by the
   §6 script; the existing `.abs()` bound test is kept — it still owns
   the magnitude-bound intent.)

### 4.2 `deltat.rs` (10)

| Group | Location | Count | Root cause |
| --- | --- | --- | --- |
| F — `extrapolate` polynomial | 77:18 (`-`→`+` in `year − 2000`); 78:11 (`+`→`-`, `+`→`*`), 78:25 (`+`→`*`), 78:21, 78:36, 78:40 (`*`→`+`, `*`→`/` on each star of `0.32217·t` and `0.005589·t·t`) | 10 | the only future-path test asserts `dt > 69.0` — every polynomial mutant still clears that bar |

**Treatment — one test, `extrapolated_delta_t_matches_published_polynomial`:**
public `delta_t` at **JD 2480765.0** — chosen because
`2451545 + 365.25 × 80` is exactly representable, so `decimal_year` is
*exactly* 2080.0 and t = 80 exactly (the JD-grid representability
discipline). Assert ΔT = **124.4632 s** (the published Espenak–Meeus
2005–2050 polynomial evaluated outside the code at t = 80) at 1e-9 s, and
quality `Predicted`.

Per-mutant displacements at t = 80 (design-stage, all 10 enumerated):

| Mutant | Δ (seconds) |
| --- | ---: |
| 77:18 `-`→`+` (t = year + 2000) | 94289.6 |
| 78:11 `+`→`-` | 51.5 |
| 78:11 `+`→`*` | 1533.0 |
| 78:25 `+`→`*` | 3048.1 |
| 78:21 `*`→`+` | 54.5 |
| 78:21 `*`→`/` | **25.77** |
| 78:36 `*`→`+` | 6364.7 |
| 78:36 `*`→`/` | 35.76 |
| 78:40 `*`→`+` | 44.7 |
| 78:40 `*`→`/` | 35.76 |

True minimum **25.77 s** vs a 1e-9 s tolerance — ~2.6e10× margin. A
single epoch suffices because every mutant's displacement was enumerated
(not assumed); no ± pair is needed since no mutant's displacement
vanishes at t = 80.

### 4.3 `tdb.rs` (9)

| Group | Location | Count | Root cause |
| --- | --- | --- | --- |
| G — USNO two-term formula | 11:24 (`+`→`-`, `+`→`*`), 11:38 (`*`→`+`), 11:47 (`-`→`+`, `-`→`/`) in `g_deg`; 13:25 (`+`→`-` between the terms), 13:37 (`*`→`/`), 13:44 (`*`→`+`, `*`→`/`) in the sine sum | 9 | the existing test asserts `\|v\| < 0.002` — but the output is bounded below 2 ms *by construction for any g*, so a bound test can never kill a phase mutant; value-pinning is mandatory |

**Treatment — one test, `tdb_minus_tt_matches_usno_formula_at_pinned_epochs`:**
two epochs, each killing all 9 alone (belt-and-braces against
coincidental agreement), asserted against literals computed outside the
code with matching evaluation order, tolerance **1e-9 s** (comfortably
above cross-libm `sin` differences of ~1 ulp ≈ 1e-19, and below every
displacement):

| Mutant | Δ at JD 2451645.0 (s) | Δ at JD 2446895.5 (s) |
| --- | ---: | ---: |
| 11:24 `+`→`-` | 3.27e-3 | 3.28e-3 |
| 11:24 `+`→`*` | 2.77e-3 | 2.70e-3 |
| 11:38 `*`→`+` | 1.01e-5 | 8.33e-4 |
| 11:47 `-`→`+` | 3.05e-3 | 3.07e-3 |
| 11:47 `-`→`/` | 1.69e-3 | 1.69e-3 |
| 13:25 `+`→`-` | 5.91e-6 | 4.84e-6 |
| 13:37 `*`→`/` | 6.34e-5 | 7.85e-5 |
| 13:44 `*`→`+` | 4.19e-6 | 4.49e-6 |
| 13:44 `*`→`/` | 6.43e-6 | **2.04e-6** |

True minimum **2.04e-6 s** — ~2000× margin. Base values ≈ 1.6457e-3 s
and 1.6493e-3 s (full-precision literals fixed in the plan by the §6
script).

### 4.4 `calendar.rs` (9)

| Group | Location | Count | Root cause |
| --- | --- | --- | --- |
| H — validation `\|\|`→`&&` | 52:58 (second `\|\|` in the `second` check) | 1 | **empirically verified during design** (mutant hand-applied, test run, reverted): the token swap re-associates by precedence to `A \|\| (B && C)`, which still rejects NaN (A short-circuits) but silently *accepts* `second = -1.0` and `second = 61.5` — no test rejects a negative or ≥ 61 second |
| I — `from_julian_day` arithmetic/branches | 84:42 (`/`→`%` in `alpha/4`), 86:21 (`-`→`+` in `b − 122.1`), 91:26 (`<`→`<=` on `e < 14`), 91:54 (`-`→`+`, `-`→`/` in `e − 13`), 92:29 (`>`→`>=` on `month > 2`), 92:59 (`-`→`+`, `-`→`/` in `c − 4715`) | 8 | existing test dates are coincidence-degenerate: at 1987, `floor(alpha/4) == alpha % 4` (both 3) and the `b − 122.1` fraction sits in the mutation-safe window; no test full-field-asserts a January (e = 14), February (e = 15), or ≥ 2000-03-01 (alpha = 16) reconstruction |

**Treatment — three tests:**

1. **`rejects_negative_and_oversized_seconds`** (kills group H) —
   `second = -1.0` → `Err(field: "second")` and `second = 61.0` (the
   exact exclusive bound) → `Err(field: "second")`; a GREEN assertion
   that `second = 60.5` (leap-second range) is accepted documents the
   `[0, 61)` intent. Under the mutant both rejections fail (values
   accepted).
2. **`from_julian_day_reconstructs_2100_new_year`** — full six-field
   assertion that JD 2488069.5 → 2100-01-01 00:00:00. Discriminating
   properties: `alpha = 16` (separates `floor(alpha/4)` = 4 from
   `alpha % 4` = 0) and January (`e = 14`, exercising the `e − 13`
   branch and the `<` boundary exactly).
3. **`from_julian_day_reconstructs_gregorian_leap_day`** — full six-field
   assertion that JD 2451604.0 → 2000-02-29 12:00:00. Discriminating
   properties: February (`e = 15` — the only month class that separates
   `e/13` → month 1 from month 2, and `month == 2.0` — the only value
   where `>` vs `>=` differ), on the Gregorian 400-year-exception leap
   day.

   Kill matrix (design-stage; each cell = the mutated six-field output
   differs from / equals the correct one):

   | Mutant | 2100-01-01 00:00 | 2000-02-29 12:00 |
   | --- | --- | --- |
   | 84:42 `/`→`%` | **kills** (day 5) | same |
   | 86:21 `-`→`+` | **kills** (2101-01-02) | **kills** (2001-02-31) |
   | 91:26 `<`→`<=` | **kills** (month 13, year 2099) | same |
   | 91:54 `-`→`+` | **kills** (month 27) | **kills** (month 28) |
   | 91:54 `-`→`/` | same (14/13 → 1 ≡ January) | **kills** (15/13 → month 1 ≠ 2) |
   | 92:29 `>`→`>=` | same (January unaffected) | **kills** (year 1999) |
   | 92:59 `-`→`+` | **kills** (year 11530) | **kills** (year 11430) |
   | 92:59 `-`→`/` | **kills** (year 1) | **kills** (year 1) |

   Every group-I mutant is killed by at least one date; all kills are
   categorical integer-field mismatches — no tolerance involved.
   Rejected-input note: a mid/late-year date (e.g. 2020-10-31) kills only
   84:42 and 86:21 and none of the month/year-branch mutants; it is not
   included (the two chosen dates cover everything).

## 5. Equivalent-mutant analysis

**Expected residual: 0 across all four files**, with no equivalent-mutant
candidate at design time. The two shapes that produced
documented equivalents in prior slices were checked:

- **Guard `||`→`&&`:** `calendar.rs` 52:58 looks like the
  `nutation.rs`/`topocentric.rs` guard shape but is *not* — precedence
  re-association makes it behaviorally distinct (group H), verified
  empirically, and it is killed, not documented.
- **`finite` → `Ok(())`** (`convert.rs` group B) is unreachable via the
  public API but perfectly reachable white-box, so it is killed by a
  direct unit test rather than documented as untestable. The overflow
  lens was applied: no finite in-window input can push `jd + dt/86400`
  non-finite (max ΔT within the window ≈ 153 s), so no public-API kill
  exists even in principle — which is exactly why the direct test is the
  right instrument, not a workaround.
- **Exact comparison boundaries** (`convert.rs` 139:25 `<`→`<=`;
  `calendar.rs` 91:26, 92:29) are all *reachable* — JD 2441317.5, e = 14,
  and month = 2 all occur for physical inputs — unlike the topocentric
  slice's unreachable ±180° wrap boundaries.

Should implementation surface an unexpected equivalence, the
killed-instead-of-documented rule from the topocentric slice applies in
reverse: document it in follow-ups with a reachability argument, never
`#[mutants::skip]`.

## 6. Reference strategy (independence discipline)

- **Formulas:** every pinned numeric literal comes from a scratchpad
  Python mirror (retained in the plan doc) of the *published* formulas —
  Espenak–Meeus ΔT polynomial (NASA eclipse site form), the USNO
  low-precision TDB−TT two-term model, Meeus ch. 7 calendar algorithm —
  written from the published sources, evaluated in f64 with matching
  operation order, and cross-validated against the crate before pinning.
  The per-mutant displacement tables in §4 come from the same mirror with
  each mutation applied in turn (per-mutant rows, never aggregated).
- **Tables:** the UT1 ΔT reference is hand linear interpolation between
  the committed CSV's 1950/1960 decade nodes; the leap-epoch reference is
  the CSV's first row (TAI−UTC = 10) plus the definitional
  TT−TAI = 32.184 s. The committed CSVs are the authority (§2 non-goal:
  their contents are not re-derived here).
- **Representability:** epochs are chosen so derived quantities are exact
  in f64 where it matters (`decimal_year(2480765.0)` = exactly 2080.0),
  and the TDB-step assertion budgets for JD-grid quantization
  (half-ulp(JD) ≈ 2.33e-10 days) per the JD-grid discipline from the
  topocentric/precession slices.
- **Strings:** the diagnostic-vocabulary expectations are the spec'd
  rendering of each variant, stated in the test, not read from the code's
  output.

## 7. Test relocation & inventory

Per AGENTS.md, all four inline `#[cfg(test)] mod tests` modules move to
co-located test files (`src/convert/tests.rs`, `src/deltat/tests.rs`,
`src/tdb/tests.rs`, `src/calendar/tests.rs`), matching the seven prior
slices and this crate's own `src/sidereal/tests.rs`. White-box unit tests
with `use super::*`; existing tests are kept — they own real intent
(path/quality routing, window boundaries, checksum pin, round-trips,
near-midnight clamp, magnitude bound) that the new tests do not replace.

| New test | File | Carries |
| --- | --- | --- |
| `display_vocabulary_is_stable` | convert | group A (5 kills) |
| `finite_guard_fails_closed_on_non_finite` | convert | group B (1 kill, white-box) |
| `utc_at_exact_leap_epoch_is_exact` | convert | group C (1 kill) + pinned exact-path JD |
| `future_utc_and_ut1_jd_values_match_hand_computation` | convert | group D (8 kills) |
| `tdb_target_applies_positive_periodic_term` | convert | group E (1 kill, signed) |
| `extrapolated_delta_t_matches_published_polynomial` | deltat | group F (10 kills) |
| `tdb_minus_tt_matches_usno_formula_at_pinned_epochs` | tdb | group G (9 kills) |
| `rejects_negative_and_oversized_seconds` | calendar | group H (1 kill) + leap-second GREEN assertion |
| `from_julian_day_reconstructs_2100_new_year` | calendar | group I (6 of 8) |
| `from_julian_day_reconstructs_gregorian_leap_day` | calendar | group I (6 of 8; union = all 8) |

## 8. Verification & acceptance criteria

1. All four §3 per-file commands report **0 missed**.
2. `mise run ci` is green; `cargo fmt` / clippy clean.
3. No source change in any of the four files other than the test-module
   relocations.
4. No `validate-*` gate file is touched.
5. No `#[mutants::skip]` is added.
6. Every pinned literal is re-confirmed against the crate (< 1e-12
   relative) by the §6 mirror before pinning, and the mirror script is
   reproduced in the plan doc.

## 9. Deliverables

1. Commit 1 — `test(time): FU-9 calendar.rs mutant triage (9 -> 0)`.
2. Commit 2 — `test(time): FU-9 deltat.rs mutant triage (10 -> 0)`.
3. Commit 3 — `test(time): FU-9 tdb.rs mutant triage (9 -> 0)`.
4. Commit 4 — `test(time): FU-9 convert.rs mutant triage (16 -> 0)`
   (last: its ΔT/TDB path pins build on the deltat/tdb references).
5. Commit 5 — `docs(follow-ups): record FU-9 pleiades-time triage`,
   noting the crate is complete, correcting the `calendar.rs` omission,
   and updating the remaining-slices list to the `pleiades-types` tail
   (`zodiac.rs` 12, `time.rs` 10, small tail).

Each per-file commit includes that file's test relocation. Branch
`fu9-time-mutant-triage`, PR flow as in prior slices.

## 10. Follow-on (out of scope here, tracked in FU-9)

Remaining after this slice: the `pleiades-types` survivors only
(`zodiac.rs` 12, `time.rs` 10, `time_range.rs` 4, and the small tail) —
the final slice of the FU-9 backlog's original three-crate baseline.
