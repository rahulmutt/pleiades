//! reference snapshot boundary summaries (era_a).

use core::fmt;
use std::sync::OnceLock;

use pleiades_types::Instant;

#[allow(unused_imports)]
use crate::reference_summary::*;
#[allow(unused_imports)]
use crate::*;

/// Compact release-facing summary for the Moon high-curvature reference window.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceLunarBoundarySummary {
    /// Number of exact Moon samples in the reference window.
    pub sample_count: usize,
    /// Number of distinct epochs in the reference window.
    pub epoch_count: usize,
    /// Earliest epoch represented in the reference window.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the reference window.
    pub latest_epoch: Instant,
}

impl ReferenceLunarBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference lunar boundary evidence: {} exact Moon samples at {}..{}; high-curvature interpolation window",
            self.sample_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
        )
    }

    /// Returns `Ok(())` when the Moon boundary summary still matches the checked-in slice.
    pub fn validate(&self) -> Result<(), ReferenceLunarBoundarySummaryValidationError> {
        let Some(expected) = reference_snapshot_lunar_boundary_summary_details() else {
            return Err(
                ReferenceLunarBoundarySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ReferenceLunarBoundarySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ReferenceLunarBoundarySummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ReferenceLunarBoundarySummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ReferenceLunarBoundarySummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated Moon boundary summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceLunarBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceLunarBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a Moon boundary summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceLunarBoundarySummaryValidationError {
    /// A summary field is out of sync with the checked-in lunar boundary evidence.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ReferenceLunarBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference lunar boundary summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceLunarBoundarySummaryValidationError {}

/// Compact release-facing summary for the major-body high-curvature reference window.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceHighCurvatureSummary {
    /// Number of exact major-body samples in the reference window.
    pub sample_count: usize,
    /// Number of distinct bodies in the reference window.
    pub body_count: usize,
    /// Bodies represented by the reference window in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs in the reference window.
    pub epoch_count: usize,
    /// Earliest epoch represented in the reference window.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the reference window.
    pub latest_epoch: Instant,
}

impl ReferenceHighCurvatureSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference major-body high-curvature evidence: {} exact samples across {} bodies and {} epochs ({}..{}); bodies: {}; high-curvature interpolation window",
            self.sample_count,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(&self.bodies),
        )
    }

    /// Returns `Ok(())` when the major-body high-curvature summary still matches the checked-in slice.
    pub fn validate(&self) -> Result<(), ReferenceHighCurvatureSummaryValidationError> {
        let Some(expected) = reference_snapshot_high_curvature_summary_details() else {
            return Err(
                ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.body_count != expected.body_count {
            return Err(
                ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "body_count",
                },
            );
        }
        if self.bodies != expected.bodies {
            return Err(
                ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync { field: "bodies" },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ReferenceHighCurvatureSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated major-body high-curvature summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ReferenceHighCurvatureSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceHighCurvatureSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a major-body high-curvature summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceHighCurvatureSummaryValidationError {
    /// A summary field is out of sync with the checked-in high-curvature evidence.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for ReferenceHighCurvatureSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference high-curvature summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceHighCurvatureSummaryValidationError {}

/// Compact release-facing summary for the major-body boundary-day reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceMajorBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a major-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceMajorBodyBoundarySummaryValidationError {
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

impl fmt::Display for ReferenceMajorBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference major-body boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference major-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference major-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference major-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceMajorBodyBoundarySummaryValidationError {}

impl ReferenceMajorBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference major-body boundary evidence: {} exact samples at {} ({}); 2001-01-08 boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), ReferenceMajorBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_major_body_boundary_entries()
            .ok_or(ReferenceMajorBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceMajorBodyBoundarySummaryValidationError::SampleCountMismatch {
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
                        ReferenceMajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceMajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceMajorBodyBoundarySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, ReferenceMajorBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceMajorBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the major-body bridge-day reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceMajorBodyBridgeSummary {
    /// Number of exact samples in the bridge slice.
    pub sample_count: usize,
    /// Bodies covered by the bridge slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the bridge slice.
    pub epoch: Instant,
}

/// Validation errors for a major-body bridge summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceMajorBodyBridgeSummaryValidationError {
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

impl fmt::Display for ReferenceMajorBodyBridgeSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference major-body bridge evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference major-body bridge evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference major-body bridge evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference major-body bridge evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceMajorBodyBridgeSummaryValidationError {}

impl ReferenceMajorBodyBridgeSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference major-body bridge evidence: {} exact samples at {} ({}); bridge sample across the major-body boundary window",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), ReferenceMajorBodyBridgeSummaryValidationError> {
        let evidence = reference_snapshot_major_body_bridge_entries()
            .ok_or(ReferenceMajorBodyBridgeSummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceMajorBodyBridgeSummaryValidationError::SampleCountMismatch {
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
                        ReferenceMajorBodyBridgeSummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceMajorBodyBridgeSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceMajorBodyBridgeSummaryValidationError::EpochMismatch {
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
    ) -> Result<String, ReferenceMajorBodyBridgeSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceMajorBodyBridgeSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the Mars/Jupiter boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceMarsJupiterBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a Mars/Jupiter boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceMarsJupiterBoundarySummaryValidationError {
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

impl fmt::Display for ReferenceMarsJupiterBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference Mars/Jupiter boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference Mars/Jupiter boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference Mars/Jupiter boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference Mars/Jupiter boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceMarsJupiterBoundarySummaryValidationError {}

impl ReferenceMarsJupiterBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference Mars/Jupiter boundary evidence: {} exact samples at {} ({}); 2001-01-09 boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), ReferenceMarsJupiterBoundarySummaryValidationError> {
        let evidence = reference_snapshot_mars_jupiter_boundary_entries()
            .ok_or(ReferenceMarsJupiterBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceMarsJupiterBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        let expected_bodies = vec![
            pleiades_backend::CelestialBody::Sun,
            pleiades_backend::CelestialBody::Moon,
            pleiades_backend::CelestialBody::Mercury,
            pleiades_backend::CelestialBody::Venus,
            pleiades_backend::CelestialBody::Mars,
            pleiades_backend::CelestialBody::Jupiter,
            pleiades_backend::CelestialBody::Saturn,
            pleiades_backend::CelestialBody::Uranus,
            pleiades_backend::CelestialBody::Neptune,
            pleiades_backend::CelestialBody::Pluto,
            pleiades_backend::CelestialBody::Ceres,
            pleiades_backend::CelestialBody::Pallas,
            pleiades_backend::CelestialBody::Juno,
            pleiades_backend::CelestialBody::Vesta,
            pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
            pleiades_backend::CelestialBody::Custom(CustomBodyId::new("asteroid", "99942-Apophis")),
        ];
        if self.sample_bodies.as_slice() != expected_bodies.as_slice() {
            for (index, (expected, found)) in expected_bodies
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        ReferenceMarsJupiterBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceMarsJupiterBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceMarsJupiterBoundarySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, ReferenceMarsJupiterBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ReferenceMarsJupiterBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Compact release-facing summary for the 1749-12-31 major-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference1749MajorBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a 1749 major-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference1749MajorBodyBoundarySummaryValidationError {
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

impl fmt::Display for Reference1749MajorBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("reference 1749 major-body boundary evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 1749 major-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 1749 major-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 1749 major-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference1749MajorBodyBoundarySummaryValidationError {}

impl Reference1749MajorBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 1749 major-body boundary evidence: {} exact samples at {} ({}); 1749-12-31 boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference1749MajorBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_1749_major_body_boundary_entries()
            .ok_or(Reference1749MajorBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference1749MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
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
                        Reference1749MajorBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference1749MajorBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference1749MajorBodyBoundarySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, Reference1749MajorBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference1749MajorBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn reference_snapshot_1500_selected_body_boundary_entries(
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
                            == REFERENCE_SNAPSHOT_1500_SELECTED_BODY_BOUNDARY_EPOCH_JD
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

/// Compact release-facing summary for the 1500-01-01 selected-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference1500SelectedBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a 1500 selected-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference1500SelectedBodyBoundarySummaryValidationError {
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

impl fmt::Display for Reference1500SelectedBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => {
                f.write_str("reference 1500 selected-body boundary evidence is unavailable")
            }
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 1500 selected-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 1500 selected-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 1500 selected-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference1500SelectedBodyBoundarySummaryValidationError {}

impl Reference1500SelectedBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 1500 selected-body boundary evidence: {} exact samples at {} ({}); 1500-01-01 selected-body boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference1500SelectedBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_1500_selected_body_boundary_entries()
            .ok_or(Reference1500SelectedBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference1500SelectedBodyBoundarySummaryValidationError::SampleCountMismatch {
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
                        Reference1500SelectedBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference1500SelectedBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference1500SelectedBodyBoundarySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, Reference1500SelectedBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference1500SelectedBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn reference_snapshot_1500_selected_body_boundary_summary_details(
) -> Option<Reference1500SelectedBodyBoundarySummary> {
    let evidence = reference_snapshot_1500_selected_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference1500SelectedBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 1500 selected-body boundary reference evidence.
pub fn reference_snapshot_1500_selected_body_boundary_summary(
) -> Option<Reference1500SelectedBodyBoundarySummary> {
    reference_snapshot_1500_selected_body_boundary_summary_details()
}

/// Returns the release-facing 1500 selected-body boundary summary string.
pub fn reference_snapshot_1500_selected_body_boundary_summary_for_report() -> String {
    match reference_snapshot_1500_selected_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 1500 selected-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 1500 selected-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the compact typed summary for the 2268932 selected-body boundary reference evidence.
pub fn reference_snapshot_2268932_selected_body_boundary_summary(
) -> Option<Reference1500SelectedBodyBoundarySummary> {
    reference_snapshot_1500_selected_body_boundary_summary()
}

/// Returns the release-facing 2268932 selected-body boundary summary string.
pub fn reference_snapshot_2268932_selected_body_boundary_summary_for_report() -> String {
    reference_snapshot_1500_selected_body_boundary_summary_for_report()
}

pub(crate) fn reference_snapshot_1600_selected_body_boundary_entries(
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
                            == REFERENCE_SNAPSHOT_1600_SELECTED_BODY_BOUNDARY_EPOCH_JD
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

/// Compact release-facing summary for the 1600-01-11 selected-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference1600SelectedBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a 1600 selected-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference1600SelectedBodyBoundarySummaryValidationError {
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

impl fmt::Display for Reference1600SelectedBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => {
                f.write_str("reference 1600 selected-body boundary evidence is unavailable")
            }
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 1600 selected-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 1600 selected-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 1600 selected-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference1600SelectedBodyBoundarySummaryValidationError {}

impl Reference1600SelectedBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 1600 selected-body boundary evidence: {} exact samples at {} ({}); 1600-01-11 selected-body boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference1600SelectedBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_1600_selected_body_boundary_entries()
            .ok_or(Reference1600SelectedBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference1600SelectedBodyBoundarySummaryValidationError::SampleCountMismatch {
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
                        Reference1600SelectedBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference1600SelectedBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference1600SelectedBodyBoundarySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, Reference1600SelectedBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference1600SelectedBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn reference_snapshot_1600_selected_body_boundary_summary_details(
) -> Option<Reference1600SelectedBodyBoundarySummary> {
    let evidence = reference_snapshot_1600_selected_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference1600SelectedBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 1600 selected-body boundary reference evidence.
pub fn reference_snapshot_1600_selected_body_boundary_summary(
) -> Option<Reference1600SelectedBodyBoundarySummary> {
    reference_snapshot_1600_selected_body_boundary_summary_details()
}

/// Returns the release-facing 1600 selected-body boundary summary string.
pub fn reference_snapshot_1600_selected_body_boundary_summary_for_report() -> String {
    match reference_snapshot_1600_selected_body_boundary_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Reference 1600 selected-body boundary evidence: unavailable ({error})")
            }
        },
        None => "Reference 1600 selected-body boundary evidence: unavailable".to_string(),
    }
}

/// Returns the compact typed summary for the 2305457 selected-body boundary reference evidence.
pub fn reference_snapshot_2305457_selected_body_boundary_summary(
) -> Option<Reference1600SelectedBodyBoundarySummary> {
    reference_snapshot_1600_selected_body_boundary_summary()
}

/// Returns the release-facing 2305457 selected-body boundary summary string.
pub fn reference_snapshot_2305457_selected_body_boundary_summary_for_report() -> String {
    reference_snapshot_1600_selected_body_boundary_summary_for_report()
}

pub(crate) fn reference_snapshot_1750_selected_body_boundary_entries(
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
                            == REFERENCE_SNAPSHOT_1750_SELECTED_BODY_BOUNDARY_EPOCH_JD
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

/// Compact release-facing summary for the 1750-01-01 selected-body boundary reference evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference1750SelectedBodyBoundarySummary {
    /// Number of exact samples in the boundary slice.
    pub sample_count: usize,
    /// Bodies covered by the boundary slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the boundary slice.
    pub epoch: Instant,
}

/// Validation errors for a 1750 selected-body boundary summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum Reference1750SelectedBodyBoundarySummaryValidationError {
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

impl fmt::Display for Reference1750SelectedBodyBoundarySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => {
                f.write_str("reference 1750 selected-body boundary evidence is unavailable")
            }
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "reference 1750 selected-body boundary evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "reference 1750 selected-body boundary evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "reference 1750 selected-body boundary evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for Reference1750SelectedBodyBoundarySummaryValidationError {}

impl Reference1750SelectedBodyBoundarySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Reference 1750 selected-body boundary evidence: {} exact samples at {} ({}); 1750-01-01 selected-body boundary sample",
            self.sample_count,
            format_instant(self.epoch),
            format_bodies(&self.sample_bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), Reference1750SelectedBodyBoundarySummaryValidationError> {
        let evidence = reference_snapshot_1750_selected_body_boundary_entries()
            .ok_or(Reference1750SelectedBodyBoundarySummaryValidationError::Empty)?;

        if self.sample_count != evidence.len() {
            return Err(
                Reference1750SelectedBodyBoundarySummaryValidationError::SampleCountMismatch {
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
                        Reference1750SelectedBodyBoundarySummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                Reference1750SelectedBodyBoundarySummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }

        if self.epoch != evidence[0].epoch {
            return Err(
                Reference1750SelectedBodyBoundarySummaryValidationError::EpochMismatch {
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
    ) -> Result<String, Reference1750SelectedBodyBoundarySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Reference1750SelectedBodyBoundarySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn reference_snapshot_1750_selected_body_boundary_summary_details(
) -> Option<Reference1750SelectedBodyBoundarySummary> {
    let evidence = reference_snapshot_1750_selected_body_boundary_entries()?;
    let mut sample_bodies = Vec::new();
    for entry in evidence {
        if !sample_bodies.contains(&entry.body) {
            sample_bodies.push(entry.body.clone());
        }
    }

    Some(Reference1750SelectedBodyBoundarySummary {
        sample_count: evidence.len(),
        sample_bodies,
        epoch: evidence[0].epoch,
    })
}

/// Returns the compact typed summary for the 1750 selected-body boundary reference evidence.
pub fn reference_snapshot_1750_selected_body_boundary_summary(
) -> Option<Reference1750SelectedBodyBoundarySummary> {
    reference_snapshot_1750_selected_body_boundary_summary_details()
}
