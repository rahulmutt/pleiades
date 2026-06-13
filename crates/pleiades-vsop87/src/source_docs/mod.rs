mod audit;
mod batch_parity;
mod documentation;
mod evidence;
mod request_corpus;
mod spec;

pub use audit::*;
pub use batch_parity::*;
pub use documentation::*;
pub use evidence::*;
pub use request_corpus::*;
pub use spec::*;

// Re-export pub(crate) items so they remain accessible as crate::source_docs::fn_name.
// These are only consumed by #[cfg(test)] code in tests.rs.
#[cfg(test)]
pub(crate) use audit::{
    build_generated_binary_audits_with_lookup,
    format_validated_generated_binary_audit_summary_for_report,
    format_validated_source_audit_summary_for_report,
};
#[cfg(test)]
pub(crate) use batch_parity::{
    format_validated_canonical_j1900_batch_parity_summary_for_report,
    format_validated_canonical_j2000_batch_parity_summary_for_report,
    format_validated_canonical_mixed_time_scale_batch_parity_summary_for_report,
    format_validated_source_body_class_evidence_summary_for_report,
    format_validated_supported_body_canonical_batch_parity_summary_for_report,
    format_validated_supported_body_j1900_ecliptic_batch_parity_summary_for_report,
    format_validated_supported_body_j1900_equatorial_batch_parity_summary_for_report,
    format_validated_supported_body_j2000_ecliptic_batch_parity_summary_for_report,
    format_validated_supported_body_j2000_equatorial_batch_parity_summary_for_report,
};
#[cfg(test)]
pub(crate) use documentation::{
    format_validated_source_documentation_health_summary_for_report,
    format_validated_source_documentation_summary_for_report, source_documentation_health_issues,
};
#[cfg(test)]
pub(crate) use evidence::{
    format_validated_canonical_epoch_evidence_summary_for_report,
    CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL, CANONICAL_EVIDENCE_SUMMARY_LABEL,
};
