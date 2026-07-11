//! ELP lunar-theory library summaries report prose.

use pleiades_elp::{LunarTheoryRequestPolicy, LunarTheorySourceSummary, LunarTheorySpecification};

fn format_validated_lunar_theory_specification_for_report(
    theory: &LunarTheorySpecification,
) -> String {
    match theory.validate() {
        Ok(()) => theory.summary_line(),
        Err(error) => format!("ELP lunar theory specification: unavailable ({error})"),
    }
}

/// Returns the release-facing one-line summary for the current lunar-theory selection.
///
/// The validation helper checks the backend-owned specification first so any
/// future drift in the rendered provenance fields shows up as an unavailable
/// report line instead of a silently stale summary.
pub(crate) fn lunar_theory_summary_for_report() -> String {
    format_validated_lunar_theory_specification_for_report(
        &pleiades_elp::lunar_theory_specification(),
    )
}

/// Returns the raw one-line summary for the current lunar-theory selection.
///
/// Validation and release tooling should prefer
/// [`lunar_theory_summary_for_report()`] so the backend-owned specification is
/// validated before the compact provenance line is rendered.
pub(crate) fn lunar_theory_summary() -> String {
    pleiades_elp::lunar_theory_specification().summary_line()
}

/// Formats the compact lunar source-selection summary for release-facing reporting.
pub(crate) fn format_lunar_theory_source_summary(summary: &LunarTheorySourceSummary) -> String {
    summary.summary_line()
}

fn format_validated_lunar_theory_source_summary_for_report(
    summary: &LunarTheorySourceSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("lunar source selection: unavailable ({error})"),
    }
}

/// Returns the release-facing one-line summary for the current lunar source selection.
///
/// The report helper validates the backend-owned source summary first so any
/// future drift in the rendered provenance fields shows up as an unavailable
/// report line instead of a silently stale summary.
pub(crate) fn lunar_theory_source_summary_for_report() -> String {
    format_validated_lunar_theory_source_summary_for_report(
        &pleiades_elp::lunar_theory_source_summary(),
    )
}

fn format_validated_lunar_theory_request_policy_for_report(
    policy: &LunarTheoryRequestPolicy,
) -> String {
    match policy.validate() {
        Ok(()) => policy.summary_line(),
        Err(error) => format!("lunar theory request policy: unavailable ({error})"),
    }
}

/// Returns the current lunar-theory request policy summary.
///
/// The report helper validates the backend-owned request-policy summary first so
/// any future drift in the supported frames, time scales, zodiac modes,
/// apparentness, or topocentric-observer posture shows up as an unavailable
/// report line instead of a silently stale summary.
pub(crate) fn lunar_theory_request_policy_summary() -> String {
    format_validated_lunar_theory_request_policy_for_report(
        &pleiades_elp::lunar_theory_request_policy(),
    )
}

/// Returns the current lunar-theory frame-treatment summary for reports.
///
/// The report helper validates the backend-owned frame-treatment summary first
/// so any future drift in the mean-obliquity wording shows up as an unavailable
/// report line instead of a silently stale summary.
pub(crate) fn lunar_theory_frame_treatment_summary_for_report() -> String {
    let summary = pleiades_elp::lunar_theory_frame_treatment_summary_details();
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line.to_string(),
        Err(error) => format!("ELP frame treatment unavailable ({error})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lunar_theory_summary_for_report_matches_the_backend_owned_specification() {
        let theory = pleiades_elp::lunar_theory_specification();
        assert_eq!(lunar_theory_summary_for_report(), theory.summary_line());
        assert_eq!(lunar_theory_summary(), theory.summary_line());
    }

    #[test]
    fn format_lunar_theory_source_summary_matches_the_summary_line() {
        let source_summary = pleiades_elp::lunar_theory_source_summary();
        assert_eq!(
            format_lunar_theory_source_summary(&source_summary),
            source_summary.summary_line()
        );
        assert_eq!(
            lunar_theory_source_summary_for_report(),
            format_lunar_theory_source_summary(&source_summary)
        );
    }

    #[test]
    fn format_validated_lunar_theory_source_summary_for_report_fails_closed_for_drifted_fields() {
        let source_summary = pleiades_elp::lunar_theory_source_summary();
        let mut drifted_source_summary = source_summary;
        drifted_source_summary.source_identifier = "not-the-current-selection";
        assert_eq!(
            format_validated_lunar_theory_source_summary_for_report(&drifted_source_summary),
            "lunar source selection: unavailable (the lunar source summary field `source_identifier` is out of sync with the current selection)"
        );
    }

    #[test]
    fn lunar_theory_request_policy_summary_matches_the_backend_owned_policy() {
        let theory = pleiades_elp::lunar_theory_specification();
        assert_eq!(
            lunar_theory_request_policy_summary(),
            theory.request_policy.summary_line()
        );
    }

    #[test]
    fn format_validated_lunar_theory_request_policy_for_report_fails_closed_for_drifted_fields() {
        let theory = pleiades_elp::lunar_theory_specification();
        let mut drifted_request_policy = theory.request_policy;
        drifted_request_policy.supported_time_scales = &[pleiades_types::TimeScale::Tt];
        assert_eq!(
            format_validated_lunar_theory_request_policy_for_report(&drifted_request_policy),
            "lunar theory request policy: unavailable (the lunar theory request policy field `supported_time_scales` is out of sync with the current selection)"
        );
    }

    #[test]
    fn format_validated_lunar_theory_specification_for_report_fails_closed_for_drifted_fields() {
        let mut drifted_spec = pleiades_elp::lunar_theory_specification();
        drifted_spec.request_policy = LunarTheoryRequestPolicy {
            supported_time_scales: &[pleiades_types::TimeScale::Tt],
            ..drifted_spec.request_policy
        };
        assert_eq!(
            format_validated_lunar_theory_specification_for_report(&drifted_spec),
            "ELP lunar theory specification: unavailable (the lunar theory specification field `request_policy.supported_time_scales` is out of sync with the current selection)"
        );
    }

    #[test]
    fn lunar_theory_frame_treatment_summary_for_report_matches_the_structured_summary() {
        let summary = pleiades_elp::lunar_theory_frame_treatment_summary_details();
        assert_eq!(
            lunar_theory_frame_treatment_summary_for_report(),
            summary.validated_summary_line().unwrap().to_string()
        );
    }
}
