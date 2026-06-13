//! Comparison sample, tolerance, audit, and report types for validation runs.

pub(crate) mod body_class;
mod sample;
mod tolerance;

pub(crate) use body_class::{
    body_class, BodyClass, BodyClassSummary, BodyClassToleranceSummary,
};
pub use sample::{
    BodyComparisonSummary, ComparisonAuditSummary, ComparisonSample, ComparisonSummary,
};
pub(crate) use tolerance::validate_comparison_tolerance;
pub use tolerance::{
    ComparisonTolerance, ComparisonToleranceEntry, ComparisonTolerancePolicySummary,
    ComparisonToleranceScope, ComparisonToleranceScopeCoverageSummary,
};
