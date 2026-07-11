//! Relocated from `pleiades-data` (report-surface relocation program, Slice C).

use pleiades_data::{packaged_body_coverage_summary_details, PackagedBodyCoverageSummary};

pub(crate) fn format_validated_packaged_body_coverage_summary_for_report(
    summary: &PackagedBodyCoverageSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Packaged body set: unavailable ({error})"),
    }
}

/// Returns the packaged body set as a human-readable provenance summary.
pub(crate) fn packaged_body_coverage_summary() -> String {
    format_validated_packaged_body_coverage_summary_for_report(
        &packaged_body_coverage_summary_details(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaged_body_coverage_summary_matches_the_packaged_body_set() {
        let summary = pleiades_data::packaged_body_coverage_summary_details();
        assert_eq!(packaged_body_coverage_summary(), summary.to_string());
    }

    #[test]
    fn packaged_body_coverage_summary_report_marks_drift_as_unavailable() {
        let mut summary = pleiades_data::packaged_body_coverage_summary_details();
        summary.bodies.swap(0, 1);

        assert_eq!(
            format_validated_packaged_body_coverage_summary_for_report(&summary),
            "Packaged body set: unavailable (the packaged body coverage summary field `bodies` is out of sync with the current bundled body set)"
        );
    }

    #[test]
    fn packaged_body_coverage_summary_matches_backend_metadata_provenance() {
        use pleiades_core::EphemerisBackend;
        let metadata = pleiades_data::packaged_backend().metadata();
        assert_eq!(
            packaged_body_coverage_summary(),
            metadata.provenance.data_sources[0]
        );
    }
}
