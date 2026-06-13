//! Artifact inspection report and benchmark data types.
//!
//! These types describe the bundled compressed artifact, its boundary
//! continuity envelope, and the decode/lookup benchmark summaries. They are
//! split out of the parent `artifact` module verbatim; see that module for the
//! inspection, rendering, and error-handling logic that produces and consumes
//! them.

use core::fmt;

use crate::ComparisonReport;
use pleiades_compression::EndianPolicy;
use pleiades_core::{CelestialBody, Instant};

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
            crate::format_duration(self.elapsed)
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
            crate::format_duration(self.elapsed)
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
            crate::format_duration(self.elapsed)
        )?;
        writeln!(
            f,
            "Nanoseconds per lookup: {:.2}",
            self.nanoseconds_per_lookup()
        )?;
        writeln!(f, "Lookups per second: {:.2}", self.lookups_per_second())
    }
}
