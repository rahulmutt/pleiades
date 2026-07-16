//! reference asteroid summaries.

use core::fmt;

use pleiades_backend::EphemerisRequest;
use pleiades_types::{CoordinateFrame, Instant};

#[allow(unused_imports)]
use crate::reference_summary::*;
#[allow(unused_imports)]
use crate::*;

/// Returns the source-backed asteroid subset present in the reference snapshot.
pub fn reference_asteroids() -> &'static [pleiades_backend::CelestialBody] {
    reference_asteroid_list()
}

/// Exact J2000 asteroid reference samples from the checked-in snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceAsteroidEvidence {
    /// Asteroid body covered by the exact snapshot row.
    pub body: pleiades_backend::CelestialBody,
    /// Exact epoch used for the reference sample.
    pub epoch: Instant,
    /// Ecliptic longitude in degrees.
    pub longitude_deg: f64,
    /// Ecliptic latitude in degrees.
    pub latitude_deg: f64,
    /// Geocentric distance in astronomical units.
    pub distance_au: f64,
}

/// Exact J2000 asteroid equatorial samples derived from the checked-in snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceAsteroidEquatorialEvidence {
    /// Asteroid body covered by the exact snapshot row.
    pub body: pleiades_backend::CelestialBody,
    /// Exact epoch used for the reference sample.
    pub epoch: Instant,
    /// Mean-obliquity equatorial coordinates derived from the ecliptic sample.
    pub equatorial: pleiades_types::EquatorialCoordinates,
}

/// Returns the exact J2000 asteroid evidence samples present in the reference snapshot.
pub fn reference_asteroid_evidence() -> &'static [ReferenceAsteroidEvidence] {
    reference_asteroid_evidence_list()
}

/// Returns the exact J2000 asteroid request corpus in the requested frame.
///
/// The requests preserve the checked-in asteroid order and stored J2000 epoch,
/// so downstream batch checks can reuse the exact selected-asteroid slice
/// without reconstructing it from the sample evidence.
pub fn reference_asteroid_requests(frame: CoordinateFrame) -> Option<Vec<EphemerisRequest>> {
    reference_asteroid_requests_with_frame_selector(|_| frame)
}

/// Returns the exact J2000 asteroid request corpus in the requested frame.
///
/// This is a compatibility alias for [`reference_asteroid_requests`].
#[doc(alias = "reference_asteroid_requests")]
pub fn reference_asteroid_request_corpus(frame: CoordinateFrame) -> Option<Vec<EphemerisRequest>> {
    reference_asteroid_requests(frame)
}

/// Returns the exact J2000 asteroid request corpus in the ecliptic frame.
///
/// This is a compatibility alias for [`reference_asteroid_request_corpus`].
#[doc(alias = "reference_asteroid_request_corpus")]
pub fn reference_asteroid_ecliptic_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_asteroid_requests(CoordinateFrame::Ecliptic)
}

/// Returns the exact J2000 asteroid request corpus in the equatorial frame.
///
/// This is a compatibility alias for [`reference_asteroid_request_corpus`].
#[doc(alias = "reference_asteroid_request_corpus")]
pub fn reference_asteroid_equatorial_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_asteroid_requests(CoordinateFrame::Equatorial)
}

/// Returns the mixed-frame exact J2000 asteroid request corpus used by batch parity checks.
///
/// The requests preserve the checked-in asteroid order and alternate between
/// ecliptic and equatorial frames so downstream tooling can reuse the exact
/// selected-asteroid batch shape without reconstructing it from sample evidence.
pub fn reference_asteroid_batch_parity_requests() -> Option<Vec<EphemerisRequest>> {
    reference_asteroid_requests_with_frame_selector(|index| {
        if index % 2 == 0 {
            CoordinateFrame::Ecliptic
        } else {
            CoordinateFrame::Equatorial
        }
    })
}

/// Returns the mixed-frame exact J2000 asteroid request corpus used by batch parity checks.
///
/// This is a compatibility alias for [`reference_asteroid_batch_parity_requests`].
#[doc(alias = "reference_asteroid_batch_parity_requests")]
pub fn reference_asteroid_batch_parity_request_corpus() -> Option<Vec<EphemerisRequest>> {
    reference_asteroid_batch_parity_requests()
}

/// Returns the exact J2000 asteroid equatorial evidence samples derived from the reference snapshot.
pub fn reference_asteroid_equatorial_evidence() -> &'static [ReferenceAsteroidEquatorialEvidence] {
    reference_asteroid_equatorial_evidence_list()
}

/// Compact release-facing summary for the exact asteroid evidence slice.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceAsteroidEvidenceSummary {
    /// Number of exact samples in the selected asteroid slice.
    pub sample_count: usize,
    /// Bodies covered by the exact asteroid evidence slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the asteroid evidence slice.
    pub epoch: Instant,
}

/// Validation errors for a reference asteroid evidence summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceAsteroidEvidenceSummaryValidationError {
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

impl fmt::Display for ReferenceAsteroidEvidenceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("selected asteroid evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "selected asteroid evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "selected asteroid evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "selected asteroid evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
        }
    }
}

impl std::error::Error for ReferenceAsteroidEvidenceSummaryValidationError {}

impl ReferenceAsteroidEvidenceSummary {
    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(&self) -> Result<(), ReferenceAsteroidEvidenceSummaryValidationError> {
        let evidence = reference_asteroid_evidence();
        if evidence.is_empty() {
            return Err(ReferenceAsteroidEvidenceSummaryValidationError::Empty);
        }

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceAsteroidEvidenceSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }
        if self.sample_bodies.as_slice() != reference_asteroids() {
            for (index, (expected, found)) in reference_asteroids()
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        ReferenceAsteroidEvidenceSummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceAsteroidEvidenceSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }
        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceAsteroidEvidenceSummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }

        Ok(())
    }
}

/// Compact release-facing summary for the equatorial asteroid evidence slice.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceAsteroidEquatorialEvidenceSummary {
    /// Number of exact samples in the selected asteroid slice.
    pub sample_count: usize,
    /// Bodies covered by the exact asteroid evidence slice in first-seen order.
    pub sample_bodies: Vec<pleiades_backend::CelestialBody>,
    /// Exact epoch shared by the asteroid evidence slice.
    pub epoch: Instant,
    /// Summary of the equatorial transform used to derive the equatorial slice.
    pub transform_note: &'static str,
}

/// Validation errors for a reference asteroid equatorial evidence summary that drifted from the current slice.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceAsteroidEquatorialEvidenceSummaryValidationError {
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
    /// The summary transform note drifted from the current evidence slice.
    TransformNoteMismatch {
        /// Value expected from the current evidence slice.
        expected: &'static str,
        /// Value recorded in the summary under validation.
        found: &'static str,
    },
}

impl fmt::Display for ReferenceAsteroidEquatorialEvidenceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("selected asteroid equatorial evidence is unavailable"),
            Self::SampleCountMismatch {
                sample_count,
                derived_sample_count,
            } => write!(
                f,
                "selected asteroid equatorial evidence sample count {sample_count} does not match derived sample count {derived_sample_count}"
            ),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "selected asteroid equatorial evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch { expected, found } => write!(
                f,
                "selected asteroid equatorial evidence epoch mismatch: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
            Self::TransformNoteMismatch { expected, found } => write!(
                f,
                "selected asteroid equatorial evidence transform note mismatch: expected '{expected}', found '{found}'"
            ),
        }
    }
}

impl std::error::Error for ReferenceAsteroidEquatorialEvidenceSummaryValidationError {}

impl ReferenceAsteroidEquatorialEvidenceSummary {
    /// Returns `Ok(())` when the summary still matches the current evidence slice.
    pub fn validate(
        &self,
    ) -> Result<(), ReferenceAsteroidEquatorialEvidenceSummaryValidationError> {
        let evidence = reference_asteroid_equatorial_evidence();
        if evidence.is_empty() {
            return Err(ReferenceAsteroidEquatorialEvidenceSummaryValidationError::Empty);
        }

        if self.sample_count != evidence.len() {
            return Err(
                ReferenceAsteroidEquatorialEvidenceSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }
        if self.sample_bodies.as_slice() != reference_asteroids() {
            for (index, (expected, found)) in reference_asteroids()
                .iter()
                .zip(self.sample_bodies.iter())
                .enumerate()
            {
                if expected != found {
                    return Err(
                        ReferenceAsteroidEquatorialEvidenceSummaryValidationError::BodyOrderMismatch {
                            index,
                            expected: expected.clone(),
                            found: found.clone(),
                        },
                    );
                }
            }
            return Err(
                ReferenceAsteroidEquatorialEvidenceSummaryValidationError::SampleCountMismatch {
                    sample_count: self.sample_count,
                    derived_sample_count: evidence.len(),
                },
            );
        }
        if self.epoch != evidence[0].epoch {
            return Err(
                ReferenceAsteroidEquatorialEvidenceSummaryValidationError::EpochMismatch {
                    expected: evidence[0].epoch,
                    found: self.epoch,
                },
            );
        }
        if self.transform_note != "mean-obliquity equatorial transform" {
            return Err(
                ReferenceAsteroidEquatorialEvidenceSummaryValidationError::TransformNoteMismatch {
                    expected: "mean-obliquity equatorial transform",
                    found: self.transform_note,
                },
            );
        }

        Ok(())
    }
}

pub(crate) fn reference_asteroid_evidence_summary_details(
) -> Option<ReferenceAsteroidEvidenceSummary> {
    let evidence = reference_asteroid_evidence();
    evidence
        .first()
        .map(|first| ReferenceAsteroidEvidenceSummary {
            sample_count: evidence.len(),
            sample_bodies: reference_asteroids().to_vec(),
            epoch: first.epoch,
        })
}

pub(crate) fn reference_asteroid_equatorial_evidence_summary_details(
) -> Option<ReferenceAsteroidEquatorialEvidenceSummary> {
    let evidence = reference_asteroid_equatorial_evidence();
    evidence
        .first()
        .map(|first| ReferenceAsteroidEquatorialEvidenceSummary {
            sample_count: evidence.len(),
            sample_bodies: reference_asteroids().to_vec(),
            epoch: first.epoch,
            transform_note: "mean-obliquity equatorial transform",
        })
}

/// Returns the compact typed summary for the exact asteroid evidence slice.
pub fn reference_asteroid_evidence_summary() -> Option<ReferenceAsteroidEvidenceSummary> {
    reference_asteroid_evidence_summary_details()
}

/// Returns the compact typed summary for the equatorial asteroid evidence slice.
pub fn reference_asteroid_equatorial_evidence_summary(
) -> Option<ReferenceAsteroidEquatorialEvidenceSummary> {
    reference_asteroid_equatorial_evidence_summary_details()
}

/// Validation errors for the exact asteroid evidence corpus drifting from
/// the checked-in reference expectations.
///
/// Promoted to `pub` (Slice D Task 7) so validate's relocated
/// `reference_asteroid_evidence_summary_for_report` copy can call the
/// `validate_reference_asteroid_evidence` gate instead of reproducing it.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceAsteroidEvidenceValidationError {
    /// The evidence corpus did not expose any samples.
    Empty,
    /// The evidence body order drifted from the expected asteroid list.
    BodyOrderMismatch {
        /// Zero-based position where the drift was detected.
        index: usize,
        /// Body expected at this position.
        expected: pleiades_backend::CelestialBody,
        /// Body found at this position.
        found: pleiades_backend::CelestialBody,
    },
    /// The evidence epoch drifted from the expected reference epoch.
    EpochMismatch {
        /// Zero-based position where the drift was detected.
        index: usize,
        /// Epoch expected at this position.
        expected: Instant,
        /// Epoch found at this position.
        found: Instant,
    },
    /// The evidence longitude at this position was not finite.
    NonFiniteLongitude {
        /// Zero-based position of the non-finite value.
        index: usize,
        /// Body at this position.
        body: pleiades_backend::CelestialBody,
    },
    /// The evidence latitude at this position was not finite.
    NonFiniteLatitude {
        /// Zero-based position of the non-finite value.
        index: usize,
        /// Body at this position.
        body: pleiades_backend::CelestialBody,
    },
    /// The evidence distance at this position was not finite.
    NonFiniteDistance {
        /// Zero-based position of the non-finite value.
        index: usize,
        /// Body at this position.
        body: pleiades_backend::CelestialBody,
    },
}

impl fmt::Display for ReferenceAsteroidEvidenceValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("exact asteroid evidence is unavailable"),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "exact asteroid evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "exact asteroid evidence epoch mismatch at index {index}: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
            Self::NonFiniteLongitude { index, body } => write!(
                f,
                "exact asteroid evidence longitude is non-finite at index {index} for body {body}"
            ),
            Self::NonFiniteLatitude { index, body } => write!(
                f,
                "exact asteroid evidence latitude is non-finite at index {index} for body {body}"
            ),
            Self::NonFiniteDistance { index, body } => write!(
                f,
                "exact asteroid evidence distance is non-finite at index {index} for body {body}"
            ),
        }
    }
}

/// Validation errors for the equatorial asteroid evidence corpus diverging
/// from the derived mean-obliquity transform.
///
/// Promoted to `pub` (Slice D Task 7) so validate's relocated
/// `reference_asteroid_equatorial_evidence_summary_for_report` copy can call
/// the `validate_reference_asteroid_equatorial_evidence` gate instead of
/// reproducing it.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceAsteroidEquatorialEvidenceValidationError {
    /// The evidence corpus did not expose any samples.
    Empty,
    /// The evidence body order drifted from the expected asteroid list.
    BodyOrderMismatch {
        /// Zero-based position where the drift was detected.
        index: usize,
        /// Body expected at this position.
        expected: pleiades_backend::CelestialBody,
        /// Body found at this position.
        found: pleiades_backend::CelestialBody,
    },
    /// The evidence epoch drifted from the expected reference epoch.
    EpochMismatch {
        /// Zero-based position where the drift was detected.
        index: usize,
        /// Epoch expected at this position.
        expected: Instant,
        /// Epoch found at this position.
        found: Instant,
    },
    /// The evidence right ascension diverged from the derived transform.
    RightAscensionMismatch {
        /// Zero-based position of the divergent value.
        index: usize,
        /// Body at this position.
        body: pleiades_backend::CelestialBody,
    },
    /// The evidence declination diverged from the derived transform.
    DeclinationMismatch {
        /// Zero-based position of the divergent value.
        index: usize,
        /// Body at this position.
        body: pleiades_backend::CelestialBody,
    },
    /// The evidence distance diverged from the derived transform.
    DistanceMismatch {
        /// Zero-based position of the divergent value.
        index: usize,
        /// Body at this position.
        body: pleiades_backend::CelestialBody,
    },
}

impl fmt::Display for ReferenceAsteroidEquatorialEvidenceValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("equatorial asteroid evidence is unavailable"),
            Self::BodyOrderMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "equatorial asteroid evidence body order mismatch at index {index}: expected {expected}, found {found}"
            ),
            Self::EpochMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "equatorial asteroid evidence epoch mismatch at index {index}: expected {}, found {}",
                format_instant(*expected),
                format_instant(*found)
            ),
            Self::RightAscensionMismatch { index, body } => write!(
                f,
                "equatorial asteroid evidence right ascension diverges from the derived mean-obliquity transform at index {index} for body {body}"
            ),
            Self::DeclinationMismatch { index, body } => write!(
                f,
                "equatorial asteroid evidence declination diverges from the derived mean-obliquity transform at index {index} for body {body}"
            ),
            Self::DistanceMismatch { index, body } => write!(
                f,
                "equatorial asteroid evidence distance diverges from the derived mean-obliquity transform at index {index} for body {body}"
            ),
        }
    }
}

/// Validates the exact asteroid evidence corpus against the checked-in
/// reference asteroid list and epoch. Promoted to `pub` (Slice D Task 7) so
/// validate's relocated `reference_asteroid_evidence_summary_for_report`
/// copy can call this validation gate instead of reproducing it.
pub fn validate_reference_asteroid_evidence(
    evidence: &[ReferenceAsteroidEvidence],
) -> Result<(), ReferenceAsteroidEvidenceValidationError> {
    if evidence.is_empty() {
        return Err(ReferenceAsteroidEvidenceValidationError::Empty);
    }

    let expected_bodies = reference_asteroids();
    if evidence.len() != expected_bodies.len() {
        return Err(
            ReferenceAsteroidEvidenceValidationError::BodyOrderMismatch {
                index: evidence.len(),
                expected: expected_bodies
                    .get(evidence.len())
                    .cloned()
                    .unwrap_or_else(|| expected_bodies[expected_bodies.len() - 1].clone()),
                found: evidence
                    .last()
                    .map(|sample| sample.body.clone())
                    .unwrap_or_else(|| expected_bodies[0].clone()),
            },
        );
    }

    let expected_epoch = reference_instant();
    for (index, (sample, expected_body)) in evidence.iter().zip(expected_bodies.iter()).enumerate()
    {
        if sample.body != *expected_body {
            return Err(
                ReferenceAsteroidEvidenceValidationError::BodyOrderMismatch {
                    index,
                    expected: expected_body.clone(),
                    found: sample.body.clone(),
                },
            );
        }
        if sample.epoch != expected_epoch {
            return Err(ReferenceAsteroidEvidenceValidationError::EpochMismatch {
                index,
                expected: expected_epoch,
                found: sample.epoch,
            });
        }
        if !sample.longitude_deg.is_finite() {
            return Err(
                ReferenceAsteroidEvidenceValidationError::NonFiniteLongitude {
                    index,
                    body: sample.body.clone(),
                },
            );
        }
        if !sample.latitude_deg.is_finite() {
            return Err(
                ReferenceAsteroidEvidenceValidationError::NonFiniteLatitude {
                    index,
                    body: sample.body.clone(),
                },
            );
        }
        if !sample.distance_au.is_finite() {
            return Err(
                ReferenceAsteroidEvidenceValidationError::NonFiniteDistance {
                    index,
                    body: sample.body.clone(),
                },
            );
        }
    }

    Ok(())
}

/// Validates the equatorial asteroid evidence corpus against the derived
/// mean-obliquity transform of the exact evidence corpus. Promoted to `pub`
/// (Slice D Task 7) so validate's relocated
/// `reference_asteroid_equatorial_evidence_summary_for_report` copy can call
/// this validation gate instead of reproducing it.
pub fn validate_reference_asteroid_equatorial_evidence(
    evidence: &[ReferenceAsteroidEquatorialEvidence],
) -> Result<(), ReferenceAsteroidEquatorialEvidenceValidationError> {
    if evidence.is_empty() {
        return Err(ReferenceAsteroidEquatorialEvidenceValidationError::Empty);
    }

    let expected_bodies = reference_asteroids();
    let exact_evidence = reference_asteroid_evidence();
    if evidence.len() != expected_bodies.len() || evidence.len() != exact_evidence.len() {
        return Err(
            ReferenceAsteroidEquatorialEvidenceValidationError::BodyOrderMismatch {
                index: evidence.len(),
                expected: expected_bodies
                    .get(evidence.len())
                    .cloned()
                    .unwrap_or_else(|| expected_bodies[expected_bodies.len() - 1].clone()),
                found: evidence
                    .last()
                    .map(|sample| sample.body.clone())
                    .unwrap_or_else(|| expected_bodies[0].clone()),
            },
        );
    }

    let expected_epoch = reference_instant();
    for (index, ((sample, expected_body), exact_sample)) in evidence
        .iter()
        .zip(expected_bodies.iter())
        .zip(exact_evidence.iter())
        .enumerate()
    {
        if sample.body != *expected_body {
            return Err(
                ReferenceAsteroidEquatorialEvidenceValidationError::BodyOrderMismatch {
                    index,
                    expected: expected_body.clone(),
                    found: sample.body.clone(),
                },
            );
        }
        if sample.epoch != expected_epoch {
            return Err(
                ReferenceAsteroidEquatorialEvidenceValidationError::EpochMismatch {
                    index,
                    expected: expected_epoch,
                    found: sample.epoch,
                },
            );
        }

        let exact_ecliptic = EclipticCoordinates::new(
            Longitude::from_degrees(exact_sample.longitude_deg),
            Latitude::from_degrees(exact_sample.latitude_deg),
            Some(exact_sample.distance_au),
        );
        let expected_equatorial = exact_ecliptic.to_equatorial(exact_sample.epoch.mean_obliquity());
        let actual_equatorial = &sample.equatorial;

        if (actual_equatorial.right_ascension.degrees()
            - expected_equatorial.right_ascension.degrees())
        .abs()
            > ASTEROID_EQUATORIAL_TOLERANCE_DEGREES
        {
            return Err(
                ReferenceAsteroidEquatorialEvidenceValidationError::RightAscensionMismatch {
                    index,
                    body: sample.body.clone(),
                },
            );
        }
        if (actual_equatorial.declination.degrees() - expected_equatorial.declination.degrees())
            .abs()
            > ASTEROID_EQUATORIAL_TOLERANCE_DEGREES
        {
            return Err(
                ReferenceAsteroidEquatorialEvidenceValidationError::DeclinationMismatch {
                    index,
                    body: sample.body.clone(),
                },
            );
        }
        match (
            actual_equatorial.distance_au,
            expected_equatorial.distance_au,
        ) {
            (Some(actual), Some(expected))
                if (actual - expected).abs() <= ASTEROID_DISTANCE_TOLERANCE_AU => {}
            (Some(_), Some(_)) => {
                return Err(
                    ReferenceAsteroidEquatorialEvidenceValidationError::DistanceMismatch {
                        index,
                        body: sample.body.clone(),
                    },
                );
            }
            _ => {
                return Err(
                    ReferenceAsteroidEquatorialEvidenceValidationError::DistanceMismatch {
                        index,
                        body: sample.body.clone(),
                    },
                );
            }
        }
    }

    Ok(())
}

/// Compact release-facing summary for the reference asteroid source coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceAsteroidSourceWindowSummary {
    /// Number of reference-asteroid samples in the source slice.
    pub sample_count: usize,
    /// Bodies covered by the source slice in first-seen order.
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

impl ReferenceAsteroidSourceWindowSummary {
    /// Returns `Ok(())` when the reference asteroid source windows still match the checked-in slice.
    pub fn validate(&self) -> Result<(), ReferenceAsteroidSourceWindowSummaryValidationError> {
        let Some(expected) = reference_asteroid_source_window_summary_details() else {
            return Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        };

        if self.sample_count != expected.sample_count {
            return Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.windows != expected.windows {
            return Err(
                ReferenceAsteroidSourceWindowSummaryValidationError::FieldOutOfSync {
                    field: "windows",
                },
            );
        }

        Ok(())
    }
}

/// Validation error for a reference asteroid source window summary that drifted from the current slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceAsteroidSourceWindowSummaryValidationError {
    /// A summary field is out of sync with the checked-in reference asteroid windows.
    FieldOutOfSync {
        /// Name of the summary field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for ReferenceAsteroidSourceWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the reference asteroid source window summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for ReferenceAsteroidSourceWindowSummaryValidationError {}

pub(crate) fn reference_asteroid_source_window_summary_details(
) -> Option<ReferenceAsteroidSourceWindowSummary> {
    let entries = reference_snapshot();
    if entries.is_empty() {
        return None;
    }

    let source_windows = reference_snapshot_source_window_summary_details()?;
    let mut windows = Vec::new();
    for body in reference_asteroids() {
        if let Some(window) = source_windows
            .windows
            .iter()
            .find(|window| window.body == *body)
        {
            windows.push(window.clone());
        }
    }

    if windows.is_empty() {
        return None;
    }

    let asteroid_entries = entries
        .iter()
        .filter(|entry| is_reference_asteroid(&entry.body))
        .collect::<Vec<_>>();

    let earliest_epoch = windows
        .iter()
        .map(|window| window.earliest_epoch)
        .min_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("reference asteroid source windows should not be empty after collection");
    let latest_epoch = windows
        .iter()
        .map(|window| window.latest_epoch)
        .max_by(|left, right| left.julian_day.days().total_cmp(&right.julian_day.days()))
        .expect("reference asteroid source windows should not be empty after collection");

    Some(ReferenceAsteroidSourceWindowSummary {
        sample_count: asteroid_entries.len(),
        sample_bodies: reference_asteroids().to_vec(),
        epoch_count: asteroid_entries
            .iter()
            .map(|entry| entry.epoch.julian_day.days().to_bits())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_epoch,
        latest_epoch,
        windows,
    })
}

/// Returns the compact typed summary for the reference asteroid source coverage.
pub fn reference_asteroid_source_window_summary() -> Option<ReferenceAsteroidSourceWindowSummary> {
    reference_asteroid_source_window_summary_details()
}

#[cfg(test)]
mod tests;
