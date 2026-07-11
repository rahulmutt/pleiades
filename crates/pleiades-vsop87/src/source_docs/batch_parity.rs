use pleiades_types::{CelestialBody, CoordinateFrame, Instant, TimeScale};
use std::fmt;

use pleiades_backend::{EphemerisBackend, EphemerisRequest, QualityAnnotation};

use crate::backend::Vsop87Backend;
use crate::profiles::{source_file_for_body, source_kind_for_body, Vsop87BodySourceKind};

use super::documentation::{body_labels_are_unique, format_celestial_bodies};
use super::evidence::{
    canonical_epoch_body_evidence, canonical_epoch_requests, canonical_epoch_samples, median_f64,
    percentile_f64, rms_f64, source_body_class, Vsop87SourceBodyClass,
};
use super::request_corpus::{
    canonical_j1900_equatorial_batch_parity_requests,
    canonical_mixed_time_scale_batch_parity_requests,
    supported_body_j1900_ecliptic_batch_parity_requests,
    supported_body_j1900_equatorial_batch_parity_requests,
    supported_body_j2000_ecliptic_batch_parity_request_corpus,
    supported_body_j2000_ecliptic_batch_parity_requests,
    supported_body_j2000_equatorial_batch_parity_requests,
};

const J1900: f64 = 2_415_020.0;
const J2000: f64 = 2_451_545.0;

fn canonical_batch_parity_counts(
    backend: &Vsop87Backend,
    requests: &[EphemerisRequest],
) -> Option<(Vec<CelestialBody>, usize, usize, usize, usize)> {
    let results = backend.positions(requests).ok()?;

    if results.len() != requests.len() {
        return None;
    }

    let mut sample_bodies = Vec::with_capacity(results.len());
    let mut exact_count = 0usize;
    let mut interpolated_count = 0usize;
    let mut approximate_count = 0usize;
    let mut unknown_count = 0usize;

    for (request, result) in requests.iter().zip(results.iter()) {
        let single = backend.position(request).ok()?;
        if single != *result {
            return None;
        }

        sample_bodies.push(result.body.clone());
        match result.quality {
            QualityAnnotation::Exact => exact_count += 1,
            QualityAnnotation::Interpolated => interpolated_count += 1,
            QualityAnnotation::Approximate => approximate_count += 1,
            QualityAnnotation::Unknown => unknown_count += 1,
            _ => unknown_count += 1,
        }
    }

    Some((
        sample_bodies,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    ))
}

fn validate_batch_parity_quality_counts(
    actual_counts: (usize, usize, usize, usize),
    expected_counts: (usize, usize, usize, usize),
) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
    if actual_counts != expected_counts {
        return Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "quality_counts",
            },
        );
    }

    Ok(())
}

fn canonical_j2000_batch_parity_expected_bodies() -> Vec<CelestialBody> {
    canonical_epoch_samples()
        .iter()
        .map(|sample| sample.body.clone())
        .collect()
}

fn canonical_j1900_batch_parity_expected_bodies() -> Vec<CelestialBody> {
    Vsop87Backend::supported_bodies().to_vec()
}

/// Validation error for a VSOP87 canonical batch-parity summary that drifted
/// from the current backend-derived counts.
#[derive(Clone, Debug, Eq, PartialEq)]

pub enum Vsop87CanonicalBatchParitySummaryValidationError {
    /// A rendered summary field no longer matches the current derived evidence.
    FieldOutOfSync {
        /// Name of the summary field that drifted from the derived evidence.
        field: &'static str,
    },
}

impl fmt::Display for Vsop87CanonicalBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the VSOP87 canonical batch parity summary field `{field}` is out of sync with the current canonical evidence"
            ),
        }
    }
}

impl std::error::Error for Vsop87CanonicalBatchParitySummaryValidationError {}

/// Backend-owned summary for the canonical J2000 batch-path regression.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalJ2000BatchParitySummary {
    /// Number of requests exercised through the batch regression.
    pub sample_count: usize,
    /// Bodies exercised through the batch regression in release-facing order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Reference epoch used by the batch regression.
    pub reference_epoch: Instant,
    /// Coordinate frame used by the batch regression.
    pub frame: CoordinateFrame,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

impl Vsop87CanonicalJ2000BatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 canonical J2000 batch parity: {} requests across {} bodies ({}) at JD {:.1} ({}) in {} frame; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.sample_count,
            self.sample_bodies.len(),
            format_celestial_bodies(&self.sample_bodies),
            self.reference_epoch.julian_day.days(),
            self.reference_epoch.scale,
            self.frame,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != canonical_j2000_batch_parity_expected_bodies() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.reference_epoch
            != Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt)
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "reference_epoch",
                },
            );
        }
        if self.frame != CoordinateFrame::Ecliptic {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" },
            );
        }
        if (self.exact_count
            + self.interpolated_count
            + self.approximate_count
            + self.unknown_count)
            != self.sample_count
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        }
        let backend = Vsop87Backend::new();
        let requests = canonical_epoch_requests();
        let Some((
            _,
            expected_exact_count,
            expected_interpolated_count,
            expected_approximate_count,
            expected_unknown_count,
        )) = canonical_batch_parity_counts(&backend, &requests)
        else {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        };
        validate_batch_parity_quality_counts(
            (
                self.exact_count,
                self.interpolated_count,
                self.approximate_count,
                self.unknown_count,
            ),
            (
                expected_exact_count,
                expected_interpolated_count,
                expected_approximate_count,
                expected_unknown_count,
            ),
        )?;

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalJ2000BatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned canonical J2000 batch-path regression summary.
pub fn canonical_j2000_batch_parity_summary() -> Option<Vsop87CanonicalJ2000BatchParitySummary> {
    let backend = Vsop87Backend::new();
    let reference_epoch = Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt);
    let requests = canonical_epoch_requests();
    let (sample_bodies, exact_count, interpolated_count, approximate_count, unknown_count) =
        canonical_batch_parity_counts(&backend, &requests)?;

    Some(Vsop87CanonicalJ2000BatchParitySummary {
        sample_count: requests.len(),
        sample_bodies,
        reference_epoch,
        frame: CoordinateFrame::Ecliptic,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

/// Backend-owned summary for the canonical mixed TT/TDB batch-path regression.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalMixedTimeScaleBatchParitySummary {
    /// Number of requests exercised through the batch regression.
    pub sample_count: usize,
    /// Bodies exercised through the batch regression in release-facing order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Reference epoch used by the batch regression.
    pub reference_epoch: Instant,
    /// Coordinate frame used by the batch regression.
    pub frame: CoordinateFrame,
    /// Number of TT-tagged results observed in the batch regression.
    pub tt_request_count: usize,
    /// Number of TDB-tagged results observed in the batch regression.
    pub tdb_request_count: usize,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

impl Vsop87CanonicalMixedTimeScaleBatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 canonical mixed TT/TDB batch parity: {} requests across {} bodies ({}) at JD {:.1} (TT/TDB mix) in {} frame; TT requests={}, TDB requests={}, quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.sample_count,
            self.sample_bodies.len(),
            format_celestial_bodies(&self.sample_bodies),
            self.reference_epoch.julian_day.days(),
            self.frame,
            self.tt_request_count,
            self.tdb_request_count,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    ///
    /// The canonical alternating TT/TDB request counts already imply the mixed-slice posture for
    /// this fixed 11-body slice, so the validation path keeps its focus on the exact counts,
    /// bodies, epoch, and frame rather than introducing a separate degenerate-mix guard.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != canonical_j2000_batch_parity_expected_bodies() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.reference_epoch
            != Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt)
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "reference_epoch",
                },
            );
        }
        if self.frame != CoordinateFrame::Ecliptic {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" },
            );
        }
        let expected_tt_request_count = self.sample_count.div_ceil(2);
        let expected_tdb_request_count = self.sample_count / 2;
        if self.tt_request_count != expected_tt_request_count {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "tt_request_count",
                },
            );
        }
        if self.tdb_request_count != expected_tdb_request_count {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "tdb_request_count",
                },
            );
        }
        if (self.exact_count
            + self.interpolated_count
            + self.approximate_count
            + self.unknown_count)
            != self.sample_count
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        }
        let backend = Vsop87Backend::new();
        let requests = canonical_mixed_time_scale_batch_parity_requests();
        let Some((
            _,
            expected_exact_count,
            expected_interpolated_count,
            expected_approximate_count,
            expected_unknown_count,
        )) = canonical_batch_parity_counts(&backend, &requests)
        else {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        };
        validate_batch_parity_quality_counts(
            (
                self.exact_count,
                self.interpolated_count,
                self.approximate_count,
                self.unknown_count,
            ),
            (
                expected_exact_count,
                expected_interpolated_count,
                expected_approximate_count,
                expected_unknown_count,
            ),
        )?;

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalMixedTimeScaleBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned canonical mixed TT/TDB batch-path regression summary.
pub fn canonical_mixed_time_scale_batch_parity_summary(
) -> Option<Vsop87CanonicalMixedTimeScaleBatchParitySummary> {
    let backend = Vsop87Backend::new();
    let reference_epoch = Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt);
    let requests = canonical_mixed_time_scale_batch_parity_requests();
    let (sample_bodies, exact_count, interpolated_count, approximate_count, unknown_count) =
        canonical_batch_parity_counts(&backend, &requests)?;
    let tt_request_count = requests
        .iter()
        .filter(|request| request.instant.scale == TimeScale::Tt)
        .count();
    let tdb_request_count = requests.len() - tt_request_count;

    Some(Vsop87CanonicalMixedTimeScaleBatchParitySummary {
        sample_count: requests.len(),
        sample_bodies,
        reference_epoch,
        frame: CoordinateFrame::Ecliptic,
        tt_request_count,
        tdb_request_count,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

/// Backend-owned summary for the canonical J1900 batch-path regression.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalJ1900BatchParitySummary {
    /// Number of requests exercised through the batch regression.
    pub sample_count: usize,
    /// Bodies exercised through the batch regression in release-facing order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Reference epoch used by the batch regression.
    pub reference_epoch: Instant,
    /// Coordinate frame used by the batch regression.
    pub frame: CoordinateFrame,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

impl Vsop87CanonicalJ1900BatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 canonical J1900 batch parity: {} requests across {} bodies ({}) at JD {:.1} ({}) in {} frame; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.sample_count,
            self.sample_bodies.len(),
            format_celestial_bodies(&self.sample_bodies),
            self.reference_epoch.julian_day.days(),
            self.reference_epoch.scale,
            self.frame,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != canonical_j1900_batch_parity_expected_bodies() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.reference_epoch
            != Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb)
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "reference_epoch",
                },
            );
        }
        if self.frame != CoordinateFrame::Equatorial {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" },
            );
        }
        if (self.exact_count
            + self.interpolated_count
            + self.approximate_count
            + self.unknown_count)
            != self.sample_count
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        }
        let backend = Vsop87Backend::new();
        let requests = canonical_j1900_equatorial_batch_parity_requests();
        let Some((
            _,
            expected_exact_count,
            expected_interpolated_count,
            expected_approximate_count,
            expected_unknown_count,
        )) = canonical_batch_parity_counts(&backend, &requests)
        else {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        };
        validate_batch_parity_quality_counts(
            (
                self.exact_count,
                self.interpolated_count,
                self.approximate_count,
                self.unknown_count,
            ),
            (
                expected_exact_count,
                expected_interpolated_count,
                expected_approximate_count,
                expected_unknown_count,
            ),
        )?;

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalJ1900BatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned canonical J1900 batch-path regression summary.
pub fn canonical_j1900_batch_parity_summary() -> Option<Vsop87CanonicalJ1900BatchParitySummary> {
    let backend = Vsop87Backend::new();
    let requests = canonical_j1900_equatorial_batch_parity_requests();
    let reference_epoch = Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb);
    let (sample_bodies, exact_count, interpolated_count, approximate_count, unknown_count) =
        canonical_batch_parity_counts(&backend, &requests)?;

    Some(Vsop87CanonicalJ1900BatchParitySummary {
        sample_count: requests.len(),
        sample_bodies,
        reference_epoch,
        frame: CoordinateFrame::Equatorial,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

/// Backend-owned summary for the supported-body J2000 ecliptic batch-path regression.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87SupportedBodyJ2000EclipticBatchParitySummary {
    /// Number of requests exercised through the batch regression.
    pub sample_count: usize,
    /// Bodies exercised through the batch regression in release-facing order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Reference epoch used by the batch regression.
    pub reference_epoch: Instant,
    /// Coordinate frame used by the batch regression.
    pub frame: CoordinateFrame,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

impl Vsop87SupportedBodyJ2000EclipticBatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 supported-body J2000 ecliptic batch parity: {} requests across {} bodies ({}) at JD {:.1} ({}) in {} frame; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.sample_count,
            self.sample_bodies.len(),
            format_celestial_bodies(&self.sample_bodies),
            self.reference_epoch.julian_day.days(),
            self.reference_epoch.scale,
            self.frame,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != Vsop87Backend::supported_bodies().to_vec() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.reference_epoch
            != Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tdb)
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "reference_epoch",
                },
            );
        }
        if self.frame != CoordinateFrame::Ecliptic {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" },
            );
        }
        if (self.exact_count
            + self.interpolated_count
            + self.approximate_count
            + self.unknown_count)
            != self.sample_count
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        }
        let backend = Vsop87Backend::new();
        let requests = supported_body_j2000_ecliptic_batch_parity_requests();
        let Some((
            _,
            expected_exact_count,
            expected_interpolated_count,
            expected_approximate_count,
            expected_unknown_count,
        )) = canonical_batch_parity_counts(&backend, &requests)
        else {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        };
        validate_batch_parity_quality_counts(
            (
                self.exact_count,
                self.interpolated_count,
                self.approximate_count,
                self.unknown_count,
            ),
            (
                expected_exact_count,
                expected_interpolated_count,
                expected_approximate_count,
                expected_unknown_count,
            ),
        )?;

        Ok(())
    }
}

impl fmt::Display for Vsop87SupportedBodyJ2000EclipticBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned supported-body J2000 ecliptic batch-path regression summary.
pub fn supported_body_j2000_ecliptic_batch_parity_summary(
) -> Option<Vsop87SupportedBodyJ2000EclipticBatchParitySummary> {
    let backend = Vsop87Backend::new();
    let requests = supported_body_j2000_ecliptic_batch_parity_request_corpus();
    let reference_epoch = requests.first()?.instant;
    let (sample_bodies, exact_count, interpolated_count, approximate_count, unknown_count) =
        canonical_batch_parity_counts(&backend, &requests)?;

    Some(Vsop87SupportedBodyJ2000EclipticBatchParitySummary {
        sample_count: requests.len(),
        sample_bodies,
        reference_epoch,
        frame: CoordinateFrame::Ecliptic,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

/// Backend-owned summary for the supported-body J2000 equatorial batch-path regression.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87SupportedBodyJ2000EquatorialBatchParitySummary {
    /// Number of requests exercised through the batch regression.
    pub sample_count: usize,
    /// Bodies exercised through the batch regression in release-facing order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Reference epoch used by the batch regression.
    pub reference_epoch: Instant,
    /// Coordinate frame used by the batch regression.
    pub frame: CoordinateFrame,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

impl Vsop87SupportedBodyJ2000EquatorialBatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 supported-body J2000 equatorial batch parity: {} requests across {} bodies ({}) at JD {:.1} ({}) in {} frame; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.sample_count,
            self.sample_bodies.len(),
            format_celestial_bodies(&self.sample_bodies),
            self.reference_epoch.julian_day.days(),
            self.reference_epoch.scale,
            self.frame,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != Vsop87Backend::supported_bodies().to_vec() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.reference_epoch
            != Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tdb)
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "reference_epoch",
                },
            );
        }
        if self.frame != CoordinateFrame::Equatorial {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" },
            );
        }
        if (self.exact_count
            + self.interpolated_count
            + self.approximate_count
            + self.unknown_count)
            != self.sample_count
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        }
        let backend = Vsop87Backend::new();
        let requests = supported_body_j2000_equatorial_batch_parity_requests();
        let Some((
            _,
            expected_exact_count,
            expected_interpolated_count,
            expected_approximate_count,
            expected_unknown_count,
        )) = canonical_batch_parity_counts(&backend, &requests)
        else {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        };
        validate_batch_parity_quality_counts(
            (
                self.exact_count,
                self.interpolated_count,
                self.approximate_count,
                self.unknown_count,
            ),
            (
                expected_exact_count,
                expected_interpolated_count,
                expected_approximate_count,
                expected_unknown_count,
            ),
        )?;

        Ok(())
    }
}

impl fmt::Display for Vsop87SupportedBodyJ2000EquatorialBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned supported-body J2000 equatorial batch-path regression summary.
pub fn supported_body_j2000_equatorial_batch_parity_summary(
) -> Option<Vsop87SupportedBodyJ2000EquatorialBatchParitySummary> {
    let backend = Vsop87Backend::new();
    let requests = supported_body_j2000_equatorial_batch_parity_requests();
    let reference_epoch = requests.first()?.instant;
    let (sample_bodies, exact_count, interpolated_count, approximate_count, unknown_count) =
        canonical_batch_parity_counts(&backend, &requests)?;

    Some(Vsop87SupportedBodyJ2000EquatorialBatchParitySummary {
        sample_count: requests.len(),
        sample_bodies,
        reference_epoch,
        frame: CoordinateFrame::Equatorial,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

/// Backend-owned summary for the supported-body J1900 ecliptic batch-path regression.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87SupportedBodyJ1900EclipticBatchParitySummary {
    /// Number of requests exercised through the batch regression.
    pub sample_count: usize,
    /// Bodies exercised through the batch regression in release-facing order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Reference epoch used by the batch regression.
    pub reference_epoch: Instant,
    /// Coordinate frame used by the batch regression.
    pub frame: CoordinateFrame,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

impl Vsop87SupportedBodyJ1900EclipticBatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 supported-body J1900 ecliptic batch parity: {} requests across {} bodies ({}) at JD {:.1} ({}) in {} frame; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.sample_count,
            self.sample_bodies.len(),
            format_celestial_bodies(&self.sample_bodies),
            self.reference_epoch.julian_day.days(),
            self.reference_epoch.scale,
            self.frame,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != Vsop87Backend::supported_bodies().to_vec() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.reference_epoch
            != Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb)
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "reference_epoch",
                },
            );
        }
        if self.frame != CoordinateFrame::Ecliptic {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" },
            );
        }
        if (self.exact_count
            + self.interpolated_count
            + self.approximate_count
            + self.unknown_count)
            != self.sample_count
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        }
        let backend = Vsop87Backend::new();
        let requests = supported_body_j1900_ecliptic_batch_parity_requests();
        let Some((
            _,
            expected_exact_count,
            expected_interpolated_count,
            expected_approximate_count,
            expected_unknown_count,
        )) = canonical_batch_parity_counts(&backend, &requests)
        else {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        };
        validate_batch_parity_quality_counts(
            (
                self.exact_count,
                self.interpolated_count,
                self.approximate_count,
                self.unknown_count,
            ),
            (
                expected_exact_count,
                expected_interpolated_count,
                expected_approximate_count,
                expected_unknown_count,
            ),
        )?;

        Ok(())
    }
}

impl fmt::Display for Vsop87SupportedBodyJ1900EclipticBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned supported-body J1900 ecliptic batch-path regression summary.
pub fn supported_body_j1900_ecliptic_batch_parity_summary(
) -> Option<Vsop87SupportedBodyJ1900EclipticBatchParitySummary> {
    let backend = Vsop87Backend::new();
    let requests = supported_body_j1900_ecliptic_batch_parity_requests();
    let reference_epoch = requests.first()?.instant;
    let (sample_bodies, exact_count, interpolated_count, approximate_count, unknown_count) =
        canonical_batch_parity_counts(&backend, &requests)?;

    Some(Vsop87SupportedBodyJ1900EclipticBatchParitySummary {
        sample_count: requests.len(),
        sample_bodies,
        reference_epoch,
        frame: CoordinateFrame::Ecliptic,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

/// Backend-owned summary for the supported-body J1900 equatorial batch-path regression.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87SupportedBodyJ1900EquatorialBatchParitySummary {
    /// Number of requests exercised through the batch regression.
    pub sample_count: usize,
    /// Bodies exercised through the batch regression in release-facing order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Reference epoch used by the batch regression.
    pub reference_epoch: Instant,
    /// Coordinate frame used by the batch regression.
    pub frame: CoordinateFrame,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

impl Vsop87SupportedBodyJ1900EquatorialBatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 supported-body J1900 equatorial batch parity: {} requests across {} bodies ({}) at JD {:.1} ({}) in {} frame; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.sample_count,
            self.sample_bodies.len(),
            format_celestial_bodies(&self.sample_bodies),
            self.reference_epoch.julian_day.days(),
            self.reference_epoch.scale,
            self.frame,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != Vsop87Backend::supported_bodies().to_vec() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.reference_epoch
            != Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb)
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "reference_epoch",
                },
            );
        }
        if self.frame != CoordinateFrame::Equatorial {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" },
            );
        }
        if (self.exact_count
            + self.interpolated_count
            + self.approximate_count
            + self.unknown_count)
            != self.sample_count
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        }
        let backend = Vsop87Backend::new();
        let requests = supported_body_j1900_equatorial_batch_parity_requests();
        let Some((
            _,
            expected_exact_count,
            expected_interpolated_count,
            expected_approximate_count,
            expected_unknown_count,
        )) = canonical_batch_parity_counts(&backend, &requests)
        else {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        };
        validate_batch_parity_quality_counts(
            (
                self.exact_count,
                self.interpolated_count,
                self.approximate_count,
                self.unknown_count,
            ),
            (
                expected_exact_count,
                expected_interpolated_count,
                expected_approximate_count,
                expected_unknown_count,
            ),
        )?;

        Ok(())
    }
}

impl fmt::Display for Vsop87SupportedBodyJ1900EquatorialBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned supported-body J1900 equatorial batch-path regression summary.
pub fn supported_body_j1900_equatorial_batch_parity_summary(
) -> Option<Vsop87SupportedBodyJ1900EquatorialBatchParitySummary> {
    let backend = Vsop87Backend::new();
    let requests = supported_body_j1900_equatorial_batch_parity_requests();
    let reference_epoch = requests.first()?.instant;
    let (sample_bodies, exact_count, interpolated_count, approximate_count, unknown_count) =
        canonical_batch_parity_counts(&backend, &requests)?;

    Some(Vsop87SupportedBodyJ1900EquatorialBatchParitySummary {
        sample_count: requests.len(),
        sample_bodies,
        reference_epoch,
        frame: CoordinateFrame::Equatorial,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

/// Backend-owned summary for the supported-body canonical batch matrix.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87SupportedBodyCanonicalBatchParitySummary {
    /// Number of supported bodies exercised by each batch slice.
    pub supported_body_count: usize,
    /// Supported-body J2000 ecliptic batch regression summary.
    pub j2000_ecliptic: Vsop87SupportedBodyJ2000EclipticBatchParitySummary,
    /// Supported-body J2000 equatorial batch regression summary.
    pub j2000_equatorial: Vsop87SupportedBodyJ2000EquatorialBatchParitySummary,
    /// Supported-body J1900 ecliptic batch regression summary.
    pub j1900_ecliptic: Vsop87SupportedBodyJ1900EclipticBatchParitySummary,
    /// Supported-body J1900 equatorial batch regression summary.
    pub j1900_equatorial: Vsop87SupportedBodyJ1900EquatorialBatchParitySummary,
}

/// Structured validation errors for a supported-body canonical batch matrix summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError {
    /// A rendered summary field no longer matches the current derived evidence.
    FieldOutOfSync {
        /// Name of the summary field that drifted from the derived evidence.
        field: &'static str,
    },
}

impl fmt::Display for Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the VSOP87 supported-body canonical batch matrix field `{field}` is out of sync with the current supported-body batch evidence"
            ),
        }
    }
}

impl std::error::Error for Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError {}

impl Vsop87SupportedBodyCanonicalBatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let bodies = if self.j2000_ecliptic.sample_bodies.is_empty() {
            "none".to_string()
        } else {
            format_celestial_bodies(&self.j2000_ecliptic.sample_bodies)
        };

        format!(
            "VSOP87 supported-body canonical batch matrix: {} bodies ({}); slices: J2000 ecliptic {} requests, J2000 equatorial {} requests, J1900 ecliptic {} requests, J1900 equatorial {} requests; batch/single parity preserved across the supported planetary set",
            self.supported_body_count,
            bodies,
            self.j2000_ecliptic.sample_count,
            self.j2000_equatorial.sample_count,
            self.j1900_ecliptic.sample_count,
            self.j1900_equatorial.sample_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    pub fn validate(
        &self,
    ) -> Result<(), Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError> {
        self.j2000_ecliptic.validate().map_err(|_| {
            Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "j2000_ecliptic",
            }
        })?;
        self.j2000_equatorial.validate().map_err(|_| {
            Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "j2000_equatorial",
            }
        })?;
        self.j1900_ecliptic.validate().map_err(|_| {
            Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "j1900_ecliptic",
            }
        })?;
        self.j1900_equatorial.validate().map_err(|_| {
            Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "j1900_equatorial",
            }
        })?;

        let supported_bodies = Vsop87Backend::supported_bodies();
        if self.supported_body_count != supported_bodies.len() {
            return Err(
                Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "supported_body_count",
                },
            );
        }

        let expected_bodies = supported_bodies.to_vec();
        macro_rules! validate_slice {
            ($field:literal, $summary:expr) => {
                let summary = &$summary;
                if summary.sample_count != self.supported_body_count {
                    return Err(
                        Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                            field: $field,
                        },
                    );
                }
                if summary.sample_bodies != expected_bodies {
                    return Err(
                        Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                            field: $field,
                        },
                    );
                }
            };
        }
        validate_slice!("j2000_ecliptic", self.j2000_ecliptic);
        validate_slice!("j2000_equatorial", self.j2000_equatorial);
        validate_slice!("j1900_ecliptic", self.j1900_ecliptic);
        validate_slice!("j1900_equatorial", self.j1900_equatorial);

        if self.j2000_ecliptic.sample_bodies != self.j2000_equatorial.sample_bodies
            || self.j2000_ecliptic.sample_bodies != self.j1900_ecliptic.sample_bodies
            || self.j2000_ecliptic.sample_bodies != self.j1900_equatorial.sample_bodies
        {
            return Err(
                Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "body_order",
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for Vsop87SupportedBodyCanonicalBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned supported-body canonical batch matrix summary.
pub fn supported_body_canonical_batch_parity_summary(
) -> Option<Vsop87SupportedBodyCanonicalBatchParitySummary> {
    Some(Vsop87SupportedBodyCanonicalBatchParitySummary {
        supported_body_count: Vsop87Backend::supported_bodies().len(),
        j2000_ecliptic: supported_body_j2000_ecliptic_batch_parity_summary()?,
        j2000_equatorial: supported_body_j2000_equatorial_batch_parity_summary()?,
        j1900_ecliptic: supported_body_j1900_ecliptic_batch_parity_summary()?,
        j1900_equatorial: supported_body_j1900_equatorial_batch_parity_summary()?,
    })
}

/// Returns the supported-body canonical batch matrix request corpus used by the VSOP87 batch-path evidence.
///
/// The requests preserve the supported-body order and concatenate the J2000/J1900
/// ecliptic and equatorial slices so validation and reproducibility tooling can
/// reuse the full supported-planet matrix without reconstructing it from the
/// summary metadata.
pub fn supported_body_canonical_batch_parity_requests() -> Vec<EphemerisRequest> {
    let mut requests = supported_body_j2000_ecliptic_batch_parity_requests();
    requests.extend(supported_body_j2000_equatorial_batch_parity_requests());
    requests.extend(supported_body_j1900_ecliptic_batch_parity_requests());
    requests.extend(supported_body_j1900_equatorial_batch_parity_requests());
    requests
}

/// Returns the supported-body canonical batch matrix request corpus used by the VSOP87 batch-path evidence.
///
/// This is a compatibility alias for [`supported_body_canonical_batch_parity_requests`].
#[doc(alias = "supported_body_canonical_batch_parity_requests")]
#[doc(alias = "supported_body_canonical_batch_matrix_request_corpus")]
pub fn supported_body_canonical_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    supported_body_canonical_batch_parity_requests()
}

/// This is a compatibility alias for [`supported_body_canonical_batch_parity_request_corpus`].
#[doc(alias = "supported_body_canonical_batch_parity_request_corpus")]
pub fn supported_body_canonical_batch_matrix_request_corpus() -> Vec<EphemerisRequest> {
    supported_body_canonical_batch_parity_request_corpus()
}

/// This is a compatibility alias for [`supported_body_canonical_batch_matrix_request_corpus`].
#[doc(alias = "supported_body_canonical_batch_matrix_request_corpus")]
pub fn supported_body_canonical_batch_matrix_requests() -> Vec<EphemerisRequest> {
    supported_body_canonical_batch_matrix_request_corpus()
}

/// This is a compatibility alias for [`supported_body_canonical_batch_matrix_requests`].
#[doc(alias = "supported_body_canonical_batch_matrix_requests")]
pub fn supported_body_canonical_request_corpus() -> Vec<EphemerisRequest> {
    supported_body_canonical_batch_matrix_requests()
}

/// Backend-owned summary for the canonical J2000 source-backed body classes.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87SourceBodyClassEvidenceSummary {
    /// Body class covered by this summary.
    pub class: Vsop87SourceBodyClass,
    /// Number of canonical samples measured for the class.
    pub sample_count: usize,
    /// Canonical bodies measured in source-backed order for the class.
    pub sample_bodies: Vec<CelestialBody>,
    /// Number of samples within the interim limits.
    pub within_interim_limits_count: usize,
    /// Number of samples outside the current interim limits.
    pub outside_interim_limit_count: usize,
    /// Bodies outside the current interim limits.
    pub outside_interim_limit_bodies: Vec<CelestialBody>,
    /// Body with the maximum absolute longitude delta.
    pub max_longitude_delta_body: CelestialBody,
    /// Calculation family behind the maximum longitude delta body.
    pub max_longitude_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum longitude delta body.
    pub max_longitude_delta_source_file: &'static str,
    /// Maximum absolute longitude delta in degrees.
    pub max_longitude_delta_deg: f64,
    /// Interim longitude delta limit for the body that drives the maximum.
    pub max_longitude_delta_limit_deg: f64,
    /// Body with the maximum absolute latitude delta.
    pub max_latitude_delta_body: CelestialBody,
    /// Calculation family behind the maximum latitude delta body.
    pub max_latitude_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum latitude delta body.
    pub max_latitude_delta_source_file: &'static str,
    /// Maximum absolute latitude delta in degrees.
    pub max_latitude_delta_deg: f64,
    /// Interim latitude delta limit for the body that drives the maximum.
    pub max_latitude_delta_limit_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: CelestialBody,
    /// Calculation family behind the maximum distance delta body.
    pub max_distance_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum distance delta body.
    pub max_distance_delta_source_file: &'static str,
    /// Maximum absolute distance delta in astronomical units.
    pub max_distance_delta_au: f64,
    /// Interim distance delta limit for the body that drives the maximum.
    pub max_distance_delta_limit_au: f64,
    /// Mean absolute longitude delta in degrees.
    pub mean_longitude_delta_deg: f64,
    /// Median absolute longitude delta in degrees.
    pub median_longitude_delta_deg: f64,
    /// 95th-percentile absolute longitude delta in degrees.
    pub percentile_longitude_delta_deg: f64,
    /// Root-mean-square longitude delta in degrees.
    pub rms_longitude_delta_deg: f64,
    /// Mean absolute latitude delta in degrees.
    pub mean_latitude_delta_deg: f64,
    /// Median absolute latitude delta in degrees.
    pub median_latitude_delta_deg: f64,
    /// 95th-percentile absolute latitude delta in degrees.
    pub percentile_latitude_delta_deg: f64,
    /// Root-mean-square latitude delta in degrees.
    pub rms_latitude_delta_deg: f64,
    /// Mean absolute distance delta in astronomical units.
    pub mean_distance_delta_au: f64,
    /// Median absolute distance delta in astronomical units.
    pub median_distance_delta_au: f64,
    /// 95th-percentile absolute distance delta in astronomical units.
    pub percentile_distance_delta_au: f64,
    /// Root-mean-square distance delta in astronomical units.
    pub rms_distance_delta_au: f64,
}

impl Vsop87SourceBodyClassEvidenceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format_source_body_class_evidence_entry(self)
    }

    /// Returns the validated compact summary line when the class evidence still matches
    /// the current canonical evidence.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Vsop87SourceBodyClassEvidenceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the summary still matches the current derived counts.
    pub fn validate(&self) -> Result<(), Vsop87SourceBodyClassEvidenceSummaryValidationError> {
        let Some(expected) = source_body_class_evidence_summary().and_then(|summaries| {
            summaries
                .into_iter()
                .find(|summary| summary.class == self.class)
        }) else {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        };

        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.within_interim_limits_count != expected.within_interim_limits_count {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "within_interim_limits_count",
                },
            );
        }
        if self.outside_interim_limit_count != expected.outside_interim_limit_count {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_count",
                },
            );
        }
        if self.within_interim_limits_count + self.outside_interim_limit_count != self.sample_count
        {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "interim_limit_counts",
                },
            );
        }
        if self.outside_interim_limit_count != self.outside_interim_limit_bodies.len() {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_bodies",
                },
            );
        }
        if self.outside_interim_limit_bodies != expected.outside_interim_limit_bodies {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_bodies",
                },
            );
        }
        if !body_labels_are_unique(&self.sample_bodies) {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if !body_labels_are_unique(&self.outside_interim_limit_bodies) {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_bodies",
                },
            );
        }
        if self
            .outside_interim_limit_bodies
            .iter()
            .any(|body| !self.sample_bodies.contains(body))
        {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_bodies",
                },
            );
        }
        if !self.sample_bodies.contains(&self.max_longitude_delta_body) {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "max_longitude_delta_body",
                },
            );
        }
        if !self.sample_bodies.contains(&self.max_latitude_delta_body) {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "max_latitude_delta_body",
                },
            );
        }
        if !self.sample_bodies.contains(&self.max_distance_delta_body) {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "max_distance_delta_body",
                },
            );
        }
        if self.max_longitude_delta_source_file.trim().is_empty() {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "max_longitude_delta_source_file",
                },
            );
        }
        if self.max_latitude_delta_source_file.trim().is_empty() {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "max_latitude_delta_source_file",
                },
            );
        }
        if self.max_distance_delta_source_file.trim().is_empty() {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "max_distance_delta_source_file",
                },
            );
        }
        validate_source_body_class_summary_peak_source_metadata(
            "max_longitude_delta_source_kind",
            "max_longitude_delta_source_file",
            &self.max_longitude_delta_body,
            self.max_longitude_delta_source_kind,
            self.max_longitude_delta_source_file,
        )?;
        validate_source_body_class_summary_peak_source_metadata(
            "max_latitude_delta_source_kind",
            "max_latitude_delta_source_file",
            &self.max_latitude_delta_body,
            self.max_latitude_delta_source_kind,
            self.max_latitude_delta_source_file,
        )?;
        validate_source_body_class_summary_peak_source_metadata(
            "max_distance_delta_source_kind",
            "max_distance_delta_source_file",
            &self.max_distance_delta_body,
            self.max_distance_delta_source_kind,
            self.max_distance_delta_source_file,
        )?;
        for (field, value) in [
            ("max_longitude_delta_deg", self.max_longitude_delta_deg),
            (
                "max_longitude_delta_limit_deg",
                self.max_longitude_delta_limit_deg,
            ),
            ("max_latitude_delta_deg", self.max_latitude_delta_deg),
            (
                "max_latitude_delta_limit_deg",
                self.max_latitude_delta_limit_deg,
            ),
            ("max_distance_delta_au", self.max_distance_delta_au),
            (
                "max_distance_delta_limit_au",
                self.max_distance_delta_limit_au,
            ),
            ("mean_longitude_delta_deg", self.mean_longitude_delta_deg),
            (
                "median_longitude_delta_deg",
                self.median_longitude_delta_deg,
            ),
            (
                "percentile_longitude_delta_deg",
                self.percentile_longitude_delta_deg,
            ),
            ("rms_longitude_delta_deg", self.rms_longitude_delta_deg),
            ("mean_latitude_delta_deg", self.mean_latitude_delta_deg),
            ("median_latitude_delta_deg", self.median_latitude_delta_deg),
            (
                "percentile_latitude_delta_deg",
                self.percentile_latitude_delta_deg,
            ),
            ("rms_latitude_delta_deg", self.rms_latitude_delta_deg),
            ("mean_distance_delta_au", self.mean_distance_delta_au),
            ("median_distance_delta_au", self.median_distance_delta_au),
            (
                "percentile_distance_delta_au",
                self.percentile_distance_delta_au,
            ),
            ("rms_distance_delta_au", self.rms_distance_delta_au),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(
                    Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync { field },
                );
            }
        }
        if self.mean_longitude_delta_deg > self.max_longitude_delta_deg
            || self.median_longitude_delta_deg > self.percentile_longitude_delta_deg
            || self.percentile_longitude_delta_deg > self.max_longitude_delta_deg
            || self.rms_longitude_delta_deg > self.max_longitude_delta_deg
        {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "median_longitude_delta_deg",
                },
            );
        }
        if self.mean_latitude_delta_deg > self.max_latitude_delta_deg
            || self.median_latitude_delta_deg > self.percentile_latitude_delta_deg
            || self.percentile_latitude_delta_deg > self.max_latitude_delta_deg
            || self.rms_latitude_delta_deg > self.max_latitude_delta_deg
        {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "median_latitude_delta_deg",
                },
            );
        }
        if self.mean_distance_delta_au > self.max_distance_delta_au
            || self.median_distance_delta_au > self.percentile_distance_delta_au
            || self.percentile_distance_delta_au > self.max_distance_delta_au
            || self.rms_distance_delta_au > self.max_distance_delta_au
        {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "median_distance_delta_au",
                },
            );
        }

        Ok(())
    }
}

/// Validation error for a VSOP87 source-backed body-class evidence summary that drifted
/// from the current canonical evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87SourceBodyClassEvidenceSummaryValidationError {
    /// A rendered summary field no longer matches the current derived evidence.
    FieldOutOfSync {
        /// Name of the summary field that drifted from the derived evidence.
        field: &'static str,
    },
}

impl fmt::Display for Vsop87SourceBodyClassEvidenceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the VSOP87 source-backed body-class evidence summary field `{field}` is out of sync with the current canonical evidence"
            ),
        }
    }
}

impl std::error::Error for Vsop87SourceBodyClassEvidenceSummaryValidationError {}

fn validate_source_body_class_summary_peak_source_metadata(
    field_kind: &'static str,
    field_file: &'static str,
    body: &CelestialBody,
    source_kind: Vsop87BodySourceKind,
    source_file: &'static str,
) -> Result<(), Vsop87SourceBodyClassEvidenceSummaryValidationError> {
    let expected_source_kind = source_kind_for_body(body.clone()).ok_or(
        Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync { field: field_kind },
    )?;
    let expected_source_file = source_file_for_body(body).ok_or(
        Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync { field: field_file },
    )?;

    if expected_source_kind != source_kind {
        return Err(
            Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                field: field_kind,
            },
        );
    }
    if expected_source_file != source_file {
        return Err(
            Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                field: field_file,
            },
        );
    }

    Ok(())
}

impl fmt::Display for Vsop87SourceBodyClassEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned canonical J2000 source-backed body-class evidence.
pub fn source_body_class_evidence_summary() -> Option<Vec<Vsop87SourceBodyClassEvidenceSummary>> {
    let evidence = canonical_epoch_body_evidence()?;
    let mut summaries = Vec::new();

    for class in Vsop87SourceBodyClass::ALL {
        let class_rows: Vec<_> = evidence
            .iter()
            .filter(|row| source_body_class(&row.body) == class)
            .collect();
        if class_rows.is_empty() {
            continue;
        }

        let sample_bodies = class_rows
            .iter()
            .map(|row| row.body.clone())
            .collect::<Vec<_>>();
        let mut longitude_values = Vec::with_capacity(class_rows.len());
        let mut latitude_values = Vec::with_capacity(class_rows.len());
        let mut distance_values = Vec::with_capacity(class_rows.len());
        let mut max_longitude_delta_body = class_rows[0].body.clone();
        let mut max_longitude_delta_source_kind = class_rows[0].source_kind;
        let mut max_longitude_delta_source_file = class_rows[0].source_file;
        let mut max_longitude_delta_deg = class_rows[0].longitude_delta_deg;
        let mut max_longitude_delta_limit_deg = class_rows[0].longitude_limit_deg;
        let mut max_latitude_delta_body = class_rows[0].body.clone();
        let mut max_latitude_delta_source_kind = class_rows[0].source_kind;
        let mut max_latitude_delta_source_file = class_rows[0].source_file;
        let mut max_latitude_delta_deg = class_rows[0].latitude_delta_deg;
        let mut max_latitude_delta_limit_deg = class_rows[0].latitude_limit_deg;
        let mut max_distance_delta_body = class_rows[0].body.clone();
        let mut max_distance_delta_source_kind = class_rows[0].source_kind;
        let mut max_distance_delta_source_file = class_rows[0].source_file;
        let mut max_distance_delta_au = class_rows[0].distance_delta_au;
        let mut max_distance_delta_limit_au = class_rows[0].distance_limit_au;
        let mut within_interim_limits_count = 0usize;
        let mut outside_interim_limit_bodies = Vec::new();

        for row in &class_rows {
            longitude_values.push(row.longitude_delta_deg);
            latitude_values.push(row.latitude_delta_deg);
            distance_values.push(row.distance_delta_au);
            if row.within_interim_limits {
                within_interim_limits_count += 1;
            } else {
                outside_interim_limit_bodies.push(row.body.clone());
            }

            if row.longitude_delta_deg > max_longitude_delta_deg {
                max_longitude_delta_body = row.body.clone();
                max_longitude_delta_source_kind = row.source_kind;
                max_longitude_delta_source_file = row.source_file;
                max_longitude_delta_deg = row.longitude_delta_deg;
                max_longitude_delta_limit_deg = row.longitude_limit_deg;
            }
            if row.latitude_delta_deg > max_latitude_delta_deg {
                max_latitude_delta_body = row.body.clone();
                max_latitude_delta_source_kind = row.source_kind;
                max_latitude_delta_source_file = row.source_file;
                max_latitude_delta_deg = row.latitude_delta_deg;
                max_latitude_delta_limit_deg = row.latitude_limit_deg;
            }
            if row.distance_delta_au > max_distance_delta_au {
                max_distance_delta_body = row.body.clone();
                max_distance_delta_source_kind = row.source_kind;
                max_distance_delta_source_file = row.source_file;
                max_distance_delta_au = row.distance_delta_au;
                max_distance_delta_limit_au = row.distance_limit_au;
            }
        }

        let sample_count = class_rows.len();
        let mut longitude_values_for_median = longitude_values.clone();
        let mut longitude_values_for_percentile = longitude_values;
        let mut latitude_values_for_median = latitude_values.clone();
        let mut latitude_values_for_percentile = latitude_values;
        let mut distance_values_for_median = distance_values.clone();
        let mut distance_values_for_percentile = distance_values;
        summaries.push(Vsop87SourceBodyClassEvidenceSummary {
            class,
            sample_count,
            sample_bodies,
            within_interim_limits_count,
            outside_interim_limit_count: sample_count - within_interim_limits_count,
            outside_interim_limit_bodies,
            max_longitude_delta_body,
            max_longitude_delta_source_kind,
            max_longitude_delta_source_file,
            max_longitude_delta_deg,
            max_longitude_delta_limit_deg,
            max_latitude_delta_body,
            max_latitude_delta_source_kind,
            max_latitude_delta_source_file,
            max_latitude_delta_deg,
            max_latitude_delta_limit_deg,
            max_distance_delta_body,
            max_distance_delta_source_kind,
            max_distance_delta_source_file,
            max_distance_delta_au,
            max_distance_delta_limit_au,
            mean_longitude_delta_deg: longitude_values_for_median.iter().sum::<f64>()
                / sample_count as f64,
            median_longitude_delta_deg: median_f64(&mut longitude_values_for_median),
            percentile_longitude_delta_deg: percentile_f64(
                &mut longitude_values_for_percentile,
                0.95,
            ),
            rms_longitude_delta_deg: rms_f64(&longitude_values_for_percentile),
            mean_latitude_delta_deg: latitude_values_for_median.iter().sum::<f64>()
                / sample_count as f64,
            median_latitude_delta_deg: median_f64(&mut latitude_values_for_median),
            percentile_latitude_delta_deg: percentile_f64(
                &mut latitude_values_for_percentile,
                0.95,
            ),
            rms_latitude_delta_deg: rms_f64(&latitude_values_for_percentile),
            mean_distance_delta_au: distance_values_for_median.iter().sum::<f64>()
                / sample_count as f64,
            median_distance_delta_au: median_f64(&mut distance_values_for_median),
            percentile_distance_delta_au: percentile_f64(&mut distance_values_for_percentile, 0.95),
            rms_distance_delta_au: rms_f64(&distance_values_for_percentile),
        });
    }

    Some(summaries)
}

/// Formats a single canonical VSOP87 body-class evidence envelope.
fn format_source_body_class_evidence_entry(
    summary: &Vsop87SourceBodyClassEvidenceSummary,
) -> String {
    let outside_note = if summary.outside_interim_limit_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.outside_interim_limit_bodies)
    };

    format!(
        "{}: samples={}, bodies: {}, within interim limits {}, outside interim limits {}; out-of-limit bodies: {}; mean Δlon={:.12}°, median Δlon={:.12}°, p95 Δlon={:.12}°, rms Δlon={:.12}°, mean Δlat={:.12}°, median Δlat={:.12}°, p95 Δlat={:.12}°, rms Δlat={:.12}°, mean Δdist={:.12} AU, median Δdist={:.12} AU, p95 Δdist={:.12} AU, rms Δdist={:.12} AU, max Δlon={:.12}° (limit {:.12}°, margin {:+.12}°; {}; {}; {}), max Δlat={:.12}° (limit {:.12}°, margin {:+.12}°; {}; {}; {}), max Δdist={:.12} AU (limit {:.12} AU, margin {:+.12} AU; {}; {}; {})",
        summary.class,
        summary.sample_count,
        format_celestial_bodies(&summary.sample_bodies),
        summary.within_interim_limits_count,
        summary.outside_interim_limit_count,
        outside_note,
        summary.mean_longitude_delta_deg,
        summary.median_longitude_delta_deg,
        summary.percentile_longitude_delta_deg,
        summary.rms_longitude_delta_deg,
        summary.mean_latitude_delta_deg,
        summary.median_latitude_delta_deg,
        summary.percentile_latitude_delta_deg,
        summary.rms_latitude_delta_deg,
        summary.mean_distance_delta_au,
        summary.median_distance_delta_au,
        summary.percentile_distance_delta_au,
        summary.rms_distance_delta_au,
        summary.max_longitude_delta_deg,
        summary.max_longitude_delta_limit_deg,
        summary.max_longitude_delta_limit_deg - summary.max_longitude_delta_deg,
        summary.max_longitude_delta_body,
        summary.max_longitude_delta_source_kind,
        summary.max_longitude_delta_source_file,
        summary.max_latitude_delta_deg,
        summary.max_latitude_delta_limit_deg,
        summary.max_latitude_delta_limit_deg - summary.max_latitude_delta_deg,
        summary.max_latitude_delta_body,
        summary.max_latitude_delta_source_kind,
        summary.max_latitude_delta_source_file,
        summary.max_distance_delta_au,
        summary.max_distance_delta_limit_au,
        summary.max_distance_delta_limit_au - summary.max_distance_delta_au,
        summary.max_distance_delta_body,
        summary.max_distance_delta_source_kind,
        summary.max_distance_delta_source_file,
    )
}
