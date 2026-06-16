# Broad Public-Data Reader Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a format-detecting reader to `pleiades-jpl` that ingests arbitrary external JPL-style products (Horizons vector-table text, Horizons API JSON, generic CSV) into the existing `SnapshotCorpus`, behind one fail-closed normalizer, with optional quarantined live fetch.

**Architecture:** A new `pleiades-jpl/src/ingest/` module tree. Format front-ends tokenize bytes into a neutral `RawCorpus` intermediate representation; one `normalize()` function does all fail-closed semantic reconciliation (frame/time/units/center/body) into `SnapshotCorpus` + `IngestProvenance`. A caller-supplied `ExpectedProfile` fills genuine silences and is recorded as `Asserted` provenance. Live Horizons fetch lives behind a default-off `horizons-fetch` Cargo feature and a `HorizonsSource` trait seam; the offline path never sees the network.

**Tech Stack:** Rust (workspace edition), `#![forbid(unsafe_code)]`, existing `pleiades-types` / `pleiades-backend` vocabulary, `ureq` (optional, fetch-only). Design spec: `docs/superpowers/specs/2026-06-16-public-data-reader-design.md`.

---

## Conventions for every task

- Fast per-crate loop: `cargo test -p pleiades-jpl` (or `-p pleiades-validate` for the CLI task).
- Before each commit, the changed crate must pass:
  - `cargo fmt --all --check`
  - `cargo clippy -p pleiades-jpl --all-targets --all-features -- -D warnings`
- The whole module is `#![forbid(unsafe_code)]`-compatible (no `unsafe`).
- Real workspace types (already exist, do not redefine):
  - `pleiades_types::{CoordinateFrame /* Ecliptic|Equatorial */, TimeScale /* Utc|Ut1|Tt|Tdb */, Instant, JulianDay}`
  - `pleiades_backend::CelestialBody` (re-export of `pleiades_types::CelestialBody`; variants include `Sun, Moon, Mercury…Pluto, Ceres, Pallas, Juno, Vesta, Custom(..)`).
  - `pleiades_jpl::SnapshotCorpus { manifest: SnapshotManifest, entries: Vec<SnapshotEntry> }`
  - `SnapshotManifest { title, source, coverage, redistribution, columns: Vec<String> }` (derives `Default`).
  - `SnapshotEntry { body: pleiades_backend::CelestialBody, epoch: Instant, x_km: f64, y_km: f64, z_km: f64 }`.

## File structure (created by this plan)

| File | Responsibility |
| --- | --- |
| `crates/pleiades-jpl/src/ingest/mod.rs` | Module root: public API (`read_public_corpus*`, `detect_format`), `ExpectedProfile`, `Units`, `Center`, `PublicCorpus`, `IngestProvenance`, re-exports. |
| `crates/pleiades-jpl/src/ingest/ir.rs` | Format-neutral IR: `RawCorpus`, `RawManifest`, `RawEphemerisRecord`. |
| `crates/pleiades-jpl/src/ingest/error.rs` | `IngestError`, `Attribute`. |
| `crates/pleiades-jpl/src/ingest/normalize.rs` | The single fail-closed normalizer + attribute resolution + body mapping. |
| `crates/pleiades-jpl/src/ingest/detect.rs` | `InputFormat`, `detect_format`. |
| `crates/pleiades-jpl/src/ingest/format/mod.rs` | Front-end dispatch (`parse_to_ir`). |
| `crates/pleiades-jpl/src/ingest/format/vector_table.rs` | Horizons `$$SOE`/`$$EOE` CSV-vector text → IR. |
| `crates/pleiades-jpl/src/ingest/format/horizons_json.rs` | Horizons API JSON envelope → IR (delegates rows to `vector_table`). |
| `crates/pleiades-jpl/src/ingest/format/generic_csv.rs` | Tolerant CSV (column aliasing) → IR. |
| `crates/pleiades-jpl/src/ingest/fetch.rs` | `#[cfg(feature = "horizons-fetch")]` `HorizonsSource` trait + `HttpHorizonsSource` + `fetch_public_corpus`. |
| `crates/pleiades-jpl/tests/fixtures/ingest/*` | Checked-in real-shaped sample inputs. |
| `crates/pleiades-jpl/tests/ingest_end_to_end.rs` | Integration tests (offline + de440 round-trip anchor). |

---

### Task 1: Scaffold the ingest module and the IR

**Files:**
- Create: `crates/pleiades-jpl/src/ingest/mod.rs`
- Create: `crates/pleiades-jpl/src/ingest/ir.rs`
- Modify: `crates/pleiades-jpl/src/lib.rs` (add `pub mod ingest;` and re-export)

- [ ] **Step 1: Write the failing test**

In `crates/pleiades-jpl/src/ingest/ir.rs`, add at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_corpus_holds_declared_and_records() {
        let raw = RawCorpus {
            declared: RawManifest {
                source_label: Some("JPL Horizons".to_string()),
                center: Some("500@0".to_string()),
                frame: Some("Ecliptic of J2000.0".to_string()),
                time_scale: Some("TDB".to_string()),
                units: Some("KM-S".to_string()),
                columns: vec!["jd".to_string(), "x".to_string()],
            },
            records: vec![RawEphemerisRecord {
                body_label: "Mars".to_string(),
                epoch_jd: 2_451_545.0,
                pos: [1.0, 2.0, 3.0],
                vel: None,
            }],
        };
        assert_eq!(raw.records.len(), 1);
        assert_eq!(raw.declared.center.as_deref(), Some("500@0"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl ingest::ir`
Expected: FAIL — `cannot find type RawCorpus` (module not yet wired / types absent).

- [ ] **Step 3: Write minimal implementation**

Put this above the test module in `crates/pleiades-jpl/src/ingest/ir.rs`:

```rust
//! Format-neutral intermediate representation for external ingestion.
//!
//! Front-ends tokenize bytes into these types verbatim — every interpreted
//! attribute is an `Option`/`String` here. No defaulting, conversion, or
//! semantic mapping happens at this layer; that is the normalizer's job.

/// A parsed-but-uninterpreted external corpus.
#[derive(Clone, Debug, PartialEq)]
pub struct RawCorpus {
    /// Header metadata lifted verbatim from the source.
    pub declared: RawManifest,
    /// Row data lifted verbatim from the source.
    pub records: Vec<RawEphemerisRecord>,
}

/// Header strings exactly as the source declared them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RawManifest {
    /// Free-form source label (e.g. "JPL Horizons API v1.2").
    pub source_label: Option<String>,
    /// Declared center/origin (e.g. "500@0").
    pub center: Option<String>,
    /// Declared reference frame string.
    pub frame: Option<String>,
    /// Declared time-scale string.
    pub time_scale: Option<String>,
    /// Declared output-units string.
    pub units: Option<String>,
    /// Raw column order as seen.
    pub columns: Vec<String>,
}

/// One uninterpreted ephemeris row.
#[derive(Clone, Debug, PartialEq)]
pub struct RawEphemerisRecord {
    /// Raw target name/id as seen.
    pub body_label: String,
    /// Numeric Julian day as seen (the scale lives in `RawManifest::time_scale`).
    pub epoch_jd: f64,
    /// Raw x,y,z in source units (no conversion yet).
    pub pos: [f64; 3],
    /// Velocity captured if present; dropped during normalization.
    pub vel: Option<[f64; 3]>,
}
```

Create `crates/pleiades-jpl/src/ingest/mod.rs`:

```rust
//! Broad public-data reader for arbitrary external JPL-style inputs.
//!
//! See `docs/superpowers/specs/2026-06-16-public-data-reader-design.md`.

pub mod ir;

pub use ir::{RawCorpus, RawEphemerisRecord, RawManifest};
```

In `crates/pleiades-jpl/src/lib.rs`, add alongside the other `mod`/`pub use` lines (e.g. just after `pub mod spk;`):

```rust
pub mod ingest;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl ingest::ir`
Expected: PASS (1 test).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/ingest/ crates/pleiades-jpl/src/lib.rs
git commit -m "feat(jpl): scaffold ingest module with format-neutral IR"
```

---

### Task 2: Error taxonomy

**Files:**
- Create: `crates/pleiades-jpl/src/ingest/error.rs`
- Modify: `crates/pleiades-jpl/src/ingest/mod.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-jpl/src/ingest/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contradiction_names_both_sides() {
        let err = IngestError::Contradiction {
            attribute: Attribute::TimeScale,
            declared: "UTC".to_string(),
            expected: "TDB".to_string(),
        };
        let text = err.to_string();
        assert!(text.contains("time scale"), "got: {text}");
        assert!(text.contains("UTC") && text.contains("TDB"), "got: {text}");
    }

    #[test]
    fn unrecognized_lists_what_it_looked_for() {
        let err = IngestError::UnrecognizedFormat {
            looked_for: vec!["horizons-json", "vector-table", "generic-csv"],
        };
        assert!(err.to_string().contains("vector-table"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl ingest::error`
Expected: FAIL — `cannot find type IngestError`.

- [ ] **Step 3: Write minimal implementation**

Put above the test module in `crates/pleiades-jpl/src/ingest/error.rs`:

```rust
//! Structured, fail-closed ingestion errors.

use core::fmt;

use super::detect::InputFormat;

/// The semantic attribute an ingestion check is about.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Attribute {
    /// Coordinate frame.
    Frame,
    /// Time scale.
    TimeScale,
    /// Output units.
    Units,
    /// Center / origin.
    Center,
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Frame => "frame",
            Self::TimeScale => "time scale",
            Self::Units => "units",
            Self::Center => "center",
        };
        f.write_str(label)
    }
}

/// Everything that can go wrong while ingesting an external product.
#[derive(Clone, Debug, PartialEq)]
pub enum IngestError {
    /// No format matched during detection.
    UnrecognizedFormat {
        /// The format names the detector tried.
        looked_for: Vec<&'static str>,
    },
    /// A front-end could not tokenize a line.
    Malformed {
        /// Which format front-end was parsing.
        format: InputFormat,
        /// 1-based line number within the source.
        line: usize,
        /// Human-readable detail.
        detail: String,
    },
    /// A required structural marker was absent (e.g. `$$EOE`).
    MissingMarker {
        /// Which format front-end was parsing.
        format: InputFormat,
        /// The marker that was expected.
        marker: &'static str,
    },
    /// A generic-CSV column header could not be mapped to a known field.
    ColumnUnresolved {
        /// The raw column header.
        column: String,
        /// Which format front-end was parsing.
        format: InputFormat,
    },
    /// The source declared a value we do not support.
    Unsupported {
        /// The attribute.
        attribute: Attribute,
        /// The unsupported value.
        value: String,
    },
    /// Source and `ExpectedProfile` disagree on an attribute.
    Contradiction {
        /// The attribute.
        attribute: Attribute,
        /// What the source declared.
        declared: String,
        /// What the caller asserted.
        expected: String,
    },
    /// An attribute was silent in the source and absent from `ExpectedProfile`.
    Undetermined {
        /// The attribute.
        attribute: Attribute,
    },
    /// A row's body label did not map to a known body.
    UnknownBody {
        /// The raw body label.
        label: String,
    },
    /// A row carried a non-finite or otherwise invalid numeric value.
    MalformedRow {
        /// The epoch the row claimed.
        epoch_jd: f64,
        /// Human-readable detail.
        detail: String,
    },
    /// A live fetch failed (only constructed under `horizons-fetch`).
    Fetch {
        /// Human-readable detail.
        detail: String,
    },
}

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnrecognizedFormat { looked_for } => {
                write!(f, "unrecognized input format; looked for: {}", looked_for.join(", "))
            }
            Self::Malformed { format, line, detail } => {
                write!(f, "malformed {format} input at line {line}: {detail}")
            }
            Self::MissingMarker { format, marker } => {
                write!(f, "missing {marker} marker in {format} input")
            }
            Self::ColumnUnresolved { column, format } => {
                write!(f, "unresolved column {column:?} in {format} input")
            }
            Self::Unsupported { attribute, value } => {
                write!(f, "unsupported {attribute}: {value:?}")
            }
            Self::Contradiction { attribute, declared, expected } => write!(
                f,
                "contradiction on {attribute}: source declared {declared:?} but caller asserted {expected:?}"
            ),
            Self::Undetermined { attribute } => {
                write!(f, "undetermined {attribute}: not in source and not asserted")
            }
            Self::UnknownBody { label } => write!(f, "unknown body label: {label:?}"),
            Self::MalformedRow { epoch_jd, detail } => {
                write!(f, "malformed row at JD {epoch_jd}: {detail}")
            }
            Self::Fetch { detail } => write!(f, "fetch failed: {detail}"),
        }
    }
}

impl std::error::Error for IngestError {}
```

Add to `crates/pleiades-jpl/src/ingest/mod.rs`:

```rust
pub mod detect;
pub mod error;

pub use error::{Attribute, IngestError};
```

(`detect` is declared now because `error.rs` references `InputFormat`; Task 4 fills it in. To keep this task compiling on its own, add the minimal `InputFormat` stub now — Task 4's test will extend it.)

Create `crates/pleiades-jpl/src/ingest/detect.rs` with just the enum for now:

```rust
//! Format detection for external ingestion inputs.

use core::fmt;

/// The external input shapes the reader understands.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputFormat {
    /// Horizons `$$SOE`/`$$EOE` CSV-vector text.
    HorizonsVectorTable,
    /// Horizons API JSON envelope.
    HorizonsApiJson,
    /// Generic JPL-style CSV.
    GenericCsv,
}

impl fmt::Display for InputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::HorizonsVectorTable => "vector-table",
            Self::HorizonsApiJson => "horizons-json",
            Self::GenericCsv => "generic-csv",
        };
        f.write_str(label)
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl ingest::error`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/ingest/
git commit -m "feat(jpl): add ingest error taxonomy and InputFormat enum"
```

---

### Task 3: Public surface types — ExpectedProfile, Units, Center, provenance, PublicCorpus

**Files:**
- Create: `crates/pleiades-jpl/src/ingest/profile.rs`
- Modify: `crates/pleiades-jpl/src/ingest/mod.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-jpl/src/ingest/profile.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{CoordinateFrame, TimeScale};

    #[test]
    fn expected_profile_defaults_to_all_silent() {
        let p = ExpectedProfile::default();
        assert!(p.frame.is_none());
        assert!(p.time_scale.is_none());
        assert!(p.units.is_none());
        assert!(p.center.is_none());
    }

    #[test]
    fn provenance_records_read_vs_asserted() {
        let prov = IngestProvenance {
            frame: Provenance::Read,
            time_scale: Provenance::Asserted,
            units: Provenance::Read,
            center: Provenance::Read,
            source_label: Some("JPL Horizons".to_string()),
            request_url: None,
        };
        assert_eq!(prov.time_scale, Provenance::Asserted);
    }

    #[test]
    fn units_and_center_are_distinct_values() {
        assert_ne!(Units::Km, Units::Au);
        let _ = ExpectedProfile {
            frame: Some(CoordinateFrame::Ecliptic),
            time_scale: Some(TimeScale::Tdb),
            units: Some(Units::Km),
            center: Some(Center::SolarSystemBarycenter),
        };
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl ingest::profile`
Expected: FAIL — `cannot find type ExpectedProfile`.

- [ ] **Step 3: Write minimal implementation**

Put above the test module in `crates/pleiades-jpl/src/ingest/profile.rs`:

```rust
//! Caller assertions and provenance for external ingestion.

use pleiades_types::{CoordinateFrame, TimeScale};

/// Output units the reader can normalize from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Units {
    /// Kilometers (position) — the corpus storage unit.
    Km,
    /// Astronomical units — converted to km on normalize.
    Au,
}

/// Supported coordinate centers/origins.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Center {
    /// Solar System Barycenter (Horizons `500@0` / `@0`).
    SolarSystemBarycenter,
}

/// Caller-asserted attributes used to fill genuine source silences.
///
/// Every field is optional: assert only what the source omits. A field that is
/// `Some` and contradicts the source is a hard error; a field that is `Some`
/// and fills a silent source is recorded as `Provenance::Asserted`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ExpectedProfile {
    /// Asserted coordinate frame.
    pub frame: Option<CoordinateFrame>,
    /// Asserted time scale.
    pub time_scale: Option<TimeScale>,
    /// Asserted output units.
    pub units: Option<Units>,
    /// Asserted center/origin.
    pub center: Option<Center>,
}

/// Whether a normalized attribute came from the source or a caller assertion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Provenance {
    /// Read from the source headers.
    Read,
    /// Filled from `ExpectedProfile`.
    Asserted,
}

/// Per-attribute provenance of a normalized corpus.
#[derive(Clone, Debug, PartialEq)]
pub struct IngestProvenance {
    /// Frame provenance.
    pub frame: Provenance,
    /// Time-scale provenance.
    pub time_scale: Provenance,
    /// Units provenance.
    pub units: Provenance,
    /// Center provenance.
    pub center: Provenance,
    /// Free-form source label, if any.
    pub source_label: Option<String>,
    /// The fetch URL, when the bytes came from a live fetch.
    pub request_url: Option<String>,
}
```

Add to `crates/pleiades-jpl/src/ingest/mod.rs`:

```rust
pub mod profile;

pub use profile::{Center, ExpectedProfile, IngestProvenance, Provenance, Units};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl ingest::profile`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/ingest/
git commit -m "feat(jpl): add ExpectedProfile, Units/Center, and ingest provenance"
```

---

### Task 4: Format detector

**Files:**
- Modify: `crates/pleiades-jpl/src/ingest/detect.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-jpl/src/ingest/detect.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_horizons_json_by_signature_keys() {
        let bytes = br#"{ "signature": { "version": "1.2" }, "result": "..." }"#;
        assert_eq!(detect_format(bytes).unwrap(), InputFormat::HorizonsApiJson);
    }

    #[test]
    fn detects_vector_table_by_soe_markers() {
        let bytes = b"Center body : SSB\n$$SOE\n2451545.0, A.D. ...\n$$EOE\n";
        assert_eq!(detect_format(bytes).unwrap(), InputFormat::HorizonsVectorTable);
    }

    #[test]
    fn detects_generic_csv_by_header_comment() {
        let bytes = b"# Columns: jd, body, x, y, z\n2451545.0, Mars, 1, 2, 3\n";
        assert_eq!(detect_format(bytes).unwrap(), InputFormat::GenericCsv);
    }

    #[test]
    fn rejects_unrecognized_input() {
        let err = detect_format(b"\x00\x01 not data").unwrap_err();
        assert!(matches!(err, crate::ingest::IngestError::UnrecognizedFormat { .. }));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl ingest::detect`
Expected: FAIL — `cannot find function detect_format`.

- [ ] **Step 3: Write minimal implementation**

Add to `crates/pleiades-jpl/src/ingest/detect.rs` (above the test module, below the existing `InputFormat`):

```rust
use super::error::IngestError;

/// Sniffs the input bytes to choose a front-end. Cheap and deterministic.
///
/// Order matters — first match wins.
pub fn detect_format(bytes: &[u8]) -> Result<InputFormat, IngestError> {
    let text = String::from_utf8_lossy(bytes);
    let trimmed = text.trim_start();

    // 1. Horizons API JSON: starts with `{` and carries the envelope keys.
    if trimmed.starts_with('{')
        && trimmed.contains("\"result\"")
        && trimmed.contains("\"signature\"")
    {
        return Ok(InputFormat::HorizonsApiJson);
    }

    // 2. Horizons vector table: bracketed by ephemeris markers.
    if text.contains("$$SOE") && text.contains("$$EOE") {
        return Ok(InputFormat::HorizonsVectorTable);
    }

    // 3. Generic CSV: a `#`-comment header block or a plain delimited row table.
    if text.lines().any(|l| l.trim_start().starts_with('#'))
        || text.lines().any(|l| l.contains(','))
    {
        return Ok(InputFormat::GenericCsv);
    }

    Err(IngestError::UnrecognizedFormat {
        looked_for: vec!["horizons-json", "vector-table", "generic-csv"],
    })
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl ingest::detect`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/ingest/detect.rs
git commit -m "feat(jpl): add fail-closed format detector"
```

---

### Task 5: Normalizer — attribute resolution (frame/time/units/center)

**Files:**
- Create: `crates/pleiades-jpl/src/ingest/normalize.rs`
- Modify: `crates/pleiades-jpl/src/ingest/mod.rs`

This task implements the ordered fail-closed resolution of the four header attributes (no row mapping yet). Row mapping is Task 6.

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-jpl/src/ingest/normalize.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::{Attribute, Center, ExpectedProfile, IngestError, Provenance, Units};
    use pleiades_types::{CoordinateFrame, TimeScale};

    fn declared(frame: &str, ts: &str, units: &str, center: &str) -> RawManifest {
        RawManifest {
            source_label: Some("JPL Horizons".to_string()),
            center: Some(center.to_string()),
            frame: Some(frame.to_string()),
            time_scale: Some(ts.to_string()),
            units: Some(units.to_string()),
            columns: vec![],
        }
    }

    #[test]
    fn resolves_all_declared_as_read() {
        let m = declared("Ecliptic of J2000.0", "TDB", "KM-S", "500@0");
        let r = resolve_attributes(&m, &ExpectedProfile::default()).unwrap();
        assert_eq!(r.frame, CoordinateFrame::Ecliptic);
        assert_eq!(r.time_scale, TimeScale::Tdb);
        assert_eq!(r.units, Units::Km);
        assert_eq!(r.center, Center::SolarSystemBarycenter);
        assert_eq!(r.provenance.frame, Provenance::Read);
        assert_eq!(r.provenance.time_scale, Provenance::Read);
    }

    #[test]
    fn fills_silent_time_scale_from_expected_as_asserted() {
        let mut m = declared("Ecliptic", "", "KM", "500@0");
        m.time_scale = None;
        let expected = ExpectedProfile { time_scale: Some(TimeScale::Tt), ..Default::default() };
        let r = resolve_attributes(&m, &expected).unwrap();
        assert_eq!(r.time_scale, TimeScale::Tt);
        assert_eq!(r.provenance.time_scale, Provenance::Asserted);
    }

    #[test]
    fn rejects_unsupported_time_scale() {
        let m = declared("Ecliptic", "UTC", "KM", "500@0");
        let err = resolve_attributes(&m, &ExpectedProfile::default()).unwrap_err();
        assert!(matches!(err, IngestError::Unsupported { attribute: Attribute::TimeScale, .. }));
    }

    #[test]
    fn rejects_contradiction_between_source_and_expected() {
        let m = declared("Ecliptic", "TDB", "KM", "500@0");
        let expected = ExpectedProfile { time_scale: Some(TimeScale::Tt), ..Default::default() };
        let err = resolve_attributes(&m, &expected).unwrap_err();
        assert!(matches!(err, IngestError::Contradiction { attribute: Attribute::TimeScale, .. }));
    }

    #[test]
    fn rejects_undetermined_when_silent_and_unasserted() {
        let mut m = declared("Ecliptic", "TDB", "KM", "500@0");
        m.frame = None;
        let err = resolve_attributes(&m, &ExpectedProfile::default()).unwrap_err();
        assert!(matches!(err, IngestError::Undetermined { attribute: Attribute::Frame }));
    }

    #[test]
    fn converts_au_units() {
        let m = declared("Ecliptic", "TDB", "AU-D", "500@0");
        let r = resolve_attributes(&m, &ExpectedProfile::default()).unwrap();
        assert_eq!(r.units, Units::Au);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl ingest::normalize`
Expected: FAIL — `cannot find function resolve_attributes`.

- [ ] **Step 3: Write minimal implementation**

Put above the test module in `crates/pleiades-jpl/src/ingest/normalize.rs`:

```rust
//! The single fail-closed normalizer: IR -> SnapshotCorpus + provenance.

use pleiades_types::{CoordinateFrame, TimeScale};

use super::error::{Attribute, IngestError};
use super::ir::RawManifest;
use super::profile::{Center, ExpectedProfile, IngestProvenance, Provenance, Units};

/// The four resolved header attributes plus their provenance.
pub(crate) struct ResolvedAttributes {
    pub frame: CoordinateFrame,
    pub time_scale: TimeScale,
    pub units: Units,
    pub center: Center,
    pub provenance: IngestProvenance,
}

/// Resolves one attribute from a declared source string and a caller assertion.
///
/// `parse` maps a non-empty declared string to `Some(value)` for supported
/// values, `None` for unsupported ones. Rules:
/// - declared supported + asserted equal      -> Read
/// - declared supported + asserted different   -> Contradiction
/// - declared present but unsupported           -> Unsupported
/// - declared silent + asserted                 -> Asserted
/// - declared silent + unasserted               -> Undetermined
fn resolve<T: Copy + PartialEq>(
    attribute: Attribute,
    declared: Option<&str>,
    asserted: Option<T>,
    parse: impl Fn(&str) -> Option<T>,
    render: impl Fn(&T) -> String,
) -> Result<(T, Provenance), IngestError> {
    match declared.map(str::trim).filter(|s| !s.is_empty()) {
        Some(raw) => match parse(raw) {
            Some(value) => {
                if let Some(want) = asserted {
                    if want != value {
                        return Err(IngestError::Contradiction {
                            attribute,
                            declared: render(&value),
                            expected: render(&want),
                        });
                    }
                }
                Ok((value, Provenance::Read))
            }
            None => Err(IngestError::Unsupported { attribute, value: raw.to_string() }),
        },
        None => match asserted {
            Some(value) => Ok((value, Provenance::Asserted)),
            None => Err(IngestError::Undetermined { attribute }),
        },
    }
}

fn parse_frame(raw: &str) -> Option<CoordinateFrame> {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("apparent") || lower.contains("topocentric") {
        return None; // unsupported observed-place markers
    }
    if lower.contains("ecliptic") {
        Some(CoordinateFrame::Ecliptic)
    } else if lower.contains("icrf") || lower.contains("equatorial") || lower.contains("frame: j2000")
    {
        Some(CoordinateFrame::Equatorial)
    } else {
        None
    }
}

fn parse_time_scale(raw: &str) -> Option<TimeScale> {
    match raw.to_ascii_uppercase().as_str() {
        "TDB" | "CT" | "BARYCENTRIC DYNAMICAL TIME" => Some(TimeScale::Tdb),
        "TT" | "TERRESTRIAL TIME" | "TDT" => Some(TimeScale::Tt),
        // UTC/UT1 are intentionally unsupported at this boundary.
        _ => None,
    }
}

fn parse_units(raw: &str) -> Option<Units> {
    let upper = raw.to_ascii_uppercase();
    if upper.starts_with("KM") {
        Some(Units::Km)
    } else if upper.starts_with("AU") {
        Some(Units::Au)
    } else {
        None
    }
}

fn parse_center(raw: &str) -> Option<Center> {
    let norm: String = raw.to_ascii_lowercase().split_whitespace().collect();
    if norm.contains("500@0") || norm == "@0" || norm.contains("barycenter") || norm.contains("ssb") {
        Some(Center::SolarSystemBarycenter)
    } else {
        None
    }
}

/// Resolves all four header attributes, failing closed per the design rules.
pub(crate) fn resolve_attributes(
    declared: &RawManifest,
    expected: &ExpectedProfile,
) -> Result<ResolvedAttributes, IngestError> {
    let (frame, frame_prov) = resolve(
        Attribute::Frame,
        declared.frame.as_deref(),
        expected.frame,
        parse_frame,
        |v| v.to_string(),
    )?;
    let (time_scale, ts_prov) = resolve(
        Attribute::TimeScale,
        declared.time_scale.as_deref(),
        expected.time_scale,
        parse_time_scale,
        |v| v.to_string(),
    )?;
    let (units, units_prov) = resolve(
        Attribute::Units,
        declared.units.as_deref(),
        expected.units,
        parse_units,
        |v| format!("{v:?}"),
    )?;
    let (center, center_prov) = resolve(
        Attribute::Center,
        declared.center.as_deref(),
        expected.center,
        parse_center,
        |v| format!("{v:?}"),
    )?;

    Ok(ResolvedAttributes {
        frame,
        time_scale,
        units,
        center,
        provenance: IngestProvenance {
            frame: frame_prov,
            time_scale: ts_prov,
            units: units_prov,
            center: center_prov,
            source_label: declared.source_label.clone(),
            request_url: None,
        },
    })
}
```

Add to `crates/pleiades-jpl/src/ingest/mod.rs`:

```rust
pub(crate) mod normalize;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl ingest::normalize`
Expected: PASS (6 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/ingest/
git commit -m "feat(jpl): add fail-closed attribute resolution to ingest normalizer"
```

---

### Task 6: Normalizer — body mapping, unit conversion, corpus assembly

**Files:**
- Modify: `crates/pleiades-jpl/src/ingest/normalize.rs`

- [ ] **Step 1: Write the failing test**

Append inside the existing `tests` module in `crates/pleiades-jpl/src/ingest/normalize.rs`:

```rust
    use crate::ingest::ir::{RawCorpus, RawEphemerisRecord};

    fn one_record(body: &str, jd: f64, pos: [f64; 3]) -> RawCorpus {
        RawCorpus {
            declared: declared("Ecliptic", "TDB", "KM-S", "500@0"),
            records: vec![RawEphemerisRecord { body_label: body.to_string(), epoch_jd: jd, pos, vel: None }],
        }
    }

    #[test]
    fn normalizes_km_record_into_entry() {
        let raw = one_record("Mars", 2_451_545.0, [1.0, 2.0, 3.0]);
        let (corpus, prov) = normalize(raw, &ExpectedProfile::default()).unwrap();
        assert_eq!(corpus.entries.len(), 1);
        let e = &corpus.entries[0];
        assert_eq!(e.body, pleiades_backend::CelestialBody::Mars);
        assert_eq!(e.epoch.julian_day.days(), 2_451_545.0);
        assert_eq!(e.epoch.scale, TimeScale::Tdb);
        assert_eq!((e.x_km, e.y_km, e.z_km), (1.0, 2.0, 3.0));
        assert_eq!(prov.frame, Provenance::Read);
    }

    #[test]
    fn converts_au_positions_to_km() {
        let mut raw = one_record("Mercury", 2_451_545.0, [1.0, 0.0, 0.0]);
        raw.declared.units = Some("AU-D".to_string());
        let (corpus, _) = normalize(raw, &ExpectedProfile::default()).unwrap();
        assert!((corpus.entries[0].x_km - 149_597_870.7).abs() < 1e-3);
    }

    #[test]
    fn maps_horizons_numeric_id_body() {
        let raw = one_record("Mars (499)", 2_451_545.0, [1.0, 2.0, 3.0]);
        let (corpus, _) = normalize(raw, &ExpectedProfile::default()).unwrap();
        assert_eq!(corpus.entries[0].body, pleiades_backend::CelestialBody::Mars);
    }

    #[test]
    fn rejects_unknown_body() {
        let raw = one_record("Wormwood", 2_451_545.0, [1.0, 2.0, 3.0]);
        let err = normalize(raw, &ExpectedProfile::default()).unwrap_err();
        assert!(matches!(err, IngestError::UnknownBody { .. }));
    }

    #[test]
    fn rejects_non_finite_row() {
        let raw = one_record("Mars", 2_451_545.0, [f64::NAN, 0.0, 0.0]);
        let err = normalize(raw, &ExpectedProfile::default()).unwrap_err();
        assert!(matches!(err, IngestError::MalformedRow { .. }));
    }

    #[test]
    fn carries_source_label_into_manifest() {
        let raw = one_record("Mars", 2_451_545.0, [1.0, 2.0, 3.0]);
        let (corpus, _) = normalize(raw, &ExpectedProfile::default()).unwrap();
        assert_eq!(corpus.manifest.source.as_deref(), Some("JPL Horizons"));
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl ingest::normalize`
Expected: FAIL — `cannot find function normalize`.

- [ ] **Step 3: Write minimal implementation**

Add to `crates/pleiades-jpl/src/ingest/normalize.rs` (above the test module). Add the needed imports to the existing `use` block at the top of the file: extend it to

```rust
use pleiades_backend::CelestialBody;
use pleiades_types::{CoordinateFrame, Instant, JulianDay, TimeScale};

use crate::backend::{SnapshotCorpus, SnapshotEntry, SnapshotManifest};
use super::ir::{RawCorpus, RawManifest};
```

(Keep the existing `error`/`profile` imports.)

Then add:

```rust
/// Kilometers per astronomical unit (matches the crate-wide constant).
const AU_IN_KM: f64 = 149_597_870.7;

/// Maps a raw Horizons/CSV body label to a built-in body.
///
/// Accepts plain names ("Mars"), Horizons "Name (id)" forms ("Mars (499)"),
/// and is case-insensitive. Unknown labels fail closed.
fn body_from_label(label: &str) -> Option<CelestialBody> {
    // Strip a trailing " (id)" suffix if present.
    let name = label.split('(').next().unwrap_or(label).trim();
    let lower = name.to_ascii_lowercase();
    let body = match lower.as_str() {
        "sun" => CelestialBody::Sun,
        "moon" => CelestialBody::Moon,
        "mercury" => CelestialBody::Mercury,
        "venus" => CelestialBody::Venus,
        // Earth has no built-in variant (the chart domain is geocentric) and is
        // not a release-claimed corpus body; an Earth label fails closed here.
        "mars" => CelestialBody::Mars,
        "jupiter" => CelestialBody::Jupiter,
        "saturn" => CelestialBody::Saturn,
        "uranus" => CelestialBody::Uranus,
        "neptune" => CelestialBody::Neptune,
        "pluto" => CelestialBody::Pluto,
        "ceres" => CelestialBody::Ceres,
        "pallas" => CelestialBody::Pallas,
        "juno" => CelestialBody::Juno,
        "vesta" => CelestialBody::Vesta,
        _ => return None,
    };
    Some(body)
}
```

```rust
/// Normalizes a raw external corpus into the typed `SnapshotCorpus`.
pub fn normalize(
    raw: RawCorpus,
    expected: &ExpectedProfile,
) -> Result<(SnapshotCorpus, IngestProvenance), IngestError> {
    let resolved = resolve_attributes(&raw.declared, expected)?;
    let km_per_unit = match resolved.units {
        Units::Km => 1.0,
        Units::Au => AU_IN_KM,
    };

    let mut entries = Vec::with_capacity(raw.records.len());
    for record in &raw.records {
        let body = body_from_label(&record.body_label)
            .ok_or_else(|| IngestError::UnknownBody { label: record.body_label.clone() })?;

        if !record.epoch_jd.is_finite() || record.pos.iter().any(|c| !c.is_finite()) {
            return Err(IngestError::MalformedRow {
                epoch_jd: record.epoch_jd,
                detail: "non-finite epoch or position component".to_string(),
            });
        }

        entries.push(SnapshotEntry {
            body,
            epoch: Instant::new(JulianDay::from_days(record.epoch_jd), resolved.time_scale),
            x_km: record.pos[0] * km_per_unit,
            y_km: record.pos[1] * km_per_unit,
            z_km: record.pos[2] * km_per_unit,
        });
    }

    let manifest = SnapshotManifest {
        title: raw.declared.source_label.clone(),
        source: raw.declared.source_label.clone(),
        coverage: None,
        redistribution: None,
        columns: raw.declared.columns.clone(),
    };

    // Frame is carried in provenance, not in SnapshotEntry (entries are frame-agnostic km).
    let _ = (resolved.frame, resolved.center); // captured in provenance below
    Ok((SnapshotCorpus { manifest, entries }, resolved.provenance))
}
```

> Note: `resolved.frame` (a `CoordinateFrame`) and `resolved.center` are intentionally not stored on `SnapshotEntry` (which is frame-agnostic km, matching the existing fixture model). They live in `IngestProvenance`. The `let _ = ...` line documents that deliberately; remove it once a later consumer reads them, or replace with a debug assertion.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl ingest::normalize`
Expected: PASS (12 tests total in the module).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/ingest/normalize.rs
git commit -m "feat(jpl): normalize ingest records into SnapshotCorpus with provenance"
```

---

### Task 7: Vector-table front-end

**Files:**
- Create: `crates/pleiades-jpl/src/ingest/format/mod.rs`
- Create: `crates/pleiades-jpl/src/ingest/format/vector_table.rs`
- Create: `crates/pleiades-jpl/tests/fixtures/ingest/horizons_vectors.txt`
- Modify: `crates/pleiades-jpl/src/ingest/mod.rs`

The vector-table front-end targets Horizons **CSV-format** VECTORS output (`CSV_FORMAT=YES`), the deterministic shape: a header with labeled metadata lines, then rows between `$$SOE`/`$$EOE` of `JDTDB, CalendarDate, X, Y, Z, VX, VY, VZ`.

- [ ] **Step 1: Create the fixture**

Create `crates/pleiades-jpl/tests/fixtures/ingest/horizons_vectors.txt`:

```
*******************************************************************************
Target body name: Mars (499)
Center body name: Solar System Barycenter (0)             {source: DE440}
Reference frame : Ecliptic of J2000.0
Output units    : KM-S
*******************************************************************************
JDTDB, Calendar Date (TDB), X, Y, Z, VX, VY, VZ,
$$SOE
2451545.000000000, A.D. 2000-Jan-01 12:00:00.0000, 1.0, 2.0, 3.0, 0.1, 0.2, 0.3,
2451546.000000000, A.D. 2000-Jan-02 12:00:00.0000, 4.0, 5.0, 6.0, 0.4, 0.5, 0.6,
$$EOE
*******************************************************************************
```

- [ ] **Step 2: Write the failing test**

Append to `crates/pleiades-jpl/src/ingest/format/vector_table.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!("../../../tests/fixtures/ingest/horizons_vectors.txt");

    #[test]
    fn parses_header_metadata() {
        let raw = parse(SAMPLE).unwrap();
        assert_eq!(raw.declared.center.as_deref(), Some("Solar System Barycenter (0)"));
        assert_eq!(raw.declared.frame.as_deref(), Some("Ecliptic of J2000.0"));
        assert_eq!(raw.declared.time_scale.as_deref(), Some("TDB"));
        assert_eq!(raw.declared.units.as_deref(), Some("KM-S"));
    }

    #[test]
    fn parses_rows_between_markers() {
        let raw = parse(SAMPLE).unwrap();
        assert_eq!(raw.records.len(), 2);
        assert_eq!(raw.records[0].body_label, "Mars (499)");
        assert_eq!(raw.records[0].epoch_jd, 2_451_545.0);
        assert_eq!(raw.records[0].pos, [1.0, 2.0, 3.0]);
        assert_eq!(raw.records[1].pos, [4.0, 5.0, 6.0]);
    }

    #[test]
    fn missing_eoe_is_an_error() {
        let truncated = SAMPLE.replace("$$EOE", "");
        let err = parse(&truncated).unwrap_err();
        assert!(matches!(
            err,
            crate::ingest::IngestError::MissingMarker { marker: "$$EOE", .. }
        ));
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl ingest::format::vector_table`
Expected: FAIL — `cannot find function parse`.

- [ ] **Step 4: Write minimal implementation**

Create `crates/pleiades-jpl/src/ingest/format/vector_table.rs` (above the test module):

```rust
//! Horizons CSV-format VECTORS text front-end.

use crate::ingest::error::IngestError;
use crate::ingest::detect::InputFormat;
use crate::ingest::ir::{RawCorpus, RawEphemerisRecord, RawManifest};

const FORMAT: InputFormat = InputFormat::HorizonsVectorTable;

/// Parses a Horizons CSV-format vector text block into the neutral IR.
pub fn parse(source: &str) -> Result<RawCorpus, IngestError> {
    let declared = parse_header(source);
    let body_label = declared_target(source);
    let records = parse_rows(source, &body_label)?;
    Ok(RawCorpus { declared, records })
}

fn header_value<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    source.lines().find_map(|line| {
        let line = line.trim();
        let rest = line.strip_prefix(key)?;
        let rest = rest.trim_start();
        let rest = rest.strip_prefix(':')?.trim();
        // Drop trailing "{...}" provenance braces Horizons appends.
        let rest = rest.split('{').next().unwrap_or(rest).trim();
        if rest.is_empty() { None } else { Some(rest) }
    })
}

fn parse_header(source: &str) -> RawManifest {
    // Output units line carries both distance and time, e.g. "KM-S"; the time
    // scale is implied by the JDTDB column label, so read it from the header.
    let time_scale = if source.contains("JDTDB") || source.contains("(TDB)") {
        Some("TDB".to_string())
    } else if source.contains("JDTT") || source.contains("(TT)") {
        Some("TT".to_string())
    } else {
        None
    };
    RawManifest {
        source_label: Some("JPL Horizons".to_string()),
        center: header_value(source, "Center body name").map(str::to_string),
        frame: header_value(source, "Reference frame").map(str::to_string),
        time_scale,
        units: header_value(source, "Output units").map(str::to_string),
        columns: vec![],
    }
}

fn declared_target(source: &str) -> String {
    header_value(source, "Target body name")
        .unwrap_or("unknown")
        .to_string()
}

fn parse_rows(source: &str, body_label: &str) -> Result<Vec<RawEphemerisRecord>, IngestError> {
    let start = source
        .find("$$SOE")
        .ok_or(IngestError::MissingMarker { format: FORMAT, marker: "$$SOE" })?;
    let after_start = &source[start + "$$SOE".len()..];
    let end = after_start
        .find("$$EOE")
        .ok_or(IngestError::MissingMarker { format: FORMAT, marker: "$$EOE" })?;
    let block = &after_start[..end];

    let mut records = Vec::new();
    for (offset, raw_line) in block.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
        // Expect: JD, CalendarDate, X, Y, Z[, VX, VY, VZ]
        if fields.len() < 5 {
            return Err(IngestError::Malformed {
                format: FORMAT,
                line: offset + 1,
                detail: format!("expected at least 5 columns, found {}", fields.len()),
            });
        }
        let parse_num = |idx: usize| -> Result<f64, IngestError> {
            fields[idx].parse::<f64>().map_err(|_| IngestError::Malformed {
                format: FORMAT,
                line: offset + 1,
                detail: format!("non-numeric field {:?}", fields[idx]),
            })
        };
        records.push(RawEphemerisRecord {
            body_label: body_label.to_string(),
            epoch_jd: parse_num(0)?,
            pos: [parse_num(2)?, parse_num(3)?, parse_num(4)?],
            vel: None,
        });
    }
    Ok(records)
}
```

Create `crates/pleiades-jpl/src/ingest/format/mod.rs`:

```rust
//! Per-format tokenizing front-ends. Each converts bytes to the neutral IR.

pub mod vector_table;
```

Add to `crates/pleiades-jpl/src/ingest/mod.rs`:

```rust
pub mod format;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl ingest::format::vector_table`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-jpl/src/ingest/ crates/pleiades-jpl/tests/fixtures/ingest/horizons_vectors.txt
git commit -m "feat(jpl): add Horizons vector-table front-end"
```

---

### Task 8: Horizons API JSON front-end (delegates rows to vector-table)

**Files:**
- Create: `crates/pleiades-jpl/src/ingest/format/horizons_json.rs`
- Create: `crates/pleiades-jpl/tests/fixtures/ingest/horizons_api.json`
- Modify: `crates/pleiades-jpl/src/ingest/format/mod.rs`

- [ ] **Step 1: Create the fixture**

Create `crates/pleiades-jpl/tests/fixtures/ingest/horizons_api.json` — a real-shaped envelope whose `result` is the same text block (newlines escaped as `\n`):

```json
{
  "signature": { "source": "NASA/JPL Horizons API", "version": "1.2" },
  "result": "Target body name: Mars (499)\nCenter body name: Solar System Barycenter (0)\nReference frame : Ecliptic of J2000.0\nOutput units    : KM-S\n$$SOE\n2451545.000000000, A.D. 2000-Jan-01 12:00:00.0000, 1.0, 2.0, 3.0, 0.1, 0.2, 0.3,\n$$EOE\n"
}
```

- [ ] **Step 2: Write the failing test**

Append to `crates/pleiades-jpl/src/ingest/format/horizons_json.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!("../../../tests/fixtures/ingest/horizons_api.json");

    #[test]
    fn extracts_result_block_and_delegates_rows() {
        let raw = parse(SAMPLE).unwrap();
        assert_eq!(raw.records.len(), 1);
        assert_eq!(raw.records[0].body_label, "Mars (499)");
        assert_eq!(raw.records[0].pos, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn carries_version_into_source_label() {
        let raw = parse(SAMPLE).unwrap();
        let label = raw.declared.source_label.unwrap();
        assert!(label.contains("1.2"), "got: {label}");
    }

    #[test]
    fn missing_result_field_errors() {
        let err = parse(r#"{ "signature": { "version": "1.2" } }"#).unwrap_err();
        assert!(matches!(
            err,
            crate::ingest::IngestError::Malformed { .. }
        ));
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl ingest::format::horizons_json`
Expected: FAIL — `cannot find function parse`.

- [ ] **Step 4: Write minimal implementation**

Create `crates/pleiades-jpl/src/ingest/format/horizons_json.rs` (above the test module):

```rust
//! Horizons API JSON front-end. Extracts the `result` text payload and the
//! signature version, then delegates row parsing to the vector-table reader.

use crate::ingest::detect::InputFormat;
use crate::ingest::error::IngestError;
use crate::ingest::ir::RawCorpus;

use super::vector_table;

const FORMAT: InputFormat = InputFormat::HorizonsApiJson;

/// Parses a Horizons API JSON envelope into the neutral IR.
pub fn parse(source: &str) -> Result<RawCorpus, IngestError> {
    let result = json_string_field(source, "result").ok_or(IngestError::Malformed {
        format: FORMAT,
        line: 0,
        detail: "missing \"result\" string field".to_string(),
    })?;
    let version = json_string_field(source, "version");

    let mut corpus = vector_table::parse(&result)?;
    let label = match version {
        Some(v) => format!("JPL Horizons API v{v}"),
        None => "JPL Horizons API".to_string(),
    };
    corpus.declared.source_label = Some(label);
    Ok(corpus)
}

/// Extracts a JSON string field value by key, handling `\n`, `\t`, `\"`, `\\`,
/// and `\/` escapes. Minimal by design — Horizons envelopes are flat and the
/// fields we read are plain strings.
fn json_string_field(source: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let key_pos = source.find(&needle)?;
    let after_key = &source[key_pos + needle.len()..];
    let colon = after_key.find(':')?;
    let rest = after_key[colon + 1..].trim_start();
    let mut chars = rest.char_indices();
    // Expect an opening quote.
    if chars.next()?.1 != '"' {
        return None;
    }
    let mut out = String::new();
    let mut escaped = false;
    for (_, c) in chars {
        if escaped {
            out.push(match c {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                other => other,
            });
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return Some(out);
        } else {
            out.push(c);
        }
    }
    None // unterminated string
}
```

Add to `crates/pleiades-jpl/src/ingest/format/mod.rs`:

```rust
pub mod horizons_json;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl ingest::format::horizons_json`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-jpl/src/ingest/ crates/pleiades-jpl/tests/fixtures/ingest/horizons_api.json
git commit -m "feat(jpl): add Horizons API JSON front-end delegating to vector-table"
```

---

### Task 9: Generic-CSV front-end with column aliasing

**Files:**
- Create: `crates/pleiades-jpl/src/ingest/format/generic_csv.rs`
- Create: `crates/pleiades-jpl/tests/fixtures/ingest/generic.csv`
- Modify: `crates/pleiades-jpl/src/ingest/format/mod.rs`

- [ ] **Step 1: Create the fixture**

Create `crates/pleiades-jpl/tests/fixtures/ingest/generic.csv` — reordered/aliased columns and `#` header metadata:

```
# Source: Local export
# Reference frame: Ecliptic of J2000.0
# Time scale: TDB
# Output units: KM
# Center: 500@0
body,JD,PX,PY,PZ
Mars,2451545.0,1.0,2.0,3.0
Venus,2451545.0,4.0,5.0,6.0
```

- [ ] **Step 2: Write the failing test**

Append to `crates/pleiades-jpl/src/ingest/format/generic_csv.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!("../../../tests/fixtures/ingest/generic.csv");

    #[test]
    fn parses_aliased_reordered_columns() {
        let raw = parse(SAMPLE).unwrap();
        assert_eq!(raw.records.len(), 2);
        assert_eq!(raw.records[0].body_label, "Mars");
        assert_eq!(raw.records[0].epoch_jd, 2_451_545.0);
        assert_eq!(raw.records[0].pos, [1.0, 2.0, 3.0]);
        assert_eq!(raw.records[1].body_label, "Venus");
    }

    #[test]
    fn reads_header_comment_metadata() {
        let raw = parse(SAMPLE).unwrap();
        assert_eq!(raw.declared.time_scale.as_deref(), Some("TDB"));
        assert_eq!(raw.declared.units.as_deref(), Some("KM"));
        assert_eq!(raw.declared.center.as_deref(), Some("500@0"));
    }

    #[test]
    fn unmappable_column_errors() {
        let bad = "body,JD,PX,PY,WAT\nMars,1.0,1,2,3\n";
        let err = parse(bad).unwrap_err();
        assert!(matches!(
            err,
            crate::ingest::IngestError::ColumnUnresolved { .. }
        ));
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl ingest::format::generic_csv`
Expected: FAIL — `cannot find function parse`.

- [ ] **Step 4: Write minimal implementation**

Create `crates/pleiades-jpl/src/ingest/format/generic_csv.rs` (above the test module):

```rust
//! Tolerant generic JPL-style CSV front-end with column aliasing.

use crate::ingest::detect::InputFormat;
use crate::ingest::error::IngestError;
use crate::ingest::ir::{RawCorpus, RawEphemerisRecord, RawManifest};

const FORMAT: InputFormat = InputFormat::GenericCsv;

#[derive(Clone, Copy, PartialEq)]
enum Column {
    Body,
    Jd,
    X,
    Y,
    Z,
}

fn alias(header: &str) -> Option<Column> {
    match header.trim().to_ascii_lowercase().as_str() {
        "body" | "target" | "name" => Some(Column::Body),
        "jd" | "jdtdb" | "epoch" | "julian_day" => Some(Column::Jd),
        "x" | "px" | "x_km" => Some(Column::X),
        "y" | "py" | "y_km" => Some(Column::Y),
        "z" | "pz" | "z_km" => Some(Column::Z),
        _ => None,
    }
}

/// Parses a generic JPL-style CSV into the neutral IR.
pub fn parse(source: &str) -> Result<RawCorpus, IngestError> {
    let declared = parse_header(source);

    let mut data_lines = source
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty() && !l.trim_start().starts_with('#'));

    let (header_no, header_line) = data_lines
        .next()
        .ok_or(IngestError::MissingMarker { format: FORMAT, marker: "header row" })?;

    let mut mapping = Vec::new();
    for raw in header_line.split(',') {
        let col = alias(raw).ok_or_else(|| IngestError::ColumnUnresolved {
            column: raw.trim().to_string(),
            format: FORMAT,
        })?;
        mapping.push(col);
    }
    let _ = header_no;

    let mut records = Vec::new();
    for (line_no, line) in data_lines {
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        if fields.len() != mapping.len() {
            return Err(IngestError::Malformed {
                format: FORMAT,
                line: line_no + 1,
                detail: format!("expected {} columns, found {}", mapping.len(), fields.len()),
            });
        }
        let mut body = None;
        let mut jd = None;
        let mut pos = [0.0_f64; 3];
        for (col, value) in mapping.iter().zip(&fields) {
            match col {
                Column::Body => body = Some((*value).to_string()),
                Column::Jd => jd = Some(parse_num(value, line_no + 1)?),
                Column::X => pos[0] = parse_num(value, line_no + 1)?,
                Column::Y => pos[1] = parse_num(value, line_no + 1)?,
                Column::Z => pos[2] = parse_num(value, line_no + 1)?,
            }
        }
        records.push(RawEphemerisRecord {
            body_label: body.ok_or(IngestError::Malformed {
                format: FORMAT,
                line: line_no + 1,
                detail: "missing body column".to_string(),
            })?,
            epoch_jd: jd.ok_or(IngestError::Malformed {
                format: FORMAT,
                line: line_no + 1,
                detail: "missing JD column".to_string(),
            })?,
            pos,
            vel: None,
        });
    }
    Ok(RawCorpus { declared, records })
}

fn parse_num(value: &str, line: usize) -> Result<f64, IngestError> {
    value.parse::<f64>().map_err(|_| IngestError::Malformed {
        format: FORMAT,
        line,
        detail: format!("non-numeric field {value:?}"),
    })
}

fn comment_value(source: &str, key: &str) -> Option<String> {
    source.lines().find_map(|line| {
        let line = line.trim();
        let body = line.strip_prefix('#')?.trim();
        let rest = body.strip_prefix(key)?.trim_start();
        let rest = rest.strip_prefix(':')?.trim();
        if rest.is_empty() { None } else { Some(rest.to_string()) }
    })
}

fn parse_header(source: &str) -> RawManifest {
    RawManifest {
        source_label: comment_value(source, "Source"),
        center: comment_value(source, "Center"),
        frame: comment_value(source, "Reference frame"),
        time_scale: comment_value(source, "Time scale"),
        units: comment_value(source, "Output units"),
        columns: vec![],
    }
}
```

Add to `crates/pleiades-jpl/src/ingest/format/mod.rs`:

```rust
pub mod generic_csv;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl ingest::format::generic_csv`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-jpl/src/ingest/ crates/pleiades-jpl/tests/fixtures/ingest/generic.csv
git commit -m "feat(jpl): add tolerant generic-CSV front-end with column aliasing"
```

---

### Task 10: Public API — detect + dispatch + normalize end-to-end

**Files:**
- Modify: `crates/pleiades-jpl/src/ingest/mod.rs`
- Modify: `crates/pleiades-jpl/src/ingest/format/mod.rs`
- Modify: `crates/pleiades-jpl/src/lib.rs` (re-export the public API)
- Create: `crates/pleiades-jpl/tests/ingest_end_to_end.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/pleiades-jpl/tests/ingest_end_to_end.rs`:

```rust
use pleiades_jpl::ingest::{read_public_corpus, read_public_corpus_as, ExpectedProfile, InputFormat, Provenance};

const VECTORS: &str = include_str!("fixtures/ingest/horizons_vectors.txt");
const JSON: &str = include_str!("fixtures/ingest/horizons_api.json");
const CSV: &str = include_str!("fixtures/ingest/generic.csv");

#[test]
fn reads_vector_table_end_to_end() {
    let out = read_public_corpus(VECTORS.as_bytes(), &ExpectedProfile::default()).unwrap();
    assert_eq!(out.corpus.entries.len(), 2);
    assert_eq!(out.corpus.entries[0].body, pleiades_backend::CelestialBody::Mars);
    assert_eq!(out.provenance.frame, Provenance::Read);
}

#[test]
fn reads_horizons_json_end_to_end() {
    let out = read_public_corpus(JSON.as_bytes(), &ExpectedProfile::default()).unwrap();
    assert_eq!(out.corpus.entries.len(), 1);
    assert!(out.provenance.source_label.unwrap().contains("API"));
}

#[test]
fn reads_generic_csv_end_to_end() {
    let out = read_public_corpus(CSV.as_bytes(), &ExpectedProfile::default()).unwrap();
    assert_eq!(out.corpus.entries.len(), 2);
}

#[test]
fn explicit_format_bypasses_detection() {
    let out = read_public_corpus_as(VECTORS.as_bytes(), InputFormat::HorizonsVectorTable, &ExpectedProfile::default()).unwrap();
    assert_eq!(out.corpus.entries.len(), 2);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl --test ingest_end_to_end`
Expected: FAIL — `read_public_corpus` not found / `ingest` API not re-exported.

- [ ] **Step 3: Write minimal implementation**

Add a dispatch helper to `crates/pleiades-jpl/src/ingest/format/mod.rs`:

```rust
use crate::ingest::detect::InputFormat;
use crate::ingest::error::IngestError;
use crate::ingest::ir::RawCorpus;

/// Routes raw text to the front-end for `format`.
pub fn parse_to_ir(format: InputFormat, source: &str) -> Result<RawCorpus, IngestError> {
    match format {
        InputFormat::HorizonsVectorTable => vector_table::parse(source),
        InputFormat::HorizonsApiJson => horizons_json::parse(source),
        InputFormat::GenericCsv => generic_csv::parse(source),
    }
}
```

Add the public API to `crates/pleiades-jpl/src/ingest/mod.rs`:

```rust
use std::path::Path;

pub use detect::{detect_format, InputFormat};

/// A successfully ingested external corpus plus its provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct PublicCorpus {
    /// The normalized corpus, ready for the existing validation/comparison surfaces.
    pub corpus: crate::backend::SnapshotCorpus,
    /// Per-attribute provenance (Read vs Asserted) and source labels.
    pub provenance: IngestProvenance,
}

/// Reads external bytes, auto-detecting the format, into a `PublicCorpus`.
pub fn read_public_corpus(
    bytes: &[u8],
    expected: &ExpectedProfile,
) -> Result<PublicCorpus, IngestError> {
    let format = detect::detect_format(bytes)?;
    read_public_corpus_as(bytes, format, expected)
}

/// Reads external bytes with an explicit format (bypassing detection).
pub fn read_public_corpus_as(
    bytes: &[u8],
    format: InputFormat,
    expected: &ExpectedProfile,
) -> Result<PublicCorpus, IngestError> {
    let text = String::from_utf8_lossy(bytes);
    let raw = format::parse_to_ir(format, &text)?;
    let (corpus, provenance) = normalize::normalize(raw, expected)?;
    Ok(PublicCorpus { corpus, provenance })
}

/// Reads an external corpus from a file path.
pub fn read_public_corpus_from_path(
    path: impl AsRef<Path>,
    expected: &ExpectedProfile,
) -> Result<PublicCorpus, IngestError> {
    let bytes = std::fs::read(path.as_ref()).map_err(|error| IngestError::Malformed {
        format: InputFormat::GenericCsv,
        line: 0,
        detail: format!("could not read {}: {error}", path.as_ref().display()),
    })?;
    read_public_corpus(&bytes, expected)
}
```

> Note: `normalize::normalize` is `pub` (Task 6) but the module is `pub(crate)`; re-exporting `normalize` items is unnecessary — `read_public_corpus_as` calls it internally. Keep `mod normalize` as `pub(crate)`.

In `crates/pleiades-jpl/src/lib.rs`, the existing `pub mod ingest;` already exposes the API as `pleiades_jpl::ingest::*`. No further re-export needed (the test imports from `pleiades_jpl::ingest`).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl --test ingest_end_to_end`
Expected: PASS (4 tests).

- [ ] **Step 5: Run the whole crate + clippy**

Run: `cargo test -p pleiades-jpl && cargo clippy -p pleiades-jpl --all-targets --all-features -- -D warnings`
Expected: PASS, no warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-jpl/src/ingest/ crates/pleiades-jpl/tests/ingest_end_to_end.rs
git commit -m "feat(jpl): wire public read_public_corpus API end-to-end"
```

---

### Task 11: de440 round-trip anchor test

This proves the AU→km / frame / body mapping agrees with the real committed corpus, not just synthetic fixtures.

**Files:**
- Modify: `crates/pleiades-jpl/tests/ingest_end_to_end.rs`
- Create: `crates/pleiades-jpl/tests/fixtures/ingest/anchor_mars.txt`

- [ ] **Step 1: Derive the anchor fixture from the committed corpus**

Pick one body+epoch present in `crates/pleiades-jpl/data/corpus/interior.csv` (e.g. Mars at a known JD). Read its committed `x_km,y_km,z_km`. Hand-author `crates/pleiades-jpl/tests/fixtures/ingest/anchor_mars.txt` as a Horizons vector block (same header style as `horizons_vectors.txt`) carrying **those exact km values** at that JD. This makes the fixture a faithful re-expression of a real corpus row.

Document the chosen body/JD and the source row in a comment at the top of the fixture (Horizons header lines are comments-friendly).

- [ ] **Step 2: Write the failing test**

Append to `crates/pleiades-jpl/tests/ingest_end_to_end.rs`:

```rust
const ANCHOR: &str = include_str!("fixtures/ingest/anchor_mars.txt");

#[test]
fn anchor_matches_committed_corpus_within_km_tolerance() {
    let out = read_public_corpus(ANCHOR.as_bytes(), &ExpectedProfile::default()).unwrap();
    let e = &out.corpus.entries[0];
    // Replace these with the exact committed values + JD chosen in Step 1:
    let expected_x_km = /* COMMITTED_X */ 0.0_f64;
    let expected_y_km = /* COMMITTED_Y */ 0.0_f64;
    let expected_z_km = /* COMMITTED_Z */ 0.0_f64;
    assert!((e.x_km - expected_x_km).abs() <= 1.0, "x dx={}", e.x_km - expected_x_km);
    assert!((e.y_km - expected_y_km).abs() <= 1.0, "y dy={}", e.y_km - expected_y_km);
    assert!((e.z_km - expected_z_km).abs() <= 1.0, "z dz={}", e.z_km - expected_z_km);
}
```

> The 1 km tolerance mirrors the existing `corpus_regen` gate's per-slice tolerance. Since the fixture carries the committed km values verbatim, this should match to floating-point round-trip (well within 1 km); the tolerance guards formatting/parse precision only.

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl --test ingest_end_to_end anchor`
Expected: FAIL until the `COMMITTED_*` placeholders are replaced with the real values from Step 1.

- [ ] **Step 4: Fill in the committed values and re-run**

Replace the three placeholders with the exact committed numbers.

Run: `cargo test -p pleiades-jpl --test ingest_end_to_end anchor`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/tests/
git commit -m "test(jpl): anchor ingest round-trip against committed de440 corpus row"
```

---

### Task 12: Live-fetch seam behind `horizons-fetch` feature

**Files:**
- Modify: `crates/pleiades-jpl/Cargo.toml` (add `[features]` + optional `ureq` dep)
- Create: `crates/pleiades-jpl/src/ingest/fetch.rs`
- Modify: `crates/pleiades-jpl/src/ingest/mod.rs`

- [ ] **Step 1: Add the feature and optional dependency**

In `crates/pleiades-jpl/Cargo.toml`, under `[dependencies]` add:

```toml
ureq = { version = "2", optional = true, default-features = false, features = ["tls"] }
```

And add a features table (create it if absent):

```toml
[features]
horizons-fetch = ["dep:ureq"]
```

> If the workspace pins dependency versions centrally (check the root `Cargo.toml` `[workspace.dependencies]`), add `ureq` there and use `ureq = { workspace = true, optional = true }` instead. Grep the root manifest first.

- [ ] **Step 2: Write the failing test**

Append to `crates/pleiades-jpl/src/ingest/fetch.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::{ExpectedProfile, Provenance};

    struct FakeSource(&'static str);

    impl HorizonsSource for FakeSource {
        fn fetch(&self, _query: &HorizonsQuery) -> Result<Vec<u8>, crate::ingest::IngestError> {
            Ok(self.0.as_bytes().to_vec())
        }
    }

    const VECTORS: &str = include_str!("../../tests/fixtures/ingest/horizons_vectors.txt");

    #[test]
    fn fetch_routes_bytes_through_offline_path() {
        let source = FakeSource(VECTORS);
        let query = HorizonsQuery {
            command: "499".to_string(),
            center: "500@0".to_string(),
            start: 2_451_545.0,
            stop: 2_451_546.0,
            step: "1d".to_string(),
            format: HorizonsWireFormat::Text,
        };
        let out = fetch_public_corpus(&source, &query, &ExpectedProfile::default()).unwrap();
        assert_eq!(out.corpus.entries.len(), 2);
        assert_eq!(out.provenance.frame, Provenance::Read);
        assert!(out.provenance.request_url.is_some());
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl --features horizons-fetch ingest::fetch`
Expected: FAIL — `cannot find type HorizonsSource`.

- [ ] **Step 4: Write minimal implementation**

Create `crates/pleiades-jpl/src/ingest/fetch.rs` (above the test module). The whole file is feature-gated:

```rust
//! Live Horizons fetch seam. Compiled only under `feature = "horizons-fetch"`.
//!
//! The fetch produces raw bytes that re-enter the same offline parse/normalize
//! path; nothing here parses ephemeris data itself.

use super::error::IngestError;
use super::profile::ExpectedProfile;
use super::{read_public_corpus, PublicCorpus};

/// The wire format to request from Horizons.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HorizonsWireFormat {
    /// Plain vector-table text.
    Text,
    /// API JSON envelope.
    Json,
}

/// A Horizons ephemeris query.
#[derive(Clone, Debug, PartialEq)]
pub struct HorizonsQuery {
    /// Target body id (Horizons `COMMAND`).
    pub command: String,
    /// Center/origin (e.g. "500@0").
    pub center: String,
    /// Start Julian day.
    pub start: f64,
    /// Stop Julian day.
    pub stop: f64,
    /// Step size (e.g. "1d").
    pub step: String,
    /// Requested wire format.
    pub format: HorizonsWireFormat,
}

impl HorizonsQuery {
    /// Builds the Horizons API request URL for this query.
    pub fn to_url(&self) -> String {
        let fmt = match self.format {
            HorizonsWireFormat::Text => "text",
            HorizonsWireFormat::Json => "json",
        };
        format!(
            "https://ssd.jpl.nasa.gov/api/horizons.api?format={fmt}&EPHEM_TYPE=VECTORS&CSV_FORMAT=YES\
&COMMAND='{}'&CENTER='{}'&START_TIME='JD{}'&STOP_TIME='JD{}'&STEP_SIZE='{}'",
            self.command, self.center, self.start, self.stop, self.step
        )
    }
}

/// A source of raw Horizons product bytes. The real implementation hits the
/// network; tests inject saved fixtures.
pub trait HorizonsSource {
    /// Returns the raw bytes of a Horizons product for `query`.
    fn fetch(&self, query: &HorizonsQuery) -> Result<Vec<u8>, IngestError>;
}

/// Fetches and ingests a Horizons product through the offline path.
pub fn fetch_public_corpus<S: HorizonsSource>(
    source: &S,
    query: &HorizonsQuery,
    expected: &ExpectedProfile,
) -> Result<PublicCorpus, IngestError> {
    let bytes = source.fetch(query)?;
    let mut out = read_public_corpus(&bytes, expected)?;
    out.provenance.request_url = Some(query.to_url());
    Ok(out)
}

/// Live HTTP implementation of [`HorizonsSource`] over `ureq`.
pub struct HttpHorizonsSource;

impl HorizonsSource for HttpHorizonsSource {
    fn fetch(&self, query: &HorizonsQuery) -> Result<Vec<u8>, IngestError> {
        let url = query.to_url();
        let response = ureq::get(&url)
            .call()
            .map_err(|error| IngestError::Fetch { detail: error.to_string() })?;
        let mut buf = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut buf)
            .map_err(|error| IngestError::Fetch { detail: error.to_string() })?;
        Ok(buf)
    }
}

use std::io::Read as _;
```

Add to `crates/pleiades-jpl/src/ingest/mod.rs`:

```rust
#[cfg(feature = "horizons-fetch")]
pub mod fetch;

#[cfg(feature = "horizons-fetch")]
pub use fetch::{fetch_public_corpus, HorizonsQuery, HorizonsSource, HorizonsWireFormat, HttpHorizonsSource};
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl --features horizons-fetch ingest::fetch`
Expected: PASS (1 test).

- [ ] **Step 6: Verify the default build stays network-free**

Run: `cargo tree -p pleiades-jpl | grep -i ureq || echo "no ureq in default build (correct)"`
Expected: prints the "no ureq" line.

Run: `cargo clippy -p pleiades-jpl --all-targets --all-features -- -D warnings`
Expected: PASS (the `--all-features` lint covers the fetch module).

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-jpl/Cargo.toml crates/pleiades-jpl/src/ingest/ Cargo.lock
git commit -m "feat(jpl): add quarantined live Horizons fetch behind horizons-fetch feature"
```

---

### Task 13: Optional gated live integration test

**Files:**
- Create: `crates/pleiades-jpl/tests/horizons_live.rs`

- [ ] **Step 1: Write the gated test**

Create `crates/pleiades-jpl/tests/horizons_live.rs`:

```rust
//! Opt-in live Horizons fetch test. Skipped unless both the `horizons-fetch`
//! feature is enabled and `PLEIADES_HORIZONS_LIVE=1` is set (mirrors the
//! kernel-gated `corpus_regen` pattern). Never runs in default CI.

#![cfg(feature = "horizons-fetch")]

use pleiades_jpl::ingest::{
    fetch_public_corpus, ExpectedProfile, HorizonsQuery, HorizonsWireFormat, HttpHorizonsSource,
};

#[test]
fn live_fetch_mars_vectors() {
    if std::env::var("PLEIADES_HORIZONS_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping live Horizons test (set PLEIADES_HORIZONS_LIVE=1 to run)");
        return;
    }
    let query = HorizonsQuery {
        command: "499".to_string(),
        center: "500@0".to_string(),
        start: 2_451_545.0,
        stop: 2_451_546.0,
        step: "1d".to_string(),
        format: HorizonsWireFormat::Text,
    };
    let out = fetch_public_corpus(&HttpHorizonsSource, &query, &ExpectedProfile::default())
        .expect("live fetch should succeed");
    assert!(!out.corpus.entries.is_empty());
}
```

- [ ] **Step 2: Verify it compiles under the feature and no-ops without the env var**

Run: `cargo test -p pleiades-jpl --features horizons-fetch --test horizons_live`
Expected: PASS (prints the skip message; no network hit).

- [ ] **Step 3: Verify it is excluded from the default build**

Run: `cargo test -p pleiades-jpl --test horizons_live`
Expected: compiles to an empty test binary (the `#![cfg(...)]` excludes everything); 0 tests run.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-jpl/tests/horizons_live.rs
git commit -m "test(jpl): add opt-in gated live Horizons fetch integration test"
```

---

### Task 14: `ingest-public` CLI subcommand (offline)

**Files:**
- Modify: `crates/pleiades-validate/src/render/cli.rs`
- Possibly create: `crates/pleiades-validate/src/ingest_report.rs` (rendering helper)
- Modify: `crates/pleiades-validate/src/lib.rs` (if a new module is added)

- [ ] **Step 1: Inspect the existing dispatch and an arg-parsing sibling**

Read `crates/pleiades-validate/src/render/cli.rs` around the `match args.first().copied()` block (line ~73) and the `--rounds` parsing example (line ~2040) and `ensure_no_extra_args` (line ~2026) to match the established style.

- [ ] **Step 2: Write the failing test**

Append a test module at the bottom of `crates/pleiades-validate/src/render/cli.rs` (or extend the existing tests):

```rust
#[cfg(test)]
mod ingest_public_tests {
    use super::render_cli;

    #[test]
    fn ingest_public_reports_detected_format_and_counts() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../pleiades-jpl/tests/fixtures/ingest/horizons_vectors.txt"
        );
        let out = render_cli(&["ingest-public", "--input", path]).unwrap();
        assert!(out.contains("vector-table"), "got: {out}");
        assert!(out.contains("rows: 2") || out.contains("entries: 2"), "got: {out}");
        assert!(out.contains("frame:"), "got: {out}");
    }

    #[test]
    fn ingest_public_requires_input() {
        let err = render_cli(&["ingest-public"]).unwrap_err();
        assert!(err.contains("--input"), "got: {err}");
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p pleiades-validate ingest_public`
Expected: FAIL — unknown subcommand / function absent.

- [ ] **Step 4: Write minimal implementation**

Add a match arm in `render_cli` (alongside the other `Some("...")` arms):

```rust
        Some("ingest-public") => render_ingest_public(&args[1..]),
```

Add the handler (in `cli.rs` or a new `ingest_report.rs` re-exported through `lib.rs`):

```rust
fn render_ingest_public(args: &[&str]) -> Result<String, String> {
    let mut input: Option<&str> = None;
    let mut format: Option<&str> = None;
    let mut iter = args.iter().copied();
    while let Some(arg) = iter.next() {
        match arg {
            "--input" => {
                let value = iter.next().ok_or("missing value for --input")?;
                if input.replace(value).is_some() {
                    return Err("duplicate --input argument".to_string());
                }
            }
            "--format" => {
                let value = iter.next().ok_or("missing value for --format")?;
                if format.replace(value).is_some() {
                    return Err("duplicate --format argument".to_string());
                }
            }
            other => return Err(format!("unexpected argument: {other}")),
        }
    }
    let input = input.ok_or("ingest-public requires --input <path>")?;

    use pleiades_jpl::ingest::{
        read_public_corpus_from_path, ExpectedProfile, InputFormat, Provenance,
    };

    // For this phase the CLI asserts nothing; the profile is all-silent.
    let expected = ExpectedProfile::default();

    let out = match format {
        None => read_public_corpus_from_path(input, &expected),
        Some(name) => {
            let fmt = match name {
                "vector-table" => InputFormat::HorizonsVectorTable,
                "horizons-json" => InputFormat::HorizonsApiJson,
                "generic-csv" => InputFormat::GenericCsv,
                other => return Err(format!("unknown --format: {other}")),
            };
            let bytes = std::fs::read(input).map_err(|e| format!("could not read {input}: {e}"))?;
            pleiades_jpl::ingest::read_public_corpus_as(&bytes, fmt, &expected)
        }
    }
    .map_err(|e| format!("ingest failed: {e}"))?;

    let detected = pleiades_jpl::ingest::detect_format(
        &std::fs::read(input).map_err(|e| format!("could not read {input}: {e}"))?,
    )
    .map(|f| f.to_string())
    .unwrap_or_else(|_| "explicit".to_string());

    let prov = |p: Provenance| match p {
        Provenance::Read => "read",
        Provenance::Asserted => "asserted",
    };
    let body_count = {
        use std::collections::BTreeSet;
        out.corpus
            .entries
            .iter()
            .map(|e| format!("{:?}", e.body))
            .collect::<BTreeSet<_>>()
            .len()
    };

    Ok(format!(
        "ingest-public\n  format: {detected}\n  rows: {}\n  bodies: {}\n  frame: {} ({})\n  time-scale: ({})\n  units: ({})\n  center: ({})",
        out.corpus.entries.len(),
        body_count,
        out.provenance
            .source_label
            .clone()
            .unwrap_or_else(|| "—".to_string()),
        prov(out.provenance.frame),
        prov(out.provenance.time_scale),
        prov(out.provenance.units),
        prov(out.provenance.center),
    ))
}
```

> Confirm `pleiades-validate/Cargo.toml` depends on `pleiades-jpl` (it does — it consumes corpus/comparison surfaces). If a helper module is added, declare it in `lib.rs` with `mod ingest_report;` and `pub use`.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p pleiades-validate ingest_public`
Expected: PASS (2 tests).

- [ ] **Step 6: Update the CLI help text**

Add a one-line entry for `ingest-public` to the `help_text()` function in `cli.rs` (find it near the `Some("help")` arm, ~line 2021), matching the existing format.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-validate/src/
git commit -m "feat(validate): add offline ingest-public subcommand"
```

---

### Task 15: Documentation and plan updates

**Files:**
- Modify: `plan/stages/01-production-reference-corpus.md`
- Modify: `PLAN.md`
- Modify: `README.md`
- Modify: `crates/pleiades-jpl/src/lib.rs` (crate doc note, optional)

- [ ] **Step 1: Update the Phase 1 stage doc**

In `plan/stages/01-production-reference-corpus.md`, change the "Remaining implementation work" bullet about the broad public-data reader to **Met**, and update the Exit-criteria note that currently says "a reader for arbitrary external public data products is still open" to reflect that the reader now exists (Horizons vector-table, API JSON, generic CSV; live fetch behind `horizons-fetch`). Leave the asteroid-kernel item open.

- [ ] **Step 2: Update PLAN.md limits**

In `PLAN.md`, revise the first "Important current limits" bullet (the `pleiades-jpl` one) so it no longer says "It is not yet a broad public-data reader for arbitrary external JPL-style data products" — now it is, for the three supported shapes. Keep the asteroid-kernel sentence. Update the `Status:` line date to the implementation date.

- [ ] **Step 3: Update README**

In `README.md`, the `pleiades-jpl` row/limits mention the corpus; add a brief note that the crate can now ingest external JPL-style products (vector-table / API JSON / generic CSV) into the corpus types, with optional live fetch behind a non-default feature.

- [ ] **Step 4: Verify the full workspace gates pass**

Run: `cargo fmt --all --check`
Run: `cargo test --workspace`
Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Run: `cargo run -q -p pleiades-validate -- workspace-audit`
Expected: all PASS.

- [ ] **Step 5: Commit**

```bash
git add PLAN.md plan/stages/01-production-reference-corpus.md README.md crates/pleiades-jpl/src/lib.rs
git commit -m "docs: mark broad public-data reader Met and document ingest surface"
```

---

## Final verification (run after all tasks)

- [ ] `cargo fmt --all --check` — clean
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean
- [ ] `cargo test --workspace` — green
- [ ] `cargo test -p pleiades-jpl --features horizons-fetch` — green (offline; live test no-ops)
- [ ] `cargo run -q -p pleiades-validate -- workspace-audit` — passes
- [ ] `cargo tree -p pleiades-jpl | grep -i ureq` — no match (default build network-free)
- [ ] Manual smoke: `cargo run -q -p pleiades-validate -- ingest-public --input crates/pleiades-jpl/tests/fixtures/ingest/horizons_vectors.txt`

## Notes carried from the spec

- **Frame/Center not stored on entries:** `SnapshotEntry` is frame-agnostic km; resolved frame/center live in `IngestProvenance`. This matches the existing fixture model.
- **Fail-closed posture:** contradiction is never auto-resolved; silent+unasserted is `Undetermined`; silent+asserted is `Asserted`; unsupported declared values (apparent/topocentric/UTC/non-SSB center) are `Unsupported`.
- **Determinism:** all default tests are offline; the only network path is feature- and env-gated.
- **Out of scope (tracked separately):** asteroid SPK kernel adoption; wiring `--fetch` into the CLI; apparent/topocentric/native-sidereal ingestion.
```