//! Packaged-artifact and source-audit policy summaries.

use crate::*;

pub(crate) fn format_vsop87_request_policy_summary() -> String {
    crate::posture::vsop87::spec::vsop87_request_policy_summary_for_report()
}

pub(crate) fn format_vsop87_source_audit_summary() -> String {
    crate::posture::vsop87::audit::source_audit_summary_for_report()
}

pub(crate) fn format_packaged_artifact_profile_summary() -> String {
    packaged_artifact_profile_summary_with_body_coverage()
}

pub(crate) fn validated_packaged_artifact_output_support_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_output_support_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact output support: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_speed_policy_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_speed_policy_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact speed policy: unavailable ({error})"),
    }
}

pub(crate) fn validated_motion_policy_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_speed_policy_summary_details();
    match summary.validate() {
        Ok(()) => format!("Motion policy: {}", summary.summary_line()),
        Err(error) => format!("Motion policy: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_access_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_access_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact access: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_generation_policy_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_generation_policy_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact generation policy: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_body_cadence_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_body_cadence_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("body cadence: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_body_class_span_cap_summary_for_report() -> String {
    format!(
        "Packaged-artifact body-class span caps: {}",
        pleiades_data::packaged_artifact_body_class_span_cap_entries_for_report()
    )
}

pub(crate) fn validated_packaged_artifact_normalized_intermediate_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_normalized_intermediate_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged artifact normalized intermediates: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_storage_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_storage_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged-artifact storage/reconstruction: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_frame_treatment_summary_for_report() -> String {
    let summary = pleiades_data::packaged_frame_treatment_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged frame treatment: unavailable ({error})"),
    }
}

pub(crate) fn ensure_packaged_artifact_storage_summary_matches_current_rendering(
    packaged_artifact_storage_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_storage_summary_text
        == validated_packaged_artifact_storage_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact storage summary no longer matches the current packaged-artifact storage posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_frame_treatment_summary_matches_current_rendering(
    packaged_frame_treatment_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_frame_treatment_summary_text
        == validated_packaged_frame_treatment_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged frame treatment summary no longer matches the current packaged frame treatment posture"
                .to_string(),
        ))
    }
}

pub(crate) fn validated_packaged_artifact_target_threshold_state_for_report() -> String {
    pleiades_data::packaged_artifact_target_threshold_state_for_report()
}

pub(crate) fn validated_packaged_artifact_target_threshold_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_target_threshold_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged-artifact target thresholds: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_target_threshold_scope_envelopes_summary_for_report(
) -> String {
    let summary =
        pleiades_data::packaged_artifact_target_threshold_scope_envelopes_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("scope envelopes: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_source_fit_holdout_sync_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_source_fit_holdout_sync_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("source-fit and hold-out sync: unavailable ({error})"),
    }
}

pub(crate) fn validated_packaged_artifact_phase2_corpus_alignment_summary_for_report() -> String {
    let summary = match pleiades_data::packaged_artifact_phase2_corpus_alignment_summary_details()
    {
        Some(summary) => summary,
        None => {
            return "Packaged-artifact phase-2 corpus alignment: unavailable (phase-2 corpus evidence should be available)".to_string()
        }
    };

    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged-artifact phase-2 corpus alignment: unavailable ({error})"),
    }
}

pub(crate) fn format_packaged_artifact_output_support_summary() -> String {
    validated_packaged_artifact_output_support_summary_for_report()
}

pub(crate) fn format_packaged_artifact_speed_policy_summary() -> String {
    validated_packaged_artifact_speed_policy_summary_for_report()
}

pub(crate) fn format_packaged_artifact_generation_policy_summary() -> String {
    validated_packaged_artifact_generation_policy_summary_for_report()
}

pub(crate) fn validate_packaged_artifact_generation_residual_bodies_summary(
    summary: &pleiades_compression::ArtifactResidualBodyCoverageSummary,
    artifact: &pleiades_compression::CompressedArtifact,
) -> Result<String, String> {
    summary
        .validated_summary_line_with_body_count(artifact)
        .map_err(|error| error.to_string())
}

pub(crate) fn validated_packaged_artifact_generation_residual_bodies_summary_for_report(
) -> Result<String, String> {
    validate_packaged_artifact_generation_residual_bodies_summary(
        &pleiades_data::packaged_artifact_generation_residual_bodies_summary_details(),
        packaged_artifact(),
    )
}

pub(crate) fn validated_packaged_artifact_production_profile_summary_for_report() -> String {
    let summary = pleiades_data::packaged_artifact_production_profile_summary_details();
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged artifact production profile draft: unavailable ({error})"),
    }
}

pub(crate) fn format_packaged_artifact_storage_summary() -> String {
    validated_packaged_artifact_storage_summary_for_report()
}

pub(crate) fn format_packaged_artifact_access_summary() -> String {
    validated_packaged_artifact_access_summary_for_report()
}

pub(crate) fn format_packaged_frame_parity_summary() -> String {
    packaged_frame_parity_summary_for_report()
}

pub(crate) fn format_lunar_frame_treatment_summary() -> String {
    crate::posture::elp::lib_summaries::lunar_theory_frame_treatment_summary_for_report()
}

pub(crate) fn format_packaged_frame_treatment_summary() -> String {
    packaged_frame_treatment_summary_for_report()
}

pub(crate) fn format_comparison_snapshot_manifest_summary() -> String {
    match validated_comparison_snapshot_manifest_summary_for_report() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("Comparison snapshot manifest: unavailable ({error})"),
    }
}
