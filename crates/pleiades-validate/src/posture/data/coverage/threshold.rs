//! Relocated from `pleiades-data` (report-surface relocation program, Slice C).

use pleiades_data::{
    packaged_artifact_fit_envelope_summary_details, packaged_artifact_fit_margin_summary_details,
    packaged_artifact_fit_outlier_summary_details, packaged_artifact_fit_threshold_summary_details,
    packaged_artifact_fit_threshold_violation_summary_details,
};

/// Returns the current packaged-artifact fit envelope after validating the structured posture.
pub(crate) fn packaged_artifact_fit_envelope_summary_for_report() -> String {
    let summary = packaged_artifact_fit_envelope_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("fit envelope: unavailable ({error})"),
    }
}

/// Returns the current packaged-artifact body/channel fit outlier summary after validating the structured posture.
pub(crate) fn packaged_artifact_fit_outlier_summary_for_report() -> String {
    let summary = packaged_artifact_fit_outlier_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("fit outliers: unavailable ({error})"),
    }
}

/// Returns the current packaged-artifact fit thresholds after validating the structured posture.
pub(crate) fn packaged_artifact_fit_threshold_summary_for_report() -> String {
    let summary = packaged_artifact_fit_threshold_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("fit thresholds: unavailable ({error})"),
    }
}

/// Returns the current packaged-artifact fit margins relative to the calibrated thresholds after validating the structured posture.
pub(crate) fn packaged_artifact_fit_margin_summary_for_report() -> String {
    let summary = packaged_artifact_fit_margin_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("fit margins: unavailable ({error})"),
    }
}

/// Returns the number of packaged-artifact fit threshold violations relative to the calibrated thresholds.
pub(crate) fn packaged_artifact_fit_threshold_violation_count_for_report() -> String {
    let summary = packaged_artifact_fit_threshold_violation_summary_details();

    match summary.validate() {
        Ok(()) => format!("fit threshold violations: {}", summary.violations.len()),
        Err(error) => format!("fit threshold violations: unavailable ({error})"),
    }
}

/// Returns the packaged-artifact fit threshold violations with field-level context.
pub(crate) fn packaged_artifact_fit_threshold_violation_summary_for_report() -> String {
    let summary = packaged_artifact_fit_threshold_violation_summary_details();

    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("fit threshold violations: unavailable ({error})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_threshold_summary_for_report_reflects_the_current_posture() {
        assert!(packaged_artifact_fit_threshold_summary_for_report()
            .contains("fit thresholds: mean Δlon≤79.299372815190°"));
        assert_eq!(
            packaged_artifact_fit_threshold_violation_count_for_report(),
            "fit threshold violations: 0"
        );
        assert_eq!(
            packaged_artifact_fit_threshold_violation_summary_for_report(),
            "fit threshold violations: 0; details: none"
        );
    }

    #[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
    #[test]
    fn fit_margin_summary_for_report_matches_details() {
        let summary = pleiades_data::packaged_artifact_fit_margin_summary_details();
        assert_eq!(
            summary.summary_line(),
            packaged_artifact_fit_margin_summary_for_report()
        );
    }
}
