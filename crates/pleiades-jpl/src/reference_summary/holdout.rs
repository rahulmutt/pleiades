//! holdout summaries.

use core::fmt;
use std::sync::OnceLock;

use pleiades_backend::{EphemerisBackend, QualityAnnotation};
use pleiades_types::{Instant, TimeScale};

#[allow(unused_imports)]
use crate::reference_summary::*;
#[allow(unused_imports)]
use crate::*;

/// Structured validation errors for an independent hold-out snapshot summary.
#[derive(Clone, Debug, PartialEq)]
pub enum IndependentHoldoutSnapshotSummaryValidationError {
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
    /// The summary did not include any epochs.
    MissingEpochs,
    /// The summary reported an invalid epoch range.
    InvalidEpochRange {
        /// Earliest epoch carried by the summary.
        earliest_epoch: Instant,
        /// Latest epoch carried by the summary.
        latest_epoch: Instant,
    },
    /// The summary drifted away from the checked-in derived evidence.
    DerivedSummaryMismatch,
}

impl IndependentHoldoutSnapshotSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingRows => "missing rows",
            Self::MissingBodies => "missing bodies",
            Self::BodyCountMismatch { .. } => "body count mismatch",
            Self::DuplicateBody { .. } => "duplicate body",
            Self::MissingEpochs => "missing epochs",
            Self::InvalidEpochRange { .. } => "invalid epoch range",
            Self::DerivedSummaryMismatch => "derived summary mismatch",
        }
    }
}

impl fmt::Display for IndependentHoldoutSnapshotSummaryValidationError {
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
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "invalid epoch range {}..{}",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch)
            ),
            Self::DerivedSummaryMismatch => f.write_str(self.label()),
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for IndependentHoldoutSnapshotSummaryValidationError {}

/// A compact coverage summary for the independent hold-out corpus used to
/// validate interpolation against rows that are not part of the main snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct IndependentHoldoutSnapshotSummary {
    /// Total number of parsed hold-out rows.
    pub row_count: usize,
    /// Number of distinct bodies covered by the hold-out corpus.
    pub body_count: usize,
    /// Bodies covered by the hold-out corpus in first-seen order.
    pub bodies: Vec<String>,
    /// Number of distinct epochs covered by the hold-out corpus.
    pub epoch_count: usize,
    /// Earliest epoch represented in the hold-out corpus.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the hold-out corpus.
    pub latest_epoch: Instant,
}

/// Returns a compact coverage summary for the independent hold-out corpus.
pub fn independent_holdout_snapshot_summary() -> Option<IndependentHoldoutSnapshotSummary> {
    let entries = independent_holdout_snapshot_entries()?;

    let mut bodies = Vec::new();
    let mut seen_bodies = BTreeSet::new();
    let mut epochs = BTreeSet::new();
    let mut earliest_epoch = entries[0].epoch;
    let mut latest_epoch = entries[0].epoch;

    for entry in entries {
        let body = entry.body.to_string();
        if seen_bodies.insert(body.clone()) {
            bodies.push(body);
        }
        epochs.insert(entry.epoch.julian_day.days().to_bits());
        if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = entry.epoch;
        }
        if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = entry.epoch;
        }
    }

    Some(IndependentHoldoutSnapshotSummary {
        row_count: entries.len(),
        body_count: bodies.len(),
        bodies,
        epoch_count: epochs.len(),
        earliest_epoch,
        latest_epoch,
    })
}

impl IndependentHoldoutSnapshotSummary {
    /// Validates that the summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), IndependentHoldoutSnapshotSummaryValidationError> {
        if self.row_count == 0 {
            return Err(IndependentHoldoutSnapshotSummaryValidationError::MissingRows);
        }
        if self.bodies.is_empty() {
            return Err(IndependentHoldoutSnapshotSummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(
                IndependentHoldoutSnapshotSummaryValidationError::BodyCountMismatch {
                    body_count: self.body_count,
                    bodies_len: self.bodies.len(),
                },
            );
        }

        for (index, body) in self.bodies.iter().enumerate() {
            if self.bodies[..index].iter().any(|other| other == body) {
                return Err(
                    IndependentHoldoutSnapshotSummaryValidationError::DuplicateBody {
                        first_index: self.bodies[..index]
                            .iter()
                            .position(|other| other == body)
                            .unwrap(),
                        second_index: index,
                        body: body.clone(),
                    },
                );
            }
        }

        if self.epoch_count == 0 {
            return Err(IndependentHoldoutSnapshotSummaryValidationError::MissingEpochs);
        }
        if self.earliest_epoch.julian_day.days() > self.latest_epoch.julian_day.days() {
            return Err(
                IndependentHoldoutSnapshotSummaryValidationError::InvalidEpochRange {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                },
            );
        }

        if independent_holdout_snapshot_summary().as_ref() != Some(self) {
            return Err(IndependentHoldoutSnapshotSummaryValidationError::DerivedSummaryMismatch);
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let bodies = if self.bodies.is_empty() {
            "none".to_string()
        } else {
            self.bodies.join(", ")
        };
        format!(
            "Independent hold-out coverage: {} rows across {} bodies and {} epochs ({}..{}); bodies: {}",
            self.row_count,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            bodies,
        )
    }

    /// Returns a compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, IndependentHoldoutSnapshotSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for IndependentHoldoutSnapshotSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A single body-window slice inside the independent hold-out snapshot source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct IndependentHoldoutSnapshotSourceWindow {
    /// The hold-out body covered by this window.
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

impl IndependentHoldoutSnapshotSourceWindow {
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

/// Compact release-facing summary for the independent hold-out snapshot source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct IndependentHoldoutSnapshotSourceWindowSummary {
    /// Number of hold-out samples in the source slice.
    pub sample_count: usize,
    /// Bodies covered by the hold-out source slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the source slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the source slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the source slice.
    pub latest_epoch: Instant,
    /// Per-body window breakdown in first-seen order.
    pub windows: Vec<IndependentHoldoutSnapshotSourceWindow>,
}

impl IndependentHoldoutSnapshotSourceWindowSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(IndependentHoldoutSnapshotSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Independent hold-out source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the hold-out source windows still match the checked-in slice.
    pub fn validate(
        &self,
    ) -> Result<(), IndependentHoldoutSnapshotSourceWindowSummaryValidationError> {
        let Some(expected) = independent_holdout_source_window_summary_details() else {
            return Err(
                IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                IndependentHoldoutSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated hold-out source window summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, IndependentHoldoutSnapshotSourceWindowSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

/// Validation error for an independent hold-out source window summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndependentHoldoutSnapshotSourceWindowSummaryValidationError {
    /// A summary field is out of sync with the checked-in hold-out source windows.
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for IndependentHoldoutSnapshotSourceWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the independent hold-out source window summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for IndependentHoldoutSnapshotSourceWindowSummaryValidationError {}

pub(crate) fn independent_holdout_source_window_summary_details(
) -> Option<IndependentHoldoutSnapshotSourceWindowSummary> {
    let entries = independent_holdout_snapshot_entries()?;
    let mut windows = Vec::new();
    for body in independent_holdout_bodies() {
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

        windows.push(IndependentHoldoutSnapshotSourceWindow {
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
        .expect("independent hold-out source windows should not be empty after collection");
    let latest_epoch = windows
        .iter()
        .map(|window| window.latest_epoch)
        .max_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("independent hold-out source windows should not be empty after collection");

    Some(IndependentHoldoutSnapshotSourceWindowSummary {
        sample_count: entries.len(),
        sample_bodies: independent_holdout_bodies().to_vec(),
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

/// Returns the compact typed summary for the independent hold-out source coverage.
pub fn independent_holdout_snapshot_source_window_summary(
) -> Option<IndependentHoldoutSnapshotSourceWindowSummary> {
    independent_holdout_source_window_summary_details()
}

/// Formats the independent hold-out source windows for release-facing reporting.
pub fn format_independent_holdout_snapshot_source_window_summary(
    summary: &IndependentHoldoutSnapshotSourceWindowSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing independent hold-out source window summary string.
pub fn independent_holdout_snapshot_source_window_summary_for_report() -> String {
    match independent_holdout_snapshot_source_window_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Independent hold-out source windows: unavailable ({error})"),
        },
        None => "Independent hold-out source windows: unavailable".to_string(),
    }
}

/// Compact release-facing summary for the independent hold-out quarter-day boundary samples.
#[derive(Clone, Debug, PartialEq)]
pub struct IndependentHoldoutQuarterDayBoundarySummary {
    /// Number of samples in the quarter-day slice.
    pub row_count: usize,
    /// Number of distinct bodies covered by the quarter-day slice.
    pub body_count: usize,
    /// Bodies covered by the quarter-day slice in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs represented by the quarter-day slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the quarter-day slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the quarter-day slice.
    pub latest_epoch: Instant,
}

/// Validation error for an independent hold-out quarter-day boundary summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndependentHoldoutQuarterDayBoundarySummaryValidationError {
    /// A summary field is out of sync with the checked-in quarter-day slice.
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for IndependentHoldoutQuarterDayBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the independent hold-out quarter-day boundary summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for IndependentHoldoutQuarterDayBoundarySummaryValidationError {}

impl IndependentHoldoutQuarterDayBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Independent hold-out quarter-day boundary samples: {} rows across {} bodies and {} epochs (JD 2451915.25 (TDB)..JD 2451915.75 (TDB)); bodies: {}",
            self.row_count,
            self.body_count,
            self.epoch_count,
            format_bodies(&self.bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the checked-in quarter-day slice.
    pub fn validate(
        &self,
    ) -> Result<(), IndependentHoldoutQuarterDayBoundarySummaryValidationError> {
        let Some(expected) = independent_holdout_quarter_day_boundary_summary_details() else {
            return Err(
                IndependentHoldoutQuarterDayBoundarySummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        };

        if self.row_count != expected.row_count {
            return Err(
                IndependentHoldoutQuarterDayBoundarySummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        }
        if self.body_count != expected.body_count {
            return Err(
                IndependentHoldoutQuarterDayBoundarySummaryValidationError::FieldOutOfSync {
                    field: "body_count",
                },
            );
        }
        if self.bodies != expected.bodies {
            return Err(
                IndependentHoldoutQuarterDayBoundarySummaryValidationError::FieldOutOfSync {
                    field: "bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                IndependentHoldoutQuarterDayBoundarySummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                IndependentHoldoutQuarterDayBoundarySummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                IndependentHoldoutQuarterDayBoundarySummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated quarter-day summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, IndependentHoldoutQuarterDayBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

pub(crate) fn independent_holdout_quarter_day_boundary_summary_details(
) -> Option<IndependentHoldoutQuarterDayBoundarySummary> {
    let entries = independent_holdout_snapshot_entries()?;
    let mut bodies = Vec::new();
    let mut epochs = BTreeSet::new();
    let mut earliest_epoch = None;
    let mut latest_epoch = None;
    let mut row_count = 0usize;

    for entry in entries {
        let epoch_days = entry.epoch.julian_day.days();
        if INDEPENDENT_HOLDOUT_QUARTER_DAY_EPOCHS
            .iter()
            .any(|candidate| candidate.to_bits() == epoch_days.to_bits())
        {
            row_count += 1;
            epochs.insert(epoch_days.to_bits());
            earliest_epoch.get_or_insert(entry.epoch);
            latest_epoch = Some(entry.epoch);
            if !bodies.contains(&entry.body) {
                bodies.push(entry.body.clone());
            }
        }
    }

    if row_count == 0 || epochs.len() != 2 || bodies.is_empty() {
        return None;
    }

    Some(IndependentHoldoutQuarterDayBoundarySummary {
        row_count,
        body_count: bodies.len(),
        bodies,
        epoch_count: epochs.len(),
        earliest_epoch: earliest_epoch?,
        latest_epoch: latest_epoch?,
    })
}

/// Returns the compact quarter-day boundary sample summary for release-facing reporting.
pub fn independent_holdout_snapshot_quarter_day_boundary_summary_for_report() -> String {
    match independent_holdout_quarter_day_boundary_summary_details() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Independent hold-out quarter-day boundary samples: unavailable ({error})")
            }
        },
        None => "Independent hold-out quarter-day boundary samples: unavailable".to_string(),
    }
}

/// Compact release-facing summary for the independent hold-out high-curvature window.
#[derive(Clone, Debug, PartialEq)]
pub struct IndependentHoldoutHighCurvatureSummary {
    /// Number of samples in the high-curvature slice.
    pub sample_count: usize,
    /// Bodies covered by the high-curvature slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the high-curvature slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the high-curvature slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the high-curvature slice.
    pub latest_epoch: Instant,
}

/// Validation error for an independent hold-out high-curvature summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndependentHoldoutHighCurvatureSummaryValidationError {
    /// A summary field is out of sync with the checked-in high-curvature slice.
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for IndependentHoldoutHighCurvatureSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the independent hold-out high-curvature summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for IndependentHoldoutHighCurvatureSummaryValidationError {}

impl IndependentHoldoutHighCurvatureSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "JPL independent hold-out high-curvature evidence: {} exact samples across {} bodies and {} epochs ({}..{}); bodies: {}; high-curvature interpolation window",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the checked-in high-curvature slice.
    pub fn validate(&self) -> Result<(), IndependentHoldoutHighCurvatureSummaryValidationError> {
        let Some(expected) = independent_holdout_high_curvature_summary_details() else {
            return Err(
                IndependentHoldoutHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                IndependentHoldoutHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                IndependentHoldoutHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                IndependentHoldoutHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                IndependentHoldoutHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                IndependentHoldoutHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, IndependentHoldoutHighCurvatureSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for IndependentHoldoutHighCurvatureSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn independent_holdout_high_curvature_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            independent_holdout_snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    matches!(
                        entry.body,
                        pleiades_backend::CelestialBody::Sun
                            | pleiades_backend::CelestialBody::Moon
                            | pleiades_backend::CelestialBody::Mercury
                            | pleiades_backend::CelestialBody::Venus
                    ) && (entry.epoch.julian_day.days() == 2_451_915.25
                        || entry.epoch.julian_day.days() == 2_451_915.75)
                })
                .cloned()
                .collect()
        })
        .as_slice();

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

pub(crate) fn independent_holdout_high_curvature_summary_details(
) -> Option<IndependentHoldoutHighCurvatureSummary> {
    let entries = independent_holdout_high_curvature_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in entries {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    let earliest_epoch = entries
        .iter()
        .map(|entry| entry.epoch)
        .min_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect(
            "independent hold-out high-curvature evidence should not be empty after collection",
        );
    let latest_epoch = entries
        .iter()
        .map(|entry| entry.epoch)
        .max_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect(
            "independent hold-out high-curvature evidence should not be empty after collection",
        );

    Some(IndependentHoldoutHighCurvatureSummary {
        sample_count: entries.len(),
        sample_bodies,
        epoch_count: entries
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
    })
}

/// Returns the compact typed summary for the independent hold-out high-curvature window.
pub fn independent_holdout_high_curvature_summary() -> Option<IndependentHoldoutHighCurvatureSummary>
{
    independent_holdout_high_curvature_summary_details()
}

/// Formats the independent hold-out high-curvature window for release-facing reporting.
pub fn format_independent_holdout_high_curvature_summary(
    summary: &IndependentHoldoutHighCurvatureSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing independent hold-out high-curvature summary string.
pub fn independent_holdout_high_curvature_summary_for_report() -> String {
    match independent_holdout_high_curvature_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("JPL independent hold-out high-curvature evidence: unavailable ({error})")
            }
        },
        None => "JPL independent hold-out high-curvature evidence: unavailable".to_string(),
    }
}

/// Compact release-facing summary for any accidental overlap between the checked-in reference snapshot and the independent hold-out snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceHoldoutOverlapSummary {
    /// Number of body/epoch pairs shared by the reference snapshot and the hold-out snapshot.
    pub shared_sample_count: usize,
    /// Shared bodies in first-seen reference order.
    pub shared_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs shared by the reference snapshot and the hold-out snapshot.
    pub shared_epoch_count: usize,
}

/// Structured validation errors for a reference/hold-out overlap summary that drifted from the current snapshots.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceHoldoutOverlapSummaryValidationError {
    /// A summary field is out of sync with the current overlap posture.
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for ReferenceHoldoutOverlapSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference/hold-out overlap summary field `{field}` is out of sync with the current snapshots"
            ),
        }
    }
}

impl std::error::Error for ReferenceHoldoutOverlapSummaryValidationError {}

impl ReferenceHoldoutOverlapSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        if self.shared_sample_count == 0 {
            return "Reference/hold-out overlap: 0 shared body-epoch pairs (reference snapshot and independent hold-out remain disjoint)".to_string();
        }

        format!(
            "Reference/hold-out overlap: {} shared body-epoch pairs across {} bodies and {} epochs; bodies: {}",
            self.shared_sample_count,
            self.shared_bodies.len(),
            self.shared_epoch_count,
            format_bodies(&self.shared_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current overlap posture.
    pub fn validate(&self) -> Result<(), ReferenceHoldoutOverlapSummaryValidationError> {
        let Some(expected) = reference_holdout_overlap_summary_details() else {
            return Err(
                ReferenceHoldoutOverlapSummaryValidationError::FieldOutOfSync {
                    field: "shared_sample_count",
                },
            );
        };

        if self.shared_sample_count != expected.shared_sample_count {
            return Err(
                ReferenceHoldoutOverlapSummaryValidationError::FieldOutOfSync {
                    field: "shared_sample_count",
                },
            );
        }
        if self.shared_bodies != expected.shared_bodies {
            return Err(
                ReferenceHoldoutOverlapSummaryValidationError::FieldOutOfSync {
                    field: "shared_bodies",
                },
            );
        }
        if self.shared_epoch_count != expected.shared_epoch_count {
            return Err(
                ReferenceHoldoutOverlapSummaryValidationError::FieldOutOfSync {
                    field: "shared_epoch_count",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceHoldoutOverlapSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceHoldoutOverlapSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn reference_holdout_overlap_summary_details() -> Option<ReferenceHoldoutOverlapSummary>
{
    let reference_entries = reference_snapshot();
    let holdout_entries = independent_holdout_snapshot_entries()?;
    let mut holdout_pairs = BTreeSet::new();
    for entry in holdout_entries {
        holdout_pairs.insert((
            entry.body.to_string(),
            entry.epoch.julian_day.days().to_bits(),
        ));
    }

    let mut shared_bodies = Vec::new();
    let mut shared_body_seen = BTreeSet::new();
    let mut shared_epochs = BTreeSet::new();
    let mut shared_sample_count = 0usize;

    for body in reference_bodies() {
        let mut body_has_overlap = false;
        for entry in reference_entries.iter().filter(|entry| entry.body == *body) {
            let key = (
                entry.body.to_string(),
                entry.epoch.julian_day.days().to_bits(),
            );
            if holdout_pairs.contains(&key) {
                shared_sample_count += 1;
                body_has_overlap = true;
                shared_epochs.insert(entry.epoch.julian_day.days().to_bits());
            }
        }
        if body_has_overlap && shared_body_seen.insert(body.to_string()) {
            shared_bodies.push(body.clone());
        }
    }

    Some(ReferenceHoldoutOverlapSummary {
        shared_sample_count,
        shared_bodies,
        shared_epoch_count: shared_epochs.len(),
    })
}

/// Returns the compact typed summary for any overlap between the reference snapshot and the independent hold-out snapshot.
pub fn reference_holdout_overlap_summary() -> Option<ReferenceHoldoutOverlapSummary> {
    static SUMMARY: OnceLock<ReferenceHoldoutOverlapSummary> = OnceLock::new();
    Some(
        SUMMARY
            .get_or_init(|| {
                reference_holdout_overlap_summary_details()
                    .expect("reference/hold-out overlap summary should exist")
            })
            .clone(),
    )
}

/// Formats the reference/hold-out overlap summary for release-facing reporting.
pub fn format_reference_holdout_overlap_summary(
    summary: &ReferenceHoldoutOverlapSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing reference/hold-out overlap summary string.
pub fn reference_holdout_overlap_summary_for_report() -> String {
    match validated_reference_holdout_overlap_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Reference/hold-out overlap: unavailable ({error})"),
    }
}

/// Returns the validated release-facing reference/hold-out overlap summary string.
pub fn validated_reference_holdout_overlap_summary_for_report() -> Result<String, String> {
    let summary = reference_holdout_overlap_summary()
        .ok_or_else(|| "reference/hold-out overlap unavailable".to_string())?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

/// Formats the independent hold-out corpus coverage for release-facing reporting.
pub fn format_independent_holdout_snapshot_summary(
    summary: &IndependentHoldoutSnapshotSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing independent hold-out coverage summary string.
pub fn independent_holdout_snapshot_summary_for_report() -> String {
    match independent_holdout_snapshot_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Independent hold-out coverage: unavailable ({error})"),
        },
        None => match independent_holdout_snapshot_error() {
            Some(error) => format!("Independent hold-out coverage: unavailable ({error})"),
            None => "Independent hold-out coverage: unavailable".to_string(),
        },
    }
}

/// A compact body-class coverage summary for the independent hold-out snapshot used by validation.
#[derive(Clone, Debug, PartialEq)]
pub struct IndependentHoldoutSnapshotBodyClassCoverageSummary {
    /// Number of rows in the hold-out snapshot.
    pub row_count: usize,
    /// Bodies covered by the hold-out snapshot in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the hold-out snapshot.
    pub epoch_count: usize,
    /// Per-body windows covered by the hold-out snapshot in first-seen order.
    pub windows: Vec<IndependentHoldoutSnapshotSourceWindow>,
}

/// Validation error for an independent hold-out body-class coverage summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError {
    /// A summary field is out of sync with the checked-in hold-out body-class coverage.
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the independent hold-out body-class coverage summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError {}

impl IndependentHoldoutSnapshotBodyClassCoverageSummary {
    /// Returns a compact body-class summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let windows = self
            .windows
            .iter()
            .map(IndependentHoldoutSnapshotSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");

        format!(
            "Independent hold-out body-class coverage: {} rows across {} bodies and {} epochs; bodies: {}; windows: {}",
            self.row_count,
            self.bodies.len(),
            self.epoch_count,
            format_bodies(&self.bodies),
            windows,
        )
    }

    /// Returns `Ok(())` when the body-class coverage summary still matches the checked-in slice.
    pub fn validate(
        &self,
    ) -> Result<(), IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError> {
        let Some(expected) = independent_holdout_snapshot_body_class_coverage_summary_details()
        else {
            return Err(
                IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        };

        if self.row_count != expected.row_count {
            return Err(
                IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "row_count",
                },
            );
        }
        if self.bodies != expected.bodies {
            return Err(
                IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated body-class coverage summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, IndependentHoldoutSnapshotBodyClassCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for IndependentHoldoutSnapshotBodyClassCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn independent_holdout_snapshot_body_class_coverage_summary_details(
) -> Option<IndependentHoldoutSnapshotBodyClassCoverageSummary> {
    let summary = independent_holdout_snapshot_summary()?;
    let source_windows = independent_holdout_source_window_summary_details()?;

    Some(IndependentHoldoutSnapshotBodyClassCoverageSummary {
        row_count: summary.row_count,
        bodies: independent_holdout_bodies().to_vec(),
        epoch_count: summary.epoch_count,
        windows: source_windows.windows,
    })
}

/// Returns a compact body-class coverage summary for the independent hold-out snapshot used by validation.
pub fn independent_holdout_snapshot_body_class_coverage_summary(
) -> Option<IndependentHoldoutSnapshotBodyClassCoverageSummary> {
    independent_holdout_snapshot_body_class_coverage_summary_details()
}

/// Returns the release-facing body-class coverage summary string for the independent hold-out snapshot.
pub fn independent_holdout_snapshot_body_class_coverage_summary_for_report() -> String {
    match independent_holdout_snapshot_body_class_coverage_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Independent hold-out body-class coverage: unavailable ({error})")
            }
        },
        None => "Independent hold-out body-class coverage: unavailable".to_string(),
    }
}

/// A compact coverage summary for the independent hold-out corpus in mixed-scale
/// batch parity mode.
#[derive(Clone, Debug, PartialEq)]
pub struct IndependentHoldoutSnapshotBatchParitySummary {
    /// Coverage summary for the checked-in hold-out corpus.
    pub snapshot: IndependentHoldoutSnapshotSummary,
    /// Number of TT requests in the batch regression.
    pub tt_request_count: usize,
    /// Number of TDB requests in the batch regression.
    pub tdb_request_count: usize,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
    /// Whether the batch regression preserved request order and batch/single parity.
    pub parity_preserved: bool,
}

/// Returns a compact mixed-scale batch parity summary for the checked-in hold-out corpus.
pub fn independent_holdout_snapshot_batch_parity_summary(
) -> Option<IndependentHoldoutSnapshotBatchParitySummary> {
    let snapshot = independent_holdout_snapshot_summary()?;
    let entries = independent_holdout_snapshot_entries()?;
    let backend = JplSnapshotBackend;
    let requests = independent_holdout_snapshot_batch_parity_requests()?;
    let results = backend.positions(&requests).ok()?;

    if results.len() != requests.len() {
        return None;
    }

    let mut tt_request_count = 0usize;
    let mut tdb_request_count = 0usize;
    let mut exact_count = 0usize;
    let mut interpolated_count = 0usize;
    let mut approximate_count = 0usize;
    let mut unknown_count = 0usize;
    let mut order_preserved = true;
    let mut single_query_parity = true;

    for ((request, result), entry) in requests.iter().zip(results.iter()).zip(entries) {
        let single = backend.position(request).ok();
        single_query_parity &= single.as_ref().is_some_and(|single| single == result);

        order_preserved &= result.body == entry.body
            && result.instant == request.instant
            && result.frame == request.frame
            && result.zodiac_mode == request.zodiac_mode
            && result.apparent == request.apparent;

        match request.instant.scale {
            TimeScale::Tt => tt_request_count += 1,
            TimeScale::Tdb => tdb_request_count += 1,
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

    Some(IndependentHoldoutSnapshotBatchParitySummary {
        snapshot,
        tt_request_count,
        tdb_request_count,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
        parity_preserved: order_preserved && single_query_parity,
    })
}

/// Structured validation errors for an independent hold-out batch parity summary.
#[derive(Clone, Debug, PartialEq)]
pub enum IndependentHoldoutSnapshotBatchParitySummaryValidationError {
    /// The nested hold-out coverage summary failed validation.
    Snapshot(IndependentHoldoutSnapshotSummaryValidationError),
    /// The number of mixed-scale requests does not match the row count.
    RequestCountMismatch {
        /// Number of requests issued on the TT time scale.
        tt_request_count: usize,
        /// Number of requests issued on the TDB time scale.
        tdb_request_count: usize,
        /// Row count carried by the summary under validation.
        row_count: usize,
    },
    /// The mixed-scale batch parity slice collapsed to a single time scale.
    TimeScaleMixMissing {
        /// Number of requests issued on the TT time scale.
        tt_request_count: usize,
        /// Number of requests issued on the TDB time scale.
        tdb_request_count: usize,
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
    /// The batch regression did not preserve request order and single-query parity.
    ParityNotPreserved,
}

impl fmt::Display for IndependentHoldoutSnapshotBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(error) => write!(f, "independent hold-out validation failed: {error}"),
            Self::RequestCountMismatch {
                tt_request_count,
                tdb_request_count,
                row_count,
            } => write!(
                f,
                "request count {}+{} does not match row count {}",
                tt_request_count, tdb_request_count, row_count,
            ),
            Self::TimeScaleMixMissing {
                tt_request_count,
                tdb_request_count,
            } => write!(
                f,
                "time-scale mix must include both TT and TDB requests (TT={}, TDB={})",
                tt_request_count, tdb_request_count,
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
            Self::ParityNotPreserved => f.write_str("batch/single parity was not preserved"),
        }
    }
}

impl std::error::Error for IndependentHoldoutSnapshotBatchParitySummaryValidationError {}

impl IndependentHoldoutSnapshotBatchParitySummary {
    /// Validates that the batch parity summary remains internally consistent.
    pub fn validate(
        &self,
    ) -> Result<(), IndependentHoldoutSnapshotBatchParitySummaryValidationError> {
        self.snapshot
            .validate()
            .map_err(IndependentHoldoutSnapshotBatchParitySummaryValidationError::Snapshot)?;

        if self.tt_request_count + self.tdb_request_count != self.snapshot.row_count {
            return Err(
                IndependentHoldoutSnapshotBatchParitySummaryValidationError::RequestCountMismatch {
                    tt_request_count: self.tt_request_count,
                    tdb_request_count: self.tdb_request_count,
                    row_count: self.snapshot.row_count,
                },
            );
        }

        if self.tt_request_count == 0 || self.tdb_request_count == 0 {
            return Err(
                IndependentHoldoutSnapshotBatchParitySummaryValidationError::TimeScaleMixMissing {
                    tt_request_count: self.tt_request_count,
                    tdb_request_count: self.tdb_request_count,
                },
            );
        }

        if self.exact_count + self.interpolated_count + self.approximate_count + self.unknown_count
            != self.snapshot.row_count
        {
            return Err(
                IndependentHoldoutSnapshotBatchParitySummaryValidationError::QualityCountMismatch {
                    exact_count: self.exact_count,
                    interpolated_count: self.interpolated_count,
                    approximate_count: self.approximate_count,
                    unknown_count: self.unknown_count,
                    row_count: self.snapshot.row_count,
                },
            );
        }

        if !self.parity_preserved {
            return Err(
                IndependentHoldoutSnapshotBatchParitySummaryValidationError::ParityNotPreserved,
            );
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, IndependentHoldoutSnapshotBatchParitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let order = if self.parity_preserved {
            "preserved"
        } else {
            "needs attention"
        };
        format!(
            "JPL independent hold-out batch parity: {} requests across {} bodies ({}) and {} epochs ({}..{}); TT requests={}, TDB requests={}; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; order={}, single-query parity={}",
            self.snapshot.row_count,
            self.snapshot.body_count,
            if self.snapshot.bodies.is_empty() {
                "none".to_string()
            } else {
                self.snapshot.bodies.join(", ")
            },
            self.snapshot.epoch_count,
            format_instant(self.snapshot.earliest_epoch),
            format_instant(self.snapshot.latest_epoch),
            self.tt_request_count,
            self.tdb_request_count,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
            order,
            order,
        )
    }
}

impl fmt::Display for IndependentHoldoutSnapshotBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the independent hold-out mixed-scale batch parity summary for release-facing reporting.
pub fn format_independent_holdout_snapshot_batch_parity_summary(
    summary: &IndependentHoldoutSnapshotBatchParitySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing independent hold-out mixed-scale batch parity summary string.
pub fn independent_holdout_snapshot_batch_parity_summary_for_report() -> String {
    match independent_holdout_snapshot_batch_parity_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("JPL independent hold-out batch parity: unavailable ({error})"),
        },
        None => "JPL independent hold-out batch parity: unavailable".to_string(),
    }
}

/// Returns the validated release-facing independent hold-out mixed-scale batch parity summary string.
pub fn validated_independent_holdout_snapshot_batch_parity_summary_for_report(
) -> Result<String, String> {
    let summary = independent_holdout_snapshot_batch_parity_summary()
        .ok_or_else(|| "JPL independent hold-out batch parity: unavailable".to_string())?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

/// A compact coverage summary for the independent hold-out corpus in
/// equatorial-frame batch parity mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IndependentHoldoutSnapshotEquatorialParitySummary {
    /// Total number of parsed hold-out rows exercised through equatorial requests.
    pub row_count: usize,
    /// Number of distinct bodies covered by the hold-out corpus.
    pub body_count: usize,
    /// Number of distinct epochs covered by the hold-out corpus.
    pub epoch_count: usize,
    /// Earliest epoch represented in the hold-out corpus.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the hold-out corpus.
    pub latest_epoch: Instant,
}

/// Returns a compact equatorial parity summary for the checked-in hold-out corpus.
pub fn independent_holdout_snapshot_equatorial_parity_summary(
) -> Option<IndependentHoldoutSnapshotEquatorialParitySummary> {
    independent_holdout_snapshot_summary().map(|summary| {
        IndependentHoldoutSnapshotEquatorialParitySummary {
            row_count: summary.row_count,
            body_count: summary.body_count,
            epoch_count: summary.epoch_count,
            earliest_epoch: summary.earliest_epoch,
            latest_epoch: summary.latest_epoch,
        }
    })
}

impl IndependentHoldoutSnapshotEquatorialParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, IndependentHoldoutSnapshotEquatorialParitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "JPL independent hold-out equatorial parity: {} rows across {} bodies and {} epochs ({}..{}); mean-obliquity transform against the checked-in ecliptic fixture",
            self.row_count,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
        )
    }
}

/// Structured validation errors for an independent hold-out equatorial parity summary.
#[derive(Clone, Debug, PartialEq)]
pub enum IndependentHoldoutSnapshotEquatorialParitySummaryValidationError {
    /// The summary did not include any rows.
    MissingRows,
    /// The summary did not include any bodies.
    MissingBodies,
    /// The body count exceeds the row count.
    BodyCountExceedsRowCount {
        /// Distinct-body count carried by the summary.
        body_count: usize,
        /// Row count carried by the summary under validation.
        row_count: usize,
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

impl fmt::Display for IndependentHoldoutSnapshotEquatorialParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRows => f.write_str("missing rows"),
            Self::MissingBodies => f.write_str("missing bodies"),
            Self::BodyCountExceedsRowCount {
                body_count,
                row_count,
            } => write!(f, "body count {body_count} exceeds row count {row_count}"),
            Self::MissingEpochs => f.write_str("missing epochs"),
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "invalid epoch range {}..{}",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch)
            ),
        }
    }
}

impl std::error::Error for IndependentHoldoutSnapshotEquatorialParitySummaryValidationError {}

impl IndependentHoldoutSnapshotEquatorialParitySummary {
    /// Validates that the equatorial parity summary remains internally consistent.
    pub fn validate(
        &self,
    ) -> Result<(), IndependentHoldoutSnapshotEquatorialParitySummaryValidationError> {
        if self.row_count == 0 {
            return Err(
                IndependentHoldoutSnapshotEquatorialParitySummaryValidationError::MissingRows,
            );
        }
        if self.body_count == 0 {
            return Err(
                IndependentHoldoutSnapshotEquatorialParitySummaryValidationError::MissingBodies,
            );
        }
        if self.body_count > self.row_count {
            return Err(
                IndependentHoldoutSnapshotEquatorialParitySummaryValidationError::BodyCountExceedsRowCount {
                    body_count: self.body_count,
                    row_count: self.row_count,
                },
            );
        }
        if self.epoch_count == 0 {
            return Err(
                IndependentHoldoutSnapshotEquatorialParitySummaryValidationError::MissingEpochs,
            );
        }
        if self.earliest_epoch.julian_day.days() > self.latest_epoch.julian_day.days() {
            return Err(
                IndependentHoldoutSnapshotEquatorialParitySummaryValidationError::InvalidEpochRange {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for IndependentHoldoutSnapshotEquatorialParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the checked-in independent hold-out equatorial parity summary for
/// release-facing reporting.
pub fn format_independent_holdout_snapshot_equatorial_parity_summary(
    summary: &IndependentHoldoutSnapshotEquatorialParitySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing independent hold-out equatorial parity summary string.
pub fn independent_holdout_snapshot_equatorial_parity_summary_for_report() -> String {
    match independent_holdout_snapshot_equatorial_parity_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("JPL independent hold-out equatorial parity: unavailable ({error})")
            }
        },
        None => "JPL independent hold-out equatorial parity: unavailable".to_string(),
    }
}

pub(crate) fn independent_holdout_source_checksum() -> u64 {
    static CHECKSUM: OnceLock<u64> = OnceLock::new();
    *CHECKSUM.get_or_init(|| {
        checksum64(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/independent_holdout_snapshot.csv"
        )))
    })
}

/// Backend-owned provenance summary for the checked-in hold-out snapshot source material.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndependentHoldoutSourceSummary {
    /// Source attribution for the hold-out snapshot.
    pub source: String,
    /// Evidence class for the hold-out snapshot.
    pub evidence_class: String,
    /// Coverage note for the hold-out snapshot.
    pub coverage: String,
    /// CSV column layout for the hold-out snapshot.
    pub columns: String,
    /// Redistribution posture for the hold-out snapshot.
    pub redistribution: String,
    /// Deterministic checksum of the checked-in hold-out snapshot source material.
    pub checksum: u64,
    /// Frame and coordinate posture described by the checked-in hold-out snapshot.
    pub frame_treatment: String,
    /// Time-scale posture described by the checked-in hold-out snapshot.
    pub time_scale: String,
}

impl IndependentHoldoutSourceSummary {
    /// Validates that the summary remains internally consistent.
    pub fn validate(&self) -> Result<(), IndependentHoldoutSourceSummaryValidationError> {
        if self.source.trim().is_empty() {
            return Err(IndependentHoldoutSourceSummaryValidationError::BlankSource);
        }
        if has_surrounding_whitespace(&self.source) {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "source",
                },
            );
        }
        if self.source != INDEPENDENT_HOLDOUT_SOURCE_EXPECTED {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync { field: "source" },
            );
        }
        if self.evidence_class.trim().is_empty() {
            return Err(IndependentHoldoutSourceSummaryValidationError::BlankEvidenceClass);
        }
        if has_surrounding_whitespace(&self.evidence_class) {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "evidence_class",
                },
            );
        }
        if self.evidence_class != INDEPENDENT_HOLDOUT_EVIDENCE_CLASS {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync {
                    field: "evidence_class",
                },
            );
        }
        if self.coverage.trim().is_empty() {
            return Err(IndependentHoldoutSourceSummaryValidationError::BlankCoverage);
        }
        if has_surrounding_whitespace(&self.coverage) {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "coverage",
                },
            );
        }
        if self.coverage != INDEPENDENT_HOLDOUT_COVERAGE_FALLBACK {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync {
                    field: "coverage",
                },
            );
        }
        if self.columns.trim().is_empty() {
            return Err(IndependentHoldoutSourceSummaryValidationError::BlankColumns);
        }
        if has_surrounding_whitespace(&self.columns) {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "columns",
                },
            );
        }
        if self.columns != INDEPENDENT_HOLDOUT_COLUMNS {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync { field: "columns" },
            );
        }
        if self.redistribution.trim().is_empty() {
            return Err(IndependentHoldoutSourceSummaryValidationError::BlankRedistribution);
        }
        if has_surrounding_whitespace(&self.redistribution) {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "redistribution",
                },
            );
        }
        if self.redistribution != INDEPENDENT_HOLDOUT_REDISTRIBUTION_FALLBACK {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync {
                    field: "redistribution",
                },
            );
        }
        if self.checksum != independent_holdout_source_checksum() {
            return Err(IndependentHoldoutSourceSummaryValidationError::ChecksumMismatch);
        }
        if self.frame_treatment.trim().is_empty() {
            return Err(IndependentHoldoutSourceSummaryValidationError::BlankFrameTreatment);
        }
        if has_surrounding_whitespace(&self.frame_treatment) {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "frame_treatment",
                },
            );
        }
        if self.frame_treatment != INDEPENDENT_HOLDOUT_FRAME_TREATMENT {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync {
                    field: "frame_treatment",
                },
            );
        }
        if self.time_scale.trim().is_empty() {
            return Err(IndependentHoldoutSourceSummaryValidationError::BlankTimeScale);
        }
        if has_surrounding_whitespace(&self.time_scale) {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "time_scale",
                },
            );
        }
        if self.time_scale != INDEPENDENT_HOLDOUT_TIME_SCALE {
            return Err(
                IndependentHoldoutSourceSummaryValidationError::FieldOutOfSync {
                    field: "time_scale",
                },
            );
        }
        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Independent hold-out source: {}; evidence class={}; coverage={}; columns={}; redistribution={}; checksum=0x{:016x}; {}; time scale={}",
            self.source, self.evidence_class, self.coverage, self.columns, self.redistribution, self.checksum, self.frame_treatment, self.time_scale
        )
    }

    /// Returns a compact summary line after validating the hold-out snapshot source summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, IndependentHoldoutSourceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

/// Structured validation errors for a hold-out snapshot provenance summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndependentHoldoutSourceSummaryValidationError {
    /// The summary did not include a non-empty source label.
    BlankSource,
    /// The summary did not include a non-empty evidence-class label.
    BlankEvidenceClass,
    /// The summary did not include a non-empty coverage label.
    BlankCoverage,
    /// The summary did not include a non-empty columns label.
    BlankColumns,
    /// The summary did not include a non-empty frame-treatment label.
    BlankFrameTreatment,
    /// The summary did not include a non-empty time-scale label.
    BlankTimeScale,
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
    /// The summary did not include a non-empty redistribution label.
    BlankRedistribution,
}

impl IndependentHoldoutSourceSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::BlankSource => "blank source",
            Self::BlankEvidenceClass => "blank evidence class",
            Self::BlankCoverage => "blank coverage",
            Self::BlankColumns => "blank columns",
            Self::BlankFrameTreatment => "blank frame treatment",
            Self::BlankTimeScale => "blank time scale",
            Self::SurroundedByWhitespace { .. } => "surrounded by whitespace",
            Self::FieldOutOfSync { .. } => "field out of sync",
            Self::ChecksumMismatch => "checksum mismatch",
            Self::BlankRedistribution => "blank redistribution",
        }
    }
}

impl fmt::Display for IndependentHoldoutSourceSummaryValidationError {
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

impl std::error::Error for IndependentHoldoutSourceSummaryValidationError {}

impl fmt::Display for IndependentHoldoutSourceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned provenance summary for the checked-in hold-out snapshot.
pub fn independent_holdout_source_summary() -> IndependentHoldoutSourceSummary {
    static SUMMARY: OnceLock<IndependentHoldoutSourceSummary> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let manifest = independent_holdout_snapshot_manifest();
            IndependentHoldoutSourceSummary {
                source: manifest
                    .source_or(INDEPENDENT_HOLDOUT_SOURCE_FALLBACK)
                    .to_string(),
                evidence_class: INDEPENDENT_HOLDOUT_EVIDENCE_CLASS.to_string(),
                coverage: manifest
                    .coverage_or(INDEPENDENT_HOLDOUT_COVERAGE_FALLBACK)
                    .to_string(),
                columns: manifest.columns_summary(),
                redistribution: manifest
                    .redistribution_or(INDEPENDENT_HOLDOUT_REDISTRIBUTION_FALLBACK)
                    .to_string(),
                checksum: independent_holdout_source_checksum(),
                frame_treatment: INDEPENDENT_HOLDOUT_FRAME_TREATMENT.to_string(),
                time_scale: INDEPENDENT_HOLDOUT_TIME_SCALE.to_string(),
            }
        })
        .clone()
}

/// Returns the source-material summary for the checked-in hold-out snapshot.
pub fn independent_holdout_source_summary_for_report() -> String {
    if let Err(error) = independent_holdout_snapshot_manifest().validate() {
        return format!("Independent hold-out source: unavailable ({error})");
    }

    let summary = independent_holdout_source_summary();
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Independent hold-out source: unavailable ({error})"),
    }
}

/// Returns the manifest summary for the checked-in hold-out snapshot.
pub fn independent_holdout_manifest_summary() -> SnapshotManifestSummary {
    SnapshotManifestSummary {
        label: "Independent hold-out manifest",
        manifest: independent_holdout_snapshot_manifest().clone(),
        source_fallback: "unknown",
        coverage_fallback: "unknown",
    }
}

/// Returns the manifest summary for the checked-in hold-out snapshot.
pub fn independent_holdout_manifest_summary_for_report() -> String {
    let manifest_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/independent_holdout_snapshot.csv"
    ));
    if let Err(error) = validate_snapshot_manifest_header_structure(
        manifest_text,
        "Independent JPL Horizons hold-out snapshot used only for interpolation validation.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
        "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs.",
        Some("repository-checked regression fixtures, not a broad public corpus."),
        &["epoch_jd", "body", "x_km", "y_km", "z_km"],
    ) {
        return format!("Independent hold-out manifest: unavailable ({error})");
    }

    let summary = independent_holdout_manifest_summary();
    match summary.validate_with_expected_metadata(
        "Independent JPL Horizons hold-out snapshot used only for interpolation validation.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
        "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs.",
        &["epoch_jd", "body", "x_km", "y_km", "z_km"],
    ) {
        Ok(()) => match validate_snapshot_manifest_footprint(
            "independent hold-out snapshot",
            independent_holdout_snapshot_entries(),
            66,
            16,
            12,
        ) {
            Ok(()) => summary.summary_line(),
            Err(error) => format!("Independent hold-out manifest: unavailable ({error})"),
        },
        Err(error) => format!("Independent hold-out manifest: unavailable ({error})"),
    }
}

/// Returns the release-facing interpolation-quality summary for the checked-in
#[derive(Clone, Debug, PartialEq)]
pub struct JplIndependentHoldoutSummary {
    /// Total number of hold-out samples.
    pub sample_count: usize,
    /// Number of distinct bodies represented by the samples.
    pub body_count: usize,
    /// Bodies represented by the samples in first-seen order.
    pub bodies: Vec<String>,
    /// Number of distinct epochs represented by the samples.
    pub epoch_count: usize,
    /// Earliest epoch represented by the samples.
    pub earliest_epoch: Instant,
    /// Latest epoch represented by the samples.
    pub latest_epoch: Instant,
    /// Largest longitude error among the samples.
    pub max_longitude_error_deg: f64,
    /// Body associated with the largest longitude error.
    pub max_longitude_error_body: String,
    /// Held-out epoch associated with the largest longitude error.
    pub max_longitude_error_epoch: Instant,
    /// Mean longitude error across the samples.
    pub mean_longitude_error_deg: f64,
    /// Median longitude error across the samples.
    pub median_longitude_error_deg: f64,
    /// 95th percentile longitude error across the samples.
    pub percentile_longitude_error_deg: f64,
    /// Root-mean-square longitude error across the samples.
    pub rms_longitude_error_deg: f64,
    /// Largest latitude error among the samples.
    pub max_latitude_error_deg: f64,
    /// Body associated with the largest latitude error.
    pub max_latitude_error_body: String,
    /// Held-out epoch associated with the largest latitude error.
    pub max_latitude_error_epoch: Instant,
    /// Mean latitude error across the samples.
    pub mean_latitude_error_deg: f64,
    /// Median latitude error across the samples.
    pub median_latitude_error_deg: f64,
    /// 95th percentile latitude error across the samples.
    pub percentile_latitude_error_deg: f64,
    /// Root-mean-square latitude error across the samples.
    pub rms_latitude_error_deg: f64,
    /// Largest distance error among the samples.
    pub max_distance_error_au: f64,
    /// Body associated with the largest distance error.
    pub max_distance_error_body: String,
    /// Held-out epoch associated with the largest distance error.
    pub max_distance_error_epoch: Instant,
    /// Mean distance error across the samples.
    pub mean_distance_error_au: f64,
    /// Median distance error across the samples.
    pub median_distance_error_au: f64,
    /// 95th percentile distance error across the samples.
    pub percentile_distance_error_au: f64,
    /// Root-mean-square distance error across the samples.
    pub rms_distance_error_au: f64,
}

/// Returns a compact validation summary for the independent hold-out rows.
pub fn jpl_independent_holdout_summary() -> Option<JplIndependentHoldoutSummary> {
    let entries = independent_holdout_snapshot_entries()?;

    let mut bodies = Vec::new();
    let mut seen_bodies = BTreeSet::new();
    let mut epochs = BTreeSet::new();
    let mut earliest_epoch = entries[0].epoch;
    let mut latest_epoch = entries[0].epoch;
    let mut max_longitude_error_deg: f64 = 0.0;
    let mut max_longitude_error_body = String::new();
    let mut max_longitude_error_epoch = entries[0].epoch;
    let mut total_longitude_error_deg = 0.0;
    let mut total_longitude_error_sq_deg = 0.0;
    let mut longitude_errors = Vec::new();
    let mut max_latitude_error_deg: f64 = 0.0;
    let mut max_latitude_error_body = String::new();
    let mut max_latitude_error_epoch = entries[0].epoch;
    let mut total_latitude_error_deg = 0.0;
    let mut total_latitude_error_sq_deg = 0.0;
    let mut latitude_errors = Vec::new();
    let mut max_distance_error_au: f64 = 0.0;
    let mut max_distance_error_body = String::new();
    let mut max_distance_error_epoch = entries[0].epoch;
    let mut total_distance_error_au = 0.0;
    let mut total_distance_error_sq_au = 0.0;
    let mut distance_errors = Vec::new();

    for entry in entries {
        let body = entry.body.to_string();
        if seen_bodies.insert(body.clone()) {
            bodies.push(body);
        }
        epochs.insert(entry.epoch.julian_day.days().to_bits());
        if entry.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = entry.epoch;
        }
        if entry.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = entry.epoch;
        }

        let interpolated = resolve_fixture_state(entry.body.clone(), entry.epoch.julian_day.days())
            .expect("independent hold-out rows should interpolate against the main snapshot")
            .entry;
        let exact_ecliptic = entry.ecliptic();
        let interpolated_ecliptic = interpolated.ecliptic();
        let exact_distance = exact_ecliptic.distance_au.unwrap_or_default();
        let interpolated_distance = interpolated_ecliptic.distance_au.unwrap_or_default();

        let longitude_error = angular_degrees_delta(
            exact_ecliptic.longitude.degrees(),
            interpolated_ecliptic.longitude.degrees(),
        );
        let latitude_error =
            (exact_ecliptic.latitude.degrees() - interpolated_ecliptic.latitude.degrees()).abs();
        let distance_error = (exact_distance - interpolated_distance).abs();

        total_longitude_error_deg += longitude_error;
        total_longitude_error_sq_deg += longitude_error * longitude_error;
        longitude_errors.push(longitude_error);
        total_latitude_error_deg += latitude_error;
        total_latitude_error_sq_deg += latitude_error * latitude_error;
        latitude_errors.push(latitude_error);
        total_distance_error_au += distance_error;
        total_distance_error_sq_au += distance_error * distance_error;
        distance_errors.push(distance_error);

        if longitude_error > max_longitude_error_deg {
            max_longitude_error_deg = longitude_error;
            max_longitude_error_body = entry.body.to_string();
            max_longitude_error_epoch = entry.epoch;
        }
        if latitude_error > max_latitude_error_deg {
            max_latitude_error_deg = latitude_error;
            max_latitude_error_body = entry.body.to_string();
            max_latitude_error_epoch = entry.epoch;
        }
        if distance_error > max_distance_error_au {
            max_distance_error_au = distance_error;
            max_distance_error_body = entry.body.to_string();
            max_distance_error_epoch = entry.epoch;
        }
    }

    let sample_count = entries.len() as f64;

    Some(JplIndependentHoldoutSummary {
        sample_count: entries.len(),
        body_count: bodies.len(),
        bodies,
        epoch_count: epochs.len(),
        earliest_epoch,
        latest_epoch,
        max_longitude_error_deg,
        max_longitude_error_body,
        max_longitude_error_epoch,
        mean_longitude_error_deg: total_longitude_error_deg / sample_count,
        median_longitude_error_deg: median_f64(&mut longitude_errors),
        percentile_longitude_error_deg: percentile_f64(&mut longitude_errors, 0.95),
        rms_longitude_error_deg: (total_longitude_error_sq_deg / sample_count).sqrt(),
        max_latitude_error_deg,
        max_latitude_error_body,
        max_latitude_error_epoch,
        mean_latitude_error_deg: total_latitude_error_deg / sample_count,
        median_latitude_error_deg: median_f64(&mut latitude_errors),
        percentile_latitude_error_deg: percentile_f64(&mut latitude_errors, 0.95),
        rms_latitude_error_deg: (total_latitude_error_sq_deg / sample_count).sqrt(),
        max_distance_error_au,
        max_distance_error_body,
        max_distance_error_epoch,
        mean_distance_error_au: total_distance_error_au / sample_count,
        median_distance_error_au: median_f64(&mut distance_errors),
        percentile_distance_error_au: percentile_f64(&mut distance_errors, 0.95),
        rms_distance_error_au: (total_distance_error_sq_au / sample_count).sqrt(),
    })
}

impl JplIndependentHoldoutSummary {
    /// Returns the compact release-facing independent hold-out summary line.
    pub fn summary_line(&self) -> String {
        fn format_body_epoch_suffix(body: &str, epoch: Instant) -> String {
            if body.is_empty() {
                String::new()
            } else {
                format!(" ({body} @ {})", format_instant(epoch))
            }
        }

        format!(
            "JPL independent hold-out: {} exact rows across {} bodies ({}) and {} epochs ({} → {}); max Δlon={:.12}°{}; mean Δlon={:.12}°; median Δlon={:.12}°; p95 Δlon={:.12}°; rms Δlon={:.12}°; max Δlat={:.12}°{}; mean Δlat={:.12}°; median Δlat={:.12}°; p95 Δlat={:.12}°; rms Δlat={:.12}°; max Δdist={:.12} AU{}; mean Δdist={:.12} AU; median Δdist={:.12} AU; p95 Δdist={:.12} AU; rms Δdist={:.12} AU; transparency evidence only, not a production tolerance envelope; independent JPL Horizons rows held out from the main snapshot corpus",
            self.sample_count,
            self.body_count,
            if self.bodies.is_empty() {
                "none".to_string()
            } else {
                self.bodies.join(", ")
            },
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            self.max_longitude_error_deg,
            format_body_epoch_suffix(&self.max_longitude_error_body, self.max_longitude_error_epoch),
            self.mean_longitude_error_deg,
            self.median_longitude_error_deg,
            self.percentile_longitude_error_deg,
            self.rms_longitude_error_deg,
            self.max_latitude_error_deg,
            format_body_epoch_suffix(&self.max_latitude_error_body, self.max_latitude_error_epoch),
            self.mean_latitude_error_deg,
            self.median_latitude_error_deg,
            self.percentile_latitude_error_deg,
            self.rms_latitude_error_deg,
            self.max_distance_error_au,
            format_body_epoch_suffix(&self.max_distance_error_body, self.max_distance_error_epoch),
            self.mean_distance_error_au,
            self.median_distance_error_au,
            self.percentile_distance_error_au,
            self.rms_distance_error_au,
        )
    }

    /// Returns the validated compact hold-out summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplInterpolationQualitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for JplIndependentHoldoutSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl JplIndependentHoldoutSummary {
    /// Validates that the hold-out summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), JplInterpolationQualitySummaryValidationError> {
        if self.sample_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingSamples);
        }
        if self.body_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BodyCountMismatch {
                    body_count: self.body_count,
                    bodies_len: self.bodies.len(),
                },
            );
        }

        let mut seen_bodies = BTreeSet::new();
        for (index, body) in self.bodies.iter().enumerate() {
            if body.trim().is_empty() {
                return Err(JplInterpolationQualitySummaryValidationError::BlankBody { index });
            }
            if !seen_bodies.insert(body) {
                return Err(
                    JplInterpolationQualitySummaryValidationError::DuplicateBody {
                        body: body.clone(),
                    },
                );
            }
        }

        if self.epoch_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingEpochs);
        }
        if self.earliest_epoch.julian_day.days() > self.latest_epoch.julian_day.days() {
            return Err(
                JplInterpolationQualitySummaryValidationError::InvalidEpochRange {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                },
            );
        }
        for (field, value) in [
            ("max_longitude_error_deg", self.max_longitude_error_deg),
            ("mean_longitude_error_deg", self.mean_longitude_error_deg),
            (
                "median_longitude_error_deg",
                self.median_longitude_error_deg,
            ),
            (
                "percentile_longitude_error_deg",
                self.percentile_longitude_error_deg,
            ),
            ("rms_longitude_error_deg", self.rms_longitude_error_deg),
            ("max_latitude_error_deg", self.max_latitude_error_deg),
            ("mean_latitude_error_deg", self.mean_latitude_error_deg),
            ("median_latitude_error_deg", self.median_latitude_error_deg),
            (
                "percentile_latitude_error_deg",
                self.percentile_latitude_error_deg,
            ),
            ("rms_latitude_error_deg", self.rms_latitude_error_deg),
            ("max_distance_error_au", self.max_distance_error_au),
            ("mean_distance_error_au", self.mean_distance_error_au),
            ("median_distance_error_au", self.median_distance_error_au),
            (
                "percentile_distance_error_au",
                self.percentile_distance_error_au,
            ),
            ("rms_distance_error_au", self.rms_distance_error_au),
        ] {
            validate_non_negative_metric(field, value)?;
        }
        if self.max_longitude_error_deg > 0.0 && self.max_longitude_error_body.trim().is_empty() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_longitude_error_body",
                },
            );
        }
        if self.max_latitude_error_deg > 0.0 && self.max_latitude_error_body.trim().is_empty() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_latitude_error_body",
                },
            );
        }
        if self.max_distance_error_au > 0.0 && self.max_distance_error_body.trim().is_empty() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_distance_error_body",
                },
            );
        }
        if jpl_independent_holdout_summary().as_ref() != Some(self) {
            return Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch);
        }

        Ok(())
    }
}

/// Formats the independent hold-out summary for release-facing reporting.
pub fn format_jpl_independent_holdout_summary(summary: &JplIndependentHoldoutSummary) -> String {
    match summary.validated_summary_line() {
        Ok(rendered) => rendered,
        Err(error) => format!("JPL independent hold-out: unavailable ({error})"),
    }
}

/// Returns the release-facing independent hold-out interpolation summary string.
pub fn jpl_independent_holdout_summary_for_report() -> String {
    match jpl_independent_holdout_summary() {
        Some(summary) => format_jpl_independent_holdout_summary(&summary),
        None => match independent_holdout_snapshot_error() {
            Some(error) => format!("JPL independent hold-out: unavailable ({error})"),
            None => "JPL independent hold-out: unavailable".to_string(),
        },
    }
}

#[cfg(test)]
mod tests;
