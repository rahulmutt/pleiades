# Civil Datetime From a TT/TDB Instant (issue #87) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `pleiades_time::from_terrestrial`, the inverse of `to_terrestrial`, and fix the forward conversion's handling of an inserted leap second so the round trip holds through it.

**Architecture:** The inverse lives in a new `convert/inverse.rs`. It quantizes the TT instant once to an integer millisecond count, then does the leap-second lookup with integer comparisons on the TAI axis; the ΔT eras (UT1, and UTC past the leap horizon) use a three-step fixed-point solve of `civil + ΔT(civil) = TT`. The forward fix converts `23:59:60.x` as the previous second plus one second and rejects `:60` anywhere it is not a real inserted leap second.

**Tech Stack:** Rust (stable, toolchain from `mise.toml`), `pleiades-types`, `proptest` (already a workspace dependency), `cargo nextest`.

**Spec:** `docs/superpowers/specs/2026-10-02-civil-from-terrestrial-design.md`. Read it before starting; this plan argues from it.

## Global Constraints

- Support window is `[SUPPORT_START_JD, SUPPORT_END_JD)` = `[2415020.5, 2488434.5)`, checked on the civil Julian day.
- No new `CivilTimeError` variants.
- No new runtime dependencies. `proptest = { workspace = true }` is added to `pleiades-time` as a dev-dependency only.
- Round trip civil → TT/TDB → civil holds to 1 ms; millisecond-aligned inputs return identical fields.
- An instant inside an inserted leap second is returned as `23:59:60.x` with the `TAI − UTC` in force before the insertion.
- `ut1_jd_from_tt` and `CivilDateTime::to_julian_day`'s accepted range are unchanged.
- No `unwrap`/`expect`/panic in library paths. Tests may unwrap.
- Run `cargo fmt --all` before every commit. Code must pass `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Tests stay in co-located test files (`<module>/tests.rs`); shared setup goes in `convert/test_support.rs`.
- Do not edit sources or commit while a background test run is in progress.
- The forward-fix commit is marked breaking (`fix(time)!:` with a `BREAKING CHANGE:` footer). Do not bump versions by hand; release-plz does it.

## Review Focus

Inputs the spec implies but does not spell out as tests. Each has a test in the named task.

1. **A finite but absurd Julian day (`1e300`, `-1e300`, `f64::MAX`).** Expected: `BeyondHorizon`, not an integer-overflow panic in the millisecond conversion. (Task 3, `absurd_julian_days_are_beyond_horizon`)
2. **A non-finite instant (NaN, ±infinity).** Expected: `NonFiniteOffset`. (Task 3, `non_finite_instants_are_rejected`)
3. **An instant half a millisecond below midnight on an ordinary day.** Expected: the next day's `00:00:00.000`, never `23:59:60.000` or `24:00`. (Task 3, `sub_millisecond_below_midnight_rounds_into_the_next_day`)
4. **The exact edges of a leap second (`23:59:60.000`, `23:59:60.999`, next `00:00:00.000`).** Expected: each round-trips to identical fields. (Task 3, `leap_second_edges_round_trip`)
5. **Feeding an inverse result straight back into the forward conversion.** Expected: always accepted, and within 1 ms of the starting instant. (Task 4, `utc_inverse_is_monotonic_and_reenters_the_forward`)

Forward-side: `second = 61.0` and `second = NaN` must still be `InvalidCivilDate { field: "second" }` after the fix. (Task 2, `out_of_range_seconds_are_still_rejected`)

## File Structure

| File | Responsibility |
|---|---|
| `crates/pleiades-time/src/leap.rs` (modify) | Leap table. Gains `pub(crate) table()` visibility and `is_insertion_day_end`. |
| `crates/pleiades-time/src/convert.rs` (modify) | Shared provenance types and the forward path. Gains the leap-second split and `mod inverse`. |
| `crates/pleiades-time/src/convert/inverse.rs` (create) | The inverse: `CivilConversion`, `from_terrestrial`, four conveniences. |
| `crates/pleiades-time/src/convert/inverse/tests.rs` (create) | Unit and property tests for the inverse. |
| `crates/pleiades-time/src/convert/test_support.rs` (create) | Test-only: the list of insertions derived from the leap table. |
| `crates/pleiades-time/src/convert/tests.rs` (modify) | Forward regression and rejection tests. |
| `crates/pleiades-time/src/calendar.rs` (modify) | Rustdoc note on `:60` aliasing only. |
| `crates/pleiades-time/src/lib.rs` (modify) | Re-exports and crate-level docs. |
| `crates/pleiades-time/Cargo.toml` (modify) | `proptest` dev-dependency. |
| `crates/pleiades-core/src/lib.rs` (modify) | Re-export `CivilConversion`, `from_terrestrial`. |
| `README.md`, `crates/pleiades-time/README.md`, `docs/time-observer-policy.md`, `docs/follow-ups.md`, the spec (modify) | Documentation. |

All commands run from `/workspace`.

---

### Task 1: Leap-table helpers

**Files:**
- Modify: `crates/pleiades-time/src/leap.rs`

**Interfaces:**
- Consumes: nothing new.
- Produces:
  - `pub(crate) fn table() -> Result<&'static [(f64, i32)], CivilTimeError>` (existing function, visibility widened). Rows are `(effective_jd_utc, tai_minus_utc)`, ascending.
  - `pub(crate) fn is_insertion_day_end(jd_next_midnight: f64) -> Result<bool, CivilTimeError>`.

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module at the bottom of `crates/pleiades-time/src/leap.rs`:

```rust
    #[test]
    fn every_row_after_the_first_inserts_exactly_one_second() {
        // The inverse conversion assumes each insertion is a single positive
        // leap second. A negative or multi-second step must fail here before
        // it can be mis-converted.
        let rows = table().unwrap();
        assert_eq!(rows.len(), 28);
        for pair in rows.windows(2) {
            assert!(pair[1].0 > pair[0].0, "rows must ascend: {pair:?}");
            assert_eq!(pair[1].1 - pair[0].1, 1, "not a +1 s step: {pair:?}");
        }
    }

    #[test]
    fn insertion_day_end_matches_only_real_insertions() {
        // 2017-01-01 00:00 UTC: the day after the 2016-12-31 leap second.
        assert!(is_insertion_day_end(2_457_754.5).unwrap());
        // 1972-07-01 00:00 UTC: the first insertion.
        assert!(is_insertion_day_end(2_441_499.5).unwrap());
        // 1972-01-01 00:00 UTC is the table's epoch, not an insertion.
        assert!(!is_insertion_day_end(2_441_317.5).unwrap());
        // An ordinary midnight.
        assert!(!is_insertion_day_end(2_457_755.5).unwrap());
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo nextest run -p pleiades-time leap::`
Expected: compile error, `cannot find function is_insertion_day_end`.

- [ ] **Step 3: Implement**

In `crates/pleiades-time/src/leap.rs`, change `fn table()` to `pub(crate) fn table()` and give it a doc comment:

```rust
/// The parsed leap-second rows `(effective_jd_utc, tai_minus_utc)`, ascending.
/// The first row is the 1972 epoch; every later row is an inserted leap second.
pub(crate) fn table() -> Result<&'static [(f64, i32)], CivilTimeError> {
    LEAP_ROWS
        .get_or_init(parse_table)
        .as_deref()
        .map_err(|e| *e)
}
```

Add after `tai_minus_utc`:

```rust
/// Whether the UTC midnight `jd_next_midnight` immediately follows an
/// inserted leap second. Effective days are exact `x.5` Julian days, so the
/// comparison is exact for a midnight built by `CivilDateTime::to_julian_day`.
pub(crate) fn is_insertion_day_end(jd_next_midnight: f64) -> Result<bool, CivilTimeError> {
    Ok(table()?
        .iter()
        .skip(1)
        .any(|&(effective, _)| effective == jd_next_midnight))
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo nextest run -p pleiades-time leap::`
Expected: PASS, 6 tests.

- [ ] **Step 5: Format and commit**

`is_insertion_day_end` has no library caller until Task 2, so the library
target reports it as dead code and strict clippy fails at this commit. That is
expected: do not add `#[allow(dead_code)]`. Clippy runs clean again at the end
of Task 2.

```bash
cargo fmt --all
git add crates/pleiades-time/src/leap.rs
git commit -m "feat(time): leap-table insertion lookup for leap-second handling (#87)"
```

---

### Task 2: Forward conversion handles the inserted leap second

**Files:**
- Create: `crates/pleiades-time/src/convert/test_support.rs`
- Modify: `crates/pleiades-time/src/convert.rs`
- Modify: `crates/pleiades-time/src/calendar.rs` (rustdoc only)
- Test: `crates/pleiades-time/src/convert/tests.rs`

**Interfaces:**
- Consumes: `leap::table()`, `leap::is_insertion_day_end(jd) -> Result<bool, CivilTimeError>` from Task 1.
- Produces:
  - `to_terrestrial` behaviour: UTC `23:59:60.x` on an insertion day maps to the TT of `23:59:59.x` plus one second, with the pre-insertion `tai_minus_utc`; any other `second` in `[60, 61)` is `InvalidCivilDate { field: "second" }`.
  - Test-only, in `crate::convert::test_support`:
    - `pub(crate) struct Insertion { pub last_day: CivilDateTime, pub next_day: CivilDateTime, pub offset_before: i32 }` (`last_day`/`next_day` are dates at `00:00:00`).
    - `pub(crate) fn insertions() -> Vec<Insertion>` (27 entries, ascending).
    - `pub(crate) fn at(date: CivilDateTime, hour: u8, minute: u8, second: f64) -> CivilDateTime`.

- [ ] **Step 1: Create the shared test helper**

Create `crates/pleiades-time/src/convert/test_support.rs`:

```rust
//! Shared setup for the forward and inverse conversion tests.

use pleiades_types::JulianDay;

use crate::calendar::CivilDateTime;
use crate::leap;

/// One inserted leap second, derived from the leap table.
pub(crate) struct Insertion {
    /// The UTC day that ends with the leap second, at `00:00:00`.
    pub last_day: CivilDateTime,
    /// The following UTC day, at `00:00:00`.
    pub next_day: CivilDateTime,
    /// `TAI − UTC` in force up to and including the leap second.
    pub offset_before: i32,
}

/// Every insertion in the leap table, ascending.
pub(crate) fn insertions() -> Vec<Insertion> {
    leap::table()
        .unwrap()
        .windows(2)
        .map(|pair| Insertion {
            last_day: CivilDateTime::from_julian_day(JulianDay::from_days(pair[1].0 - 1.0)),
            next_day: CivilDateTime::from_julian_day(JulianDay::from_days(pair[1].0)),
            offset_before: pair[0].1,
        })
        .collect()
}

/// `date` with its time of day replaced.
pub(crate) fn at(date: CivilDateTime, hour: u8, minute: u8, second: f64) -> CivilDateTime {
    CivilDateTime::new(date.year, date.month, date.day, hour, minute, second)
}
```

In `crates/pleiades-time/src/convert.rs`, add above the existing `#[cfg(test)] mod tests;` at the bottom:

```rust
#[cfg(test)]
mod test_support;
```

- [ ] **Step 2: Write the failing tests**

Append to `crates/pleiades-time/src/convert/tests.rs`:

```rust
use super::test_support::{at, insertions};

fn seconds_between(earlier: &CivilInstant, later: &CivilInstant) -> f64 {
    (later.instant.julian_day.days() - earlier.instant.julian_day.days()) * SECONDS_PER_DAY
}

#[test]
fn insertion_helper_matches_known_dates() {
    // Anchors the table-derived helper to dates known independently.
    let all = insertions();
    assert_eq!(all.len(), 27);
    let first = &all[0];
    assert_eq!(
        (first.last_day.year, first.last_day.month, first.last_day.day),
        (1972, 6, 30)
    );
    assert_eq!(first.offset_before, 10);
    let last = &all[26];
    assert_eq!(
        (last.last_day.year, last.last_day.month, last.last_day.day),
        (2016, 12, 31)
    );
    assert_eq!(
        (last.next_day.year, last.next_day.month, last.next_day.day),
        (2017, 1, 1)
    );
    assert_eq!(last.offset_before, 36);
}

#[test]
fn utc_leap_second_sits_between_its_neighbours() {
    // 2016-12-31T23:59:60.5 is one second after 23:59:59.5 and one second
    // before 2017-01-01T00:00:00.5. Before the fix it aliased the latter.
    let before = tt_from_utc_civil(CivilDateTime::new(2016, 12, 31, 23, 59, 59.5)).unwrap();
    let inside = tt_from_utc_civil(CivilDateTime::new(2016, 12, 31, 23, 59, 60.5)).unwrap();
    let after = tt_from_utc_civil(CivilDateTime::new(2017, 1, 1, 0, 0, 0.5)).unwrap();
    // A Julian day near 2.46e6 resolves ~40 µs; 2e-4 s clears the noise.
    assert!((seconds_between(&before, &inside) - 1.0).abs() < 2e-4);
    assert!((seconds_between(&inside, &after) - 1.0).abs() < 2e-4);
    assert_eq!(inside.provenance.tai_minus_utc, Some(36));
    assert_eq!(inside.provenance.quality, ConversionQuality::Exact);
    assert_eq!(after.provenance.tai_minus_utc, Some(37));
}

#[test]
fn every_inserted_leap_second_is_one_second_long() {
    for insertion in insertions() {
        let last_ordinary = tt_from_utc_civil(at(insertion.last_day, 23, 59, 59.0)).unwrap();
        let leap = tt_from_utc_civil(at(insertion.last_day, 23, 59, 60.0)).unwrap();
        let midnight = tt_from_utc_civil(at(insertion.next_day, 0, 0, 0.0)).unwrap();
        assert!((seconds_between(&last_ordinary, &leap) - 1.0).abs() < 2e-4);
        assert!((seconds_between(&leap, &midnight) - 1.0).abs() < 2e-4);
        assert_eq!(leap.provenance.tai_minus_utc, Some(insertion.offset_before));
        assert_eq!(
            midnight.provenance.tai_minus_utc,
            Some(insertion.offset_before + 1)
        );
    }
}

#[test]
fn leap_second_converts_to_tdb_too() {
    let tt = tt_from_utc_civil(CivilDateTime::new(2016, 12, 31, 23, 59, 60.5)).unwrap();
    let tdb = tdb_from_utc_civil(CivilDateTime::new(2016, 12, 31, 23, 59, 60.5)).unwrap();
    assert_eq!(tdb.instant.scale, TimeScale::Tdb);
    let gap = seconds_between(&tt, &tdb);
    assert!(gap.abs() < 2e-3, "TDB − TT is bounded by 2 ms, got {gap}");
}

#[test]
fn second_sixty_is_rejected_where_no_leap_second_was_inserted() {
    let invalid = Err(CivilTimeError::InvalidCivilDate { field: "second" });
    // A day with no insertion.
    assert_eq!(
        tt_from_utc_civil(CivilDateTime::new(2016, 12, 30, 23, 59, 60.0)),
        invalid
    );
    // The right day, the wrong minute.
    assert_eq!(
        tt_from_utc_civil(CivilDateTime::new(2016, 12, 31, 12, 0, 60.0)),
        invalid
    );
    // 1972-01-01 is the table's epoch, not an insertion.
    assert_eq!(
        tt_from_utc_civil(CivilDateTime::new(1971, 12, 31, 23, 59, 60.0)),
        invalid
    );
    // UT1 has no leap seconds.
    assert_eq!(
        tt_from_ut1_civil(CivilDateTime::new(2016, 12, 31, 23, 59, 60.5)),
        invalid
    );
}

#[test]
fn out_of_range_seconds_are_still_rejected() {
    let invalid = Err(CivilTimeError::InvalidCivilDate { field: "second" });
    assert_eq!(
        tt_from_utc_civil(CivilDateTime::new(2016, 12, 31, 23, 59, 61.0)),
        invalid
    );
    assert_eq!(
        tt_from_utc_civil(CivilDateTime::new(2016, 12, 31, 23, 59, f64::NAN)),
        invalid
    );
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo nextest run -p pleiades-time convert::tests`
Expected: `insertion_helper_matches_known_dates`, `leap_second_converts_to_tdb_too` and `out_of_range_seconds_are_still_rejected` PASS. `utc_leap_second_sits_between_its_neighbours` and `every_inserted_leap_second_is_one_second_long` FAIL (the leap second measures 2 s from `:59` and 0 s from midnight). `second_sixty_is_rejected_where_no_leap_second_was_inserted` FAILS (the inputs are accepted).

- [ ] **Step 4: Implement the split**

In `crates/pleiades-time/src/convert.rs`, add above `to_terrestrial`:

```rust
/// Separates an inserted leap second from the datetime that carries it.
///
/// A Julian day cannot express the 86,401st second of a day: `23:59:60.x`
/// aliases the next day's `00:00:00.x`. For a UTC input on a day that ends
/// with an inserted leap second, this returns the same datetime one second
/// earlier plus `true`, so the caller converts `23:59:59.x` under the offset
/// still in force and then adds the second back. Any other `second` in
/// `[60, 61)` is invalid.
fn leap_second_split(
    civil: CivilDateTime,
    source: TimeScale,
) -> Result<(CivilDateTime, bool), CivilTimeError> {
    if !(60.0..61.0).contains(&civil.second) {
        return Ok((civil, false));
    }
    let invalid = CivilTimeError::InvalidCivilDate { field: "second" };
    if source != TimeScale::Utc || civil.hour != 23 || civil.minute != 59 {
        return Err(invalid);
    }
    let midnight = CivilDateTime::new(civil.year, civil.month, civil.day, 0, 0, 0.0);
    let next_midnight_jd = midnight.to_julian_day()?.days() + 1.0;
    if !leap::is_insertion_day_end(next_midnight_jd)? {
        return Err(invalid);
    }
    Ok((
        CivilDateTime {
            second: civil.second - 1.0,
            ..civil
        },
        true,
    ))
}
```

In `to_terrestrial`, replace these two lines:

```rust
    let jd_civil = civil.to_julian_day()?.days();
    let (jd_tt, provenance) = to_tt(jd_civil, source, target)?;
```

with:

```rust
    let (civil, in_leap_second) = leap_second_split(civil, source)?;
    let jd_civil = civil.to_julian_day()?.days();
    let (jd_tt, provenance) = to_tt(jd_civil, source, target)?;
    let jd_tt = if in_leap_second {
        jd_tt + 1.0 / SECONDS_PER_DAY
    } else {
        jd_tt
    };
```

Extend the rustdoc of `to_terrestrial`, inserting before its `# Examples` heading:

```rust
/// # Leap seconds
///
/// A UTC `second` in `[60, 61)` is accepted only at `23:59` on a day that
/// ends with an inserted leap second, and converts to the instant one second
/// after `23:59:59.x` under the `TAI − UTC` in force before the insertion.
/// On any other day or minute, and for UT1 input, it is
/// [`CivilTimeError::InvalidCivilDate`] with `field: "second"`.
///
```

- [ ] **Step 5: Note the aliasing on `to_julian_day`**

In `crates/pleiades-time/src/calendar.rs`, replace the doc comment of `to_julian_day`:

```rust
    /// Converts to a Julian Day using the proleptic-Gregorian Meeus formula.
    ///
    /// The conversion is scale-agnostic and does not consult the leap-second
    /// table: a `second` in `[60, 61)` yields the same Julian day as the first
    /// second of the following minute. Use [`crate::to_terrestrial`] for a UTC
    /// datetime that may fall in an inserted leap second.
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo nextest run -p pleiades-time`
Expected: PASS, all tests in the crate, including the pre-existing ones.

- [ ] **Step 7: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-time --all-targets --all-features -- -D warnings
git add crates/pleiades-time/src/convert.rs crates/pleiades-time/src/convert/test_support.rs crates/pleiades-time/src/convert/tests.rs crates/pleiades-time/src/calendar.rs
git commit -m "fix(time)!: convert an inserted UTC leap second to the correct TT (#87)

23:59:60.x aliased the next day's 00:00:00.x in the Julian day, so it
picked up the post-insertion TAI-UTC and landed one second late.

BREAKING CHANGE: to_terrestrial now rejects a second in [60, 61) with
InvalidCivilDate unless it is UTC 23:59:60.x on a day that ends with an
inserted leap second. Such inputs were previously read as the next minute."
```

---

### Task 3: The inverse conversion

**Files:**
- Create: `crates/pleiades-time/src/convert/inverse.rs`
- Create: `crates/pleiades-time/src/convert/inverse/tests.rs`
- Modify: `crates/pleiades-time/src/convert.rs` (declare the module, re-export)
- Modify: `crates/pleiades-time/src/lib.rs` (re-export, crate docs)

**Interfaces:**
- Consumes:
  - `leap::table() -> Result<&'static [(f64, i32)], CivilTimeError>`, `leap::VALID_THROUGH_JD: f64`, `leap::LEAP_EPOCH_JD: f64`.
  - `deltat::delta_t(jd: f64) -> Result<(f64, DeltaTQuality), CivilTimeError>`, `deltat::OBSERVED_THROUGH_JD: f64`.
  - `tdb::tdb_minus_tt_seconds(jd_tt: f64) -> f64`.
  - From `convert.rs` (private, reachable as `super::`): `SOURCES: &str`, `ConversionPath`, `ConversionQuality`, `ConversionProvenance`, `SUPPORT_START_JD`, `SUPPORT_END_JD`.
  - Test-only: `crate::convert::test_support::{at, insertions, Insertion}`; the fixed forward from Task 2.
- Produces (public, re-exported from the crate root):
  - `pub struct CivilConversion { pub civil: CivilDateTime, pub scale: TimeScale, pub provenance: ConversionProvenance }`
  - `pub fn from_terrestrial(instant: Instant, target: TimeScale) -> Result<CivilConversion, CivilTimeError>`
  - `pub fn utc_civil_from_tt(instant: Instant) -> Result<CivilConversion, CivilTimeError>`, and likewise `utc_civil_from_tdb`, `ut1_civil_from_tt`, `ut1_civil_from_tdb`.
- Produces (private to `inverse`, used by Task 4's tests through `super::`):
  - `const MS_PER_DAY: i64 = 86_400_000`
  - `fn ms_from_jd(jd: f64) -> i64`
  - `fn jd_from_ms(ms: i64) -> f64`
  - `fn civil_from_ms(ms: i64) -> CivilDateTime`

- [ ] **Step 1: Wire the empty module**

Create `crates/pleiades-time/src/convert/inverse.rs` containing only:

```rust
//! Inverse orchestrator: a TT/TDB `Instant` -> civil UTC/UT1 datetime with provenance.

#[cfg(test)]
mod tests;
```

Create `crates/pleiades-time/src/convert/inverse/tests.rs` containing only:

```rust
use super::*;
```

In `crates/pleiades-time/src/convert.rs`, add after the `use` block at the top:

```rust
mod inverse;

pub use inverse::{
    from_terrestrial, ut1_civil_from_tdb, ut1_civil_from_tt, utc_civil_from_tdb,
    utc_civil_from_tt, CivilConversion,
};
```

- [ ] **Step 2: Write the failing unit tests**

Replace `crates/pleiades-time/src/convert/inverse/tests.rs` with:

```rust
use super::*;

use crate::convert::test_support::{at, insertions};
use crate::convert::{tt_from_ut1_civil, tt_from_utc_civil, tdb_from_utc_civil};
use crate::deltat::OBSERVED_THROUGH_JD;

fn tt(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

/// `instant` moved by `seconds`, keeping its scale.
fn shifted(instant: Instant, seconds: f64) -> Instant {
    Instant::new(
        JulianDay::from_days(instant.julian_day.days() + seconds / SECONDS_PER_DAY),
        instant.scale,
    )
}

#[test]
fn millisecond_axis_is_exact_at_midnights_and_round_trips() {
    // JD 2457754.5 is 2017-01-01 00:00: day number 2457755, millisecond 0.
    assert_eq!(ms_from_jd(2_457_754.5), 2_457_755 * MS_PER_DAY);
    assert_eq!(jd_from_ms(2_457_755 * MS_PER_DAY), 2_457_754.5);
    // Noon, plus 1 ms.
    let noon_plus = 2_457_755 * MS_PER_DAY + 43_200_001;
    assert_eq!(ms_from_jd(jd_from_ms(noon_plus)), noon_plus);
    assert_eq!(
        civil_from_ms(noon_plus),
        CivilDateTime::new(2017, 1, 1, 12, 0, 0.001)
    );
    // The last millisecond of a day stays on that day.
    assert_eq!(
        civil_from_ms(2_457_755 * MS_PER_DAY - 1),
        CivilDateTime::new(2016, 12, 31, 23, 59, 59.999)
    );
}

#[test]
fn utc_modern_inverts_exactly() {
    let civil = CivilDateTime::new(2017, 1, 1, 0, 0, 0.0);
    let forward = tt_from_utc_civil(civil).unwrap();
    let back = from_terrestrial(forward.instant, TimeScale::Utc).unwrap();
    assert_eq!(back.civil, civil);
    assert_eq!(back.scale, TimeScale::Utc);
    assert_eq!(back.provenance, forward.provenance);
    assert_eq!(back.provenance.quality, ConversionQuality::Exact);
    assert_eq!(back.provenance.tai_minus_utc, Some(37));
}

#[test]
fn tdb_input_inverts_to_the_same_utc() {
    let civil = CivilDateTime::new(2024, 3, 20, 3, 6, 21.25);
    let forward = tdb_from_utc_civil(civil).unwrap();
    let back = utc_civil_from_tdb(forward.instant).unwrap();
    assert_eq!(back.civil, civil);
    assert_eq!(back.provenance, forward.provenance);
}

#[test]
fn every_insertion_round_trips_through_the_leap_second() {
    for insertion in insertions() {
        let before = insertion.offset_before;
        let samples = [
            (at(insertion.last_day, 23, 59, 59.5), before),
            (at(insertion.last_day, 23, 59, 60.0), before),
            (at(insertion.last_day, 23, 59, 60.5), before),
            (at(insertion.next_day, 0, 0, 0.0), before + 1),
            (at(insertion.next_day, 0, 0, 0.5), before + 1),
        ];
        let mut previous_jd: Option<f64> = None;
        for (civil, offset) in samples {
            let forward = tt_from_utc_civil(civil).unwrap();
            let jd = forward.instant.julian_day.days();
            if let Some(previous) = previous_jd {
                let step = (jd - previous) * SECONDS_PER_DAY;
                assert!((step - 0.5).abs() < 2e-4, "{civil:?}: step {step}");
            }
            previous_jd = Some(jd);
            let back = utc_civil_from_tt(forward.instant).unwrap();
            assert_eq!(back.civil, civil);
            assert_eq!(back.provenance.tai_minus_utc, Some(offset));
            assert_eq!(back.provenance, forward.provenance);
        }
    }
}

#[test]
fn leap_second_edges_round_trip() {
    let edges = [
        CivilDateTime::new(2016, 12, 31, 23, 59, 59.999),
        CivilDateTime::new(2016, 12, 31, 23, 59, 60.0),
        CivilDateTime::new(2016, 12, 31, 23, 59, 60.999),
        CivilDateTime::new(2017, 1, 1, 0, 0, 0.0),
    ];
    for civil in edges {
        let forward = tt_from_utc_civil(civil).unwrap();
        let back = utc_civil_from_tt(forward.instant).unwrap();
        assert_eq!(back.civil, civil);
    }
}

#[test]
fn sub_millisecond_below_midnight_rounds_into_the_next_day() {
    // An ordinary midnight: 0.4 ms before it rounds up to 00:00:00.000 of the
    // new day, never to 23:59:60.000 or 24:00.
    let midnight = CivilDateTime::new(2020, 3, 1, 0, 0, 0.0);
    let forward = tt_from_utc_civil(midnight).unwrap();
    let back = utc_civil_from_tt(shifted(forward.instant, -0.0004)).unwrap();
    assert_eq!(back.civil, midnight);
    // 0.6 ms before it rounds down to the last millisecond of the old day.
    let back = utc_civil_from_tt(shifted(forward.instant, -0.0006)).unwrap();
    assert_eq!(back.civil, CivilDateTime::new(2020, 2, 29, 23, 59, 59.999));
}

#[test]
fn utc_before_1972_is_rejected() {
    let epoch = CivilDateTime::new(1972, 1, 1, 0, 0, 0.0);
    let forward = tt_from_utc_civil(epoch).unwrap();
    assert_eq!(utc_civil_from_tt(forward.instant).unwrap().civil, epoch);
    assert_eq!(
        utc_civil_from_tt(shifted(forward.instant, -0.002)),
        Err(CivilTimeError::UtcBeforeLeapEpoch)
    );
    // 1950 is inside the window but before UTC exists.
    assert_eq!(
        utc_civil_from_tt(tt(2_433_282.5)),
        Err(CivilTimeError::UtcBeforeLeapEpoch)
    );
    // UT1 is defined there.
    assert!(ut1_civil_from_tt(tt(2_433_282.5)).is_ok());
}

#[test]
fn leap_horizon_is_continuous() {
    // 2026-06-30 00:00:00 UTC is the last leap-second-exact instant.
    let horizon = CivilDateTime::new(2026, 6, 30, 0, 0, 0.0);
    let forward = tt_from_utc_civil(horizon).unwrap();
    assert_eq!(forward.provenance.quality, ConversionQuality::Exact);
    let back = utc_civil_from_tt(forward.instant).unwrap();
    assert_eq!(back.civil, horizon);
    assert_eq!(back.provenance, forward.provenance);

    let later = CivilDateTime::new(2026, 6, 30, 0, 0, 1.0);
    let forward = tt_from_utc_civil(later).unwrap();
    assert_eq!(forward.provenance.quality, ConversionQuality::Predicted);
    let back = utc_civil_from_tt(forward.instant).unwrap();
    assert_eq!(back.civil, later);
    assert_eq!(back.provenance.path, ConversionPath::FutureExtrapolated);
    assert_eq!(back.provenance.quality, ConversionQuality::Predicted);
    assert_eq!(back.provenance.tai_minus_utc, None);
    let delta_t = back.provenance.delta_t_seconds.unwrap();
    assert!((delta_t - forward.provenance.delta_t_seconds.unwrap()).abs() < 1e-6);
}

#[test]
fn ut1_inverts_with_the_forward_quality_tiers() {
    for (civil, path, quality) in [
        (
            CivilDateTime::new(1950, 1, 1, 0, 0, 0.0),
            ConversionPath::Ut1DeltaT,
            ConversionQuality::Observed,
        ),
        (
            CivilDateTime::new(2022, 1, 1, 6, 30, 15.5),
            ConversionPath::Ut1DeltaT,
            ConversionQuality::Observed,
        ),
        (
            CivilDateTime::new(2090, 6, 1, 0, 0, 0.0),
            ConversionPath::FutureExtrapolated,
            ConversionQuality::Predicted,
        ),
    ] {
        let forward = tt_from_ut1_civil(civil).unwrap();
        let back = from_terrestrial(forward.instant, TimeScale::Ut1).unwrap();
        assert_eq!(back.civil, civil);
        assert_eq!(back.scale, TimeScale::Ut1);
        assert_eq!(back.provenance.path, path);
        assert_eq!(back.provenance.quality, quality);
        assert_eq!(back.provenance.tai_minus_utc, None);
        let delta_t = back.provenance.delta_t_seconds.unwrap();
        assert!((delta_t - forward.provenance.delta_t_seconds.unwrap()).abs() < 1e-6);
    }
}

#[test]
fn ambiguous_window_at_the_2020_node_resolves_after_the_node() {
    // ΔT steps from 69.4 s to 69.184 s at the 2020 node, so TT values in
    // [node + 69.184 s, node + 69.4 s) have a UT1 preimage on each side.
    // The inverse returns the one at or after the node.
    let instant = tt(OBSERVED_THROUGH_JD + 69.3 / SECONDS_PER_DAY);
    let back = ut1_civil_from_tt(instant).unwrap();
    assert_eq!(back.civil, CivilDateTime::new(2020, 1, 1, 0, 0, 0.116));
    assert!((back.provenance.delta_t_seconds.unwrap() - 69.184).abs() < 1e-9);
}

#[test]
fn results_outside_the_window_are_beyond_horizon() {
    let start = tt_from_ut1_civil(CivilDateTime::new(1900, 1, 1, 0, 0, 0.0)).unwrap();
    assert!(ut1_civil_from_tt(start.instant).is_ok());
    assert!(matches!(
        ut1_civil_from_tt(shifted(start.instant, -1.0)),
        Err(CivilTimeError::BeyondHorizon { .. })
    ));
    let end = tt_from_ut1_civil(CivilDateTime::new(2100, 12, 31, 23, 59, 59.0)).unwrap();
    assert!(ut1_civil_from_tt(end.instant).is_ok());
    assert!(matches!(
        ut1_civil_from_tt(shifted(end.instant, 2.0)),
        Err(CivilTimeError::BeyondHorizon { .. })
    ));
    // Before 1900 a UTC target reports the window, not the 1972 epoch.
    assert!(matches!(
        utc_civil_from_tt(tt(2_396_758.5)),
        Err(CivilTimeError::BeyondHorizon { .. })
    ));
}

#[test]
fn absurd_julian_days_are_beyond_horizon() {
    for jd in [1e300, -1e300, f64::MAX, f64::MIN, 0.0] {
        for target in [TimeScale::Utc, TimeScale::Ut1] {
            assert!(
                matches!(
                    from_terrestrial(tt(jd), target),
                    Err(CivilTimeError::BeyondHorizon { .. })
                ),
                "jd {jd} target {target}"
            );
        }
    }
}

#[test]
fn non_finite_instants_are_rejected() {
    for jd in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            from_terrestrial(tt(jd), TimeScale::Utc),
            Err(CivilTimeError::NonFiniteOffset)
        );
    }
}

#[test]
fn unsupported_scales_are_rejected() {
    let jd = JulianDay::from_days(2_451_545.0);
    let utc = Instant::new(jd, TimeScale::Utc);
    assert_eq!(
        from_terrestrial(utc, TimeScale::Utc),
        Err(CivilTimeError::UnsupportedScale {
            source: TimeScale::Utc,
            target: TimeScale::Utc,
        })
    );
    assert_eq!(
        from_terrestrial(tt(2_451_545.0), TimeScale::Tdb),
        Err(CivilTimeError::UnsupportedScale {
            source: TimeScale::Tt,
            target: TimeScale::Tdb,
        })
    );
}

#[test]
fn conveniences_require_their_named_source_scale() {
    let as_tt = tt(2_451_545.0);
    let as_tdb = Instant::new(as_tt.julian_day, TimeScale::Tdb);
    assert!(utc_civil_from_tt(as_tt).is_ok());
    assert!(ut1_civil_from_tt(as_tt).is_ok());
    assert!(utc_civil_from_tdb(as_tdb).is_ok());
    assert!(ut1_civil_from_tdb(as_tdb).is_ok());
    assert_eq!(
        utc_civil_from_tdb(as_tt),
        Err(CivilTimeError::UnsupportedScale {
            source: TimeScale::Tt,
            target: TimeScale::Utc,
        })
    );
    assert_eq!(
        ut1_civil_from_tt(as_tdb),
        Err(CivilTimeError::UnsupportedScale {
            source: TimeScale::Tdb,
            target: TimeScale::Ut1,
        })
    );
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo nextest run -p pleiades-time inverse`
Expected: compile errors, unresolved `from_terrestrial`, `CivilConversion`, `ms_from_jd`, and the others.

- [ ] **Step 4: Implement the inverse**

Replace `crates/pleiades-time/src/convert/inverse.rs` with:

```rust
//! Inverse orchestrator: a TT/TDB `Instant` -> civil UTC/UT1 datetime with provenance.
//!
//! The instant is quantized once to a whole number of milliseconds. Leap
//! thresholds are exact integers on that axis, so the leap-second lookup is
//! integer comparison and a millisecond-aligned civil datetime survives the
//! round trip through `to_terrestrial` with identical fields.

use pleiades_types::{Instant, JulianDay, TimeScale, SECONDS_PER_DAY};

use super::{
    ConversionPath, ConversionProvenance, ConversionQuality, SOURCES, SUPPORT_END_JD,
    SUPPORT_START_JD,
};
use crate::calendar::CivilDateTime;
use crate::deltat::{self, DeltaTQuality};
use crate::error::CivilTimeError;
use crate::{leap, tdb};

const MS_PER_DAY: i64 = 86_400_000;
/// TT − TAI = 32.184 s, in milliseconds.
const TT_MINUS_TAI_MS: i64 = 32_184;

/// A civil datetime recovered from a TT/TDB instant, plus how it was produced.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CivilConversion {
    /// The civil datetime in `scale`, rounded to the millisecond. `second` is
    /// in `[60, 61)` only for a UTC result inside an inserted leap second.
    pub civil: CivilDateTime,
    /// The civil scale of `civil`: `Utc` or `Ut1`.
    pub scale: TimeScale,
    /// How the datetime was produced, including its truthful quality tier.
    pub provenance: ConversionProvenance,
}

/// Milliseconds on a uniform axis whose days begin at midnight: the day
/// number `floor(JD + 0.5)` times [`MS_PER_DAY`], plus the rounded
/// millisecond of day. The caller bounds `jd` to the support window first,
/// so the product cannot overflow.
fn ms_from_jd(jd: f64) -> i64 {
    let shifted = jd + 0.5;
    let day = shifted.floor();
    let ms_of_day = ((shifted - day) * MS_PER_DAY as f64).round();
    day as i64 * MS_PER_DAY + ms_of_day as i64
}

/// The Julian day of a millisecond count from [`ms_from_jd`].
fn jd_from_ms(ms: i64) -> f64 {
    let day = ms.div_euclid(MS_PER_DAY);
    let ms_of_day = ms.rem_euclid(MS_PER_DAY);
    day as f64 - 0.5 + ms_of_day as f64 / MS_PER_DAY as f64
}

/// The civil datetime of a millisecond count. The date comes from the
/// calendar inverse at an exact midnight; the time of day is split from the
/// integer millisecond, so no field is rounded separately.
fn civil_from_ms(ms: i64) -> CivilDateTime {
    let day = ms.div_euclid(MS_PER_DAY);
    let ms_of_day = ms.rem_euclid(MS_PER_DAY);
    let date = CivilDateTime::from_julian_day(JulianDay::from_days(day as f64 - 0.5));
    let hour = ms_of_day / 3_600_000;
    let minute = (ms_of_day % 3_600_000) / 60_000;
    let second = (ms_of_day % 60_000) as f64 / 1_000.0;
    CivilDateTime::new(
        date.year,
        date.month,
        date.day,
        hour as u8,
        minute as u8,
        second,
    )
}

fn exact_provenance(tai_minus_utc: i32) -> ConversionProvenance {
    ConversionProvenance {
        path: ConversionPath::UtcLeapSecond,
        quality: ConversionQuality::Exact,
        delta_t_seconds: None,
        tai_minus_utc: Some(tai_minus_utc),
        sources: SOURCES,
    }
}

/// A civil result before the support-window check: the civil millisecond
/// count used for that check, the datetime, and its provenance.
type Solved = (i64, CivilDateTime, ConversionProvenance);

/// UTC from TT. Inside the leap table the lookup runs on the TAI axis, where
/// row `i` takes effect at `effective_i + secs_i` and the second before each
/// later threshold is the inserted leap second.
fn utc_from_tt(jd_tt: f64) -> Result<Solved, CivilTimeError> {
    let stale = CivilTimeError::StaleTimeData {
        kind: "leap-second",
    };
    let rows = leap::table()?;
    let threshold =
        |&(effective, secs): &(f64, i32)| ms_from_jd(effective) + i64::from(secs) * 1_000;
    let (first, last) = match (rows.first(), rows.last()) {
        (Some(first), Some(last)) => (first, last),
        _ => return Err(stale),
    };
    let tai_ms = ms_from_jd(jd_tt) - TT_MINUS_TAI_MS;
    if tai_ms < threshold(first) {
        return Err(if jd_tt < SUPPORT_START_JD {
            CivilTimeError::BeyondHorizon { jd: jd_tt }
        } else {
            CivilTimeError::UtcBeforeLeapEpoch
        });
    }
    let horizon_ms = ms_from_jd(leap::VALID_THROUGH_JD) + i64::from(last.1) * 1_000;
    if tai_ms > horizon_ms {
        return civil_via_delta_t(jd_tt, TimeScale::Utc);
    }
    let index = rows
        .iter()
        .rposition(|row| tai_ms >= threshold(row))
        .ok_or(stale)?;
    let (_, secs) = rows[index];
    if let Some(next) = rows.get(index + 1) {
        let leap_start_ms = threshold(next) - 1_000;
        if tai_ms >= leap_start_ms {
            let last_second_ms = ms_from_jd(next.0) - 1_000;
            let mut civil = civil_from_ms(last_second_ms);
            civil.second = 60.0 + (tai_ms - leap_start_ms) as f64 / 1_000.0;
            return Ok((last_second_ms, civil, exact_provenance(secs)));
        }
    }
    let utc_ms = tai_ms - i64::from(secs) * 1_000;
    Ok((utc_ms, civil_from_ms(utc_ms), exact_provenance(secs)))
}

/// Solves `civil + ΔT(civil) = TT` for the civil Julian day, the equation the
/// forward conversion evaluates. ΔT changes by under 3e-8 s per second, so
/// each step shrinks the error by that factor and three steps are ample.
/// At the 0.216 s step in ΔT at the 2020 node the start value is already past
/// the node, so the solve settles on the post-node branch.
fn civil_via_delta_t(jd_tt: f64, target: TimeScale) -> Result<Solved, CivilTimeError> {
    let (mut delta_t, mut delta_t_quality) = deltat::delta_t(jd_tt)?;
    for _ in 0..3 {
        (delta_t, delta_t_quality) = deltat::delta_t(jd_tt - delta_t / SECONDS_PER_DAY)?;
    }
    let civil_ms = ms_from_jd(jd_tt - delta_t / SECONDS_PER_DAY);
    // Mirrors the forward: UTC reaches this path only past the leap horizon
    // and is always Predicted; UT1 reports the ΔT tier it used.
    let (path, quality) = match (target, delta_t_quality) {
        (TimeScale::Ut1, DeltaTQuality::Observed | DeltaTQuality::LeapSecondBound) => {
            (ConversionPath::Ut1DeltaT, ConversionQuality::Observed)
        }
        _ => (
            ConversionPath::FutureExtrapolated,
            ConversionQuality::Predicted,
        ),
    };
    Ok((
        civil_ms,
        civil_from_ms(civil_ms),
        ConversionProvenance {
            path,
            quality,
            delta_t_seconds: Some(delta_t),
            tai_minus_utc: None,
            sources: SOURCES,
        },
    ))
}

/// Converts a TT or TDB instant to a civil datetime in `target` (UTC or UT1):
/// the inverse of [`to_terrestrial`](super::to_terrestrial).
///
/// The source scale is read from `instant.scale`. The result is rounded to
/// the millisecond, and converting it back with `to_terrestrial` returns the
/// starting instant to within 1 ms. The provenance uses the same vocabulary
/// and the same epoch tiers as the forward conversion: UTC from 1972 through
/// the leap-second table is `Exact`; UTC beyond the table and UT1 use the
/// Delta-T model and are `Observed` or `Predicted`.
///
/// # Leap seconds
///
/// A UTC result inside an inserted leap second has `second` in `[60, 61)` on
/// the day the second was appended to, with the `TAI − UTC` in force before
/// the insertion. The result is a [`CivilDateTime`] rather than a UTC-tagged
/// `Instant` because a Julian day cannot represent that second.
///
/// # UT1 near 2020-01-01
///
/// Delta-T steps down by 0.216 s at the last observed node (2020-01-01), so
/// TT instants in a 0.216 s window there correspond to two UT1 datetimes.
/// This function returns the one at or after the node.
///
/// # Errors
///
/// - [`CivilTimeError::UnsupportedScale`] unless the instant is TT or TDB and
///   `target` is UTC or UT1.
/// - [`CivilTimeError::NonFiniteOffset`] for a non-finite Julian day.
/// - [`CivilTimeError::BeyondHorizon`] when the civil datetime falls outside
///   the 1900–2100 support window.
/// - [`CivilTimeError::UtcBeforeLeapEpoch`] for a UTC target before
///   1972-01-01; use UT1 there.
/// - [`CivilTimeError::StaleTimeData`] if a pinned table fails its checksum.
///
/// # Examples
///
/// ```
/// use pleiades_time::{from_terrestrial, tt_from_utc_civil, CivilDateTime};
/// use pleiades_types::TimeScale;
///
/// // Half a second into the leap second inserted at the end of 2016.
/// let leap = CivilDateTime::new(2016, 12, 31, 23, 59, 60.5);
/// let tt = tt_from_utc_civil(leap).unwrap();
/// let back = from_terrestrial(tt.instant, TimeScale::Utc).unwrap();
/// assert_eq!(back.civil, leap);
/// assert_eq!(back.scale, TimeScale::Utc);
/// assert_eq!(back.provenance.tai_minus_utc, Some(36));
/// ```
pub fn from_terrestrial(
    instant: Instant,
    target: TimeScale,
) -> Result<CivilConversion, CivilTimeError> {
    let source = instant.scale;
    if !matches!(source, TimeScale::Tt | TimeScale::Tdb)
        || !matches!(target, TimeScale::Utc | TimeScale::Ut1)
    {
        return Err(CivilTimeError::UnsupportedScale { source, target });
    }
    let jd = instant.julian_day.days();
    if !jd.is_finite() {
        return Err(CivilTimeError::NonFiniteOffset);
    }
    // The forward evaluates the periodic term at the TT day; evaluating it at
    // the TDB day differs by under 1e-12 s.
    let jd_tt = if source == TimeScale::Tdb {
        jd - tdb::tdb_minus_tt_seconds(jd) / SECONDS_PER_DAY
    } else {
        jd
    };
    // TT and civil time differ by minutes at most, so anything more than a
    // day outside the window is out of range. Rejecting it here also keeps
    // the millisecond count inside i64.
    if !(SUPPORT_START_JD - 1.0..SUPPORT_END_JD + 1.0).contains(&jd_tt) {
        return Err(CivilTimeError::BeyondHorizon { jd: jd_tt });
    }
    let (civil_ms, civil, provenance) = if target == TimeScale::Utc {
        utc_from_tt(jd_tt)?
    } else {
        civil_via_delta_t(jd_tt, target)?
    };
    if !(ms_from_jd(SUPPORT_START_JD)..ms_from_jd(SUPPORT_END_JD)).contains(&civil_ms) {
        return Err(CivilTimeError::BeyondHorizon {
            jd: jd_from_ms(civil_ms),
        });
    }
    Ok(CivilConversion {
        civil,
        scale: target,
        provenance,
    })
}

fn from_scale(
    instant: Instant,
    source: TimeScale,
    target: TimeScale,
) -> Result<CivilConversion, CivilTimeError> {
    if instant.scale != source {
        return Err(CivilTimeError::UnsupportedScale {
            source: instant.scale,
            target,
        });
    }
    from_terrestrial(instant, target)
}

/// Convenience: TT instant -> UTC civil. Rejects an instant not tagged TT.
pub fn utc_civil_from_tt(instant: Instant) -> Result<CivilConversion, CivilTimeError> {
    from_scale(instant, TimeScale::Tt, TimeScale::Utc)
}
/// Convenience: TDB instant -> UTC civil. Rejects an instant not tagged TDB.
pub fn utc_civil_from_tdb(instant: Instant) -> Result<CivilConversion, CivilTimeError> {
    from_scale(instant, TimeScale::Tdb, TimeScale::Utc)
}
/// Convenience: TT instant -> UT1 civil. Rejects an instant not tagged TT.
pub fn ut1_civil_from_tt(instant: Instant) -> Result<CivilConversion, CivilTimeError> {
    from_scale(instant, TimeScale::Tt, TimeScale::Ut1)
}
/// Convenience: TDB instant -> UT1 civil. Rejects an instant not tagged TDB.
pub fn ut1_civil_from_tdb(instant: Instant) -> Result<CivilConversion, CivilTimeError> {
    from_scale(instant, TimeScale::Tdb, TimeScale::Ut1)
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 5: Re-export from the crate root**

In `crates/pleiades-time/src/lib.rs`, replace the `pub use convert::{...};` block with:

```rust
pub use convert::{
    from_terrestrial, tdb_from_ut1_civil, tdb_from_utc_civil, to_terrestrial, tt_from_ut1_civil,
    tt_from_utc_civil, ut1_civil_from_tdb, ut1_civil_from_tt, ut1_jd_from_tt, utc_civil_from_tdb,
    utc_civil_from_tt, CivilConversion, CivilInstant, ConversionPath, ConversionProvenance,
    ConversionQuality, SUPPORT_END_JD, SUPPORT_START_JD,
};
```

In the crate-level docs at the top of the same file, add after the closing ```` ``` ```` of the existing example (before `#![deny(missing_docs)]`):

```rust
//!
//! The inverse, [`from_terrestrial`], turns a TT or TDB instant (for example
//! one returned by an event finder) back into a civil UTC or UT1 datetime
//! with the same provenance:
//!
//! ```
//! use pleiades_time::{tdb_from_utc_civil, utc_civil_from_tdb, CivilDateTime};
//!
//! let civil = CivilDateTime::new(2024, 3, 20, 3, 6, 21.0);
//! let tdb = tdb_from_utc_civil(civil).expect("inside the 1900-2100 support window");
//! let back = utc_civil_from_tdb(tdb.instant).expect("UTC is defined from 1972");
//! assert_eq!(back.civil, civil);
//! ```
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo nextest run -p pleiades-time && cargo test --doc -p pleiades-time`
Expected: PASS, every unit test and every doctest.

If `ambiguous_window_at_the_2020_node_resolves_after_the_node` fails on the `0.116` literal, print the returned datetime before changing anything: the expected value is `node + (69.3 − 69.184) s`. A different second means the ΔT step is not what the spec states; stop and report rather than adjusting the literal.

- [ ] **Step 7: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-time --all-targets --all-features -- -D warnings
git add crates/pleiades-time/src/convert.rs crates/pleiades-time/src/convert/inverse.rs crates/pleiades-time/src/convert/inverse/tests.rs crates/pleiades-time/src/lib.rs
git commit -m "feat(time): convert a TT or TDB instant back to civil UTC or UT1 (#87)"
```

---

### Task 4: Round-trip and monotonicity properties

**Files:**
- Modify: `crates/pleiades-time/Cargo.toml`
- Modify: `Cargo.lock` (regenerated by cargo)
- Test: `crates/pleiades-time/src/convert/inverse/tests.rs`

**Interfaces:**
- Consumes: from Task 3, `from_terrestrial`, `utc_civil_from_tt`, and the private `ms_from_jd`, `jd_from_ms`, `civil_from_ms`, `MS_PER_DAY` (reachable through `use super::*`); from Task 2, `to_terrestrial` and `test_support::{at, insertions}`.
- Produces: tests only.

- [ ] **Step 1: Add the dev-dependency**

In `crates/pleiades-time/Cargo.toml`, add after the `[dependencies]` table:

```toml
[dev-dependencies]
proptest = { workspace = true }
```

- [ ] **Step 2: Write the property tests**

Append to `crates/pleiades-time/src/convert/inverse/tests.rs`:

```rust
mod properties {
    use proptest::prelude::*;

    use super::*;
    use crate::convert::to_terrestrial;

    /// Asserts the inverse reproduced the forward's provenance. ΔT is solved
    /// rather than looked up, so it is compared to a microsecond.
    fn assert_same_provenance(
        back: &ConversionProvenance,
        forward: &ConversionProvenance,
    ) -> Result<(), TestCaseError> {
        prop_assert_eq!(back.path, forward.path);
        prop_assert_eq!(back.quality, forward.quality);
        prop_assert_eq!(back.tai_minus_utc, forward.tai_minus_utc);
        match (back.delta_t_seconds, forward.delta_t_seconds) {
            (Some(a), Some(b)) => prop_assert!((a - b).abs() < 1e-6, "ΔT {a} vs {b}"),
            (None, None) => {}
            other => prop_assert!(false, "ΔT presence differs: {other:?}"),
        }
        Ok(())
    }

    /// Orders civil datetimes; `:60` sorts after `:59` on the same minute.
    fn key(civil: CivilDateTime) -> (i32, u8, u8, u8, u8, f64) {
        (
            civil.year,
            civil.month,
            civil.day,
            civil.hour,
            civil.minute,
            civil.second,
        )
    }

    proptest! {
        #[test]
        fn utc_round_trips_through_tt_and_tdb(
            ms in ms_from_jd(leap::LEAP_EPOCH_JD)..ms_from_jd(SUPPORT_END_JD),
        ) {
            let civil = civil_from_ms(ms);
            for target in [TimeScale::Tt, TimeScale::Tdb] {
                let forward = to_terrestrial(civil, TimeScale::Utc, target).unwrap();
                let back = from_terrestrial(forward.instant, TimeScale::Utc).unwrap();
                prop_assert_eq!(back.civil, civil);
                prop_assert_eq!(back.scale, TimeScale::Utc);
                assert_same_provenance(&back.provenance, &forward.provenance)?;
            }
        }

        #[test]
        fn leap_seconds_round_trip_at_every_millisecond(
            index in 0usize..27,
            ms in 0i64..1_000,
        ) {
            let insertion = &insertions()[index];
            let civil = at(insertion.last_day, 23, 59, 60.0 + ms as f64 / 1_000.0);
            for target in [TimeScale::Tt, TimeScale::Tdb] {
                let forward = to_terrestrial(civil, TimeScale::Utc, target).unwrap();
                let back = from_terrestrial(forward.instant, TimeScale::Utc).unwrap();
                prop_assert_eq!(back.civil, civil);
                prop_assert_eq!(back.provenance.tai_minus_utc, Some(insertion.offset_before));
            }
        }

        #[test]
        fn ut1_round_trips_away_from_the_2020_node(
            ms in ms_from_jd(SUPPORT_START_JD)..ms_from_jd(SUPPORT_END_JD),
        ) {
            // The forward is two-to-one in a 0.216 s window at the node.
            let node_ms = ms_from_jd(crate::deltat::OBSERVED_THROUGH_JD);
            prop_assume!((ms - node_ms).abs() > 500);
            let civil = civil_from_ms(ms);
            for target in [TimeScale::Tt, TimeScale::Tdb] {
                let forward = to_terrestrial(civil, TimeScale::Ut1, target).unwrap();
                let back = from_terrestrial(forward.instant, TimeScale::Ut1).unwrap();
                prop_assert_eq!(back.civil, civil);
                prop_assert_eq!(back.scale, TimeScale::Ut1);
                assert_same_provenance(&back.provenance, &forward.provenance)?;
            }
        }

        #[test]
        fn utc_inverse_is_monotonic_and_reenters_the_forward(
            // TT from one minute after the UTC epoch; `gap` reaches past two
            // days so pairs straddle leap seconds and the leap horizon.
            a in (ms_from_jd(leap::LEAP_EPOCH_JD) + 60_000)
                ..(ms_from_jd(SUPPORT_END_JD) - 300_000_000),
            gap in 2i64..200_000_000,
        ) {
            let b = a + gap;
            let civil_a = utc_civil_from_tt(tt(jd_from_ms(a))).unwrap().civil;
            let civil_b = utc_civil_from_tt(tt(jd_from_ms(b))).unwrap().civil;
            prop_assert!(key(civil_a) < key(civil_b), "{civil_a:?} !< {civil_b:?}");
            // Every result is a datetime the forward accepts, and it lands
            // within 1 ms of where it came from.
            let again = to_terrestrial(civil_a, TimeScale::Utc, TimeScale::Tt).unwrap();
            let error = (again.instant.julian_day.days() - jd_from_ms(a)) * SECONDS_PER_DAY;
            prop_assert!(error.abs() < 1e-3, "re-entry error {error} s");
        }
    }
}
```

- [ ] **Step 3: Run the property tests**

Run: `cargo nextest run -p pleiades-time properties`
Expected: PASS, 4 tests. These exercise code from Task 3, so they are expected to pass on first run; they are the regression net, not a red-green cycle.

To confirm they can fail, temporarily change `threshold(next) - 1_000` to `threshold(next)` in `utc_from_tt` (`crates/pleiades-time/src/convert/inverse.rs`), re-run, and confirm `leap_seconds_round_trip_at_every_millisecond` FAILS. Then restore the line exactly and re-run to green. Check `git diff crates/pleiades-time/src/convert/inverse.rs` is empty before continuing.

If a property fails on unmodified code, proptest prints the minimal failing input. Do not widen a tolerance or narrow a range to make it pass: reproduce the input as a unit test in the same file, find the cause, and fix the implementation. Commit any `proptest-regressions` file proptest writes.

- [ ] **Step 4: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-time --all-targets --all-features -- -D warnings
git add crates/pleiades-time/Cargo.toml Cargo.lock crates/pleiades-time/src/convert/inverse/tests.rs
git commit -m "test(time): round-trip and monotonicity properties for the civil inverse (#87)"
```

---

### Task 5: Core re-export, documentation, and the blocking tier

**Files:**
- Modify: `crates/pleiades-core/src/lib.rs:133`
- Modify: `crates/pleiades-time/README.md`
- Modify: `README.md:96`
- Modify: `docs/time-observer-policy.md` (the "Time scales and Delta T" list, after the bullet beginning "**Built-in civil UTC/UT1 → TT/TDB conversion is now provided**", and the rise/set paragraph)
- Modify: `docs/follow-ups.md` (append `FU-19`)
- Modify: `docs/superpowers/specs/2026-10-02-civil-from-terrestrial-design.md` (status line)

**Interfaces:**
- Consumes: `pleiades_time::{CivilConversion, from_terrestrial}` from Task 3.
- Produces: `pleiades_core::{CivilConversion, from_terrestrial}`.

- [ ] **Step 1: Re-export from `pleiades-core`**

In `crates/pleiades-core/src/lib.rs`, replace:

```rust
pub use pleiades_time::{CivilDateTime, CivilInstant, CivilTimeError, ConversionProvenance};
```

with:

```rust
pub use pleiades_time::{
    from_terrestrial, CivilConversion, CivilDateTime, CivilInstant, CivilTimeError,
    ConversionProvenance,
};
```

Run: `cargo build -p pleiades-core`
Expected: builds. If `from_terrestrial` collides with an existing name in `pleiades-core`, stop and report; do not rename.

- [ ] **Step 2: Crate README**

Replace `crates/pleiades-time/README.md` with:

```markdown
# pleiades-time

Civil-time conversion for the `pleiades` workspace: Gregorian calendar to Julian
Day, leap seconds, Delta-T, and TT/TDB with typed conversion provenance.

- `to_terrestrial` converts a civil UTC or UT1 datetime to a TT or TDB
  `Instant`.
- `from_terrestrial` converts a TT or TDB `Instant` back to a civil UTC or UT1
  datetime, rounded to the millisecond. A UTC result inside an inserted leap
  second is returned as `23:59:60.x`.

Both cover 1900–2100 and report the same `ConversionProvenance`: `exact` for
UTC inside the leap-second table (from 1972), `observed` or `predicted` where
the Delta-T model is used. A round trip returns the starting datetime to
within 1 ms.
```

- [ ] **Step 3: Workspace README**

In `README.md`, in the crates table row for `pleiades-time` (line 96), replace:

```
Civil-time conversion: civil UTC/UT1 calendar datetimes → TT/TDB `Instant`s (1900–2100,
```

with:

```
Civil-time conversion: civil UTC/UT1 calendar datetimes → TT/TDB `Instant`s and back (`from_terrestrial`, millisecond precision, leap seconds as `23:59:60`) (1900–2100,
```

The row is a single line; keep it a single line.

- [ ] **Step 4: Time policy doc**

In `docs/time-observer-policy.md`, add a new bullet directly after the bullet that begins "**Built-in civil UTC/UT1 → TT/TDB conversion is now provided by `pleiades-time`.**":

```markdown
- **The inverse is `pleiades_time::from_terrestrial` (since #87).** It converts a TT or TDB `Instant`, such as one returned by an event finder, to a civil UTC or UT1 `CivilDateTime` over the same 1900–2100 window, with the same `ConversionProvenance` tiers. Results are rounded to the millisecond and a round trip through `to_terrestrial` holds to 1 ms. An instant inside an inserted leap second is returned as `23:59:60.x`; the forward conversion accepts that form only at a real insertion and rejects `:60` elsewhere. Before 1972 a UTC target is `UtcBeforeLeapEpoch`; use UT1. At the 2020 Delta-T node, where the model steps by 0.216 s, the UT1 inverse returns the datetime at or after the node.
```

In the same file, in the "Rise/set/transit and horizontal coordinates (since #74)" paragraph, replace:

```
read it as civil time with `ut1_instant` or `pleiades-time`.
```

with:

```
read it as civil time with `pleiades_time::from_terrestrial` (or `ut1_instant` for a UT1 `Instant`).
```

- [ ] **Step 5: Follow-ups entry**

Append to the end of `docs/follow-ups.md`:

```markdown

## FU-19: Civil datetime from a TT or TDB instant (issue #87)

**Status:** resolved (2026-10-02) · Spec
`docs/superpowers/specs/2026-10-02-civil-from-terrestrial-design.md`, plan
`docs/superpowers/plans/2026-10-02-civil-from-terrestrial.md`.

**What:** `pleiades-time` converted civil UTC/UT1 to TT/TDB but not back, so
callers of the event finders had to re-implement the leap-second table and
Delta-T. Resolved by `from_terrestrial` (and four scale-checked conveniences),
which quantizes to the millisecond and looks up leap seconds on the TAI axis.
The same change fixed the forward conversion of an inserted leap second:
`23:59:60.x` aliased the next day's `00:00:00.x` in the Julian day and landed
one second late in TT. `:60` is now accepted only at a real insertion.

**Deferred:**

- A civil-time convenience on `pleiades-events` results and CLI output of
  civil times for events. Callers pass the returned `Instant` to
  `from_terrestrial`.
- The 0.216 s Delta-T step at the 2020 node is unchanged; the UT1 inverse
  picks the post-node branch there.
- Sub-millisecond precision would need a (day, seconds-of-day) representation
  in both directions; a Julian day near 2.46e6 resolves about 40 µs.
```

- [ ] **Step 6: Mark the spec implemented**

In `docs/superpowers/specs/2026-10-02-civil-from-terrestrial-design.md`, replace:

```
**Status:** approved design, not yet implemented ·
```

with:

```
**Status:** implemented (2026-10-02) ·
```

- [ ] **Step 7: Run the blocking tier**

Run, in the foreground, one at a time, with no edits in between:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
mise run test
mise run doctest
mise run docs
```

Expected: each exits 0. `mise run test` covers `pleiades-core` (whose `chart/request.rs` uses the forward conversion) and `pleiades-cli`; a failure there that mentions a civil datetime with `second >= 60` means a caller relied on the old aliasing. Report it with the test name and output rather than loosening the new check.

- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-core/src/lib.rs crates/pleiades-time/README.md README.md docs/time-observer-policy.md docs/follow-ups.md docs/superpowers/specs/2026-10-02-civil-from-terrestrial-design.md
git commit -m "docs: document the civil inverse and re-export it from pleiades-core (#87)"
```

---

## Self-Review Notes

- **Spec coverage.** Public API and errors: Task 3. Algorithm steps 0–5: Task 3. Forward fix and `to_julian_day` rustdoc: Task 2. Table-shape guard: Task 1. 2020 node: Task 3 unit test, Task 4 exclusion. Precision and round-trip properties: Tasks 3 and 4. Module layout and shared test setup: Tasks 2 and 3. Documentation and consumer-visible changes: Tasks 2 (breaking commit) and 5.
- **Deviation from the spec's test list.** The spec asks for the 1972 boundary to fail "one millisecond earlier"; the test uses 2 ms so Julian-day noise (about 40 µs) cannot land the sample on the boundary millisecond.
- **Type consistency.** `Insertion { last_day, next_day, offset_before }`, `insertions()`, `at(date, hour, minute, second)`, `ms_from_jd`, `jd_from_ms`, `civil_from_ms`, `MS_PER_DAY`, `CivilConversion { civil, scale, provenance }` are used with the same names and shapes in every task.
