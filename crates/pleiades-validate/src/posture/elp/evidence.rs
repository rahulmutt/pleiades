//! ELP lunar-theory evidence report prose.

use pleiades_elp::{
    LunarApparentComparisonSummary, LunarEquatorialReferenceBatchParitySummary,
    LunarEquatorialReferenceEvidenceSummary, LunarReferenceBatchParitySummary,
    LunarReferenceEvidenceSummary, LunarSourceWindowSummary,
};

/// Formats the lunar equatorial batch-parity evidence for release-facing reporting.
pub(crate) fn format_lunar_equatorial_reference_batch_parity_summary(
    summary: &LunarEquatorialReferenceBatchParitySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing lunar equatorial batch-parity summary string.
pub(crate) fn lunar_equatorial_reference_batch_parity_summary_for_report() -> String {
    match pleiades_elp::lunar_equatorial_reference_batch_parity_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format_lunar_equatorial_reference_batch_parity_summary(&summary),
            Err(error) => {
                format!("lunar equatorial reference batch parity: unavailable ({error})")
            }
        },
        None => "lunar equatorial reference batch parity: unavailable".to_string(),
    }
}

/// Formats the lunar reference evidence summary for release-facing reporting.
pub(crate) fn format_lunar_reference_evidence_summary(
    summary: &LunarReferenceEvidenceSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing lunar reference evidence summary string.
pub(crate) fn lunar_reference_evidence_summary_for_report() -> String {
    match pleiades_elp::lunar_reference_evidence_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format_lunar_reference_evidence_summary(&summary),
            Err(error) => format!("lunar reference evidence: unavailable ({error})"),
        },
        None => "lunar reference evidence: unavailable".to_string(),
    }
}

/// Formats the broader lunar source-window summary for release-facing reporting.
fn format_validated_lunar_source_window_summary_for_report(
    summary: &LunarSourceWindowSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("lunar source windows: unavailable ({error})"),
    }
}

/// Formats the broader lunar source-window summary for release-facing reporting.
pub(crate) fn format_lunar_source_window_summary(summary: &LunarSourceWindowSummary) -> String {
    format_validated_lunar_source_window_summary_for_report(summary)
}

/// Returns the validated release-facing broader lunar source-window summary string.
pub(crate) fn validated_lunar_source_window_summary_for_report() -> Result<String, String> {
    pleiades_elp::lunar_source_window_summary()
        .ok_or_else(|| {
            "the lunar source-window summary is unavailable from the current evidence".to_string()
        })
        .and_then(|summary| {
            summary
                .validated_summary_line()
                .map_err(|error| error.to_string())
        })
}

/// Returns the release-facing broader lunar source-window summary string.
pub(crate) fn lunar_source_window_summary_for_report() -> String {
    match pleiades_elp::lunar_source_window_summary() {
        Some(summary) => format_validated_lunar_source_window_summary_for_report(&summary),
        None => "lunar source windows: unavailable".to_string(),
    }
}

/// Formats the lunar mixed TT/TDB batch-parity evidence for release-facing reporting.
pub(crate) fn format_lunar_reference_batch_parity_summary(
    summary: &LunarReferenceBatchParitySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing lunar mixed TT/TDB batch-parity summary string.
pub(crate) fn lunar_reference_batch_parity_summary_for_report() -> String {
    match pleiades_elp::lunar_reference_batch_parity_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format_lunar_reference_batch_parity_summary(&summary),
            Err(error) => {
                format!("lunar reference mixed TT/TDB batch parity: unavailable ({error})")
            }
        },
        None => "lunar reference mixed TT/TDB batch parity: unavailable".to_string(),
    }
}

/// Formats the lunar equatorial reference evidence summary for release-facing reporting.
pub(crate) fn format_lunar_equatorial_reference_evidence_summary(
    summary: &LunarEquatorialReferenceEvidenceSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing lunar equatorial reference evidence summary string.
pub(crate) fn lunar_equatorial_reference_evidence_summary_for_report() -> String {
    match pleiades_elp::lunar_equatorial_reference_evidence_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format_lunar_equatorial_reference_evidence_summary(&summary),
            Err(error) => format!("lunar equatorial reference evidence: unavailable ({error})"),
        },
        None => "lunar equatorial reference evidence: unavailable".to_string(),
    }
}

/// Formats the reference-only apparent Moon comparison summary for release-facing reporting.
pub(crate) fn format_lunar_apparent_comparison_summary(
    summary: &LunarApparentComparisonSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing one-line apparent comparison summary.
pub(crate) fn lunar_apparent_comparison_summary_for_report() -> String {
    match pleiades_elp::lunar_apparent_comparison_summary() {
        Some(summary) if summary.validate().is_ok() => {
            format_lunar_apparent_comparison_summary(&summary)
        }
        Some(_) | None => "lunar apparent comparison evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing lunar equatorial reference error envelope string.
///
/// Calls the retained [`pleiades_elp::LunarEquatorialReferenceEvidenceEnvelope::validated_summary_line`]
/// bridge instead of duplicating the envelope's validation logic, which stays
/// crate-private to `pleiades-elp` (report-surface relocation program, Slice B).
pub(crate) fn lunar_equatorial_reference_evidence_envelope_for_report() -> String {
    match pleiades_elp::lunar_equatorial_reference_evidence_envelope() {
        Some(envelope) => match envelope.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("lunar equatorial reference error envelope: unavailable ({error})")
            }
        },
        None => "lunar equatorial reference error envelope: unavailable".to_string(),
    }
}

/// Returns the release-facing lunar high-curvature continuity evidence string.
///
/// Calls the retained
/// [`pleiades_elp::LunarHighCurvatureContinuityEnvelope::validated_summary_line`]
/// bridge (via the now-`pub` `lunar_high_curvature_continuity_envelope`
/// constructor) instead of duplicating the envelope's validation logic, which
/// stays crate-private to `pleiades-elp` (report-surface relocation program,
/// Slice B).
pub(crate) fn lunar_high_curvature_continuity_evidence_for_report() -> String {
    match pleiades_elp::lunar_high_curvature_continuity_envelope() {
        Some(envelope) => match envelope.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(_) => "lunar high-curvature continuity evidence: unavailable".to_string(),
        },
        None => "lunar high-curvature continuity evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing lunar high-curvature equatorial continuity evidence string.
///
/// Calls the retained
/// [`pleiades_elp::LunarHighCurvatureEquatorialContinuityEnvelope::validated_summary_line`]
/// bridge (via the now-`pub` `lunar_high_curvature_equatorial_continuity_envelope`
/// constructor) instead of duplicating the envelope's validation logic, which
/// stays crate-private to `pleiades-elp` (report-surface relocation program,
/// Slice B).
pub(crate) fn lunar_high_curvature_equatorial_continuity_evidence_for_report() -> String {
    match pleiades_elp::lunar_high_curvature_equatorial_continuity_envelope() {
        Some(envelope) => match envelope.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(_) => {
                "lunar high-curvature equatorial continuity evidence: unavailable".to_string()
            }
        },
        None => "lunar high-curvature equatorial continuity evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing lunar reference error envelope string.
///
/// Calls the retained [`pleiades_elp::LunarReferenceEvidenceEnvelope::validated_summary_line`]
/// bridge instead of duplicating the envelope's validation logic, which stays
/// crate-private to `pleiades-elp` (report-surface relocation program, Slice B).
pub(crate) fn lunar_reference_evidence_envelope_for_report() -> String {
    match pleiades_elp::lunar_reference_evidence_envelope() {
        Some(envelope) => match envelope.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("lunar reference error envelope: unavailable ({error})")
            }
        },
        None => "lunar reference error envelope: unavailable".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lunar_equatorial_reference_batch_parity_summary_for_report_matches_the_summary_line() {
        let summary = pleiades_elp::lunar_equatorial_reference_batch_parity_summary()
            .expect("canonical equatorial batch-parity summary should be available");
        assert_eq!(
            format_lunar_equatorial_reference_batch_parity_summary(&summary),
            summary.summary_line()
        );
        assert_eq!(
            lunar_equatorial_reference_batch_parity_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn lunar_reference_evidence_summary_for_report_matches_the_summary_line() {
        let summary = pleiades_elp::lunar_reference_evidence_summary()
            .expect("canonical reference evidence summary should be available");
        assert_eq!(
            format_lunar_reference_evidence_summary(&summary),
            summary.summary_line()
        );
        assert_eq!(
            lunar_reference_evidence_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn lunar_source_window_summary_for_report_matches_the_summary_line() {
        let summary = pleiades_elp::lunar_source_window_summary()
            .expect("canonical source-window summary should be available");
        assert_eq!(
            format_lunar_source_window_summary(&summary),
            summary.summary_line()
        );
        assert_eq!(
            lunar_source_window_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(
            validated_lunar_source_window_summary_for_report().unwrap(),
            summary.summary_line()
        );
    }

    #[test]
    fn lunar_reference_batch_parity_summary_for_report_matches_the_summary_line() {
        let summary = pleiades_elp::lunar_reference_batch_parity_summary()
            .expect("canonical mixed TT/TDB batch-parity summary should be available");
        assert_eq!(
            format_lunar_reference_batch_parity_summary(&summary),
            summary.summary_line()
        );
        assert_eq!(
            lunar_reference_batch_parity_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn lunar_equatorial_reference_evidence_summary_for_report_matches_the_summary_line() {
        let summary = pleiades_elp::lunar_equatorial_reference_evidence_summary()
            .expect("canonical equatorial reference evidence summary should be available");
        assert_eq!(
            format_lunar_equatorial_reference_evidence_summary(&summary),
            summary.summary_line()
        );
        assert_eq!(
            lunar_equatorial_reference_evidence_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn lunar_apparent_comparison_summary_for_report_matches_the_summary_line() {
        let summary = pleiades_elp::lunar_apparent_comparison_summary()
            .expect("canonical apparent comparison summary should be available");
        assert_eq!(
            format_lunar_apparent_comparison_summary(&summary),
            summary.summary_line()
        );
        assert_eq!(
            lunar_apparent_comparison_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn lunar_equatorial_reference_evidence_envelope_for_report_matches_the_summary_line() {
        let envelope = pleiades_elp::lunar_equatorial_reference_evidence_envelope()
            .expect("canonical equatorial reference error envelope should be available");
        assert_eq!(
            lunar_equatorial_reference_evidence_envelope_for_report(),
            envelope.summary_line()
        );
    }

    #[test]
    fn lunar_high_curvature_continuity_evidence_for_report_matches_the_summary_line() {
        let envelope = pleiades_elp::lunar_high_curvature_continuity_envelope()
            .expect("canonical high-curvature continuity envelope should be available");
        assert_eq!(
            lunar_high_curvature_continuity_evidence_for_report(),
            envelope.validated_summary_line().unwrap()
        );
    }

    #[test]
    fn lunar_high_curvature_equatorial_continuity_evidence_for_report_matches_the_summary_line() {
        let envelope = pleiades_elp::lunar_high_curvature_equatorial_continuity_envelope()
            .expect("canonical high-curvature equatorial continuity envelope should be available");
        assert_eq!(
            lunar_high_curvature_equatorial_continuity_evidence_for_report(),
            envelope.validated_summary_line().unwrap()
        );
    }

    #[test]
    fn lunar_reference_evidence_envelope_for_report_matches_the_summary_line() {
        let envelope = pleiades_elp::lunar_reference_evidence_envelope()
            .expect("canonical reference error envelope should be available");
        assert_eq!(
            lunar_reference_evidence_envelope_for_report(),
            envelope.summary_line()
        );
    }
}
