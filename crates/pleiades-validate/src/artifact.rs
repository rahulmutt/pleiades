use core::fmt;
use std::time::Instant as StdInstant;

use crate::{
    compare_backends, default_candidate_backend, ComparisonReport, ComparisonSample,
    ValidationCorpus,
};
use pleiades_compression::{join_display, CompressedArtifact, CompressionError, EndianPolicy};
use pleiades_core::{
    Angle, Apparentness, BackendFamily, CelestialBody, CoordinateFrame, EclipticCoordinates,
    EphemerisRequest, Instant, JulianDay, ZodiacMode,
};
use pleiades_data::{
    packaged_artifact, packaged_artifact_profile_summary_with_body_coverage,
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
            "{}: {} segments, {} → {}, {} samples, {} boundary checks, mean boundary Δlon={:.12}°, rms boundary Δlon={:.12}°, mean boundary Δlat={:.12}°, rms boundary Δlat={:.12}°, mean boundary Δdist={}, rms boundary Δdist={}, max boundary Δlon={:.12}°, Δlat={:.12}°, Δdist={}",
            self.body,
            self.segment_count,
            self.earliest,
            self.latest,
            self.sample_count,
            self.boundary_checks,
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

impl ArtifactInspectionReport {
    fn from_artifact(
        artifact: &CompressedArtifact,
        encoded_bytes: usize,
    ) -> Result<Self, ArtifactInspectionError> {
        let decoded = CompressedArtifact::decode(&artifact.encode()?)?;
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
            bodies,
        };
        artifact_boundary_envelope_summary(&report)
            .validate()
            .map_err(ArtifactInspectionError::BoundaryEnvelope)?;
        Ok(report)
    }
}

/// Renders the bundled artifact validation report.
pub fn render_artifact_report() -> Result<String, ArtifactInspectionError> {
    let artifact = packaged_artifact();
    let encoded = artifact.encode()?;
    let report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())?;
    Ok(report.to_string())
}

/// Returns the aggregate packaged-artifact boundary envelope used by reports.
pub fn artifact_boundary_envelope_summary_for_report(
) -> Result<ArtifactBoundaryEnvelopeSummary, ArtifactInspectionError> {
    let artifact = packaged_artifact();
    let encoded = artifact.encode()?;
    let report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())?;
    let summary = artifact_boundary_envelope_summary(&report);
    summary
        .validate()
        .map_err(ArtifactInspectionError::BoundaryEnvelope)?;
    Ok(summary)
}

/// Renders a compact summary of the bundled artifact validation report.
pub fn render_artifact_summary() -> Result<String, ArtifactInspectionError> {
    let artifact = packaged_artifact();
    let encoded = artifact.encode()?;
    let report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())?;
    Ok(render_artifact_summary_text(&report))
}

pub(crate) fn benchmark_packaged_artifact_decode(
    rounds: usize,
) -> Result<ArtifactDecodeBenchmarkReport, ArtifactInspectionError> {
    let artifact = packaged_artifact();
    let encoded = artifact.encode()?;
    let start = StdInstant::now();
    for _ in 0..rounds {
        std::hint::black_box(CompressedArtifact::decode(&encoded)?);
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

pub(crate) fn packaged_artifact_corpus() -> ValidationCorpus {
    artifact_comparison_corpus(packaged_artifact())
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

    Ok(ArtifactBodyInspection {
        body: body.body.clone(),
        segment_count: body.segments.len(),
        earliest,
        latest,
        sample_count,
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

fn format_residual_bodies(bodies: &[CelestialBody]) -> String {
    if bodies.is_empty() {
        "none".to_string()
    } else {
        join_display(bodies)
    }
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
    text.push_str(&artifact_boundary_envelope_summary(report).summary_line());
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
    let median = crate::comparison_median_envelope(&report.model_comparison.samples);
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
    text.push_str("\nArtifact decode benchmark\n");
    text.push_str("  ");
    text.push_str(&report.decode_benchmark.summary_line());
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
}

impl core::fmt::Display for ArtifactInspectionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Compression(error) => write!(f, "{error}"),
            Self::Validation(error) => write!(f, "{error}"),
            Self::BoundaryEnvelope(error) => write!(f, "{error}"),
            Self::DecodeBenchmark(error) => write!(f, "{error}"),
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

#[cfg(test)]
mod tests {
    use super::{
        ArtifactBodyInspection, ArtifactBoundaryEnvelopeSummary,
        ArtifactBoundaryEnvelopeSummaryValidationError, ArtifactDecodeBenchmarkReport,
        ArtifactDecodeBenchmarkReportValidationError,
    };
    use pleiades_core::{CelestialBody, Instant, JulianDay, TimeScale};
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

    #[test]
    fn body_inspection_summary_includes_mean_boundary_deltas() {
        let inspection = ArtifactBodyInspection {
            body: CelestialBody::Sun,
            segment_count: 2,
            earliest: instant(1.0),
            latest: instant(2.0),
            sample_count: 6,
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
        assert!(summary.contains("6 samples, 2 boundary checks"));
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
        assert!(summary.validate().is_ok());
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
}
