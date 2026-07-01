//! reference snapshot core general_b summaries.

use core::fmt;
use std::sync::OnceLock;

use pleiades_types::{Apparentness, CoordinateFrame, Instant, TimeScale, ZodiacMode};

#[allow(unused_imports)]
use crate::reference_summary::*;
#[allow(unused_imports)]
use crate::*;

pub(crate) fn reference_snapshot_2451917_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2451917_MAJOR_BODY_BOUNDARY_EPOCH_JD
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

pub(crate) fn reference_snapshot_2451917_major_body_boundary_summary_details(
) -> Option<Reference2451917MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_2451917_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2451917MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2451917 major-body boundary reference evidence.
pub fn reference_snapshot_2451917_major_body_boundary_summary(
) -> Option<Reference2451917MajorBodyBoundarySummary> {
    reference_snapshot_2451917_major_body_boundary_summary_details()
}

/// Returns the release-facing 2451917 major-body boundary summary string.
pub fn reference_snapshot_2451917_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2451917_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2451917 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451917 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2451919_major_body_boundary_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2451919_MAJOR_BODY_BOUNDARY_EPOCH_JD
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

pub(crate) fn reference_snapshot_2451919_major_body_boundary_summary_details(
) -> Option<Reference2451919MajorBodyBoundarySummary> {
    let evidence = reference_snapshot_2451919_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2451919MajorBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2451919 major-body boundary reference evidence.
pub fn reference_snapshot_2451919_major_body_boundary_summary(
) -> Option<Reference2451919MajorBodyBoundarySummary> {
    reference_snapshot_2451919_major_body_boundary_summary_details()
}

/// Returns the release-facing 2451919 major-body boundary summary string.
pub fn reference_snapshot_2451919_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2451919_major_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2451919 major-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451919 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2451916_major_body_interior_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2451916_MAJOR_BODY_INTERIOR_EPOCH_JD
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

pub(crate) fn reference_snapshot_2451916_major_body_interior_summary_details(
) -> Option<Reference2451916MajorBodyInteriorSummary> {
    let evidence = reference_snapshot_2451916_major_body_interior_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2451916MajorBodyInteriorSummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2451916 major-body interior reference evidence.
pub fn reference_snapshot_2451916_major_body_interior_summary(
) -> Option<Reference2451916MajorBodyInteriorSummary> {
    reference_snapshot_2451916_major_body_interior_summary_details()
}

/// Returns the release-facing 2451916 major-body interior summary string.
pub fn reference_snapshot_2451916_major_body_interior_summary_for_report() -> String {
    match reference_snapshot_2451916_major_body_interior_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2451916 major-body interior evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451916 major-body interior evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2451920_major_body_interior_entries(
) -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    is_comparison_body(&entry.body)
                        && entry.epoch.julian_day.days()
                            == REFERENCE_SNAPSHOT_2451920_MAJOR_BODY_INTERIOR_EPOCH_JD
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

pub(crate) fn reference_snapshot_2451920_major_body_interior_summary_details(
) -> Option<Reference2451920MajorBodyInteriorSummary> {
    let evidence = reference_snapshot_2451920_major_body_interior_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2451920MajorBodyInteriorSummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2451920 major-body interior reference evidence.
pub fn reference_snapshot_2451920_major_body_interior_summary(
) -> Option<Reference2451920MajorBodyInteriorSummary> {
    reference_snapshot_2451920_major_body_interior_summary_details()
}

/// Returns the release-facing 2451920 major-body interior summary string.
pub fn reference_snapshot_2451920_major_body_interior_summary_for_report() -> String {
    match reference_snapshot_2451920_major_body_interior_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2451920 major-body interior evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451920 major-body interior evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_major_body_boundary_window_summary_details(
) -> Option<ReferenceMajorBodyBoundaryWindowSummary> {
    let entries = reference_snapshot_major_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in entries {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    let mut windows = Vec::new();
    for body in &sample_bodies {
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

        windows.push(ReferenceMajorBodyBoundaryWindow {
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
        .expect("reference major-body boundary windows should not be empty after collection");
    let latest_epoch = windows
        .iter()
        .map(|window| window.latest_epoch)
        .max_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("reference major-body boundary windows should not be empty after collection");

    Some(ReferenceMajorBodyBoundaryWindowSummary {
        sample_count: entries.len(),
        sample_bodies,
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

/// Returns the compact typed summary for the major-body boundary-day reference coverage windows.
pub fn reference_snapshot_major_body_boundary_window_summary(
) -> Option<ReferenceMajorBodyBoundaryWindowSummary> {
    static SUMMARY: OnceLock<ReferenceMajorBodyBoundaryWindowSummary> = OnceLock::new();
    Some(
        SUMMARY
            .get_or_init(|| {
                reference_snapshot_major_body_boundary_window_summary_details().expect(
                    "reference major-body boundary windows should not be empty after collection",
                )
            })
            .clone(),
    )
}

/// Returns the release-facing major-body boundary-day window summary string.
pub fn reference_snapshot_major_body_boundary_window_summary_for_report() -> String {
    match reference_snapshot_major_body_boundary_window_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference major-body boundary windows: unavailable ({error})")
            }
        },
        None => "Reference major-body boundary windows: unavailable".to_string(),
    }
}

pub(crate) const REFERENCE_PRE_BRIDGE_BOUNDARY_EPOCH_JD: f64 = 2_451_914.5;

pub(crate) const REFERENCE_DENSE_BOUNDARY_EPOCH_JD: f64 = 2_451_916.5;

pub(crate) const REFERENCE_SPARSE_BOUNDARY_EPOCH_JD: f64 = 2_451_915.5;

pub(crate) fn reference_snapshot_sparse_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| entry.epoch.julian_day.days() == REFERENCE_SPARSE_BOUNDARY_EPOCH_JD)
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

pub(crate) fn reference_snapshot_sparse_boundary_missing_bodies(
    sample_bodies: &[pleiades_backend::CelestialBody],
) -> Vec<pleiades_backend::CelestialBody> {
    reference_bodies()
        .iter()
        .filter(|body| !sample_bodies.contains(body))
        .cloned()
        .collect()
}

/// Compact release-facing summary for the sparse asteroid-only boundary day and its remaining coverage gap inside the reference snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotSparseBoundarySummary {
    /// Number of exact samples in the sparse boundary day.
    pub sample_count: usize,
    /// Bodies covered by the sparse boundary day in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Bodies missing from the sparse boundary day in release order.
    pub missing_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the sparse boundary day.
    pub epoch: Instant,
}

/// Validation errors for a sparse boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceSnapshotSparseBoundarySummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        /// Sample count carried by the summary under validation.
        sample_count: usize,
        /// Sample count recomputed from the current evidence slice.
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        /// Zero-based position in the compared list where the drift was detected.
        index: usize,
        /// Body expected at this position from the current evidence slice.
        expected: pleiades_backend::CelestialBody,
        /// Body recorded in the summary at this position.
        found: pleiades_backend::CelestialBody,
    },
    /// The summary missing-body list drifted from the current evidence slice.
    MissingBodiesMismatch {
        /// Zero-based position in the compared list where the drift was detected.
        index: usize,
        /// Body expected at this position from the current evidence slice.
        expected: pleiades_backend::CelestialBody,
        /// Body recorded in the summary at this position.
        found: pleiades_backend::CelestialBody,
    },
    /// The summary missing-body count drifted from the current evidence slice.
    MissingBodiesCountMismatch {
        /// Count of missing bodies carried by the summary.
        missing_body_count: usize,
        /// Missing-body count recomputed from the current evidence slice.
        derived_missing_body_count: usize,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch {
        /// Epoch derived from the current evidence slice.
        expected: Instant,
        /// Epoch recorded in the summary under validation.
        found: Instant,
    },
}

impl fmt::Display for ReferenceSnapshotSparseBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference snapshot boundary day is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference snapshot boundary day sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference snapshot boundary day body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::MissingBodiesMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference snapshot boundary day missing-body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::MissingBodiesCountMismatch {
                missing_body_count,
                derived_missing_body_count,
            } => write!(
                f,
                "reference snapshot boundary day missing-body count {missing_body_count} does not match derived missing-body count {derived_missing_body_count}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference snapshot boundary day epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceSnapshotSparseBoundarySummaryValidationError {}

impl ReferenceSnapshotSparseBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        if self.missing_bodies.is_empty() {
            format!(
                "Reference snapshot boundary day: {} exact samples at {} ({})",
                self.sample_count,
                format_instant(self.epoch),
                format_bodies(&self.sample_bodies),
            )
        } else {
            format!(
                "Reference snapshot boundary day: {} exact samples at {} ({}); sparse boundary day; missing bodies: {}",
                self.sample_count,
                format_instant(self.epoch),
                format_bodies(&self.sample_bodies),
                format_bodies(&self.missing_bodies),
            )
        }
    }

    /// Returns `Ok(())` when the sparse boundary summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), ReferenceSnapshotSparseBoundarySummaryValidationError> {
        let evidence = reference_snapshot_sparse_boundary_entries()
            .ok_or(ReferenceSnapshotSparseBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceSnapshotSparseBoundarySummaryValidationError::SampleCountMismatch {
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
                        ReferenceSnapshotSparseBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceSnapshotSparseBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let expected_missing_bodies =
            reference_snapshot_sparse_boundary_missing_bodies(&self.sample_bodies);
        if self.missing_bodies.as_slice() != expected_missing_bodies.as_slice() {
            for (index, (expected, found)) in expected_missing_bodies
                .iter()
                .zip(self.missing_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        ReferenceSnapshotSparseBoundarySummaryValidationError::MissingBodiesMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceSnapshotSparseBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.missing_bodies.len() != expected_missing_bodies.len() {
            return Err(
                ReferenceSnapshotSparseBoundarySummaryValidationError::MissingBodiesCountMismatch {
                    missing_body_count: self.missing_bodies.len(),
                    derived_missing_body_count: expected_missing_bodies.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceSnapshotSparseBoundarySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, ReferenceSnapshotSparseBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceSnapshotSparseBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn reference_snapshot_sparse_boundary_summary_details(
) -> Option<ReferenceSnapshotSparseBoundarySummary> {
    let evidence = reference_snapshot_sparse_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    let missing_bodies = reference_snapshot_sparse_boundary_missing_bodies(&sample_bodies);

    Some(ReferenceSnapshotSparseBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        missing_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the boundary day in the reference snapshot.
pub fn reference_snapshot_sparse_boundary_summary() -> Option<ReferenceSnapshotSparseBoundarySummary>
{
    reference_snapshot_sparse_boundary_summary_details()
}

/// Returns the release-facing boundary day summary string.
pub fn reference_snapshot_sparse_boundary_summary_for_report() -> String {
    match reference_snapshot_sparse_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("Reference snapshot boundary day: unavailable ({error})"),
        },
        None => "Reference snapshot boundary day: unavailable".to_string(),
    }
}

/// Compact release-facing summary for the pre-bridge 2451914.5 boundary day in the reference snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotPreBridgeBoundarySummary {
    /// Number of exact samples in the pre-bridge boundary day.
    pub sample_count: usize,
    /// Bodies covered by the pre-bridge boundary day in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the pre-bridge boundary day.
    pub epoch: Instant,
}

/// Validation errors for a pre-bridge boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceSnapshotPreBridgeBoundarySummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        /// Sample count carried by the summary under validation.
        sample_count: usize,
        /// Sample count recomputed from the current evidence slice.
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        /// Zero-based position in the compared list where the drift was detected.
        index: usize,
        /// Body expected at this position from the current evidence slice.
        expected: pleiades_backend::CelestialBody,
        /// Body recorded in the summary at this position.
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch {
        /// Epoch derived from the current evidence slice.
        expected: Instant,
        /// Epoch recorded in the summary under validation.
        found: Instant,
    },
}

impl fmt::Display for ReferenceSnapshotPreBridgeBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference snapshot pre-bridge boundary day is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference snapshot pre-bridge boundary day sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference snapshot pre-bridge boundary day body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference snapshot pre-bridge boundary day epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceSnapshotPreBridgeBoundarySummaryValidationError {}

impl ReferenceSnapshotPreBridgeBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference snapshot pre-bridge boundary day: {} exact samples at {} ({}); pre-bridge boundary day",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the pre-bridge boundary summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), ReferenceSnapshotPreBridgeBoundarySummaryValidationError> {
        let evidence = reference_snapshot_pre_bridge_boundary_entries()
            .ok_or(ReferenceSnapshotPreBridgeBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceSnapshotPreBridgeBoundarySummaryValidationError::SampleCountMismatch {
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
                        ReferenceSnapshotPreBridgeBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceSnapshotPreBridgeBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceSnapshotPreBridgeBoundarySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, ReferenceSnapshotPreBridgeBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceSnapshotPreBridgeBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn reference_snapshot_pre_bridge_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| {
                    entry.epoch.julian_day.days() == REFERENCE_PRE_BRIDGE_BOUNDARY_EPOCH_JD
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

pub(crate) fn reference_snapshot_pre_bridge_boundary_summary_details(
) -> Option<ReferenceSnapshotPreBridgeBoundarySummary> {
    let evidence = reference_snapshot_pre_bridge_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(ReferenceSnapshotPreBridgeBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the pre-bridge boundary day in the reference snapshot.
pub fn reference_snapshot_pre_bridge_boundary_summary(
) -> Option<ReferenceSnapshotPreBridgeBoundarySummary> {
    reference_snapshot_pre_bridge_boundary_summary_details()
}

/// Returns the release-facing pre-bridge boundary day summary string.
pub fn reference_snapshot_pre_bridge_boundary_summary_for_report() -> String {
    match reference_snapshot_pre_bridge_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference snapshot pre-bridge boundary day: unavailable ({error})")
            }
        },
        None => "Reference snapshot pre-bridge boundary day: unavailable".to_string(),
    }
}

/// Returns the compact typed summary for the 2451914 major-body pre-bridge boundary evidence.
pub fn reference_snapshot_2451914_major_body_pre_bridge_summary(
) -> Option<ReferenceSnapshotPreBridgeBoundarySummary> {
    reference_snapshot_pre_bridge_boundary_summary()
}

/// Returns the release-facing 2451914 major-body pre-bridge boundary summary string.
pub fn reference_snapshot_2451914_major_body_pre_bridge_summary_for_report() -> String {
    reference_snapshot_pre_bridge_boundary_summary_for_report()
}

pub(crate) const REFERENCE_BRIDGE_DAY_EPOCH: f64 = 2_451_914.0;

/// Compact release-facing summary for the 2451914.0 bridge day in the reference snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotBridgeDaySummary {
    /// Number of exact samples in the bridge day.
    pub sample_count: usize,
    /// Bodies covered by the bridge day in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the bridge day.
    pub epoch: Instant,
}

/// Validation errors for a bridge-day summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceSnapshotBridgeDaySummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        /// Sample count carried by the summary under validation.
        sample_count: usize,
        /// Sample count recomputed from the current evidence slice.
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        /// Zero-based position in the compared list where the drift was detected.
        index: usize,
        /// Body expected at this position from the current evidence slice.
        expected: pleiades_backend::CelestialBody,
        /// Body recorded in the summary at this position.
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch {
        /// Epoch derived from the current evidence slice.
        expected: Instant,
        /// Epoch recorded in the summary under validation.
        found: Instant,
    },
}

impl fmt::Display for ReferenceSnapshotBridgeDaySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference snapshot bridge day is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference snapshot bridge day sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference snapshot bridge day body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference snapshot bridge day epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceSnapshotBridgeDaySummaryValidationError {}

impl ReferenceSnapshotBridgeDaySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference snapshot bridge day: {} exact samples at {} ({}); bridge sample across the reference boundary window",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), ReferenceSnapshotBridgeDaySummaryValidationError> {
        let evidence = reference_snapshot_bridge_day_entries()
            .ok_or(ReferenceSnapshotBridgeDaySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceSnapshotBridgeDaySummaryValidationError::SampleCountMismatch {
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
                        ReferenceSnapshotBridgeDaySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceSnapshotBridgeDaySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceSnapshotBridgeDaySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, ReferenceSnapshotBridgeDaySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceSnapshotBridgeDaySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the compact typed summary for the 2451914 major-body bridge evidence.
pub fn reference_snapshot_2451914_major_body_bridge_summary(
) -> Option<ReferenceSnapshotBridgeDaySummary> {
    reference_snapshot_bridge_day_summary()
}

/// Returns the release-facing 2451914 major-body bridge summary string.
pub fn reference_snapshot_2451914_major_body_bridge_summary_for_report() -> String {
    reference_snapshot_bridge_day_summary_for_report()
}

/// Returns the compact typed summary for the 2451915 major-body bridge evidence.
pub fn reference_snapshot_2451915_major_body_bridge_summary(
) -> Option<ReferenceMajorBodyBridgeSummary> {
    reference_snapshot_major_body_bridge_summary()
}

/// Returns the release-facing 2451915 major-body bridge summary string.
pub fn reference_snapshot_2451915_major_body_bridge_summary_for_report() -> String {
    match reference_snapshot_2451915_major_body_bridge_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format!(
                "Reference 2451915 major-body bridge evidence: {} exact samples at {} ({}); 2451915 major-body bridge sample",
                summary.sample_count,
                format_instant(summary.epoch),
                format_bodies(&summary.sample_bodies),
            ),
            Err(error) => {
                format!("Reference 2451915 major-body bridge evidence: unavailable ({error})")
            }
        },
        None => "Reference 2451915 major-body bridge evidence: unavailable".to_string(),
    }
}

/// Compact release-facing summary for the dense 2451916.5 boundary day in the reference snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotDenseBoundarySummary {
    /// Number of exact samples in the dense boundary day.
    pub sample_count: usize,
    /// Bodies covered by the dense boundary day in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the dense boundary day.
    pub epoch: Instant,
}

/// Validation errors for a dense boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceSnapshotDenseBoundarySummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        /// Sample count carried by the summary under validation.
        sample_count: usize,
        /// Sample count recomputed from the current evidence slice.
        derived_sample_count: usize,
    },
    /// The summary body list drifted from the current evidence slice.
    BodyOrderMismatch {
        /// Zero-based position in the compared list where the drift was detected.
        index: usize,
        /// Body expected at this position from the current evidence slice.
        expected: pleiades_backend::CelestialBody,
        /// Body recorded in the summary at this position.
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch {
        /// Epoch derived from the current evidence slice.
        expected: Instant,
        /// Epoch recorded in the summary under validation.
        found: Instant,
    },
}

impl fmt::Display for ReferenceSnapshotDenseBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference snapshot dense boundary day is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference snapshot dense boundary day sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference snapshot dense boundary day body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference snapshot dense boundary day epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceSnapshotDenseBoundarySummaryValidationError {}

impl ReferenceSnapshotDenseBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference snapshot dense boundary day: {} exact samples at {} ({}); dense boundary day",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the dense boundary summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), ReferenceSnapshotDenseBoundarySummaryValidationError> {
        let evidence = reference_snapshot_dense_boundary_entries()
            .ok_or(ReferenceSnapshotDenseBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceSnapshotDenseBoundarySummaryValidationError::SampleCountMismatch {
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
                        ReferenceSnapshotDenseBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceSnapshotDenseBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceSnapshotDenseBoundarySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, ReferenceSnapshotDenseBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceSnapshotDenseBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn reference_snapshot_dense_boundary_entries() -> Option<&'static [SnapshotEntry]> {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    let entries = ENTRIES
        .get_or_init(|| {
            snapshot_entries()
                .into_iter()
                .flatten()
                .filter(|entry| entry.epoch.julian_day.days() == REFERENCE_DENSE_BOUNDARY_EPOCH_JD)
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

pub(crate) fn reference_snapshot_dense_boundary_summary_details(
) -> Option<ReferenceSnapshotDenseBoundarySummary> {
    let evidence = reference_snapshot_dense_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(ReferenceSnapshotDenseBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the dense boundary day in the reference snapshot.
pub fn reference_snapshot_dense_boundary_summary() -> Option<ReferenceSnapshotDenseBoundarySummary>
{
    reference_snapshot_dense_boundary_summary_details()
}

/// Returns the release-facing dense boundary day summary string.
pub fn reference_snapshot_dense_boundary_summary_for_report() -> String {
    match reference_snapshot_dense_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference snapshot dense boundary day: unavailable ({error})")
            }
        },
        None => "Reference snapshot dense boundary day: unavailable".to_string(),
    }
}

/// Returns the typed summary for the 2451916.5 dense boundary day in the reference snapshot.
pub fn reference_snapshot_2451916_major_body_dense_boundary_summary(
) -> Option<ReferenceSnapshotDenseBoundarySummary> {
    reference_snapshot_dense_boundary_summary()
}

/// Returns the release-facing 2451916.5 dense boundary day summary string.
pub fn reference_snapshot_2451916_major_body_dense_boundary_summary_for_report() -> String {
    match reference_snapshot_2451916_major_body_dense_boundary_summary() {
        Some(summary) => format!(
            "Reference 2451916 major-body dense boundary evidence: {} exact samples at {} ({}); dense boundary day",
            summary.sample_count,
            format_instant(summary.epoch),
            format_bodies(&summary.sample_bodies),
        ),
        None => "Reference 2451916 major-body dense boundary evidence: unavailable".to_string(),
    }
}

/// Returns the typed summary for the 2451916 major-body boundary reference evidence.
pub fn reference_snapshot_2451916_major_body_boundary_summary(
) -> Option<ReferenceSnapshotDenseBoundarySummary> {
    reference_snapshot_2451916_major_body_dense_boundary_summary()
}

/// Returns the release-facing 2451916 major-body boundary summary string.
pub fn reference_snapshot_2451916_major_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2451916_major_body_boundary_summary() {
        Some(summary) => format!(
            "Reference 2451916 major-body boundary evidence: {} exact samples at {} ({}); dense boundary day",
            summary.sample_count,
            format_instant(summary.epoch),
            format_bodies(&summary.sample_bodies),
        ),
        None => "Reference 2451916 major-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_high_curvature_window_summary_details(
) -> Option<ReferenceHighCurvatureWindowSummary> {
    let entries = reference_snapshot_high_curvature_entries()?;
    let earliest_epoch = entries
        .iter()
        .min_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("reference high-curvature evidence should not be empty after collection");
    let latest_epoch = entries
        .iter()
        .max_by(|left, right| {
            left.epoch
                .julian_day
                .days()
                .total_cmp(&right.epoch.julian_day.days())
        })
        .map(|entry| entry.epoch)
        .expect("reference high-curvature evidence should not be empty after collection");

    let mut sample_bodies = Vec::new();
    let mut windows = Vec::new();
    for entry in entries {
        if sample_bodies.contains(&entry.body) {
            continue;
        }
        let body_entries = entries
            .iter()
            .filter(|candidate| candidate.body == entry.body)
            .collect::<Vec<_>>();
        let body_earliest_epoch = body_entries
            .iter()
            .min_by(|left, right| {
                left.epoch
                    .julian_day
                    .days()
                    .total_cmp(&right.epoch.julian_day.days())
            })
            .map(|candidate| candidate.epoch)
            .expect("reference high-curvature body window should not be empty after collection");
        let body_latest_epoch = body_entries
            .iter()
            .max_by(|left, right| {
                left.epoch
                    .julian_day
                    .days()
                    .total_cmp(&right.epoch.julian_day.days())
            })
            .map(|candidate| candidate.epoch)
            .expect("reference high-curvature body window should not be empty after collection");

        sample_bodies.push(entry.body.clone());
        windows.push(ReferenceHighCurvatureWindow {
            body: entry.body.clone(),
            sample_count: body_entries.len(),
            epoch_count: body_entries
                .iter()
                .map(|candidate| candidate.epoch.julian_day.days().to_bits())
                .collect::<BTreeSet<_>>()
                .len(),
            earliest_epoch: body_earliest_epoch,
            latest_epoch: body_latest_epoch,
        });
    }

    Some(ReferenceHighCurvatureWindowSummary {
        sample_count: entries.len(),
        sample_bodies,
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

/// Returns the compact typed summary for the major-body high-curvature reference coverage.
pub fn reference_snapshot_high_curvature_window_summary(
) -> Option<ReferenceHighCurvatureWindowSummary> {
    reference_snapshot_high_curvature_window_summary_details()
}

/// Returns the release-facing major-body high-curvature window summary string.
pub fn reference_snapshot_high_curvature_window_summary_for_report() -> String {
    match reference_snapshot_high_curvature_window_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference major-body high-curvature windows: unavailable ({error})")
            }
        },
        None => "Reference major-body high-curvature windows: unavailable".to_string(),
    }
}

pub(crate) const REFERENCE_SNAPSHOT_EVIDENCE_CLASS: &str = "reference";

pub(crate) const REFERENCE_SNAPSHOT_SOURCE_EXPECTED: &str =
    "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.";

pub(crate) const REFERENCE_SNAPSHOT_SOURCE_FALLBACK: &str =
    "NASA/JPL Horizons API vector tables (DE441)";

pub(crate) const REFERENCE_SNAPSHOT_COVERAGE_FALLBACK: &str =
    "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; selected bodies sampled at 1900-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2451915.25 and 2451915.75 for Sun, Moon, Mercury, Venus; major bodies sampled at 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, and 2453000.5; major bodies sampled at 2451915.5 for Sun through Pluto; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2378498.5, 2451910.5 through 2451919.5, with 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 boundary coverage, 2003-12-27, 2132-08-31, 2500-01-01, and 2634167; asteroid:99942-Apophis is now also sampled at 2378498.5 and 2451917.5 to complete the selected-asteroid bridge.";

pub(crate) const REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK: &str =
    "repository-checked regression fixtures, not a broad public corpus.";

pub(crate) const REFERENCE_SNAPSHOT_FRAME_TREATMENT: &str = "geocentric ecliptic J2000";

pub(crate) const REFERENCE_SNAPSHOT_TIME_SCALE: &str = "TDB";

pub(crate) const REFERENCE_SNAPSHOT_COLUMNS: &str = "epoch_jd, body, x_km, y_km, z_km";

pub(crate) const INDEPENDENT_HOLDOUT_EVIDENCE_CLASS: &str = "hold-out";

pub(crate) const INDEPENDENT_HOLDOUT_SOURCE_EXPECTED: &str =
    "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.";

pub(crate) const INDEPENDENT_HOLDOUT_SOURCE_FALLBACK: &str =
    "NASA/JPL Horizons API vector tables (DE441)";

pub(crate) const INDEPENDENT_HOLDOUT_COVERAGE_FALLBACK: &str =
    "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs.";

pub(crate) const INDEPENDENT_HOLDOUT_COLUMNS: &str = "epoch_jd, body, x_km, y_km, z_km";

pub(crate) const INDEPENDENT_HOLDOUT_REDISTRIBUTION_FALLBACK: &str =
    "repository-checked regression fixtures, not a broad public corpus.";

pub(crate) const INDEPENDENT_HOLDOUT_FRAME_TREATMENT: &str = "geocentric ecliptic J2000";

pub(crate) const INDEPENDENT_HOLDOUT_TIME_SCALE: &str = "TDB";

pub(crate) fn reference_snapshot_source_checksum() -> u64 {
    static CHECKSUM: OnceLock<u64> = OnceLock::new();
    *CHECKSUM.get_or_init(|| {
        checksum64(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/reference_snapshot.csv"
        )))
    })
}

/// Backend-owned provenance summary for the checked-in reference snapshot source material.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotSourceSummary {
    /// Source attribution for the checked-in reference snapshot.
    pub source: String,
    /// Evidence class for the checked-in reference snapshot.
    pub evidence_class: String,
    /// Body and epoch coverage described by the checked-in reference snapshot.
    pub coverage: String,
    /// Column schema described by the checked-in reference snapshot.
    pub columns: String,
    /// Redistribution posture described by the checked-in reference snapshot.
    pub redistribution: String,
    /// Deterministic checksum of the checked-in reference snapshot source material.
    pub checksum: u64,
    /// Frame and coordinate posture described by the checked-in reference snapshot.
    pub frame_treatment: String,
    /// Time-scale posture described by the checked-in reference snapshot.
    pub time_scale: String,
    /// Reference epoch used by the checked-in snapshot.
    pub reference_epoch: Instant,
}

impl ReferenceSnapshotSourceSummary {
    /// Validates that the summary remains internally consistent.
    pub fn validate(&self) -> Result<(), ReferenceSnapshotSourceSummaryValidationError> {
        if self.source.trim().is_empty() {
            return Err(ReferenceSnapshotSourceSummaryValidationError::BlankSource);
        }
        if has_surrounding_whitespace(&self.source) {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "source",
                },
            );
        }
        if self.source != REFERENCE_SNAPSHOT_SOURCE_EXPECTED {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync { field: "source" },
            );
        }
        if self.evidence_class.trim().is_empty() {
            return Err(ReferenceSnapshotSourceSummaryValidationError::BlankEvidenceClass);
        }
        if has_surrounding_whitespace(&self.evidence_class) {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "evidence_class",
                },
            );
        }
        if self.evidence_class != REFERENCE_SNAPSHOT_EVIDENCE_CLASS {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync {
                    field: "evidence_class",
                },
            );
        }
        if self.coverage.trim().is_empty() {
            return Err(ReferenceSnapshotSourceSummaryValidationError::BlankCoverage);
        }
        if has_surrounding_whitespace(&self.coverage) {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "coverage",
                },
            );
        }
        if self.coverage != REFERENCE_SNAPSHOT_COVERAGE_FALLBACK {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync { field: "coverage" },
            );
        }
        if self.columns.trim().is_empty() {
            return Err(ReferenceSnapshotSourceSummaryValidationError::BlankColumns);
        }
        if has_surrounding_whitespace(&self.columns) {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "columns",
                },
            );
        }
        if self.columns != REFERENCE_SNAPSHOT_COLUMNS {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync { field: "columns" },
            );
        }
        if self.redistribution.trim().is_empty() {
            return Err(ReferenceSnapshotSourceSummaryValidationError::BlankRedistribution);
        }
        if has_surrounding_whitespace(&self.redistribution) {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "redistribution",
                },
            );
        }
        if self.redistribution != REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync {
                    field: "redistribution",
                },
            );
        }
        if self.checksum != reference_snapshot_source_checksum() {
            return Err(ReferenceSnapshotSourceSummaryValidationError::ChecksumMismatch);
        }
        if self.frame_treatment.trim().is_empty() {
            return Err(ReferenceSnapshotSourceSummaryValidationError::BlankFrameTreatment);
        }
        if has_surrounding_whitespace(&self.frame_treatment) {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "frame_treatment",
                },
            );
        }
        if self.frame_treatment != REFERENCE_SNAPSHOT_FRAME_TREATMENT {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync {
                    field: "frame_treatment",
                },
            );
        }
        if self.time_scale.trim().is_empty() {
            return Err(ReferenceSnapshotSourceSummaryValidationError::BlankTimeScale);
        }
        if has_surrounding_whitespace(&self.time_scale) {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::SurroundedByWhitespace {
                    field: "time_scale",
                },
            );
        }
        if self.time_scale != REFERENCE_SNAPSHOT_TIME_SCALE {
            return Err(
                ReferenceSnapshotSourceSummaryValidationError::FieldOutOfSync {
                    field: "time_scale",
                },
            );
        }
        if self.reference_epoch != reference_instant() {
            return Err(ReferenceSnapshotSourceSummaryValidationError::ReferenceEpochMismatch);
        }
        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference snapshot source: {}; evidence class={}; coverage={}; columns={}; redistribution={}; checksum=0x{:016x}; {}; time scale={}; TDB reference epoch {}",
            self.source,
            self.evidence_class,
            self.coverage,
            self.columns,
            self.redistribution,
            self.checksum,
            self.frame_treatment,
            self.time_scale,
            format_instant(self.reference_epoch),
        )
    }

    /// Returns a compact summary line after validating the reference snapshot source summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceSnapshotSourceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

/// Structured validation errors for a reference snapshot provenance summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceSnapshotSourceSummaryValidationError {
    /// The summary did not include a non-empty source label.
    BlankSource,
    /// The summary did not include a non-empty evidence-class label.
    BlankEvidenceClass,
    /// The summary did not include a non-empty coverage label.
    BlankCoverage,
    /// The summary did not include a non-empty frame-treatment label.
    BlankFrameTreatment,
    /// The summary did not include a non-empty time-scale label.
    BlankTimeScale,
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
    /// The summary carried an unexpected reference epoch.
    ReferenceEpochMismatch,
}

impl ReferenceSnapshotSourceSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::BlankSource => "blank source",
            Self::BlankEvidenceClass => "blank evidence class",
            Self::BlankCoverage => "blank coverage",
            Self::BlankFrameTreatment => "blank frame treatment",
            Self::BlankTimeScale => "blank time scale",
            Self::BlankColumns => "blank columns",
            Self::BlankRedistribution => "blank redistribution",
            Self::SurroundedByWhitespace { .. } => "surrounded by whitespace",
            Self::FieldOutOfSync { .. } => "field out of sync",
            Self::ChecksumMismatch => "checksum mismatch",
            Self::ReferenceEpochMismatch => "reference epoch mismatch",
        }
    }
}

impl fmt::Display for ReferenceSnapshotSourceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurroundedByWhitespace { field } => {
                write!(f, "{field} contains surrounding whitespace")
            }
            Self::FieldOutOfSync { field } => write!(f, "{field} is out of sync"),
            Self::ChecksumMismatch => f.write_str("checksum mismatch"),
            Self::ReferenceEpochMismatch => f.write_str("reference epoch mismatch"),
            _ => f.write_str(self.label()),
        }
    }
}

impl std::error::Error for ReferenceSnapshotSourceSummaryValidationError {}

impl fmt::Display for ReferenceSnapshotSourceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned provenance summary for the checked-in reference snapshot.
pub fn reference_snapshot_source_summary() -> ReferenceSnapshotSourceSummary {
    static SUMMARY: OnceLock<ReferenceSnapshotSourceSummary> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let manifest = reference_snapshot_manifest();
            let source = manifest.source_or(REFERENCE_SNAPSHOT_SOURCE_FALLBACK);
            ReferenceSnapshotSourceSummary {
                source: source.to_string(),
                evidence_class: REFERENCE_SNAPSHOT_EVIDENCE_CLASS.to_string(),
                coverage: manifest
                    .coverage_or(REFERENCE_SNAPSHOT_COVERAGE_FALLBACK)
                    .to_string(),
                columns: manifest.columns.join(", "),
                redistribution: manifest
                    .redistribution_or(REFERENCE_SNAPSHOT_REDISTRIBUTION_FALLBACK)
                    .to_string(),
                checksum: reference_snapshot_source_checksum(),
                frame_treatment: REFERENCE_SNAPSHOT_FRAME_TREATMENT.to_string(),
                time_scale: REFERENCE_SNAPSHOT_TIME_SCALE.to_string(),
                reference_epoch: reference_instant(),
            }
        })
        .clone()
}

/// Returns the source-material summary for the checked-in reference snapshot.
pub fn reference_snapshot_source_summary_for_report() -> String {
    if let Err(error) = reference_snapshot_manifest().validate() {
        return format!("Reference snapshot source: unavailable ({error})");
    }

    let summary = reference_snapshot_source_summary();
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Reference snapshot source: unavailable ({error})"),
    }
}

/// A single body-window slice inside the checked-in reference snapshot source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotSourceWindow {
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

impl ReferenceSnapshotSourceWindow {
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

impl fmt::Display for ReferenceSnapshotSourceWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the checked-in reference snapshot source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotSourceWindowSummary {
    /// Number of reference-snapshot samples in the source slice.
    pub sample_count: usize,
    /// Bodies covered by the checked-in source slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the source slice.
    pub epoch_count: usize,
    /// Earliest epoch represented in the source slice.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the source slice.
    pub latest_epoch: Instant,
    /// Per-body window breakdown in first-seen order.
    pub windows: Vec<ReferenceSnapshotSourceWindow>,
}

impl ReferenceSnapshotSourceWindowSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let window_summary = self
            .windows
            .iter()
            .map(ReferenceSnapshotSourceWindow::summary_line)
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Reference snapshot source windows: {} source-backed samples across {} bodies and {} epochs ({}..{}); windows: {}",
            self.sample_count,
            self.sample_bodies.len(),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            window_summary,
        )
    }

    /// Returns `Ok(())` when the reference snapshot source windows still match the checked-in slice.
    pub fn validate(&self) -> Result<(), ReferenceSnapshotSourceWindowSummaryValidationError> {
        let Some(expected) = reference_snapshot_source_window_summary_details() else {
            return Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                ReferenceSnapshotSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated reference snapshot source window summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceSnapshotSourceWindowSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceSnapshotSourceWindowSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a reference snapshot source window summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceSnapshotSourceWindowSummaryValidationError {
    /// A summary field is out of sync with the checked-in reference snapshot source windows.
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for ReferenceSnapshotSourceWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference snapshot source window summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceSnapshotSourceWindowSummaryValidationError {}

pub(crate) fn reference_snapshot_source_window_summary_details(
) -> Option<ReferenceSnapshotSourceWindowSummary> {
    let entries = reference_snapshot();
    if entries.is_empty() {
        return None;
    }

    let mut sample_bodies = Vec::new();
    for entry in entries {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    let mut windows = Vec::new();
    for body in &sample_bodies {
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

        windows.push(ReferenceSnapshotSourceWindow {
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
        .expect("reference snapshot source windows should not be empty after collection");
    let latest_epoch = windows
        .iter()
        .map(|window| window.latest_epoch)
        .max_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("reference snapshot source windows should not be empty after collection");

    Some(ReferenceSnapshotSourceWindowSummary {
        sample_count: entries.len(),
        sample_bodies,
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

/// Returns the compact typed summary for the checked-in reference snapshot source coverage.
pub fn reference_snapshot_source_window_summary() -> Option<ReferenceSnapshotSourceWindowSummary> {
    reference_snapshot_source_window_summary_details()
}

/// Formats the checked-in reference snapshot source windows for release-facing reporting.
pub fn format_reference_snapshot_source_window_summary(
    summary: &ReferenceSnapshotSourceWindowSummary,
) -> String {
    summary.summary_line()
}

pub(crate) fn format_validated_reference_snapshot_source_window_summary_for_report(
    summary: &ReferenceSnapshotSourceWindowSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Reference snapshot source windows: unavailable ({error})"),
    }
}

/// Returns the body-window summary for the checked-in reference snapshot.
pub fn reference_snapshot_source_window_summary_for_report() -> String {
    match reference_snapshot_source_window_summary() {
        Some(summary) => {
            format_validated_reference_snapshot_source_window_summary_for_report(&summary)
        }
        None => "Reference snapshot source windows: unavailable".to_string(),
    }
}

/// Returns the validated body-window summary for the checked-in reference snapshot.
pub fn validated_reference_snapshot_source_window_summary_for_report() -> Result<String, String> {
    let summary = reference_snapshot_source_window_summary()
        .ok_or_else(|| "reference snapshot source windows unavailable".to_string())?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

/// Returns the manifest summary for the checked-in reference snapshot.
pub fn reference_snapshot_manifest_summary() -> SnapshotManifestSummary {
    SnapshotManifestSummary {
        label: "Reference snapshot manifest",
        manifest: reference_snapshot_manifest().clone(),
        source_fallback: "unknown",
        coverage_fallback: "unknown",
    }
}

/// Returns the manifest summary for the checked-in reference snapshot.
pub fn reference_snapshot_manifest_summary_for_report() -> String {
    let manifest_text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/reference_snapshot.csv"
    ));
    if let Err(error) = validate_snapshot_manifest_header_structure(
        manifest_text,
        "JPL Horizons reference snapshot.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
        "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; selected bodies sampled at 1900-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2451915.25 and 2451915.75 for Sun, Moon, Mercury, Venus; major bodies sampled at 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, and 2453000.5; major bodies sampled at 2451915.5 for Sun through Pluto; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2378498.5, 2451910.5 through 2451919.5, with 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 boundary coverage, 2003-12-27, 2132-08-31, 2500-01-01, and 2634167; asteroid:99942-Apophis is now also sampled at 2378498.5 and 2451917.5 to complete the selected-asteroid bridge.",
        Some("repository-checked regression fixtures, not a broad public corpus."),
        &["epoch_jd", "body", "x_km", "y_km", "z_km"],
    ) {
        return format!("Reference snapshot manifest: unavailable ({error})");
    }

    let summary = reference_snapshot_manifest_summary();
    match summary.validate_with_expected_metadata(
        "JPL Horizons reference snapshot.",
        "NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.",
        "major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; selected bodies sampled at 1900-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2451915.25 and 2451915.75 for Sun, Moon, Mercury, Venus; major bodies sampled at 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, and 2453000.5; major bodies sampled at 2451915.5 for Sun through Pluto; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2378498.5, 2451910.5 through 2451919.5, with 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 boundary coverage, 2003-12-27, 2132-08-31, 2500-01-01, and 2634167; asteroid:99942-Apophis is now also sampled at 2378498.5 and 2451917.5 to complete the selected-asteroid bridge.",
        &["epoch_jd", "body", "x_km", "y_km", "z_km"],
    ) {
        Ok(()) => match validate_snapshot_manifest_footprint(
            "reference snapshot",
            snapshot_entries(),
            277,
            16,
            23,
        ) {
            Ok(()) => summary.summary_line(),
            Err(error) => format!("Reference snapshot manifest: unavailable ({error})"),
        },
        Err(error) => format!("Reference snapshot manifest: unavailable ({error})"),
    }
}

pub(crate) const CHECKED_IN_SNAPSHOT_SCHEMA_COLUMNS: [&str; 5] =
    ["epoch_jd", "body", "x_km", "y_km", "z_km"];

pub(crate) const JPL_SNAPSHOT_EVIDENCE_CLASSIFICATION_SUMMARY: &str = "JPL evidence classification: release-tolerance=reference/comparison/production-generation validation summaries; hold-out=independent hold-out rows and interpolation-quality summaries; fixture exactness=reference snapshot exact J2000 evidence; provenance-only=source and manifest summaries";

pub(crate) const JPL_SOURCE_POSTURE_SUMMARY: &str = "JPL source posture: documented hybrid snapshot/hold-out fixture backend with a separate generation-input path; pure-Rust include_str! ingestion and reusable CSV parsing entry points; not a broad public reader/corpus provider";

pub(crate) const JPL_PROVENANCE_ONLY_SUMMARY: &str = "JPL provenance-only evidence: source and manifest summaries are provenance-only evidence; they validate corpus provenance and checksum posture but are excluded from tolerance, hold-out, and fixture-exactness claims";

pub(crate) const JPL_SNAPSHOT_REQUEST_POLICY: JplSnapshotRequestPolicy = JplSnapshotRequestPolicy {
    supported_frames: &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
    supported_time_scales: &[TimeScale::Tt, TimeScale::Tdb],
    supported_zodiac_modes: &[ZodiacMode::Tropical],
    supported_apparentness: &[Apparentness::Mean],
    supports_topocentric_observer: false,
};

pub(crate) const JPL_INTERPOLATION_POSTURE_SOURCE: &str =
    "leave-one-out runtime interpolation evidence derived from the checked-in reference snapshot";

pub(crate) const JPL_INTERPOLATION_POSTURE_DETAIL: &str = "transparency evidence only";

pub(crate) const JPL_INTERPOLATION_POSTURE_ENVELOPE: &str = "not a production tolerance envelope";

pub(crate) fn median_f64(values: &mut [f64]) -> f64 {
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

pub(crate) fn percentile_f64(values: &mut [f64], percentile: f64) -> f64 {
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let percentile = percentile.clamp(0.0, 1.0);
    if values.len() == 1 {
        return values[0];
    }

    let position = percentile * (values.len() - 1) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;
    if lower_index == upper_index {
        values[lower_index]
    } else {
        let weight = position - lower_index as f64;
        values[lower_index] * (1.0 - weight) + values[upper_index] * weight
    }
}

pub(crate) const JPL_INTERPOLATION_QUALITY_DERIVATION: &str =
    "leave-one-out interpolation evidence derived from the checked-in reference snapshot";
