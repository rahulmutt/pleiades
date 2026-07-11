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
pub(crate) use audit::build_generated_binary_audits_with_lookup;
#[cfg(test)]
pub(crate) use documentation::source_documentation_health_issues;
#[cfg(test)]
pub(crate) use evidence::{
    CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL, CANONICAL_EVIDENCE_SUMMARY_LABEL,
};
