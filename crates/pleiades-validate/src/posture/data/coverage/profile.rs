//! Relocated from `pleiades-data` (report-surface relocation program, Slice C).

use std::sync::OnceLock;

use pleiades_data::{
    packaged_artifact_generation_manifest_details, packaged_artifact_output_support_summary_details,
    packaged_artifact_production_profile_summary_details, packaged_artifact_profile_summary_details,
    packaged_artifact_profile_summary_with_output_support,
    packaged_artifact_speed_policy_summary_details,
};

/// Returns the current packaged-artifact production-profile draft after validating the structured posture.
pub(crate) fn packaged_artifact_production_profile_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_production_profile_summary_details();
            match summary.validated_summary_line() {
                Ok(line) => line,
                Err(error) => {
                    format!("Packaged artifact production profile draft: unavailable ({error})")
                }
            }
        })
        .clone()
}

/// Returns the current deterministic packaged-artifact generation manifest after validation.
pub(crate) fn packaged_artifact_generation_manifest_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let manifest = packaged_artifact_generation_manifest_details();
            match manifest.validated_summary_line() {
                Ok(line) => line,
                Err(error) => {
                    format!("Packaged artifact generation manifest: unavailable ({error})")
                }
            }
        })
        .clone()
}

/// Returns the current deterministic packaged-artifact generation manifest checksum after validation.
pub(crate) fn packaged_artifact_generation_manifest_checksum_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let manifest = packaged_artifact_generation_manifest_details();
            match manifest.validate() {
                Ok(()) => format!(
                    "Packaged artifact generation manifest checksum: 0x{:016x}",
                    manifest.manifest_checksum
                ),
                Err(error) => {
                    format!("Packaged artifact generation manifest checksum: unavailable ({error})")
                }
            }
        })
        .clone()
}

/// Returns the current packaged-artifact profile coverage summary for reporting.
pub(crate) fn packaged_artifact_profile_coverage_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_profile_summary_details();
            match summary.validate() {
                Ok(()) => match summary
                    .profile_coverage_summary()
                    .validated_summary_line_with_bodies()
                {
                    Ok(line) => line,
                    Err(error) => format!("Artifact profile coverage: unavailable ({error})"),
                },
                Err(error) => format!("Artifact profile coverage: unavailable ({error})"),
            }
        })
        .clone()
}

/// Returns the current packaged-artifact profile summary with output support for reporting.
pub(crate) fn packaged_artifact_profile_summary_with_output_support_for_report() -> String {
    packaged_artifact_profile_summary_with_output_support()
}

/// Returns the output-support semantics of the packaged artifact profile for reporting.
pub(crate) fn packaged_artifact_output_support_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_output_support_summary_details();
            match summary.validated_summary_line() {
                Ok(rendered) => rendered,
                Err(error) => format!("unavailable ({error})"),
            }
        })
        .clone()
}

/// Returns the packaged-artifact speed-policy semantics for reporting.
pub(crate) fn packaged_artifact_speed_policy_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_speed_policy_summary_details();
            match summary.validate() {
                Ok(()) => summary.summary_line(),
                Err(error) => format!("unavailable ({error})"),
            }
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_support_summary_for_report_matches_profile_posture() {
        let summary = pleiades_data::packaged_artifact_profile_summary_details();
        assert_eq!(
            packaged_artifact_output_support_summary_for_report(),
            summary.profile.output_support_summary_line()
        );
        let output_support_summary = pleiades_data::packaged_artifact_output_support_summary_details();
        assert_eq!(
            packaged_artifact_output_support_summary_for_report(),
            output_support_summary.summary_line()
        );
    }

    #[test]
    fn profile_coverage_summary_for_report_matches_profile_posture() {
        let summary = pleiades_data::packaged_artifact_profile_summary_details();
        assert_eq!(
            packaged_artifact_profile_coverage_summary_for_report(),
            summary
                .profile_coverage_summary()
                .summary_line_with_bodies()
        );
    }

    #[test]
    fn profile_summary_with_output_support_for_report_matches_profile_posture() {
        let summary = pleiades_data::packaged_artifact_profile_summary_details();
        assert_eq!(
            packaged_artifact_profile_summary_with_output_support_for_report(),
            summary.summary_line_with_output_support()
        );
    }

    #[test]
    fn production_profile_summary_for_report_reflects_the_current_posture() {
        assert!(packaged_artifact_production_profile_summary_for_report()
            .contains("Packaged artifact production profile draft:"));
    }

    #[test]
    fn speed_policy_summary_for_report_reflects_the_current_posture() {
        let summary = pleiades_data::packaged_artifact_speed_policy_summary_details();
        assert_eq!(
            packaged_artifact_speed_policy_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn generation_manifest_for_report_reflects_the_current_posture() {
        assert!(packaged_artifact_generation_manifest_for_report()
            .contains("Packaged artifact generation manifest:"));
    }
}
