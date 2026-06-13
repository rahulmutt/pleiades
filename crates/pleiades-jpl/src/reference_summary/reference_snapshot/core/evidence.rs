//! core evidence summaries.

use core::fmt;

use pleiades_types::Instant;

#[allow(unused_imports)]
use crate::reference_summary::*;
#[allow(unused_imports)]
use crate::*;

pub(crate) fn reference_snapshot_exact_j2000_entries() -> Vec<&'static SnapshotEntry> {
    reference_snapshot()
        .iter()
        .filter(|entry| entry.epoch.julian_day.days() == 2451545.0)
        .collect()
}

/// Compact release-facing summary for the exact J2000 reference snapshot slice.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotExactJ2000EvidenceSummary {
    /// Number of exact samples in the reference snapshot J2000 slice.
    pub sample_count: usize,
    /// Bodies covered by the exact J2000 reference snapshot slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the reference snapshot J2000 slice.
    pub epoch: Instant,
}

/// Validation errors for a reference snapshot J2000 evidence summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceSnapshotExactJ2000EvidenceSummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary sample count drifted from the current evidence slice.
    SampleCountMismatch {
        sample_count: usize,
        derived_sample_count: usize,
    },
    /// The summary body order drifted from the current evidence slice.
    BodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
}

impl fmt::Display for ReferenceSnapshotExactJ2000EvidenceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference snapshot exact J2000 evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference snapshot exact J2000 evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference snapshot exact J2000 evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference snapshot exact J2000 evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceSnapshotExactJ2000EvidenceSummaryValidationError {}

impl ReferenceSnapshotExactJ2000EvidenceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference snapshot exact J2000 evidence: {} exact J2000 samples at {} ({})",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(
        &self,
    ) -> Result<(), ReferenceSnapshotExactJ2000EvidenceSummaryValidationError> {
        let evidence = reference_snapshot_exact_j2000_entries();
        if evidence.is_empty() {
            return Err(ReferenceSnapshotExactJ2000EvidenceSummaryValidationError::Empty);
        }

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceSnapshotExactJ2000EvidenceSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let expected_bodies = reference_bodies()
            .iter()
            .filter(|body| evidence.iter().any(|entry| &entry.body == *body))
            .cloned()
            .collect::<Vec<_>>();
        if self.sample_bodies != expected_bodies {
            for (index, (expected, found)) in expected_bodies
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        ReferenceSnapshotExactJ2000EvidenceSummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceSnapshotExactJ2000EvidenceSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceSnapshotExactJ2000EvidenceSummaryValidationError::EpochMismatch {
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
    ) -> Result<String, ReferenceSnapshotExactJ2000EvidenceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceSnapshotExactJ2000EvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the release-facing reference snapshot exact J2000 evidence summary.
pub fn reference_snapshot_exact_j2000_evidence_summary(
) -> Option<ReferenceSnapshotExactJ2000EvidenceSummary> {
    let evidence = reference_snapshot_exact_j2000_entries();
    if evidence.is_empty() {
        return None;
    }

    Some(ReferenceSnapshotExactJ2000EvidenceSummary {
        sample_count: evidence.len(),
        sample_bodies: reference_bodies()
            .iter()
            .filter(|body| evidence.iter().any(|entry| &entry.body == *body))
            .cloned()
            .collect(),
        epoch: evidence[0].epoch,
    })
}

/// Returns the validated release-facing reference snapshot exact J2000 evidence summary string.
pub fn validated_reference_snapshot_exact_j2000_evidence_summary_for_report(
) -> Result<String, String> {
    let summary = reference_snapshot_exact_j2000_evidence_summary()
        .ok_or_else(|| "reference snapshot exact J2000 evidence unavailable".to_string())?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

/// Returns the release-facing reference snapshot exact J2000 evidence summary string.
pub fn reference_snapshot_exact_j2000_evidence_summary_for_report() -> String {
    match validated_reference_snapshot_exact_j2000_evidence_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) if error == "reference snapshot exact J2000 evidence unavailable" => {
            "Reference snapshot exact J2000 evidence: unavailable".to_string()
        }
        Err(error) => format!("Reference snapshot exact J2000 evidence: unavailable ({error})"),
    }
}

/// Compact release-facing summary for the exact J2000 reference snapshot body classes.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSnapshotExactJ2000BodyClassCoverageSummary {
    /// Number of major-body rows in the exact J2000 slice.
    pub major_body_row_count: usize,
    /// Major bodies covered by the exact J2000 slice in first-seen order.
    pub major_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of selected-asteroid rows in the exact J2000 slice.
    pub asteroid_row_count: usize,
    /// Selected asteroids covered by the exact J2000 slice in first-seen order.
    pub asteroid_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the exact J2000 slice.
    pub epoch: Instant,
}

/// Validation errors for a reference snapshot exact J2000 body-class summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceSnapshotExactJ2000BodyClassCoverageSummaryValidationError {
    /// The summary did not expose any samples.
    Empty,
    /// The summary major-body count drifted from the current evidence slice.
    MajorBodyCountMismatch {
        major_body_row_count: usize,
        derived_major_body_row_count: usize,
    },
    /// The summary major-body order drifted from the current evidence slice.
    MajorBodyOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary asteroid count drifted from the current evidence slice.
    AsteroidCountMismatch {
        asteroid_row_count: usize,
        derived_asteroid_row_count: usize,
    },
    /// The summary asteroid order drifted from the current evidence slice.
    AsteroidOrderMismatch {
        index: usize,
        expected: pleiades_backend::CelestialBody,
        found: pleiades_backend::CelestialBody,
    },
    /// The summary epoch drifted from the current evidence slice.
    EpochMismatch { expected: Instant, found: Instant },
}

impl fmt::Display for ReferenceSnapshotExactJ2000BodyClassCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference snapshot exact J2000 body-class coverage is unavailable"),
            Self::MajorBodyCountMismatch {
                major_body_row_count,
                derived_major_body_row_count,
            } => write!(
                f,
                "reference snapshot exact J2000 body-class coverage major-body count {major_body_row_count} does not match derived major-body count {derived_major_body_row_count}"
            ),
            Self::MajorBodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference snapshot exact J2000 body-class coverage major-body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::AsteroidCountMismatch {
                asteroid_row_count,
                derived_asteroid_row_count,
            } => write!(
                f,
                "reference snapshot exact J2000 body-class coverage asteroid count {asteroid_row_count} does not match derived asteroid count {derived_asteroid_row_count}"
            ),
            Self::AsteroidOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference snapshot exact J2000 body-class coverage asteroid order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference snapshot exact J2000 body-class coverage epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceSnapshotExactJ2000BodyClassCoverageSummaryValidationError {}

impl ReferenceSnapshotExactJ2000BodyClassCoverageSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference snapshot exact J2000 body-class coverage: {} major-body samples across {} bodies and 1 epoch ({}); {} selected-asteroid samples across {} bodies and 1 epoch ({})",
            self.major_body_row_count,
            self.major_bodies.len(),
            format_bodies(&self.major_bodies),
            self.asteroid_row_count,
            self.asteroid_bodies.len(),
            format_bodies(&self.asteroid_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(
        &self,
    ) -> Result<(), ReferenceSnapshotExactJ2000BodyClassCoverageSummaryValidationError> {
        let evidence = reference_snapshot_exact_j2000_entries();
        if evidence.is_empty() {
            return Err(ReferenceSnapshotExactJ2000BodyClassCoverageSummaryValidationError::Empty);
        }

        let expected_major_bodies = reference_bodies()
            .iter()
            .filter(|body| {
                is_comparison_body(body) && evidence.iter().any(|entry| &entry.body == *body)
            })
            .cloned()
            .collect::<Vec<_>>();
        if self.major_body_row_count != expected_major_bodies.len() {
            return Err(
                ReferenceSnapshotExactJ2000BodyClassCoverageSummaryValidationError::MajorBodyCountMismatch {
                    major_body_row_count: self.major_body_row_count,
                    derived_major_body_row_count: expected_major_bodies.len(),
                },
            );
        }
        if self.major_bodies != expected_major_bodies {
            for (index, (expected, found)) in expected_major_bodies
                .iter()
                .zip(self.major_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        ReferenceSnapshotExactJ2000BodyClassCoverageSummaryValidationError::MajorBodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceSnapshotExactJ2000BodyClassCoverageSummaryValidationError::MajorBodyCountMismatch {
                    major_body_row_count: self.major_body_row_count,
                    derived_major_body_row_count: expected_major_bodies.len(),
                },
            );
        }

        let expected_asteroid_bodies = reference_asteroids()
            .iter()
            .filter(|body| evidence.iter().any(|entry| &entry.body == *body))
            .cloned()
            .collect::<Vec<_>>();
        if self.asteroid_row_count != expected_asteroid_bodies.len() {
            return Err(
                ReferenceSnapshotExactJ2000BodyClassCoverageSummaryValidationError::AsteroidCountMismatch {
                    asteroid_row_count: self.asteroid_row_count,
                    derived_asteroid_row_count: expected_asteroid_bodies.len(),
                },
            );
        }
        if self.asteroid_bodies != expected_asteroid_bodies {
            for (index, (expected, found)) in expected_asteroid_bodies
                .iter()
                .zip(self.asteroid_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        ReferenceSnapshotExactJ2000BodyClassCoverageSummaryValidationError::AsteroidOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceSnapshotExactJ2000BodyClassCoverageSummaryValidationError::AsteroidCountMismatch {
                    asteroid_row_count: self.asteroid_row_count,
                    derived_asteroid_row_count: expected_asteroid_bodies.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceSnapshotExactJ2000BodyClassCoverageSummaryValidationError::EpochMismatch {
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
    ) -> Result<String, ReferenceSnapshotExactJ2000BodyClassCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceSnapshotExactJ2000BodyClassCoverageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the release-facing exact J2000 body-class coverage summary.
pub fn reference_snapshot_exact_j2000_body_class_coverage_summary(
) -> Option<ReferenceSnapshotExactJ2000BodyClassCoverageSummary> {
    let evidence = reference_snapshot_exact_j2000_entries();
    if evidence.is_empty() {
        return None;
    }

    Some(ReferenceSnapshotExactJ2000BodyClassCoverageSummary {
        major_body_row_count: reference_bodies()
            .iter()
            .filter(|body| {
                is_comparison_body(body) && evidence.iter().any(|entry| &entry.body == *body)
            })
            .count(),
        major_bodies: reference_bodies()
            .iter()
            .filter(|body| {
                is_comparison_body(body) && evidence.iter().any(|entry| &entry.body == *body)
            })
            .cloned()
            .collect(),
        asteroid_row_count: reference_asteroids()
            .iter()
            .filter(|body| evidence.iter().any(|entry| &entry.body == *body))
            .count(),
        asteroid_bodies: reference_asteroids()
            .iter()
            .filter(|body| evidence.iter().any(|entry| &entry.body == *body))
            .cloned()
            .collect(),
        epoch: evidence[0].epoch,
    })
}

/// Returns the validated release-facing exact J2000 body-class coverage summary string.
pub fn validated_reference_snapshot_exact_j2000_body_class_coverage_summary_for_report(
) -> Result<String, String> {
    let summary =
        reference_snapshot_exact_j2000_body_class_coverage_summary().ok_or_else(|| {
            "reference snapshot exact J2000 body-class coverage unavailable".to_string()
        })?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

/// Returns the release-facing exact J2000 body-class coverage summary string.
pub fn reference_snapshot_exact_j2000_body_class_coverage_summary_for_report() -> String {
    match validated_reference_snapshot_exact_j2000_body_class_coverage_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) if error == "reference snapshot exact J2000 body-class coverage unavailable" => {
            "Reference snapshot exact J2000 body-class coverage: unavailable".to_string()
        }
        Err(error) => {
            format!("Reference snapshot exact J2000 body-class coverage: unavailable ({error})")
        }
    }
}
