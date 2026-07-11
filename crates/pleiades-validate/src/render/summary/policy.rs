//! Reference, lunar, frame, and request/comparison policy summary text.

use crate::*;

pub(crate) fn render_reference_snapshot_summary_text() -> String {
    format!(
        "Reference snapshot summary\n{}\n",
        reference_snapshot_summary_for_report()
    )
}

pub(crate) fn render_reference_snapshot_exact_j2000_evidence_text() -> String {
    format!(
        "Reference snapshot exact J2000 evidence summary\n{}\n",
        reference_snapshot_exact_j2000_evidence_summary_for_report()
    )
}

pub(crate) fn render_lunar_reference_error_envelope_summary_text() -> String {
    format!(
        "Lunar reference error envelope summary\n{}\n",
        crate::posture::elp::evidence::lunar_reference_evidence_envelope_for_report()
    )
}

pub(crate) fn render_lunar_reference_evidence_summary_text() -> String {
    format!(
        "Lunar reference evidence summary\n{}\n",
        crate::posture::elp::evidence::lunar_reference_evidence_summary_for_report()
    )
}

pub(crate) fn render_lunar_equatorial_reference_error_envelope_summary_text() -> String {
    format!(
        "Lunar equatorial reference error envelope summary\n{}\n",
        crate::posture::elp::evidence::lunar_equatorial_reference_evidence_envelope_for_report()
    )
}

pub(crate) fn render_lunar_apparent_comparison_summary_text() -> String {
    format!(
        "Lunar apparent comparison summary\n{}\n",
        crate::posture::elp::evidence::lunar_apparent_comparison_summary_for_report()
    )
}

pub(crate) fn render_frame_policy_summary_text() -> String {
    match frame_policy_summary_details().validated_summary_line() {
        Ok(summary) => format!("Frame policy summary\nFrame policy: {}\n", summary),
        Err(error) => format!("Frame policy summary\nFrame policy unavailable ({error})\n"),
    }
}

pub(crate) fn render_reference_holdout_overlap_summary_text() -> String {
    match validated_reference_holdout_overlap_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Reference/hold-out overlap: unavailable ({error})"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RequestPolicyReportKind {
    Policy,
    Semantics,
}

impl RequestPolicyReportKind {
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Policy => "Request policy summary\n",
            Self::Semantics => "Request semantics summary\n",
        }
    }

    pub(crate) const fn unavailable_prefix(self) -> &'static str {
        match self {
            Self::Policy => "Request policy summary unavailable",
            Self::Semantics => "Request semantics summary unavailable",
        }
    }
}

pub(crate) fn validate_request_policy_report_title(
    kind: RequestPolicyReportKind,
    title: &str,
) -> Result<(), String> {
    let expected = kind.title();
    if title != expected {
        return Err(format!("{} ({title})", kind.unavailable_prefix()));
    }
    Ok(())
}

pub(crate) fn render_request_policy_like_summary_text(
    title: &'static str,
    kind: RequestPolicyReportKind,
) -> String {
    let time_scale_policy = time_scale_policy_summary_for_report();
    if let Err(error) = validate_request_policy_report_title(kind, title) {
        return error;
    }

    let mut text = String::from(title);
    text.push_str(&format_request_semantics_summary_for_report(
        &time_scale_policy,
    ));
    text
}

pub(crate) fn render_request_policy_summary_text() -> String {
    render_request_policy_like_summary_text(
        "Request policy summary\n",
        RequestPolicyReportKind::Policy,
    )
}

pub(crate) fn render_request_semantics_summary_text() -> String {
    use std::fmt::Write as _;

    let mut text = render_request_policy_like_summary_text(
        "Request semantics summary\n",
        RequestPolicyReportKind::Semantics,
    );
    let _ = writeln!(
        text,
        "Unsupported modes: {}",
        current_compatibility_profile().unsupported_modes_summary_line()
    );
    text
}

pub(crate) fn render_unsupported_modes_summary_text() -> String {
    format!(
        "Unsupported modes summary\nUnsupported modes: {}\n",
        current_compatibility_profile().unsupported_modes_summary_line()
    )
}

pub(crate) fn render_request_surface_summary_text() -> String {
    format!(
        "Request surface summary\n{}\n",
        request_surface_summary_for_report()
    )
}

pub(crate) fn render_comparison_tolerance_policy_summary_text_from_report(
    report: Result<ComparisonReport, String>,
) -> String {
    match report {
        Ok(report) => format!(
            "Comparison tolerance policy summary\nComparison tolerance policy: {}\n",
            format_comparison_tolerance_policy_for_report(&report)
        ),
        Err(error) => format!(
            "Comparison tolerance policy summary\nComparison tolerance policy unavailable ({error})\n"
        ),
    }
}

pub(crate) fn render_comparison_tolerance_policy_summary_text() -> String {
    render_comparison_tolerance_policy_summary_text_from_report(
        comparison_report_for_default_render(),
    )
}
pub(crate) fn render_comparison_tolerance_scope_coverage_summary_text_from_summary(
    summary: Result<ComparisonTolerancePolicySummary, String>,
) -> String {
    use std::fmt::Write as _;

    let summary = match summary {
        Ok(summary) => match summary.validate() {
            Ok(()) => summary,
            Err(error) => {
                return format!("Comparison tolerance scope coverage summary\nComparison tolerance scope coverage unavailable ({error})\n");
            }
        },
        Err(error) => {
            return format!("Comparison tolerance scope coverage summary\nComparison tolerance scope coverage unavailable ({error})\n");
        }
    };

    let mut text = String::from("Comparison tolerance scope coverage summary\n");
    let _ = writeln!(
        text,
        "Scope coverage posture: {} rows",
        summary.coverage.len()
    );
    for coverage in &summary.coverage {
        let _ = writeln!(text, "  {}", coverage.summary_line());
    }
    text
}

pub(crate) fn render_comparison_tolerance_scope_coverage_summary_text() -> String {
    let summary = match comparison_report_for_default_render() {
        Ok(report) => validated_comparison_tolerance_policy_summary_for_report(&report),
        Err(error) => Err(error),
    };

    render_comparison_tolerance_scope_coverage_summary_text_from_summary(summary)
}

pub(crate) fn render_comparison_body_class_tolerance_summary_text_from_summaries(
    summaries: Result<Vec<BodyClassToleranceSummary>, String>,
) -> String {
    use std::fmt::Write as _;

    let summaries = match summaries {
        Ok(summaries) => summaries,
        Err(error) => {
            return format!("Comparison body-class tolerance summary\nComparison body-class tolerance unavailable ({error})\n");
        }
    };

    if summaries.is_empty() {
        return "Comparison body-class tolerance summary\nComparison body-class tolerance unavailable (comparison report did not produce any body-class tolerance summaries)\n".to_string();
    }

    for summary in &summaries {
        if let Err(error) = summary.validate() {
            return format!("Comparison body-class tolerance summary\nComparison body-class tolerance unavailable ({error})\n");
        }
    }

    let mut text = String::from("Comparison body-class tolerance summary\n");
    let _ = writeln!(text, "Body-class tolerance posture: {}", summaries.len());
    for summary in summaries {
        let _ = writeln!(
            text,
            "  {}",
            format_body_class_tolerance_envelope_for_report(&summary)
        );
    }
    text
}

pub(crate) fn render_comparison_body_class_tolerance_summary_text() -> String {
    let summaries = match comparison_report_for_default_render() {
        Ok(report) => Ok(report.body_class_tolerance_summaries()),
        Err(error) => Err(error),
    };

    render_comparison_body_class_tolerance_summary_text_from_summaries(summaries)
}

pub(crate) fn render_comparison_body_class_tolerance_posture_summary_text() -> String {
    match validated_comparison_body_class_tolerance_posture_for_report() {
        Ok(summary) => format!(
            "Comparison body-class tolerance posture summary\n{}\n",
            summary
        ),
        Err(error) => format!(
            "Comparison body-class tolerance posture summary\nComparison body-class tolerance unavailable ({error})\n"
        ),
    }
}

pub(crate) fn render_comparison_envelope_summary_text() -> String {
    let report = match comparison_report_for_default_render() {
        Ok(report) => report,
        Err(error) => {
            return format!(
                "Comparison envelope summary\nComparison envelope unavailable ({error})\n"
            );
        }
    };
    let envelope = comparison_envelope_summary(&report.summary, &report.samples);
    let summary_line = envelope
        .validated_summary_line(&report.samples)
        .unwrap_or_else(|error| format!("comparison envelope unavailable ({error})"));
    let percentile_line = envelope
        .validated_percentile_line(&report.samples)
        .unwrap_or_else(|error| format!("comparison percentile envelope unavailable ({error})"));

    format!(
        "Comparison envelope summary\nSummary line: {summary_line}\nPercentile line: {percentile_line}\n"
    )
}

pub(crate) fn ensure_comparison_envelope_summary_matches_current_rendering(
    comparison_envelope_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_envelope_summary_text == render_comparison_envelope_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "comparison envelope summary no longer matches the current comparison envelope posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_comparison_body_class_tolerance_summary_matches_current_rendering(
    comparison_body_class_tolerance_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_body_class_tolerance_summary_text
        == render_comparison_body_class_tolerance_summary_text()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "comparison body-class tolerance summary no longer matches the current comparison body-class tolerance posture"
                .to_string(),
        ))
    }
}
