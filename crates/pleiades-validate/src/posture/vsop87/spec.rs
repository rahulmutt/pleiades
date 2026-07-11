//! VSOP87 specification report prose relocated from
//! `pleiades-vsop87::source_docs::spec` (report-surface relocation program,
//! Slice B). Rendering only — the functional crate keeps the structured data
//! and their constructors.

use pleiades_vsop87::{
    format_source_specifications, frame_treatment_summary_details, source_specifications,
    validate_source_specifications, vsop87_request_policy,
};

/// Returns the release-facing source-specification catalog string.
pub(crate) fn source_specifications_for_report() -> String {
    let specs = source_specifications();
    match validate_source_specifications(&specs) {
        Ok(()) => format_source_specifications(&specs),
        Err(error) => format!("VSOP87 source specifications: unavailable ({error})"),
    }
}

/// Returns the release-facing frame-treatment summary for VSOP87-backed results.
///
/// The backend-owned note is validated before the compact report line is
/// rendered, so a drifted summary becomes an unavailable report rather than a
/// stale cached string.
pub(crate) fn frame_treatment_summary_for_report() -> String {
    let summary = frame_treatment_summary_details();
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line.to_string(),
        Err(error) => format!("VSOP87 frame treatment unavailable ({error})"),
    }
}

/// Returns the release-facing VSOP87 request policy summary string.
pub(crate) fn vsop87_request_policy_summary_for_report() -> String {
    let policy = vsop87_request_policy();
    match policy.validate() {
        Ok(()) => policy.to_string(),
        Err(error) => format!("VSOP87 request policy: unavailable ({error})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pleiades_vsop87::format_source_specification;

    #[test]
    fn source_specifications_for_report_matches_source_specification_formatting() {
        let specs = source_specifications();
        let expected_joined = specs
            .iter()
            .map(format_source_specification)
            .collect::<Vec<_>>()
            .join(", ");

        assert_eq!(source_specifications_for_report(), expected_joined);
        assert!(source_specifications_for_report().contains("body=Neptune"));
    }

    #[test]
    fn frame_treatment_summary_for_report_matches_the_structured_summary() {
        let summary = frame_treatment_summary_details();

        assert_eq!(frame_treatment_summary_for_report(), summary.to_string());
    }

    #[test]
    fn vsop87_request_policy_summary_for_report_matches_the_structured_summary() {
        let policy = vsop87_request_policy();

        assert_eq!(
            vsop87_request_policy_summary_for_report(),
            policy.summary_line()
        );
    }
}
