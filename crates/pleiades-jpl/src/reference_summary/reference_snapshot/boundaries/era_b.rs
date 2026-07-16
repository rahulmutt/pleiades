//! reference snapshot boundary summaries (era_b).

use core::fmt;
use std::sync::OnceLock;

use pleiades_types::Instant;

#[allow(unused_imports)]
use crate::reference_summary::*;
#[allow(unused_imports)]
use crate::*;

pub(crate) fn reference_snapshot_1900_selected_body_boundary_entries(
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
                            == REFERENCE_SNAPSHOT_1900_SELECTED_BODY_BOUNDARY_EPOCH_JD
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

/// Compact release-facing summary for the 1900-01-01 selected-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference1900SelectedBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a 1900 selected-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference1900SelectedBodyBoundarySummaryValidationError {
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

impl fmt::Display for Reference1900SelectedBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => {
                f.write_str("reference 1900 selected-body boundary evidence is unavailable")
            }
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 1900 selected-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 1900 selected-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 1900 selected-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference1900SelectedBodyBoundarySummaryValidationError {}

impl Reference1900SelectedBodyBoundarySummary {
    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference1900SelectedBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_1900_selected_body_boundary_entries()
            .ok_or(Reference1900SelectedBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference1900SelectedBodyBoundarySummaryValidationError::SampleCountMismatch {
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
                        Reference1900SelectedBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference1900SelectedBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference1900SelectedBodyBoundarySummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }

        Ok(())
    }
}

pub(crate) fn reference_snapshot_1900_selected_body_boundary_summary_details(
) -> Option<Reference1900SelectedBodyBoundarySummary> {
    let evidence = reference_snapshot_1900_selected_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference1900SelectedBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 1900 selected-body boundary reference evidence.
pub fn reference_snapshot_1900_selected_body_boundary_summary(
) -> Option<Reference1900SelectedBodyBoundarySummary> {
    reference_snapshot_1900_selected_body_boundary_summary_details()
}

/// Returns the compact typed summary for the 2415020 selected-body boundary reference evidence.
#[doc(alias = "reference_snapshot_1900_selected_body_boundary_summary")]
pub fn reference_snapshot_2415020_selected_body_boundary_summary(
) -> Option<Reference1900SelectedBodyBoundarySummary> {
    reference_snapshot_1900_selected_body_boundary_summary()
}

/// Compact release-facing summary for the 2453000.5 major-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference2453000MajorBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}
