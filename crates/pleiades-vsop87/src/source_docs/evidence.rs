use pleiades_types::{
    CelestialBody, CoordinateFrame, EclipticCoordinates, Instant, Latitude, Longitude, TimeScale,
};
use std::fmt;

use pleiades_backend::{EphemerisBackend, EphemerisRequest};

use crate::backend::Vsop87Backend;
use crate::profiles::{
    body_catalog_entries, body_catalog_entry_for_body, body_source_profiles, source_file_for_body,
    source_kind_for_body, Vsop87BodySourceKind,
};
use crate::transforms::signed_longitude_delta_degrees;

use super::documentation::{body_labels_are_unique, format_celestial_bodies};
use super::spec::source_specifications;

const J2000: f64 = 2_451_545.0;

///
/// These values are the same full-file public IMCCE VSOP87B reference points
/// exercised by the backend regression tests. The validation tooling uses them
/// to render measured deltas against the checked-in source-backed coefficient
/// paths while the generated-table pipeline continues to expand.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalEpochSample {
    /// Body measured at the canonical epoch.
    pub body: CelestialBody,
    /// Reference geocentric ecliptic longitude in degrees.
    pub expected_longitude_deg: f64,
    /// Reference geocentric ecliptic latitude in degrees.
    pub expected_latitude_deg: f64,
    /// Reference geocentric distance in astronomical units.
    pub expected_distance_au: f64,
    /// Maximum acceptable geocentric longitude delta in degrees.
    pub max_longitude_delta_deg: f64,
    /// Maximum acceptable geocentric latitude delta in degrees.
    pub max_latitude_delta_deg: f64,
    /// Maximum acceptable geocentric distance delta in astronomical units.
    pub max_distance_delta_au: f64,
}

/// Returns the canonical J2000 epoch samples for the current source-backed bodies.
pub fn canonical_epoch_samples() -> Vec<Vsop87CanonicalEpochSample> {
    body_catalog_entries()
        .iter()
        .filter_map(|entry| entry.canonical_sample.clone())
        .collect()
}

/// Builds a batch request corpus for the provided bodies at a shared instant
/// and coordinate frame.
///
/// The requests preserve the iterator order and default to tropical mean
/// geometric geocentric output, which makes the helper suitable for canonical
/// batch regressions and reproducibility tooling.
pub fn requests_for_bodies_at<I>(
    bodies: I,
    instant: Instant,
    frame: CoordinateFrame,
) -> Vec<EphemerisRequest>
where
    I: IntoIterator<Item = CelestialBody>,
{
    bodies
        .into_iter()
        .map(|body| {
            let mut request = EphemerisRequest::new(body, instant);
            request.frame = frame;
            request
        })
        .collect()
}

/// Returns the canonical J2000 request corpus used by the VSOP87 batch-path evidence.
///
/// The requests preserve the source-backed body order, use the shared J2000 TT
/// instant, and keep the geocentric ecliptic frame.
#[doc(alias = "canonical_epoch_requests")]
pub fn canonical_j2000_batch_parity_requests() -> Vec<EphemerisRequest> {
    requests_for_bodies_at(
        canonical_epoch_samples()
            .into_iter()
            .map(|sample| sample.body),
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
        CoordinateFrame::Ecliptic,
    )
}

/// Returns the canonical J2000 request corpus used by the VSOP87 batch-path evidence.
///
/// This is a compatibility alias for [`canonical_j2000_batch_parity_requests`].
pub fn canonical_epoch_requests() -> Vec<EphemerisRequest> {
    canonical_j2000_batch_parity_requests()
}

/// Public release-facing error envelope for one body at the canonical J2000
/// comparison epoch.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalBodyEvidence {
    /// Body measured at the canonical epoch.
    pub body: CelestialBody,
    /// Calculation family used for the body.
    pub source_kind: Vsop87BodySourceKind,
    /// Public coefficient file backing the body.
    pub source_file: &'static str,
    /// Human-readable provenance detail for the body.
    pub provenance: &'static str,
    /// Absolute geocentric longitude delta in degrees.
    pub longitude_delta_deg: f64,
    /// Absolute geocentric latitude delta in degrees.
    pub latitude_delta_deg: f64,
    /// Absolute geocentric distance delta in astronomical units.
    pub distance_delta_au: f64,
    /// Interim longitude delta limit used for this body.
    pub longitude_limit_deg: f64,
    /// Interim latitude delta limit used for this body.
    pub latitude_limit_deg: f64,
    /// Interim distance delta limit used for this body.
    pub distance_limit_au: f64,
    /// Whether the body is within the current interim limits.
    pub within_interim_limits: bool,
}

impl Vsop87CanonicalBodyEvidence {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: kind={}, source={}, provenance={}, Δlon={:.12}°/limit {:.12}°/margin {:+.12}°, Δlat={:.12}°/limit {:.12}°/margin {:+.12}°, Δdist={:.12} AU/limit {:.12} AU/margin {:+.12} AU, status {}",
            self.body,
            self.source_kind,
            self.source_file,
            self.provenance,
            self.longitude_delta_deg,
            self.longitude_limit_deg,
            self.longitude_limit_deg - self.longitude_delta_deg,
            self.latitude_delta_deg,
            self.latitude_limit_deg,
            self.latitude_limit_deg - self.latitude_delta_deg,
            self.distance_delta_au,
            self.distance_limit_au,
            self.distance_limit_au - self.distance_delta_au,
            if self.within_interim_limits {
                "within interim limits"
            } else {
                "outside interim limits"
            },
        )
    }

    /// Returns `Ok(())` when the row still matches the current canonical evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBodyEvidenceValidationError> {
        let entry = body_catalog_entry_for_body(&self.body).ok_or(
            Vsop87CanonicalBodyEvidenceValidationError::UnknownBody {
                body: self.body.clone(),
            },
        )?;

        if self.source_kind != entry.source_profile.kind {
            return Err(
                Vsop87CanonicalBodyEvidenceValidationError::SourceKindMismatch {
                    body: self.body.clone(),
                    expected: entry.source_profile.kind,
                    found: self.source_kind,
                },
            );
        }

        let specification = entry.source_specification.as_ref().ok_or(
            Vsop87CanonicalBodyEvidenceValidationError::MissingSourceSpecification {
                body: self.body.clone(),
            },
        )?;

        if self.source_file != specification.source_file {
            return Err(
                Vsop87CanonicalBodyEvidenceValidationError::SourceFileMismatch {
                    body: self.body.clone(),
                    expected: specification.source_file,
                    found: self.source_file,
                },
            );
        }
        if self.provenance != entry.source_profile.provenance {
            return Err(
                Vsop87CanonicalBodyEvidenceValidationError::ProvenanceMismatch {
                    body: self.body.clone(),
                    expected: entry.source_profile.provenance,
                    found: self.provenance,
                },
            );
        }

        for (field, value) in [
            ("longitude_delta_deg", self.longitude_delta_deg),
            ("latitude_delta_deg", self.latitude_delta_deg),
            ("distance_delta_au", self.distance_delta_au),
            ("longitude_limit_deg", self.longitude_limit_deg),
            ("latitude_limit_deg", self.latitude_limit_deg),
            ("distance_limit_au", self.distance_limit_au),
        ] {
            if !value.is_finite() {
                return Err(
                    Vsop87CanonicalBodyEvidenceValidationError::NonFiniteMetric {
                        body: self.body.clone(),
                        field,
                    },
                );
            }
            if value < 0.0 {
                return Err(Vsop87CanonicalBodyEvidenceValidationError::NegativeMetric {
                    body: self.body.clone(),
                    field,
                });
            }
        }

        let within_limits = self.longitude_delta_deg <= self.longitude_limit_deg
            && self.latitude_delta_deg <= self.latitude_limit_deg
            && self.distance_delta_au <= self.distance_limit_au;
        if within_limits != self.within_interim_limits {
            return Err(
                Vsop87CanonicalBodyEvidenceValidationError::InterimLimitStatusMismatch {
                    body: self.body.clone(),
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalBodyEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a canonical VSOP87 body-evidence row that drifted
/// from the current canonical evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87CanonicalBodyEvidenceValidationError {
    /// The catalog no longer contains the body.
    UnknownBody {
        /// Body absent from the current catalog.
        body: CelestialBody,
    },
    /// The declared source family drifted out of sync with the current catalog.
    SourceKindMismatch {
        /// Body whose source kind drifted from the catalog.
        body: CelestialBody,
        /// Source kind the catalog expects for the body.
        expected: Vsop87BodySourceKind,
        /// Source kind found on the drifted evidence row.
        found: Vsop87BodySourceKind,
    },
    /// The catalog no longer exposes a source specification for the body.
    MissingSourceSpecification {
        /// Body that lacks a source specification in the catalog.
        body: CelestialBody,
    },
    /// The public source file drifted out of sync with the current catalog.
    SourceFileMismatch {
        /// Body whose source file drifted from the catalog.
        body: CelestialBody,
        /// Source file the catalog expects for the body.
        expected: &'static str,
        /// Source file found on the drifted evidence row.
        found: &'static str,
    },
    /// The provenance text drifted out of sync with the current catalog.
    ProvenanceMismatch {
        /// Body whose provenance text drifted from the catalog.
        body: CelestialBody,
        /// Provenance text the catalog expects for the body.
        expected: &'static str,
        /// Provenance text found on the drifted evidence row.
        found: &'static str,
    },
    /// A numeric field is not finite.
    NonFiniteMetric {
        /// Body whose metric is non-finite.
        body: CelestialBody,
        /// Name of the non-finite metric field.
        field: &'static str,
    },
    /// A numeric field is negative.
    NegativeMetric {
        /// Body whose metric is negative.
        body: CelestialBody,
        /// Name of the negative metric field.
        field: &'static str,
    },
    /// The derived interim-limit status drifted away from the current metrics.
    InterimLimitStatusMismatch {
        /// Body whose interim-limit status drifted from its metrics.
        body: CelestialBody,
    },
}

impl fmt::Display for Vsop87CanonicalBodyEvidenceValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownBody { body } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} is no longer present in the catalog"
            ),
            Self::SourceKindMismatch { body, expected, found } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} expects kind {expected} but found {found}"
            ),
            Self::MissingSourceSpecification { body } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} no longer has a source specification in the catalog"
            ),
            Self::SourceFileMismatch { body, expected, found } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} expects source file `{expected}` but found `{found}`"
            ),
            Self::ProvenanceMismatch { body, expected, found } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} expects provenance `{expected}` but found `{found}`"
            ),
            Self::NonFiniteMetric { body, field } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} has a non-finite `{field}` value"
            ),
            Self::NegativeMetric { body, field } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} has a negative `{field}` value"
            ),
            Self::InterimLimitStatusMismatch { body } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} has a mismatched interim-limit status"
            ),
        }
    }
}

impl std::error::Error for Vsop87CanonicalBodyEvidenceValidationError {}

/// Public summary of the canonical J2000 error envelope.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalEvidenceSummary {
    /// Number of canonical samples measured.
    pub sample_count: usize,
    /// Canonical bodies measured in source-backed order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Body with the maximum absolute geocentric longitude delta.
    pub max_longitude_delta_body: CelestialBody,
    /// Calculation family behind the maximum longitude delta body.
    pub max_longitude_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum longitude delta body.
    pub max_longitude_delta_source_file: &'static str,
    /// Maximum absolute geocentric longitude delta in degrees.
    pub max_longitude_delta_deg: f64,
    /// Interim longitude delta limit for the body that drives the maximum.
    pub max_longitude_delta_limit_deg: f64,
    /// Body with the maximum absolute geocentric latitude delta.
    pub max_latitude_delta_body: CelestialBody,
    /// Calculation family behind the maximum latitude delta body.
    pub max_latitude_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum latitude delta body.
    pub max_latitude_delta_source_file: &'static str,
    /// Maximum absolute geocentric latitude delta in degrees.
    pub max_latitude_delta_deg: f64,
    /// Interim latitude delta limit for the body that drives the maximum.
    pub max_latitude_delta_limit_deg: f64,
    /// Body with the maximum absolute geocentric distance delta.
    pub max_distance_delta_body: CelestialBody,
    /// Calculation family behind the maximum distance delta body.
    pub max_distance_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum distance delta body.
    pub max_distance_delta_source_file: &'static str,
    /// Maximum absolute geocentric distance delta in astronomical units.
    pub max_distance_delta_au: f64,
    /// Interim distance delta limit for the body that drives the maximum.
    pub max_distance_delta_limit_au: f64,
    /// Mean absolute geocentric longitude delta in degrees.
    pub mean_longitude_delta_deg: f64,
    /// Median absolute geocentric longitude delta in degrees.
    pub median_longitude_delta_deg: f64,
    /// 95th percentile absolute geocentric longitude delta in degrees.
    pub percentile_longitude_delta_deg: f64,
    /// Root-mean-square geocentric longitude delta in degrees.
    pub rms_longitude_delta_deg: f64,
    /// Mean absolute geocentric latitude delta in degrees.
    pub mean_latitude_delta_deg: f64,
    /// Median absolute geocentric latitude delta in degrees.
    pub median_latitude_delta_deg: f64,
    /// 95th percentile absolute geocentric latitude delta in degrees.
    pub percentile_latitude_delta_deg: f64,
    /// Root-mean-square geocentric latitude delta in degrees.
    pub rms_latitude_delta_deg: f64,
    /// Mean absolute geocentric distance delta in astronomical units.
    pub mean_distance_delta_au: f64,
    /// Median absolute geocentric distance delta in astronomical units.
    pub median_distance_delta_au: f64,
    /// 95th percentile absolute geocentric distance delta in astronomical units.
    pub percentile_distance_delta_au: f64,
    /// Root-mean-square geocentric distance delta in astronomical units.
    pub rms_distance_delta_au: f64,
    /// Number of samples that exceeded at least one interim limit.
    pub out_of_limit_count: usize,
    /// Whether every measured body remained within the interim limits.
    pub within_interim_limits: bool,
}

impl Vsop87CanonicalEvidenceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    ///
    /// This mirrors the release-facing free-function renderer relocated to
    /// `pleiades-validate`'s posture module (report-surface relocation
    /// program, Slice B); the rendering logic stays here too because this
    /// inherent method (and `Display`) must keep working without a
    /// dependency on `pleiades-validate`.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 canonical J2000 source-backed evidence: {} samples, bodies: {}, status {}, mean Δlon={:.12}°, median Δlon={:.12}°, p95 Δlon={:.12}°, rms Δlon={:.12}°, mean Δlat={:.12}°, median Δlat={:.12}°, p95 Δlat={:.12}°, rms Δlat={:.12}°, mean Δdist={:.12} AU, median Δdist={:.12} AU, p95 Δdist={:.12} AU, rms Δdist={:.12} AU, out-of-limit samples {}, max Δlon={:.12}° (limit {:.12}°, margin {:+.12}°; {}; {}; {}), max Δlat={:.12}° (limit {:.12}°, margin {:+.12}°; {}; {}; {}), max Δdist={:.12} AU (limit {:.12} AU, margin {:+.12} AU; {}; {}; {})",
            self.sample_count,
            format_celestial_bodies(&self.sample_bodies),
            if self.within_interim_limits {
                "within interim limits"
            } else {
                "outside interim limits"
            },
            self.mean_longitude_delta_deg,
            self.median_longitude_delta_deg,
            self.percentile_longitude_delta_deg,
            self.rms_longitude_delta_deg,
            self.mean_latitude_delta_deg,
            self.median_latitude_delta_deg,
            self.percentile_latitude_delta_deg,
            self.rms_latitude_delta_deg,
            self.mean_distance_delta_au,
            self.median_distance_delta_au,
            self.percentile_distance_delta_au,
            self.rms_distance_delta_au,
            self.out_of_limit_count,
            self.max_longitude_delta_deg,
            self.max_longitude_delta_limit_deg,
            self.max_longitude_delta_limit_deg - self.max_longitude_delta_deg,
            self.max_longitude_delta_body,
            self.max_longitude_delta_source_kind,
            self.max_longitude_delta_source_file,
            self.max_latitude_delta_deg,
            self.max_latitude_delta_limit_deg,
            self.max_latitude_delta_limit_deg - self.max_latitude_delta_deg,
            self.max_latitude_delta_body,
            self.max_latitude_delta_source_kind,
            self.max_latitude_delta_source_file,
            self.max_distance_delta_au,
            self.max_distance_delta_limit_au,
            self.max_distance_delta_limit_au - self.max_distance_delta_au,
            self.max_distance_delta_body,
            self.max_distance_delta_source_kind,
            self.max_distance_delta_source_file,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
        validate_canonical_evidence_summary_bodies(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            self.sample_count,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_longitude_delta_body",
            &self.max_longitude_delta_body,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_latitude_delta_body",
            &self.max_latitude_delta_body,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_body",
            &self.max_distance_delta_body,
            &self.sample_bodies,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_longitude_delta_source_file",
            self.max_longitude_delta_source_file,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_latitude_delta_source_file",
            self.max_latitude_delta_source_file,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_source_file",
            self.max_distance_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_longitude_delta_source_kind",
            "max_longitude_delta_source_file",
            &self.max_longitude_delta_body,
            self.max_longitude_delta_source_kind,
            self.max_longitude_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_latitude_delta_source_kind",
            "max_latitude_delta_source_file",
            &self.max_latitude_delta_body,
            self.max_latitude_delta_source_kind,
            self.max_latitude_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
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
            validate_finite_non_negative_measure(CANONICAL_EVIDENCE_SUMMARY_LABEL, field, value)?;
        }

        validate_canonical_evidence_summary_metric_order(&Vsop87MetricOrderingValidation {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            mean_field: "mean_longitude_delta_deg",
            median_field: "median_longitude_delta_deg",
            percentile_field: "percentile_longitude_delta_deg",
            rms_field: "rms_longitude_delta_deg",
            mean: self.mean_longitude_delta_deg,
            median: self.median_longitude_delta_deg,
            percentile: self.percentile_longitude_delta_deg,
            rms: self.rms_longitude_delta_deg,
            max: self.max_longitude_delta_deg,
        })?;
        validate_canonical_evidence_summary_metric_order(&Vsop87MetricOrderingValidation {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            mean_field: "mean_latitude_delta_deg",
            median_field: "median_latitude_delta_deg",
            percentile_field: "percentile_latitude_delta_deg",
            rms_field: "rms_latitude_delta_deg",
            mean: self.mean_latitude_delta_deg,
            median: self.median_latitude_delta_deg,
            percentile: self.percentile_latitude_delta_deg,
            rms: self.rms_latitude_delta_deg,
            max: self.max_latitude_delta_deg,
        })?;
        validate_canonical_evidence_summary_metric_order(&Vsop87MetricOrderingValidation {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            mean_field: "mean_distance_delta_au",
            median_field: "median_distance_delta_au",
            percentile_field: "percentile_distance_delta_au",
            rms_field: "rms_distance_delta_au",
            mean: self.mean_distance_delta_au,
            median: self.median_distance_delta_au,
            percentile: self.percentile_distance_delta_au,
            rms: self.rms_distance_delta_au,
            max: self.max_distance_delta_au,
        })?;

        let Some(body_evidence) = canonical_epoch_body_evidence() else {
            return Err(
                Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                    summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
                    field: "sample_count",
                },
            );
        };
        let expected_out_of_limit_count = body_evidence
            .iter()
            .filter(|evidence| !evidence.within_interim_limits)
            .count();

        if self.out_of_limit_count != expected_out_of_limit_count {
            return Err(
                Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                    summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
                    field: "out_of_limit_count",
                },
            );
        }
        if self.within_interim_limits != (expected_out_of_limit_count == 0) {
            return Err(
                Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                    summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
                    field: "within_interim_limits",
                },
            );
        }

        validate_canonical_epoch_evidence_summary_against_current_evidence(self)?;

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Backend-owned summary for the canonical J2000 equatorial companion evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalEquatorialBodyEvidence {
    /// Body measured at the canonical epoch.
    pub body: CelestialBody,
    /// Calculation family used for the body.
    pub source_kind: Vsop87BodySourceKind,
    /// Public source file backing the body.
    pub source_file: &'static str,
    /// Human-readable provenance detail for the body.
    pub provenance: &'static str,
    /// Absolute right ascension delta in degrees.
    pub right_ascension_delta_deg: f64,
    /// Absolute declination delta in degrees.
    pub declination_delta_deg: f64,
    /// Absolute distance delta in astronomical units.
    pub distance_delta_au: f64,
}

/// Public summary of the canonical J2000 equatorial companion evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalEquatorialEvidenceSummary {
    /// Number of canonical samples measured.
    pub sample_count: usize,
    /// Canonical bodies measured in source-backed order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Body with the maximum absolute right ascension delta.
    pub max_right_ascension_delta_body: CelestialBody,
    /// Calculation family behind the maximum right ascension delta body.
    pub max_right_ascension_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum right ascension delta body.
    pub max_right_ascension_delta_source_file: &'static str,
    /// Maximum absolute right ascension delta in degrees.
    pub max_right_ascension_delta_deg: f64,
    /// Body with the maximum absolute declination delta.
    pub max_declination_delta_body: CelestialBody,
    /// Calculation family behind the maximum declination delta body.
    pub max_declination_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum declination delta body.
    pub max_declination_delta_source_file: &'static str,
    /// Maximum absolute declination delta in degrees.
    pub max_declination_delta_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: CelestialBody,
    /// Calculation family behind the maximum distance delta body.
    pub max_distance_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum distance delta body.
    pub max_distance_delta_source_file: &'static str,
    /// Maximum absolute distance delta in astronomical units.
    pub max_distance_delta_au: f64,
    /// Mean absolute right ascension delta in degrees.
    pub mean_right_ascension_delta_deg: f64,
    /// Median absolute right ascension delta in degrees.
    pub median_right_ascension_delta_deg: f64,
    /// 95th percentile absolute right ascension delta in degrees.
    pub percentile_right_ascension_delta_deg: f64,
    /// Root-mean-square right ascension delta in degrees.
    pub rms_right_ascension_delta_deg: f64,
    /// Mean absolute declination delta in degrees.
    pub mean_declination_delta_deg: f64,
    /// Median absolute declination delta in degrees.
    pub median_declination_delta_deg: f64,
    /// 95th percentile absolute declination delta in degrees.
    pub percentile_declination_delta_deg: f64,
    /// Root-mean-square declination delta in degrees.
    pub rms_declination_delta_deg: f64,
    /// Mean absolute distance delta in astronomical units.
    pub mean_distance_delta_au: f64,
    /// Median absolute distance delta in astronomical units.
    pub median_distance_delta_au: f64,
    /// 95th percentile absolute distance delta in astronomical units.
    pub percentile_distance_delta_au: f64,
    /// Root-mean-square distance delta in astronomical units.
    pub rms_distance_delta_au: f64,
}

impl Vsop87CanonicalEquatorialEvidenceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    ///
    /// This mirrors the release-facing free-function renderer relocated to
    /// `pleiades-validate`'s posture module (report-surface relocation
    /// program, Slice B); the rendering logic stays here too because this
    /// inherent method (and `Display`) must keep working without a
    /// dependency on `pleiades-validate`.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 canonical J2000 equatorial companion evidence: {} samples, bodies: {}, mean Δra={:.12}°, median Δra={:.12}°, p95 Δra={:.12}°, rms Δra={:.12}°, mean Δdec={:.12}°, median Δdec={:.12}°, p95 Δdec={:.12}°, rms Δdec={:.12}°, mean Δdist={:.12} AU, median Δdist={:.12} AU, p95 Δdist={:.12} AU, rms Δdist={:.12} AU, max Δra={:.12}° ({}; {}; {}), max Δdec={:.12}° ({}; {}; {}), max Δdist={:.12} AU ({}; {}; {})",
            self.sample_count,
            format_celestial_bodies(&self.sample_bodies),
            self.mean_right_ascension_delta_deg,
            self.median_right_ascension_delta_deg,
            self.percentile_right_ascension_delta_deg,
            self.rms_right_ascension_delta_deg,
            self.mean_declination_delta_deg,
            self.median_declination_delta_deg,
            self.percentile_declination_delta_deg,
            self.rms_declination_delta_deg,
            self.mean_distance_delta_au,
            self.median_distance_delta_au,
            self.percentile_distance_delta_au,
            self.rms_distance_delta_au,
            self.max_right_ascension_delta_deg,
            self.max_right_ascension_delta_body,
            self.max_right_ascension_delta_source_kind,
            self.max_right_ascension_delta_source_file,
            self.max_declination_delta_deg,
            self.max_declination_delta_body,
            self.max_declination_delta_source_kind,
            self.max_declination_delta_source_file,
            self.max_distance_delta_au,
            self.max_distance_delta_body,
            self.max_distance_delta_source_kind,
            self.max_distance_delta_source_file,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
        validate_canonical_evidence_summary_bodies(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            self.sample_count,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_right_ascension_delta_body",
            &self.max_right_ascension_delta_body,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_declination_delta_body",
            &self.max_declination_delta_body,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_body",
            &self.max_distance_delta_body,
            &self.sample_bodies,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_right_ascension_delta_source_file",
            self.max_right_ascension_delta_source_file,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_declination_delta_source_file",
            self.max_declination_delta_source_file,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_source_file",
            self.max_distance_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_right_ascension_delta_source_kind",
            "max_right_ascension_delta_source_file",
            &self.max_right_ascension_delta_body,
            self.max_right_ascension_delta_source_kind,
            self.max_right_ascension_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_declination_delta_source_kind",
            "max_declination_delta_source_file",
            &self.max_declination_delta_body,
            self.max_declination_delta_source_kind,
            self.max_declination_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_source_kind",
            "max_distance_delta_source_file",
            &self.max_distance_delta_body,
            self.max_distance_delta_source_kind,
            self.max_distance_delta_source_file,
        )?;
        for (field, value) in [
            (
                "max_right_ascension_delta_deg",
                self.max_right_ascension_delta_deg,
            ),
            ("max_declination_delta_deg", self.max_declination_delta_deg),
            ("max_distance_delta_au", self.max_distance_delta_au),
            (
                "mean_right_ascension_delta_deg",
                self.mean_right_ascension_delta_deg,
            ),
            (
                "median_right_ascension_delta_deg",
                self.median_right_ascension_delta_deg,
            ),
            (
                "percentile_right_ascension_delta_deg",
                self.percentile_right_ascension_delta_deg,
            ),
            (
                "rms_right_ascension_delta_deg",
                self.rms_right_ascension_delta_deg,
            ),
            (
                "mean_declination_delta_deg",
                self.mean_declination_delta_deg,
            ),
            (
                "median_declination_delta_deg",
                self.median_declination_delta_deg,
            ),
            (
                "percentile_declination_delta_deg",
                self.percentile_declination_delta_deg,
            ),
            ("rms_declination_delta_deg", self.rms_declination_delta_deg),
            ("mean_distance_delta_au", self.mean_distance_delta_au),
            ("median_distance_delta_au", self.median_distance_delta_au),
            (
                "percentile_distance_delta_au",
                self.percentile_distance_delta_au,
            ),
            ("rms_distance_delta_au", self.rms_distance_delta_au),
        ] {
            validate_finite_non_negative_measure(
                CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
                field,
                value,
            )?;
        }

        validate_canonical_evidence_summary_metric_order(&Vsop87MetricOrderingValidation {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            mean_field: "mean_right_ascension_delta_deg",
            median_field: "median_right_ascension_delta_deg",
            percentile_field: "percentile_right_ascension_delta_deg",
            rms_field: "rms_right_ascension_delta_deg",
            mean: self.mean_right_ascension_delta_deg,
            median: self.median_right_ascension_delta_deg,
            percentile: self.percentile_right_ascension_delta_deg,
            rms: self.rms_right_ascension_delta_deg,
            max: self.max_right_ascension_delta_deg,
        })?;
        validate_canonical_evidence_summary_metric_order(&Vsop87MetricOrderingValidation {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            mean_field: "mean_declination_delta_deg",
            median_field: "median_declination_delta_deg",
            percentile_field: "percentile_declination_delta_deg",
            rms_field: "rms_declination_delta_deg",
            mean: self.mean_declination_delta_deg,
            median: self.median_declination_delta_deg,
            percentile: self.percentile_declination_delta_deg,
            rms: self.rms_declination_delta_deg,
            max: self.max_declination_delta_deg,
        })?;
        validate_canonical_evidence_summary_metric_order(&Vsop87MetricOrderingValidation {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            mean_field: "mean_distance_delta_au",
            median_field: "median_distance_delta_au",
            percentile_field: "percentile_distance_delta_au",
            rms_field: "rms_distance_delta_au",
            mean: self.mean_distance_delta_au,
            median: self.median_distance_delta_au,
            percentile: self.percentile_distance_delta_au,
            rms: self.rms_distance_delta_au,
            max: self.max_distance_delta_au,
        })?;

        validate_canonical_epoch_equatorial_evidence_summary_against_current_evidence(self)?;

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalEquatorialEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) const CANONICAL_EVIDENCE_SUMMARY_LABEL: &str =
    "VSOP87 canonical J2000 source-backed evidence";
pub(crate) const CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL: &str =
    "VSOP87 canonical J2000 equatorial companion evidence";
const CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL: &str =
    "VSOP87 canonical J2000 equatorial body-class evidence";

/// Validation error for a canonical VSOP87 evidence summary that drifted from
/// the current derived evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87CanonicalEvidenceSummaryValidationError {
    /// A rendered summary field no longer matches the current derived evidence.
    FieldOutOfSync {
        /// Label of the summary that carries the drifted field.
        summary: &'static str,
        /// Name of the field that drifted from the derived evidence.
        field: &'static str,
    },
    /// The sample body list now contains a duplicate entry.
    DuplicateBody {
        /// Label of the summary whose sample list has the duplicate.
        summary: &'static str,
        /// Body that appears more than once in the sample list.
        body: CelestialBody,
    },
    /// A reported peak body is absent from the sample body list.
    PeakBodyNotInSamples {
        /// Label of the summary whose peak body is missing.
        summary: &'static str,
        /// Name of the peak-metric field naming the absent body.
        field: &'static str,
        /// Peak body that is absent from the sample list.
        body: CelestialBody,
    },
    /// A reported source file is blank or whitespace only.
    BlankSourceFile {
        /// Label of the summary that carries the blank source file.
        summary: &'static str,
        /// Name of the field whose source file is blank.
        field: &'static str,
    },
}

impl fmt::Display for Vsop87CanonicalEvidenceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { summary, field } => write!(
                f,
                "the {summary} summary field `{field}` is out of sync with the current canonical evidence"
            ),
            Self::DuplicateBody { summary, body } => write!(
                f,
                "the {summary} summary lists body `{body}` more than once"
            ),
            Self::PeakBodyNotInSamples {
                summary,
                field,
                body,
            } => write!(
                f,
                "the {summary} summary field `{field}` points at body `{body}` which is absent from the sample body list"
            ),
            Self::BlankSourceFile { summary, field } => write!(
                f,
                "the {summary} summary field `{field}` is blank"
            ),
        }
    }
}

impl std::error::Error for Vsop87CanonicalEvidenceSummaryValidationError {}

fn validate_canonical_evidence_summary_bodies(
    summary: &'static str,
    sample_count: usize,
    sample_bodies: &[CelestialBody],
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    if sample_count == 0 {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary,
                field: "sample_count",
            },
        );
    }
    if sample_count != sample_bodies.len() {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary,
                field: "sample_count",
            },
        );
    }

    for (index, body) in sample_bodies.iter().enumerate() {
        if sample_bodies[..index].contains(body) {
            return Err(
                Vsop87CanonicalEvidenceSummaryValidationError::DuplicateBody {
                    summary,
                    body: body.clone(),
                },
            );
        }
    }

    Ok(())
}

fn validate_canonical_evidence_summary_peak_body(
    summary: &'static str,
    field: &'static str,
    body: &CelestialBody,
    sample_bodies: &[CelestialBody],
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    if sample_bodies.contains(body) {
        Ok(())
    } else {
        Err(
            Vsop87CanonicalEvidenceSummaryValidationError::PeakBodyNotInSamples {
                summary,
                field,
                body: body.clone(),
            },
        )
    }
}

fn validate_non_empty_source_file(
    summary: &'static str,
    field: &'static str,
    source_file: &'static str,
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    if source_file.trim().is_empty() {
        Err(Vsop87CanonicalEvidenceSummaryValidationError::BlankSourceFile { summary, field })
    } else {
        Ok(())
    }
}

fn validate_finite_non_negative_measure(
    summary: &'static str,
    field: &'static str,
    value: f64,
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync { summary, field })
    }
}

struct Vsop87MetricOrderingValidation {
    summary: &'static str,
    mean_field: &'static str,
    median_field: &'static str,
    percentile_field: &'static str,
    rms_field: &'static str,
    mean: f64,
    median: f64,
    percentile: f64,
    rms: f64,
    max: f64,
}

fn validate_canonical_evidence_summary_metric_order(
    metric: &Vsop87MetricOrderingValidation,
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    if metric.mean > metric.max {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary: metric.summary,
                field: metric.mean_field,
            },
        );
    }
    if metric.median > metric.percentile {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary: metric.summary,
                field: metric.median_field,
            },
        );
    }
    if metric.percentile > metric.max {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary: metric.summary,
                field: metric.percentile_field,
            },
        );
    }
    if metric.rms > metric.max {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary: metric.summary,
                field: metric.rms_field,
            },
        );
    }

    Ok(())
}

fn validate_canonical_evidence_summary_peak_source_metadata(
    summary: &'static str,
    field_kind: &'static str,
    field_file: &'static str,
    body: &CelestialBody,
    source_kind: Vsop87BodySourceKind,
    source_file: &'static str,
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    let expected_source_kind = source_kind_for_body(body.clone()).ok_or(
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary,
            field: field_kind,
        },
    )?;
    let expected_source_file = source_file_for_body(body).ok_or(
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary,
            field: field_file,
        },
    )?;

    if expected_source_kind != source_kind {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary,
                field: field_kind,
            },
        );
    }
    if expected_source_file != source_file {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary,
                field: field_file,
            },
        );
    }

    Ok(())
}

/// Aggregate evidence summary across the source-backed canonical samples.
///
/// Rolls up per-body residuals (longitude, latitude, distance) against the
/// interim limits for the mean, J2000 ecliptic source-backed paths, excluding
/// the mean-element Pluto fallback.
pub struct Vsop87SourceBodyEvidenceSummary {
    /// Number of canonical samples measured.
    pub sample_count: usize,
    /// Canonical bodies measured in source-backed order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Number of samples within the interim limits.
    pub within_interim_limits_count: usize,
    /// Number of vendored full-file source-backed samples.
    pub vendored_full_file_count: usize,
    /// Number of generated-binary source-backed samples.
    pub generated_binary_count: usize,
    /// Number of truncated-slice source-backed samples.
    pub truncated_count: usize,
    /// Number of bodies outside the current interim limits.
    pub outside_interim_limit_count: usize,
    /// Bodies outside the current interim limits.
    pub outside_interim_limit_bodies: Vec<CelestialBody>,
}

impl Vsop87SourceBodyEvidenceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    ///
    /// This mirrors the release-facing free-function renderer relocated to
    /// `pleiades-validate`'s posture module (report-surface relocation
    /// program, Slice B); the rendering logic stays here too because this
    /// inherent method (and `Display`) must keep working without a
    /// dependency on `pleiades-validate`.
    pub fn summary_line(&self) -> String {
        let outside_note = if self.outside_interim_limit_bodies.is_empty() {
            "none".to_string()
        } else {
            format_celestial_bodies(&self.outside_interim_limit_bodies)
        };

        let bodies = format_celestial_bodies(&self.sample_bodies);

        if self.generated_binary_count == 0 && self.truncated_count == 0 {
            format!(
                "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file), source-backed body order: {}, {} within interim limits, {} outside interim limits; outside interim limits: {}",
                self.sample_count,
                self.vendored_full_file_count,
                bodies,
                self.within_interim_limits_count,
                self.outside_interim_limit_count,
                outside_note,
            )
        } else if self.generated_binary_count > 0 && self.truncated_count == 0 {
            format!(
                "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file, {} generated binary), source-backed body order: {}, {} within interim limits, {} outside interim limits; outside interim limits: {}",
                self.sample_count,
                self.vendored_full_file_count,
                self.generated_binary_count,
                bodies,
                self.within_interim_limits_count,
                self.outside_interim_limit_count,
                outside_note,
            )
        } else if self.generated_binary_count == 0 {
            format!(
                "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file, {} truncated slice), source-backed body order: {}, {} within interim limits, {} outside interim limits; outside interim limits: {}",
                self.sample_count,
                self.vendored_full_file_count,
                self.truncated_count,
                bodies,
                self.within_interim_limits_count,
                self.outside_interim_limit_count,
                outside_note,
            )
        } else {
            format!(
                "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file, {} generated binary, {} truncated slice), source-backed body order: {}, {} within interim limits, {} outside interim limits; outside interim limits: {}",
                self.sample_count,
                self.vendored_full_file_count,
                self.generated_binary_count,
                self.truncated_count,
                bodies,
                self.within_interim_limits_count,
                self.outside_interim_limit_count,
                outside_note,
            )
        }
    }

    /// Returns the validated summary line used by release-facing reporting.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Vsop87SourceBodyEvidenceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the summary still matches the current derived counts.
    pub fn validate(&self) -> Result<(), Vsop87SourceBodyEvidenceSummaryValidationError> {
        let Some(expected) = source_body_evidence_summary() else {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        };

        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_count == 0 {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.within_interim_limits_count != expected.within_interim_limits_count {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "within_interim_limits_count",
                },
            );
        }
        if self.outside_interim_limit_count != expected.outside_interim_limit_count {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_count",
                },
            );
        }
        if self.within_interim_limits_count + self.outside_interim_limit_count != self.sample_count
        {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "interim_limit_counts",
                },
            );
        }
        if self.outside_interim_limit_count != self.outside_interim_limit_bodies.len() {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_bodies",
                },
            );
        }
        if self.outside_interim_limit_bodies != expected.outside_interim_limit_bodies {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_bodies",
                },
            );
        }
        if !body_labels_are_unique(&self.sample_bodies) {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if !body_labels_are_unique(&self.outside_interim_limit_bodies) {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_bodies",
                },
            );
        }
        if self.vendored_full_file_count + self.generated_binary_count + self.truncated_count
            != self.sample_count
        {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "source_kind_counts",
                },
            );
        }

        Ok(())
    }
}

/// Validation error for a VSOP87 source-backed body-evidence summary that drifted
/// from the current canonical evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87SourceBodyEvidenceSummaryValidationError {
    /// A rendered summary field no longer matches the current derived evidence.
    FieldOutOfSync {
        /// Name of the summary field that drifted from the derived evidence.
        field: &'static str,
    },
}

impl fmt::Display for Vsop87SourceBodyEvidenceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the VSOP87 source-backed body evidence summary field `{field}` is out of sync with the current canonical evidence"
            ),
        }
    }
}

impl std::error::Error for Vsop87SourceBodyEvidenceSummaryValidationError {}

impl fmt::Display for Vsop87SourceBodyEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns a backend-owned summary of the canonical VSOP87 body evidence.
pub fn source_body_evidence_summary() -> Option<Vsop87SourceBodyEvidenceSummary> {
    let evidence = canonical_epoch_body_evidence()?;
    Some(Vsop87SourceBodyEvidenceSummary {
        sample_count: evidence.len(),
        sample_bodies: evidence.iter().map(|row| row.body.clone()).collect(),
        within_interim_limits_count: evidence
            .iter()
            .filter(|row| row.within_interim_limits)
            .count(),
        vendored_full_file_count: evidence
            .iter()
            .filter(|row| row.source_kind == Vsop87BodySourceKind::VendoredVsop87b)
            .count(),
        generated_binary_count: evidence
            .iter()
            .filter(|row| row.source_kind == Vsop87BodySourceKind::GeneratedBinaryVsop87b)
            .count(),
        truncated_count: evidence
            .iter()
            .filter(|row| row.source_kind == Vsop87BodySourceKind::TruncatedVsop87b)
            .count(),
        outside_interim_limit_count: evidence
            .iter()
            .filter(|row| !row.within_interim_limits)
            .count(),
        outside_interim_limit_bodies: evidence
            .into_iter()
            .filter(|row| !row.within_interim_limits)
            .map(|row| row.body)
            .collect(),
    })
}

const CANONICAL_OUTLIER_NOTE_LABEL: &str = "VSOP87 canonical J2000 interim outliers";

/// Public summary of the canonical J2000 interim-outlier note.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalOutlierSummary {
    /// Bodies outside the current interim limits.
    pub outlier_bodies: Vec<CelestialBody>,
}

/// Structured validation errors for a canonical J2000 interim-outlier summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87CanonicalOutlierSummaryValidationError {
    /// The summary drifted away from the current derived evidence.
    FieldOutOfSync {
        /// Name of the summary field that drifted from the derived evidence.
        field: &'static str,
    },
}

impl fmt::Display for Vsop87CanonicalOutlierSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the VSOP87 canonical outlier summary field `{field}` is out of sync with the current canonical evidence"
            ),
        }
    }
}

impl std::error::Error for Vsop87CanonicalOutlierSummaryValidationError {}

impl Vsop87CanonicalOutlierSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        if self.outlier_bodies.is_empty() {
            format!("{CANONICAL_OUTLIER_NOTE_LABEL}: none")
        } else {
            format!(
                "{CANONICAL_OUTLIER_NOTE_LABEL}: {}",
                format_celestial_bodies(&self.outlier_bodies)
            )
        }
    }

    /// Returns the validated compact summary line used in release-facing reporting.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Vsop87CanonicalOutlierSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the summary still matches the current derived evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalOutlierSummaryValidationError> {
        let Some(expected) = canonical_epoch_outlier_summary() else {
            return Err(
                Vsop87CanonicalOutlierSummaryValidationError::FieldOutOfSync {
                    field: "outlier_bodies",
                },
            );
        };

        if self.outlier_bodies != expected.outlier_bodies {
            return Err(
                Vsop87CanonicalOutlierSummaryValidationError::FieldOutOfSync {
                    field: "outlier_bodies",
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalOutlierSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the release-facing canonical VSOP87 J2000 interim-outlier summary.
pub fn canonical_epoch_outlier_summary() -> Option<Vsop87CanonicalOutlierSummary> {
    let outlier_bodies = canonical_epoch_body_evidence().map(|evidence| {
        evidence
            .into_iter()
            .filter(|row| !row.within_interim_limits)
            .map(|row| row.body)
            .collect::<Vec<_>>()
    })?;

    Some(Vsop87CanonicalOutlierSummary { outlier_bodies })
}

/// Builds the per-body canonical-epoch evidence rows by evaluating the backend
/// at the J2000 reference sample and comparing each body's mean, J2000 ecliptic
/// result against its expected longitude/latitude/distance and interim limits.
///
/// Returns `None` if the backend output cannot be matched to the sample set or a
/// profile fails validation.
pub fn canonical_epoch_body_evidence() -> Option<Vec<Vsop87CanonicalBodyEvidence>> {
    let backend = Vsop87Backend::new();
    let profiles = body_source_profiles();
    let specs = source_specifications();
    let samples = canonical_epoch_samples();
    let requests = canonical_epoch_requests();
    let results = backend.positions(&requests).ok()?;

    if results.len() != samples.len() {
        return None;
    }

    let mut evidence = Vec::with_capacity(samples.len());

    for (sample, result) in samples.into_iter().zip(results) {
        if result.body != sample.body {
            return None;
        }

        let profile = profiles
            .iter()
            .find(|profile| profile.body == sample.body)?;
        profile.validate().ok()?;
        let spec = specs.iter().find(|spec| spec.body == sample.body)?;
        let ecliptic = result.ecliptic?;
        let distance = ecliptic.distance_au?;

        let longitude_delta = signed_longitude_delta_degrees(
            sample.expected_longitude_deg,
            ecliptic.longitude.degrees(),
        )
        .abs();
        let latitude_delta = (ecliptic.latitude.degrees() - sample.expected_latitude_deg).abs();
        let distance_delta = (distance - sample.expected_distance_au).abs();
        let within_interim_limits = longitude_delta <= sample.max_longitude_delta_deg
            && latitude_delta <= sample.max_latitude_delta_deg
            && distance_delta <= sample.max_distance_delta_au;

        let row = Vsop87CanonicalBodyEvidence {
            body: sample.body,
            source_kind: profile.kind,
            source_file: spec.source_file,
            provenance: profile.provenance,
            longitude_delta_deg: longitude_delta,
            latitude_delta_deg: latitude_delta,
            distance_delta_au: distance_delta,
            longitude_limit_deg: sample.max_longitude_delta_deg,
            latitude_limit_deg: sample.max_latitude_delta_deg,
            distance_limit_au: sample.max_distance_delta_au,
            within_interim_limits,
        };
        row.validate().ok()?;
        evidence.push(row);
    }

    Some(evidence)
}

fn derive_canonical_epoch_evidence_summary(
    body_evidence: &[Vsop87CanonicalBodyEvidence],
) -> Option<Vsop87CanonicalEvidenceSummary> {
    let sample_bodies = body_evidence
        .iter()
        .map(|evidence| evidence.body.clone())
        .collect::<Vec<_>>();
    let first = body_evidence.first()?;
    let mut sample_count = 0usize;
    let mut max_longitude_delta_body = first.body.clone();
    let mut max_longitude_delta_source_kind = first.source_kind;
    let mut max_longitude_delta_source_file = first.source_file;
    let mut max_longitude_delta_deg = first.longitude_delta_deg;
    let mut max_longitude_delta_limit_deg = first.longitude_limit_deg;
    let mut max_latitude_delta_body = first.body.clone();
    let mut max_latitude_delta_source_kind = first.source_kind;
    let mut max_latitude_delta_source_file = first.source_file;
    let mut max_latitude_delta_deg = first.latitude_delta_deg;
    let mut max_latitude_delta_limit_deg = first.latitude_limit_deg;
    let mut max_distance_delta_body = first.body.clone();
    let mut max_distance_delta_source_kind = first.source_kind;
    let mut max_distance_delta_source_file = first.source_file;
    let mut max_distance_delta_au = first.distance_delta_au;
    let mut max_distance_delta_limit_au = first.distance_limit_au;
    let mut total_longitude_delta_deg = 0.0;
    let mut total_latitude_delta_deg = 0.0;
    let mut total_distance_delta_au = 0.0;
    let mut longitude_values = Vec::with_capacity(body_evidence.len());
    let mut latitude_values = Vec::with_capacity(body_evidence.len());
    let mut distance_values = Vec::with_capacity(body_evidence.len());
    let mut out_of_limit_count = 0usize;
    let mut within_interim_limits = true;

    for evidence in body_evidence {
        sample_count += 1;
        total_longitude_delta_deg += evidence.longitude_delta_deg;
        total_latitude_delta_deg += evidence.latitude_delta_deg;
        total_distance_delta_au += evidence.distance_delta_au;
        longitude_values.push(evidence.longitude_delta_deg);
        latitude_values.push(evidence.latitude_delta_deg);
        distance_values.push(evidence.distance_delta_au);
        if !evidence.within_interim_limits {
            out_of_limit_count += 1;
        }
        if evidence.longitude_delta_deg >= max_longitude_delta_deg {
            max_longitude_delta_deg = evidence.longitude_delta_deg;
            max_longitude_delta_body = evidence.body.clone();
            max_longitude_delta_source_kind = evidence.source_kind;
            max_longitude_delta_source_file = evidence.source_file;
            max_longitude_delta_limit_deg = evidence.longitude_limit_deg;
        }
        if evidence.latitude_delta_deg >= max_latitude_delta_deg {
            max_latitude_delta_deg = evidence.latitude_delta_deg;
            max_latitude_delta_body = evidence.body.clone();
            max_latitude_delta_source_kind = evidence.source_kind;
            max_latitude_delta_source_file = evidence.source_file;
            max_latitude_delta_limit_deg = evidence.latitude_limit_deg;
        }
        if evidence.distance_delta_au >= max_distance_delta_au {
            max_distance_delta_au = evidence.distance_delta_au;
            max_distance_delta_body = evidence.body.clone();
            max_distance_delta_source_kind = evidence.source_kind;
            max_distance_delta_source_file = evidence.source_file;
            max_distance_delta_limit_au = evidence.distance_limit_au;
        }
        within_interim_limits &= evidence.within_interim_limits;
    }

    Some(Vsop87CanonicalEvidenceSummary {
        sample_count,
        sample_bodies,
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
        mean_longitude_delta_deg: total_longitude_delta_deg / sample_count as f64,
        median_longitude_delta_deg: median_f64(&mut longitude_values),
        percentile_longitude_delta_deg: percentile_f64(&mut longitude_values, 0.95),
        rms_longitude_delta_deg: rms_f64(&longitude_values),
        mean_latitude_delta_deg: total_latitude_delta_deg / sample_count as f64,
        median_latitude_delta_deg: median_f64(&mut latitude_values),
        percentile_latitude_delta_deg: percentile_f64(&mut latitude_values, 0.95),
        rms_latitude_delta_deg: rms_f64(&latitude_values),
        mean_distance_delta_au: total_distance_delta_au / sample_count as f64,
        median_distance_delta_au: median_f64(&mut distance_values),
        percentile_distance_delta_au: percentile_f64(&mut distance_values, 0.95),
        rms_distance_delta_au: rms_f64(&distance_values),
        out_of_limit_count,
        within_interim_limits,
    })
}

fn validate_canonical_epoch_evidence_summary_against_current_evidence(
    summary: &Vsop87CanonicalEvidenceSummary,
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    let body_evidence = canonical_epoch_body_evidence().ok_or(
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            field: "sample_count",
        },
    )?;
    let expected = derive_canonical_epoch_evidence_summary(&body_evidence).ok_or(
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            field: "sample_count",
        },
    )?;

    macro_rules! check_field {
        ($field:literal, $left:expr, $right:expr) => {
            if $left != $right {
                return Err(
                    Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                        summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
                        field: $field,
                    },
                );
            }
        };
    }

    check_field!("sample_count", summary.sample_count, expected.sample_count);
    check_field!(
        "sample_bodies",
        summary.sample_bodies,
        expected.sample_bodies
    );
    check_field!(
        "max_longitude_delta_body",
        summary.max_longitude_delta_body,
        expected.max_longitude_delta_body
    );
    check_field!(
        "max_longitude_delta_source_kind",
        summary.max_longitude_delta_source_kind,
        expected.max_longitude_delta_source_kind
    );
    check_field!(
        "max_longitude_delta_source_file",
        summary.max_longitude_delta_source_file,
        expected.max_longitude_delta_source_file
    );
    check_field!(
        "max_longitude_delta_deg",
        summary.max_longitude_delta_deg,
        expected.max_longitude_delta_deg
    );
    check_field!(
        "max_longitude_delta_limit_deg",
        summary.max_longitude_delta_limit_deg,
        expected.max_longitude_delta_limit_deg
    );
    check_field!(
        "max_latitude_delta_body",
        summary.max_latitude_delta_body,
        expected.max_latitude_delta_body
    );
    check_field!(
        "max_latitude_delta_source_kind",
        summary.max_latitude_delta_source_kind,
        expected.max_latitude_delta_source_kind
    );
    check_field!(
        "max_latitude_delta_source_file",
        summary.max_latitude_delta_source_file,
        expected.max_latitude_delta_source_file
    );
    check_field!(
        "max_latitude_delta_deg",
        summary.max_latitude_delta_deg,
        expected.max_latitude_delta_deg
    );
    check_field!(
        "max_latitude_delta_limit_deg",
        summary.max_latitude_delta_limit_deg,
        expected.max_latitude_delta_limit_deg
    );
    check_field!(
        "max_distance_delta_body",
        summary.max_distance_delta_body,
        expected.max_distance_delta_body
    );
    check_field!(
        "max_distance_delta_source_kind",
        summary.max_distance_delta_source_kind,
        expected.max_distance_delta_source_kind
    );
    check_field!(
        "max_distance_delta_source_file",
        summary.max_distance_delta_source_file,
        expected.max_distance_delta_source_file
    );
    check_field!(
        "max_distance_delta_au",
        summary.max_distance_delta_au,
        expected.max_distance_delta_au
    );
    check_field!(
        "max_distance_delta_limit_au",
        summary.max_distance_delta_limit_au,
        expected.max_distance_delta_limit_au
    );
    check_field!(
        "mean_longitude_delta_deg",
        summary.mean_longitude_delta_deg,
        expected.mean_longitude_delta_deg
    );
    check_field!(
        "median_longitude_delta_deg",
        summary.median_longitude_delta_deg,
        expected.median_longitude_delta_deg
    );
    check_field!(
        "percentile_longitude_delta_deg",
        summary.percentile_longitude_delta_deg,
        expected.percentile_longitude_delta_deg
    );
    check_field!(
        "rms_longitude_delta_deg",
        summary.rms_longitude_delta_deg,
        expected.rms_longitude_delta_deg
    );
    check_field!(
        "mean_latitude_delta_deg",
        summary.mean_latitude_delta_deg,
        expected.mean_latitude_delta_deg
    );
    check_field!(
        "median_latitude_delta_deg",
        summary.median_latitude_delta_deg,
        expected.median_latitude_delta_deg
    );
    check_field!(
        "percentile_latitude_delta_deg",
        summary.percentile_latitude_delta_deg,
        expected.percentile_latitude_delta_deg
    );
    check_field!(
        "rms_latitude_delta_deg",
        summary.rms_latitude_delta_deg,
        expected.rms_latitude_delta_deg
    );
    check_field!(
        "mean_distance_delta_au",
        summary.mean_distance_delta_au,
        expected.mean_distance_delta_au
    );
    check_field!(
        "median_distance_delta_au",
        summary.median_distance_delta_au,
        expected.median_distance_delta_au
    );
    check_field!(
        "percentile_distance_delta_au",
        summary.percentile_distance_delta_au,
        expected.percentile_distance_delta_au
    );
    check_field!(
        "rms_distance_delta_au",
        summary.rms_distance_delta_au,
        expected.rms_distance_delta_au
    );
    check_field!(
        "out_of_limit_count",
        summary.out_of_limit_count,
        expected.out_of_limit_count
    );
    check_field!(
        "within_interim_limits",
        summary.within_interim_limits,
        expected.within_interim_limits
    );

    Ok(())
}

/// Returns the canonical J2000 error envelope summary used by release-facing
/// validation reports.
pub fn canonical_epoch_evidence_summary() -> Option<Vsop87CanonicalEvidenceSummary> {
    let body_evidence = canonical_epoch_body_evidence()?;
    derive_canonical_epoch_evidence_summary(&body_evidence)
}

/// Returns the canonical J2000 equatorial companion evidence used by
/// validation reporting.
pub fn canonical_epoch_equatorial_body_evidence(
) -> Option<Vec<Vsop87CanonicalEquatorialBodyEvidence>> {
    let backend = Vsop87Backend::new();
    let profiles = body_source_profiles();
    let specs = source_specifications();
    let samples = canonical_epoch_samples();
    let requests = canonical_epoch_requests()
        .into_iter()
        .map(|mut request| {
            request.frame = CoordinateFrame::Equatorial;
            request
        })
        .collect::<Vec<_>>();
    let results = backend.positions(&requests).ok()?;
    let reference_obliquity =
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt).mean_obliquity();

    if results.len() != samples.len() {
        return None;
    }

    let mut evidence = Vec::with_capacity(samples.len());

    for (sample, result) in samples.into_iter().zip(results) {
        if result.body != sample.body {
            return None;
        }

        let profile = profiles
            .iter()
            .find(|profile| profile.body == sample.body)?;
        profile.validate().ok()?;
        let spec = specs.iter().find(|spec| spec.body == sample.body)?;
        let expected_ecliptic = EclipticCoordinates::new(
            Longitude::from_degrees(sample.expected_longitude_deg),
            Latitude::from_degrees(sample.expected_latitude_deg),
            Some(sample.expected_distance_au),
        );
        let expected_equatorial = expected_ecliptic.to_equatorial(reference_obliquity);
        let actual_equatorial = result.equatorial?;

        evidence.push(Vsop87CanonicalEquatorialBodyEvidence {
            body: sample.body,
            source_kind: profile.kind,
            source_file: spec.source_file,
            provenance: profile.provenance,
            right_ascension_delta_deg: signed_longitude_delta_degrees(
                expected_equatorial.right_ascension.degrees(),
                actual_equatorial.right_ascension.degrees(),
            )
            .abs(),
            declination_delta_deg: (actual_equatorial.declination.degrees()
                - expected_equatorial.declination.degrees())
            .abs(),
            distance_delta_au: (actual_equatorial.distance_au? - expected_equatorial.distance_au?)
                .abs(),
        });
    }

    Some(evidence)
}

fn derive_canonical_epoch_equatorial_evidence_summary(
    body_evidence: &[Vsop87CanonicalEquatorialBodyEvidence],
) -> Option<Vsop87CanonicalEquatorialEvidenceSummary> {
    let sample_bodies = body_evidence
        .iter()
        .map(|evidence| evidence.body.clone())
        .collect::<Vec<_>>();
    let first = body_evidence.first()?;
    let mut sample_count = 0usize;
    let mut max_right_ascension_delta_body = first.body.clone();
    let mut max_right_ascension_delta_source_kind = first.source_kind;
    let mut max_right_ascension_delta_source_file = first.source_file;
    let mut max_right_ascension_delta_deg = first.right_ascension_delta_deg;
    let mut max_declination_delta_body = first.body.clone();
    let mut max_declination_delta_source_kind = first.source_kind;
    let mut max_declination_delta_source_file = first.source_file;
    let mut max_declination_delta_deg = first.declination_delta_deg;
    let mut max_distance_delta_body = first.body.clone();
    let mut max_distance_delta_source_kind = first.source_kind;
    let mut max_distance_delta_source_file = first.source_file;
    let mut max_distance_delta_au = first.distance_delta_au;
    let mut total_right_ascension_delta_deg = 0.0;
    let mut total_declination_delta_deg = 0.0;
    let mut total_distance_delta_au = 0.0;
    let mut right_ascension_values = Vec::with_capacity(body_evidence.len());
    let mut declination_values = Vec::with_capacity(body_evidence.len());
    let mut distance_values = Vec::with_capacity(body_evidence.len());

    for evidence in body_evidence {
        sample_count += 1;
        total_right_ascension_delta_deg += evidence.right_ascension_delta_deg;
        total_declination_delta_deg += evidence.declination_delta_deg;
        total_distance_delta_au += evidence.distance_delta_au;
        right_ascension_values.push(evidence.right_ascension_delta_deg);
        declination_values.push(evidence.declination_delta_deg);
        distance_values.push(evidence.distance_delta_au);
        if evidence.right_ascension_delta_deg >= max_right_ascension_delta_deg {
            max_right_ascension_delta_deg = evidence.right_ascension_delta_deg;
            max_right_ascension_delta_body = evidence.body.clone();
            max_right_ascension_delta_source_kind = evidence.source_kind;
            max_right_ascension_delta_source_file = evidence.source_file;
        }
        if evidence.declination_delta_deg >= max_declination_delta_deg {
            max_declination_delta_deg = evidence.declination_delta_deg;
            max_declination_delta_body = evidence.body.clone();
            max_declination_delta_source_kind = evidence.source_kind;
            max_declination_delta_source_file = evidence.source_file;
        }
        if evidence.distance_delta_au >= max_distance_delta_au {
            max_distance_delta_au = evidence.distance_delta_au;
            max_distance_delta_body = evidence.body.clone();
            max_distance_delta_source_kind = evidence.source_kind;
            max_distance_delta_source_file = evidence.source_file;
        }
    }

    Some(Vsop87CanonicalEquatorialEvidenceSummary {
        sample_count,
        sample_bodies,
        max_right_ascension_delta_body,
        max_right_ascension_delta_source_kind,
        max_right_ascension_delta_source_file,
        max_right_ascension_delta_deg,
        max_declination_delta_body,
        max_declination_delta_source_kind,
        max_declination_delta_source_file,
        max_declination_delta_deg,
        max_distance_delta_body,
        max_distance_delta_source_kind,
        max_distance_delta_source_file,
        max_distance_delta_au,
        mean_right_ascension_delta_deg: total_right_ascension_delta_deg / sample_count as f64,
        median_right_ascension_delta_deg: median_f64(&mut right_ascension_values),
        percentile_right_ascension_delta_deg: percentile_f64(&mut right_ascension_values, 0.95),
        rms_right_ascension_delta_deg: rms_f64(&right_ascension_values),
        mean_declination_delta_deg: total_declination_delta_deg / sample_count as f64,
        median_declination_delta_deg: median_f64(&mut declination_values),
        percentile_declination_delta_deg: percentile_f64(&mut declination_values, 0.95),
        rms_declination_delta_deg: rms_f64(&declination_values),
        mean_distance_delta_au: total_distance_delta_au / sample_count as f64,
        median_distance_delta_au: median_f64(&mut distance_values),
        percentile_distance_delta_au: percentile_f64(&mut distance_values, 0.95),
        rms_distance_delta_au: rms_f64(&distance_values),
    })
}

fn validate_canonical_epoch_equatorial_evidence_summary_against_current_evidence(
    summary: &Vsop87CanonicalEquatorialEvidenceSummary,
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    let body_evidence = canonical_epoch_equatorial_body_evidence().ok_or(
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            field: "sample_count",
        },
    )?;
    let expected = derive_canonical_epoch_equatorial_evidence_summary(&body_evidence).ok_or(
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            field: "sample_count",
        },
    )?;

    macro_rules! check_field {
        ($field:literal, $left:expr, $right:expr) => {
            if $left != $right {
                return Err(
                    Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                        summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
                        field: $field,
                    },
                );
            }
        };
    }

    check_field!("sample_count", summary.sample_count, expected.sample_count);
    check_field!(
        "sample_bodies",
        summary.sample_bodies,
        expected.sample_bodies
    );
    check_field!(
        "max_right_ascension_delta_body",
        summary.max_right_ascension_delta_body,
        expected.max_right_ascension_delta_body
    );
    check_field!(
        "max_right_ascension_delta_source_kind",
        summary.max_right_ascension_delta_source_kind,
        expected.max_right_ascension_delta_source_kind
    );
    check_field!(
        "max_right_ascension_delta_source_file",
        summary.max_right_ascension_delta_source_file,
        expected.max_right_ascension_delta_source_file
    );
    check_field!(
        "max_right_ascension_delta_deg",
        summary.max_right_ascension_delta_deg,
        expected.max_right_ascension_delta_deg
    );
    check_field!(
        "max_declination_delta_body",
        summary.max_declination_delta_body,
        expected.max_declination_delta_body
    );
    check_field!(
        "max_declination_delta_source_kind",
        summary.max_declination_delta_source_kind,
        expected.max_declination_delta_source_kind
    );
    check_field!(
        "max_declination_delta_source_file",
        summary.max_declination_delta_source_file,
        expected.max_declination_delta_source_file
    );
    check_field!(
        "max_declination_delta_deg",
        summary.max_declination_delta_deg,
        expected.max_declination_delta_deg
    );
    check_field!(
        "max_distance_delta_body",
        summary.max_distance_delta_body,
        expected.max_distance_delta_body
    );
    check_field!(
        "max_distance_delta_source_kind",
        summary.max_distance_delta_source_kind,
        expected.max_distance_delta_source_kind
    );
    check_field!(
        "max_distance_delta_source_file",
        summary.max_distance_delta_source_file,
        expected.max_distance_delta_source_file
    );
    check_field!(
        "max_distance_delta_au",
        summary.max_distance_delta_au,
        expected.max_distance_delta_au
    );
    check_field!(
        "mean_right_ascension_delta_deg",
        summary.mean_right_ascension_delta_deg,
        expected.mean_right_ascension_delta_deg
    );
    check_field!(
        "median_right_ascension_delta_deg",
        summary.median_right_ascension_delta_deg,
        expected.median_right_ascension_delta_deg
    );
    check_field!(
        "percentile_right_ascension_delta_deg",
        summary.percentile_right_ascension_delta_deg,
        expected.percentile_right_ascension_delta_deg
    );
    check_field!(
        "rms_right_ascension_delta_deg",
        summary.rms_right_ascension_delta_deg,
        expected.rms_right_ascension_delta_deg
    );
    check_field!(
        "mean_declination_delta_deg",
        summary.mean_declination_delta_deg,
        expected.mean_declination_delta_deg
    );
    check_field!(
        "median_declination_delta_deg",
        summary.median_declination_delta_deg,
        expected.median_declination_delta_deg
    );
    check_field!(
        "percentile_declination_delta_deg",
        summary.percentile_declination_delta_deg,
        expected.percentile_declination_delta_deg
    );
    check_field!(
        "rms_declination_delta_deg",
        summary.rms_declination_delta_deg,
        expected.rms_declination_delta_deg
    );
    check_field!(
        "mean_distance_delta_au",
        summary.mean_distance_delta_au,
        expected.mean_distance_delta_au
    );
    check_field!(
        "median_distance_delta_au",
        summary.median_distance_delta_au,
        expected.median_distance_delta_au
    );
    check_field!(
        "percentile_distance_delta_au",
        summary.percentile_distance_delta_au,
        expected.percentile_distance_delta_au
    );
    check_field!(
        "rms_distance_delta_au",
        summary.rms_distance_delta_au,
        expected.rms_distance_delta_au
    );

    Ok(())
}

/// Returns the canonical J2000 equatorial companion evidence summary used by
/// release-facing validation reports.
pub fn canonical_epoch_equatorial_evidence_summary(
) -> Option<Vsop87CanonicalEquatorialEvidenceSummary> {
    let body_evidence = canonical_epoch_equatorial_body_evidence()?;
    derive_canonical_epoch_equatorial_evidence_summary(&body_evidence)
}

/// Backend-owned summary for the canonical J2000 equatorial body classes.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalEquatorialBodyClassEvidenceSummary {
    /// Body class covered by this summary.
    pub class: Vsop87SourceBodyClass,
    /// Number of canonical samples measured for the class.
    pub sample_count: usize,
    /// Canonical bodies measured in source-backed order for the class.
    pub sample_bodies: Vec<CelestialBody>,
    /// Body with the maximum absolute right ascension delta.
    pub max_right_ascension_delta_body: CelestialBody,
    /// Calculation family behind the maximum right ascension delta body.
    pub max_right_ascension_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum right ascension delta body.
    pub max_right_ascension_delta_source_file: &'static str,
    /// Maximum absolute right ascension delta in degrees.
    pub max_right_ascension_delta_deg: f64,
    /// Body with the maximum absolute declination delta.
    pub max_declination_delta_body: CelestialBody,
    /// Calculation family behind the maximum declination delta body.
    pub max_declination_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum declination delta body.
    pub max_declination_delta_source_file: &'static str,
    /// Maximum absolute declination delta in degrees.
    pub max_declination_delta_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: CelestialBody,
    /// Calculation family behind the maximum distance delta body.
    pub max_distance_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum distance delta body.
    pub max_distance_delta_source_file: &'static str,
    /// Maximum absolute distance delta in astronomical units.
    pub max_distance_delta_au: f64,
    /// Mean absolute right ascension delta in degrees.
    pub mean_right_ascension_delta_deg: f64,
    /// Median absolute right ascension delta in degrees.
    pub median_right_ascension_delta_deg: f64,
    /// 95th percentile absolute right ascension delta in degrees.
    pub percentile_right_ascension_delta_deg: f64,
    /// Root-mean-square right ascension delta in degrees.
    pub rms_right_ascension_delta_deg: f64,
    /// Mean absolute declination delta in degrees.
    pub mean_declination_delta_deg: f64,
    /// Median absolute declination delta in degrees.
    pub median_declination_delta_deg: f64,
    /// 95th percentile absolute declination delta in degrees.
    pub percentile_declination_delta_deg: f64,
    /// Root-mean-square declination delta in degrees.
    pub rms_declination_delta_deg: f64,
    /// Mean absolute distance delta in astronomical units.
    pub mean_distance_delta_au: f64,
    /// Median absolute distance delta in astronomical units.
    pub median_distance_delta_au: f64,
    /// 95th percentile absolute distance delta in astronomical units.
    pub percentile_distance_delta_au: f64,
    /// Root-mean-square distance delta in astronomical units.
    pub rms_distance_delta_au: f64,
}

impl Vsop87CanonicalEquatorialBodyClassEvidenceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format_canonical_equatorial_body_class_evidence_entry(self)
    }

    /// Returns `Ok(())` when the summary still matches the current derived evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
        validate_canonical_evidence_summary_bodies(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            self.sample_count,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_right_ascension_delta_body",
            &self.max_right_ascension_delta_body,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_declination_delta_body",
            &self.max_declination_delta_body,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_body",
            &self.max_distance_delta_body,
            &self.sample_bodies,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_right_ascension_delta_source_file",
            self.max_right_ascension_delta_source_file,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_declination_delta_source_file",
            self.max_declination_delta_source_file,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_source_file",
            self.max_distance_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_right_ascension_delta_source_kind",
            "max_right_ascension_delta_source_file",
            &self.max_right_ascension_delta_body,
            self.max_right_ascension_delta_source_kind,
            self.max_right_ascension_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_declination_delta_source_kind",
            "max_declination_delta_source_file",
            &self.max_declination_delta_body,
            self.max_declination_delta_source_kind,
            self.max_declination_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_source_kind",
            "max_distance_delta_source_file",
            &self.max_distance_delta_body,
            self.max_distance_delta_source_kind,
            self.max_distance_delta_source_file,
        )?;
        for (field, value) in [
            (
                "max_right_ascension_delta_deg",
                self.max_right_ascension_delta_deg,
            ),
            ("max_declination_delta_deg", self.max_declination_delta_deg),
            ("max_distance_delta_au", self.max_distance_delta_au),
            (
                "mean_right_ascension_delta_deg",
                self.mean_right_ascension_delta_deg,
            ),
            (
                "median_right_ascension_delta_deg",
                self.median_right_ascension_delta_deg,
            ),
            (
                "percentile_right_ascension_delta_deg",
                self.percentile_right_ascension_delta_deg,
            ),
            (
                "rms_right_ascension_delta_deg",
                self.rms_right_ascension_delta_deg,
            ),
            (
                "mean_declination_delta_deg",
                self.mean_declination_delta_deg,
            ),
            (
                "median_declination_delta_deg",
                self.median_declination_delta_deg,
            ),
            (
                "percentile_declination_delta_deg",
                self.percentile_declination_delta_deg,
            ),
            ("rms_declination_delta_deg", self.rms_declination_delta_deg),
            ("mean_distance_delta_au", self.mean_distance_delta_au),
            ("median_distance_delta_au", self.median_distance_delta_au),
            (
                "percentile_distance_delta_au",
                self.percentile_distance_delta_au,
            ),
            ("rms_distance_delta_au", self.rms_distance_delta_au),
        ] {
            validate_finite_non_negative_measure(
                CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
                field,
                value,
            )?;
        }

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalEquatorialBodyClassEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned canonical J2000 equatorial body-class evidence.
pub fn canonical_epoch_equatorial_body_class_evidence_summary(
) -> Option<Vec<Vsop87CanonicalEquatorialBodyClassEvidenceSummary>> {
    let evidence = canonical_epoch_equatorial_body_evidence()?;
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
        let mut right_ascension_values = Vec::with_capacity(class_rows.len());
        let mut declination_values = Vec::with_capacity(class_rows.len());
        let mut distance_values = Vec::with_capacity(class_rows.len());
        let mut max_right_ascension_delta_body = class_rows[0].body.clone();
        let mut max_right_ascension_delta_source_kind = class_rows[0].source_kind;
        let mut max_right_ascension_delta_source_file = class_rows[0].source_file;
        let mut max_right_ascension_delta_deg = class_rows[0].right_ascension_delta_deg;
        let mut max_declination_delta_body = class_rows[0].body.clone();
        let mut max_declination_delta_source_kind = class_rows[0].source_kind;
        let mut max_declination_delta_source_file = class_rows[0].source_file;
        let mut max_declination_delta_deg = class_rows[0].declination_delta_deg;
        let mut max_distance_delta_body = class_rows[0].body.clone();
        let mut max_distance_delta_source_kind = class_rows[0].source_kind;
        let mut max_distance_delta_source_file = class_rows[0].source_file;
        let mut max_distance_delta_au = class_rows[0].distance_delta_au;

        for row in &class_rows {
            right_ascension_values.push(row.right_ascension_delta_deg);
            declination_values.push(row.declination_delta_deg);
            distance_values.push(row.distance_delta_au);

            if row.right_ascension_delta_deg > max_right_ascension_delta_deg {
                max_right_ascension_delta_body = row.body.clone();
                max_right_ascension_delta_source_kind = row.source_kind;
                max_right_ascension_delta_source_file = row.source_file;
                max_right_ascension_delta_deg = row.right_ascension_delta_deg;
            }
            if row.declination_delta_deg > max_declination_delta_deg {
                max_declination_delta_body = row.body.clone();
                max_declination_delta_source_kind = row.source_kind;
                max_declination_delta_source_file = row.source_file;
                max_declination_delta_deg = row.declination_delta_deg;
            }
            if row.distance_delta_au > max_distance_delta_au {
                max_distance_delta_body = row.body.clone();
                max_distance_delta_source_kind = row.source_kind;
                max_distance_delta_source_file = row.source_file;
                max_distance_delta_au = row.distance_delta_au;
            }
        }

        let sample_count = class_rows.len();
        let mut right_ascension_values_for_median = right_ascension_values.clone();
        let mut right_ascension_values_for_percentile = right_ascension_values;
        let mut declination_values_for_median = declination_values.clone();
        let mut declination_values_for_percentile = declination_values;
        let mut distance_values_for_median = distance_values.clone();
        let mut distance_values_for_percentile = distance_values;
        summaries.push(Vsop87CanonicalEquatorialBodyClassEvidenceSummary {
            class,
            sample_count,
            sample_bodies,
            max_right_ascension_delta_body,
            max_right_ascension_delta_source_kind,
            max_right_ascension_delta_source_file,
            max_right_ascension_delta_deg,
            max_declination_delta_body,
            max_declination_delta_source_kind,
            max_declination_delta_source_file,
            max_declination_delta_deg,
            max_distance_delta_body,
            max_distance_delta_source_kind,
            max_distance_delta_source_file,
            max_distance_delta_au,
            mean_right_ascension_delta_deg: right_ascension_values_for_median.iter().sum::<f64>()
                / sample_count as f64,
            median_right_ascension_delta_deg: median_f64(&mut right_ascension_values_for_median),
            percentile_right_ascension_delta_deg: percentile_f64(
                &mut right_ascension_values_for_percentile,
                0.95,
            ),
            rms_right_ascension_delta_deg: rms_f64(&right_ascension_values_for_percentile),
            mean_declination_delta_deg: declination_values_for_median.iter().sum::<f64>()
                / sample_count as f64,
            median_declination_delta_deg: median_f64(&mut declination_values_for_median),
            percentile_declination_delta_deg: percentile_f64(
                &mut declination_values_for_percentile,
                0.95,
            ),
            rms_declination_delta_deg: rms_f64(&declination_values_for_percentile),
            mean_distance_delta_au: distance_values_for_median.iter().sum::<f64>()
                / sample_count as f64,
            median_distance_delta_au: median_f64(&mut distance_values_for_median),
            percentile_distance_delta_au: percentile_f64(&mut distance_values_for_percentile, 0.95),
            rms_distance_delta_au: rms_f64(&distance_values_for_percentile),
        });
    }

    Some(summaries)
}

/// Formats a single canonical VSOP87 equatorial body-class evidence envelope.
fn format_canonical_equatorial_body_class_evidence_entry(
    summary: &Vsop87CanonicalEquatorialBodyClassEvidenceSummary,
) -> String {
    format!(
        "{}: samples={}, bodies: {}, mean Δra={:.12}°, median Δra={:.12}°, p95 Δra={:.12}°, rms Δra={:.12}°, mean Δdec={:.12}°, median Δdec={:.12}°, p95 Δdec={:.12}°, rms Δdec={:.12}°, mean Δdist={:.12} AU, median Δdist={:.12} AU, p95 Δdist={:.12} AU, rms Δdist={:.12} AU, max Δra={:.12}° ({}; {}), max Δdec={:.12}° ({}; {}), max Δdist={:.12} AU ({}; {})",
        summary.class,
        summary.sample_count,
        format_celestial_bodies(&summary.sample_bodies),
        summary.mean_right_ascension_delta_deg,
        summary.median_right_ascension_delta_deg,
        summary.percentile_right_ascension_delta_deg,
        summary.rms_right_ascension_delta_deg,
        summary.mean_declination_delta_deg,
        summary.median_declination_delta_deg,
        summary.percentile_declination_delta_deg,
        summary.rms_declination_delta_deg,
        summary.mean_distance_delta_au,
        summary.median_distance_delta_au,
        summary.percentile_distance_delta_au,
        summary.rms_distance_delta_au,
        summary.max_right_ascension_delta_deg,
        summary.max_right_ascension_delta_source_kind,
        summary.max_right_ascension_delta_source_file,
        summary.max_declination_delta_deg,
        summary.max_declination_delta_source_kind,
        summary.max_declination_delta_source_file,
        summary.max_distance_delta_au,
        summary.max_distance_delta_source_kind,
        summary.max_distance_delta_source_file,
    )
}

pub(crate) fn median_f64(values: &mut [f64]) -> f64 {
    values.sort_by(|left, right| left.total_cmp(right));
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

pub(crate) fn percentile_f64(values: &mut [f64], percentile: f64) -> f64 {
    values.sort_by(|left, right| left.total_cmp(right));
    let percentile = percentile.clamp(0.0, 1.0);
    if values.len() == 1 {
        return values[0];
    }
    let position = percentile * (values.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        values[lower]
    } else {
        let fraction = position - lower as f64;
        values[lower] + (values[upper] - values[lower]) * fraction
    }
}

pub(crate) fn rms_f64(values: &[f64]) -> f64 {
    let mean_square = values.iter().map(|value| value * value).sum::<f64>() / values.len() as f64;
    mean_square.sqrt()
}
/// Body classes used for source-backed VSOP87 error-envelope rollups.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vsop87SourceBodyClass {
    /// The source-backed solar body.
    Luminary,
    /// The source-backed planetary bodies.
    MajorPlanet,
}

impl Vsop87SourceBodyClass {
    pub(crate) const ALL: [Self; 2] = [Self::Luminary, Self::MajorPlanet];

    /// Human-readable label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Luminary => "Luminary",
            Self::MajorPlanet => "Major planets",
        }
    }
}

impl fmt::Display for Vsop87SourceBodyClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

pub(crate) fn source_body_class(body: &CelestialBody) -> Vsop87SourceBodyClass {
    match body {
        CelestialBody::Sun => Vsop87SourceBodyClass::Luminary,
        _ => Vsop87SourceBodyClass::MajorPlanet,
    }
}
