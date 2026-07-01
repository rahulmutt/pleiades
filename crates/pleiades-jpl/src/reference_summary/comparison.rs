//! comparison summaries.

use core::fmt;
use std::sync::OnceLock;

use pleiades_backend::{EphemerisBackend, EphemerisRequest, QualityAnnotation};
use pleiades_types::{Apparentness, CoordinateFrame, Instant, TimeScale, ZodiacMode};

#[allow(unused_imports)]
use crate::reference_summary::*;
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, PartialEq)]
/// Release-facing summary of the checked-in comparison snapshot corpus (rows, bodies, epoch span).
pub struct ComparisonSnapshotSummary {
    /// Total number of parsed snapshot rows.
    pub row_count: usize,
    /// Number of distinct bodies covered by the comparison corpus.
    pub body_count: usize,
    /// Bodies covered by the comparison corpus in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the comparison corpus.
    pub epoch_count: usize,
    /// Earliest epoch represented in the comparison corpus.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the comparison corpus.
    pub latest_epoch: Instant,
}

/// Structured validation errors for a comparison snapshot summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ComparisonSnapshotSummaryValidationError {
    /// The summary did not include any rows.
    MissingRows,
    /// The summary did not include any bodies.
    MissingBodies,
    /// The declared body count did not match the number of listed bodies.
    BodyCountMismatch {
        /// Distinct-body count carried by the summary.
        body_count: usize,
        /// Number of bodies actually listed in the summary.
        bodies_len: usize,
    },
    /// The summary reused a body after trimming its display form.
    DuplicateBody {
        /// Index of the first occurrence in the compared pair.
        first_index: usize,
        /// Index of the second (duplicate) occurrence in the compared pair.
        second_index: usize,
        /// Body designation involved in the mismatch.
        body: String,
    },
    /// The summary body order diverged from the checked-in comparison corpus.
    BodyOrderMismatch {
        /// Zero-based position in the compared list where the drift was detected.
        index: usize,
        /// Value expected from the current evidence slice.
        expected: String,
        /// Value recorded in the summary under validation.
        found: String,
    },
    /// The summary did not include any epochs.
    MissingEpochs,
    /// The summary reported an invalid epoch range.
    InvalidEpochRange {
        /// Earliest epoch carried by the summary.
        earliest_epoch: Instant,
        /// Latest epoch carried by the summary.
        latest_epoch: Instant,
    },
}

impl ComparisonSnapshotSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingRows => "missing rows",
            Self::MissingBodies => "missing bodies",
            Self::BodyCountMismatch { .. } => "body count mismatch",
            Self::DuplicateBody { .. } => "duplicate body",
            Self::BodyOrderMismatch { .. } => "body order mismatch",
            Self::MissingEpochs => "missing epochs",
            Self::InvalidEpochRange { .. } => "invalid epoch range",
        }
    }
}

impl fmt::Display for ComparisonSnapshotSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BodyCountMismatch {
                body_count,
                bodies_len,
            } => write!(f, "body count {body_count} does not match listed bodies {bodies_len}"),
            Self::DuplicateBody {
                first_index,
                second_index,
                body,
            } => write!(
                f,
                "duplicate body '{body}' at index {second_index} (first seen at index {first_index})"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "body order mismatch at index {index}: expected '{expected}', found '{found}'"
            ),
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "invalid epoch range {}..{}",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch)
            ),
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for ComparisonSnapshotSummaryValidationError {}

/// Returns a compact coverage summary for the comparison snapshot used by validation.
pub fn comparison_snapshot_summary() -> Option<ComparisonSnapshotSummary> {
    let entries = comparison_snapshot();
    if entries.is_empty() {
        return None;
    }

    let mut bodies = BTreeSet::new();
    let mut epochs = BTreeSet::new();
    let mut earliest_epoch = entries[0].epoch;
    let mut latest_epoch = entries[0].epoch;

    for entry in entries {
        bodies.insert(entry.body.to_string());
        epochs.insert(entry.epoch.julian_day.days().to_bits());
        if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = entry.epoch;
        }
        if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = entry.epoch;
        }
    }

    Some(ComparisonSnapshotSummary {
        row_count: entries.len(),
        body_count: bodies.len(),
        bodies: comparison_body_list().to_vec(),
        epoch_count: epochs.len(),
        earliest_epoch,
        latest_epoch,
    })
}

/// A compact body-class coverage summary for the comparison snapshot used by validation.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonSnapshotBodyClassCoverageSummary {
    /// Number of rows in the comparison snapshot.
    pub row_count: usize,
    /// Bodies covered by the comparison snapshot in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the comparison snapshot.
    pub epoch_count: usize,
    /// Per-body windows covered by the comparison snapshot in first-seen order.
    pub windows: Vec<ComparisonSnapshotSourceWindow>,
}

/// Validation error for a comparison snapshot body-class coverage summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComparisonSnapshotBodyClassCoverageSummaryValidationError {
    /// The comparison snapshot body-class coverage summary is unavailable.
    Unavailable,
    /// A summary field is out of sync with the checked-in body-class coverage.
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for ComparisonSnapshotBodyClassCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => f.write_str("the comparison snapshot body-class coverage summary is unavailable"),
            Self::FieldOutOfSync { field } => write!(
                f,
                "the comparison snapshot body-class coverage summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ComparisonSnapshotBodyClassCoverageSummaryValidationError {}

impl ComparisonSnapshotBodyClassCoverageSummary {
    /// Returns a compact body-class summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let windows = self
            .windows
            .iter()
            .map(ComparisonSnapshotSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");

        format!(
            "Comparison snapshot body-class coverage: {} rows across {} bodies and {} epochs; bodies: {}; windows: {}",
            self.row_count,
            self.body_count(),
            self.epoch_count,
            format_bodies(&self.bodies),
            windows,
        )
    }

    fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// Returns `Ok(())` when the body-class coverage summary still matches the checked-in slice.
    pub fn validate(
        &self,
    ) -> Result<(), ComparisonSnapshotBodyClassCoverageSummaryValidationError> {
        let Some(expected) = comparison_snapshot_body_class_coverage_summary_details() else {
            return Err(
                ComparisonSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        };

        if self.row_count != expected.row_count {
            return Err(
                ComparisonSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        }
        if self.bodies != expected.bodies {
            return Err(
                ComparisonSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ComparisonSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                ComparisonSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated body-class coverage summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ComparisonSnapshotBodyClassCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonSnapshotBodyClassCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn comparison_snapshot_body_class_coverage_summary_details(
) -> Option<ComparisonSnapshotBodyClassCoverageSummary> {
    let summary = comparison_snapshot_summary()?;
    let source_windows = comparison_snapshot_source_window_summary_details()?;

    Some(ComparisonSnapshotBodyClassCoverageSummary {
        row_count: summary.row_count,
        bodies: summary.bodies.to_vec(),
        epoch_count: summary.epoch_count,
        windows: source_windows.windows,
    })
}

/// Returns the compact body-class coverage summary for the comparison snapshot used by validation.
pub fn comparison_snapshot_body_class_coverage_summary(
) -> Option<ComparisonSnapshotBodyClassCoverageSummary> {
    comparison_snapshot_body_class_coverage_summary_details()
}

/// Returns the release-facing body-class coverage summary string for the comparison snapshot.
pub fn comparison_snapshot_body_class_coverage_summary_for_report() -> String {
    match comparison_snapshot_body_class_coverage_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Comparison snapshot body-class coverage: unavailable ({error})"),
        },
        None => "Comparison snapshot body-class coverage: unavailable".to_string(),
    }
}

/// Returns the validated release-facing body-class coverage summary string for the comparison snapshot.
pub fn validated_comparison_snapshot_body_class_coverage_summary_for_report(
) -> Result<String, String> {
    let summary = comparison_snapshot_body_class_coverage_summary().ok_or_else(|| {
        ComparisonSnapshotBodyClassCoverageSummaryValidationError::Unavailable.to_string()
    })?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

pub(crate) fn comparison_snapshot_source_checksum() -> u64 {
    static CHECKSUM: OnceLock<u64> = OnceLock::new();
    *CHECKSUM.get_or_init(|| {
        checksum64(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/j2000_snapshot.csv"
        )))
    })
}

/// Backend-owned provenance summary for the comparison snapshot used by validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComparisonSnapshotSourceSummary {
    /// Source attribution for the comparison snapshot.
    pub source: String,
    /// Coverage note for the comparison snapshot.
    pub coverage: String,
    /// Redistribution posture for the comparison snapshot.
    pub redistribution: String,
    /// CSV column layout for the comparison snapshot.
    pub columns: String,
    /// Deterministic checksum of the checked-in comparison snapshot source material.
    pub checksum: u64,
}

/// Structured validation errors for a comparison snapshot provenance summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComparisonSnapshotSourceSummaryValidationError {
    /// The summary did not include a non-empty source label.
    BlankSource,
    /// The summary did not include a non-empty coverage label.
    BlankCoverage,
    /// The summary did not include a non-empty redistribution label.
    BlankRedistribution,
    /// The summary did not include a non-empty columns label.
    BlankColumns,
    /// The summary carried surrounding whitespace in one of its labels.
    SurroundedByWhitespace {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
    /// One of the canonical summary fields drifted from the checked-in slice.
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
    /// The summary checksum drifted from the checked-in source material.
    ChecksumMismatch,
}

impl ComparisonSnapshotSourceSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::BlankSource => "blank source",
            Self::BlankCoverage => "blank coverage",
            Self::BlankRedistribution => "blank redistribution",
            Self::BlankColumns => "blank columns",
            Self::SurroundedByWhitespace { .. } => "surrounded by whitespace",
            Self::FieldOutOfSync { .. } => "field out of sync",
            Self::ChecksumMismatch => "checksum mismatch",
        }
    }
}

impl fmt::Display for ComparisonSnapshotSourceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurroundedByWhitespace { field } => {
                write!(f, "{field} contains surrounding whitespace")
            }
            Self::FieldOutOfSync { field } => write!(f, "{field} is out of sync"),
            Self::ChecksumMismatch => f.write_str("checksum mismatch"),
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for ComparisonSnapshotSourceSummaryValidationError {}

impl ComparisonSnapshotSourceSummary {
    /// Validates that the summary remains internally consistent.
    pub fn validate(&self) -> Result<(), ComparisonSnapshotSourceSummaryValidationError> {
        if self.source.trim().is_empty() {
            return Err(ComparisonSnapshotSourceSummaryValidationError::BlankSource);
        }
        if has_surrounding_whitespace(&self.source) {
            return Err(
                ComparisonSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "source",
                },
            );
        }
        if self.source != COMPARISON_SNAPSHOT_SOURCE_EXPECTED {
            return Err(
                ComparisonSnapshotSourceSummaryValidationError::FieldOutOfSync { field: "source" },
            );
        }
        if self.coverage.trim().is_empty() {
            return Err(ComparisonSnapshotSourceSummaryValidationError::BlankCoverage);
        }
        if has_surrounding_whitespace(&self.coverage) {
            return Err(
                ComparisonSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "coverage",
                },
            );
        }
        if self.coverage != COMPARISON_SNAPSHOT_COVERAGE_EXPECTED {
            return Err(
                ComparisonSnapshotSourceSummaryValidationError::FieldOutOfSync {
                    field: "coverage",
                },
            );
        }
        if self.redistribution.trim().is_empty() {
            return Err(ComparisonSnapshotSourceSummaryValidationError::BlankRedistribution);
        }
        if has_surrounding_whitespace(&self.redistribution) {
            return Err(
                ComparisonSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "redistribution",
                },
            );
        }
        if self.redistribution != COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED {
            return Err(
                ComparisonSnapshotSourceSummaryValidationError::FieldOutOfSync {
                    field: "redistribution",
                },
            );
        }
        if self.columns.trim().is_empty() {
            return Err(ComparisonSnapshotSourceSummaryValidationError::BlankColumns);
        }
        if has_surrounding_whitespace(&self.columns) {
            return Err(
                ComparisonSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "columns",
                },
            );
        }
        if self.columns != COMPARISON_SNAPSHOT_COLUMNS {
            return Err(
                ComparisonSnapshotSourceSummaryValidationError::FieldOutOfSync { field: "columns" },
            );
        }
        if self.checksum != comparison_snapshot_source_checksum() {
            return Err(ComparisonSnapshotSourceSummaryValidationError::ChecksumMismatch);
        }
        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Comparison snapshot source: {}; coverage={}; redistribution={}; columns={}; checksum=0x{:016x}",
            self.source, self.coverage, self.redistribution, self.columns, self.checksum
        )
    }

    /// Returns a compact summary line after validating the comparison snapshot source summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ComparisonSnapshotSourceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonSnapshotSourceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned provenance summary for the comparison snapshot.
pub fn comparison_snapshot_source_summary() -> ComparisonSnapshotSourceSummary {
    static SUMMARY: OnceLock<ComparisonSnapshotSourceSummary> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let manifest = comparison_snapshot_manifest();
            ComparisonSnapshotSourceSummary {
                source: manifest
                    .source_or(COMPARISON_SNAPSHOT_SOURCE_EXPECTED)
                    .to_string(),
                coverage: manifest
                    .coverage_or(COMPARISON_SNAPSHOT_COVERAGE_EXPECTED)
                    .to_string(),
                redistribution: manifest
                    .redistribution_or(COMPARISON_SNAPSHOT_REDISTRIBUTION_FALLBACK)
                    .to_string(),
                columns: manifest.columns_summary(),
                checksum: comparison_snapshot_source_checksum(),
            }
        })
        .clone()
}

/// Formats the source/material summary for the comparison snapshot used by validation.
pub fn format_comparison_snapshot_source_summary(
    summary: &ComparisonSnapshotSourceSummary,
) -> String {
    summary.summary_line()
}

pub(crate) fn format_validated_comparison_snapshot_source_summary_for_report(
    summary: &ComparisonSnapshotSourceSummary,
    manifest: &SnapshotManifest,
) -> String {
    if let Err(error) = manifest.validate() {
        return format!("Comparison snapshot source: unavailable ({error})");
    }

    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Comparison snapshot source: unavailable ({error})"),
    }
}

/// Returns the source/material summary for the comparison snapshot used by validation.
pub fn comparison_snapshot_source_summary_for_report() -> String {
    format_validated_comparison_snapshot_source_summary_for_report(
        &comparison_snapshot_source_summary(),
        comparison_snapshot_manifest(),
    )
}

/// Returns the validated source/material summary for the comparison snapshot.
pub fn validated_comparison_snapshot_source_summary_for_report() -> Result<String, String> {
    let manifest = comparison_snapshot_manifest();
    manifest.validate().map_err(|error| error.to_string())?;
    comparison_snapshot_source_summary()
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

/// Returns the validated source-window summary for the comparison snapshot.
pub fn validated_comparison_snapshot_source_window_summary_for_report() -> Result<String, String> {
    match comparison_snapshot_source_window_summary() {
        Some(summary) => summary
            .validated_summary_line()
            .map_err(|error| error.to_string()),
        None => Err("comparison snapshot source windows unavailable".to_string()),
    }
}

/// A single body-window slice inside the comparison snapshot source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonSnapshotSourceWindow {
    /// The snapshot body covered by this window.
    pub body: pleiades_backend::CelestialBody,
    /// Number of samples for the body.
    pub sample_count: usize,
    /// Number of distinct epochs represented for the body.
    pub epoch_count: usize,
    /// Earliest epoch represented for the body.
    pub earliest_epoch: Instant,
    /// Latest epoch represented for the body.
    pub latest_epoch: Instant,
}

impl ComparisonSnapshotSourceWindow {
    /// Returns a compact body-window summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let time_span = if self.earliest_epoch == self.latest_epoch {
            format_instant(self.earliest_epoch)
        } else {
            format!(
                "{}..{}",
                format_instant(self.earliest_epoch),
                format_instant(self.latest_epoch)
            )
        };

        format!(
            "{}: {} samples across {} epochs at {}",
            self.body, self.sample_count, self.epoch_count, time_span
        )
    }
}

/// Compact release-facing summary for the comparison snapshot source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonSnapshotSourceWindowSummary {
    /// Number of comparison-snapshot samples in the source slice.
    pub sample_count: usize,
    /// Bodies covered by the comparison snapshot source slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the source slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the source slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the source slice.
    pub latest_epoch: Instant,
    /// Per-body window breakdown in first-seen order.
    pub windows: Vec<ComparisonSnapshotSourceWindow>,
}

impl fmt::Display for ComparisonSnapshotSourceWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl ComparisonSnapshotSourceWindowSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(ComparisonSnapshotSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Comparison snapshot source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the comparison snapshot source windows still match the checked-in slice.
    pub fn validate(&self) -> Result<(), ComparisonSnapshotSourceWindowSummaryValidationError> {
        let Some(expected) = comparison_snapshot_source_window_summary_details() else {
            return Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                ComparisonSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated comparison snapshot source window summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ComparisonSnapshotSourceWindowSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonSnapshotSourceWindowSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a comparison snapshot source window summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComparisonSnapshotSourceWindowSummaryValidationError {
    /// A summary field is out of sync with the checked-in comparison snapshot source windows.
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for ComparisonSnapshotSourceWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the comparison snapshot source window summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ComparisonSnapshotSourceWindowSummaryValidationError {}

pub(crate) fn comparison_snapshot_source_window_summary_details(
) -> Option<ComparisonSnapshotSourceWindowSummary> {
    let entries = comparison_snapshot();
    if entries.is_empty() {
        return None;
    }

    let mut windows = Vec::new();
    for body in comparison_body_list() {
        let body_entries = entries
            .iter()
            .filter(|entry| entry.body == *body)
            .collect::<Vec<_>>();
        if body_entries.is_empty() {
            continue;
        }

        let mut earliest_epoch = body_entries[0].epoch;
        let mut latest_epoch = body_entries[0].epoch;
        let mut epochs = BTreeSet::new();
        for entry in &body_entries {
            epochs.insert(entry.epoch.julian_day.days().to_bits());
            if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
                earliest_epoch = entry.epoch;
            }
            if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
                latest_epoch = entry.epoch;
            }
        }

        windows.push(ComparisonSnapshotSourceWindow {
            body: body.clone(),
            sample_count: body_entries.len(),
            epoch_count: epochs.len(),
            earliest_epoch,
            latest_epoch,
        });
    }

    if windows.is_empty() {
        return None;
    }

    let earliest_epoch = windows
        .iter()
        .map(|window| window.earliest_epoch)
        .min_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("comparison snapshot source windows should not be empty after collection");
    let latest_epoch = windows
        .iter()
        .map(|window| window.latest_epoch)
        .max_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("comparison snapshot source windows should not be empty after collection");

    Some(ComparisonSnapshotSourceWindowSummary {
        sample_count: entries.len(),
        sample_bodies: comparison_body_list().to_vec(),
        epoch_count: entries
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
        windows,
    })
}

/// Returns the compact typed summary for the comparison snapshot source coverage.
pub fn comparison_snapshot_source_window_summary() -> Option<ComparisonSnapshotSourceWindowSummary>
{
    comparison_snapshot_source_window_summary_details()
}

/// Formats the comparison snapshot source windows for release-facing reporting.
pub fn format_comparison_snapshot_source_window_summary(
    summary: &ComparisonSnapshotSourceWindowSummary,
) -> String {
    summary.summary_line()
}

/// Returns the body-window summary for the comparison snapshot.
pub fn comparison_snapshot_source_window_summary_for_report() -> String {
    match validated_comparison_snapshot_source_window_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) if error == "comparison snapshot source windows unavailable" => {
            "Comparison snapshot source windows: unavailable".to_string()
        }
        Err(error) => format!("Comparison snapshot source windows: unavailable ({error})"),
    }
}

/// Returns the manifest summary for the comparison snapshot used by validation.
pub fn comparison_snapshot_manifest_summary() -> SnapshotManifestSummary {
    SnapshotManifestSummary {
        label: "Comparison snapshot manifest",
        manifest: comparison_snapshot_manifest().clone(),
        source_fallback: "unknown",
        coverage_fallback:
            "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.",
    }
}

/// Returns the manifest summary for the comparison snapshot used by validation.
pub fn comparison_snapshot_manifest_summary_for_report() -> String {
    match validated_comparison_snapshot_manifest_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Comparison snapshot manifest: unavailable ({error})"),
    }
}

/// Returns the validated manifest summary for the comparison snapshot used by validation.
pub fn validated_comparison_snapshot_manifest_summary_for_report() -> Result<String, String> {
    let manifest_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/j2000_snapshot.csv"
    ));
    validate_snapshot_manifest_header_structure(
        manifest_text,
        "JPL Horizons reference snapshot.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.",
        "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.",
        Some(COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED),
        &["body", "x_km", "y_km", "z_km"],
    )
    .map_err(|error| error.to_string())?;

    let summary = comparison_snapshot_manifest_summary();
    summary
        .validate_with_expected_metadata_and_redistribution(
            "JPL Horizons reference snapshot.",
            "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000, TDB 2451545.0.",
            "Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, and Pluto at J2000.",
            COMPARISON_SNAPSHOT_REDISTRIBUTION_EXPECTED,
            &["body", "x_km", "y_km", "z_km"],
        )
        .map_err(|error| error.to_string())?;

    Ok(summary.summary_line())
}

impl ComparisonSnapshotSummary {
    /// Validates that the summary remains internally consistent.
    pub fn validate(&self) -> Result<(), ComparisonSnapshotSummaryValidationError> {
        if self.row_count == 0 {
            return Err(ComparisonSnapshotSummaryValidationError::MissingRows);
        }
        if self.bodies.is_empty() {
            return Err(ComparisonSnapshotSummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(
                ComparisonSnapshotSummaryValidationError::BodyCountMismatch {
                    body_count: self.body_count,
                    bodies_len: self.bodies.len(),
                },
            );
        }

        for (index, body) in self.bodies.iter().enumerate() {
            if self.bodies[..index].iter().any(|other| other == body) {
                return Err(ComparisonSnapshotSummaryValidationError::DuplicateBody {
                    first_index: self.bodies[..index]
                        .iter()
                        .position(|other| other == body)
                        .unwrap(),
                    second_index: index,
                    body: body.to_string(),
                });
            }
        }

        let expected_bodies = comparison_body_list();
        if self.bodies != expected_bodies {
            let mismatch_index = self
                .bodies
                .iter()
                .zip(expected_bodies.iter())
                .position(|(actual, expected)| actual != expected)
                .unwrap_or_else(|| self.bodies.len().min(expected_bodies.len()));
            return Err(
                ComparisonSnapshotSummaryValidationError::BodyOrderMismatch {
                    index: mismatch_index,
                    expected: expected_bodies
                        .get(mismatch_index)
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "<end of comparison body list>".to_string()),
                    found: self
                        .bodies
                        .get(mismatch_index)
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "<end of summary body list>".to_string()),
                },
            );
        }

        if self.epoch_count == 0 {
            return Err(ComparisonSnapshotSummaryValidationError::MissingEpochs);
        }
        if self.earliest_epoch.julian_day.days() > self.latest_epoch.julian_day.days() {
            return Err(
                ComparisonSnapshotSummaryValidationError::InvalidEpochRange {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Comparison snapshot coverage: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}",
            self.row_count,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(&self.bodies),
        )
    }

    /// Returns a compact summary line after validating the comparison snapshot summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ComparisonSnapshotSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonSnapshotSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the comparison snapshot coverage for release-facing reporting.
pub fn format_comparison_snapshot_summary(summary: &ComparisonSnapshotSummary) -> String {
    summary.summary_line()
}

/// Returns the release-facing comparison snapshot coverage summary string.
pub fn comparison_snapshot_summary_for_report() -> String {
    match comparison_snapshot_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Comparison snapshot coverage: unavailable ({error})"),
        },
        None => "Comparison snapshot coverage: unavailable".to_string(),
    }
}

/// A compact coverage summary for the checked-in comparison snapshot in mixed-frame batch parity mode.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonSnapshotBatchParitySummary {
    /// Comparison snapshot coverage exercised through the batch regression.
    pub snapshot: ComparisonSnapshotSummary,
    /// Number of ecliptic requests in the mixed-frame batch regression.
    pub ecliptic_request_count: usize,
    /// Number of equatorial requests in the mixed-frame batch regression.
    pub equatorial_request_count: usize,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

/// Returns a compact mixed-frame batch parity summary for the checked-in comparison snapshot.
pub fn comparison_snapshot_batch_parity_summary() -> Option<ComparisonSnapshotBatchParitySummary> {
    let snapshot = comparison_snapshot_summary()?;
    let backend = JplSnapshotBackend;
    let requests = comparison_snapshot_batch_parity_requests()?;
    let results = backend.positions(&requests).ok()?;

    if results.len() != requests.len() {
        return None;
    }

    let mut ecliptic_request_count = 0usize;
    let mut equatorial_request_count = 0usize;
    let mut exact_count = 0usize;
    let mut interpolated_count = 0usize;
    let mut approximate_count = 0usize;
    let mut unknown_count = 0usize;

    for ((request, result), entry) in requests
        .iter()
        .zip(results.iter())
        .zip(comparison_snapshot())
    {
        let single = backend.position(request).ok()?;
        if single != *result {
            return None;
        }

        if result.body != entry.body
            || result.instant.julian_day != entry.epoch.julian_day
            || result.frame != request.frame
        {
            return None;
        }

        let ecliptic = result
            .ecliptic
            .as_ref()
            .expect("comparison snapshot batch parity rows should include ecliptic coordinates");
        if *ecliptic != entry.ecliptic() {
            return None;
        }

        if request.frame == CoordinateFrame::Equatorial {
            let expected_equatorial = ecliptic.to_equatorial(result.instant.mean_obliquity());
            let equatorial = result
                .equatorial
                .as_ref()
                .expect("equatorial batch parity rows should include equatorial coordinates");
            if *equatorial != expected_equatorial {
                return None;
            }
        }

        match request.frame {
            CoordinateFrame::Ecliptic => ecliptic_request_count += 1,
            CoordinateFrame::Equatorial => equatorial_request_count += 1,
            _ => return None,
        }

        match result.quality {
            QualityAnnotation::Exact => exact_count += 1,
            QualityAnnotation::Interpolated => interpolated_count += 1,
            QualityAnnotation::Approximate => approximate_count += 1,
            QualityAnnotation::Unknown => unknown_count += 1,
            _ => unknown_count += 1,
        }
    }

    Some(ComparisonSnapshotBatchParitySummary {
        snapshot,
        ecliptic_request_count,
        equatorial_request_count,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

/// Structured validation errors for a comparison snapshot batch parity summary.
#[derive(Clone, Debug, PartialEq)]
pub enum ComparisonSnapshotBatchParitySummaryValidationError {
    /// The comparison snapshot batch parity summary is unavailable.
    Unavailable,
    /// The nested comparison snapshot summary failed validation.
    Snapshot(ComparisonSnapshotSummaryValidationError),
    /// The number of mixed-frame requests does not match the row count.
    RequestCountMismatch {
        /// Number of ecliptic-frame requests carried by the summary.
        ecliptic_request_count: usize,
        /// Number of equatorial-frame requests carried by the summary.
        equatorial_request_count: usize,
        /// Row count carried by the summary under validation.
        row_count: usize,
    },
    /// The quality counts do not match the row count.
    QualityCountMismatch {
        /// Number of samples classified as exact (fixture-served).
        exact_count: usize,
        /// Number of samples classified as interpolated.
        interpolated_count: usize,
        /// Number of samples classified as approximate.
        approximate_count: usize,
        /// Number of samples with an unknown classification.
        unknown_count: usize,
        /// Row count carried by the summary under validation.
        row_count: usize,
    },
    /// The summary drifted away from the checked-in derived evidence.
    DerivedSummaryMismatch,
}

impl fmt::Display for ComparisonSnapshotBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => {
                f.write_str("the comparison snapshot batch parity summary is unavailable")
            }
            Self::Snapshot(error) => write!(f, "comparison snapshot validation failed: {error}"),
            Self::RequestCountMismatch {
                ecliptic_request_count,
                equatorial_request_count,
                row_count,
            } => write!(
                f,
                "request count {}+{} does not match row count {}",
                ecliptic_request_count, equatorial_request_count, row_count,
            ),
            Self::QualityCountMismatch {
                exact_count,
                interpolated_count,
                approximate_count,
                unknown_count,
                row_count,
            } => write!(
                f,
                "quality counts {}+{}+{}+{} do not match row count {}",
                exact_count, interpolated_count, approximate_count, unknown_count, row_count,
            ),
            Self::DerivedSummaryMismatch => f.write_str("derived summary mismatch"),
        }
    }
}

impl std::error::Error for ComparisonSnapshotBatchParitySummaryValidationError {}

impl ComparisonSnapshotBatchParitySummary {
    /// Validates that the batch parity summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), ComparisonSnapshotBatchParitySummaryValidationError> {
        self.snapshot
            .validate()
            .map_err(ComparisonSnapshotBatchParitySummaryValidationError::Snapshot)?;

        if self.ecliptic_request_count + self.equatorial_request_count != self.snapshot.row_count {
            return Err(
                ComparisonSnapshotBatchParitySummaryValidationError::RequestCountMismatch {
                    ecliptic_request_count: self.ecliptic_request_count,
                    equatorial_request_count: self.equatorial_request_count,
                    row_count: self.snapshot.row_count,
                },
            );
        }

        if self.exact_count + self.interpolated_count + self.approximate_count + self.unknown_count
            != self.snapshot.row_count
        {
            return Err(
                ComparisonSnapshotBatchParitySummaryValidationError::QualityCountMismatch {
                    exact_count: self.exact_count,
                    interpolated_count: self.interpolated_count,
                    approximate_count: self.approximate_count,
                    unknown_count: self.unknown_count,
                    row_count: self.snapshot.row_count,
                },
            );
        }

        if comparison_snapshot_batch_parity_summary().as_ref() != Some(self) {
            return Err(
                ComparisonSnapshotBatchParitySummaryValidationError::DerivedSummaryMismatch,
            );
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "JPL comparison snapshot batch parity: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}; frame mix: {} ecliptic, {} equatorial; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.snapshot.row_count,
            self.snapshot.body_count,
            self.snapshot.epoch_count,
            format_instant(self.snapshot.earliest_epoch),
            format_instant(self.snapshot.latest_epoch),
            format_bodies(&self.snapshot.bodies),
            self.ecliptic_request_count,
            self.equatorial_request_count,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns a compact summary line after validating the batch parity summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ComparisonSnapshotBatchParitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ComparisonSnapshotBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the checked-in comparison snapshot batch parity summary for release-facing reporting.
pub fn format_comparison_snapshot_batch_parity_summary(
    summary: &ComparisonSnapshotBatchParitySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing comparison snapshot batch parity summary string.
pub fn comparison_snapshot_batch_parity_summary_for_report() -> String {
    match comparison_snapshot_batch_parity_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("JPL comparison snapshot batch parity: unavailable ({error})"),
        },
        None => "JPL comparison snapshot batch parity: unavailable".to_string(),
    }
}

/// Returns the validated release-facing comparison snapshot batch parity summary string.
pub fn validated_comparison_snapshot_batch_parity_summary_for_report() -> Result<String, String> {
    let summary = comparison_snapshot_batch_parity_summary().ok_or_else(|| {
        ComparisonSnapshotBatchParitySummaryValidationError::Unavailable.to_string()
    })?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

/// Returns the comparison-only subset used by the stage-4 validation corpus.
pub fn comparison_snapshot() -> &'static [SnapshotEntry] {
    comparison_snapshot_entries()
}

/// Returns the comparison-snapshot request corpus in the requested frame.
///
/// The requests preserve the checked-in row order and retag the comparison rows
/// onto the TT request time scale currently used by the validation corpus, which
/// lets downstream tooling reuse the exact batch shape without reconstructing it
/// from the snapshot metadata in each caller.
pub fn comparison_snapshot_requests(frame: CoordinateFrame) -> Option<Vec<EphemerisRequest>> {
    let entries = comparison_snapshot();
    if entries.is_empty() {
        return None;
    }

    Some(
        entries
            .iter()
            .map(|entry| EphemerisRequest {
                body: entry.body.clone(),
                instant: Instant::new(entry.epoch.julian_day, TimeScale::Tt),
                observer: None,
                frame,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect(),
    )
}

/// This is a compatibility alias for [`comparison_snapshot_requests`].
#[doc(alias = "comparison_snapshot_requests")]
pub fn comparison_snapshot_request_corpus(frame: CoordinateFrame) -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_requests(frame)
}

/// Returns the ecliptic comparison-snapshot request corpus used by validation tooling.
///
/// This is a compatibility alias for [`comparison_snapshot_request_corpus`].
#[doc(alias = "comparison_snapshot_requests")]
pub fn comparison_snapshot_ecliptic_request_corpus() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_request_corpus(CoordinateFrame::Ecliptic)
}

/// Returns the ecliptic comparison-snapshot request corpus used by validation tooling.
///
/// This is a compatibility alias for [`comparison_snapshot_ecliptic_request_corpus`].
#[doc(alias = "comparison_snapshot_ecliptic_request_corpus")]
pub fn comparison_snapshot_ecliptic_requests() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_ecliptic_request_corpus()
}

/// Returns the equatorial comparison-snapshot request corpus used by parity checks.
///
/// This is a compatibility alias for [`comparison_snapshot_requests`].
#[doc(alias = "comparison_snapshot_requests")]
pub fn comparison_snapshot_equatorial_parity_requests() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_requests(CoordinateFrame::Equatorial)
}

/// Returns the equatorial comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`comparison_snapshot_equatorial_parity_requests`].
#[doc(alias = "comparison_snapshot_equatorial_parity_requests")]
pub fn comparison_snapshot_equatorial_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_equatorial_parity_requests()
}

/// Returns the equatorial comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`comparison_snapshot_equatorial_batch_parity_requests`].
#[doc(alias = "comparison_snapshot_equatorial_batch_parity_requests")]
pub fn comparison_snapshot_equatorial_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>>
{
    comparison_snapshot_equatorial_batch_parity_requests()
}

/// Returns the equatorial comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`comparison_snapshot_equatorial_request_corpus`].
#[doc(alias = "comparison_snapshot_equatorial_request_corpus")]
pub fn comparison_snapshot_equatorial_requests() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_equatorial_request_corpus()
}

/// Returns the equatorial comparison-snapshot request corpus used by parity checks.
///
/// This is a compatibility alias for [`comparison_snapshot_equatorial_parity_requests`].
#[doc(alias = "comparison_snapshot_equatorial_parity_requests")]
pub fn comparison_snapshot_equatorial_request_corpus() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_equatorial_parity_requests()
}

/// Returns the equatorial comparison-snapshot request corpus used by parity checks.
///
/// This is a compatibility alias for [`comparison_snapshot_equatorial_parity_requests`].
#[doc(alias = "comparison_snapshot_equatorial_parity_requests")]
pub fn comparison_snapshot_equatorial_parity_request_corpus() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_equatorial_parity_requests()
}

/// Returns the mixed-frame comparison-snapshot request corpus used by batch parity checks.
///
/// The requests preserve the checked-in row order and alternate between ecliptic
/// and equatorial frames so downstream tooling can reuse the exact validation
/// batch shape without reconstructing it from snapshot metadata.
pub fn comparison_snapshot_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    let entries = comparison_snapshot();
    if entries.is_empty() {
        return None;
    }

    Some(
        entries
            .iter()
            .enumerate()
            .map(|(index, entry)| EphemerisRequest {
                body: entry.body.clone(),
                instant: Instant::new(entry.epoch.julian_day, TimeScale::Tt),
                observer: None,
                frame: if index % 2 == 0 {
                    CoordinateFrame::Ecliptic
                } else {
                    CoordinateFrame::Equatorial
                },
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect(),
    )
}

/// Returns the mixed-frame comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`comparison_snapshot_batch_parity_requests`].
#[doc(alias = "comparison_snapshot_batch_parity_requests")]
pub fn comparison_snapshot_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_batch_parity_requests()
}

/// Returns the mixed-scale comparison-snapshot request corpus used by batch parity checks.
///
/// The requests preserve the checked-in row order, keep the ecliptic frame, and
/// alternate TT/TDB labels so downstream tooling can reuse the exact validation
/// batch shape without reconstructing it from snapshot metadata.
pub fn comparison_snapshot_mixed_time_scale_batch_parity_requests() -> Option<Vec<EphemerisRequest>>
{
    let entries = comparison_snapshot();
    if entries.is_empty() {
        return None;
    }

    Some(
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
            .collect(),
    )
}

/// Returns the mixed-scale comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`comparison_snapshot_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "comparison_snapshot_mixed_time_scale_batch_parity_requests")]
pub fn comparison_snapshot_mixed_time_scale_batch_parity_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_mixed_time_scale_batch_parity_requests()
}

/// Returns the mixed TT/TDB comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`comparison_snapshot_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "comparison_snapshot_mixed_time_scale_batch_parity_requests")]
pub fn comparison_snapshot_mixed_tt_tdb_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_mixed_time_scale_batch_parity_requests()
}

/// Returns the mixed TT/TDB comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`comparison_snapshot_mixed_tt_tdb_batch_parity_requests`].
#[doc(alias = "comparison_snapshot_mixed_tt_tdb_batch_parity_requests")]
pub fn comparison_snapshot_mixed_tt_tdb_batch_parity_request_corpus(
) -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_mixed_tt_tdb_batch_parity_requests()
}

/// Returns the mixed TT/TDB comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`comparison_snapshot_mixed_time_scale_request_corpus`].
#[doc(alias = "comparison_snapshot_mixed_time_scale_request_corpus")]
pub fn comparison_snapshot_mixed_tt_tdb_request_corpus() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_mixed_tt_tdb_batch_parity_requests()
}

/// Returns the mixed TT/TDB comparison-snapshot request corpus used by batch parity checks.
///
/// This is a compatibility alias for
/// [`comparison_snapshot_mixed_tt_tdb_batch_parity_requests`].
#[doc(alias = "comparison_snapshot_mixed_tt_tdb_batch_parity_requests")]
pub fn comparison_snapshot_mixed_time_scale_request_corpus() -> Option<Vec<EphemerisRequest>> {
    comparison_snapshot_mixed_time_scale_batch_parity_requests()
}

/// Returns the parsed manifest for the comparison snapshot.
pub fn comparison_snapshot_manifest() -> &'static SnapshotManifest {
    static MANIFEST: OnceLock<SnapshotManifest> = OnceLock::new();
    MANIFEST.get_or_init(|| {
        parse_snapshot_manifest(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/j2000_snapshot.csv"
        )))
    })
}

/// Returns the comparison-only body coverage used by validation tooling.
pub fn comparison_bodies() -> &'static [pleiades_backend::CelestialBody] {
    comparison_body_list()
}

#[cfg(test)]
mod tests;
