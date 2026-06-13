//! Comparison sample, tolerance, audit, and report types for validation runs.

pub(crate) mod audit;
pub(crate) mod body_class;
pub(crate) mod report;
mod sample;
mod tolerance;

pub(crate) use report::{
    body_class_summaries, body_class_tolerance_summaries, comparison_tolerance_for_body,
    comparison_tolerance_policy_entries, comparison_tolerance_scope_for_body,
};
pub use report::{
    compare_backends, comparison_tolerance_catalog_entries, default_candidate_backend,
    default_reference_backend, ComparisonReport,
};

pub(crate) use audit::{
    comparison_audit_summary, comparison_audit_summary_for_report, comparison_audit_totals,
    format_regression_bodies, format_summary_body,
};
pub use audit::{RegressionArchive, RegressionFinding};
pub(crate) use body_class::{body_class, BodyClass, BodyClassSummary, BodyClassToleranceSummary};
pub use sample::{
    BodyComparisonSummary, ComparisonAuditSummary, ComparisonSample, ComparisonSummary,
};
pub(crate) use tolerance::validate_comparison_tolerance;
pub use tolerance::{
    ComparisonTolerance, ComparisonToleranceEntry, ComparisonTolerancePolicySummary,
    ComparisonToleranceScope, ComparisonToleranceScopeCoverageSummary,
};
