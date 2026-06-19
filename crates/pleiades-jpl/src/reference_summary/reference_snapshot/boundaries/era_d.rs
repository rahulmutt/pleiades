//! reference snapshot boundary summaries (era_d).

use core::fmt;
use std::sync::OnceLock;

use pleiades_types::Instant;

#[allow(unused_imports)]
use crate::reference_summary::*;
#[allow(unused_imports)]
use crate::*;

impl Reference2451917MajorBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 2451917 major-body boundary evidence: {} exact samples at {} ({}); 2001-01-08 boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference2451917MajorBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_2451917_major_body_boundary_entries()
            .ok_or(Reference2451917MajorBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference2451917MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let mut expected_bodies = Vec::new();
        for entry in evidence {
            if !expected_bodies.contains(&entry.body) {
                expected_bodies.push(entry.body.clone());
            }
        }
        if self.sample_bodies.as_slice() != expected_bodies.as_slice() {
            for (index, (expected, found)) in expected_bodies
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        Reference2451917MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference2451917MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference2451917MajorBodyBoundarySummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Reference2451917MajorBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference2451917MajorBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the 2451919.5 major-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference2451919MajorBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a 2451919 major-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference2451919MajorBodyBoundarySummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
}

impl fmt::Display for Reference2451919MajorBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference 2451919 major-body boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 2451919 major-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 2451919 major-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 2451919 major-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference2451919MajorBodyBoundarySummaryValidationError {}

impl Reference2451919MajorBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 2451919 major-body boundary evidence: {} exact samples at {} ({}); 2001-01-10 boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference2451919MajorBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_2451919_major_body_boundary_entries()
            .ok_or(Reference2451919MajorBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference2451919MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let mut expected_bodies = Vec::new();
        for entry in evidence {
            if !expected_bodies.contains(&entry.body) {
                expected_bodies.push(entry.body.clone());
            }
        }
        if self.sample_bodies.as_slice() != expected_bodies.as_slice() {
            for (index, (expected, found)) in expected_bodies
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        Reference2451919MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference2451919MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference2451919MajorBodyBoundarySummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Reference2451919MajorBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference2451919MajorBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the 2451916.0 major-body interior reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference2451916MajorBodyInteriorSummary {
    /// Number of exact samples in the interior slice.
    pub sample_count: usize,
    /// Bodies covered by the interior slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the interior slice.
    pub epoch: Instant,
}

/// Validation errors for a 2451916 major-body interior summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference2451916MajorBodyInteriorSummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
}

impl fmt::Display for Reference2451916MajorBodyInteriorSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference 2451916 major-body interior evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 2451916 major-body interior evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 2451916 major-body interior evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 2451916 major-body interior evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference2451916MajorBodyInteriorSummaryValidationError {}

impl Reference2451916MajorBodyInteriorSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 2451916 major-body interior evidence: {} exact samples at {} ({}); 2001-01-06 interior reference sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference2451916MajorBodyInteriorSummaryValidationError> {
        let evidence = reference_snapshot_2451916_major_body_interior_entries()
            .ok_or(Reference2451916MajorBodyInteriorSummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference2451916MajorBodyInteriorSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let mut expected_bodies = Vec::new();
        for entry in evidence {
            if !expected_bodies.contains(&entry.body) {
                expected_bodies.push(entry.body.clone());
            }
        }
        if self.sample_bodies.as_slice() != expected_bodies.as_slice() {
            for (index, (expected, found)) in expected_bodies
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        Reference2451916MajorBodyInteriorSummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference2451916MajorBodyInteriorSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference2451916MajorBodyInteriorSummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Reference2451916MajorBodyInteriorSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference2451916MajorBodyInteriorSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the 2451920.5 major-body interior reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference2451920MajorBodyInteriorSummary {
    /// Number of exact samples in the interior slice.
    pub sample_count: usize,
    /// Bodies covered by the interior slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the interior slice.
    pub epoch: Instant,
}

/// Validation errors for a 2451920 major-body interior summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference2451920MajorBodyInteriorSummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
}

impl fmt::Display for Reference2451920MajorBodyInteriorSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference 2451920 major-body interior evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 2451920 major-body interior evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 2451920 major-body interior evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 2451920 major-body interior evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference2451920MajorBodyInteriorSummaryValidationError {}

impl Reference2451920MajorBodyInteriorSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 2451920 major-body interior evidence: {} exact samples at {} ({}); 2001-01-13 interior reference sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference2451920MajorBodyInteriorSummaryValidationError> {
        let evidence = reference_snapshot_2451920_major_body_interior_entries()
            .ok_or(Reference2451920MajorBodyInteriorSummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference2451920MajorBodyInteriorSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let mut expected_bodies = Vec::new();
        for entry in evidence {
            if !expected_bodies.contains(&entry.body) {
                expected_bodies.push(entry.body.clone());
            }
        }
        if self.sample_bodies.as_slice() != expected_bodies.as_slice() {
            for (index, (expected, found)) in expected_bodies
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        Reference2451920MajorBodyInteriorSummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference2451920MajorBodyInteriorSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference2451920MajorBodyInteriorSummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }

        Ok(())
    }

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Reference2451920MajorBodyInteriorSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference2451920MajorBodyInteriorSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A single body-window slice inside the major-body boundary-day reference coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceMajorBodyBoundaryWindow {
    /// The body covered by this window.
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

impl ReferenceMajorBodyBoundaryWindow {
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

impl fmt::Display for ReferenceMajorBodyBoundaryWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the major-body boundary-day reference coverage windows.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceMajorBodyBoundaryWindowSummary {
    /// Number of samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the boundary slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the boundary slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the boundary slice.
    pub latest_epoch: Instant,
    /// Per-body window breakdown in first-seen order.
    pub windows: Vec<ReferenceMajorBodyBoundaryWindow>,
}

/// Validation errors for a major-body boundary window summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceMajorBodyBoundaryWindowSummaryValidationError {
    /// The summary no longer matches the checked-in boundary window slice.
    DerivedSummaryMismatch,
}

impl fmt::Display for ReferenceMajorBodyBoundaryWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DerivedSummaryMismatch => write!(
                f,
                "the reference major-body boundary window summary is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceMajorBodyBoundaryWindowSummaryValidationError {}

impl ReferenceMajorBodyBoundaryWindowSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(ReferenceMajorBodyBoundaryWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");

        format!(
            "Reference major-body boundary windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current boundary window slice.
    pub fn validate(&self) -> Result<(), ReferenceMajorBodyBoundaryWindowSummaryValidationError> {
        let Some(expected) = reference_snapshot_major_body_boundary_window_summary_details() else {
            return Err(
                ReferenceMajorBodyBoundaryWindowSummaryValidationError::DerivedSummaryMismatch,
            );
        };

        if self != &expected {
            return Err(
                ReferenceMajorBodyBoundaryWindowSummaryValidationError::DerivedSummaryMismatch,
            );
        }

        Ok(())
    }

    /// Returns the validated summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceMajorBodyBoundaryWindowSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceMajorBodyBoundaryWindowSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn reference_snapshot_bridge_day_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| entry.epoch.julian_day.days() == REFERENCE_BRIDGE_DAY_EPOCH)
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

pub(crate) fn reference_snapshot_bridge_day_summary_details(
) -> Option<ReferenceSnapshotBridgeDaySummary> {
    let evidence = reference_snapshot_bridge_day_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(ReferenceSnapshotBridgeDaySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the bridge day in the reference snapshot.
pub fn reference_snapshot_bridge_day_summary() -> Option<ReferenceSnapshotBridgeDaySummary> {
    reference_snapshot_bridge_day_summary_details()
}

/// Returns the release-facing bridge day summary string.
pub fn reference_snapshot_bridge_day_summary_for_report() -> String {
    match reference_snapshot_bridge_day_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Reference snapshot bridge day: unavailable ({error})"),
        },
        None => "Reference snapshot bridge day: unavailable".to_string(),
    }
}

/// Returns the validated release-facing bridge day summary string.
pub fn validated_reference_snapshot_bridge_day_summary_for_report() -> Result<String, String> {
    let summary = reference_snapshot_bridge_day_summary()
        .ok_or_else(|| "reference snapshot bridge day unavailable".to_string())?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

/// Returns the compact typed summary for the 2451914 bridge-day evidence.
pub fn reference_snapshot_2451914_bridge_day_summary() -> Option<ReferenceSnapshotBridgeDaySummary>
{
    reference_snapshot_bridge_day_summary()
}

/// Returns the release-facing 2451914 bridge-day summary string.
pub fn reference_snapshot_2451914_bridge_day_summary_for_report() -> String {
    reference_snapshot_bridge_day_summary_for_report()
}

/// Returns the compact typed summary for the 2451914 major-body bridge-day evidence.
pub fn reference_snapshot_2451914_major_body_bridge_day_summary(
) -> Option<ReferenceSnapshotBridgeDaySummary> {
    reference_snapshot_bridge_day_summary()
}

/// Returns the release-facing 2451914 major-body bridge-day summary string.
pub fn reference_snapshot_2451914_major_body_bridge_day_summary_for_report() -> String {
    match reference_snapshot_2451914_major_body_bridge_day_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format!(
                "Reference 2451914 major-body bridge-day evidence: {} exact samples at {} ({}); 2451914 major-body bridge-day sample",
                summary.sample_count,
                format_instant(summary.epoch),
                format_bodies(&summary.sample_bodies),
            ),
            Err(error) => {
                format!("Reference 2451914 major-body bridge-day evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451914 major-body bridge-day evidence: unavailable".to_string(),
    }
}

/// A single body-window slice inside the major-body high-curvature reference coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceHighCurvatureWindow {
    /// The body covered by this window.
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

impl ReferenceHighCurvatureWindow {
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

impl fmt::Display for ReferenceHighCurvatureWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the major-body high-curvature reference coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceHighCurvatureWindowSummary {
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
    /// Per-body window breakdown in first-seen order.
    pub windows: Vec<ReferenceHighCurvatureWindow>,
}

impl ReferenceHighCurvatureWindowSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(ReferenceHighCurvatureWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Reference major-body high-curvature windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the high-curvature window summary still matches the checked-in slice.
    pub fn validate(&self) -> Result<(), ReferenceHighCurvatureWindowSummaryValidationError> {
        let Some(expected) = reference_snapshot_high_curvature_window_summary_details() else {
            return Err(
                ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                ReferenceHighCurvatureWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated high-curvature window summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceHighCurvatureWindowSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceHighCurvatureWindowSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a high-curvature window summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceHighCurvatureWindowSummaryValidationError {
    /// A summary field is out of sync with the checked-in high-curvature windows.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ReferenceHighCurvatureWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference high-curvature window summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceHighCurvatureWindowSummaryValidationError {}

/// A single epoch slice inside the major-body high-curvature reference coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceHighCurvatureEpochCoverage {
    /// The epoch covered by this slice.
    pub epoch: Instant,
    /// Number of bodies covered at the epoch.
    pub body_count: usize,
    /// Bodies covered by the epoch slice in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
}

impl ReferenceHighCurvatureEpochCoverage {
    /// Returns a compact epoch-coverage summary used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: {} bodies ({})",
            format_instant(self.epoch),
            self.body_count,
            format_bodies(&self.bodies),
        )
    }
}

impl fmt::Display for ReferenceHighCurvatureEpochCoverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the major-body high-curvature epoch coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceHighCurvatureEpochCoverageSummary {
    /// Number of exact samples in the high-curvature slice.
    pub sample_count: usize,
    /// Number of distinct epochs covered by the high-curvature slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the high-curvature slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the high-curvature slice.
    pub latest_epoch: Instant,
    /// Per-epoch body breakdown in first-seen order.
    pub windows: Vec<ReferenceHighCurvatureEpochCoverage>,
}

impl ReferenceHighCurvatureEpochCoverageSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(ReferenceHighCurvatureEpochCoverage::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Reference major-body high-curvature epoch coverage: {} exact samples across {} epochs ({}..{}); epochs: {}",
            self.sample_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the epoch coverage summary still matches the checked-in slice.
    pub fn validate(
        &self,
    ) -> Result<(), ReferenceHighCurvatureEpochCoverageSummaryValidationError> {
        let Some(expected) = reference_snapshot_high_curvature_epoch_coverage_summary_details()
        else {
            return Err(
                ReferenceHighCurvatureEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ReferenceHighCurvatureEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ReferenceHighCurvatureEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ReferenceHighCurvatureEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ReferenceHighCurvatureEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                ReferenceHighCurvatureEpochCoverageSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated epoch coverage summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceHighCurvatureEpochCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceHighCurvatureEpochCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a high-curvature epoch summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceHighCurvatureEpochCoverageSummaryValidationError {
    /// A summary field is out of sync with the checked-in epoch coverage.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ReferenceHighCurvatureEpochCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference high-curvature epoch coverage summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceHighCurvatureEpochCoverageSummaryValidationError {}
