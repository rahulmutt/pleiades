//! Release-bundle rendering-alignment helpers and manifest value parsing.

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
