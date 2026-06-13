//! Release-bundle verification: rendering-alignment checks and bundle audit.

use std::path::Path;

use super::bundle::*;
use super::bundle_manifest::*;
use crate::*;

pub(crate) fn ensure_packaged_artifact_phase2_alignment_matches_source_fit_holdout_sync(
    source_fit_holdout_sync_summary_text: &str,
    phase2_corpus_alignment_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    let source_fit_holdout_sync_payload = extract_single_summary_payload(
        source_fit_holdout_sync_summary_text,
        "Packaged-artifact source-fit and hold-out sync: ",
    )?;
    let phase2_corpus_alignment_payload = extract_single_summary_payload(
        phase2_corpus_alignment_summary_text,
        "Packaged-artifact phase-2 corpus alignment: ",
    )?;
    let Some((_, embedded_phase2_corpus_alignment_payload)) =
        source_fit_holdout_sync_payload.rsplit_once("phase 2 corpus alignment=")
    else {
        return Err(ReleaseBundleError::Verification(
            "packaged-artifact source-fit and hold-out sync summary is missing its phase-2 corpus alignment payload".to_string(),
        ));
    };

    if embedded_phase2_corpus_alignment_payload != phase2_corpus_alignment_payload {
        return Err(ReleaseBundleError::Verification(
            "packaged-artifact source-fit and hold-out sync summary phase-2 corpus alignment payload does not match the packaged-artifact phase-2 corpus alignment summary".to_string(),
        ));
    }

    Ok(())
}

pub(crate) fn ensure_packaged_artifact_target_threshold_phase2_alignment_matches_source_fit_holdout_sync(
    target_threshold_summary_text: &str,
    source_fit_holdout_sync_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    let target_threshold_payload = extract_single_summary_payload(
        target_threshold_summary_text,
        "Packaged-artifact target thresholds: ",
    )?;
    let source_fit_holdout_sync_payload = extract_single_summary_payload(
        source_fit_holdout_sync_summary_text,
        "Packaged-artifact source-fit and hold-out sync: ",
    )?;

    let Some((_, target_phase2_payload)) =
        target_threshold_payload.rsplit_once("phase 2 corpus alignment=")
    else {
        return Err(ReleaseBundleError::Verification(
            "packaged-artifact target-threshold summary is missing its phase-2 corpus alignment payload".to_string(),
        ));
    };
    let Some((_, sync_phase2_payload)) =
        source_fit_holdout_sync_payload.rsplit_once("phase 2 corpus alignment=")
    else {
        return Err(ReleaseBundleError::Verification(
            "packaged-artifact source-fit and hold-out sync summary is missing its phase-2 corpus alignment payload".to_string(),
        ));
    };

    if target_phase2_payload != sync_phase2_payload {
        return Err(ReleaseBundleError::Verification(
            "packaged-artifact target-threshold summary phase-2 corpus alignment payload does not match the packaged-artifact source-fit and hold-out sync summary".to_string(),
        ));
    }

    Ok(())
}

pub(crate) fn ensure_packaged_artifact_phase2_corpus_alignment_summary_matches_current_rendering(
    packaged_artifact_phase2_corpus_alignment_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_phase2_corpus_alignment_summary_text
        == validated_packaged_artifact_phase2_corpus_alignment_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact phase-2 corpus alignment summary no longer matches the current packaged-artifact phase-2 corpus alignment posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_production_generation_source_summary_matches_source_windows(
    production_generation_source_summary_text: &str,
    production_generation_source_window_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    fn summary_payload<'a>(text: &'a str, prefix: &str) -> Result<&'a str, ReleaseBundleError> {
        if let Some(payload) = text
            .lines()
            .filter_map(|line| line.strip_prefix(prefix))
            .next()
        {
            if text
                .lines()
                .filter_map(|line| line.strip_prefix(prefix))
                .nth(1)
                .is_some()
            {
                return Err(ReleaseBundleError::Verification(format!(
                    "duplicate entry: {prefix}"
                )));
            }
            if payload != payload.trim() {
                return Err(ReleaseBundleError::Verification(format!(
                    "unexpected leading or trailing whitespace in manifest entry: {prefix}"
                )));
            }
            return Ok(payload);
        }

        if text.lines().count() != 1 {
            return Err(ReleaseBundleError::Verification(format!(
                "missing manifest entry: {prefix}"
            )));
        }
        if text != text.trim() {
            return Err(ReleaseBundleError::Verification(format!(
                "unexpected leading or trailing whitespace in manifest entry: {prefix}"
            )));
        }

        Ok(text)
    }

    let production_generation_source_summary_payload = summary_payload(
        production_generation_source_summary_text,
        "Production generation source: ",
    )?;
    let production_generation_source_window_summary_payload = summary_payload(
        production_generation_source_window_summary_text,
        "Production generation source windows: ",
    )?;
    let expected_source_window_fragment = format!(
        "source windows={}",
        production_generation_source_window_summary_payload.trim(),
    );

    if !production_generation_source_summary_payload.contains(&expected_source_window_fragment) {
        return Err(ReleaseBundleError::Verification(
            "production generation source summary source windows payload does not match the production generation source window summary".to_string(),
        ));
    }

    Ok(())
}

pub(crate) fn ensure_production_generation_source_window_summary_matches_current_rendering(
    production_generation_source_window_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if production_generation_source_window_summary_text
        == pleiades_jpl::validated_production_generation_snapshot_window_summary_for_report()
            .map_err(ReleaseBundleError::Verification)?
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "production generation source window summary no longer matches the current production-generation source-window posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_production_generation_quarter_day_boundary_summary_matches_current_rendering(
    production_generation_quarter_day_boundary_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if production_generation_quarter_day_boundary_summary_text
        == pleiades_jpl::production_generation_quarter_day_boundary_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "production generation quarter-day boundary summary no longer matches the current production-generation quarter-day boundary posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_independent_holdout_source_window_summary_matches_current_rendering(
    independent_holdout_source_window_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if independent_holdout_source_window_summary_text
        == independent_holdout_snapshot_source_window_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "independent-holdout source window summary no longer matches the current independent-holdout source-window posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_independent_holdout_quarter_day_boundary_summary_matches_current_rendering(
    independent_holdout_quarter_day_boundary_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if independent_holdout_quarter_day_boundary_summary_text
        == independent_holdout_snapshot_quarter_day_boundary_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "independent-holdout quarter-day boundary summary no longer matches the current independent-holdout quarter-day boundary posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_independent_holdout_equatorial_parity_summary_matches_current_rendering(
    independent_holdout_equatorial_parity_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if independent_holdout_equatorial_parity_summary_text
        == jpl_independent_holdout_snapshot_equatorial_parity_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "independent-holdout equatorial parity summary no longer matches the current independent-holdout equatorial parity posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_independent_holdout_body_class_coverage_summary_matches_current_rendering(
    independent_holdout_body_class_coverage_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if independent_holdout_body_class_coverage_summary_text
        == independent_holdout_snapshot_body_class_coverage_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "independent-holdout body-class coverage summary no longer matches the current independent-holdout body-class coverage posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_production_generation_source_summary_matches_current_rendering(
    production_generation_source_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if production_generation_source_summary_text
        == pleiades_jpl::validated_production_generation_source_summary_for_report()
            .map_err(ReleaseBundleError::Verification)?
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "production generation source summary no longer matches the current production-generation source posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_production_generation_source_revision_summary_matches_current_rendering(
    production_generation_source_revision_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if production_generation_source_revision_summary_text
        == validated_production_generation_source_revision_summary_for_report()
            .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "production generation source revision summary no longer matches the current production-generation source-revision posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_production_generation_manifest_summary_matches_current_rendering(
    production_generation_manifest_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if production_generation_manifest_summary_text
        == validated_production_generation_manifest_summary_text_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "production generation manifest summary no longer matches the current production-generation manifest posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_production_generation_manifest_checksum_summary_matches_current_rendering(
    production_generation_manifest_checksum_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if production_generation_manifest_checksum_summary_text
        == production_generation_manifest_checksum_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "production generation manifest checksum summary no longer matches the current production-generation manifest checksum posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_production_generation_boundary_source_summary_matches_current_rendering(
    production_generation_boundary_source_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if production_generation_boundary_source_summary_text
        == production_generation_boundary_source_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "production generation boundary source summary no longer matches the current production-generation boundary source posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_production_generation_boundary_window_summary_matches_current_rendering(
    production_generation_boundary_window_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if production_generation_boundary_window_summary_text
        == production_generation_boundary_window_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "production generation boundary window summary no longer matches the current production-generation boundary window posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_production_generation_boundary_request_corpus_summary_matches_current_rendering(
    production_generation_boundary_request_corpus_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if production_generation_boundary_request_corpus_summary_text
        == production_generation_boundary_request_corpus_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "production generation boundary request corpus summary no longer matches the current production-generation boundary request corpus posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_production_generation_boundary_request_corpus_equatorial_summary_matches_current_rendering(
    production_generation_boundary_request_corpus_equatorial_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if production_generation_boundary_request_corpus_equatorial_summary_text
        == production_generation_boundary_request_corpus_equatorial_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "production generation boundary request corpus equatorial summary no longer matches the current production-generation boundary request corpus equatorial posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_production_generation_summary_matches_current_rendering(
    production_generation_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if production_generation_summary_text == production_generation_snapshot_summary_for_report() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "production generation summary no longer matches the current production-generation posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_production_generation_body_class_coverage_summary_matches_current_rendering(
    production_generation_body_class_coverage_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if production_generation_body_class_coverage_summary_text
        == validated_production_generation_body_class_coverage_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "production generation body-class coverage summary no longer matches the current production-generation body-class coverage posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_holdout_overlap_summary_matches_current_rendering(
    reference_holdout_overlap_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_holdout_overlap_summary_text
        == validated_reference_holdout_overlap_summary_for_report()
            .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference/hold-out overlap summary no longer matches the current reference/hold-out overlap posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_snapshot_bridge_day_summary_matches_current_rendering(
    reference_snapshot_bridge_day_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_snapshot_bridge_day_summary_text
        == reference_snapshot_bridge_day_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference snapshot bridge day summary no longer matches the current reference snapshot bridge day posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_snapshot_2451917_major_body_boundary_summary_matches_current_rendering(
    reference_snapshot_2451917_major_body_boundary_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_snapshot_2451917_major_body_boundary_summary_text
        == reference_snapshot_2451917_major_body_boundary_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference snapshot 2451917 major-body boundary summary no longer matches the current reference snapshot 2451917 major-body boundary posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_snapshot_2451918_major_body_boundary_summary_matches_current_rendering(
    reference_snapshot_2451918_major_body_boundary_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_snapshot_2451918_major_body_boundary_summary_text
        == reference_snapshot_2451918_major_body_boundary_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference snapshot 2451918 major-body boundary summary no longer matches the current reference snapshot 2451918 major-body boundary posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_snapshot_2451919_major_body_boundary_summary_matches_current_rendering(
    reference_snapshot_2451919_major_body_boundary_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_snapshot_2451919_major_body_boundary_summary_text
        == reference_snapshot_2451919_major_body_boundary_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference snapshot 2451919 major-body boundary summary no longer matches the current reference snapshot 2451919 major-body boundary posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_snapshot_major_body_boundary_window_summary_matches_current_rendering(
    reference_snapshot_major_body_boundary_window_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_snapshot_major_body_boundary_window_summary_text
        == reference_snapshot_major_body_boundary_window_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference snapshot major-body boundary window summary no longer matches the current reference snapshot major-body boundary window posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_snapshot_boundary_epoch_coverage_summary_matches_current_rendering(
    reference_snapshot_boundary_epoch_coverage_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_snapshot_boundary_epoch_coverage_summary_text
        == reference_snapshot_boundary_epoch_coverage_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference snapshot boundary epoch coverage summary no longer matches the current reference snapshot boundary epoch coverage posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_snapshot_pre_bridge_boundary_summary_matches_current_rendering(
    reference_snapshot_pre_bridge_boundary_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_snapshot_pre_bridge_boundary_summary_text
        == reference_snapshot_pre_bridge_boundary_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference snapshot pre-bridge boundary summary no longer matches the current reference snapshot pre-bridge boundary posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_snapshot_sparse_boundary_summary_matches_current_rendering(
    reference_snapshot_sparse_boundary_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_snapshot_sparse_boundary_summary_text
        == reference_snapshot_sparse_boundary_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference snapshot sparse boundary summary no longer matches the current reference snapshot sparse boundary posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_snapshot_summary_matches_current_rendering(
    reference_snapshot_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_snapshot_summary_text == reference_snapshot_summary_for_report() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference snapshot summary no longer matches the current reference snapshot coverage"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_snapshot_equatorial_parity_summary_matches_current_rendering(
    reference_snapshot_equatorial_parity_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_snapshot_equatorial_parity_summary_text
        == reference_snapshot_equatorial_parity_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference snapshot equatorial parity summary no longer matches the current reference snapshot equatorial parity posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_snapshot_exact_j2000_evidence_summary_matches_current_rendering(
    reference_snapshot_exact_j2000_evidence_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_snapshot_exact_j2000_evidence_summary_text
        == reference_snapshot_exact_j2000_evidence_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference snapshot exact J2000 evidence summary no longer matches the current reference snapshot exact J2000 evidence posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_snapshot_manifest_summary_matches_current_rendering(
    reference_snapshot_manifest_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_snapshot_manifest_summary_text == reference_snapshot_manifest_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference snapshot manifest summary no longer matches the current reference snapshot manifest posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_snapshot_source_summary_matches_current_rendering(
    reference_snapshot_source_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_snapshot_source_summary_text == reference_snapshot_source_summary_for_report() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference snapshot source summary no longer matches the current reference snapshot source posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_snapshot_body_class_coverage_summary_matches_current_rendering(
    reference_snapshot_body_class_coverage_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_snapshot_body_class_coverage_summary_text
        == reference_snapshot_body_class_coverage_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference snapshot body-class coverage summary no longer matches the current reference snapshot body-class coverage posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_snapshot_source_window_summary_matches_current_rendering(
    reference_snapshot_source_window_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_snapshot_source_window_summary_text
        == pleiades_jpl::validated_reference_snapshot_source_window_summary_for_report()
            .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference snapshot source window summary no longer matches the current reference snapshot source-window posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_asteroid_source_window_summary_matches_current_rendering(
    reference_asteroid_source_window_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_asteroid_source_window_summary_text
        == reference_asteroid_source_window_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference asteroid source window summary no longer matches the current reference asteroid source-window posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_reference_asteroid_equatorial_evidence_summary_matches_current_rendering(
    reference_asteroid_equatorial_evidence_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if reference_asteroid_equatorial_evidence_summary_text
        == reference_asteroid_equatorial_evidence_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "reference asteroid equatorial evidence summary no longer matches the current reference asteroid equatorial evidence posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_comparison_snapshot_source_summary_matches_current_rendering(
    comparison_snapshot_source_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_snapshot_source_summary_text == comparison_snapshot_source_summary_for_report() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "comparison snapshot source summary no longer matches the current comparison snapshot source posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_comparison_snapshot_manifest_summary_matches_current_rendering(
    comparison_snapshot_manifest_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_snapshot_manifest_summary_text
        == validated_comparison_snapshot_manifest_summary_for_report()
            .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "comparison snapshot manifest summary no longer matches the current comparison snapshot manifest posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_comparison_snapshot_source_window_summary_matches_current_rendering(
    comparison_snapshot_source_window_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_snapshot_source_window_summary_text
        == pleiades_jpl::validated_comparison_snapshot_source_window_summary_for_report()
            .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "comparison snapshot source window summary no longer matches the current comparison snapshot source-window posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_comparison_snapshot_body_class_coverage_summary_matches_current_rendering(
    comparison_snapshot_body_class_coverage_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_snapshot_body_class_coverage_summary_text
        == validated_comparison_snapshot_body_class_coverage_summary_for_report()
            .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "comparison snapshot body-class coverage summary no longer matches the current comparison snapshot body-class coverage posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_target_threshold_summary_matches_current_rendering(
    packaged_artifact_target_threshold_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_target_threshold_summary_text
        == validated_packaged_artifact_target_threshold_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact target-threshold summary no longer matches the current packaged-artifact target-threshold posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_target_threshold_state_matches_current_rendering(
    packaged_artifact_target_threshold_state_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_target_threshold_state_summary_text
        == validated_packaged_artifact_target_threshold_state_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact target-threshold state summary no longer matches the current packaged-artifact target-threshold posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_source_fit_holdout_sync_summary_matches_current_rendering(
    packaged_artifact_source_fit_holdout_sync_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_source_fit_holdout_sync_summary_text
        == validated_packaged_artifact_source_fit_holdout_sync_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact source-fit and hold-out sync summary no longer matches the current packaged-artifact source-fit and hold-out sync posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_target_threshold_scope_envelopes_summary_matches_current_rendering(
    packaged_artifact_target_threshold_scope_envelopes_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_target_threshold_scope_envelopes_summary_text
        == validated_packaged_artifact_target_threshold_scope_envelopes_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact target-threshold scope envelopes summary no longer matches the current packaged-artifact target-threshold scope envelopes posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_profile_coverage_summary_matches_current_rendering(
    packaged_artifact_profile_coverage_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_profile_coverage_summary_text
        == packaged_artifact_profile_coverage_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact profile coverage summary no longer matches the current packaged-artifact profile coverage posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_output_support_summary_matches_current_rendering(
    packaged_artifact_output_support_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_output_support_summary_text
        == packaged_artifact_output_support_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact output support summary no longer matches the current packaged-artifact output-support posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_access_summary_matches_current_rendering(
    packaged_artifact_access_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_access_summary_text == packaged_artifact_access_summary_for_report() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact access summary no longer matches the current packaged-artifact access posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_speed_policy_summary_matches_current_rendering(
    packaged_artifact_speed_policy_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_speed_policy_summary_text
        == packaged_artifact_speed_policy_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact speed policy summary no longer matches the current packaged-artifact speed-policy posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_production_profile_summary_matches_current_rendering(
    packaged_artifact_production_profile_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_production_profile_summary_text
        == validated_packaged_artifact_production_profile_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact production-profile summary no longer matches the current packaged-artifact production-profile posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_lookup_epoch_policy_summary_matches_current_rendering(
    packaged_lookup_epoch_policy_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_lookup_epoch_policy_summary_text
        == packaged_lookup_epoch_policy_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact lookup-epoch policy summary no longer matches the current packaged-artifact lookup-epoch posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_generation_policy_summary_matches_current_rendering(
    packaged_artifact_generation_policy_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_generation_policy_summary_text
        == packaged_artifact_generation_policy_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact generation policy summary no longer matches the current packaged-artifact generation-policy posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_generation_residual_bodies_summary_matches_current_rendering(
    packaged_artifact_generation_residual_bodies_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_generation_residual_bodies_summary_text
        == validated_packaged_artifact_generation_residual_bodies_summary_for_report()
            .map_err(ReleaseBundleError::Verification)?
            .as_str()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact generation residual bodies summary no longer matches the current packaged-artifact residual-body posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_regeneration_summary_matches_current_rendering(
    packaged_artifact_regeneration_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_regeneration_summary_text
        == packaged_artifact_regeneration_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact regeneration summary no longer matches the current packaged-artifact regeneration posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_generation_manifest_summary_matches_current_rendering(
    packaged_artifact_generation_manifest_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_generation_manifest_summary_text
        == packaged_artifact_generation_manifest_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact generation manifest summary no longer matches the current packaged-artifact generation-manifest posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_generation_manifest_checksum_summary_matches_current_rendering(
    packaged_artifact_generation_manifest_checksum_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_generation_manifest_checksum_summary_text
        == packaged_artifact_generation_manifest_checksum_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact generation manifest checksum summary no longer matches the current packaged-artifact generation-manifest checksum posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_normalized_intermediate_summary_matches_current_rendering(
    packaged_artifact_normalized_intermediate_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_normalized_intermediate_summary_text
        == validated_packaged_artifact_normalized_intermediate_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact normalized intermediate summary no longer matches the current packaged-artifact normalized-intermediate posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_fit_sample_classes_summary_matches_current_rendering(
    packaged_artifact_fit_sample_classes_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_fit_sample_classes_summary_text
        == packaged_artifact_fit_sample_classes_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact fit sample classes summary no longer matches the current packaged-artifact fit sample classes posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_fit_threshold_violation_count_summary_matches_current_rendering(
    packaged_artifact_fit_threshold_violation_count_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_fit_threshold_violation_count_summary_text
        == packaged_artifact_fit_threshold_violation_count_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact fit threshold violation count summary no longer matches the current packaged-artifact fit threshold violation count posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_fit_threshold_violations_summary_matches_current_rendering(
    packaged_artifact_fit_threshold_violations_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_fit_threshold_violations_summary_text
        == packaged_artifact_fit_threshold_violation_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact fit threshold violations summary no longer matches the current packaged-artifact fit threshold violations posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_body_cadence_summary_matches_current_rendering(
    packaged_artifact_body_cadence_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_body_cadence_summary_text
        == validated_packaged_artifact_body_cadence_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact body cadence summary no longer matches the current packaged-artifact body cadence posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_packaged_artifact_body_class_span_cap_summary_matches_current_rendering(
    packaged_artifact_body_class_span_cap_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if packaged_artifact_body_class_span_cap_summary_text
        == validated_packaged_artifact_body_class_span_cap_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "packaged-artifact body-class span cap summary no longer matches the current packaged-artifact body-class span cap posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_benchmark_corpus_summary_matches_current_rendering(
    benchmark_corpus_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if benchmark_corpus_summary_text
        == validated_benchmark_corpus_summary_for_report()
            .map_err(ReleaseBundleError::Verification)?
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "benchmark corpus summary no longer matches the current benchmark-corpus posture"
                .to_string(),
        ))
    }
}

#[cfg(test)]
pub(crate) fn ensure_chart_benchmark_corpus_summary_matches_current_rendering(
    chart_benchmark_corpus_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if chart_benchmark_corpus_summary_text
        == validated_chart_benchmark_corpus_summary_for_report()
            .map_err(ReleaseBundleError::Verification)?
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "chart benchmark corpus summary no longer matches the current chart-benchmark corpus posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_interpolation_quality_request_corpus_summary_matches_current_rendering(
    interpolation_quality_request_corpus_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if interpolation_quality_request_corpus_summary_text
        == interpolation_quality_sample_request_corpus_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "interpolation-quality sample request corpus summary no longer matches the current interpolation-quality sample request corpus posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_validation_report_fit_envelope_matches_current_rendering(
    validation_report_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    let expected_line = format!(
        "  Packaged-artifact fit envelope: {}",
        packaged_artifact_fit_envelope_summary_for_report()
    );

    if validation_report_summary_text
        .lines()
        .any(|line| line == expected_line)
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "validation report summary no longer matches the current packaged-artifact fit envelope posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_validation_report_fit_margin_matches_current_rendering(
    validation_report_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    let expected_line = format!(
        "  Packaged-artifact fit margins: {}",
        report_summary_payload(
            packaged_artifact_fit_margin_summary_for_report(),
            "fit margins: ",
        )
    );

    if validation_report_summary_text
        .lines()
        .any(|line| line == expected_line)
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "validation report summary no longer matches the current packaged-artifact fit margins posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_validation_report_fit_outliers_matches_current_rendering(
    validation_report_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    let expected_line = format!(
        "  Packaged-artifact fit outliers: {}",
        packaged_artifact_fit_outlier_summary_for_report()
    );

    if validation_report_summary_text
        .lines()
        .any(|line| line == expected_line)
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "validation report summary no longer matches the current packaged-artifact fit outliers posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_validation_report_fit_sample_classes_matches_current_rendering(
    validation_report_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    let expected_line = format!(
        "  Packaged-artifact fit sample classes: {}",
        packaged_artifact_fit_sample_classes_summary_for_report()
    );

    if validation_report_summary_text
        .lines()
        .any(|line| line == expected_line)
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "validation report summary no longer matches the current packaged-artifact fit sample classes posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_validation_report_fit_threshold_violation_count_matches_current_rendering(
    validation_report_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    let expected_line = format!(
        "  Packaged-artifact fit threshold violation count: {}",
        report_summary_payload(
            packaged_artifact_fit_threshold_violation_count_for_report(),
            "fit threshold violations: ",
        )
    );

    if validation_report_summary_text
        .lines()
        .any(|line| line == expected_line)
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "validation report summary no longer matches the current packaged-artifact fit threshold violation count posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_validation_report_fit_threshold_violations_matches_current_rendering(
    validation_report_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    let expected_line = format!(
        "  Packaged-artifact fit threshold violations: {}",
        report_summary_payload(
            packaged_artifact_fit_threshold_violation_summary_for_report(),
            "fit threshold violations: ",
        )
    );

    if validation_report_summary_text
        .lines()
        .any(|line| line == expected_line)
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "validation report summary no longer matches the current packaged-artifact fit threshold violations posture"
                .to_string(),
        ))
    }
}

pub(crate) fn normalize_validation_report_summary_for_verification(text: &str) -> String {
    const UNSTABLE_PREFIXES: [&str; 8] = [
        "  ns/request (single):",
        "  ns/request (batch):",
        "  batch throughput:",
        "  ns/decode:",
        "  decodes per second:",
        "  decode elapsed:",
        "  ns/chart:",
        "  charts per second:",
    ];

    text.lines()
        .filter(|line| {
            !UNSTABLE_PREFIXES
                .iter()
                .any(|prefix| line.starts_with(prefix))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn ensure_validation_report_summary_matches_current_rendering(
    validation_report_summary_text: &str,
    rounds: usize,
) -> Result<(), ReleaseBundleError> {
    match render_validation_report_summary(rounds) {
        Ok(expected)
            if normalize_validation_report_summary_for_verification(
                validation_report_summary_text,
            ) == normalize_validation_report_summary_for_verification(&expected) =>
        {
            Ok(())
        }
        Ok(_) => Err(ReleaseBundleError::Verification(
            "validation report summary no longer matches the current validation report posture"
                .to_string(),
        )),
        Err(error) => Err(ReleaseBundleError::Verification(error.to_string())),
    }
}

pub(crate) fn normalize_validation_report_for_verification(text: &str) -> String {
    const UNSTABLE_PREFIXES: [&str; 8] = [
        "  ns/request (single):",
        "  ns/request (batch):",
        "  batch throughput:",
        "  ns/decode:",
        "  decodes per second:",
        "  decode elapsed:",
        "  ns/chart:",
        "  charts per second:",
    ];

    text.lines()
        .filter(|line| {
            !UNSTABLE_PREFIXES
                .iter()
                .any(|prefix| line.starts_with(prefix))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn ensure_validation_report_matches_current_rendering(
    validation_report_text: &str,
    rounds: usize,
) -> Result<(), ReleaseBundleError> {
    match render_validation_report(rounds) {
        Ok(expected)
            if normalize_validation_report_for_verification(validation_report_text)
                == normalize_validation_report_for_verification(&expected) =>
        {
            Ok(())
        }
        Ok(_) => Err(ReleaseBundleError::Verification(
            "validation report no longer matches the current validation report posture".to_string(),
        )),
        Err(error) => Err(ReleaseBundleError::Verification(error.to_string())),
    }
}

pub(crate) fn ensure_backend_matrix_selected_asteroid_source_lines_match_current_rendering(
    backend_matrix_text: &str,
) -> Result<(), ReleaseBundleError> {
    let selected_asteroid_source_evidence =
        validated_selected_asteroid_source_evidence_summary_for_report()
            .map_err(ReleaseBundleError::Verification)?;
    let selected_asteroid_source_window =
        validated_selected_asteroid_source_window_summary_for_report()
            .map_err(ReleaseBundleError::Verification)?;

    if backend_matrix_text.contains(&selected_asteroid_source_evidence)
        && backend_matrix_text.contains(&selected_asteroid_source_window)
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "backend matrix no longer matches the current selected-asteroid source evidence/window posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_selected_asteroid_source_request_corpus_summary_matches_current_rendering(
    selected_asteroid_source_request_corpus_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    let expected = validated_selected_asteroid_source_request_corpus_summary_for_report()
        .map_err(ReleaseBundleError::Verification)?;

    if selected_asteroid_source_request_corpus_summary_text == expected {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "selected asteroid source request corpus summary no longer matches the current selected-asteroid request corpus posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_request_policy_summary_matches_current_rendering(
    request_policy_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if request_policy_summary_text == render_request_policy_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "request policy summary no longer matches the current request-policy posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_time_scale_policy_summary_matches_current_rendering(
    time_scale_policy_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if time_scale_policy_summary_text == render_time_scale_policy_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "time-scale policy summary no longer matches the current time-scale posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_utc_convenience_policy_summary_matches_current_rendering(
    utc_convenience_policy_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if utc_convenience_policy_summary_text == render_utc_convenience_policy_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "UTC convenience policy summary no longer matches the current UTC-convenience posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_delta_t_policy_summary_matches_current_rendering(
    delta_t_policy_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if delta_t_policy_summary_text == render_delta_t_policy_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "delta-t policy summary no longer matches the current delta-t posture".to_string(),
        ))
    }
}

pub(crate) fn ensure_observer_policy_summary_matches_current_rendering(
    observer_policy_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if observer_policy_summary_text == render_observer_policy_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "observer policy summary no longer matches the current observer-policy posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_apparentness_policy_summary_matches_current_rendering(
    apparentness_policy_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if apparentness_policy_summary_text == render_apparentness_policy_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "apparentness policy summary no longer matches the current apparentness-policy posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_body_date_channel_claims_summary_matches_current_rendering(
    body_date_channel_claims_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if body_date_channel_claims_summary_text == render_body_date_channel_claims_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "body/date/channel claims summary no longer matches the current body/date/channel claims posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_release_body_claims_summary_matches_current_rendering(
    release_body_claims_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    let current_summary = validated_release_body_claims_summary_line_for_report()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    if release_body_claims_summary_text == current_summary {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "release body claims summary no longer matches the current release-grade body-claims posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_pluto_fallback_summary_matches_current_rendering(
    pluto_fallback_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    let current_summary = validated_pluto_fallback_summary_line_for_report()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    if pluto_fallback_summary_text == current_summary {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "Pluto fallback summary no longer matches the current Pluto fallback posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_request_semantics_summary_matches_current_rendering(
    request_semantics_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if request_semantics_summary_text == render_request_semantics_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "request-semantics summary no longer matches the current request-semantics posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_unsupported_modes_summary_matches_current_rendering(
    unsupported_modes_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if unsupported_modes_summary_text == render_unsupported_modes_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "unsupported-modes summary no longer matches the current unsupported-modes posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_zodiac_policy_summary_matches_current_rendering(
    zodiac_policy_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if zodiac_policy_summary_text == render_zodiac_policy_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "zodiac policy summary no longer matches the current zodiac posture".to_string(),
        ))
    }
}

pub(crate) fn ensure_request_surface_summary_matches_current_rendering(
    request_surface_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if request_surface_summary_text == render_request_surface_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "request surface summary no longer matches the current request-surface inventory"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_backend_matrix_report_matches_current_rendering(
    backend_matrix_text: &str,
) -> Result<(), ReleaseBundleError> {
    let expected = render_backend_matrix_report().map_err(|error| {
        ReleaseBundleError::Verification(format!("backend matrix unavailable ({error})"))
    })?;

    if backend_matrix_text == expected {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "backend matrix no longer matches the current backend-matrix posture".to_string(),
        ))
    }
}

pub(crate) fn ensure_backend_matrix_summary_matches_current_rendering(
    backend_matrix_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if backend_matrix_summary_text == render_backend_matrix_summary() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "backend matrix summary no longer matches the current backend-matrix posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_compatibility_caveats_summary_matches_current_rendering(
    compatibility_caveats_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if compatibility_caveats_summary_text == render_compatibility_caveats_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "compatibility caveats summary no longer matches the current compatibility-caveats posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_native_sidereal_policy_summary_matches_current_rendering(
    native_sidereal_policy_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if native_sidereal_policy_summary_text == render_native_sidereal_policy_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "native sidereal policy summary no longer matches the current native-sidereal posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_lunar_theory_limitations_summary_matches_current_rendering(
    lunar_theory_limitations_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if lunar_theory_limitations_summary_text == lunar_theory_limitations_summary_for_report() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "lunar theory limitations summary no longer matches the current lunar-theory limitations posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_lunar_theory_source_selection_summary_matches_current_rendering(
    lunar_theory_source_selection_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    let expected = pleiades_elp::validated_lunar_theory_source_selection_summary_for_report()
        .map_err(ReleaseBundleError::Verification)?;

    if lunar_theory_source_selection_summary_text == expected {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "lunar theory source selection summary no longer matches the current lunar source-selection posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_lunar_theory_source_family_summary_matches_current_rendering(
    lunar_theory_source_family_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if lunar_theory_source_family_summary_text
        == pleiades_elp::lunar_theory_source_family_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "lunar theory source family summary no longer matches the current lunar source-family posture"
                .to_string(),
        ))
    }
}

pub(crate) fn validated_lunar_theory_catalog_validation_summary_for_report() -> String {
    let summary = pleiades_elp::lunar_theory_catalog_validation_summary();
    match &summary.validation_result {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("lunar theory catalog validation: unavailable ({error})"),
    }
}

pub(crate) fn ensure_lunar_theory_catalog_validation_summary_matches_current_rendering(
    lunar_theory_catalog_validation_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if lunar_theory_catalog_validation_summary_text
        == validated_lunar_theory_catalog_validation_summary_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "lunar theory catalog validation summary no longer matches the current lunar theory catalog posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_release_house_validation_summary_matches_current_rendering(
    release_house_validation_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if release_house_validation_summary_text == release_house_validation_summary_for_report() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "release house validation summary no longer matches the current release-house-validation posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_target_house_scope_summary_matches_current_rendering(
    target_house_scope_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if target_house_scope_summary_text == render_target_house_scope_summary() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "target-house scope summary no longer matches the current compatibility-profile target-house-scope posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_target_ayanamsa_scope_summary_matches_current_rendering(
    target_ayanamsa_scope_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if target_ayanamsa_scope_summary_text == render_target_ayanamsa_scope_summary() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "target-ayanamsa scope summary no longer matches the current compatibility-profile target-ayanamsa-scope posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_house_formula_families_summary_matches_current_rendering(
    house_formula_families_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if house_formula_families_summary_text == format_house_formula_families_for_report() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "house formula families summary no longer matches the current house-formula-families posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_house_latitude_sensitive_summary_matches_current_rendering(
    house_latitude_sensitive_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if house_latitude_sensitive_summary_text == format_latitude_sensitive_house_systems_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "house latitude-sensitive summary no longer matches the current latitude-sensitive-house posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_house_latitude_sensitive_constraints_summary_matches_current_rendering(
    house_latitude_sensitive_constraints_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if house_latitude_sensitive_constraints_summary_text
        == format_latitude_sensitive_house_constraints_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "house latitude-sensitive constraints summary no longer matches the current latitude-sensitive-house constraints posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_house_latitude_sensitive_failure_modes_summary_matches_current_rendering(
    house_latitude_sensitive_failure_modes_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if house_latitude_sensitive_failure_modes_summary_text
        == format_latitude_sensitive_house_failure_modes_for_report()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "house latitude-sensitive failure-modes summary no longer matches the current latitude-sensitive-house failure-modes posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_workspace_provenance_summary_matches_current_rendering(
    workspace_provenance_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if workspace_provenance_summary_text == workspace_provenance_summary_for_report() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "workspace provenance summary no longer matches the current workspace provenance posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_native_dependency_audit_summary_matches_current_rendering(
    native_dependency_audit_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    let current_text = render_native_dependency_audit_summary()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    if native_dependency_audit_summary_text == current_text {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "native-dependency audit summary no longer matches the current workspace audit posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_api_stability_summary_matches_current_rendering(
    api_stability_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if api_stability_summary_text == render_api_stability_summary_text() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "API stability summary no longer matches the current API-stability posture".to_string(),
        ))
    }
}
pub(crate) fn verify_release_bundle(
    output_dir: impl AsRef<Path>,
) -> Result<ReleaseBundle, ReleaseBundleError> {
    verify_release_bundle_internal(output_dir, true)
}

pub(crate) fn verify_release_bundle_internal(
    output_dir: impl AsRef<Path>,
    validate_validation_report_summary: bool,
) -> Result<ReleaseBundle, ReleaseBundleError> {
    let output_dir = output_dir.as_ref();
    let profile_path = output_dir.join("compatibility-profile.txt");
    let profile_summary_path = output_dir.join("compatibility-profile-summary.txt");
    let release_notes_path = output_dir.join("release-notes.txt");
    let release_notes_summary_path = output_dir.join("release-notes-summary.txt");
    let release_summary_path = output_dir.join("release-summary.txt");
    let release_profile_identifiers_path = output_dir.join("release-profile-identifiers.txt");
    let release_profile_identifiers_summary_path =
        output_dir.join("release-profile-identifiers-summary.txt");
    let release_house_system_canonical_names_summary_path =
        output_dir.join("release-house-system-canonical-names-summary.txt");
    let release_ayanamsa_canonical_names_summary_path =
        output_dir.join("release-ayanamsa-canonical-names-summary.txt");
    let release_house_validation_summary_path =
        output_dir.join("release-house-validation-summary.txt");
    let target_house_scope_summary_path = output_dir.join("target-house-scope-summary.txt");
    let target_ayanamsa_scope_summary_path = output_dir.join("target-ayanamsa-scope-summary.txt");
    let house_code_aliases_summary_path = output_dir.join("house-code-aliases-summary.txt");
    let house_formula_families_summary_path = output_dir.join("house-formula-families-summary.txt");
    let house_latitude_sensitive_summary_path =
        output_dir.join("house-latitude-sensitive-summary.txt");
    let house_latitude_sensitive_constraints_summary_path =
        output_dir.join("house-latitude-sensitive-constraints-summary.txt");
    let house_latitude_sensitive_failure_modes_summary_path =
        output_dir.join("house-latitude-sensitive-failure-modes-summary.txt");
    let release_checklist_path = output_dir.join("release-checklist.txt");
    let release_checklist_summary_path = output_dir.join("release-checklist-summary.txt");
    let backend_matrix_path = output_dir.join("backend-matrix.txt");
    let backend_matrix_summary_path = output_dir.join("backend-matrix-summary.txt");
    let api_stability_path = output_dir.join("api-stability.txt");
    let api_stability_summary_path = output_dir.join("api-stability-summary.txt");
    let comparison_corpus_summary_path = output_dir.join("comparison-corpus-summary.txt");
    let source_corpus_summary_path = output_dir.join("source-corpus-summary.txt");
    let comparison_snapshot_summary_path = output_dir.join("comparison-snapshot-summary.txt");
    let comparison_snapshot_source_summary_path =
        output_dir.join("comparison-snapshot-source-summary.txt");
    let comparison_snapshot_source_window_summary_path =
        output_dir.join("comparison-snapshot-source-window-summary.txt");
    let comparison_snapshot_body_class_coverage_summary_path =
        output_dir.join("comparison-snapshot-body-class-coverage-summary.txt");
    let comparison_snapshot_manifest_summary_path =
        output_dir.join("comparison-snapshot-manifest-summary.txt");
    let comparison_envelope_summary_path = output_dir.join("comparison-envelope-summary.txt");
    let comparison_body_class_tolerance_summary_path =
        output_dir.join("comparison-body-class-tolerance-summary.txt");
    let comparison_body_class_error_envelope_summary_path =
        output_dir.join("comparison-body-class-error-envelope-summary.txt");
    let comparison_corpus_release_guard_summary_path =
        output_dir.join("comparison-corpus-release-guard-summary.txt");
    let comparison_corpus_guard_summary_path =
        output_dir.join("comparison-corpus-guard-summary.txt");
    let reference_holdout_overlap_summary_path =
        output_dir.join("reference-holdout-overlap-summary.txt");
    let reference_snapshot_bridge_day_summary_path =
        output_dir.join("reference-snapshot-bridge-day-summary.txt");
    let reference_snapshot_major_body_boundary_window_summary_path =
        output_dir.join("reference-snapshot-major-body-boundary-window-summary.txt");
    let reference_snapshot_boundary_epoch_coverage_summary_path =
        output_dir.join("reference-snapshot-boundary-epoch-coverage-summary.txt");
    let reference_snapshot_pre_bridge_boundary_summary_path =
        output_dir.join("reference-snapshot-pre-bridge-boundary-summary.txt");
    let reference_snapshot_2451917_major_body_boundary_summary_path =
        output_dir.join("reference-snapshot-2451917-major-body-boundary-summary.txt");
    let reference_snapshot_2451918_major_body_boundary_summary_path =
        output_dir.join("reference-snapshot-2451918-major-body-boundary-summary.txt");
    let reference_snapshot_2451919_major_body_boundary_summary_path =
        output_dir.join("reference-snapshot-2451919-major-body-boundary-summary.txt");
    let reference_snapshot_2451916_major_body_dense_boundary_summary_path =
        output_dir.join("reference-snapshot-2451916-major-body-dense-boundary-summary.txt");
    let reference_snapshot_sparse_boundary_summary_path =
        output_dir.join("reference-snapshot-sparse-boundary-summary.txt");
    let reference_snapshot_exact_j2000_evidence_summary_path =
        output_dir.join("reference-snapshot-exact-j2000-evidence-summary.txt");
    let reference_snapshot_source_summary_path =
        output_dir.join("reference-snapshot-source-summary.txt");
    let reference_snapshot_source_window_summary_path =
        output_dir.join("reference-snapshot-source-window-summary.txt");
    let reference_snapshot_manifest_summary_path =
        output_dir.join("reference-snapshot-manifest-summary.txt");
    let reference_snapshot_body_class_coverage_summary_path =
        output_dir.join("reference-snapshot-body-class-coverage-summary.txt");
    let reference_snapshot_equatorial_parity_summary_path =
        output_dir.join("reference-snapshot-equatorial-parity-summary.txt");
    let reference_asteroid_source_window_summary_path =
        output_dir.join("reference-asteroid-source-window-summary.txt");
    let reference_asteroid_equatorial_evidence_summary_path =
        output_dir.join("reference-asteroid-equatorial-evidence-summary.txt");
    let independent_holdout_source_window_summary_path =
        output_dir.join("independent-holdout-source-window-summary.txt");
    let independent_holdout_equatorial_parity_summary_path =
        output_dir.join("independent-holdout-equatorial-parity-summary.txt");
    let independent_holdout_body_class_coverage_summary_path =
        output_dir.join("independent-holdout-body-class-coverage-summary.txt");
    let independent_holdout_quarter_day_boundary_summary_path =
        output_dir.join("independent-holdout-quarter-day-boundary-summary.txt");
    let production_generation_boundary_source_summary_path =
        output_dir.join("production-generation-boundary-source-summary.txt");
    let production_generation_boundary_window_summary_path =
        output_dir.join("production-generation-boundary-window-summary.txt");
    let production_generation_boundary_request_corpus_summary_path =
        output_dir.join("production-generation-boundary-request-corpus-summary.txt");
    let production_generation_boundary_request_corpus_equatorial_summary_path =
        output_dir.join("production-generation-boundary-request-corpus-equatorial-summary.txt");
    let production_generation_source_revision_summary_path =
        output_dir.join("production-generation-source-revision-summary.txt");
    let production_generation_source_window_summary_path =
        output_dir.join("production-generation-source-window-summary.txt");
    let production_generation_quarter_day_boundary_summary_path =
        output_dir.join("production-generation-quarter-day-boundary-summary.txt");
    let production_generation_corpus_shape_summary_path =
        output_dir.join("production-generation-corpus-shape-summary.txt");
    let catalog_posture_summary_path = output_dir.join("catalog-posture-summary.txt");
    let production_generation_manifest_summary_path =
        output_dir.join("production-generation-manifest-summary.txt");
    let production_generation_manifest_checksum_path =
        output_dir.join("production-generation-manifest-checksum-summary.txt");
    let reference_snapshot_summary_path = output_dir.join("reference-snapshot-summary.txt");
    let catalog_inventory_summary_path = output_dir.join("catalog-inventory-summary.txt");
    let custom_definition_ayanamsa_labels_summary_path =
        output_dir.join("custom-definition-ayanamsa-labels-summary.txt");
    let ayanamsa_provenance_summary_path = output_dir.join("ayanamsa-provenance-summary.txt");
    let validation_report_summary_path = output_dir.join("validation-report-summary.txt");
    let workspace_provenance_summary_path = output_dir.join("workspace-provenance-summary.txt");
    let release_body_claims_summary_path = output_dir.join("release-body-claims-summary.txt");
    let pluto_fallback_summary_path = output_dir.join("pluto-fallback-summary.txt");
    let request_policy_summary_path = output_dir.join("request-policy-summary.txt");
    let observer_policy_summary_path = output_dir.join("observer-policy-summary.txt");
    let apparentness_policy_summary_path = output_dir.join("apparentness-policy-summary.txt");
    let request_semantics_summary_path = output_dir.join("request-semantics-summary.txt");
    let unsupported_modes_summary_path = output_dir.join("unsupported-modes-summary.txt");
    let time_scale_policy_summary_path = output_dir.join("time-scale-policy-summary.txt");
    let utc_convenience_policy_summary_path = output_dir.join("utc-convenience-policy-summary.txt");
    let delta_t_policy_summary_path = output_dir.join("delta-t-policy-summary.txt");
    let native_sidereal_policy_summary_path = output_dir.join("native-sidereal-policy-summary.txt");
    let zodiac_policy_summary_path = output_dir.join("zodiac-policy-summary.txt");
    let lunar_theory_limitations_summary_path =
        output_dir.join("lunar-theory-limitations-summary.txt");
    let lunar_theory_source_selection_summary_path =
        output_dir.join("lunar-theory-source-selection-summary.txt");
    let lunar_theory_source_family_summary_path =
        output_dir.join("lunar-theory-source-family-summary.txt");
    let lunar_source_window_summary_path = output_dir.join("lunar-source-window-summary.txt");
    let _lunar_reference_error_envelope_summary_path =
        output_dir.join("lunar-reference-error-envelope-summary.txt");
    let _lunar_equatorial_reference_error_envelope_summary_path =
        output_dir.join("lunar-equatorial-reference-error-envelope-summary.txt");
    let _lunar_apparent_comparison_summary_path =
        output_dir.join("lunar-apparent-comparison-summary.txt");
    let lunar_theory_catalog_validation_summary_path =
        output_dir.join("lunar-theory-catalog-validation-summary.txt");
    let request_surface_summary_path = output_dir.join("request-surface-summary.txt");
    let compatibility_caveats_summary_path = output_dir.join("compatibility-caveats-summary.txt");
    let workspace_audit_summary_path = output_dir.join("workspace-audit-summary.txt");
    let native_dependency_audit_summary_path =
        output_dir.join("native-dependency-audit-summary.txt");
    let artifact_summary_path = output_dir.join("artifact-summary.txt");
    let packaged_artifact_path = output_dir.join("packaged-artifact.bin");
    let packaged_artifact_checksum_path = output_dir.join("packaged-artifact.checksum.txt");
    let packaged_artifact_profile_coverage_summary_path =
        output_dir.join("packaged-artifact-profile-coverage-summary.txt");
    let packaged_artifact_access_summary_path =
        output_dir.join("packaged-artifact-access-summary.txt");
    let packaged_artifact_output_support_summary_path =
        output_dir.join("packaged-artifact-output-support-summary.txt");
    let packaged_artifact_fit_sample_classes_summary_path =
        output_dir.join("packaged-artifact-fit-sample-classes-summary.txt");
    let packaged_artifact_fit_threshold_violation_count_summary_path =
        output_dir.join("packaged-artifact-fit-threshold-violation-count-summary.txt");
    let packaged_artifact_fit_threshold_violations_summary_path =
        output_dir.join("packaged-artifact-fit-threshold-violations-summary.txt");
    let packaged_artifact_body_cadence_summary_path =
        output_dir.join("packaged-artifact-body-cadence-summary.txt");
    let packaged_artifact_body_class_span_cap_summary_path =
        output_dir.join("packaged-artifact-body-class-span-cap-summary.txt");
    let packaged_artifact_normalized_intermediate_summary_path =
        output_dir.join("packaged-artifact-normalized-intermediate-summary.txt");
    let packaged_artifact_speed_policy_summary_path =
        output_dir.join("packaged-artifact-speed-policy-summary.txt");
    let packaged_artifact_storage_summary_path =
        output_dir.join("packaged-artifact-storage-summary.txt");
    let packaged_artifact_production_profile_summary_path =
        output_dir.join("packaged-artifact-production-profile-summary.txt");
    let packaged_frame_treatment_summary_path =
        output_dir.join("packaged-frame-treatment-summary.txt");
    let packaged_artifact_target_threshold_summary_path =
        output_dir.join("packaged-artifact-target-threshold-summary.txt");
    let packaged_artifact_target_threshold_state_summary_path =
        output_dir.join("packaged-artifact-target-threshold-state-summary.txt");
    let packaged_artifact_source_fit_holdout_sync_summary_path =
        output_dir.join("packaged-artifact-source-fit-holdout-sync-summary.txt");
    let packaged_artifact_target_threshold_scope_envelopes_summary_path =
        output_dir.join("packaged-artifact-target-threshold-scope-envelopes-summary.txt");
    let packaged_artifact_phase2_corpus_alignment_summary_path =
        output_dir.join("packaged-artifact-phase2-corpus-alignment-summary.txt");
    let packaged_lookup_epoch_policy_summary_path =
        output_dir.join("packaged-lookup-epoch-policy-summary.txt");
    let packaged_artifact_generation_policy_summary_path =
        output_dir.join("packaged-artifact-generation-policy-summary.txt");
    let packaged_artifact_generation_residual_bodies_summary_path =
        output_dir.join("packaged-artifact-generation-residual-bodies-summary.txt");
    let packaged_artifact_regeneration_summary_path =
        output_dir.join("packaged-artifact-regeneration-summary.txt");
    let packaged_artifact_generation_manifest_path =
        output_dir.join("packaged-artifact-generation-manifest.txt");
    let packaged_artifact_generation_manifest_summary_path =
        output_dir.join("packaged-artifact-generation-manifest-summary.txt");
    let packaged_artifact_generation_manifest_checksum_summary_path =
        output_dir.join("packaged-artifact-generation-manifest-checksum-summary.txt");
    let packaged_artifact_generation_manifest_checksum_path =
        output_dir.join("packaged-artifact-generation-manifest.checksum.txt");
    let benchmark_corpus_summary_path = output_dir.join("benchmark-corpus-summary.txt");
    let chart_benchmark_corpus_summary_path = output_dir.join("chart-benchmark-corpus-summary.txt");
    let selected_asteroid_source_request_corpus_summary_path =
        output_dir.join("selected-asteroid-source-request-corpus-summary.txt");
    let selected_asteroid_source_request_corpus_equatorial_summary_path =
        output_dir.join("selected-asteroid-source-request-corpus-equatorial-summary.txt");
    let selected_asteroid_source_window_summary_path =
        output_dir.join("selected-asteroid-source-window-summary.txt");
    let interpolation_quality_request_corpus_summary_path =
        output_dir.join("interpolation-quality-request-corpus-summary.txt");
    let benchmark_report_path = output_dir.join("benchmark-report.txt");
    let validation_report_path = output_dir.join("validation-report.txt");
    let manifest_path = output_dir.join("bundle-manifest.txt");
    let manifest_checksum_path = output_dir.join("bundle-manifest.checksum.txt");

    for (path, label) in [
        (&profile_path, "compatibility profile"),
        (&profile_summary_path, "compatibility profile summary"),
        (&release_notes_path, "release notes"),
        (&release_notes_summary_path, "release notes summary"),
        (&release_summary_path, "release summary"),
        (
            &release_profile_identifiers_path,
            "release-profile identifiers",
        ),
        (
            &release_house_validation_summary_path,
            "release house validation summary",
        ),
        (
            &house_formula_families_summary_path,
            "house formula families summary",
        ),
        (
            &house_latitude_sensitive_summary_path,
            "house latitude-sensitive summary",
        ),
        (
            &house_latitude_sensitive_constraints_summary_path,
            "house latitude-sensitive constraints summary",
        ),
        (
            &house_latitude_sensitive_failure_modes_summary_path,
            "house latitude-sensitive failure-modes summary",
        ),
        (&release_checklist_path, "release checklist"),
        (&release_checklist_summary_path, "release checklist summary"),
        (&backend_matrix_path, "backend matrix"),
        (&backend_matrix_summary_path, "backend matrix summary"),
        (&api_stability_path, "API stability"),
        (&api_stability_summary_path, "API stability summary"),
        (
            &comparison_envelope_summary_path,
            "comparison envelope summary",
        ),
        (
            &comparison_corpus_release_guard_summary_path,
            "comparison-corpus release-guard summary",
        ),
        (&source_corpus_summary_path, "source corpus summary"),
        (
            &comparison_corpus_guard_summary_path,
            "comparison-corpus guard summary alias",
        ),
        (
            &reference_holdout_overlap_summary_path,
            "reference-holdout overlap summary",
        ),
        (
            &reference_snapshot_bridge_day_summary_path,
            "reference snapshot bridge day summary",
        ),
        (
            &reference_snapshot_major_body_boundary_window_summary_path,
            "reference snapshot major-body boundary window summary",
        ),
        (
            &reference_snapshot_boundary_epoch_coverage_summary_path,
            "reference snapshot boundary epoch coverage summary",
        ),
        (
            &reference_snapshot_pre_bridge_boundary_summary_path,
            "reference snapshot pre-bridge boundary summary",
        ),
        (
            &reference_snapshot_2451918_major_body_boundary_summary_path,
            "reference snapshot 2451918 major-body boundary summary",
        ),
        (
            &reference_snapshot_2451919_major_body_boundary_summary_path,
            "reference snapshot 2451919 major-body boundary summary",
        ),
        (
            &reference_snapshot_source_summary_path,
            "reference snapshot source summary",
        ),
        (
            &reference_asteroid_source_window_summary_path,
            "reference asteroid source window summary",
        ),
        (
            &independent_holdout_source_window_summary_path,
            "independent-holdout source window summary",
        ),
        (
            &independent_holdout_quarter_day_boundary_summary_path,
            "independent-holdout quarter-day boundary summary",
        ),
        (
            &production_generation_boundary_source_summary_path,
            "production generation boundary source summary",
        ),
        (
            &production_generation_boundary_request_corpus_summary_path,
            "production generation boundary request corpus summary",
        ),
        (&catalog_inventory_summary_path, "catalog inventory summary"),
        (&validation_report_summary_path, "validation report summary"),
        (&request_policy_summary_path, "request policy summary"),
        (
            &lunar_theory_limitations_summary_path,
            "lunar-theory limitations summary",
        ),
        (
            &lunar_theory_catalog_validation_summary_path,
            "lunar-theory catalog validation summary",
        ),
        (
            &compatibility_caveats_summary_path,
            "compatibility caveats summary",
        ),
        (&workspace_audit_summary_path, "workspace audit summary"),
        (
            &native_dependency_audit_summary_path,
            "native-dependency audit summary",
        ),
        (&artifact_summary_path, "artifact summary"),
        (
            &packaged_artifact_production_profile_summary_path,
            "packaged-artifact production-profile summary",
        ),
        (
            &packaged_frame_treatment_summary_path,
            "packaged frame treatment summary",
        ),
        (
            &packaged_artifact_target_threshold_summary_path,
            "packaged-artifact target-threshold summary",
        ),
        (
            &packaged_artifact_source_fit_holdout_sync_summary_path,
            "packaged-artifact source-fit and hold-out sync summary",
        ),
        (
            &packaged_artifact_target_threshold_scope_envelopes_summary_path,
            "packaged-artifact target-threshold scope envelopes summary",
        ),
        (
            &packaged_artifact_phase2_corpus_alignment_summary_path,
            "packaged-artifact phase-2 corpus alignment summary",
        ),
        (
            &packaged_artifact_regeneration_summary_path,
            "packaged-artifact regeneration summary",
        ),
        (
            &packaged_artifact_generation_manifest_path,
            "packaged-artifact generation manifest",
        ),
        (
            &packaged_artifact_generation_manifest_summary_path,
            "packaged-artifact generation manifest summary",
        ),
        (
            &packaged_artifact_generation_manifest_checksum_path,
            "packaged-artifact generation manifest checksum sidecar",
        ),
        (&benchmark_report_path, "benchmark report"),
        (&validation_report_path, "validation report"),
        (&manifest_path, "bundle manifest"),
        (&manifest_checksum_path, "bundle manifest checksum sidecar"),
    ] {
        ensure_release_bundle_regular_file(path, label)?;
    }

    let profile_text = read_required_bundle_text(&profile_path, "compatibility profile")?;
    let profile_summary_text =
        read_required_bundle_text(&profile_summary_path, "compatibility profile summary")?;
    let release_notes_text = read_required_bundle_text(&release_notes_path, "release notes")?;
    let release_notes_summary_text =
        read_required_bundle_text(&release_notes_summary_path, "release notes summary")?;
    let release_summary_text = read_required_bundle_text(&release_summary_path, "release summary")?;
    let release_profile_identifiers_text = read_required_bundle_text(
        &release_profile_identifiers_path,
        "release-profile identifiers",
    )?;
    let release_profile_identifiers_summary_text = read_required_bundle_text(
        &release_profile_identifiers_summary_path,
        "release-profile identifiers summary",
    )?;
    let release_house_validation_summary_text = read_required_bundle_text(
        &release_house_validation_summary_path,
        "release house validation summary",
    )?;
    ensure_release_house_validation_summary_matches_current_rendering(
        &release_house_validation_summary_text,
    )?;
    let target_house_scope_summary_text = read_required_bundle_text(
        &target_house_scope_summary_path,
        "target house scope summary",
    )?;
    ensure_target_house_scope_summary_matches_current_rendering(&target_house_scope_summary_text)?;
    let target_ayanamsa_scope_summary_text = read_required_bundle_text(
        &target_ayanamsa_scope_summary_path,
        "target ayanamsa scope summary",
    )?;
    ensure_target_ayanamsa_scope_summary_matches_current_rendering(
        &target_ayanamsa_scope_summary_text,
    )?;
    let house_code_aliases_summary_text = read_required_bundle_text(
        &house_code_aliases_summary_path,
        "house code aliases summary",
    )?;
    if house_code_aliases_summary_text
        != validated_house_code_aliases_summary_for_report()
            .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?
    {
        return Err(ReleaseBundleError::Verification(
            "house code aliases summary no longer matches the current house-code-aliases posture"
                .to_string(),
        ));
    }
    let house_formula_families_summary_text = read_required_bundle_text(
        &house_formula_families_summary_path,
        "house formula families summary",
    )?;
    ensure_house_formula_families_summary_matches_current_rendering(
        &house_formula_families_summary_text,
    )?;
    let house_latitude_sensitive_summary_text = read_required_bundle_text(
        &house_latitude_sensitive_summary_path,
        "house latitude-sensitive summary",
    )?;
    ensure_house_latitude_sensitive_summary_matches_current_rendering(
        &house_latitude_sensitive_summary_text,
    )?;
    let house_latitude_sensitive_constraints_summary_text = read_required_bundle_text(
        &house_latitude_sensitive_constraints_summary_path,
        "house latitude-sensitive constraints summary",
    )?;
    ensure_house_latitude_sensitive_constraints_summary_matches_current_rendering(
        &house_latitude_sensitive_constraints_summary_text,
    )?;
    let house_latitude_sensitive_failure_modes_summary_text = read_required_bundle_text(
        &house_latitude_sensitive_failure_modes_summary_path,
        "house latitude-sensitive failure-modes summary",
    )?;
    ensure_house_latitude_sensitive_failure_modes_summary_matches_current_rendering(
        &house_latitude_sensitive_failure_modes_summary_text,
    )?;
    let target_house_scope_summary_checksum = checksum64(&target_house_scope_summary_text);
    let target_ayanamsa_scope_summary_checksum = checksum64(&target_ayanamsa_scope_summary_text);
    let house_code_aliases_summary_checksum = checksum64(&house_code_aliases_summary_text);
    let house_formula_families_summary_checksum = checksum64(&house_formula_families_summary_text);
    let house_latitude_sensitive_summary_checksum =
        checksum64(&house_latitude_sensitive_summary_text);
    let house_latitude_sensitive_constraints_summary_checksum =
        checksum64(&house_latitude_sensitive_constraints_summary_text);
    let house_latitude_sensitive_failure_modes_summary_checksum =
        checksum64(&house_latitude_sensitive_failure_modes_summary_text);
    let release_checklist_text =
        read_required_bundle_text(&release_checklist_path, "release checklist")?;
    let release_checklist_summary_text =
        read_required_bundle_text(&release_checklist_summary_path, "release checklist summary")?;
    let release_house_validation_summary_checksum =
        checksum64(&release_house_validation_summary_text);
    let backend_matrix_text = read_required_bundle_text(&backend_matrix_path, "backend matrix")?;
    let backend_matrix_summary_text =
        read_required_bundle_text(&backend_matrix_summary_path, "backend matrix summary")?;
    ensure_backend_matrix_report_matches_current_rendering(&backend_matrix_text)?;
    ensure_backend_matrix_summary_matches_current_rendering(&backend_matrix_summary_text)?;
    let api_stability_text = read_required_bundle_text(&api_stability_path, "API stability")?;
    let api_stability_summary_text =
        read_required_bundle_text(&api_stability_summary_path, "API stability summary")?;
    ensure_api_stability_summary_matches_current_rendering(&api_stability_summary_text)?;
    let comparison_corpus_summary_text =
        read_required_bundle_text(&comparison_corpus_summary_path, "comparison corpus summary")?;
    ensure_comparison_corpus_summary_matches_current_rendering(&comparison_corpus_summary_text)?;
    let source_corpus_summary_text =
        read_required_bundle_text(&source_corpus_summary_path, "source corpus summary")?;
    let jpl_source_posture_summary_path = output_dir.join("jpl-source-posture-summary.txt");
    let jpl_source_posture_summary_text = read_required_bundle_text(
        &jpl_source_posture_summary_path,
        "JPL source posture summary",
    )?;
    if jpl_source_posture_summary_text != jpl_source_posture_summary_for_report() {
        return Err(ReleaseBundleError::Verification(
            "JPL source posture summary no longer matches the current JPL source posture"
                .to_string(),
        ));
    }
    let jpl_provenance_only_summary_path = output_dir.join("jpl-provenance-only-summary.txt");
    let jpl_provenance_only_summary_text = read_required_bundle_text(
        &jpl_provenance_only_summary_path,
        "JPL provenance-only evidence summary",
    )?;
    if jpl_provenance_only_summary_text != jpl_provenance_only_summary_for_report() {
        return Err(ReleaseBundleError::Verification(
            "JPL provenance-only evidence summary no longer matches the current JPL provenance-only posture"
                .to_string(),
        ));
    }
    let jpl_provenance_only_summary_checksum = checksum64(&jpl_provenance_only_summary_text);
    let comparison_snapshot_summary_text = read_required_bundle_text(
        &comparison_snapshot_summary_path,
        "comparison snapshot summary",
    )?;
    let comparison_snapshot_source_summary_text = read_required_bundle_text(
        &comparison_snapshot_source_summary_path,
        "comparison snapshot source summary",
    )?;
    ensure_comparison_snapshot_source_summary_matches_current_rendering(
        &comparison_snapshot_source_summary_text,
    )?;
    let comparison_snapshot_source_window_summary_text = read_required_bundle_text(
        &comparison_snapshot_source_window_summary_path,
        "comparison snapshot source window summary",
    )?;
    ensure_comparison_snapshot_source_window_summary_matches_current_rendering(
        &comparison_snapshot_source_window_summary_text,
    )?;
    let comparison_snapshot_body_class_coverage_summary_text = read_required_bundle_text(
        &comparison_snapshot_body_class_coverage_summary_path,
        "comparison snapshot body-class coverage summary",
    )?;
    ensure_comparison_snapshot_body_class_coverage_summary_matches_current_rendering(
        &comparison_snapshot_body_class_coverage_summary_text,
    )?;
    let comparison_snapshot_body_class_coverage_summary_checksum =
        checksum64(&comparison_snapshot_body_class_coverage_summary_text);
    let comparison_snapshot_manifest_summary_text = read_required_bundle_text(
        &comparison_snapshot_manifest_summary_path,
        "comparison snapshot manifest summary",
    )?;
    ensure_comparison_snapshot_manifest_summary_matches_current_rendering(
        &comparison_snapshot_manifest_summary_text,
    )?;
    let comparison_snapshot_manifest_summary_checksum =
        checksum64(&comparison_snapshot_manifest_summary_text);
    let comparison_envelope_summary_text = read_required_bundle_text(
        &comparison_envelope_summary_path,
        "comparison envelope summary",
    )?;
    let comparison_body_class_tolerance_summary_text = read_required_bundle_text(
        &comparison_body_class_tolerance_summary_path,
        "comparison body-class tolerance summary",
    )?;
    let comparison_body_class_tolerance_summary_checksum =
        checksum64(&comparison_body_class_tolerance_summary_text);
    let comparison_body_class_error_envelope_summary_text = read_required_bundle_text(
        &comparison_body_class_error_envelope_summary_path,
        "comparison body-class error-envelope summary",
    )?;
    let comparison_body_class_error_envelope_summary_checksum =
        checksum64(&comparison_body_class_error_envelope_summary_text);
    let comparison_corpus_release_guard_summary_text = read_required_bundle_text(
        &comparison_corpus_release_guard_summary_path,
        "comparison-corpus release-guard summary",
    )?;
    let reference_holdout_overlap_summary_text = read_required_bundle_text(
        &reference_holdout_overlap_summary_path,
        "reference-holdout overlap summary",
    )?;
    ensure_reference_holdout_overlap_summary_matches_current_rendering(
        &reference_holdout_overlap_summary_text,
    )?;
    let reference_snapshot_bridge_day_summary_text = read_required_bundle_text(
        &reference_snapshot_bridge_day_summary_path,
        "reference snapshot bridge day summary",
    )?;
    ensure_reference_snapshot_bridge_day_summary_matches_current_rendering(
        &reference_snapshot_bridge_day_summary_text,
    )?;
    let reference_snapshot_major_body_boundary_window_summary_text = read_required_bundle_text(
        &reference_snapshot_major_body_boundary_window_summary_path,
        "reference snapshot major-body boundary window summary",
    )?;
    ensure_reference_snapshot_major_body_boundary_window_summary_matches_current_rendering(
        &reference_snapshot_major_body_boundary_window_summary_text,
    )?;
    let reference_snapshot_boundary_epoch_coverage_summary_text = read_required_bundle_text(
        &reference_snapshot_boundary_epoch_coverage_summary_path,
        "reference snapshot boundary epoch coverage summary",
    )?;
    ensure_reference_snapshot_boundary_epoch_coverage_summary_matches_current_rendering(
        &reference_snapshot_boundary_epoch_coverage_summary_text,
    )?;
    let reference_snapshot_pre_bridge_boundary_summary_text = read_required_bundle_text(
        &reference_snapshot_pre_bridge_boundary_summary_path,
        "reference snapshot pre-bridge boundary summary",
    )?;
    ensure_reference_snapshot_pre_bridge_boundary_summary_matches_current_rendering(
        &reference_snapshot_pre_bridge_boundary_summary_text,
    )?;
    let reference_snapshot_2451917_major_body_boundary_summary_text = read_required_bundle_text(
        &reference_snapshot_2451917_major_body_boundary_summary_path,
        "reference snapshot 2451917 major-body boundary summary",
    )?;
    ensure_reference_snapshot_2451917_major_body_boundary_summary_matches_current_rendering(
        &reference_snapshot_2451917_major_body_boundary_summary_text,
    )?;
    let reference_snapshot_2451918_major_body_boundary_summary_text = read_required_bundle_text(
        &reference_snapshot_2451918_major_body_boundary_summary_path,
        "reference snapshot 2451918 major-body boundary summary",
    )?;
    ensure_reference_snapshot_2451918_major_body_boundary_summary_matches_current_rendering(
        &reference_snapshot_2451918_major_body_boundary_summary_text,
    )?;
    let reference_snapshot_2451919_major_body_boundary_summary_text = read_required_bundle_text(
        &reference_snapshot_2451919_major_body_boundary_summary_path,
        "reference snapshot 2451919 major-body boundary summary",
    )?;
    ensure_reference_snapshot_2451919_major_body_boundary_summary_matches_current_rendering(
        &reference_snapshot_2451919_major_body_boundary_summary_text,
    )?;
    let reference_snapshot_2451916_major_body_dense_boundary_summary_text =
        read_required_bundle_text(
            &reference_snapshot_2451916_major_body_dense_boundary_summary_path,
            "reference snapshot 2451916 major-body dense boundary summary",
        )?;
    let reference_snapshot_2451916_major_body_dense_boundary_summary_checksum =
        checksum64(&reference_snapshot_2451916_major_body_dense_boundary_summary_text);
    if reference_snapshot_2451916_major_body_dense_boundary_summary_text
        != reference_snapshot_2451916_major_body_dense_boundary_summary_for_report()
    {
        return Err(ReleaseBundleError::Verification(
            "reference snapshot 2451916 major-body dense boundary summary no longer matches the current reference snapshot 2451916 major-body dense boundary posture".to_string(),
        ));
    }
    let reference_snapshot_sparse_boundary_summary_text = read_required_bundle_text(
        &reference_snapshot_sparse_boundary_summary_path,
        "reference snapshot sparse boundary summary",
    )?;
    let reference_snapshot_exact_j2000_evidence_summary_text = read_required_bundle_text(
        &reference_snapshot_exact_j2000_evidence_summary_path,
        "reference snapshot exact J2000 evidence summary",
    )?;
    ensure_reference_snapshot_exact_j2000_evidence_summary_matches_current_rendering(
        &reference_snapshot_exact_j2000_evidence_summary_text,
    )?;
    ensure_reference_snapshot_sparse_boundary_summary_matches_current_rendering(
        &reference_snapshot_sparse_boundary_summary_text,
    )?;
    let reference_snapshot_source_summary_text = read_required_bundle_text(
        &reference_snapshot_source_summary_path,
        "reference snapshot source summary",
    )?;
    ensure_reference_snapshot_source_summary_matches_current_rendering(
        &reference_snapshot_source_summary_text,
    )?;
    let reference_snapshot_source_window_summary_text = read_required_bundle_text(
        &reference_snapshot_source_window_summary_path,
        "reference snapshot source window summary",
    )?;
    ensure_reference_snapshot_source_window_summary_matches_current_rendering(
        &reference_snapshot_source_window_summary_text,
    )?;
    let reference_snapshot_manifest_summary_text = read_required_bundle_text(
        &reference_snapshot_manifest_summary_path,
        "reference snapshot manifest summary",
    )?;
    ensure_reference_snapshot_manifest_summary_matches_current_rendering(
        &reference_snapshot_manifest_summary_text,
    )?;
    let reference_snapshot_manifest_summary_checksum =
        checksum64(&reference_snapshot_manifest_summary_text);
    let reference_snapshot_body_class_coverage_summary_text = read_required_bundle_text(
        &reference_snapshot_body_class_coverage_summary_path,
        "reference snapshot body-class coverage summary",
    )?;
    ensure_reference_snapshot_body_class_coverage_summary_matches_current_rendering(
        &reference_snapshot_body_class_coverage_summary_text,
    )?;
    let reference_snapshot_body_class_coverage_summary_checksum =
        checksum64(&reference_snapshot_body_class_coverage_summary_text);
    let reference_snapshot_equatorial_parity_summary_text = read_required_bundle_text(
        &reference_snapshot_equatorial_parity_summary_path,
        "reference snapshot equatorial parity summary",
    )?;
    ensure_reference_snapshot_equatorial_parity_summary_matches_current_rendering(
        &reference_snapshot_equatorial_parity_summary_text,
    )?;
    let reference_snapshot_equatorial_parity_summary_checksum =
        checksum64(&reference_snapshot_equatorial_parity_summary_text);
    let reference_asteroid_source_window_summary_text = read_required_bundle_text(
        &reference_asteroid_source_window_summary_path,
        "reference asteroid source window summary",
    )?;
    ensure_reference_asteroid_source_window_summary_matches_current_rendering(
        &reference_asteroid_source_window_summary_text,
    )?;
    let reference_asteroid_equatorial_evidence_summary_text = read_required_bundle_text(
        &reference_asteroid_equatorial_evidence_summary_path,
        "reference asteroid equatorial evidence summary",
    )?;
    ensure_reference_asteroid_equatorial_evidence_summary_matches_current_rendering(
        &reference_asteroid_equatorial_evidence_summary_text,
    )?;
    let reference_asteroid_equatorial_evidence_summary_checksum =
        checksum64(&reference_asteroid_equatorial_evidence_summary_text);
    let independent_holdout_source_window_summary_text = read_required_bundle_text(
        &independent_holdout_source_window_summary_path,
        "independent-holdout source window summary",
    )?;
    let independent_holdout_equatorial_parity_summary_text = read_required_bundle_text(
        &independent_holdout_equatorial_parity_summary_path,
        "independent-holdout equatorial parity summary",
    )?;
    ensure_independent_holdout_equatorial_parity_summary_matches_current_rendering(
        &independent_holdout_equatorial_parity_summary_text,
    )?;
    let independent_holdout_equatorial_parity_summary_checksum =
        checksum64(&independent_holdout_equatorial_parity_summary_text);
    let independent_holdout_body_class_coverage_summary_text = read_required_bundle_text(
        &independent_holdout_body_class_coverage_summary_path,
        "independent-holdout body-class coverage summary",
    )?;
    let independent_holdout_body_class_coverage_summary_checksum =
        checksum64(&independent_holdout_body_class_coverage_summary_text);
    let independent_holdout_quarter_day_boundary_summary_text = read_required_bundle_text(
        &independent_holdout_quarter_day_boundary_summary_path,
        "independent-holdout quarter-day boundary summary",
    )?;
    ensure_independent_holdout_quarter_day_boundary_summary_matches_current_rendering(
        &independent_holdout_quarter_day_boundary_summary_text,
    )?;
    let independent_holdout_quarter_day_boundary_summary_checksum =
        checksum64(&independent_holdout_quarter_day_boundary_summary_text);
    let production_generation_source_revision_summary_text = read_required_bundle_text(
        &production_generation_source_revision_summary_path,
        "production generation source revision summary",
    )?;
    ensure_production_generation_source_revision_summary_matches_current_rendering(
        &production_generation_source_revision_summary_text,
    )?;
    let production_generation_source_window_summary_text = read_required_bundle_text(
        &production_generation_source_window_summary_path,
        "production generation source window summary",
    )?;
    let production_generation_quarter_day_boundary_summary_text = read_required_bundle_text(
        &production_generation_quarter_day_boundary_summary_path,
        "production generation quarter-day boundary summary",
    )?;
    let production_generation_corpus_shape_summary_text = read_required_bundle_text(
        &production_generation_corpus_shape_summary_path,
        "production generation corpus shape summary",
    )?;
    let production_generation_boundary_source_summary_text = read_required_bundle_text(
        &production_generation_boundary_source_summary_path,
        "production generation boundary source summary",
    )?;
    let production_generation_boundary_window_summary_text = read_required_bundle_text(
        &production_generation_boundary_window_summary_path,
        "production generation boundary window summary",
    )?;
    let production_generation_boundary_request_corpus_summary_text = read_required_bundle_text(
        &production_generation_boundary_request_corpus_summary_path,
        "production generation boundary request corpus summary",
    )?;
    let production_generation_boundary_request_corpus_equatorial_summary_text =
        read_required_bundle_text(
            &production_generation_boundary_request_corpus_equatorial_summary_path,
            "production generation boundary request corpus equatorial summary",
        )?;
    ensure_production_generation_boundary_source_summary_matches_current_rendering(
        &production_generation_boundary_source_summary_text,
    )?;
    ensure_production_generation_boundary_window_summary_matches_current_rendering(
        &production_generation_boundary_window_summary_text,
    )?;
    ensure_production_generation_boundary_request_corpus_summary_matches_current_rendering(
        &production_generation_boundary_request_corpus_summary_text,
    )?;
    ensure_production_generation_boundary_request_corpus_equatorial_summary_matches_current_rendering(
        &production_generation_boundary_request_corpus_equatorial_summary_text,
    )?;
    let production_generation_boundary_window_summary_checksum =
        checksum64(&production_generation_boundary_window_summary_text);
    let production_generation_boundary_request_corpus_equatorial_summary_checksum =
        checksum64(&production_generation_boundary_request_corpus_equatorial_summary_text);
    let production_generation_manifest_summary_text = read_required_bundle_text(
        &production_generation_manifest_summary_path,
        "production generation manifest summary",
    )?;
    ensure_production_generation_manifest_summary_matches_current_rendering(
        &production_generation_manifest_summary_text,
    )?;
    let production_generation_manifest_checksum_summary_text = read_required_bundle_text(
        &production_generation_manifest_checksum_path,
        "production generation manifest checksum summary",
    )?;
    ensure_production_generation_manifest_checksum_summary_matches_current_rendering(
        &production_generation_manifest_checksum_summary_text,
    )?;
    let reference_snapshot_summary_text = read_required_bundle_text(
        &reference_snapshot_summary_path,
        "reference snapshot summary",
    )?;
    ensure_reference_snapshot_summary_matches_current_rendering(&reference_snapshot_summary_text)?;
    let catalog_inventory_summary_text =
        read_required_bundle_text(&catalog_inventory_summary_path, "catalog inventory summary")?;
    ensure_catalog_inventory_summary_matches_current_rendering(&catalog_inventory_summary_text)?;
    let catalog_posture_summary_text =
        read_required_bundle_text(&catalog_posture_summary_path, "catalog posture summary")?;
    let custom_definition_ayanamsa_labels_summary_text = read_required_bundle_text(
        &custom_definition_ayanamsa_labels_summary_path,
        "custom-definition ayanamsa labels summary",
    )?;
    let ayanamsa_provenance_summary_text = read_required_bundle_text(
        &ayanamsa_provenance_summary_path,
        "ayanamsa provenance summary",
    )?;
    let validation_report_summary_text =
        read_required_bundle_text(&validation_report_summary_path, "validation report summary")?;
    let release_body_claims_summary_text = read_required_bundle_text(
        &release_body_claims_summary_path,
        "release body claims summary",
    )?;
    let body_date_channel_claims_summary_path =
        output_dir.join("body-date-channel-claims-summary.txt");
    let body_date_channel_claims_summary_text = read_required_bundle_text(
        &body_date_channel_claims_summary_path,
        "body/date/channel claims summary",
    )?;
    let body_date_channel_claims_summary_checksum =
        checksum64(&body_date_channel_claims_summary_text);
    let pluto_fallback_summary_text =
        read_required_bundle_text(&pluto_fallback_summary_path, "pluto fallback summary")?;
    let request_policy_summary_text =
        read_required_bundle_text(&request_policy_summary_path, "request policy summary")?;
    let observer_policy_summary_text =
        read_required_bundle_text(&observer_policy_summary_path, "observer policy summary")?;
    let apparentness_policy_summary_text = read_required_bundle_text(
        &apparentness_policy_summary_path,
        "apparentness policy summary",
    )?;
    let request_semantics_summary_text =
        read_required_bundle_text(&request_semantics_summary_path, "request-semantics summary")?;
    let unsupported_modes_summary_text =
        read_required_bundle_text(&unsupported_modes_summary_path, "unsupported-modes summary")?;
    let time_scale_policy_summary_text =
        read_required_bundle_text(&time_scale_policy_summary_path, "time-scale policy summary")?;
    let utc_convenience_policy_summary_text = read_required_bundle_text(
        &utc_convenience_policy_summary_path,
        "utc-convenience policy summary",
    )?;
    let delta_t_policy_summary_text =
        read_required_bundle_text(&delta_t_policy_summary_path, "delta-t policy summary")?;
    let native_sidereal_policy_summary_text = read_required_bundle_text(
        &native_sidereal_policy_summary_path,
        "native sidereal policy summary",
    )?;
    let zodiac_policy_summary_text =
        read_required_bundle_text(&zodiac_policy_summary_path, "zodiac policy summary")?;
    let lunar_theory_limitations_summary_text = read_required_bundle_text(
        &lunar_theory_limitations_summary_path,
        "lunar theory limitations summary",
    )?;
    ensure_lunar_theory_limitations_summary_matches_current_rendering(
        &lunar_theory_limitations_summary_text,
    )?;
    let lunar_theory_source_selection_summary_text = read_required_bundle_text(
        &lunar_theory_source_selection_summary_path,
        "lunar theory source selection summary",
    )?;
    let lunar_theory_source_family_summary_text = read_required_bundle_text(
        &lunar_theory_source_family_summary_path,
        "lunar theory source family summary",
    )?;
    let lunar_source_window_summary_text = read_required_bundle_text(
        &lunar_source_window_summary_path,
        "lunar source window summary",
    )?;
    ensure_lunar_theory_source_selection_summary_matches_current_rendering(
        &lunar_theory_source_selection_summary_text,
    )?;
    ensure_lunar_theory_source_family_summary_matches_current_rendering(
        &lunar_theory_source_family_summary_text,
    )?;
    if lunar_source_window_summary_text != lunar_source_window_summary_for_report() {
        return Err(ReleaseBundleError::Verification(
            "lunar source window summary no longer matches the current lunar source-window posture"
                .to_string(),
        ));
    }
    let lunar_theory_source_selection_summary_checksum =
        checksum64(&lunar_theory_source_selection_summary_text);
    let lunar_theory_source_family_summary_checksum =
        checksum64(&lunar_theory_source_family_summary_text);
    let lunar_source_window_summary_checksum = checksum64(&lunar_source_window_summary_text);
    let lunar_theory_catalog_validation_summary_text = read_required_bundle_text(
        &lunar_theory_catalog_validation_summary_path,
        "lunar theory catalog validation summary",
    )?;
    ensure_lunar_theory_catalog_validation_summary_matches_current_rendering(
        &lunar_theory_catalog_validation_summary_text,
    )?;
    let request_surface_summary_text =
        read_required_bundle_text(&request_surface_summary_path, "request surface summary")?;
    ensure_request_policy_summary_matches_current_rendering(&request_policy_summary_text)?;
    ensure_request_surface_summary_matches_current_rendering(&request_surface_summary_text)?;
    let compatibility_caveats_summary_text = read_required_bundle_text(
        &compatibility_caveats_summary_path,
        "compatibility caveats summary",
    )?;
    ensure_compatibility_caveats_summary_matches_current_rendering(
        &compatibility_caveats_summary_text,
    )?;
    let workspace_provenance_summary_text = read_required_bundle_text(
        &workspace_provenance_summary_path,
        "workspace provenance summary",
    )?;
    ensure_workspace_provenance_summary_matches_current_rendering(
        &workspace_provenance_summary_text,
    )?;
    let workspace_audit_summary_text =
        read_required_bundle_text(&workspace_audit_summary_path, "workspace audit summary")?;
    let workspace_provenance_summary_checksum = checksum64(&workspace_provenance_summary_text);
    let native_dependency_audit_summary_text = read_required_bundle_text(
        &native_dependency_audit_summary_path,
        "native-dependency audit summary",
    )?;
    if workspace_audit_summary_text != native_dependency_audit_summary_text {
        return Err(ReleaseBundleError::Verification(
            "native-dependency audit summary no longer matches the workspace audit summary"
                .to_string(),
        ));
    }
    ensure_native_dependency_audit_summary_matches_current_rendering(
        &native_dependency_audit_summary_text,
    )?;
    let native_dependency_audit_summary_checksum =
        checksum64(&native_dependency_audit_summary_text);
    let artifact_summary_text =
        read_required_bundle_text(&artifact_summary_path, "artifact summary")?;
    let packaged_artifact_profile_coverage_summary_text = read_required_bundle_text(
        &packaged_artifact_profile_coverage_summary_path,
        "packaged-artifact profile coverage summary",
    )?;
    ensure_packaged_artifact_profile_coverage_summary_matches_current_rendering(
        &packaged_artifact_profile_coverage_summary_text,
    )?;
    let packaged_artifact_access_summary_text = read_required_bundle_text(
        &packaged_artifact_access_summary_path,
        "packaged-artifact access summary",
    )?;
    ensure_packaged_artifact_access_summary_matches_current_rendering(
        &packaged_artifact_access_summary_text,
    )?;
    let packaged_artifact_access_summary_checksum =
        checksum64(&packaged_artifact_access_summary_text);
    let packaged_artifact_output_support_summary_text = read_required_bundle_text(
        &packaged_artifact_output_support_summary_path,
        "packaged-artifact output support summary",
    )?;
    ensure_packaged_artifact_output_support_summary_matches_current_rendering(
        &packaged_artifact_output_support_summary_text,
    )?;
    let packaged_artifact_output_support_summary_checksum =
        checksum64(&packaged_artifact_output_support_summary_text);
    let packaged_artifact_fit_sample_classes_summary_text = read_required_bundle_text(
        &packaged_artifact_fit_sample_classes_summary_path,
        "packaged-artifact fit sample classes summary",
    )?;
    ensure_packaged_artifact_fit_sample_classes_summary_matches_current_rendering(
        &packaged_artifact_fit_sample_classes_summary_text,
    )?;
    let packaged_artifact_fit_sample_classes_summary_checksum =
        checksum64(&packaged_artifact_fit_sample_classes_summary_text);
    let packaged_artifact_fit_threshold_violation_count_summary_text = read_required_bundle_text(
        &packaged_artifact_fit_threshold_violation_count_summary_path,
        "packaged-artifact fit threshold violation count summary",
    )?;
    ensure_packaged_artifact_fit_threshold_violation_count_summary_matches_current_rendering(
        &packaged_artifact_fit_threshold_violation_count_summary_text,
    )?;
    let packaged_artifact_fit_threshold_violation_count_summary_checksum =
        checksum64(&packaged_artifact_fit_threshold_violation_count_summary_text);
    let packaged_artifact_fit_threshold_violations_summary_text = read_required_bundle_text(
        &packaged_artifact_fit_threshold_violations_summary_path,
        "packaged-artifact fit threshold violations summary",
    )?;
    ensure_packaged_artifact_fit_threshold_violations_summary_matches_current_rendering(
        &packaged_artifact_fit_threshold_violations_summary_text,
    )?;
    let packaged_artifact_fit_threshold_violations_summary_checksum =
        checksum64(&packaged_artifact_fit_threshold_violations_summary_text);
    let packaged_artifact_body_cadence_summary_text = read_required_bundle_text(
        &packaged_artifact_body_cadence_summary_path,
        "packaged-artifact body cadence summary",
    )?;
    ensure_packaged_artifact_body_cadence_summary_matches_current_rendering(
        &packaged_artifact_body_cadence_summary_text,
    )?;
    let packaged_artifact_body_cadence_summary_checksum =
        checksum64(&packaged_artifact_body_cadence_summary_text);
    let packaged_artifact_body_class_span_cap_summary_text = read_required_bundle_text(
        &packaged_artifact_body_class_span_cap_summary_path,
        "packaged-artifact body-class span cap summary",
    )?;
    ensure_packaged_artifact_body_class_span_cap_summary_matches_current_rendering(
        &packaged_artifact_body_class_span_cap_summary_text,
    )?;
    let packaged_artifact_body_class_span_cap_summary_checksum =
        checksum64(&packaged_artifact_body_class_span_cap_summary_text);
    let packaged_artifact_normalized_intermediate_summary_text = read_required_bundle_text(
        &packaged_artifact_normalized_intermediate_summary_path,
        "packaged-artifact normalized intermediate summary",
    )?;
    let packaged_artifact_normalized_intermediate_summary_checksum =
        checksum64(&packaged_artifact_normalized_intermediate_summary_text);
    let packaged_artifact_speed_policy_summary_text = read_required_bundle_text(
        &packaged_artifact_speed_policy_summary_path,
        "packaged-artifact speed policy summary",
    )?;
    ensure_packaged_artifact_speed_policy_summary_matches_current_rendering(
        &packaged_artifact_speed_policy_summary_text,
    )?;
    let packaged_artifact_speed_policy_summary_checksum =
        checksum64(&packaged_artifact_speed_policy_summary_text);
    let packaged_artifact_storage_summary_text = read_required_bundle_text(
        &packaged_artifact_storage_summary_path,
        "packaged-artifact storage summary",
    )?;
    ensure_packaged_artifact_storage_summary_matches_current_rendering(
        &packaged_artifact_storage_summary_text,
    )?;
    let packaged_artifact_storage_summary_checksum =
        checksum64(&packaged_artifact_storage_summary_text);
    let packaged_artifact_production_profile_summary_text = read_required_bundle_text(
        &packaged_artifact_production_profile_summary_path,
        "packaged-artifact production-profile summary",
    )?;
    ensure_packaged_artifact_production_profile_summary_matches_current_rendering(
        &packaged_artifact_production_profile_summary_text,
    )?;
    let packaged_frame_treatment_summary_text = read_required_bundle_text(
        &packaged_frame_treatment_summary_path,
        "packaged frame treatment summary",
    )?;
    ensure_packaged_frame_treatment_summary_matches_current_rendering(
        &packaged_frame_treatment_summary_text,
    )?;
    let packaged_artifact_target_threshold_summary_text = read_required_bundle_text(
        &packaged_artifact_target_threshold_summary_path,
        "packaged-artifact target-threshold summary",
    )?;
    let packaged_artifact_target_threshold_state_summary_text = read_required_bundle_text(
        &packaged_artifact_target_threshold_state_summary_path,
        "packaged-artifact target-threshold state summary",
    )?;
    let packaged_artifact_source_fit_holdout_sync_summary_text = read_required_bundle_text(
        &packaged_artifact_source_fit_holdout_sync_summary_path,
        "packaged-artifact source-fit and hold-out sync summary",
    )?;
    let packaged_artifact_source_fit_holdout_sync_summary_checksum =
        checksum64(&packaged_artifact_source_fit_holdout_sync_summary_text);
    let packaged_artifact_target_threshold_scope_envelopes_summary_text =
        read_required_bundle_text(
            &packaged_artifact_target_threshold_scope_envelopes_summary_path,
            "packaged-artifact target-threshold scope envelopes summary",
        )?;
    ensure_packaged_artifact_target_threshold_scope_envelopes_summary_matches_current_rendering(
        &packaged_artifact_target_threshold_scope_envelopes_summary_text,
    )?;
    let packaged_artifact_phase2_corpus_alignment_summary_text = read_required_bundle_text(
        &packaged_artifact_phase2_corpus_alignment_summary_path,
        "packaged-artifact phase-2 corpus alignment summary",
    )?;
    let packaged_lookup_epoch_policy_summary_text = read_required_bundle_text(
        &packaged_lookup_epoch_policy_summary_path,
        "packaged lookup-epoch policy summary",
    )?;
    ensure_packaged_lookup_epoch_policy_summary_matches_current_rendering(
        &packaged_lookup_epoch_policy_summary_text,
    )?;
    let packaged_artifact_generation_policy_summary_text = read_required_bundle_text(
        &packaged_artifact_generation_policy_summary_path,
        "packaged-artifact generation policy summary",
    )?;
    ensure_packaged_artifact_generation_policy_summary_matches_current_rendering(
        &packaged_artifact_generation_policy_summary_text,
    )?;
    let packaged_artifact_generation_policy_summary_checksum =
        checksum64(&packaged_artifact_generation_policy_summary_text);
    let packaged_artifact_generation_residual_bodies_summary_text = read_required_bundle_text(
        &packaged_artifact_generation_residual_bodies_summary_path,
        "packaged-artifact generation residual bodies summary",
    )?;
    ensure_packaged_artifact_generation_residual_bodies_summary_matches_current_rendering(
        &packaged_artifact_generation_residual_bodies_summary_text,
    )?;
    let packaged_artifact_generation_residual_bodies_summary_checksum =
        checksum64(&packaged_artifact_generation_residual_bodies_summary_text);
    let packaged_artifact_regeneration_summary_text = read_required_bundle_text(
        &packaged_artifact_regeneration_summary_path,
        "packaged-artifact regeneration summary",
    )?;
    ensure_packaged_artifact_regeneration_summary_matches_current_rendering(
        &packaged_artifact_regeneration_summary_text,
    )?;
    let packaged_artifact_regeneration_summary_checksum =
        checksum64(&packaged_artifact_regeneration_summary_text);
    let packaged_artifact_generation_manifest_text = read_required_bundle_text(
        &packaged_artifact_generation_manifest_path,
        "packaged-artifact generation manifest",
    )?;
    let packaged_artifact_generation_manifest_summary_text = read_required_bundle_text(
        &packaged_artifact_generation_manifest_summary_path,
        "packaged-artifact generation manifest summary",
    )?;
    ensure_packaged_artifact_generation_manifest_summary_matches_current_rendering(
        &packaged_artifact_generation_manifest_summary_text,
    )?;
    let packaged_artifact_generation_manifest_checksum_summary_text = read_required_bundle_text(
        &packaged_artifact_generation_manifest_checksum_summary_path,
        "packaged-artifact generation manifest checksum summary",
    )?;
    ensure_packaged_artifact_generation_manifest_checksum_summary_matches_current_rendering(
        &packaged_artifact_generation_manifest_checksum_summary_text,
    )?;
    let packaged_artifact_generation_manifest_checksum_summary_checksum =
        checksum64(&packaged_artifact_generation_manifest_checksum_summary_text);
    let packaged_artifact_generation_manifest_checksum_text = read_required_bundle_text(
        &packaged_artifact_generation_manifest_checksum_path,
        "packaged-artifact generation manifest checksum sidecar",
    )?;
    let benchmark_corpus_summary_text =
        read_required_bundle_text(&benchmark_corpus_summary_path, "benchmark corpus summary")?;
    ensure_benchmark_corpus_summary_matches_current_rendering(&benchmark_corpus_summary_text)?;
    let chart_benchmark_corpus_summary_text = read_required_bundle_text(
        &chart_benchmark_corpus_summary_path,
        "chart benchmark corpus summary",
    )?;
    if chart_benchmark_corpus_summary_text
        != validated_chart_benchmark_corpus_summary_for_report()
            .map_err(ReleaseBundleError::Verification)?
    {
        return Err(ReleaseBundleError::Verification(
            "chart benchmark corpus summary no longer matches the current chart-benchmark corpus posture"
                .to_string(),
        ));
    }
    let selected_asteroid_source_request_corpus_summary_text = read_required_bundle_text(
        &selected_asteroid_source_request_corpus_summary_path,
        "selected asteroid source request corpus summary",
    )?;
    ensure_selected_asteroid_source_request_corpus_summary_matches_current_rendering(
        &selected_asteroid_source_request_corpus_summary_text,
    )?;
    let selected_asteroid_source_request_corpus_summary_checksum =
        checksum64(&selected_asteroid_source_request_corpus_summary_text);
    let selected_asteroid_source_request_corpus_equatorial_summary_text =
        read_required_bundle_text(
            &selected_asteroid_source_request_corpus_equatorial_summary_path,
            "selected asteroid source request corpus equatorial summary",
        )?;
    if selected_asteroid_source_request_corpus_equatorial_summary_text
        != validated_selected_asteroid_source_request_corpus_equatorial_summary_for_report()
            .map_err(ReleaseBundleError::Verification)?
    {
        return Err(ReleaseBundleError::Verification(
            "selected asteroid source request corpus equatorial summary no longer matches the current selected asteroid source request corpus equatorial posture".to_string(),
        ));
    }
    let selected_asteroid_source_request_corpus_equatorial_summary_checksum =
        checksum64(&selected_asteroid_source_request_corpus_equatorial_summary_text);
    let selected_asteroid_source_window_summary_text = read_required_bundle_text(
        &selected_asteroid_source_window_summary_path,
        "selected asteroid source window summary",
    )?;
    if selected_asteroid_source_window_summary_text
        != selected_asteroid_source_window_summary_for_report()
    {
        return Err(ReleaseBundleError::Verification(
            "selected asteroid source window summary no longer matches the current selected asteroid source window posture"
                .to_string(),
        ));
    }
    let selected_asteroid_source_window_summary_checksum =
        checksum64(&selected_asteroid_source_window_summary_text);
    let interpolation_quality_request_corpus_summary_text = read_required_bundle_text(
        &interpolation_quality_request_corpus_summary_path,
        "interpolation-quality sample request corpus summary",
    )?;
    ensure_interpolation_quality_request_corpus_summary_matches_current_rendering(
        &interpolation_quality_request_corpus_summary_text,
    )?;
    let interpolation_quality_request_corpus_summary_checksum =
        checksum64(&interpolation_quality_request_corpus_summary_text);
    let benchmark_report_text =
        read_required_bundle_text(&benchmark_report_path, "benchmark report")?;
    let validation_report_text =
        read_required_bundle_text(&validation_report_path, "validation report")?;
    let manifest_text = read_required_bundle_text(&manifest_path, "bundle manifest")?;
    let manifest_checksum_text =
        read_required_bundle_text(&manifest_checksum_path, "bundle manifest checksum sidecar")?;

    let manifest = ParsedReleaseBundleManifest::parse(&manifest_text)?;
    ensure_release_bundle_manifest_is_canonical(&manifest_text)?;
    ensure_release_bundle_directory_contents(output_dir)?;
    ensure_canonical_manifest_value(&manifest.source_revision, "source revision")?;
    ensure_canonical_manifest_value(&manifest.workspace_status, "workspace status")?;
    ensure_canonical_manifest_value(&manifest.rustc_version, "rustc version")?;
    ensure_canonical_manifest_value(&manifest.cargo_version, "cargo version")?;
    ensure_canonical_manifest_value(&manifest.profile_id, "profile id")?;
    ensure_canonical_manifest_value(
        &manifest.api_stability_posture_id,
        "API stability posture id",
    )?;
    ensure_validation_report_matches_current_rendering(
        &validation_report_text,
        manifest.validation_rounds,
    )?;
    // The fit-envelope / fit-margin / fit-outlier posture lines are staged in
    // validation-report-summary.txt, not the full validation report body.
    ensure_validation_report_fit_envelope_matches_current_rendering(
        &validation_report_summary_text,
    )?;
    ensure_validation_report_fit_margin_matches_current_rendering(&validation_report_summary_text)?;
    ensure_validation_report_fit_outliers_matches_current_rendering(
        &validation_report_summary_text,
    )?;
    ensure_validation_report_fit_sample_classes_matches_current_rendering(
        &validation_report_summary_text,
    )?;
    ensure_validation_report_fit_threshold_violation_count_matches_current_rendering(
        &validation_report_summary_text,
    )?;
    ensure_validation_report_fit_threshold_violations_matches_current_rendering(
        &validation_report_summary_text,
    )?;
    if validate_validation_report_summary {
        ensure_validation_report_summary_matches_current_rendering(
            &validation_report_summary_text,
            manifest.validation_rounds,
        )?;
    }
    ensure_backend_matrix_selected_asteroid_source_lines_match_current_rendering(
        &backend_matrix_text,
    )?;
    if manifest.profile_path != "compatibility-profile.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected profile file entry: {}",
            manifest.profile_path
        )));
    }
    if manifest.profile_summary_path != "compatibility-profile-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected compatibility profile summary file entry: {}",
            manifest.profile_summary_path
        )));
    }
    if manifest.release_notes_path != "release-notes.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release notes file entry: {}",
            manifest.release_notes_path
        )));
    }
    if manifest.release_notes_summary_path != "release-notes-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release notes summary file entry: {}",
            manifest.release_notes_summary_path
        )));
    }
    if manifest.release_summary_path != "release-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release summary file entry: {}",
            manifest.release_summary_path
        )));
    }
    if manifest.release_profile_identifiers_path != "release-profile-identifiers.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release-profile identifiers file entry: {}",
            manifest.release_profile_identifiers_path
        )));
    }
    if manifest.release_profile_identifiers_summary_path
        != "release-profile-identifiers-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release-profile identifiers summary file entry: {}",
            manifest.release_profile_identifiers_summary_path
        )));
    }
    if manifest.release_house_system_canonical_names_summary_path
        != "release-house-system-canonical-names-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release-house-system canonical names summary file entry: {}",
            manifest.release_house_system_canonical_names_summary_path
        )));
    }
    if manifest.release_ayanamsa_canonical_names_summary_path
        != "release-ayanamsa-canonical-names-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release-ayanamsa canonical names summary file entry: {}",
            manifest.release_ayanamsa_canonical_names_summary_path
        )));
    }
    if manifest.release_house_validation_summary_path != "release-house-validation-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release house validation summary file entry: {}",
            manifest.release_house_validation_summary_path
        )));
    }
    if manifest.target_house_scope_summary_path != "target-house-scope-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected target-house scope summary file entry: {}",
            manifest.target_house_scope_summary_path
        )));
    }
    if manifest.target_ayanamsa_scope_summary_path != "target-ayanamsa-scope-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected target-ayanamsa scope summary file entry: {}",
            manifest.target_ayanamsa_scope_summary_path
        )));
    }
    if manifest.house_code_aliases_summary_path != "house-code-aliases-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected house code aliases summary file entry: {}",
            manifest.house_code_aliases_summary_path
        )));
    }
    if manifest.house_formula_families_summary_path != "house-formula-families-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected house formula families summary file entry: {}",
            manifest.house_formula_families_summary_path
        )));
    }
    if manifest.house_latitude_sensitive_summary_path != "house-latitude-sensitive-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected house latitude-sensitive summary file entry: {}",
            manifest.house_latitude_sensitive_summary_path
        )));
    }
    if manifest.house_latitude_sensitive_constraints_summary_path
        != "house-latitude-sensitive-constraints-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected house latitude-sensitive constraints summary file entry: {}",
            manifest.house_latitude_sensitive_constraints_summary_path
        )));
    }
    if manifest.house_latitude_sensitive_failure_modes_summary_path
        != "house-latitude-sensitive-failure-modes-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected house latitude-sensitive failure-modes summary file entry: {}",
            manifest.house_latitude_sensitive_failure_modes_summary_path
        )));
    }
    if manifest.release_checklist_path != "release-checklist.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release checklist file entry: {}",
            manifest.release_checklist_path
        )));
    }
    if manifest.release_checklist_summary_path != "release-checklist-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release checklist summary file entry: {}",
            manifest.release_checklist_summary_path
        )));
    }
    if manifest.backend_matrix_path != "backend-matrix.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected backend matrix file entry: {}",
            manifest.backend_matrix_path
        )));
    }
    if manifest.backend_matrix_summary_path != "backend-matrix-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected backend matrix summary file entry: {}",
            manifest.backend_matrix_summary_path
        )));
    }
    if manifest.api_stability_path != "api-stability.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected API stability file entry: {}",
            manifest.api_stability_path
        )));
    }
    if manifest.api_stability_summary_path != "api-stability-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected API stability summary file entry: {}",
            manifest.api_stability_summary_path
        )));
    }
    if manifest.comparison_envelope_summary_path != "comparison-envelope-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected comparison envelope summary file entry: {}",
            manifest.comparison_envelope_summary_path
        )));
    }
    if manifest.comparison_body_class_tolerance_summary_path
        != "comparison-body-class-tolerance-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected comparison body-class tolerance summary file entry: {}",
            manifest.comparison_body_class_tolerance_summary_path
        )));
    }
    if manifest.comparison_body_class_error_envelope_summary_path
        != "comparison-body-class-error-envelope-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected comparison body-class error-envelope summary file entry: {}",
            manifest.comparison_body_class_error_envelope_summary_path
        )));
    }
    if manifest.comparison_corpus_release_guard_summary_path
        != "comparison-corpus-release-guard-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected comparison-corpus release-guard summary file entry: {}",
            manifest.comparison_corpus_release_guard_summary_path
        )));
    }
    if manifest.reference_holdout_overlap_summary_path != "reference-holdout-overlap-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference-holdout overlap summary file entry: {}",
            manifest.reference_holdout_overlap_summary_path
        )));
    }
    if manifest.catalog_inventory_summary_path != "catalog-inventory-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected catalog inventory summary file entry: {}",
            manifest.catalog_inventory_summary_path
        )));
    }
    if manifest.catalog_posture_summary_path != "catalog-posture-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected catalog posture summary file entry: {}",
            manifest.catalog_posture_summary_path
        )));
    }
    if manifest.custom_definition_ayanamsa_labels_summary_path
        != "custom-definition-ayanamsa-labels-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected custom-definition ayanamsa labels summary file entry: {}",
            manifest.custom_definition_ayanamsa_labels_summary_path
        )));
    }
    if manifest.ayanamsa_provenance_summary_path != "ayanamsa-provenance-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected ayanamsa provenance summary file entry: {}",
            manifest.ayanamsa_provenance_summary_path
        )));
    }
    if manifest.validation_report_summary_path != "validation-report-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected validation report summary file entry: {}",
            manifest.validation_report_summary_path
        )));
    }
    if manifest.release_body_claims_summary_path != "release-body-claims-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release body claims summary file entry: {}",
            manifest.release_body_claims_summary_path
        )));
    }
    if manifest.body_date_channel_claims_summary_path != "body-date-channel-claims-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected body/date/channel claims summary file entry: {}",
            manifest.body_date_channel_claims_summary_path
        )));
    }
    if manifest.pluto_fallback_summary_path != "pluto-fallback-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected pluto fallback summary file entry: {}",
            manifest.pluto_fallback_summary_path
        )));
    }
    if manifest.request_policy_summary_path != "request-policy-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected request policy summary file entry: {}",
            manifest.request_policy_summary_path
        )));
    }
    if manifest.observer_policy_summary_path != "observer-policy-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected observer policy summary file entry: {}",
            manifest.observer_policy_summary_path
        )));
    }
    if manifest.apparentness_policy_summary_path != "apparentness-policy-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected apparentness policy summary file entry: {}",
            manifest.apparentness_policy_summary_path
        )));
    }
    if manifest.request_semantics_summary_path != "request-semantics-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected request-semantics summary file entry: {}",
            manifest.request_semantics_summary_path
        )));
    }
    if manifest.unsupported_modes_summary_path != "unsupported-modes-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected unsupported-modes summary file entry: {}",
            manifest.unsupported_modes_summary_path
        )));
    }
    if manifest.time_scale_policy_summary_path != "time-scale-policy-summary.txt" {
        if manifest.utc_convenience_policy_summary_path != "utc-convenience-policy-summary.txt" {
            return Err(ReleaseBundleError::Verification(format!(
                "unexpected UTC convenience policy summary file entry: {}",
                manifest.utc_convenience_policy_summary_path
            )));
        }
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected time-scale policy summary file entry: {}",
            manifest.time_scale_policy_summary_path
        )));
    }
    if manifest.delta_t_policy_summary_path != "delta-t-policy-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected delta-t policy summary file entry: {}",
            manifest.delta_t_policy_summary_path
        )));
    }
    if manifest.native_sidereal_policy_summary_path != "native-sidereal-policy-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected native sidereal policy summary file entry: {}",
            manifest.native_sidereal_policy_summary_path
        )));
    }
    if manifest.zodiac_policy_summary_path != "zodiac-policy-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected zodiac policy summary file entry: {}",
            manifest.zodiac_policy_summary_path
        )));
    }
    if manifest.lunar_theory_limitations_summary_path != "lunar-theory-limitations-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected lunar theory limitations summary file entry: {}",
            manifest.lunar_theory_limitations_summary_path
        )));
    }
    if manifest.lunar_theory_source_selection_summary_path
        != "lunar-theory-source-selection-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected lunar theory source selection summary file entry: {}",
            manifest.lunar_theory_source_selection_summary_path
        )));
    }
    if manifest.lunar_theory_source_family_summary_path != "lunar-theory-source-family-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected lunar theory source family summary file entry: {}",
            manifest.lunar_theory_source_family_summary_path
        )));
    }
    if manifest.lunar_source_window_summary_path != "lunar-source-window-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected lunar source window summary file entry: {}",
            manifest.lunar_source_window_summary_path
        )));
    }
    if manifest.lunar_theory_catalog_validation_summary_path
        != "lunar-theory-catalog-validation-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected lunar theory catalog validation summary file entry: {}",
            manifest.lunar_theory_catalog_validation_summary_path
        )));
    }
    if manifest.request_surface_summary_path != "request-surface-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected request surface summary file entry: {}",
            manifest.request_surface_summary_path
        )));
    }
    if manifest.compatibility_caveats_summary_path != "compatibility-caveats-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected compatibility caveats summary file entry: {}",
            manifest.compatibility_caveats_summary_path
        )));
    }
    if manifest.workspace_provenance_summary_path != "workspace-provenance-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected workspace provenance summary file entry: {}",
            manifest.workspace_provenance_summary_path
        )));
    }
    if manifest.workspace_audit_summary_path != "workspace-audit-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected workspace audit summary file entry: {}",
            manifest.workspace_audit_summary_path
        )));
    }
    if manifest.native_dependency_audit_summary_path != "native-dependency-audit-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected native-dependency audit summary file entry: {}",
            manifest.native_dependency_audit_summary_path
        )));
    }
    if manifest.artifact_summary_path != "artifact-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected artifact summary file entry: {}",
            manifest.artifact_summary_path
        )));
    }
    if manifest.packaged_artifact_path != "packaged-artifact.bin" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact binary file entry: {}",
            manifest.packaged_artifact_path
        )));
    }
    if manifest.packaged_artifact_checksum_path != "packaged-artifact.checksum.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact checksum sidecar file entry: {}",
            manifest.packaged_artifact_checksum_path
        )));
    }
    if manifest.packaged_artifact_profile_coverage_summary_path
        != "packaged-artifact-profile-coverage-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact profile coverage summary file entry: {}",
            manifest.packaged_artifact_profile_coverage_summary_path
        )));
    }
    if manifest.packaged_artifact_access_summary_path != "packaged-artifact-access-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact access summary file entry: {}",
            manifest.packaged_artifact_access_summary_path
        )));
    }
    if manifest.packaged_artifact_speed_policy_summary_path
        != "packaged-artifact-speed-policy-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact speed policy summary file entry: {}",
            manifest.packaged_artifact_speed_policy_summary_path
        )));
    }
    if manifest.packaged_lookup_epoch_policy_summary_path
        != "packaged-lookup-epoch-policy-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged lookup-epoch policy summary file entry: {}",
            manifest.packaged_lookup_epoch_policy_summary_path
        )));
    }
    if manifest.packaged_artifact_generation_policy_summary_path
        != "packaged-artifact-generation-policy-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact generation policy summary file entry: {}",
            manifest.packaged_artifact_generation_policy_summary_path
        )));
    }
    if manifest.packaged_artifact_generation_residual_bodies_summary_path
        != "packaged-artifact-generation-residual-bodies-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact generation residual bodies summary file entry: {}",
            manifest.packaged_artifact_generation_residual_bodies_summary_path
        )));
    }
    if manifest.packaged_artifact_generation_manifest_path
        != "packaged-artifact-generation-manifest.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact generation manifest file entry: {}",
            manifest.packaged_artifact_generation_manifest_path
        )));
    }
    if manifest.packaged_artifact_generation_manifest_summary_path
        != "packaged-artifact-generation-manifest-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact generation manifest summary file entry: {}",
            manifest.packaged_artifact_generation_manifest_summary_path
        )));
    }
    if manifest.packaged_artifact_generation_manifest_checksum_summary_path
        != "packaged-artifact-generation-manifest-checksum-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact generation manifest checksum summary file entry: {}",
            manifest.packaged_artifact_generation_manifest_checksum_summary_path
        )));
    }
    if manifest.packaged_artifact_generation_manifest_checksum_path
        != "packaged-artifact-generation-manifest.checksum.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact generation manifest checksum sidecar file entry: {}",
            manifest.packaged_artifact_generation_manifest_checksum_path
        )));
    }
    if manifest.benchmark_report_path != "benchmark-report.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected benchmark report file entry: {}",
            manifest.benchmark_report_path
        )));
    }
    if manifest.validation_report_path != "validation-report.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected validation report file entry: {}",
            manifest.validation_report_path
        )));
    }

    let compatibility_profile_checksum = checksum64(&profile_text);
    let compatibility_profile_summary_checksum = checksum64(&profile_summary_text);
    let release_notes_summary_checksum = checksum64(&release_notes_summary_text);
    let release_summary_checksum = checksum64(&release_summary_text);
    let release_profile_identifiers_checksum = checksum64(&release_profile_identifiers_text);
    let release_profile_identifiers_summary_checksum =
        checksum64(&release_profile_identifiers_summary_text);
    let release_house_system_canonical_names_summary_text =
        render_release_house_system_canonical_names_summary();
    let release_house_system_canonical_names_summary_checksum =
        checksum64(&release_house_system_canonical_names_summary_text);
    let release_ayanamsa_canonical_names_summary_text =
        render_release_ayanamsa_canonical_names_summary();
    let release_ayanamsa_canonical_names_summary_checksum =
        checksum64(&release_ayanamsa_canonical_names_summary_text);
    let release_checklist_checksum = checksum64(&release_checklist_text);
    let release_checklist_summary_checksum = checksum64(&release_checklist_summary_text);
    let backend_matrix_checksum = checksum64(&backend_matrix_text);
    let backend_matrix_summary_checksum = checksum64(&backend_matrix_summary_text);
    let api_stability_checksum = checksum64(&api_stability_text);
    let api_stability_summary_checksum = checksum64(&api_stability_summary_text);
    let comparison_corpus_summary_checksum = checksum64(&comparison_corpus_summary_text);
    let source_corpus_summary_checksum = checksum64(&source_corpus_summary_text);
    if source_corpus_summary_text != source_corpus_summary_for_report() {
        return Err(ReleaseBundleError::Verification(
            "source corpus summary no longer matches the current source-corpus posture".to_string(),
        ));
    }
    let jpl_source_posture_summary_checksum = checksum64(&jpl_source_posture_summary_text);
    let comparison_snapshot_summary_checksum = checksum64(&comparison_snapshot_summary_text);
    let comparison_snapshot_source_summary_checksum =
        checksum64(&comparison_snapshot_source_summary_text);
    let comparison_snapshot_source_window_summary_checksum =
        checksum64(&comparison_snapshot_source_window_summary_text);
    let comparison_envelope_summary_checksum = checksum64(&comparison_envelope_summary_text);
    let comparison_corpus_release_guard_summary_checksum =
        checksum64(&comparison_corpus_release_guard_summary_text);
    let reference_holdout_overlap_summary_checksum =
        checksum64(&reference_holdout_overlap_summary_text);
    let reference_snapshot_bridge_day_summary_checksum =
        checksum64(&reference_snapshot_bridge_day_summary_text);
    let reference_snapshot_major_body_boundary_window_summary_checksum =
        checksum64(&reference_snapshot_major_body_boundary_window_summary_text);
    let reference_snapshot_boundary_epoch_coverage_summary_checksum =
        checksum64(&reference_snapshot_boundary_epoch_coverage_summary_text);
    let reference_snapshot_pre_bridge_boundary_summary_checksum =
        checksum64(&reference_snapshot_pre_bridge_boundary_summary_text);
    let reference_snapshot_2451917_major_body_boundary_summary_checksum =
        checksum64(&reference_snapshot_2451917_major_body_boundary_summary_text);
    let reference_snapshot_sparse_boundary_summary_text =
        reference_snapshot_sparse_boundary_summary_for_report();
    let reference_snapshot_2451918_major_body_boundary_summary_checksum =
        checksum64(&reference_snapshot_2451918_major_body_boundary_summary_text);
    let reference_snapshot_2451919_major_body_boundary_summary_checksum =
        checksum64(&reference_snapshot_2451919_major_body_boundary_summary_text);
    let reference_snapshot_sparse_boundary_summary_checksum =
        checksum64(&reference_snapshot_sparse_boundary_summary_text);
    let reference_snapshot_exact_j2000_evidence_summary_checksum =
        checksum64(&reference_snapshot_exact_j2000_evidence_summary_text);
    let reference_snapshot_source_summary_checksum =
        checksum64(&reference_snapshot_source_summary_text);
    let reference_snapshot_source_window_summary_checksum =
        checksum64(&reference_snapshot_source_window_summary_text);
    let reference_asteroid_source_window_summary_checksum =
        checksum64(&reference_asteroid_source_window_summary_text);
    let reference_snapshot_summary_checksum = checksum64(&reference_snapshot_summary_text);
    let independent_holdout_source_window_summary_checksum =
        checksum64(&independent_holdout_source_window_summary_text);
    let production_generation_boundary_source_summary_checksum =
        checksum64(&production_generation_boundary_source_summary_text);
    let production_generation_boundary_request_corpus_summary_checksum =
        checksum64(&production_generation_boundary_request_corpus_summary_text);
    let production_generation_summary_text = production_generation_snapshot_summary_for_report();
    let production_generation_summary_checksum = checksum64(&production_generation_summary_text);
    ensure_production_generation_summary_matches_current_rendering(
        &production_generation_summary_text,
    )?;
    let production_generation_body_class_coverage_summary_text =
        validated_production_generation_body_class_coverage_summary_for_report();
    let production_generation_body_class_coverage_summary_checksum =
        checksum64(&production_generation_body_class_coverage_summary_text);
    let production_generation_source_summary_text =
        production_generation_source_summary_for_report();
    let production_generation_source_summary_checksum =
        checksum64(&production_generation_source_summary_text);
    ensure_production_generation_source_summary_matches_current_rendering(
        &production_generation_source_summary_text,
    )?;
    let production_generation_source_revision_summary_checksum =
        checksum64(&production_generation_source_revision_summary_text);
    ensure_production_generation_body_class_coverage_summary_matches_current_rendering(
        &production_generation_body_class_coverage_summary_text,
    )?;
    let production_generation_source_window_summary_checksum =
        checksum64(&production_generation_source_window_summary_text);
    let production_generation_quarter_day_boundary_summary_checksum =
        checksum64(&production_generation_quarter_day_boundary_summary_text);
    let production_generation_corpus_shape_summary_checksum =
        checksum64(&production_generation_corpus_shape_summary_text);
    let production_generation_corpus_shape_summary_report =
        validated_production_generation_corpus_shape_summary_for_report()
            .map_err(ReleaseBundleError::Verification)?;
    if production_generation_corpus_shape_summary_text
        != production_generation_corpus_shape_summary_report
    {
        return Err(ReleaseBundleError::Verification(
            "production generation corpus shape summary no longer matches the current production-generation corpus shape posture"
                .to_string(),
        ));
    }
    let production_generation_manifest_summary_checksum =
        checksum64(&production_generation_manifest_summary_text);
    let production_generation_manifest_checksum_summary_checksum =
        checksum64(&production_generation_manifest_checksum_summary_text);
    let catalog_inventory_summary_checksum = checksum64(&catalog_inventory_summary_text);
    let catalog_posture_summary_checksum = checksum64(&catalog_posture_summary_text);
    let custom_definition_ayanamsa_labels_summary_checksum =
        checksum64(&custom_definition_ayanamsa_labels_summary_text);
    let ayanamsa_provenance_summary_checksum = checksum64(&ayanamsa_provenance_summary_text);
    let validation_report_summary_checksum = checksum64(&validation_report_summary_text);
    let release_body_claims_summary_checksum = checksum64(&release_body_claims_summary_text);
    let pluto_fallback_summary_checksum = checksum64(&pluto_fallback_summary_text);
    let request_policy_summary_checksum = checksum64(&request_policy_summary_text);
    let observer_policy_summary_checksum = checksum64(&observer_policy_summary_text);
    let apparentness_policy_summary_checksum = checksum64(&apparentness_policy_summary_text);
    let request_semantics_summary_checksum = checksum64(&request_semantics_summary_text);
    let unsupported_modes_summary_checksum = checksum64(&unsupported_modes_summary_text);
    let time_scale_policy_summary_checksum = checksum64(&time_scale_policy_summary_text);
    let utc_convenience_policy_summary_checksum = checksum64(&utc_convenience_policy_summary_text);
    let delta_t_policy_summary_checksum = checksum64(&delta_t_policy_summary_text);
    let native_sidereal_policy_summary_checksum = checksum64(&native_sidereal_policy_summary_text);
    let zodiac_policy_summary_checksum = checksum64(&zodiac_policy_summary_text);
    let lunar_theory_limitations_summary_checksum =
        checksum64(&lunar_theory_limitations_summary_text);
    let lunar_theory_catalog_validation_summary_checksum =
        checksum64(&lunar_theory_catalog_validation_summary_text);
    let request_surface_summary_checksum = checksum64(&request_surface_summary_text);
    let compatibility_caveats_summary_checksum = checksum64(&compatibility_caveats_summary_text);
    let workspace_audit_summary_checksum = checksum64(&workspace_audit_summary_text);
    let artifact_summary_checksum = checksum64(&artifact_summary_text);
    let packaged_artifact_profile_coverage_summary_checksum =
        checksum64(&packaged_artifact_profile_coverage_summary_text);
    let packaged_artifact_production_profile_summary_checksum =
        checksum64(&packaged_artifact_production_profile_summary_text);
    let packaged_frame_treatment_summary_checksum =
        checksum64(&packaged_frame_treatment_summary_text);
    let packaged_artifact_target_threshold_summary_checksum =
        checksum64(&packaged_artifact_target_threshold_summary_text);
    let packaged_artifact_target_threshold_state_summary_checksum =
        checksum64(&packaged_artifact_target_threshold_state_summary_text);
    let packaged_artifact_target_threshold_scope_envelopes_summary_checksum =
        checksum64(&packaged_artifact_target_threshold_scope_envelopes_summary_text);
    ensure_packaged_artifact_target_threshold_summary_matches_current_rendering(
        &packaged_artifact_target_threshold_summary_text,
    )?;
    ensure_packaged_artifact_target_threshold_state_matches_current_rendering(
        &packaged_artifact_target_threshold_state_summary_text,
    )?;
    ensure_packaged_artifact_source_fit_holdout_sync_summary_matches_current_rendering(
        &packaged_artifact_source_fit_holdout_sync_summary_text,
    )?;
    let packaged_artifact_phase2_corpus_alignment_summary_checksum =
        checksum64(&packaged_artifact_phase2_corpus_alignment_summary_text);
    let packaged_lookup_epoch_policy_summary_checksum =
        checksum64(&packaged_lookup_epoch_policy_summary_text);
    let packaged_artifact_generation_manifest_checksum =
        checksum64(&packaged_artifact_generation_manifest_text);
    let packaged_artifact_generation_manifest_checksum_text_checksum =
        checksum64(&packaged_artifact_generation_manifest_checksum_text);
    let packaged_artifact_generation_manifest_checksum_value = parse_checksum_value(
        &packaged_artifact_generation_manifest_checksum_text,
        "packaged-artifact generation manifest checksum sidecar",
    )?;
    let packaged_artifact_generation_manifest_summary_checksum =
        checksum64(&packaged_artifact_generation_manifest_summary_text);
    let benchmark_corpus_summary_checksum = checksum64(&benchmark_corpus_summary_text);
    let chart_benchmark_corpus_summary_checksum = checksum64(&chart_benchmark_corpus_summary_text);
    let benchmark_report_checksum = checksum64(&benchmark_report_text);
    let validation_report_checksum = checksum64(&validation_report_text);
    let manifest_checksum = checksum64(&manifest_text);
    let manifest_checksum_value =
        parse_checksum_value(&manifest_checksum_text, "bundle manifest checksum sidecar")?;
    let profile_id = extract_prefixed_value(&profile_text, "Compatibility profile: ")?;
    let api_stability_posture_id =
        extract_prefixed_value(&api_stability_text, "API stability posture: ")?;

    ensure_packaged_artifact_phase2_alignment_matches_source_fit_holdout_sync(
        &packaged_artifact_source_fit_holdout_sync_summary_text,
        &packaged_artifact_phase2_corpus_alignment_summary_text,
    )?;
    ensure_packaged_artifact_target_threshold_phase2_alignment_matches_source_fit_holdout_sync(
        &packaged_artifact_target_threshold_summary_text,
        &packaged_artifact_source_fit_holdout_sync_summary_text,
    )?;
    ensure_packaged_artifact_phase2_corpus_alignment_summary_matches_current_rendering(
        &packaged_artifact_phase2_corpus_alignment_summary_text,
    )?;
    ensure_comparison_corpus_release_guard_summary_matches_current_rendering(
        &comparison_corpus_release_guard_summary_text,
    )?;
    ensure_production_generation_source_summary_matches_source_windows(
        &production_generation_source_summary_text,
        &production_generation_source_window_summary_text,
    )?;
    ensure_production_generation_source_window_summary_matches_current_rendering(
        &production_generation_source_window_summary_text,
    )?;
    ensure_production_generation_quarter_day_boundary_summary_matches_current_rendering(
        &production_generation_quarter_day_boundary_summary_text,
    )?;

    if manifest.release_summary_checksum != release_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "release summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_summary_checksum, release_summary_checksum
        )));
    }
    if manifest.release_profile_identifiers_checksum != release_profile_identifiers_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "release-profile identifiers checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_profile_identifiers_checksum, release_profile_identifiers_checksum
        )));
    }
    if manifest.release_profile_identifiers_summary_checksum
        != release_profile_identifiers_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "release-profile identifiers summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_profile_identifiers_summary_checksum,
            release_profile_identifiers_summary_checksum
        )));
    }
    if manifest.release_house_system_canonical_names_summary_checksum
        != release_house_system_canonical_names_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "release-house-system canonical names summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_house_system_canonical_names_summary_checksum,
            release_house_system_canonical_names_summary_checksum
        )));
    }
    if manifest.release_ayanamsa_canonical_names_summary_checksum
        != release_ayanamsa_canonical_names_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "release-ayanamsa canonical names summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_ayanamsa_canonical_names_summary_checksum,
            release_ayanamsa_canonical_names_summary_checksum
        )));
    }
    ensure_release_house_system_canonical_names_summary_matches_current_rendering(
        &release_house_system_canonical_names_summary_text,
    )?;
    ensure_release_ayanamsa_canonical_names_summary_matches_current_rendering(
        &release_ayanamsa_canonical_names_summary_text,
    )?;
    if manifest.release_house_validation_summary_checksum
        != release_house_validation_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "release house validation summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_house_validation_summary_checksum,
            release_house_validation_summary_checksum
        )));
    }
    if manifest.target_house_scope_summary_checksum != target_house_scope_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "target-house scope summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.target_house_scope_summary_checksum, target_house_scope_summary_checksum
        )));
    }
    if manifest.target_ayanamsa_scope_summary_checksum != target_ayanamsa_scope_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "target-ayanamsa scope summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.target_ayanamsa_scope_summary_checksum, target_ayanamsa_scope_summary_checksum
        )));
    }
    if manifest.house_code_aliases_summary_checksum != house_code_aliases_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "house code aliases summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.house_code_aliases_summary_checksum,
            house_code_aliases_summary_checksum
        )));
    }
    if manifest.house_formula_families_summary_checksum != house_formula_families_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "house formula families summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.house_formula_families_summary_checksum,
            house_formula_families_summary_checksum
        )));
    }
    if manifest.house_latitude_sensitive_summary_checksum
        != house_latitude_sensitive_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "house latitude-sensitive summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.house_latitude_sensitive_summary_checksum,
            house_latitude_sensitive_summary_checksum
        )));
    }
    if manifest.house_latitude_sensitive_constraints_summary_checksum
        != house_latitude_sensitive_constraints_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "house latitude-sensitive constraints summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.house_latitude_sensitive_constraints_summary_checksum,
            house_latitude_sensitive_constraints_summary_checksum
        )));
    }
    if manifest.house_latitude_sensitive_failure_modes_summary_checksum
        != house_latitude_sensitive_failure_modes_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "house latitude-sensitive failure-modes summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.house_latitude_sensitive_failure_modes_summary_checksum,
            house_latitude_sensitive_failure_modes_summary_checksum
        )));
    }
    if manifest.release_checklist_checksum != release_checklist_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "release checklist checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_checklist_checksum, release_checklist_checksum
        )));
    }
    if manifest.release_checklist_summary_checksum != release_checklist_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "release checklist summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_checklist_summary_checksum, release_checklist_summary_checksum
        )));
    }
    ensure_release_checklist_summary_matches_current_rendering(&release_checklist_summary_text)?;
    if manifest.profile_id != profile_id {
        return Err(ReleaseBundleError::Verification(format!(
            "profile id mismatch: manifest has {}, file has {}",
            manifest.profile_id, profile_id
        )));
    }
    if manifest.profile_checksum != compatibility_profile_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "compatibility profile checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.profile_checksum, compatibility_profile_checksum
        )));
    }
    if manifest.profile_summary_checksum != compatibility_profile_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "compatibility profile summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.profile_summary_checksum, compatibility_profile_summary_checksum
        )));
    }
    if manifest.release_notes_summary_checksum != release_notes_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "release notes summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_notes_summary_checksum, release_notes_summary_checksum
        )));
    }
    let release_notes_checksum = checksum64(&release_notes_text);
    if manifest.release_notes_checksum != release_notes_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "release notes checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_notes_checksum, release_notes_checksum
        )));
    }
    if manifest.backend_matrix_checksum != backend_matrix_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "backend matrix checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.backend_matrix_checksum, backend_matrix_checksum
        )));
    }
    if manifest.backend_matrix_summary_checksum != backend_matrix_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "backend matrix summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.backend_matrix_summary_checksum, backend_matrix_summary_checksum
        )));
    }
    if manifest.api_stability_posture_id != api_stability_posture_id {
        return Err(ReleaseBundleError::Verification(format!(
            "API stability posture id mismatch: manifest has {}, file has {}",
            manifest.api_stability_posture_id, api_stability_posture_id
        )));
    }
    if manifest.api_stability_checksum != api_stability_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "API stability checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.api_stability_checksum, api_stability_checksum
        )));
    }
    if manifest.api_stability_summary_checksum != api_stability_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "API stability summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.api_stability_summary_checksum, api_stability_summary_checksum
        )));
    }
    if manifest.comparison_corpus_summary_checksum != comparison_corpus_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "comparison corpus summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.comparison_corpus_summary_checksum, comparison_corpus_summary_checksum
        )));
    }
    if manifest.source_corpus_summary_checksum != source_corpus_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "source corpus summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.source_corpus_summary_checksum, source_corpus_summary_checksum
        )));
    }
    if manifest.jpl_source_posture_summary_checksum != jpl_source_posture_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "JPL source posture summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.jpl_source_posture_summary_checksum, jpl_source_posture_summary_checksum
        )));
    }
    if manifest.jpl_provenance_only_summary_checksum != jpl_provenance_only_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "JPL provenance-only evidence summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.jpl_provenance_only_summary_checksum, jpl_provenance_only_summary_checksum
        )));
    }
    if manifest.comparison_snapshot_summary_checksum != comparison_snapshot_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "comparison snapshot summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.comparison_snapshot_summary_checksum, comparison_snapshot_summary_checksum
        )));
    }

    ensure_release_profile_line_alignment("release notes", &release_notes_text, profile_id)?;
    ensure_release_notes_matches_current_rendering(&release_notes_text)?;
    ensure_release_profile_summary_alignment(
        "release notes summary",
        &release_notes_summary_text,
        profile_id,
        api_stability_posture_id,
    )?;
    ensure_release_notes_summary_matches_current_rendering(&release_notes_summary_text)?;
    ensure_release_profile_summary_alignment(
        "release summary",
        &release_summary_text,
        profile_id,
        api_stability_posture_id,
    )?;
    ensure_release_profile_summary_alignment(
        "release checklist",
        &release_checklist_text,
        profile_id,
        api_stability_posture_id,
    )?;
    ensure_release_profile_identifiers_alignment(
        &release_profile_identifiers_text,
        profile_id,
        api_stability_posture_id,
    )?;
    ensure_release_profile_identifiers_summary_matches_current_rendering(
        &release_profile_identifiers_summary_text,
    )?;
    ensure_compatibility_profile_summary_matches_current_rendering(&profile_summary_text)?;
    ensure_release_summary_matches_current_rendering(&release_summary_text)?;

    if manifest.comparison_corpus_summary_path != "comparison-corpus-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected comparison corpus summary file entry: {}",
            manifest.comparison_corpus_summary_path
        )));
    }
    if manifest.source_corpus_summary_path != "source-corpus-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected source corpus summary file entry: {}",
            manifest.source_corpus_summary_path
        )));
    }
    if manifest.jpl_source_posture_summary_path != "jpl-source-posture-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected JPL source posture summary file entry: {}",
            manifest.jpl_source_posture_summary_path
        )));
    }
    if manifest.jpl_provenance_only_summary_path != "jpl-provenance-only-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected JPL provenance-only evidence summary file entry: {}",
            manifest.jpl_provenance_only_summary_path
        )));
    }
    if manifest.comparison_snapshot_summary_path != "comparison-snapshot-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected comparison snapshot summary file entry: {}",
            manifest.comparison_snapshot_summary_path
        )));
    }
    if manifest.comparison_snapshot_source_summary_path != "comparison-snapshot-source-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected comparison snapshot source summary file entry: {}",
            manifest.comparison_snapshot_source_summary_path
        )));
    }
    if manifest.comparison_snapshot_source_summary_checksum
        != comparison_snapshot_source_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "comparison snapshot source summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.comparison_snapshot_source_summary_checksum,
            comparison_snapshot_source_summary_checksum
        )));
    }
    if manifest.comparison_snapshot_source_window_summary_path
        != "comparison-snapshot-source-window-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected comparison snapshot source window summary file entry: {}",
            manifest.comparison_snapshot_source_window_summary_path
        )));
    }
    if manifest.comparison_snapshot_source_window_summary_checksum
        != comparison_snapshot_source_window_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "comparison snapshot source window summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.comparison_snapshot_source_window_summary_checksum,
            comparison_snapshot_source_window_summary_checksum
        )));
    }
    if manifest.comparison_snapshot_body_class_coverage_summary_path
        != "comparison-snapshot-body-class-coverage-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected comparison snapshot body-class coverage summary file entry: {}",
            manifest.comparison_snapshot_body_class_coverage_summary_path
        )));
    }
    if manifest.comparison_snapshot_body_class_coverage_summary_checksum
        != comparison_snapshot_body_class_coverage_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "comparison snapshot body-class coverage summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.comparison_snapshot_body_class_coverage_summary_checksum,
            comparison_snapshot_body_class_coverage_summary_checksum
        )));
    }
    if manifest.comparison_snapshot_manifest_summary_path
        != "comparison-snapshot-manifest-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected comparison snapshot manifest summary file entry: {}",
            manifest.comparison_snapshot_manifest_summary_path
        )));
    }
    if manifest.comparison_snapshot_manifest_summary_checksum
        != comparison_snapshot_manifest_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "comparison snapshot manifest summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.comparison_snapshot_manifest_summary_checksum,
            comparison_snapshot_manifest_summary_checksum
        )));
    }
    if manifest.comparison_envelope_summary_checksum != comparison_envelope_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "comparison envelope summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.comparison_envelope_summary_checksum, comparison_envelope_summary_checksum
        )));
    }
    ensure_comparison_envelope_summary_matches_current_rendering(
        &comparison_envelope_summary_text,
    )?;
    if manifest.comparison_body_class_tolerance_summary_checksum
        != comparison_body_class_tolerance_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "comparison body-class tolerance summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.comparison_body_class_tolerance_summary_checksum, comparison_body_class_tolerance_summary_checksum
        )));
    }
    ensure_comparison_body_class_tolerance_summary_matches_current_rendering(
        &comparison_body_class_tolerance_summary_text,
    )?;
    if manifest.comparison_body_class_error_envelope_summary_checksum
        != comparison_body_class_error_envelope_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "comparison body-class error-envelope summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.comparison_body_class_error_envelope_summary_checksum,
            comparison_body_class_error_envelope_summary_checksum
        )));
    }
    if comparison_body_class_error_envelope_summary_text
        != comparison_body_class_error_envelope_summary_for_report()
    {
        return Err(ReleaseBundleError::Verification(
            "comparison body-class error-envelope summary no longer matches the current comparison body-class error-envelope posture".to_string(),
        ));
    }
    if manifest.comparison_corpus_release_guard_summary_checksum
        != comparison_corpus_release_guard_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "comparison-corpus release-guard summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.comparison_corpus_release_guard_summary_checksum,
            comparison_corpus_release_guard_summary_checksum
        )));
    }
    if manifest.reference_holdout_overlap_summary_checksum
        != reference_holdout_overlap_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference-holdout overlap summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_holdout_overlap_summary_checksum,
            reference_holdout_overlap_summary_checksum
        )));
    }
    if manifest.reference_snapshot_bridge_day_summary_path
        != "reference-snapshot-bridge-day-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot bridge day summary file entry: {}",
            manifest.reference_snapshot_bridge_day_summary_path
        )));
    }
    if manifest.reference_snapshot_bridge_day_summary_checksum
        != reference_snapshot_bridge_day_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot bridge day summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_bridge_day_summary_checksum,
            reference_snapshot_bridge_day_summary_checksum
        )));
    }
    if manifest.reference_snapshot_major_body_boundary_window_summary_path
        != "reference-snapshot-major-body-boundary-window-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot major-body boundary window summary file entry: {}",
            manifest.reference_snapshot_major_body_boundary_window_summary_path
        )));
    }
    if manifest.reference_snapshot_major_body_boundary_window_summary_checksum
        != reference_snapshot_major_body_boundary_window_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot major-body boundary window summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_major_body_boundary_window_summary_checksum,
            reference_snapshot_major_body_boundary_window_summary_checksum
        )));
    }
    if manifest.reference_snapshot_boundary_epoch_coverage_summary_path
        != "reference-snapshot-boundary-epoch-coverage-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot boundary epoch coverage summary file entry: {}",
            manifest.reference_snapshot_boundary_epoch_coverage_summary_path
        )));
    }
    if manifest.reference_snapshot_boundary_epoch_coverage_summary_checksum
        != reference_snapshot_boundary_epoch_coverage_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot boundary epoch coverage summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_boundary_epoch_coverage_summary_checksum,
            reference_snapshot_boundary_epoch_coverage_summary_checksum
        )));
    }
    if manifest.reference_snapshot_pre_bridge_boundary_summary_path
        != "reference-snapshot-pre-bridge-boundary-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot pre-bridge boundary summary file entry: {}",
            manifest.reference_snapshot_pre_bridge_boundary_summary_path
        )));
    }
    if manifest.reference_snapshot_pre_bridge_boundary_summary_checksum
        != reference_snapshot_pre_bridge_boundary_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot pre-bridge boundary summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_pre_bridge_boundary_summary_checksum,
            reference_snapshot_pre_bridge_boundary_summary_checksum
        )));
    }
    if manifest.reference_snapshot_2451917_major_body_boundary_summary_path
        != "reference-snapshot-2451917-major-body-boundary-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot 2451917 major-body boundary summary file entry: {}",
            manifest.reference_snapshot_2451917_major_body_boundary_summary_path
        )));
    }
    if manifest.reference_snapshot_2451917_major_body_boundary_summary_checksum
        != reference_snapshot_2451917_major_body_boundary_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot 2451917 major-body boundary summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_2451917_major_body_boundary_summary_checksum,
            reference_snapshot_2451917_major_body_boundary_summary_checksum
        )));
    }
    if manifest.reference_snapshot_2451918_major_body_boundary_summary_path
        != "reference-snapshot-2451918-major-body-boundary-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot 2451918 major-body boundary summary file entry: {}",
            manifest.reference_snapshot_2451918_major_body_boundary_summary_path
        )));
    }
    if manifest.reference_snapshot_2451918_major_body_boundary_summary_checksum
        != reference_snapshot_2451918_major_body_boundary_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot 2451918 major-body boundary summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_2451918_major_body_boundary_summary_checksum,
            reference_snapshot_2451918_major_body_boundary_summary_checksum
        )));
    }
    if manifest.reference_snapshot_2451919_major_body_boundary_summary_path
        != "reference-snapshot-2451919-major-body-boundary-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot 2451919 major-body boundary summary file entry: {}",
            manifest.reference_snapshot_2451919_major_body_boundary_summary_path
        )));
    }
    if manifest.reference_snapshot_2451919_major_body_boundary_summary_checksum
        != reference_snapshot_2451919_major_body_boundary_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot 2451919 major-body boundary summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_2451919_major_body_boundary_summary_checksum,
            reference_snapshot_2451919_major_body_boundary_summary_checksum
        )));
    }
    if manifest.reference_snapshot_2451916_major_body_dense_boundary_summary_path
        != "reference-snapshot-2451916-major-body-dense-boundary-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot 2451916 major-body dense boundary summary file entry: {}",
            manifest.reference_snapshot_2451916_major_body_dense_boundary_summary_path
        )));
    }
    if manifest.reference_snapshot_2451916_major_body_dense_boundary_summary_checksum
        != reference_snapshot_2451916_major_body_dense_boundary_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot 2451916 major-body dense boundary summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_2451916_major_body_dense_boundary_summary_checksum,
            reference_snapshot_2451916_major_body_dense_boundary_summary_checksum
        )));
    }
    if manifest.reference_snapshot_sparse_boundary_summary_path
        != "reference-snapshot-sparse-boundary-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot sparse boundary summary file entry: {}",
            manifest.reference_snapshot_sparse_boundary_summary_path
        )));
    }
    if manifest.reference_snapshot_sparse_boundary_summary_checksum
        != reference_snapshot_sparse_boundary_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot sparse boundary summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_sparse_boundary_summary_checksum,
            reference_snapshot_sparse_boundary_summary_checksum
        )));
    }
    if manifest.reference_snapshot_exact_j2000_evidence_summary_path
        != "reference-snapshot-exact-j2000-evidence-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot exact J2000 evidence summary file entry: {}",
            manifest.reference_snapshot_exact_j2000_evidence_summary_path
        )));
    }
    if manifest.reference_snapshot_exact_j2000_evidence_summary_checksum
        != reference_snapshot_exact_j2000_evidence_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot exact J2000 evidence summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_exact_j2000_evidence_summary_checksum,
            reference_snapshot_exact_j2000_evidence_summary_checksum
        )));
    }
    if manifest.reference_snapshot_source_summary_path != "reference-snapshot-source-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot source summary file entry: {}",
            manifest.reference_snapshot_source_summary_path
        )));
    }
    if manifest.reference_snapshot_source_summary_checksum
        != reference_snapshot_source_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot source summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_source_summary_checksum,
            reference_snapshot_source_summary_checksum
        )));
    }
    if manifest.reference_snapshot_source_window_summary_path
        != "reference-snapshot-source-window-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot source window summary file entry: {}",
            manifest.reference_snapshot_source_window_summary_path
        )));
    }
    if manifest.reference_snapshot_source_window_summary_checksum
        != reference_snapshot_source_window_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot source window summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_source_window_summary_checksum,
            reference_snapshot_source_window_summary_checksum
        )));
    }
    if manifest.reference_snapshot_manifest_summary_path
        != "reference-snapshot-manifest-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot manifest summary file entry: {}",
            manifest.reference_snapshot_manifest_summary_path
        )));
    }
    if manifest.reference_snapshot_manifest_summary_checksum
        != reference_snapshot_manifest_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot manifest summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_manifest_summary_checksum,
            reference_snapshot_manifest_summary_checksum
        )));
    }
    if manifest.reference_snapshot_body_class_coverage_summary_path
        != "reference-snapshot-body-class-coverage-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot body-class coverage summary file entry: {}",
            manifest.reference_snapshot_body_class_coverage_summary_path
        )));
    }
    if manifest.reference_snapshot_body_class_coverage_summary_checksum
        != reference_snapshot_body_class_coverage_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot body-class coverage summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_body_class_coverage_summary_checksum,
            reference_snapshot_body_class_coverage_summary_checksum
        )));
    }
    if manifest.reference_snapshot_equatorial_parity_summary_path
        != "reference-snapshot-equatorial-parity-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot equatorial parity summary file entry: {}",
            manifest.reference_snapshot_equatorial_parity_summary_path
        )));
    }
    if manifest.reference_snapshot_equatorial_parity_summary_checksum
        != reference_snapshot_equatorial_parity_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot equatorial parity summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_equatorial_parity_summary_checksum,
            reference_snapshot_equatorial_parity_summary_checksum
        )));
    }
    if manifest.reference_asteroid_source_window_summary_path
        != "reference-asteroid-source-window-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference asteroid source window summary file entry: {}",
            manifest.reference_asteroid_source_window_summary_path
        )));
    }
    if manifest.reference_asteroid_source_window_summary_checksum
        != reference_asteroid_source_window_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference asteroid source window summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_asteroid_source_window_summary_checksum,
            reference_asteroid_source_window_summary_checksum
        )));
    }
    if manifest.reference_asteroid_equatorial_evidence_summary_path
        != "reference-asteroid-equatorial-evidence-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference asteroid equatorial evidence summary file entry: {}",
            manifest.reference_asteroid_equatorial_evidence_summary_path
        )));
    }
    if manifest.reference_asteroid_equatorial_evidence_summary_checksum
        != reference_asteroid_equatorial_evidence_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "reference asteroid equatorial evidence summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_asteroid_equatorial_evidence_summary_checksum,
            reference_asteroid_equatorial_evidence_summary_checksum
        )));
    }
    if manifest.independent_holdout_source_window_summary_path
        != "independent-holdout-source-window-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected independent-holdout source window summary file entry: {}",
            manifest.independent_holdout_source_window_summary_path
        )));
    }
    if manifest.independent_holdout_source_window_summary_checksum
        != independent_holdout_source_window_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "independent-holdout source window summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.independent_holdout_source_window_summary_checksum,
            independent_holdout_source_window_summary_checksum
        )));
    }
    ensure_independent_holdout_source_window_summary_matches_current_rendering(
        &independent_holdout_source_window_summary_text,
    )?;
    if manifest.independent_holdout_equatorial_parity_summary_path
        != "independent-holdout-equatorial-parity-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected independent-holdout equatorial parity summary file entry: {}",
            manifest.independent_holdout_equatorial_parity_summary_path
        )));
    }
    if manifest.independent_holdout_equatorial_parity_summary_checksum
        != independent_holdout_equatorial_parity_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "independent-holdout equatorial parity summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.independent_holdout_equatorial_parity_summary_checksum,
            independent_holdout_equatorial_parity_summary_checksum
        )));
    }
    if manifest.independent_holdout_body_class_coverage_summary_path
        != "independent-holdout-body-class-coverage-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected independent-holdout body-class coverage summary file entry: {}",
            manifest.independent_holdout_body_class_coverage_summary_path
        )));
    }
    if manifest.independent_holdout_body_class_coverage_summary_checksum
        != independent_holdout_body_class_coverage_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "independent-holdout body-class coverage summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.independent_holdout_body_class_coverage_summary_checksum,
            independent_holdout_body_class_coverage_summary_checksum
        )));
    }
    ensure_independent_holdout_body_class_coverage_summary_matches_current_rendering(
        &independent_holdout_body_class_coverage_summary_text,
    )?;
    if manifest.independent_holdout_quarter_day_boundary_summary_path
        != "independent-holdout-quarter-day-boundary-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected independent-holdout quarter-day boundary summary file entry: {}",
            manifest.independent_holdout_quarter_day_boundary_summary_path
        )));
    }
    if manifest.independent_holdout_quarter_day_boundary_summary_checksum
        != independent_holdout_quarter_day_boundary_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "independent-holdout quarter-day boundary summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.independent_holdout_quarter_day_boundary_summary_checksum,
            independent_holdout_quarter_day_boundary_summary_checksum
        )));
    }
    ensure_independent_holdout_quarter_day_boundary_summary_matches_current_rendering(
        &independent_holdout_quarter_day_boundary_summary_text,
    )?;
    if manifest.production_generation_boundary_source_summary_path
        != "production-generation-boundary-source-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected production generation boundary source summary file entry: {}",
            manifest.production_generation_boundary_source_summary_path
        )));
    }
    if manifest.production_generation_boundary_source_summary_checksum
        != production_generation_boundary_source_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "production generation boundary source summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.production_generation_boundary_source_summary_checksum,
            production_generation_boundary_source_summary_checksum
        )));
    }
    if manifest.production_generation_boundary_window_summary_path
        != "production-generation-boundary-window-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected production generation boundary window summary file entry: {}",
            manifest.production_generation_boundary_window_summary_path
        )));
    }
    if manifest.production_generation_boundary_window_summary_checksum
        != production_generation_boundary_window_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "production generation boundary window summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.production_generation_boundary_window_summary_checksum,
            production_generation_boundary_window_summary_checksum
        )));
    }
    if manifest.production_generation_boundary_request_corpus_summary_path
        != "production-generation-boundary-request-corpus-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected production generation boundary request corpus summary file entry: {}",
            manifest.production_generation_boundary_request_corpus_summary_path
        )));
    }
    if manifest.production_generation_boundary_request_corpus_summary_checksum
        != production_generation_boundary_request_corpus_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "production generation boundary request corpus summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.production_generation_boundary_request_corpus_summary_checksum,
            production_generation_boundary_request_corpus_summary_checksum
        )));
    }
    if manifest.production_generation_boundary_request_corpus_equatorial_summary_path
        != "production-generation-boundary-request-corpus-equatorial-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected production generation boundary request corpus equatorial summary file entry: {}",
            manifest.production_generation_boundary_request_corpus_equatorial_summary_path
        )));
    }
    if manifest.production_generation_boundary_request_corpus_equatorial_summary_checksum
        != production_generation_boundary_request_corpus_equatorial_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "production generation boundary request corpus equatorial summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.production_generation_boundary_request_corpus_equatorial_summary_checksum,
            production_generation_boundary_request_corpus_equatorial_summary_checksum
        )));
    }
    if manifest.reference_snapshot_summary_path != "reference-snapshot-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected reference snapshot summary file entry: {}",
            manifest.reference_snapshot_summary_path
        )));
    }
    if manifest.reference_snapshot_summary_checksum != reference_snapshot_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "reference snapshot summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.reference_snapshot_summary_checksum, reference_snapshot_summary_checksum
        )));
    }
    if manifest.production_generation_summary_path != "production-generation-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected production generation summary file entry: {}",
            manifest.production_generation_summary_path
        )));
    }
    if manifest.production_generation_summary_checksum != production_generation_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "production generation summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.production_generation_summary_checksum, production_generation_summary_checksum
        )));
    }
    if manifest.production_generation_body_class_coverage_summary_path
        != "production-generation-body-class-coverage-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected production generation body-class coverage summary file entry: {}",
            manifest.production_generation_body_class_coverage_summary_path
        )));
    }
    if manifest.production_generation_body_class_coverage_summary_checksum
        != production_generation_body_class_coverage_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "production generation body-class coverage summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.production_generation_body_class_coverage_summary_checksum, production_generation_body_class_coverage_summary_checksum
        )));
    }
    ensure_production_generation_body_class_coverage_summary_matches_current_rendering(
        &production_generation_body_class_coverage_summary_text,
    )?;
    if manifest.production_generation_source_summary_path
        != "production-generation-source-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected production generation source summary file entry: {}",
            manifest.production_generation_source_summary_path
        )));
    }
    if manifest.production_generation_source_summary_checksum
        != production_generation_source_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "production generation source summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.production_generation_source_summary_checksum,
            production_generation_source_summary_checksum
        )));
    }
    if manifest.production_generation_source_revision_summary_path
        != "production-generation-source-revision-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected production generation source revision summary file entry: {}",
            manifest.production_generation_source_revision_summary_path
        )));
    }
    if manifest.production_generation_source_revision_summary_checksum
        != production_generation_source_revision_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "production generation source revision summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.production_generation_source_revision_summary_checksum,
            production_generation_source_revision_summary_checksum
        )));
    }
    if manifest.production_generation_source_window_summary_path
        != "production-generation-source-window-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected production generation source window summary file entry: {}",
            manifest.production_generation_source_window_summary_path
        )));
    }
    if manifest.production_generation_source_window_summary_checksum
        != production_generation_source_window_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "production generation source window summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.production_generation_source_window_summary_checksum,
            production_generation_source_window_summary_checksum
        )));
    }
    if manifest.production_generation_quarter_day_boundary_summary_path
        != "production-generation-quarter-day-boundary-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected production generation quarter-day boundary summary file entry: {}",
            manifest.production_generation_quarter_day_boundary_summary_path
        )));
    }
    if manifest.production_generation_quarter_day_boundary_summary_checksum
        != production_generation_quarter_day_boundary_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "production generation quarter-day boundary summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.production_generation_quarter_day_boundary_summary_checksum,
            production_generation_quarter_day_boundary_summary_checksum
        )));
    }
    if manifest.production_generation_corpus_shape_summary_path
        != "production-generation-corpus-shape-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected production generation corpus shape summary file entry: {}",
            manifest.production_generation_corpus_shape_summary_path
        )));
    }
    if manifest.production_generation_corpus_shape_summary_checksum
        != production_generation_corpus_shape_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "production generation corpus shape summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.production_generation_corpus_shape_summary_checksum,
            production_generation_corpus_shape_summary_checksum
        )));
    }
    if manifest.production_generation_manifest_summary_path
        != "production-generation-manifest-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected production generation manifest summary file entry: {}",
            manifest.production_generation_manifest_summary_path
        )));
    }
    if manifest.production_generation_manifest_summary_checksum
        != production_generation_manifest_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "production generation manifest summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.production_generation_manifest_summary_checksum,
            production_generation_manifest_summary_checksum
        )));
    }
    if manifest.production_generation_manifest_checksum_path
        != "production-generation-manifest-checksum-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected production generation manifest checksum file entry: {}",
            manifest.production_generation_manifest_checksum_path
        )));
    }
    if manifest.production_generation_manifest_checksum_checksum
        != production_generation_manifest_checksum_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "production generation manifest checksum summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.production_generation_manifest_checksum_checksum,
            production_generation_manifest_checksum_summary_checksum
        )));
    }
    if manifest.catalog_inventory_summary_checksum != catalog_inventory_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "catalog inventory summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.catalog_inventory_summary_checksum, catalog_inventory_summary_checksum
        )));
    }
    if manifest.catalog_posture_summary_checksum != catalog_posture_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "catalog posture summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.catalog_posture_summary_checksum, catalog_posture_summary_checksum
        )));
    }
    if catalog_posture_summary_text != render_catalog_posture_summary() {
        return Err(ReleaseBundleError::Verification(
            "catalog posture summary no longer matches the current catalog posture".to_string(),
        ));
    }
    ensure_catalog_inventory_alignment(&catalog_inventory_summary_text)?;
    if manifest.custom_definition_ayanamsa_labels_summary_checksum
        != custom_definition_ayanamsa_labels_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "custom-definition ayanamsa labels summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.custom_definition_ayanamsa_labels_summary_checksum,
            custom_definition_ayanamsa_labels_summary_checksum
        )));
    }
    if manifest.ayanamsa_provenance_summary_checksum != ayanamsa_provenance_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "ayanamsa provenance summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.ayanamsa_provenance_summary_checksum,
            ayanamsa_provenance_summary_checksum
        )));
    }
    if ayanamsa_provenance_summary_text != format_ayanamsa_provenance_for_report() {
        return Err(ReleaseBundleError::Verification(
            "ayanamsa provenance summary no longer matches the current ayanamsa provenance posture"
                .to_string(),
        ));
    }
    ensure_custom_definition_ayanamsa_labels_alignment(
        &custom_definition_ayanamsa_labels_summary_text,
    )?;
    if manifest.validation_report_summary_checksum != validation_report_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "validation report summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.validation_report_summary_checksum, validation_report_summary_checksum
        )));
    }
    if manifest.release_body_claims_summary_checksum != release_body_claims_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "release body claims summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.release_body_claims_summary_checksum, release_body_claims_summary_checksum
        )));
    }
    ensure_release_body_claims_summary_matches_current_rendering(
        &release_body_claims_summary_text,
    )?;
    if manifest.body_date_channel_claims_summary_checksum
        != body_date_channel_claims_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "body/date/channel claims summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.body_date_channel_claims_summary_checksum, body_date_channel_claims_summary_checksum
        )));
    }
    if manifest.pluto_fallback_summary_checksum != pluto_fallback_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "pluto fallback summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.pluto_fallback_summary_checksum, pluto_fallback_summary_checksum
        )));
    }
    ensure_pluto_fallback_summary_matches_current_rendering(&pluto_fallback_summary_text)?;
    if let Err(error) = validate_release_body_claims_posture(
        &release_body_claims_summary_text,
        &pluto_fallback_summary_text,
    ) {
        return Err(ReleaseBundleError::Verification(error));
    }
    ensure_body_date_channel_claims_summary_matches_current_rendering(
        &body_date_channel_claims_summary_text,
    )?;
    if manifest.request_policy_summary_checksum != request_policy_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "request policy summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.request_policy_summary_checksum, request_policy_summary_checksum
        )));
    }
    ensure_request_policy_summary_matches_current_rendering(&request_policy_summary_text)?;
    if manifest.observer_policy_summary_checksum != observer_policy_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "observer policy summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.observer_policy_summary_checksum, observer_policy_summary_checksum
        )));
    }
    ensure_observer_policy_summary_matches_current_rendering(&observer_policy_summary_text)?;
    if manifest.apparentness_policy_summary_checksum != apparentness_policy_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "apparentness policy summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.apparentness_policy_summary_checksum, apparentness_policy_summary_checksum
        )));
    }
    ensure_apparentness_policy_summary_matches_current_rendering(
        &apparentness_policy_summary_text,
    )?;
    if manifest.request_semantics_summary_checksum != request_semantics_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "request-semantics summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.request_semantics_summary_checksum, request_semantics_summary_checksum
        )));
    }
    ensure_request_semantics_summary_matches_current_rendering(&request_semantics_summary_text)?;
    ensure_unsupported_modes_summary_matches_current_rendering(&unsupported_modes_summary_text)?;
    if manifest.unsupported_modes_summary_checksum != unsupported_modes_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "unsupported-modes summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.unsupported_modes_summary_checksum, unsupported_modes_summary_checksum
        )));
    }
    if manifest.time_scale_policy_summary_checksum != time_scale_policy_summary_checksum {
        if manifest.utc_convenience_policy_summary_checksum
            != utc_convenience_policy_summary_checksum
        {
            return Err(ReleaseBundleError::Verification(format!(
            "UTC convenience policy summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.utc_convenience_policy_summary_checksum, utc_convenience_policy_summary_checksum
        )));
        }
        return Err(ReleaseBundleError::Verification(format!(
            "time-scale policy summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.time_scale_policy_summary_checksum, time_scale_policy_summary_checksum
        )));
    }
    if manifest.delta_t_policy_summary_checksum != delta_t_policy_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "delta-t policy summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.delta_t_policy_summary_checksum, delta_t_policy_summary_checksum
        )));
    }
    if manifest.native_sidereal_policy_summary_checksum != native_sidereal_policy_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "native sidereal policy summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.native_sidereal_policy_summary_checksum, native_sidereal_policy_summary_checksum
        )));
    }
    if manifest.zodiac_policy_summary_checksum != zodiac_policy_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "zodiac policy summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.zodiac_policy_summary_checksum, zodiac_policy_summary_checksum
        )));
    }
    ensure_native_sidereal_policy_summary_matches_current_rendering(
        &native_sidereal_policy_summary_text,
    )?;
    ensure_zodiac_policy_summary_matches_current_rendering(&zodiac_policy_summary_text)?;
    ensure_time_scale_policy_summary_matches_current_rendering(&time_scale_policy_summary_text)?;
    ensure_utc_convenience_policy_summary_matches_current_rendering(
        &utc_convenience_policy_summary_text,
    )?;
    ensure_delta_t_policy_summary_matches_current_rendering(&delta_t_policy_summary_text)?;
    if manifest.lunar_theory_limitations_summary_checksum
        != lunar_theory_limitations_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "lunar theory limitations summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.lunar_theory_limitations_summary_checksum,
            lunar_theory_limitations_summary_checksum
        )));
    }
    if manifest.lunar_theory_source_selection_summary_checksum
        != lunar_theory_source_selection_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "lunar theory source selection summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.lunar_theory_source_selection_summary_checksum,
            lunar_theory_source_selection_summary_checksum
        )));
    }
    if manifest.lunar_theory_source_family_summary_checksum
        != lunar_theory_source_family_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "lunar theory source family summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.lunar_theory_source_family_summary_checksum,
            lunar_theory_source_family_summary_checksum
        )));
    }
    if manifest.lunar_source_window_summary_checksum != lunar_source_window_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "lunar source window summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.lunar_source_window_summary_checksum,
            lunar_source_window_summary_checksum
        )));
    }
    if manifest.lunar_theory_catalog_validation_summary_checksum
        != lunar_theory_catalog_validation_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "lunar theory catalog validation summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.lunar_theory_catalog_validation_summary_checksum,
            lunar_theory_catalog_validation_summary_checksum
        )));
    }
    if manifest.request_surface_summary_checksum != request_surface_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "request surface summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.request_surface_summary_checksum, request_surface_summary_checksum
        )));
    }
    if manifest.compatibility_caveats_summary_checksum != compatibility_caveats_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "compatibility caveats summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.compatibility_caveats_summary_checksum, compatibility_caveats_summary_checksum
        )));
    }
    if manifest.workspace_provenance_summary_checksum != workspace_provenance_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "workspace provenance summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.workspace_provenance_summary_checksum, workspace_provenance_summary_checksum
        )));
    }
    if manifest.workspace_audit_summary_checksum != workspace_audit_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "workspace audit summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.workspace_audit_summary_checksum, workspace_audit_summary_checksum
        )));
    }
    if manifest.native_dependency_audit_summary_checksum != native_dependency_audit_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "native-dependency audit summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.native_dependency_audit_summary_checksum, native_dependency_audit_summary_checksum
        )));
    }
    if manifest.artifact_summary_checksum != artifact_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "artifact summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.artifact_summary_checksum, artifact_summary_checksum
        )));
    }
    if manifest.packaged_artifact_profile_coverage_summary_checksum
        != packaged_artifact_profile_coverage_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact profile coverage summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_profile_coverage_summary_checksum,
            packaged_artifact_profile_coverage_summary_checksum
        )));
    }
    if manifest.packaged_artifact_access_summary_checksum
        != packaged_artifact_access_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact access summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_access_summary_checksum, packaged_artifact_access_summary_checksum
        )));
    }
    if manifest.packaged_artifact_output_support_summary_path
        != "packaged-artifact-output-support-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact output support summary file entry: {}",
            manifest.packaged_artifact_output_support_summary_path
        )));
    }
    if manifest.packaged_artifact_output_support_summary_checksum
        != packaged_artifact_output_support_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact output support summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_output_support_summary_checksum, packaged_artifact_output_support_summary_checksum
        )));
    }
    if manifest.packaged_artifact_fit_sample_classes_summary_path
        != "packaged-artifact-fit-sample-classes-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact fit sample classes summary file entry: {}",
            manifest.packaged_artifact_fit_sample_classes_summary_path
        )));
    }
    if manifest.packaged_artifact_fit_sample_classes_summary_checksum
        != packaged_artifact_fit_sample_classes_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact fit sample classes summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_fit_sample_classes_summary_checksum, packaged_artifact_fit_sample_classes_summary_checksum
        )));
    }
    if manifest.packaged_artifact_fit_threshold_violation_count_summary_path
        != "packaged-artifact-fit-threshold-violation-count-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact fit threshold violation count summary file entry: {}",
            manifest.packaged_artifact_fit_threshold_violation_count_summary_path
        )));
    }
    if manifest.packaged_artifact_fit_threshold_violation_count_summary_checksum
        != packaged_artifact_fit_threshold_violation_count_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact fit threshold violation count summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_fit_threshold_violation_count_summary_checksum, packaged_artifact_fit_threshold_violation_count_summary_checksum
        )));
    }
    if manifest.packaged_artifact_fit_threshold_violations_summary_path
        != "packaged-artifact-fit-threshold-violations-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact fit threshold violations summary file entry: {}",
            manifest.packaged_artifact_fit_threshold_violations_summary_path
        )));
    }
    if manifest.packaged_artifact_fit_threshold_violations_summary_checksum
        != packaged_artifact_fit_threshold_violations_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact fit threshold violations summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_fit_threshold_violations_summary_checksum, packaged_artifact_fit_threshold_violations_summary_checksum
        )));
    }
    if manifest.packaged_artifact_body_cadence_summary_path
        != "packaged-artifact-body-cadence-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact body cadence summary file entry: {}",
            manifest.packaged_artifact_body_cadence_summary_path
        )));
    }
    if manifest.packaged_artifact_body_cadence_summary_checksum
        != packaged_artifact_body_cadence_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact body cadence summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_body_cadence_summary_checksum, packaged_artifact_body_cadence_summary_checksum
        )));
    }
    if manifest.packaged_artifact_body_class_span_cap_summary_path
        != "packaged-artifact-body-class-span-cap-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact body-class span cap summary file entry: {}",
            manifest.packaged_artifact_body_class_span_cap_summary_path
        )));
    }
    if manifest.packaged_artifact_body_class_span_cap_summary_checksum
        != packaged_artifact_body_class_span_cap_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact body-class span cap summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_body_class_span_cap_summary_checksum, packaged_artifact_body_class_span_cap_summary_checksum
        )));
    }
    if manifest.packaged_artifact_normalized_intermediate_summary_path
        != "packaged-artifact-normalized-intermediate-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact normalized intermediate summary file entry: {}",
            manifest.packaged_artifact_normalized_intermediate_summary_path
        )));
    }
    if manifest.packaged_artifact_normalized_intermediate_summary_checksum
        != packaged_artifact_normalized_intermediate_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact normalized intermediate summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_normalized_intermediate_summary_checksum, packaged_artifact_normalized_intermediate_summary_checksum
        )));
    }
    ensure_packaged_artifact_normalized_intermediate_summary_matches_current_rendering(
        &packaged_artifact_normalized_intermediate_summary_text,
    )?;
    if manifest.packaged_artifact_speed_policy_summary_checksum
        != packaged_artifact_speed_policy_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact speed policy summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_speed_policy_summary_checksum, packaged_artifact_speed_policy_summary_checksum
        )));
    }
    if manifest.packaged_artifact_storage_summary_checksum
        != packaged_artifact_storage_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact storage summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_storage_summary_checksum, packaged_artifact_storage_summary_checksum
        )));
    }
    if manifest.packaged_artifact_production_profile_summary_checksum
        != packaged_artifact_production_profile_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact production-profile summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_production_profile_summary_checksum,
            packaged_artifact_production_profile_summary_checksum
        )));
    }
    if manifest.packaged_frame_treatment_summary_checksum
        != packaged_frame_treatment_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged frame treatment summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_frame_treatment_summary_checksum,
            packaged_frame_treatment_summary_checksum
        )));
    }
    if manifest.packaged_artifact_target_threshold_summary_checksum
        != packaged_artifact_target_threshold_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact target-threshold summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_target_threshold_summary_checksum,
            packaged_artifact_target_threshold_summary_checksum
        )));
    }
    if manifest.packaged_artifact_target_threshold_state_summary_checksum
        != packaged_artifact_target_threshold_state_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact target-threshold state summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_target_threshold_state_summary_checksum,
            packaged_artifact_target_threshold_state_summary_checksum
        )));
    }
    if manifest.packaged_artifact_target_threshold_state_summary_path
        != "packaged-artifact-target-threshold-state-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact target-threshold state summary file entry: {}",
            manifest.packaged_artifact_target_threshold_state_summary_path
        )));
    }
    if manifest.packaged_artifact_source_fit_holdout_sync_summary_path
        != "packaged-artifact-source-fit-holdout-sync-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact source-fit and hold-out sync summary file entry: {}",
            manifest.packaged_artifact_source_fit_holdout_sync_summary_path
        )));
    }
    if manifest.packaged_artifact_source_fit_holdout_sync_summary_checksum
        != packaged_artifact_source_fit_holdout_sync_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact source-fit and hold-out sync summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_source_fit_holdout_sync_summary_checksum,
            packaged_artifact_source_fit_holdout_sync_summary_checksum
        )));
    }
    if manifest.packaged_artifact_target_threshold_scope_envelopes_summary_path
        != "packaged-artifact-target-threshold-scope-envelopes-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact target-threshold scope envelopes summary file entry: {}",
            manifest.packaged_artifact_target_threshold_scope_envelopes_summary_path
        )));
    }
    if manifest.packaged_artifact_phase2_corpus_alignment_summary_path
        != "packaged-artifact-phase2-corpus-alignment-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact phase-2 corpus alignment summary file entry: {}",
            manifest.packaged_artifact_phase2_corpus_alignment_summary_path
        )));
    }
    if manifest.packaged_artifact_target_threshold_scope_envelopes_summary_checksum
        != packaged_artifact_target_threshold_scope_envelopes_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact target-threshold scope envelopes summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_target_threshold_scope_envelopes_summary_checksum,
            packaged_artifact_target_threshold_scope_envelopes_summary_checksum
        )));
    }
    if manifest.packaged_artifact_phase2_corpus_alignment_summary_checksum
        != packaged_artifact_phase2_corpus_alignment_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact phase-2 corpus alignment summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_phase2_corpus_alignment_summary_checksum,
            packaged_artifact_phase2_corpus_alignment_summary_checksum
        )));
    }
    if manifest.packaged_lookup_epoch_policy_summary_checksum
        != packaged_lookup_epoch_policy_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged lookup-epoch policy summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_lookup_epoch_policy_summary_checksum,
            packaged_lookup_epoch_policy_summary_checksum
        )));
    }
    if manifest.packaged_artifact_generation_policy_summary_checksum
        != packaged_artifact_generation_policy_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact generation policy summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_generation_policy_summary_checksum,
            packaged_artifact_generation_policy_summary_checksum
        )));
    }
    if manifest.packaged_artifact_generation_residual_bodies_summary_checksum
        != packaged_artifact_generation_residual_bodies_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact generation residual bodies summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_generation_residual_bodies_summary_checksum,
            packaged_artifact_generation_residual_bodies_summary_checksum
        )));
    }
    if manifest.packaged_artifact_regeneration_summary_path
        != "packaged-artifact-regeneration-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected packaged-artifact regeneration summary file entry: {}",
            manifest.packaged_artifact_regeneration_summary_path
        )));
    }
    if manifest.packaged_artifact_regeneration_summary_checksum
        != packaged_artifact_regeneration_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact regeneration summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_regeneration_summary_checksum,
            packaged_artifact_regeneration_summary_checksum
        )));
    }
    if manifest.packaged_artifact_generation_manifest_checksum
        != packaged_artifact_generation_manifest_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact generation manifest checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_generation_manifest_checksum,
            packaged_artifact_generation_manifest_checksum
        )));
    }
    if manifest.packaged_artifact_generation_manifest_summary_checksum
        != packaged_artifact_generation_manifest_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact generation manifest summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_generation_manifest_summary_checksum,
            packaged_artifact_generation_manifest_summary_checksum
        )));
    }
    if manifest.packaged_artifact_generation_manifest_checksum_summary_checksum
        != packaged_artifact_generation_manifest_checksum_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact generation manifest checksum summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_generation_manifest_checksum_summary_checksum,
            packaged_artifact_generation_manifest_checksum_summary_checksum
        )));
    }
    if manifest.packaged_artifact_generation_manifest_checksum_checksum
        != packaged_artifact_generation_manifest_checksum_text_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact generation manifest checksum sidecar checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.packaged_artifact_generation_manifest_checksum_checksum,
            packaged_artifact_generation_manifest_checksum_text_checksum
        )));
    }
    if packaged_artifact_generation_manifest_checksum_value
        != packaged_artifact_generation_manifest_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "packaged-artifact generation manifest checksum sidecar value mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            packaged_artifact_generation_manifest_checksum,
            packaged_artifact_generation_manifest_checksum_value
        )));
    }
    if manifest.benchmark_corpus_summary_path != "benchmark-corpus-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected benchmark corpus summary file entry: {}",
            manifest.benchmark_corpus_summary_path
        )));
    }
    if manifest.benchmark_corpus_summary_checksum != benchmark_corpus_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "benchmark corpus summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.benchmark_corpus_summary_checksum, benchmark_corpus_summary_checksum
        )));
    }
    if manifest.chart_benchmark_corpus_summary_path != "chart-benchmark-corpus-summary.txt" {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected chart benchmark corpus summary file entry: {}",
            manifest.chart_benchmark_corpus_summary_path
        )));
    }
    if manifest.chart_benchmark_corpus_summary_checksum != chart_benchmark_corpus_summary_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "chart benchmark corpus summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.chart_benchmark_corpus_summary_checksum, chart_benchmark_corpus_summary_checksum
        )));
    }
    if manifest.selected_asteroid_source_request_corpus_summary_path
        != "selected-asteroid-source-request-corpus-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected selected asteroid source request corpus summary file entry: {}",
            manifest.selected_asteroid_source_request_corpus_summary_path
        )));
    }
    if manifest.selected_asteroid_source_request_corpus_summary_checksum
        != selected_asteroid_source_request_corpus_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "selected asteroid source request corpus summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.selected_asteroid_source_request_corpus_summary_checksum,
            selected_asteroid_source_request_corpus_summary_checksum
        )));
    }
    if manifest.selected_asteroid_source_request_corpus_equatorial_summary_path
        != "selected-asteroid-source-request-corpus-equatorial-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected selected asteroid source request corpus equatorial summary file entry: {}",
            manifest.selected_asteroid_source_request_corpus_equatorial_summary_path
        )));
    }
    if manifest.selected_asteroid_source_request_corpus_equatorial_summary_checksum
        != selected_asteroid_source_request_corpus_equatorial_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "selected asteroid source request corpus equatorial summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.selected_asteroid_source_request_corpus_equatorial_summary_checksum,
            selected_asteroid_source_request_corpus_equatorial_summary_checksum
        )));
    }
    if manifest.selected_asteroid_source_window_summary_path
        != "selected-asteroid-source-window-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected selected asteroid source window summary file entry: {}",
            manifest.selected_asteroid_source_window_summary_path
        )));
    }
    if manifest.selected_asteroid_source_window_summary_checksum
        != selected_asteroid_source_window_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "selected asteroid source window summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.selected_asteroid_source_window_summary_checksum,
            selected_asteroid_source_window_summary_checksum
        )));
    }
    if manifest.interpolation_quality_request_corpus_summary_path
        != "interpolation-quality-request-corpus-summary.txt"
    {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected interpolation-quality sample request corpus summary file entry: {}",
            manifest.interpolation_quality_request_corpus_summary_path
        )));
    }
    if manifest.interpolation_quality_request_corpus_summary_checksum
        != interpolation_quality_request_corpus_summary_checksum
    {
        return Err(ReleaseBundleError::Verification(format!(
            "interpolation-quality sample request corpus summary checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.interpolation_quality_request_corpus_summary_checksum,
            interpolation_quality_request_corpus_summary_checksum
        )));
    }
    if manifest.benchmark_report_checksum != benchmark_report_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "benchmark report checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.benchmark_report_checksum, benchmark_report_checksum
        )));
    }
    if manifest.validation_report_checksum != validation_report_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "validation report checksum mismatch: manifest has 0x{:016x}, file has 0x{:016x}",
            manifest.validation_report_checksum, validation_report_checksum
        )));
    }
    if manifest_checksum_value != manifest_checksum {
        return Err(ReleaseBundleError::Verification(format!(
            "bundle manifest checksum mismatch: manifest has 0x{:016x}, checksum file has 0x{:016x}",
            manifest_checksum, manifest_checksum_value
        )));
    }

    let bundle = ReleaseBundle {
        source_revision: manifest.source_revision,
        workspace_status: manifest.workspace_status,
        rustc_version: manifest.rustc_version,
        cargo_version: manifest.cargo_version,
        output_dir: output_dir.to_path_buf(),
        compatibility_profile_path: profile_path,
        compatibility_profile_summary_path: profile_summary_path,
        release_notes_path,
        release_notes_summary_path,
        release_summary_path,
        release_profile_identifiers_path,
        release_profile_identifiers_summary_path,
        release_house_system_canonical_names_summary_path,
        release_ayanamsa_canonical_names_summary_path,
        release_house_validation_summary_path,
        house_code_aliases_summary_path,
        house_formula_families_summary_path,
        house_latitude_sensitive_summary_path,
        release_checklist_path,
        release_checklist_summary_path,
        backend_matrix_path,
        backend_matrix_summary_path,
        api_stability_path,
        api_stability_summary_path,
        comparison_envelope_summary_path,
        comparison_body_class_tolerance_summary_path,
        comparison_body_class_error_envelope_summary_path,
        comparison_corpus_release_guard_summary_path,
        catalog_inventory_summary_path,
        validation_report_summary_path,
        workspace_provenance_summary_path,
        workspace_audit_summary_path,
        native_dependency_audit_summary_path,
        artifact_summary_path,
        packaged_artifact_path,
        packaged_artifact_checksum_path,
        packaged_artifact_profile_coverage_summary_path,
        interpolation_quality_request_corpus_summary_path,
        packaged_artifact_storage_summary_path,
        packaged_artifact_generation_manifest_path,
        packaged_artifact_generation_manifest_summary_path,
        packaged_artifact_generation_manifest_checksum_summary_path,
        packaged_artifact_generation_manifest_checksum_path,
        benchmark_report_path,
        validation_report_path,
        manifest_path,
        manifest_checksum_path,
        compatibility_profile_bytes: profile_text.len(),
        compatibility_profile_summary_bytes: profile_summary_text.len(),
        release_notes_bytes: release_notes_text.len(),
        release_notes_summary_bytes: release_notes_summary_text.len(),
        release_summary_bytes: release_summary_text.len(),
        release_profile_identifiers_bytes: release_profile_identifiers_text.len(),
        release_profile_identifiers_summary_bytes: release_profile_identifiers_summary_text.len(),
        release_house_system_canonical_names_summary_bytes:
            release_house_system_canonical_names_summary_text.len(),
        release_ayanamsa_canonical_names_summary_bytes:
            release_ayanamsa_canonical_names_summary_text.len(),
        release_house_validation_summary_bytes: release_house_validation_summary_text.len(),
        house_code_aliases_summary_bytes: house_code_aliases_summary_text.len(),
        house_formula_families_summary_bytes: house_formula_families_summary_text.len(),
        house_latitude_sensitive_summary_bytes: house_latitude_sensitive_summary_text.len(),
        release_checklist_bytes: release_checklist_text.len(),
        release_checklist_summary_bytes: release_checklist_summary_text.len(),
        backend_matrix_bytes: backend_matrix_text.len(),
        backend_matrix_summary_bytes: backend_matrix_summary_text.len(),
        api_stability_bytes: api_stability_text.len(),
        api_stability_summary_bytes: api_stability_summary_text.len(),
        comparison_envelope_summary_bytes: comparison_envelope_summary_text.len(),
        comparison_body_class_tolerance_summary_bytes: comparison_body_class_tolerance_summary_text
            .len(),
        comparison_body_class_error_envelope_summary_bytes:
            comparison_body_class_error_envelope_summary_text.len(),
        comparison_corpus_release_guard_summary_bytes: comparison_corpus_release_guard_summary_text
            .len(),
        reference_holdout_overlap_summary_bytes: reference_holdout_overlap_summary_text.len(),
        catalog_inventory_summary_bytes: catalog_inventory_summary_text.len(),
        validation_report_summary_bytes: validation_report_summary_text.len(),
        workspace_provenance_summary_bytes: workspace_provenance_summary_text.len(),
        workspace_audit_summary_bytes: workspace_audit_summary_text.len(),
        native_dependency_audit_summary_bytes: native_dependency_audit_summary_text.len(),
        artifact_summary_bytes: artifact_summary_text.len(),
        packaged_artifact_profile_coverage_summary_bytes:
            packaged_artifact_profile_coverage_summary_text.len(),
        packaged_artifact_generation_manifest_bytes: packaged_artifact_generation_manifest_text
            .len(),
        packaged_artifact_generation_manifest_summary_bytes:
            packaged_artifact_generation_manifest_summary_text.len(),
        packaged_artifact_generation_manifest_checksum_summary_bytes:
            packaged_artifact_generation_manifest_checksum_summary_text.len(),
        packaged_artifact_generation_manifest_checksum_bytes:
            packaged_artifact_generation_manifest_checksum_text.len(),
        benchmark_report_bytes: benchmark_report_text.len(),
        validation_report_bytes: validation_report_text.len(),
        manifest_checksum_bytes: manifest_checksum_text.len(),
        compatibility_profile_checksum,
        compatibility_profile_summary_checksum,
        release_notes_checksum,
        release_notes_summary_checksum,
        release_summary_checksum,
        release_profile_identifiers_checksum,
        release_profile_identifiers_summary_checksum,
        release_house_system_canonical_names_summary_checksum,
        release_ayanamsa_canonical_names_summary_checksum,
        release_house_validation_summary_checksum,
        house_code_aliases_summary_checksum,
        house_formula_families_summary_checksum,
        house_latitude_sensitive_summary_checksum,
        release_checklist_checksum,
        release_checklist_summary_checksum,
        backend_matrix_checksum,
        backend_matrix_summary_checksum,
        api_stability_checksum,
        api_stability_summary_checksum,
        comparison_envelope_summary_checksum,
        comparison_body_class_tolerance_summary_checksum,
        comparison_body_class_error_envelope_summary_checksum,
        comparison_corpus_release_guard_summary_checksum,
        reference_holdout_overlap_summary_checksum,
        catalog_inventory_summary_checksum,
        validation_report_summary_checksum,
        workspace_provenance_summary_checksum,
        workspace_audit_summary_checksum,
        native_dependency_audit_summary_checksum,
        artifact_summary_checksum,
        packaged_artifact_profile_coverage_summary_checksum,
        packaged_artifact_generation_manifest_checksum,
        packaged_artifact_generation_manifest_summary_checksum,
        packaged_artifact_generation_manifest_checksum_summary_checksum,
        packaged_artifact_generation_manifest_checksum_checksum:
            packaged_artifact_generation_manifest_checksum_text_checksum,
        benchmark_report_checksum,
        validation_report_checksum,
        manifest_checksum: manifest_checksum_value,
        validation_rounds: manifest.validation_rounds,
    };
    bundle.validate()?;
    Ok(bundle)
}

pub(crate) fn parse_manifest_string(
    text: &str,
    prefix: &str,
) -> Result<String, ReleaseBundleError> {
    extract_prefixed_value(text, prefix).map(|value| value.to_string())
}

pub(crate) fn ensure_canonical_manifest_value(
    value: &str,
    field_name: &str,
) -> Result<(), ReleaseBundleError> {
    if value.is_empty() {
        return Err(ReleaseBundleError::Verification(format!(
            "missing {field_name} entry"
        )));
    }

    if value != value.trim() {
        return Err(ReleaseBundleError::Verification(format!(
            "invalid {field_name} entry: unexpected leading or trailing whitespace"
        )));
    }

    if value.contains('\n') || value.contains('\r') {
        return Err(ReleaseBundleError::Verification(format!(
            "invalid {field_name} entry: unexpected line break"
        )));
    }

    if value.eq_ignore_ascii_case("unknown") {
        return Err(ReleaseBundleError::Verification(format!(
            "invalid {field_name} entry: placeholder values are not allowed"
        )));
    }

    Ok(())
}

pub(crate) fn parse_manifest_usize(
    text: &str,
    prefix: &str,
    field_name: &str,
) -> Result<usize, ReleaseBundleError> {
    let value = extract_prefixed_value(text, prefix)?;
    value.parse::<usize>().map_err(|error| {
        ReleaseBundleError::Verification(format!("invalid {field_name} entry: {error}"))
    })
}

pub(crate) fn parse_manifest_checksum(text: &str, prefix: &str) -> Result<u64, ReleaseBundleError> {
    let value = extract_prefixed_value(text, prefix)?;
    let value = value.strip_prefix("0x").ok_or_else(|| {
        ReleaseBundleError::Verification(format!(
            "missing 0x prefix for {prefix} (found {value:?})"
        ))
    })?;
    if value.len() != 16 || !value.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='f')) {
        return Err(ReleaseBundleError::Verification(format!(
            "invalid {prefix} value: expected exactly 16 lowercase hex digits (found {value:?})"
        )));
    }
    u64::from_str_radix(value, 16).map_err(|error| {
        ReleaseBundleError::Verification(format!("invalid {prefix} value: {error}"))
    })
}

pub(crate) fn parse_checksum_value(text: &str, label: &str) -> Result<u64, ReleaseBundleError> {
    let mut lines = text.lines();
    let Some(line) = lines.next() else {
        return Err(ReleaseBundleError::Verification(format!("missing {label}")));
    };

    if lines.next().is_some() {
        return Err(ReleaseBundleError::Verification(format!(
            "invalid {label} value: expected exactly one checksum line"
        )));
    }

    if line != line.trim() {
        return Err(ReleaseBundleError::Verification(format!(
            "invalid {label} value: unexpected leading or trailing whitespace (found {line:?})"
        )));
    }

    let value = line.strip_prefix("0x").ok_or_else(|| {
        ReleaseBundleError::Verification(format!("missing 0x prefix for {label} (found {line:?})"))
    })?;
    if value.len() != 16 || !value.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='f')) {
        return Err(ReleaseBundleError::Verification(format!(
            "invalid {label} value: expected exactly 16 lowercase hex digits (found {line:?})"
        )));
    }
    u64::from_str_radix(value, 16).map_err(|error| {
        ReleaseBundleError::Verification(format!("invalid {label} value: {error}"))
    })
}

pub(crate) fn extract_prefixed_value<'a>(
    text: &'a str,
    prefix: &str,
) -> Result<&'a str, ReleaseBundleError> {
    let mut matches = text.lines().filter_map(|line| line.strip_prefix(prefix));

    let Some(value) = matches.next() else {
        return Err(ReleaseBundleError::Verification(format!(
            "missing manifest entry: {prefix}"
        )));
    };

    if matches.next().is_some() {
        return Err(ReleaseBundleError::Verification(format!(
            "duplicate entry: {prefix}"
        )));
    }

    if value.is_empty() {
        return Ok(value);
    }

    let value = if prefix.ends_with(' ') {
        value
    } else {
        let Some(value) = value.strip_prefix(' ') else {
            return Err(ReleaseBundleError::Verification(format!(
                "unexpected whitespace in manifest entry: {prefix}"
            )));
        };
        value
    };

    if value != value.trim() {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected leading or trailing whitespace in manifest entry: {prefix}"
        )));
    }

    Ok(value)
}

pub(crate) fn ensure_release_profile_line_alignment(
    label: &str,
    text: &str,
    expected_profile_id: &str,
) -> Result<(), ReleaseBundleError> {
    let profile_id = match extract_prefixed_value(text, "Profile: ") {
        Ok(profile_id) => profile_id,
        Err(ReleaseBundleError::Verification(message)) => {
            return Err(ReleaseBundleError::Verification(format!(
                "{label}: {message}"
            )));
        }
        Err(error) => return Err(error),
    };

    if profile_id != expected_profile_id {
        return Err(ReleaseBundleError::Verification(format!(
            "{label} profile id mismatch: expected {expected_profile_id}, found {profile_id}"
        )));
    }

    Ok(())
}

pub(crate) fn ensure_release_profile_summary_alignment(
    label: &str,
    text: &str,
    expected_profile_id: &str,
    expected_api_stability_posture_id: &str,
) -> Result<(), ReleaseBundleError> {
    ensure_release_profile_line_alignment(label, text, expected_profile_id)?;

    let api_stability_posture_id = match extract_prefixed_value(text, "API stability posture: ") {
        Ok(api_stability_posture_id) => api_stability_posture_id,
        Err(ReleaseBundleError::Verification(message)) => {
            return Err(ReleaseBundleError::Verification(format!(
                "{label}: {message}"
            )));
        }
        Err(error) => return Err(error),
    };

    if api_stability_posture_id != expected_api_stability_posture_id {
        return Err(ReleaseBundleError::Verification(format!(
            "{label} API stability posture id mismatch: expected {expected_api_stability_posture_id}, found {api_stability_posture_id}"
        )));
    }

    Ok(())
}

pub(crate) fn ensure_release_profile_identifiers_alignment(
    text: &str,
    expected_profile_id: &str,
    expected_api_stability_posture_id: &str,
) -> Result<(), ReleaseBundleError> {
    let expected = format!(
        "Release profile identifiers: v1 compatibility={expected_profile_id}, api-stability={expected_api_stability_posture_id}"
    );
    let found = text.trim_end();
    if found != expected {
        return Err(ReleaseBundleError::Verification(format!(
            "release-profile identifiers mismatch: expected '{expected}', found '{found}'"
        )));
    }

    Ok(())
}

pub(crate) fn ensure_release_profile_identifiers_summary_matches_current_rendering(
    release_profile_identifiers_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if release_profile_identifiers_summary_text.trim_end()
        == render_release_profile_identifiers_summary().trim_end()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "release-profile identifiers summary no longer matches the current release-profile identifiers posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_release_house_system_canonical_names_summary_matches_current_rendering(
    release_house_system_canonical_names_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if release_house_system_canonical_names_summary_text.trim_end()
        == render_release_house_system_canonical_names_summary().trim_end()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "release-house-system canonical names summary no longer matches the current release-house-system canonical names posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_release_ayanamsa_canonical_names_summary_matches_current_rendering(
    release_ayanamsa_canonical_names_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if release_ayanamsa_canonical_names_summary_text.trim_end()
        == render_release_ayanamsa_canonical_names_summary().trim_end()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "release-ayanamsa canonical names summary no longer matches the current release-ayanamsa canonical names posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_release_notes_matches_current_rendering(
    release_notes_text: &str,
) -> Result<(), ReleaseBundleError> {
    if release_notes_text.trim_end() == render_release_notes_text().trim_end() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "release notes no longer matches the current release notes posture".to_string(),
        ))
    }
}

pub(crate) fn ensure_release_notes_summary_matches_current_rendering(
    release_notes_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if release_notes_summary_text.trim_end() == render_release_notes_summary_text().trim_end() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "release notes summary no longer matches the current release notes summary posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_release_checklist_summary_matches_current_rendering(
    release_checklist_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if release_checklist_summary_text.trim_end()
        == render_release_checklist_summary_text().trim_end()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "release checklist summary no longer matches the current release checklist summary posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_compatibility_profile_summary_matches_current_rendering(
    compatibility_profile_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if compatibility_profile_summary_text.trim_end()
        == render_compatibility_profile_summary_text().trim_end()
    {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "compatibility profile summary no longer matches the current compatibility profile summary posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_release_summary_matches_current_rendering(
    release_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if release_summary_text.trim_end() == render_release_summary_text().trim_end() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "release summary no longer matches the current release summary posture".to_string(),
        ))
    }
}

pub(crate) fn ensure_catalog_inventory_alignment(text: &str) -> Result<(), ReleaseBundleError> {
    let expected = current_compatibility_profile()
        .validated_catalog_inventory_summary_line()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let found = text.trim_end();
    if found != expected {
        return Err(ReleaseBundleError::Verification(format!(
            "catalog inventory summary mismatch: expected '{expected}', found '{found}'"
        )));
    }

    Ok(())
}

pub(crate) fn ensure_catalog_inventory_summary_matches_current_rendering(
    catalog_inventory_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if catalog_inventory_summary_text.trim_end() == render_catalog_inventory_summary().trim_end() {
        Ok(())
    } else {
        Err(ReleaseBundleError::Verification(
            "catalog inventory summary no longer matches the current catalog inventory posture"
                .to_string(),
        ))
    }
}

pub(crate) fn ensure_custom_definition_ayanamsa_labels_alignment(
    text: &str,
) -> Result<(), ReleaseBundleError> {
    let expected = current_compatibility_profile()
        .validated_custom_definition_ayanamsa_labels_summary_line()
        .map_err(|error| ReleaseBundleError::Verification(error.to_string()))?;
    let found = text.trim_end();
    if found != expected {
        return Err(ReleaseBundleError::Verification(format!(
            "custom-definition ayanamsa labels summary mismatch: expected '{expected}', found '{found}'"
        )));
    }

    Ok(())
}
