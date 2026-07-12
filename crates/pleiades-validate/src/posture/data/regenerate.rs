//! Relocated from `pleiades-data` (report-surface relocation program, Slice C).
//!
//! Rendering-only prose for packaged-artifact body-class span-cap and body-cadence
//! posture. The functional crate keeps the structured summary records, their
//! constructors, and inherent methods; the regeneration pipeline also stays.

use pleiades_compression::join_display;
use pleiades_data::{
    packaged_artifact_body_cadence_summary_details,
    packaged_artifact_body_class_span_cap_summary_details, PackagedArtifactBodyCadenceSummary,
    PackagedArtifactBodyClassSpanCapSummary,
};

/// Returns the current packaged-artifact body-class span caps after validating the structured posture.
pub(crate) fn packaged_artifact_body_class_span_cap_summary_for_report() -> String {
    let summary = packaged_artifact_body_class_span_cap_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("body-class span caps: unavailable ({error})"),
    }
}

/// Reconstructs the packaged-artifact body-class span-cap entry line from the
/// retained structured record. Mirrors `pleiades-data`'s private
/// `PackagedArtifactBodyClassSpanCapSummary::entries_summary_line` (which stays
/// private) using the public `entries` field, byte-for-byte.
fn entries_summary_line(summary: &PackagedArtifactBodyClassSpanCapSummary) -> String {
    let entries = summary
        .entries
        .iter()
        .map(|(label, days)| format!("{label}={days:.0} days"))
        .collect::<Vec<_>>();

    join_display(&entries)
}

/// Returns the current packaged-artifact body-class span-cap entries after validating the structured posture.
pub(crate) fn packaged_artifact_body_class_span_cap_entries_for_report() -> String {
    let summary = packaged_artifact_body_class_span_cap_summary_details();
    match summary.validated_summary_line() {
        Ok(_) => entries_summary_line(&summary),
        Err(error) => format!("unavailable ({error})"),
    }
}

fn render_packaged_artifact_body_cadence_summary(
    summary: &PackagedArtifactBodyCadenceSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("body cadence: unavailable ({error})"),
    }
}

/// Returns the current packaged-artifact body cadence as a compact human-readable line.
pub(crate) fn packaged_artifact_body_cadence_summary_for_report() -> String {
    render_packaged_artifact_body_cadence_summary(&packaged_artifact_body_cadence_summary_details())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_class_span_cap_summary_report_matches_retained_details() {
        // Relocated from `pleiades-data` `tests/coverage.rs`.
        assert_eq!(
            packaged_artifact_body_class_span_cap_summary_for_report(),
            packaged_artifact_body_class_span_cap_summary_details().to_string()
        );
    }

    #[test]
    fn body_class_span_cap_entries_report_pins_current_string() {
        // Relocated from `pleiades-data` `tests/coverage.rs`.
        assert_eq!(
            packaged_artifact_body_class_span_cap_entries_for_report(),
            "luminaries=256 days, inner planets=384 days, outer planets=768 days, pluto=1536 days, lunar points=256 days, selected asteroids=256 days, custom bodies=512 days"
        );
    }

    #[test]
    fn body_cadence_summary_report_matches_retained_details() {
        // Relocated from `pleiades-data` `tests/coverage.rs`.
        assert_eq!(
            packaged_artifact_body_cadence_summary_for_report(),
            packaged_artifact_body_cadence_summary_details().summary_line()
        );
    }
}
