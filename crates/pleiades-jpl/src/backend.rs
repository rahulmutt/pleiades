//! Snapshot-backed `EphemerisBackend` implementation, fixture interpolation,
//! and manifest/row corpus parsing for the checked-in JPL reference snapshot.

use core::fmt;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use pleiades_backend::{
    validate_observer_policy, validate_request_policy, validate_zodiac_policy, AccuracyClass,
    BackendCapabilities, BackendFamily, BackendId, BackendMetadata, BackendProvenance,
    EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest, EphemerisResult,
    QualityAnnotation,
};
use pleiades_types::{
    Apparentness, CoordinateFrame, CustomBodyId, EclipticCoordinates, Instant, JulianDay, Latitude,
    Longitude, Motion, TimeRange, TimeScale, ZodiacMode,
};

use crate::*;

/// Interpolation path used for a hold-out quality sample.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterpolationQualityKind {
    /// Four-point interpolation on a four-sample window.
    Cubic,
    /// Three-point interpolation on a three-sample window.
    Quadratic,
    /// Two-point linear fallback between adjacent samples.
    Linear,
}

impl InterpolationQualityKind {
    /// Human-readable label for release-facing reporting.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cubic => "cubic",
            Self::Quadratic => "quadratic",
            Self::Linear => "linear",
        }
    }
}

/// A coarse hold-out check for the snapshot backend's current interpolation path.
#[derive(Clone, Debug, PartialEq)]
pub struct InterpolationQualitySample {
    /// Body evaluated by this check.
    pub body: pleiades_backend::CelestialBody,
    /// Held-out exact epoch used for comparison.
    pub epoch: Instant,
    /// Interpolation path selected for the held-out sample.
    pub interpolation_kind: InterpolationQualityKind,
    /// Span between the bracketing fixture entries in days.
    pub bracket_span_days: f64,
    /// Absolute wrapped longitude error in degrees.
    pub longitude_error_deg: f64,
    /// Absolute latitude error in degrees.
    pub latitude_error_deg: f64,
    /// Absolute distance error in astronomical units.
    pub distance_error_au: f64,
}

/// Validation errors for an interpolation-quality hold-out sample that drifted
/// away from the checked-in evidence.
#[derive(Clone, Debug, PartialEq)]
pub enum InterpolationQualitySampleValidationError {
    /// The stored epoch no longer uses TDB.
    NonTdbEpoch {
        /// Body evaluated by the sample.
        body: pleiades_backend::CelestialBody,
        /// The time scale that drifted into the sample.
        found: TimeScale,
    },
    /// A rendered field is no longer finite.
    NonFiniteField {
        /// Body evaluated by the sample.
        body: pleiades_backend::CelestialBody,
        /// Name of the field that drifted.
        field: &'static str,
    },
    /// A rendered field should stay non-negative.
    NegativeField {
        /// Body evaluated by the sample.
        body: pleiades_backend::CelestialBody,
        /// Name of the field that drifted.
        field: &'static str,
    },
}

impl fmt::Display for InterpolationQualitySampleValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonTdbEpoch { body, found } => {
                write!(
                    f,
                    "interpolation sample for {body} must use TDB, found {found}"
                )
            }
            Self::NonFiniteField { body, field } => {
                write!(f, "interpolation sample for {body} has non-finite {field}")
            }
            Self::NegativeField { body, field } => {
                write!(f, "interpolation sample for {body} has negative {field}")
            }
        }
    }
}

impl std::error::Error for InterpolationQualitySampleValidationError {}

impl InterpolationQualitySample {
    /// Returns a compact release-facing summary line.
    pub fn summary_line(&self) -> String {
        format!(
            "{} at {}: {} interpolation, bracket span {:.1} d, |Δlon|={:.12}°, |Δlat|={:.12}°, |Δdist|={:.12} AU",
            self.body,
            self.epoch.summary_line(),
            self.interpolation_kind.label(),
            self.bracket_span_days,
            self.longitude_error_deg,
            self.latitude_error_deg,
            self.distance_error_au,
        )
    }

    /// Returns `Ok(())` when the sample still matches the checked-in evidence.
    pub fn validate(&self) -> Result<(), InterpolationQualitySampleValidationError> {
        if self.epoch.scale != TimeScale::Tdb {
            return Err(InterpolationQualitySampleValidationError::NonTdbEpoch {
                body: self.body.clone(),
                found: self.epoch.scale,
            });
        }

        if !self.epoch.julian_day.days().is_finite() {
            return Err(InterpolationQualitySampleValidationError::NonFiniteField {
                body: self.body.clone(),
                field: "epoch",
            });
        }

        for (field, value) in [
            ("bracket_span_days", self.bracket_span_days),
            ("longitude_error_deg", self.longitude_error_deg),
            ("latitude_error_deg", self.latitude_error_deg),
            ("distance_error_au", self.distance_error_au),
        ] {
            if !value.is_finite() {
                return Err(InterpolationQualitySampleValidationError::NonFiniteField {
                    body: self.body.clone(),
                    field,
                });
            }
            if value < 0.0 {
                return Err(InterpolationQualitySampleValidationError::NegativeField {
                    body: self.body.clone(),
                    field,
                });
            }
        }

        Ok(())
    }
}

impl fmt::Display for InterpolationQualitySample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A reference-backend implementation backed by JPL Horizons fixture data.
#[derive(Debug, Default, Clone, Copy)]
pub struct JplSnapshotBackend;

impl JplSnapshotBackend {
    /// Creates a new snapshot backend.
    pub const fn new() -> Self {
        Self
    }
}

impl EphemerisBackend for JplSnapshotBackend {
    fn metadata(&self) -> BackendMetadata {
        let bodies = reference_bodies().to_vec();
        let epochs = reference_epochs();
        let dataset_missing = snapshot_error().is_some();
        BackendMetadata {
            id: BackendId::new("jpl-snapshot"),
            version: "0.1.0".to_string(),
            family: BackendFamily::ReferenceData,
            provenance: BackendProvenance {
                summary: "NASA/JPL Horizons DE441 geocentric fixture with exact epoch lookup, cubic interpolation on four-sample windows, and mean-obliquity equatorial output"
                    .to_string(),
                data_sources: vec![
                    "NASA/JPL Horizons API vector tables (DE441)".to_string(),
                    "Checked-in derivative CSV fixture: epoch_jd,body,x_km,y_km,z_km".to_string(),
                    "Cubic interpolation on four-sample windows, quadratic interpolation on three-sample windows, and linear fallback between adjacent same-body fixture samples".to_string(),
                ],
            },
            nominal_range: if dataset_missing {
                TimeRange::new(None, None)
            } else {
                TimeRange::new(epochs.first().copied(), epochs.last().copied())
            },
            supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
            body_coverage: bodies,
            supported_frames: vec![CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
            capabilities: BackendCapabilities {
                geocentric: true,
                topocentric: false,
                apparent: false,
                mean: true,
                batch: true,
                native_sidereal: false,
            },
            accuracy: AccuracyClass::Exact,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: pleiades_backend::CelestialBody) -> bool {
        snapshot_entries()
            .map(|entries| entries.iter().any(|entry| entry.body == body))
            .unwrap_or(false)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        validate_request_policy(
            req,
            "the JPL snapshot backend",
            &[TimeScale::Tt, TimeScale::Tdb],
            &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
            true,
            false,
        )?;

        validate_zodiac_policy(req, "the JPL snapshot backend", &[ZodiacMode::Tropical])?;

        validate_observer_policy(req, "the JPL snapshot backend", false)?;

        if let Some(error) = snapshot_error() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::MissingDataset,
                format!("the JPL snapshot corpus could not be loaded: {error}"),
            ));
        }

        let resolved = resolve_fixture_state(req.body.clone(), req.instant.julian_day.days())?;

        let mut result = EphemerisResult::new(
            BackendId::new("jpl-snapshot"),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        let ecliptic = resolved.entry.ecliptic();
        result.ecliptic = Some(ecliptic);
        result.equatorial = Some(ecliptic.to_equatorial(req.instant.mean_obliquity()));
        result.motion = None::<Motion>;
        result.quality = resolved.quality;
        Ok(result)
    }
}

/// File-level metadata parsed from a checked-in JPL-style snapshot.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SnapshotManifest {
    /// Human-readable title comment from the fixture.
    pub title: Option<String>,
    /// Source comment from the fixture.
    pub source: Option<String>,
    /// Coverage comment from the fixture.
    pub coverage: Option<String>,
    /// Redistribution posture comment from the fixture.
    pub redistribution: Option<String>,
    /// Parsed columns comment from the fixture.
    pub columns: Vec<String>,
}

/// Structured validation errors for a parsed JPL snapshot manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotManifestValidationError {
    /// The manifest did not include a human-readable title comment.
    MissingTitle,
    /// The manifest did not include a source provenance comment.
    MissingSource,
    /// The manifest did not include any column names.
    MissingColumns,
    /// The manifest included a blank coverage comment after trimming.
    BlankCoverage,
    /// The manifest included a blank redistribution comment after trimming.
    BlankRedistribution,
    /// The manifest carried surrounding whitespace in a provenance field.
    SurroundedByWhitespace { field: &'static str },
    /// A parsed column name was blank after trimming.
    BlankColumn { index: usize },
    /// The manifest reused a column name after trimming.
    DuplicateColumn {
        first_index: usize,
        second_index: usize,
        name: String,
    },
}

impl SnapshotManifestValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingTitle => "missing title",
            Self::MissingSource => "missing source",
            Self::MissingColumns => "missing columns",
            Self::BlankCoverage => "blank coverage",
            Self::BlankRedistribution => "blank redistribution",
            Self::SurroundedByWhitespace { .. } => "surrounded by whitespace",
            Self::BlankColumn { .. } => "blank column",
            Self::DuplicateColumn { .. } => "duplicate column",
        }
    }
}

impl fmt::Display for SnapshotManifestValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurroundedByWhitespace { field } => {
                write!(f, "{field} contains surrounding whitespace")
            }
            Self::BlankColumn { index } => write!(f, "blank column at index {index}"),
            Self::DuplicateColumn {
                first_index,
                second_index,
                name,
            } => write!(
                f,
                "duplicate column '{name}' at index {second_index} (first seen at index {first_index})"
            ),
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for SnapshotManifestValidationError {}

impl SnapshotManifest {
    fn trimmed_or<'a>(value: Option<&'a str>, fallback: &'static str) -> Cow<'a, str> {
        match value.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) => Cow::Borrowed(value),
            None => Cow::Borrowed(fallback),
        }
    }

    /// Returns the source label, or the provided fallback when the manifest omits it.
    pub fn source_or(&self, fallback: &'static str) -> Cow<'_, str> {
        Self::trimmed_or(self.source.as_deref(), fallback)
    }

    /// Returns the coverage label, or the provided fallback when the manifest omits it.
    pub fn coverage_or(&self, fallback: &'static str) -> Cow<'_, str> {
        Self::trimmed_or(self.coverage.as_deref(), fallback)
    }

    /// Returns the redistribution label, or the provided fallback when the manifest omits it.
    pub fn redistribution_or(&self, fallback: &'static str) -> Cow<'_, str> {
        Self::trimmed_or(self.redistribution.as_deref(), fallback)
    }

    pub(crate) fn columns_summary(&self) -> String {
        if self.columns.is_empty() {
            "none".to_string()
        } else {
            self.columns.join(", ")
        }
    }

    /// Validates that the parsed manifest still exposes the expected title,
    /// source, optional coverage, and column metadata.
    pub fn validate(&self) -> Result<(), SnapshotManifestValidationError> {
        if self
            .title
            .as_deref()
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .is_none()
        {
            return Err(SnapshotManifestValidationError::MissingTitle);
        }
        if self
            .title
            .as_deref()
            .is_some_and(has_surrounding_whitespace)
        {
            return Err(SnapshotManifestValidationError::SurroundedByWhitespace { field: "title" });
        }
        if self
            .source
            .as_deref()
            .map(str::trim)
            .filter(|source| !source.is_empty())
            .is_none()
        {
            return Err(SnapshotManifestValidationError::MissingSource);
        }
        if self
            .source
            .as_deref()
            .is_some_and(has_surrounding_whitespace)
        {
            return Err(SnapshotManifestValidationError::SurroundedByWhitespace {
                field: "source",
            });
        }
        if matches!(self.coverage.as_deref(), Some(coverage) if coverage.trim().is_empty()) {
            return Err(SnapshotManifestValidationError::BlankCoverage);
        }
        if self
            .coverage
            .as_deref()
            .is_some_and(has_surrounding_whitespace)
        {
            return Err(SnapshotManifestValidationError::SurroundedByWhitespace {
                field: "coverage",
            });
        }
        if matches!(self.redistribution.as_deref(), Some(redistribution) if redistribution.trim().is_empty())
        {
            return Err(SnapshotManifestValidationError::BlankRedistribution);
        }
        if self
            .redistribution
            .as_deref()
            .is_some_and(has_surrounding_whitespace)
        {
            return Err(SnapshotManifestValidationError::SurroundedByWhitespace {
                field: "redistribution",
            });
        }
        if self.columns.is_empty() {
            return Err(SnapshotManifestValidationError::MissingColumns);
        }
        if let Some((index, _)) = self
            .columns
            .iter()
            .enumerate()
            .find(|(_, column)| column.trim().is_empty())
        {
            return Err(SnapshotManifestValidationError::BlankColumn { index });
        }

        let mut first_seen_columns = BTreeMap::new();
        for (index, column) in self.columns.iter().enumerate() {
            let name = column.trim();
            if let Some(first_index) = first_seen_columns.insert(name, index) {
                return Err(SnapshotManifestValidationError::DuplicateColumn {
                    first_index,
                    second_index: index,
                    name: name.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Formats the parsed manifest into the compact release-facing summary line.
    pub fn summary_line(&self, label: &str) -> String {
        self.summary_line_with_defaults(label, "unknown", "unknown")
    }

    /// Formats the parsed manifest using explicit default labels for missing provenance.
    pub fn summary_line_with_defaults(
        &self,
        label: &str,
        source_fallback: &'static str,
        coverage_fallback: &'static str,
    ) -> String {
        let title = self
            .title
            .as_deref()
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .unwrap_or("unknown");
        let source = self.source_or(source_fallback);
        let coverage = self.coverage_or(coverage_fallback);
        let columns = self.columns_summary();
        let mut text =
            format!("{label}: {title}; source={source}; coverage={coverage}; columns={columns}");
        if let Some(redistribution) = self
            .redistribution
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            text.push_str("; redistribution=");
            text.push_str(redistribution);
        }
        text
    }
}

impl fmt::Display for SnapshotManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line_with_defaults("Snapshot manifest", "unknown", "unknown"))
    }
}

/// A typed manifest summary for JPL snapshot provenance reporting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotManifestSummary {
    /// Release-facing label for the manifest summary.
    pub label: &'static str,
    /// Parsed manifest to render.
    pub manifest: SnapshotManifest,
    /// Default source label used when the manifest omits one.
    pub source_fallback: &'static str,
    /// Default coverage label used when the manifest omits one.
    pub coverage_fallback: &'static str,
}

/// Structured validation errors for a JPL snapshot manifest summary wrapper.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotManifestSummaryValidationError {
    /// The summary label was blank after trimming.
    BlankLabel,
    /// The summary label carried surrounding whitespace.
    SurroundedByWhitespace { field: &'static str },
    /// The nested manifest failed validation.
    Manifest(SnapshotManifestValidationError),
    /// The parsed provenance field does not match the expected release-facing value.
    MetadataMismatch {
        field: &'static str,
        expected: String,
        found: String,
    },
    /// The parsed column schema does not match the expected release-facing layout.
    ColumnsMismatch { expected: String, found: String },
}

impl fmt::Display for SnapshotManifestSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankLabel => f.write_str("blank label"),
            Self::SurroundedByWhitespace { field } => {
                write!(f, "{field} contains surrounding whitespace")
            }
            Self::Manifest(error) => write!(f, "manifest {error}"),
            Self::MetadataMismatch {
                field,
                expected,
                found,
            } => write!(f, "{field} mismatch: expected {expected} but found {found}"),
            Self::ColumnsMismatch { expected, found } => {
                write!(
                    f,
                    "column schema mismatch: expected {expected} but found {found}"
                )
            }
        }
    }
}

impl std::error::Error for SnapshotManifestSummaryValidationError {}

impl SnapshotManifestSummary {
    /// Validates that the wrapper still matches a usable manifest label and payload.
    pub fn validate(&self) -> Result<(), SnapshotManifestSummaryValidationError> {
        if self.label.trim().is_empty() {
            return Err(SnapshotManifestSummaryValidationError::BlankLabel);
        }
        if has_surrounding_whitespace(self.label) {
            return Err(
                SnapshotManifestSummaryValidationError::SurroundedByWhitespace { field: "label" },
            );
        }
        self.manifest
            .validate()
            .map_err(SnapshotManifestSummaryValidationError::Manifest)
    }

    /// Validates that the wrapper matches the expected provenance and column layout.
    pub fn validate_with_expected_metadata(
        &self,
        expected_title: &str,
        expected_source: &str,
        expected_coverage: &str,
        expected_columns: &[&str],
    ) -> Result<(), SnapshotManifestSummaryValidationError> {
        self.validate()?;

        let Some(title) = self.manifest.title.as_deref() else {
            return Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "title",
                expected: expected_title.to_string(),
                found: String::new(),
            });
        };
        if title != expected_title {
            return Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "title",
                expected: expected_title.to_string(),
                found: title.to_string(),
            });
        }

        let Some(source) = self.manifest.source.as_deref() else {
            return Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "source",
                expected: expected_source.to_string(),
                found: String::new(),
            });
        };
        if source != expected_source {
            return Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "source",
                expected: expected_source.to_string(),
                found: source.to_string(),
            });
        }

        let Some(coverage) = self.manifest.coverage.as_deref() else {
            return Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "coverage",
                expected: expected_coverage.to_string(),
                found: String::new(),
            });
        };
        if coverage != expected_coverage {
            return Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "coverage",
                expected: expected_coverage.to_string(),
                found: coverage.to_string(),
            });
        }

        if !self
            .manifest
            .columns
            .iter()
            .map(String::as_str)
            .eq(expected_columns.iter().copied())
        {
            return Err(SnapshotManifestSummaryValidationError::ColumnsMismatch {
                expected: expected_columns.join(", "),
                found: self.manifest.columns.join(", "),
            });
        }

        Ok(())
    }

    /// Validates that the wrapper matches the expected provenance and redistribution posture.
    pub fn validate_with_expected_metadata_and_redistribution(
        &self,
        expected_title: &str,
        expected_source: &str,
        expected_coverage: &str,
        expected_redistribution: &str,
        expected_columns: &[&str],
    ) -> Result<(), SnapshotManifestSummaryValidationError> {
        self.validate_with_expected_metadata(
            expected_title,
            expected_source,
            expected_coverage,
            expected_columns,
        )?;

        let Some(redistribution) = self.manifest.redistribution.as_deref() else {
            return Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "redistribution",
                expected: expected_redistribution.to_string(),
                found: String::new(),
            });
        };
        if redistribution != expected_redistribution {
            return Err(SnapshotManifestSummaryValidationError::MetadataMismatch {
                field: "redistribution",
                expected: expected_redistribution.to_string(),
                found: redistribution.to_string(),
            });
        }

        Ok(())
    }

    /// Validates that the wrapper matches the expected column layout.
    pub fn validate_with_expected_columns(
        &self,
        expected_columns: &[&str],
    ) -> Result<(), SnapshotManifestSummaryValidationError> {
        self.validate()?;

        if !self
            .manifest
            .columns
            .iter()
            .map(String::as_str)
            .eq(expected_columns.iter().copied())
        {
            return Err(SnapshotManifestSummaryValidationError::ColumnsMismatch {
                expected: expected_columns.join(", "),
                found: self.manifest.columns.join(", "),
            });
        }

        Ok(())
    }

    /// Returns the compact release-facing summary line for the manifest wrapper.
    pub fn summary_line(&self) -> String {
        self.manifest.summary_line_with_defaults(
            self.label,
            self.source_fallback,
            self.coverage_fallback,
        )
    }

    /// Returns the validated compact release-facing summary line for the manifest wrapper.
    pub fn validated_summary_line(&self) -> Result<String, SnapshotManifestSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns the validated summary line after checking a specific column layout.
    pub fn validated_summary_line_with_expected_columns(
        &self,
        expected_columns: &[&str],
    ) -> Result<String, SnapshotManifestSummaryValidationError> {
        self.validate_with_expected_columns(expected_columns)?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for SnapshotManifestSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Parsed manifest and row data for a JPL-style snapshot corpus.
///
/// This bundles the public header metadata with the CSV rows so callers can
/// ingest or reproduce broader source corpora from arbitrary checked text.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotCorpus {
    /// Parsed header metadata from the snapshot source.
    pub manifest: SnapshotManifest,
    /// Parsed row data from the snapshot source.
    pub entries: Vec<SnapshotEntry>,
}

impl SnapshotCorpus {
    /// Returns the parsed manifest and row data as owned parts.
    pub fn into_parts(self) -> (SnapshotManifest, Vec<SnapshotEntry>) {
        (self.manifest, self.entries)
    }
}

/// Separate manifest and row inputs for a JPL-style snapshot corpus.
///
/// This helper lets callers keep public provenance metadata and row tables in
/// distinct checked inputs while still using the same pure-Rust parser path.
#[derive(Clone, Copy, Debug)]
pub struct SnapshotCorpusSources<'a> {
    /// Text containing the manifest/header comments.
    pub manifest: &'a str,
    /// Text containing the row CSV data.
    pub entries: &'a str,
}

impl SnapshotCorpusSources<'_> {
    /// Parses the split corpus inputs into a single typed corpus.
    pub fn parse(self) -> Result<SnapshotCorpus, SnapshotLoadError> {
        parse_snapshot_corpus_from_sources(self.manifest, self.entries)
    }
}

/// Separate manifest and row file inputs for a JPL-style snapshot corpus.
#[derive(Clone, Debug)]
pub struct SnapshotCorpusPathSources {
    /// Path to the manifest/header comments.
    pub manifest: PathBuf,
    /// Path to the row CSV data.
    pub entries: PathBuf,
}

impl SnapshotCorpusPathSources {
    /// Loads and parses the split corpus inputs from disk into a single typed corpus.
    pub fn load(self) -> Result<SnapshotCorpus, SnapshotCorpusLoadError> {
        load_snapshot_corpus_from_paths(&self.manifest, &self.entries)
    }
}

/// Errors that can occur while loading split corpus inputs from disk.
#[derive(Debug)]
pub enum SnapshotCorpusLoadError {
    /// The manifest file could not be read.
    ManifestIo {
        /// Path that was attempted.
        path: PathBuf,
        /// The underlying I/O error.
        error: std::io::Error,
    },
    /// The row file could not be read.
    EntriesIo {
        /// Path that was attempted.
        path: PathBuf,
        /// The underlying I/O error.
        error: std::io::Error,
    },
    /// The row file parsed with a snapshot-row error.
    EntriesParse {
        /// Path that was attempted.
        path: PathBuf,
        /// The underlying parse error.
        error: SnapshotLoadError,
    },
}

impl fmt::Display for SnapshotCorpusLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ManifestIo { path, error } => write!(
                f,
                "failed to read snapshot manifest at {}: {}",
                path.display(),
                error
            ),
            Self::EntriesIo { path, error } => write!(
                f,
                "failed to read snapshot rows at {}: {}",
                path.display(),
                error
            ),
            Self::EntriesParse { path, error } => write!(
                f,
                "failed to parse snapshot rows at {}: {}",
                path.display(),
                error
            ),
        }
    }
}

impl std::error::Error for SnapshotCorpusLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ManifestIo { error, .. } => Some(error),
            Self::EntriesIo { error, .. } => Some(error),
            Self::EntriesParse { error, .. } => Some(error),
        }
    }
}

/// Loads and parses a JPL-style snapshot corpus from split manifest and row files.
pub fn load_snapshot_corpus_from_paths(
    manifest_path: impl AsRef<Path>,
    entries_path: impl AsRef<Path>,
) -> Result<SnapshotCorpus, SnapshotCorpusLoadError> {
    let manifest_path = manifest_path.as_ref();
    let entries_path = entries_path.as_ref();

    let manifest =
        fs::read_to_string(manifest_path).map_err(|error| SnapshotCorpusLoadError::ManifestIo {
            path: manifest_path.to_path_buf(),
            error,
        })?;
    let entries =
        fs::read_to_string(entries_path).map_err(|error| SnapshotCorpusLoadError::EntriesIo {
            path: entries_path.to_path_buf(),
            error,
        })?;

    parse_snapshot_corpus_from_sources(&manifest, &entries).map_err(|error| {
        SnapshotCorpusLoadError::EntriesParse {
            path: entries_path.to_path_buf(),
            error,
        }
    })
}

pub(crate) fn has_surrounding_whitespace(value: &str) -> bool {
    value.trim() != value || value.contains('\n') || value.contains('\r')
}

/// Parses the header block from a checked-in JPL-style snapshot source.
///
/// This is intentionally small and deterministic so public CSV fixtures or
/// public-data derivatives can reuse the same pure-Rust header reader.
pub fn parse_snapshot_manifest(source: &str) -> SnapshotManifest {
    let mut manifest = SnapshotManifest::default();

    for line in source.lines() {
        let trimmed = line.trim();
        let Some(comment) = trimmed.strip_prefix('#') else {
            continue;
        };
        let comment = comment.trim();
        if let Some(value) = comment.strip_prefix("Source:") {
            manifest.source = Some(value.trim().to_string());
        } else if let Some(value) = comment.strip_prefix("Coverage:") {
            manifest.coverage = Some(value.trim().to_string());
        } else if let Some(value) = comment.strip_prefix("Redistribution:") {
            manifest.redistribution = Some(value.trim().to_string());
        } else if let Some(value) = comment.strip_prefix("Columns:") {
            manifest.columns = value
                .split(',')
                .map(|column| column.trim().to_string())
                .collect();
        } else if manifest.title.is_none() && !comment.is_empty() {
            manifest.title = Some(comment.to_string());
        }
    }

    manifest
}

/// Structured validation errors for a checked-in snapshot manifest header block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SnapshotManifestHeaderStructureError {
    /// The manifest comment block contained an unexpected number of non-empty lines.
    CommentCountMismatch { expected: usize, found: usize },
    /// A specific manifest comment line drifted from the canonical header structure.
    CommentMismatch {
        index: usize,
        field: &'static str,
        expected: String,
        found: String,
    },
}

impl fmt::Display for SnapshotManifestHeaderStructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommentCountMismatch { expected, found } => write!(
                f,
                "unexpected manifest comment count: expected {expected}, found {found}"
            ),
            Self::CommentMismatch {
                index,
                field,
                expected,
                found,
            } => write!(
                f,
                "manifest comment {index} ({field}) mismatch: expected {expected} but found {found}"
            ),
        }
    }
}

impl std::error::Error for SnapshotManifestHeaderStructureError {}

pub(crate) fn validate_snapshot_manifest_header_structure(
    source: &str,
    expected_title: &str,
    expected_source: &str,
    expected_coverage: &str,
    expected_redistribution: Option<&str>,
    expected_columns: &[&str],
) -> Result<(), SnapshotManifestHeaderStructureError> {
    let mut expected_comments = vec![
        expected_title.to_string(),
        format!("Source: {expected_source}"),
        format!("Coverage: {expected_coverage}"),
    ];
    if let Some(expected_redistribution) = expected_redistribution {
        expected_comments.push(format!("Redistribution: {expected_redistribution}"));
    }
    expected_comments.push(format!("Columns: {}", expected_columns.join(",")));
    let comments = source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let comment = trimmed.strip_prefix('#')?.trim();
            if comment.is_empty() {
                None
            } else {
                Some(comment.to_string())
            }
        })
        .collect::<Vec<_>>();

    if comments.len() != expected_comments.len() {
        return Err(SnapshotManifestHeaderStructureError::CommentCountMismatch {
            expected: expected_comments.len(),
            found: comments.len(),
        });
    }

    for (index, (found, expected)) in comments.iter().zip(expected_comments.iter()).enumerate() {
        if found != expected {
            return Err(SnapshotManifestHeaderStructureError::CommentMismatch {
                index,
                field: match index {
                    0 => "title",
                    1 => "source",
                    2 => "coverage",
                    3 => "redistribution",
                    _ => "columns",
                },
                expected: expected.clone(),
                found: found.clone(),
            });
        }
    }

    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SnapshotManifestFootprintValidationError {
    MissingEntries {
        label: &'static str,
    },
    RowCountMismatch {
        label: &'static str,
        expected: usize,
        found: usize,
    },
    BodyCountMismatch {
        label: &'static str,
        expected: usize,
        found: usize,
    },
    EpochCountMismatch {
        label: &'static str,
        expected: usize,
        found: usize,
    },
}

impl fmt::Display for SnapshotManifestFootprintValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEntries { label } => {
                write!(f, "{label} entries are unavailable")
            }
            Self::RowCountMismatch {
                label,
                expected,
                found,
            } => write!(
                f,
                "{label} row count mismatch: expected {expected}, found {found}"
            ),
            Self::BodyCountMismatch {
                label,
                expected,
                found,
            } => write!(
                f,
                "{label} body count mismatch: expected {expected}, found {found}"
            ),
            Self::EpochCountMismatch {
                label,
                expected,
                found,
            } => write!(
                f,
                "{label} epoch count mismatch: expected {expected}, found {found}"
            ),
        }
    }
}

impl std::error::Error for SnapshotManifestFootprintValidationError {}

pub(crate) fn validate_snapshot_manifest_footprint(
    label: &'static str,
    entries: Option<&[SnapshotEntry]>,
    expected_row_count: usize,
    expected_body_count: usize,
    expected_epoch_count: usize,
) -> Result<(), SnapshotManifestFootprintValidationError> {
    let Some(entries) = entries else {
        return Err(SnapshotManifestFootprintValidationError::MissingEntries { label });
    };

    let row_count = entries.len();
    if row_count != expected_row_count {
        return Err(SnapshotManifestFootprintValidationError::RowCountMismatch {
            label,
            expected: expected_row_count,
            found: row_count,
        });
    }

    let body_count = entries
        .iter()
        .map(|entry| entry.body.to_string())
        .collect::<BTreeSet<_>>()
        .len();
    if body_count != expected_body_count {
        return Err(
            SnapshotManifestFootprintValidationError::BodyCountMismatch {
                label,
                expected: expected_body_count,
                found: body_count,
            },
        );
    }

    let epoch_count = entries
        .iter()
        .map(|entry| entry.epoch.julian_day.days().to_bits())
        .collect::<BTreeSet<_>>()
        .len();
    if epoch_count != expected_epoch_count {
        return Err(
            SnapshotManifestFootprintValidationError::EpochCountMismatch {
                label,
                expected: expected_epoch_count,
                found: epoch_count,
            },
        );
    }

    Ok(())
}

/// Returns the parsed manifest for the checked-in reference snapshot.
pub fn reference_snapshot_manifest() -> &'static SnapshotManifest {
    static MANIFEST: OnceLock<SnapshotManifest> = OnceLock::new();
    MANIFEST.get_or_init(|| {
        parse_snapshot_manifest(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/reference_snapshot.csv"
        )))
    })
}

/// Returns the parsed manifest for the checked-in hold-out snapshot.
pub fn independent_holdout_snapshot_manifest() -> &'static SnapshotManifest {
    static MANIFEST: OnceLock<SnapshotManifest> = OnceLock::new();
    MANIFEST.get_or_init(|| {
        parse_snapshot_manifest(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/independent_holdout_snapshot.csv"
        )))
    })
}

/// One parsed record from the reference fixture.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotEntry {
    /// The body covered by the entry.
    pub body: pleiades_backend::CelestialBody,
    /// The epoch covered by the entry.
    pub epoch: Instant,
    /// Cartesian X position in kilometers.
    pub x_km: f64,
    /// Cartesian Y position in kilometers.
    pub y_km: f64,
    /// Cartesian Z position in kilometers.
    pub z_km: f64,
}

impl SnapshotEntry {
    pub(crate) fn ecliptic(&self) -> EclipticCoordinates {
        let radius_km =
            (self.x_km * self.x_km + self.y_km * self.y_km + self.z_km * self.z_km).sqrt();
        let longitude = Longitude::from_degrees(self.y_km.atan2(self.x_km).to_degrees());
        let latitude =
            Latitude::from_degrees((self.z_km / radius_km).clamp(-1.0, 1.0).asin().to_degrees());
        EclipticCoordinates::new(longitude, latitude, Some(radius_km / AU_IN_KM))
    }

    fn interpolate_linear(before: &Self, after: &Self, epoch_jd: f64) -> Self {
        let span_days = after.epoch.julian_day.days() - before.epoch.julian_day.days();
        let fraction = (epoch_jd - before.epoch.julian_day.days()) / span_days;
        Self {
            body: before.body.clone(),
            epoch: Instant::new(JulianDay::from_days(epoch_jd), TimeScale::Tdb),
            x_km: lerp(before.x_km, after.x_km, fraction),
            y_km: lerp(before.y_km, after.y_km, fraction),
            z_km: lerp(before.z_km, after.z_km, fraction),
        }
    }

    pub(crate) fn interpolate_quadratic(a: &Self, b: &Self, c: &Self, epoch_jd: f64) -> Self {
        let xs = [
            a.epoch.julian_day.days(),
            b.epoch.julian_day.days(),
            c.epoch.julian_day.days(),
        ];
        Self {
            body: a.body.clone(),
            epoch: Instant::new(JulianDay::from_days(epoch_jd), TimeScale::Tdb),
            x_km: lagrange_interpolate_3(epoch_jd, xs, [a.x_km, b.x_km, c.x_km]),
            y_km: lagrange_interpolate_3(epoch_jd, xs, [a.y_km, b.y_km, c.y_km]),
            z_km: lagrange_interpolate_3(epoch_jd, xs, [a.z_km, b.z_km, c.z_km]),
        }
    }

    pub(crate) fn interpolate_cubic(a: &Self, b: &Self, c: &Self, d: &Self, epoch_jd: f64) -> Self {
        let xs = [
            a.epoch.julian_day.days(),
            b.epoch.julian_day.days(),
            c.epoch.julian_day.days(),
            d.epoch.julian_day.days(),
        ];
        Self {
            body: a.body.clone(),
            epoch: Instant::new(JulianDay::from_days(epoch_jd), TimeScale::Tdb),
            x_km: lagrange_interpolate_4(epoch_jd, xs, [a.x_km, b.x_km, c.x_km, d.x_km]),
            y_km: lagrange_interpolate_4(epoch_jd, xs, [a.y_km, b.y_km, c.y_km, d.y_km]),
            z_km: lagrange_interpolate_4(epoch_jd, xs, [a.z_km, b.z_km, c.z_km, d.z_km]),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedFixtureState {
    pub(crate) entry: SnapshotEntry,
    quality: QualityAnnotation,
}

enum SnapshotState {
    Loaded(Vec<SnapshotEntry>),
    Failed(SnapshotLoadError),
}

impl SnapshotState {
    fn entries(&self) -> Option<&[SnapshotEntry]> {
        match self {
            Self::Loaded(entries) => Some(entries.as_slice()),
            Self::Failed(_) => None,
        }
    }

    fn error(&self) -> Option<&SnapshotLoadError> {
        match self {
            Self::Loaded(_) => None,
            Self::Failed(error) => Some(error),
        }
    }
}

/// Structured parser error for checked-in JPL-style snapshot rows.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotLoadError {
    line_number: usize,
    kind: SnapshotLoadErrorKind,
}

impl SnapshotLoadError {
    fn new(line_number: usize, kind: SnapshotLoadErrorKind) -> Self {
        Self { line_number, kind }
    }

    /// Returns the 1-indexed line number that triggered the parse failure.
    pub const fn line_number(&self) -> usize {
        self.line_number
    }

    /// Returns the structured parse failure kind.
    pub const fn kind(&self) -> &SnapshotLoadErrorKind {
        &self.kind
    }
}

impl fmt::Display for SnapshotLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line_number, self.kind)
    }
}

impl std::error::Error for SnapshotLoadError {}

/// Error kinds produced while parsing checked-in JPL-style snapshot rows.
#[derive(Clone, Debug, PartialEq)]
pub enum SnapshotLoadErrorKind {
    MissingColumn {
        column: &'static str,
    },
    UnexpectedExtraColumns,
    BlankBody,
    UnsupportedBody {
        body: String,
    },
    InvalidNumber {
        column: &'static str,
        value: String,
    },
    DuplicateEntry {
        body: String,
        epoch: Instant,
        first_line: usize,
    },
}

impl SnapshotLoadErrorKind {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingColumn { .. } => "missing column",
            Self::UnexpectedExtraColumns => "unexpected extra columns",
            Self::BlankBody => "blank body",
            Self::UnsupportedBody { .. } => "unsupported body",
            Self::InvalidNumber { .. } => "invalid number",
            Self::DuplicateEntry { .. } => "duplicate entry",
        }
    }
}

impl fmt::Display for SnapshotLoadErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingColumn { column } => write!(f, "missing {column} column"),
            Self::UnexpectedExtraColumns => f.write_str("unexpected extra columns"),
            Self::BlankBody => f.write_str("blank body"),
            Self::UnsupportedBody { body } => write!(f, "unsupported body '{body}'"),
            Self::InvalidNumber { column, value } => {
                write!(f, "invalid {column} value '{value}'")
            }
            Self::DuplicateEntry {
                body,
                epoch,
                first_line,
            } => {
                write!(
                    f,
                    "duplicate row for body '{body}' at {} (first seen at line {first_line})",
                    format_instant(*epoch)
                )
            }
        }
    }
}

fn lerp(start: f64, end: f64, fraction: f64) -> f64 {
    start + (end - start) * fraction
}

fn lagrange_interpolate_3(x: f64, xs: [f64; 3], ys: [f64; 3]) -> f64 {
    let [x0, x1, x2] = xs;
    let [y0, y1, y2] = ys;

    let l0 = (x - x1) * (x - x2) / ((x0 - x1) * (x0 - x2));
    let l1 = (x - x0) * (x - x2) / ((x1 - x0) * (x1 - x2));
    let l2 = (x - x0) * (x - x1) / ((x2 - x0) * (x2 - x1));

    y0 * l0 + y1 * l1 + y2 * l2
}

fn lagrange_interpolate_4(x: f64, xs: [f64; 4], ys: [f64; 4]) -> f64 {
    let [x0, x1, x2, x3] = xs;
    let [y0, y1, y2, y3] = ys;

    let l0 = (x - x1) * (x - x2) * (x - x3) / ((x0 - x1) * (x0 - x2) * (x0 - x3));
    let l1 = (x - x0) * (x - x2) * (x - x3) / ((x1 - x0) * (x1 - x2) * (x1 - x3));
    let l2 = (x - x0) * (x - x1) * (x - x3) / ((x2 - x0) * (x2 - x1) * (x2 - x3));
    let l3 = (x - x0) * (x - x1) * (x - x2) / ((x3 - x0) * (x3 - x1) * (x3 - x2));

    y0 * l0 + y1 * l1 + y2 * l2 + y3 * l3
}

pub(crate) fn interpolate_fixture_state(
    entries: &[SnapshotEntry],
    body: pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Option<SnapshotEntry> {
    let mut body_entries = entries
        .iter()
        .filter(|entry| entry.body == body)
        .collect::<Vec<_>>();

    if body_entries.len() < 3 {
        return None;
    }

    body_entries.sort_by(|left, right| {
        left.epoch
            .julian_day
            .days()
            .partial_cmp(&right.epoch.julian_day.days())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let body_entry_count = body_entries.len();
    let mut ranked = body_entries
        .into_iter()
        .map(|entry| ((entry.epoch.julian_day.days() - epoch_jd).abs(), entry))
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        left.0
            .partial_cmp(&right.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                left.1
                    .epoch
                    .julian_day
                    .days()
                    .partial_cmp(&right.1.epoch.julian_day.days())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let window_size = if body_entry_count >= 4 { 4 } else { 3 };
    let mut selected = ranked
        .into_iter()
        .take(window_size)
        .map(|(_, entry)| entry)
        .collect::<Vec<_>>();

    match selected.len() {
        4 => {
            selected.sort_by(|left, right| {
                left.epoch
                    .julian_day
                    .days()
                    .partial_cmp(&right.epoch.julian_day.days())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            Some(SnapshotEntry::interpolate_cubic(
                selected[0],
                selected[1],
                selected[2],
                selected[3],
                epoch_jd,
            ))
        }
        3 => {
            selected.sort_by(|left, right| {
                left.epoch
                    .julian_day
                    .days()
                    .partial_cmp(&right.epoch.julian_day.days())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            Some(SnapshotEntry::interpolate_quadratic(
                selected[0],
                selected[1],
                selected[2],
                epoch_jd,
            ))
        }
        _ => None,
    }
}

pub(crate) fn angular_degrees_delta(left: f64, right: f64) -> f64 {
    let delta = (left - right + 180.0).rem_euclid(360.0) - 180.0;
    delta.abs()
}

fn snapshot_state() -> &'static SnapshotState {
    static STATE: OnceLock<SnapshotState> = OnceLock::new();
    STATE.get_or_init(|| match load_snapshot() {
        Ok(entries) => SnapshotState::Loaded(entries),
        Err(error) => SnapshotState::Failed(error),
    })
}

pub(crate) fn snapshot_entries() -> Option<&'static [SnapshotEntry]> {
    snapshot_state().entries()
}

fn snapshot_error() -> Option<&'static SnapshotLoadError> {
    snapshot_state().error()
}

pub(crate) fn snapshot_bodies() -> &'static [pleiades_backend::CelestialBody] {
    static BODIES: OnceLock<Vec<pleiades_backend::CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            let mut bodies = Vec::new();
            if let Some(entries) = snapshot_entries() {
                for entry in entries {
                    if !bodies.contains(&entry.body) {
                        bodies.push(entry.body.clone());
                    }
                }
            }
            bodies
        })
        .as_slice()
}

pub(crate) const REFERENCE_SNAPSHOT_1749_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_360_233.5;
pub(crate) const REFERENCE_SNAPSHOT_1500_SELECTED_BODY_BOUNDARY_EPOCH_JD: f64 = 2_268_932.5;
pub(crate) const REFERENCE_SNAPSHOT_1600_SELECTED_BODY_BOUNDARY_EPOCH_JD: f64 = 2_305_457.5;
pub(crate) const REFERENCE_SNAPSHOT_1750_SELECTED_BODY_BOUNDARY_EPOCH_JD: f64 = 2_360_234.5;
pub(crate) const REFERENCE_SNAPSHOT_1900_SELECTED_BODY_BOUNDARY_EPOCH_JD: f64 = 2_415_020.5;
pub(crate) const REFERENCE_SNAPSHOT_2200_SELECTED_BODY_BOUNDARY_EPOCH_JD: f64 = 2_524_593.5;
pub(crate) const REFERENCE_SNAPSHOT_2500_SELECTED_BODY_BOUNDARY_EPOCH_JD: f64 = 2_634_167.0;
pub(crate) const REFERENCE_SNAPSHOT_REFERENCE_ONLY_EPOCH_JD: f64 = 2_378_498.5;
pub(crate) const REFERENCE_SNAPSHOT_1800_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_378_499.0;
pub(crate) const REFERENCE_SNAPSHOT_2400000_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_400_000.0;
pub(crate) const REFERENCE_SNAPSHOT_2451545_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_451_545.0;
pub(crate) const REFERENCE_SNAPSHOT_2500_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_500_000.0;
pub(crate) const REFERENCE_SNAPSHOT_2600000_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_600_000.0;
pub(crate) const REFERENCE_SNAPSHOT_2451910_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_451_910.5;
pub(crate) const REFERENCE_SNAPSHOT_2451911_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_451_911.5;
pub(crate) const REFERENCE_SNAPSHOT_2451912_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_451_912.5;
pub(crate) const REFERENCE_SNAPSHOT_2451913_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_451_913.5;
pub(crate) const REFERENCE_SNAPSHOT_2451914_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_451_914.5;
pub(crate) const REFERENCE_SNAPSHOT_2451915_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_451_915.5;
pub(crate) const REFERENCE_SNAPSHOT_2451917_MAJOR_BODY_BRIDGE_EPOCH_JD: f64 = 2_451_917.0;
pub(crate) const REFERENCE_SNAPSHOT_2451917_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_451_917.5;
pub(crate) const REFERENCE_SNAPSHOT_2451919_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_451_919.5;
pub(crate) const REFERENCE_SNAPSHOT_2451916_MAJOR_BODY_INTERIOR_EPOCH_JD: f64 = 2_451_916.0;
pub(crate) const REFERENCE_SNAPSHOT_2451920_MAJOR_BODY_INTERIOR_EPOCH_JD: f64 = 2_451_920.5;
pub(crate) const REFERENCE_SNAPSHOT_2453000_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_453_000.5;
pub(crate) const REFERENCE_SNAPSHOT_2500000_MAJOR_BODY_BOUNDARY_EPOCH_JD: f64 = 2_500_000.0;
const REFERENCE_SNAPSHOT_BOUNDARY_ONLY_EPOCH_JD: f64 = 2_451_917.5;

fn is_reference_snapshot_only_epoch(epoch: f64) -> bool {
    matches!(
        epoch,
        x if x == REFERENCE_SNAPSHOT_REFERENCE_ONLY_EPOCH_JD
            || x == REFERENCE_SNAPSHOT_BOUNDARY_ONLY_EPOCH_JD
    )
}

pub(crate) fn comparison_snapshot_entries() -> &'static [SnapshotEntry] {
    static SNAPSHOT: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    SNAPSHOT
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days() != 2_451_913.5
                        && !is_reference_snapshot_only_epoch(entry.epoch.julian_day.days())
                })
                .cloned()
                .collect()
        })
        .as_slice()
}

fn independent_holdout_state() -> &'static SnapshotState {
    static STATE: OnceLock<SnapshotState> = OnceLock::new();
    STATE.get_or_init(|| {
        match load_snapshot_from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/independent_holdout_snapshot.csv"
        ))) {
            Ok(entries) => SnapshotState::Loaded(entries),
            Err(error) => SnapshotState::Failed(error),
        }
    })
}

/// Returns the parsed independent hold-out fixture entries.
///
/// The entries preserve the checked-in order from the derivative CSV so
/// downstream validation and reproducibility tooling can rebuild the exact
/// hold-out request corpus without re-parsing the fixture.
pub fn independent_holdout_snapshot_entries() -> Option<&'static [SnapshotEntry]> {
    independent_holdout_state().entries()
}

/// Returns the independent hold-out request corpus in the requested frame.
///
/// The requests preserve the checked-in row order and the stored epochs from
/// the derivative CSV. Callers can reuse this corpus for exact batch checks or
/// retag the returned instants with a different time-scale policy if needed.
pub fn independent_holdout_snapshot_requests(
    frame: CoordinateFrame,
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_entries().map(|entries| {
        entries
            .iter()
            .map(|entry| EphemerisRequest {
                body: entry.body.clone(),
                instant: entry.epoch,
                observer: None,
                frame,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// This is a compatibility alias for [`independent_holdout_snapshot_requests`].
#[doc(alias = "independent_holdout_snapshot_requests")]
pub fn independent_holdout_snapshot_request_corpus(
    frame: CoordinateFrame,
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_requests(frame)
}

/// Returns the ecliptic independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_requests`].
#[doc(alias = "independent_holdout_snapshot_requests")]
pub fn independent_holdout_snapshot_ecliptic_request_corpus() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_requests(CoordinateFrame::Ecliptic)
}

/// Returns the ecliptic independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_ecliptic_request_corpus`].
#[doc(alias = "independent_holdout_snapshot_ecliptic_request_corpus")]
pub fn independent_holdout_snapshot_ecliptic_requests() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_ecliptic_request_corpus()
}

/// Returns the equatorial independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_requests`].
#[doc(alias = "independent_holdout_snapshot_requests")]
pub fn independent_holdout_snapshot_equatorial_parity_requests() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_requests(CoordinateFrame::Equatorial)
}

/// Returns the equatorial independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_equatorial_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_equatorial_parity_requests")]
pub fn independent_holdout_snapshot_equatorial_batch_parity_requests(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_equatorial_parity_requests()
}

/// Returns the equatorial independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_equatorial_batch_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_equatorial_batch_parity_requests")]
pub fn independent_holdout_snapshot_equatorial_batch_parity_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_equatorial_batch_parity_requests()
}

/// Returns the equatorial independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_equatorial_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_equatorial_parity_requests")]
pub fn independent_holdout_snapshot_equatorial_request_corpus() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_equatorial_parity_requests()
}

/// Returns the equatorial independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_equatorial_request_corpus`].
#[doc(alias = "independent_holdout_snapshot_equatorial_request_corpus")]
pub fn independent_holdout_snapshot_equatorial_requests() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_equatorial_request_corpus()
}

/// Returns the equatorial independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`independent_holdout_snapshot_equatorial_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_equatorial_parity_requests")]
pub fn independent_holdout_snapshot_equatorial_parity_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_equatorial_parity_requests()
}

/// Returns the mixed-scale independent hold-out request corpus used by batch parity checks.
///
/// The requests preserve the checked-in row order, alternate TT and TDB labels
/// per row, and keep the ecliptic frame so downstream tooling can reuse the
/// exact mixed-scale batch slice without reconstructing it from the snapshot metadata.
pub fn independent_holdout_snapshot_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_entries().map(|entries| {
        entries
            .iter()
            .enumerate()
            .map(|(index, entry)| EphemerisRequest {
                body: entry.body.clone(),
                instant: Instant::new(
                    entry.epoch.julian_day,
                    if index % 2 == 0 {
                        TimeScale::Tt
                    } else {
                        TimeScale::Tdb
                    },
                ),
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// Returns the mixed-scale independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`independent_holdout_snapshot_batch_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_batch_parity_requests")]
pub fn independent_holdout_snapshot_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_batch_parity_requests()
}

/// Returns the mixed TT/TDB independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`independent_holdout_snapshot_batch_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_batch_parity_requests")]
pub fn independent_holdout_snapshot_mixed_time_scale_batch_parity_requests(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_batch_parity_requests()
}

/// Returns the mixed TT/TDB independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`independent_holdout_snapshot_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_mixed_time_scale_batch_parity_requests")]
pub fn independent_holdout_snapshot_mixed_time_scale_batch_parity_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_mixed_time_scale_batch_parity_requests()
}

/// Returns the mixed TT/TDB independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`independent_holdout_snapshot_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_mixed_time_scale_batch_parity_requests")]
pub fn independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_mixed_time_scale_batch_parity_requests()
}

/// Returns the mixed TT/TDB independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests")]
pub fn independent_holdout_snapshot_mixed_tt_tdb_batch_parity_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests()
}

/// Returns the mixed TT/TDB independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`independent_holdout_snapshot_mixed_time_scale_request_corpus`].
#[doc(alias = "independent_holdout_snapshot_mixed_time_scale_request_corpus")]
pub fn independent_holdout_snapshot_mixed_tt_tdb_request_corpus() -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests()
}

/// Returns the mixed TT/TDB independent hold-out request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests`].
#[doc(alias = "independent_holdout_snapshot_mixed_tt_tdb_batch_parity_requests")]
pub fn independent_holdout_snapshot_mixed_time_scale_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    independent_holdout_snapshot_mixed_time_scale_batch_parity_requests()
}

pub(crate) fn independent_holdout_bodies() -> &'static [pleiades_backend::CelestialBody] {
    static BODIES: OnceLock<Vec<pleiades_backend::CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            let mut bodies = Vec::new();
            if let Some(entries) = independent_holdout_snapshot_entries() {
                for entry in entries {
                    if !bodies.contains(&entry.body) {
                        bodies.push(entry.body.clone());
                    }
                }
            }
            bodies
        })
        .as_slice()
}

pub(crate) fn independent_holdout_snapshot_error() -> Option<&'static SnapshotLoadError> {
    independent_holdout_state().error()
}

pub(crate) fn comparison_body_list() -> &'static [pleiades_backend::CelestialBody] {
    static BODIES: OnceLock<Vec<pleiades_backend::CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            let mut bodies = Vec::new();
            for entry in comparison_snapshot_entries() {
                if !bodies.contains(&entry.body) {
                    bodies.push(entry.body.clone());
                }
            }
            bodies
        })
        .as_slice()
}

pub(crate) fn reference_asteroid_list() -> &'static [pleiades_backend::CelestialBody] {
    static BODIES: OnceLock<Vec<pleiades_backend::CelestialBody>> = OnceLock::new();
    BODIES
        .get_or_init(|| {
            let mut bodies = Vec::new();
            for entry in snapshot_entries().into_iter().flatten() {
                if is_reference_asteroid(&entry.body) && !bodies.contains(&entry.body) {
                    bodies.push(entry.body.clone());
                }
            }
            bodies
        })
        .as_slice()
}

pub(crate) fn reference_asteroid_requests_with_frame_selector(
    frame_for_index: impl Fn(usize) -> CoordinateFrame,
) -> Option<Vec<EphemerisRequest>> {
    let evidence = reference_asteroid_evidence();
    if evidence.is_empty() {
        return None;
    }

    Some(
        evidence
            .iter()
            .enumerate()
            .map(|(index, sample)| EphemerisRequest {
                body: sample.body.clone(),
                instant: sample.epoch,
                observer: None,
                frame: frame_for_index(index),
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect(),
    )
}

pub(crate) fn reference_asteroid_evidence_list() -> &'static [ReferenceAsteroidEvidence] {
    static EVIDENCE: OnceLock<Vec<ReferenceAsteroidEvidence>> = OnceLock::new();
    EVIDENCE
        .get_or_init(|| {
            let mut evidence = Vec::new();
            let Some(entries) = snapshot_entries() else {
                return evidence;
            };

            for body in reference_asteroid_list() {
                if let Some(entry) = entries.iter().find(|entry| {
                    &entry.body == body && entry.epoch.julian_day.days() == REFERENCE_EPOCH_JD
                }) {
                    let ecliptic = entry.ecliptic();
                    evidence.push(ReferenceAsteroidEvidence {
                        body: body.clone(),
                        epoch: entry.epoch,
                        longitude_deg: ecliptic.longitude.degrees(),
                        latitude_deg: ecliptic.latitude.degrees(),
                        distance_au: ecliptic.distance_au.unwrap_or_default(),
                    });
                }
            }

            evidence
        })
        .as_slice()
}

pub(crate) fn reference_asteroid_equatorial_evidence_list(
) -> &'static [ReferenceAsteroidEquatorialEvidence] {
    static EVIDENCE: OnceLock<Vec<ReferenceAsteroidEquatorialEvidence>> = OnceLock::new();
    EVIDENCE
        .get_or_init(|| {
            reference_asteroid_evidence()
                .iter()
                .map(|sample| {
                    let ecliptic = EclipticCoordinates::new(
                        Longitude::from_degrees(sample.longitude_deg),
                        Latitude::from_degrees(sample.latitude_deg),
                        Some(sample.distance_au),
                    );
                    ReferenceAsteroidEquatorialEvidence {
                        body: sample.body.clone(),
                        epoch: sample.epoch,
                        equatorial: ecliptic.to_equatorial(sample.epoch.mean_obliquity()),
                    }
                })
                .collect()
        })
        .as_slice()
}

pub(crate) fn interpolation_quality_sample_list() -> &'static [InterpolationQualitySample] {
    static SAMPLES: OnceLock<Vec<InterpolationQualitySample>> = OnceLock::new();
    SAMPLES
        .get_or_init(|| {
            let mut samples = Vec::new();
            let Some(entries) = snapshot_entries() else {
                return samples;
            };

            let entries = entries
                .iter()
                .filter(|entry| !is_reference_snapshot_only_epoch(entry.epoch.julian_day.days()))
                .cloned()
                .collect::<Vec<_>>();

            push_interpolation_quality_samples_for_bodies(
                &mut samples,
                &entries,
                comparison_body_list(),
            );
            push_interpolation_quality_samples_for_bodies(
                &mut samples,
                &entries,
                reference_asteroid_list(),
            );

            samples
        })
        .as_slice()
}

fn push_interpolation_quality_samples_for_bodies(
    samples: &mut Vec<InterpolationQualitySample>,
    entries: &[SnapshotEntry],
    bodies: &[pleiades_backend::CelestialBody],
) {
    for body in bodies {
        let mut body_entries = entries
            .iter()
            .filter(|entry| &entry.body == body)
            .collect::<Vec<_>>();
        body_entries.sort_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .partial_cmp(&right.epoch.julian_day.days())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for window in body_entries.windows(3) {
            let before = window[0];
            let exact = window[1];
            let after = window[2];
            let epoch_jd = exact.epoch.julian_day.days();
            let leave_one_out_entries = entries
                .iter()
                .filter(|entry| {
                    entry.body != exact.body || entry.epoch.julian_day.days() != epoch_jd
                })
                .cloned()
                .collect::<Vec<_>>();
            let interpolation_kind = match body_entries.len().saturating_sub(1) {
                0..=2 => InterpolationQualityKind::Linear,
                3 => InterpolationQualityKind::Quadratic,
                _ => InterpolationQualityKind::Cubic,
            };
            let interpolated = resolve_fixture_state_from_entries(
                &leave_one_out_entries,
                exact.body.clone(),
                epoch_jd,
            )
            .expect("held-out sample should still interpolate")
            .entry;
            let exact_ecliptic = exact.ecliptic();
            let interpolated_ecliptic = interpolated.ecliptic();
            let exact_distance = exact_ecliptic.distance_au.unwrap_or_default();
            let interpolated_distance = interpolated_ecliptic.distance_au.unwrap_or_default();

            samples.push(InterpolationQualitySample {
                body: exact.body.clone(),
                epoch: exact.epoch,
                interpolation_kind,
                bracket_span_days: after.epoch.julian_day.days() - before.epoch.julian_day.days(),
                longitude_error_deg: angular_degrees_delta(
                    exact_ecliptic.longitude.degrees(),
                    interpolated_ecliptic.longitude.degrees(),
                ),
                latitude_error_deg: (exact_ecliptic.latitude.degrees()
                    - interpolated_ecliptic.latitude.degrees())
                .abs(),
                distance_error_au: (exact_distance - interpolated_distance).abs(),
            });
        }
    }
}

pub(crate) fn snapshot_instants() -> &'static [Instant] {
    static INSTANTS: OnceLock<Vec<Instant>> = OnceLock::new();
    INSTANTS
        .get_or_init(|| {
            let mut instants = Vec::new();
            if let Some(entries) = snapshot_entries() {
                for entry in entries {
                    if !instants.contains(&entry.epoch) {
                        instants.push(entry.epoch);
                    }
                }
            }
            instants
        })
        .as_slice()
}

pub(crate) fn resolve_fixture_state(
    body: pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Result<ResolvedFixtureState, EphemerisError> {
    let Some(entries) = snapshot_entries() else {
        return Err(EphemerisError::new(
            EphemerisErrorKind::MissingDataset,
            "the JPL fixture corpus is unavailable",
        ));
    };

    resolve_fixture_state_from_entries(entries, body, epoch_jd)
}

fn resolve_fixture_state_from_entries(
    entries: &[SnapshotEntry],
    body: pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Result<ResolvedFixtureState, EphemerisError> {
    let mut exact = None;
    let mut before = None;
    let mut after = None;
    let mut body_seen = false;

    for entry in entries.iter().filter(|entry| entry.body == body) {
        body_seen = true;
        let entry_jd = entry.epoch.julian_day.days();
        if entry_jd == epoch_jd {
            exact = Some(entry);
            break;
        }
        if entry_jd < epoch_jd
            && before.is_none_or(|candidate: &SnapshotEntry| {
                entry_jd > candidate.epoch.julian_day.days()
            })
        {
            before = Some(entry);
        }
        if entry_jd > epoch_jd
            && after.is_none_or(|candidate: &SnapshotEntry| {
                entry_jd < candidate.epoch.julian_day.days()
            })
        {
            after = Some(entry);
        }
    }

    if let Some(entry) = exact {
        return Ok(ResolvedFixtureState {
            entry: entry.clone(),
            quality: QualityAnnotation::Exact,
        });
    }

    if !body_seen {
        return Err(EphemerisError::new(
            EphemerisErrorKind::UnsupportedBody,
            format!("the JPL fixture corpus does not include {body}"),
        ));
    }

    if before.is_some() && after.is_some() {
        if let Some(entry) = interpolate_fixture_state(entries, body.clone(), epoch_jd) {
            return Ok(ResolvedFixtureState {
                entry,
                quality: QualityAnnotation::Interpolated,
            });
        }
    }

    match (before, after) {
        (Some(before), Some(after)) => Ok(ResolvedFixtureState {
            entry: SnapshotEntry::interpolate_linear(before, after, epoch_jd),
            quality: QualityAnnotation::Interpolated,
        }),
        _ => Err(EphemerisError::new(
            EphemerisErrorKind::OutOfRangeInstant,
            "the requested instant is outside adjacent JPL fixture samples for that body",
        )),
    }
}

fn load_snapshot() -> Result<Vec<SnapshotEntry>, SnapshotLoadError> {
    load_snapshot_from_csv(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/reference_snapshot.csv"
    )))
}

/// Parses a JPL-style snapshot corpus from raw CSV text.
///
/// This is a reusable pure-Rust ingestion entry point for broader public-data
/// derivatives and checked-in corpus fixtures.
pub fn load_snapshot_from_csv(source: &str) -> Result<Vec<SnapshotEntry>, SnapshotLoadError> {
    parse_snapshot_entries(source)
}

/// Backward-compatible alias for [`load_snapshot_from_csv`].
#[doc(alias = "load_snapshot_from_csv")]
pub fn load_snapshot_from_str(source: &str) -> Result<Vec<SnapshotEntry>, SnapshotLoadError> {
    load_snapshot_from_csv(source)
}

/// Parses a JPL-style snapshot corpus into its manifest and row data.
///
/// This combines the pure-Rust header parser with the row parser so callers can
/// ingest a broader checked corpus without reconstructing the split manually.
pub fn parse_snapshot_corpus(source: &str) -> Result<SnapshotCorpus, SnapshotLoadError> {
    Ok(SnapshotCorpus {
        manifest: parse_snapshot_manifest(source),
        entries: parse_snapshot_entries(source)?,
    })
}

/// Parses split manifest and row inputs into a single JPL-style snapshot corpus.
///
/// This is useful when public provenance metadata is stored separately from the
/// row table or when a corpus-generation pipeline emits distinct manifest and
/// data artifacts.
pub fn parse_snapshot_corpus_from_sources(
    manifest_source: &str,
    entries_source: &str,
) -> Result<SnapshotCorpus, SnapshotLoadError> {
    Ok(SnapshotCorpus {
        manifest: parse_snapshot_manifest(manifest_source),
        entries: parse_snapshot_entries(entries_source)?,
    })
}

/// Parses the row section of a checked-in JPL-style snapshot source.
///
/// The reader accepts the same CSV shape used by the bundled reference and
/// hold-out fixtures, including custom `catalog:designation` asteroid labels.
pub fn parse_snapshot_entries(source: &str) -> Result<Vec<SnapshotEntry>, SnapshotLoadError> {
    let mut seen_entries = BTreeMap::new();

    source
        .lines()
        .enumerate()
        .map(|(index, line)| {
            parse_snapshot_line(index + 1, line).map(|entry| entry.map(|entry| (index + 1, entry)))
        })
        .try_fold(Vec::new(), |mut entries, record| {
            if let Some((line_number, entry)) = record? {
                let entry_key = (
                    entry.body.to_string(),
                    entry.epoch.julian_day.days().to_bits(),
                );
                if let Some(first_line) = seen_entries.get(&entry_key).copied() {
                    return Err(SnapshotLoadError::new(
                        line_number,
                        SnapshotLoadErrorKind::DuplicateEntry {
                            body: entry_key.0,
                            epoch: entry.epoch,
                            first_line,
                        },
                    ));
                }
                seen_entries.insert(entry_key, line_number);
                entries.push(entry);
            }
            Ok(entries)
        })
}

fn parse_snapshot_line(
    line_number: usize,
    line: &str,
) -> Result<Option<SnapshotEntry>, SnapshotLoadError> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(None);
    }

    let mut parts = trimmed.split(',').map(str::trim);
    let epoch_jd = next_part(&mut parts, line_number, "epoch")?;
    let body = next_part(&mut parts, line_number, "body")?;
    let x_km = next_part(&mut parts, line_number, "x")?;
    let y_km = next_part(&mut parts, line_number, "y")?;
    let z_km = next_part(&mut parts, line_number, "z")?;

    if parts.next().is_some() {
        return Err(SnapshotLoadError::new(
            line_number,
            SnapshotLoadErrorKind::UnexpectedExtraColumns,
        ));
    }

    Ok(Some(SnapshotEntry {
        body: parse_body(body, line_number)?,
        epoch: Instant::new(
            JulianDay::from_days(parse_f64(epoch_jd, line_number, "epoch_jd")?),
            TimeScale::Tdb,
        ),
        x_km: parse_f64(x_km, line_number, "x_km")?,
        y_km: parse_f64(y_km, line_number, "y_km")?,
        z_km: parse_f64(z_km, line_number, "z_km")?,
    }))
}

fn next_part<'a>(
    parts: &mut impl Iterator<Item = &'a str>,
    line_number: usize,
    column: &'static str,
) -> Result<&'a str, SnapshotLoadError> {
    parts.next().ok_or_else(|| {
        SnapshotLoadError::new(line_number, SnapshotLoadErrorKind::MissingColumn { column })
    })
}

fn parse_body(
    body: &str,
    line_number: usize,
) -> Result<pleiades_backend::CelestialBody, SnapshotLoadError> {
    if body.is_empty() {
        return Err(SnapshotLoadError::new(
            line_number,
            SnapshotLoadErrorKind::BlankBody,
        ));
    }

    let body = match body {
        "Sun" => pleiades_backend::CelestialBody::Sun,
        "Moon" => pleiades_backend::CelestialBody::Moon,
        "Mercury" => pleiades_backend::CelestialBody::Mercury,
        "Venus" => pleiades_backend::CelestialBody::Venus,
        "Mars" => pleiades_backend::CelestialBody::Mars,
        "Jupiter" => pleiades_backend::CelestialBody::Jupiter,
        "Saturn" => pleiades_backend::CelestialBody::Saturn,
        "Uranus" => pleiades_backend::CelestialBody::Uranus,
        "Neptune" => pleiades_backend::CelestialBody::Neptune,
        "Pluto" => pleiades_backend::CelestialBody::Pluto,
        "Ceres" => pleiades_backend::CelestialBody::Ceres,
        "Pallas" => pleiades_backend::CelestialBody::Pallas,
        "Juno" => pleiades_backend::CelestialBody::Juno,
        "Vesta" => pleiades_backend::CelestialBody::Vesta,
        other => {
            let Some((catalog, designation)) = other.split_once(':') else {
                return Err(SnapshotLoadError::new(
                    line_number,
                    SnapshotLoadErrorKind::UnsupportedBody {
                        body: other.to_string(),
                    },
                ));
            };

            let catalog = catalog.trim();
            let designation = designation.trim();
            if catalog.is_empty() || designation.is_empty() {
                return Err(SnapshotLoadError::new(
                    line_number,
                    SnapshotLoadErrorKind::UnsupportedBody {
                        body: other.to_string(),
                    },
                ));
            }

            pleiades_backend::CelestialBody::Custom(CustomBodyId::new(catalog, designation))
        }
    };

    Ok(body)
}

pub(crate) fn is_comparison_body(body: &pleiades_backend::CelestialBody) -> bool {
    matches!(
        body,
        pleiades_backend::CelestialBody::Sun
            | pleiades_backend::CelestialBody::Moon
            | pleiades_backend::CelestialBody::Mercury
            | pleiades_backend::CelestialBody::Venus
            | pleiades_backend::CelestialBody::Mars
            | pleiades_backend::CelestialBody::Jupiter
            | pleiades_backend::CelestialBody::Saturn
            | pleiades_backend::CelestialBody::Uranus
            | pleiades_backend::CelestialBody::Neptune
            | pleiades_backend::CelestialBody::Pluto
    )
}

pub(crate) fn is_reference_asteroid(body: &pleiades_backend::CelestialBody) -> bool {
    match body {
        pleiades_backend::CelestialBody::Ceres
        | pleiades_backend::CelestialBody::Pallas
        | pleiades_backend::CelestialBody::Juno
        | pleiades_backend::CelestialBody::Vesta => true,
        pleiades_backend::CelestialBody::Custom(custom) if custom.catalog == "asteroid" => true,
        _ => false,
    }
}

fn parse_f64(
    value: &str,
    line_number: usize,
    column: &'static str,
) -> Result<f64, SnapshotLoadError> {
    value.parse::<f64>().map_err(|_error| {
        SnapshotLoadError::new(
            line_number,
            SnapshotLoadErrorKind::InvalidNumber {
                column,
                value: value.to_string(),
            },
        )
    })
}

#[cfg(test)]
mod tests;
