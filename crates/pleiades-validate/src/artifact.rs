use core::fmt;
use std::sync::OnceLock;
use std::time::Instant as StdInstant;

use crate::{
    compare_backends, default_candidate_backend, ComparisonReport, ComparisonSample,
    ValidationCorpus,
};
use pleiades_compression::{join_display, CompressedArtifact, CompressionError, EndianPolicy};
use pleiades_core::{
    Angle, Apparentness, BackendFamily, CelestialBody, CoordinateFrame, EclipticCoordinates,
    EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest, Instant, JulianDay,
    ZodiacMode,
};
use pleiades_data::{
    packaged_artifact, packaged_artifact_generation_manifest_for_report,
    packaged_artifact_output_support_summary_for_report,
    packaged_artifact_production_profile_summary_for_report,
    packaged_artifact_profile_summary_with_body_coverage,
    packaged_artifact_regeneration_summary_for_report,
    packaged_artifact_storage_summary_for_report, packaged_backend,
    packaged_frame_treatment_summary_details, packaged_request_policy_summary_details,
};

/// A report describing the bundled compressed artifact and its boundary checks.
#[derive(Clone, Debug)]
pub struct ArtifactInspectionReport {
    /// Human-readable label from the artifact header.
    pub generation_label: String,
    /// Source/provenance summary from the artifact header.
    pub source: String,
    /// Artifact format version.
    pub version: u16,
    /// Byte-order policy encoded in the artifact header.
    pub endian_policy: EndianPolicy,
    /// The encoded artifact checksum.
    pub checksum: u64,
    /// Size of the encoded artifact in bytes.
    pub encoded_bytes: usize,
    /// Whether the codec roundtrip preserved the artifact structure.
    pub roundtrip_ok: bool,
    /// Whether the decoded checksum matches the encoded checksum.
    pub checksum_ok: bool,
    /// Number of bodies in the artifact.
    pub body_count: usize,
    /// Number of segments across all bodies.
    pub segment_count: usize,
    /// Number of segments that carry residual-correction channels.
    pub residual_segment_count: usize,
    /// Bodies that carry at least one residual-correction segment.
    pub residual_bodies: Vec<CelestialBody>,
    /// Earliest covered instant.
    pub earliest: Instant,
    /// Latest covered instant.
    pub latest: Instant,
    /// Comparison report against the algorithmic baseline.
    pub model_comparison: ComparisonReport,
    /// Decode benchmark for the packaged artifact.
    pub decode_benchmark: ArtifactDecodeBenchmarkReport,
    /// Lookup benchmark for the packaged artifact.
    pub lookup_benchmark: ArtifactLookupBenchmarkReport,
    /// Batch lookup benchmark for the packaged artifact.
    pub batch_lookup_benchmark: ArtifactBatchLookupBenchmarkReport,
    /// Per-body validation summaries.
    pub bodies: Vec<ArtifactBodyInspection>,
}

/// Validation summary for a single body in the packaged artifact.
#[derive(Clone, Debug)]
pub struct ArtifactBodyInspection {
    /// Body identifier.
    pub body: CelestialBody,
    /// Number of segments for the body.
    pub segment_count: usize,
    /// Earliest segment start.
    pub earliest: Instant,
    /// Latest segment end.
    pub latest: Instant,
    /// Number of sample lookups exercised for this body.
    pub sample_count: usize,
    /// Smallest observed segment span in days.
    pub min_segment_span_days: f64,
    /// Largest observed segment span in days.
    pub max_segment_span_days: f64,
    /// Mean observed segment span in days.
    pub mean_segment_span_days: f64,
    /// Number of segments that carry residual-correction channels.
    pub residual_segment_count: usize,
    /// Number of shared segment boundaries checked for continuity.
    pub boundary_checks: usize,
    /// Sum of longitude deltas across all checked boundaries.
    pub sum_boundary_longitude_delta_deg: f64,
    /// Sum of squared longitude deltas across all checked boundaries.
    pub sum_boundary_longitude_delta_deg_sq: f64,
    /// Sum of latitude deltas across all checked boundaries.
    pub sum_boundary_latitude_delta_deg: f64,
    /// Sum of squared latitude deltas across all checked boundaries.
    pub sum_boundary_latitude_delta_deg_sq: f64,
    /// Sum of distance deltas across all checked boundaries that had distances.
    pub sum_boundary_distance_delta_au: Option<f64>,
    /// Sum of squared distance deltas across all checked boundaries that had distances.
    pub sum_boundary_distance_delta_au_sq: Option<f64>,
    /// Number of checked boundaries that had a distance delta.
    pub boundary_distance_checks: usize,
    /// Maximum longitude delta observed at any checked boundary.
    pub max_boundary_longitude_delta_deg: f64,
    /// Maximum latitude delta observed at any checked boundary.
    pub max_boundary_latitude_delta_deg: f64,
    /// Maximum distance delta observed at any checked boundary.
    pub max_boundary_distance_delta_au: Option<f64>,
}

impl ArtifactBodyInspection {
    /// Returns the mean longitude delta across the checked boundaries.
    pub fn mean_boundary_longitude_delta_deg(&self) -> f64 {
        if self.boundary_checks == 0 {
            0.0
        } else {
            self.sum_boundary_longitude_delta_deg / self.boundary_checks as f64
        }
    }

    /// Returns the root-mean-square longitude delta across the checked boundaries.
    pub fn rms_boundary_longitude_delta_deg(&self) -> f64 {
        if self.boundary_checks == 0 {
            0.0
        } else {
            (self.sum_boundary_longitude_delta_deg_sq / self.boundary_checks as f64).sqrt()
        }
    }

    /// Returns the mean latitude delta across the checked boundaries.
    pub fn mean_boundary_latitude_delta_deg(&self) -> f64 {
        if self.boundary_checks == 0 {
            0.0
        } else {
            self.sum_boundary_latitude_delta_deg / self.boundary_checks as f64
        }
    }

    /// Returns the root-mean-square latitude delta across the checked boundaries.
    pub fn rms_boundary_latitude_delta_deg(&self) -> f64 {
        if self.boundary_checks == 0 {
            0.0
        } else {
            (self.sum_boundary_latitude_delta_deg_sq / self.boundary_checks as f64).sqrt()
        }
    }

    /// Returns the mean distance delta across the checked boundaries that had distances.
    pub fn mean_boundary_distance_delta_au(&self) -> Option<f64> {
        self.sum_boundary_distance_delta_au.map(|sum| {
            if self.boundary_distance_checks == 0 {
                0.0
            } else {
                sum / self.boundary_distance_checks as f64
            }
        })
    }

    /// Returns the root-mean-square distance delta across the checked boundaries that had distances.
    pub fn rms_boundary_distance_delta_au(&self) -> Option<f64> {
        self.sum_boundary_distance_delta_au_sq.map(|sum| {
            if self.boundary_distance_checks == 0 {
                0.0
            } else {
                (sum / self.boundary_distance_checks as f64).sqrt()
            }
        })
    }

    /// Returns a compact one-line summary of the body inspection envelope.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: {} segments, span days={:.12}..{:.12} (mean {:.12}), {} → {}, {} samples, {} boundary checks, {} residual-bearing segments, mean boundary Δlon={:.12}°, rms boundary Δlon={:.12}°, mean boundary Δlat={:.12}°, rms boundary Δlat={:.12}°, mean boundary Δdist={}, rms boundary Δdist={}, max boundary Δlon={:.12}°, Δlat={:.12}°, Δdist={}",
            self.body,
            self.segment_count,
            self.min_segment_span_days,
            self.max_segment_span_days,
            self.mean_segment_span_days,
            self.earliest,
            self.latest,
            self.sample_count,
            self.boundary_checks,
            self.residual_segment_count,
            self.mean_boundary_longitude_delta_deg(),
            self.rms_boundary_longitude_delta_deg(),
            self.mean_boundary_latitude_delta_deg(),
            self.rms_boundary_latitude_delta_deg(),
            self.mean_boundary_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.rms_boundary_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.max_boundary_longitude_delta_deg,
            self.max_boundary_latitude_delta_deg,
            self.max_boundary_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
        )
    }
}

impl fmt::Display for ArtifactBodyInspection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Aggregate boundary continuity envelope for the packaged artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct ArtifactBoundaryEnvelopeSummary {
    /// Number of bundled bodies contributing boundary checks.
    pub body_count: usize,
    /// Total number of boundary checks across all bundled bodies.
    pub boundary_check_count: usize,
    /// Sum of longitude deltas across all boundary checks.
    pub sum_boundary_longitude_delta_deg: f64,
    /// Sum of squared longitude deltas across all boundary checks.
    pub sum_boundary_longitude_delta_deg_sq: f64,
    /// Sum of latitude deltas across all boundary checks.
    pub sum_boundary_latitude_delta_deg: f64,
    /// Sum of squared latitude deltas across all boundary checks.
    pub sum_boundary_latitude_delta_deg_sq: f64,
    /// Sum of distance deltas across all boundary checks that had distances.
    pub sum_boundary_distance_delta_au: Option<f64>,
    /// Sum of squared distance deltas across all boundary checks that had distances.
    pub sum_boundary_distance_delta_au_sq: Option<f64>,
    /// Number of boundary checks that had a distance delta.
    pub boundary_distance_check_count: usize,
    /// Body that produced the largest longitude delta.
    pub max_boundary_longitude_delta_body: Option<CelestialBody>,
    /// Maximum longitude delta observed at a shared boundary.
    pub max_boundary_longitude_delta_deg: f64,
    /// Body that produced the largest latitude delta.
    pub max_boundary_latitude_delta_body: Option<CelestialBody>,
    /// Maximum latitude delta observed at a shared boundary.
    pub max_boundary_latitude_delta_deg: f64,
    /// Body that produced the largest distance delta.
    pub max_boundary_distance_delta_body: Option<CelestialBody>,
    /// Maximum distance delta observed at a shared boundary.
    pub max_boundary_distance_delta_au: Option<f64>,
}

/// Errors returned when a packaged-artifact boundary continuity summary is internally inconsistent.
#[derive(Clone, Debug, PartialEq)]
pub enum ArtifactBoundaryEnvelopeSummaryValidationError {
    /// A stored numeric field was not finite.
    NonFiniteValue {
        /// Field name for the offending value.
        field: &'static str,
        /// Offending value.
        value: f64,
    },
    /// The distance-channel counters or aggregates disagreed with each other.
    InconsistentDistanceCoverage {
        /// Number of checks with a distance channel.
        boundary_distance_check_count: usize,
        /// Whether the summed distance channel is present.
        has_sum: bool,
        /// Whether the squared distance channel is present.
        has_sum_sq: bool,
        /// Whether a maximum distance delta is present.
        has_max: bool,
    },
    /// The zero-check summary still carried data that should have been empty.
    UnexpectedDataForEmptySummary {
        /// Field that should have stayed empty or zero.
        field: &'static str,
    },
    /// A non-empty summary was missing the body label that should identify the maximum longitude delta.
    MissingLongitudeBody,
    /// A non-empty summary was missing the body label that should identify the maximum latitude delta.
    MissingLatitudeBody,
    /// A non-empty distance-channel summary was missing the body label that should identify the maximum distance delta.
    MissingDistanceBody,
}

impl ArtifactBoundaryEnvelopeSummaryValidationError {
    /// Returns a compact one-line rendering of the validation failure.
    pub fn summary_line(&self) -> String {
        match self {
            Self::NonFiniteValue { field, value } => {
                format!("artifact boundary summary field `{field}` must be finite, got {value}")
            }
            Self::InconsistentDistanceCoverage {
                boundary_distance_check_count,
                has_sum,
                has_sum_sq,
                has_max,
            } => format!(
                "artifact boundary summary distance coverage is inconsistent: {boundary_distance_check_count} distance checks, sum={has_sum}, squared_sum={has_sum_sq}, max={has_max}"
            ),
            Self::UnexpectedDataForEmptySummary { field } => {
                format!("artifact boundary summary field `{field}` must be empty when there are no boundary checks")
            }
            Self::MissingLongitudeBody => {
                "artifact boundary summary is missing the maximum-longitude body label".to_string()
            }
            Self::MissingLatitudeBody => {
                "artifact boundary summary is missing the maximum-latitude body label".to_string()
            }
            Self::MissingDistanceBody => {
                "artifact boundary summary is missing the maximum-distance body label".to_string()
            }
        }
    }
}

impl fmt::Display for ArtifactBoundaryEnvelopeSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for ArtifactBoundaryEnvelopeSummaryValidationError {}

impl ArtifactBoundaryEnvelopeSummary {
    /// Returns the mean longitude delta across all checked boundaries.
    pub fn mean_boundary_longitude_delta_deg(&self) -> f64 {
        if self.boundary_check_count == 0 {
            0.0
        } else {
            self.sum_boundary_longitude_delta_deg / self.boundary_check_count as f64
        }
    }

    /// Returns the root-mean-square longitude delta across all checked boundaries.
    pub fn rms_boundary_longitude_delta_deg(&self) -> f64 {
        if self.boundary_check_count == 0 {
            0.0
        } else {
            (self.sum_boundary_longitude_delta_deg_sq / self.boundary_check_count as f64).sqrt()
        }
    }

    /// Returns the mean latitude delta across all checked boundaries.
    pub fn mean_boundary_latitude_delta_deg(&self) -> f64 {
        if self.boundary_check_count == 0 {
            0.0
        } else {
            self.sum_boundary_latitude_delta_deg / self.boundary_check_count as f64
        }
    }

    /// Returns the root-mean-square latitude delta across all checked boundaries.
    pub fn rms_boundary_latitude_delta_deg(&self) -> f64 {
        if self.boundary_check_count == 0 {
            0.0
        } else {
            (self.sum_boundary_latitude_delta_deg_sq / self.boundary_check_count as f64).sqrt()
        }
    }

    /// Returns the mean distance delta across all checked boundaries with a distance channel.
    pub fn mean_boundary_distance_delta_au(&self) -> Option<f64> {
        self.sum_boundary_distance_delta_au.map(|sum| {
            if self.boundary_distance_check_count == 0 {
                0.0
            } else {
                sum / self.boundary_distance_check_count as f64
            }
        })
    }

    /// Returns the root-mean-square distance delta across all checked boundaries with a distance channel.
    pub fn rms_boundary_distance_delta_au(&self) -> Option<f64> {
        self.sum_boundary_distance_delta_au_sq.map(|sum| {
            if self.boundary_distance_check_count == 0 {
                0.0
            } else {
                (sum / self.boundary_distance_check_count as f64).sqrt()
            }
        })
    }

    /// Validates the stored summary invariants before it is formatted or reused by release reports.
    pub fn validate(&self) -> Result<(), ArtifactBoundaryEnvelopeSummaryValidationError> {
        for (field, value) in [
            (
                "sum_boundary_longitude_delta_deg",
                self.sum_boundary_longitude_delta_deg,
            ),
            (
                "sum_boundary_longitude_delta_deg_sq",
                self.sum_boundary_longitude_delta_deg_sq,
            ),
            (
                "sum_boundary_latitude_delta_deg",
                self.sum_boundary_latitude_delta_deg,
            ),
            (
                "sum_boundary_latitude_delta_deg_sq",
                self.sum_boundary_latitude_delta_deg_sq,
            ),
            (
                "max_boundary_longitude_delta_deg",
                self.max_boundary_longitude_delta_deg,
            ),
            (
                "max_boundary_latitude_delta_deg",
                self.max_boundary_latitude_delta_deg,
            ),
        ] {
            if !value.is_finite() {
                return Err(
                    ArtifactBoundaryEnvelopeSummaryValidationError::NonFiniteValue { field, value },
                );
            }
        }

        if let Some(value) = self.sum_boundary_distance_delta_au {
            if !value.is_finite() {
                return Err(
                    ArtifactBoundaryEnvelopeSummaryValidationError::NonFiniteValue {
                        field: "sum_boundary_distance_delta_au",
                        value,
                    },
                );
            }
        }
        if let Some(value) = self.sum_boundary_distance_delta_au_sq {
            if !value.is_finite() {
                return Err(
                    ArtifactBoundaryEnvelopeSummaryValidationError::NonFiniteValue {
                        field: "sum_boundary_distance_delta_au_sq",
                        value,
                    },
                );
            }
        }
        if let Some(value) = self.max_boundary_distance_delta_au {
            if !value.is_finite() {
                return Err(
                    ArtifactBoundaryEnvelopeSummaryValidationError::NonFiniteValue {
                        field: "max_boundary_distance_delta_au",
                        value,
                    },
                );
            }
        }

        if self.boundary_distance_check_count > self.boundary_check_count {
            return Err(
                ArtifactBoundaryEnvelopeSummaryValidationError::InconsistentDistanceCoverage {
                    boundary_distance_check_count: self.boundary_distance_check_count,
                    has_sum: self.sum_boundary_distance_delta_au.is_some(),
                    has_sum_sq: self.sum_boundary_distance_delta_au_sq.is_some(),
                    has_max: self.max_boundary_distance_delta_au.is_some(),
                },
            );
        }

        match self.boundary_check_count {
            0 => {
                if self.sum_boundary_longitude_delta_deg != 0.0 {
                    return Err(ArtifactBoundaryEnvelopeSummaryValidationError::UnexpectedDataForEmptySummary {
                        field: "sum_boundary_longitude_delta_deg",
                    });
                }
                if self.sum_boundary_longitude_delta_deg_sq != 0.0 {
                    return Err(ArtifactBoundaryEnvelopeSummaryValidationError::UnexpectedDataForEmptySummary {
                        field: "sum_boundary_longitude_delta_deg_sq",
                    });
                }
                if self.sum_boundary_latitude_delta_deg != 0.0 {
                    return Err(ArtifactBoundaryEnvelopeSummaryValidationError::UnexpectedDataForEmptySummary {
                        field: "sum_boundary_latitude_delta_deg",
                    });
                }
                if self.sum_boundary_latitude_delta_deg_sq != 0.0 {
                    return Err(ArtifactBoundaryEnvelopeSummaryValidationError::UnexpectedDataForEmptySummary {
                        field: "sum_boundary_latitude_delta_deg_sq",
                    });
                }
                if self.boundary_distance_check_count != 0
                    || self.sum_boundary_distance_delta_au.is_some()
                    || self.sum_boundary_distance_delta_au_sq.is_some()
                    || self.max_boundary_distance_delta_au.is_some()
                {
                    return Err(ArtifactBoundaryEnvelopeSummaryValidationError::UnexpectedDataForEmptySummary {
                        field: "distance boundary data",
                    });
                }
                if self.max_boundary_longitude_delta_body.is_some() {
                    return Err(ArtifactBoundaryEnvelopeSummaryValidationError::UnexpectedDataForEmptySummary {
                        field: "max_boundary_longitude_delta_body",
                    });
                }
                if self.max_boundary_latitude_delta_body.is_some() {
                    return Err(ArtifactBoundaryEnvelopeSummaryValidationError::UnexpectedDataForEmptySummary {
                        field: "max_boundary_latitude_delta_body",
                    });
                }
            }
            _ => {
                if self.max_boundary_longitude_delta_body.is_none() {
                    return Err(
                        ArtifactBoundaryEnvelopeSummaryValidationError::MissingLongitudeBody,
                    );
                }
                if self.max_boundary_latitude_delta_body.is_none() {
                    return Err(
                        ArtifactBoundaryEnvelopeSummaryValidationError::MissingLatitudeBody,
                    );
                }
                match (
                    self.boundary_distance_check_count,
                    self.sum_boundary_distance_delta_au,
                    self.sum_boundary_distance_delta_au_sq,
                    self.max_boundary_distance_delta_au,
                ) {
                    (0, None, None, None) => {}
                    (0, _, _, _) => {
                        return Err(ArtifactBoundaryEnvelopeSummaryValidationError::InconsistentDistanceCoverage {
                            boundary_distance_check_count: 0,
                            has_sum: self.sum_boundary_distance_delta_au.is_some(),
                            has_sum_sq: self.sum_boundary_distance_delta_au_sq.is_some(),
                            has_max: self.max_boundary_distance_delta_au.is_some(),
                        });
                    }
                    (_, Some(_), Some(_), Some(_)) => {}
                    (count, has_sum, has_sum_sq, has_max) => {
                        return Err(ArtifactBoundaryEnvelopeSummaryValidationError::InconsistentDistanceCoverage {
                            boundary_distance_check_count: count,
                            has_sum: has_sum.is_some(),
                            has_sum_sq: has_sum_sq.is_some(),
                            has_max: has_max.is_some(),
                        });
                    }
                }
                if self.boundary_distance_check_count > 0
                    && self.max_boundary_distance_delta_body.is_none()
                {
                    return Err(
                        ArtifactBoundaryEnvelopeSummaryValidationError::MissingDistanceBody,
                    );
                }
                if self.boundary_distance_check_count == 0
                    && self.max_boundary_distance_delta_body.is_some()
                {
                    return Err(ArtifactBoundaryEnvelopeSummaryValidationError::UnexpectedDataForEmptySummary {
                        field: "max_boundary_distance_delta_body",
                    });
                }
            }
        }

        Ok(())
    }

    /// Returns the aggregate boundary envelope as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        if self.boundary_check_count == 0 {
            return format!(
                "Artifact boundary envelope: 0 checks across {} bundled bodies",
                self.body_count
            );
        }

        format!(
            "Artifact boundary envelope: {} checks across {} bundled bodies, mean boundary Δlon={:.12}°, rms boundary Δlon={:.12}°, mean boundary Δlat={:.12}°, rms boundary Δlat={:.12}°, mean boundary Δdist={}{}, rms boundary Δdist={}{}, max boundary Δlon={:.12}°{}, max boundary Δlat={:.12}°{}, max boundary Δdist={}{}",
            self.boundary_check_count,
            self.body_count,
            self.mean_boundary_longitude_delta_deg(),
            self.rms_boundary_longitude_delta_deg(),
            self.mean_boundary_latitude_delta_deg(),
            self.rms_boundary_latitude_delta_deg(),
            self.mean_boundary_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            if self.boundary_distance_check_count > 0 {
                format!(" ({} distance checks)", self.boundary_distance_check_count)
            } else {
                String::new()
            },
            self.rms_boundary_distance_delta_au()
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            if self.boundary_distance_check_count > 0 {
                format!(" ({} distance checks)", self.boundary_distance_check_count)
            } else {
                String::new()
            },
            self.max_boundary_longitude_delta_deg,
            self.max_boundary_longitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            self.max_boundary_latitude_delta_deg,
            self.max_boundary_latitude_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
            self.max_boundary_distance_delta_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.max_boundary_distance_delta_body
                .as_ref()
                .map(|body| format!(" ({body})"))
                .unwrap_or_default(),
        )
    }

    /// Returns the validated aggregate boundary envelope as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ArtifactBoundaryEnvelopeSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for ArtifactBoundaryEnvelopeSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Benchmark summary for decoding the packaged compressed artifact.
#[derive(Clone, Debug)]
pub struct ArtifactDecodeBenchmarkReport {
    /// Human-readable label from the artifact header.
    pub artifact_label: String,
    /// Source/provenance summary from the artifact header.
    pub source: String,
    /// Number of benchmark rounds.
    pub rounds: usize,
    /// Number of artifact decodes per round.
    pub sample_count: usize,
    /// Size of the encoded artifact in bytes.
    pub encoded_bytes: usize,
    /// Total elapsed time for the decode path.
    pub elapsed: std::time::Duration,
}

/// Errors returned when a packaged-artifact decode benchmark report is
/// internally inconsistent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactDecodeBenchmarkReportValidationError {
    /// The artifact label was blank.
    BlankArtifactLabel,
    /// The source/provenance summary was blank.
    BlankSource,
    /// The benchmark was configured with zero rounds.
    ZeroRounds,
    /// The benchmark was configured with zero decodes per round.
    ZeroSampleCount,
    /// The encoded artifact size was zero bytes.
    ZeroEncodedBytes,
}

impl ArtifactDecodeBenchmarkReportValidationError {
    /// Returns the stable summary label for the validation failure.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::BlankArtifactLabel => "blank artifact label",
            Self::BlankSource => "blank source",
            Self::ZeroRounds => "zero rounds",
            Self::ZeroSampleCount => "zero decodes per round",
            Self::ZeroEncodedBytes => "zero encoded bytes",
        }
    }
}

impl fmt::Display for ArtifactDecodeBenchmarkReportValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl std::error::Error for ArtifactDecodeBenchmarkReportValidationError {}

/// Errors returned when a packaged-artifact lookup benchmark report is
/// internally inconsistent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactLookupBenchmarkReportValidationError {
    /// The artifact label was blank.
    BlankArtifactLabel,
    /// The source/provenance summary was blank.
    BlankSource,
    /// The benchmark corpus name was blank.
    BlankCorpusName,
    /// The benchmark was configured with zero rounds.
    ZeroRounds,
    /// The benchmark was configured with zero lookups per round.
    ZeroSampleCount,
    /// The encoded artifact size was zero bytes.
    ZeroEncodedBytes,
}

impl ArtifactLookupBenchmarkReportValidationError {
    /// Returns the stable summary label for the validation failure.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::BlankArtifactLabel => "blank artifact label",
            Self::BlankSource => "blank source",
            Self::BlankCorpusName => "blank corpus name",
            Self::ZeroRounds => "zero rounds",
            Self::ZeroSampleCount => "zero lookups per round",
            Self::ZeroEncodedBytes => "zero encoded bytes",
        }
    }
}

impl fmt::Display for ArtifactLookupBenchmarkReportValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl std::error::Error for ArtifactLookupBenchmarkReportValidationError {}

impl ArtifactDecodeBenchmarkReport {
    /// Returns the average number of nanoseconds per artifact decode.
    pub fn nanoseconds_per_decode(&self) -> f64 {
        let total_decodes = self.rounds as f64 * self.sample_count as f64;
        if total_decodes == 0.0 {
            return 0.0;
        }

        self.elapsed.as_secs_f64() * 1_000_000_000.0 / total_decodes
    }

    /// Returns the average throughput in artifact decodes per second.
    pub fn decodes_per_second(&self) -> f64 {
        let total_decodes = self.rounds as f64 * self.sample_count as f64;
        if self.elapsed.is_zero() || total_decodes == 0.0 {
            return 0.0;
        }

        total_decodes / self.elapsed.as_secs_f64()
    }

    /// Validates the decoded benchmark metadata before the report is formatted.
    pub fn validate(&self) -> Result<(), ArtifactDecodeBenchmarkReportValidationError> {
        if self.artifact_label.trim().is_empty() {
            return Err(ArtifactDecodeBenchmarkReportValidationError::BlankArtifactLabel);
        }
        if self.source.trim().is_empty() {
            return Err(ArtifactDecodeBenchmarkReportValidationError::BlankSource);
        }
        if self.rounds == 0 {
            return Err(ArtifactDecodeBenchmarkReportValidationError::ZeroRounds);
        }
        if self.sample_count == 0 {
            return Err(ArtifactDecodeBenchmarkReportValidationError::ZeroSampleCount);
        }
        if self.encoded_bytes == 0 {
            return Err(ArtifactDecodeBenchmarkReportValidationError::ZeroEncodedBytes);
        }

        Ok(())
    }

    /// Validates the benchmark metadata before returning the compact summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ArtifactDecodeBenchmarkReportValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns a compact one-line summary of the packaged-artifact decode benchmark.
    pub fn summary_line(&self) -> String {
        format!(
            "artifact={}; source={}; rounds={}; decodes per round={}; encoded bytes={}; ns/decode={:.2}; decodes/s={:.2}",
            self.artifact_label,
            self.source,
            self.rounds,
            self.sample_count,
            self.encoded_bytes,
            self.nanoseconds_per_decode(),
            self.decodes_per_second(),
        )
    }
}

impl fmt::Display for ArtifactDecodeBenchmarkReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Artifact decode benchmark report")?;
        writeln!(f, "Artifact: {}", self.artifact_label)?;
        writeln!(f, "Source: {}", self.source)?;
        writeln!(f, "Rounds: {}", self.rounds)?;
        writeln!(f, "Decodes per round: {}", self.sample_count)?;
        writeln!(f, "Encoded bytes: {}", self.encoded_bytes)?;
        writeln!(
            f,
            "Decode elapsed: {}",
            super::format_duration(self.elapsed)
        )?;
        writeln!(
            f,
            "Nanoseconds per decode: {:.2}",
            self.nanoseconds_per_decode()
        )?;
        writeln!(f, "Decodes per second: {:.2}", self.decodes_per_second())
    }
}

/// Benchmark summary for lookup performance against the packaged compressed artifact.
#[derive(Clone, Debug)]
pub struct ArtifactLookupBenchmarkReport {
    /// Human-readable label from the artifact header.
    pub artifact_label: String,
    /// Source/provenance summary from the artifact header.
    pub source: String,
    /// Corpus name used for the benchmark.
    pub corpus_name: String,
    /// Number of benchmark rounds.
    pub rounds: usize,
    /// Number of lookups per round.
    pub sample_count: usize,
    /// Size of the encoded artifact in bytes.
    pub encoded_bytes: usize,
    /// Total elapsed time for the lookup path.
    pub elapsed: std::time::Duration,
}

impl ArtifactLookupBenchmarkReport {
    /// Returns the average number of nanoseconds per artifact lookup.
    pub fn nanoseconds_per_lookup(&self) -> f64 {
        let total_lookups = self.rounds as f64 * self.sample_count as f64;
        if total_lookups == 0.0 {
            return 0.0;
        }

        self.elapsed.as_secs_f64() * 1_000_000_000.0 / total_lookups
    }

    /// Returns the average throughput in artifact lookups per second.
    pub fn lookups_per_second(&self) -> f64 {
        let total_lookups = self.rounds as f64 * self.sample_count as f64;
        if self.elapsed.is_zero() || total_lookups == 0.0 {
            return 0.0;
        }

        total_lookups / self.elapsed.as_secs_f64()
    }

    /// Validates the lookup benchmark metadata before the report is formatted.
    pub fn validate(&self) -> Result<(), ArtifactLookupBenchmarkReportValidationError> {
        if self.artifact_label.trim().is_empty() {
            return Err(ArtifactLookupBenchmarkReportValidationError::BlankArtifactLabel);
        }
        if self.source.trim().is_empty() {
            return Err(ArtifactLookupBenchmarkReportValidationError::BlankSource);
        }
        if self.corpus_name.trim().is_empty() {
            return Err(ArtifactLookupBenchmarkReportValidationError::BlankCorpusName);
        }
        if self.rounds == 0 {
            return Err(ArtifactLookupBenchmarkReportValidationError::ZeroRounds);
        }
        if self.sample_count == 0 {
            return Err(ArtifactLookupBenchmarkReportValidationError::ZeroSampleCount);
        }
        if self.encoded_bytes == 0 {
            return Err(ArtifactLookupBenchmarkReportValidationError::ZeroEncodedBytes);
        }

        Ok(())
    }

    /// Validates the benchmark metadata before returning the compact summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ArtifactLookupBenchmarkReportValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns a compact one-line summary of the packaged-artifact lookup benchmark.
    pub fn summary_line(&self) -> String {
        format!(
            "artifact={}; source={}; corpus={}; rounds={}; lookups per round={}; encoded bytes={}; ns/lookup={:.2}; lookups/s={:.2}",
            self.artifact_label,
            self.source,
            self.corpus_name,
            self.rounds,
            self.sample_count,
            self.encoded_bytes,
            self.nanoseconds_per_lookup(),
            self.lookups_per_second(),
        )
    }
}

impl fmt::Display for ArtifactLookupBenchmarkReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Artifact lookup benchmark report")?;
        writeln!(f, "Artifact: {}", self.artifact_label)?;
        writeln!(f, "Source: {}", self.source)?;
        writeln!(f, "Corpus: {}", self.corpus_name)?;
        writeln!(f, "Rounds: {}", self.rounds)?;
        writeln!(f, "Lookups per round: {}", self.sample_count)?;
        writeln!(f, "Encoded bytes: {}", self.encoded_bytes)?;
        writeln!(
            f,
            "Lookup elapsed: {}",
            super::format_duration(self.elapsed)
        )?;
        writeln!(
            f,
            "Nanoseconds per lookup: {:.2}",
            self.nanoseconds_per_lookup()
        )?;
        writeln!(f, "Lookups per second: {:.2}", self.lookups_per_second())
    }
}

/// Benchmark summary for batch lookup performance against the packaged compressed artifact.
#[derive(Clone, Debug)]
pub struct ArtifactBatchLookupBenchmarkReport {
    /// Human-readable label from the artifact header.
    pub artifact_label: String,
    /// Source/provenance summary from the artifact header.
    pub source: String,
    /// Corpus name used for the benchmark.
    pub corpus_name: String,
    /// Number of benchmark rounds.
    pub rounds: usize,
    /// Number of lookups per batch.
    pub batch_size: usize,
    /// Size of the encoded artifact in bytes.
    pub encoded_bytes: usize,
    /// Total elapsed time for the batch lookup path.
    pub elapsed: std::time::Duration,
}

/// Errors returned when a packaged-artifact batch lookup benchmark report is
/// internally inconsistent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactBatchLookupBenchmarkReportValidationError {
    /// The artifact label was blank.
    BlankArtifactLabel,
    /// The source/provenance summary was blank.
    BlankSource,
    /// The benchmark corpus name was blank.
    BlankCorpusName,
    /// The benchmark was configured with zero rounds.
    ZeroRounds,
    /// The benchmark was configured with zero lookups per batch.
    ZeroBatchSize,
    /// The encoded artifact size was zero bytes.
    ZeroEncodedBytes,
}

impl ArtifactBatchLookupBenchmarkReportValidationError {
    /// Returns the stable summary label for the validation failure.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::BlankArtifactLabel => "blank artifact label",
            Self::BlankSource => "blank source",
            Self::BlankCorpusName => "blank corpus name",
            Self::ZeroRounds => "zero rounds",
            Self::ZeroBatchSize => "zero lookups per batch",
            Self::ZeroEncodedBytes => "zero encoded bytes",
        }
    }
}

impl fmt::Display for ArtifactBatchLookupBenchmarkReportValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl std::error::Error for ArtifactBatchLookupBenchmarkReportValidationError {}

impl ArtifactBatchLookupBenchmarkReport {
    /// Returns the average number of nanoseconds per artifact lookup.
    pub fn nanoseconds_per_lookup(&self) -> f64 {
        let total_lookups = self.rounds as f64 * self.batch_size as f64;
        if total_lookups == 0.0 {
            return 0.0;
        }

        self.elapsed.as_secs_f64() * 1_000_000_000.0 / total_lookups
    }

    /// Returns the average throughput in artifact lookups per second.
    pub fn lookups_per_second(&self) -> f64 {
        let total_lookups = self.rounds as f64 * self.batch_size as f64;
        if self.elapsed.is_zero() || total_lookups == 0.0 {
            return 0.0;
        }

        total_lookups / self.elapsed.as_secs_f64()
    }

    /// Validates the batch lookup benchmark metadata before the report is formatted.
    pub fn validate(&self) -> Result<(), ArtifactBatchLookupBenchmarkReportValidationError> {
        if self.artifact_label.trim().is_empty() {
            return Err(ArtifactBatchLookupBenchmarkReportValidationError::BlankArtifactLabel);
        }
        if self.source.trim().is_empty() {
            return Err(ArtifactBatchLookupBenchmarkReportValidationError::BlankSource);
        }
        if self.corpus_name.trim().is_empty() {
            return Err(ArtifactBatchLookupBenchmarkReportValidationError::BlankCorpusName);
        }
        if self.rounds == 0 {
            return Err(ArtifactBatchLookupBenchmarkReportValidationError::ZeroRounds);
        }
        if self.batch_size == 0 {
            return Err(ArtifactBatchLookupBenchmarkReportValidationError::ZeroBatchSize);
        }
        if self.encoded_bytes == 0 {
            return Err(ArtifactBatchLookupBenchmarkReportValidationError::ZeroEncodedBytes);
        }

        Ok(())
    }

    /// Validates the benchmark metadata before returning the compact summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, ArtifactBatchLookupBenchmarkReportValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns a compact one-line summary of the packaged-artifact batch lookup benchmark.
    pub fn summary_line(&self) -> String {
        format!(
            "artifact={}; source={}; corpus={}; rounds={}; lookups per batch={}; encoded bytes={}; ns/lookup={:.2}; lookups/s={:.2}",
            self.artifact_label,
            self.source,
            self.corpus_name,
            self.rounds,
            self.batch_size,
            self.encoded_bytes,
            self.nanoseconds_per_lookup(),
            self.lookups_per_second(),
        )
    }
}

impl fmt::Display for ArtifactBatchLookupBenchmarkReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Artifact batch lookup benchmark report")?;
        writeln!(f, "Artifact: {}", self.artifact_label)?;
        writeln!(f, "Source: {}", self.source)?;
        writeln!(f, "Corpus: {}", self.corpus_name)?;
        writeln!(f, "Rounds: {}", self.rounds)?;
        writeln!(f, "Lookups per batch: {}", self.batch_size)?;
        writeln!(f, "Encoded bytes: {}", self.encoded_bytes)?;
        writeln!(
            f,
            "Batch lookup elapsed: {}",
            super::format_duration(self.elapsed)
        )?;
        writeln!(
            f,
            "Nanoseconds per lookup: {:.2}",
            self.nanoseconds_per_lookup()
        )?;
        writeln!(f, "Lookups per second: {:.2}", self.lookups_per_second())
    }
}

fn packaged_artifact_encoded_bytes() -> &'static [u8] {
    static ENCODED: OnceLock<Vec<u8>> = OnceLock::new();
    ENCODED.get_or_init(|| {
        packaged_artifact()
            .encode()
            .expect("packaged artifact should encode")
    })
}

fn packaged_artifact_inspection_report() -> &'static ArtifactInspectionReport {
    static REPORT: OnceLock<ArtifactInspectionReport> = OnceLock::new();
    REPORT.get_or_init(|| {
        let artifact = packaged_artifact();
        let encoded = packaged_artifact_encoded_bytes();
        ArtifactInspectionReport::from_encoded_artifact(artifact, encoded, encoded.len())
            .expect("packaged artifact inspection report should build")
    })
}

impl ArtifactInspectionReport {
    #[cfg(test)]
    fn from_artifact(
        artifact: &CompressedArtifact,
        encoded_bytes: usize,
    ) -> Result<Self, ArtifactInspectionError> {
        let encoded = artifact.encode()?;
        Self::from_encoded_artifact(artifact, &encoded, encoded_bytes)
    }

    fn from_encoded_artifact(
        artifact: &CompressedArtifact,
        encoded: &[u8],
        encoded_bytes: usize,
    ) -> Result<Self, ArtifactInspectionError> {
        let decoded = CompressedArtifact::decode(encoded)?;
        let mut bodies = Vec::with_capacity(decoded.bodies.len());
        let mut segment_count = 0usize;
        let mut earliest: Option<Instant> = None;
        let mut latest: Option<Instant> = None;

        for body in &decoded.bodies {
            let inspection = inspect_body(&decoded, body)?;
            segment_count += inspection.segment_count;
            earliest = Some(match earliest {
                Some(current)
                    if current.julian_day.days() <= inspection.earliest.julian_day.days() =>
                {
                    current
                }
                Some(_) => inspection.earliest,
                None => inspection.earliest,
            });
            latest = Some(match latest {
                Some(current)
                    if current.julian_day.days() >= inspection.latest.julian_day.days() =>
                {
                    current
                }
                Some(_) => inspection.latest,
                None => inspection.latest,
            });
            bodies.push(inspection);
        }

        let comparison_corpus = artifact_model_comparison_corpus(&decoded);
        let model_comparison = compare_backends(
            &default_candidate_backend(),
            &packaged_backend(),
            &comparison_corpus,
        )?;
        let decode_benchmark = benchmark_packaged_artifact_decode(1)?;
        let lookup_benchmark = benchmark_packaged_artifact_lookup(1)?;
        let batch_lookup_benchmark = benchmark_packaged_artifact_batch_lookup(1)?;
        let residual_segment_count = decoded.residual_segment_count();
        let residual_bodies = decoded.residual_bodies();

        let report = Self {
            generation_label: decoded.header.generation_label,
            source: decoded.header.source,
            version: decoded.header.version,
            endian_policy: decoded.header.endian_policy,
            checksum: decoded.checksum,
            encoded_bytes,
            roundtrip_ok: true,
            checksum_ok: decoded.checksum == artifact.checksum,
            body_count: decoded.bodies.len(),
            segment_count,
            residual_segment_count,
            residual_bodies,
            earliest: earliest.unwrap_or_else(|| artifact_first_instant(artifact)),
            latest: latest.unwrap_or_else(|| artifact_first_instant(artifact)),
            model_comparison,
            decode_benchmark,
            lookup_benchmark,
            batch_lookup_benchmark,
            bodies,
        };
        report.validate()?;
        Ok(report)
    }

    /// Returns a compact one-line summary of the inspection report.
    pub fn summary_line(&self) -> String {
        format!(
            "artifact inspection: {} bundled bodies, {} segments, residual-bearing segments: {}, residual-bearing bodies: {}, body classes: {}; coverage: {} → {}, roundtrip={}, checksum={}, encoded bytes={}",
            self.body_count,
            self.segment_count,
            self.residual_segment_count,
            format_residual_bodies(&self.residual_bodies),
            format_body_class_coverage(self),
            self.earliest,
            self.latest,
            yes_no(self.roundtrip_ok),
            yes_no(self.checksum_ok),
            self.encoded_bytes,
        )
    }

    /// Validates the inspection report before returning the compact summary line.
    pub fn validated_summary_line(&self) -> Result<String, ArtifactInspectionError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    pub fn validate(&self) -> Result<(), ArtifactInspectionError> {
        if !self.roundtrip_ok {
            return Err(report_validation_error(
                "artifact inspection report roundtrip status drifted from the decoded artifact",
            ));
        }

        if !self.checksum_ok {
            return Err(report_validation_error(
                "artifact inspection report checksum status drifted from the decoded artifact",
            ));
        }

        if self.body_count != self.bodies.len() {
            return Err(report_validation_error(
                "artifact inspection report field `body_count` does not match the inspected body set",
            ));
        }

        let expected_segment_count: usize = self.bodies.iter().map(|body| body.segment_count).sum();
        if self.segment_count != expected_segment_count {
            return Err(report_validation_error(
                "artifact inspection report field `segment_count` does not match the inspected body set",
            ));
        }

        if self.residual_segment_count == 0 {
            if !self.residual_bodies.is_empty() {
                return Err(report_validation_error(
                    "artifact inspection report field `residual_bodies` must stay empty when no residual segments are present",
                ));
            }
        } else {
            if self.residual_bodies.is_empty() {
                return Err(report_validation_error(
                    "artifact inspection report field `residual_bodies` must name the residual-bearing bodies",
                ));
            }
            if self.residual_segment_count > self.segment_count {
                return Err(report_validation_error(
                    "artifact inspection report field `residual_segment_count` exceeds the total inspected segments",
                ));
            }

            let expected_residual_bodies = self
                .bodies
                .iter()
                .filter(|inspection| inspection.residual_segment_count > 0)
                .map(|inspection| inspection.body.clone())
                .collect::<Vec<_>>();

            if self.residual_bodies != expected_residual_bodies {
                return Err(report_validation_error(
                    "artifact inspection report field `residual_bodies` does not match the inspected residual-bearing body set",
                ));
            }

            let expected_residual_segment_count: usize = self
                .bodies
                .iter()
                .map(|inspection| inspection.residual_segment_count)
                .sum();
            if self.residual_segment_count != expected_residual_segment_count {
                return Err(report_validation_error(
                    "artifact inspection report field `residual_segment_count` does not match the inspected residual-bearing segment count",
                ));
            }
        }

        if self
            .residual_bodies
            .iter()
            .enumerate()
            .any(|(index, body)| self.residual_bodies[..index].contains(body))
        {
            return Err(report_validation_error(
                "artifact inspection report field `residual_bodies` contains duplicate entries",
            ));
        }

        if self.residual_bodies.iter().any(|body| {
            !self
                .bodies
                .iter()
                .any(|inspection| inspection.body == *body)
        }) {
            return Err(report_validation_error(
                "artifact inspection report field `residual_bodies` references a body not present in the inspected artifact",
            ));
        }

        if self.earliest.julian_day.days() > self.latest.julian_day.days() {
            return Err(report_validation_error(
                "artifact inspection report coverage bounds are inverted",
            ));
        }

        self.model_comparison.summary.validate()?;
        self.decode_benchmark.validate()?;
        self.lookup_benchmark.validate()?;
        self.batch_lookup_benchmark.validate()?;

        if self.decode_benchmark.artifact_label != self.generation_label {
            return Err(report_validation_error(
                "artifact inspection report decode benchmark artifact label does not match the decoded artifact header",
            ));
        }

        if self.decode_benchmark.source != self.source {
            return Err(report_validation_error(
                "artifact inspection report decode benchmark source does not match the decoded artifact header",
            ));
        }

        if self.decode_benchmark.encoded_bytes != self.encoded_bytes {
            return Err(report_validation_error(
                "artifact inspection report decode benchmark encoded byte count does not match the decoded artifact",
            ));
        }

        if self.lookup_benchmark.artifact_label != self.generation_label {
            return Err(report_validation_error(
                "artifact inspection report lookup benchmark artifact label does not match the decoded artifact header",
            ));
        }

        if self.lookup_benchmark.source != self.source {
            return Err(report_validation_error(
                "artifact inspection report lookup benchmark source does not match the decoded artifact header",
            ));
        }

        if self.lookup_benchmark.encoded_bytes != self.encoded_bytes {
            return Err(report_validation_error(
                "artifact inspection report lookup benchmark encoded byte count does not match the decoded artifact",
            ));
        }

        if self.batch_lookup_benchmark.artifact_label != self.generation_label {
            return Err(report_validation_error(
                "artifact inspection report batch lookup benchmark artifact label does not match the decoded artifact header",
            ));
        }

        if self.batch_lookup_benchmark.source != self.source {
            return Err(report_validation_error(
                "artifact inspection report batch lookup benchmark source does not match the decoded artifact header",
            ));
        }

        if self.batch_lookup_benchmark.encoded_bytes != self.encoded_bytes {
            return Err(report_validation_error(
                "artifact inspection report batch lookup benchmark encoded byte count does not match the decoded artifact",
            ));
        }

        artifact_boundary_envelope_summary(self)
            .validate()
            .map_err(ArtifactInspectionError::BoundaryEnvelope)?;
        Ok(())
    }
}

/// Renders the bundled artifact validation report.
pub fn render_artifact_report() -> Result<String, ArtifactInspectionError> {
    Ok(packaged_artifact_inspection_report().to_string())
}

/// Returns the aggregate packaged-artifact boundary envelope used by reports.
pub fn artifact_boundary_envelope_summary_for_report(
) -> Result<ArtifactBoundaryEnvelopeSummary, ArtifactInspectionError> {
    let report = packaged_artifact_inspection_report();
    let summary = artifact_boundary_envelope_summary(report);
    summary
        .validate()
        .map_err(ArtifactInspectionError::BoundaryEnvelope)?;
    Ok(summary)
}

/// Renders a compact summary of the bundled artifact validation report.
pub fn render_artifact_summary() -> Result<String, ArtifactInspectionError> {
    Ok(render_artifact_summary_text(
        packaged_artifact_inspection_report(),
    ))
}

/// Returns the compact artifact inspection summary used by release-facing reports.
pub fn artifact_inspection_summary_for_report() -> Result<String, ArtifactInspectionError> {
    packaged_artifact_inspection_report().validated_summary_line()
}

pub(crate) fn benchmark_packaged_artifact_decode(
    rounds: usize,
) -> Result<ArtifactDecodeBenchmarkReport, ArtifactInspectionError> {
    let artifact = packaged_artifact();
    let encoded = packaged_artifact_encoded_bytes();
    let start = StdInstant::now();
    for _ in 0..rounds {
        std::hint::black_box(CompressedArtifact::decode(encoded)?);
    }
    let elapsed = start.elapsed();

    let report = ArtifactDecodeBenchmarkReport {
        artifact_label: artifact.header.generation_label.clone(),
        source: artifact.header.source.clone(),
        rounds,
        sample_count: 1,
        encoded_bytes: encoded.len(),
        elapsed,
    };
    report.validate()?;

    Ok(report)
}

pub(crate) fn benchmark_packaged_artifact_lookup(
    rounds: usize,
) -> Result<ArtifactLookupBenchmarkReport, ArtifactInspectionError> {
    let artifact = packaged_artifact();
    let encoded = packaged_artifact_encoded_bytes();
    let corpus = artifact_timing_corpus(artifact);
    let sample_count = corpus.requests.len();
    let start = StdInstant::now();
    for _ in 0..rounds {
        for request in &corpus.requests {
            std::hint::black_box(artifact.lookup_ecliptic(&request.body, request.instant)?);
        }
    }
    let elapsed = start.elapsed();

    let report = ArtifactLookupBenchmarkReport {
        artifact_label: artifact.header.generation_label.clone(),
        source: artifact.header.source.clone(),
        corpus_name: corpus.name,
        rounds,
        sample_count,
        encoded_bytes: encoded.len(),
        elapsed,
    };
    report.validate()?;

    Ok(report)
}

pub(crate) fn benchmark_packaged_artifact_batch_lookup(
    rounds: usize,
) -> Result<ArtifactBatchLookupBenchmarkReport, ArtifactInspectionError> {
    let artifact = packaged_artifact();
    let encoded = packaged_artifact_encoded_bytes();
    let corpus = artifact_timing_corpus(artifact);
    let batch_size = corpus.requests.len();
    let backend = packaged_backend();
    let start = StdInstant::now();
    for _ in 0..rounds {
        let results = backend.positions(&corpus.requests)?;
        std::hint::black_box(results);
    }
    let elapsed = start.elapsed();

    let report = ArtifactBatchLookupBenchmarkReport {
        artifact_label: artifact.header.generation_label.clone(),
        source: artifact.header.source.clone(),
        corpus_name: corpus.name,
        rounds,
        batch_size,
        encoded_bytes: encoded.len(),
        elapsed,
    };
    report.validate()?;

    Ok(report)
}

pub(crate) fn packaged_artifact_corpus() -> ValidationCorpus {
    artifact_comparison_corpus(packaged_artifact())
}

fn artifact_timing_corpus(artifact: &CompressedArtifact) -> ValidationCorpus {
    let mut corpus = artifact_model_comparison_corpus(artifact);
    corpus.name = "Packaged artifact timing subset".to_string();
    corpus.description = "Reduced timing subset of the packaged artifact comparison corpus.";
    corpus.requests.truncate(1);
    corpus
}

fn artifact_model_comparison_corpus(artifact: &CompressedArtifact) -> ValidationCorpus {
    artifact_comparison_corpus_filtered(artifact, |body| !matches!(body, CelestialBody::Custom(_)))
}

fn artifact_comparison_corpus(artifact: &CompressedArtifact) -> ValidationCorpus {
    artifact_comparison_corpus_filtered(artifact, |_| true)
}

fn artifact_comparison_corpus_filtered<F>(
    artifact: &CompressedArtifact,
    include_body: F,
) -> ValidationCorpus
where
    F: Fn(&CelestialBody) -> bool,
{
    let mut requests = Vec::new();

    for body in &artifact.bodies {
        if !include_body(&body.body) {
            continue;
        }

        for segment in &body.segments {
            let midpoint = midpoint(segment.start, segment.end);
            for instant in [segment.start, midpoint, segment.end] {
                requests.push(EphemerisRequest {
                    body: body.body.clone(),
                    instant,
                    observer: None,
                    frame: CoordinateFrame::Ecliptic,
                    zodiac_mode: ZodiacMode::Tropical,
                    apparent: Apparentness::Mean,
                });
            }
        }
    }

    ValidationCorpus {
        name: "Packaged artifact error envelope".to_string(),
        description: "Comparison corpus built from packaged artifact coverage at segment endpoints and midpoints so the bundled data can be measured against the algorithmic baseline.",
        apparentness: Apparentness::Mean,
        requests,
    }
}

fn inspect_body(
    artifact: &CompressedArtifact,
    body: &pleiades_compression::BodyArtifact,
) -> Result<ArtifactBodyInspection, ArtifactInspectionError> {
    let mut sample_count = 0usize;
    let mut boundary_checks = 0usize;
    let mut sum_boundary_longitude_delta_deg = 0.0;
    let mut sum_boundary_longitude_delta_deg_sq = 0.0;
    let mut sum_boundary_latitude_delta_deg = 0.0;
    let mut sum_boundary_latitude_delta_deg_sq = 0.0;
    let mut sum_boundary_distance_delta_au: Option<f64> = None;
    let mut sum_boundary_distance_delta_au_sq: Option<f64> = None;
    let mut boundary_distance_checks = 0usize;
    let mut max_boundary_longitude_delta_deg: f64 = 0.0;
    let mut max_boundary_latitude_delta_deg: f64 = 0.0;
    let mut max_boundary_distance_delta_au: Option<f64> = None;

    for segment in &body.segments {
        let midpoint = midpoint(segment.start, segment.end);
        for instant in [segment.start, midpoint, segment.end] {
            artifact.lookup_ecliptic(&body.body, instant)?;
            sample_count += 1;
        }
    }

    for pair in body.segments.windows(2) {
        let left = artifact.lookup_ecliptic(&body.body, pair[0].end)?;
        let right = artifact.lookup_ecliptic(&body.body, pair[1].start)?;
        let delta = boundary_delta(&left, &right);
        boundary_checks += 1;
        sum_boundary_longitude_delta_deg += delta.longitude_delta_deg;
        sum_boundary_longitude_delta_deg_sq += delta.longitude_delta_deg.powi(2);
        sum_boundary_latitude_delta_deg += delta.latitude_delta_deg;
        sum_boundary_latitude_delta_deg_sq += delta.latitude_delta_deg.powi(2);
        max_boundary_longitude_delta_deg =
            max_boundary_longitude_delta_deg.max(delta.longitude_delta_deg);
        max_boundary_latitude_delta_deg =
            max_boundary_latitude_delta_deg.max(delta.latitude_delta_deg);
        max_boundary_distance_delta_au =
            match (max_boundary_distance_delta_au, delta.distance_delta_au) {
                (Some(current), Some(next)) => Some(current.max(next)),
                (None, Some(next)) => Some(next),
                (current, None) => current,
            };
        if let Some(distance_delta_au) = delta.distance_delta_au {
            boundary_distance_checks += 1;
            sum_boundary_distance_delta_au =
                Some(sum_boundary_distance_delta_au.unwrap_or(0.0) + distance_delta_au);
            sum_boundary_distance_delta_au_sq =
                Some(sum_boundary_distance_delta_au_sq.unwrap_or(0.0) + distance_delta_au.powi(2));
        }
    }

    let earliest = body
        .segments
        .first()
        .map(|segment| segment.start)
        .unwrap_or_else(|| artifact_first_instant(artifact));
    let latest = body
        .segments
        .last()
        .map(|segment| segment.end)
        .unwrap_or_else(|| artifact_first_instant(artifact));
    let mut min_segment_span_days: f64 = f64::INFINITY;
    let mut max_segment_span_days: f64 = 0.0;
    let mut sum_segment_span_days: f64 = 0.0;
    for segment in &body.segments {
        let span_days = segment.end.julian_day.days() - segment.start.julian_day.days();
        min_segment_span_days = min_segment_span_days.min(span_days);
        max_segment_span_days = max_segment_span_days.max(span_days);
        sum_segment_span_days += span_days;
    }
    if min_segment_span_days.is_infinite() {
        min_segment_span_days = 0.0;
    }
    let mean_segment_span_days = if body.segments.is_empty() {
        0.0
    } else {
        sum_segment_span_days / body.segments.len() as f64
    };

    Ok(ArtifactBodyInspection {
        body: body.body.clone(),
        segment_count: body.segments.len(),
        earliest,
        latest,
        sample_count,
        min_segment_span_days,
        max_segment_span_days,
        mean_segment_span_days,
        residual_segment_count: body
            .segments
            .iter()
            .filter(|segment| !segment.residual_channels.is_empty())
            .count(),
        boundary_checks,
        sum_boundary_longitude_delta_deg,
        sum_boundary_longitude_delta_deg_sq,
        sum_boundary_latitude_delta_deg,
        sum_boundary_latitude_delta_deg_sq,
        sum_boundary_distance_delta_au,
        sum_boundary_distance_delta_au_sq,
        boundary_distance_checks,
        max_boundary_longitude_delta_deg,
        max_boundary_latitude_delta_deg,
        max_boundary_distance_delta_au,
    })
}

fn report_validation_error(message: &'static str) -> ArtifactInspectionError {
    ArtifactInspectionError::Validation(EphemerisError::new(
        EphemerisErrorKind::InvalidRequest,
        message,
    ))
}

fn format_residual_bodies(bodies: &[CelestialBody]) -> String {
    if bodies.is_empty() {
        "none".to_string()
    } else {
        join_display(bodies)
    }
}

fn format_body_class_coverage(report: &ArtifactInspectionReport) -> String {
    let mut luminaries = 0usize;
    let mut major_planets = 0usize;
    let mut lunar_points = 0usize;
    let mut built_in_asteroids = 0usize;
    let mut custom_bodies = 0usize;
    let mut other_bodies = 0usize;

    for body in &report.bodies {
        match body.body {
            CelestialBody::Sun | CelestialBody::Moon => luminaries += 1,
            CelestialBody::Mercury
            | CelestialBody::Venus
            | CelestialBody::Mars
            | CelestialBody::Jupiter
            | CelestialBody::Saturn
            | CelestialBody::Uranus
            | CelestialBody::Neptune
            | CelestialBody::Pluto => major_planets += 1,
            CelestialBody::MeanNode
            | CelestialBody::TrueNode
            | CelestialBody::MeanApogee
            | CelestialBody::TrueApogee
            | CelestialBody::MeanPerigee
            | CelestialBody::TruePerigee => lunar_points += 1,
            CelestialBody::Ceres
            | CelestialBody::Pallas
            | CelestialBody::Juno
            | CelestialBody::Vesta => built_in_asteroids += 1,
            CelestialBody::Custom(_) => custom_bodies += 1,
            _ => other_bodies += 1,
        }
    }

    format!(
        "luminaries={luminaries}; major planets={major_planets}; lunar points={lunar_points}; built-in asteroids={built_in_asteroids}; custom bodies={custom_bodies}; other bodies={other_bodies}"
    )
}

fn render_artifact_summary_text(report: &ArtifactInspectionReport) -> String {
    let mut text = String::new();

    text.push_str("Artifact summary\n");
    text.push_str("  label: ");
    text.push_str(&report.generation_label);
    text.push('\n');
    text.push_str("  source: ");
    text.push_str(&report.source);
    text.push('\n');
    text.push_str("  regeneration provenance: ");
    text.push_str(&packaged_artifact_regeneration_summary_for_report());
    text.push('\n');
    text.push_str("  version: ");
    text.push_str(&report.version.to_string());
    text.push('\n');
    text.push_str("  byte order: ");
    text.push_str(report.endian_policy.label());
    text.push('\n');
    text.push_str(&format!("  checksum: 0x{:016x}\n", report.checksum));
    text.push_str("  encoded bytes: ");
    text.push_str(&report.encoded_bytes.to_string());
    text.push('\n');
    text.push_str("  Artifact profile: ");
    text.push_str(&packaged_artifact_profile_summary_with_body_coverage());
    text.push('\n');
    text.push_str("  Artifact output support: ");
    text.push_str(&packaged_artifact_output_support_summary_for_report());
    text.push('\n');
    text.push_str("  Body classes: ");
    text.push_str(&format_body_class_coverage(report));
    text.push('\n');
    text.push_str("  Body-class cadence: ");
    text.push_str(&format_body_class_cadence(report));
    text.push('\n');
    text.push_str("  Body-class span caps: ");
    text.push_str(&format_body_class_span_caps());
    text.push('\n');
    text.push_str("  Production profile skeleton: ");
    text.push_str(&packaged_artifact_production_profile_summary_for_report());
    text.push('\n');
    text.push_str("  Generation manifest: ");
    text.push_str(&packaged_artifact_generation_manifest_for_report());
    text.push('\n');
    text.push_str("  Artifact request policy: ");
    text.push_str(&packaged_request_policy_summary_details().to_string());
    text.push('\n');
    text.push_str("  Artifact storage: ");
    text.push_str(&packaged_artifact_storage_summary_for_report());
    text.push('\n');
    text.push_str("  Packaged frame treatment: ");
    text.push_str(&packaged_frame_treatment_summary_details().to_string());
    text.push('\n');
    text.push_str("  coverage: ");
    text.push_str(&report.earliest.julian_day.to_string());
    text.push_str(" → ");
    text.push_str(&report.latest.julian_day.to_string());
    text.push('\n');
    text.push_str("  bodies: ");
    text.push_str(&report.body_count.to_string());
    text.push_str(" total\n");
    text.push_str("  segments: ");
    text.push_str(&report.segment_count.to_string());
    text.push_str(" total\n");
    text.push_str("  residual-bearing segments: ");
    text.push_str(&report.residual_segment_count.to_string());
    text.push('\n');
    text.push_str("  residual-bearing bodies: ");
    text.push_str(&format_residual_bodies(&report.residual_bodies));
    text.push('\n');
    text.push_str("  ");
    text.push_str(
        &artifact_boundary_envelope_summary(report)
            .validated_summary_line()
            .unwrap_or_else(|error| format!("Artifact boundary envelope: unavailable ({error})")),
    );
    text.push('\n');
    text.push_str("  roundtrip decode: ");
    text.push_str(yes_no(report.roundtrip_ok));
    text.push('\n');
    text.push_str("  checksum verified: ");
    text.push_str(yes_no(report.checksum_ok));
    text.push('\n');
    if report
        .bodies
        .iter()
        .any(|body| matches!(body.body, CelestialBody::Custom(_)))
    {
        text.push_str(
            "  note: custom bodies are included in decode and boundary checks, but omitted from the algorithmic comparison corpus.\n",
        );
    }
    text.push('\n');
    text.push_str("Model error envelope\n");
    text.push_str("  baseline backend: ");
    text.push_str(&report.model_comparison.reference_backend.id.to_string());
    text.push('\n');
    text.push_str("  candidate backend: ");
    text.push_str(&report.model_comparison.candidate_backend.id.to_string());
    text.push('\n');
    text.push_str("  corpus: ");
    text.push_str(&report.model_comparison.corpus_name);
    text.push('\n');
    text.push_str("  samples: ");
    text.push_str(&report.model_comparison.summary.sample_count.to_string());
    text.push('\n');
    text.push_str(&format!(
        "  max longitude delta: {:.12}°\n",
        report.model_comparison.summary.max_longitude_delta_deg
    ));
    text.push_str(&format!(
        "  mean longitude delta: {:.12}°\n",
        report.model_comparison.summary.mean_longitude_delta_deg
    ));
    let median = crate::comparison_median_envelope(&report.model_comparison.samples)
        .expect("median envelope should exist");
    let percentile = crate::comparison_percentile_envelope(&report.model_comparison.samples, 0.95);
    text.push_str(&format!(
        "  median longitude delta: {:.12}°\n",
        median.longitude_delta_deg
    ));
    text.push_str(&format!(
        "  95th percentile longitude delta: {:.12}°\n",
        percentile.longitude_delta_deg
    ));
    text.push_str(&format!(
        "  rms longitude delta: {:.12}°\n",
        report.model_comparison.summary.rms_longitude_delta_deg
    ));
    text.push_str(&format!(
        "  max latitude delta: {:.12}°\n",
        report.model_comparison.summary.max_latitude_delta_deg
    ));
    text.push_str(&format!(
        "  mean latitude delta: {:.12}°\n",
        report.model_comparison.summary.mean_latitude_delta_deg
    ));
    text.push_str(&format!(
        "  median latitude delta: {:.12}°\n",
        median.latitude_delta_deg
    ));
    text.push_str(&format!(
        "  95th percentile latitude delta: {:.12}°\n",
        percentile.latitude_delta_deg
    ));
    text.push_str(&format!(
        "  rms latitude delta: {:.12}°\n",
        report.model_comparison.summary.rms_latitude_delta_deg
    ));
    if let Some(value) = report.model_comparison.summary.max_distance_delta_au {
        text.push_str(&format!("  max distance delta: {:.12} AU\n", value));
    }
    if let Some(value) = report.model_comparison.summary.mean_distance_delta_au {
        text.push_str(&format!("  mean distance delta: {:.12} AU\n", value));
    }
    if let Some(value) = median.distance_delta_au {
        text.push_str(&format!("  median distance delta: {:.12} AU\n", value));
    }
    if let Some(value) = percentile.distance_delta_au {
        text.push_str(&format!(
            "  95th percentile distance delta: {:.12} AU\n",
            value
        ));
    }
    if let Some(value) = report.model_comparison.summary.rms_distance_delta_au {
        text.push_str(&format!("  rms distance delta: {:.12} AU\n", value));
    }
    text.push_str("\nExpected tolerance status\n");
    let tolerance_summaries = report.model_comparison.tolerance_summaries();
    if tolerance_summaries.is_empty() {
        text.push_str("  none\n");
    } else {
        for summary in &tolerance_summaries {
            text.push_str("  ");
            text.push_str(&summary.body.to_string());
            text.push_str(": backend family=");
            text.push_str(backend_family_label(&summary.tolerance.backend_family));
            text.push_str(", profile=");
            text.push_str(summary.tolerance.profile);
            text.push_str(", status=");
            text.push_str(if summary.within_tolerance {
                "within"
            } else {
                "exceeded"
            });
            text.push_str(", limit Δlon≤");
            text.push_str(&format!(
                "{:.6}°",
                summary.tolerance.max_longitude_delta_deg
            ));
            text.push_str(", margin Δlon=");
            text.push_str(&format!("{:+.12}°", summary.longitude_margin_deg));
            text.push_str(", limit Δlat≤");
            text.push_str(&format!("{:.6}°", summary.tolerance.max_latitude_delta_deg));
            text.push_str(", margin Δlat=");
            text.push_str(&format!("{:+.12}°", summary.latitude_margin_deg));
            text.push_str(", limit Δdist=");
            text.push_str(
                &summary
                    .tolerance
                    .max_distance_delta_au
                    .map(|value| format!("{value:.6} AU"))
                    .unwrap_or_else(|| "n/a".to_string()),
            );
            text.push_str(", margin Δdist=");
            text.push_str(
                &summary
                    .distance_margin_au
                    .map(|value| format!("{value:+.12} AU"))
                    .unwrap_or_else(|| "n/a".to_string()),
            );
            text.push('\n');
        }
    }
    let within_tolerance_body_count = tolerance_summaries
        .iter()
        .filter(|summary| summary.within_tolerance)
        .count();
    let outside_tolerance_body_count = tolerance_summaries.len() - within_tolerance_body_count;
    let regression_count = report.model_comparison.notable_regressions().len();
    text.push_str("\nComparison tolerance audit\n");
    text.push_str("  bodies checked: ");
    text.push_str(&tolerance_summaries.len().to_string());
    text.push('\n');
    text.push_str("  within tolerance bodies: ");
    text.push_str(&within_tolerance_body_count.to_string());
    text.push('\n');
    text.push_str("  outside tolerance bodies: ");
    text.push_str(&outside_tolerance_body_count.to_string());
    text.push('\n');
    text.push_str("  notable regressions: ");
    text.push_str(&regression_count.to_string());
    text.push('\n');
    text.push_str("\nArtifact lookup benchmark\n");
    text.push_str("  ");
    text.push_str(
        &report
            .lookup_benchmark
            .validated_summary_line()
            .unwrap_or_else(|error| format!("Artifact lookup benchmark: unavailable ({error})")),
    );
    text.push('\n');
    text.push_str("\nArtifact batch lookup benchmark\n");
    text.push_str("  ");
    text.push_str(
        &report
            .batch_lookup_benchmark
            .validated_summary_line()
            .unwrap_or_else(|error| {
                format!("Artifact batch lookup benchmark: unavailable ({error})")
            }),
    );
    text.push('\n');
    text.push_str("\nArtifact decode benchmark\n");
    text.push_str("  ");
    text.push_str(
        &report
            .decode_benchmark
            .validated_summary_line()
            .unwrap_or_else(|error| format!("Artifact decode benchmark: unavailable ({error})")),
    );
    text.push('\n');

    text.push_str("\nArtifact fit outliers by channel\n");
    text.push_str("  ");
    text.push_str(&pleiades_data::packaged_artifact_fit_channel_outlier_summary_for_report());
    text.push('\n');

    text.push_str("\nRelease summary: release-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Workspace audit: workspace-audit / audit\n");
    text.push_str(
        "See validate-artifact for the full body-class envelopes and regression details.\nSee release-summary for the compact one-screen release overview.\n",
    );

    text
}

fn midpoint(start: Instant, end: Instant) -> Instant {
    let start_days = start.julian_day.days();
    let end_days = end.julian_day.days();
    Instant::new(
        JulianDay::from_days((start_days + end_days) / 2.0),
        start.scale,
    )
}

fn artifact_first_instant(artifact: &CompressedArtifact) -> Instant {
    artifact
        .bodies
        .iter()
        .flat_map(|body| body.segments.iter())
        .map(|segment| segment.start)
        .min_by(|left, right| {
            left.julian_day
                .days()
                .partial_cmp(&right.julian_day.days())
                .unwrap_or(core::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| {
            Instant::new(
                pleiades_core::JulianDay::from_days(0.0),
                pleiades_core::TimeScale::Tt,
            )
        })
}

struct BoundaryDelta {
    longitude_delta_deg: f64,
    latitude_delta_deg: f64,
    distance_delta_au: Option<f64>,
}

fn artifact_boundary_envelope_summary(
    report: &ArtifactInspectionReport,
) -> ArtifactBoundaryEnvelopeSummary {
    let mut summary = ArtifactBoundaryEnvelopeSummary {
        body_count: report.bodies.len(),
        boundary_check_count: 0,
        sum_boundary_longitude_delta_deg: 0.0,
        sum_boundary_longitude_delta_deg_sq: 0.0,
        sum_boundary_latitude_delta_deg: 0.0,
        sum_boundary_latitude_delta_deg_sq: 0.0,
        sum_boundary_distance_delta_au: None,
        sum_boundary_distance_delta_au_sq: None,
        boundary_distance_check_count: 0,
        max_boundary_longitude_delta_body: None,
        max_boundary_longitude_delta_deg: 0.0,
        max_boundary_latitude_delta_body: None,
        max_boundary_latitude_delta_deg: 0.0,
        max_boundary_distance_delta_body: None,
        max_boundary_distance_delta_au: None,
    };

    for body in &report.bodies {
        summary.boundary_check_count += body.boundary_checks;
        summary.sum_boundary_longitude_delta_deg += body.sum_boundary_longitude_delta_deg;
        summary.sum_boundary_longitude_delta_deg_sq += body.sum_boundary_longitude_delta_deg_sq;
        summary.sum_boundary_latitude_delta_deg += body.sum_boundary_latitude_delta_deg;
        summary.sum_boundary_latitude_delta_deg_sq += body.sum_boundary_latitude_delta_deg_sq;
        if let Some(sum) = body.sum_boundary_distance_delta_au {
            summary.sum_boundary_distance_delta_au =
                Some(summary.sum_boundary_distance_delta_au.unwrap_or(0.0) + sum);
            summary.sum_boundary_distance_delta_au_sq = Some(
                summary.sum_boundary_distance_delta_au_sq.unwrap_or(0.0)
                    + body.sum_boundary_distance_delta_au_sq.unwrap_or(0.0),
            );
            summary.boundary_distance_check_count += body.boundary_distance_checks;
        }
        if body.boundary_checks == 0 {
            continue;
        }

        if body.max_boundary_longitude_delta_deg >= summary.max_boundary_longitude_delta_deg {
            summary.max_boundary_longitude_delta_deg = body.max_boundary_longitude_delta_deg;
            summary.max_boundary_longitude_delta_body = Some(body.body.clone());
        }
        if body.max_boundary_latitude_delta_deg >= summary.max_boundary_latitude_delta_deg {
            summary.max_boundary_latitude_delta_deg = body.max_boundary_latitude_delta_deg;
            summary.max_boundary_latitude_delta_body = Some(body.body.clone());
        }
        match (
            summary.max_boundary_distance_delta_au,
            body.max_boundary_distance_delta_au,
        ) {
            (Some(current), Some(next)) if next < current => {}
            (_, Some(next)) => {
                summary.max_boundary_distance_delta_au = Some(next);
                summary.max_boundary_distance_delta_body = Some(body.body.clone());
            }
            _ => {}
        }
    }

    summary
}

fn boundary_delta(left: &EclipticCoordinates, right: &EclipticCoordinates) -> BoundaryDelta {
    let longitude_delta_deg =
        Angle::from_degrees(left.longitude.degrees() - right.longitude.degrees())
            .normalized_signed()
            .degrees()
            .abs();
    let latitude_delta_deg = (left.latitude.degrees() - right.latitude.degrees()).abs();
    let distance_delta_au = match (left.distance_au, right.distance_au) {
        (Some(left), Some(right)) => Some((left - right).abs()),
        _ => None,
    };

    BoundaryDelta {
        longitude_delta_deg,
        latitude_delta_deg,
        distance_delta_au,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BodyClass {
    Luminary,
    MajorPlanet,
    LunarPoint,
    Asteroid,
    Custom,
}

impl BodyClass {
    const ALL: [Self; 5] = [
        Self::Luminary,
        Self::MajorPlanet,
        Self::LunarPoint,
        Self::Asteroid,
        Self::Custom,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Luminary => "Luminaries",
            Self::MajorPlanet => "Major planets",
            Self::LunarPoint => "Lunar points",
            Self::Asteroid => "Asteroids",
            Self::Custom => "Custom bodies",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Luminary => 0,
            Self::MajorPlanet => 1,
            Self::LunarPoint => 2,
            Self::Asteroid => 3,
            Self::Custom => 4,
        }
    }
}

fn body_class(body: &CelestialBody) -> BodyClass {
    match body {
        CelestialBody::Sun | CelestialBody::Moon => BodyClass::Luminary,
        CelestialBody::Mercury
        | CelestialBody::Venus
        | CelestialBody::Mars
        | CelestialBody::Jupiter
        | CelestialBody::Saturn
        | CelestialBody::Uranus
        | CelestialBody::Neptune
        | CelestialBody::Pluto => BodyClass::MajorPlanet,
        CelestialBody::MeanNode
        | CelestialBody::TrueNode
        | CelestialBody::MeanApogee
        | CelestialBody::TrueApogee
        | CelestialBody::MeanPerigee
        | CelestialBody::TruePerigee => BodyClass::LunarPoint,
        CelestialBody::Ceres
        | CelestialBody::Pallas
        | CelestialBody::Juno
        | CelestialBody::Vesta => BodyClass::Asteroid,
        CelestialBody::Custom(_) => BodyClass::Custom,
        _ => BodyClass::Custom,
    }
}

#[derive(Clone, Debug)]
struct BodyClassSummary {
    class: BodyClass,
    sample_count: usize,
    max_longitude_delta_body: Option<CelestialBody>,
    max_longitude_delta_deg: f64,
    sum_longitude_delta_deg: f64,
    max_latitude_delta_body: Option<CelestialBody>,
    max_latitude_delta_deg: f64,
    sum_latitude_delta_deg: f64,
    max_distance_delta_body: Option<CelestialBody>,
    max_distance_delta_au: Option<f64>,
    sum_distance_delta_au: f64,
    distance_count: usize,
}

impl BodyClassSummary {
    const fn new(class: BodyClass) -> Self {
        Self {
            class,
            sample_count: 0,
            max_longitude_delta_body: None,
            max_longitude_delta_deg: 0.0,
            sum_longitude_delta_deg: 0.0,
            max_latitude_delta_body: None,
            max_latitude_delta_deg: 0.0,
            sum_latitude_delta_deg: 0.0,
            max_distance_delta_body: None,
            max_distance_delta_au: None,
            sum_distance_delta_au: 0.0,
            distance_count: 0,
        }
    }

    fn update(&mut self, sample: &ComparisonSample) {
        self.sample_count += 1;
        self.sum_longitude_delta_deg += sample.longitude_delta_deg;
        if sample.longitude_delta_deg >= self.max_longitude_delta_deg {
            self.max_longitude_delta_deg = sample.longitude_delta_deg;
            self.max_longitude_delta_body = Some(sample.body.clone());
        }
        self.sum_latitude_delta_deg += sample.latitude_delta_deg;
        if sample.latitude_delta_deg >= self.max_latitude_delta_deg {
            self.max_latitude_delta_deg = sample.latitude_delta_deg;
            self.max_latitude_delta_body = Some(sample.body.clone());
        }

        if let Some(distance_delta_au) = sample.distance_delta_au {
            match self.max_distance_delta_au {
                Some(current) if distance_delta_au < current => {}
                _ => {
                    self.max_distance_delta_au = Some(distance_delta_au);
                    self.max_distance_delta_body = Some(sample.body.clone());
                }
            }
            self.sum_distance_delta_au += distance_delta_au;
            self.distance_count += 1;
        }
    }

    fn mean_longitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.sum_longitude_delta_deg / self.sample_count as f64
        }
    }

    fn mean_latitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.sum_latitude_delta_deg / self.sample_count as f64
        }
    }

    fn mean_distance_delta_au(&self) -> Option<f64> {
        if self.distance_count == 0 {
            None
        } else {
            Some(self.sum_distance_delta_au / self.distance_count as f64)
        }
    }

    fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.sample_count == 0 {
            return Ok(());
        }

        writeln!(f, "  {}", self.class.label())?;
        writeln!(f, "    samples: {}", self.sample_count)?;
        if let (Some(body), value) = (
            self.max_longitude_delta_body.as_ref(),
            self.max_longitude_delta_deg,
        ) {
            writeln!(f, "    max longitude delta: {:.12}° ({})", value, body)?;
        }
        writeln!(
            f,
            "    mean longitude delta: {:.12}°",
            self.mean_longitude_delta_deg()
        )?;
        if let (Some(body), value) = (
            self.max_latitude_delta_body.as_ref(),
            self.max_latitude_delta_deg,
        ) {
            writeln!(f, "    max latitude delta: {:.12}° ({})", value, body)?;
        }
        writeln!(
            f,
            "    mean latitude delta: {:.12}°",
            self.mean_latitude_delta_deg()
        )?;
        if let (Some(body), Some(value)) = (
            self.max_distance_delta_body.as_ref(),
            self.max_distance_delta_au,
        ) {
            writeln!(f, "    max distance delta: {:.12} AU ({})", value, body)?;
        }
        if let Some(value) = self.mean_distance_delta_au() {
            writeln!(f, "    mean distance delta: {:.12} AU", value)?;
        }

        Ok(())
    }
}

fn write_body_class_envelopes(
    f: &mut fmt::Formatter<'_>,
    samples: &[ComparisonSample],
) -> fmt::Result {
    writeln!(f, "Body-class error envelopes")?;

    let mut summaries = BodyClass::ALL.map(BodyClassSummary::new);
    for sample in samples {
        summaries[body_class(&sample.body).index()].update(sample);
    }

    let mut has_entries = false;
    for summary in &summaries {
        if summary.sample_count > 0 {
            has_entries = true;
            summary.render(f)?;
        }
    }

    if !has_entries {
        writeln!(f, "  none")?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct BodyClassCadenceAccumulator {
    class: BodyClass,
    body_count: usize,
    segment_count: usize,
    min_segment_span_days: f64,
    max_segment_span_days: f64,
    sum_segment_span_days: f64,
}

impl BodyClassCadenceAccumulator {
    const fn new(class: BodyClass) -> Self {
        Self {
            class,
            body_count: 0,
            segment_count: 0,
            min_segment_span_days: f64::INFINITY,
            max_segment_span_days: 0.0,
            sum_segment_span_days: 0.0,
        }
    }

    fn push(&mut self, body: &ArtifactBodyInspection) {
        self.body_count += 1;
        self.segment_count += body.segment_count;
        if body.segment_count == 0 {
            return;
        }

        self.min_segment_span_days = self.min_segment_span_days.min(body.min_segment_span_days);
        self.max_segment_span_days = self.max_segment_span_days.max(body.max_segment_span_days);
        self.sum_segment_span_days += body.mean_segment_span_days * body.segment_count as f64;
    }

    fn finish(self) -> Option<BodyClassCadenceSummary> {
        if self.body_count == 0 {
            return None;
        }

        Some(BodyClassCadenceSummary {
            class: self.class,
            body_count: self.body_count,
            segment_count: self.segment_count,
            min_segment_span_days: if self.min_segment_span_days.is_infinite() {
                0.0
            } else {
                self.min_segment_span_days
            },
            max_segment_span_days: self.max_segment_span_days,
            mean_segment_span_days: if self.segment_count == 0 {
                0.0
            } else {
                self.sum_segment_span_days / self.segment_count as f64
            },
        })
    }
}

#[derive(Clone, Debug)]
struct BodyClassCadenceSummary {
    class: BodyClass,
    body_count: usize,
    segment_count: usize,
    min_segment_span_days: f64,
    max_segment_span_days: f64,
    mean_segment_span_days: f64,
}

impl BodyClassCadenceSummary {
    fn summary_line(&self) -> String {
        format!(
            "{}: {} bodies, {} segments, span days={:.12}..{:.12} (mean {:.12})",
            self.class.label(),
            self.body_count,
            self.segment_count,
            self.min_segment_span_days,
            self.max_segment_span_days,
            self.mean_segment_span_days,
        )
    }
}

fn body_class_cadence_summaries(report: &ArtifactInspectionReport) -> Vec<BodyClassCadenceSummary> {
    let mut accumulators = BodyClass::ALL.map(BodyClassCadenceAccumulator::new);
    for body in &report.bodies {
        accumulators[body_class(&body.body).index()].push(body);
    }

    accumulators
        .into_iter()
        .filter_map(BodyClassCadenceAccumulator::finish)
        .collect()
}

fn format_body_class_cadence(report: &ArtifactInspectionReport) -> String {
    let summaries = body_class_cadence_summaries(report);
    if summaries.is_empty() {
        "none".to_string()
    } else {
        summaries
            .into_iter()
            .map(|summary| summary.summary_line())
            .collect::<Vec<_>>()
            .join("; ")
    }
}

fn format_body_class_span_caps() -> String {
    let summary = pleiades_data::packaged_artifact_body_class_span_cap_summary_for_report();
    summary
        .strip_prefix("body-class span caps: ")
        .unwrap_or(&summary)
        .to_string()
}

impl fmt::Display for ArtifactInspectionReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Artifact validation report")?;
        writeln!(f, "  label: {}", self.generation_label)?;
        writeln!(f, "  source: {}", self.source)?;
        writeln!(f, "  version: {}", self.version)?;
        writeln!(f, "  byte order: {}", self.endian_policy.label())?;
        writeln!(f, "  checksum: 0x{:016x}", self.checksum)?;
        writeln!(f, "  encoded bytes: {}", self.encoded_bytes)?;
        writeln!(f, "  roundtrip decode: {}", yes_no(self.roundtrip_ok))?;
        writeln!(f, "  checksum verified: {}", yes_no(self.checksum_ok))?;
        writeln!(f, "  bodies: {}", self.body_count)?;
        writeln!(f, "  segments: {}", self.segment_count)?;
        writeln!(
            f,
            "  residual-bearing segments: {}",
            self.residual_segment_count
        )?;
        writeln!(
            f,
            "  residual-bearing bodies: {}",
            format_residual_bodies(&self.residual_bodies)
        )?;
        writeln!(
            f,
            "  coverage: {} → {}",
            self.earliest.julian_day, self.latest.julian_day
        )?;
        writeln!(f, "  {}", artifact_boundary_envelope_summary(self))?;
        writeln!(
            f,
            "  Artifact request policy: {}",
            packaged_request_policy_summary_details()
        )?;
        writeln!(
            f,
            "  Artifact output support: {}",
            packaged_artifact_output_support_summary_for_report()
        )?;
        writeln!(f)?;
        writeln!(f, "Bodies")?;
        for body in &self.bodies {
            writeln!(f, "  {}", body)?;
        }

        writeln!(f)?;
        if self
            .bodies
            .iter()
            .any(|body| matches!(body.body, CelestialBody::Custom(_)))
        {
            writeln!(f, "Note: custom bodies are included in decode and boundary checks, but omitted from the algorithmic comparison corpus.")?;
            writeln!(f)?;
        }
        writeln!(f, "Model error envelope")?;
        writeln!(
            f,
            "  baseline backend: {}",
            self.model_comparison.reference_backend.id
        )?;
        writeln!(
            f,
            "  candidate backend: {}",
            self.model_comparison.candidate_backend.id
        )?;
        writeln!(f, "  corpus: {}", self.model_comparison.corpus_name)?;
        writeln!(
            f,
            "  samples: {}",
            self.model_comparison.summary.sample_count
        )?;
        writeln!(
            f,
            "  max longitude delta: {:.12}°",
            self.model_comparison.summary.max_longitude_delta_deg
        )?;
        writeln!(
            f,
            "  mean longitude delta: {:.12}°",
            self.model_comparison.summary.mean_longitude_delta_deg
        )?;
        writeln!(
            f,
            "  max latitude delta: {:.12}°",
            self.model_comparison.summary.max_latitude_delta_deg
        )?;
        writeln!(
            f,
            "  mean latitude delta: {:.12}°",
            self.model_comparison.summary.mean_latitude_delta_deg
        )?;
        if let Some(value) = self.model_comparison.summary.max_distance_delta_au {
            writeln!(f, "  max distance delta: {:.12} AU", value)?;
        }
        if let Some(value) = self.model_comparison.summary.mean_distance_delta_au {
            writeln!(f, "  mean distance delta: {:.12} AU", value)?;
        }

        writeln!(f)?;
        write_body_class_envelopes(f, &self.model_comparison.samples)?;
        writeln!(f, "Body-class cadence")?;
        writeln!(f, "  {}", format_body_class_cadence(self))?;
        writeln!(f, "Body-class span caps")?;
        writeln!(f, "  {}", format_body_class_span_caps())?;
        writeln!(f)?;

        let notable_regressions = self.model_comparison.notable_regressions();
        writeln!(f, "  notable regressions")?;
        if notable_regressions.is_empty() {
            writeln!(f, "    none")?;
        } else {
            for finding in notable_regressions {
                writeln!(f, "    {}", finding.summary_line())?;
            }
        }

        writeln!(f)?;
        writeln!(f, "Artifact lookup benchmark")?;
        writeln!(f, "  {}", self.lookup_benchmark.summary_line())?;
        writeln!(f, "  artifact: {}", self.lookup_benchmark.artifact_label)?;
        writeln!(f, "  source: {}", self.lookup_benchmark.source)?;
        writeln!(f, "  rounds: {}", self.lookup_benchmark.rounds)?;
        writeln!(
            f,
            "  lookups per round: {}",
            self.lookup_benchmark.sample_count
        )?;
        writeln!(
            f,
            "  encoded bytes: {}",
            self.lookup_benchmark.encoded_bytes
        )?;
        writeln!(
            f,
            "  elapsed: {}",
            super::format_duration(self.lookup_benchmark.elapsed)
        )?;
        writeln!(
            f,
            "  nanoseconds per lookup: {:.2}",
            self.lookup_benchmark.nanoseconds_per_lookup()
        )?;
        writeln!(
            f,
            "  lookups per second: {:.2}",
            self.lookup_benchmark.lookups_per_second()
        )?;

        writeln!(f)?;
        writeln!(f, "Artifact batch lookup benchmark")?;
        writeln!(f, "  {}", self.batch_lookup_benchmark.summary_line())?;
        writeln!(
            f,
            "  artifact: {}",
            self.batch_lookup_benchmark.artifact_label
        )?;
        writeln!(f, "  source: {}", self.batch_lookup_benchmark.source)?;
        writeln!(f, "  rounds: {}", self.batch_lookup_benchmark.rounds)?;
        writeln!(
            f,
            "  lookups per batch: {}",
            self.batch_lookup_benchmark.batch_size
        )?;
        writeln!(
            f,
            "  encoded bytes: {}",
            self.batch_lookup_benchmark.encoded_bytes
        )?;
        writeln!(
            f,
            "  elapsed: {}",
            super::format_duration(self.batch_lookup_benchmark.elapsed)
        )?;
        writeln!(
            f,
            "  nanoseconds per lookup: {:.2}",
            self.batch_lookup_benchmark.nanoseconds_per_lookup()
        )?;
        writeln!(
            f,
            "  lookups per second: {:.2}",
            self.batch_lookup_benchmark.lookups_per_second()
        )?;

        writeln!(f)?;
        writeln!(f, "Artifact decode benchmark")?;
        writeln!(f, "  {}", self.decode_benchmark.summary_line())?;
        writeln!(f, "  artifact: {}", self.decode_benchmark.artifact_label)?;
        writeln!(f, "  source: {}", self.decode_benchmark.source)?;
        writeln!(f, "  rounds: {}", self.decode_benchmark.rounds)?;
        writeln!(
            f,
            "  decodes per round: {}",
            self.decode_benchmark.sample_count
        )?;
        writeln!(
            f,
            "  encoded bytes: {}",
            self.decode_benchmark.encoded_bytes
        )?;
        writeln!(
            f,
            "  elapsed: {}",
            super::format_duration(self.decode_benchmark.elapsed)
        )?;
        writeln!(
            f,
            "  nanoseconds per decode: {:.2}",
            self.decode_benchmark.nanoseconds_per_decode()
        )?;
        writeln!(
            f,
            "  decodes per second: {:.2}",
            self.decode_benchmark.decodes_per_second()
        )?;

        Ok(())
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "ok"
    } else {
        "failed"
    }
}

fn backend_family_label(family: &BackendFamily) -> &'static str {
    match family {
        BackendFamily::Algorithmic => "algorithmic",
        BackendFamily::ReferenceData => "reference data",
        BackendFamily::CompressedData => "compressed data",
        BackendFamily::Composite => "composite",
        BackendFamily::Other(_) => "other",
        _ => "other",
    }
}

/// Errors produced while building the artifact inspection report.
#[derive(Debug)]
pub enum ArtifactInspectionError {
    /// Compression or codec failure while decoding the packaged artifact.
    Compression(CompressionError),
    /// Validation failure while comparing the packaged artifact to the baseline backend.
    Validation(pleiades_core::EphemerisError),
    /// Validation failure while checking the aggregated artifact boundary envelope.
    BoundaryEnvelope(ArtifactBoundaryEnvelopeSummaryValidationError),
    /// Validation failure while checking the packaged-artifact decode benchmark summary.
    DecodeBenchmark(ArtifactDecodeBenchmarkReportValidationError),
    /// Validation failure while checking the packaged-artifact lookup benchmark summary.
    LookupBenchmark(ArtifactLookupBenchmarkReportValidationError),
    /// Validation failure while checking the packaged-artifact batch lookup benchmark summary.
    BatchLookupBenchmark(ArtifactBatchLookupBenchmarkReportValidationError),
}

impl core::fmt::Display for ArtifactInspectionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Compression(error) => write!(f, "{error}"),
            Self::Validation(error) => write!(f, "{error}"),
            Self::BoundaryEnvelope(error) => write!(f, "{error}"),
            Self::DecodeBenchmark(error) => write!(f, "{error}"),
            Self::LookupBenchmark(error) => write!(f, "{error}"),
            Self::BatchLookupBenchmark(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ArtifactInspectionError {}

impl From<CompressionError> for ArtifactInspectionError {
    fn from(error: CompressionError) -> Self {
        Self::Compression(error)
    }
}

impl From<pleiades_core::EphemerisError> for ArtifactInspectionError {
    fn from(error: pleiades_core::EphemerisError) -> Self {
        Self::Validation(error)
    }
}

impl From<ArtifactDecodeBenchmarkReportValidationError> for ArtifactInspectionError {
    fn from(error: ArtifactDecodeBenchmarkReportValidationError) -> Self {
        Self::DecodeBenchmark(error)
    }
}

impl From<ArtifactLookupBenchmarkReportValidationError> for ArtifactInspectionError {
    fn from(error: ArtifactLookupBenchmarkReportValidationError) -> Self {
        Self::LookupBenchmark(error)
    }
}

impl From<ArtifactBatchLookupBenchmarkReportValidationError> for ArtifactInspectionError {
    fn from(error: ArtifactBatchLookupBenchmarkReportValidationError) -> Self {
        Self::BatchLookupBenchmark(error)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ArtifactBatchLookupBenchmarkReport, ArtifactBatchLookupBenchmarkReportValidationError,
        ArtifactBodyInspection, ArtifactBoundaryEnvelopeSummary,
        ArtifactBoundaryEnvelopeSummaryValidationError, ArtifactDecodeBenchmarkReport,
        ArtifactDecodeBenchmarkReportValidationError, ArtifactInspectionReport,
        ArtifactLookupBenchmarkReport, ArtifactLookupBenchmarkReportValidationError,
    };
    use pleiades_core::{CelestialBody, Instant, JulianDay, TimeScale};
    use pleiades_data::packaged_artifact;
    use std::time::Duration;

    fn instant(days: f64) -> Instant {
        Instant::new(JulianDay::from_days(days), TimeScale::Tt)
    }

    fn decode_benchmark_report() -> ArtifactDecodeBenchmarkReport {
        ArtifactDecodeBenchmarkReport {
            artifact_label: "packaged artifact".to_string(),
            source: "public reference snapshot".to_string(),
            rounds: 2,
            sample_count: 3,
            encoded_bytes: 128,
            elapsed: Duration::from_millis(5),
        }
    }

    fn lookup_benchmark_report() -> ArtifactLookupBenchmarkReport {
        ArtifactLookupBenchmarkReport {
            artifact_label: "packaged artifact".to_string(),
            source: "public reference snapshot".to_string(),
            corpus_name: "packaged artifact lookup corpus".to_string(),
            rounds: 2,
            sample_count: 4,
            encoded_bytes: 128,
            elapsed: Duration::from_millis(8),
        }
    }

    fn batch_lookup_benchmark_report() -> ArtifactBatchLookupBenchmarkReport {
        ArtifactBatchLookupBenchmarkReport {
            artifact_label: "packaged artifact".to_string(),
            source: "public reference snapshot".to_string(),
            corpus_name: "packaged artifact lookup corpus".to_string(),
            rounds: 2,
            batch_size: 4,
            encoded_bytes: 128,
            elapsed: Duration::from_millis(8),
        }
    }

    #[test]
    fn body_inspection_summary_includes_mean_boundary_deltas() {
        let inspection = ArtifactBodyInspection {
            body: CelestialBody::Sun,
            segment_count: 2,
            earliest: instant(1.0),
            latest: instant(2.0),
            sample_count: 6,
            min_segment_span_days: 0.5,
            max_segment_span_days: 1.5,
            mean_segment_span_days: 1.0,
            residual_segment_count: 1,
            boundary_checks: 2,
            sum_boundary_longitude_delta_deg: 0.20,
            sum_boundary_longitude_delta_deg_sq: 0.05,
            sum_boundary_latitude_delta_deg: 0.40,
            sum_boundary_latitude_delta_deg_sq: 0.20,
            sum_boundary_distance_delta_au: Some(0.60),
            sum_boundary_distance_delta_au_sq: Some(0.45),
            boundary_distance_checks: 2,
            max_boundary_longitude_delta_deg: 0.15,
            max_boundary_latitude_delta_deg: 0.30,
            max_boundary_distance_delta_au: Some(0.45),
        };

        let summary = inspection.summary_line();
        assert!(summary.contains("Sun: 2 segments,"));
        assert!(summary.contains("JD 1 TT → JD 2 TT"));
        assert!(summary.contains("6 samples, 2 boundary checks, 1 residual-bearing segments"));
        assert!(summary.contains("span days=0.500000000000..1.500000000000 (mean 1.000000000000)"));
        assert!(summary.contains("mean boundary Δlon=0.100000000000°"));
        assert!(summary.contains("rms boundary Δlon=0.158113883008°"));
        assert!(summary.contains("mean boundary Δlat=0.200000000000°"));
        assert!(summary.contains("rms boundary Δlat=0.316227766017°"));
        assert!(summary.contains("mean boundary Δdist=0.300000000000 AU"));
        assert!(summary.contains("rms boundary Δdist=0.474341649025 AU"));
        assert!(summary.contains("max boundary Δlon=0.150000000000°"));
        assert!(summary.contains("Δlat=0.300000000000°"));
        assert!(summary.contains("Δdist=0.450000000000 AU"));
    }

    #[test]
    fn artifact_lookup_benchmark_report_validated_summary_line_matches_summary_line() {
        let report = lookup_benchmark_report();

        let summary = report.summary_line();
        assert!(summary.contains("artifact=packaged artifact"));
        assert!(summary.contains("corpus=packaged artifact lookup corpus"));
        assert!(matches!(
            report.validated_summary_line(),
            Ok(rendered) if rendered == summary
        ));
    }

    #[test]
    fn artifact_lookup_benchmark_report_validated_summary_line_rejects_drift() {
        let mut report = lookup_benchmark_report();
        report.corpus_name = " ".to_string();

        assert!(matches!(
            report.validated_summary_line(),
            Err(ArtifactLookupBenchmarkReportValidationError::BlankCorpusName)
        ));
    }

    #[test]
    fn artifact_decode_benchmark_report_validated_summary_line_matches_summary_line() {
        let report = decode_benchmark_report();

        let summary = report.summary_line();
        assert!(summary.contains("artifact=packaged artifact"));
        assert!(summary.contains("source=public reference snapshot"));
        assert!(matches!(
            report.validated_summary_line(),
            Ok(rendered) if rendered == summary
        ));
    }

    #[test]
    fn artifact_decode_benchmark_report_validated_summary_line_rejects_drift() {
        let mut report = decode_benchmark_report();
        report.encoded_bytes = 0;

        assert!(matches!(
            report.validated_summary_line(),
            Err(ArtifactDecodeBenchmarkReportValidationError::ZeroEncodedBytes)
        ));
    }

    #[test]
    fn artifact_inspection_report_summary_line_includes_residual_bodies_and_checksum_status() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");

        let summary = report.summary_line();
        assert!(summary.contains("artifact inspection:"));
        assert!(summary.contains("residual-bearing segments:"));
        assert!(summary.contains("residual-bearing bodies: Moon"));
        assert!(summary.contains("body classes: luminaries=2; major planets=8; lunar points=0; built-in asteroids=0; custom bodies=1; other bodies=0"));
        assert!(summary.contains("roundtrip=ok"));
        assert!(summary.contains("checksum=ok"));
        assert!(summary.contains("encoded bytes="));
        assert!(summary.contains(&format!("encoded bytes={}", report.encoded_bytes)));
        assert!(matches!(
            report.validated_summary_line(),
            Ok(rendered) if rendered == summary
        ));
    }

    #[test]
    fn render_artifact_summary_includes_span_caps() {
        let rendered = super::render_artifact_summary().expect("artifact summary should render");

        assert!(rendered.contains("Artifact summary"));
        assert!(rendered.contains("Body-class cadence:"));
        assert!(rendered.contains("Body-class span caps: luminaries=256 days, inner planets=384 days, outer planets=768 days, pluto=1536 days, lunar points=256 days, selected asteroids=256 days, custom bodies=512 days"));
    }

    #[test]
    fn artifact_inspection_report_validated_summary_line_rejects_drift() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let mut report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");
        report.lookup_benchmark.corpus_name = " ".to_string();

        let error = report
            .validated_summary_line()
            .expect_err("lookup benchmark drift should fail validation");
        assert!(matches!(
            error,
            super::ArtifactInspectionError::LookupBenchmark(
                ArtifactLookupBenchmarkReportValidationError::BlankCorpusName
            )
        ));
    }

    #[test]
    fn artifact_inspection_report_validate_rejects_body_count_drift() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let mut report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");
        report.body_count += 1;

        let error = report
            .validate()
            .expect_err("body count drift should fail validation");
        assert!(error
            .to_string()
            .contains("artifact inspection report field `body_count`"));
    }

    #[test]
    fn artifact_inspection_report_validate_rejects_residual_body_coverage_drift() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let mut report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");
        report.residual_bodies.push(CelestialBody::Sun);

        let error = report
            .validate()
            .expect_err("residual body drift should fail validation");
        assert!(error
            .to_string()
            .contains("artifact inspection report field `residual_bodies` does not match the inspected residual-bearing body set"));
    }

    #[test]
    fn artifact_inspection_report_validate_rejects_residual_segment_count_drift() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let mut report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");
        report.residual_segment_count += 1;

        let error = report
            .validate()
            .expect_err("residual segment count drift should fail validation");
        assert!(error
            .to_string()
            .contains("artifact inspection report field `residual_segment_count` does not match the inspected residual-bearing segment count"));
    }

    #[test]
    fn artifact_inspection_report_validate_rejects_decode_benchmark_drift() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let mut report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");
        report.decode_benchmark.encoded_bytes += 1;

        let error = report
            .validate()
            .expect_err("decode benchmark drift should fail validation");
        assert!(error.to_string().contains(
            "artifact inspection report decode benchmark encoded byte count does not match"
        ));
    }

    #[test]
    fn artifact_inspection_report_validate_rejects_lookup_benchmark_drift() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let mut report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");
        report.lookup_benchmark.corpus_name = " ".to_string();

        let error = report
            .validate()
            .expect_err("lookup benchmark drift should fail validation");
        assert!(matches!(
            error,
            super::ArtifactInspectionError::LookupBenchmark(
                ArtifactLookupBenchmarkReportValidationError::BlankCorpusName
            )
        ));
    }

    #[test]
    fn artifact_inspection_report_validate_rejects_batch_lookup_benchmark_drift() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let mut report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");
        report.batch_lookup_benchmark.batch_size = 0;

        let error = report
            .validate()
            .expect_err("batch lookup benchmark drift should fail validation");
        assert!(matches!(
            error,
            super::ArtifactInspectionError::BatchLookupBenchmark(
                ArtifactBatchLookupBenchmarkReportValidationError::ZeroBatchSize
            )
        ));
    }

    #[test]
    fn boundary_envelope_summary_includes_mean_boundary_deltas() {
        let summary = ArtifactBoundaryEnvelopeSummary {
            body_count: 2,
            boundary_check_count: 3,
            sum_boundary_longitude_delta_deg: 0.30,
            sum_boundary_longitude_delta_deg_sq: 0.07,
            sum_boundary_latitude_delta_deg: 0.60,
            sum_boundary_latitude_delta_deg_sq: 0.29,
            sum_boundary_distance_delta_au: Some(0.90),
            sum_boundary_distance_delta_au_sq: Some(0.63),
            boundary_distance_check_count: 3,
            max_boundary_longitude_delta_body: Some(CelestialBody::Moon),
            max_boundary_longitude_delta_deg: 0.18,
            max_boundary_latitude_delta_body: Some(CelestialBody::Sun),
            max_boundary_latitude_delta_deg: 0.27,
            max_boundary_distance_delta_body: Some(CelestialBody::Moon),
            max_boundary_distance_delta_au: Some(0.33),
        };

        let rendered = summary.summary_line();
        assert!(rendered.contains("Artifact boundary envelope: 3 checks across 2 bundled bodies"));
        assert!(rendered.contains("mean boundary Δlon=0.100000000000°"));
        assert!(rendered.contains("rms boundary Δlon=0.152752523165°"));
        assert!(rendered.contains("mean boundary Δlat=0.200000000000°"));
        assert!(rendered.contains("rms boundary Δlat=0.310912635103°"));
        assert!(rendered.contains("mean boundary Δdist=0.300000000000 AU (3 distance checks)"));
        assert!(rendered.contains("rms boundary Δdist=0.458257569496 AU (3 distance checks)"));
        assert!(rendered.contains("max boundary Δlon=0.180000000000° (Moon)"));
        assert!(rendered.contains("max boundary Δlat=0.270000000000° (Sun)"));
        assert!(rendered.contains("max boundary Δdist=0.330000000000 AU (Moon)"));
        assert_eq!(
            summary
                .validated_summary_line()
                .expect("boundary summary should validate"),
            rendered
        );
    }

    #[test]
    fn boundary_envelope_summary_rejects_inconsistent_distance_channels() {
        let summary = ArtifactBoundaryEnvelopeSummary {
            body_count: 1,
            boundary_check_count: 2,
            sum_boundary_longitude_delta_deg: 0.30,
            sum_boundary_longitude_delta_deg_sq: 0.07,
            sum_boundary_latitude_delta_deg: 0.60,
            sum_boundary_latitude_delta_deg_sq: 0.29,
            sum_boundary_distance_delta_au: Some(0.90),
            sum_boundary_distance_delta_au_sq: None,
            boundary_distance_check_count: 1,
            max_boundary_longitude_delta_body: Some(CelestialBody::Moon),
            max_boundary_longitude_delta_deg: 0.18,
            max_boundary_latitude_delta_body: Some(CelestialBody::Sun),
            max_boundary_latitude_delta_deg: 0.27,
            max_boundary_distance_delta_body: None,
            max_boundary_distance_delta_au: None,
        };

        let error = summary
            .validate()
            .expect_err("inconsistent distance coverage should fail");
        assert!(matches!(
            error,
            ArtifactBoundaryEnvelopeSummaryValidationError::InconsistentDistanceCoverage {
                boundary_distance_check_count: 1,
                has_sum: true,
                has_sum_sq: false,
                has_max: false,
            }
        ));
    }

    #[test]
    fn decode_benchmark_report_validate_accepts_compact_metadata() {
        let report = decode_benchmark_report();

        assert!(report.validate().is_ok());
        assert!((report.nanoseconds_per_decode() - 833_333.3333333334).abs() < 1e-9);
        assert!((report.decodes_per_second() - 1_200.0).abs() < 1e-9);
        assert!(report.to_string().contains("Artifact: packaged artifact"));
    }

    #[test]
    fn decode_benchmark_report_summary_line_mentions_the_provenance_and_throughput() {
        let report = decode_benchmark_report();

        let summary = report.summary_line();
        assert!(summary.contains("artifact=packaged artifact"));
        assert!(summary.contains("source=public reference snapshot"));
        assert!(summary.contains("rounds=2"));
        assert!(summary.contains("decodes per round=3"));
        assert!(summary.contains("encoded bytes=128"));
        assert!(summary.contains("ns/decode=833333.33"));
        assert!(summary.contains("decodes/s=1200.00"));
    }

    #[test]
    fn decode_benchmark_report_validate_rejects_invalid_metadata() {
        let mut report = decode_benchmark_report();
        report.artifact_label = "   ".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactDecodeBenchmarkReportValidationError::BlankArtifactLabel)
        ));

        let mut report = decode_benchmark_report();
        report.source = "\t".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactDecodeBenchmarkReportValidationError::BlankSource)
        ));

        let mut report = decode_benchmark_report();
        report.rounds = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactDecodeBenchmarkReportValidationError::ZeroRounds)
        ));

        let mut report = decode_benchmark_report();
        report.sample_count = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactDecodeBenchmarkReportValidationError::ZeroSampleCount)
        ));

        let mut report = decode_benchmark_report();
        report.encoded_bytes = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactDecodeBenchmarkReportValidationError::ZeroEncodedBytes)
        ));
    }

    #[test]
    fn lookup_benchmark_report_summary_line_mentions_the_provenance_and_throughput() {
        let report = lookup_benchmark_report();

        let summary = report.summary_line();
        assert!(summary.contains("artifact=packaged artifact"));
        assert!(summary.contains("source=public reference snapshot"));
        assert!(summary.contains("corpus=packaged artifact lookup corpus"));
        assert!(summary.contains("rounds=2"));
        assert!(summary.contains("lookups per round=4"));
        assert!(summary.contains("encoded bytes=128"));
        assert!(summary.contains("ns/lookup=1000000.00"));
        assert!(summary.contains("lookups/s=1000.00"));
    }

    #[test]
    fn batch_lookup_benchmark_report_summary_line_mentions_the_provenance_and_throughput() {
        let report = batch_lookup_benchmark_report();

        let summary = report.summary_line();
        assert!(summary.contains("artifact=packaged artifact"));
        assert!(summary.contains("source=public reference snapshot"));
        assert!(summary.contains("corpus=packaged artifact lookup corpus"));
        assert!(summary.contains("rounds=2"));
        assert!(summary.contains("lookups per batch=4"));
        assert!(summary.contains("encoded bytes=128"));
        assert!(summary.contains("ns/lookup=1000000.00"));
        assert!(summary.contains("lookups/s=1000.00"));
    }

    #[test]
    fn batch_lookup_benchmark_report_validate_rejects_invalid_metadata() {
        let mut report = batch_lookup_benchmark_report();
        report.artifact_label = "   ".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactBatchLookupBenchmarkReportValidationError::BlankArtifactLabel)
        ));

        let mut report = batch_lookup_benchmark_report();
        report.source = "	".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactBatchLookupBenchmarkReportValidationError::BlankSource)
        ));

        let mut report = batch_lookup_benchmark_report();
        report.corpus_name = " ".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactBatchLookupBenchmarkReportValidationError::BlankCorpusName)
        ));

        let mut report = batch_lookup_benchmark_report();
        report.rounds = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactBatchLookupBenchmarkReportValidationError::ZeroRounds)
        ));

        let mut report = batch_lookup_benchmark_report();
        report.batch_size = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactBatchLookupBenchmarkReportValidationError::ZeroBatchSize)
        ));

        let mut report = batch_lookup_benchmark_report();
        report.encoded_bytes = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactBatchLookupBenchmarkReportValidationError::ZeroEncodedBytes)
        ));
    }

    #[test]
    fn lookup_benchmark_report_validate_rejects_invalid_metadata() {
        let mut report = lookup_benchmark_report();
        report.artifact_label = "   ".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactLookupBenchmarkReportValidationError::BlankArtifactLabel)
        ));

        let mut report = lookup_benchmark_report();
        report.source = "\t".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactLookupBenchmarkReportValidationError::BlankSource)
        ));

        let mut report = lookup_benchmark_report();
        report.corpus_name = " ".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactLookupBenchmarkReportValidationError::BlankCorpusName)
        ));

        let mut report = lookup_benchmark_report();
        report.rounds = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactLookupBenchmarkReportValidationError::ZeroRounds)
        ));

        let mut report = lookup_benchmark_report();
        report.sample_count = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactLookupBenchmarkReportValidationError::ZeroSampleCount)
        ));

        let mut report = lookup_benchmark_report();
        report.encoded_bytes = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactLookupBenchmarkReportValidationError::ZeroEncodedBytes)
        ));
    }
}
