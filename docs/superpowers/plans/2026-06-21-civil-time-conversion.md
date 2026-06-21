# Civil-Time Conversion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a new `pleiades-time` crate that converts civil (UTC/UT1) calendar datetimes to TT/TDB `Instant`s across 1900–2100, carrying typed provenance and a tiered Exact/Observed/Predicted quality marker.

**Architecture:** A standalone crate `pleiades-time` (depends only on `pleiades-types`) with independently testable layers — `calendar` (Gregorian↔JD), `leap` (leap-second table), `deltat` (ΔT table + extrapolation), `tdb` (TT↔TDB periodic term), and a `convert` orchestrator that selects the path and tags quality. Data tables are committed CSV files embedded with `include_str!`, FNV-1a checksum-pinned, and fail closed on drift. Integration into `pleiades-core` (`ChartRequest::from_civil`) and the CLI (`--civil`) is additive — the existing caller-supplied retagging contract is untouched.

**Tech Stack:** Rust 2021, `no`-extra-deps (std + optional `serde`), workspace cargo, FNV-1a checksums (mirroring `pleiades-jpl::spk::corpus_manifest::corpus_checksum64`).

## Global Constraints

- Edition 2021; `version.workspace = true` (0.2.0); `rust-version = 1.96.0`; `license = "MIT OR Apache-2.0"` — copy the `[package]` field-inheritance style from `crates/pleiades-types/Cargo.toml` verbatim.
- Only dependency is `pleiades-types` (path/workspace dep) plus optional `serde` behind a `serde` feature, matching `pleiades-types`.
- Published crate (like its peers) — add to `[workspace.dependencies]` and `members`.
- Supported civil window is **1900-01-01 .. 2100-12-31** inclusive (JD 2415020.5 .. 2488434.49999…). Outside → hard error.
- UTC is only valid from **1972-01-01** (JD 2441317.5); earlier UTC input is an error.
- Checksums use FNV-1a 64-bit with `FNV_OFFSET_BASIS = 0xcbf2_9ce4_8422_2325`, `FNV_PRIME = 0x0000_0001_0000_01b3` — byte-identical to `corpus_checksum64`.
- Every public error/summary type exposes `summary_line()` and `Display`, matching the repo convention (see `TimeScaleConversionError`, `DeltaTPolicySummary`).
- Anchor constants (verified): JD(2000-01-01 12:00) = 2451545.0; JD(1972-01-01 00:00) = 2441317.5; JD(1900-01-01 00:00) = 2415020.5; JD(2100-01-01 00:00) = 2488069.5. `TT = TAI + 32.184 s`.
- `pleiades_types::Instant` (in `crates/pleiades-types/src/time.rs`) has **public fields** `julian_day: JulianDay` and `scale: TimeScale` (no `scale()`/`julian_day()` methods); read the JD as `instant.julian_day.days()`. `Instant::new(julian_day, scale)`.
- `pleiades_core::ChartRequest` lives at `crates/pleiades-core/src/chart/request.rs`; `ChartRequest::new(instant: Instant)` takes only the instant and defaults the rest; `bodies`, `instant`, etc. are public fields; `default_chart_bodies()` returns the default body set.

---

## File Structure

```
crates/pleiades-time/
  Cargo.toml
  src/
    lib.rs            # crate root, re-exports, fnv1a64 helper
    error.rs          # CivilTimeError
    calendar.rs       # CivilDateTime <-> JulianDay
    leap.rs           # leap-second table load + lookup + checksum gate
    deltat.rs         # observed ΔT table + extrapolation + DeltaTQuality
    tdb.rs            # TT <-> TDB periodic term
    convert.rs        # orchestrator: CivilInstant, ConversionProvenance, to_terrestrial
    policy.rs         # CivilTimePolicySummary
  data/
    leap-seconds.csv      # effective_jd_utc,tai_minus_utc
    delta-t-observed.csv  # year,delta_t_seconds
```

Integration touch points (later tasks):
- `Cargo.toml` (workspace), `crates/pleiades-core/{Cargo.toml,src/chart.rs,src/lib.rs}`
- `crates/pleiades-cli/src/commands/chart.rs`
- `crates/pleiades-backend/src/policy/{utc.rs,delta_t.rs,current.rs}` (posture wording)
- `docs/time-observer-policy.md`, `README.md`, `PLAN.md`

---

## Task 1: Scaffold the `pleiades-time` crate

**Files:**
- Create: `crates/pleiades-time/Cargo.toml`
- Create: `crates/pleiades-time/src/lib.rs`
- Modify: `Cargo.toml` (workspace `members` + `[workspace.dependencies]`)
- Test: `crates/pleiades-time/src/lib.rs` (inline `#[cfg(test)]`)

**Interfaces:**
- Produces: `pleiades_time::fnv1a64(&str) -> u64` (shared checksum helper used by `leap`/`deltat`).

- [ ] **Step 1: Create the crate manifest**

`crates/pleiades-time/Cargo.toml`:
```toml
[package]
name = "pleiades-time"
description = "Civil-time conversion for the pleiades astrology workspace: Gregorian calendar, leap seconds, Delta-T, and TT/TDB with typed provenance."
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
repository.workspace = true
homepage.workspace = true
keywords.workspace = true
categories.workspace = true
readme = "README.md"

[features]
serde = ["dep:serde", "pleiades-types/serde"]

[dependencies]
pleiades-types = { workspace = true }
serde = { workspace = true, optional = true }

[package.metadata.docs.rs]
all-features = true
```

- [ ] **Step 2: Create a minimal `README.md`**

`crates/pleiades-time/README.md`:
```markdown
# pleiades-time

Civil-time conversion for the `pleiades` workspace: Gregorian calendar to Julian
Day, leap seconds, Delta-T, and TT/TDB with typed conversion provenance.
```

- [ ] **Step 3: Write the crate root with the shared checksum helper + a failing test**

`crates/pleiades-time/src/lib.rs`:
```rust
//! Civil-time conversion: Gregorian calendar, leap seconds, Delta-T, and
//! TT/TDB output with typed provenance.

/// Deterministic 64-bit content checksum (FNV-1a), byte-identical to
/// `pleiades_jpl::spk::corpus_manifest::corpus_checksum64`. Used to detect drift
/// between a checked-in data table and its pinned checksum. Not cryptographic.
pub fn fnv1a64(text: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0001_0000_01b3;
    let mut hash = FNV_OFFSET_BASIS;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a64_is_deterministic_and_sensitive() {
        assert_eq!(fnv1a64("abc"), fnv1a64("abc"));
        assert_ne!(fnv1a64("abc"), fnv1a64("abd"));
    }
}
```

- [ ] **Step 4: Register the crate in the workspace**

In `Cargo.toml`, add to `members` (keep alphabetical, after `pleiades-jpl`):
```toml
    "crates/pleiades-time",
```
And add to `[workspace.dependencies]` (after the `pleiades-jpl` line):
```toml
pleiades-time = { path = "crates/pleiades-time", version = "0.2.0" }
```

- [ ] **Step 5: Build and test**

Run: `cargo test -p pleiades-time`
Expected: compiles; `fnv1a64_is_deterministic_and_sensitive` PASSES.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-time Cargo.toml
git commit -m "feat(time): scaffold pleiades-time crate with fnv1a64 helper"
```

---

## Task 2: `error` module — `CivilTimeError`

**Files:**
- Create: `crates/pleiades-time/src/error.rs`
- Modify: `crates/pleiades-time/src/lib.rs` (add `mod error;` + re-export)
- Test: `crates/pleiades-time/src/error.rs` (inline tests)

**Interfaces:**
- Produces: `pleiades_time::CivilTimeError` with variants `InvalidCivilDate { field: &'static str }`, `UtcBeforeLeapEpoch`, `BeyondHorizon { jd: f64 }`, `UnsupportedScale { source: TimeScale, target: TimeScale }`, `StaleTimeData { kind: &'static str }`, `NonFiniteOffset`; methods `summary_line(&self) -> String`, `Display`, `std::error::Error`.

- [ ] **Step 1: Write the failing test**

`crates/pleiades-time/src/error.rs`:
```rust
//! Structured, fail-closed errors for civil-time conversion.

use core::fmt;

use pleiades_types::TimeScale;

/// Error returned when a civil-time conversion cannot be performed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CivilTimeError {
    /// A calendar field (month/day/hour/minute/second) was out of range or non-finite.
    InvalidCivilDate { field: &'static str },
    /// A UTC-tagged instant fell before the 1972 leap-second epoch.
    UtcBeforeLeapEpoch,
    /// The instant fell outside the documented support window (1900–2100).
    BeyondHorizon { jd: f64 },
    /// The requested source/target scale pair is not a civil conversion this crate performs.
    UnsupportedScale { source: TimeScale, target: TimeScale },
    /// A pinned data table failed its checksum/freshness gate.
    StaleTimeData { kind: &'static str },
    /// A computed offset was not finite (defensive).
    NonFiniteOffset,
}

impl CivilTimeError {
    /// Compact one-line rendering for diagnostics and release-facing summaries.
    pub fn summary_line(&self) -> String {
        match self {
            Self::InvalidCivilDate { field } => {
                format!("invalid civil date field: {field}")
            }
            Self::UtcBeforeLeapEpoch => {
                "UTC civil input is undefined before 1972-01-01; tag pre-1972 input as UT1".to_string()
            }
            Self::BeyondHorizon { jd } => {
                format!("civil instant JD {jd} is outside the supported 1900–2100 window")
            }
            Self::UnsupportedScale { source, target } => {
                format!("unsupported civil conversion: {source} -> {target}")
            }
            Self::StaleTimeData { kind } => {
                format!("{kind} time-data table failed its checksum/freshness gate")
            }
            Self::NonFiniteOffset => "computed time-scale offset was not finite".to_string(),
        }
    }
}

impl fmt::Display for CivilTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for CivilTimeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_lines_are_distinct_and_nonempty() {
        let errors = [
            CivilTimeError::InvalidCivilDate { field: "month" },
            CivilTimeError::UtcBeforeLeapEpoch,
            CivilTimeError::BeyondHorizon { jd: 1.0 },
            CivilTimeError::UnsupportedScale { source: TimeScale::Tt, target: TimeScale::Utc },
            CivilTimeError::StaleTimeData { kind: "leap-second" },
            CivilTimeError::NonFiniteOffset,
        ];
        for e in errors {
            assert!(!e.summary_line().is_empty());
            assert_eq!(e.to_string(), e.summary_line());
        }
    }
}
```

- [ ] **Step 2: Wire the module into `lib.rs`**

Add near the top of `crates/pleiades-time/src/lib.rs` (after the doc comment):
```rust
mod error;

pub use error::CivilTimeError;
```

- [ ] **Step 3: Run the test to verify it passes**

Run: `cargo test -p pleiades-time error::`
Expected: `summary_lines_are_distinct_and_nonempty` PASSES.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-time/src/error.rs crates/pleiades-time/src/lib.rs
git commit -m "feat(time): add CivilTimeError"
```

---

## Task 3: `calendar` module — `CivilDateTime` ↔ `JulianDay`

**Files:**
- Create: `crates/pleiades-time/src/calendar.rs`
- Modify: `crates/pleiades-time/src/lib.rs`
- Test: `crates/pleiades-time/src/calendar.rs` (inline tests)

**Interfaces:**
- Consumes: `CivilTimeError`, `pleiades_types::JulianDay`.
- Produces: `pleiades_time::CivilDateTime { year: i32, month: u8, day: u8, hour: u8, minute: u8, second: f64 }`; `CivilDateTime::new(...) -> Self`; `CivilDateTime::to_julian_day(&self) -> Result<JulianDay, CivilTimeError>`; `CivilDateTime::from_julian_day(JulianDay) -> CivilDateTime`.

- [ ] **Step 1: Write the failing tests**

`crates/pleiades-time/src/calendar.rs`:
```rust
//! Proleptic-Gregorian civil datetime <-> Julian Day (Meeus, ch. 7).

use pleiades_types::JulianDay;

use crate::error::CivilTimeError;

/// A civil calendar datetime in the proleptic Gregorian calendar. Scale-agnostic:
/// the caller tags it UTC or UT1 at conversion time.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CivilDateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: f64,
}

impl CivilDateTime {
    /// Creates a civil datetime without validation. Validation happens in `to_julian_day`.
    pub fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: f64) -> Self {
        Self { year, month, day, hour, minute, second }
    }

    fn validate(&self) -> Result<(), CivilTimeError> {
        if !(1..=12).contains(&self.month) {
            return Err(CivilTimeError::InvalidCivilDate { field: "month" });
        }
        if !(1..=31).contains(&self.day) {
            return Err(CivilTimeError::InvalidCivilDate { field: "day" });
        }
        if self.hour > 23 {
            return Err(CivilTimeError::InvalidCivilDate { field: "hour" });
        }
        if self.minute > 59 {
            return Err(CivilTimeError::InvalidCivilDate { field: "minute" });
        }
        if !self.second.is_finite() || self.second < 0.0 || self.second >= 61.0 {
            return Err(CivilTimeError::InvalidCivilDate { field: "second" });
        }
        Ok(())
    }

    /// Converts to a Julian Day using the proleptic-Gregorian Meeus formula.
    pub fn to_julian_day(&self) -> Result<JulianDay, CivilTimeError> {
        self.validate()?;
        let day_frac = self.day as f64
            + (self.hour as f64 + self.minute as f64 / 60.0 + self.second / 3600.0) / 24.0;
        let (y, m) = if self.month <= 2 {
            (self.year - 1, self.month as i32 + 12)
        } else {
            (self.year, self.month as i32)
        };
        let a = (y as f64 / 100.0).floor();
        let b = 2.0 - a + (a / 4.0).floor();
        let jd = (365.25 * (y as f64 + 4716.0)).floor()
            + (30.6001 * (m as f64 + 1.0)).floor()
            + day_frac
            + b
            - 1524.5;
        Ok(JulianDay::from_days(jd))
    }

    /// Inverse: reconstructs a civil datetime from a Julian Day (proleptic Gregorian).
    pub fn from_julian_day(jd: JulianDay) -> Self {
        let jd = jd.days() + 0.5;
        let z = jd.floor();
        let f = jd - z;
        let alpha = ((z - 1867216.25) / 36524.25).floor();
        let a = z + 1.0 + alpha - (alpha / 4.0).floor();
        let b = a + 1524.0;
        let c = ((b - 122.1) / 365.25).floor();
        let d = (365.25 * c).floor();
        let e = ((b - d) / 30.6001).floor();
        let day_frac = b - d - (30.6001 * e).floor() + f;
        let day = day_frac.floor();
        let month = if e < 14.0 { e - 1.0 } else { e - 13.0 };
        let year = if month > 2.0 { c - 4716.0 } else { c - 4715.0 };
        let mut rem_hours = (day_frac - day) * 24.0;
        let hour = rem_hours.floor();
        rem_hours = (rem_hours - hour) * 60.0;
        let minute = rem_hours.floor();
        let second = (rem_hours - minute) * 60.0;
        Self {
            year: year as i32,
            month: month as u8,
            day: day as u8,
            hour: hour as u8,
            minute: minute as u8,
            second,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2000_noon_is_jd_2451545() {
        let jd = CivilDateTime::new(2000, 1, 1, 12, 0, 0.0).to_julian_day().unwrap();
        assert!((jd.days() - 2451545.0).abs() < 1e-6, "got {}", jd.days());
    }

    #[test]
    fn epoch_anchors_match_known_jds() {
        let cases = [
            (1900, 1, 1, 2415020.5),
            (1972, 1, 1, 2441317.5),
            (2100, 1, 1, 2488069.5),
        ];
        for (y, m, d, expected) in cases {
            let jd = CivilDateTime::new(y, m, d, 0, 0, 0.0).to_julian_day().unwrap();
            assert!((jd.days() - expected).abs() < 1e-6, "{y}-{m}-{d}: got {}", jd.days());
        }
    }

    #[test]
    fn round_trips_within_a_millisecond() {
        let original = CivilDateTime::new(1987, 4, 10, 19, 21, 0.0);
        let jd = original.to_julian_day().unwrap();
        let back = CivilDateTime::from_julian_day(jd);
        assert_eq!(back.year, 1987);
        assert_eq!(back.month, 4);
        assert_eq!(back.day, 10);
        assert_eq!(back.hour, 19);
        assert_eq!(back.minute, 21);
        assert!(back.second < 0.001 || back.second > 59.999);
    }

    #[test]
    fn rejects_bad_fields() {
        assert_eq!(
            CivilDateTime::new(2000, 13, 1, 0, 0, 0.0).to_julian_day(),
            Err(CivilTimeError::InvalidCivilDate { field: "month" })
        );
        assert_eq!(
            CivilDateTime::new(2000, 1, 1, 0, 0, f64::NAN).to_julian_day(),
            Err(CivilTimeError::InvalidCivilDate { field: "second" })
        );
    }
}
```

- [ ] **Step 2: Wire the module into `lib.rs`**

Add to `crates/pleiades-time/src/lib.rs`:
```rust
mod calendar;

pub use calendar::CivilDateTime;
```

- [ ] **Step 3: Run the test to verify it fails, then passes**

Run: `cargo test -p pleiades-time calendar::`
Expected: all four calendar tests PASS (J2000 = 2451545.0, anchors, round-trip, field rejection).

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-time/src/calendar.rs crates/pleiades-time/src/lib.rs
git commit -m "feat(time): add CivilDateTime calendar <-> JulianDay conversion"
```

---

## Task 4: `leap` module — leap-second table + checksum gate

**Files:**
- Create: `crates/pleiades-time/data/leap-seconds.csv`
- Create: `crates/pleiades-time/src/leap.rs`
- Modify: `crates/pleiades-time/src/lib.rs`
- Test: `crates/pleiades-time/src/leap.rs` (inline tests)

**Interfaces:**
- Consumes: `CivilTimeError`, `fnv1a64`, `pleiades_types::JulianDay`.
- Produces: `leap::tai_minus_utc(jd_utc: f64) -> Result<Option<i32>, CivilTimeError>` (`Ok(None)` = before 1972 or after `valid_through`; `Err(StaleTimeData)` = checksum drift); `leap::VALID_THROUGH_JD: f64`; `leap::LEAP_EPOCH_JD: f64` (2441317.5).

- [ ] **Step 1: Create the pinned data file**

`crates/pleiades-time/data/leap-seconds.csv` (effective UTC JD at 00:00, cumulative `TAI − UTC` seconds; source IERS Bulletin C, as-of 2026-06):
```csv
effective_jd_utc,tai_minus_utc
2441317.5,10
2441499.5,11
2441683.5,12
2442048.5,13
2442413.5,14
2442778.5,15
2443144.5,16
2443509.5,17
2443874.5,18
2444239.5,19
2444786.5,20
2445151.5,21
2445516.5,22
2446247.5,23
2447161.5,24
2447892.5,25
2448257.5,26
2448804.5,27
2449169.5,28
2449534.5,29
2450083.5,30
2450630.5,31
2451179.5,32
2453736.5,33
2454832.5,34
2456109.5,35
2457204.5,36
2457754.5,37
```

- [ ] **Step 2: Write the module with a deliberately wrong checksum constant + a failing test**

`crates/pleiades-time/src/leap.rs`:
```rust
//! Leap-second table (`TAI − UTC`) and lookup, checksum-pinned and fail-closed.

use crate::error::CivilTimeError;
use crate::fnv1a64;

const LEAP_CSV: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/leap-seconds.csv"));

/// FNV-1a checksum of `data/leap-seconds.csv`. Regenerate with the `pinned_checksum`
/// test below if the table is updated, and bump `VALID_THROUGH_JD` accordingly.
const LEAP_CSV_CHECKSUM: u64 = 0; // replaced in Step 4

/// JD of the first UTC leap-second epoch (1972-01-01 00:00).
pub const LEAP_EPOCH_JD: f64 = 2441317.5;

/// Last UTC date the table is authoritative for (2025-12-31 00:00; no leap second
/// announced through 2025 per IERS Bulletin C, as-of 2026-06).
pub const VALID_THROUGH_JD: f64 = 2461040.5;

fn table() -> Result<Vec<(f64, i32)>, CivilTimeError> {
    if fnv1a64(LEAP_CSV) != LEAP_CSV_CHECKSUM {
        return Err(CivilTimeError::StaleTimeData { kind: "leap-second" });
    }
    let mut rows = Vec::new();
    for line in LEAP_CSV.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split(',');
        let jd: f64 = parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .ok_or(CivilTimeError::StaleTimeData { kind: "leap-second" })?;
        let secs: i32 = parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .ok_or(CivilTimeError::StaleTimeData { kind: "leap-second" })?;
        rows.push((jd, secs));
    }
    Ok(rows)
}

/// Returns `TAI − UTC` in whole seconds for a UTC instant, or `Ok(None)` if the
/// instant is before 1972 or after the table's validated horizon.
pub fn tai_minus_utc(jd_utc: f64) -> Result<Option<i32>, CivilTimeError> {
    if jd_utc < LEAP_EPOCH_JD || jd_utc > VALID_THROUGH_JD {
        return Ok(None);
    }
    let rows = table()?;
    let mut current = None;
    for (effective, secs) in rows {
        if jd_utc >= effective {
            current = Some(secs);
        }
    }
    Ok(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_checksum() {
        // If this fails after a deliberate table edit, copy the printed value into
        // LEAP_CSV_CHECKSUM and bump VALID_THROUGH_JD.
        assert_eq!(fnv1a64(LEAP_CSV), LEAP_CSV_CHECKSUM, "checksum = {}", fnv1a64(LEAP_CSV));
    }

    #[test]
    fn lookup_at_known_boundaries() {
        // 2017-01-01 (JD 2457754.5) -> 37
        assert_eq!(tai_minus_utc(2457754.5).unwrap(), Some(37));
        // 2000-01-01 12:00 (JD 2451545.0) -> 32 (1999-01-01 epoch)
        assert_eq!(tai_minus_utc(2451545.0).unwrap(), Some(32));
        // 1972-01-01 -> 10
        assert_eq!(tai_minus_utc(2441317.5).unwrap(), Some(10));
    }

    #[test]
    fn returns_none_outside_window() {
        assert_eq!(tai_minus_utc(2441317.4).unwrap(), None); // before 1972
        assert_eq!(tai_minus_utc(VALID_THROUGH_JD + 1.0).unwrap(), None); // past horizon
    }
}
```

- [ ] **Step 3: Wire into `lib.rs`**

Add to `crates/pleiades-time/src/lib.rs`:
```rust
pub mod leap;
```

- [ ] **Step 4: Compute and pin the real checksum**

Run: `cargo test -p pleiades-time leap::tests::pinned_checksum -- --nocapture`
Expected: FAIL printing `checksum = <N>`. Copy `<N>` into `LEAP_CSV_CHECKSUM` (e.g. `const LEAP_CSV_CHECKSUM: u64 = 0xXXXX;`).

- [ ] **Step 5: Run the leap tests to verify they pass**

Run: `cargo test -p pleiades-time leap::`
Expected: `pinned_checksum`, `lookup_at_known_boundaries`, `returns_none_outside_window` all PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-time/data/leap-seconds.csv crates/pleiades-time/src/leap.rs crates/pleiades-time/src/lib.rs
git commit -m "feat(time): add checksum-pinned leap-second table and lookup"
```

---

## Task 5: `deltat` module — observed ΔT table + extrapolation

**Files:**
- Create: `crates/pleiades-time/data/delta-t-observed.csv`
- Create: `crates/pleiades-time/src/deltat.rs`
- Modify: `crates/pleiades-time/src/lib.rs`
- Test: `crates/pleiades-time/src/deltat.rs` (inline tests)

**Interfaces:**
- Consumes: `CivilTimeError`, `fnv1a64`, `pleiades_types::JulianDay`.
- Produces: `pleiades_time::DeltaTQuality { Observed, Predicted }` (note: `Exact` lives on `ConversionProvenance`, not here — `deltat` never returns `Exact`); `deltat::delta_t(jd: f64) -> Result<(f64, DeltaTQuality), CivilTimeError>`; `deltat::OBSERVED_THROUGH_JD: f64`.

- [ ] **Step 1: Create the pinned data file**

`crates/pleiades-time/data/delta-t-observed.csv` (decadal observed `ΔT = TT − UT1` seconds; source IERS/USNO + Espenak–Meeus, as-of 2026-06):
```csv
year,delta_t_seconds
1900,-2.8
1910,10.4
1920,21.2
1930,24.0
1940,24.3
1950,29.1
1960,33.2
1970,40.2
1980,50.5
1990,56.9
2000,63.8
2010,66.1
2020,69.4
```

- [ ] **Step 2: Write the module with a wrong checksum + failing tests**

`crates/pleiades-time/src/deltat.rs`:
```rust
//! Delta-T (`ΔT = TT − UT1`): checksum-pinned observed table with linear
//! interpolation, plus a documented polynomial extrapolation beyond it.

use crate::error::CivilTimeError;
use crate::fnv1a64;

const DELTA_T_CSV: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/delta-t-observed.csv"));

/// FNV-1a checksum of `data/delta-t-observed.csv`; pinned in Step 4.
const DELTA_T_CSV_CHECKSUM: u64 = 0; // replaced in Step 4

/// JD of the last observed ΔT node (2020-01-01 00:00). Beyond this, ΔT is `Predicted`.
pub const OBSERVED_THROUGH_JD: f64 = 2458849.5;

/// Quality of a Delta-T value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeltaTQuality {
    /// Interpolated from the committed observation table.
    Observed,
    /// Extrapolated by the documented polynomial beyond the observed table.
    Predicted,
}

fn table() -> Result<Vec<(f64, f64)>, CivilTimeError> {
    if fnv1a64(DELTA_T_CSV) != DELTA_T_CSV_CHECKSUM {
        return Err(CivilTimeError::StaleTimeData { kind: "delta-t" });
    }
    let mut rows = Vec::new();
    for line in DELTA_T_CSV.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split(',');
        let year: f64 = parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .ok_or(CivilTimeError::StaleTimeData { kind: "delta-t" })?;
        let dt: f64 = parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .ok_or(CivilTimeError::StaleTimeData { kind: "delta-t" })?;
        rows.push((year, dt));
    }
    Ok(rows)
}

/// Approximate decimal year from a Julian Day (good enough for ΔT, which varies slowly).
fn decimal_year(jd: f64) -> f64 {
    2000.0 + (jd - 2451545.0) / 365.25
}

/// Espenak–Meeus future extrapolation (years 2005–2050 form), used beyond the
/// observed table. See https://eclipse.gsfc.nasa.gov/SEcat5/deltatpoly.html
fn extrapolate(year: f64) -> f64 {
    let t = year - 2000.0;
    62.92 + 0.32217 * t + 0.005589 * t * t
}

/// Returns `(ΔT seconds, quality)` for a Julian Day. The orchestrator is
/// responsible for the 1900–2100 horizon; this function extrapolates past the
/// observed table without an upper bound.
pub fn delta_t(jd: f64) -> Result<(f64, DeltaTQuality), CivilTimeError> {
    let year = decimal_year(jd);
    let rows = table()?;
    let first = rows[0];
    let last = rows[rows.len() - 1];
    if year <= first.0 {
        return Ok((first.1, DeltaTQuality::Observed));
    }
    if year >= last.0 {
        // Past the observed table -> predicted extrapolation.
        return Ok((extrapolate(year), DeltaTQuality::Predicted));
    }
    // Linear interpolation between bracketing observed nodes.
    for pair in rows.windows(2) {
        let (y0, d0) = pair[0];
        let (y1, d1) = pair[1];
        if year >= y0 && year <= y1 {
            let frac = (year - y0) / (y1 - y0);
            return Ok((d0 + frac * (d1 - d0), DeltaTQuality::Observed));
        }
    }
    unreachable!("year is between first and last nodes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_checksum() {
        assert_eq!(
            fnv1a64(DELTA_T_CSV),
            DELTA_T_CSV_CHECKSUM,
            "checksum = {}",
            fnv1a64(DELTA_T_CSV)
        );
    }

    #[test]
    fn observed_spot_values() {
        // 2000-01-01 12:00 -> node 2000 -> 63.8, Observed
        let (dt, q) = delta_t(2451545.0).unwrap();
        assert!((dt - 63.8).abs() < 0.5, "got {dt}");
        assert_eq!(q, DeltaTQuality::Observed);
        // 1900 node -> -2.8
        let (dt, _) = delta_t(2415020.5).unwrap();
        assert!((dt - (-2.8)).abs() < 0.5, "got {dt}");
    }

    #[test]
    fn future_is_predicted() {
        // 2080-ish: past the 2020 observed node -> Predicted
        let (dt, q) = delta_t(2480000.0).unwrap();
        assert_eq!(q, DeltaTQuality::Predicted);
        assert!(dt > 69.0, "got {dt}");
    }
}
```

- [ ] **Step 3: Wire into `lib.rs`**

Add to `crates/pleiades-time/src/lib.rs`:
```rust
pub mod deltat;

pub use deltat::DeltaTQuality;
```

- [ ] **Step 4: Pin the checksum**

Run: `cargo test -p pleiades-time deltat::tests::pinned_checksum -- --nocapture`
Expected: FAIL printing `checksum = <N>`. Copy `<N>` into `DELTA_T_CSV_CHECKSUM`.

- [ ] **Step 5: Run the deltat tests**

Run: `cargo test -p pleiades-time deltat::`
Expected: `pinned_checksum`, `observed_spot_values`, `future_is_predicted` PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-time/data/delta-t-observed.csv crates/pleiades-time/src/deltat.rs crates/pleiades-time/src/lib.rs
git commit -m "feat(time): add checksum-pinned Delta-T table with extrapolation"
```

---

## Task 6: `tdb` module — TT ↔ TDB periodic term

**Files:**
- Create: `crates/pleiades-time/src/tdb.rs`
- Modify: `crates/pleiades-time/src/lib.rs`
- Test: `crates/pleiades-time/src/tdb.rs` (inline tests)

**Interfaces:**
- Produces: `tdb::tdb_minus_tt_seconds(jd_tt: f64) -> f64` (deterministic, sub-ms).

- [ ] **Step 1: Write the failing test + implementation**

`crates/pleiades-time/src/tdb.rs`:
```rust
//! Deterministic TT <-> TDB periodic term (USNO approximation, sub-millisecond).

/// `TDB − TT` in seconds for a TT Julian Day. Standard low-precision model:
/// `g = 357.53° + 0.9856003° * (JD_TT − 2451545.0)`,
/// `TDB − TT = 0.001658 sin g + 0.000014 sin 2g`.
pub fn tdb_minus_tt_seconds(jd_tt: f64) -> f64 {
    let g_deg = 357.53 + 0.985_600_3 * (jd_tt - 2451545.0);
    let g = g_deg.to_radians();
    0.001_658 * g.sin() + 0.000_014 * (2.0 * g).sin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_below_two_milliseconds() {
        for offset in [-36525.0, -18262.0, 0.0, 18262.0, 36525.0] {
            let v = tdb_minus_tt_seconds(2451545.0 + offset);
            assert!(v.abs() < 0.002, "TDB-TT {v} out of bound at offset {offset}");
        }
    }
}
```

- [ ] **Step 2: Wire into `lib.rs`**

Add to `crates/pleiades-time/src/lib.rs`:
```rust
pub mod tdb;
```

- [ ] **Step 3: Run the test**

Run: `cargo test -p pleiades-time tdb::`
Expected: `bounded_below_two_milliseconds` PASSES.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-time/src/tdb.rs crates/pleiades-time/src/lib.rs
git commit -m "feat(time): add TT<->TDB periodic term"
```

---

## Task 7: `convert` orchestrator — `to_terrestrial` + provenance

**Files:**
- Create: `crates/pleiades-time/src/convert.rs`
- Modify: `crates/pleiades-time/src/lib.rs`
- Test: `crates/pleiades-time/src/convert.rs` (inline tests)

**Interfaces:**
- Consumes: `CivilDateTime`, `CivilTimeError`, `DeltaTQuality`, `leap::tai_minus_utc`, `deltat::delta_t`, `tdb::tdb_minus_tt_seconds`, `pleiades_types::{Instant, JulianDay, TimeScale}`.
- Produces:
  - `pleiades_time::ConversionPath { UtcLeapSecond, Ut1DeltaT, FutureExtrapolated }`
  - `pleiades_time::ConversionProvenance { path, quality, delta_t_seconds: Option<f64>, tai_minus_utc: Option<i32>, sources: &'static str }` with `summary_line(&self) -> String` + `Display`
  - `pleiades_time::CivilInstant { instant: Instant, provenance: ConversionProvenance }`
  - `pleiades_time::to_terrestrial(civil, source: TimeScale, target: TimeScale) -> Result<CivilInstant, CivilTimeError>`
  - wrappers `tt_from_utc_civil`, `tdb_from_utc_civil`, `tt_from_ut1_civil`, `tdb_from_ut1_civil`
  - `pleiades_time::SUPPORT_START_JD`, `SUPPORT_END_JD`

- [ ] **Step 1: Write the failing tests + implementation**

`crates/pleiades-time/src/convert.rs`:
```rust
//! Orchestrator: civil datetime + scales -> tagged TT/TDB Instant with provenance.

use core::fmt;

use pleiades_types::{Instant, JulianDay, TimeScale, SECONDS_PER_DAY};

use crate::calendar::CivilDateTime;
use crate::deltat::{self, DeltaTQuality};
use crate::error::CivilTimeError;
use crate::leap;
use crate::tdb;

/// Start of the supported civil window (1900-01-01 00:00).
pub const SUPPORT_START_JD: f64 = 2415020.5;
/// End of the supported civil window (2100-12-31 24:00 ~= 2101-01-01 00:00).
pub const SUPPORT_END_JD: f64 = 2488434.5;

/// TT − TAI, in seconds (fixed by definition).
const TT_MINUS_TAI: f64 = 32.184;

const SOURCES: &str =
    "leap-seconds.csv (IERS Bulletin C); delta-t-observed.csv (IERS/USNO + Espenak–Meeus); as-of 2026-06";

/// Which path the orchestrator took.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConversionPath {
    /// UTC input converted via the leap-second table (exact).
    UtcLeapSecond,
    /// UT1 input converted via the observed Delta-T table.
    Ut1DeltaT,
    /// Input beyond a validated table, converted via Delta-T extrapolation.
    FutureExtrapolated,
}

impl fmt::Display for ConversionPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UtcLeapSecond => "utc-leap-second",
            Self::Ut1DeltaT => "ut1-delta-t",
            Self::FutureExtrapolated => "future-extrapolated",
        })
    }
}

/// Overall conversion quality, the truthful-claims marker on every result.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConversionQuality {
    /// Leap-second-exact (no Delta-T model error).
    Exact,
    /// From the observed Delta-T table.
    Observed,
    /// From Delta-T extrapolation.
    Predicted,
}

impl fmt::Display for ConversionQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Exact => "exact",
            Self::Observed => "observed",
            Self::Predicted => "predicted",
        })
    }
}

/// Provenance describing how a civil instant was converted.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConversionProvenance {
    pub path: ConversionPath,
    pub quality: ConversionQuality,
    pub delta_t_seconds: Option<f64>,
    pub tai_minus_utc: Option<i32>,
    pub sources: &'static str,
}

impl ConversionProvenance {
    /// Compact one-line rendering for diagnostics and release-facing summaries.
    pub fn summary_line(&self) -> String {
        format!(
            "civil-time path={} quality={} delta_t={} tai_minus_utc={}",
            self.path,
            self.quality,
            self.delta_t_seconds
                .map(|d| format!("{d:.3}s"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.tai_minus_utc
                .map(|t| format!("{t}s"))
                .unwrap_or_else(|| "n/a".to_string()),
        )
    }
}

impl fmt::Display for ConversionProvenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A converted instant plus the provenance describing how it was produced.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CivilInstant {
    pub instant: Instant,
    pub provenance: ConversionProvenance,
}

fn finite(jd: f64) -> Result<(), CivilTimeError> {
    if jd.is_finite() {
        Ok(())
    } else {
        Err(CivilTimeError::NonFiniteOffset)
    }
}

/// Builds a TT instant (and provenance) from a civil Julian Day tagged UTC or UT1.
fn to_tt(jd_civil: f64, source: TimeScale) -> Result<(f64, ConversionProvenance), CivilTimeError> {
    if jd_civil < SUPPORT_START_JD || jd_civil > SUPPORT_END_JD {
        return Err(CivilTimeError::BeyondHorizon { jd: jd_civil });
    }
    match source {
        TimeScale::Utc => {
            if jd_civil < leap::LEAP_EPOCH_JD {
                return Err(CivilTimeError::UtcBeforeLeapEpoch);
            }
            if let Some(tai_minus_utc) = leap::tai_minus_utc(jd_civil)? {
                let offset = tai_minus_utc as f64 + TT_MINUS_TAI;
                let jd_tt = jd_civil + offset / SECONDS_PER_DAY;
                finite(jd_tt)?;
                return Ok((
                    jd_tt,
                    ConversionProvenance {
                        path: ConversionPath::UtcLeapSecond,
                        quality: ConversionQuality::Exact,
                        delta_t_seconds: None,
                        tai_minus_utc: Some(tai_minus_utc),
                        sources: SOURCES,
                    },
                ));
            }
            // Future UTC beyond the leap table: fall back to Delta-T extrapolation.
            let (dt, _q) = deltat::delta_t(jd_civil)?;
            let jd_tt = jd_civil + dt / SECONDS_PER_DAY;
            finite(jd_tt)?;
            Ok((
                jd_tt,
                ConversionProvenance {
                    path: ConversionPath::FutureExtrapolated,
                    quality: ConversionQuality::Predicted,
                    delta_t_seconds: Some(dt),
                    tai_minus_utc: None,
                    sources: SOURCES,
                },
            ))
        }
        TimeScale::Ut1 => {
            let (dt, q) = deltat::delta_t(jd_civil)?;
            let jd_tt = jd_civil + dt / SECONDS_PER_DAY;
            finite(jd_tt)?;
            let (path, quality) = match q {
                DeltaTQuality::Observed => (ConversionPath::Ut1DeltaT, ConversionQuality::Observed),
                DeltaTQuality::Predicted => {
                    (ConversionPath::FutureExtrapolated, ConversionQuality::Predicted)
                }
            };
            Ok((
                jd_tt,
                ConversionProvenance {
                    path,
                    quality,
                    delta_t_seconds: Some(dt),
                    tai_minus_utc: None,
                    sources: SOURCES,
                },
            ))
        }
        other => Err(CivilTimeError::UnsupportedScale { source: other, target: TimeScale::Tt }),
    }
}

/// Converts a civil datetime tagged `source` (UTC or UT1) to `target` (TT or TDB).
pub fn to_terrestrial(
    civil: CivilDateTime,
    source: TimeScale,
    target: TimeScale,
) -> Result<CivilInstant, CivilTimeError> {
    if !matches!(source, TimeScale::Utc | TimeScale::Ut1) {
        return Err(CivilTimeError::UnsupportedScale { source, target });
    }
    if !matches!(target, TimeScale::Tt | TimeScale::Tdb) {
        return Err(CivilTimeError::UnsupportedScale { source, target });
    }
    let jd_civil = civil.to_julian_day()?.days();
    let (jd_tt, provenance) = to_tt(jd_civil, source)?;
    let (jd_out, scale) = match target {
        TimeScale::Tt => (jd_tt, TimeScale::Tt),
        TimeScale::Tdb => {
            let jd_tdb = jd_tt + tdb::tdb_minus_tt_seconds(jd_tt) / SECONDS_PER_DAY;
            finite(jd_tdb)?;
            (jd_tdb, TimeScale::Tdb)
        }
        _ => unreachable!("guarded above"),
    };
    Ok(CivilInstant {
        instant: Instant::new(JulianDay::from_days(jd_out), scale),
        provenance,
    })
}

/// Convenience: UTC civil -> TT.
pub fn tt_from_utc_civil(civil: CivilDateTime) -> Result<CivilInstant, CivilTimeError> {
    to_terrestrial(civil, TimeScale::Utc, TimeScale::Tt)
}
/// Convenience: UTC civil -> TDB.
pub fn tdb_from_utc_civil(civil: CivilDateTime) -> Result<CivilInstant, CivilTimeError> {
    to_terrestrial(civil, TimeScale::Utc, TimeScale::Tdb)
}
/// Convenience: UT1 civil -> TT.
pub fn tt_from_ut1_civil(civil: CivilDateTime) -> Result<CivilInstant, CivilTimeError> {
    to_terrestrial(civil, TimeScale::Ut1, TimeScale::Tt)
}
/// Convenience: UT1 civil -> TDB.
pub fn tdb_from_ut1_civil(civil: CivilDateTime) -> Result<CivilInstant, CivilTimeError> {
    to_terrestrial(civil, TimeScale::Ut1, TimeScale::Tdb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_modern_is_exact() {
        // 2017-01-01 00:00 UTC -> TT. Offset = 37 + 32.184 = 69.184 s.
        let civil = CivilDateTime::new(2017, 1, 1, 0, 0, 0.0);
        let out = tt_from_utc_civil(civil).unwrap();
        assert_eq!(out.instant.scale, TimeScale::Tt);
        assert_eq!(out.provenance.quality, ConversionQuality::Exact);
        assert_eq!(out.provenance.tai_minus_utc, Some(37));
        let expected_jd = civil.to_julian_day().unwrap().days() + 69.184 / SECONDS_PER_DAY;
        assert!((out.instant.julian_day.days() - expected_jd).abs() < 1e-9);
    }

    #[test]
    fn ut1_historical_is_observed() {
        let civil = CivilDateTime::new(1950, 1, 1, 0, 0, 0.0);
        let out = tt_from_ut1_civil(civil).unwrap();
        assert_eq!(out.provenance.quality, ConversionQuality::Observed);
        assert!(out.provenance.delta_t_seconds.unwrap() > 28.0);
    }

    #[test]
    fn future_utc_is_predicted() {
        let civil = CivilDateTime::new(2090, 6, 1, 0, 0, 0.0);
        let out = tt_from_utc_civil(civil).unwrap();
        assert_eq!(out.provenance.quality, ConversionQuality::Predicted);
        assert_eq!(out.provenance.path, ConversionPath::FutureExtrapolated);
    }

    #[test]
    fn pre_1972_utc_is_rejected() {
        let civil = CivilDateTime::new(1965, 1, 1, 0, 0, 0.0);
        assert_eq!(tt_from_utc_civil(civil), Err(CivilTimeError::UtcBeforeLeapEpoch));
    }

    #[test]
    fn outside_window_is_rejected() {
        let civil = CivilDateTime::new(1880, 1, 1, 0, 0, 0.0);
        assert!(matches!(
            tt_from_ut1_civil(civil),
            Err(CivilTimeError::BeyondHorizon { .. })
        ));
    }

    #[test]
    fn bad_target_scale_is_rejected() {
        let civil = CivilDateTime::new(2000, 1, 1, 0, 0, 0.0);
        assert_eq!(
            to_terrestrial(civil, TimeScale::Utc, TimeScale::Ut1),
            Err(CivilTimeError::UnsupportedScale { source: TimeScale::Utc, target: TimeScale::Ut1 })
        );
    }

    #[test]
    fn tdb_differs_from_tt_sub_millisecond() {
        let civil = CivilDateTime::new(2000, 4, 1, 0, 0, 0.0);
        let tt = tt_from_utc_civil(civil).unwrap().instant.julian_day().days();
        let tdb = tdb_from_utc_civil(civil).unwrap().instant.julian_day().days();
        let diff_s = (tdb - tt).abs() * SECONDS_PER_DAY;
        assert!(diff_s < 0.002 && diff_s > 0.0, "diff {diff_s}s");
    }
}
```

NOTE: `Instant` exposes public fields `scale` and `julian_day` (a `JulianDay`) — the tests above use `out.instant.scale` and `out.instant.julian_day.days()` accordingly (no method-call parentheses).

- [ ] **Step 2: Wire into `lib.rs`**

Add to `crates/pleiades-time/src/lib.rs`:
```rust
mod convert;

pub use convert::{
    to_terrestrial, tdb_from_ut1_civil, tdb_from_utc_civil, tt_from_ut1_civil, tt_from_utc_civil,
    CivilInstant, ConversionPath, ConversionProvenance, ConversionQuality, SUPPORT_END_JD,
    SUPPORT_START_JD,
};
```

- [ ] **Step 3: Run the convert tests**

Run: `cargo test -p pleiades-time convert::`
Expected: all seven tests PASS. (If `scale()`/`julian_day()` names mismatch, fix per the NOTE and re-run.)

- [ ] **Step 4: Run the whole crate**

Run: `cargo test -p pleiades-time`
Expected: every test PASSES.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-time/src/convert.rs crates/pleiades-time/src/lib.rs
git commit -m "feat(time): add civil-time orchestrator with tiered provenance"
```

---

## Task 8: `policy` module + reverse the backend non-goal posture

**Files:**
- Create: `crates/pleiades-time/src/policy.rs`
- Modify: `crates/pleiades-time/src/lib.rs`
- Modify: `crates/pleiades-backend/src/policy/current.rs` (posture text constants)
- Modify: `crates/pleiades-backend/src/policy_tests.rs` (assert the new posture)
- Test: `crates/pleiades-time/src/policy.rs` (inline tests)

**Interfaces:**
- Produces: `pleiades_time::CivilTimePolicySummary` with `current() -> Self`, `summary_line(&self) -> &'static str`, `validate(&self) -> Result<(), CivilTimePolicyError>`, `Display`. Mirrors the `DeltaTPolicySummary` shape in `crates/pleiades-backend/src/policy/delta_t.rs`.

- [ ] **Step 1: Locate the current posture constants**

Run: `grep -n "CURRENT_DELTA_T_POLICY_SUMMARY_TEXT\|CURRENT_UTC_CONVENIENCE" crates/pleiades-backend/src/policy/current.rs`
Read the two constants; they currently say built-in Delta-T / UTC conversion is deferred/out of scope.

- [ ] **Step 2: Write the `pleiades-time` policy summary (failing test)**

`crates/pleiades-time/src/policy.rs`:
```rust
//! Typed, drift-checked summary of the civil-time conversion posture.

use core::fmt;

/// The canonical one-line civil-time posture. Update with the implementation if
/// the supported window or tier model changes; the validator fails closed on drift.
pub const CURRENT_CIVIL_TIME_POLICY_SUMMARY_TEXT: &str =
    "Civil UTC/UT1 input converts to TT/TDB over 1900–2100: leap-second-exact UTC, observed/extrapolated Delta-T elsewhere, each result tagged exact/observed/predicted.";

/// Validation error for the civil-time policy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CivilTimePolicyError {
    BlankSummary,
    WhitespacePaddedSummary,
    EmbeddedLineBreak,
    CurrentPolicyOutOfSync,
}

impl fmt::Display for CivilTimePolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BlankSummary => "civil-time policy summary is blank",
            Self::WhitespacePaddedSummary => "civil-time policy summary has surrounding whitespace",
            Self::EmbeddedLineBreak => "civil-time policy summary contains a line break",
            Self::CurrentPolicyOutOfSync => "civil-time policy summary is out of sync with the current posture",
        })
    }
}

impl std::error::Error for CivilTimePolicyError {}

/// Compact summary of the current civil-time conversion posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CivilTimePolicySummary {
    summary: &'static str,
}

impl CivilTimePolicySummary {
    pub const fn new(summary: &'static str) -> Self {
        Self { summary }
    }
    pub const fn current() -> Self {
        Self::new(CURRENT_CIVIL_TIME_POLICY_SUMMARY_TEXT)
    }
    pub const fn summary_line(self) -> &'static str {
        self.summary
    }
    pub fn validate(&self) -> Result<(), CivilTimePolicyError> {
        if self.summary.trim().is_empty() {
            Err(CivilTimePolicyError::BlankSummary)
        } else if self.summary.trim() != self.summary {
            Err(CivilTimePolicyError::WhitespacePaddedSummary)
        } else if self.summary.contains('\n') || self.summary.contains('\r') {
            Err(CivilTimePolicyError::EmbeddedLineBreak)
        } else if self.summary != CURRENT_CIVIL_TIME_POLICY_SUMMARY_TEXT {
            Err(CivilTimePolicyError::CurrentPolicyOutOfSync)
        } else {
            Ok(())
        }
    }
    pub fn validated_summary_line(&self) -> Result<&'static str, CivilTimePolicyError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for CivilTimePolicySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.summary_line())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_summary_validates() {
        assert!(CivilTimePolicySummary::current().validate().is_ok());
    }

    #[test]
    fn drifted_summary_is_rejected() {
        assert_eq!(
            CivilTimePolicySummary::new("stale").validate(),
            Err(CivilTimePolicyError::CurrentPolicyOutOfSync)
        );
    }
}
```

- [ ] **Step 3: Wire into `lib.rs`**

Add to `crates/pleiades-time/src/lib.rs`:
```rust
pub mod policy;

pub use policy::{CivilTimePolicyError, CivilTimePolicySummary};
```

- [ ] **Step 4: Update the backend posture constants**

In `crates/pleiades-backend/src/policy/current.rs`, replace the deferred wording in the UTC-convenience and Delta-T posture constants so they describe the now-supported `pleiades-time` path (point to the crate; state that built-in civil UTC/UT1 → TT/TDB now exists with tiered quality). Keep them single-line, no surrounding whitespace.

- [ ] **Step 5: Update the backend policy tests to the new posture**

In `crates/pleiades-backend/src/policy_tests.rs`, update the expected-string assertions that pin the old "deferred/out of scope" UTC-convenience and Delta-T wording to the new posture text. (Search for `utc_convenience_policy_summary_for_report` and `delta_t_policy_summary_for_report` assertions.)

- [ ] **Step 6: Run affected tests**

Run: `cargo test -p pleiades-time policy:: && cargo test -p pleiades-backend policy`
Expected: `pleiades-time` policy tests PASS; backend policy tests PASS with the new posture.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-time/src/policy.rs crates/pleiades-time/src/lib.rs crates/pleiades-backend/src/policy/current.rs crates/pleiades-backend/src/policy_tests.rs
git commit -m "feat(time): add civil-time policy summary and reverse backend non-goal posture"
```

---

## Task 9: `pleiades-core` integration — `ChartRequest::from_civil`

**Files:**
- Modify: `crates/pleiades-core/Cargo.toml` (add `pleiades-time` dep)
- Modify: `crates/pleiades-core/src/chart.rs` (add `from_civil` + provenance field/accessor)
- Modify: `crates/pleiades-core/src/lib.rs` (re-export civil-time types)
- Test: `crates/pleiades-core/src/chart.rs` (inline tests) or the crate's existing test module

**Interfaces:**
- Consumes: `pleiades_time::{CivilDateTime, to_terrestrial, ConversionProvenance}`, `pleiades_types::TimeScale`, existing `ChartRequest::new(instant)` and its public `bodies` field.
- Produces: `pleiades_core::CivilChartRequest { pub request: ChartRequest, pub provenance: ConversionProvenance }`; `ChartRequest::from_civil(civil: CivilDateTime, source: TimeScale, target: TimeScale, bodies: Vec<CelestialBody>) -> Result<CivilChartRequest, CivilTimeError>`. (A wrapper is used rather than adding a field to the all-public-fields `ChartRequest`, so the change is additive and non-breaking.)

- [ ] **Step 1: Add the dependency**

In `crates/pleiades-core/Cargo.toml`, under `[dependencies]`:
```toml
pleiades-time = { workspace = true }
```

- [ ] **Step 2: Confirm the `ChartRequest` shape**

Run: `sed -n '50,90p' crates/pleiades-core/src/chart/request.rs`
Confirm `ChartRequest::new(instant: Instant)` takes only the instant, that `bodies` is a public field, and that `default_chart_bodies()` is in scope. The civil builder will use `ChartRequest::new(...)` then set `request.bodies`, leaving all existing builders untouched.

- [ ] **Step 3: Write the failing test**

Add to the test module in `crates/pleiades-core/src/chart/request.rs`:
```rust
#[test]
fn from_civil_builds_tt_request_with_provenance() {
    use pleiades_time::CivilDateTime;
    use pleiades_types::TimeScale;

    let civil = CivilDateTime::new(2017, 1, 1, 0, 0, 0.0);
    let built = ChartRequest::from_civil(
        civil,
        TimeScale::Utc,
        TimeScale::Tt,
        default_chart_bodies().to_vec(),
    )
    .unwrap();
    assert_eq!(built.provenance.tai_minus_utc, Some(37));
    assert_eq!(built.request.instant.scale, TimeScale::Tt);
}
```

- [ ] **Step 4: Add the `CivilChartRequest` wrapper + `from_civil`**

In `crates/pleiades-core/src/chart/request.rs`:
```rust
/// A chart request built from a civil datetime, paired with the time-conversion
/// provenance produced by `pleiades-time`.
#[derive(Clone, Debug, PartialEq)]
pub struct CivilChartRequest {
    pub request: ChartRequest,
    pub provenance: pleiades_time::ConversionProvenance,
}

impl ChartRequest {
    /// Builds a chart request from a civil datetime, converting to TT/TDB through
    /// `pleiades-time`. Additive: the caller-supplied retagging builders are
    /// unaffected. The returned wrapper carries the conversion provenance.
    pub fn from_civil(
        civil: pleiades_time::CivilDateTime,
        source: pleiades_types::TimeScale,
        target: pleiades_types::TimeScale,
        bodies: Vec<crate::CelestialBody>,
    ) -> Result<CivilChartRequest, pleiades_time::CivilTimeError> {
        let converted = pleiades_time::to_terrestrial(civil, source, target)?;
        let mut request = ChartRequest::new(converted.instant);
        request.bodies = bodies;
        Ok(CivilChartRequest { request, provenance: converted.provenance })
    }
}
```
(If `CelestialBody` is not in scope as `crate::CelestialBody`, import it consistently with how `request.rs` already names body types.)

- [ ] **Step 5: Re-export from the core crate root**

In `crates/pleiades-core/src/lib.rs`, add (and export `CivilChartRequest` from the `chart` module path it lives under):
```rust
pub use pleiades_time::{CivilDateTime, CivilInstant, CivilTimeError, ConversionProvenance};
```
Ensure `CivilChartRequest` is re-exported alongside `ChartRequest` wherever the `chart` module re-exports its public types.

- [ ] **Step 6: Run the tests**

Run: `cargo test -p pleiades-core from_civil`
Expected: `from_civil_builds_tt_request_with_provenance` PASSES.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-core/Cargo.toml crates/pleiades-core/src/chart/request.rs crates/pleiades-core/src/lib.rs
git commit -m "feat(core): add ChartRequest::from_civil via pleiades-time"
```

---

## Task 10: CLI `--civil` support

**Files:**
- Modify: `crates/pleiades-cli/Cargo.toml` (ensure `pleiades-time` reachable — usually via `pleiades-core` re-export)
- Modify: `crates/pleiades-cli/src/commands/chart.rs` (parse `--civil`, `--civil-scale`, `--civil-target`; render provenance)
- Modify: `crates/pleiades-cli/src/help.rs` (document the flags)
- Test: `crates/pleiades-cli/src/commands/chart.rs` (inline tests)

**Interfaces:**
- Consumes: `pleiades_core::{CivilDateTime}` (or `pleiades_time` directly), `ChartRequest::from_civil`.
- Produces: CLI behavior — `--civil "YYYY-MM-DDTHH:MM:SS"` + `--civil-scale utc|ut1` (default `utc`) + `--civil-target tt|tdb` (default `tt`).

- [ ] **Step 1: Read the existing flag-parsing structure**

Run: `sed -n '212,300p' crates/pleiades-cli/src/commands/chart.rs`
Note the `match arg { ... }` loop and how `--jd`/`--tt`/`--utc` set state, and how mutually-exclusive flags are rejected (the existing offset-flag rejection messages).

- [ ] **Step 2: Write a failing CLI test**

Add to the chart test module in `crates/pleiades-cli/src/commands/chart.rs`:
```rust
#[test]
fn civil_flag_parses_and_reports_provenance() {
    let out = render_chart(&[
        "--civil", "2017-01-01T00:00:00",
        "--civil-scale", "utc",
        "--body", "sun",
    ])
    .expect("chart renders");
    assert!(out.contains("path=utc-leap-second"), "missing provenance: {out}");
    assert!(out.contains("quality=exact"), "missing quality: {out}");
}

#[test]
fn civil_flag_conflicts_with_jd() {
    let err = render_chart(&["--civil", "2017-01-01T00:00:00", "--jd", "2451545.0"])
        .expect_err("should reject mixing --civil with --jd");
    assert!(err.contains("--civil"));
}
```

- [ ] **Step 3: Add a civil-datetime parser helper**

In `crates/pleiades-cli/src/commands/chart.rs`, add:
```rust
fn parse_civil(value: Option<&str>) -> Result<pleiades_core::CivilDateTime, String> {
    let raw = value.ok_or_else(|| "--civil requires a YYYY-MM-DDTHH:MM:SS value".to_string())?;
    let (date, time) = raw
        .split_once('T')
        .ok_or_else(|| format!("--civil value '{raw}' must be YYYY-MM-DDTHH:MM:SS"))?;
    let d: Vec<&str> = date.split('-').collect();
    let t: Vec<&str> = time.split(':').collect();
    if d.len() != 3 || t.len() != 3 {
        return Err(format!("--civil value '{raw}' must be YYYY-MM-DDTHH:MM:SS"));
    }
    let year = d[0].parse::<i32>().map_err(|_| "--civil year".to_string())?;
    let month = d[1].parse::<u8>().map_err(|_| "--civil month".to_string())?;
    let day = d[2].parse::<u8>().map_err(|_| "--civil day".to_string())?;
    let hour = t[0].parse::<u8>().map_err(|_| "--civil hour".to_string())?;
    let minute = t[1].parse::<u8>().map_err(|_| "--civil minute".to_string())?;
    let second = t[2].parse::<f64>().map_err(|_| "--civil second".to_string())?;
    Ok(pleiades_core::CivilDateTime::new(year, month, day, hour, minute, second))
}
```

- [ ] **Step 4: Wire the flags into `render_chart`**

In the `render_chart` flag loop add state + arms:
```rust
            "--civil" => civil = Some(parse_civil(iter.next())?),
            "--civil-scale" => civil_scale = match iter.next() {
                Some("utc") => TimeScale::Utc,
                Some("ut1") => TimeScale::Ut1,
                other => return Err(format!("--civil-scale must be utc|ut1, got {other:?}")),
            },
            "--civil-target" => civil_target = match iter.next() {
                Some("tt") => TimeScale::Tt,
                Some("tdb") => TimeScale::Tdb,
                other => return Err(format!("--civil-target must be tt|tdb, got {other:?}")),
            },
```
with state declared alongside the other `let mut`s:
```rust
    let mut civil: Option<pleiades_core::CivilDateTime> = None;
    let mut civil_scale = TimeScale::Utc;
    let mut civil_target = TimeScale::Tt;
```
After parsing, before building the request, reject conflicts and branch:
```rust
    if civil.is_some() && (jd.is_some() || time_scale_explicit) {
        return Err("--civil cannot be combined with --jd or manual time-scale flags".to_string());
    }
    if let Some(civil) = civil {
        let built = ChartRequest::from_civil(civil, civil_scale, civil_target, bodies.clone())
            .map_err(|e| e.summary_line())?;
        // Build the snapshot from `built.request` via the existing engine path,
        // then append `built.provenance.summary_line()` to the rendered output.
    }
```
Render the provenance line by appending `built.provenance.summary_line()` to the chart output string. Reuse the existing render path that turns a `ChartRequest` into output (found in Step 1) with `built.request`; do not duplicate it. `ChartRequest::from_civil` returns the `CivilChartRequest` wrapper from Task 9 (`.request` + `.provenance`).

- [ ] **Step 5: Document the flags in help**

In `crates/pleiades-cli/src/help.rs`, add `--civil`, `--civil-scale`, and `--civil-target` to the `chart` help block, noting they cannot combine with `--jd`/manual time-scale flags and that the output reports conversion provenance and quality.

- [ ] **Step 6: Run the CLI tests**

Run: `cargo test -p pleiades-cli chart`
Expected: `civil_flag_parses_and_reports_provenance` and `civil_flag_conflicts_with_jd` PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-cli/src/commands/chart.rs crates/pleiades-cli/src/help.rs crates/pleiades-cli/Cargo.toml
git commit -m "feat(cli): add --civil chart input with conversion provenance output"
```

---

## Task 11: End-to-end corpus cross-check

**Files:**
- Create: `crates/pleiades-time/tests/corpus_crosscheck.rs` (integration test) OR add to an existing `pleiades-core`/`pleiades-data` test module if that is where corpus access lives.
- Test: the new file.

**Interfaces:**
- Consumes: `pleiades_time::tt_from_utc_civil`, the packaged backend (`pleiades-data`/`pleiades-core` chart path), and the de440 corpus comparison already used by the workspace.

- [ ] **Step 1: Locate an existing corpus-vs-backend comparison test**

Run: `grep -rln "corpus" crates/pleiades-data/src crates/pleiades-core/src | head` and read one comparison test to learn how a known epoch is queried and compared to the corpus (the tolerance helpers and the body-longitude accessor).

- [ ] **Step 2: Write the cross-check test**

Create `crates/pleiades-time/tests/corpus_crosscheck.rs`. Because `pleiades-time` must not depend on the backend, place the actual comparison in whichever crate already has corpus access (per Step 1) if a cross-crate dependency would be introduced; otherwise keep a self-contained numeric check here. Minimal self-contained form:
```rust
//! Confirms a civil->TT conversion lands on the same JD a manual TT tag would,
//! so downstream backend lookups see no civil-conversion drift.
use pleiades_time::{tt_from_utc_civil, CivilDateTime};
use pleiades_types::{Instant, JulianDay, TimeScale, SECONDS_PER_DAY};

#[test]
fn civil_to_tt_matches_manual_tt_tag() {
    // 2000-01-01 12:00:00 UTC. TAI-UTC=32 -> TT offset 64.184s.
    let civil = CivilDateTime::new(2000, 1, 1, 12, 0, 0.0);
    let converted = tt_from_utc_civil(civil).unwrap();
    let utc_jd = civil.to_julian_day().unwrap().days();
    let manual = Instant::new(
        JulianDay::from_days(utc_jd + 64.184 / SECONDS_PER_DAY),
        TimeScale::Tt,
    );
    let drift_s =
        (converted.instant.julian_day.days() - manual.julian_day.days()).abs() * SECONDS_PER_DAY;
    assert!(drift_s < 1e-3, "civil->TT drift {drift_s}s exceeds 1 ms");
}
```
If Step 1 shows a practical way to feed `converted.instant` into the packaged backend and compare a body longitude to the corpus within the artifact's sub-arcsec budget, prefer that stronger assertion and place it in the corpus-owning crate's test module instead.

- [ ] **Step 3: Run the test**

Run: `cargo test -p pleiades-time --test corpus_crosscheck`
Expected: `civil_to_tt_matches_manual_tt_tag` PASSES.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-time/tests/corpus_crosscheck.rs
git commit -m "test(time): cross-check civil->TT against manual TT tagging"
```

---

## Task 12: Docs, README, and PLAN alignment

**Files:**
- Modify: `docs/time-observer-policy.md`
- Modify: `README.md`
- Modify: `PLAN.md` and `plan/stages/04-advanced-request-modes.md`
- Add: rustdoc examples on `pleiades_time::to_terrestrial` and the wrappers

**Interfaces:** none (documentation).

- [ ] **Step 1: Update the time-observer policy doc**

In `docs/time-observer-policy.md`, change the "Time scales and Delta T" and "Follow-up work" sections: built-in UTC/UT1 → TT/TDB conversion is now provided by `pleiades-time` with leap-second-exact UTC, observed/extrapolated Delta-T, TT↔TDB periodic term, the 1900–2100 support window, and the `exact`/`observed`/`predicted` quality marker. Keep the caller-supplied retagging contract documented as the still-supported lower-level path.

- [ ] **Step 2: Update README current limits**

In `README.md`, change the "direct backend requests accept TT/TDB; UTC/UT1 require caller-supplied conversion offsets" limit to note that `pleiades-time` now provides built-in civil UTC/UT1 → TT/TDB conversion (tiered quality, 1900–2100), while direct backends still consume TT/TDB. Add `pleiades-time` to the workspace-layout table.

- [ ] **Step 3: Update PLAN.md Phase 4 status**

In `PLAN.md` and `plan/stages/04-advanced-request-modes.md`, mark civil-time (UTC/UT1 + Delta-T) conversion as implemented; leave apparent-place, topocentric, and native-sidereal as the remaining Phase 4 work. Update the "Current priority" / status stamp accordingly.

- [ ] **Step 4: Add a rustdoc example**

On `pleiades_time::to_terrestrial` (in `convert.rs`), add a doc example:
```rust
/// ```
/// use pleiades_time::{to_terrestrial, CivilDateTime};
/// use pleiades_types::TimeScale;
///
/// let civil = CivilDateTime::new(2017, 1, 1, 0, 0, 0.0);
/// let out = to_terrestrial(civil, TimeScale::Utc, TimeScale::Tt).unwrap();
/// assert_eq!(out.instant.scale, TimeScale::Tt);
/// assert_eq!(out.provenance.tai_minus_utc, Some(37));
/// ```
```

- [ ] **Step 5: Verify docs build and examples run**

Run: `cargo test -p pleiades-time --doc`
Expected: the doc example PASSES.

- [ ] **Step 6: Commit**

```bash
git add docs/time-observer-policy.md README.md PLAN.md plan/stages/04-advanced-request-modes.md crates/pleiades-time/src/convert.rs
git commit -m "docs: document built-in civil-time conversion; mark Phase 4 progress"
```

---

## Task 13: Full workspace verification

**Files:** none (verification only).

- [ ] **Step 1: Format**

Run: `cargo fmt --all`
Then: `git diff --stat` — review any reformatting, commit if non-empty:
```bash
git add -A && git commit -m "style: cargo fmt" || true
```

- [ ] **Step 2: Clippy (workspace, deny warnings)**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: no warnings. Fix any in `pleiades-time`/touched files; commit.

- [ ] **Step 3: Full test suite**

Run: `cargo test --workspace`
Expected: all tests PASS, including the updated `pleiades-backend` policy tests and the new `pleiades-time` tests.

- [ ] **Step 4: Confirm the audit/validation surfaces still pass**

Run: `grep -rn "delta_t_policy_summary_for_report\|utc_convenience_policy_summary_for_report" crates/pleiades-validate/src | head`
If validation render/summary tests pin the old posture wording, update those expected strings to the new posture (same change as Task 8 Step 5) and re-run `cargo test -p pleiades-validate`.

- [ ] **Step 5: Final commit (if Step 4 required changes)**

```bash
git add -A
git commit -m "test(validate): align summaries with built-in civil-time posture"
```

---

## Self-Review

**Spec coverage** — every design section maps to a task:
- Full civil bridge (leap UTC / Delta-T UT1 / TT↔TDB) → Tasks 4, 5, 6, 7.
- Pinned ΔT table + extrapolation → Task 5.
- Calendar + scale layers → Tasks 3 (+ 7).
- New `pleiades-time` crate → Task 1.
- Tiered Exact/Observed/Predicted + typed provenance → Task 7.
- Fail-closed data gate (StaleTimeData, checksum) → Tasks 4, 5 (self-check on load); surfaced in errors.
- Additive `pleiades-core` integration → Task 9.
- CLI `--civil` + provenance render → Task 10.
- Error type with all variants → Task 2 (consumed throughout).
- Policy summary + reversing the non-goal posture → Task 8.
- Validation fixtures (anchors, ΔT spot values, TDB bound, round-trip, fail-closed) → Tasks 3, 5, 6, 7.
- End-to-end corpus cross-check → Task 11.
- Docs/README/PLAN/policy alignment → Tasks 8, 12, 13.
- Exit criteria (fixtures + rustdoc + CLI + metadata surfacing + release/policy wording) → Tasks 7, 10, 12, 13.

**Type consistency** — `CivilTimeError` variants defined in Task 2 are used unchanged in Tasks 3/4/5/7. `DeltaTQuality { Observed, Predicted }` (Task 5) is distinct from `ConversionQuality { Exact, Observed, Predicted }` (Task 7); the orchestrator maps between them explicitly — the design's "quality marker" is `ConversionQuality`, and the doc note in Task 5 records that `deltat` never emits `Exact`. `CivilInstant`, `ConversionProvenance`, `to_terrestrial`, and the four wrappers keep identical names across Tasks 7, 9, 10, 11, 12. Instant access uses the verified **public fields** `instant.scale` and `instant.julian_day.days()` (no method calls) everywhere. Task 9 returns the `CivilChartRequest { request, provenance }` wrapper (not a mutated `ChartRequest`), and Task 10 consumes `.request` / `.provenance` accordingly — verified against `chart/request.rs` where `ChartRequest::new(instant)` takes only the instant and `bodies` is a public field.

**Placeholder scan** — no `TBD`/`TODO`; each code step shows complete code. Two checksum constants are intentionally `0` then pinned via a printed-value step (Tasks 4/5 Step 4) — that is a deliberate, instructed procedure, not a placeholder. Integration steps that must match an existing signature (core `ChartRequest::new`, CLI render path) include an explicit grep/read step and instruction to adapt, because those signatures live in code the implementer must inspect.
