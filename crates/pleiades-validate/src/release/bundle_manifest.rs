//! Release-bundle manifest parsing and bundle-structure checks.

use std::path::Path;

use super::bundle::*;
use super::bundle_verify::*;
use crate::*;

#[derive(Debug)]
pub(crate) struct ParsedReleaseBundleManifest {
    pub(crate) profile_path: String,
    pub(crate) profile_checksum: u64,
    pub(crate) profile_summary_path: String,
    pub(crate) profile_summary_checksum: u64,
    pub(crate) release_notes_path: String,
    pub(crate) release_notes_checksum: u64,
    pub(crate) release_notes_summary_path: String,
    pub(crate) release_notes_summary_checksum: u64,
    pub(crate) release_summary_path: String,
    pub(crate) release_summary_checksum: u64,
    pub(crate) release_profile_identifiers_path: String,
    pub(crate) release_profile_identifiers_checksum: u64,
    pub(crate) release_profile_identifiers_summary_path: String,
    pub(crate) release_profile_identifiers_summary_checksum: u64,
    pub(crate) release_house_system_canonical_names_summary_path: String,
    pub(crate) release_house_system_canonical_names_summary_checksum: u64,
    pub(crate) release_ayanamsa_canonical_names_summary_path: String,
    pub(crate) release_ayanamsa_canonical_names_summary_checksum: u64,
    pub(crate) release_house_validation_summary_path: String,
    pub(crate) release_house_validation_summary_checksum: u64,
    pub(crate) target_house_scope_summary_path: String,
    pub(crate) target_house_scope_summary_checksum: u64,
    pub(crate) target_ayanamsa_scope_summary_path: String,
    pub(crate) target_ayanamsa_scope_summary_checksum: u64,
    pub(crate) house_code_aliases_summary_path: String,
    pub(crate) house_code_aliases_summary_checksum: u64,
    pub(crate) house_formula_families_summary_path: String,
    pub(crate) house_formula_families_summary_checksum: u64,
    pub(crate) house_latitude_sensitive_summary_path: String,
    pub(crate) house_latitude_sensitive_summary_checksum: u64,
    pub(crate) house_latitude_sensitive_constraints_summary_path: String,
    pub(crate) house_latitude_sensitive_constraints_summary_checksum: u64,
    pub(crate) house_latitude_sensitive_failure_modes_summary_path: String,
    pub(crate) house_latitude_sensitive_failure_modes_summary_checksum: u64,
    pub(crate) release_checklist_path: String,
    pub(crate) release_checklist_checksum: u64,
    pub(crate) release_checklist_summary_path: String,
    pub(crate) release_checklist_summary_checksum: u64,
    pub(crate) backend_matrix_path: String,
    pub(crate) backend_matrix_checksum: u64,
    pub(crate) backend_matrix_summary_path: String,
    pub(crate) backend_matrix_summary_checksum: u64,
    pub(crate) api_stability_path: String,
    pub(crate) api_stability_checksum: u64,
    pub(crate) api_stability_summary_path: String,
    pub(crate) api_stability_summary_checksum: u64,
    pub(crate) comparison_corpus_summary_path: String,
    pub(crate) comparison_corpus_summary_checksum: u64,
    pub(crate) source_corpus_summary_path: String,
    pub(crate) source_corpus_summary_checksum: u64,
    pub(crate) jpl_source_posture_summary_path: String,
    pub(crate) jpl_source_posture_summary_checksum: u64,
    pub(crate) jpl_provenance_only_summary_path: String,
    pub(crate) jpl_provenance_only_summary_checksum: u64,
    pub(crate) comparison_snapshot_summary_path: String,
    pub(crate) comparison_snapshot_summary_checksum: u64,
    pub(crate) comparison_snapshot_source_summary_path: String,
    pub(crate) comparison_snapshot_source_summary_checksum: u64,
    pub(crate) comparison_snapshot_source_window_summary_path: String,
    pub(crate) comparison_snapshot_source_window_summary_checksum: u64,
    pub(crate) comparison_snapshot_body_class_coverage_summary_path: String,
    pub(crate) comparison_snapshot_body_class_coverage_summary_checksum: u64,
    pub(crate) comparison_snapshot_manifest_summary_path: String,
    pub(crate) comparison_snapshot_manifest_summary_checksum: u64,
    pub(crate) comparison_envelope_summary_path: String,
    pub(crate) comparison_envelope_summary_checksum: u64,
    pub(crate) comparison_body_class_tolerance_summary_path: String,
    pub(crate) comparison_body_class_tolerance_summary_checksum: u64,
    pub(crate) comparison_body_class_error_envelope_summary_path: String,
    pub(crate) comparison_body_class_error_envelope_summary_checksum: u64,
    pub(crate) comparison_corpus_release_guard_summary_path: String,
    pub(crate) comparison_corpus_release_guard_summary_checksum: u64,
    pub(crate) reference_holdout_overlap_summary_path: String,
    pub(crate) reference_holdout_overlap_summary_checksum: u64,
    pub(crate) reference_snapshot_bridge_day_summary_path: String,
    pub(crate) reference_snapshot_bridge_day_summary_checksum: u64,
    pub(crate) reference_snapshot_major_body_boundary_window_summary_path: String,
    pub(crate) reference_snapshot_major_body_boundary_window_summary_checksum: u64,
    pub(crate) reference_snapshot_boundary_epoch_coverage_summary_path: String,
    pub(crate) reference_snapshot_boundary_epoch_coverage_summary_checksum: u64,
    pub(crate) reference_snapshot_pre_bridge_boundary_summary_path: String,
    pub(crate) reference_snapshot_pre_bridge_boundary_summary_checksum: u64,
    pub(crate) reference_snapshot_2451917_major_body_boundary_summary_path: String,
    pub(crate) reference_snapshot_2451917_major_body_boundary_summary_checksum: u64,
    pub(crate) reference_snapshot_2451918_major_body_boundary_summary_path: String,
    pub(crate) reference_snapshot_2451918_major_body_boundary_summary_checksum: u64,
    pub(crate) reference_snapshot_2451919_major_body_boundary_summary_path: String,
    pub(crate) reference_snapshot_2451919_major_body_boundary_summary_checksum: u64,
    pub(crate) reference_snapshot_2451916_major_body_dense_boundary_summary_path: String,
    pub(crate) reference_snapshot_2451916_major_body_dense_boundary_summary_checksum: u64,
    pub(crate) reference_snapshot_sparse_boundary_summary_path: String,
    pub(crate) reference_snapshot_sparse_boundary_summary_checksum: u64,
    pub(crate) reference_snapshot_exact_j2000_evidence_summary_path: String,
    pub(crate) reference_snapshot_exact_j2000_evidence_summary_checksum: u64,
    pub(crate) reference_snapshot_source_summary_path: String,
    pub(crate) reference_snapshot_source_summary_checksum: u64,
    pub(crate) reference_snapshot_source_window_summary_path: String,
    pub(crate) reference_snapshot_source_window_summary_checksum: u64,
    pub(crate) reference_snapshot_manifest_summary_path: String,
    pub(crate) reference_snapshot_manifest_summary_checksum: u64,
    pub(crate) reference_snapshot_body_class_coverage_summary_path: String,
    pub(crate) reference_snapshot_body_class_coverage_summary_checksum: u64,
    pub(crate) reference_snapshot_equatorial_parity_summary_path: String,
    pub(crate) reference_snapshot_equatorial_parity_summary_checksum: u64,
    pub(crate) reference_asteroid_source_window_summary_path: String,
    pub(crate) reference_asteroid_source_window_summary_checksum: u64,
    pub(crate) reference_asteroid_equatorial_evidence_summary_path: String,
    pub(crate) reference_asteroid_equatorial_evidence_summary_checksum: u64,
    pub(crate) independent_holdout_source_window_summary_path: String,
    pub(crate) independent_holdout_source_window_summary_checksum: u64,
    pub(crate) independent_holdout_equatorial_parity_summary_path: String,
    pub(crate) independent_holdout_equatorial_parity_summary_checksum: u64,
    pub(crate) independent_holdout_body_class_coverage_summary_path: String,
    pub(crate) independent_holdout_body_class_coverage_summary_checksum: u64,
    pub(crate) independent_holdout_quarter_day_boundary_summary_path: String,
    pub(crate) independent_holdout_quarter_day_boundary_summary_checksum: u64,
    pub(crate) production_generation_boundary_source_summary_path: String,
    pub(crate) production_generation_boundary_source_summary_checksum: u64,
    pub(crate) production_generation_boundary_window_summary_path: String,
    pub(crate) production_generation_boundary_window_summary_checksum: u64,
    pub(crate) production_generation_boundary_request_corpus_summary_path: String,
    pub(crate) production_generation_boundary_request_corpus_summary_checksum: u64,
    pub(crate) production_generation_boundary_request_corpus_equatorial_summary_path: String,
    pub(crate) production_generation_boundary_request_corpus_equatorial_summary_checksum: u64,
    pub(crate) reference_snapshot_summary_path: String,
    pub(crate) reference_snapshot_summary_checksum: u64,
    pub(crate) production_generation_summary_path: String,
    pub(crate) production_generation_summary_checksum: u64,
    pub(crate) production_generation_body_class_coverage_summary_path: String,
    pub(crate) production_generation_body_class_coverage_summary_checksum: u64,
    pub(crate) production_generation_source_summary_path: String,
    pub(crate) production_generation_source_summary_checksum: u64,
    pub(crate) production_generation_source_revision_summary_path: String,
    pub(crate) production_generation_source_revision_summary_checksum: u64,
    pub(crate) production_generation_source_window_summary_path: String,
    pub(crate) production_generation_source_window_summary_checksum: u64,
    pub(crate) production_generation_quarter_day_boundary_summary_path: String,
    pub(crate) production_generation_quarter_day_boundary_summary_checksum: u64,
    pub(crate) production_generation_corpus_shape_summary_path: String,
    pub(crate) production_generation_corpus_shape_summary_checksum: u64,
    pub(crate) production_generation_manifest_summary_path: String,
    pub(crate) production_generation_manifest_summary_checksum: u64,
    pub(crate) production_generation_manifest_checksum_path: String,
    pub(crate) production_generation_manifest_checksum_checksum: u64,
    pub(crate) catalog_inventory_summary_path: String,
    pub(crate) catalog_inventory_summary_checksum: u64,
    pub(crate) catalog_posture_summary_path: String,
    pub(crate) catalog_posture_summary_checksum: u64,
    pub(crate) custom_definition_ayanamsa_labels_summary_path: String,
    pub(crate) custom_definition_ayanamsa_labels_summary_checksum: u64,
    pub(crate) ayanamsa_provenance_summary_path: String,
    pub(crate) ayanamsa_provenance_summary_checksum: u64,
    pub(crate) validation_report_summary_path: String,
    pub(crate) validation_report_summary_checksum: u64,
    pub(crate) release_body_claims_summary_path: String,
    pub(crate) release_body_claims_summary_checksum: u64,
    pub(crate) body_date_channel_claims_summary_path: String,
    pub(crate) body_date_channel_claims_summary_checksum: u64,
    pub(crate) pluto_fallback_summary_path: String,
    pub(crate) pluto_fallback_summary_checksum: u64,
    pub(crate) request_policy_summary_path: String,
    pub(crate) request_policy_summary_checksum: u64,
    pub(crate) observer_policy_summary_path: String,
    pub(crate) observer_policy_summary_checksum: u64,
    pub(crate) apparentness_policy_summary_path: String,
    pub(crate) apparentness_policy_summary_checksum: u64,
    pub(crate) request_semantics_summary_path: String,
    pub(crate) request_semantics_summary_checksum: u64,
    pub(crate) unsupported_modes_summary_path: String,
    pub(crate) unsupported_modes_summary_checksum: u64,
    pub(crate) time_scale_policy_summary_path: String,
    pub(crate) utc_convenience_policy_summary_path: String,
    pub(crate) utc_convenience_policy_summary_checksum: u64,
    pub(crate) time_scale_policy_summary_checksum: u64,
    pub(crate) delta_t_policy_summary_path: String,
    pub(crate) delta_t_policy_summary_checksum: u64,
    pub(crate) native_sidereal_policy_summary_path: String,
    pub(crate) native_sidereal_policy_summary_checksum: u64,
    pub(crate) zodiac_policy_summary_path: String,
    pub(crate) zodiac_policy_summary_checksum: u64,
    pub(crate) lunar_theory_limitations_summary_path: String,
    pub(crate) lunar_theory_limitations_summary_checksum: u64,
    pub(crate) lunar_theory_source_selection_summary_path: String,
    pub(crate) lunar_theory_source_selection_summary_checksum: u64,
    pub(crate) lunar_theory_source_family_summary_path: String,
    pub(crate) lunar_theory_source_family_summary_checksum: u64,
    pub(crate) lunar_source_window_summary_path: String,
    pub(crate) lunar_source_window_summary_checksum: u64,
    pub(crate) lunar_theory_catalog_validation_summary_path: String,
    pub(crate) lunar_theory_catalog_validation_summary_checksum: u64,
    pub(crate) request_surface_summary_path: String,
    pub(crate) request_surface_summary_checksum: u64,
    pub(crate) compatibility_caveats_summary_path: String,
    pub(crate) compatibility_caveats_summary_checksum: u64,
    pub(crate) workspace_provenance_summary_path: String,
    pub(crate) workspace_provenance_summary_checksum: u64,
    pub(crate) workspace_audit_summary_path: String,
    pub(crate) workspace_audit_summary_checksum: u64,
    pub(crate) native_dependency_audit_summary_path: String,
    pub(crate) native_dependency_audit_summary_checksum: u64,
    pub(crate) artifact_summary_path: String,
    pub(crate) artifact_summary_checksum: u64,
    pub(crate) packaged_artifact_path: String,
    pub(crate) packaged_artifact_checksum_path: String,
    pub(crate) packaged_artifact_profile_coverage_summary_path: String,
    pub(crate) packaged_artifact_profile_coverage_summary_checksum: u64,
    pub(crate) packaged_artifact_access_summary_path: String,
    pub(crate) packaged_artifact_access_summary_checksum: u64,
    pub(crate) packaged_artifact_output_support_summary_path: String,
    pub(crate) packaged_artifact_output_support_summary_checksum: u64,
    pub(crate) packaged_artifact_fit_sample_classes_summary_path: String,
    pub(crate) packaged_artifact_fit_sample_classes_summary_checksum: u64,
    pub(crate) packaged_artifact_fit_threshold_violation_count_summary_path: String,
    pub(crate) packaged_artifact_fit_threshold_violation_count_summary_checksum: u64,
    pub(crate) packaged_artifact_fit_threshold_violations_summary_path: String,
    pub(crate) packaged_artifact_fit_threshold_violations_summary_checksum: u64,
    pub(crate) packaged_artifact_body_cadence_summary_path: String,
    pub(crate) packaged_artifact_body_cadence_summary_checksum: u64,
    pub(crate) packaged_artifact_body_class_span_cap_summary_path: String,
    pub(crate) packaged_artifact_body_class_span_cap_summary_checksum: u64,
    pub(crate) packaged_artifact_normalized_intermediate_summary_path: String,
    pub(crate) packaged_artifact_normalized_intermediate_summary_checksum: u64,
    pub(crate) packaged_artifact_speed_policy_summary_path: String,
    pub(crate) packaged_artifact_speed_policy_summary_checksum: u64,
    pub(crate) packaged_artifact_storage_summary_checksum: u64,
    pub(crate) packaged_artifact_production_profile_summary_checksum: u64,
    pub(crate) packaged_frame_treatment_summary_checksum: u64,
    pub(crate) packaged_artifact_target_threshold_summary_checksum: u64,
    pub(crate) packaged_artifact_target_threshold_state_summary_path: String,
    pub(crate) packaged_artifact_target_threshold_state_summary_checksum: u64,
    pub(crate) packaged_artifact_source_fit_holdout_sync_summary_path: String,
    pub(crate) packaged_artifact_source_fit_holdout_sync_summary_checksum: u64,
    pub(crate) packaged_artifact_target_threshold_scope_envelopes_summary_path: String,
    pub(crate) packaged_artifact_target_threshold_scope_envelopes_summary_checksum: u64,
    pub(crate) packaged_artifact_phase2_corpus_alignment_summary_path: String,
    pub(crate) packaged_artifact_phase2_corpus_alignment_summary_checksum: u64,
    pub(crate) packaged_lookup_epoch_policy_summary_path: String,
    pub(crate) packaged_lookup_epoch_policy_summary_checksum: u64,
    pub(crate) packaged_artifact_generation_policy_summary_path: String,
    pub(crate) packaged_artifact_generation_policy_summary_checksum: u64,
    pub(crate) packaged_artifact_generation_residual_bodies_summary_path: String,
    pub(crate) packaged_artifact_generation_residual_bodies_summary_checksum: u64,
    pub(crate) packaged_artifact_regeneration_summary_path: String,
    pub(crate) packaged_artifact_regeneration_summary_checksum: u64,
    pub(crate) packaged_artifact_generation_manifest_path: String,
    pub(crate) packaged_artifact_generation_manifest_checksum: u64,
    pub(crate) packaged_artifact_generation_manifest_summary_path: String,
    pub(crate) packaged_artifact_generation_manifest_summary_checksum: u64,
    pub(crate) packaged_artifact_generation_manifest_checksum_summary_path: String,
    pub(crate) packaged_artifact_generation_manifest_checksum_summary_checksum: u64,
    pub(crate) packaged_artifact_generation_manifest_checksum_path: String,
    pub(crate) packaged_artifact_generation_manifest_checksum_checksum: u64,
    pub(crate) benchmark_corpus_summary_path: String,
    pub(crate) benchmark_corpus_summary_checksum: u64,
    pub(crate) chart_benchmark_corpus_summary_path: String,
    pub(crate) chart_benchmark_corpus_summary_checksum: u64,
    pub(crate) selected_asteroid_source_request_corpus_summary_path: String,
    pub(crate) selected_asteroid_source_request_corpus_summary_checksum: u64,
    pub(crate) selected_asteroid_source_request_corpus_equatorial_summary_path: String,
    pub(crate) selected_asteroid_source_request_corpus_equatorial_summary_checksum: u64,
    pub(crate) selected_asteroid_source_window_summary_path: String,
    pub(crate) selected_asteroid_source_window_summary_checksum: u64,
    pub(crate) interpolation_quality_request_corpus_summary_path: String,
    pub(crate) interpolation_quality_request_corpus_summary_checksum: u64,
    pub(crate) benchmark_report_path: String,
    pub(crate) benchmark_report_checksum: u64,
    pub(crate) validation_report_path: String,
    pub(crate) validation_report_checksum: u64,
    pub(crate) source_revision: String,
    pub(crate) workspace_status: String,
    pub(crate) rustc_version: String,
    pub(crate) cargo_version: String,
    pub(crate) profile_id: String,
    pub(crate) api_stability_posture_id: String,
    pub(crate) validation_rounds: usize,
}

impl ParsedReleaseBundleManifest {
    pub(crate) fn parse(text: &str) -> Result<Self, ReleaseBundleError> {
        Ok(Self {
            profile_path: parse_manifest_string(text, "profile:")?,
            profile_checksum: parse_manifest_checksum(text, "profile checksum (fnv1a-64):")?,
            profile_summary_path: parse_manifest_string(text, "profile summary:")?,
            profile_summary_checksum: parse_manifest_checksum(
                text,
                "profile summary checksum (fnv1a-64):",
            )?,
            release_notes_path: parse_manifest_string(text, "release notes:")?,
            release_notes_checksum: parse_manifest_checksum(
                text,
                "release notes checksum (fnv1a-64):",
            )?,
            release_notes_summary_path: parse_manifest_string(text, "release notes summary:")?,
            release_notes_summary_checksum: parse_manifest_checksum(
                text,
                "release notes summary checksum (fnv1a-64):",
            )?,
            release_summary_path: parse_manifest_string(text, "release summary:")?,
            release_summary_checksum: parse_manifest_checksum(
                text,
                "release summary checksum (fnv1a-64):",
            )?,
            release_profile_identifiers_path: parse_manifest_string(
                text,
                "release-profile identifiers:",
            )?,
            release_profile_identifiers_checksum: parse_manifest_checksum(
                text,
                "release-profile identifiers checksum (fnv1a-64):",
            )?,
            release_profile_identifiers_summary_path: parse_manifest_string(
                text,
                "release-profile identifiers summary:",
            )?,
            release_profile_identifiers_summary_checksum: parse_manifest_checksum(
                text,
                "release-profile identifiers summary checksum (fnv1a-64):",
            )?,
            release_house_system_canonical_names_summary_path: parse_manifest_string(
                text,
                "release-house-system-canonical-names summary:",
            )?,
            release_house_system_canonical_names_summary_checksum: parse_manifest_checksum(
                text,
                "release-house-system-canonical-names summary checksum (fnv1a-64):",
            )?,
            release_ayanamsa_canonical_names_summary_path: parse_manifest_string(
                text,
                "release-ayanamsa-canonical-names summary:",
            )?,
            release_ayanamsa_canonical_names_summary_checksum: parse_manifest_checksum(
                text,
                "release-ayanamsa-canonical-names summary checksum (fnv1a-64):",
            )?,
            release_house_validation_summary_path: parse_manifest_string(
                text,
                "release-house-validation summary:",
            )?,
            release_house_validation_summary_checksum: parse_manifest_checksum(
                text,
                "release-house-validation summary checksum (fnv1a-64):",
            )?,
            target_house_scope_summary_path: parse_manifest_string(
                text,
                "target-house-scope summary:",
            )?,
            target_house_scope_summary_checksum: parse_manifest_checksum(
                text,
                "target-house-scope summary checksum (fnv1a-64):",
            )?,
            target_ayanamsa_scope_summary_path: parse_manifest_string(
                text,
                "target-ayanamsa-scope summary:",
            )?,
            target_ayanamsa_scope_summary_checksum: parse_manifest_checksum(
                text,
                "target-ayanamsa-scope summary checksum (fnv1a-64):",
            )?,
            house_code_aliases_summary_path: parse_manifest_string(
                text,
                "house code aliases summary:",
            )?,
            house_code_aliases_summary_checksum: parse_manifest_checksum(
                text,
                "house code aliases summary checksum (fnv1a-64):",
            )?,
            house_formula_families_summary_path: parse_manifest_string(
                text,
                "house formula families summary:",
            )?,
            house_formula_families_summary_checksum: parse_manifest_checksum(
                text,
                "house formula families summary checksum (fnv1a-64):",
            )?,
            house_latitude_sensitive_summary_path: parse_manifest_string(
                text,
                "house latitude-sensitive summary:",
            )?,
            house_latitude_sensitive_constraints_summary_path: parse_manifest_string(
                text,
                "house latitude-sensitive constraints summary:",
            )?,
            house_latitude_sensitive_constraints_summary_checksum: parse_manifest_checksum(
                text,
                "house latitude-sensitive constraints summary checksum (fnv1a-64):",
            )?,
            house_latitude_sensitive_failure_modes_summary_path: parse_manifest_string(
                text,
                "house latitude-sensitive failure-modes summary:",
            )?,
            house_latitude_sensitive_failure_modes_summary_checksum: parse_manifest_checksum(
                text,
                "house latitude-sensitive failure-modes summary checksum (fnv1a-64):",
            )?,
            house_latitude_sensitive_summary_checksum: parse_manifest_checksum(
                text,
                "house latitude-sensitive summary checksum (fnv1a-64):",
            )?,
            release_checklist_path: parse_manifest_string(text, "release checklist:")?,
            release_checklist_checksum: parse_manifest_checksum(
                text,
                "release checklist checksum (fnv1a-64):",
            )?,
            release_checklist_summary_path: parse_manifest_string(
                text,
                "release checklist summary:",
            )?,
            release_checklist_summary_checksum: parse_manifest_checksum(
                text,
                "release checklist summary checksum (fnv1a-64):",
            )?,
            backend_matrix_path: parse_manifest_string(text, "backend matrix:")?,
            backend_matrix_checksum: parse_manifest_checksum(
                text,
                "backend matrix checksum (fnv1a-64):",
            )?,
            backend_matrix_summary_path: parse_manifest_string(text, "backend matrix summary:")?,
            backend_matrix_summary_checksum: parse_manifest_checksum(
                text,
                "backend matrix summary checksum (fnv1a-64):",
            )?,
            api_stability_path: parse_manifest_string(text, "api stability posture:")?,
            api_stability_checksum: parse_manifest_checksum(
                text,
                "api stability checksum (fnv1a-64):",
            )?,
            api_stability_summary_path: parse_manifest_string(text, "api stability summary:")?,
            api_stability_summary_checksum: parse_manifest_checksum(
                text,
                "api stability summary checksum (fnv1a-64):",
            )?,
            comparison_corpus_summary_path: parse_manifest_string(
                text,
                "comparison-corpus summary:",
            )?,
            comparison_corpus_summary_checksum: parse_manifest_checksum(
                text,
                "comparison-corpus summary checksum (fnv1a-64):",
            )?,
            source_corpus_summary_path: parse_manifest_string(text, "source-corpus summary:")?,
            source_corpus_summary_checksum: parse_manifest_checksum(
                text,
                "source-corpus summary checksum (fnv1a-64):",
            )?,
            jpl_source_posture_summary_path: parse_manifest_string(
                text,
                "jpl source posture summary:",
            )?,
            jpl_source_posture_summary_checksum: parse_manifest_checksum(
                text,
                "jpl source posture summary checksum (fnv1a-64):",
            )?,
            jpl_provenance_only_summary_path: parse_manifest_string(
                text,
                "jpl provenance-only evidence summary:",
            )?,
            jpl_provenance_only_summary_checksum: parse_manifest_checksum(
                text,
                "jpl provenance-only evidence summary checksum (fnv1a-64):",
            )?,
            comparison_snapshot_summary_path: parse_manifest_string(
                text,
                "comparison-snapshot summary:",
            )?,
            comparison_snapshot_summary_checksum: parse_manifest_checksum(
                text,
                "comparison-snapshot summary checksum (fnv1a-64):",
            )?,
            comparison_snapshot_source_summary_path: parse_manifest_string(
                text,
                "comparison-snapshot source summary:",
            )?,
            comparison_snapshot_source_summary_checksum: parse_manifest_checksum(
                text,
                "comparison-snapshot source summary checksum (fnv1a-64):",
            )?,
            comparison_snapshot_source_window_summary_path: parse_manifest_string(
                text,
                "comparison-snapshot source window summary:",
            )?,
            comparison_snapshot_source_window_summary_checksum: parse_manifest_checksum(
                text,
                "comparison-snapshot source window summary checksum (fnv1a-64):",
            )?,
            comparison_snapshot_body_class_coverage_summary_path: parse_manifest_string(
                text,
                "comparison-snapshot body-class coverage summary:",
            )?,
            comparison_snapshot_body_class_coverage_summary_checksum: parse_manifest_checksum(
                text,
                "comparison-snapshot body-class coverage summary checksum (fnv1a-64):",
            )?,
            comparison_snapshot_manifest_summary_path: parse_manifest_string(
                text,
                "comparison-snapshot manifest summary:",
            )?,
            comparison_snapshot_manifest_summary_checksum: parse_manifest_checksum(
                text,
                "comparison-snapshot manifest summary checksum (fnv1a-64):",
            )?,
            comparison_envelope_summary_path: parse_manifest_string(
                text,
                "comparison-envelope summary:",
            )?,
            comparison_envelope_summary_checksum: parse_manifest_checksum(
                text,
                "comparison-envelope summary checksum (fnv1a-64):",
            )?,
            comparison_body_class_tolerance_summary_path: parse_manifest_string(
                text,
                "comparison-body-class-tolerance summary:",
            )?,
            comparison_body_class_tolerance_summary_checksum: parse_manifest_checksum(
                text,
                "comparison-body-class-tolerance summary checksum (fnv1a-64):",
            )?,
            comparison_body_class_error_envelope_summary_path: parse_manifest_string(
                text,
                "comparison-body-class-error-envelope summary:",
            )?,
            comparison_body_class_error_envelope_summary_checksum: parse_manifest_checksum(
                text,
                "comparison-body-class-error-envelope summary checksum (fnv1a-64):",
            )?,
            comparison_corpus_release_guard_summary_path: parse_manifest_string(
                text,
                "comparison-corpus release-guard summary:",
            )?,
            comparison_corpus_release_guard_summary_checksum: parse_manifest_checksum(
                text,
                "comparison-corpus release-guard summary checksum (fnv1a-64):",
            )?,
            reference_holdout_overlap_summary_path: parse_manifest_string(
                text,
                "reference-holdout overlap summary:",
            )?,
            reference_holdout_overlap_summary_checksum: parse_manifest_checksum(
                text,
                "reference-holdout overlap summary checksum (fnv1a-64):",
            )?,
            reference_snapshot_bridge_day_summary_path: parse_manifest_string(
                text,
                "reference snapshot bridge day summary:",
            )?,
            reference_snapshot_bridge_day_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot bridge day summary checksum (fnv1a-64):",
            )?,
            reference_snapshot_major_body_boundary_window_summary_path: parse_manifest_string(
                text,
                "reference snapshot major-body boundary window summary:",
            )?,
            reference_snapshot_major_body_boundary_window_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot major-body boundary window summary checksum (fnv1a-64):",
            )?,
            reference_snapshot_boundary_epoch_coverage_summary_path: parse_manifest_string(
                text,
                "reference snapshot boundary epoch coverage summary:",
            )?,
            reference_snapshot_boundary_epoch_coverage_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot boundary epoch coverage summary checksum (fnv1a-64):",
            )?,
            reference_snapshot_pre_bridge_boundary_summary_path: parse_manifest_string(
                text,
                "reference snapshot pre-bridge boundary summary:",
            )?,
            reference_snapshot_pre_bridge_boundary_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot pre-bridge boundary summary checksum (fnv1a-64):",
            )?,
            reference_snapshot_2451917_major_body_boundary_summary_path: parse_manifest_string(
                text,
                "reference snapshot 2451917 major-body boundary summary:",
            )?,
            reference_snapshot_2451917_major_body_boundary_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot 2451917 major-body boundary summary checksum (fnv1a-64):",
            )?,
            reference_snapshot_2451918_major_body_boundary_summary_path: parse_manifest_string(
                text,
                "reference snapshot 2451918 major-body boundary summary:",
            )?,
            reference_snapshot_2451918_major_body_boundary_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot 2451918 major-body boundary summary checksum (fnv1a-64):",
            )?,
            reference_snapshot_2451919_major_body_boundary_summary_path: parse_manifest_string(
                text,
                "reference snapshot 2451919 major-body boundary summary:",
            )?,
            reference_snapshot_2451919_major_body_boundary_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot 2451919 major-body boundary summary checksum (fnv1a-64):",
            )?,
            reference_snapshot_2451916_major_body_dense_boundary_summary_path: parse_manifest_string(
                text,
                "reference snapshot 2451916 major-body dense boundary summary:",
            )?,
            reference_snapshot_2451916_major_body_dense_boundary_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot 2451916 major-body dense boundary summary checksum (fnv1a-64):",
            )?,
            reference_snapshot_sparse_boundary_summary_path: parse_manifest_string(
                text,
                "reference snapshot sparse boundary summary:",
            )?,
            reference_snapshot_sparse_boundary_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot sparse boundary summary checksum (fnv1a-64):",
            )?,
            reference_snapshot_exact_j2000_evidence_summary_path: parse_manifest_string(
                text,
                "reference snapshot exact J2000 evidence summary:",
            )?,
            reference_snapshot_exact_j2000_evidence_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot exact J2000 evidence summary checksum (fnv1a-64):",
            )?,
            reference_snapshot_source_summary_path: parse_manifest_string(
                text,
                "reference snapshot source summary:",
            )?,
            reference_snapshot_source_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot source summary checksum (fnv1a-64):",
            )?,
            reference_snapshot_source_window_summary_path: parse_manifest_string(
                text,
                "reference snapshot source window summary:",
            )?,
            reference_snapshot_source_window_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot source window summary checksum (fnv1a-64):",
            )?,
            reference_snapshot_manifest_summary_path: parse_manifest_string(
                text,
                "reference snapshot manifest summary:",
            )?,
            reference_snapshot_manifest_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot manifest summary checksum (fnv1a-64):",
            )?,
            reference_snapshot_body_class_coverage_summary_path: parse_manifest_string(
                text,
                "reference snapshot body-class coverage summary:",
            )?,
            reference_snapshot_body_class_coverage_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot body-class coverage summary checksum (fnv1a-64):",
            )?,
            reference_snapshot_equatorial_parity_summary_path: parse_manifest_string(
                text,
                "reference snapshot equatorial parity summary:",
            )?,
            reference_snapshot_equatorial_parity_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot equatorial parity summary checksum (fnv1a-64):",
            )?,
            reference_asteroid_source_window_summary_path: parse_manifest_string(
                text,
                "reference asteroid source window summary:",
            )?,
            reference_asteroid_source_window_summary_checksum: parse_manifest_checksum(
                text,
                "reference asteroid source window summary checksum (fnv1a-64):",
            )?,
            reference_asteroid_equatorial_evidence_summary_path: parse_manifest_string(
                text,
                "reference asteroid equatorial evidence summary:",
            )?,
            reference_asteroid_equatorial_evidence_summary_checksum: parse_manifest_checksum(
                text,
                "reference asteroid equatorial evidence summary checksum (fnv1a-64):",
            )?,
            independent_holdout_source_window_summary_path: parse_manifest_string(
                text,
                "independent-holdout source window summary:",
            )?,
            independent_holdout_source_window_summary_checksum: parse_manifest_checksum(
                text,
                "independent-holdout source window summary checksum (fnv1a-64):",
            )?,
            independent_holdout_equatorial_parity_summary_path: parse_manifest_string(
                text,
                "independent-holdout equatorial parity summary:",
            )?,
            independent_holdout_equatorial_parity_summary_checksum: parse_manifest_checksum(
                text,
                "independent-holdout equatorial parity summary checksum (fnv1a-64):",
            )?,
            independent_holdout_body_class_coverage_summary_path: parse_manifest_string(
                text,
                "independent-holdout body-class coverage summary:",
            )?,
            independent_holdout_body_class_coverage_summary_checksum: parse_manifest_checksum(
                text,
                "independent-holdout body-class coverage summary checksum (fnv1a-64):",
            )?,
            independent_holdout_quarter_day_boundary_summary_path: parse_manifest_string(
                text,
                "independent-holdout quarter-day boundary summary:",
            )?,
            independent_holdout_quarter_day_boundary_summary_checksum: parse_manifest_checksum(
                text,
                "independent-holdout quarter-day boundary summary checksum (fnv1a-64):",
            )?,
            production_generation_boundary_source_summary_path: parse_manifest_string(
                text,
                "production generation boundary source summary:",
            )?,
            production_generation_boundary_source_summary_checksum: parse_manifest_checksum(
                text,
                "production generation boundary source summary checksum (fnv1a-64):",
            )?,
            production_generation_boundary_window_summary_path: parse_manifest_string(
                text,
                "production generation boundary window summary:",
            )?,
            production_generation_boundary_window_summary_checksum: parse_manifest_checksum(
                text,
                "production generation boundary window summary checksum (fnv1a-64):",
            )?,
            production_generation_boundary_request_corpus_summary_path: parse_manifest_string(
                text,
                "production generation boundary request corpus summary:",
            )?,
            production_generation_boundary_request_corpus_summary_checksum: parse_manifest_checksum(
                text,
                "production generation boundary request corpus summary checksum (fnv1a-64):",
            )?,
            production_generation_boundary_request_corpus_equatorial_summary_path:
                parse_manifest_string(
                    text,
                    "production generation boundary request corpus equatorial summary:",
                )?,
            production_generation_boundary_request_corpus_equatorial_summary_checksum:
                parse_manifest_checksum(
                    text,
                    "production generation boundary request corpus equatorial summary checksum (fnv1a-64):",
                )?,
            reference_snapshot_summary_path: parse_manifest_string(
                text,
                "reference snapshot summary:",
            )?,
            reference_snapshot_summary_checksum: parse_manifest_checksum(
                text,
                "reference snapshot summary checksum (fnv1a-64):",
            )?,
            production_generation_summary_path: parse_manifest_string(
                text,
                "production generation summary:",
            )?,
            production_generation_summary_checksum: parse_manifest_checksum(
                text,
                "production generation summary checksum (fnv1a-64):",
            )?,
            production_generation_body_class_coverage_summary_path: parse_manifest_string(
                text,
                "production generation body-class coverage summary:",
            )?,
            production_generation_body_class_coverage_summary_checksum: parse_manifest_checksum(
                text,
                "production generation body-class coverage summary checksum (fnv1a-64):",
            )?,
            production_generation_source_summary_path: parse_manifest_string(
                text,
                "production generation source summary:",
            )?,
            production_generation_source_summary_checksum: parse_manifest_checksum(
                text,
                "production generation source summary checksum (fnv1a-64):",
            )?,
            production_generation_source_revision_summary_path: parse_manifest_string(
                text,
                "production generation source revision summary:",
            )?,
            production_generation_source_revision_summary_checksum: parse_manifest_checksum(
                text,
                "production generation source revision summary checksum (fnv1a-64):",
            )?,
            production_generation_source_window_summary_path: parse_manifest_string(
                text,
                "production generation source window summary:",
            )?,
            production_generation_source_window_summary_checksum: parse_manifest_checksum(
                text,
                "production generation source window summary checksum (fnv1a-64):",
            )?,
            production_generation_quarter_day_boundary_summary_path: parse_manifest_string(
                text,
                "production generation quarter-day boundary summary:",
            )?,
            production_generation_quarter_day_boundary_summary_checksum: parse_manifest_checksum(
                text,
                "production generation quarter-day boundary summary checksum (fnv1a-64):",
            )?,
            production_generation_corpus_shape_summary_path: parse_manifest_string(
                text,
                "production generation corpus shape summary:",
            )?,
            production_generation_corpus_shape_summary_checksum: parse_manifest_checksum(
                text,
                "production generation corpus shape summary checksum (fnv1a-64):",
            )?,
            production_generation_manifest_summary_path: parse_manifest_string(
                text,
                "production generation manifest summary:",
            )?,
            production_generation_manifest_summary_checksum: parse_manifest_checksum(
                text,
                "production generation manifest summary checksum (fnv1a-64):",
            )?,
            production_generation_manifest_checksum_path: parse_manifest_string(
                text,
                "production generation manifest checksum summary:",
            )?,
            production_generation_manifest_checksum_checksum: parse_manifest_checksum(
                text,
                "production generation manifest checksum summary checksum (fnv1a-64):",
            )?,
            catalog_inventory_summary_path: parse_manifest_string(
                text,
                "catalog inventory summary:",
            )?,
            catalog_inventory_summary_checksum: parse_manifest_checksum(
                text,
                "catalog inventory summary checksum (fnv1a-64):",
            )?,
            catalog_posture_summary_path: parse_manifest_string(
                text,
                "catalog posture summary:",
            )?,
            catalog_posture_summary_checksum: parse_manifest_checksum(
                text,
                "catalog posture summary checksum (fnv1a-64):",
            )?,
            custom_definition_ayanamsa_labels_summary_path: parse_manifest_string(
                text,
                "custom-definition ayanamsa labels summary:",
            )?,
            custom_definition_ayanamsa_labels_summary_checksum: parse_manifest_checksum(
                text,
                "custom-definition ayanamsa labels summary checksum (fnv1a-64):",
            )?,
            ayanamsa_provenance_summary_path: parse_manifest_string(
                text,
                "ayanamsa provenance summary:",
            )?,
            ayanamsa_provenance_summary_checksum: parse_manifest_checksum(
                text,
                "ayanamsa provenance summary checksum (fnv1a-64):",
            )?,
            validation_report_summary_path: parse_manifest_string(
                text,
                "validation report summary:",
            )?,
            validation_report_summary_checksum: parse_manifest_checksum(
                text,
                "validation report summary checksum (fnv1a-64):",
            )?,
            workspace_provenance_summary_path: parse_manifest_string(
                text,
                "workspace provenance summary:",
            )?,
            workspace_provenance_summary_checksum: parse_manifest_checksum(
                text,
                "workspace provenance summary checksum (fnv1a-64):",
            )?,
            release_body_claims_summary_path: parse_manifest_string(
                text,
                "release body claims summary:",
            )?,
            release_body_claims_summary_checksum: parse_manifest_checksum(
                text,
                "release body claims summary checksum (fnv1a-64):",
            )?,
            body_date_channel_claims_summary_path: parse_manifest_string(
                text,
                "body/date/channel claims summary:",
            )?,
            body_date_channel_claims_summary_checksum: parse_manifest_checksum(
                text,
                "body/date/channel claims summary checksum (fnv1a-64):",
            )?,
            pluto_fallback_summary_path: parse_manifest_string(text, "pluto fallback summary:")?,
            pluto_fallback_summary_checksum: parse_manifest_checksum(
                text,
                "pluto fallback summary checksum (fnv1a-64):",
            )?,
            request_policy_summary_path: parse_manifest_string(text, "request policy summary:")?,
            request_policy_summary_checksum: parse_manifest_checksum(
                text,
                "request policy summary checksum (fnv1a-64):",
            )?,
            observer_policy_summary_path: parse_manifest_string(
                text,
                "observer policy summary:",
            )?,
            observer_policy_summary_checksum: parse_manifest_checksum(
                text,
                "observer policy summary checksum (fnv1a-64):",
            )?,
            apparentness_policy_summary_path: parse_manifest_string(
                text,
                "apparentness policy summary:",
            )?,
            apparentness_policy_summary_checksum: parse_manifest_checksum(
                text,
                "apparentness policy summary checksum (fnv1a-64):",
            )?,
            request_semantics_summary_path: parse_manifest_string(
                text,
                "request-semantics summary:",
            )?,
            request_semantics_summary_checksum: parse_manifest_checksum(
                text,
                "request-semantics summary checksum (fnv1a-64):",
            )?,
            unsupported_modes_summary_path: parse_manifest_string(
                text,
                "unsupported-modes summary:",
            )?,
            unsupported_modes_summary_checksum: parse_manifest_checksum(
                text,
                "unsupported-modes summary checksum (fnv1a-64):",
            )?,
            time_scale_policy_summary_path: parse_manifest_string(
                text,
                "time-scale policy summary:",
            )?,
            time_scale_policy_summary_checksum: parse_manifest_checksum(
                text,
                "time-scale policy summary checksum (fnv1a-64):",
            )?,
            utc_convenience_policy_summary_path: parse_manifest_string(
                text,
                "utc-convenience policy summary:",
            )?,
            utc_convenience_policy_summary_checksum: parse_manifest_checksum(
                text,
                "utc-convenience policy summary checksum (fnv1a-64):",
            )?,
            delta_t_policy_summary_path: parse_manifest_string(text, "delta-t policy summary:")?,
            delta_t_policy_summary_checksum: parse_manifest_checksum(
                text,
                "delta-t policy summary checksum (fnv1a-64):",
            )?,
            native_sidereal_policy_summary_path: parse_manifest_string(
                text,
                "native sidereal policy summary:",
            )?,
            native_sidereal_policy_summary_checksum: parse_manifest_checksum(
                text,
                "native sidereal policy summary checksum (fnv1a-64):",
            )?,
            zodiac_policy_summary_path: parse_manifest_string(
                text,
                "zodiac policy summary:",
            )?,
            zodiac_policy_summary_checksum: parse_manifest_checksum(
                text,
                "zodiac policy summary checksum (fnv1a-64):",
            )?,
            lunar_theory_limitations_summary_path: parse_manifest_string(
                text,
                "lunar theory limitations summary:",
            )?,
            lunar_theory_limitations_summary_checksum: parse_manifest_checksum(
                text,
                "lunar theory limitations summary checksum (fnv1a-64):",
            )?,
            lunar_theory_source_selection_summary_path: parse_manifest_string(
                text,
                "lunar theory source selection summary:",
            )?,
            lunar_theory_source_selection_summary_checksum: parse_manifest_checksum(
                text,
                "lunar theory source selection summary checksum (fnv1a-64):",
            )?,
            lunar_theory_source_family_summary_path: parse_manifest_string(
                text,
                "lunar theory source family summary:",
            )?,
            lunar_theory_source_family_summary_checksum: parse_manifest_checksum(
                text,
                "lunar theory source family summary checksum (fnv1a-64):",
            )?,
            lunar_source_window_summary_path: parse_manifest_string(
                text,
                "lunar theory source window summary:",
            )?,
            lunar_source_window_summary_checksum: parse_manifest_checksum(
                text,
                "lunar theory source window summary checksum (fnv1a-64):",
            )?,
            lunar_theory_catalog_validation_summary_path: parse_manifest_string(
                text,
                "lunar theory catalog validation summary:",
            )?,
            lunar_theory_catalog_validation_summary_checksum: parse_manifest_checksum(
                text,
                "lunar theory catalog validation summary checksum (fnv1a-64):",
            )?,
            request_surface_summary_path: parse_manifest_string(text, "request surface summary:")?,
            request_surface_summary_checksum: parse_manifest_checksum(
                text,
                "request surface summary checksum (fnv1a-64):",
            )?,
            compatibility_caveats_summary_path: parse_manifest_string(
                text,
                "compatibility caveats summary:",
            )?,
            compatibility_caveats_summary_checksum: parse_manifest_checksum(
                text,
                "compatibility caveats summary checksum (fnv1a-64):",
            )?,
            workspace_audit_summary_path: parse_manifest_string(text, "workspace audit summary:")?,
            workspace_audit_summary_checksum: parse_manifest_checksum(
                text,
                "workspace audit summary checksum (fnv1a-64):",
            )?,
            native_dependency_audit_summary_path: parse_manifest_string(
                text,
                "native-dependency audit summary:",
            )?,
            native_dependency_audit_summary_checksum: parse_manifest_checksum(
                text,
                "native-dependency audit summary checksum (fnv1a-64):",
            )?,
            artifact_summary_path: parse_manifest_string(text, "artifact summary:")?,
            artifact_summary_checksum: parse_manifest_checksum(
                text,
                "artifact summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_path: parse_manifest_string(text, "packaged-artifact:")?,
            packaged_artifact_checksum_path: parse_manifest_string(
                text,
                "packaged-artifact checksum sidecar:",
            )?,
            packaged_artifact_profile_coverage_summary_path: parse_manifest_string(
                text,
                "packaged-artifact profile coverage summary:",
            )?,
            packaged_artifact_profile_coverage_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact profile coverage summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_access_summary_path: parse_manifest_string(
                text,
                "packaged-artifact access summary:",
            )?,
            packaged_artifact_access_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact access summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_output_support_summary_path: parse_manifest_string(
                text,
                "packaged-artifact output support summary:",
            )?,
            packaged_artifact_output_support_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact output support summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_fit_sample_classes_summary_path: parse_manifest_string(
                text,
                "packaged-artifact fit sample classes summary:",
            )?,
            packaged_artifact_fit_sample_classes_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact fit sample classes summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_fit_threshold_violation_count_summary_path: parse_manifest_string(
                text,
                "packaged-artifact fit threshold violation count summary:",
            )?,
            packaged_artifact_fit_threshold_violation_count_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact fit threshold violation count summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_fit_threshold_violations_summary_path: parse_manifest_string(
                text,
                "packaged-artifact fit threshold violations summary:",
            )?,
            packaged_artifact_fit_threshold_violations_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact fit threshold violations summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_body_cadence_summary_path: parse_manifest_string(
                text,
                "packaged-artifact body cadence summary:",
            )?,
            packaged_artifact_body_cadence_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact body cadence summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_body_class_span_cap_summary_path: parse_manifest_string(
                text,
                "packaged-artifact body-class span cap summary:",
            )?,
            packaged_artifact_body_class_span_cap_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact body-class span cap summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_normalized_intermediate_summary_path: parse_manifest_string(
                text,
                "packaged-artifact normalized intermediate summary:",
            )?,
            packaged_artifact_normalized_intermediate_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact normalized intermediate summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_speed_policy_summary_path: parse_manifest_string(
                text,
                "packaged-artifact speed policy summary:",
            )?,
            packaged_artifact_speed_policy_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact speed policy summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_storage_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact storage summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_production_profile_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact production-profile summary checksum (fnv1a-64):",
            )?,
            packaged_frame_treatment_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-frame-treatment summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_target_threshold_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact target-threshold summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_target_threshold_state_summary_path: parse_manifest_string(
                text,
                "packaged-artifact target-threshold state summary:",
            )?,
            packaged_artifact_target_threshold_state_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact target-threshold state summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_source_fit_holdout_sync_summary_path: parse_manifest_string(
                text,
                "packaged-artifact source-fit and hold-out sync summary:",
            )?,
            packaged_artifact_source_fit_holdout_sync_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact source-fit and hold-out sync summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_target_threshold_scope_envelopes_summary_path: parse_manifest_string(
                text,
                "packaged-artifact target-threshold scope envelopes summary:",
            )?,
            packaged_artifact_target_threshold_scope_envelopes_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact target-threshold scope envelopes summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_phase2_corpus_alignment_summary_path: parse_manifest_string(
                text,
                "packaged-artifact phase-2 corpus alignment summary:",
            )?,
            packaged_artifact_phase2_corpus_alignment_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact phase-2 corpus alignment summary checksum (fnv1a-64):",
            )?,
            packaged_lookup_epoch_policy_summary_path: parse_manifest_string(
                text,
                "packaged-artifact lookup-epoch policy summary:",
            )?,
            packaged_lookup_epoch_policy_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact lookup-epoch policy summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_generation_policy_summary_path: parse_manifest_string(
                text,
                "packaged-artifact generation policy summary:",
            )?,
            packaged_artifact_generation_policy_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact generation policy summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_generation_residual_bodies_summary_path: parse_manifest_string(
                text,
                "packaged-artifact generation residual bodies summary:",
            )?,
            packaged_artifact_generation_residual_bodies_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact generation residual bodies summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_regeneration_summary_path: parse_manifest_string(
                text,
                "packaged-artifact regeneration summary:",
            )?,
            packaged_artifact_regeneration_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact regeneration summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_generation_manifest_path: parse_manifest_string(
                text,
                "packaged-artifact generation manifest:",
            )?,
            packaged_artifact_generation_manifest_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact generation manifest checksum (fnv1a-64):",
            )?,
            packaged_artifact_generation_manifest_summary_path: parse_manifest_string(
                text,
                "packaged-artifact generation manifest summary:",
            )?,
            packaged_artifact_generation_manifest_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact generation manifest summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_generation_manifest_checksum_summary_path: parse_manifest_string(
                text,
                "packaged-artifact generation manifest checksum summary:",
            )?,
            packaged_artifact_generation_manifest_checksum_summary_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact generation manifest checksum summary checksum (fnv1a-64):",
            )?,
            packaged_artifact_generation_manifest_checksum_path: parse_manifest_string(
                text,
                "packaged-artifact generation manifest checksum sidecar:",
            )?,
            packaged_artifact_generation_manifest_checksum_checksum: parse_manifest_checksum(
                text,
                "packaged-artifact generation manifest checksum sidecar checksum (fnv1a-64):",
            )?,
            benchmark_corpus_summary_path: parse_manifest_string(
                text,
                "benchmark-corpus summary:",
            )?,
            benchmark_corpus_summary_checksum: parse_manifest_checksum(
                text,
                "benchmark-corpus summary checksum (fnv1a-64):",
            )?,
            chart_benchmark_corpus_summary_path: parse_manifest_string(
                text,
                "chart-benchmark-corpus summary:",
            )?,
            chart_benchmark_corpus_summary_checksum: parse_manifest_checksum(
                text,
                "chart-benchmark-corpus summary checksum (fnv1a-64):",
            )?,
            selected_asteroid_source_request_corpus_summary_path: parse_manifest_string(
                text,
                "selected asteroid source request corpus summary:",
            )?,
            selected_asteroid_source_request_corpus_summary_checksum: parse_manifest_checksum(
                text,
                "selected asteroid source request corpus summary checksum (fnv1a-64):",
            )?,
            selected_asteroid_source_request_corpus_equatorial_summary_path:
                parse_manifest_string(
                    text,
                    "selected asteroid source request corpus equatorial summary:",
                )?,
            selected_asteroid_source_request_corpus_equatorial_summary_checksum:
                parse_manifest_checksum(
                    text,
                    "selected asteroid source request corpus equatorial summary checksum (fnv1a-64):",
                )?,
            selected_asteroid_source_window_summary_path: parse_manifest_string(
                text,
                "selected asteroid source window summary:",
            )?,
            selected_asteroid_source_window_summary_checksum: parse_manifest_checksum(
                text,
                "selected asteroid source window summary checksum (fnv1a-64):",
            )?,
            interpolation_quality_request_corpus_summary_path: parse_manifest_string(
                text,
                "interpolation-quality sample request corpus summary:",
            )?,
            interpolation_quality_request_corpus_summary_checksum: parse_manifest_checksum(
                text,
                "interpolation-quality sample request corpus summary checksum (fnv1a-64):",
            )?,
            benchmark_report_path: parse_manifest_string(text, "benchmark report:")?,
            benchmark_report_checksum: parse_manifest_checksum(
                text,
                "benchmark report checksum (fnv1a-64):",
            )?,
            validation_report_path: parse_manifest_string(text, "validation report:")?,
            validation_report_checksum: parse_manifest_checksum(
                text,
                "validation report checksum (fnv1a-64):",
            )?,
            source_revision: parse_manifest_string(text, "source revision:")?,
            workspace_status: parse_manifest_string(text, "workspace status:")?,
            rustc_version: parse_manifest_string(text, "rustc version:")?,
            cargo_version: parse_manifest_string(text, "cargo version:")?,
            profile_id: parse_manifest_string(text, "profile id:")?,
            api_stability_posture_id: parse_manifest_string(text, "api stability posture id:")?,
            validation_rounds: parse_manifest_usize(
                text,
                "validation rounds:",
                "validation rounds",
            )?,
        })
    }
}

pub(crate) fn ensure_release_bundle_directory_contents(
    output_dir: &Path,
) -> Result<(), ReleaseBundleError> {
    let expected_entries: BTreeSet<String> = [
        "compatibility-profile.txt",
        "compatibility-profile-summary.txt",
        "release-notes.txt",
        "release-notes-summary.txt",
        "release-summary.txt",
        "release-profile-identifiers.txt",
        "release-profile-identifiers-summary.txt",
        "release-house-system-canonical-names-summary.txt",
        "release-ayanamsa-canonical-names-summary.txt",
        "release-house-validation-summary.txt",
        "target-house-scope-summary.txt",
        "target-ayanamsa-scope-summary.txt",
        "house-code-aliases-summary.txt",
        "house-formula-families-summary.txt",
        "house-latitude-sensitive-summary.txt",
        "house-latitude-sensitive-constraints-summary.txt",
        "house-latitude-sensitive-failure-modes-summary.txt",
        "release-checklist.txt",
        "release-checklist-summary.txt",
        "backend-matrix.txt",
        "backend-matrix-summary.txt",
        "api-stability.txt",
        "api-stability-summary.txt",
        "comparison-corpus-summary.txt",
        "source-corpus-summary.txt",
        "jpl-source-posture-summary.txt",
        "jpl-provenance-only-summary.txt",
        "comparison-snapshot-summary.txt",
        "comparison-snapshot-source-summary.txt",
        "comparison-snapshot-source-window-summary.txt",
        "comparison-snapshot-body-class-coverage-summary.txt",
        "comparison-snapshot-manifest-summary.txt",
        "comparison-envelope-summary.txt",
        "comparison-body-class-tolerance-summary.txt",
        "comparison-body-class-error-envelope-summary.txt",
        "benchmark-corpus-summary.txt",
        "chart-benchmark-corpus-summary.txt",
        "selected-asteroid-source-request-corpus-summary.txt",
        "selected-asteroid-source-request-corpus-equatorial-summary.txt",
        "selected-asteroid-source-window-summary.txt",
        "interpolation-quality-request-corpus-summary.txt",
        "comparison-corpus-release-guard-summary.txt",
        "comparison-corpus-guard-summary.txt",
        "reference-holdout-overlap-summary.txt",
        "reference-snapshot-bridge-day-summary.txt",
        "reference-snapshot-major-body-boundary-window-summary.txt",
        "reference-snapshot-boundary-epoch-coverage-summary.txt",
        "reference-snapshot-pre-bridge-boundary-summary.txt",
        "reference-snapshot-2451917-major-body-boundary-summary.txt",
        "reference-snapshot-2451918-major-body-boundary-summary.txt",
        "reference-snapshot-2451919-major-body-boundary-summary.txt",
        "reference-snapshot-2451916-major-body-dense-boundary-summary.txt",
        "reference-snapshot-sparse-boundary-summary.txt",
        "reference-snapshot-exact-j2000-evidence-summary.txt",
        "reference-snapshot-source-summary.txt",
        "reference-snapshot-source-window-summary.txt",
        "reference-snapshot-manifest-summary.txt",
        "reference-snapshot-body-class-coverage-summary.txt",
        "reference-snapshot-equatorial-parity-summary.txt",
        "reference-asteroid-source-window-summary.txt",
        "reference-asteroid-equatorial-evidence-summary.txt",
        "independent-holdout-source-window-summary.txt",
        "independent-holdout-equatorial-parity-summary.txt",
        "independent-holdout-body-class-coverage-summary.txt",
        "independent-holdout-quarter-day-boundary-summary.txt",
        "production-generation-boundary-source-summary.txt",
        "production-generation-boundary-window-summary.txt",
        "production-generation-boundary-request-corpus-summary.txt",
        "production-generation-boundary-request-corpus-equatorial-summary.txt",
        "production-generation-summary.txt",
        "production-generation-body-class-coverage-summary.txt",
        "production-generation-source-summary.txt",
        "production-generation-source-revision-summary.txt",
        "production-generation-source-window-summary.txt",
        "production-generation-quarter-day-boundary-summary.txt",
        "production-generation-corpus-shape-summary.txt",
        "production-generation-manifest-summary.txt",
        "production-generation-manifest-checksum-summary.txt",
        "reference-snapshot-summary.txt",
        "catalog-inventory-summary.txt",
        "catalog-posture-summary.txt",
        "custom-definition-ayanamsa-labels-summary.txt",
        "ayanamsa-provenance-summary.txt",
        "validation-report-summary.txt",
        "release-body-claims-summary.txt",
        "body-date-channel-claims-summary.txt",
        "pluto-fallback-summary.txt",
        "request-policy-summary.txt",
        "observer-policy-summary.txt",
        "apparentness-policy-summary.txt",
        "request-semantics-summary.txt",
        "unsupported-modes-summary.txt",
        "time-scale-policy-summary.txt",
        "utc-convenience-policy-summary.txt",
        "delta-t-policy-summary.txt",
        "zodiac-policy-summary.txt",
        "native-sidereal-policy-summary.txt",
        "lunar-theory-limitations-summary.txt",
        "lunar-theory-source-selection-summary.txt",
        "lunar-theory-source-family-summary.txt",
        "lunar-source-window-summary.txt",
        "lunar-theory-catalog-validation-summary.txt",
        "request-surface-summary.txt",
        "compatibility-caveats-summary.txt",
        "workspace-provenance-summary.txt",
        "workspace-audit-summary.txt",
        "native-dependency-audit-summary.txt",
        "artifact-summary.txt",
        "packaged-artifact.bin",
        "packaged-artifact.checksum.txt",
        "packaged-artifact-profile-coverage-summary.txt",
        "packaged-artifact-access-summary.txt",
        "packaged-artifact-output-support-summary.txt",
        "packaged-artifact-fit-sample-classes-summary.txt",
        "packaged-artifact-fit-threshold-violation-count-summary.txt",
        "packaged-artifact-fit-threshold-violations-summary.txt",
        "packaged-artifact-body-cadence-summary.txt",
        "packaged-artifact-body-class-span-cap-summary.txt",
        "packaged-artifact-normalized-intermediate-summary.txt",
        "packaged-artifact-speed-policy-summary.txt",
        "packaged-artifact-storage-summary.txt",
        "packaged-artifact-production-profile-summary.txt",
        "packaged-frame-treatment-summary.txt",
        "packaged-artifact-target-threshold-summary.txt",
        "packaged-artifact-target-threshold-state-summary.txt",
        "packaged-artifact-source-fit-holdout-sync-summary.txt",
        "packaged-artifact-target-threshold-scope-envelopes-summary.txt",
        "packaged-artifact-phase2-corpus-alignment-summary.txt",
        "packaged-lookup-epoch-policy-summary.txt",
        "packaged-artifact-generation-policy-summary.txt",
        "packaged-artifact-generation-residual-bodies-summary.txt",
        "packaged-artifact-regeneration-summary.txt",
        "packaged-artifact-generation-manifest.txt",
        "packaged-artifact-generation-manifest-summary.txt",
        "packaged-artifact-generation-manifest-checksum-summary.txt",
        "packaged-artifact-generation-manifest.checksum.txt",
        "benchmark-report.txt",
        "validation-report.txt",
        "lunar-reference-error-envelope-summary.txt",
        "lunar-equatorial-reference-error-envelope-summary.txt",
        "lunar-apparent-comparison-summary.txt",
        "bundle-manifest.txt",
        "bundle-manifest.checksum.txt",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    let mut actual_entries = BTreeSet::new();
    for entry in fs::read_dir(output_dir)? {
        actual_entries.insert(entry?.file_name().to_string_lossy().into_owned());
    }

    if actual_entries != expected_entries {
        let unexpected = actual_entries
            .difference(&expected_entries)
            .cloned()
            .collect::<Vec<_>>();
        let missing = expected_entries
            .difference(&actual_entries)
            .cloned()
            .collect::<Vec<_>>();
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release bundle directory contents: unexpected [{}], missing [{}]",
            unexpected.join(", "),
            missing.join(", ")
        )));
    }

    Ok(())
}

pub(crate) fn ensure_release_bundle_manifest_is_canonical(
    manifest_text: &str,
) -> Result<(), ReleaseBundleError> {
    const EXPECTED_MANIFEST_LINES: [&str; 288] = [
        "Release bundle manifest",
        "profile:",
        "profile checksum (fnv1a-64):",
        "profile summary:",
        "profile summary checksum (fnv1a-64):",
        "release notes:",
        "release notes checksum (fnv1a-64):",
        "release notes summary:",
        "release notes summary checksum (fnv1a-64):",
        "release summary:",
        "release summary checksum (fnv1a-64):",
        "release-profile identifiers:",
        "release-profile identifiers checksum (fnv1a-64):",
        "release-profile identifiers summary:",
        "release-profile identifiers summary checksum (fnv1a-64):",
        "release-house-system-canonical-names summary:",
        "release-house-system-canonical-names summary checksum (fnv1a-64):",
        "release-ayanamsa-canonical-names summary:",
        "release-ayanamsa-canonical-names summary checksum (fnv1a-64):",
        "release-house-validation summary:",
        "release-house-validation summary checksum (fnv1a-64):",
        "target-house-scope summary:",
        "target-house-scope summary checksum (fnv1a-64):",
        "target-ayanamsa-scope summary:",
        "target-ayanamsa-scope summary checksum (fnv1a-64):",
        "house code aliases summary:",
        "house code aliases summary checksum (fnv1a-64):",
        "house formula families summary:",
        "house formula families summary checksum (fnv1a-64):",
        "house latitude-sensitive summary:",
        "house latitude-sensitive summary checksum (fnv1a-64):",
        "house latitude-sensitive constraints summary:",
        "house latitude-sensitive constraints summary checksum (fnv1a-64):",
        "house latitude-sensitive failure-modes summary:",
        "house latitude-sensitive failure-modes summary checksum (fnv1a-64):",
        "release checklist:",
        "release checklist checksum (fnv1a-64):",
        "release checklist summary:",
        "release checklist summary checksum (fnv1a-64):",
        "backend matrix:",
        "backend matrix checksum (fnv1a-64):",
        "backend matrix summary:",
        "backend matrix summary checksum (fnv1a-64):",
        "api stability posture:",
        "api stability checksum (fnv1a-64):",
        "api stability summary:",
        "api stability summary checksum (fnv1a-64):",
        "comparison-corpus summary:",
        "comparison-corpus summary checksum (fnv1a-64):",
        "source-corpus summary:",
        "source-corpus summary checksum (fnv1a-64):",
        "jpl source posture summary:",
        "jpl source posture summary checksum (fnv1a-64):",
        "jpl provenance-only evidence summary:",
        "jpl provenance-only evidence summary checksum (fnv1a-64):",
        "comparison-snapshot summary:",
        "comparison-snapshot summary checksum (fnv1a-64):",
        "comparison-snapshot source summary:",
        "comparison-snapshot source summary checksum (fnv1a-64):",
        "comparison-snapshot source window summary:",
        "comparison-snapshot source window summary checksum (fnv1a-64):",
        "comparison-snapshot body-class coverage summary:",
        "comparison-snapshot body-class coverage summary checksum (fnv1a-64):",
        "comparison-snapshot manifest summary:",
        "comparison-snapshot manifest summary checksum (fnv1a-64):",
        "comparison-envelope summary:",
        "comparison-envelope summary checksum (fnv1a-64):",
        "comparison-body-class-tolerance summary:",
        "comparison-body-class-tolerance summary checksum (fnv1a-64):",
        "comparison-body-class-error-envelope summary:",
        "comparison-body-class-error-envelope summary checksum (fnv1a-64):",
        "comparison-corpus release-guard summary:",
        "comparison-corpus release-guard summary checksum (fnv1a-64):",
        "reference-holdout overlap summary:",
        "reference-holdout overlap summary checksum (fnv1a-64):",
        "reference snapshot bridge day summary:",
        "reference snapshot bridge day summary checksum (fnv1a-64):",
        "reference snapshot major-body boundary window summary:",
        "reference snapshot major-body boundary window summary checksum (fnv1a-64):",
        "reference snapshot boundary epoch coverage summary:",
        "reference snapshot boundary epoch coverage summary checksum (fnv1a-64):",
        "reference snapshot pre-bridge boundary summary:",
        "reference snapshot pre-bridge boundary summary checksum (fnv1a-64):",
        "reference snapshot 2451917 major-body boundary summary:",
        "reference snapshot 2451917 major-body boundary summary checksum (fnv1a-64):",
        "reference snapshot 2451918 major-body boundary summary:",
        "reference snapshot 2451918 major-body boundary summary checksum (fnv1a-64):",
        "reference snapshot 2451919 major-body boundary summary:",
        "reference snapshot 2451919 major-body boundary summary checksum (fnv1a-64):",
        "reference snapshot 2451916 major-body dense boundary summary:",
        "reference snapshot 2451916 major-body dense boundary summary checksum (fnv1a-64):",
        "reference snapshot sparse boundary summary:",
        "reference snapshot sparse boundary summary checksum (fnv1a-64):",
        "reference snapshot exact J2000 evidence summary:",
        "reference snapshot exact J2000 evidence summary checksum (fnv1a-64):",
        "reference snapshot source summary:",
        "reference snapshot source summary checksum (fnv1a-64):",
        "reference snapshot source window summary:",
        "reference snapshot source window summary checksum (fnv1a-64):",
        "reference snapshot manifest summary:",
        "reference snapshot manifest summary checksum (fnv1a-64):",
        "reference snapshot body-class coverage summary:",
        "reference snapshot body-class coverage summary checksum (fnv1a-64):",
        "reference snapshot equatorial parity summary:",
        "reference snapshot equatorial parity summary checksum (fnv1a-64):",
        "reference asteroid source window summary:",
        "reference asteroid source window summary checksum (fnv1a-64):",
        "reference asteroid equatorial evidence summary:",
        "reference asteroid equatorial evidence summary checksum (fnv1a-64):",
        "independent-holdout source window summary:",
        "independent-holdout source window summary checksum (fnv1a-64):",
        "independent-holdout equatorial parity summary:",
        "independent-holdout equatorial parity summary checksum (fnv1a-64):",
        "independent-holdout body-class coverage summary:",
        "independent-holdout body-class coverage summary checksum (fnv1a-64):",
        "independent-holdout quarter-day boundary summary:",
        "independent-holdout quarter-day boundary summary checksum (fnv1a-64):",
        "production generation boundary source summary:",
        "production generation boundary source summary checksum (fnv1a-64):",
        "production generation boundary window summary:",
        "production generation boundary window summary checksum (fnv1a-64):",
        "production generation boundary request corpus summary:",
        "production generation boundary request corpus summary checksum (fnv1a-64):",
        "production generation boundary request corpus equatorial summary:",
        "production generation boundary request corpus equatorial summary checksum (fnv1a-64):",
        "reference snapshot summary:",
        "reference snapshot summary checksum (fnv1a-64):",
        "production generation summary:",
        "production generation summary checksum (fnv1a-64):",
        "production generation body-class coverage summary:",
        "production generation body-class coverage summary checksum (fnv1a-64):",
        "production generation source summary:",
        "production generation source summary checksum (fnv1a-64):",
        "production generation source revision summary:",
        "production generation source revision summary checksum (fnv1a-64):",
        "production generation source window summary:",
        "production generation source window summary checksum (fnv1a-64):",
        "production generation quarter-day boundary summary:",
        "production generation quarter-day boundary summary checksum (fnv1a-64):",
        "production generation corpus shape summary:",
        "production generation corpus shape summary checksum (fnv1a-64):",
        "production generation manifest summary:",
        "production generation manifest summary checksum (fnv1a-64):",
        "production generation manifest checksum summary:",
        "production generation manifest checksum summary checksum (fnv1a-64):",
        "catalog inventory summary:",
        "catalog inventory summary checksum (fnv1a-64):",
        "catalog posture summary:",
        "catalog posture summary checksum (fnv1a-64):",
        "custom-definition ayanamsa labels summary:",
        "custom-definition ayanamsa labels summary checksum (fnv1a-64):",
        "ayanamsa provenance summary:",
        "ayanamsa provenance summary checksum (fnv1a-64):",
        "validation report summary:",
        "validation report summary checksum (fnv1a-64):",
        "workspace provenance summary:",
        "workspace provenance summary checksum (fnv1a-64):",
        "release body claims summary:",
        "release body claims summary checksum (fnv1a-64):",
        "body/date/channel claims summary:",
        "body/date/channel claims summary checksum (fnv1a-64):",
        "pluto fallback summary:",
        "pluto fallback summary checksum (fnv1a-64):",
        "request policy summary:",
        "request policy summary checksum (fnv1a-64):",
        "observer policy summary:",
        "observer policy summary checksum (fnv1a-64):",
        "apparentness policy summary:",
        "apparentness policy summary checksum (fnv1a-64):",
        "request-semantics summary:",
        "request-semantics summary checksum (fnv1a-64):",
        "unsupported-modes summary:",
        "unsupported-modes summary checksum (fnv1a-64):",
        "time-scale policy summary:",
        "time-scale policy summary checksum (fnv1a-64):",
        "utc-convenience policy summary:",
        "utc-convenience policy summary checksum (fnv1a-64):",
        "delta-t policy summary:",
        "delta-t policy summary checksum (fnv1a-64):",
        "native sidereal policy summary:",
        "native sidereal policy summary checksum (fnv1a-64):",
        "zodiac policy summary:",
        "zodiac policy summary checksum (fnv1a-64):",
        "lunar theory limitations summary:",
        "lunar theory limitations summary checksum (fnv1a-64):",
        "lunar theory source selection summary:",
        "lunar theory source selection summary checksum (fnv1a-64):",
        "lunar theory source family summary:",
        "lunar theory source family summary checksum (fnv1a-64):",
        "lunar theory source window summary:",
        "lunar theory source window summary checksum (fnv1a-64):",
        "lunar reference error envelope summary:",
        "lunar reference error envelope summary checksum (fnv1a-64):",
        "lunar equatorial reference error envelope summary:",
        "lunar equatorial reference error envelope summary checksum (fnv1a-64):",
        "lunar apparent comparison summary:",
        "lunar apparent comparison summary checksum (fnv1a-64):",
        "lunar theory catalog validation summary:",
        "lunar theory catalog validation summary checksum (fnv1a-64):",
        "request surface summary:",
        "request surface summary checksum (fnv1a-64):",
        "compatibility caveats summary:",
        "compatibility caveats summary checksum (fnv1a-64):",
        "workspace audit summary:",
        "workspace audit summary checksum (fnv1a-64):",
        "native-dependency audit summary:",
        "native-dependency audit summary checksum (fnv1a-64):",
        "artifact summary:",
        "artifact summary checksum (fnv1a-64):",
        "packaged-artifact:",
        "packaged-artifact checksum (fnv1a-64):",
        "packaged-artifact checksum sidecar:",
        "packaged-artifact checksum sidecar checksum (fnv1a-64):",
        "packaged-artifact profile coverage summary:",
        "packaged-artifact profile coverage summary checksum (fnv1a-64):",
        "packaged-artifact access summary:",
        "packaged-artifact access summary checksum (fnv1a-64):",
        "packaged-artifact output support summary:",
        "packaged-artifact output support summary checksum (fnv1a-64):",
        "packaged-artifact fit sample classes summary:",
        "packaged-artifact fit sample classes summary checksum (fnv1a-64):",
        "packaged-artifact fit threshold violation count summary:",
        "packaged-artifact fit threshold violation count summary checksum (fnv1a-64):",
        "packaged-artifact fit threshold violations summary:",
        "packaged-artifact fit threshold violations summary checksum (fnv1a-64):",
        "packaged-artifact normalized intermediate summary:",
        "packaged-artifact normalized intermediate summary checksum (fnv1a-64):",
        "packaged-artifact speed policy summary:",
        "packaged-artifact speed policy summary checksum (fnv1a-64):",
        "packaged-artifact storage summary:",
        "packaged-artifact storage summary checksum (fnv1a-64):",
        "packaged-artifact production-profile summary:",
        "packaged-artifact production-profile summary checksum (fnv1a-64):",
        "packaged-frame-treatment summary:",
        "packaged-frame-treatment summary checksum (fnv1a-64):",
        "packaged-artifact target-threshold summary:",
        "packaged-artifact target-threshold summary checksum (fnv1a-64):",
        "packaged-artifact target-threshold state summary:",
        "packaged-artifact target-threshold state summary checksum (fnv1a-64):",
        "packaged-artifact source-fit and hold-out sync summary:",
        "packaged-artifact source-fit and hold-out sync summary checksum (fnv1a-64):",
        "packaged-artifact target-threshold scope envelopes summary:",
        "packaged-artifact target-threshold scope envelopes summary checksum (fnv1a-64):",
        "packaged-artifact phase-2 corpus alignment summary:",
        "packaged-artifact phase-2 corpus alignment summary checksum (fnv1a-64):",
        "packaged-artifact lookup-epoch policy summary:",
        "packaged-artifact lookup-epoch policy summary checksum (fnv1a-64):",
        "packaged-artifact generation policy summary:",
        "packaged-artifact generation policy summary checksum (fnv1a-64):",
        "packaged-artifact generation residual bodies summary:",
        "packaged-artifact generation residual bodies summary checksum (fnv1a-64):",
        "packaged-artifact regeneration summary:",
        "packaged-artifact regeneration summary checksum (fnv1a-64):",
        "packaged-artifact generation manifest:",
        "packaged-artifact generation manifest checksum (fnv1a-64):",
        "packaged-artifact generation manifest checksum sidecar:",
        "packaged-artifact generation manifest checksum sidecar checksum (fnv1a-64):",
        "packaged-artifact generation manifest summary:",
        "packaged-artifact generation manifest summary checksum (fnv1a-64):",
        "packaged-artifact generation manifest checksum summary:",
        "packaged-artifact generation manifest checksum summary checksum (fnv1a-64):",
        "benchmark-corpus summary:",
        "benchmark-corpus summary checksum (fnv1a-64):",
        "chart-benchmark-corpus summary:",
        "chart-benchmark-corpus summary checksum (fnv1a-64):",
        "selected asteroid source request corpus summary:",
        "selected asteroid source request corpus summary checksum (fnv1a-64):",
        "selected asteroid source request corpus equatorial summary:",
        "selected asteroid source request corpus equatorial summary checksum (fnv1a-64):",
        "selected asteroid source window summary:",
        "selected asteroid source window summary checksum (fnv1a-64):",
        "interpolation-quality sample request corpus summary:",
        "interpolation-quality sample request corpus summary checksum (fnv1a-64):",
        "benchmark report:",
        "benchmark report checksum (fnv1a-64):",
        "validation report:",
        "validation report checksum (fnv1a-64):",
        "source revision:",
        "workspace status:",
        "rustc version:",
        "cargo version:",
        "profile id:",
        "api stability posture id:",
        "validation rounds:",
        "packaged-artifact body cadence summary:",
        "packaged-artifact body cadence summary checksum (fnv1a-64):",
        "packaged-artifact body-class span cap summary:",
        "packaged-artifact body-class span cap summary checksum (fnv1a-64):",
    ];

    let lines = manifest_text.lines().collect::<Vec<_>>();
    if lines.len() != EXPECTED_MANIFEST_LINES.len() {
        return Err(ReleaseBundleError::Verification(format!(
            "unexpected release bundle manifest line count: expected {}, found {}",
            EXPECTED_MANIFEST_LINES.len(),
            lines.len()
        )));
    }

    for (index, (line, expected_prefix)) in lines.iter().zip(EXPECTED_MANIFEST_LINES).enumerate() {
        if !line.starts_with(expected_prefix) {
            return Err(ReleaseBundleError::Verification(format!(
                "unexpected release bundle manifest line {}: expected prefix `{}`, found `{}`",
                index + 1,
                expected_prefix,
                line
            )));
        }
    }

    Ok(())
}

pub(crate) fn ensure_release_bundle_regular_file(
    path: &Path,
    label: &str,
) -> Result<(), ReleaseBundleError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(()),
        Ok(_) => Err(ReleaseBundleError::Verification(format!(
            "unexpected non-regular {label} file: {}",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ReleaseBundleError::Io(error)),
    }
}

pub(crate) fn read_required_bundle_text(
    path: &Path,
    label: &str,
) -> Result<String, ReleaseBundleError> {
    fs::read_to_string(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            ReleaseBundleError::Verification(format!("missing {label} file: {}", path.display()))
        } else {
            ReleaseBundleError::Io(error)
        }
    })
}

pub(crate) fn extract_single_summary_payload<'a>(
    text: &'a str,
    prefix: &str,
) -> Result<&'a str, ReleaseBundleError> {
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
