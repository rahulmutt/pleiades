//! VSOP87 source-audit and generated-binary-audit report prose relocated
//! from `pleiades-vsop87::source_docs::audit` (report-surface relocation
//! program, Slice B). Rendering only — the functional crate keeps the
//! structured data and their constructors.
//!
//! `generated_binary_audit_summary_for_report` could not be copied verbatim:
//! its original body called `pleiades-vsop87`'s crate-private
//! `build_generated_binary_audits_with_lookup` and a fully private
//! aggregation helper, both inaccessible across the crate boundary. It is
//! reconstructed here from the retained public API
//! (`generated_binary_audits`, `validate_generated_binary_audits`,
//! `generated_binary_audit_summary`), which the original vsop87 test suite
//! already asserted renders byte-identically to the direct-build path (see
//! the moved test `generated_binary_audit_summary_for_report_matches_the_backend_formatter`
//! below).

use pleiades_vsop87::{
    generated_binary_audit_summary, generated_binary_audits, source_audit_summary,
    validate_generated_binary_audits, Vsop87GeneratedBlobAuditSummary, Vsop87SourceAuditSummary,
};

/// Formats the current VSOP87 reproducibility audit for reporting.
pub(crate) fn format_source_audit_summary(summary: &Vsop87SourceAuditSummary) -> String {
    summary.summary_line()
}

pub(crate) fn format_validated_source_audit_summary_for_report(
    summary: &Vsop87SourceAuditSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(rendered) => rendered,
        Err(error) => format!("VSOP87 source audit: unavailable ({error})"),
    }
}

/// Returns the release-facing reproducibility audit summary string.
pub(crate) fn source_audit_summary_for_report() -> String {
    format_validated_source_audit_summary_for_report(&source_audit_summary())
}

/// Formats the checked-in generated VSOP87B blob audit for reporting.
pub(crate) fn format_generated_binary_audit_summary(
    summary: &Vsop87GeneratedBlobAuditSummary,
) -> String {
    summary.summary_line()
}

pub(crate) fn format_validated_generated_binary_audit_summary_for_report(
    summary: &Vsop87GeneratedBlobAuditSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(rendered) => rendered,
        Err(error) => format!("VSOP87 generated binary audit: unavailable ({error})"),
    }
}

/// Returns the release-facing generated binary audit summary string.
pub(crate) fn generated_binary_audit_summary_for_report() -> String {
    let audits = generated_binary_audits();

    if let Err(error) = validate_generated_binary_audits(&audits) {
        return format!("VSOP87 generated binary audit: unavailable ({error})");
    }

    let summary = generated_binary_audit_summary();
    format_validated_generated_binary_audit_summary_for_report(&summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    use pleiades_vsop87::{
        Vsop87GeneratedBlobAuditSummaryValidationError, Vsop87SourceAuditSummaryValidationError,
    };

    #[test]
    fn source_audit_summary_for_report_matches_the_backend_formatter() {
        let summary = source_audit_summary();
        assert_eq!(
            source_audit_summary_for_report(),
            "VSOP87 source audit: 8 source-backed bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune) across 8 source files (VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep); 8 vendored full-file inputs, 35080 total terms, max source size 949753 bytes / 7141 lines, 8 deterministic fingerprints"
        );
        assert_eq!(
            source_audit_summary_for_report(),
            format_source_audit_summary(&summary)
        );
    }

    #[test]
    fn format_validated_source_audit_summary_for_report_reports_drifted_fields() {
        let mut summary = source_audit_summary();
        summary.fingerprint_count += 1;

        assert_eq!(
            summary.validate(),
            Err(Vsop87SourceAuditSummaryValidationError::FieldOutOfSync {
                field: "fingerprint_count"
            })
        );
        assert_eq!(
            format_validated_source_audit_summary_for_report(&summary),
            "VSOP87 source audit: unavailable (the VSOP87 source audit summary field `fingerprint_count` is out of sync with the current manifest)"
        );
    }

    #[test]
    fn generated_binary_audit_summary_for_report_matches_the_backend_formatter() {
        let summary = generated_binary_audit_summary();

        assert_eq!(
            generated_binary_audit_summary_for_report(),
            format_generated_binary_audit_summary(&summary)
        );
        assert!(generated_binary_audit_summary_for_report().contains(
            "VSOP87 generated binary audit: 8 checked-in blobs across 8 source files (bodies: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep)"
        ));
    }

    #[test]
    fn format_validated_generated_binary_audit_summary_for_report_reports_drifted_fields() {
        let mut summary = generated_binary_audit_summary();
        summary.source_file_count += 1;

        assert_eq!(
            summary.validate(),
            Err(
                Vsop87GeneratedBlobAuditSummaryValidationError::FieldOutOfSync {
                    field: "source_file_count"
                }
            )
        );
        assert_eq!(
            format_validated_generated_binary_audit_summary_for_report(&summary),
            "VSOP87 generated binary audit: unavailable (the VSOP87 generated binary audit summary field `source_file_count` is out of sync with the current manifest)"
        );
    }
}
