//! Validation, comparison, and benchmarking helpers for the workspace.
//!
//! The validation crate compares the algorithmic chart backends against the
//! checked-in JPL Horizons snapshot corpus and renders reproducible reports for
//! stage-4 work.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Instant as StdInstant;

mod artifact;
mod chart_benchmark;
mod comparison;
mod compatibility;
mod corpus;
mod house_validation;
mod provenance;
mod release;
mod render;
mod report;

pub use render::{banner, render_cli};

#[cfg(test)]
pub(crate) use comparison::body_class::BodyClassSummaryValidationError;
use comparison::validate_comparison_tolerance;
use comparison::{
    body_class, body_class_summaries, body_class_tolerance_summaries, comparison_audit_summary,
    comparison_audit_summary_for_report, comparison_audit_totals, comparison_tolerance_for_body,
    comparison_tolerance_policy_entries, comparison_tolerance_scope_for_body,
    format_regression_bodies, format_summary_body, BodyClass, BodyClassSummary,
    BodyClassToleranceSummary,
};
pub use comparison::{
    compare_backends, comparison_tolerance_catalog_entries, default_candidate_backend,
    default_reference_backend, BodyComparisonSummary, ComparisonAuditSummary, ComparisonReport,
    ComparisonSample, ComparisonSummary, ComparisonTolerance, ComparisonToleranceEntry,
    ComparisonTolerancePolicySummary, ComparisonToleranceScope,
    ComparisonToleranceScopeCoverageSummary, RegressionArchive, RegressionFinding,
};
pub use compatibility::{
    compatibility_profile_verification_summary, verify_compatibility_profile,
    CompatibilityProfileVerificationSummary,
};
#[cfg(test)]
pub(crate) use compatibility::{
    ensure_profile_descriptor_metadata, ensure_unique_profile_label,
    is_intentional_custom_definition_ayanamsa_homograph, verify_ayanamsa_aliases,
    verify_custom_definition_labels, verify_house_system_aliases,
    verify_profile_catalog_partitions_are_disjoint, verify_profile_text_section,
    verify_profile_text_sections_are_disjoint, INTENTIONAL_CUSTOM_DEFINITION_AYANAMSA_HOMOGRAPHS,
};
use compatibility::{has_surrounding_whitespace, summarize_validation_reference_points};
use corpus::benchmark_timing_corpus;
pub use corpus::{
    benchmark_corpus, default_corpus, release_grade_corpus, CorpusSummary, ValidationCorpus,
};
pub use provenance::{
    benchmark_provenance_text, workspace_provenance, workspace_provenance_summary_for_report,
    WorkspaceProvenance, WorkspaceProvenanceValidationError,
};
#[cfg(test)]
pub(crate) use release::bundle_verify_helpers::{
    ensure_backend_matrix_report_matches_current_rendering,
    ensure_backend_matrix_summary_matches_current_rendering,
    ensure_benchmark_corpus_summary_matches_current_rendering, ensure_catalog_inventory_alignment,
    ensure_chart_benchmark_corpus_summary_matches_current_rendering,
    ensure_comparison_snapshot_source_summary_matches_current_rendering,
    ensure_custom_definition_ayanamsa_labels_alignment,
    ensure_independent_holdout_source_window_summary_matches_current_rendering,
    ensure_lunar_theory_catalog_validation_summary_matches_current_rendering,
    ensure_packaged_artifact_generation_manifest_summary_matches_current_rendering,
    ensure_packaged_artifact_normalized_intermediate_summary_matches_current_rendering,
    ensure_packaged_artifact_phase2_alignment_matches_source_fit_holdout_sync,
    ensure_packaged_artifact_phase2_corpus_alignment_summary_matches_current_rendering,
    ensure_packaged_artifact_source_fit_holdout_sync_summary_matches_current_rendering,
    ensure_packaged_artifact_target_threshold_phase2_alignment_matches_source_fit_holdout_sync,
    ensure_packaged_artifact_target_threshold_summary_matches_current_rendering,
    ensure_production_generation_source_summary_matches_source_windows,
    ensure_production_generation_source_window_summary_matches_current_rendering,
    ensure_reference_snapshot_bridge_day_summary_matches_current_rendering,
    ensure_reference_snapshot_manifest_summary_matches_current_rendering,
    ensure_reference_snapshot_source_summary_matches_current_rendering,
    ensure_reference_snapshot_source_window_summary_matches_current_rendering,
    ensure_release_house_validation_summary_matches_current_rendering,
    ensure_release_notes_summary_matches_current_rendering,
    ensure_release_profile_identifiers_alignment,
    ensure_release_profile_identifiers_summary_matches_current_rendering,
    ensure_release_profile_line_alignment, ensure_release_profile_summary_alignment,
    ensure_request_policy_summary_matches_current_rendering,
    ensure_request_semantics_summary_matches_current_rendering,
    ensure_request_surface_summary_matches_current_rendering,
};
#[cfg(test)]
pub(crate) use release::workspace_audit::{
    audit_build_script_path, audit_lockfile_text, audit_manifest_text,
    audit_publishable_crate_files, audit_publishable_manifest_text, audit_tool_manifest_text,
    audit_workspace_manifest_publish_text, manifest_declares_publish_false, manifest_is_package,
    manifest_package_name,
};
use release::{
    release_checklist_bundle_contents, release_checklist_external_publishing_reminders,
    release_checklist_manual_bundle_workflow, release_checklist_repository_managed_release_gates,
    render_release_checklist_summary_text, render_release_checklist_text,
    render_release_notes_summary_text, render_release_notes_text, render_release_smoke_text,
    render_release_summary_text, render_workspace_audit_summary_text,
    validated_lunar_theory_catalog_validation_summary_for_report, verify_release_bundle,
};
pub use release::{
    release_checklist_summary, render_release_bundle, workspace_audit_report,
    workspace_audit_summary, ReleaseBundle, ReleaseBundleError, ReleaseChecklistSummary,
    WorkspaceAuditReport, WorkspaceAuditSummary, WorkspaceAuditViolation,
};
#[cfg(test)]
pub(crate) use render::cli::validate_release_gate_at;
pub(crate) use render::summary::*;
pub use render::summary::{
    current_request_surface_summary, render_api_stability_summary, render_backend_matrix_report,
    render_backend_matrix_summary, render_release_profile_identifiers_summary,
    RequestSurfaceSummary,
};
pub(crate) use render::text::*;
pub use render::text::{
    benchmark_backend, comparison_envelope_summary, comparison_median_envelope,
    comparison_tail_envelope, mean_obliquity_frame_round_trip_sample_corpus,
    mean_obliquity_frame_round_trip_summary,
    packaged_artifact_fit_sample_classes_summary_for_report, render_ayanamsa_audit_summary,
    render_benchmark_matrix_summary, render_benchmark_report, render_catalog_inventory_summary,
    render_catalog_posture_summary, render_comparison_audit_report,
    render_comparison_audit_summary, render_comparison_report,
    render_compatibility_caveats_summary, render_compatibility_profile_summary,
    render_custom_definition_ayanamsa_labels_summary, render_delta_t_policy_summary,
    render_house_latitude_sensitive_failure_modes_summary, render_known_gaps_summary,
    render_native_dependency_audit_summary, render_release_ayanamsa_canonical_names_summary,
    render_release_checklist, render_release_checklist_summary,
    render_release_house_system_canonical_names_summary, render_release_notes,
    render_release_notes_summary, render_release_summary, render_request_policy_summary,
    render_request_surface_summary, render_target_ayanamsa_scope_summary,
    render_target_house_scope_summary, render_validation_report, render_validation_report_summary,
    render_workspace_audit_summary, BodyToleranceSummary, ComparisonEnvelopeSummary,
    ComparisonMedianEnvelope, ComparisonPercentileEnvelope, MeanObliquityFrameRoundTripSummary,
};
pub use report::{BenchmarkReport, ValidationReport};

use artifact::ArtifactBoundaryEnvelopeSummary;
pub use artifact::{
    artifact_boundary_envelope_summary_for_report, artifact_inspection_summary_for_report,
    render_artifact_report, render_artifact_summary, ArtifactBatchLookupBenchmarkReport,
    ArtifactBatchLookupBenchmarkReportValidationError, ArtifactBodyInspection,
    ArtifactDecodeBenchmarkReport, ArtifactDecodeBenchmarkReportValidationError,
    ArtifactInspectionReport, ArtifactLookupBenchmarkReport,
    ArtifactLookupBenchmarkReportValidationError,
};
pub use chart_benchmark::{
    benchmark_chart_backend, chart_benchmark_corpus_summary, ChartBenchmarkReport,
};
pub use house_validation::{
    house_validation_report, house_validation_summary_for_report,
    house_validation_summary_line_for_report, release_house_validation_report,
    release_house_validation_summary_for_report,
    validated_house_validation_summary_line_for_report, HouseValidationReport,
    HouseValidationReportValidationError, HouseValidationSample, HouseValidationScenario,
};

use pleiades_ayanamsa::{
    ayanamsa_catalog_validation_summary, baseline_ayanamsas, built_in_ayanamsas, descriptor,
    metadata_coverage, release_ayanamsas, resolve_ayanamsa, validate_ayanamsa_catalog,
};
use pleiades_backend::{
    delta_t_policy_summary_for_report, frame_policy_summary_details,
    pluto_fallback_summary_for_report, release_body_claims_summary_for_report,
    request_policy_summary_for_report, time_scale_policy_summary_for_report,
    unsupported_modes_summary_for_report,
    validate_release_body_claims_posture as validate_release_body_claims_posture_backend,
    validated_frame_policy_summary_for_report, validated_pluto_fallback_summary_line_for_report,
    validated_release_body_claims_summary_line_for_report,
    validated_zodiac_policy_summary_for_report,
};
#[cfg(test)]
use pleiades_core::default_chart_bodies;
use pleiades_core::{
    catalog_posture_summary_for_report as core_catalog_posture_summary_for_report,
    compatibility_caveats_summary_for_report as core_compatibility_caveats_summary_for_report,
    current_api_stability_profile, current_compatibility_profile,
    current_release_profile_identifiers, validate_custom_definition_labels,
    validated_catalog_inventory_summary_for_report as core_validated_catalog_inventory_summary_for_report,
    validated_catalog_posture_summary_for_report as core_validated_catalog_posture_summary_for_report,
    validated_custom_definition_ayanamsa_labels_summary_for_report,
    validated_house_code_aliases_summary_for_report as core_validated_house_code_aliases_summary_for_report,
    validated_house_formula_families_summary_for_report, validated_known_gaps_summary_for_report,
    validated_latitude_sensitive_house_constraints_summary_for_report,
    validated_latitude_sensitive_house_failure_modes_summary_for_report,
    validated_latitude_sensitive_house_systems_summary_for_report,
    validated_release_ayanamsa_canonical_names_summary_for_report as core_validated_release_ayanamsa_canonical_names_summary_for_report,
    validated_release_house_system_canonical_names_summary_for_report as core_validated_release_house_system_canonical_names_summary_for_report,
    validated_release_profile_identifiers_summary_for_report as core_validated_release_profile_identifiers_summary_for_report,
    validated_target_ayanamsa_scope_summary_for_report as core_validated_target_ayanamsa_scope_summary_for_report,
    validated_target_house_scope_summary_for_report as core_validated_target_house_scope_summary_for_report,
    AccuracyClass, Angle, Apparentness, BackendCapabilities, BackendFamily, BackendMetadata,
    CelestialBody, CompatibilityProfile, CompositeBackend, CoordinateFrame, EclipticCoordinates,
    EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisResult, Instant, JulianDay,
    Longitude, ReleaseProfileIdentifiers, TimeRange, TimeScale,
};
use pleiades_data::{
    packaged_artifact, packaged_artifact_access_summary_for_report,
    packaged_artifact_body_class_span_cap_entries_for_report, packaged_artifact_bytes,
    packaged_artifact_fit_envelope_summary_details,
    packaged_artifact_fit_envelope_summary_for_report,
    packaged_artifact_fit_margin_summary_for_report,
    packaged_artifact_fit_outlier_summary_for_report,
    packaged_artifact_fit_threshold_summary_details,
    packaged_artifact_fit_threshold_summary_for_report,
    packaged_artifact_fit_threshold_violation_count_for_report,
    packaged_artifact_fit_threshold_violation_summary_for_report,
    packaged_artifact_generation_manifest_checksum_for_report,
    packaged_artifact_generation_manifest_for_report,
    packaged_artifact_generation_policy_summary_for_report,
    packaged_artifact_normalized_intermediate_summary_for_report,
    packaged_artifact_output_support_summary_for_report,
    packaged_artifact_profile_coverage_summary_for_report,
    packaged_artifact_profile_summary_with_body_coverage,
    packaged_artifact_regeneration_summary_for_report,
    packaged_artifact_speed_policy_summary_for_report,
    packaged_artifact_target_threshold_summary_details, packaged_frame_parity_summary_for_report,
    packaged_frame_treatment_summary_for_report, packaged_lookup_epoch_policy_summary_for_report,
    packaged_mixed_tt_tdb_batch_parity_summary_for_report,
    packaged_request_policy_summary_for_report, PackagedDataBackend,
};
use pleiades_elp::{
    lunar_apparent_comparison_evidence, lunar_apparent_comparison_summary,
    lunar_apparent_comparison_summary_for_report,
    lunar_equatorial_reference_batch_parity_summary_for_report,
    lunar_equatorial_reference_evidence, lunar_equatorial_reference_evidence_envelope_for_report,
    lunar_equatorial_reference_evidence_summary,
    lunar_equatorial_reference_evidence_summary_for_report,
    lunar_high_curvature_continuity_evidence_for_report,
    lunar_high_curvature_equatorial_continuity_evidence_for_report,
    lunar_reference_batch_parity_summary_for_report, lunar_reference_evidence,
    lunar_reference_evidence_envelope_for_report, lunar_reference_evidence_summary,
    lunar_reference_evidence_summary_for_report, lunar_source_window_summary_for_report,
    lunar_theory_capability_summary_for_report, lunar_theory_catalog_summary_for_report,
    lunar_theory_frame_treatment_summary_for_report, lunar_theory_limitations_summary_for_report,
    lunar_theory_request_policy_summary, lunar_theory_source_summary_for_report,
    lunar_theory_specification, lunar_theory_summary_for_report,
    validated_lunar_source_window_summary_for_report, ElpBackend,
};
use pleiades_houses::{
    baseline_house_systems, built_in_house_systems, release_house_systems, resolve_house_system,
    validate_house_catalog,
};

#[cfg(test)]
use pleiades_jpl::{
    production_generation_manifest_summary_for_report,
    production_generation_snapshot_body_class_coverage_summary_for_report,
};

use pleiades_jpl::{
    comparison_snapshot_body_class_coverage_summary_for_report,
    comparison_snapshot_source_summary_for_report,
    comparison_snapshot_source_window_summary_for_report, comparison_snapshot_summary_for_report,
    format_jpl_interpolation_quality_summary_for_report,
    frame_treatment_summary_for_report as jpl_frame_treatment_summary_for_report,
    independent_holdout_high_curvature_summary_for_report,
    independent_holdout_manifest_summary_for_report,
    independent_holdout_snapshot_body_class_coverage_summary_for_report,
    independent_holdout_snapshot_equatorial_parity_summary_for_report as jpl_independent_holdout_snapshot_equatorial_parity_summary_for_report,
    independent_holdout_snapshot_quarter_day_boundary_summary_for_report,
    independent_holdout_snapshot_source_window_summary_for_report,
    independent_holdout_source_summary_for_report,
    interpolation_quality_sample_request_corpus_summary_for_report, interpolation_quality_samples,
    jpl_independent_holdout_summary_for_report,
    jpl_interpolation_body_class_error_envelopes_for_report, jpl_interpolation_posture_summary,
    jpl_interpolation_posture_summary_for_report,
    jpl_interpolation_quality_kind_coverage_for_report, jpl_provenance_only_summary_for_report,
    jpl_snapshot_batch_error_taxonomy_summary_for_report,
    jpl_snapshot_evidence_classification_summary_for_report,
    jpl_snapshot_evidence_summary_for_report, jpl_snapshot_request_policy_summary_for_report,
    jpl_source_corpus_contract_summary_for_report, jpl_source_posture_summary_for_report,
    production_generation_boundary_body_class_coverage_summary_for_report,
    production_generation_boundary_request_corpus_equatorial_summary_for_report,
    production_generation_boundary_request_corpus_summary_for_report,
    production_generation_boundary_source_summary_for_report,
    production_generation_boundary_summary_for_report,
    production_generation_boundary_window_summary_for_report,
    production_generation_corpus_shape_summary_for_report,
    production_generation_manifest_checksum_for_report,
    production_generation_snapshot_summary_for_report,
    production_generation_snapshot_window_summary_for_report,
    production_generation_source_revision_summary_for_report,
    production_generation_source_summary_for_report,
    reference_asteroid_equatorial_evidence_summary_for_report, reference_asteroid_evidence,
    reference_asteroid_evidence_summary_for_report,
    reference_asteroid_source_window_summary_for_report, reference_asteroids,
    reference_snapshot_1500_selected_body_boundary_summary_for_report,
    reference_snapshot_1600_selected_body_boundary_summary_for_report,
    reference_snapshot_1749_major_body_boundary_summary_for_report,
    reference_snapshot_1750_major_body_interior_summary_for_report,
    reference_snapshot_1750_selected_body_boundary_summary_for_report,
    reference_snapshot_1800_major_body_boundary_summary_for_report,
    reference_snapshot_1900_selected_body_boundary_summary_for_report,
    reference_snapshot_2200_selected_body_boundary_summary_for_report,
    reference_snapshot_2268932_selected_body_boundary_summary_for_report,
    reference_snapshot_2305457_selected_body_boundary_summary_for_report,
    reference_snapshot_2360233_major_body_boundary_summary_for_report,
    reference_snapshot_2360234_major_body_interior_summary_for_report,
    reference_snapshot_2378498_major_body_boundary_summary_for_report,
    reference_snapshot_2378499_major_body_boundary_summary_for_report,
    reference_snapshot_2400000_major_body_boundary_summary_for_report,
    reference_snapshot_2415020_selected_body_boundary_summary_for_report,
    reference_snapshot_2451545_major_body_boundary_summary_for_report,
    reference_snapshot_2451910_major_body_boundary_summary_for_report,
    reference_snapshot_2451911_major_body_boundary_summary_for_report,
    reference_snapshot_2451912_major_body_boundary_summary_for_report,
    reference_snapshot_2451913_major_body_boundary_summary_for_report,
    reference_snapshot_2451914_bridge_day_summary_for_report,
    reference_snapshot_2451914_major_body_boundary_summary_for_report,
    reference_snapshot_2451914_major_body_bridge_day_summary_for_report,
    reference_snapshot_2451914_major_body_bridge_summary_for_report,
    reference_snapshot_2451914_major_body_pre_bridge_summary_for_report,
    reference_snapshot_2451915_major_body_boundary_summary_for_report,
    reference_snapshot_2451915_major_body_bridge_summary_for_report,
    reference_snapshot_2451916_major_body_boundary_summary_for_report,
    reference_snapshot_2451916_major_body_dense_boundary_summary_for_report,
    reference_snapshot_2451916_major_body_interior_summary_for_report,
    reference_snapshot_2451917_major_body_boundary_summary_for_report,
    reference_snapshot_2451917_major_body_bridge_summary_for_report,
    reference_snapshot_2451918_major_body_boundary_summary_for_report,
    reference_snapshot_2451919_major_body_boundary_summary_for_report,
    reference_snapshot_2451920_major_body_interior_summary_for_report,
    reference_snapshot_2453000_major_body_boundary_summary_for_report,
    reference_snapshot_2500000_major_body_boundary_summary_for_report,
    reference_snapshot_2500_major_body_boundary_summary_for_report,
    reference_snapshot_2500_selected_body_boundary_summary_for_report,
    reference_snapshot_2524593_selected_body_boundary_summary_for_report,
    reference_snapshot_2600000_major_body_boundary_summary_for_report,
    reference_snapshot_2634167_selected_body_boundary_summary_for_report,
    reference_snapshot_body_class_coverage_summary_for_report,
    reference_snapshot_boundary_epoch_coverage_summary_for_report,
    reference_snapshot_bridge_day_summary_for_report,
    reference_snapshot_dense_boundary_summary_for_report,
    reference_snapshot_early_major_body_boundary_summary_for_report,
    reference_snapshot_equatorial_parity_summary_for_report,
    reference_snapshot_exact_j2000_evidence_summary_for_report,
    reference_snapshot_high_curvature_epoch_coverage_summary_for_report,
    reference_snapshot_high_curvature_summary_for_report,
    reference_snapshot_high_curvature_window_summary_for_report,
    reference_snapshot_lunar_boundary_summary_for_report,
    reference_snapshot_major_body_boundary_summary_for_report,
    reference_snapshot_major_body_boundary_window_summary_for_report,
    reference_snapshot_major_body_bridge_summary_for_report,
    reference_snapshot_manifest_summary_for_report,
    reference_snapshot_mars_jupiter_boundary_summary_for_report,
    reference_snapshot_mars_outer_boundary_summary_for_report,
    reference_snapshot_pre_bridge_boundary_summary_for_report,
    reference_snapshot_source_summary_for_report,
    reference_snapshot_source_window_summary_for_report,
    reference_snapshot_sparse_boundary_summary_for_report, reference_snapshot_summary_for_report,
    selected_asteroid_batch_parity_summary_for_report,
    selected_asteroid_boundary_summary_for_report, selected_asteroid_bridge_summary_for_report,
    selected_asteroid_dense_boundary_summary_for_report,
    selected_asteroid_source_2378498_summary_for_report,
    selected_asteroid_source_2451917_summary_for_report,
    selected_asteroid_source_2453000_summary_for_report,
    selected_asteroid_source_2500000_summary_for_report,
    selected_asteroid_source_2634167_summary_for_report,
    selected_asteroid_source_evidence_summary_for_report,
    selected_asteroid_source_request_corpus_summary_for_report,
    selected_asteroid_source_window_summary_for_report,
    selected_asteroid_terminal_boundary_summary_for_report,
    validated_checked_in_snapshot_schema_summary_for_report,
    validated_comparison_snapshot_batch_parity_summary_for_report,
    validated_comparison_snapshot_body_class_coverage_summary_for_report,
    validated_comparison_snapshot_manifest_summary_for_report,
    validated_comparison_snapshot_source_summary_for_report,
    validated_comparison_snapshot_source_window_summary_for_report,
    validated_independent_holdout_snapshot_batch_parity_summary_for_report,
    validated_production_generation_corpus_shape_summary_for_report,
    validated_production_generation_manifest_summary_for_report,
    validated_production_generation_snapshot_body_class_coverage_summary_for_report,
    validated_production_generation_source_revision_summary_for_report,
    validated_production_generation_source_summary_for_report,
    validated_reference_asteroid_source_window_summary_for_report,
    validated_reference_holdout_overlap_summary_for_report,
    validated_reference_snapshot_batch_parity_summary_for_report,
    validated_reference_snapshot_mixed_time_scale_batch_parity_summary_for_report,
    validated_selected_asteroid_source_evidence_summary_for_report,
    validated_selected_asteroid_source_request_corpus_equatorial_summary_for_report,
    validated_selected_asteroid_source_request_corpus_summary_for_report,
    validated_selected_asteroid_source_window_summary_for_report, JplSnapshotBackend,
};
use pleiades_vsop87::{
    body_source_profiles, canonical_epoch_equatorial_body_class_evidence_summary_for_report,
    canonical_epoch_equatorial_evidence_summary_for_report,
    canonical_epoch_evidence_summary_for_report, canonical_epoch_outlier_note_for_report,
    canonical_j1900_batch_parity_summary_for_report,
    canonical_j2000_batch_parity_summary_for_report,
    canonical_mixed_time_scale_batch_parity_summary_for_report, frame_treatment_summary_for_report,
    generated_binary_audit_summary_for_report, source_audit_summary_for_report, source_audits,
    source_body_class_evidence_summary_for_report, source_body_evidence_summary_for_report,
    source_documentation_health_summary, source_documentation_summary, source_specifications,
    supported_body_j1900_ecliptic_batch_parity_summary_for_report,
    supported_body_j1900_equatorial_batch_parity_summary_for_report,
    supported_body_j2000_ecliptic_batch_parity_summary_for_report,
    supported_body_j2000_equatorial_batch_parity_summary_for_report,
    vsop87_request_policy_summary_for_report, Vsop87Backend,
};

fn comparison_snapshot_batch_parity_summary_text() -> String {
    validated_comparison_snapshot_batch_parity_summary_for_report().unwrap_or_else(|error| {
        format!("JPL comparison snapshot batch parity: unavailable ({error})")
    })
}

fn reference_snapshot_batch_parity_summary_text() -> String {
    validated_reference_snapshot_batch_parity_summary_for_report().unwrap_or_else(|error| {
        format!("JPL reference snapshot batch parity: unavailable ({error})")
    })
}

fn reference_snapshot_mixed_time_scale_batch_parity_summary_text() -> String {
    validated_reference_snapshot_mixed_time_scale_batch_parity_summary_for_report().unwrap_or_else(
        |error| format!("JPL reference snapshot mixed TT/TDB batch parity: unavailable ({error})"),
    )
}

fn validated_production_generation_manifest_summary_text_for_report() -> String {
    validated_production_generation_manifest_summary_for_report()
        .unwrap_or_else(|error| format!("Production generation manifest: unavailable ({error})"))
}

fn independent_holdout_snapshot_batch_parity_summary_text() -> String {
    validated_independent_holdout_snapshot_batch_parity_summary_for_report().unwrap_or_else(
        |error| format!("JPL independent hold-out batch parity: unavailable ({error})"),
    )
}

const DEFAULT_BENCHMARK_ROUNDS: usize = 10_000;
const SUMMARY_BENCHMARK_ROUNDS: usize = 1;
const BANNER: &str = "pleiades-validate stage 4 tool";
const LUMINARY_LONGITUDE_THRESHOLD_DEG: f64 = 7.5;
const LUMINARY_LATITUDE_THRESHOLD_DEG: f64 = 0.75;
const LUMINARY_DISTANCE_THRESHOLD_AU: f64 = 0.001;
const MAJOR_PLANET_LONGITUDE_THRESHOLD_DEG: f64 = 0.01;
const MAJOR_PLANET_LATITUDE_THRESHOLD_DEG: f64 = 0.01;
const MAJOR_PLANET_DISTANCE_THRESHOLD_AU: f64 = 0.001;
const LUNAR_POINT_LONGITUDE_THRESHOLD_DEG: f64 = 0.1;
const LUNAR_POINT_LATITUDE_THRESHOLD_DEG: f64 = 0.01;
const LUNAR_POINT_DISTANCE_THRESHOLD_AU: f64 = 0.001;
const ASTEROID_LONGITUDE_THRESHOLD_DEG: f64 = 0.25;
const ASTEROID_LATITUDE_THRESHOLD_DEG: f64 = 0.05;
const ASTEROID_DISTANCE_THRESHOLD_AU: f64 = 0.01;
const CUSTOM_LONGITUDE_THRESHOLD_DEG: f64 = 0.25;
const CUSTOM_LATITUDE_THRESHOLD_DEG: f64 = 0.05;
const CUSTOM_DISTANCE_THRESHOLD_AU: f64 = 0.01;
const PLUTO_LONGITUDE_THRESHOLD_DEG: f64 = 45.0;
const PLUTO_LATITUDE_THRESHOLD_DEG: f64 = 1.0;
const PLUTO_DISTANCE_THRESHOLD_AU: f64 = 0.25;

#[cfg(test)]
mod tests;
