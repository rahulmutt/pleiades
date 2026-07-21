# FU-9 `pleiades-time` Mutant Triage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive the 44 surviving mutants in `pleiades-time`'s `calendar.rs` (9), `deltat.rs` (10), `tdb.rs` (9), and `convert.rs` (16) to 0 with intent-expressing white-box tests, closing the crate's FU-9 backlog entirely.

**Architecture:** Tests-only slice. Each file's inline `#[cfg(test)] mod tests` moves to a co-located `src/<module>/tests.rs` (the crate's `sidereal` pattern), then gains targeted tests whose expected values come from independent references (published formulas evaluated outside the code, hand interpolation of the committed CSV tables, exact-string vocabulary). Design doc with per-mutant margin tables: `docs/superpowers/specs/2026-07-21-fu9-time-mutant-triage-design.md`.

**Tech Stack:** Rust (stable, per `mise.toml`), cargo-nextest, cargo-mutants 27.1.0.

## Global Constraints

- **Tests-only:** no production-code change in any of the four files; the only source edits are the four test-module relocations (spec §2).
- **No parity-gate change:** no `validate-*` file is touched; the mutants tier stays report-only (spec §2).
- **No `#[mutants::skip]`** anywhere (spec §8.5).
- Branch: `fu9-time-mutant-triage` (already created; the spec is commit 1 on it).
- Expected values must come from independent references, never from the code's own output (spec §6). Every pinned literal below was produced by the Appendix script.
- Relocation pattern (from `crates/pleiades-time/src/sidereal.rs`): the module file ends with `#[cfg(test)]\nmod tests;`; `src/<module>/tests.rs` starts with `use super::*;`. Keep tests white-box unit tests; do not convert to integration tests.
- Per-file mutants command (run from repo root; output dir under `target/` so nothing lands in the working tree):
  `cargo mutants -p pleiades-time --test-tool nextest --test-workspace=false --file crates/pleiades-time/src/<file>.rs -o target/mutants-<file>`
  Acceptance for each task: `0 missed` in the summary line.

---

### Task 1: `calendar.rs` (9 → 0)

**Files:**
- Modify: `crates/pleiades-time/src/calendar.rs` (replace lines 118–208, the inline `#[cfg(test)] mod tests { ... }` block, with a module declaration)
- Create: `crates/pleiades-time/src/calendar/tests.rs`

**Interfaces:**
- Consumes: `CivilDateTime`, `CivilTimeError`, `pleiades_types::JulianDay` — all reachable via `use super::*;` (calendar.rs imports `JulianDay` at the top).
- Produces: nothing consumed by later tasks.

- [ ] **Step 1: Relocate the inline test module**

In `calendar.rs`, delete the entire `#[cfg(test)] mod tests { ... }` block (lines 118–208) and put in its place:

```rust
#[cfg(test)]
mod tests;
```

Create `crates/pleiades-time/src/calendar/tests.rs` containing `use super::*;` followed by the six existing test functions (`j2000_noon_is_jd_2451545`, `epoch_anchors_match_known_jds`, `round_trips_within_a_millisecond`, `round_trips_nonzero_seconds_without_minute_corruption`, `rejects_bad_fields`, `near_midnight_does_not_overflow_hour`) moved **verbatim** — copy their bodies exactly as they appear in the deleted block, with the `#[test]` attributes, un-indented one level (they are now at file scope inside the module file).

- [ ] **Step 2: Verify the relocation is behavior-preserving**

Run: `cargo nextest run -p pleiades-time calendar`
Expected: all 6 existing calendar tests PASS (same count as before the move).

- [ ] **Step 3: Add the three new tests** (append to `calendar/tests.rs`)

```rust
#[test]
fn rejects_negative_and_oversized_seconds() {
    // The second field's contract is [0.0, 61.0): negative and >= 61 are
    // invalid; [60, 61) is deliberately accepted (leap seconds).
    assert_eq!(
        CivilDateTime::new(2000, 1, 1, 0, 0, -1.0).to_julian_day(),
        Err(CivilTimeError::InvalidCivilDate { field: "second" })
    );
    assert_eq!(
        CivilDateTime::new(2000, 1, 1, 0, 0, 61.0).to_julian_day(),
        Err(CivilTimeError::InvalidCivilDate { field: "second" })
    );
    assert!(CivilDateTime::new(2016, 12, 31, 23, 59, 60.5)
        .to_julian_day()
        .is_ok());
}

#[test]
fn from_julian_day_reconstructs_2100_new_year() {
    // JD 2488069.5 = 2100-01-01 00:00:00 (proleptic Gregorian; the known
    // epoch anchor already pinned in epoch_anchors_match_known_jds).
    // Discriminating properties (spec §4.4): alpha = 16, where
    // floor(alpha/4) = 4 differs from alpha % 4 = 0, and January, where
    // e = 14 exercises the e-13 month branch and the e < 14 boundary.
    let back = CivilDateTime::from_julian_day(JulianDay::from_days(2_488_069.5));
    assert_eq!(back.year, 2100);
    assert_eq!(back.month, 1);
    assert_eq!(back.day, 1);
    assert_eq!(back.hour, 0);
    assert_eq!(back.minute, 0);
    assert!(back.second.abs() < 0.001, "second: {}", back.second);
}

#[test]
fn from_julian_day_reconstructs_gregorian_leap_day() {
    // JD 2451604.0 = 2000-02-29 12:00:00 — February is the only month
    // class where e = 15 (separating e - 13 = 2 from e / 13 = 1) and
    // month == 2.0 (the only value where > vs >= differ in the year
    // branch), on the Gregorian 400-year-exception leap day.
    let back = CivilDateTime::from_julian_day(JulianDay::from_days(2_451_604.0));
    assert_eq!(back.year, 2000);
    assert_eq!(back.month, 2);
    assert_eq!(back.day, 29);
    assert_eq!(back.hour, 12);
    assert_eq!(back.minute, 0);
    assert!(back.second.abs() < 0.001, "second: {}", back.second);
}
```

- [ ] **Step 4: Run the new tests against the unmutated crate**

Run: `cargo nextest run -p pleiades-time calendar`
Expected: 9 tests PASS. (These characterize correct behavior; their "red" evidence is the mutant baseline. A failure here means a pinned expectation is wrong — stop and re-derive, do not adjust the assertion to match the code.)

- [ ] **Step 5: Verify 0 surviving mutants**

Run: `cargo mutants -p pleiades-time --test-tool nextest --test-workspace=false --file crates/pleiades-time/src/calendar.rs -o target/mutants-calendar`
Expected: summary line ends `0 missed` (baseline was 9 missed).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-time/src/calendar.rs crates/pleiades-time/src/calendar/tests.rs
git commit -m "test(time): FU-9 calendar.rs mutant triage (9 -> 0)"
```

---

### Task 2: `deltat.rs` (10 → 0)

**Files:**
- Modify: `crates/pleiades-time/src/deltat.rs` (replace lines 107–151, the inline `#[cfg(test)] mod tests { ... }` block, with a module declaration)
- Create: `crates/pleiades-time/src/deltat/tests.rs`

**Interfaces:**
- Consumes: `delta_t`, `DeltaTQuality`, `fnv1a64`, `DELTA_T_CSV`, `DELTA_T_CSV_CHECKSUM`, `OBSERVED_THROUGH_JD` via `use super::*;`.
- Produces: the pinned extrapolation reference (124.4632 s at t = 80) that Task 4's future-UTC test builds on (same polynomial, different epoch).

- [ ] **Step 1: Relocate the inline test module**

In `deltat.rs`, delete the `#[cfg(test)] mod tests { ... }` block (lines 107–151) and put in its place:

```rust
#[cfg(test)]
mod tests;
```

Create `crates/pleiades-time/src/deltat/tests.rs` with `use super::*;` followed by the four existing tests (`pinned_checksum`, `observed_spot_values`, `boundary_at_observed_through_jd`, `future_is_predicted`) moved verbatim.

- [ ] **Step 2: Verify the relocation is behavior-preserving**

Run: `cargo nextest run -p pleiades-time deltat`
Expected: all 4 existing tests PASS.

- [ ] **Step 3: Add the new test** (append to `deltat/tests.rs`)

```rust
#[test]
fn extrapolated_delta_t_matches_published_polynomial() {
    // JD 2480765.0 = 2451545 + 365.25 * 80 exactly (representable), so
    // decimal_year is exactly 2080.0 and t = 80. Espenak-Meeus 2005-2050
    // polynomial evaluated outside the code (see the design doc's
    // Appendix script): 62.92 + 0.32217*80 + 0.005589*80*80 = 124.4632.
    // Smallest mutant displacement at t = 80 is 25.77 s (spec §4.2), a
    // ~2.6e10x margin over the 1e-9 s tolerance.
    let (dt, q) = delta_t(2_480_765.0).unwrap();
    assert_eq!(q, DeltaTQuality::Predicted);
    assert!((dt - 124.4632).abs() < 1e-9, "got {dt}");
}
```

- [ ] **Step 4: Run the new test against the unmutated crate**

Run: `cargo nextest run -p pleiades-time deltat`
Expected: 5 tests PASS. (On failure: re-derive, never re-pin from the code's output.)

- [ ] **Step 5: Verify 0 surviving mutants**

Run: `cargo mutants -p pleiades-time --test-tool nextest --test-workspace=false --file crates/pleiades-time/src/deltat.rs -o target/mutants-deltat`
Expected: summary line ends `0 missed` (baseline was 10 missed).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-time/src/deltat.rs crates/pleiades-time/src/deltat/tests.rs
git commit -m "test(time): FU-9 deltat.rs mutant triage (10 -> 0)"
```

---

### Task 3: `tdb.rs` (9 → 0)

**Files:**
- Modify: `crates/pleiades-time/src/tdb.rs` (replace lines 16–30, the inline `#[cfg(test)] mod tests { ... }` block, with a module declaration)
- Create: `crates/pleiades-time/src/tdb/tests.rs`

**Interfaces:**
- Consumes: `tdb_minus_tt_seconds` via `use super::*;`.
- Produces: the two pinned USNO literals Task 4's TDB-step test builds on (same formula, different epoch).

- [ ] **Step 1: Relocate the inline test module**

In `tdb.rs`, delete the `#[cfg(test)] mod tests { ... }` block (lines 16–30) and put in its place:

```rust
#[cfg(test)]
mod tests;
```

Create `crates/pleiades-time/src/tdb/tests.rs` with `use super::*;` followed by the existing `bounded_below_two_milliseconds` test moved verbatim (it keeps the magnitude-bound intent; spec §4.3 explains why it can never kill a phase mutant on its own).

- [ ] **Step 2: Verify the relocation is behavior-preserving**

Run: `cargo nextest run -p pleiades-time tdb`
Expected: all matched tests PASS — the substring filter matches
`tdb::tests::bounded_below_two_milliseconds` plus convert's
`tdb_differs_from_tt_sub_millisecond` (2 tests at this point).

- [ ] **Step 3: Add the new test** (append to `tdb/tests.rs`)

```rust
#[test]
fn tdb_minus_tt_matches_usno_formula_at_pinned_epochs() {
    // USNO two-term model evaluated outside the code with matching
    // operation order (design doc Appendix script). The output is bounded
    // below 2 ms by construction for ANY g, so only value-pinning can
    // constrain the phase; each epoch kills all 9 baseline survivors
    // alone (smallest displacement 2.04e-6 s vs the 1e-9 s tolerance,
    // which in turn sits far above cross-libm sin differences of ~1 ulp).
    let v1 = tdb_minus_tt_seconds(2_451_645.0);
    assert!((v1 - 0.001_645_689_159_645_154_8).abs() < 1e-9, "got {v1}");
    let v2 = tdb_minus_tt_seconds(2_446_895.5);
    assert!((v2 - 0.001_649_315_495_175_222_8).abs() < 1e-9, "got {v2}");
}
```

- [ ] **Step 4: Run the new test against the unmutated crate**

Run: `cargo nextest run -p pleiades-time tdb`
Expected: all matched tests PASS (3 at this point: the two `tdb::tests`
tests plus convert's `tdb_differs_from_tt_sub_millisecond`, which the
substring filter also matches).

- [ ] **Step 5: Verify 0 surviving mutants**

Run: `cargo mutants -p pleiades-time --test-tool nextest --test-workspace=false --file crates/pleiades-time/src/tdb.rs -o target/mutants-tdb`
Expected: summary line ends `0 missed` (baseline was 9 missed).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-time/src/tdb.rs crates/pleiades-time/src/tdb/tests.rs
git commit -m "test(time): FU-9 tdb.rs mutant triage (9 -> 0)"
```

---

### Task 4: `convert.rs` (16 → 0)

**Files:**
- Modify: `crates/pleiades-time/src/convert.rs` (replace lines 268–371, the inline `#[cfg(test)] mod tests { ... }` block, with a module declaration)
- Create: `crates/pleiades-time/src/convert/tests.rs`

**Interfaces:**
- Consumes: `to_terrestrial`, `tt_from_utc_civil`, `tdb_from_utc_civil`, `tt_from_ut1_civil`, `ut1_jd_from_tt`, `ConversionPath`, `ConversionQuality`, `ConversionProvenance`, the private `finite`, `CivilDateTime`, `CivilTimeError`, `TimeScale`, `SECONDS_PER_DAY` — all via `use super::*;` (convert.rs's own `use` lines are visible to the child test module).
- Produces: nothing consumed by later tasks.

- [ ] **Step 1: Relocate the inline test module**

In `convert.rs`, delete the `#[cfg(test)] mod tests { ... }` block (lines 268–371) and put in its place:

```rust
#[cfg(test)]
mod tests;
```

Create `crates/pleiades-time/src/convert/tests.rs` with `use super::*;` followed by the eleven existing tests (`utc_modern_is_exact`, `ut1_historical_is_observed`, `future_utc_is_predicted`, `pre_1972_utc_is_rejected`, `outside_window_is_rejected`, `bad_target_scale_is_rejected`, `end_of_2100_is_accepted`, `start_of_2101_ut1_is_rejected`, `start_of_2101_utc_is_rejected`, `tdb_differs_from_tt_sub_millisecond`, `ut1_is_earlier_than_tt_by_delta_t`) moved verbatim.

- [ ] **Step 2: Verify the relocation is behavior-preserving**

Run: `cargo nextest run -p pleiades-time convert`
Expected: all 11 existing tests PASS.

- [ ] **Step 3: Add the five new tests** (append to `convert/tests.rs`)

```rust
#[test]
fn display_vocabulary_is_stable() {
    // The diagnostic vocabulary is release-facing; pin every variant.
    assert_eq!(ConversionPath::UtcLeapSecond.to_string(), "utc-leap-second");
    assert_eq!(ConversionPath::Ut1DeltaT.to_string(), "ut1-delta-t");
    assert_eq!(
        ConversionPath::FutureExtrapolated.to_string(),
        "future-extrapolated"
    );
    assert_eq!(ConversionQuality::Exact.to_string(), "exact");
    assert_eq!(ConversionQuality::Observed.to_string(), "observed");
    assert_eq!(ConversionQuality::Predicted.to_string(), "predicted");

    let exact = ConversionProvenance {
        path: ConversionPath::UtcLeapSecond,
        quality: ConversionQuality::Exact,
        delta_t_seconds: None,
        tai_minus_utc: Some(37),
        sources: "test",
    };
    assert_eq!(
        exact.summary_line(),
        "civil-time path=utc-leap-second quality=exact delta_t=n/a tai_minus_utc=37s"
    );
    let observed = ConversionProvenance {
        path: ConversionPath::Ut1DeltaT,
        quality: ConversionQuality::Observed,
        delta_t_seconds: Some(63.8),
        tai_minus_utc: None,
        sources: "test",
    };
    assert_eq!(
        observed.summary_line(),
        "civil-time path=ut1-delta-t quality=observed delta_t=63.800s tai_minus_utc=n/a"
    );
    assert_eq!(
        observed.to_string(),
        "civil-time path=ut1-delta-t quality=observed delta_t=63.800s tai_minus_utc=n/a"
    );
}

#[test]
fn finite_guard_fails_closed_on_non_finite() {
    // White-box: no public input can reach this guard (the 1900-2101
    // window bounds jd and dT is bounded ~153 s inside it — spec §4.1
    // group B, overflow lens checked), so its fail-closed contract is
    // asserted directly.
    assert!(finite(f64::NAN).is_err());
    assert!(finite(f64::INFINITY).is_err());
    assert!(finite(f64::NEG_INFINITY).is_err());
    assert!(finite(0.0).is_ok());
}

#[test]
fn utc_at_exact_leap_epoch_is_exact() {
    // 1972-01-01 00:00:00 UTC is exactly LEAP_EPOCH_JD (2441317.5): the
    // first instant of the leap-second era belongs to it, not to the
    // pre-1972 rejection. jd_tt = 2441317.5 + (10 + 32.184)/86400,
    // computed outside the code.
    let out = tt_from_utc_civil(CivilDateTime::new(1972, 1, 1, 0, 0, 0.0)).unwrap();
    assert_eq!(out.provenance.path, ConversionPath::UtcLeapSecond);
    assert_eq!(out.provenance.quality, ConversionQuality::Exact);
    assert_eq!(out.provenance.tai_minus_utc, Some(10));
    let jd = out.instant.julian_day.days();
    assert!((jd - 2_441_317.500_488_240_7).abs() < 1e-9, "got {jd}");
}

#[test]
fn future_utc_and_ut1_jd_values_match_hand_computation() {
    // Future-UTC path: 2090-06-01 00:00 UTC -> jd_civil 2484568.5 (past
    // the leap table's VALID_THROUGH_JD, inside the support window).
    // dT = Espenak-Meeus polynomial at decimal_year(2484568.5), evaluated
    // outside the code: 137.73624952070443 s. Smallest mutant
    // displacement on this path is 3.19e-3 days (spec §4.1 group D).
    let out = tt_from_utc_civil(CivilDateTime::new(2090, 6, 1, 0, 0, 0.0)).unwrap();
    assert_eq!(out.provenance.path, ConversionPath::FutureExtrapolated);
    let dt = out.provenance.delta_t_seconds.unwrap();
    assert!((dt - 137.736_249_520_704_43).abs() < 1e-9, "dt {dt}");
    let jd = out.instant.julian_day.days();
    assert!((jd - 2_484_568.501_594_169_5).abs() < 1e-9, "jd_tt {jd}");

    // UT1 path: 1955-06-15 00:00 UT1 -> jd_civil 2435273.5. dT hand-
    // interpolated between the committed 1950 (29.1 s) and 1960 (33.2 s)
    // decade nodes: 31.334934976043815 s. Smallest mutant displacement
    // on this path is 7.25e-4 days.
    let out = tt_from_ut1_civil(CivilDateTime::new(1955, 6, 15, 0, 0, 0.0)).unwrap();
    assert_eq!(out.provenance.quality, ConversionQuality::Observed);
    let dt = out.provenance.delta_t_seconds.unwrap();
    assert!((dt - 31.334_934_976_043_815).abs() < 1e-9, "dt {dt}");
    let jd = out.instant.julian_day.days();
    assert!((jd - 2_435_273.500_362_673).abs() < 1e-9, "jd_tt {jd}");
}

#[test]
fn tdb_target_applies_positive_periodic_term() {
    // 2000-04-01 sits near the annual peak of TDB - TT (g ~ 87 deg),
    // where the USNO term is +1.6569e-3 s (evaluated outside the code at
    // jd_tt = 2451635.5 + 64.184/86400). The SIGNED assertion pins what
    // the .abs() bound test cannot: the +/- mutant lands at -1.6569e-3 s,
    // a 3.3e-3 s displacement vs the 5e-5 s tolerance (which budgets
    // half-ulp(JD) ~ 2.0e-5 s of JD-grid quantization on the stored TDB
    // day; the observable diff computes to 1.64956e-3 s, 7.3e-6 s from
    // the formula value — inside budget).
    let civil = CivilDateTime::new(2000, 4, 1, 0, 0, 0.0);
    let tt = tt_from_utc_civil(civil).unwrap();
    assert_eq!(tt.provenance.tai_minus_utc, Some(32));
    let jd_tt = tt.instant.julian_day.days();
    let jd_tdb = tdb_from_utc_civil(civil).unwrap().instant.julian_day.days();
    let diff_s = (jd_tdb - jd_tt) * SECONDS_PER_DAY;
    assert!(
        (diff_s - 0.001_656_892_188_342_611_6).abs() < 5e-5,
        "TDB-TT {diff_s}s"
    );
}
```

- [ ] **Step 4: Run the new tests against the unmutated crate**

Run: `cargo nextest run -p pleiades-time convert`
Expected: 16 tests PASS. (On failure: re-derive the literal with the Appendix script; never re-pin from the code's output.)

- [ ] **Step 5: Verify 0 surviving mutants**

Run: `cargo mutants -p pleiades-time --test-tool nextest --test-workspace=false --file crates/pleiades-time/src/convert.rs -o target/mutants-convert`
Expected: summary line ends `0 missed` (baseline was 16 missed).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-time/src/convert.rs crates/pleiades-time/src/convert/tests.rs
git commit -m "test(time): FU-9 convert.rs mutant triage (16 -> 0)"
```

---

### Task 5: Follow-ups progress entry

**Files:**
- Modify: `docs/follow-ups.md` (FU-9 section: append a progress entry after the 2026-07-21 precession+lighttime entry, i.e. after the line ending `(zodiac.rs 12, time.rs 10, and the small tail).`)

**Interfaces:**
- Consumes: the four `0 missed` results from Tasks 1–4.
- Produces: the corrected remaining-slices statement future slices start from.

- [ ] **Step 1: Append the progress entry**

Insert into `docs/follow-ups.md`, inside FU-9, immediately after the precession+lighttime progress paragraph (before the closing `---` of the FU-9 section):

```markdown
**Progress (2026-07-21) — `pleiades-time` non-sidereal (`calendar.rs` +
`deltat.rs` + `tdb.rs` + `convert.rs`):** triaged from `9 + 10 + 9 + 16`
→ `0` surviving mutants (spec/plan:
`docs/superpowers/specs/2026-07-21-fu9-time-mutant-triage-design.md`),
closing out `pleiades-time` entirely. Scope note: the previous entries'
remaining-slices line listed only `convert.rs`/`deltat.rs`/`tdb.rs` —
`calendar.rs` (9 survivors in the baseline notes) was omitted by
transcription oversight; this slice covers it. The first four-file
slice, and **tests-only** like refraction/topocentric/sidereal/
precession: the only source edits were relocating the four inline test
modules to `src/<module>/tests.rs`. All four baselines confirmed by the
authoritative per-file commands (calendar: `130 tested, 9 missed, 120
caught, 1 unviable`; deltat: `65 tested, 10 missed, 52 caught, 3
unviable`; tdb: `17 tested, 9 missed, 8 caught`; convert: `47 tested,
16 missed, 23 caught, 8 unviable`), all matching the whole-workspace
figures exactly. Root causes: diagnostics never string-asserted and JD
values never pinned on either ΔT path (`convert.rs`); a `dt > 69.0`
bound as the only extrapolation test (`deltat.rs`); a magnitude-bound
test that can never kill a phase mutant because the USNO term is <2 ms
by construction for any g (`tdb.rs`); and coincidence-degenerate test
dates — at 1987, `floor(alpha/4) == alpha % 4` — plus no January
(e=14), February (e=15), or negative/≥61-second coverage
(`calendar.rs`, where the surviving `||`→`&&` re-associates by
precedence to `A || (B && C)`, verified by hand-applying the mutant:
it silently accepts `second = -1.0` and `61.5`). Kills: pinned literals
from a Python mirror of the published formulas (Espenak–Meeus at
exactly t = 80 via representable JD 2480765.0, margin ~2.6e10×; USNO
two-term at two epochs, min displacement 2.04e-6 s vs 1e-9 s
tolerance), hand-interpolated ΔT table references, a signed TDB−TT
assertion near the annual peak (the topocentric slice's sign-free
`.abs()` lesson), exact leap-epoch boundary acceptance, full six-field
`from_julian_day` literals at 2100-01-01 (alpha=16, e=14) and
2000-02-29 12:00 (e=15, month==2), and direct white-box fail-closed
tests of the `finite` guard (unreachable via the bounded public API —
overflow lens checked — so tested at the seam, per the `apparent.rs`
private-primitive precedent). **No documented residual this slice** —
a genuine `0` across all four files; no equivalent-mutant candidates
surfaced at design time (like sidereal). No parity gate was touched;
the tier stays report-only; `mise run ci` is green. **Remaining
slices:** `pleiades-types` only (`zodiac.rs` 12, `time.rs` 10,
`time_range.rs` 4, and the small tail) — the final slice of the FU-9
baseline.
```

- [ ] **Step 2: Verify the docs render/state**

Run: `grep -n "closing out .pleiades-time. entirely" docs/follow-ups.md`
Expected: one hit inside the FU-9 section.

- [ ] **Step 3: Commit**

```bash
git add docs/follow-ups.md
git commit -m "docs(follow-ups): record FU-9 pleiades-time triage (44 -> 0, crate complete)"
```

---

### Task 6: Workspace verification

**Files:** none (verification only).

**Interfaces:**
- Consumes: everything above.
- Produces: the green-CI evidence required before the finishing-a-development-branch flow.

- [ ] **Step 1: Formatting and lints**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: both clean, no output/warnings.

- [ ] **Step 2: Full blocking CI tier**

Run: `mise run ci`
Expected: green (exit 0). If any stage fails, fix within this branch's scope (tests-only) before proceeding.

- [ ] **Step 3: Confirm acceptance criteria (spec §8)**

Run: `git diff main --stat -- crates/`
Expected: exactly eight `crates/` paths — the four module files (relocation-only diffs) and the four new `tests.rs` files. No `validate-*` file, no `#[mutants::skip]` anywhere (`grep -rn "mutants::skip" crates/pleiades-time/` → no hits).

---

## Appendix: reference mirror script (spec §6)

Independent reference for every pinned literal and per-mutant margin in
this plan and the spec's §4 tables. Written from the published sources
(Espenak–Meeus ΔT polynomial, USNO TDB−TT two-term model, Meeus ch. 7
calendar algorithm, IERS leap table first row + definitional
TT−TAI = 32.184 s), evaluated in f64 with the crate's operation order.
Not committed to the repo; reproduced here per the predecessor slices'
convention.

```python
import math

def extrapolate(year):                     # Espenak-Meeus 2005-2050 form
    t = year - 2000.0
    return 62.92 + 0.32217 * t + 0.005589 * t * t

def decimal_year(jd):
    return 2000.0 + (jd - 2451545.0) / 365.25

def tdb_minus_tt(jd):                      # USNO low-precision model
    g_deg = 357.53 + 0.9856003 * (jd - 2451545.0)
    g = math.radians(g_deg)
    return 0.001658 * math.sin(g) + 0.000014 * math.sin(2.0 * g)

def to_jd(y, mo, d, h, mi, s):             # Meeus ch. 7, proleptic Gregorian
    day_frac = d + (h + mi / 60.0 + s / 3600.0) / 24.0
    if mo <= 2:
        y, mo = y - 1, mo + 12
    a = math.floor(y / 100.0)
    b = 2.0 - a + math.floor(a / 4.0)
    return (math.floor(365.25 * (y + 4716.0))
            + math.floor(30.6001 * (mo + 1.0)) + day_frac + b - 1524.5)

# deltat: t = 80 exactly
print(repr(extrapolate(decimal_year(2451545.0 + 365.25 * 80))))  # 124.4632

# tdb: two pinned epochs
print(repr(tdb_minus_tt(2451645.0)))   # 0.0016456891596451548
print(repr(tdb_minus_tt(2446895.5)))   # 0.0016493154951752228

# convert: leap-epoch, future-UTC, UT1, TDB-step
print(repr(2441317.5 + (10.0 + 32.184) / 86400.0))  # 2441317.5004882407
jd = to_jd(2090, 6, 1, 0, 0, 0.0)                   # 2484568.5
dt = extrapolate(decimal_year(jd))                  # 137.73624952070443
print(repr(jd + dt / 86400.0))                      # 2484568.5015941695
jd = to_jd(1955, 6, 15, 0, 0, 0.0)                  # 2435273.5
frac = (decimal_year(jd) - 1950.0) / 10.0
dt = 29.1 + frac * (33.2 - 29.1)                    # 31.334934976043815
print(repr(jd + dt / 86400.0))                      # 2435273.500362673
jd_tt = to_jd(2000, 4, 1, 0, 0, 0.0) + (32.0 + 32.184) / 86400.0
print(repr(tdb_minus_tt(jd_tt)))                    # 0.0016568921883426116
```

The per-mutant displacement tables (spec §4.1 D, §4.2, §4.3) and the
calendar kill matrix (spec §4.4) come from the same mirror with each
mutation applied in turn; the margin minima are 7.25e-4 days (convert
group D), 25.77 s (deltat), 2.04e-6 s (tdb), and categorical
integer-field mismatches (calendar).
