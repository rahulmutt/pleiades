//! reference snapshot boundary summaries (era_b).

use core::fmt;
use std::sync::OnceLock;

use pleiades_types::Instant;

#[allow(unused_imports)]
use crate::reference_summary::*;
#[allow(unused_imports)]
use crate::*;

/// Returns the release-facing 1750 selected-body boundary summary string.
pub fn reference_snapshot_1750_selected_body_boundary_summary_for_report() -> String {
    match reference_snapshot_1750_selected_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 1750 selected-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 1750 selected-body boundary evidence: unavailable".to_string(),
    }
}

/// Compact release-facing summary for the 1750-01-01 major-body interior comparison reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference1750MajorBodyInteriorSummary {
    /// Number of exact samples in the interior comparison slice.
    pub sample_count: usize,
    /// Bodies covered by the interior comparison slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the interior comparison slice.
    pub epoch: Instant,
}

/// Validation errors for a 1750 major-body interior comparison summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference1750MajorBodyInteriorSummaryValidationError {
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

impl fmt::Display for Reference1750MajorBodyInteriorSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => {
                f.write_str("reference 1750 major-body interior comparison evidence is unavailable")
            }
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 1750 major-body interior comparison evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 1750 major-body interior comparison evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 1750 major-body interior comparison evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference1750MajorBodyInteriorSummaryValidationError {}

impl Reference1750MajorBodyInteriorSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 1750 major-body interior comparison evidence: {} exact samples at {} ({}); 1750-01-01 interior comparison sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference1750MajorBodyInteriorSummaryValidationError> {
        let evidence = reference_snapshot_1750_major_body_interior_entries()
            .ok_or(Reference1750MajorBodyInteriorSummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference1750MajorBodyInteriorSummaryValidationError::SampleCountMismatch {
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
                        Reference1750MajorBodyInteriorSummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference1750MajorBodyInteriorSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference1750MajorBodyInteriorSummaryValidationError::EpochMismatch {
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
    ) -> Result<String, Reference1750MajorBodyInteriorSummaryValidationError> {
        self.validate().map(|()| self.summary_line())
    }
}

impl fmt::Display for Reference1750MajorBodyInteriorSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the 2360234.5 major-body interior comparison reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference2360234MajorBodyInteriorSummary {
    /// Number of exact samples in the interior comparison slice.
    pub sample_count: usize,
    /// Bodies covered by the interior comparison slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the interior comparison slice.
    pub epoch: Instant,
}

/// Validation errors for a 2360234 major-body interior comparison summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference2360234MajorBodyInteriorSummaryValidationError {
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

impl fmt::Display for Reference2360234MajorBodyInteriorSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => {
                f.write_str("reference 2360234 major-body interior comparison evidence is unavailable")
            }
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 2360234 major-body interior comparison evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 2360234 major-body interior comparison evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 2360234 major-body interior comparison evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference2360234MajorBodyInteriorSummaryValidationError {}

impl Reference2360234MajorBodyInteriorSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 2360234 major-body interior comparison evidence: {} exact samples at {} ({}); 1750-01-01 interior comparison sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference2360234MajorBodyInteriorSummaryValidationError> {
        let evidence = reference_snapshot_2360234_major_body_interior_entries()
            .ok_or(Reference2360234MajorBodyInteriorSummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference2360234MajorBodyInteriorSummaryValidationError::SampleCountMismatch {
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
                        Reference2360234MajorBodyInteriorSummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference2360234MajorBodyInteriorSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference2360234MajorBodyInteriorSummaryValidationError::EpochMismatch {
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
    ) -> Result<String, Reference2360234MajorBodyInteriorSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference2360234MajorBodyInteriorSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

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
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 1900 selected-body boundary evidence: {} exact samples at {} ({}); 1900-01-01 selected-body boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

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

    /// Returns the compact summary line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Reference1900SelectedBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference1900SelectedBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
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

/// Returns the release-facing 1900 selected-body boundary summary string.
pub fn reference_snapshot_1900_selected_body_boundary_summary_for_report() -> String {
    match reference_snapshot_1900_selected_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 1900 selected-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 1900 selected-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the compact typed summary for the 2415020 selected-body boundary reference evidence.
#[doc(alias = "reference_snapshot_1900_selected_body_boundary_summary")]
pub fn reference_snapshot_2415020_selected_body_boundary_summary(
) -> Option<Reference1900SelectedBodyBoundarySummary> {
    reference_snapshot_1900_selected_body_boundary_summary()
}

/// Returns the release-facing 2415020 selected-body boundary summary string.
pub fn reference_snapshot_2415020_selected_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2415020_selected_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(_) => format_selected_body_boundary_summary_line(
                "2415020",
                summary.sample_count,
                &summary.sample_bodies,
                summary.epoch,
                "1900-01-01",
            ),
            Err(error) => {
                format!("Reference 2415020 selected-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2415020 selected-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2200_selected_body_boundary_entries(
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
                            == REFERENCE_SNAPSHOT_2200_SELECTED_BODY_BOUNDARY_EPOCH_JD
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

/// Compact release-facing summary for the 2200-01-01 selected-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference2200SelectedBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a 2200 selected-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference2200SelectedBodyBoundarySummaryValidationError {
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

impl fmt::Display for Reference2200SelectedBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => {
                f.write_str("reference 2200 selected-body boundary evidence is unavailable")
            }
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 2200 selected-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 2200 selected-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 2200 selected-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference2200SelectedBodyBoundarySummaryValidationError {}

pub(crate) fn format_selected_body_boundary_summary_line(
    epoch_label: &str,
    sample_count: usize,
    sample_bodies: &[pleiades_backend::CelestialBody],
    epoch: Instant,
    sample_label: &str,
) -> String {
    format!(
        "Reference {epoch_label} selected-body boundary evidence: {} exact samples at {} ({}); {sample_label} selected-body boundary sample",
        sample_count,
        format_instant(epoch),
        format_bodies(sample_bodies),
    )
}

impl Reference2200SelectedBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format_selected_body_boundary_summary_line(
            "2200",
            self.sample_count,
            &self.sample_bodies,
            self.epoch,
            "2200-01-01",
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference2200SelectedBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_2200_selected_body_boundary_entries()
            .ok_or(Reference2200SelectedBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference2200SelectedBodyBoundarySummaryValidationError::SampleCountMismatch {
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
                        Reference2200SelectedBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference2200SelectedBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference2200SelectedBodyBoundarySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, Reference2200SelectedBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference2200SelectedBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn reference_snapshot_2200_selected_body_boundary_summary_details(
) -> Option<Reference2200SelectedBodyBoundarySummary> {
    let evidence = reference_snapshot_2200_selected_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2200SelectedBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2200 selected-body boundary reference evidence.
pub fn reference_snapshot_2200_selected_body_boundary_summary(
) -> Option<Reference2200SelectedBodyBoundarySummary> {
    reference_snapshot_2200_selected_body_boundary_summary_details()
}

/// Returns the release-facing 2200 selected-body boundary summary string.
pub fn reference_snapshot_2200_selected_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2200_selected_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2200 selected-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2200 selected-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the compact typed summary for the 2524593 selected-body boundary evidence.
pub fn reference_snapshot_2524593_selected_body_boundary_summary(
) -> Option<Reference2200SelectedBodyBoundarySummary> {
    reference_snapshot_2200_selected_body_boundary_summary()
}

/// Returns the release-facing 2524593 selected-body boundary summary string.
pub fn reference_snapshot_2524593_selected_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2524593_selected_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(_) => format_selected_body_boundary_summary_line(
                "2524593",
                summary.sample_count,
                &summary.sample_bodies,
                summary.epoch,
                "2200-01-01",
            ),
            Err(error) => {
                format!("Reference 2524593 selected-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2524593 selected-body boundary evidence: unavailable".to_string(),
    }
}

pub(crate) fn reference_snapshot_2500_selected_body_boundary_entries(
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
                            == REFERENCE_SNAPSHOT_2500_SELECTED_BODY_BOUNDARY_EPOCH_JD
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

/// Compact release-facing summary for the 2500-01-01 selected-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference2500SelectedBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a 2500 selected-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference2500SelectedBodyBoundarySummaryValidationError {
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

impl fmt::Display for Reference2500SelectedBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => {
                f.write_str("reference 2500 selected-body boundary evidence is unavailable")
            }
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 2500 selected-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 2500 selected-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 2500 selected-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference2500SelectedBodyBoundarySummaryValidationError {}

impl Reference2500SelectedBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 2500 selected-body boundary evidence: {} exact samples at {} ({}); 2500-01-01 selected-body boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference2500SelectedBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_2500_selected_body_boundary_entries()
            .ok_or(Reference2500SelectedBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference2500SelectedBodyBoundarySummaryValidationError::SampleCountMismatch {
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
                        Reference2500SelectedBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference2500SelectedBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference2500SelectedBodyBoundarySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, Reference2500SelectedBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference2500SelectedBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn reference_snapshot_2500_selected_body_boundary_summary_details(
) -> Option<Reference2500SelectedBodyBoundarySummary> {
    let evidence = reference_snapshot_2500_selected_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference2500SelectedBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 2500 selected-body boundary reference evidence.
pub fn reference_snapshot_2500_selected_body_boundary_summary(
) -> Option<Reference2500SelectedBodyBoundarySummary> {
    reference_snapshot_2500_selected_body_boundary_summary_details()
}

/// Returns the release-facing 2500 selected-body boundary summary string.
pub fn reference_snapshot_2500_selected_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2500_selected_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 2500 selected-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2500 selected-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the compact typed summary for the 2634167 selected-body boundary reference evidence.
#[doc(alias = "reference_snapshot_2500_selected_body_boundary_summary")]
pub fn reference_snapshot_2634167_selected_body_boundary_summary(
) -> Option<Reference2500SelectedBodyBoundarySummary> {
    reference_snapshot_2500_selected_body_boundary_summary()
}

/// Returns the release-facing 2634167 selected-body boundary summary string.
pub fn reference_snapshot_2634167_selected_body_boundary_summary_for_report() -> String {
    match reference_snapshot_2634167_selected_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(_) => format_selected_body_boundary_summary_line(
                "2634167",
                summary.sample_count,
                &summary.sample_bodies,
                summary.epoch,
                "2500-01-01",
            ),
            Err(error) => {
                format!("Reference 2634167 selected-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 2634167 selected-body boundary evidence: unavailable".to_string(),
    }
}

/// Compact release-facing summary for the early major-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceEarlyMajorBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for an early major-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceEarlyMajorBodyBoundarySummaryValidationError {
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

impl fmt::Display for ReferenceEarlyMajorBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference early major-body boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference early major-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference early major-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference early major-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceEarlyMajorBodyBoundarySummaryValidationError {}

impl ReferenceEarlyMajorBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference early major-body boundary evidence: {} exact samples at {} ({})",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), ReferenceEarlyMajorBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_early_major_body_boundary_entries()
            .ok_or(ReferenceEarlyMajorBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceEarlyMajorBodyBoundarySummaryValidationError::SampleCountMismatch {
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
                        ReferenceEarlyMajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceEarlyMajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceEarlyMajorBodyBoundarySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, ReferenceEarlyMajorBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceEarlyMajorBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the 1800-01-03 major-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference1800MajorBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for an 1800 major-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference1800MajorBodyBoundarySummaryValidationError {
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

impl fmt::Display for Reference1800MajorBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference 1800 major-body boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 1800 major-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 1800 major-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 1800 major-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference1800MajorBodyBoundarySummaryValidationError {}

impl Reference1800MajorBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 1800 major-body boundary evidence: {} exact samples at {} ({}); 1800-01-03 boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference1800MajorBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_1800_major_body_boundary_entries()
            .ok_or(Reference1800MajorBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference1800MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
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
                        Reference1800MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference1800MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference1800MajorBodyBoundarySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, Reference1800MajorBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference1800MajorBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the 2500-01-01 major-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference2500MajorBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a 2500 major-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference2500MajorBodyBoundarySummaryValidationError {
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

impl fmt::Display for Reference2500MajorBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference 2500 major-body boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 2500 major-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 2500 major-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 2500 major-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference2500MajorBodyBoundarySummaryValidationError {}

impl Reference2500MajorBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 2500 major-body boundary evidence: {} exact samples at {} ({}); 2500-01-01 boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference2500MajorBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_2500_major_body_boundary_entries()
            .ok_or(Reference2500MajorBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference2500MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
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
                        Reference2500MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference2500MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference2500MajorBodyBoundarySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, Reference2500MajorBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference2500MajorBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
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
