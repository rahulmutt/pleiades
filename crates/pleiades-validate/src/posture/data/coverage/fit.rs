//! Relocated from `pleiades-data` (report-surface relocation program, Slice C).

use pleiades_data::packaged_artifact_fit_channel_outlier_summary_details;

/// Returns the current packaged-artifact fit outliers by channel after validating the structured posture.
pub(crate) fn packaged_artifact_fit_channel_outlier_summary_for_report() -> String {
    let summary = packaged_artifact_fit_channel_outlier_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("fit outliers by channel: unavailable ({error})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
    #[test]
    fn fit_channel_outlier_summary_for_report_matches_details() {
        let by_channel_summary =
            pleiades_data::packaged_artifact_fit_channel_outlier_summary_details();
        let by_channel = by_channel_summary.summary_line();
        assert_eq!(
            packaged_artifact_fit_channel_outlier_summary_for_report(),
            by_channel
        );
    }
}
