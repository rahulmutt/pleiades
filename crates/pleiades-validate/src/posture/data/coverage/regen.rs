//! Relocated from `pleiades-data` (report-surface relocation program, Slice C).

use std::sync::OnceLock;

use pleiades_data::{
    packaged_artifact_normalized_intermediate_summary_details,
    packaged_artifact_regeneration_summary_details,
};

/// Returns the packaged-artifact regeneration provenance summary after validating
/// the structured source and coverage metadata.
pub fn packaged_artifact_regeneration_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_regeneration_summary_details();
            match summary.validated_summary_line() {
                Ok(line) => line,
                Err(error) => {
                    format!("Packaged artifact regeneration source: unavailable ({error})")
                }
            }
        })
        .clone()
}

/// Returns the normalized-intermediate provenance summary.
pub fn packaged_artifact_normalized_intermediate_summary_for_report() -> String {
    let summary = packaged_artifact_normalized_intermediate_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged artifact normalized intermediates: unavailable ({error})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
    #[test]
    fn normalized_intermediate_summary_for_report_matches_regeneration_details() {
        let summary = pleiades_data::packaged_artifact_regeneration_summary_details();
        assert_eq!(
            summary.normalized_intermediates.summary_line(),
            packaged_artifact_normalized_intermediate_summary_for_report()
        );
    }
}
