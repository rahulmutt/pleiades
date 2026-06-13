//! Comparison sample, tolerance, audit, and report types for validation runs.

mod sample;
mod tolerance;

pub use sample::{
    BodyComparisonSummary, ComparisonAuditSummary, ComparisonSample, ComparisonSummary,
};
pub use tolerance::{
    ComparisonTolerance, ComparisonToleranceEntry, ComparisonTolerancePolicySummary,
    ComparisonToleranceScope, ComparisonToleranceScopeCoverageSummary,
};
pub(crate) use tolerance::validate_comparison_tolerance;
