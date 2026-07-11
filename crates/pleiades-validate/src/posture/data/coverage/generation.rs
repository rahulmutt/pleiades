//! Relocated from `pleiades-data` (report-surface relocation program, Slice C).

use std::sync::OnceLock;

use pleiades_data::{
    packaged_artifact, packaged_artifact_generation_policy_summary_details,
    packaged_artifact_generation_residual_bodies_summary_details,
};

/// Returns the current packaged-artifact generation policy summary after validating the structured posture.
pub(crate) fn packaged_artifact_generation_policy_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_generation_policy_summary_details();
            match summary.validate() {
                Ok(()) => summary.to_string(),
                Err(error) => format!("Packaged-artifact generation policy: unavailable ({error})"),
            }
        })
        .clone()
}

/// Returns the current packaged-artifact residual-bearing body set after validating the structured posture.
pub(crate) fn packaged_artifact_generation_residual_bodies_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let artifact = packaged_artifact();
            let summary = packaged_artifact_generation_residual_bodies_summary_details();

            match summary.validated_summary_line_with_body_count(artifact) {
                Ok(line) => line,
                Err(error) => format!("residual bodies: unavailable ({error})"),
            }
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
    #[test]
    fn generation_residual_bodies_summary_for_report_matches_details() {
        let residual_bodies =
            pleiades_data::packaged_artifact_generation_residual_bodies_summary_details();
        assert_eq!(
            packaged_artifact_generation_residual_bodies_summary_for_report(),
            residual_bodies.summary_line_with_body_count()
        );
    }

    #[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
    #[test]
    fn generation_residual_bodies_summary_for_report_appears_in_regeneration_provenance() {
        let summary = pleiades_data::packaged_artifact_regeneration_summary_details();
        assert_eq!(
            summary.residual_body_line(),
            packaged_artifact_generation_residual_bodies_summary_for_report()
        );
        let provenance = summary.summary_line();
        assert!(
            provenance.contains(&packaged_artifact_generation_residual_bodies_summary_for_report())
        );
    }
}
