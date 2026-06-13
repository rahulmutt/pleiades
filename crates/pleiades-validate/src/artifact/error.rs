//! Error type for artifact inspection and its `From` conversions.
//!
//! Moved verbatim from the parent `artifact` module.

use pleiades_compression::CompressionError;

use crate::artifact::{
    ArtifactBatchLookupBenchmarkReportValidationError,
    ArtifactBoundaryEnvelopeSummaryValidationError, ArtifactDecodeBenchmarkReportValidationError,
    ArtifactLookupBenchmarkReportValidationError,
};

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
